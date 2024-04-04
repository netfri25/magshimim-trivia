use std::sync::Arc;
use std::time::Duration;

use crate::db::Score;
use crate::managers::login::LoggedUser;
use crate::managers::room::RoomID;
use crate::managers::statistics::Statistics;
use crate::messages::{Request, RequestInfo, RequestResult, Response};

use super::{Handler, RequestHandlerFactory};

pub struct MenuRequestHandler {
    user: LoggedUser,
    factory: Arc<RequestHandlerFactory>,
}

impl MenuRequestHandler {
    pub fn new(factory: Arc<RequestHandlerFactory>, user: LoggedUser) -> Self {
        Self { factory, user }
    }

    fn logout(&self) -> anyhow::Result<RequestResult> {
        todo!()
    }

    fn get_rooms(&self) -> anyhow::Result<RequestResult> {
        todo!()
    }

    fn get_players_in_room(&self, id: RoomID) -> anyhow::Result<RequestResult> {
        todo!()
    }

    fn get_personal_stats(&self) -> anyhow::Result<Statistics> {
        todo!()
    }

    fn get_high_scores(&self) -> anyhow::Result<[Score; 5]> {
        todo!()
    }

    fn join_room(&self, id: RoomID) -> anyhow::Result<RequestResult> {
        todo!()
    }

    fn create_room(
        &self,
        room_name: String,
        max_users: usize,
        questions: usize,
        answer_timeout: Duration,
    ) -> anyhow::Result<RequestResult> {
        todo!()
    }
}

impl Handler for MenuRequestHandler {
    fn relevant(&self, request_info: &RequestInfo) -> bool {
        let accepted = [
            Request::is_create_room,
            Request::is_room_list,
            Request::is_players_in_room,
            Request::is_join_room,
            Request::is_statistics,
            Request::is_logout,
        ];

        accepted.iter().any(|f| f(&request_info.data))
    }

    fn handle(&mut self, request_info: RequestInfo) -> anyhow::Result<RequestResult> {
        match request_info.data {
            Request::PlayersInRoom(room_id) => self.get_players_in_room(room_id),
            Request::JoinRoom(id) => self.join_room(id),
            Request::CreateRoom {
                name,
                max_users,
                questions,
                answer_timeout,
            } => self.create_room(name, max_users, questions, answer_timeout),
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
            Request::Logout => self.logout(),
            Request::RoomList => self.get_rooms(),

            _ => Ok(RequestResult::new_error("Invalid request")),
        }
    }
}
