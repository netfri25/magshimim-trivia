use std::sync::{Arc, Mutex};

use crate::db::Database;
use crate::managers::{LoginManager, RoomManager};

use super::{Handler, LoginRequestHandler};

pub struct RequestHandlerFactory {
    login_manager: Arc<Mutex<LoginManager>>,
    room_manager: Arc<Mutex<RoomManager>>,
    db: Arc<Mutex<dyn Database>>,
}

impl RequestHandlerFactory {
    pub fn new(db: Arc<Mutex<dyn Database>>) -> Self {
        let login_manager = LoginManager::new(db.clone());
        let login_manager = Arc::new(Mutex::new(login_manager));
        let room_manager = RoomManager::new();
        let room_manager = Arc::new(Mutex::new(room_manager));
        Self {
            login_manager,
            room_manager,
            db,
        }
    }

    pub fn create_login_request_handler(self: &Arc<Self>) -> Box<dyn Handler> {
        Box::new(LoginRequestHandler::new(self.clone()))
    }

    pub fn get_login_manager(&self) -> Arc<Mutex<LoginManager>> {
        self.login_manager.clone()
    }

    pub fn get_room_manager(&self) -> Arc<Mutex<RoomManager>> {
        self.room_manager.clone()
    }
}
