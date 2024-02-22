use axum::{response::IntoResponse, routing::post, Json, Router};
use serde_json::json;
use tracing::info;

pub fn task() -> Router {
    Router::new().route("/", post(elf_route))
}

async fn elf_route(text: String) -> impl IntoResponse {
    let elf_count = text.matches("elf").count();

    let elf_on_a_shelf = b"elf on a shelf";

    let elf_on_a_shelf_count = text
        .as_bytes()
        .windows(elf_on_a_shelf.len())
        .inspect(|&x| {
            info!("Inspecting element: {:?}", String::from_utf8_lossy(x));
        })
        .filter(|w| w == elf_on_a_shelf)
        .count();

    Json(json!({
        "elf": elf_count,
        "elf on a shelf": elf_on_a_shelf_count,
        "shelf with no elf on it": text.matches("shelf").count() - elf_on_a_shelf_count
    }))
}
