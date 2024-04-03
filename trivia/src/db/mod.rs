pub mod sqlite;
pub use sqlite::SqliteDatabase;

pub mod question;
pub use question::Question;

pub mod opentdb;

pub trait Database: Send {
    fn open(&mut self) -> anyhow::Result<()>;

    // consumes the connection, meaning that it can't be used anymore
    fn close(self) -> anyhow::Result<()>;

    fn user_exists(&self, username: &str) -> anyhow::Result<bool>;
    fn password_matches(&self, username: &str, password: &str) -> anyhow::Result<bool>;
    fn add_user(&mut self, username: &str, password: &str, email: &str) -> anyhow::Result<()>;

    fn get_questions(&self, amount: u8) -> Vec<Question>;
}
