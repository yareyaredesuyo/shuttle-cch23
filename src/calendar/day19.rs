use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, RwLock,
    },
};

use axum::{
    extract::{
        ws::{Message::Text, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};

use tokio::sync::broadcast::{self, Sender};

use anyhow;

use super::error::AppError;

use futures_util::{sink::SinkExt, stream::StreamExt};

use tracing::warn;

pub fn task() -> Router {
    Router::new()
        .route("/ws/ping", get(ping_route))
        .route("/reset", post(reset_route))
        .route("/views", get(views_route))
        .route("/ws/room/:room/user/:user", get(tweet_route))
        .with_state(TwitterState {
            views: Arc::new(AtomicU64::new(0)),
            rooms: Arc::new(RwLock::new(HashMap::new())),
        })
}

async fn ping_route(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(ping_handler)
}

async fn ping_handler(mut ws: WebSocket) {
    let mut served = false;
    while let Some(Ok(msg)) = ws.recv().await {
        if let Ok(msg) = msg.to_text() {
            match msg {
                "serve" => served = true,
                "ping" if served => {
                    let _ = ws.send("pong".into()).await;
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct TweetInput {
    message: String,
}

impl TryFrom<&String> for TweetInput {
    type Error = AppError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let tweet_input = serde_json::from_str::<Self>(value)
            .map_err(|e| anyhow::anyhow!("Error parsing TweetInput: {}", e))?;

        if tweet_input.message.len() > 128 {
            return Err(anyhow::anyhow!("Message length cannot be over 128").into());
        }

        Ok(tweet_input)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Tweet {
    user: String,
    message: TweetInput,
}

impl From<Tweet> for String {
    fn from(value: Tweet) -> Self {
        format!(
            r#"{{
      "user": "{}",
      "message": "{}"
    }}"#,
            value.user, value.message.message
        )
    }
}

#[derive(Debug)]
struct RoomState {
    sender: Sender<Tweet>,
}

#[derive(Clone, Debug)]
struct TwitterState {
    views: Arc<AtomicU64>,
    rooms: Arc<RwLock<HashMap<i32, RoomState>>>,
}

impl RoomState {
    fn new() -> Self {
        Self {
            sender: broadcast::channel(100).0,
        }
    }
}

async fn reset_route(State(state): State<TwitterState>) {
    state.views.store(0, Ordering::Relaxed);
}

async fn views_route(State(state): State<TwitterState>) -> String {
    state.views.load(Ordering::Relaxed).to_string()
}

async fn tweet_route(
    ws: WebSocketUpgrade,
    Path((room, user)): Path<(i32, String)>,
    State(state): State<TwitterState>,
) -> Response {
    ws.on_upgrade(move |c| handle_tweet(c, room, user, Arc::new(state)))
}

async fn handle_tweet(socket: WebSocket, room: i32, user: String, state: Arc<TwitterState>) {
    // socket.
    let (mut sender, mut receiver) = socket.split();
    #[allow(unused_assignments)]
    let mut room_sender = None::<Sender<Tweet>>;

    if let Some(room_state) = state.rooms.read().unwrap().get(&room) {
        room_sender = Some(room_state.sender.clone());
    }

    if room_sender.is_none() {
        let mut rooms = state.rooms.write().unwrap();
        let room_state = rooms.entry(room).or_insert_with(RoomState::new);
        room_sender = Some(room_state.sender.clone());
    }

    let room_sender = room_sender.unwrap();
    let mut room_receiver = room_sender.subscribe();

    let mut send = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            let Ok(msg) = msg else {
                return;
            };

            if let Text(text) = &msg {
                match TweetInput::try_from(text) {
                    Ok(message) => {
                        // info!("Parsed {:?}", message);
                        let user = user.clone();
                        let _ = room_sender.send(Tweet { user, message }).unwrap();
                    }
                    Err(_) => warn!("Failed to parse TweetInput"),
                }
            }
        }
    });

    let mut receive = tokio::spawn(async move {
        while let Ok(msg) = room_receiver.recv().await {
            let msg_str: String = msg.into();
            let _ = state.views.fetch_add(1, Ordering::Relaxed);
            sender.send(Text(msg_str)).await.unwrap();
        }
    });

    tokio::select! {
        _ = (&mut send) => receive.abort(),
        _ = (&mut receive) => send.abort(),
    };
}
