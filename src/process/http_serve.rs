use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use tower_http::services::ServeDir;
use tracing::{info, warn};

#[derive(Debug)]
struct HttpServeState {
    path: PathBuf,
}

pub async fn process_http_serve(path: PathBuf, port: u16) -> Result<()> {
    info!("Serving {:?} on port {}", path, port);

    let state = HttpServeState { path: path.clone() };

    let router = Router::new()
        .nest_service("/tower", ServeDir::new(path))
        .route("/", get(index_handler))
        .route("/*path", get(file_handler))
        .with_state(Arc::new(state));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}

async fn file_handler(
    State(state): State<Arc<HttpServeState>>,
    Path(path): Path<String>,
) -> Response {
    let p = std::path::Path::new(&state.path).join(path);
    info!("Reading file {:?}", p);
    if !p.exists() {
        (
            StatusCode::NOT_FOUND,
            format!("File {:?} not found", p.display()),
        )
            .into_response()
    } else {
        // TODO: test p is a directory
        // if it is a directory, list all files/subdirectories
        // as <li><a href="/path/to/file">file name</a></li>
        // <html><body><ul>...</ul></body></html>
        if p.is_dir() {
            info!("Reading directory {:?}", p);
            let mut content = String::new();
            content.push_str("<html><body><ul>");
            for entry in std::fs::read_dir(p).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                let name = entry.file_name();
                content.push_str(&format!(
                    r#"<li><a href="/{}">{}</a></li>"#,
                    path.display(),
                    name.to_string_lossy()
                ));
            }
            content.push_str("</ul></body></html>");
            Html(content).into_response()
        } else {
            match tokio::fs::read_to_string(p).await {
                Ok(content) => {
                    info!("Read {} bytes", content.len());
                    (StatusCode::OK, content).into_response()
                }
                Err(e) => {
                    warn!("Error reading file: {:?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
                }
            }
        }
    }
}

async fn index_handler(State(state): State<Arc<HttpServeState>>) -> Response {
    file_handler(State(state), Path("".into())).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_handler() {
        let state = Arc::new(HttpServeState {
            path: PathBuf::from("."),
        });

        let res = file_handler(State(state), Path("Cargo.toml".to_string())).await;
        let status = res.status();
        assert_eq!(status, StatusCode::OK);
    }
}
