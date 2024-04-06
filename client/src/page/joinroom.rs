use iced::widget::container;

use crate::action::Action;
use crate::message::Message;

use trivia::messages::{Request, Response};
use trivia::managers::room::Room;

use super::Page;

#[derive(Debug, Clone)]
pub enum Msg {
    EnterRoom,
}

#[derive(Default)]
pub struct JoinRoomPage {
    rooms: Vec<Room>,
}

impl Page for JoinRoomPage {
    fn update(&mut self, message: Message) -> Action {
        if let Message::Response(response) = message {
            match response.as_ref() {
                Response::RoomList(rooms) => {
                    self.rooms = rooms.clone();
                }

                _ => eprintln!("response ignored: {:?}", response),
            }

            return Action::none();
        }

        let Message::JoinRoom(msg) = message else {
            return Action::none();
        };

        match msg {
            Msg::EnterRoom => todo!(),
        }
    }

    fn view(&self) -> iced::Element<Message> {
        container("hello").into()
    }
}

impl JoinRoomPage {
    pub fn new() -> (Self, Request) {
        (Self::default(), Request::RoomList)
    }
}
