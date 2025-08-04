use axum::extract::State;
use axum::Form;
use axum::response::{Html, IntoResponse, Redirect};
use chrono::Utc;
use sqlx::{query, query_as};
use validator::Validate;
use crate::AppState;
use crate::auth::AuthUser;
use crate::errors::AppError;
use crate::models::{Fixture, FixtureWithPrediction, GameweekPredictions, Prediction};
use crate::templates::predictions::{CurrentGameweekInfo, PredictionsTemplate};

pub async fn current_gameweek(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let user_id = auth_user.user.id;

    let current_gameweek = query!(
        "SELECT id, week_number, season, deadline FROM gameweeks WHERE is_active = true LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await?;

    let current_gameweek = match current_gameweek {
        Some(gw) => gw,
        None => {
            let template = PredictionsTemplate {
                user: &auth_user.user,
                current_gameweek: None,
                fixtures_with_predictions: vec![],
                deadline_passed: false,
                already_submitted: false,
                error: Some("No active gameweek found".to_string())
            };

            return Ok(Html(template.render()?))
        }
    };

    let deadline_passed = current_gameweek.deadline <= Utc::now();

    let fixtures = query_as::<_, Fixture>(
        "SELECT * FROM fixtures WHERE gameweek_id = $1 ORDER BY fixture_order"
    )
    .bind(current_gameweek.id)
    .fetch_all(&state.db)
    .await?;

    if fixtures.len() != 6 {
        let template = PredictionsTemplate {
            user: &auth_user.user,
            current_gameweek: Some(CurrentGameweekInfo {
                id: current_gameweek.id,
                week_number: current_gameweek.week_number,
                season: current_gameweek.season,
                deadline: current_gameweek.deadline
            }),
            fixtures_with_predictions: vec![],
            deadline_passed,
            already_submitted: false,
            error: Some("This gameweek doesn't have 6 fixtures set up yet".to_string())
        };

        return Ok(Html(template.render()?))
    }

    let existing_predictions = query_as::<_, Prediction>(
        r#"
            SELECT p.* FROM predictions p
            JOIN fixtures f ON p.fixture_id = f.id
            WHERE f.gameweek_id = $1 AND p.user_id = $2
        "#
    )
    .bind(current_gameweek.id)
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    let already_submitted = existing_predictions.len() == 6;

    let fixtures_with_predictions: Vec<FixtureWithPrediction> = fixtures
        .into_iter()
        .map(|fixture| {
            let prediction = existing_predictions
                .iter()
                .find(|p| p.fixture_id == fixture.id)
                .cloned();

            FixtureWithPrediction {
                fixture,
                prediction
            }
        })
        .collect();

    let template = PredictionsTemplate {
        user: &auth_user.user,
        current_gameweek: Some(CurrentGameweekInfo {
            id: current_gameweek.id,
            week_number: current_gameweek.week_number,
            season: current_gameweek.season,
            deadline: current_gameweek.deadline
        }),
        fixtures_with_predictions,
        deadline_passed,
        already_submitted,
        error: None,
    };

    Ok(Html(template.render()?))
}

pub async fn submit(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Form(input): Form<GameweekPredictions>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = auth_user.user.id;

    // Validate all predictions
    for prediction in &input.predictions {
        prediction.validate()?;
    }

    // Get the current active gameweek
    let current_gameweek = query!(
        "SELECT id, deadline FROM gameweeks WHERE is_active = true LIMIT 1"
    )
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    // Check if deadline has passed
    if current_gameweek.deadline <= Utc::now() {
        return Err(AppError::DeadlinePassed);
    }

    // Check if predictions are exactly 6
    if input.predictions.len() != 6 {
        return Err(AppError::InvalidPrediction);
    }

    // Verify all fixture IDs belong to the current gameweek
    let fixture_ids: Vec<_> = input.predictions.iter().map(|p| p.fixture_id).collect();
    let valid_fixtures = query!(
        "SELECT COUNT(*) as count FROM fixtures WHERE gameweek_id = $1 AND id = ANY($2)",
        current_gameweek.id,
        &fixture_ids
    )
        .fetch_one(&state.db)
        .await?;

    if valid_fixtures.count != Some(6) {
        return Err(AppError::InvalidPrediction);
    }

    // Check if user already has predictions for this gameweek
    let existing_count = query!(
        r#"
        SELECT COUNT(*) as count FROM predictions p
        JOIN fixtures f ON p.fixture_id = f.id
        WHERE f.gameweek_id = $1 AND p.user_id = $2
        "#,
        current_gameweek.id,
        user_id
    )
        .fetch_one(&state.db)
        .await?;

    if existing_count.count.unwrap_or(0) > 0 {
        return Err(AppError::PredictionsAlreadySubmitted);
    }

    // Insert all predictions
    for prediction in input.predictions {
        query!(
            r#"
            INSERT INTO predictions (user_id, fixture_id, home_score_prediction, away_score_prediction)
            VALUES ($1, $2, $3, $4)
            "#,
            user_id,
            prediction.fixture_id,
            prediction.home_score_prediction,
            prediction.away_score_prediction
        )
            .execute(&state.db)
            .await?;
    }

    Ok(Redirect::to("/predictions"))
}