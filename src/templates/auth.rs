// templates/auth.rs

use askama::Template;

#[derive(Template)]
#[template(path = "auth/login.html")]
pub struct LoginTemplate {
    pub error: Option<String>,

    pub has_user: bool,
    pub display_name: String,
    pub is_admin: bool,
}

#[derive(Template)]
#[template(path = "auth/register.html")]
pub struct RegisterTemplate {
    pub error: Option<String>,

    pub has_user: bool,
    pub display_name: String,
    pub is_admin: bool,
}

// Helper implementations
impl LoginTemplate {
    pub fn new(error: Option<String>) -> Self {
        Self {
            error,
            has_user: false,
            display_name: "Guest".to_string(),
            is_admin: false,
        }
    }
}

impl RegisterTemplate {
    pub fn new(error: Option<String>) -> Self {
        Self {
            error,
            has_user: false,
            display_name: "Guest".to_string(),
            is_admin: false,
        }
    }
}