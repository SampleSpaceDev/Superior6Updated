// config.rs

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub bcrypt_cost: u32,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://localhost/superior6".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "supersecretkey".to_string()),
            bcrypt_cost: env::var("BCRYPT_COST")
                .unwrap_or_else(|_| "12".to_string())
                .parse()
                .unwrap_or(12),
        })
    }
}