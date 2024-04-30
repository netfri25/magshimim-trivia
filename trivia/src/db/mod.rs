pub mod sqlite;
use std::time::Duration;

pub use sqlite::SqliteDatabase;

pub mod question;
use question::QuestionData;

use crate::managers::game::{GameData, Score};
use crate::managers::statistics::Highscores;

pub mod opentdb;

pub trait Database {
    fn user_exists(&self, username: &str) -> Result<bool, Error>;
    fn password_matches(&self, username: &str, password: &str) -> Result<bool, Error>;
    fn add_user(&self, username: &str, password: &str, email: &str) -> Result<(), Error>;

    fn get_questions(&self, amount: usize) -> Result<Vec<QuestionData>, Error>;
    fn get_player_average_answer_time(&self, username: &str) -> Result<Duration, Error>;
    fn get_correct_answers_count(&self, username: &str) -> Result<i64, Error>;
    fn get_total_answers_count(&self, username: &str) -> Result<i64, Error>;
    fn get_games_count(&self, username: &str) -> Result<i64, Error>;
    fn get_score(&self, username: &str) -> Result<Score, Error>;
    fn get_five_highscores(&self) -> Result<Highscores, Error>;

    fn submit_game_data(&self, username: &str, game_data: GameData) -> Result<(), Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("user {0:?} doesn't exist")]
    UserDoesntExist(String),

    #[error("no correct answer for ({question_id}) {question_content:?}")]
    NoCorrectAnswer {
        question_id: i64,
        question_content: String,
    },

    #[error("DB: {0}")]
    InternalDBError(#[from] ::sqlite::Error),

    #[error("OpenTDB: {0}")]
    OpenTDB(#[from] opentdb::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
