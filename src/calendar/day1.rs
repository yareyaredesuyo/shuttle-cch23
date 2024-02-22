use axum::{extract::Path, routing::get, Router};

pub fn task() -> Router {
    Router::new().route("/*params", get(sled))
}

async fn sled(params: Path<String>) -> String {
    params
        .split('/')
        .filter_map(|s| s.parse().ok())
        .fold(0, |acc, x: i32| acc ^ x)
        .pow(3)
        .to_string()
}
