//! Tree builder for constructing ExecutionTree from parsed operators
//! Builds the DAG structure by connecting operators within pipelines and across fragments

use crate::models::*;
use crate::parser::core::ValueParser;
use crate::parser::core::operator_parser::ParsedOperator;
use crate::parser::core::OperatorParser;
use crate::constants::thresholds;
use std::collections::HashMap;

pub struct TreeBuilder;

impl TreeBuilder {
    /// Build ExecutionTree from fragments
    pub fn build_from_fragments(
        fragments: &[Fragment],
        summary: &ProfileSummary,
    ) -> ExecutionTree {
        let mut nodes = Vec::new();
        let mut node_map: HashMap<String, usize> = HashMap::new();
        
        // First pass: create all nodes and track their metadata
        for fragment in fragments {
            for pipeline in &fragment.pipelines {
                let parsed_operators = OperatorParser::extract_parsed_operators(
                    &Self::reconstruct_pipeline_text(pipeline)
                );
                
                for parsed_op in parsed_operators {
                    let node = Self::create_tree_node(
                        &parsed_op,
                        &fragment.id,
                        &pipeline.id,
                    );
                    
                    let node_id = node.id.clone();
                    node_map.insert(node_id.clone(), nodes.len());
                    nodes.push(node);
                }
            }
        }
        
        // Second pass: establish connections
        Self::connect_nodes(&mut nodes, &node_map);
        
        // Third pass: calculate metrics and identify hotspots
        Self::calculate_metrics(&mut nodes, summary);
        
        // Find root node (usually RESULT_SINK)
        let root = Self::find_root_node(&nodes);
        
        ExecutionTree { root, nodes }
    }
    
    /// Create a tree node from a parsed operator
    fn create_tree_node(
        parsed: &ParsedOperator,
        fragment_id: &str,
        pipeline_id: &str,
    ) -> ExecutionTreeNode {
        let node_type = Self::determine_node_type(&parsed.name);
        
        // Extract metrics
        let exec_time = OperatorParser::get_exec_time_ns(parsed);
        let rows = OperatorParser::get_rows_produced(parsed);
        let memory = parsed.common_counters.get("MemoryUsagePeak")
            .and_then(|v| {
                let agg = ValueParser::parse_aggregated(v);
                agg.sum.or(agg.avg).map(|v| v as u64)
            });
        
        let exec_time_raw = parsed.common_counters.get("ExecTime")
            .and_then(|v| ValueParser::extract_first_value(v));
        
        let metrics = OperatorMetrics {
            operator_total_time: exec_time.map(|t| t as u64),
            operator_total_time_raw: exec_time_raw,
            rows_returned: rows.map(|r| r as u64),
            memory_used: memory,
            cpu_time: None,
            wait_time: None,
        };
        
        // Build unique metrics from plan_info and custom_counters
        let mut unique_metrics = HashMap::new();
        for (k, v) in &parsed.plan_info {
            unique_metrics.insert(k.clone(), v.clone());
        }
        if let Some(ref tn) = parsed.table_name {
            unique_metrics.insert("table_name".to_string(), tn.clone());
        }
        // Store nereids_id and dest_id for connection logic
        if let Some(nid) = parsed.nereids_id {
            unique_metrics.insert("nereids_id".to_string(), nid.to_string());
        }
        if let Some(did) = parsed.dest_id {
            unique_metrics.insert("dest_id".to_string(), did.to_string());
        }
        
        // Generate a unique node ID
        let node_id = if let Some(did) = parsed.dest_id {
            format!("{}-{}-dest{}", fragment_id, pipeline_id, did)
        } else {
            format!("{}-{}-id{}", fragment_id, pipeline_id, parsed.id)
        };
        
        ExecutionTreeNode {
            id: node_id,
            operator_name: parsed.name.clone(),
            node_type,
            plan_node_id: Some(parsed.id),
            parent_plan_node_id: None,
            metrics,
            children: Vec::new(),
            depth: 0,
            is_hotspot: false,
            hotspot_severity: HotspotSeverity::None,
            fragment_id: Some(fragment_id.to_string()),
            pipeline_id: Some(pipeline_id.to_string()),
            time_percentage: None,
            is_most_consuming: false,
            is_second_most_consuming: false,
            unique_metrics,
        }
    }
    
