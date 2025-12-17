use warp::Filter;
use serde_json::json;
use serde::{Serialize, Deserialize};
use crate::static_files::StaticFiles;

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

pub async fn start_server(host: String, port: u16) {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST"]);

    // Health check endpoint
    let health = warp::path("health")
        .and(warp::get())
        .map(|| warp::reply::json(&json!({"status": "ok"})));

    // Analyze profile from JSON body
    let analyze_profile_json = warp::path("api")
        .and(warp::path("analyze"))
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 1024 * 50))
        .and(warp::body::json())
        .and_then(handle_analyze_profile);

    // Analyze profile from file upload
    use crate::constants::file_limits;
    let analyze_profile_file = warp::path("api")
        .and(warp::path("analyze-file"))
        .and(warp::post())
        .and(warp::body::content_length_limit(file_limits::MAX_UPLOAD_SIZE))
        .and(warp::multipart::form().max_length(file_limits::MAX_UPLOAD_SIZE))
        .and_then(handle_analyze_profile_file);

    // API routes
    let api_routes = health
        .or(analyze_profile_json)
        .or(analyze_profile_file);

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

async fn handle_analyze_profile(req: AnalyzeRequest) -> Result<impl warp::Reply, warp::Rejection> {
    match crate::analyze_profile(&req.profile_text) {
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

async fn handle_analyze_profile_file(mut form: warp::multipart::FormData) -> Result<impl warp::Reply, warp::Rejection> {
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
    
    match crate::analyze_profile(&profile_text) {
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

