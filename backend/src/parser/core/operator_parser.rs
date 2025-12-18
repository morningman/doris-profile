//! Operator parser for Doris profile
//! Parses individual operators with their PlanInfo, CommonCounters, and CustomCounters

use crate::models::Operator;
use crate::parser::core::ValueParser;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

/// Regex for operator header: "OPERATOR_NAME(nereids_id=X)(id=Y):" or "OPERATOR_NAME(id=Y):"
/// Also handles special cases like "FILE_SCAN_OPERATOR (id=20. nereids_id=1791. table name = web_sales):"
static OPERATOR_HEADER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*([A-Z][A-Z0-9_]*(?:_OPERATOR)?)\s*(?:\(nereids_id=(\d+)\))?\s*\(id=(-?\d+)").unwrap()
});

/// Alternative regex for operators with different format
static OPERATOR_HEADER_ALT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*([A-Z][A-Z0-9_]*(?:_OPERATOR)?)\s+\(id=(-?\d+)\.?\s*(?:nereids_id=(\d+))?").unwrap()
});

/// Regex for DATA_STREAM_SINK with dest_id
static DATA_STREAM_SINK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"DATA_STREAM_SINK_OPERATOR\(dest_id=(\d+)\)").unwrap()
});

/// Regex for LOCAL_EXCHANGE operators with type
static LOCAL_EXCHANGE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"LOCAL_EXCHANGE_(?:SINK_)?OPERATOR\(([A-Z_]+)\)\(id=(-?\d+)\)").unwrap()
});

/// Regex for metric lines: "- MetricName: value"
static METRIC_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*-\s+([^:]+):\s*(.*)$").unwrap()
});

/// Parsed operator with all details
#[derive(Debug, Clone)]
pub struct ParsedOperator {
    pub name: String,
    pub id: i32,
    pub nereids_id: Option<i32>,
    pub dest_id: Option<i32>,  // For DATA_STREAM_SINK
    pub exchange_type: Option<String>,  // For LOCAL_EXCHANGE
    pub plan_info: HashMap<String, String>,
    pub common_counters: HashMap<String, String>,
    pub custom_counters: HashMap<String, String>,
    pub table_name: Option<String>,
}

pub struct OperatorParser;

impl OperatorParser {
    /// Check if a line is an operator header
    pub fn is_operator_header(line: &str) -> bool {
        let trimmed = line.trim();
        
        // Skip metric lines and section headers
        if trimmed.starts_with('-') || trimmed.starts_with("CommonCounters") 
            || trimmed.starts_with("CustomCounters") || trimmed.starts_with("RuntimeFilterInfo") {
            return false;
        }
        
        OPERATOR_HEADER_REGEX.is_match(trimmed) 
            || OPERATOR_HEADER_ALT_REGEX.is_match(trimmed)
            || DATA_STREAM_SINK_REGEX.is_match(trimmed)
            || LOCAL_EXCHANGE_REGEX.is_match(trimmed)
    }
    
    /// Extract all operators from pipeline text
    pub fn extract_operators(text: &str) -> Vec<Operator> {
        let parsed = Self::extract_parsed_operators(text);
        
        parsed.into_iter().map(|p| {
            let metrics = Self::merge_metrics(&p);
            Operator {
                id: p.id.to_string(),
                name: p.name,
                metrics,
            }
        }).collect()
    }
    
    /// Extract operators with full details
    pub fn extract_parsed_operators(text: &str) -> Vec<ParsedOperator> {
        let mut operators = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            
            if Self::is_operator_header(line.trim()) {
                let start_idx = i;
                let base_indent = Self::get_indent(line);
                
                // Find end of this operator
                let mut end_idx = lines.len();
                for j in (i + 1)..lines.len() {
                    let next_line = lines[j];
                    let next_indent = Self::get_indent(next_line);
                    let next_trimmed = next_line.trim();
                    
                    // Skip empty lines
                    if next_trimmed.is_empty() {
                        continue;
                    }
                    
                    // Operator ends when we hit another operator at same or less indent
                    if next_indent <= base_indent && Self::is_operator_header(next_trimmed) {
                        end_idx = j;
                        break;
                    }
                    
                    // Also check for Pipeline or Fragment headers
                    if next_trimmed.starts_with("Pipeline ") || next_trimmed.starts_with("Fragment ") {
                        end_idx = j;
                        break;
                    }
                }
                
                let operator_text = lines[start_idx..end_idx].join("\n");
                if let Some(parsed) = Self::parse_operator(&operator_text) {
                    operators.push(parsed);
                }
                
                i = end_idx;
            } else {
                i += 1;
            }
        }
        
