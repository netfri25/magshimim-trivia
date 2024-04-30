use serde::{Deserialize, Serialize};

use crate::db::Database;

pub struct LoginManager<'db> {
    db: &'db (dyn Database + Sync),
    connected: Vec<LoggedUser>,
}

impl<'db> LoginManager<'db> {
    pub fn new(db: &'db (dyn Database + Sync)) -> Self {
        Self {
            db,
            connected: Default::default(),
        }
    }

    pub fn signup(
        &mut self,
        username: impl Into<String>,
        password: &str,
        email: &str,
    ) -> Result<Option<String>, crate::db::Error> {
        let username = username.into();

        if self.db.user_exists(&username)? {
            return Ok(Some("username already exists".into())); // no error, but the user already exists
        }

        self.db.add_user(&username, password, email)?;
        Ok(None) // everything is ok
    }

    // TODO: return proper types to represent the outcome better
    pub fn login(
        &mut self,
        username: impl Into<String>,
        password: &str,
    ) -> Result<Option<String>, crate::db::Error> {
        let username = username.into();
        if self
            .connected
            .iter()
            .any(|user| user.username() == username)
        {
            return Ok(Some("user already connected".into()));
        }

        if !self.db.user_exists(&username)? {
            return Ok(Some("user doesn't exist".into()));
        }

        if !self.db.password_matches(&username, password)? {
            return Ok(Some("invalid password".into()));
        }

        self.connected.push(LoggedUser::new(username));
        Ok(None)
    }

    pub fn logut(&mut self, username: &str) {
        let Some(index) = self
            .connected
            .iter()
            .position(|user| username == user.username())
        else {
            return;
        };

        // O(1) removal, but doesn't preserve ordering
        self.connected.swap_remove(index);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LoggedUser {
    pub username: String,
}

impl LoggedUser {
    pub fn new(username: impl Into<String>) -> Self {
        let username = username.into();
        Self { username }
    }

    pub fn username(&self) -> &str {
        &self.username
    }
}

impl PartialEq<str> for LoggedUser {
    fn eq(&self, other: &str) -> bool {
        self.username() == other
    }
}
