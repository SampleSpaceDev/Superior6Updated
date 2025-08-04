// handlers/admin.rs

use crate::auth::AdminUser;
use crate::errors::AppError;
use crate::models::{CreateFixture, CreateGameweek, Fixture, Gameweek, GameweekResults};
use crate::scoring::calculate_gameweek_scores;
use crate::templates::admin::{
    AdminDashboardTemplate, FixtureInfo, FixturesTemplate, GameweekInfo,
    GameweeksTemplate, ResultsTemplate
};
use crate::AppState;
use askama::Template;
use axum::extract::State;
use axum::response::{Html, IntoResponse, Redirect};
use axum::Form;
use chrono::Utc;
use sqlx::{query, query_as};
use validator::Validate;

pub async fn dashboard(
    State(state): State<AppState>,
    admin_user: AdminUser,
) -> Result<impl IntoResponse, AppError> {
    // Get current active gameweek
    let active_gameweek = query!(
        "SELECT id, week_number, season, deadline, is_completed FROM gameweeks WHERE is_active = true LIMIT 1"
    )
        .fetch_optional(&state.db)
        .await?;

    // Get total users count
    let total_users = query!(
        "SELECT COUNT(*) as count FROM users WHERE is_admin = false"
    )
        .fetch_one(&state.db)
        .await?
        .count
        .unwrap_or(0) as i32;

    // Get recent gameweeks
    let recent_gameweeks = query!(
        r#"
        SELECT id, week_number, season, deadline, is_active, is_completed
        FROM gameweeks
        ORDER BY season DESC, week_number DESC
        LIMIT 5
        "#
    )
        .fetch_all(&state.db)
        .await?;

    let gameweeks: Vec<GameweekInfo> = recent_gameweeks
        .into_iter()
        .map(|gw| GameweekInfo {
            id: gw.id,
            week_number: gw.week_number,
            season: gw.season,
            deadline: gw.deadline,
            is_active: gw.is_active.unwrap_or(false),
            is_completed: gw.is_completed.unwrap_or(false),
        })
        .collect();

    let template = AdminDashboardTemplate {
        user: &admin_user.user,
        active_gameweek: active_gameweek.map(|gw| GameweekInfo {
            id: gw.id,
            week_number: gw.week_number,
            season: gw.season,
            deadline: gw.deadline,
            is_active: true,
            is_completed: gw.is_completed.unwrap_or(false),
        }),
        total_users,
        recent_gameweeks: gameweeks,
    };

    Ok(Html(template.render()?))
}

pub async fn gameweeks(
    State(state): State<AppState>,
    admin_user: AdminUser,
) -> Result<impl IntoResponse, AppError> {
    let gameweeks = query_as::<_, Gameweek>(
        "SELECT * FROM gameweeks ORDER BY season DESC, week_number DESC"
    )
        .fetch_all(&state.db)
        .await?;

    let template = GameweeksTemplate {
        user: &admin_user.user,
        gameweeks,
        error: None,
        success: None,
    };

    Ok(Html(template.render()?))
}

pub async fn create_gameweek(
    State(state): State<AppState>,
    _admin_user: AdminUser,
    Form(input): Form<CreateGameweek>,
) -> Result<impl IntoResponse, AppError> {
    input.validate()?;

    // Check if gameweek already exists
    let existing = query!(
        "SELECT id FROM gameweeks WHERE week_number = $1 AND season = $2",
        input.week_number,
        input.season
    )
        .fetch_optional(&state.db)
        .await?;

    if existing.is_some() {
        let gameweeks = query_as::<_, Gameweek>(
            "SELECT * FROM gameweeks ORDER BY season DESC, week_number DESC"
        )
            .fetch_all(&state.db)
            .await?;

        let template = GameweeksTemplate {
            user: &_admin_user.user,
            gameweeks,
            error: Some("Gameweek already exists for this season".to_string()),
            success: None,
        };
        return Ok(Html(template.render()?));
    }

    // Deactivate all other gameweeks before creating new one
    query!("UPDATE gameweeks SET is_active = false")
        .execute(&state.db)
        .await?;

    // Create new gameweek (it will be active by default)
    query!(
        r#"
        INSERT INTO gameweeks (week_number, season, deadline, is_active)
        VALUES ($1, $2, $3, true)
        "#,
        input.week_number,
        input.season,
        input.deadline
    )
        .execute(&state.db)
        .await?;

    Ok(Redirect::to("/admin/gameweeks"))
}

pub async fn fixtures(
    State(state): State<AppState>,
    admin_user: AdminUser,
) -> Result<impl IntoResponse, AppError> {
    // Get current active gameweek
    let active_gameweek = query!(
        "SELECT id, week_number, season FROM gameweeks WHERE is_active = true LIMIT 1"
    )
        .fetch_optional(&state.db)
        .await?;

    let fixtures = if let Some(ref gw) = active_gameweek {
        query_as::<_, Fixture>(
            "SELECT * FROM fixtures WHERE gameweek_id = $1 ORDER BY fixture_order"
        )
            .bind(gw.id)
            .fetch_all(&state.db)
            .await?
    } else {
        vec![]
    };

    let template = FixturesTemplate {
        user: &admin_user.user,
        active_gameweek: active_gameweek.map(|gw| GameweekInfo {
            id: gw.id,
            week_number: gw.week_number,
            season: gw.season,
            deadline: Utc::now(), // Placeholder
            is_active: true,
            is_completed: false,
        }),
        fixtures,
        error: None,
        success: None,
    };

    Ok(Html(template.render()?))
}

