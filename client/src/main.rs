use std::{net::ToSocketAddrs, sync::Arc, time::Duration};

use iced::{
    alignment::Horizontal,
    font, keyboard,
    widget::{column, container, text},
    Application, Command, Font, Length, Settings,
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
    settings.default_font = Font::with_name("Varela Round");
    Client::run(settings).unwrap();
}

struct Client<A> {
    page: Box<dyn Page>,
    conn: Arc<Connection>,
    addr: A,
    err: String,
}

impl<A> Application for Client<A>
where
    A: ToSocketAddrs + Send + Clone + 'static,
{
    type Message = Message;
    type Executor = iced::executor::Default;
    type Theme = iced::Theme;
    type Flags = A;

    fn new(addr: A) -> (Self, Command<Message>) {
        let page = Box::<LoginPage>::default();
        let cmd = Self::connect(addr.clone());

        (
            Self {
                conn: Arc::default(),
                page,
                addr,
                err: String::default(),
            },
            Command::batch([
                cmd,
                font::load(iced_aw::BOOTSTRAP_FONT_BYTES).map(Message::FontLoaded),
            ]),
        )
    }

    fn title(&self) -> String {
        "Trivia Client".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        // log the messages that relate to the server
        match message {
            Message::Connect => return Self::connect(self.addr.clone()),

            Message::Connected(conn) => {
                eprintln!("connected to server!");
                self.conn = conn;
                self.err.clear();
                return Command::none();
            }

            Message::Error(err) => {
                self.err = format!("Error: {}", err);
                eprintln!("[ERROR]: {:?}", err);
                return Command::none();
            }

            Message::Response(ref response) => {
                eprintln!("[RECV]: {:?}", response);
            }

            Message::Quit => std::process::exit(0),

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
        let mut subs = Vec::with_capacity(3);
        subs.push(self.page.subscription());
        subs.push(iced::event::listen_with(handle_event));

        if !self.conn.is_connected() {
            let sub = iced::time::every(Duration::from_secs(5)).map(|_| Message::Connect);
            subs.push(sub);
        }

        iced::Subscription::batch(subs)
    }
}

impl<A> Client<A>
where
    A: ToSocketAddrs + Send + 'static,
{
    pub fn make_request(&mut self, req: Option<Request<'static>>) -> Command<Message> {
        let Some(req) = req else {
            return Command::none();
        };

        Command::perform(
            {
                let conn = self.conn.clone();
                async move { conn.send_and_recv(req).await }
            },
            response_as_message,
        )
    }

    pub fn connect(addr: A) -> Command<Message> {
        Command::perform(
            async move { Connection::connect(addr).map(Arc::new) },
            |result| match result {
                Ok(conn) => Message::Connected(conn),
                Err(err) => Message::Error(Arc::new(err)),
            },
        )
    }
}

fn response_as_message(resp: Result<Response, connection::Error>) -> Message {
    match resp {
        Ok(Response::Error(msg)) => Message::Error(Arc::new(connection::Error::ResponseErr(msg))),
        Ok(response) => Message::Response(Arc::new(response)),
        Err(err) => Message::Error(Arc::new(err)),
    }
}

fn handle_event(event: iced::Event, _status: iced::event::Status) -> Option<Message> {
    match event {
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Escape),
            ..
        }) => Some(Message::Quit),

        _ => None,
    }
}
