use iced::alignment::Horizontal;
use iced::widget::{container, text};
use iced::Length;

use crate::action::Action;
use crate::consts;
use crate::message::Message;

use super::Page;

#[derive(Debug, Clone)]
pub enum Msg {}

#[derive(Default)]
pub struct CreateQuestionPage {
    question: String,
    answers: Vec<String>,
    correct_answer_index: Option<usize>,
}

impl Page for CreateQuestionPage {
    fn update(&mut self, message: Message) -> Action {
        let Message::CreateQuestion(msg) = message else {
            return Action::none();
        };

        match msg {}
    }

    fn view(&self) -> iced::Element<Message> {
        let title = text("High Scores")
            .size(consts::TITLE_SIZE)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);

        container(title)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
