use std::collections::HashMap;
use std::iter;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::anyhow;

use crate::db::question::QuestionData;
use crate::db::{self, Database};

use super::login::LoggedUser;
use super::room::Room;

pub type GameID = i64;

pub struct GameManager {
    db: Arc<Mutex<dyn Database>>,
    games: HashMap<GameID, Game>,
}

impl GameManager {
    pub fn new(db: Arc<Mutex<dyn Database>>) -> Self {
        Self {
            db,
            games: Default::default(),
        }
    }

    pub fn create_game(&mut self, room: Room) -> Result<&Game, db::Error> {
        let questions = self
            .db
            .lock()
            .unwrap()
            .get_questions(room.room_data().questions_count)?;
        let game = Game::new(room.users().iter().cloned(), questions);
        Ok(self.games.entry(game.id).or_insert(game))
    }

    pub fn delete_game(&mut self, id: &GameID) {
        self.games.remove(id);
    }

    pub fn submit_game_results(&mut self, game_id: &GameID) -> Result<(), db::Error> {
        let game = self
            .games
            .remove(game_id)
            .ok_or(anyhow!("game {game_id} doesn't exist"))?; // TODO: proper error

        game.players.into_iter().try_for_each(|(user, data)| {
            self.db
                .lock()
                .unwrap()
                .submit_game_data(user.username(), data)
        })
    }
}

pub struct Game {
    pub id: GameID,
    pub questions: Vec<QuestionData>,
    pub players: HashMap<LoggedUser, GameData>,
}

impl Game {
    pub fn new(users: impl Iterator<Item = LoggedUser>, questions: Vec<QuestionData>) -> Self {
        static COUNTER: Mutex<GameID> = Mutex::new(0);
        let id = {
            let mut counter = COUNTER.lock().unwrap();
            *counter += 1;
            *counter
        };

        let players = users.zip(iter::repeat_with(GameData::default)).collect();

        Self {
            id,
            players,
            questions,
        }
    }

    pub fn get_question_for_user(&mut self, user: LoggedUser) -> Option<&QuestionData> {
        let game_data = self.players.get_mut(&user)?;
        let index = game_data.current_question_index;
        game_data.current_question_index += 1;
        self.questions.get(index)
    }

    // returns the correct answer index
    pub fn submit_answer(
        &mut self,
        user: LoggedUser,
        answer_index: usize,
        answer_time: Duration,
    ) -> Result<usize, db::Error> {
        let game_data = self
            .players
            .get_mut(&user)
            .ok_or(db::Error::UserDoesntExist(user.username))?;

        // TODO: proper error
        let question = self
            .questions
            .get(game_data.current_question_index)
            .ok_or_else(|| anyhow!("CRITICAL ERROR: unexpected current question index"))?;

        let correct = question.correct_answer_index == answer_index;
        game_data.submit_answer(correct, answer_time);
        Ok(question.correct_answer_index)
    }

    pub fn remove_user(&mut self, user: &LoggedUser) {
        self.players.remove(user);
    }
}

#[derive(Debug, Default, Clone)]
pub struct GameData {
    pub current_question_index: usize,
    pub correct_answers: u32,
    pub wrong_answers: u32,
    pub avg_time: Duration,
}

impl GameData {
    pub fn submit_answer(&mut self, correct: bool, answer_time: Duration) {
        let old_total = self.correct_answers + self.wrong_answers;
        let old_total_time = self.avg_time.as_secs_f64() * old_total as f64;
        let total_time = old_total_time + answer_time.as_secs_f64();
        let avg_time = total_time / (old_total + 1) as f64;
        self.avg_time = Duration::from_secs_f64(avg_time);

        if correct {
            self.correct_answers += 1;
        } else {
            self.wrong_answers += 1;
        }
    }
}
