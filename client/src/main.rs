use std::sync::Arc;

use iced::{Application, Command, Settings};

mod message;
use message::Message;

mod page;
use page::{LoginPage, MainMenuPage, Page};

mod action;
use action::Action;

mod connection;
use connection::Connection;
use trivia::messages::Response;

mod consts;

fn main() {
    let mut settings = Settings::default();
    settings.window.size = iced::Size::new(800., 600.);
    settings.window.position = iced::window::Position::Centered;
    settings.flags = "127.0.0.1:6969";
    Client::run(settings).unwrap();
}

struct Client {
    page: Box<dyn Page>,
    conn: Connection,
}

impl Application for Client {
    type Message = Message;
    type Executor = iced::executor::Default;
    type Theme = iced::Theme;
    type Flags = &'static str;

    fn new(addr: &'static str) -> (Self, Command<Message>) {
        let conn = Connection::default();
        let page = Box::<LoginPage>::default();
        let cmd = Command::perform(
            {
                let conn = conn.clone();
                async move { conn.connect(addr) }
            },
            |result| match result {
                Ok(()) => Message::Connected,
                Err(err) => Message::Error(Arc::new(err)),
            },
        );

        (Self { conn, page }, cmd)
    }

    fn title(&self) -> String {
        "Trivia Client".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        // log the messages that relate to the server
        match &message {
            Message::Connected => {
                eprintln!("connected to server!");
            }

            Message::Error(err) => {
                eprintln!("[ERROR]: {}", err);
            }

            Message::Response(response) => {
                eprintln!("server responded: {:?}", response);
            }

            _ => {},
        };

        let action = self.page.update(message);
        match action {
            Action::Switch(new_page, cmd) => {
                self.page = new_page;
                return cmd;
            },

            Action::MakeRequest(req) => {
                eprintln!("sending: {:?}", req);
                return Command::perform(
                    {
                        let conn = self.conn.clone();
                        async move { conn.send_recv(req).await }
                    },

                    |result| match result {
                        Ok(Response::Error { msg }) => Message::Error(Arc::new(connection::Error::ResponseErr(msg))),
                        Ok(response) => Message::Response(Arc::new(response)),
                        Err(err) => Message::Error(Arc::new(err)),
                    }
                );
            }

            Action::Command(cmd) => return cmd,

            Action::Quit => std::process::exit(0),

            Action::Nothing => {}
        }

        Command::none()
    }

    fn view(&self) -> iced::Element<Self::Message> {
        self.page.view()
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::GruvboxDark
    }
}
