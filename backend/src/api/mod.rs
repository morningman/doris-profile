use warp::Filter;
use serde_json::json;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use crate::static_files::StaticFiles;
use crate::{AiDiagnosisService, ProfileComposer, HotSpotDetector, SuggestionEngine};
use crate::config::DefaultSuggestionsConfig;

#[derive(Deserialize)]
struct AnalyzeRequest {
    profile_text: String,
}

#[derive(Serialize)]
struct AnalyzeResponse {
    success: bool,
    error: Option<String>,
    data: Option<crate::models::ProfileAnalysisResponse>,
}

#[derive(Deserialize)]
struct DiagnoseNodeRequest {
    profile_text: String,
    node_id: String,
    #[serde(default = "default_language")]
    language: String,
}

fn default_language() -> String {
    "en".to_string()
}

#[derive(Serialize)]
struct DiagnoseNodeResponse {
    success: bool,
    error: Option<String>,
    suggestion: Option<String>,
    suggestion_source: Option<String>,
}

#[derive(Clone)]
struct AppState {
    ai_service: Option<Arc<AiDiagnosisService>>,
    default_config: Arc<DefaultSuggestionsConfig>,
}

pub async fn start_server(
    host: String,
    port: u16,
    ai_service: Option<Arc<AiDiagnosisService>>,
    default_config: Arc<DefaultSuggestionsConfig>,
) {
    let app_state = Arc::new(AppState {
        ai_service,
        default_config,
    });
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST"]);

    // Health check endpoint
    let health = warp::path("health")
        .and(warp::get())
        .map(|| warp::reply::json(&json!({"status": "ok"})));

    let state_filter = warp::any().map(move || app_state.clone());
    
    // Analyze profile from JSON body
    let analyze_profile_json = warp::path("api")
        .and(warp::path("analyze"))
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 1024 * 50))
        .and(warp::body::json())
        .and(state_filter.clone())
        .and_then(handle_analyze_profile);

    // Analyze profile from file upload
    use crate::constants::file_limits;
    let analyze_profile_file = warp::path("api")
        .and(warp::path("analyze-file"))
        .and(warp::post())
        .and(warp::body::content_length_limit(file_limits::MAX_UPLOAD_SIZE))
        .and(warp::multipart::form().max_length(file_limits::MAX_UPLOAD_SIZE))
        .and(state_filter.clone())
        .and_then(handle_analyze_profile_file);

    // Diagnose single node with AI
    let diagnose_node = warp::path("api")
        .and(warp::path("diagnose-node"))
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 1024 * 50))
        .and(warp::body::json())
        .and(state_filter.clone())
        .and_then(handle_diagnose_node);

    // API routes
    let api_routes = health
        .or(analyze_profile_json)
        .or(analyze_profile_file)
        .or(diagnose_node);

    // Static file serving for frontend
    let static_routes = warp::get()
        .and(warp::path::tail())
        .and_then(serve_static);

    // Combine all routes
    let routes = api_routes
        .or(static_routes)
        .with(cors)
        .with(warp::log("api"));

    let addr: std::net::IpAddr = host.parse().unwrap_or_else(|_| {
        eprintln!("Invalid host address, using 0.0.0.0");
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0))
    });

    warp::serve(routes)
        .run((addr, port))
        .await;
}

async fn handle_analyze_profile(
    req: AnalyzeRequest,
    state: Arc<AppState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match analyze_profile_with_ai(&req.profile_text, &state).await {
        Ok(result) => {
            let response = AnalyzeResponse {
                success: true,
                error: None,
                data: Some(result),
            };
            Ok(warp::reply::json(&response))
        }
        Err(err) => {
            let response = AnalyzeResponse {
                success: false,
                error: Some(err),
                data: None,
            };
            Ok(warp::reply::json(&response))
        }
    }
}

async fn analyze_profile_with_ai(
    profile_text: &str,
    state: &AppState,
) -> Result<crate::models::ProfileAnalysisResponse, String> {
    // 1. Parse profile
    let mut composer = ProfileComposer::new();
    let profile = composer.parse(profile_text)
        .map_err(|e| format!("Failed to parse profile: {:?}", e))?;
    
    // 2. Detect hotspots
    let mut hotspots = HotSpotDetector::analyze(&profile);
    
    // 3. Fill suggestions with default config only (skip AI for initial load)
    SuggestionEngine::fill_suggestions(
        &mut hotspots,
        &profile,
        state.ai_service.as_deref(),
        &state.default_config,
        true,  // skip_ai = true for initial analysis
    ).await;
    
    // 4. Generate conclusion and score
    let conclusion = SuggestionEngine::generate_conclusion(&hotspots, &profile);
    let suggestions = SuggestionEngine::generate_suggestions(&hotspots);
    let performance_score = SuggestionEngine::calculate_performance_score(&hotspots, &profile);
    let execution_tree = profile.execution_tree.clone();
    let summary = Some(profile.summary.clone());
    
    Ok(crate::models::ProfileAnalysisResponse {
        hotspots,
        conclusion,
        suggestions,
        performance_score,
        execution_tree,
        summary,
    })
}

