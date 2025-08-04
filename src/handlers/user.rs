// handlers/user.rs

use axum::extract::State;
use axum::response::{Html, IntoResponse};
use sqlx::query;
use crate::AppState;
use crate::auth::AuthUser;
use crate::errors::AppError;
use crate::templates::user::{DashboardTemplate, RecentGameweek, UserStats};

pub async fn dashboard(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let user_id = auth_user.user.id;

    let season_stats = query!(
        r#"
        SELECT
            ss.total_points,
            ss.total_exact_scores,
            ss.total_correct_results,
            ss.gameweeks_played,
            ss.season,
            (
                SELECT COUNT(*) + 1
                FROM season_scores ss2
                WHERE ss2.season = ss.season
                AND (
                    ss2.total_points > ss.total_points OR
                    (ss2.total_points = ss.total_points AND ss2.total_exact_scores > ss.total_exact_scores)
                )
            ) as position
        FROM season_scores ss
        WHERE ss.user_id = $1
        AND ss.season = (SELECT season FROM gameweeks WHERE is_active = true LIMIT 1)
        "#,
        user_id
    )
    .fetch_optional(&state.db)
    .await?;

    let user_stats = season_stats.map(|stats| UserStats {
        total_points: stats.total_points.unwrap_or(0),
        total_exact_scores: stats.total_exact_scores.unwrap_or(0),
        total_correct_results: stats.total_correct_results.unwrap_or(0),
        gameweeks_played: stats.gameweeks_played.unwrap_or(0),
        season: stats.season,
        position: stats.position.unwrap_or(0) as i32,
    });

    let recent_gameweeks = query!(
        r#"
        SELECT
            gw.week_number,
            gw.season,
            gs.total_points,
            gs.exact_scores,
            gs.correct_results,
            gw.is_completed
        FROM gameweeks gw
        LEFT JOIN gameweek_scores gs ON gw.id = gs.gameweek_id AND gs.user_id = $1
        WHERE gw.season = (SELECT season FROM gameweeks WHERE is_active = true LIMIT 1)
        ORDER BY gw.week_number DESC
        LIMIT 5
        "#,
        user_id
    )
    .fetch_all(&state.db)
    .await?;

    let recent_gameweeks: Vec<RecentGameweek> = recent_gameweeks
        .into_iter()
        .map(|gw| RecentGameweek {
            week_number: gw.week_number,
            season: gw.season,
            total_points: gw.total_points.unwrap_or(0),
            exact_scores: gw.exact_scores.unwrap_or(0),
            correct_results: gw.correct_results.unwrap_or(0),
            is_completed: gw.is_completed.unwrap_or(false),
        })
        .collect();

    let current_gameweek = query!(
        "SELECT id, week_number, season, deadline FROM gameweeks WHERE is_active = true LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await?;

    let has_predictions = if let Some(ref gw) = current_gameweek {
        query!(
            r#"
            SELECT COUNT(*) as count
            FROM predictions p
            JOIN fixtures f ON p.fixture_id = f.id
            WHERE f.gameweek_id = $1 AND p.user_id = $2
            "#,
            gw.id,
            user_id
        )
        .fetch_one(&state.db)
        .await?
        .count
        .unwrap_or(0) > 0
    } else {
        false
    };

    let template = DashboardTemplate {
        user: &auth_user.user,
        user_stats,
        recent_gameweeks,
        current_gameweek,
        has_predictions
    };

    Ok(Html(template.render()?))
}