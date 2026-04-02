use cosmic::iced::Length;
use cosmic::prelude::*;
use cosmic::theme;
use cosmic::widget;

use crate::app::{Message, Operator, QuickFinancial, TvmField};

fn calc_button<'a>(label: &str, message: Message, style: theme::Button) -> Element<'a, Message> {
    widget::button::custom(
        widget::container(widget::text(label.to_string()).size(20.0))
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
) -> Element<'a, Message> {
    let spacing = cosmic::theme::active().cosmic().spacing;

    widget::row::with_capacity(3)
        .push(
            widget::container(widget::text(label.to_string()).size(14.0))
                .width(Length::Fixed(50.0))
                .center_y(Length::Fill),
        )
        .push(
            widget::text_input("0", value)
                .on_input(move |v| Message::TvmFieldChanged(field, v))
                .width(Length::Fill),
        )
        .push(
            widget::button::custom(
                widget::container(widget::text("Solve").size(14.0))
                    .center(Length::Fill)
                    .width(Length::Fill),
            )
            .class(theme::Button::Suggested)
            .on_press(Message::TvmSolve(field))
            .width(Length::Fixed(60.0)),
        )
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .align_y(cosmic::iced::Alignment::Center)
        .into()
}

pub fn view<'a>(
    tvm_n: &'a str,
    tvm_rate: &'a str,
    tvm_pv: &'a str,
    tvm_pmt: &'a str,
    tvm_fv: &'a str,
) -> Element<'a, Message> {
    let spacing = cosmic::theme::active().cosmic().spacing;

    // TVM section
    let tvm_section = widget::column::with_capacity(5)
        .push(tvm_row("N", TvmField::N, tvm_n))
        .push(tvm_row("I/Y %", TvmField::Rate, tvm_rate))
        .push(tvm_row("PV", TvmField::Pv, tvm_pv))
        .push(tvm_row("PMT", TvmField::Pmt, tvm_pmt))
        .push(tvm_row("FV", TvmField::Fv, tvm_fv))
        .spacing(spacing.space_xxs)
        .width(Length::Fill);

    // Quick functions row
    let quick_row = widget::row::with_capacity(4)
        .push(calc_button("Margin", Message::QuickFinancial(QuickFinancial::Margin), theme::Button::Standard))
        .push(calc_button("Markup", Message::QuickFinancial(QuickFinancial::Markup), theme::Button::Standard))
        .push(calc_button("Tax+", Message::QuickFinancial(QuickFinancial::TaxAdd), theme::Button::Standard))
        .push(calc_button("Tax−", Message::QuickFinancial(QuickFinancial::TaxRemove), theme::Button::Standard))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(Length::Fill);

    // Numpad
    let row1 = widget::row::with_capacity(4)
        .push(calc_button("C", Message::Clear, theme::Button::Standard))
        .push(calc_button("±", Message::ToggleSign, theme::Button::Standard))
        .push(calc_button("%", Message::Percent, theme::Button::Standard))
        .push(calc_button("÷", Message::Operator(Operator::Divide), theme::Button::Standard))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(Length::Fill);

    let row2 = widget::row::with_capacity(4)
        .push(calc_button("7", Message::Number(7), theme::Button::Standard))
        .push(calc_button("8", Message::Number(8), theme::Button::Standard))
        .push(calc_button("9", Message::Number(9), theme::Button::Standard))
        .push(calc_button("×", Message::Operator(Operator::Multiply), theme::Button::Standard))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(Length::Fill);

    let row3 = widget::row::with_capacity(4)
        .push(calc_button("4", Message::Number(4), theme::Button::Standard))
        .push(calc_button("5", Message::Number(5), theme::Button::Standard))
        .push(calc_button("6", Message::Number(6), theme::Button::Standard))
        .push(calc_button("−", Message::Operator(Operator::Subtract), theme::Button::Standard))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(Length::Fill);

    let row4 = widget::row::with_capacity(4)
        .push(calc_button("1", Message::Number(1), theme::Button::Standard))
        .push(calc_button("2", Message::Number(2), theme::Button::Standard))
        .push(calc_button("3", Message::Number(3), theme::Button::Standard))
        .push(calc_button("+", Message::Operator(Operator::Add), theme::Button::Standard))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(Length::Fill);

    let row5 = widget::row::with_capacity(4)
        .push(calc_button("0", Message::Number(0), theme::Button::Standard))
        .push(calc_button(".", Message::Decimal, theme::Button::Standard))
        .push(calc_button("⌫", Message::Backspace, theme::Button::Standard))
        .push(calc_button("=", Message::Evaluate, theme::Button::Suggested))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(Length::Fill);

    widget::column::with_capacity(8)
        .push(tvm_section)
        .push(quick_row)
        .push(row1)
        .push(row2)
        .push(row3)
        .push(row4)
        .push(row5)
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
