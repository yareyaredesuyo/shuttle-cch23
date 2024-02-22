use std::sync::RwLock;

use super::error::AppError;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Datelike, Utc};
use serde_json::json;
use std::time::Instant;
use std::{collections::HashMap, sync::Arc};
use tracing::info;
use ulid::Ulid;
use uuid::Uuid;

type SharedState = Arc<std::sync::RwLock<AppState>>;

#[derive(Default)]
struct AppState {
    state: HashMap<String, Instant>,
}

pub fn task() -> Router {
    let shared_state = SharedState::default();

    Router::new()
        .route("/save/:data", post(save_state_route))
        .route("/load/:data", get(load_state_route))
        .route("/ulids", post(ulids_route))
        .route("/ulids/:weekday", post(ulids_weekday_route))
        .with_state(shared_state)
}

async fn save_state_route(Path(key): Path<String>, State(state): State<Arc<RwLock<AppState>>>) {
    let state = &mut state.write().unwrap().state;
    state.insert(key, Instant::now());
}

async fn load_state_route(Path(id): Path<String>, State(state): State<SharedState>) -> String {
    let state_info = &state.read().unwrap().state;
    let time = state_info.get(&id).unwrap();
    (*time).elapsed().as_secs().to_string()
}

async fn ulids_route(data: Json<Vec<String>>) -> Json<Vec<String>> {
    let ids: Vec<String> = data
        .iter()
        .map(|id| Uuid::from(Ulid::from_string(id).unwrap()).to_string())
        .rev()
        .collect();
    Json(ids)
}

async fn ulids_weekday_route(
    Path(weekday): Path<u32>,
    data: Json<Vec<String>>,
) -> Result<impl IntoResponse, AppError> {
    let mut christmas_eve_count = 0;
    let mut weekday_count = 0;
    let mut future_day_count = 0;
    let mut lsb_count = 0;

    let dates: Vec<DateTime<Utc>> = data
        .iter()
        .map(|id| Ulid::from_string(id).unwrap())
        .inspect(|ulid| lsb_count += (ulid.0 & 1) as u32)
        .map(|ulid| DateTime::<Utc>::from(ulid.datetime()))
        .collect();

    for date in dates {
        info!("{:?}", date);
        if date.month() == 12 && date.day() == 24 {
            christmas_eve_count += 1;
        }
        if date.weekday().num_days_from_monday() == weekday {
            weekday_count += 1;
        }
        if date > Utc::now() {
            future_day_count += 1;
        }
    }

    Ok(Json(json!({
        "christmas eve": christmas_eve_count,
        "weekday": weekday_count,
        "in the future": future_day_count,
        "LSB is 1": lsb_count,
    })))
}
