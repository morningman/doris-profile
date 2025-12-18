use clap::Parser;
use doris_profile_analyzer::{ConfigLoader, AiDiagnosisService};
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(name = "doris-profile-analyzer")]
#[command(author = "Doris Community")]
#[command(version = "0.1.0")]
#[command(about = "Doris Profile Analyzer - Analyze query profiles with embedded web UI", long_about = None)]
struct Args {
    /// Server port
    #[arg(short, long, default_value = "3030")]
    port: u16,

    /// Server host
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("Doris Profile Analyzer v0.1.0");
    
    // Load configurations
    let ai_config = ConfigLoader::load_ai_config()
        .unwrap_or_else(|e| {
            eprintln!("Failed to load AI config: {}, using defaults", e);
            ConfigLoader::default_ai_config()
        });
    
    let default_suggestions = ConfigLoader::load_default_suggestions()
        .unwrap_or_else(|e| {
            eprintln!("Failed to load default suggestions: {}", e);
            panic!("Cannot start without default suggestions config");
        });
    
    println!("AI Diagnosis: {}", if ai_config.ai_diagnosis.enabled { "Enabled" } else { "Disabled" });
    
    // Create AI service
    let ai_service = if ai_config.ai_diagnosis.enabled {
        Some(Arc::new(AiDiagnosisService::new(ai_config)))
    } else {
        None
    };
    
    println!("Starting server on http://{}:{}", args.host, args.port);
    println!("Frontend: http://{}:{}", args.host, args.port);
    println!("API: http://{}:{}/health, /api/analyze, /api/analyze-file", args.host, args.port);
    println!();

    doris_profile_analyzer::api::start_server(
        args.host,
        args.port,
        ai_service,
        Arc::new(default_suggestions),
    ).await;
    Ok(())
}

