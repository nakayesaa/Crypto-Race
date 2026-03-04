use std::sync::{Arc, Mutex};
use axum::extract::ws::Message;
use axum::{
    Router,
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    response::IntoResponse,
    routing::get,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tower_http::cors::{Any, CorsLayer};

pub type ClientRegistry = Arc<Mutex<Vec<mpsc::UnboundedSender<String>>>>;

async fn ws_handler(
    web_socket: WebSocketUpgrade,
    State(registry): State<ClientRegistry>,
) -> impl IntoResponse {
    web_socket.on_upgrade(|socket| handle_client(socket, registry))
}

async fn handle_client(socket: WebSocket, registry: ClientRegistry) {
    tracing::info!("WebSocket client connected");

    let (mut sink, mut stream) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    registry.lock().unwrap().push(tx);
    let forward_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sink
                .send(Message::Text(msg.into()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    while let Some(result) = stream.next().await {
        if result.is_err() {
            break;
        }
    }

    forward_task.abort();
    tracing::info!("WebSocket client disconnected");
}

pub fn create_router(registry: ClientRegistry) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/ws", get(ws_handler))
        .layer(cors)
        .with_state(registry)
}
