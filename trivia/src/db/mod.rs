pub mod sqlite;
use std::time::Duration;

pub use sqlite::SqliteDatabase;

pub mod question;
use question::QuestionData;

pub mod opentdb;


pub type Score = f64;

pub trait Database: Send {
    fn open(&mut self) -> anyhow::Result<()>;

    // consumes the connection, meaning that it can't be used anymore
    fn close(self) -> anyhow::Result<()>;

    fn user_exists(&self, username: &str) -> anyhow::Result<bool>;
    fn password_matches(&self, username: &str, password: &str) -> anyhow::Result<bool>;
    fn add_user(&mut self, username: &str, password: &str, email: &str) -> anyhow::Result<()>;

    fn get_questions(&self, amount: u8) -> anyhow::Result<Vec<QuestionData>>;
    fn get_player_average_answer_time(&self, username: &str) -> anyhow::Result<Duration>;
    fn get_correct_answers_count(&self, username: &str) -> anyhow::Result<i64>;
    fn get_total_answers_count(&self, username: &str) -> anyhow::Result<i64>;
    fn get_games_count(&self, username: &str) -> anyhow::Result<i64>;
    fn get_score(&self, username: &str) -> anyhow::Result<Score>;

    // if there are less than 5 scores, it will be filled with zeros
    fn get_five_highscores(&self) -> anyhow::Result<[Score; 5]>;
}
