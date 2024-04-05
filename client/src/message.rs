use crate::page;
use derive_more::From;

#[derive(From, Debug, Clone)]
#[non_exhaustive]
pub enum Message {
    Login(page::login::Msg),
    Register(page::register::Msg),
}
