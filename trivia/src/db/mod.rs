use std::time::Duration;

use chrono::NaiveDate;

use crate::email::Email;
use crate::managers::game::{GameData, Score};
use crate::managers::statistics::Highscores;
use crate::messages::{Address, PhoneNumber};
use crate::password::Password;
use crate::username::Username;

pub mod turbosql;
pub use turbosql::TurboSqliteDatabase;

pub mod question;
pub use question::QuestionData;

pub mod opentdb;

// TODO: should I switch every username parameter from &str to &Username?
pub trait Database {
    fn user_exists(&self, username: &Username) -> Result<bool, Error>;
    fn password_matches(&self, username: &Username, password: &Password) -> Result<bool, Error>;
    fn add_user(
        &self,
        username: Username,
        password: Password,
        email: Email,
        phone: PhoneNumber,
        address: Address,
        birth_date: NaiveDate,
    ) -> Result<(), Error>;

    fn get_questions(&self, amount: usize) -> Result<Vec<QuestionData>, Error>;
    fn get_player_average_answer_time(&self, username: &Username) -> Result<Duration, Error>;
    fn get_correct_answers_count(&self, username: &Username) -> Result<i64, Error>;
    fn get_total_answers_count(&self, username: &Username) -> Result<i64, Error>;
    fn get_games_count(&self, username: &Username) -> Result<i64, Error>;
    fn get_score(&self, username: &Username) -> Result<Score, Error>;
    fn get_five_highscores(&self) -> Result<Highscores, Error>;

    fn submit_game_data(&self, username: &Username, game_data: GameData) -> Result<(), Error>;

    // the Ok variant tells if the question was added
    fn add_question(&self, question: &QuestionData) -> Result<bool, Error>;

    fn populate_questions(&self, amount: u8) -> Result<(), Error> {
        opentdb::get_questions(amount)?
            .into_iter()
            .try_for_each(|question| self.add_question(&QuestionData::from(question)).map(drop))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("user {0:?} doesn't exist")]
    UserDoesntExist(Username),

    #[error("no correct answer for ({question_id}) {question_content:?}")]
    NoCorrectAnswer {
        question_id: i64,
        question_content: String,
    },

    #[error("DB: {0}")]
    InternalDBError(#[from] ::turbosql::Error),

    #[error("OpenTDB: {0}")]
    OpenTDB(#[from] opentdb::Error),

    #[error("invalid username: {0}")]
    Username(#[from] crate::username::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
