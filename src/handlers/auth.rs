// handlers/auth.rs

use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::State;
use axum::Form;
use axum::response::{Html, Redirect};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::{CookieJar};
use sqlx::{query, query_as};
use time::Duration;
use validator::Validate;
use crate::AppState;
use crate::auth::{hash_password, verify_password, Claims, OptionalAuthUser};
use crate::errors::AppError;
use crate::models::{CreateUser, LoginUser, User};
use crate::templates::auth::{LoginTemplate, RegisterTemplate};

pub async fn register_form(
    State(_state): State<AppState>,
    auth_user: OptionalAuthUser
) -> Result<impl IntoResponse, AppError> {
    if auth_user.user.is_some() {
        return Ok(Redirect::to("/dashboard").into_response())
    }

    let template = RegisterTemplate {
        error: None,
    };

    Ok(Html(template.render()?).into_response())
}

pub async fn register(
    State(state): State<AppState>,
    Form(input): Form<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    input.validate()?;

    let existing_user = query!(
        "SELECT id FROM users WHERE email = $1",
        input.email
    )
    .fetch_optional(&state.db)
    .await?;

    if existing_user.is_some() {
        let template = RegisterTemplate {
            error: Some("Email already exists".to_string()),
        };
        return Ok(Html(template.render()?).into_response());
    }

    let password_hash = hash_password(&input.password)?;

    let user = query_as::<_, User>(
        r#"
        INSERT INTO users (name, display_name, email, password_hash)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#
    )
    .bind(&input.name)
    .bind(&input.display_name)
    .bind(&input.email)
    .bind(&password_hash)
    .fetch_one(&state.db)
    .await?;

    let token = Claims::new(&user, &state.config.jwt_secret)?;

    let cookie = Cookie::build(("auth_token", token))
        .path("/")
        .http_only(true)
        .build();

    Ok((CookieJar::new().add(cookie), Redirect::to("/dashboard")))
}

pub async fn login_form(
    State(_state): State<AppState>,
    auth_user: OptionalAuthUser
) -> Result<impl IntoResponse, AppError> {
    if auth_user.user.is_some() {
        return Ok(Redirect::to("/dashboard").into_response())
    }

    let template = LoginTemplate {
        error: None,
    };

    Ok(Html(template.render()?).into_response())
}

pub async fn login(
    State(state): State<AppState>,
    Form(input): Form<LoginUser>,
) -> Result<impl IntoResponse, AppError> {
    input.validate()?;

    let user = query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(&input.email)
    .fetch_optional(&state.db)
    .await?;

    let user = match user {
        Some(user) => user,
        None => {
            let template = LoginTemplate {
                error: Some("Invalid email or password".to_string()),
            };
            return Ok(Html(template.render()?));
        }
    };

    if !verify_password(&input.password, &user.password_hash)? {
        let template = LoginTemplate {
            error: Some("Invalid email or password".to_string()),
        };
        return Ok(Html(template.render()?));
    }

    let token = Claims::new(&user, &state.config.jwt_secret)?;

    let cookie = Cookie::build(("auth_token", token))
        .path("/")
        .http_only(true)
        .build();

    Ok((CookieJar::new().add(cookie), Redirect::to("/dashboard")))
}

pub async fn logout() -> impl IntoResponse {
    let cookie = Cookie::build(("auth_token", ""))
        .path("/")
        .max_age(Duration::seconds(0))
        .build();

    (CookieJar::new().add(cookie), Redirect::to("/"))
}