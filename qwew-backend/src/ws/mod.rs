pub mod tickets;

use std::{collections::HashMap, sync::Arc};
use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Query},
    response::Response,
    Extension, http::StatusCode,
};
use tokio::sync::{mpsc, RwLock};
use futures_util::{StreamExt, SinkExt};
use serde::Deserialize;
use tickets::WsTickets;

pub type ConnectedUsers = Arc<RwLock<HashMap<i64, mpsc::UnboundedSender<Message>>>>;

#[derive(Deserialize)]
pub struct WsQuery {
    ticket: String,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    Extension(tickets): Extension<WsTickets>,
    Extension(users): Extension<ConnectedUsers>,
) -> Result<Response, StatusCode> {
    let ticket = tickets::redeem_ticket(&query.ticket, &tickets).await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, ticket.user_id, ticket.username, users)))
}

async fn handle_socket(socket: WebSocket, user_id: i64, username: String, users: ConnectedUsers) {
    let (mut sink, mut stream) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    users.write().await.insert(user_id, tx);
    tracing::info!("{} (id={}) connected", username, user_id);

    // forward outgoing messages to the socket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sink.send(msg).await.is_err() {
                break;
            }
        }
    });

    // read incoming messages (placeholder — will handle chat here later)
    while let Some(Ok(_msg)) = stream.next().await {}

    send_task.abort();
    users.write().await.remove(&user_id);
    tracing::info!("{} (id={}) disconnected", username, user_id);
}
