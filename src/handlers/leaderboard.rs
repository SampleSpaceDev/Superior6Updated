// handlers/leaderboard.rs

use askama::Template;
use axum::extract::State;
use axum::response::{Html, IntoResponse};
use chrono::Utc;
use sqlx::query;
use crate::AppState;
use crate::auth::OptionalAuthUser;
use crate::errors::AppError;
use crate::models::UserWithScore;
use crate::templates::leaderboard::{SeasonLeaderboardTemplate, WeeklyLeaderboardTemplate};

pub async fn season(
    State(state): State<AppState>,
    auth_user: OptionalAuthUser,
) -> Result<impl IntoResponse, AppError> {
    // Get current season
    let current_season = query!(
        "SELECT season FROM gameweeks WHERE is_active = true LIMIT 1"
    )
        .fetch_optional(&state.db)
        .await?;

    let season = current_season
        .map(|s| s.season)
        .unwrap_or_else(|| "2024-25".to_string());

    // Get season leaderboard
    let leaderboard_data = query!(
        r#"
        SELECT
            u.id, u.name, u.display_name, u.email, u.password_hash, u.is_admin, u.created_at, u.updated_at,
            COALESCE(ss.total_points, 0) as total_points,
            COALESCE(ss.total_exact_scores, 0) as exact_scores,
            COALESCE(ss.total_correct_results, 0) as correct_results,
            ROW_NUMBER() OVER (ORDER BY COALESCE(ss.total_points, 0) DESC, COALESCE(ss.total_exact_scores, 0) DESC) as position
        FROM users u
        LEFT JOIN season_scores ss ON u.id = ss.user_id AND ss.season = $1
        WHERE u.is_admin = false
        ORDER BY total_points DESC, exact_scores DESC, u.display_name ASC
        "#,
        season
    )
        .fetch_all(&state.db)
        .await?;

    let leaderboard: Vec<UserWithScore> = leaderboard_data
        .into_iter()
        .map(|row| UserWithScore {
            user: crate::models::User {
                id: row.id,
                name: row.name,
                display_name: row.display_name,
                email: row.email,
                password_hash: row.password_hash,
                is_admin: row.is_admin.unwrap_or(false),
                created_at: row.created_at.unwrap_or(Utc::now()),
                updated_at: row.updated_at.unwrap_or(Utc::now()),
            },
            score: row.total_points.unwrap_or(0),
            exact_scores: row.exact_scores.unwrap_or(0),
            correct_results: row.correct_results.unwrap_or(0),
            position: row.position.unwrap_or(0) as i32,
        })
        .collect();

    let template = SeasonLeaderboardTemplate {
        user: auth_user.user.as_ref(),
        season: &season,
        leaderboard,
    };

    Ok(Html(template.render()?))
}

pub async fn weekly(
    State(state): State<AppState>,
    auth_user: OptionalAuthUser,
) -> Result<impl IntoResponse, AppError> {
    // Get current active gameweek
    let current_gameweek = query!(
        "SELECT id, week_number, season FROM gameweeks WHERE is_active = true LIMIT 1"
    )
        .fetch_optional(&state.db)
        .await?;

    let (gameweek_id, week_number, season) = match current_gameweek {
        Some(gw) => (gw.id, gw.week_number, gw.season),
        None => {
            let template = WeeklyLeaderboardTemplate {
                user: auth_user.user.as_ref(),
                week_number: 0,
                season: "No active gameweek",
                leaderboard: vec![],
                error: Some("No active gameweek found".to_string()),
            };
            return Ok(Html(template.render()?));
        }
    };

    // Get weekly leaderboard
    let leaderboard_data = query!(
        r#"
        SELECT
            u.id, u.name, u.display_name, u.email, u.password_hash, u.is_admin, u.created_at, u.updated_at,
            COALESCE(gs.total_points, 0) as total_points,
            COALESCE(gs.exact_scores, 0) as exact_scores,
            COALESCE(gs.correct_results, 0) as correct_results,
            ROW_NUMBER() OVER (ORDER BY COALESCE(gs.total_points, 0) DESC, COALESCE(gs.exact_scores, 0) DESC) as position
        FROM users u
        LEFT JOIN gameweek_scores gs ON u.id = gs.user_id AND gs.gameweek_id = $1
        WHERE u.is_admin = false
        ORDER BY total_points DESC, exact_scores DESC, u.display_name ASC
        "#,
        gameweek_id
    )
        .fetch_all(&state.db)
        .await?;

    let leaderboard: Vec<UserWithScore> = leaderboard_data
        .into_iter()
        .map(|row| UserWithScore {
            user: crate::models::User {
                id: row.id,
                name: row.name,
                display_name: row.display_name,
                email: row.email,
                password_hash: row.password_hash,
                is_admin: row.is_admin.unwrap_or(false),
                created_at: row.created_at.unwrap_or(Utc::now()),
                updated_at: row.updated_at.unwrap_or(Utc::now()),
            },
            score: row.total_points.unwrap_or(0),
            exact_scores: row.exact_scores.unwrap_or(0),
            correct_results: row.correct_results.unwrap_or(0),
            position: row.position.unwrap_or(0) as i32,
        })
        .collect();

    let template = WeeklyLeaderboardTemplate {
        user: auth_user.user.as_ref(),
        week_number,
        season: &season,
        leaderboard,
        error: None,
    };

    Ok(Html(template.render()?))
}