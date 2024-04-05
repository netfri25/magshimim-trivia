use iced::{Sandbox, Settings};

mod message;
use message::Message;

mod page;
use page::Page;

mod action;
use action::Action;

mod consts;

fn main() {
    let mut settings = Settings::default();
    settings.window.size = iced::Size::new(800., 600.);
    settings.window.position = iced::window::Position::Centered;
    Client::run(settings).unwrap();
}

// TODO: add TcpStream connection (maybe async?)
struct Client {
    page: Box<dyn Page>,
}

// TODO: convert to Application
impl Sandbox for Client {
    type Message = Message;

    fn new() -> Self {
        let page = Box::<page::login::Page>::default();
        Self { page }
    }

    fn title(&self) -> String {
        "Trivia Client".to_string()
    }

    fn update(&mut self, message: Self::Message) {
        let action = self.page.update(message);
        match action {
            Action::GoTo(new_page) => {
                self.page = new_page
            }

            Action::MakeRequest(req) => {
                eprintln!("{:?}", req);
            }

            Action::Nothing => {},
        }
    }

    fn view(&self) -> iced::Element<Self::Message> {
        self.page.view()
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::GruvboxDark
    }
}
