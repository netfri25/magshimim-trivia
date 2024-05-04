use std::borrow::Cow;
use std::time::Duration;

use crate::db::question::QuestionData;
use crate::db::Database;
use crate::managers::room::{RoomData, RoomID, RoomState};
use crate::managers::statistics::{Highscores, Statistics};
use crate::messages::{Request, RequestInfo, RequestResult, Response};
use crate::username::Username;

use super::{Error, Handler, RequestHandlerFactory};

pub struct MenuRequestHandler<'db, 'factory, DB: ?Sized> {
    user: Username,
    factory: &'factory RequestHandlerFactory<'db, DB>,
}

impl<'db, 'factory: 'db, DB> Handler<'db> for MenuRequestHandler<'db, 'factory, DB>
where
    DB: Database + Sync + ?Sized,
{
    fn relevant(&self, request_info: &RequestInfo) -> bool {
        use Request::*;
        matches!(
            request_info.data,
            CreateRoom { .. } | RoomList | JoinRoom(_) | Statistics | CreateQuestion(_) | Logout
        )
    }

    fn handle(&mut self, request_info: RequestInfo) -> Result<RequestResult<'db>, Error> {
        match request_info.data {
            Request::JoinRoom(id) => Ok(self.join_room(id)),
            Request::CreateRoom {
                name,
                max_users,
                questions,
                answer_timeout,
            } => Ok(self.create_room(name, max_users, questions, answer_timeout)),
            Request::Statistics => {
                let user_statistics = self.get_personal_stats()?;
                let high_scores = self.get_high_scores()?;
                let response = Response::Statistics {
                    user_statistics,
                    high_scores,
                };
                let result = RequestResult::without_handler(response);
                Ok(result)
            }
            Request::Logout => Ok(self.logout()),
            Request::RoomList => Ok(self.get_rooms()),
            Request::CreateQuestion(question) => self.create_question(question),

            _ => Ok(RequestResult::new_error("Invalid request")),
        }
    }
}

impl<'db, 'factory: 'db, DB> MenuRequestHandler<'db, 'factory, DB>
where
    DB: Database + Sync + ?Sized,
{
    pub fn new(factory: &'factory RequestHandlerFactory<'db, DB>, user: Username) -> Self {
        Self { factory, user }
    }

    fn logout(&self) -> RequestResult<'db> {
        RequestResult::new(
            Response::Logout,
            self.factory.create_login_request_handler(),
        )
    }

    fn get_rooms(&self) -> RequestResult<'db> {
        let room_manager = self.factory.get_room_manager();
        let room_manager_lock = room_manager.read().unwrap();
        let rooms = room_manager_lock.rooms().cloned().collect();
        let response = Response::RoomList(rooms);
        RequestResult::without_handler(response)
    }

    #[allow(unused)]
    fn get_players_in_room(&self, id: RoomID) -> RequestResult {
        let room_manager = self.factory.get_room_manager();
        let room_manager_lock = room_manager.read().unwrap();
        if let Some(room) = room_manager_lock.room(id) {
            let users = room.users().to_vec();
            let response = Response::PlayersInRoom(users);
            RequestResult::without_handler(response)
        } else {
            RequestResult::new_error("invalid room ID")
        }
    }

    fn get_personal_stats(&self) -> Result<Statistics, crate::db::Error> {
        let statistics_manager = self.factory.get_statistics_manager();
        let statistics_manager_lock = statistics_manager.read().unwrap();
        statistics_manager_lock.get_user_statistics(&self.user)
    }

    fn get_high_scores(&self) -> Result<Highscores, crate::db::Error> {
        let statistics_manager = self.factory.get_statistics_manager();
        let statistics_manager_lock = statistics_manager.read().unwrap();
        statistics_manager_lock.get_high_scores()
    }

    fn join_room(&self, id: RoomID) -> RequestResult<'db> {
        let room_manager = self.factory.get_room_manager();
        let room_manager_lock = room_manager.read().unwrap();
        let Some(room) = room_manager_lock.room(id) else {
            return RequestResult::new_error("invalid room ID");
        };

        if room.is_full() {
            return RequestResult::new_error("room is full");
        }

        if room.room_data().state == RoomState::InGame {
            return RequestResult::new_error("can't join a room that is already in game");
        }

        drop(room_manager_lock);
        let mut room_manager_lock = room_manager.write().unwrap();
        let Some(room) = room_manager_lock.room_mut(id) else {
            return RequestResult::new_error("invalid room ID"); // should never reach here
        };

        room.add_user(self.user.clone());
        let resp = Response::JoinRoom;
        let handler = self
            .factory
            .create_room_user_request_handler(self.user.clone(), false, id);
        RequestResult::new(resp, handler)
    }

    fn create_room(
        &self,
        room_name: Cow<str>,
        max_users: usize,
        questions: usize,
        answer_timeout: Duration,
    ) -> RequestResult<'db> {
        let room_data = RoomData::new(room_name, max_users, questions, answer_timeout);
        let id = room_data.room_id;
        let room_manager = self.factory.get_room_manager();
        let mut room_manager_lock = room_manager.write().unwrap();
        room_manager_lock.create_room(self.user.clone(), room_data);
        let resp = Response::CreateRoom;
        let handler = self
            .factory
            .create_room_user_request_handler(self.user.clone(), true, id);
        RequestResult::new(resp, handler)
    }

    fn create_question(&self, question: QuestionData) -> Result<RequestResult<'db>, Error> {
        let added = self.factory.db().add_question(&question)?;
        if !added {
            Ok(RequestResult::new_error("question already exists"))
        } else {
            let resp = Response::CreateQuestion;
            Ok(RequestResult::without_handler(resp))
        }
    }
}
