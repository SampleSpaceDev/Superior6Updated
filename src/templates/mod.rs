// templates/mod.rs

use askama::Template;
use crate::models::User;

pub mod auth;
pub mod home;
pub mod user;
pub mod predictions;
pub mod admin;
pub mod leaderboard;

#[derive(Template)]
#[template(path = "base.html")]
pub struct BaseTemplate<'a> {
    pub title: &'a str,
    pub user: Option<&'a User>,
    pub content: &'a str,

    pub has_user: bool,
    pub display_name: &'a str,
    pub is_admin: bool,
}

impl<'a> BaseTemplate<'a> {
    pub fn new(title: &'a str, user: Option<&'a User>, content: &'a str) -> Self {
        Self {
            title,
            user,
            content,
            has_user: user.is_some(),
            display_name: user.map(|u| u.display_name.as_str()).unwrap_or("Guest"),
            is_admin: user.map(|u| u.is_admin).unwrap_or(false),
        }
    }
}