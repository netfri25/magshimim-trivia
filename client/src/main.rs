use std::sync::Arc;

use iced::{
    alignment::Horizontal,
    widget::{column, container, text},
    Application, Command, Length, Settings,
};

mod message;
use message::Message;

mod page;
use page::{LoginPage, Page};

mod action;
use action::Action;

mod connection;
use connection::Connection;
use trivia::messages::{Request, Response};

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
        let page = Box::<LoginPage>::default();
        let cmd = Command::perform(
            {
                async move { Connection::connect(addr) }
            },
            |result| match result {
                Ok(conn) => Message::Connected(conn),
                Err(err) => Message::Error(Arc::new(err)),
            },
        );

        (
            Self {
                conn: Connection::default(),
                page,
                err: String::default(),
            },
            cmd,
        )
    }

    fn title(&self) -> String {
        "Trivia Client".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        // log the messages that relate to the server
        match message {
            Message::Connected(conn) => {
                eprintln!("connected to server!");
                self.conn = conn;
                return Command::none();
            }

            Message::Error(err) => {
                self.err = format!("Error: {}", err);
                eprintln!("[ERROR]: {}", err);
                return Command::none();
            }

            Message::Response(ref response) => {
                eprintln!("[RECV]: {:?}", response);
            }

            _ => {}
        };

        self.err.clear();

        let action = self.page.update(message);
        match action {
            Action::Switch(new_page, req) => {
                self.page = new_page;
                return self.make_request(req);
            }

            Action::MakeRequest(req) => {
                eprintln!("[SEND]: {:?}", req);
                return self.make_request(Some(req));
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

    fn subscription(&self) -> iced::Subscription<Message> {
        self.page.subscription()
    }
}

impl Client {
    pub fn make_request(&mut self, req: Option<Request>) -> Command<Message> {
        let Some(req) = req else {
            return Command::none();
        };

        Command::perform(
            {
                let conn = self.conn.clone();
                async move { conn.send_recv(req).await }
            },
            response_as_message,
        )
    }
}

fn response_as_message(resp: Result<Response, connection::Error>) -> Message {
    match resp {
        Ok(Response::Error { msg }) => {
            Message::Error(Arc::new(connection::Error::ResponseErr(msg)))
        }
        Ok(response) => Message::Response(Arc::new(response)),
        Err(err) => Message::Error(Arc::new(err)),
    }
}
