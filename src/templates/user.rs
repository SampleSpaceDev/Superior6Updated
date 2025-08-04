// templates/user.rs

use askama::Template;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::models::User;

#[derive(Debug)]
pub struct UserStats {
    pub total_points: i32,
    pub total_exact_scores: i32,
    pub total_correct_results: i32,
    pub gameweeks_played: i32,
    pub season: String,
    pub position: i32,
}

#[derive(Debug)]
pub struct RecentGameweek {
    pub week_number: i32,
    pub season: String,
    pub total_points: i32,
    pub exact_scores: i32,
    pub correct_results: i32,
    pub is_completed: bool,
}

#[derive(Debug, Clone)]
pub struct CurrentGameweek {
    pub id: Uuid,
    pub week_number: i32,
    pub season: String,
    pub deadline: DateTime<Utc>,

    pub is_completed: bool,
}

#[derive(Template)]
#[template(path = "user/dashboard.html")]
pub struct DashboardTemplate<'a> {
    pub user: &'a User,
    pub user_stats: Option<UserStats>,
    pub recent_gameweeks: Vec<RecentGameweek>,
    pub current_gameweek: Option<CurrentGameweek>,
    pub has_predictions: bool,

    pub has_user: bool,
    pub display_name: String,
    pub is_admin: bool,
    pub has_current_gameweek: bool,
}

impl<'a> DashboardTemplate<'a> {
    pub fn new(
        user: &'a User,
        user_stats: Option<UserStats>,
        recent_gameweeks: Vec<RecentGameweek>,
        current_gameweek: Option<CurrentGameweek>,
        has_predictions: bool,
    ) -> Self {
        Self {
            user,
            user_stats,
            recent_gameweeks,
            current_gameweek: current_gameweek.clone(),
            has_predictions,
            has_user: true,
            display_name: user.display_name.clone(),
            is_admin: user.is_admin,
            has_current_gameweek: current_gameweek.is_some(),
        }
    }
}