use std::sync::Arc;

use crate::managers::login::LoggedUser;
use crate::managers::room::RoomID;
use crate::messages::{Request, RequestResult};
use crate::messages::{RequestInfo, Response};

use super::{Error, Handler, RequestHandlerFactory};

pub struct RoomAdminRequestHandler {
    room_id: RoomID,
    admin: LoggedUser,
    factory: Arc<RequestHandlerFactory>,
}

impl Handler for RoomAdminRequestHandler {
    fn relevant(&self, request_info: &RequestInfo) -> bool {
        use Request::*;
        matches!(request_info.data, CloseRoom | StartGame | RoomState,)
    }

    fn handle(&mut self, request_info: RequestInfo) -> Result<RequestResult, Error> {
        match request_info.data {
            Request::CloseRoom => self.close_room(),
            Request::StartGame => self.start_game(),
            Request::RoomState => self.room_state(),
            _ => Ok(RequestResult::new_error("Invalid request")),
        }
    }
}

impl RoomAdminRequestHandler {
    pub fn new(factory: Arc<RequestHandlerFactory>, admin: LoggedUser, room_id: RoomID) -> Self {
        Self {
            room_id,
            admin,
            factory,
        }
    }

    pub fn close_room(&mut self) -> Result<RequestResult, Error> {
        let room_manager = self.factory.get_room_manager();
        if let Some(room) = room_manager.lock().unwrap().delete_room(self.room_id) {
            let users = room.users();
            // TODO: disoconnect all users
        };

        Ok(RequestResult::without_handler(Response::CloseRoom))
    }

    pub fn start_game(&mut self) -> Result<RequestResult, Error> {
        let room_manager = self.factory.get_room_manager();
        if let Some(room) = room_manager.lock().unwrap().room_mut(self.room_id) {
            todo!("mark the room as InGame");
        }

        Ok(RequestResult::without_handler(Response::StartGame))
    }

    pub fn room_state(&self) -> Result<RequestResult, Error> {
        let room_manager = self.factory.get_room_manager();
        let Some(room) = room_manager.lock().unwrap().room(self.room_id).cloned() else {
            return Ok(RequestResult::new_error("Room doesn't exist"));
        };

        Ok(RequestResult::without_handler(Response::RoomState {
            state: room.room_data().state,
            players: room.users().to_vec(),
            question_count: room.room_data().questions_count,
            time_per_question: room.room_data().time_per_question,
        }))
    }
}
