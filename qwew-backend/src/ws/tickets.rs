use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use chrono::{Utc, Duration};
use uuid::Uuid;

#[derive(Clone)]
pub struct Ticket {
    pub user_id: i64,
    pub username: String,
    pub expires_at: chrono::DateTime<Utc>,
}

pub type WsTickets = Arc<RwLock<HashMap<String, Ticket>>>;

pub async fn issue_ticket(user_id: i64, username: String, tickets: &WsTickets) -> String {
    let key = Uuid::new_v4().to_string();
    let ticket = Ticket {
        user_id,
        username,
        expires_at: Utc::now() + Duration::seconds(30),
    };
    tickets.write().await.insert(key.clone(), ticket);
    key
}

// consumes the ticket — one-time use, lazy expiry cleanup
pub async fn redeem_ticket(key: &str, tickets: &WsTickets) -> Option<Ticket> {
    let mut map = tickets.write().await;
    let ticket = map.remove(key)?;
    if Utc::now() > ticket.expires_at {
        return None;
    }
    // lazy cleanup of other expired tickets while we have the write lock
    map.retain(|_, t| Utc::now() <= t.expires_at);
    Some(ticket)
}
