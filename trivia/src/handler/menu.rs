use crate::messages::{RequestInfo, RequestResult};

use super::Handler;

pub struct MenuRequestHandler;

impl Handler for MenuRequestHandler {
    fn relevant(&self, request_info: &RequestInfo) -> bool {
        todo!()
    }

    fn handle(&mut self, request_info: RequestInfo) -> anyhow::Result<RequestResult> {
        todo!()
    }
}
