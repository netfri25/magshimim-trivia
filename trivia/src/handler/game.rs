use crate::messages::{Request, RequestInfo, RequestResult};

use super::{Error, Handler};

pub struct GameRequestHandler { }

impl Handler for GameRequestHandler {
    fn relevant(&self, request_info: &RequestInfo) -> bool {
        use Request::*;
        matches!(request_info.data, LeaveGame | Question | SubmitAnswer(_))
    }

    fn handle(&mut self, request_info: RequestInfo) -> Result<RequestResult, Error> {
        match request_info.data {
            Request::Question => self.get_question(),
            Request::LeaveGame => self.leave_game(),
            Request::SubmitAnswer(answer) => self.submit_answer(answer),
            _ => Ok(RequestResult::new_error("Invalid request")),
        }
    }
}

impl GameRequestHandler {
    fn get_question(&self) -> Result<RequestResult, Error> {
        todo!()
    }

    fn leave_game(&self) -> Result<RequestResult, Error> {
        todo!()
    }

    fn submit_answer(&self, answer_index: usize) -> Result<RequestResult, Error> {
        todo!()
    }
}
