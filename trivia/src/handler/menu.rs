use std::sync::Arc;

use crate::messages::{RequestInfo, RequestResult};
use crate::managers::login::LoggedUser;

use super::{Handler, RequestHandlerFactory};

pub struct MenuRequestHandler {
    user: LoggedUser,
    factory: Arc<RequestHandlerFactory>,
}

impl MenuRequestHandler {
    pub fn new(factory: Arc<RequestHandlerFactory>, user: LoggedUser) -> Self {
        Self { factory, user }
    }

    fn signout(request_info: RequestInfo) -> anyhow::Result<RequestResult> {
        todo!()
    }

    fn get_rooms(request_info: RequestInfo) -> anyhow::Result<RequestResult> {
        todo!()
    }

    fn get_players_in_room(request_info: RequestInfo) -> anyhow::Result<RequestResult> {
        todo!()
    }

    fn get_personal_stats(request_info: RequestInfo) -> anyhow::Result<RequestResult> {
        todo!()
    }

    fn get_high_scores(request_info: RequestInfo) -> anyhow::Result<RequestResult> {
        todo!()
    }

    fn join_room(request_info: RequestInfo) -> anyhow::Result<RequestResult> {
        todo!()
    }

    fn create_room(request_info: RequestInfo) -> anyhow::Result<RequestResult> {
        todo!()
    }
}

impl Handler for MenuRequestHandler {
    fn relevant(&self, request_info: &RequestInfo) -> bool {
        todo!()
    }

    fn handle(&mut self, request_info: RequestInfo) -> anyhow::Result<RequestResult> {
        todo!()
    }
}
