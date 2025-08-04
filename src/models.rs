// models.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub email: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUser {
    #[validate(length(min = 2, max = 255))]
    pub name: String,
    #[validate(length(min = 2, max = 100))]
    pub display_name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginUser {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Gameweek {
    pub id: Uuid,
    pub week_number: i32,
    pub season: String,
    pub deadline: DateTime<Utc>,
    pub is_active: bool,
    pub is_completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateGameweek {
    pub week_number: i32,
    #[validate(length(min = 7, max = 20))]
    pub season: String,
    pub deadline: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Fixture {
    pub id: Uuid,
    pub gameweek_id: Uuid,
    pub home_team: String,
    pub away_team: String,
    pub kickoff_time: DateTime<Utc>,
    pub home_score: Option<i32>,
    pub away_score: Option<i32>,
    pub fixture_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateFixture {
    #[validate(length(min = 2, max = 255))]
    pub home_team: String,
    #[validate(length(min = 2, max = 255))]
    pub away_team: String,
    pub kickoff_time: DateTime<Utc>,
    #[validate(range(min = 1, max = 6))]
    pub fixture_order: i32,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Prediction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub fixture_id: Uuid,
    pub home_score_prediction: i32,
    pub away_score_prediction: i32,
    pub points_awarded: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePrediction {
    pub fixture_id: Uuid,
    #[validate(range(min = 0, max = 20))]
    pub home_score_prediction: i32,
    #[validate(range(min = 0, max = 20))]
    pub away_score_prediction: i32,
}

#[derive(Debug, Deserialize)]
pub struct GameweekPredictions {
    pub predictions: Vec<CreatePrediction>
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct GameweekScore {
    pub id: Uuid,
    pub user_id: Uuid,
    pub gameweek_id: Uuid,
    pub total_points: i32,
    pub exact_scores: i32,
    pub correct_results: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct SeasonScore {
    pub id: Uuid,
    pub user_id: Uuid,
    pub season: String,
    pub total_points: i32,
    pub total_exact_scores: i32,
    pub total_correct_results: i32,
    pub gameweeks_played: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// DTOs for templates
#[derive(Debug, Serialize)]
pub struct UserWithScore {
    pub user: User,
    pub score: i32,
    pub exact_scores: i32,
    pub correct_results: i32,
    pub position: i32,
}

#[derive(Debug, Serialize)]
pub struct FixtureWithPrediction {
    pub fixture: Fixture,
    pub prediction: Option<Prediction>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct FixtureResult {
    pub fixture_id: Uuid,
    #[validate(range(min = 0))]
    pub home_score: i32,
    #[validate(range(min = 0))]
    pub away_score: i32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct GameweekResults {
    pub results: Vec<FixtureResult>,
}