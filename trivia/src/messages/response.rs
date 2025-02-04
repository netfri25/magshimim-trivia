use std::io::{Read, Write};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::db::question::QuestionData;
use crate::handler::{self, Handler};
use crate::managers::game::calc_score;
use crate::managers::room::{Room, RoomState};
use crate::managers::statistics::{Highscores, Statistics};
use crate::username::Username;

use super::Error;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Response {
    Error(String),
    Login(Result<(), handler::login::Error>),
    Signup(Result<(), handler::login::Error>),
    Logout,
    RoomList(Vec<Room>),
    PlayersInRoom(Result<Vec<Username>, handler::menu::Error>),
    JoinRoom(Result<(), handler::menu::Error>),
    CreateRoom,
    PersonalStats(Result<Statistics, handler::menu::Error>),
    Highscores(Highscores),
    CloseRoom(Result<(), handler::room_user::Error>),
    StartGame(Result<(), handler::room_user::Error>),
    RoomState {
        state: RoomState,
        name: String,
        players: Vec<Username>,
        question_count: usize,
        time_per_question: Duration,
    },
    LeaveRoom,
    LeaveGame,
    CorrectAnswer(String),

    // the `correct_answer_index` will be set to usize::MAX so that the client can't cheat
    // additionally, the answers will be shuffled when sent to the user
    Question(Result<Option<QuestionData>, handler::game::Error>), // None => no more questions
    GameResult(Vec<PlayerResults>), // Will be sent to everyone when the game is over
    CreateQuestion(Result<(), handler::menu::Error>),
}

impl Response {
    pub fn read_from(reader: &mut impl Read) -> Result<Self, Error> {
        let mut buf_data_len = [0; 4];
        reader.read_exact(&mut buf_data_len)?;
        let data_len = u32::from_le_bytes(buf_data_len);
        let data_len = data_len as usize;

        let mut buf = vec![0; data_len];
        reader.read_exact(&mut buf)?;

        let response = serde_json::from_slice(&buf)?;
        Ok(response)
    }

    pub fn write_to(&self, writer: &mut impl Write) -> Result<(), Error> {
        let json = serde_json::to_vec(self)?;
        let len = json.len() as u32;
        let len_bytes = len.to_le_bytes();
        writer.write_all(&len_bytes)?;
        writer.write_all(&json)?;
        Ok(())
    }

    pub fn new_error(msg: impl ToString) -> Self {
        let msg = msg.to_string();
        Self::Error(msg)
    }
}

pub struct RequestResult<'db> {
    pub response: Response,
    pub new_handler: Option<Box<dyn Handler<'db> + 'db>>,
}

impl<'db> RequestResult<'db> {
    pub fn new(response: Response, new_handler: impl Handler<'db> + 'db) -> Self {
        let new_handler = Some(Box::new(new_handler) as Box<dyn Handler>);
        Self {
            response,
            new_handler,
        }
    }

    pub fn without_handler(response: Response) -> Self {
        Self {
            response,
            new_handler: None,
        }
    }

    pub fn new_error(msg: impl ToString) -> Self {
        Self::without_handler(Response::new_error(msg))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerResults {
    pub username: Username,
    pub correct_answers: u32,
    pub wrong_answers: u32,
    pub avg_time: Duration,
    pub score: f64,
}

impl PlayerResults {
    pub fn new(
        username: Username,
        correct_answers: u32,
        wrong_answers: u32,
        avg_time: Duration,
    ) -> Self {
        let score = calc_score(avg_time, correct_answers as i64);
        Self {
            username,
            correct_answers,
            wrong_answers,
            avg_time,
            score,
        }
    }
}
