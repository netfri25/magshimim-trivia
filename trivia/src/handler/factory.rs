use std::sync::Mutex;

use crate::db::Database;
use crate::managers::game::GameID;
use crate::managers::login::LoggedUser;
use crate::managers::room::RoomID;
use crate::managers::{GameManager, LoginManager, RoomManager, StatisticsManager};

use super::{
    GameRequestHandler, Handler, LoginRequestHandler, MenuRequestHandler, RoomUserRequestHandler,
};

pub struct RequestHandlerFactory<'db> {
    login_manager: Mutex<LoginManager<'db>>,
    room_manager: Mutex<RoomManager>,
    statistics_manager: Mutex<StatisticsManager<'db>>,
    game_manager: Mutex<GameManager<'db>>,
}

impl<'db, 'me: 'db> RequestHandlerFactory<'db> {
    pub fn new(db: &'db Mutex<dyn Database>) -> Self {
        let login_manager = LoginManager::new(db);
        let login_manager = Mutex::new(login_manager);
        let room_manager = RoomManager::new();
        let room_manager = Mutex::new(room_manager);
        let statistics_manager = StatisticsManager::new(db);
        let statistics_manager = Mutex::new(statistics_manager);
        let game_manager = GameManager::new(db);
        let game_manager = Mutex::new(game_manager);
        Self {
            login_manager,
            room_manager,
            statistics_manager,
            game_manager,
        }
    }

    pub fn create_login_request_handler(&'me self) -> impl Handler<'db> + 'me {
        LoginRequestHandler::new(self)
    }

    pub fn create_menu_request_handler(
        &'me self,
        logged_user: LoggedUser,
    ) -> impl Handler<'db> + 'me {
        MenuRequestHandler::new(self, logged_user)
    }

    pub fn create_room_user_request_handler(
        &'me self,
        user: LoggedUser,
        is_admin: bool,
        room_id: RoomID,
    ) -> impl Handler<'db> + 'me {
        RoomUserRequestHandler::new(self, user, is_admin, room_id)
    }

    pub fn create_game_request_handler(
        &'me self,
        user: LoggedUser,
        game_id: GameID,
    ) -> impl Handler<'db> + 'me {
        GameRequestHandler::new(self, user, game_id)
    }

    pub fn get_login_manager(&'me self) -> &'me Mutex<LoginManager<'db>> {
        &self.login_manager
    }

    pub fn get_room_manager(&'me self) -> &'me Mutex<RoomManager> {
        &self.room_manager
    }

    pub fn get_statistics_manager(&'me self) -> &'me Mutex<StatisticsManager<'db>> {
        &self.statistics_manager
    }

    pub fn get_game_manager(&'me self) -> &'me Mutex<GameManager<'db>> {
        &self.game_manager
    }
}
