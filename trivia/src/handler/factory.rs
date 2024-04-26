use std::sync::{Arc, Mutex};

use crate::db::Database;
use crate::managers::game::GameID;
use crate::managers::login::LoggedUser;
use crate::managers::room::RoomID;
use crate::managers::{GameManager, LoginManager, RoomManager, StatisticsManager};

use super::{GameRequestHandler, Handler, LoginRequestHandler, MenuRequestHandler, RoomAdminRequestHandler, RoomMemberRequestHandler};

pub struct RequestHandlerFactory {
    login_manager: Arc<Mutex<LoginManager>>,
    room_manager: Arc<Mutex<RoomManager>>,
    statistics_manager: Arc<Mutex<StatisticsManager>>,
    game_manager: Arc<Mutex<GameManager>>,
}

impl RequestHandlerFactory {
    pub fn new(db: Arc<Mutex<dyn Database>>) -> Self {
        let login_manager = LoginManager::new(db.clone());
        let login_manager = Arc::new(Mutex::new(login_manager));
        let room_manager = RoomManager::new();
        let room_manager = Arc::new(Mutex::new(room_manager));
        let statistics_manager = StatisticsManager::new(db.clone());
        let statistics_manager = Arc::new(Mutex::new(statistics_manager));
        let game_manager = GameManager::new(db.clone());
        let game_manager = Arc::new(Mutex::new(game_manager));
        Self {
            login_manager,
            room_manager,
            statistics_manager,
            game_manager,
        }
    }

    pub fn create_login_request_handler(self: &Arc<Self>) -> Box<dyn Handler> {
        Box::new(LoginRequestHandler::new(self.clone()))
    }

    pub fn create_menu_request_handler(self: &Arc<Self>, logged_user: LoggedUser) -> Box<dyn Handler> {
        Box::new(MenuRequestHandler::new(self.clone(), logged_user))
    }

    pub fn create_room_admin_request_handler(self: &Arc<Self>, admin: LoggedUser, room_id: RoomID) -> Box<dyn Handler> {
        Box::new(RoomAdminRequestHandler::new(self.clone(), admin, room_id))
    }

    pub fn create_room_member_request_handler(self: &Arc<Self>, member: LoggedUser, room_id: RoomID) -> Box<dyn Handler> {
        Box::new(RoomMemberRequestHandler::new(self.clone(), member, room_id))
    }

    pub fn create_game_request_handler(self: &Arc<Self>, user: LoggedUser, game_id: GameID) -> Box<dyn Handler> {
        Box::new(GameRequestHandler::new(self.clone(), user, game_id))
    }

    pub fn get_login_manager(&self) -> Arc<Mutex<LoginManager>> {
        self.login_manager.clone()
    }

    pub fn get_room_manager(&self) -> Arc<Mutex<RoomManager>> {
        self.room_manager.clone()
    }

    pub fn get_statistics_manager(&self) -> Arc<Mutex<StatisticsManager>> {
        self.statistics_manager.clone()
    }

    pub fn get_game_manager(&self) -> Arc<Mutex<GameManager>> {
        self.game_manager.clone()
    }
}
