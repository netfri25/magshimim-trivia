use std::sync::Arc;

use crate::managers::login::LoggedUser;
use crate::managers::room::Room;
use crate::messages::{Request, RequestResult};
use crate::messages::{RequestInfo, Response};

use super::{Error, Handler, RequestHandlerFactory};

pub struct RoomAdminRequestHandler {
    room: Room,
    admin: LoggedUser,
    factory: Arc<RequestHandlerFactory>,
}

impl Handler for RoomAdminRequestHandler {
    fn relevant(&self, request_info: &RequestInfo) -> bool {
        use Request::*;
        matches!(
            request_info.data,
            CloseRoom | StartGame | RoomState,
        )
    }

    fn handle(&mut self, request_info: RequestInfo) -> Result<RequestResult, Error> {
        todo!()
    }
}

impl RoomAdminRequestHandler {
    pub fn new(factory: Arc<RequestHandlerFactory>, admin: LoggedUser, room: Room) -> Self {
        Self {
            room,
            admin,
            factory,
        }
    }

    pub fn close_room(&mut self) -> Result<RequestResult, Error> {
        todo!()
    }

    pub fn start_game(&mut self) -> Result<RequestResult, Error> {
        todo!()
    }

    pub fn room_state(&self) -> Result<RequestResult, Error> {
        Ok(RequestResult::without_handler(Response::RoomState {
            state: self.room.room_data().state,
            players: self.room.users().to_vec(),
            question_count: self.room.room_data().questions_count,
            time_per_question: self.room.room_data().time_per_question,
        }))
    }
}
