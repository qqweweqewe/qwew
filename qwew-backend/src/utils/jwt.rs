use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
use crate::config::AppConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub username: String,
    pub exp: usize,
}

pub fn generate_token(user_id: i64, username: String, config: &AppConfig) -> String {
    let expiration = Utc::now()
        + Duration::hours(config.jwt_expires_in_hours);

    let claims = Claims {
        sub: user_id,
        username,
        exp: expiration.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    ).unwrap()
}

pub fn decode_token(token: &str, config: &AppConfig) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}
