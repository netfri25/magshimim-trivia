use std::sync::{Arc, Mutex};

use crate::db::Database;
use crate::managers::login::LoggedUser;
use crate::managers::{LoginManager, RoomManager, StatisticsManager};

use super::{Handler, LoginRequestHandler, MenuRequestHandler};

pub struct RequestHandlerFactory {
    login_manager: Arc<Mutex<LoginManager>>,
    room_manager: Arc<Mutex<RoomManager>>,
    statistics_manager: Arc<Mutex<StatisticsManager>>,
    db: Arc<Mutex<dyn Database>>,
}

impl RequestHandlerFactory {
    pub fn new(db: Arc<Mutex<dyn Database>>) -> Self {
        let login_manager = LoginManager::new(db.clone());
        let login_manager = Arc::new(Mutex::new(login_manager));
        let room_manager = RoomManager::new();
        let room_manager = Arc::new(Mutex::new(room_manager));
        let statistics_manager = StatisticsManager::new(db.clone());
        let statistics_manager = Arc::new(Mutex::new(statistics_manager));
        Self {
            login_manager,
            room_manager,
            statistics_manager,
            db,
        }
    }

    pub fn create_login_request_handler(self: &Arc<Self>) -> Box<dyn Handler> {
        Box::new(LoginRequestHandler::new(self.clone()))
    }

    pub fn create_menu_request_handler(self: &Arc<Self>, logged_user: LoggedUser) -> Box<dyn Handler> {
        Box::new(MenuRequestHandler::new(self.clone(), logged_user))
    }

    pub fn get_login_manager(&self) -> Arc<Mutex<LoginManager>> {
        self.login_manager.clone()
    }

    pub fn get_room_manager(&self) -> Arc<Mutex<RoomManager>> {
        self.room_manager.clone()
    }

    pub fn get_statistics_manager(&self) -> Arc<Mutex<RoomManager>> {
        self.room_manager.clone()
    }
}
