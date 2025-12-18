use crate::models::*;
use crate::constants::scores;
use crate::ai::AiDiagnosisService;
use crate::config::DefaultSuggestionsConfig;

/// SuggestionEngine generates optimization suggestions based on detected hotspots
pub struct SuggestionEngine;

impl SuggestionEngine {
    /// Fill suggestions for hotspots using AI or default suggestions
    pub async fn fill_suggestions(
        hotspots: &mut Vec<HotSpot>,
        profile: &Profile,
        ai_service: Option<&AiDiagnosisService>,
        default_config: &DefaultSuggestionsConfig,
    ) {
        for hotspot in hotspots.iter_mut() {
            // Find corresponding node
            if let Some(ref tree) = profile.execution_tree {
                if let Some(node) = tree.nodes.iter().find(|n| n.id == hotspot.node_id) {
                    // Try to use AI to generate suggestion
                    let (suggestion, source) = if let Some(ai) = ai_service {
                        if ai.is_enabled() {
                            match ai.generate_suggestion(node, profile).await {
                                Ok(s) => {
                                    (s, "ai".to_string())
                                }
                                Err(e) => {
                                    let error_msg = format!("AI Suggestion failed: {}", e);
                                    eprintln!("{} for node {}, using default", error_msg, node.id);
                                    let default_suggestion = Self::get_default_suggestion(
                                        &hotspot.operator_name, 
                                        &hotspot.severity, 
                                        default_config
                                    );
                                    (default_suggestion, error_msg)
                                }
                            }
                        } else {
                            let default_suggestion = Self::get_default_suggestion(
                                &hotspot.operator_name, 
                                &hotspot.severity, 
                                default_config
                            );
                            (default_suggestion, "AI Suggestion is not enabled".to_string())
                        }
                    } else {
                        let default_suggestion = Self::get_default_suggestion(
                            &hotspot.operator_name, 
                            &hotspot.severity, 
                            default_config
                        );
                        (default_suggestion, "AI Suggestion is not enabled".to_string())
                    };
                    
                    hotspot.suggestion = Some(suggestion);
                    hotspot.suggestion_source = Some(source);
                }
            }
        }
    }
    
    /// Get default suggestion from configuration file
    fn get_default_suggestion(
        operator_name: &str,
        severity: &HotspotSeverity,
        config: &DefaultSuggestionsConfig,
    ) -> String {
        // Try to match operator name
        let suggestions = config.suggestions.get(operator_name)
            .or_else(|| config.suggestions.get("DEFAULT"));
        
        if let Some(sev_suggestions) = suggestions {
            let suggestions_list = match severity {
                HotspotSeverity::Critical => &sev_suggestions.critical,
                HotspotSeverity::High => &sev_suggestions.high,
                HotspotSeverity::Medium => &sev_suggestions.medium,
                HotspotSeverity::Low => &sev_suggestions.low,
                HotspotSeverity::None => return "暂无优化建议".to_string(),
            };
            
            // Return first suggestion if available, or combine multiple
            if suggestions_list.is_empty() {
                "暂无优化建议".to_string()
            } else {
                suggestions_list.join("\n")
            }
        } else {
            "暂无优化建议".to_string()
        }
    }
    

    /// Generate a conclusion summary based on hotspots and profile
    pub fn generate_conclusion(hotspots: &[HotSpot], profile: &Profile) -> String {
        let total_time = profile.summary.total_time.clone();
        let hotspot_count = hotspots.len();
        
        if hotspots.is_empty() {
            return format!(
                "Query completed in {} with no significant performance issues detected.",
                total_time
            );
        }
        
        let critical_count = hotspots.iter()
            .filter(|h| h.severity == HotspotSeverity::Critical)
            .count();
        
        let high_count = hotspots.iter()
            .filter(|h| h.severity == HotspotSeverity::High)
            .count();
        
        if critical_count > 0 {
            format!(
                "Query completed in {} with {} critical performance bottleneck(s) and {} total issue(s) detected. Immediate attention recommended.",
                total_time, critical_count, hotspot_count
            )
        } else if high_count > 0 {
            format!(
                "Query completed in {} with {} high-severity issue(s) and {} total issue(s) detected. Optimization recommended.",
                total_time, high_count, hotspot_count
            )
        } else {
            format!(
                "Query completed in {} with {} minor performance issue(s) detected.",
                total_time, hotspot_count
            )
        }
    }
    
    /// Generate optimization suggestions based on detected hotspots
    pub fn generate_suggestions(hotspots: &[HotSpot]) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        let mut seen_categories: std::collections::HashSet<String> = std::collections::HashSet::new();
        
