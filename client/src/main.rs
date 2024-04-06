use std::sync::Arc;

use iced::{alignment::Horizontal, widget::{container, text, column}, Application, Command, Length, Settings};

mod message;
use message::Message;

mod page;
use page::{LoginPage, Page};

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
    err: String,
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

        (Self { conn, page, err: String::default() }, cmd)
    }

    fn title(&self) -> String {
        "Trivia Client".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        // log the messages that relate to the server
        match &message {
            Message::Connected => {
                eprintln!("connected to server!");
                return Command::none();
            }

            Message::Error(err) => {
                self.err = format!("Error: {}", err);
                eprintln!("[ERROR]: {}", err);
                return Command::none();
            }

            Message::Response(response) => {
                eprintln!("[RECV]: {:?}", response);
            }

            _ => {},
        };

        self.err.clear();

        let action = self.page.update(message);
        match action {
            Action::Switch(new_page, cmd) => {
                self.page = new_page;
                return cmd;
            },

            Action::MakeRequest(req) => {
                eprintln!("[SEND]: {:?}", req);
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
        let page = self.page.view();

        let err = text(&self.err)
            .size(consts::ERR_SIZE)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center)
            .style(consts::ERR_COLOR);

        container(column![page, err].padding(2).spacing(5))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::GruvboxDark
    }
}
