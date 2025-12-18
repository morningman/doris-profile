//! Tree builder for constructing ExecutionTree from parsed operators
//! Builds the DAG structure by connecting operators within pipelines and across fragments

use crate::models::*;
use crate::parser::engine::ValueParser;
use crate::parser::engine::operator_parser::ParsedOperator;
use crate::parser::engine::OperatorParser;
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
                let parsed_operators = OperatorParser::extract_parsed_operators(&pipeline.raw_text);
                
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
        let input_rows = OperatorParser::get_input_rows(parsed);
        let memory = parsed.common_counters.iter()
            .find(|item| item.key == "MemoryUsagePeak")
            .and_then(|item| {
                let agg = ValueParser::parse_aggregated(&item.value);
                agg.sum.or(agg.avg).map(|v| v as u64)
            });
        
        let exec_time_raw = parsed.common_counters.iter()
            .find(|item| item.key == "ExecTime")
            .and_then(|item| ValueParser::extract_first_value(&item.value));
        
        let metrics = OperatorMetrics {
            operator_total_time: exec_time.map(|t| t as u64),
            operator_total_time_raw: exec_time_raw,
            rows_returned: rows.map(|r| r as u64),
            input_rows: input_rows.map(|r| r as u64),
            memory_used: memory,
            cpu_time: None,
            wait_time: None,
        };
        
        // Build unique metrics from plan_info and custom_counters
        let mut unique_metrics = HashMap::new();
        for item in &parsed.plan_info {
            unique_metrics.insert(item.key.clone(), item.value.clone());
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
            table_name: parsed.table_name.clone(),
            time_percentage: None,
            is_most_consuming: false,
            is_second_most_consuming: false,
            unique_metrics,
            plan_info: parsed.plan_info.clone(),
            common_counters: parsed.common_counters.clone(),
            custom_counters: parsed.custom_counters.clone(),
        }
    }
    
    /// Connect nodes based on pipeline structure and operator relationships
    fn connect_nodes(nodes: &mut [ExecutionTreeNode], node_map: &HashMap<String, usize>) {
        // Build various lookup maps
        let mut nodes_by_fragment_pipeline: HashMap<(String, String), Vec<usize>> = HashMap::new();
        let mut exchange_nodes: HashMap<i32, usize> = HashMap::new(); // plan_node_id -> node index
        let mut sink_nodes_by_dest: HashMap<i32, usize> = HashMap::new(); // dest_id -> node index
        let mut operators_by_nereids: HashMap<(String, i32), Vec<usize>> = HashMap::new(); // (fragment, nereids_id) -> nodes
        let mut local_exchange_nodes: HashMap<(String, i32), usize> = HashMap::new(); // (fragment, plan_node_id) -> node index
        let mut local_exchange_sink_nodes: HashMap<(String, i32), usize> = HashMap::new(); // (fragment, plan_node_id) -> node index
        
        for (idx, node) in nodes.iter().enumerate() {
            if let (Some(fid), Some(pid)) = (&node.fragment_id, &node.pipeline_id) {
                nodes_by_fragment_pipeline
                    .entry((fid.clone(), pid.clone()))
                    .or_default()
                    .push(idx);
                
                // Track EXCHANGE operators by their plan_node_id (non-LOCAL)
                if node.operator_name.contains("EXCHANGE_OPERATOR") 
                    && !node.operator_name.contains("SINK") 
                    && !node.operator_name.contains("LOCAL") {
                    if let Some(plan_id) = node.plan_node_id {
                        exchange_nodes.insert(plan_id, idx);
                    }
                }
                
                // Track LOCAL_EXCHANGE operators by (fragment, plan_node_id)
                if node.operator_name.contains("LOCAL_EXCHANGE_OPERATOR") && !node.operator_name.contains("SINK") {
                    if let Some(plan_id) = node.plan_node_id {
                        local_exchange_nodes.insert((fid.clone(), plan_id), idx);
                    }
                }
                
                // Track LOCAL_EXCHANGE_SINK operators by (fragment, plan_node_id)
                if node.operator_name.contains("LOCAL_EXCHANGE_SINK") {
                    if let Some(plan_id) = node.plan_node_id {
                        local_exchange_sink_nodes.insert((fid.clone(), plan_id), idx);
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
        
        // 2. Connect non-SINK operators to their corresponding SINK operators
        // Example: SORT_OPERATOR -> SORT_SINK_OPERATOR (in different pipelines)
        // Use name-based matching: OPERATOR_NAME -> OPERATOR_NAME_SINK
        // First, collect all connections to be made
        let mut sink_connections: Vec<(usize, String)> = Vec::new();
        
        for (idx, node) in nodes.iter().enumerate() {
            if !node.operator_name.contains("SINK") {
                // Try to find corresponding SINK operator in the same fragment
                let expected_sink_name = if node.operator_name.ends_with("_OPERATOR") {
                    node.operator_name.replace("_OPERATOR", "_SINK_OPERATOR")
                } else {
                    format!("{}_SINK", node.operator_name)
                };
                
                if let Some(fid) = &node.fragment_id {
                    // Find SINK with matching name AND plan_node_id in a different pipeline
                    for (other_idx, other_node) in nodes.iter().enumerate() {
                        if other_idx != idx 
                            && other_node.fragment_id.as_ref() == Some(fid)
                            && other_node.pipeline_id != node.pipeline_id
                            && other_node.operator_name == expected_sink_name
                            && other_node.plan_node_id == node.plan_node_id {  // 关键修复：检查 plan_node_id 匹配
                            // Record connection: non-SINK -> SINK
                            sink_connections.push((idx, other_node.id.clone()));
                            break; // Only connect to one SINK
                        }
                    }
                }
                
                // Also try nereids_id matching for cases where name doesn't match exactly
                if let Some(nereids_str) = node.unique_metrics.get("nereids_id") {
                    if let Ok(nereids_id) = nereids_str.parse::<i32>() {
                        if let Some(fid) = &node.fragment_id {
                            if let Some(matching_nodes) = operators_by_nereids.get(&(fid.clone(), nereids_id)) {
                                for &match_idx in matching_nodes {
                                    if match_idx != idx 
                                        && nodes[match_idx].operator_name.contains("SINK")
                                        && !nodes[match_idx].operator_name.contains("DATA_STREAM_SINK")
                                        && !nodes[match_idx].operator_name.contains("RESULT_SINK")
                                        && nodes[match_idx].pipeline_id != node.pipeline_id {
                                        // Record connection
                                        sink_connections.push((idx, nodes[match_idx].id.clone()));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Apply sink connections
        for (parent_idx, child_id) in sink_connections {
            if !nodes[parent_idx].children.contains(&child_id) {
                nodes[parent_idx].children.push(child_id);
            }
        }
        
        // 3. Connect EXCHANGE operators to DATA_STREAM_SINK operators (cross-fragment)
        // DATA_STREAM_SINK(dest_id=X) sends data to EXCHANGE_OPERATOR(id=X)
        // In the tree structure: EXCHANGE receives from DATA_STREAM_SINK
        // So EXCHANGE (parent/consumer) has DATA_STREAM_SINK as child (producer)
        for (dest_id, sink_idx) in &sink_nodes_by_dest {
            if let Some(&exchange_idx) = exchange_nodes.get(dest_id) {
                // EXCHANGE receives from SINK, so SINK is a child of EXCHANGE
                let sink_id = nodes[*sink_idx].id.clone();
                if !nodes[exchange_idx].children.contains(&sink_id) {
                    nodes[exchange_idx].children.push(sink_id);
                }
            }
        }
        
        // 4. Connect LOCAL_EXCHANGE_OPERATOR to LOCAL_EXCHANGE_SINK_OPERATOR
        // LOCAL_EXCHANGE and LOCAL_EXCHANGE_SINK with the same id should be connected
        // LOCAL_EXCHANGE (parent/consumer) -> LOCAL_EXCHANGE_SINK (child/producer)
        for ((frag, exchange_id), exchange_idx) in &local_exchange_nodes {
            if let Some(&sink_idx) = local_exchange_sink_nodes.get(&(frag.clone(), *exchange_id)) {
                // LOCAL_EXCHANGE receives from LOCAL_EXCHANGE_SINK
                let sink_id = nodes[sink_idx].id.clone();
                if !nodes[*exchange_idx].children.contains(&sink_id) {
                    nodes[*exchange_idx].children.push(sink_id);
                }
            }
        }
        
        // Update depths based on the tree structure
        Self::update_depths(nodes, node_map);
        
        // DEBUG: Check for nodes with multiple parents
        Self::check_multiple_parents(nodes);
    }
    
    /// Check and report nodes with multiple parents
    fn check_multiple_parents(nodes: &[ExecutionTreeNode]) {
        use std::collections::HashMap;
        
        // Count parent references for each node
        let mut parent_count: HashMap<&String, Vec<&str>> = HashMap::new();
        
        for node in nodes {
            for child_id in &node.children {
                parent_count.entry(child_id).or_default().push(&node.operator_name);
            }
        }
        
        // Report nodes with multiple parents
        let mut has_multiple_parents = false;
        for node in nodes {
            if let Some(parents) = parent_count.get(&node.id) {
                if parents.len() > 1 {
                    has_multiple_parents = true;
                    eprintln!("⚠️  节点有多个父节点: {} (id={}, F={:?}, P={:?}, plan_node_id={:?})",
                        node.operator_name,
                        node.id,
                        node.fragment_id,
                        node.pipeline_id,
                        node.plan_node_id);
                    eprintln!("   父节点: {:?}", parents);
                }
            }
        }
        
        if !has_multiple_parents {
            eprintln!("✅ 所有节点都最多只有一个父节点");
        }
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
            table_name: None,
            time_percentage: None,
            is_most_consuming: false,
            is_second_most_consuming: false,
            unique_metrics: HashMap::new(),
            plan_info: Vec::new(),
            common_counters: Vec::new(),
            custom_counters: Vec::new(),
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
            // Reconstruct operator header based on type
            if op.name.contains("DATA_STREAM_SINK") {
                // DATA_STREAM_SINK_OPERATOR(dest_id=X):
                if let Some(dest_id) = op.metrics.get("dest_id") {
                    text.push_str(&format!("   {}(dest_id={}):\n", op.name, dest_id));
                } else {
                    text.push_str(&format!("   {}:\n", op.name));
                }
            } else if op.name.contains("LOCAL_EXCHANGE") {
                // LOCAL_EXCHANGE_OPERATOR(type)(id=X):
                if let Some(exchange_type) = op.metrics.get("exchange_type") {
                    text.push_str(&format!("   {}({})(id={}):\n", op.name, exchange_type, op.id));
                } else {
                    text.push_str(&format!("   {}(id={}):\n", op.name, op.id));
                }
            } else {
                // Standard format: OPERATOR(nereids_id=X)(id=Y):
                // Or alternate format: OPERATOR (id=X. nereids_id=Y. ...)
                if let Some(nereids_id) = op.metrics.get("nereids_id") {
                    // Check if there's table_name
                    if let Some(table_name) = op.metrics.get("table_name") {
                        text.push_str(&format!("   {}(nereids_id={})(id={}, table name = {}):\n", 
                            op.name, nereids_id, op.id, table_name));
                    } else {
                        text.push_str(&format!("   {}(nereids_id={})(id={}):\n", 
                            op.name, nereids_id, op.id));
                    }
                } else {
                    text.push_str(&format!("   {}(id={}):\n", op.name, op.id));
                }
            }
            
            text.push_str("     CommonCounters:\n");
            for (k, v) in &op.metrics {
                // Skip metadata fields that were already in the header
                if k == "dest_id" || k == "nereids_id" || k == "exchange_type" || k == "table_name" {
                    continue;
                }
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