async fn handle_analyze_profile_file(
    mut form: warp::multipart::FormData,
    state: Arc<AppState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    use futures::TryStreamExt;
    use bytes::Buf;
    
    let mut profile_text = String::new();
    
    while let Some(part) = form.try_next().await.map_err(|_| warp::reject::reject())? {
        if part.name() == "file" {
            let data = part.stream().try_fold(Vec::new(), |mut acc, chunk| async move {
                let chunk_bytes = chunk.chunk();
                acc.extend_from_slice(chunk_bytes);
                Ok(acc)
            }).await.map_err(|_| warp::reject::reject())?;
            
            profile_text = String::from_utf8(data).map_err(|_| warp::reject::reject())?;
            break;
        }
    }
    
    if profile_text.is_empty() {
        return Ok(warp::reply::json(&json!({
            "success": false,
            "error": "No file provided",
            "data": null
        })));
    }
    
    match analyze_profile_with_ai(&profile_text, &state).await {
        Ok(result) => {
            let response = AnalyzeResponse {
                success: true,
                error: None,
                data: Some(result),
            };
            Ok(warp::reply::json(&response))
        }
        Err(err) => {
            let response = AnalyzeResponse {
                success: false,
                error: Some(format!("Failed to parse profile: {}", err)),
                data: None,
            };
            Ok(warp::reply::json(&response))
        }
    }
}

async fn handle_diagnose_node(
    req: DiagnoseNodeRequest,
    state: Arc<AppState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match diagnose_single_node(&req.profile_text, &req.node_id, &req.language, &state).await {
        Ok((suggestion, source)) => {
            let response = DiagnoseNodeResponse {
                success: true,
                error: None,
                suggestion: Some(suggestion),
                suggestion_source: Some(source),
            };
            Ok(warp::reply::json(&response))
        }
        Err(err) => {
            let response = DiagnoseNodeResponse {
                success: false,
                error: Some(err),
                suggestion: None,
                suggestion_source: None,
            };
            Ok(warp::reply::json(&response))
        }
    }
}

async fn diagnose_single_node(
    profile_text: &str,
    node_id: &str,
    language: &str,
    state: &AppState,
) -> Result<(String, String), String> {
    // 1. Parse profile
    let mut composer = ProfileComposer::new();
    let profile = composer.parse(profile_text)
        .map_err(|e| format!("Failed to parse profile: {:?}", e))?;
    
    // 2. Find the node in execution tree
    let tree = profile.execution_tree.as_ref()
        .ok_or_else(|| "No execution tree found".to_string())?;
    
    let node = tree.nodes.iter()
        .find(|n| n.id == node_id)
        .ok_or_else(|| format!("Node {} not found", node_id))?;
    
    // 3. Try to get AI suggestion
    if let Some(ref ai_service) = state.ai_service {
        if ai_service.is_enabled() {
            match ai_service.generate_suggestion_with_language(node, &profile, language).await {
                Ok(suggestion) => {
                    return Ok((suggestion, "ai".to_string()));
                }
                Err(e) => {
                    let error_msg = format!("AI Suggestion failed: {}", e);
                    eprintln!("{} for node {}", error_msg, node_id);
                    // Fall through to default suggestion
                }
            }
        }
    }
    
    // 4. Use default suggestion as fallback
    // Find corresponding hotspot to get operator name and severity
    let hotspots = HotSpotDetector::analyze(&profile);
    if let Some(hotspot) = hotspots.iter().find(|h| h.node_id == node_id) {
        let default_suggestion = SuggestionEngine::get_default_suggestion_public(
            &hotspot.operator_name,
            &hotspot.severity,
            &state.default_config,
        );
        Ok((default_suggestion, "AI Suggestion is not enabled".to_string()))
    } else {
        Err(format!("Hotspot for node {} not found", node_id))
    }
}

async fn serve_static(path: warp::path::Tail) -> Result<impl warp::Reply, warp::Rejection> {
    let path_str = path.as_str();
    
    // Default to index.html for root path
    let file_path = if path_str.is_empty() || path_str == "/" {
        "index.html"
    } else {
        path_str.trim_start_matches('/')
    };
    
    tracing::debug!("Requesting static file: {}", file_path);
    
    match StaticFiles::get(file_path) {
        Some(content) => {
            let mime = mime_guess::from_path(file_path).first_or_octet_stream();
            tracing::debug!("Serving {} with mime type {}", file_path, mime.as_ref());
            
            Ok(warp::reply::with_header(
                warp::reply::with_header(
                    content.data.into_owned(),
                    "content-type",
                    mime.as_ref(),
                ),
                "cache-control",
                "public, max-age=3600",
            ))
        }
        None => {
            // Fallback to index.html for SPA routing
            match StaticFiles::get("index.html") {
                Some(content) => {
                    Ok(warp::reply::with_header(
                        warp::reply::with_header(
                            content.data.into_owned(),
                            "content-type",
                            "text/html",
                        ),
                        "cache-control",
                        "no-cache",
                    ))
                }
                None => Err(warp::reject::not_found()),
            }
        }
    }
}

