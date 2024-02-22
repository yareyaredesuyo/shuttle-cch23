use std::collections::HashMap;

use axum::{response::IntoResponse, routing::get, Json, Router};
use axum_extra::extract::cookie::CookieJar;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

pub fn task() -> Router {
    Router::new()
        .route("/decode", get(decode_route))
        .route("/bake", get(bake_route))
}

async fn decode_route(jar: CookieJar) -> impl IntoResponse {
    let recipe_value: String = jar
        .get("recipe")
        .map_or_else(|| String::new(), |cookie| cookie.value().to_string());

    info!("{:?}", recipe_value);

    let decoded = general_purpose::STANDARD
        .decode(recipe_value)
        .expect("cannot decode");

    let utf8_string = String::from_utf8(decoded).expect("Invalid UTF-8");
    info!("{:?}", utf8_string);

    let json: serde_json::Value =
        serde_json::from_str(&utf8_string).expect("cannot convert to json");

    Json(json)
}

#[derive(Serialize, Deserialize, Debug)]
struct Bake {
    recipe: HashMap<String, i64>,
    pantry: HashMap<String, i64>,
}

async fn bake_route(jar: CookieJar) -> impl IntoResponse {
    let encoded_recipe = jar.get("recipe").unwrap();
    let decoded_recipe = general_purpose::STANDARD
        .decode(encoded_recipe.value())
        .unwrap();

    let mut bake = serde_json::from_slice::<Bake>(&decoded_recipe).unwrap();

    let cookie_count = bake
        .recipe
        .iter()
        .map(|(ingredient, amount_needed)| {
            if amount_needed == &0 {
                i64::MAX
            } else if let Some(amount) = bake.pantry.get(ingredient) {
                amount / amount_needed
            } else {
                0
            }
        })
        .min()
        .unwrap_or(0);

    for (ingredient, amount_in_store) in bake.pantry.iter_mut() {
        if let Some(amount_needed) = bake.recipe.get(ingredient) {
            *amount_in_store -= amount_needed * cookie_count
        }
    }

    Json(json!({"cookies": cookie_count, "pantry": bake.pantry}))
}
