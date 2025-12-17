use clap::Parser;

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
    println!("Starting server on http://{}:{}", args.host, args.port);
    println!("Frontend: http://{}:{}", args.host, args.port);
    println!("API: http://{}:{}/health, /api/analyze, /api/analyze-file", args.host, args.port);
    println!();

    doris_profile_analyzer::api::start_server(args.host, args.port).await;
    Ok(())
}

