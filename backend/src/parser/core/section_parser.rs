//! Section parser for extracting major sections from Doris profile
//! Handles Summary, ChangedSessionVariables, and MergedProfile sections

use crate::models::ProfileSummary;
use crate::parser::error::{ParseError, ParseResult};
use crate::parser::core::ValueParser;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

/// Regex for parsing summary lines like "   - Key: Value"
static SUMMARY_LINE_REGEX: Lazy<Regex> = 
    Lazy::new(|| Regex::new(r"^\s*-\s+([^:]+):\s*(.*)$").unwrap());

/// Regex for parsing Execution Summary lines
static EXEC_SUMMARY_LINE_REGEX: Lazy<Regex> = 
    Lazy::new(|| Regex::new(r"^\s+-\s+([^:]+):\s*(.*)$").unwrap());

pub struct SectionParser;

impl SectionParser {
    /// Parse the Summary section of the profile
    pub fn parse_summary(text: &str) -> ParseResult<ProfileSummary> {
        let summary_block = Self::extract_section(text, "Summary:")?;
        
        let mut fields = HashMap::new();
        for line in summary_block.lines() {
            if let Some(cap) = SUMMARY_LINE_REGEX.captures(line) {
                let key = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                let value = cap.get(2).map(|m| m.as_str().trim()).unwrap_or("");
                if !key.is_empty() {
                    fields.insert(key.to_string(), value.to_string());
                }
            }
        }
        
        // Also parse Execution Summary if present
        if let Ok(exec_block) = Self::extract_section(text, "Execution Summary:") {
            for line in exec_block.lines() {
                if let Some(cap) = EXEC_SUMMARY_LINE_REGEX.captures(line) {
                    let key = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                    let value = cap.get(2).map(|m| m.as_str().trim()).unwrap_or("");
                    if !key.is_empty() {
                        fields.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }
        
        let total_time = fields.get("Total").cloned().unwrap_or_default();
        let total_time_ms = Self::parse_total_time_ms(&total_time);
        
        Ok(ProfileSummary {
            query_id: fields.get("Profile ID").cloned().unwrap_or_default(),
            start_time: fields.get("Start Time").cloned().unwrap_or_default(),
            end_time: fields.get("End Time").cloned().unwrap_or_default(),
            total_time,
            query_state: fields.get("Task State").cloned().unwrap_or_default(),
            doris_version: fields.get("Doris Version").cloned().unwrap_or_default(),
            sql_statement: fields.get("Sql Statement").cloned().unwrap_or_default(),
            query_type: fields.get("Task Type").cloned(),
            user: fields.get("User").cloned(),
            default_db: fields.get("Default Db").cloned(),
            variables: HashMap::new(),
            total_time_ms,
            query_peak_memory: None,
        })
    }
    
    /// Parse the ChangedSessionVariables section as JSON
    pub fn parse_session_variables(text: &str) -> ParseResult<Vec<HashMap<String, String>>> {
        let start_marker = "ChangedSessionVariables:";
        let start_pos = text.find(start_marker)
            .ok_or_else(|| ParseError::MissingField("ChangedSessionVariables section".to_string()))?;
        
        let after_marker = &text[start_pos + start_marker.len()..];
        
        // Find the JSON array
        let json_start = after_marker.find('[')
            .ok_or_else(|| ParseError::InvalidFormat("No JSON array found".to_string()))?;
        
        // Find matching closing bracket
        let mut depth = 0;
        let mut json_end = json_start;
        for (i, c) in after_marker[json_start..].char_indices() {
            match c {
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        json_end = json_start + i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        
        let json_str = &after_marker[json_start..json_end];
        
        serde_json::from_str(json_str)
            .map_err(|e| ParseError::InvalidFormat(format!("Failed to parse JSON: {}", e)))
    }
    
    /// Extract the MergedProfile section
    pub fn extract_merged_profile(text: &str) -> ParseResult<String> {
        Self::extract_section(text, "MergedProfile:")
    }
    
    /// Extract a section from text starting with the given marker
    pub fn extract_section(text: &str, marker: &str) -> ParseResult<String> {
        let start_pos = text.find(marker)
            .ok_or_else(|| ParseError::MissingField(format!("{} section", marker)))?;
        
        let section_start = start_pos + marker.len();
        let remaining = &text[section_start..];
        
        // Find the end of this section (next top-level section or EOF)
        let section_end = Self::find_section_end(remaining);
        
        Ok(remaining[..section_end].to_string())
    }
    
    /// Find where the current section ends
    fn find_section_end(text: &str) -> usize {
        let lines: Vec<&str> = text.lines().collect();
        
        for (i, line) in lines.iter().enumerate() {
            // Skip first line
            if i == 0 {
                continue;
            }
            
            let trimmed = line.trim();
            
            // Check for next top-level section markers
            if !trimmed.is_empty() 
                && !trimmed.starts_with('-') 
                && !trimmed.starts_with('[')
                && !trimmed.starts_with('{')
                && Self::is_section_header(trimmed) 
            {
                // Calculate position of this line
                let mut pos = 0;
                for j in 0..i {
                    pos += lines[j].len() + 1; // +1 for newline
                }
                return pos;
            }
        }
        
        text.len()
    }
    
    /// Check if a line is a top-level section header
    fn is_section_header(line: &str) -> bool {
        // These are top-level sections (not indented subsections)
        let headers = [
            "Summary:",
            "Execution Summary:",
            "ChangedSessionVariables:",
            "MergedProfile:",
            // Note: "Fragments:" is NOT a top-level section - it's inside MergedProfile
        ];
        
        headers.iter().any(|h| line.starts_with(h))
    }
    
    /// Parse total time string to milliseconds
    fn parse_total_time_ms(time_str: &str) -> Option<f64> {
        ValueParser::parse_time_to_ms(time_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_summary() {
        let profile_text = r#"Summary:
   - Profile ID: 37f4f7ab99a741ed-8fd24882055ce279
   - Task Type: QUERY
   - Start Time: 2025-12-15 19:55:24
   - End Time: 2025-12-15 19:55:25
   - Total: 1sec240ms
   - Task State: OK
   - User: root
Execution Summary:
   - Parse SQL Time: 2ms
"#;
        
        let summary = SectionParser::parse_summary(profile_text).unwrap();
        assert_eq!(summary.query_id, "37f4f7ab99a741ed-8fd24882055ce279");
        assert_eq!(summary.query_state, "OK");
        assert_eq!(summary.total_time, "1sec240ms");
        assert!(summary.total_time_ms.is_some());
    }
    
    #[test]
    fn test_parse_session_variables() {
        let text = r#"ChangedSessionVariables:
[
  {
    "VarName": "enable_profile",
    "CurrentValue": "true",
    "DefaultValue": "false"
  }
]
MergedProfile:
"#;
        
        let vars = SectionParser::parse_session_variables(text).unwrap();
        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0].get("VarName").unwrap(), "enable_profile");
    }
    
    #[test]
    fn test_extract_merged_profile() {
        let text = r#"ChangedSessionVariables:
[]
MergedProfile:
     Fragments:
       Fragment 0:
         Pipeline 0(instance_num=1):
           RESULT_SINK_OPERATOR(id=2147483647):
             CommonCounters:
               - ExecTime: avg 95.241us
"#;
        
        let merged = SectionParser::extract_merged_profile(text).unwrap();
        assert!(merged.contains("Fragment 0:"), "Should contain Fragment 0, got: {}", merged);
        assert!(merged.contains("Pipeline 0"), "Should contain Pipeline 0");
    }
}

