use std::collections::HashMap;
use std::iter;
use std::time::Duration;

use anyhow::anyhow;

use crate::db::question::QuestionData;
use crate::db::{self, Database};

use super::login::LoggedUser;
use super::room::{Room, RoomID};

pub type Score = f64;
pub type GameID = RoomID;

pub struct GameManager<'db> {
    db: &'db (dyn Database + Sync),
    games: HashMap<GameID, Game>,
}

impl<'db> GameManager<'db> {
    pub fn new(db: &'db (dyn Database + Sync)) -> Self {
        Self {
            db,
            games: Default::default(),
        }
    }

    pub fn create_game(&mut self, room: &Room) -> Result<&Game, db::Error> {
        let questions = self.db.get_questions(room.room_data().questions_count)?;
        let game = Game::new(
            room.room_data().room_id,
            room.users().iter().cloned(),
            questions,
            room.room_data().time_per_question,
        );
        Ok(self.games.entry(game.id).or_insert(game))
    }

    pub fn delete_game(&mut self, id: &GameID) {
        // I don't care if the submission fails
        self.submit_game_results(id).ok();
        self.games.remove(id);
    }

    pub fn game(&self, game_id: &GameID) -> Option<&Game> {
        self.games.get(game_id)
    }

    pub fn game_mut(&mut self, game_id: &GameID) -> Option<&mut Game> {
        self.games.get_mut(game_id)
    }

    pub fn submit_game_results(&mut self, game_id: &GameID) -> Result<(), db::Error> {
        let game = self
            .games
            .remove(game_id)
            .ok_or(anyhow!("game {game_id} doesn't exist"))?; // TODO: proper error

        game.players
            .into_iter()
            .try_for_each(|(user, data)| self.db.submit_game_data(user.username(), data))
    }
}

pub struct Game {
    id: GameID,
    questions: Vec<QuestionData>,
    time_per_question: Duration,
    players: HashMap<LoggedUser, GameData>,
}

impl Game {
    pub fn new(
        id: RoomID,
        users: impl Iterator<Item = LoggedUser>,
        questions: Vec<QuestionData>,
        time_per_question: Duration,
    ) -> Self {
        let players = users.zip(iter::repeat_with(GameData::default)).collect();

        Self {
            id,
            players,
            questions,
            time_per_question,
        }
    }

    pub fn id(&self) -> GameID {
        self.id
    }

    pub fn get_question_for_user(&mut self, user: &LoggedUser) -> Option<&QuestionData> {
        let game_data = self.players.get_mut(user)?;
        game_data.current_question_index += 1;
        let index = game_data.current_question_index - 1;
        self.questions.get(index)
    }

    // returns the correct answer index and goes to the next question
    pub fn submit_answer(
        &mut self,
        user: LoggedUser,
        answer: String,
        answer_time: Duration,
    ) -> Result<&str, db::Error> {
        let game_data = self
            .players
            .get_mut(&user)
            .ok_or(db::Error::UserDoesntExist(user.username))?;

        // TODO: proper error
        let question = self
            .questions
            .get(game_data.current_question_index - 1)
            .ok_or_else(|| anyhow!("CRITICAL ERROR: unexpected current question index"))?;

        if answer_time < self.time_per_question {
            let correct = question.correct_answer() == answer;
            game_data.submit_answer(correct, answer_time);
        }

        Ok(question.correct_answer())
    }

    pub fn remove_user(&mut self, user: &LoggedUser) {
        if let Some(data) = self.players.get_mut(user) {
            // mark as if the user has finished
            data.left = true;
        }
    }

    pub fn users(&self) -> impl Iterator<Item = &LoggedUser> {
        self.players.keys()
    }

    pub fn is_empty(&self) -> bool {
        self.players.values().all(|data| data.left)
    }

    // NOTE: can be optimized, but I don't really care about performance
    pub fn all_finished(&self) -> bool {
        // because that I'm using (current_question_index - 1) then I'm comparing with `>` instead of `>=`
        self.players
            .values()
            .all(|data| data.left || data.current_question_index > self.questions.len())
    }

    pub fn results(&self) -> impl Iterator<Item = (&LoggedUser, &GameData)> {
        self.players.iter()
    }
}

#[derive(Debug, Default, Clone)]
pub struct GameData {
    pub current_question_index: usize,
    pub correct_answers: u32,
    pub wrong_answers: u32,
    pub avg_time: Duration,
    pub left: bool,
}

impl GameData {
    pub fn submit_answer(&mut self, correct: bool, answer_time: Duration) {
        self.left = false; // if the user left and came back
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

pub fn calc_score(answer_time: Duration, correct_answers: i64) -> Score {
    // TODO: the user can just spam wrong answers and still get a really good score
    //       find a way to prevent this, meaning a new score evaluation algorithm
    let score = correct_answers as f64 / answer_time.as_secs_f64();

    if score.is_normal() {
        score
    } else {
        0.
    }
}
