// handlers/home.rs

use askama::Template;
use axum::extract::State;
use axum::response::{Html, IntoResponse};
use sqlx::query;
use crate::AppState;
use crate::auth::OptionalAuthUser;
use crate::errors::AppError;
use crate::templates::home::{CurrentGameweek, HomeTemplate, TopPlayer};

pub async fn index(
    State(state): State<AppState>,
    auth_user: OptionalAuthUser,
) -> Result<impl IntoResponse, AppError> {
    let current_gameweek_data = query!(
        "SELECT week_number, season, deadline FROM gameweeks WHERE is_active = true LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await?;

    let current_gameweek = current_gameweek_data.map(|gw| CurrentGameweek {
        week_number: gw.week_number,
        season: gw.season,
        deadline: gw.deadline,
    });

    let top_players_data = query!(
        r#"
        SELECT u.display_name, ss.total_points, ss.season
        FROM season_scores ss
        JOIN users u ON ss.user_id = u.id
        WHERE ss.season = (SELECT season FROM gameweeks WHERE is_active = true LIMIT 1)
        ORDER BY ss.total_points DESC, ss.total_exact_scores DESC
        LIMIT 5
        "#
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let top_players = top_players_data.into_iter().map(|row| TopPlayer {
        display_name: row.display_name,
        total_points: row.total_points.unwrap_or(0),
        season: row.season,
    }).collect();

    let template = HomeTemplate {
        user: auth_user.user.as_ref(),
        current_gameweek,
        top_players,
    };

    Ok(Html(template.render()?))
}