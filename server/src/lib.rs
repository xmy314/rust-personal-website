use axum::body::Body;
use axum::http::{Response, StatusCode};
use axum::{response::IntoResponse, routing::get, Router};
use clap::Parser;
use std::path::PathBuf;
use tokio::fs;
use tower::{ServiceBuilder, ServiceExt};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

// Setup the command line interface with clap.
#[derive(Parser, Debug)]
#[clap(name = "server", about = "A server for our wasm project!")]
pub struct Opt {
    /// set the log level
    #[clap(short = 'l', long = "log", default_value = "debug")]
    pub log_level: String,

    /// set the listen addr
    #[clap(short = 'a', long = "addr", default_value = "::1")]
    pub addr: String,

    /// set the listen port
    #[clap(short = 'p', long = "port", default_value = "8080")]
    pub port: u16,

    /// set the directory where static files are to be found
    #[clap(long = "static-dir", default_value = "./dist")]
    pub static_dir: String,
}

pub async fn setup_app(static_dir: String) -> Router {
    let app = Router::new()
        .route("/api/hello/", get(hello))
        .fallback_service(get(|req: axum::http::Request<Body>| async move {
            match ServeDir::new(&static_dir).oneshot(req).await {
                Ok(res) => {
                    let status = res.status();
                    match status {
                        StatusCode::NOT_FOUND => {
                            let index_path = PathBuf::from(&static_dir).join("index.html");
                            let index_content = match fs::read_to_string(index_path).await {
                                Err(_) => {
                                    let pwd = std::env::current_dir().unwrap();
                                    return Response::builder()
                                        .status(StatusCode::NOT_FOUND)
                                        .body(Body::from(format!(
                                            "index file not found at {}",
                                            pwd.display()
                                        )))
                                        .unwrap();
                                }
                                Ok(index_content) => index_content,
                            };

                            Response::builder()
                                .status(StatusCode::OK)
                                .body(Body::from(index_content))
                                .unwrap()
                        }
                        _ => res.map(Body::new),
                    }
                }
                Err(err) => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(format!("error: {err}")))
                    .expect("error response"),
            }
        }))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    app
}

async fn hello() -> impl IntoResponse {
    "hello from server +!"
}
