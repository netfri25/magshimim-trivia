use std::borrow::Cow;
use std::time::{self, Duration, Instant, SystemTime};

use serde::{Deserialize, Serialize};
use tiny_rng::{Rand, Rng};

use crate::db::Database;
use crate::managers::game::GameID;
use crate::messages::{PlayerResults, Request, RequestInfo, RequestResult, Response};
use crate::username::Username;

use super::{Handler, RequestHandlerFactory};

pub struct GameRequestHandler<'db, 'factory, DB: ?Sized> {
    game_id: GameID,
    user: Username,
    question_sent_at: Instant,
    factory: &'factory RequestHandlerFactory<'db, DB>,
}

impl<'db, 'factory: 'db, DB> Handler<'db> for GameRequestHandler<'db, 'factory, DB>
where
    DB: Database + Sync + ?Sized,
{
    fn relevant(&self, request_info: &RequestInfo) -> bool {
        use Request::*;
        matches!(
            request_info.data,
            LeaveGame | Question | SubmitAnswer(_) | GameResult
        )
    }

    fn handle(&mut self, request_info: RequestInfo) -> Result<RequestResult<'db>, super::Error> {
        match request_info.data {
            Request::Question => {
                self.question_sent_at = Instant::now();
                self.get_question()
            }
            Request::SubmitAnswer(answer) => self.submit_answer(
                answer,
                request_info.time.duration_since(self.question_sent_at),
            ),
            Request::GameResult => self.game_results(),
            Request::Logout | Request::LeaveGame => self.leave_game(),
            _ => Ok(RequestResult::new_error("Invalid request")),
        }
    }
}

impl<'db, 'factory: 'db, DB> GameRequestHandler<'db, 'factory, DB>
where
    DB: Database + Sync + ?Sized,
{
    pub fn new(
        factory: &'factory RequestHandlerFactory<'db, DB>,
        user: Username,
        game_id: GameID,
    ) -> Self {
        Self {
            game_id,
            user,
            question_sent_at: Instant::now(),
            factory,
        }
    }

    fn get_question(&self) -> Result<RequestResult<'db>, super::Error> {
        let game_manager = self.factory.game_manager();
        let mut game_manager_lock = game_manager.write().unwrap();
        let Some(game) = game_manager_lock.game_mut(&self.game_id) else {
            let resp = Response::Question(Err(Error::UnknownGameID(self.game_id)));
            return Ok(RequestResult::without_handler(resp));
        };

        let mut question = game.get_question_for_user(&self.user).cloned();

        drop(game_manager_lock);

        // change any information that can give away the correct answer
        if let Some(ref mut question) = question {
            let mut rng = Rng::from_seed(
                SystemTime::now()
                    .duration_since(time::UNIX_EPOCH)
                    .expect("clock can't go back from 0")
                    .as_secs(),
            );
            question.correct_answer_index = usize::MAX;
            rng.shuffle(question.answers.as_mut_slice())
        }

        Ok(RequestResult::without_handler(Response::Question(Ok(
            question,
        ))))
    }

    fn leave_game(&self) -> Result<RequestResult<'db>, super::Error> {
        let game_manager = self.factory.game_manager();
        let mut game_manager_lock = game_manager.write().unwrap();
        if let Some(game) = game_manager_lock.game_mut(&self.game_id) {
            game.remove_user(&self.user);

            // no more players are left
            if game.is_empty() {
                game_manager_lock.delete_game(&self.game_id);
                self.factory
                    .room_manager()
                    .write()
                    .unwrap()
                    .delete_room(self.game_id);
            }
        };

        drop(game_manager_lock);

        let resp = Response::LeaveGame;
        let handler = self.factory.create_menu_request_handler(self.user.clone());

        Ok(RequestResult::new(resp, handler))
    }

    #[allow(redundant_semicolons, unused_parens)]
    fn submit_answer(
        &self,
        answer: Cow<str>,
        answer_duration: Duration,
    ) -> Result<RequestResult<'db>, super::Error> {
        let game_manager = self.factory.game_manager();
        let mut game_manager_lock = game_manager.write().unwrap();
        let Some(game) = game_manager_lock.game_mut(&self.game_id) else {
            let resp = Response::Question(Err(Error::UnknownGameID(self.game_id)));
            return Ok(RequestResult::without_handler(resp));
        };

        let correct_answer = game
            .submit_answer(self.user.clone(), answer, answer_duration)?
            .to_string();

        let resp = Response::CorrectAnswer(correct_answer);
        Ok(RequestResult::without_handler(resp))
    }

    fn game_results(&self) -> Result<RequestResult<'db>, super::Error> {
        let game_manager = self.factory.game_manager();
        let game_manager_lock = game_manager.read().unwrap();
        let Some(game) = game_manager_lock.game(&self.game_id) else {
            return Ok(RequestResult::without_handler(Response::LeaveGame));
        };

        if game.all_finished() {
            let mut results: Vec<_> = game
                .results()
                .map(|(user, data)| {
                    PlayerResults::new(
                        user.clone(),
                        data.correct_answers,
                        data.wrong_answers,
                        data.avg_time,
                    )
                })
                .collect();
            results.sort_by(|res1, res2| res1.score.total_cmp(&res2.score).reverse());

            drop(game_manager_lock);
            self.leave_game()?;

            let resp = Response::GameResult(results);
            let handler = self.factory.create_menu_request_handler(self.user.clone());
            Ok(RequestResult::new(resp, handler))
        } else {
            let resp = Response::GameResult(vec![]);
            Ok(RequestResult::without_handler(resp))
        }
    }
}

#[derive(Debug, Serialize, Deserialize, thiserror::Error, PartialEq)]
pub enum Error {
    #[error("unknown game id {0}")]
    UnknownGameID(GameID),
}
