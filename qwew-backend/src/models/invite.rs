use serde::Serialize;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize)]
pub struct CreateInviteResponse {
    pub code: String,
    pub expires_at: DateTime<Utc>,
}
