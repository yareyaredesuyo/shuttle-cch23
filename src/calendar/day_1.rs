use axum::{http::StatusCode, routing::get, Router};

pub fn task() -> Router {
    Router::new()
        .route("/", get(hello_world))
        .route("/-1/error", get(internal_server_error))
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn internal_server_error() -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}
