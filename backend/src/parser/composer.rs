use crate::models::*;
use crate::parser::error::{ParseError, ParseResult};
use std::collections::HashMap;

/// ProfileComposer is responsible for parsing Doris profile text
/// and composing it into a structured Profile object.
/// 
/// TODO: Implement actual parsing logic based on Doris profile format.
/// Currently returns stub/mock data for framework testing.
pub struct ProfileComposer {
    // Add parser state fields as needed
}

impl ProfileComposer {
    /// Create a new ProfileComposer instance
    pub fn new() -> Self {
        Self {}
    }
    
    /// Parse the profile text and return a structured Profile
    /// 
    /// # Arguments
    /// * `profile_text` - The raw profile text to parse
    /// 
    /// # Returns
    /// * `ParseResult<Profile>` - The parsed profile or an error
    pub fn parse(&mut self, profile_text: &str) -> ParseResult<Profile> {
        // TODO: Implement actual Doris profile parsing logic
        // For now, return stub data to verify the framework works
        
        if profile_text.trim().is_empty() {
            return Err(ParseError::InvalidFormat("Empty profile text".to_string()));
        }
        
        // Create stub profile data
        let summary = self.parse_summary_stub(profile_text);
        let fragments = self.parse_fragments_stub();
        let execution_tree = self.build_execution_tree_stub();
        
        Ok(Profile {
            summary,
            fragments,
            execution_tree: Some(execution_tree),
        })
    }
    
    /// Stub implementation for parsing summary
    fn parse_summary_stub(&self, profile_text: &str) -> ProfileSummary {
        // Extract some basic info if possible, otherwise use placeholders
        let query_id = self.extract_field(profile_text, "Query ID")
            .unwrap_or_else(|| "stub-query-id-12345".to_string());
        
        let sql_statement = self.extract_sql(profile_text)
            .unwrap_or_else(|| "SELECT * FROM table".to_string());
        
        ProfileSummary {
            query_id,
            start_time: "2024-01-01 00:00:00".to_string(),
            end_time: "2024-01-01 00:00:01".to_string(),
            total_time: "1s".to_string(),
            query_state: "FINISHED".to_string(),
            doris_version: "2.1.0".to_string(),
            sql_statement,
            query_type: Some("SELECT".to_string()),
            user: Some("root".to_string()),
            default_db: Some("default".to_string()),
            variables: HashMap::new(),
            total_time_ms: Some(1000.0),
            query_peak_memory: Some(1024 * 1024 * 100), // 100MB
        }
    }
    
    /// Stub implementation for parsing fragments
    fn parse_fragments_stub(&self) -> Vec<Fragment> {
        vec![
            Fragment {
                id: "Fragment 0".to_string(),
                backend_addresses: vec!["127.0.0.1:9060".to_string()],
                instance_ids: vec!["instance-0".to_string()],
                pipelines: vec![
                    Pipeline {
                        id: "Pipeline 0".to_string(),
                        metrics: HashMap::new(),
                        operators: vec![
                            Operator {
                                id: "0".to_string(),
                                name: "OlapScanNode".to_string(),
                                metrics: HashMap::new(),
                            },
                        ],
                    },
                ],
            },
        ]
    }
    
    /// Stub implementation for building execution tree
    fn build_execution_tree_stub(&self) -> ExecutionTree {
        let root_node = ExecutionTreeNode {
            id: "node-0".to_string(),
            operator_name: "RESULT_SINK".to_string(),
            node_type: NodeType::ResultSink,
            plan_node_id: Some(0),
            parent_plan_node_id: None,
            metrics: OperatorMetrics {
                operator_total_time: Some(100_000_000), // 100ms in ns
                operator_total_time_raw: Some("100ms".to_string()),
                rows_returned: Some(1000),
                memory_used: Some(1024 * 1024),
                cpu_time: Some(50_000_000),
                wait_time: Some(10_000_000),
            },
            children: vec!["node-1".to_string()],
            depth: 0,
            is_hotspot: false,
            hotspot_severity: HotspotSeverity::None,
            fragment_id: Some("Fragment 0".to_string()),
            pipeline_id: Some("Pipeline 0".to_string()),
            time_percentage: Some(10.0),
            is_most_consuming: false,
            is_second_most_consuming: false,
            unique_metrics: HashMap::new(),
        };
        
        let scan_node = ExecutionTreeNode {
            id: "node-1".to_string(),
            operator_name: "OLAP_SCAN".to_string(),
            node_type: NodeType::OlapScan,
            plan_node_id: Some(1),
            parent_plan_node_id: Some(0),
            metrics: OperatorMetrics {
                operator_total_time: Some(900_000_000), // 900ms in ns
                operator_total_time_raw: Some("900ms".to_string()),
                rows_returned: Some(100000),
                memory_used: Some(1024 * 1024 * 50),
                cpu_time: Some(800_000_000),
                wait_time: Some(50_000_000),
            },
            children: vec![],
            depth: 1,
            is_hotspot: true,
            hotspot_severity: HotspotSeverity::High,
            fragment_id: Some("Fragment 0".to_string()),
            pipeline_id: Some("Pipeline 0".to_string()),
            time_percentage: Some(90.0),
            is_most_consuming: true,
            is_second_most_consuming: false,
            unique_metrics: HashMap::new(),
        };
        
        ExecutionTree {
            root: root_node.clone(),
            nodes: vec![root_node, scan_node],
        }
    }
    
    /// Try to extract a field value from profile text
    fn extract_field(&self, text: &str, field_name: &str) -> Option<String> {
        for line in text.lines() {
            if line.contains(field_name) {
                if let Some(pos) = line.find(':') {
                    return Some(line[pos + 1..].trim().to_string());
                }
                if let Some(pos) = line.find('=') {
                    return Some(line[pos + 1..].trim().to_string());
                }
            }
        }
        None
    }
    
    /// Try to extract SQL statement from profile text
    fn extract_sql(&self, text: &str) -> Option<String> {
        // Look for common SQL patterns
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.to_uppercase().starts_with("SELECT") 
                || trimmed.to_uppercase().starts_with("INSERT")
                || trimmed.to_uppercase().starts_with("UPDATE")
                || trimmed.to_uppercase().starts_with("DELETE") {
                return Some(trimmed.to_string());
            }
        }
        None
    }
}

impl Default for ProfileComposer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_empty_profile() {
        let mut composer = ProfileComposer::new();
        let result = composer.parse("");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_parse_stub_profile() {
        let mut composer = ProfileComposer::new();
        let result = composer.parse("Query ID: test-123\nSELECT * FROM users");
        assert!(result.is_ok());
        
        let profile = result.unwrap();
        assert!(!profile.summary.query_id.is_empty());
        assert!(profile.execution_tree.is_some());
    }
}

