use std::collections::HashMap;

use axum::{extract::Path, routing::get, Router};

use reqwest;

use super::error::AppError;
use anyhow;

pub fn task() -> Router {
    Router::new()
        .route("/weight/:id", get(pokedex_route))
        .route("/drop/:id", get(pokedex_drop_route))
}

async fn pokedex_get(id: u32) -> anyhow::Result<HashMap<String, serde_json::Value>> {
    let url = format!("https://pokeapi.co/api/v2/pokemon/{id}");
    let res = reqwest::get(url).await?.json().await?;

    Ok(res)
}

async fn pokedex_route(id: Path<u32>) -> Result<String, AppError> {
    let res = pokedex_get(id.0).await?;
    let weight = res.get("weight").unwrap().as_f64().unwrap();

    Ok(format!("{}", weight / 10.0))
}

async fn pokedex_drop_route(id: Path<u32>) -> Result<String, AppError> {
    const GRAVITY: f64 = 9.825;
    const CHIMNEY_HEIGHT: f64 = 10.0;

    let res = pokedex_get(id.0).await?;
    let weight = res.get("weight").unwrap().as_f64().unwrap();

    let velocity = (2.0 * GRAVITY * CHIMNEY_HEIGHT).sqrt();
    let momentum = weight * velocity / 10.0;

    Ok(momentum.to_string())
}
