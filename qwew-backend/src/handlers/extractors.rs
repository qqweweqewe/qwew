use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    Extension,
};
use crate::config::AppConfig;

#[derive(Debug)]
pub struct CurrentUser {
    pub user_id: i64,
    pub username: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Extension(config) = Extension::<AppConfig>::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let auth_header = parts.headers
            .get("authorization")
            .and_then(|value| value.to_str().ok());

        let token = match auth_header {
            Some(header) if header.starts_with("Bearer ") => &header[7..].trim(),
            _ => return Err(StatusCode::UNAUTHORIZED),
        };

        match crate::utils::jwt::decode_token(token, &config) {
            Ok(claims) => Ok(CurrentUser {
                user_id: claims.sub,
                username: claims.username,
            }),
            Err(_) => Err(StatusCode::UNAUTHORIZED),
        }
    }
}
