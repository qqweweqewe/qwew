use axum::{extract::Query, Extension, Json, http::StatusCode};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::handlers::extractors::CurrentUser;

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
}

#[derive(Serialize, FromRow)]
pub struct UserResult {
    pub user_id: i64,
    pub username: String,
}

pub async fn search_users(
    current_user: CurrentUser,
    Query(query): Query<SearchQuery>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<UserResult>>, StatusCode> {
    if query.q.trim().is_empty() {
        return Ok(Json(vec![]));
    }

    let results = sqlx::query_as(
        r#"
        SELECT id AS user_id, username
        FROM users
        WHERE username ILIKE $1
          AND id != $2
        ORDER BY username
        LIMIT 20
        "#,
    )
    .bind(format!("{}%", query.q.trim()))
    .bind(current_user.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("search_users: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(results))
}
