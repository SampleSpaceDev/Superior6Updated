// templates/admin.rs

use askama::Template;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::models::{User, Gameweek, Fixture};

#[derive(Debug)]
pub struct GameweekInfo {
    pub id: Uuid,
    pub week_number: i32,
    pub season: String,
    pub deadline: DateTime<Utc>,
    pub is_active: bool,
    pub is_completed: bool,
}

#[derive(Debug)]
pub struct FixtureInfo {
    pub id: Uuid,
    pub home_team: String,
    pub away_team: String,
    pub kickoff_time: DateTime<Utc>,
    pub home_score: Option<i32>,
    pub away_score: Option<i32>,
    pub fixture_order: i32,
}

#[derive(Template)]
#[template(path = "admin/dashboard.html")]
pub struct AdminDashboardTemplate<'a> {
    pub user: &'a User,
    pub active_gameweek: Option<GameweekInfo>,
    pub total_users: i32,
    pub recent_gameweeks: Vec<GameweekInfo>,
}

#[derive(Template)]
#[template(path = "admin/gameweeks.html")]
pub struct GameweeksTemplate<'a> {
    pub user: &'a User,
    pub gameweeks: Vec<Gameweek>,
    pub error: Option<String>,
    pub success: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/fixtures.html")]
pub struct FixturesTemplate<'a> {
    pub user: &'a User,
    pub active_gameweek: Option<GameweekInfo>,
    pub fixtures: Vec<Fixture>,
    pub error: Option<String>,
    pub success: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/results.html")]
pub struct ResultsTemplate<'a> {
    pub user: &'a User,
    pub active_gameweek: Option<GameweekInfo>,
    pub fixtures: Vec<FixtureInfo>,
    pub error: Option<String>,
    pub success: Option<String>,
}