use axum::{http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

pub fn task() -> Router {
    Router::new()
        .route("/strength", post(strength_route))
        .route("/contest", post(contest_route))
}

#[derive(Debug, Deserialize, Serialize)]
struct Reindeer {
    name: String,
    strength: u32,
    #[serde(default)]
    speed: f64,
    #[serde(default)]
    height: u32,
    #[serde(default)]
    antler_width: u32,
    #[serde(default)]
    snow_magic_power: u32,
    #[serde(default)]
    favorite_food: String,
    #[serde(default)]
    #[serde(rename = "cAnD13s_3ATeN-yesT3rdAy")]
    candies: u32,
}

async fn strength_route(Json(payload): Json<Vec<Reindeer>>) -> String {
    let total_strength: u32 = payload.iter().map(|reindeer| reindeer.strength).sum();
    total_strength.to_string()
}

#[derive(Debug, Serialize)]
struct ContestResult {
    fastest: String,
    tallest: String,
    magician: String,
    consumer: String,
}

async fn contest_route(
    Json(payload): Json<Vec<Reindeer>>,
) -> Result<Json<ContestResult>, (StatusCode, &'static str)> {
    if payload.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No reindeer data provided"));
    }

    let fastest_reindeer = payload
        .iter()
        .max_by(|a, b| {
            a.speed
                .partial_cmp(&b.speed)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap();

    let tallest_reindeer = payload
        .iter()
        .max_by(|a, b| a.height.cmp(&b.height))
        .unwrap();

    let magician_reindeer = payload
        .iter()
        .max_by(|a, b| a.snow_magic_power.cmp(&b.snow_magic_power))
        .unwrap();

    let consumer_reindeer = payload
        .iter()
        .max_by(|a, b| a.candies.cmp(&b.candies))
        .unwrap();

    let result = ContestResult {
        fastest: format!(
            "Speeding past the finish line with a strength of {} is {}",
            fastest_reindeer.strength, fastest_reindeer.name
        ),
        tallest: format!(
            "{} is standing tall with his {} cm wide antlers",
            tallest_reindeer.name, tallest_reindeer.antler_width
        ),
        magician: format!(
            "{} could blast you away with a snow magic power of {}",
            magician_reindeer.name, magician_reindeer.snow_magic_power
        ),
        consumer: format!(
            "{} ate lots of candies, but also some {}",
            consumer_reindeer.name, consumer_reindeer.favorite_food
        ),
    };

    Ok(Json(result))
}
