use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use crate::config::AppConfig;
use crate::utils::jwt::Claims;

#[derive(Debug)]
pub struct CurrentUser {
    pub user_id: i64,
    pub username: String,
}

#[async_trait]
impl FromRequestParts<AppConfig> for CurrentUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        config: &AppConfig,
    ) -> Result<Self, Self::Rejection> {
        
        let auth_header = parts.headers
            .get("authorization")
            .and_then(|v| v.to_str().ok());

        let token = match auth_header {
            Some(h) if h.starts_with("Bearer ") => &h[7..].trim(),
            _ => return Err(StatusCode::UNAUTHORIZED),
        };

        match crate::utils::jwt::decode_token(token, config) {
            Ok(claims) => Ok(CurrentUser {
                user_id: claims.sub,
                username: claims.username,
            }),
            Err(e) => {
                tracing::warn!("JWT decode failed: {}", e);
                Err(StatusCode::UNAUTHORIZED)
            }
        }
    }
}
