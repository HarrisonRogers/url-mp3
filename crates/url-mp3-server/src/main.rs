use axum::{
    body::Body,
    extract::Query,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio_stream::wrappers::ReceiverStream;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use url_mp3_core::{download_mp3, ensure_binaries, get_default_output_dir, get_libs_dir, is_youtube_url};

#[derive(Deserialize)]
struct DownloadQuery {
    url: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

async fn download_handler(
    Query(query): Query<DownloadQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let libs_dir = get_libs_dir();
    let output_dir = get_default_output_dir();

    // Validate URL
    if !is_youtube_url(&query.url) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid YouTube URL".to_string(),
            }),
        ));
    }

    // Download the file
    let result = download_mp3(&libs_dir, &output_dir, &query.url).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let filename = result
        .path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "download.mp3".to_string());

    // Stream the file
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<axum::body::Bytes, std::io::Error>>(32);
    let filepath = result.path.clone();

    tokio::spawn(async move {
        use tokio::io::AsyncReadExt;

        let mut file = match tokio::fs::File::open(&filepath).await {
            Ok(f) => f,
            Err(_) => return,
        };

        let mut buffer = vec![0u8; 8192];
        loop {
            match file.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => {
                    if tx
                        .send(Ok(axum::body::Bytes::copy_from_slice(&buffer[..n])))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let stream = ReceiverStream::new(rx);
    let body = Body::from_stream(stream);

    Ok((
        [
            (header::CONTENT_TYPE, "audio/mpeg"),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", filename).leak() as &'static str,
            ),
        ],
        body,
    ))
}

async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "url_mp3_server=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let libs_dir = get_libs_dir();
    let output_dir = get_default_output_dir();

    std::fs::create_dir_all(&output_dir)?;

    tracing::info!("Checking for yt-dlp and ffmpeg binaries...");
    ensure_binaries(&libs_dir).await?;
    tracing::info!("Ready!");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/download", get(download_handler))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    tracing::info!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
