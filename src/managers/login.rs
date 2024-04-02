use std::sync::{Arc, Mutex};

use crate::db::Database;

use super::logged_user::LoggedUser;

pub struct LoginManager {
    db: Arc<Mutex<dyn Database>>,
    connected: Vec<LoggedUser>,
}

impl LoginManager {
    pub fn new(db: Arc<Mutex<dyn Database>>) -> Self {
        Self {
            db,
            connected: Default::default(),
        }
    }

    pub fn signup(&mut self, username: impl Into<String>, password: &str, email: &str) -> anyhow::Result<()> {
        let username = username.into();
        self.db.lock().unwrap().add_user(&username, password, email)?;
        Ok(())
    }

    // TODO: return proper types to represent the outcome better
    pub fn login(&mut self, username: impl Into<String>, password: &str) -> anyhow::Result<Option<String>> {
        let username = username.into();
        if self.connected.iter().any(|user| user.username() == username) {
            return Ok(Some("user already connected".into()));
        }

        if !self.db.lock().unwrap().user_exists(&username)? {
            return Ok(Some("user doesn't exist".into()));
        }

        if !self.db.lock().unwrap().password_matches(&username, password)? {
            return Ok(Some("invalid password".into()))
        }

        self.connected.push(LoggedUser::new(username));
        Ok(None)
    }

    pub fn logut(&mut self, username: &str) {
        let Some(index) = self.connected.iter().position(|user| username == user.username()) else {
            return
        };

        // O(1) removal, but doesn't preserve ordering
        self.connected.swap_remove(index);
    }
}
