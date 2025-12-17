// This is a temporary test file - will be removed after testing

#[cfg(test)]
mod tests {
    use crate::parser::ProfileComposer;
    use std::fs;

    #[test]
    fn test_parse_real_profile() {
        let profile_text = fs::read_to_string("../test/test-profile.txt")
            .expect("Failed to read test profile");
        
        let mut composer = ProfileComposer::new();
        let result = composer.parse(&profile_text);
        
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        
        let profile = result.unwrap();
        
        // Verify summary
        assert_eq!(profile.summary.query_id, "37f4f7ab99a741ed-8fd24882055ce279");
        assert_eq!(profile.summary.query_state, "OK");
        assert!(profile.summary.total_time_ms.is_some());
        
        // Verify fragments
        assert!(!profile.fragments.is_empty(), "Should have fragments");
        println!("Found {} fragments", profile.fragments.len());
        
        for frag in profile.fragments.iter() {
            println!("  Fragment {}: {} pipelines", frag.id, frag.pipelines.len());
            for pipe in &frag.pipelines {
                println!("    {}: {} operators", pipe.id, pipe.operators.len());
            }
        }
        
        // Verify execution tree
        assert!(profile.execution_tree.is_some(), "Should have execution tree");
        let tree = profile.execution_tree.unwrap();
        println!("Execution tree has {} nodes", tree.nodes.len());
        
        // Check for known operators
        let has_result_sink = tree.nodes.iter().any(|n| n.operator_name.contains("RESULT_SINK"));
        let has_scan = tree.nodes.iter().any(|n| n.operator_name.contains("SCAN"));
        
        assert!(has_result_sink, "Should have RESULT_SINK operator");
        assert!(has_scan, "Should have SCAN operator");
        
        // Print hotspots
        let hotspots: Vec<_> = tree.nodes.iter().filter(|n| n.is_hotspot).collect();
        println!("Found {} hotspot nodes:", hotspots.len());
        for hs in &hotspots {
            println!("  {} ({:?}): {:.2}%", 
                hs.operator_name, 
                hs.hotspot_severity,
                hs.time_percentage.unwrap_or(0.0)
            );
        }
    }
}

