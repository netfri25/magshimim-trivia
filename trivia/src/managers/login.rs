use chrono::NaiveDate;

use crate::db::Database;
use crate::email::Email;
use crate::messages::{Address, PhoneNumber};
use crate::password::Password;
use crate::username::Username;

pub struct LoginManager<'db, DB: ?Sized> {
    db: &'db DB,
    connected: Vec<Username>,
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
    ) -> Result<Option<Error>, crate::db::Error> {
        if self.db.user_exists(&username)? {
            return Ok(Some(Error::UserAlreadyExists(username))); // no error, but the user already exists
        }
        self.db
            .add_user(username, password, email, phone, address, birth_date)?;
        Ok(None) // everything is ok
    }

    pub fn login(
        &mut self,
        username: Username,
        password: Password,
    ) -> Result<Option<Error>, crate::db::Error> {
        if self.connected.iter().any(|logged| logged == &username) {
            return Ok(Some(Error::UserAlreadyConnected(username)));
        }

        if !self.db.user_exists(&username)? {
            return Ok(Some(Error::UserDoesntExist(username)));
        }

        if !self.db.password_matches(&username, &password)? {
            return Ok(Some(Error::WrongPassword));
        }

        self.connected.push(username);
        Ok(None)
    }

    pub fn logut(&mut self, username: &Username) {
        let Some(index) = self.connected.iter().position(|logged| logged == username) else {
            return;
        };

        // O(1) removal, but doesn't preserve ordering
        self.connected.swap_remove(index);
    }
}

#[derive(Debug, thiserror::Error)]
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
