use cosmic::iced::Length;
use cosmic::prelude::*;
use cosmic::theme;
use cosmic::widget;

use crate::app::{Message, Operator, QuickFinancial, TvmField};

const QUICK_ROW_HEIGHT: f32 = 28.0;
const NUM_ROW_HEIGHT: f32 = 34.0;

fn quick_button<'a>(label: &str, message: Message, style: theme::Button) -> Element<'a, Message> {
    widget::button::custom(
        widget::container(widget::text(label.to_string()).size(11.0))
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

fn tvm_row<'a>(
    label: &str,
    field: TvmField,
    value: &'a str,
    is_active: bool,
) -> Element<'a, Message> {
    let spacing = cosmic::theme::active().cosmic().spacing;

    let label_style = if is_active {
        theme::Button::Suggested
    } else {
        theme::Button::Standard
    };

    widget::row::with_capacity(3)
        .push(
            widget::button::custom(
                widget::container(widget::text(label.to_string()).size(12.0))
                    .center(Length::Fill)
                    .width(Length::Fill),
            )
            .class(label_style)
            .width(Length::Fixed(44.0))
            .on_press(Message::TvmSelectField(field)),
        )
        .push(
            widget::text_input("0", value)
                .on_input(move |v| Message::TvmFieldChanged(field, v))
                .width(Length::Fill),
        )
        .push(
            widget::button::custom(
                widget::container(widget::text("Solve").size(10.0))
                    .center(Length::Fill)
                    .width(Length::Fill),
            )
            .class(theme::Button::Suggested)
            .on_press(Message::TvmSolve(field))
            .width(Length::Fixed(50.0)),
        )
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .align_y(cosmic::iced::Alignment::Center)
        .into()
}

pub fn view<'a>(
    active_field: TvmField,
    tvm_n: &'a str,
    tvm_rate: &'a str,
    tvm_pv: &'a str,
    tvm_pmt: &'a str,
    tvm_fv: &'a str,
    row_spacing: u16,
) -> Element<'a, Message> {
    let spacing = cosmic::theme::active().cosmic().spacing;

    // TVM section
    let tvm_section = widget::column::with_capacity(5)
        .push(tvm_row(
            "N",
            TvmField::N,
            tvm_n,
            matches!(active_field, TvmField::N),
        ))
        .push(tvm_row(
            "I/Y %",
            TvmField::Rate,
            tvm_rate,
            matches!(active_field, TvmField::Rate),
        ))
        .push(tvm_row(
            "PV",
            TvmField::Pv,
            tvm_pv,
            matches!(active_field, TvmField::Pv),
        ))
        .push(tvm_row(
            "PMT",
            TvmField::Pmt,
            tvm_pmt,
            matches!(active_field, TvmField::Pmt),
        ))
        .push(tvm_row(
            "FV",
            TvmField::Fv,
            tvm_fv,
            matches!(active_field, TvmField::Fv),
        ))
        .spacing(spacing.space_xxs)
        .width(Length::Fill);

    // Quick functions row
    let quick_row = widget::row::with_capacity(4)
        .push(quick_button(
            "Margin",
            Message::QuickFinancial(QuickFinancial::Margin),
            theme::Button::Standard,
        ))
        .push(quick_button(
            "Markup",
            Message::QuickFinancial(QuickFinancial::Markup),
            theme::Button::Standard,
        ))
        .push(quick_button(
            "Tax+",
            Message::QuickFinancial(QuickFinancial::TaxAdd),
            theme::Button::Standard,
        ))
        .push(quick_button(
            "Tax−",
            Message::QuickFinancial(QuickFinancial::TaxRemove),
            theme::Button::Standard,
        ))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(QUICK_ROW_HEIGHT);

    // Numpad
    let row1 = widget::row::with_capacity(4)
        .push(calc_button("C", Message::Clear, theme::Button::Standard))
        .push(calc_button(
            "±",
            Message::ToggleSign,
            theme::Button::Standard,
        ))
        .push(calc_button("%", Message::Percent, theme::Button::Standard))
        .push(calc_button(
            "÷",
            Message::Operator(Operator::Divide),
            theme::Button::Standard,
        ))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(NUM_ROW_HEIGHT);

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
            "×",
            Message::Operator(Operator::Multiply),
            theme::Button::Standard,
        ))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(NUM_ROW_HEIGHT);

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
            "−",
            Message::Operator(Operator::Subtract),
            theme::Button::Standard,
        ))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(NUM_ROW_HEIGHT);

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
            "+",
            Message::Operator(Operator::Add),
            theme::Button::Standard,
        ))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(NUM_ROW_HEIGHT);

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
            "=",
            Message::Evaluate,
            theme::Button::Suggested,
        ))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(NUM_ROW_HEIGHT);

    widget::column::with_capacity(8)
        .push(tvm_section)
        .push(quick_row)
        .push(row1)
        .push(row2)
        .push(row3)
        .push(row4)
        .push(row5)
        .spacing(row_spacing)
        .width(Length::Fill)
        .into()
}