pub async fn create_fixtures(
    State(state): State<AppState>,
    admin_user: AdminUser,
    Form(fixtures): Form<Vec<CreateFixture>>,
) -> Result<impl IntoResponse, AppError> {
    // Get current active gameweek
    let active_gameweek = query!(
        "SELECT id FROM gameweeks WHERE is_active = true LIMIT 1"
    )
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    // Validate we have exactly 6 fixtures
    if fixtures.len() != 6 {
        let existing_fixtures = query_as::<_, Fixture>(
            "SELECT * FROM fixtures WHERE gameweek_id = $1 ORDER BY fixture_order"
        )
            .bind(active_gameweek.id)
            .fetch_all(&state.db)
            .await?;

        let template = FixturesTemplate {
            user: &admin_user.user,
            active_gameweek: Some(GameweekInfo {
                id: active_gameweek.id,
                week_number: 0,
                season: "".to_string(),
                deadline: Utc::now(),
                is_active: true,
                is_completed: false,
            }),
            fixtures: existing_fixtures,
            error: Some("You must provide exactly 6 fixtures".to_string()),
            success: None,
        };
        return Ok(Html(template.render()?));
    }

    // Validate all fixtures
    for fixture in &fixtures {
        fixture.validate()?;
    }

    // Delete existing fixtures for this gameweek
    query!(
        "DELETE FROM fixtures WHERE gameweek_id = $1",
        active_gameweek.id
    )
        .execute(&state.db)
        .await?;

    // Insert new fixtures
    for fixture in fixtures {
        query!(
            r#"
            INSERT INTO fixtures (gameweek_id, home_team, away_team, kickoff_time, fixture_order)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            active_gameweek.id,
            fixture.home_team,
            fixture.away_team,
            fixture.kickoff_time,
            fixture.fixture_order
        )
            .execute(&state.db)
            .await?;
    }

    Ok(Redirect::to("/admin/fixtures"))
}

pub async fn results(
    State(state): State<AppState>,
    admin_user: AdminUser,
) -> Result<impl IntoResponse, AppError> {
    // Get current active gameweek
    let active_gameweek = query!(
        "SELECT id, week_number, season FROM gameweeks WHERE is_active = true LIMIT 1"
    )
        .fetch_optional(&state.db)
        .await?;

    let fixtures_with_results = if let Some(ref gw) = active_gameweek {
        let fixtures = query_as::<_, Fixture>(
            "SELECT * FROM fixtures WHERE gameweek_id = $1 ORDER BY fixture_order"
        )
            .bind(gw.id)
            .fetch_all(&state.db)
            .await?;

        fixtures.into_iter().map(|f| FixtureInfo {
            id: f.id,
            home_team: f.home_team,
            away_team: f.away_team,
            kickoff_time: f.kickoff_time,
            home_score: f.home_score,
            away_score: f.away_score,
            fixture_order: f.fixture_order,
        }).collect()
    } else {
        vec![]
    };

    let template = ResultsTemplate {
        user: &admin_user.user,
        active_gameweek: active_gameweek.map(|gw| GameweekInfo {
            id: gw.id,
            week_number: gw.week_number,
            season: gw.season,
            deadline: Utc::now(), // Placeholder
            is_active: true,
            is_completed: false,
        }),
        fixtures: fixtures_with_results,
        error: None,
        success: None,
    };

    Ok(Html(template.render()?))
}

pub async fn submit_results(
    State(state): State<AppState>,
    _admin_user: AdminUser,
    Form(input): Form<GameweekResults>,
) -> Result<impl IntoResponse, AppError> {
    // Get current active gameweek
    let active_gameweek = query!(
        "SELECT id FROM gameweeks WHERE is_active = true LIMIT 1"
    )
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    // Validate we have exactly 6 results
    if input.results.len() != 6 {
        return Err(AppError::InvalidPrediction);
    }

    // Validate all results
    for result in &input.results {
        result.validate()?;
    }

    // Update fixture results
    for result in input.results {
        query!(
            "UPDATE fixtures SET home_score = $1, away_score = $2 WHERE id = $3",
            result.home_score,
            result.away_score,
            result.fixture_id
        )
            .execute(&state.db)
            .await?;
    }

    // Calculate scores for this gameweek
    calculate_gameweek_scores(&state.db, active_gameweek.id).await?;

    // Mark gameweek as completed
    query!(
        "UPDATE gameweeks SET is_completed = true WHERE id = $1",
        active_gameweek.id
    )
        .execute(&state.db)
        .await?;

    Ok(Redirect::to("/admin/results"))
}