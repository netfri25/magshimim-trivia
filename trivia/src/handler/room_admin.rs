use std::sync::Arc;

use crate::managers::login::LoggedUser;
use crate::managers::room::{RoomID, RoomState};
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
            Request::Logout => self.close_room(),
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

    fn close_room(&mut self) -> Result<RequestResult, Error> {
        let room_manager = self.factory.get_room_manager();
        room_manager.lock().unwrap().delete_room(self.room_id);
        let resp = Response::CloseRoom;
        let handler = self.factory.create_menu_request_handler(self.admin.clone());
        Ok(RequestResult::new(resp, handler))
    }

    fn start_game(&mut self) -> Result<RequestResult, Error> {
        let room_manager = self.factory.get_room_manager();
        if !room_manager.lock().unwrap().set_state(self.room_id, RoomState::InGame) {
            return Ok(RequestResult::new_error("Room doesn't exist"))
        }

        let room_manager = self.factory.get_room_manager();
        let room_manager_lock = room_manager.lock().unwrap();

        let Some(room) = room_manager_lock.room(self.room_id) else {
            return Ok(RequestResult::new_error("Room doesn't exist"));
        };

        let game_id = self.factory.get_game_manager().lock().unwrap().create_game(room)?.id();

        drop(room_manager_lock);

        let resp = Response::StartGame;
        let handler = self.factory.create_game_request_handler(self.admin.clone(), game_id);
        Ok(RequestResult::new(resp, handler))
    }

    fn room_state(&self) -> Result<RequestResult, Error> {
        let room_manager = self.factory.get_room_manager();
        let Some(room) = room_manager.lock().unwrap().room(self.room_id).cloned() else {
            return Ok(RequestResult::new_error("Room doesn't exist"));
        };

        Ok(RequestResult::without_handler(Response::RoomState {
            state: room.room_data().state,
            name: room.room_data().name.clone(),
            players: room.users().to_vec(),
            question_count: room.room_data().questions_count,
            time_per_question: room.room_data().time_per_question,
        }))
    }
}
