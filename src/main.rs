// main.rs

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    routing::post,
    Router
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir};

mod config;
mod models;
mod handlers;
mod auth;
mod scoring;
mod templates;
mod errors;

use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<Config>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let config = Arc::new(Config::from_env()?);

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("src/migrations")
        .run(&db)
        .await?;

    let app_state = AppState { db, config };
    let app = create_app(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6666").await?;
    println!("Superior 6 server listening on: {}", listener.local_addr()?);

    axum::serve(listener, app)
        .await?;

    Ok(())
}

fn create_app(state: AppState) -> Router {
    Router::new()
        // Static files (CSS, JS, images, etc.)
        .nest_service("/static", ServeDir::new("static"))

        // Public routes
        .route("/", get(handlers::home::index))
        .route("/register", get(handlers::auth::register_form).post(handlers::auth::register))
        .route("/login", get(handlers::auth::login_form).post(handlers::auth::login))
        .route("/logout", post(handlers::auth::logout))

        // Protected user routes
        .route("/dashboard", get(handlers::user::dashboard))
        .route("/predictions", get(handlers::predictions::current_gameweek))
        .route("/predictions/submit", post(handlers::predictions::submit))
        .route("/leaderboard", get(handlers::leaderboard::season))
        .route("/leaderboard/weekly", get(handlers::leaderboard::weekly))

        // Admin routes
        .route("/admin", get(handlers::admin::dashboard))
        .route("/admin/gameweeks", get(handlers::admin::gameweeks).post(handlers::admin::create_gameweek))
        .route("/admin/fixtures", get(handlers::admin::fixtures).post(handlers::admin::create_fixtures))
        .route("/admin/results", get(handlers::admin::results).post(handlers::admin::submit_results))

        // Health check
        .route("/health", get(health_check))

        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
        )
        .with_state(state)
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "Superior 6 is running!")
}