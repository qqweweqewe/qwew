use argon2::{PasswordHash, PasswordVerifier};
use axum::{Json, Extension, http::StatusCode};
use sqlx::PgPool;
use crate::models::user::{AuthResponse, CreateUserRequest, LoginRequest, User};
use crate::config::AppConfig;
use crate::utils::jwt::generate_token;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};

pub async fn register(
    Extension(pool): Extension<PgPool>,
    Extension(config): Extension<AppConfig>,   // ← New
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    
    if payload.username.len() < 3 || payload.username.len() > 30 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    let user: User = sqlx::query_as(
        r#"
        INSERT INTO users (username, password_hash)
        VALUES ($1, $2)
        RETURNING id, username, password_hash, created_at, last_seen_at
        "#
    )
    .bind(&payload.username)
    .bind(&password_hash)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create user: {}", e);
        if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
            StatusCode::CONFLICT
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    let token = generate_token(user.id, user.username.clone(), &config);

    Ok(Json(AuthResponse {
        user_id: user.id,
        username: user.username,
        token,
    }))
}

pub async fn login(
    Extension(pool): Extension<PgPool>,
    Extension(config): Extension<AppConfig>,
    Json(payload): Json<LoginRequest>
) -> Result<Json<AuthResponse>, StatusCode> {

    // get user
    let user: User = sqlx::query_as(
        r#"
        SELECT id, username, password_hash, created_at, last_seen_at
        FROM users
        WHERE username = $1
        "#
    ).bind(&payload.username)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // verify passwd
    let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let argon2 = Argon2::default();

    if argon2.verify_password(&payload.password.as_bytes(), &parsed_hash).is_err() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = generate_token(user.id, user.username.clone(), &config);

    Ok(
        Json(
            AuthResponse { 
                user_id: user.id, 
                username: user.username, 
                token 
            }
        )
    )
}