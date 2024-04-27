use derive_more::From;
use std::sync::Arc;
use trivia::messages::Response;

use crate::connection;
use crate::page::{
    createroom, game, joinroom, login, mainmenu, register, results, room, statistics,
};

#[derive(From, Debug, Clone)]
#[non_exhaustive]
pub enum Message {
    Quit,
    Connect,

    Connected(Arc<connection::Connection>),
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

    #[from]
    Game(game::Msg),

    #[from]
    Results(results::Msg),
}
