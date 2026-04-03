// INSERT INTO invites (code, created_by, expires_at)
// VALUES ('bootstrap', 1, NOW() + INTERVAL '1 year');
// that's needed

use axum::{Extension, Json, http::StatusCode};
use sqlx::PgPool;
use chrono::{Utc, Duration};
use rand::Rng;
use crate::{handlers::extractors::CurrentUser, models::invite::CreateInviteResponse};

pub async fn create_invite(
    current_user: CurrentUser,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<CreateInviteResponse>, StatusCode> {
    let bytes = rand::thread_rng().gen::<[u8; 8]>();
    let code = bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    let expires_at = Utc::now() + Duration::days(3);

    sqlx::query(
        "INSERT INTO invites (code, created_by, expires_at) VALUES ($1, $2, $3)"
    )
    .bind(&code)
    .bind(current_user.user_id)
    .bind(expires_at)
    .execute(&pool)
    .await
    .map_err(|e| {
        tracing::error!("create_invite: db error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(CreateInviteResponse { code, expires_at }))
}
