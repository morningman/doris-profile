//! ProfileComposer - Main entry point for parsing Doris profiles
//! Orchestrates all sub-parsers to build a complete Profile

use crate::models::*;
use crate::parser::error::{ParseError, ParseResult};
use crate::parser::core::{SectionParser, FragmentParser, TreeBuilder};

/// ProfileComposer is responsible for parsing Doris profile text
/// and composing it into a structured Profile object.
pub struct ProfileComposer {
    // Parser state if needed
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
        if profile_text.trim().is_empty() {
            return Err(ParseError::InvalidFormat("Empty profile text".to_string()));
        }
        
        // Parse Summary section
        let mut summary = SectionParser::parse_summary(profile_text)?;
        
        // Parse ChangedSessionVariables (optional)
        if let Ok(variables) = SectionParser::parse_session_variables(profile_text) {
            summary.session_variables = variables;
        }
        
        // Extract MergedProfile section
        let merged_profile = SectionParser::extract_merged_profile(profile_text)?;
        
        // Parse Fragments from MergedProfile
        let fragments = FragmentParser::extract_all_fragments(&merged_profile);
        
        if fragments.is_empty() {
            return Err(ParseError::InvalidFormat("No fragments found in profile".to_string()));
        }
        
        // Build execution tree from fragments
        let execution_tree = TreeBuilder::build_from_fragments(&fragments, &summary);
        
        Ok(Profile {
            summary,
            fragments,
            execution_tree: Some(execution_tree),
        })
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
    fn test_parse_minimal_profile() {
        let profile_text = r#"Summary:
   - Profile ID: test-123
   - Task Type: QUERY
   - Start Time: 2025-01-01 00:00:00
   - End Time: 2025-01-01 00:00:01
   - Total: 1sec
   - Task State: OK
   - User: root
   - Default Db: test
   - Sql Statement: SELECT 1
Execution Summary:
   - Workload Group: normal
ChangedSessionVariables:
[]
MergedProfile:
     Fragments:
       Fragment 0:
         Pipeline 0(instance_num=1):
           - WaitWorkerTime: avg 18.605us, max 18.605us, min 18.605us
           RESULT_SINK_OPERATOR(id=2147483647):
             CommonCounters:
               - ExecTime: avg 95.241us, max 95.241us, min 95.241us
               - InputRows: sum 1, avg 1, max 1, min 1
             CustomCounters:
"#;
        
        let mut composer = ProfileComposer::new();
        let result = composer.parse(profile_text);
        
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        
        let profile = result.unwrap();
        assert_eq!(profile.summary.query_id, "test-123");
        assert_eq!(profile.summary.query_state, "OK");
        assert!(!profile.fragments.is_empty());
        assert!(profile.execution_tree.is_some());
        
        let tree = profile.execution_tree.unwrap();
        assert!(!tree.nodes.is_empty());
        assert!(tree.root.operator_name.contains("RESULT_SINK"));
    }
    
    #[test]
    fn test_parse_complex_profile() {
        let profile_text = r#"Summary:
   - Profile ID: 37f4f7ab99a741ed-8fd24882055ce279
   - Task Type: QUERY
   - Start Time: 2025-12-15 19:55:24
   - End Time: 2025-12-15 19:55:25
   - Total: 1sec240ms
   - Task State: OK
   - User: root
   - Default Catalog: iceberg
   - Default Db: tpcds1000_parquet
   - Sql Statement: SELECT * FROM test
Execution Summary:
   - Workload Group: normal
   - Parse SQL Time: 2ms
ChangedSessionVariables:
[
  {
    "VarName": "enable_profile",
    "CurrentValue": "true",
    "DefaultValue": "false"
  }
]
MergedProfile:
     Fragments:
       Fragment 0:
         Pipeline 0(instance_num=1):
           - WaitWorkerTime: avg 18.605us, max 18.605us, min 18.605us
           RESULT_SINK_OPERATOR(id=2147483647):
             CommonCounters:
               - ExecTime: avg 95.241us, max 95.241us, min 95.241us
           SORT_OPERATOR(nereids_id=1966)(id=28):
             CommonCounters:
               - ExecTime: avg 14.905us, max 14.905us, min 14.905us
               - RowsProduced: sum 1, avg 1, max 1, min 1
         Pipeline 1(instance_num=1):
           DATA_STREAM_SINK_OPERATOR(dest_id=25):
             CommonCounters:
               - ExecTime: avg 80.264us, max 145.694us, min 18.570us
       Fragment 1:
         Pipeline 0(instance_num=48):
           EXCHANGE_OPERATOR(id=25):
             CommonCounters:
               - ExecTime: avg 14.767us, max 31.250us, min 8.681us
           FILE_SCAN_OPERATOR (id=20. nereids_id=1791. table name = web_sales):
             CommonCounters:
               - ExecTime: avg 683.359ms, max 807.567ms, min 544.247ms
               - RowsProduced: sum 183.75K (183750), avg 3.828K (3828), max 4.278K (4278), min 3.452K (3452)
"#;
        
        let mut composer = ProfileComposer::new();
        let result = composer.parse(profile_text);
        
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        
        let profile = result.unwrap();
        assert_eq!(profile.summary.query_id, "37f4f7ab99a741ed-8fd24882055ce279");
        assert_eq!(profile.fragments.len(), 2);
        
        // Check execution tree
        let tree = profile.execution_tree.as_ref().unwrap();
        assert!(tree.nodes.len() >= 4);
        
        // Find FILE_SCAN operator and check it's the most consuming
        let scan_node = tree.nodes.iter().find(|n| n.operator_name.contains("FILE_SCAN"));
        assert!(scan_node.is_some());
        let scan = scan_node.unwrap();
        assert!(scan.time_percentage.is_some());
        assert!(scan.is_most_consuming || scan.time_percentage.unwrap() > 50.0);
    }
}
