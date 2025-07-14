use axum::{
    extract::ws::{WebSocketUpgrade, WebSocket, Message},
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{StreamExt, SinkExt};
use std::{
    net::SocketAddr,
    sync::Arc,
};
use tokio::sync::broadcast;
use serde::{Serialize, Deserialize};

// Type alias for the broadcast channel sender
// We'll use this to broadcast messages to all clients
// Each client gets a receiver (subscriber)
type Tx = broadcast::Sender<String>;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum WhiteboardEvent {
    DrawFreehand { x: f64, y: f64, dragging: bool },
    DrawLine { from: (f64, f64), to: (f64, f64), color: String, width: f64 },
    DrawRect { from: (f64, f64), to: (f64, f64), color: String, width: f64 },
    DrawCircle { center: (f64, f64), radius: f64, color: String, width: f64 },
    AddText { pos: (f64, f64), text: String, color: String, size: f64 },
    Pan { dx: f64, dy: f64 },
    Zoom { factor: f64 },
}

#[tokio::main]
async fn main() {
    // Create a broadcast channel for drawing events
    let (tx, _rx) = broadcast::channel::<String>(100);
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(Arc::new(tx));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Backend running at ws://{}", addr);
    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app
    )
    .await
    .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(tx): State<Arc<Tx>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, tx))
}

async fn handle_socket(socket: WebSocket, tx: Arc<Tx>) {
    let mut rx = tx.subscribe();
    let (mut sender, mut receiver) = socket.split();

    // Task to forward broadcast messages to this client
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Receive messages from this client and broadcast them
    while let Some(Ok(Message::Text(msg))) = receiver.next().await {
        let _ = tx.send(msg);
    }

    send_task.abort();
}
