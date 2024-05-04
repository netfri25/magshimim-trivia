use std::sync::RwLock;

use crate::db::Database;
use crate::managers::game::GameID;
use crate::managers::room::RoomID;
use crate::managers::{GameManager, LoginManager, RoomManager, StatisticsManager};
use crate::username::Username;

use super::{
    GameRequestHandler, Handler, LoginRequestHandler, MenuRequestHandler, RoomUserRequestHandler,
};

pub struct RequestHandlerFactory<'db, DB: ?Sized> {
    login_manager: RwLock<LoginManager<'db, DB>>,
    room_manager: RwLock<RoomManager>,
    statistics_manager: RwLock<StatisticsManager<'db, DB>>,
    game_manager: RwLock<GameManager<'db, DB>>,
    db: &'db DB,
}

impl<'db, 'me: 'db, DB> RequestHandlerFactory<'db, DB>
where
    DB: Database + Sync + ?Sized,
{
    pub fn new(db: &'db DB) -> Self {
        let login_manager = LoginManager::new(db);
        let login_manager = RwLock::new(login_manager);
        let room_manager = RoomManager::new();
        let room_manager = RwLock::new(room_manager);
        let statistics_manager = StatisticsManager::new(db);
        let statistics_manager = RwLock::new(statistics_manager);
        let game_manager = GameManager::new(db);
        let game_manager = RwLock::new(game_manager);
        Self {
            db,
            login_manager,
            room_manager,
            statistics_manager,
            game_manager,
        }
    }

    pub fn db(&'me self) -> &'db DB {
        self.db
    }

    pub fn create_login_request_handler(&'me self) -> impl Handler<'db> + 'me {
        LoginRequestHandler::new(self)
    }

    pub fn create_menu_request_handler(
        &'me self,
        logged_user: Username,
    ) -> impl Handler<'db> + 'me {
        MenuRequestHandler::new(self, logged_user)
    }

    pub fn create_room_user_request_handler(
        &'me self,
        user: Username,
        is_admin: bool,
        room_id: RoomID,
    ) -> impl Handler<'db> + 'me {
        RoomUserRequestHandler::new(self, user, is_admin, room_id)
    }

    pub fn create_game_request_handler(
        &'me self,
        user: Username,
        game_id: GameID,
    ) -> impl Handler<'db> + 'me {
        GameRequestHandler::new(self, user, game_id)
    }

    pub fn get_login_manager(&'me self) -> &'me RwLock<LoginManager<'db, DB>> {
        &self.login_manager
    }

    pub fn get_room_manager(&'me self) -> &'me RwLock<RoomManager> {
        &self.room_manager
    }

    pub fn get_statistics_manager(&'me self) -> &'me RwLock<StatisticsManager<'db, DB>> {
        &self.statistics_manager
    }

    pub fn get_game_manager(&'me self) -> &'me RwLock<GameManager<'db, DB>> {
        &self.game_manager
    }
}
