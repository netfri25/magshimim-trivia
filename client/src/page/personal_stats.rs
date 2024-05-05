use iced::alignment::Horizontal;
use iced::widget::{column, container, horizontal_space, row, text};
use iced::{Alignment, Length};
use trivia::managers::statistics::Statistics;

use crate::action::Action;
use crate::consts;
use crate::message::Message;

use super::{Page, StatisticsPage};

pub struct PersonalStatsPage {
    stats: Statistics,
}

impl Page for PersonalStatsPage {
    fn update(&mut self, _message: Message) -> Action {
        Action::none()
    }

    fn view(&self) -> iced::Element<Message> {
        let title = text("Statistics")
            .size(consts::TITLE_SIZE)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);

        let fields_col = column![
            field("Correct Answers", format!("{}", self.stats.correct_answers)),
            field("Total Answers", format!("{}", self.stats.total_answers)),
            field(
                "Average Answer Time",
                format!("{:.02?}", self.stats.average_answer_time)
            ),
            field("Total Games", format!("{}", self.stats.total_games)),
            field("Score", format!("{:.02}", self.stats.score)),
        ]
        .spacing(20)
        .align_items(Alignment::Center);

        let fields = row![
            horizontal_space().width(Length::FillPortion(1)),
            fields_col.width(Length::FillPortion(2)),
            horizontal_space().width(Length::FillPortion(1)),
        ]
        .width(Length::Fill)
        .align_items(Alignment::Center);

        container(column![
            container(title)
                .height(Length::FillPortion(1))
                .padding(consts::TITLES_PADDING * 2),
            fields.height(Length::FillPortion(3)),
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    fn quit(&mut self) -> Action {
        Action::switch(StatisticsPage::default())
    }
}

impl PersonalStatsPage {
    pub fn new(stats: Statistics) -> Self {
        Self { stats }
    }
}

fn field(name: &'static str, value: String) -> iced::Element<Message> {
    container(row![
        text(name).size(22),
        horizontal_space(),
        text(value).size(22),
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
