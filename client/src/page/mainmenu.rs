use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, text};
use iced::{Alignment, Length};

use crate::action::Action;
use crate::consts;
use crate::message::Message;

use super::{CreateRoomPage, JoinRoomPage, Page, StatisticsPage};

#[derive(Debug, Clone)]
pub enum Msg {
    CreateRoom,
    JoinRoom,
    Statistics,
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
            Msg::Quit => Action::quit(),
        }
    }

    fn view(&self) -> iced::Element<Message> {
        let title = text("Trivia")
            .size(consts::TITLE_SIZE)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);

        let create_room_button = menu_button("Create Room", Msg::CreateRoom);
        let join_room_button = menu_button("Join Room", Msg::JoinRoom);
        let statistics_button = menu_button("Statistics", Msg::Statistics);
        let quit_button = menu_button("Quit", Msg::Quit);

        let buttons = column![
            create_room_button,
            join_room_button,
            statistics_button,
            quit_button,
        ]
        .align_items(Alignment::Center)
        .width(Length::Fill)
        .spacing(35);

        container(column![
            container(title)
                .height(Length::FillPortion(1))
                .padding(consts::TITLES_PADDING * 2),
            buttons.height(Length::FillPortion(3)),
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }
}

fn menu_button(button_text: &'static str, msg: Msg) -> iced::Element<Message> {
    let button_text = text(button_text)
        .size(30)
        .horizontal_alignment(Horizontal::Center)
        .vertical_alignment(Vertical::Center);

    button(button_text)
        .width(200)
        .height(80)
        .on_press(msg.into())
        .into()
}
