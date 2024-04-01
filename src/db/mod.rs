use std::error::Error;
use std::io;

pub mod sqlite;
pub use sqlite::SqliteDatabase;

type DBResult<T> = Result<T, Box<dyn Error + Send + Sync + 'static>>;

pub trait Database {
    fn open(&mut self) -> DBResult<()>;

    // consumes the connection, meaning that it can't be used anymore
    fn close(self) -> DBResult<()>;

    fn user_exists(&self, username: &str) -> DBResult<bool>;
    fn password_matches(&self, username: &str, password: &str) -> DBResult<bool>;
    fn add_user(&mut self, username: &str, password: &str, email: &str) -> DBResult<()>;
}
