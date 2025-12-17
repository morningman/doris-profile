//! Value parser for Doris profile metrics
//! Handles parsing of time, memory, count, and aggregated value formats

use once_cell::sync::Lazy;
use regex::Regex;

/// Regex for parsing complex time format like "1sec240ms"
static COMPLEX_TIME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\d+)sec(\d+)ms").unwrap()
});

/// Regex for parsing memory values like "128.00 B", "4.00 KB", "2.24 MB", "1.40 GB"
static MEMORY_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\d+(?:\.\d+)?)\s*(B|KB|MB|GB|TB)").unwrap()
});

/// Regex for parsing count values like "183.75K (183750)", "720.000376M (720000376)"
static COUNT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\d+(?:\.\d+)?)\s*([KMB])?\s*(?:\((\d+)\))?").unwrap()
});

/// Regex for parsing aggregated metrics like "avg 95.241us, max 95.241us, min 95.241us"
static AGGREGATED_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(sum|avg|max|min)\s+([^,]+)").unwrap()
});

/// Parsed aggregated metric with sum/avg/max/min values
#[derive(Debug, Clone, Default)]
pub struct AggregatedValue {
    pub sum: Option<i64>,
    pub avg: Option<i64>,
    pub max: Option<i64>,
    pub min: Option<i64>,
    pub raw: String,
}

/// Value parser utility
pub struct ValueParser;

impl ValueParser {
    /// Parse a time string to nanoseconds
    /// Supports formats: "1sec240ms", "835.207ms", "18.605us", "0ns", "N/A"
    pub fn parse_time_to_ns(time_str: &str) -> Option<i64> {
        let trimmed = time_str.trim();
        
        if trimmed.is_empty() || trimmed == "N/A" || trimmed == "0" {
            return Some(0);
        }
        
        // Handle "XsecYms" format
        if let Some(caps) = COMPLEX_TIME_REGEX.captures(trimmed) {
            let secs: i64 = caps.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
            let ms: i64 = caps.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
            return Some(secs * 1_000_000_000 + ms * 1_000_000);
        }
        
        // Handle single unit formats
        if trimmed.ends_with("ns") {
            let num_str = trimmed.trim_end_matches("ns").trim();
            return num_str.parse::<f64>().ok().map(|n| n as i64);
        }
        
        if trimmed.ends_with("us") {
            let num_str = trimmed.trim_end_matches("us").trim();
            return num_str.parse::<f64>().ok().map(|n| (n * 1_000.0) as i64);
        }
        
        if trimmed.ends_with("ms") {
            let num_str = trimmed.trim_end_matches("ms").trim();
            return num_str.parse::<f64>().ok().map(|n| (n * 1_000_000.0) as i64);
        }
        
        if trimmed.ends_with('s') && !trimmed.ends_with("ms") && !trimmed.ends_with("us") && !trimmed.ends_with("ns") {
            let num_str = trimmed.trim_end_matches('s').trim();
            return num_str.parse::<f64>().ok().map(|n| (n * 1_000_000_000.0) as i64);
        }
        
        // Try parsing as plain number (assume nanoseconds)
        trimmed.parse::<f64>().ok().map(|n| n as i64)
    }
    
    /// Parse a time string to milliseconds
    pub fn parse_time_to_ms(time_str: &str) -> Option<f64> {
        Self::parse_time_to_ns(time_str).map(|ns| ns as f64 / 1_000_000.0)
    }
    
    /// Parse a memory string to bytes
    /// Supports formats: "128.00 B", "4.00 KB", "2.24 MB", "1.40 GB", "0.00 "
    pub fn parse_memory_to_bytes(mem_str: &str) -> Option<u64> {
        let trimmed = mem_str.trim();
        
        if trimmed.is_empty() || trimmed == "N/A" || trimmed == "0.00 " || trimmed == "0.00" {
            return Some(0);
        }
        
        if let Some(caps) = MEMORY_REGEX.captures(trimmed) {
            let value: f64 = caps.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(0.0);
            let unit = caps.get(2).map(|m| m.as_str()).unwrap_or("B");
            
            let multiplier: f64 = match unit {
                "B" => 1.0,
                "KB" => 1024.0,
                "MB" => 1024.0 * 1024.0,
                "GB" => 1024.0 * 1024.0 * 1024.0,
                "TB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
                _ => 1.0,
            };
            
            return Some((value * multiplier) as u64);
        }
        
        None
    }
    
