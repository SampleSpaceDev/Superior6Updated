// templates/home.rs

use crate::models::User;
use askama::Template;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct CurrentGameweek {
    pub week_number: i32,
    pub season: String,
    pub deadline: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TopPlayer {
    pub display_name: String,
    pub total_points: i32,
    pub season: String,
}

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomeTemplate<'a> {
    pub user: Option<&'a User>,
    pub current_gameweek: Option<CurrentGameweek>,
    pub top_players: Vec<TopPlayer>,

    pub has_user: bool,
    pub has_gameweek: bool,
    pub has_players: bool,

    pub display_name: String,
    pub is_admin: bool,
}

impl<'a> HomeTemplate<'a> {
    pub fn new(
        user: Option<&'a User>,
        current_gameweek: Option<CurrentGameweek>,
        top_players: Vec<TopPlayer>,
    ) -> Self {
        Self {
            user,
            current_gameweek: current_gameweek.clone(),
            top_players: top_players.clone(),
            has_user: user.is_some(),
            has_gameweek: current_gameweek.is_some(),
            has_players: !top_players.is_empty(),
            display_name: user.map(|u| u.display_name.clone()).unwrap_or_else(|| "Guest".to_string()),
            is_admin: user.map(|u| u.is_admin).unwrap_or(false),
        }
    }
}