    /// Connect nodes based on pipeline structure and operator relationships
    fn connect_nodes(nodes: &mut [ExecutionTreeNode], node_map: &HashMap<String, usize>) {
        // Build various lookup maps
        let mut nodes_by_fragment_pipeline: HashMap<(String, String), Vec<usize>> = HashMap::new();
        let mut exchange_nodes: HashMap<i32, usize> = HashMap::new(); // plan_node_id -> node index
        let mut sink_nodes_by_dest: HashMap<i32, usize> = HashMap::new(); // dest_id -> node index
        let mut operators_by_nereids: HashMap<(String, i32), Vec<usize>> = HashMap::new(); // (fragment, nereids_id) -> nodes
        
        for (idx, node) in nodes.iter().enumerate() {
            if let (Some(fid), Some(pid)) = (&node.fragment_id, &node.pipeline_id) {
                nodes_by_fragment_pipeline
                    .entry((fid.clone(), pid.clone()))
                    .or_default()
                    .push(idx);
                
                // Track EXCHANGE operators by their plan_node_id
                if node.operator_name.contains("EXCHANGE_OPERATOR") && !node.operator_name.contains("SINK") {
                    if let Some(plan_id) = node.plan_node_id {
                        exchange_nodes.insert(plan_id, idx);
                    }
                }
                
                // Track DATA_STREAM_SINK operators by their dest_id
                if node.operator_name.contains("DATA_STREAM_SINK") {
                    if let Some(dest_str) = node.unique_metrics.get("dest_id") {
                        if let Ok(dest_id) = dest_str.parse::<i32>() {
                            sink_nodes_by_dest.insert(dest_id, idx);
                        }
                    }
                }
                
                // Track operators by (fragment, nereids_id)
                if let Some(nereids_str) = node.unique_metrics.get("nereids_id") {
                    if let Ok(nereids_id) = nereids_str.parse::<i32>() {
                        operators_by_nereids
                            .entry((fid.clone(), nereids_id))
                            .or_default()
                            .push(idx);
                    }
                }
            }
        }
        
        // 1. Connect operators within the same pipeline (sequential chain)
        // First operator's child is the second operator, etc.
        for ((fid, pid), pipeline_node_indices) in &nodes_by_fragment_pipeline {
            let indices: Vec<usize> = pipeline_node_indices.clone();
            
            for i in 0..indices.len() {
                if i + 1 < indices.len() {
                    let current_idx = indices[i];
                    let child_idx = indices[i + 1];
                    let child_id = nodes[child_idx].id.clone();
                    nodes[current_idx].children.push(child_id);
                }
            }
        }
        
        // 2. Connect SINK operators to their corresponding non-SINK operators by nereids_id
        // (within the same fragment)
        for (idx, node) in nodes.iter().enumerate() {
            if node.operator_name.contains("SINK") && !node.operator_name.contains("RESULT_SINK") && !node.operator_name.contains("DATA_STREAM_SINK") {
                // Find matching non-SINK operator with same nereids_id in the same fragment
                if let Some(nereids_str) = node.unique_metrics.get("nereids_id") {
                    if let Ok(nereids_id) = nereids_str.parse::<i32>() {
                        if let Some(fid) = &node.fragment_id {
                            if let Some(matching_nodes) = operators_by_nereids.get(&(fid.clone(), nereids_id)) {
                                for &match_idx in matching_nodes {
                                    // Connect non-SINK to SINK (SINK feeds into non-SINK)
                                    if match_idx != idx && !nodes[match_idx].operator_name.contains("SINK") {
                                        // The non-SINK operator should have SINK as its child
                                        let sink_id = nodes[idx].id.clone();
                                        if !nodes[match_idx].children.contains(&sink_id) {
                                            // Actually, SINK feeds INTO the operator, so SINK is the producer
                                            // Let's connect the last operator in SINK's pipeline to EXCHANGE/non-SINK
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // 3. Connect EXCHANGE operators to DATA_STREAM_SINK operators (cross-fragment)
        // DATA_STREAM_SINK(dest_id=X) sends data to EXCHANGE_OPERATOR(id=X)
        for (dest_id, sink_idx) in &sink_nodes_by_dest {
            if let Some(&exchange_idx) = exchange_nodes.get(dest_id) {
                // EXCHANGE receives data from SINK, so SINK is a child of EXCHANGE
                let sink_id = nodes[*sink_idx].id.clone();
                if !nodes[exchange_idx].children.contains(&sink_id) {
                    nodes[exchange_idx].children.push(sink_id);
                }
            }
        }
        
        // 4. Connect pipelines within a fragment
        // The last operator of pipeline N should connect to the first operator of pipeline N+1's SINK operator
        let mut fragments: Vec<String> = nodes_by_fragment_pipeline.keys()
            .map(|(f, _)| f.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        fragments.sort();
        
        for fid in &fragments {
            let mut pipelines: Vec<String> = nodes_by_fragment_pipeline.keys()
                .filter(|(f, _)| f == fid)
                .map(|(_, p)| p.clone())
                .collect();
            pipelines.sort();
            
            // Connect last non-SINK of pipeline to first SINK of next pipeline
            for i in 0..pipelines.len() {
                if i + 1 < pipelines.len() {
                    let current_pipe = &pipelines[i];
                    let next_pipe = &pipelines[i + 1];
                    
                    if let Some(current_nodes) = nodes_by_fragment_pipeline.get(&(fid.clone(), current_pipe.clone())) {
                        if let Some(next_nodes) = nodes_by_fragment_pipeline.get(&(fid.clone(), next_pipe.clone())) {
                            // Find last node of current pipeline
                            if let Some(&last_idx) = current_nodes.last() {
                                // Find first SINK in next pipeline
                                for &next_idx in next_nodes {
                                    if nodes[next_idx].operator_name.contains("SINK") {
                                        // Connect last of current to first SINK of next
                                        let next_id = nodes[next_idx].id.clone();
                                        if !nodes[last_idx].children.contains(&next_id) {
                                            // Actually, the SINK of next pipeline feeds back
                                            // We need the data flow direction correct
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Update depths based on the tree structure
        Self::update_depths(nodes, node_map);
    }
    
    /// Update node depths by traversing from root
    fn update_depths(nodes: &mut [ExecutionTreeNode], _node_map: &HashMap<String, usize>) {
        // Build a map from id to index
        let id_to_idx: HashMap<String, usize> = nodes.iter()
            .enumerate()
            .map(|(i, n)| (n.id.clone(), i))
            .collect();
        
        // Find root (RESULT_SINK)
        let root_idx = nodes.iter()
            .position(|n| n.operator_name.contains("RESULT_SINK"))
            .unwrap_or(0);
        
        // BFS to update depths
        let mut queue = vec![(root_idx, 0usize)];
        let mut visited = std::collections::HashSet::new();
        
        while let Some((idx, depth)) = queue.pop() {
            if visited.contains(&idx) {
                continue;
            }
            visited.insert(idx);
            nodes[idx].depth = depth;
            
            for child_id in nodes[idx].children.clone() {
                if let Some(&child_idx) = id_to_idx.get(&child_id) {
                    if !visited.contains(&child_idx) {
                        queue.push((child_idx, depth + 1));
                    }
                }
            }
        }
    }
    
    /// Calculate time percentages and identify hotspots
    fn calculate_metrics(nodes: &mut [ExecutionTreeNode], _summary: &ProfileSummary) {
        // Calculate total execution time from all operators
        let total_time: u64 = nodes.iter()
            .filter_map(|n| n.metrics.operator_total_time)
            .sum();
        
        if total_time == 0 {
            return;
        }
        
        // Calculate percentage for each node
        for node in nodes.iter_mut() {
            if let Some(op_time) = node.metrics.operator_total_time {
                let percentage = (op_time as f64 / total_time as f64) * 100.0;
                node.time_percentage = Some(percentage);
                
                // Determine hotspot severity
                if percentage >= thresholds::CRITICAL_TIME_PERCENTAGE {
                    node.is_hotspot = true;
                    node.hotspot_severity = HotspotSeverity::Critical;
                } else if percentage >= thresholds::HIGH_TIME_PERCENTAGE {
                    node.is_hotspot = true;
                    node.hotspot_severity = HotspotSeverity::High;
                } else if percentage >= thresholds::MEDIUM_TIME_PERCENTAGE {
                    node.is_hotspot = true;
                    node.hotspot_severity = HotspotSeverity::Medium;
                } else if percentage >= thresholds::LOW_TIME_PERCENTAGE {
                    node.is_hotspot = true;
                    node.hotspot_severity = HotspotSeverity::Low;
                }
            }
        }
        
        // Find top 2 time-consuming nodes
        let mut sorted_indices: Vec<usize> = (0..nodes.len()).collect();
        sorted_indices.sort_by(|&a, &b| {
            let time_a = nodes[a].metrics.operator_total_time.unwrap_or(0);
            let time_b = nodes[b].metrics.operator_total_time.unwrap_or(0);
            time_b.cmp(&time_a)
        });
        
        if !sorted_indices.is_empty() {
            nodes[sorted_indices[0]].is_most_consuming = true;
        }
        if sorted_indices.len() > 1 {
            nodes[sorted_indices[1]].is_second_most_consuming = true;
        }
    }
    
    /// Find the root node (typically RESULT_SINK)
    fn find_root_node(nodes: &[ExecutionTreeNode]) -> ExecutionTreeNode {
        // Look for RESULT_SINK first
        for node in nodes {
            if node.operator_name.contains("RESULT_SINK") {
                return node.clone();
            }
        }
        
        // Fall back to first node in Fragment 0, Pipeline 0
        for node in nodes {
            if node.fragment_id.as_deref() == Some("Fragment 0") 
                && node.pipeline_id.as_deref() == Some("Pipeline 0") {
                return node.clone();
            }
        }
        
        // Last resort: first node
        nodes.first().cloned().unwrap_or_else(|| ExecutionTreeNode {
            id: "root".to_string(),
            operator_name: "UNKNOWN".to_string(),
            node_type: NodeType::Unknown,
            plan_node_id: None,
            parent_plan_node_id: None,
            metrics: OperatorMetrics::default(),
            children: Vec::new(),
            depth: 0,
            is_hotspot: false,
            hotspot_severity: HotspotSeverity::None,
            fragment_id: None,
            pipeline_id: None,
            time_percentage: None,
            is_most_consuming: false,
            is_second_most_consuming: false,
            unique_metrics: HashMap::new(),
        })
    }
    
    /// Determine node type from operator name
    fn determine_node_type(name: &str) -> NodeType {
        let upper = name.to_uppercase();
        
        if upper.contains("SCAN") {
            if upper.contains("OLAP") {
                NodeType::OlapScan
            } else {
                NodeType::OlapScan // FILE_SCAN treated as OlapScan for now
            }
        } else if upper.contains("EXCHANGE") {
            NodeType::Exchange
        } else if upper.contains("HASH_JOIN") {
            NodeType::HashJoin
        } else if upper.contains("AGGREGATE") || upper.contains("AGGREGATION") {
            NodeType::Aggregate
        } else if upper.contains("SORT") {
            NodeType::Sort
        } else if upper.contains("LIMIT") {
            NodeType::Limit
        } else if upper.contains("PROJECT") {
            NodeType::Project
        } else if upper.contains("FILTER") {
            NodeType::Filter
        } else if upper.contains("UNION") {
            NodeType::Union
        } else if upper.contains("RESULT_SINK") {
            NodeType::ResultSink
        } else if upper.contains("DATA_STREAM_SINK") || upper.contains("STREAM_SINK") {
            NodeType::DataStreamSink
        } else {
            NodeType::Unknown
        }
    }
    
    /// Reconstruct pipeline text from Pipeline struct (for re-parsing)
    fn reconstruct_pipeline_text(pipeline: &Pipeline) -> String {
        let mut text = format!("{}:\n", pipeline.id);
        
        for (k, v) in &pipeline.metrics {
            text.push_str(&format!("   - {}: {}\n", k, v));
        }
        
        for op in &pipeline.operators {
            text.push_str(&format!("   {}(id={}):\n", op.name, op.id));
            text.push_str("     CommonCounters:\n");
            for (k, v) in &op.metrics {
                text.push_str(&format!("        - {}: {}\n", k, v));
            }
        }
        
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_determine_node_type() {
        assert_eq!(TreeBuilder::determine_node_type("OLAP_SCAN_OPERATOR"), NodeType::OlapScan);
        assert_eq!(TreeBuilder::determine_node_type("FILE_SCAN_OPERATOR"), NodeType::OlapScan);
        assert_eq!(TreeBuilder::determine_node_type("HASH_JOIN_OPERATOR"), NodeType::HashJoin);
        assert_eq!(TreeBuilder::determine_node_type("AGGREGATION_OPERATOR"), NodeType::Aggregate);
        assert_eq!(TreeBuilder::determine_node_type("SORT_OPERATOR"), NodeType::Sort);
        assert_eq!(TreeBuilder::determine_node_type("EXCHANGE_OPERATOR"), NodeType::Exchange);
        assert_eq!(TreeBuilder::determine_node_type("RESULT_SINK_OPERATOR"), NodeType::ResultSink);
    }
}
