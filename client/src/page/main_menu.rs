use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, horizontal_space, row, text, vertical_space};
use iced::{Alignment, Length};

use crate::action::Action;
use crate::consts;
use crate::message::Message;

use super::{CreateQuestionPage, CreateRoomPage, JoinRoomPage, Page, StatisticsPage};

#[derive(Debug, Clone)]
pub enum Msg {
    CreateRoom,
    JoinRoom,
    Statistics,
    CreateQuestion,
    Quit,
}

#[derive(Default)]
pub struct MainMenuPage;

impl Page for MainMenuPage {
    fn update(&mut self, message: Message) -> Action {
        let Message::MainMenu(msg) = message else {
            return Action::none();
        };

        match msg {
            Msg::CreateRoom => Action::switch(CreateRoomPage::default()),
            Msg::JoinRoom => {
                let (page, req) = JoinRoomPage::new();
                Action::switch_and_request(page, req)
            }
            Msg::Statistics => Action::switch(StatisticsPage::default()),
            Msg::CreateQuestion => Action::switch(CreateQuestionPage::default()),
            Msg::Quit => self.quit()
        }
    }

    fn view(&self) -> iced::Element<Message> {
        let title = text("Trivia")
            .size(consts::TITLE_SIZE)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);

        let buttons = column![
            menu_button("Create Room", Msg::CreateRoom),
            vertical_space(),
            menu_button("Join Room", Msg::JoinRoom),
            vertical_space(),
            menu_button("Statistics", Msg::Statistics),
            vertical_space(),
            menu_button("New Question", Msg::CreateQuestion),
            vertical_space(),
            menu_button("Quit", Msg::Quit),
        ]
        .align_items(Alignment::Center);

        container(column![
            container(title)
                .height(Length::FillPortion(1))
                .padding(consts::TITLES_PADDING * 2),
            row![
                horizontal_space().width(Length::FillPortion(3)),
                buttons.width(Length::FillPortion(3)),
                horizontal_space().width(Length::FillPortion(3)),
            ]
            .height(Length::FillPortion(3)),
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    fn quit(&mut self) -> Action {
        std::process::exit(0)
    }
}

fn menu_button(button_text: &'static str, msg: Msg) -> iced::Element<Message> {
    let button_text = text(button_text)
        .size(30)
        .width(Length::Fill)
        .height(Length::Fill)
        .horizontal_alignment(Horizontal::Center)
        .vertical_alignment(Vertical::Center);

    button(button_text)
        .width(Length::Fill)
        .height(80)
        .on_press(msg.into())
        .into()
}
