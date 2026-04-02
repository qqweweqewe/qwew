use axum::{Extension, Json};
use serde_json::json;
use crate::{handlers::extractors::CurrentUser, ws::tickets::WsTickets};

pub async fn issue_ticket(
    current_user: CurrentUser,
    Extension(tickets): Extension<WsTickets>,
) -> Json<serde_json::Value> {
    let ticket = crate::ws::tickets::issue_ticket(current_user.user_id, current_user.username, &tickets).await;
    Json(json!({ "ticket": ticket }))
}
