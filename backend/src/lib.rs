pub mod parser;
pub mod models;
pub mod diagnostic;
pub mod api;
pub mod constants;
pub mod static_files;
pub mod config;
pub mod ai;

pub use models::*;
pub use diagnostic::performance_bottleneck::PerformanceBottleneck;
pub use diagnostic::optimization_advisor::OptimizationAdvisor;
pub use parser::ProfileComposer;
pub use config::ConfigLoader;
pub use ai::AiDiagnosisService;

/// Main entry point for analyzing a Doris profile text
pub fn analyze_profile(profile_text: &str) -> Result<ProfileAnalysisResponse, String> {
    let mut composer = ProfileComposer::new();
    let profile = composer.parse(profile_text)
        .map_err(|e| format!("Failed to parse profile: {:?}", e))?;

    let hotspots = PerformanceBottleneck::analyze(&profile);
    let conclusion = OptimizationAdvisor::generate_conclusion(&hotspots, &profile);
    let suggestions = OptimizationAdvisor::generate_suggestions(&hotspots);
    let performance_score = OptimizationAdvisor::calculate_performance_score(&hotspots, &profile);
    let execution_tree = profile.execution_tree.clone();
    let summary = Some(profile.summary.clone());

    Ok(ProfileAnalysisResponse {
        hotspots,
        conclusion,
        suggestions,
        performance_score,
        execution_tree,
        summary,
    })
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_full_analysis_pipeline() {
        let profile_text = fs::read_to_string("../test/test-profile.txt")
            .expect("Failed to read test profile");
        
        let result = analyze_profile(&profile_text);
        assert!(result.is_ok(), "Analysis failed: {:?}", result.err());
        
        let response = result.unwrap();
        
        // Verify summary
        assert!(response.summary.is_some());
        let summary = response.summary.unwrap();
        assert_eq!(summary.query_id, "37f4f7ab99a741ed-8fd24882055ce279");
        
        // Verify execution tree
        assert!(response.execution_tree.is_some());
        let tree = response.execution_tree.unwrap();
        assert!(!tree.nodes.is_empty());
        
        // Verify hotspots are detected
        assert!(!response.hotspots.is_empty(), "Should detect hotspots");
        
        // Verify performance score is reasonable
        assert!(response.performance_score <= 100);
        
        // Verify conclusion is generated
        assert!(!response.conclusion.is_empty());
        
        println!("Summary: {}", summary.query_id);
        println!("Total time: {}", summary.total_time);
        println!("Execution tree nodes: {}", tree.nodes.len());
        println!("Hotspots detected: {}", response.hotspots.len());
        println!("Performance score: {}", response.performance_score);
        println!("Conclusion: {}", response.conclusion);
        println!("Suggestions: {}", response.suggestions.len());
        
        for hotspot in &response.hotspots {
            println!("  Hotspot: {} ({:?}) - {}", 
                hotspot.operator_name, 
                hotspot.severity,
                hotspot.time_percentage.map(|p| format!("{:.1}%", p)).unwrap_or_default()
            );
        }
    }
}

