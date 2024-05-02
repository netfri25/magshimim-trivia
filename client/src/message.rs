use derive_more::From;
use std::sync::Arc;
use trivia::messages::Response;

use crate::connection;
use crate::page::{
    create_question, create_room, game, join_room, login, main_menu, register, results, room,
    statistics,
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
    MainMenu(main_menu::Msg),

    #[from]
    CreateRoom(create_room::Msg),

    #[from]
    JoinRoom(join_room::Msg),

    #[from]
    Room(room::Msg),

    #[from]
    Statistics(statistics::Msg),

    #[from]
    Game(game::Msg),

    #[from]
    Results(results::Msg),

    #[from]
    CreateQuestion(create_question::Msg),
}
