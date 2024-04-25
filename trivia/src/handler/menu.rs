use std::sync::Arc;
use std::time::Duration;

use crate::managers::game::Score;
use crate::managers::login::LoggedUser;
use crate::managers::room::{RoomData, RoomID};
use crate::managers::statistics::Statistics;
use crate::messages::{Request, RequestInfo, RequestResult, Response};

use super::{Error, Handler, RequestHandlerFactory};

pub struct MenuRequestHandler {
    user: LoggedUser,
    factory: Arc<RequestHandlerFactory>,
}

impl Handler for MenuRequestHandler {
    fn relevant(&self, request_info: &RequestInfo) -> bool {
        let accepted = [
            Request::is_create_room,
            Request::is_room_list,
            Request::is_join_room,
            Request::is_statistics,
            Request::is_logout,
        ];

        accepted.iter().any(|f| f(&request_info.data))
    }

    fn handle(&mut self, request_info: RequestInfo) -> Result<RequestResult, Error> {
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

            _ => Ok(RequestResult::new_error("Invalid request")),
        }
    }
}

impl MenuRequestHandler {
    pub fn new(factory: Arc<RequestHandlerFactory>, user: LoggedUser) -> Self {
        Self { factory, user }
    }

    fn logout(&self) -> RequestResult {
        RequestResult::new(
            Response::Logout,
            self.factory.create_login_request_handler(),
        )
    }

    fn get_rooms(&self) -> RequestResult {
        let room_manager = self.factory.get_room_manager();
        let room_manager_lock = room_manager.lock().unwrap();
        let rooms = room_manager_lock.rooms().cloned().collect();
        let response = Response::RoomList(rooms);
        RequestResult::without_handler(response)
    }

    fn get_players_in_room(&self, id: RoomID) -> RequestResult {
        let room_manager = self.factory.get_room_manager();
        let room_manager_lock = room_manager.lock().unwrap();
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
        let statistics_manager_lock = statistics_manager.lock().unwrap();
        statistics_manager_lock.get_user_statistics(self.user.username())
    }

    fn get_high_scores(&self) -> Result<[Option<(String, Score)>; 5], crate::db::Error> {
        let statistics_manager = self.factory.get_statistics_manager();
        let statistics_manager_lock = statistics_manager.lock().unwrap();
        statistics_manager_lock.get_high_scores()
    }

    fn join_room(&self, id: RoomID) -> RequestResult {
        let room_manager = self.factory.get_room_manager();
        let mut room_manager_lock = room_manager.lock().unwrap();
        let Some(room) = room_manager_lock.room_mut(id) else {
            return RequestResult::new_error("invalid room ID");
        };

        if room.is_full() {
            return RequestResult::new_error("room is full");
        }

        room.add_user(self.user.clone());
        let resp = Response::JoinRoom;
        let handler = self
            .factory
            .create_room_member_request_handler(self.user.clone(), id);
        RequestResult::new(resp, handler)
    }

    fn create_room(
        &self,
        room_name: String,
        max_users: usize,
        questions: usize,
        answer_timeout: Duration,
    ) -> RequestResult {
        let room_data = RoomData::new(room_name, max_users, questions, answer_timeout);
        let id = room_data.room_id;
        let room_manager = self.factory.get_room_manager();
        let mut room_manager_lock = room_manager.lock().unwrap();
        room_manager_lock.create_room(self.user.clone(), room_data);
        let resp = Response::CreateRoom(id);
        let handler = self
            .factory
            .create_room_admin_request_handler(self.user.clone(), id);
        RequestResult::new(resp, handler)
    }
}
