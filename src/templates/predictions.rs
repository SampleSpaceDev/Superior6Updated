// templates/predictions.rs

use askama::Template;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::models::{User, FixtureWithPrediction};

#[derive(Debug, Clone)]
pub struct CurrentGameweekInfo {
    pub id: Uuid,
    pub week_number: i32,
    pub season: String,
    pub deadline: DateTime<Utc>,
}

#[derive(Template)]
#[template(path = "predictions/current.html")]
pub struct PredictionsTemplate<'a> {
    pub user: &'a User,
    pub current_gameweek: Option<CurrentGameweekInfo>,
    pub fixtures_with_predictions: Vec<FixtureWithPrediction>,
    pub deadline_passed: bool,
    pub already_submitted: bool,
    pub error: Option<String>,

    pub has_user: bool,
    pub display_name: String,
    pub is_admin: bool,
    pub has_gameweek: bool,
}

impl<'a> PredictionsTemplate<'a> {
    pub fn new(
        user: &'a User,
        current_gameweek: Option<CurrentGameweekInfo>,
        fixtures_with_predictions: Vec<FixtureWithPrediction>,
        deadline_passed: bool,
        already_submitted: bool,
        error: Option<String>,
    ) -> Self {
        Self {
            user,
            current_gameweek: current_gameweek.clone(),
            fixtures_with_predictions,
            deadline_passed,
            already_submitted,
            error,

            has_user: true,
            display_name: user.display_name.clone(),
            is_admin: user.is_admin,
            has_gameweek: current_gameweek.is_some(),
        }
    }
}