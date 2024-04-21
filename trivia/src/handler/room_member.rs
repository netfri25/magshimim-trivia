use std::sync::Arc;

use crate::managers::login::LoggedUser;
use crate::managers::room::RoomID;
use crate::messages::{Request, RequestInfo, RequestResult, Response};

use super::{Error, Handler, RequestHandlerFactory};

pub struct RoomMemberRequestHandler {
    room_id: RoomID,
    member: LoggedUser,
    factory: Arc<RequestHandlerFactory>,
}

impl Handler for RoomMemberRequestHandler {
    fn relevant(&self, request_info: &RequestInfo) -> bool {
        use Request::*;
        matches!(request_info.data, LeaveRoom | RoomState)
    }

    fn handle(&mut self, request_info: RequestInfo) -> Result<RequestResult, Error> {
        match request_info.data {
            Request::LeaveRoom => self.leave_room(),
            Request::RoomState => self.room_state(),
            _ => Ok(RequestResult::new_error("Invalid request")),
        }
    }
}

impl RoomMemberRequestHandler {
    pub fn new(factory: Arc<RequestHandlerFactory>, member: LoggedUser, room_id: RoomID) -> Self {
        Self {
            factory,
            member,
            room_id,
        }
    }

    fn leave_room(&self) -> Result<RequestResult, Error> {
        let room_manager = self.factory.get_room_manager();
        let mut room_manager_lock = room_manager.lock().unwrap();
        if let Some(room) = room_manager_lock.room_mut(self.room_id) {
            room.remove_user(&self.member);
            // TODO: think about which handler should be used in here
            Ok(RequestResult::without_handler(Response::LeaveRoom))
        } else {
            Ok(RequestResult::new_error("Room doesn't exist"))
        }
    }

    fn room_state(&self) -> Result<RequestResult, Error> {
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
