use std::sync::{Arc, Mutex};

use crate::db::Database;

use super::logged_user::LoggedUser;

type ManagerResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub struct LoginManager {
    db: Arc<Mutex<dyn Database + Send + Sync>>,
    connected: Vec<LoggedUser>,
}

impl LoginManager {
    pub fn new(db: Arc<Mutex<dyn Database + Send + Sync>>) -> Self {
        Self {
            db,
            connected: Default::default(),
        }
    }

    pub fn signup(&mut self, username: impl Into<String>, password: &str, email: &str) -> ManagerResult<()> {
        let username = username.into();
        self.db.lock().unwrap().add_user(&username, password, email)?;
        Ok(())
    }

    pub fn login(&mut self, username: impl Into<String>) {
        self.connected.push(LoggedUser::new(username));
    }

    pub fn logut(&mut self, username: &str) {
        let Some(index) = self.connected.iter().position(|user| username == user.username()) else {
            return
        };

        // O(1) removal, but doesn't preserve ordering
        self.connected.swap_remove(index);
    }
}
