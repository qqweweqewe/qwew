use axum::{extract::Path, Extension, Json, http::StatusCode};
use sqlx::PgPool;
use crate::{handlers::extractors::CurrentUser, models::message::{Conversation, Message}};

pub async fn get_conversations(
    current_user: CurrentUser,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<Conversation>>, StatusCode> {
    let convos = sqlx::query_as(
        r#"
        SELECT
            c.id, c.user1_id, c.user2_id, c.created_at,
            u.username AS other_username,
            m.content  AS last_message,
            m.created_at AS last_message_at,
            COUNT(msg2.id) FILTER (
                WHERE msg2.sender_id != $1
                AND NOT EXISTS (
                    SELECT 1 FROM read_receipts rr
                    WHERE rr.message_id = msg2.id AND rr.user_id = $1
                )
            ) AS unread_count
        FROM conversations c
        JOIN users u ON u.id = CASE
            WHEN c.user1_id = $1 THEN c.user2_id
            ELSE c.user1_id
        END
        LEFT JOIN LATERAL (
            SELECT content, created_at FROM messages
            WHERE conversation_id = c.id
            ORDER BY created_at DESC LIMIT 1
        ) m ON true
        LEFT JOIN messages msg2 ON msg2.conversation_id = c.id
        WHERE c.user1_id = $1 OR c.user2_id = $1
        GROUP BY c.id, u.username, m.content, m.created_at
        ORDER BY last_message_at DESC NULLS LAST
        "#,
    )
    .bind(current_user.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("get_conversations: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(convos))
}

pub async fn get_history(
    current_user: CurrentUser,
    Path(conversation_id): Path<i64>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<Message>>, StatusCode> {
    // verify the user is a participant
    let is_participant: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM conversations WHERE id = $1 AND (user1_id = $2 OR user2_id = $2))"
    )
    .bind(conversation_id)
    .bind(current_user.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::warn!("get_history: participant check failed for user={} convo={}: {}", current_user.user_id, conversation_id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if !is_participant {
        tracing::warn!("get_history: user={} tried to access convo={}", current_user.user_id, conversation_id);
        return Err(StatusCode::FORBIDDEN);
    }

    let messages = sqlx::query_as(
        r#"
        SELECT id, conversation_id, sender_id, content, created_at
        FROM messages
        WHERE conversation_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(conversation_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("get_history: db error for convo={}: {}", conversation_id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(messages))
}
