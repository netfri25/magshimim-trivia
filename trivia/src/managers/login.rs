use std::collections::HashSet;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::db::Database;
use crate::email::Email;
use crate::messages::{Address, PhoneNumber};
use crate::password::Password;
use crate::username::Username;

pub struct LoginManager<'db, DB: ?Sized> {
    db: &'db DB,
    connected: HashSet<Username>,
}

impl<'db, DB> LoginManager<'db, DB>
where
    DB: Database + Sync + ?Sized,
{
    pub fn new(db: &'db DB) -> Self {
        Self {
            db,
            connected: Default::default(),
        }
    }

    pub fn signup(
        &mut self,
        username: Username,
        password: Password,
        email: Email,
        phone: PhoneNumber,
        address: Address,
        birth_date: NaiveDate,
    ) -> Result<Result<(), Error>, crate::db::Error> {
        if self.db.user_exists(&username)? {
            return Ok(Err(Error::UserAlreadyExists(username))); // no error, but the user already exists
        }
        self.db
            .add_user(username, password, email, phone, address, birth_date)?;
        Ok(Ok(())) // everything is ok
    }

    pub fn login(
        &mut self,
        username: Username,
        password: Password,
    ) -> Result<Result<(), Error>, crate::db::Error> {
        if !self.db.user_exists(&username)? {
            return Ok(Err(Error::UserDoesntExist(username)));
        }

        if !self.db.password_matches(&username, &password)? {
            return Ok(Err(Error::WrongPassword));
        }

        if let Some(username) = self.connected.replace(username) {
            return Ok(Err(Error::UserAlreadyConnected(username)));
        }

        Ok(Ok(()))
    }

    pub fn logout(&mut self, username: &Username) {
        self.connected.remove(username);
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("user {:?} already connected", .0.as_ref())]
    UserAlreadyConnected(Username),

    #[error("user {:?} already exists", .0.as_ref())]
    UserAlreadyExists(Username),

    #[error("user {:?} doesn't exist", .0.as_ref())]
    UserDoesntExist(Username),

    #[error("wrong password")]
    WrongPassword,
}
