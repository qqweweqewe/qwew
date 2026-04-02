use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub jwt_secret: String,
    pub jwt_expires_in_hours: i64,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "super-secret-jwt-key-change-this-in-production".to_string()),
            
            jwt_expires_in_hours: env::var("JWT_EXPIRES_IN_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .unwrap_or(24),
        }
    }
}
