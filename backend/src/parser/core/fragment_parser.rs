//! Fragment and Pipeline parser for Doris MergedProfile
//! Parses the hierarchical Fragment -> Pipeline -> Operator structure

use crate::models::{Fragment, Pipeline};
use crate::parser::error::ParseResult;
use crate::parser::core::OperatorParser;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

/// Regex for Fragment header: "Fragment 0:"
static FRAGMENT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*Fragment\s+(\d+):").unwrap()
});

/// Regex for Pipeline header: "Pipeline 0(instance_num=1):"
static PIPELINE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*Pipeline\s+(\d+)\(instance_num=(\d+)\):").unwrap()
});

/// Regex for pipeline-level metrics: "- WaitWorkerTime: avg 18.605us, max 18.605us, min 18.605us"
static PIPELINE_METRIC_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s+-\s+([^:]+):\s+(.+)$").unwrap()
});

pub struct FragmentParser;

impl FragmentParser {
    /// Extract all fragments from MergedProfile text
    pub fn extract_all_fragments(text: &str) -> Vec<Fragment> {
        let mut fragments = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        
        // First, find the "Fragments:" section if present
        let start_line = lines.iter().position(|l| l.trim().starts_with("Fragments:")).unwrap_or(0);
        
        let mut i = start_line;
        while i < lines.len() {
            let line = lines[i];
            
            if let Some(caps) = FRAGMENT_REGEX.captures(line.trim()) {
                let id = caps.get(1).unwrap().as_str().to_string();
                let start_idx = i;
                let base_indent = Self::get_indent(line);
                
                // Find end of this fragment
                let mut end_idx = lines.len();
                for j in (i + 1)..lines.len() {
                    let next_line = lines[j];
                    
                    // Skip empty lines
                    if next_line.trim().is_empty() {
                        continue;
                    }
                    
                    let next_indent = Self::get_indent(next_line);
                    
                    // Fragment ends when we hit another Fragment at same or less indent
                    if next_indent <= base_indent && FRAGMENT_REGEX.is_match(next_line.trim()) {
                        end_idx = j;
                        break;
                    }
                }
                
                let fragment_text = lines[start_idx..end_idx].join("\n");
                
                if let Ok(fragment) = Self::parse_fragment(&fragment_text, &id) {
                    fragments.push(fragment);
                }
                
                i = end_idx;
            } else {
                i += 1;
            }
        }
        
        fragments
    }
    
    /// Parse a single fragment from text
    pub fn parse_fragment(text: &str, id: &str) -> ParseResult<Fragment> {
        let pipelines = Self::parse_pipelines(text)?;
        
        Ok(Fragment {
            id: format!("Fragment {}", id),
            backend_addresses: Vec::new(),
            instance_ids: Vec::new(),
            pipelines,
        })
    }
    
    /// Parse all pipelines within a fragment
    fn parse_pipelines(text: &str) -> ParseResult<Vec<Pipeline>> {
        let mut pipelines = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            
            if let Some(caps) = PIPELINE_REGEX.captures(line.trim()) {
                let id = caps.get(1).unwrap().as_str().to_string();
                let instance_num = caps.get(2).unwrap().as_str().to_string();
                let start_idx = i;
                let base_indent = Self::get_indent(line);
                
                // Find end of this pipeline
                let mut end_idx = lines.len();
                for j in (i + 1)..lines.len() {
                    let next_line = lines[j];
                    let next_indent = Self::get_indent(next_line);
                    
                    // Pipeline ends when we hit another Pipeline or Fragment at same or less indent
                    if next_indent <= base_indent {
                        let trimmed = next_line.trim();
                        if PIPELINE_REGEX.is_match(trimmed) || FRAGMENT_REGEX.is_match(trimmed) {
                            end_idx = j;
                            break;
                        }
                    }
                }
                
                let pipeline_text = lines[start_idx..end_idx].join("\n");
                let pipeline = Self::parse_single_pipeline(&pipeline_text, &id, &instance_num)?;
                pipelines.push(pipeline);
                i = end_idx;
            } else {
                i += 1;
            }
        }
        
        Ok(pipelines)
    }
    
    /// Parse a single pipeline
    fn parse_single_pipeline(text: &str, id: &str, instance_num: &str) -> ParseResult<Pipeline> {
        let mut metrics = HashMap::new();
        metrics.insert("instance_num".to_string(), instance_num.to_string());
        
        // Extract pipeline-level metrics (lines starting with "- " before operators)
        for line in text.lines() {
            let trimmed = line.trim();
            
            // Stop when we hit an operator header
            if OperatorParser::is_operator_header(trimmed) {
                break;
            }
            
            if let Some(caps) = PIPELINE_METRIC_REGEX.captures(trimmed) {
                let key = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                let value = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
                if !key.is_empty() {
                    metrics.insert(key.to_string(), value.to_string());
                }
            }
        }
        
        // Extract operators
        let operators = OperatorParser::extract_operators(text);
        
        Ok(Pipeline {
            id: format!("Pipeline {}", id),
            metrics,
            operators,
        })
    }
    
    /// Get the indentation level of a line (number of leading spaces)
    fn get_indent(line: &str) -> usize {
        line.len() - line.trim_start().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fragment_regex() {
        assert!(FRAGMENT_REGEX.is_match("       Fragment 0:"));
        assert!(FRAGMENT_REGEX.is_match("Fragment 1:"));
        
        let caps = FRAGMENT_REGEX.captures("       Fragment 0:").unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "0");
    }
    
    #[test]
    fn test_pipeline_regex() {
        assert!(PIPELINE_REGEX.is_match("         Pipeline 0(instance_num=1):"));
        assert!(PIPELINE_REGEX.is_match("Pipeline 3(instance_num=48):"));
        
        let caps = PIPELINE_REGEX.captures("Pipeline 3(instance_num=48):").unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "3");
        assert_eq!(caps.get(2).unwrap().as_str(), "48");
    }
    
    #[test]
    fn test_parse_fragment() {
        let text = r#"Fragment 0:
         Pipeline 0(instance_num=1):
            - WaitWorkerTime: avg 18.605us, max 18.605us, min 18.605us
           RESULT_SINK_OPERATOR(id=2147483647):
             CommonCounters:
                - ExecTime: avg 95.241us, max 95.241us, min 95.241us
"#;
        
        let fragment = FragmentParser::parse_fragment(text, "0").unwrap();
        assert_eq!(fragment.id, "Fragment 0");
        assert_eq!(fragment.pipelines.len(), 1);
        assert_eq!(fragment.pipelines[0].id, "Pipeline 0");
    }
    
    #[test]
    fn test_extract_all_fragments() {
        let text = r#"
     Fragments:
       Fragment 0:
         Pipeline 0(instance_num=1):
           - WaitWorkerTime: avg 18.605us
           RESULT_SINK_OPERATOR(id=2147483647):
             CommonCounters:
               - ExecTime: avg 95.241us
       Fragment 1:
         Pipeline 0(instance_num=48):
           EXCHANGE_OPERATOR(id=25):
             CommonCounters:
               - ExecTime: avg 14.767us
"#;
        
        let fragments = FragmentParser::extract_all_fragments(text);
        assert_eq!(fragments.len(), 2, "Expected 2 fragments, got {}", fragments.len());
        assert_eq!(fragments[0].id, "Fragment 0");
        assert_eq!(fragments[1].id, "Fragment 1");
    }
}