        operators
    }
    
    /// Parse a single operator block
    fn parse_operator(text: &str) -> Option<ParsedOperator> {
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return None;
        }
        
        let header = lines[0].trim();
        
        // Parse header to extract name, id, nereids_id
        let (name, id, nereids_id, dest_id, exchange_type, table_name) = Self::parse_header(header)?;
        
        let mut plan_info = HashMap::new();
        let mut common_counters = HashMap::new();
        let mut custom_counters = HashMap::new();
        
        let mut current_section = "none";
        let mut in_plan_info = false;
        
        for line in lines.iter().skip(1) {
            let trimmed = line.trim();
            
            if trimmed.is_empty() {
                continue;
            }
            
            // Check for section headers
            if trimmed == "CommonCounters:" {
                current_section = "common";
                in_plan_info = false;
                continue;
            } else if trimmed == "CustomCounters:" {
                current_section = "custom";
                in_plan_info = false;
                continue;
            } else if trimmed == "- PlanInfo" {
                in_plan_info = true;
                current_section = "plan";
                continue;
            } else if trimmed.starts_with("RuntimeFilterInfo:") {
                // RuntimeFilterInfo is part of custom counters
                current_section = "custom";
                continue;
            }
            
            // Parse metric lines
            if let Some(caps) = METRIC_REGEX.captures(trimmed) {
                let key = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                let value = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
                
                if in_plan_info {
                    plan_info.insert(key.to_string(), value.to_string());
                } else {
                    match current_section {
                        "common" => { common_counters.insert(key.to_string(), value.to_string()); }
                        "custom" => { custom_counters.insert(key.to_string(), value.to_string()); }
                        _ => {}
                    }
                }
            }
        }
        
        Some(ParsedOperator {
            name,
            id,
            nereids_id,
            dest_id,
            exchange_type,
            plan_info,
            common_counters,
            custom_counters,
            table_name,
        })
    }
    
    /// Parse operator header line
    fn parse_header(header: &str) -> Option<(String, i32, Option<i32>, Option<i32>, Option<String>, Option<String>)> {
        let trimmed = header.trim().trim_end_matches(':');
        
        // Check for DATA_STREAM_SINK with dest_id
        if let Some(caps) = DATA_STREAM_SINK_REGEX.captures(trimmed) {
            let dest_id: i32 = caps.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
            return Some((
                "DATA_STREAM_SINK_OPERATOR".to_string(),
                -1, // No explicit id
                None,
                Some(dest_id),
                None,
                None,
            ));
        }
        
        // Check for LOCAL_EXCHANGE with type
        if let Some(caps) = LOCAL_EXCHANGE_REGEX.captures(trimmed) {
            let exchange_type = caps.get(1).map(|m| m.as_str().to_string());
            let id: i32 = caps.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
            let name = if trimmed.contains("SINK") {
                "LOCAL_EXCHANGE_SINK_OPERATOR"
            } else {
                "LOCAL_EXCHANGE_OPERATOR"
            };
            return Some((name.to_string(), id, None, None, exchange_type, None));
        }
        
        // Try standard format: OPERATOR(nereids_id=X)(id=Y)
        if let Some(caps) = OPERATOR_HEADER_REGEX.captures(trimmed) {
            let name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            let nereids_id: Option<i32> = caps.get(2).and_then(|m| m.as_str().parse().ok());
            let id: i32 = caps.get(3).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
            
            // Extract table name if present
            let table_name = Self::extract_table_name(trimmed);
            
            return Some((name, id, nereids_id, None, None, table_name));
        }
        
        // Try alternate format: OPERATOR (id=X. nereids_id=Y. ...)
        if let Some(caps) = OPERATOR_HEADER_ALT_REGEX.captures(trimmed) {
            let name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            let id: i32 = caps.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
            let nereids_id: Option<i32> = caps.get(3).and_then(|m| m.as_str().parse().ok());
            
            // Extract table name if present
            let table_name = Self::extract_table_name(trimmed);
            
            return Some((name, id, nereids_id, None, None, table_name));
        }
        
        None
    }
    
    /// Extract table name from operator header
    fn extract_table_name(header: &str) -> Option<String> {
        if let Some(pos) = header.find("table name = ") {
            let rest = &header[pos + 13..];
            let end = rest.find(')').unwrap_or(rest.len());
            return Some(rest[..end].trim().to_string());
        }
        None
    }
    
    /// Merge all metrics into a single HashMap
    fn merge_metrics(parsed: &ParsedOperator) -> HashMap<String, String> {
        let mut metrics = HashMap::new();
        
        // Add ID info
        if let Some(nid) = parsed.nereids_id {
            metrics.insert("nereids_id".to_string(), nid.to_string());
        }
        if let Some(did) = parsed.dest_id {
            metrics.insert("dest_id".to_string(), did.to_string());
        }
        if let Some(ref et) = parsed.exchange_type {
            metrics.insert("exchange_type".to_string(), et.clone());
        }
        if let Some(ref tn) = parsed.table_name {
            metrics.insert("table_name".to_string(), tn.clone());
        }
        
        // Add plan info
        for (k, v) in &parsed.plan_info {
            metrics.insert(format!("plan_{}", k), v.clone());
        }
        
        // Add common counters
        for (k, v) in &parsed.common_counters {
            metrics.insert(k.clone(), v.clone());
        }
        
        // Add custom counters
        for (k, v) in &parsed.custom_counters {
            metrics.insert(k.clone(), v.clone());
        }
        
        metrics
    }
    
    /// Get indentation of a line
    fn get_indent(line: &str) -> usize {
        line.len() - line.trim_start().len()
    }
    
    /// Get the execution time in nanoseconds from an operator
    pub fn get_exec_time_ns(operator: &ParsedOperator) -> Option<i64> {
        operator.common_counters.get("ExecTime")
            .and_then(|v| ValueParser::parse_aggregated(v).avg)
    }
    
    /// Get rows produced by an operator
    pub fn get_rows_produced(operator: &ParsedOperator) -> Option<i64> {
        operator.common_counters.get("RowsProduced")
            .and_then(|v| {
                let agg = ValueParser::parse_aggregated(v);
                agg.sum.or(agg.avg)
            })
    }
    
    pub fn get_input_rows(operator: &ParsedOperator) -> Option<i64> {
        operator.common_counters.get("InputRows")
            .and_then(|v| {
                let agg = ValueParser::parse_aggregated(v);
                agg.sum.or(agg.avg)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_operator_header() {
        assert!(OperatorParser::is_operator_header("RESULT_SINK_OPERATOR(id=2147483647):"));
        assert!(OperatorParser::is_operator_header("SORT_OPERATOR(nereids_id=1966)(id=28):"));
        assert!(OperatorParser::is_operator_header("FILE_SCAN_OPERATOR (id=20. nereids_id=1791. table name = web_sales):"));
        assert!(OperatorParser::is_operator_header("DATA_STREAM_SINK_OPERATOR(dest_id=25):"));
        assert!(OperatorParser::is_operator_header("LOCAL_EXCHANGE_OPERATOR(PASSTHROUGH)(id=-10):"));
        
        assert!(!OperatorParser::is_operator_header("- ExecTime: avg 95.241us"));
        assert!(!OperatorParser::is_operator_header("CommonCounters:"));
    }
    
    #[test]
    fn test_parse_header() {
        let (name, id, nid, _did, _et, _tn) = OperatorParser::parse_header("SORT_OPERATOR(nereids_id=1966)(id=28):").unwrap();
        assert_eq!(name, "SORT_OPERATOR");
        assert_eq!(id, 28);
        assert_eq!(nid, Some(1966));
        
        let (name, _id, _, did, _, _) = OperatorParser::parse_header("DATA_STREAM_SINK_OPERATOR(dest_id=25):").unwrap();
        assert_eq!(name, "DATA_STREAM_SINK_OPERATOR");
        assert_eq!(did, Some(25));
    }
    
    #[test]
    fn test_extract_operators() {
        let text = r#"Pipeline 0(instance_num=1):
           RESULT_SINK_OPERATOR(id=2147483647):
             CommonCounters:
                - ExecTime: avg 95.241us, max 95.241us, min 95.241us
                - InputRows: sum 1, avg 1, max 1, min 1
           SORT_OPERATOR(nereids_id=1966)(id=28):
             CommonCounters:
                - ExecTime: avg 14.905us, max 14.905us, min 14.905us
"#;
        
        let operators = OperatorParser::extract_operators(text);
        assert_eq!(operators.len(), 2);
        assert_eq!(operators[0].name, "RESULT_SINK_OPERATOR");
        assert_eq!(operators[1].name, "SORT_OPERATOR");
    }
}

