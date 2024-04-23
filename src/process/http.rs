use anyhow::Result;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::services::ServeDir;
use tracing::{info, warn};

#[derive(Debug)]
struct HttpServeState {
    path: PathBuf,
}

pub async fn process_http_serve(path: PathBuf, port: u16) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("serving {:?} on port {}", path, port);

    let state = HttpServeState { path: path.clone() };

    // let dir_service = ServeDir::new(path)
    //     .append_index_html_on_directories(true)
    //     .precompressed_gzip()
    //     .precompressed_br()
    //     .precompressed_deflate()
    //     .precompressed_zstd();

    let router = Router::new()
        .route("/*path", get(file_handler))
        .nest_service("/tower", ServeDir::new(path))
        .with_state(Arc::new(state));

    let lister = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(lister, router).await?;

    Ok(())
}

async fn file_handler(
    State(state): State<Arc<HttpServeState>>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let p = std::path::Path::new(&state.path).join(path);
    info!("reading file {:?}", p);
    if !p.exists() {
        return Ok((
            StatusCode::NOT_FOUND,
            Html(format!("file not found: {:?}", p.display())),
        ));
    }

    if p.is_dir() {
        let dirs = p.read_dir().unwrap();
        let mut files: Vec<String> = vec![];
        for entry in dirs.flatten() {
            let file_path = entry.path();
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            let file_path_str = file_path.to_string_lossy();
            let html = format!(
                r#"<li><a href="{}">{}</a></li>"#,
                file_path_str.trim_start_matches('.'),
                file_name_str
            );
            files.push(html);
        }

        let body = files.join("\n");

        return Ok((
            StatusCode::OK,
            Html(format!(
                r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>files</title>
        </head>
        <body>
        <div><ul>{}</ul></div>
        </body>
        </html>"#,
                body
            )),
        ));
    }

    match tokio::fs::read_to_string(p).await {
        Ok(content) => {
            info!("Read {} bytes", content.len());
            Ok((StatusCode::OK, Html(content)))
        }
        Err(e) => {
            warn!("Error reading file: {:?}", e);
            Err((
                StatusCode::NOT_FOUND,
                Html(format!("File read error: {}", e)),
            ))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_file_handle() {
        let state = Arc::new(HttpServeState {
            path: PathBuf::from("."),
        });
        let resp = file_handler(State(state), Path("Cargo.toml".to_string())).await;
        let resp = resp.into_response();

        assert_eq!(resp.status(), StatusCode::OK);
    }
}
