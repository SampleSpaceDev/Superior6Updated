// templates/leaderboard.rs

use askama::Template;
use crate::models::{User, UserWithScore};

#[derive(Template)]
#[template(path = "leaderboard/season.html")]
pub struct SeasonLeaderboardTemplate<'a> {
    pub user: Option<&'a User>,
    pub season: &'a str,
    pub leaderboard: Vec<UserWithScore>,

    pub has_user: bool,
    pub display_name: String,
    pub is_admin: bool,
    pub error: Option<String>,
}

#[derive(Template)]
#[template(path = "leaderboard/weekly.html")]
pub struct WeeklyLeaderboardTemplate<'a> {
    pub user: Option<&'a User>,
    pub week_number: i32,
    pub season: &'a str,
    pub leaderboard: Vec<UserWithScore>,
    pub error: Option<String>,

    pub has_user: bool,
    pub display_name: String,
    pub is_admin: bool,
}

impl<'a> SeasonLeaderboardTemplate<'a> {
    pub fn new(
        user: Option<&'a User>,
        season: &'a str,
        leaderboard: Vec<UserWithScore>,
        error: Option<String>,
    ) -> Self {
        Self {
            user,
            season,
            leaderboard,
            error,
            has_user: user.is_some(),
            display_name: user.map(|u| u.display_name.clone()).unwrap_or_else(|| "Guest".to_string()),
            is_admin: user.map(|u| u.is_admin).unwrap_or(false),
        }
    }
}

impl<'a> WeeklyLeaderboardTemplate<'a> {
    pub fn new(
        user: Option<&'a User>,
        week_number: i32,
        season: &'a str,
        leaderboard: Vec<UserWithScore>,
        error: Option<String>,
    ) -> Self {
        Self {
            user,
            week_number,
            season,
            leaderboard,
            error,
            has_user: user.is_some(),
            display_name: user.map(|u| u.display_name.clone()).unwrap_or_else(|| "Guest".to_string()),
            is_admin: user.map(|u| u.is_admin).unwrap_or(false),
        }
    }
}