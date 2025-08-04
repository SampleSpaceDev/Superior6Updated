// scoring.rs

use std::cmp::Ordering;
use sqlx::{query, query_as, PgPool};
use uuid::Uuid;
use crate::errors::AppError;
use crate::models::{Fixture, Prediction};

pub const POINTS_EXACT_SCORE: i32 = 5;
pub const POINTS_CORRECT_RESULT: i32 = 2;

#[derive(Debug, Clone, PartialEq)]
pub enum MatchResult {
    HomeWin,
    Draw,
    AwayWin,
}

impl MatchResult {
    pub fn from_scores(home_score: i32, away_score: i32) -> Self {
        match home_score.cmp(&away_score) {
            Ordering::Greater => MatchResult::HomeWin,
            Ordering::Equal => MatchResult::Draw,
            Ordering::Less => MatchResult::AwayWin,
        }
    }
}

pub fn calculate_points(
    actual_home: i32,
    actual_away: i32,
    predicted_home: i32,
    predicted_away: i32,
) -> i32 {
    if actual_home == predicted_home && actual_away == predicted_away {
        return POINTS_EXACT_SCORE;
    }

    let actual_result = MatchResult::from_scores(actual_home, actual_away);
    let predicted_result = MatchResult::from_scores(predicted_home, predicted_away);

    if actual_result == predicted_result {
        return POINTS_CORRECT_RESULT;
    }

    0 // No points awarded
}

pub async fn calculate_gameweek_scores(
    db: &PgPool,
    gameweek_id: Uuid,
) -> Result<(), AppError> {
    let fixtures = query_as::<_, Fixture>(
        "SELECT * FROM fixtures WHERE gameweek_id = $1 AND home_score IS NOT NULL AND away_score IS NOT NULL",
    )
    .bind(gameweek_id)
    .fetch_all(db)
    .await?;

    if fixtures.is_empty() {
        return Ok(());
    }

    let fixture_ids: Vec<Uuid> = fixtures.iter().map(|f| f.id).collect();

    let predictions = query_as::<_, Prediction>(
        "SELECT * FROM predictions WHERE fixture_id = ANY($1)",
    )
    .bind(&fixture_ids)
    .fetch_all(db)
    .await?;

    for prediction in predictions {
        if let Some(fixture) = fixtures.iter().find(|f| f.id == prediction.fixture_id) {
            let points = calculate_points(
                fixture.home_score.unwrap(),
                fixture.away_score.unwrap(),
                prediction.home_score_prediction,
                prediction.away_score_prediction,
            );

            query(
                "UPDATE predictions SET points_awarded = $1 WHERE id = $2"
            )
            .bind(points)
            .bind(prediction.id)
            .execute(db)
            .await?;
        }
    }

    let user_scores = query!(
        r#"
        SELECT
            p.user_id,
            SUM(p.points_awarded) as total_points,
            COUNT(CASE WHEN p.points_awarded = $1 THEN 1 END) as exact_scores,
            COUNT(CASE WHEN p.points_awarded = $2 THEN 1 END) as correct_results
        FROM predictions p
        JOIN fixtures f ON p.fixture_id = f.id
        WHERE f.gameweek_id = $3 AND f.home_score IS NOT NULL
        GROUP BY p.user_id
        "#,
        POINTS_EXACT_SCORE,
        POINTS_CORRECT_RESULT,
        gameweek_id
    )
    .fetch_all(db)
    .await?;

    for score in user_scores {
        query(
            r#"
            INSERT INTO gameweek_scores (user_id, gameweek_id, total_points, exact_scores, correct_results)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (user_id, gameweek_id)
            DO UPDATE SET
                total_points = EXCLUDED.total_points,
                exact_scores = EXCLUDED.exact_scores,
                correct_results = EXCLUDED.correct_results,
                updated_at = NOW()
            "#
        )
        .bind(score.user_id)
        .bind(gameweek_id)
        .bind(score.total_points.unwrap_or(0) as i32)
        .bind(score.exact_scores.unwrap_or(0) as i32)
        .bind(score.correct_results.unwrap_or(0) as i32)
        .execute(db)
        .await?;
    }

    update_season_scores(db, gameweek_id).await?;

    Ok(())
}

pub async fn update_season_scores(
    db: &PgPool,
    gameweek_id: Uuid
) -> Result<(), AppError> {
    let gameweek = query!(
        "SELECT season FROM gameweeks WHERE id = $1",
        gameweek_id
    )
    .fetch_one(db)
    .await?;

    let season_totals = query!(
        r#"
        SELECT
            gs.user_id,
            SUM(gs.total_points) as total_points,
            SUM(gs.exact_scores) as total_exact_scores,
            SUM(gs.correct_results) as total_correct_results,
            COUNT(gs.gameweek_id) as gameweeks_played
        FROM gameweek_scores gs
        JOIN gameweeks gw ON gs.gameweek_id = gw.id
        WHERE gw.season = $1
        GROUP BY gs.user_id
        "#,
        gameweek.season
    )
    .fetch_all(db)
    .await?;

    for total in season_totals {
        query(
            r#"
            INSERT INTO season_scores (user_id, season, total_points, total_exact_scores, total_correct_results, gameweeks_played)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (user_id, season)
            DO UPDATE SET
                total_points = EXCLUDED.total_points,
                total_exact_scores = EXCLUDED.total_exact_scores,
                total_correct_results = EXCLUDED.total_correct_results,
                gameweeks_played = EXCLUDED.gameweeks_played,
                updated_at = NOW()
            "#
        )
        .bind(total.user_id)
        .bind(&gameweek.season)
        .bind(total.total_points.unwrap_or(0) as i32)
        .bind(total.total_exact_scores.unwrap_or(0) as i32)
        .bind(total.total_correct_results.unwrap_or(0) as i32)
        .bind(total.gameweeks_played.unwrap_or(0) as i32)
        .execute(db)
        .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_exact_score() {
        assert_eq!(calculate_points(2, 1, 2, 1), POINTS_EXACT_SCORE);
        assert_eq!(calculate_points(0, 0, 0, 0), POINTS_EXACT_SCORE);
        assert_eq!(calculate_points(3, 2, 3, 3), POINTS_EXACT_SCORE);
    }

    #[test]
    fn test_correct_result() {
        // Home wins
        assert_eq!(calculate_points(2, 1, 3, 0), POINTS_CORRECT_RESULT);
        assert_eq!(calculate_points(1, 0, 2, 1), POINTS_CORRECT_RESULT);

        // Draws
        assert_eq!(calculate_points(1, 1, 2, 2), POINTS_CORRECT_RESULT);
        assert_eq!(calculate_points(0, 0, 3, 3), POINTS_CORRECT_RESULT);

        // Away wins
        assert_eq!(calculate_points(0, 1, 1, 2), POINTS_CORRECT_RESULT);
        assert_eq!(calculate_points(1, 3, 0, 1), POINTS_CORRECT_RESULT);
    }

    #[test]
    fn test_no_points() {
        // Wrong result
        assert_eq!(calculate_points(2, 1, 1, 2), 0); // Home win vs Away win
        assert_eq!(calculate_points(1, 1, 2, 0), 0); // Draw vs Home win
        assert_eq!(calculate_points(0, 2, 1, 1), 0); // Away win vs Draw
    }

    #[test]
    fn test_match_result() {
        assert_eq!(MatchResult::from_scores(2, 1), MatchResult::HomeWin);
        assert_eq!(MatchResult::from_scores(1, 1), MatchResult::Draw);
        assert_eq!(MatchResult::from_scores(0, 2), MatchResult::AwayWin);
    }
}