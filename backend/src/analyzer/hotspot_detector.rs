use crate::models::*;
use crate::constants::thresholds;

/// HotSpotDetector analyzes execution tree nodes to identify performance bottlenecks
pub struct HotSpotDetector;

impl HotSpotDetector {
    /// Analyze a profile and return a list of detected hotspots
    pub fn analyze(profile: &Profile) -> Vec<HotSpot> {
        let mut hotspots = Vec::new();
        
        if let Some(ref tree) = profile.execution_tree {
            for node in &tree.nodes {
                if let Some(hotspot) = Self::analyze_node(node) {
                    hotspots.push(hotspot);
                }
            }
        }
        
        // Sort hotspots by severity (most severe first)
        hotspots.sort_by(|a, b| {
            Self::severity_rank(&b.severity).cmp(&Self::severity_rank(&a.severity))
        });
        
        hotspots
    }
    
    /// Analyze a single node for potential hotspots
    fn analyze_node(node: &ExecutionTreeNode) -> Option<HotSpot> {
        // Check if node is marked as a hotspot
        if !node.is_hotspot && node.hotspot_severity == HotspotSeverity::None {
            // Still check time percentage
            if let Some(pct) = node.time_percentage {
                if pct < thresholds::LOW_TIME_PERCENTAGE {
                    return None;
                }
            } else {
                return None;
            }
        }
        
        let severity = Self::determine_severity(node);
        if severity == HotspotSeverity::None {
            return None;
        }
        
        let (description, impact, suggestion) = Self::generate_analysis(node, &severity);
        
        Some(HotSpot {
            node_id: node.id.clone(),
            node_path: Self::build_node_path(node),
            operator_name: node.operator_name.clone(),
            severity,
            description,
            impact,
            time_percentage: node.time_percentage,
            suggestion: Some(suggestion),
        })
    }
    
    /// Determine hotspot severity based on node metrics
    fn determine_severity(node: &ExecutionTreeNode) -> HotspotSeverity {
        // Use node's own severity if set
        if node.hotspot_severity != HotspotSeverity::None {
            return node.hotspot_severity;
        }
        
        // Determine by time percentage
        if let Some(pct) = node.time_percentage {
            if pct >= thresholds::CRITICAL_TIME_PERCENTAGE {
                return HotspotSeverity::Critical;
            } else if pct >= thresholds::HIGH_TIME_PERCENTAGE {
                return HotspotSeverity::High;
            } else if pct >= thresholds::MEDIUM_TIME_PERCENTAGE {
                return HotspotSeverity::Medium;
            } else if pct >= thresholds::LOW_TIME_PERCENTAGE {
                return HotspotSeverity::Low;
            }
        }
        
        HotspotSeverity::None
    }
    
    /// Generate analysis description, impact, and suggestion for a hotspot
    fn generate_analysis(node: &ExecutionTreeNode, severity: &HotspotSeverity) -> (String, String, String) {
        let pct_str = node.time_percentage
            .map(|p| format!("{:.1}%", p))
            .unwrap_or_else(|| "N/A".to_string());
        
        match node.node_type {
            NodeType::OlapScan => {
                let description = format!(
                    "{} operator consuming {} of total execution time",
                    node.operator_name, pct_str
                );
                let impact = match severity {
                    HotspotSeverity::Critical => "Critical impact on query performance. Scan operation is the primary bottleneck.".to_string(),
                    HotspotSeverity::High => "High impact on query performance. Consider optimizing scan filters.".to_string(),
                    _ => "Moderate impact on query performance.".to_string(),
                };
                let suggestion = "Consider adding indexes, optimizing predicates, or reducing data scan range with partition pruning.".to_string();
                (description, impact, suggestion)
            }
            NodeType::HashJoin => {
                let description = format!(
                    "{} operator consuming {} of total execution time",
                    node.operator_name, pct_str
                );
                let impact = "Join operation may be processing large datasets or using suboptimal join strategy.".to_string();
                let suggestion = "Consider reordering joins, adding indexes on join columns, or using broadcast join for small tables.".to_string();
                (description, impact, suggestion)
            }
            NodeType::Aggregate => {
                let description = format!(
                    "{} operator consuming {} of total execution time",
                    node.operator_name, pct_str
                );
                let impact = "Aggregation operation may be processing many distinct values or large datasets.".to_string();
                let suggestion = "Consider pre-aggregating data, using approximate aggregation, or reducing group by cardinality.".to_string();
                (description, impact, suggestion)
            }
            NodeType::Sort => {
                let description = format!(
                    "{} operator consuming {} of total execution time",
                    node.operator_name, pct_str
                );
                let impact = "Sort operation may be processing large datasets.".to_string();
                let suggestion = "Consider adding index on sort columns, limiting result set, or using TOP-N optimization.".to_string();
                (description, impact, suggestion)
            }
            NodeType::Exchange => {
                let description = format!(
                    "{} operator consuming {} of total execution time",
                    node.operator_name, pct_str
                );
                let impact = "Data shuffle between nodes may be causing network bottleneck.".to_string();
                let suggestion = "Consider colocate join, reducing shuffle data, or optimizing partition strategy.".to_string();
                (description, impact, suggestion)
            }
            _ => {
                let description = format!(
                    "{} operator consuming {} of total execution time",
                    node.operator_name, pct_str
                );
                let impact = "This operator is consuming significant execution time.".to_string();
                let suggestion = "Review the operator's input data and execution strategy.".to_string();
                (description, impact, suggestion)
            }
        }
    }
    
    /// Build a human-readable path for the node
    fn build_node_path(node: &ExecutionTreeNode) -> String {
        let mut path = String::new();
        
        if let Some(ref frag_id) = node.fragment_id {
            path.push_str(frag_id);
            path.push_str(" > ");
        }
        
        if let Some(ref pipe_id) = node.pipeline_id {
            path.push_str(pipe_id);
            path.push_str(" > ");
        }
        
        path.push_str(&node.operator_name);
        
        if let Some(plan_id) = node.plan_node_id {
            path.push_str(&format!(" (Plan Node {})", plan_id));
        }
        
        path
    }
    
    /// Get numeric rank for severity (higher = more severe)
    fn severity_rank(severity: &HotspotSeverity) -> u8 {
        match severity {
            HotspotSeverity::Critical => 4,
            HotspotSeverity::High => 3,
            HotspotSeverity::Medium => 2,
            HotspotSeverity::Low => 1,
            HotspotSeverity::None => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_severity_determination() {
        let node = ExecutionTreeNode {
            id: "test".to_string(),
            operator_name: "OLAP_SCAN".to_string(),
            node_type: NodeType::OlapScan,
            plan_node_id: None,
            parent_plan_node_id: None,
            metrics: OperatorMetrics::default(),
            children: vec![],
            depth: 0,
            is_hotspot: false,
            hotspot_severity: HotspotSeverity::None,
            fragment_id: None,
            pipeline_id: None,
            time_percentage: Some(60.0),
            is_most_consuming: false,
            is_second_most_consuming: false,
            unique_metrics: HashMap::new(),
        };
        
        let severity = HotSpotDetector::determine_severity(&node);
        assert_eq!(severity, HotspotSeverity::Critical);
    }
}

