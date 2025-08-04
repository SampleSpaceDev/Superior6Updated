// auth.rs

use axum::{async_trait, RequestPartsExt};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::extract::CookieJar;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::query_as;
use uuid::Uuid;
use crate::AppState;
use crate::errors::AppError;
use crate::models::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub is_admin: bool,
    pub exp: i64,
}

impl Claims {
    pub fn new(user: &User, jwt_secret: &str) -> Result<String, AppError> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(30))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            is_admin: user.is_admin,
            exp: expiration,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_ref()),
        )
        .map_err(|_| AppError::TokenCreation)
    }

    pub fn from_token(token: &str, jwt_secret: &str) -> Result<Self, AppError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_ref()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| AppError::InvalidToken)
    }
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    hash(password, DEFAULT_COST).map_err(|_| AppError::HashingError)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    verify(password, hash).map_err(|_| AppError::HashingError)
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user: User,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookies = parts
            .extract::<CookieJar>()
            .await
            .map_err(|_| AppError::InvalidToken)?;

        let token = cookies
            .get("auth_token")
            .map(|cookie| cookie.value())
            .ok_or(AppError::MissingToken)?;

        let claims = Claims::from_token(token, &state.config.jwt_secret)?;

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::InvalidToken)?;

        let user = query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1",
        )
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| AppError::UserNotFound)?;

        Ok(AuthUser { user })
    }
}

#[derive(Debug, Clone)]
pub struct AdminUser {
    pub user: User,
}

#[async_trait]
impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, state).await?;

        if !auth_user.user.is_admin {
            return Err(AppError::Forbidden);
        }

        Ok(AdminUser { user: auth_user.user })
    }
}

#[derive(Debug, Clone)]
pub struct OptionalAuthUser {
    pub user: Option<User>,
}

#[async_trait]
impl FromRequestParts<AppState> for OptionalAuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState
    ) -> Result<Self, Self::Rejection> {
        match AuthUser::from_request_parts(parts, state).await {
            Ok(auth_user) => Ok(OptionalAuthUser { user: Some(auth_user.user) }),
            Err(_) => Ok(OptionalAuthUser { user: None }),
        }
    }
}