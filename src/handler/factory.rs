use std::sync::{Arc, Mutex};

use crate::{db::Database, managers::LoginManager};

use super::{Handler, LoginRequestHandler};

pub struct RequestHandlerFactory {
    login_manager: Arc<Mutex<LoginManager>>,
    db: Arc<Mutex<dyn Database>>,
}

impl RequestHandlerFactory {
    pub fn new(db: Arc<Mutex<dyn Database>>) -> Self {
        let login_manager = LoginManager::new(db.clone());
        let login_manager = Arc::new(Mutex::new(login_manager));
        Self { db, login_manager }
    }

    pub fn create_login_request_handler(self: &Arc<Self>) -> Box<dyn Handler> {
        Box::new(LoginRequestHandler::new(self.clone()))
    }

    pub fn get_login_manager(&self) -> Arc<Mutex<LoginManager>> {
        self.login_manager.clone()
    }
}
