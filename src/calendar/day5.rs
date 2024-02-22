use axum::{extract::Query, response::IntoResponse, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub fn task() -> Router {
    Router::new().route("/", post(pager_route))
}

#[derive(Debug, Deserialize, Serialize)]
struct Pagination {
    offset: Option<usize>,
    limit: Option<usize>,
    split: Option<usize>,
}

async fn pager_route(
    Query(pagination): Query<Pagination>,
    Json(input): Json<Vec<String>>,
) -> impl IntoResponse {
    let offset = pagination.offset.unwrap_or(0);

    let data = if let Some(limit) = pagination.limit {
        input[offset..(offset + limit).min(input.len())].to_vec()
    } else {
        input[offset..].to_vec()
    };

    if let Some(split_size) = pagination.split {
        Json(json!(data.chunks(split_size).collect::<Vec<_>>()))
    } else {
        Json(json!(data.to_vec()))
    }
}
