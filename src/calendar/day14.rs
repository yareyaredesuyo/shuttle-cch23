use axum::{response::IntoResponse, routing::post, Json, Router};
use html_escape;
use serde::Deserialize;

pub fn task() -> Router {
    Router::new()
        .route("/unsafe", post(unsafe_route))
        .route("/safe", post(safe_route))
}

#[derive(Deserialize, Debug)]
struct Content {
    content: String,
}

async fn unsafe_route(req: Json<Content>) -> impl IntoResponse {
    format!(
        r#"<html>
  <head>
    <title>CCH23 Day 14</title>
  </head>
  <body>
    {}
  </body>
</html>"#,
        req.content
    )
}

async fn safe_route(req: Json<Content>) -> impl IntoResponse {
    format!(
        r#"<html>
  <head>
    <title>CCH23 Day 14</title>
  </head>
  <body>
    {}
  </body>
</html>"#,
        html_escape::encode_double_quoted_attribute(&req.content)
    )
}
