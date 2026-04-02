use cosmic::iced::{Color, Length};
use cosmic::prelude::*;
use cosmic::widget;
use cosmic::theme;

use crate::app::Message;
use crate::app::config::{HistoryEntry, Mode};

fn mode_badge_color(mode: &Mode) -> Color {
    match mode {
        Mode::Standard => Color::from_rgb(0.3, 0.6, 1.0),
        Mode::Engineering => Color::from_rgb(0.3, 0.8, 0.5),
        Mode::Financial => Color::from_rgb(1.0, 0.7, 0.3),
    }
}

pub fn view<'a>(history: &'a [HistoryEntry]) -> Element<'a, Message> {
    let spacing = cosmic::theme::active().cosmic().spacing;

    if history.is_empty() {
        return widget::container(widget::text::body("No history yet"))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(cosmic::iced::Alignment::Center)
            .align_y(cosmic::iced::Alignment::Center)
            .into();
    }

    let mut list = widget::column::with_capacity(history.len())
        .spacing(spacing.space_xxs);

    for (index, entry) in history.iter().enumerate().rev() {
        let badge = widget::text(entry.mode.label())
            .size(12.0)
            .class(theme::Text::Color(mode_badge_color(&entry.mode)));

        let header = widget::row::with_capacity(2)
            .push(badge)
            .push(widget::text::body(&entry.expression))
            .spacing(spacing.space_xs)
            .align_y(cosmic::iced::Alignment::Center);

        let result_line = widget::text::title4(format!("= {}", &entry.result))
            .width(Length::Fill)
            .align_x(cosmic::iced::Alignment::End);

        let content = widget::column::with_capacity(2)
            .push(header)
            .push(result_line)
            .spacing(spacing.space_xxxs)
            .width(Length::Fill);

        let button = widget::button::custom(content)
            .class(theme::Button::Standard)
            .width(Length::Fill)
            .on_press(Message::HistorySelect(index));

        list = list.push(button);
    }

    widget::scrollable(list)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
