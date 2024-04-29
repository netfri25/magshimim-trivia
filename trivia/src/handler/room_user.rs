use crate::managers::login::LoggedUser;
use crate::managers::room::{RoomID, RoomState};
use crate::messages::{Request, RequestInfo, RequestResult, Response};

use super::{Error, Handler, RequestHandlerFactory};

pub struct RoomUserRequestHandler<'db, 'factory> {
    room_id: RoomID,
    user: LoggedUser,
    is_admin: bool,
    factory: &'factory RequestHandlerFactory<'db>,
}

impl<'db, 'factory: 'db> Handler<'db> for RoomUserRequestHandler<'db, 'factory> {
    fn relevant(&self, request_info: &RequestInfo) -> bool {
        use Request::*;
        matches!(
            request_info.data,
            LeaveRoom | RoomState | StartGame | CloseRoom
        )
    }

    fn handle(&mut self, request_info: RequestInfo) -> Result<RequestResult<'db>, Error> {
        match request_info.data {
            Request::RoomState => self.room_state(),
            Request::CloseRoom => self.close_room(),
            Request::StartGame => self.start_game(),
            Request::LeaveRoom | Request::Logout => self.leave_room(),
            _ => Ok(RequestResult::new_error("Invalid request")),
        }
    }
}

impl<'db, 'factory: 'db> RoomUserRequestHandler<'db, 'factory> {
    pub fn new(
        factory: &'factory RequestHandlerFactory<'db>,
        user: LoggedUser,
        is_admin: bool,
        room_id: RoomID,
    ) -> Self {
        Self {
            factory,
            user,
            is_admin,
            room_id,
        }
    }

    fn leave_room(&mut self) -> Result<RequestResult<'db>, Error> {
        let room_manager = self.factory.get_room_manager();
        let mut room_manager_lock = room_manager.lock().unwrap();
        if let Some(room) = room_manager_lock.room_mut(self.room_id) {
            room.remove_user(&self.user);
            if room.is_empty() {
                room_manager_lock.delete_room(self.room_id);
            }
        }

        drop(room_manager_lock);

        let resp = Response::LeaveRoom;
        let handler = self.factory.create_menu_request_handler(self.user.clone());
        Ok(RequestResult::new(resp, handler))
    }

    fn room_state(&mut self) -> Result<RequestResult<'db>, Error> {
        let room_manager = self.factory.get_room_manager();
        let Some(room) = room_manager.lock().unwrap().room(self.room_id).cloned() else {
            return self.leave_room();
        };

        if room.room_data().state == RoomState::InGame {
            let resp = Response::StartGame;
            let handler = self
                .factory
                .create_game_request_handler(self.user.clone(), self.room_id);
            return Ok(RequestResult::new(resp, handler));
        }

        Ok(RequestResult::without_handler(Response::RoomState {
            state: room.room_data().state,
            name: room.room_data().name.clone(),
            players: room.users().to_vec(),
            question_count: room.room_data().questions_count,
            time_per_question: room.room_data().time_per_question,
        }))
    }

    fn close_room(&mut self) -> Result<RequestResult<'db>, Error> {
        if !self.is_admin {
            return Ok(RequestResult::new_error(
                "only the room admin can close the room",
            ));
        }

        let room_manager = self.factory.get_room_manager();
        room_manager.lock().unwrap().delete_room(self.room_id);
        let resp = Response::CloseRoom;
        let handler = self.factory.create_menu_request_handler(self.user.clone());
        Ok(RequestResult::new(resp, handler))
    }

    fn start_game(&mut self) -> Result<RequestResult<'db>, Error> {
        if !self.is_admin {
            return Ok(RequestResult::new_error(
                "only the room admin can start the game",
            ));
        }

        let room_manager = self.factory.get_room_manager();
        if !room_manager
            .lock()
            .unwrap()
            .set_state(self.room_id, RoomState::InGame)
        {
            return Ok(RequestResult::new_error("Room doesn't exist"));
        }

        let room_manager = self.factory.get_room_manager();
        let room_manager_lock = room_manager.lock().unwrap();

        let Some(room) = room_manager_lock.room(self.room_id) else {
            return Ok(RequestResult::new_error("Room doesn't exist"));
        };

        let game_id = self
            .factory
            .get_game_manager()
            .lock()
            .unwrap()
            .create_game(room)?
            .id();

        drop(room_manager_lock);

        let resp = Response::StartGame;
        let handler = self
            .factory
            .create_game_request_handler(self.user.clone(), game_id);
        Ok(RequestResult::new(resp, handler))
    }
}
