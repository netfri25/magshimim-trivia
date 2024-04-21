use derive_more::From;
use std::sync::Arc;
use trivia::messages::Response;

use crate::connection;
use crate::page::{createroom, joinroom, login, mainmenu, register, room, statistics};

#[derive(From, Debug, Clone)]
#[non_exhaustive]
pub enum Message {
    Connected(connection::Connection),
    Error(Arc<connection::Error>),
    Response(Arc<Response>),

    #[from]
    Login(login::Msg),

    #[from]
    Register(register::Msg),

    #[from]
    MainMenu(mainmenu::Msg),

    #[from]
    CreateRoom(createroom::Msg),

    #[from]
    JoinRoom(joinroom::Msg),

    #[from]
    Room(room::Msg),

    #[from]
    Statistics(statistics::Msg),
}
