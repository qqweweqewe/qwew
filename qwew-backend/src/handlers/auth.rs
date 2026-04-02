use argon2::{PasswordHash, PasswordVerifier};
use axum::{Json, Extension, http::StatusCode, response::{IntoResponse, Response}};
use sqlx::PgPool;
use crate::models::user::{AuthResponse, CreateUserRequest, LoginRequest, User};
use crate::handlers::extractors::CurrentUser;
use serde_json::json;
use crate::config::AppConfig;
use crate::utils::jwt::generate_token;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};

pub(crate) struct AppError(StatusCode, &'static str);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}

pub async fn register(
    Extension(pool): Extension<PgPool>,
    Extension(config): Extension<AppConfig>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<AuthResponse>, AppError> {

    if payload.username.len() < 3 || payload.username.len() > 30 {
        return Err(AppError(StatusCode::BAD_REQUEST, "username must be 3–30 characters"));
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| AppError(StatusCode::INTERNAL_SERVER_ERROR, "internal error"))?
        .to_string();

    let mut tx = pool.begin().await
        .map_err(|e| { tracing::error!("register: failed to begin tx: {}", e); AppError(StatusCode::INTERNAL_SERVER_ERROR, "internal error") })?;

    // validate + consume invite (lazy expiry cleanup included)
    let invite_valid: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM invites WHERE code = $1 AND used_by IS NULL AND (expires_at IS NULL OR expires_at > NOW()))"
    )
    .bind(&payload.invite_code)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| { tracing::error!("register: invite check failed: {}", e); AppError(StatusCode::INTERNAL_SERVER_ERROR, "internal error") })?;

    if !invite_valid {
        // lazy cleanup of expired invites
        let _ = sqlx::query("DELETE FROM invites WHERE expires_at IS NOT NULL AND expires_at <= NOW()")
            .execute(&mut *tx).await;
        let _ = tx.rollback().await;
        tracing::warn!("register: invalid or expired invite code used: {}", payload.invite_code);
        return Err(AppError(StatusCode::BAD_REQUEST, "invalid or expired invite code"));
    }

    let user: User = sqlx::query_as(
        "INSERT INTO users (username, password_hash) VALUES ($1, $2) RETURNING id, username, password_hash, created_at, last_seen_at"
    )
    .bind(&payload.username)
    .bind(&password_hash)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
            tracing::warn!("register: username already taken: {}", payload.username);
            AppError(StatusCode::CONFLICT, "username already taken")
        } else {
            tracing::error!("register: db error: {}", e);
            AppError(StatusCode::INTERNAL_SERVER_ERROR, "internal error")
        }
    })?;

    // mark invite as used
    sqlx::query("UPDATE invites SET used_by = $1, used_at = NOW() WHERE code = $2")
        .bind(user.id)
        .bind(&payload.invite_code)
        .execute(&mut *tx)
        .await
        .map_err(|e| { tracing::error!("register: failed to consume invite: {}", e); AppError(StatusCode::INTERNAL_SERVER_ERROR, "internal error") })?;

    tx.commit().await
        .map_err(|e| { tracing::error!("register: failed to commit tx: {}", e); AppError(StatusCode::INTERNAL_SERVER_ERROR, "internal error") })?;

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
) -> Result<Json<AuthResponse>, AppError> {

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
    .map_err(|_| AppError(StatusCode::UNAUTHORIZED, "invalid credentials"))?;

    // verify passwd
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| AppError(StatusCode::INTERNAL_SERVER_ERROR, "internal error"))?;

    let argon2 = Argon2::default();

    if argon2.verify_password(&payload.password.as_bytes(), &parsed_hash).is_err() {
        return Err(AppError(StatusCode::UNAUTHORIZED, "invalid credentials"));
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

pub async fn get_me(
    current_user: CurrentUser,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(json!({
        "success": true,
        "user_id": current_user.user_id,
        "username": current_user.username,
    })))
}
