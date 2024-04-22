use anyhow::Result;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
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
            format!("file not found: {:?}", p.display()),
        ));
        // return (
        //     StatusCode::NOT_FOUND,
        //     format!("file not found: {:?}", p.display()),
        // );
    }

    // TODO: test p is a directory
    // if it is a directory, list all files/subdirectories
    // as <li><a href="/path/to/file">file name</a></li>
    // <html><body><ul>...</ul></body></html>

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
                file_path_str, file_name_str
            );
            files.push(html);
        }

        let body = files.join("\n");
        let html = format!(
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
        );

        // let html = Html::from(format!(
        //     r#"<!DOCTYPE html>
        // <html lang="en">
        // <head>
        //     <meta charset="UTF-8">
        //     <title>files</title>
        // </head>
        // <body>
        // <div><ul>{}</ul></div>
        // </body>
        // </html>"#,
        //     body
        // ));

        return Ok((
            StatusCode::OK,
            Response::builder()
                .header("Content-Type", "text/html")
                .body(html)
                .unwrap()
                .into_body(),
        ));
    }

    // let mut response = Response::builder();
    // .status(200)
    // .header("X-Custom-Foo", "Bar")
    // .body(())
    // .unwrap();

    match tokio::fs::read_to_string(p).await {
        Ok(content) => {
            info!("Read {} bytes", content.len());
            Ok((StatusCode::OK, content))
            // (StatusCode::OK, content)
        }
        Err(e) => {
            warn!("Error reading file: {:?}", e);
            Err((StatusCode::NOT_FOUND, format!("File not found: {}", e)))
            // (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    }
}

#[cfg(test)]
mod test {
    // use super::*;
    //
    // #[tokio::test]
    // async fn test_file_handle() {
    //     // let state = Arc::new(HttpServeState {
    //     //     path: PathBuf::from("."),
    //     // });
    //     // let (status, content) = file_handler(State(state), Path("Cargo.toml".to_string())).await;
    //     //
    //     // assert_eq!(status, StatusCode::OK);
    //     // assert!(content.trim().starts_with("[package]"));
    // }
}