        for hotspot in hotspots {
            // Skip if we already have a suggestion for this category
            let category_key = format!("{:?}-{}", hotspot.severity, &hotspot.operator_name);
            if seen_categories.contains(&category_key) {
                continue;
            }
            seen_categories.insert(category_key);
            
            if let Some(ref suggestion_text) = hotspot.suggestion {
                let (priority, category) = Self::categorize_suggestion(hotspot);
                
                suggestions.push(Suggestion {
                    title: format!("Optimize {} operator", hotspot.operator_name),
                    description: suggestion_text.clone(),
                    priority,
                    category,
                });
            }
        }
        
        // Add general suggestions if there are many hotspots
        if hotspots.len() >= 3 {
            suggestions.push(Suggestion {
                title: "Consider query restructuring".to_string(),
                description: "Multiple performance bottlenecks detected. Consider breaking the query into smaller parts or restructuring the query logic.".to_string(),
                priority: SuggestionPriority::Medium,
                category: SuggestionCategory::Query,
            });
        }
        
        // Sort suggestions by priority
        suggestions.sort_by(|a, b| {
            Self::priority_rank(&b.priority).cmp(&Self::priority_rank(&a.priority))
        });
        
        suggestions
    }
    
    /// Calculate overall performance score (0-100)
    pub fn calculate_performance_score(hotspots: &[HotSpot], profile: &Profile) -> u32 {
        if hotspots.is_empty() {
            return 95; // Excellent score if no hotspots
        }
        
        // Start with base score
        let mut score: i32 = 100;
        
        // Deduct points based on hotspot severity
        for hotspot in hotspots {
            match hotspot.severity {
                HotspotSeverity::Critical => score -= 30,
                HotspotSeverity::High => score -= 20,
                HotspotSeverity::Medium => score -= 10,
                HotspotSeverity::Low => score -= 5,
                HotspotSeverity::None => {}
            }
        }
        
        // Consider total execution time
        if let Some(total_ms) = profile.summary.total_time_ms {
            if total_ms > 60000.0 {
                score -= 15; // Very long query
            } else if total_ms > 10000.0 {
                score -= 10; // Long query
            } else if total_ms > 5000.0 {
                score -= 5; // Moderate query
            }
        }
        
        // Ensure score is within valid range
        score.clamp(0, 100) as u32
    }
    
    /// Get score category description
    pub fn get_score_category(score: u32) -> &'static str {
        if score >= scores::EXCELLENT {
            "Excellent"
        } else if score >= scores::GOOD {
            "Good"
        } else if score >= scores::FAIR {
            "Fair"
        } else if score >= scores::POOR {
            "Poor"
        } else {
            "Critical"
        }
    }
    
    /// Categorize a suggestion based on the hotspot
    fn categorize_suggestion(hotspot: &HotSpot) -> (SuggestionPriority, SuggestionCategory) {
        let priority = match hotspot.severity {
            HotspotSeverity::Critical => SuggestionPriority::Critical,
            HotspotSeverity::High => SuggestionPriority::High,
            HotspotSeverity::Medium => SuggestionPriority::Medium,
            HotspotSeverity::Low => SuggestionPriority::Low,
            HotspotSeverity::None => SuggestionPriority::Low,
        };
        
        let category = match hotspot.operator_name.as_str() {
            name if name.contains("SCAN") => SuggestionCategory::Schema,
            name if name.contains("JOIN") => SuggestionCategory::Query,
            name if name.contains("EXCHANGE") => SuggestionCategory::Configuration,
            name if name.contains("AGGREGATE") => SuggestionCategory::Query,
            _ => SuggestionCategory::Query,
        };
        
        (priority, category)
    }
    
    /// Get numeric rank for priority (higher = more important)
    fn priority_rank(priority: &SuggestionPriority) -> u8 {
        match priority {
            SuggestionPriority::Critical => 4,
            SuggestionPriority::High => 3,
            SuggestionPriority::Medium => 2,
            SuggestionPriority::Low => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_profile() -> Profile {
        Profile {
            summary: ProfileSummary {
                query_id: "test-123".to_string(),
                total_time: "1s".to_string(),
                total_time_ms: Some(1000.0),
                ..Default::default()
            },
            fragments: vec![],
            execution_tree: None,
        }
    }
    
    #[test]
    fn test_generate_conclusion_no_hotspots() {
        let profile = create_test_profile();
        let conclusion = SuggestionEngine::generate_conclusion(&[], &profile);
        assert!(conclusion.contains("no significant performance issues"));
    }
    
    #[test]
    fn test_calculate_score_no_hotspots() {
        let profile = create_test_profile();
        let score = SuggestionEngine::calculate_performance_score(&[], &profile);
        assert!(score >= 90);
    }
    
    #[test]
    fn test_score_category() {
        assert_eq!(SuggestionEngine::get_score_category(95), "Excellent");
        assert_eq!(SuggestionEngine::get_score_category(75), "Good");
        assert_eq!(SuggestionEngine::get_score_category(55), "Fair");
        assert_eq!(SuggestionEngine::get_score_category(35), "Poor");
        assert_eq!(SuggestionEngine::get_score_category(15), "Critical");
    }
}

