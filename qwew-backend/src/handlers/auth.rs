use axum::{Json, Extension, http::StatusCode};
use sqlx::PgPool;
use crate::models::user::{CreateUserRequest, AuthResponse, User};
use argon2::{self, Argon2, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use rand::rngs::OsRng;

pub async fn register(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    
    // TODO: add proper validation

    // hash the password securely
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    // insert user into database
    let user: User = sqlx::query_as(
        r#"
        INSERT INTO users (username, password_hash)
        VALUES ($1, $2)
        RETURNING id, username, password_hash, created_at, last_seen_at
        "#
    )
    .bind(payload.username.clone())
    .bind(password_hash)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create user: {}", e);
        if e.to_string().contains("unique") {
            StatusCode::CONFLICT // already taken
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    // TODO: generate JWT token later

    Ok(Json(AuthResponse {
        user_id: user.id,
        username: user.username,
        token: "temporary_token".to_string(), // placeholder for now
    }))
}
