use serde::{Deserialize, Serialize};

use crate::db::Database;
use crate::managers::room::{RoomID, RoomState};
use crate::messages::{Request, RequestInfo, RequestResult, Response};
use crate::username::Username;

use super::{Handler, RequestHandlerFactory};

pub struct RoomUserRequestHandler<'db, 'factory, DB: ?Sized> {
    room_id: RoomID,
    user: Username,
    is_admin: bool,
    factory: &'factory RequestHandlerFactory<'db, DB>,
}

impl<'db, 'factory: 'db, DB> Handler<'db> for RoomUserRequestHandler<'db, 'factory, DB>
where
    DB: Database + Sync + ?Sized,
{
    fn relevant(&self, request_info: &RequestInfo) -> bool {
        use Request::*;
        matches!(
            request_info.data,
            LeaveRoom | RoomState | StartGame | CloseRoom
        )
    }

    fn handle(&mut self, request_info: RequestInfo) -> Result<RequestResult<'db>, super::Error> {
        match request_info.data {
            Request::RoomState => self.room_state(),
            Request::CloseRoom => self.close_room(),
            Request::StartGame => self.start_game(),
            Request::LeaveRoom | Request::Logout => self.leave_room(),
            _ => Ok(RequestResult::new_error("Invalid request")),
        }
    }
}

impl<'db, 'factory: 'db, DB> RoomUserRequestHandler<'db, 'factory, DB>
where
    DB: Database + Sync + ?Sized,
{
    pub fn new(
        factory: &'factory RequestHandlerFactory<'db, DB>,
        user: Username,
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

    fn leave_room(&mut self) -> Result<RequestResult<'db>, super::Error> {
        let room_manager = self.factory.room_manager();
        let mut room_manager_lock = room_manager.write().unwrap();
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

    fn room_state(&mut self) -> Result<RequestResult<'db>, super::Error> {
        let room_manager = self.factory.room_manager();
        let Some(room) = room_manager.read().unwrap().room(self.room_id).cloned() else {
            return self.leave_room();
        };

        if room.room_data().state == RoomState::InGame {
            let resp = Response::StartGame(Ok(()));
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

    fn close_room(&mut self) -> Result<RequestResult<'db>, super::Error> {
        if !self.is_admin {
            let resp = Response::CloseRoom(Err(Error::NotAdmin));
            return Ok(RequestResult::without_handler(resp));
        }

        let room_manager = self.factory.room_manager();
        room_manager.write().unwrap().delete_room(self.room_id);
        let resp = Response::CloseRoom(Ok(()));
        let handler = self.factory.create_menu_request_handler(self.user.clone());
        Ok(RequestResult::new(resp, handler))
    }

    fn start_game(&mut self) -> Result<RequestResult<'db>, super::Error> {
        if !self.is_admin {
            let resp = Response::StartGame(Err(Error::NotAdmin));
            return Ok(RequestResult::without_handler(resp));
        }

        self.factory
            .room_manager()
            .write()
            .unwrap()
            .set_state(self.room_id, RoomState::InGame);

        let room_manager = self.factory.room_manager();
        let room_manager_lock = room_manager.read().unwrap();

        let Some(room) = room_manager_lock.room(self.room_id) else {
            let resp = Response::StartGame(Err(Error::UnknownRoomID(self.room_id)));
            return Ok(RequestResult::without_handler(resp));
        };

        let game_id = self
            .factory
            .game_manager()
            .write()
            .unwrap()
            .create_game(room)?
            .id();

        drop(room_manager_lock);

        let resp = Response::StartGame(Ok(()));
        let handler = self
            .factory
            .create_game_request_handler(self.user.clone(), game_id);
        Ok(RequestResult::new(resp, handler))
    }
}

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum Error {
    #[error("admin only action")]
    NotAdmin,

    #[error("unknown room id {0}")]
    UnknownRoomID(RoomID),
}
