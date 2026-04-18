use cosmic::iced::Length;
use cosmic::prelude::*;
use cosmic::theme;
use cosmic::widget;

use crate::app::{Message, Operator};

const ROW_HEIGHT: f32 = 40.0;

fn calc_button<'a>(label: &str, message: Message, style: theme::Button) -> Element<'a, Message> {
    widget::button::custom(
        widget::container(widget::text(label.to_string()).size(16.0))
            .center(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .class(style)
    .width(Length::Fill)
    .height(Length::Fill)
    .on_press(message)
    .into()
}

pub fn view<'a>(row_spacing: u16) -> Element<'a, Message> {
    let spacing = cosmic::theme::active().cosmic().spacing;

    let row1 = widget::row::with_capacity(4)
        .push(calc_button("C", Message::Clear, theme::Button::Standard))
        .push(calc_button(
            "(",
            Message::ParenOpen,
            theme::Button::Standard,
        ))
        .push(calc_button(
            ")",
            Message::ParenClose,
            theme::Button::Standard,
        ))
        .push(calc_button("%", Message::Percent, theme::Button::Standard))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(ROW_HEIGHT);

    let row2 = widget::row::with_capacity(4)
        .push(calc_button(
            "7",
            Message::Number(7),
            theme::Button::Standard,
        ))
        .push(calc_button(
            "8",
            Message::Number(8),
            theme::Button::Standard,
        ))
        .push(calc_button(
            "9",
            Message::Number(9),
            theme::Button::Standard,
        ))
        .push(calc_button(
            "÷",
            Message::Operator(Operator::Divide),
            theme::Button::Standard,
        ))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(ROW_HEIGHT);

    let row3 = widget::row::with_capacity(4)
        .push(calc_button(
            "4",
            Message::Number(4),
            theme::Button::Standard,
        ))
        .push(calc_button(
            "5",
            Message::Number(5),
            theme::Button::Standard,
        ))
        .push(calc_button(
            "6",
            Message::Number(6),
            theme::Button::Standard,
        ))
        .push(calc_button(
            "×",
            Message::Operator(Operator::Multiply),
            theme::Button::Standard,
        ))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(ROW_HEIGHT);

    let row4 = widget::row::with_capacity(4)
        .push(calc_button(
            "1",
            Message::Number(1),
            theme::Button::Standard,
        ))
        .push(calc_button(
            "2",
            Message::Number(2),
            theme::Button::Standard,
        ))
        .push(calc_button(
            "3",
            Message::Number(3),
            theme::Button::Standard,
        ))
        .push(calc_button(
            "−",
            Message::Operator(Operator::Subtract),
            theme::Button::Standard,
        ))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(ROW_HEIGHT);

    let row5 = widget::row::with_capacity(4)
        .push(calc_button(
            "0",
            Message::Number(0),
            theme::Button::Standard,
        ))
        .push(calc_button(".", Message::Decimal, theme::Button::Standard))
        .push(calc_button(
            "⌫",
            Message::Backspace,
            theme::Button::Standard,
        ))
        .push(calc_button(
            "+",
            Message::Operator(Operator::Add),
            theme::Button::Standard,
        ))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(ROW_HEIGHT);

    let row6 = widget::row::with_capacity(1)
        .push(calc_button(
            "=",
            Message::Evaluate,
            theme::Button::Suggested,
        ))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(ROW_HEIGHT);

    widget::column::with_capacity(6)
        .push(row1)
        .push(row2)
        .push(row3)
        .push(row4)
        .push(row5)
        .push(row6)
        .spacing(row_spacing)
        .width(Length::Fill)
        .into()
}