    /// Parse a count string to integer
    /// Supports formats: "183.75K (183750)", "720.000376M (720000376)", "1.298K (1298)"
    pub fn parse_count(count_str: &str) -> Option<i64> {
        let trimmed = count_str.trim();
        
        if trimmed.is_empty() || trimmed == "N/A" {
            return Some(0);
        }
        
        // Check if there's an exact value in parentheses
        if let Some(start) = trimmed.find('(') {
            if let Some(end) = trimmed.find(')') {
                let exact = &trimmed[start + 1..end];
                if let Ok(val) = exact.parse::<i64>() {
                    return Some(val);
                }
            }
        }
        
        // Parse with suffix
        if let Some(caps) = COUNT_REGEX.captures(trimmed) {
            let value: f64 = caps.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(0.0);
            let suffix = caps.get(2).map(|m| m.as_str());
            
            let multiplier: f64 = match suffix {
                Some("K") => 1_000.0,
                Some("M") => 1_000_000.0,
                Some("B") => 1_000_000_000.0,
                _ => 1.0,
            };
            
            return Some((value * multiplier) as i64);
        }
        
        // Try plain number
        trimmed.parse::<i64>().ok()
    }
    
    /// Parse an aggregated metric string
    /// Format: "avg 95.241us, max 95.241us, min 95.241us" or "sum 1, avg 1, max 1, min 1"
    pub fn parse_aggregated(value_str: &str) -> AggregatedValue {
        let mut result = AggregatedValue {
            raw: value_str.to_string(),
            ..Default::default()
        };
        
        for caps in AGGREGATED_REGEX.captures_iter(value_str) {
            let stat_type = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let value_part = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
            
            // Try to parse as time first, then as count
            let parsed_value = Self::parse_time_to_ns(value_part)
                .or_else(|| Self::parse_count(value_part));
            
            match stat_type {
                "sum" => result.sum = parsed_value,
                "avg" => result.avg = parsed_value,
                "max" => result.max = parsed_value,
                "min" => result.min = parsed_value,
                _ => {}
            }
        }
        
        result
    }
    
    /// Parse aggregated metric and return avg value in milliseconds (for time metrics)
    pub fn parse_aggregated_time_ms(value_str: &str) -> Option<f64> {
        let agg = Self::parse_aggregated(value_str);
        agg.avg.map(|ns| ns as f64 / 1_000_000.0)
    }
    
    /// Extract the first numeric value from a metric string
    pub fn extract_first_value(value_str: &str) -> Option<String> {
        let trimmed = value_str.trim();
        
        // Look for "avg X" pattern first
        if let Some(pos) = trimmed.find("avg ") {
            let rest = &trimmed[pos + 4..];
            if let Some(end) = rest.find(',') {
                return Some(rest[..end].trim().to_string());
            }
            return Some(rest.trim().to_string());
        }
        
        // Otherwise return the whole string
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_time_to_ns() {
        assert_eq!(ValueParser::parse_time_to_ns("0ns"), Some(0));
        assert_eq!(ValueParser::parse_time_to_ns("100ns"), Some(100));
        assert_eq!(ValueParser::parse_time_to_ns("18.605us"), Some(18605));
        assert_eq!(ValueParser::parse_time_to_ns("835.207ms"), Some(835207000));
        assert_eq!(ValueParser::parse_time_to_ns("1sec240ms"), Some(1240000000));
        assert_eq!(ValueParser::parse_time_to_ns("N/A"), Some(0));
    }
    
    #[test]
    fn test_parse_memory_to_bytes() {
        assert_eq!(ValueParser::parse_memory_to_bytes("0.00 "), Some(0));
        assert_eq!(ValueParser::parse_memory_to_bytes("128.00 B"), Some(128));
        assert_eq!(ValueParser::parse_memory_to_bytes("4.00 KB"), Some(4096));
        assert_eq!(ValueParser::parse_memory_to_bytes("2.24 MB"), Some(2348810));
        assert_eq!(ValueParser::parse_memory_to_bytes("1.40 GB"), Some(1503238553));
    }
    
    #[test]
    fn test_parse_count() {
        assert_eq!(ValueParser::parse_count("1"), Some(1));
        assert_eq!(ValueParser::parse_count("183.75K (183750)"), Some(183750));
        assert_eq!(ValueParser::parse_count("720.000376M (720000376)"), Some(720000376));
        assert_eq!(ValueParser::parse_count("1.298K (1298)"), Some(1298));
    }
    
    #[test]
    fn test_parse_aggregated() {
        let agg = ValueParser::parse_aggregated("avg 95.241us, max 95.241us, min 95.241us");
        assert_eq!(agg.avg, Some(95241));
        assert_eq!(agg.max, Some(95241));
        assert_eq!(agg.min, Some(95241));
        
        let agg2 = ValueParser::parse_aggregated("sum 1, avg 1, max 1, min 1");
        assert_eq!(agg2.sum, Some(1));
        assert_eq!(agg2.avg, Some(1));
    }
}

