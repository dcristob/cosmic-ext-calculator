use cosmic::iced::Length;
use cosmic::prelude::*;
use cosmic::widget;
use cosmic::theme;

use crate::app::config::AngleMode;
use crate::app::{BaseDisplay, BitwiseOp, Message, Operator};

const FUNC_ROW_HEIGHT: f32 = 32.0;
const NUM_ROW_HEIGHT: f32 = 40.0;

fn func_button<'a>(label: &str, message: Message, style: theme::Button) -> Element<'a, Message> {
    widget::button::custom(
        widget::container(widget::text(label.to_string()).size(13.0))
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

pub fn view<'a>(angle_mode: AngleMode, row_spacing: u16) -> Element<'a, Message> {
    let spacing = cosmic::theme::active().cosmic().spacing;

    // Angle mode bar (3 buttons)
    let deg_style = if angle_mode == AngleMode::Deg {
        theme::Button::Suggested
    } else {
        theme::Button::Standard
    };
    let rad_style = if angle_mode == AngleMode::Rad {
        theme::Button::Suggested
    } else {
        theme::Button::Standard
    };
    let grad_style = if angle_mode == AngleMode::Grad {
        theme::Button::Suggested
    } else {
        theme::Button::Standard
    };

    let angle_row = widget::row::with_capacity(3)
        .push(func_button("DEG", Message::AngleModeChanged(AngleMode::Deg), deg_style))
        .push(func_button("RAD", Message::AngleModeChanged(AngleMode::Rad), rad_style))
        .push(func_button("GRAD", Message::AngleModeChanged(AngleMode::Grad), grad_style))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(FUNC_ROW_HEIGHT);

    // Trig row (6 cols)
    let trig_row = widget::row::with_capacity(6)
        .push(func_button("sin", Message::EngFunction("sin(".into()), theme::Button::Standard))
        .push(func_button("cos", Message::EngFunction("cos(".into()), theme::Button::Standard))
        .push(func_button("tan", Message::EngFunction("tan(".into()), theme::Button::Standard))
        .push(func_button("sin⁻¹", Message::EngFunction("asin(".into()), theme::Button::Standard))
        .push(func_button("cos⁻¹", Message::EngFunction("acos(".into()), theme::Button::Standard))
        .push(func_button("tan⁻¹", Message::EngFunction("atan(".into()), theme::Button::Standard))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(FUNC_ROW_HEIGHT);

    // Math row (6 cols)
    let math_row = widget::row::with_capacity(6)
        .push(func_button("log", Message::EngFunction("log(".into()), theme::Button::Standard))
        .push(func_button("ln", Message::EngFunction("ln(".into()), theme::Button::Standard))
        .push(func_button("x²", Message::EngFunction("^2".into()), theme::Button::Standard))
        .push(func_button("√x", Message::EngFunction("sqrt(".into()), theme::Button::Standard))
        .push(func_button("xʸ", Message::EngFunction("^".into()), theme::Button::Standard))
        .push(func_button("n!", Message::EngFunction("!".into()), theme::Button::Standard))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(FUNC_ROW_HEIGHT);

    // Bitwise row (6 cols)
    let bitwise_row = widget::row::with_capacity(6)
        .push(func_button("AND", Message::BitwiseOp(BitwiseOp::And), theme::Button::Standard))
        .push(func_button("OR", Message::BitwiseOp(BitwiseOp::Or), theme::Button::Standard))
        .push(func_button("XOR", Message::BitwiseOp(BitwiseOp::Xor), theme::Button::Standard))
        .push(func_button("NOT", Message::BitwiseOp(BitwiseOp::Not), theme::Button::Standard))
        .push(func_button("≪", Message::BitwiseOp(BitwiseOp::Shl), theme::Button::Standard))
        .push(func_button("≫", Message::BitwiseOp(BitwiseOp::Shr), theme::Button::Standard))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(FUNC_ROW_HEIGHT);

    // Base/constants row (6 cols)
    let base_row = widget::row::with_capacity(6)
        .push(func_button("HEX", Message::BaseDisplay(BaseDisplay::Hex), theme::Button::Standard))
        .push(func_button("DEC", Message::BaseDisplay(BaseDisplay::Dec), theme::Button::Standard))
        .push(func_button("OCT", Message::BaseDisplay(BaseDisplay::Oct), theme::Button::Standard))
        .push(func_button("BIN", Message::BaseDisplay(BaseDisplay::Bin), theme::Button::Standard))
        .push(func_button("π", Message::EngFunction("pi".into()), theme::Button::Standard))
        .push(func_button("e", Message::EngFunction("e".into()), theme::Button::Standard))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(FUNC_ROW_HEIGHT);

    // Standard numpad (4 cols)
    let num_row1 = widget::row::with_capacity(4)
        .push(calc_button("C", Message::Clear, theme::Button::Standard))
        .push(calc_button("(", Message::ParenOpen, theme::Button::Standard))
        .push(calc_button(")", Message::ParenClose, theme::Button::Standard))
        .push(calc_button("÷", Message::Operator(Operator::Divide), theme::Button::Standard))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(NUM_ROW_HEIGHT);

    let num_row2 = widget::row::with_capacity(4)
        .push(calc_button("7", Message::Number(7), theme::Button::Standard))
        .push(calc_button("8", Message::Number(8), theme::Button::Standard))
        .push(calc_button("9", Message::Number(9), theme::Button::Standard))
        .push(calc_button("×", Message::Operator(Operator::Multiply), theme::Button::Standard))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(NUM_ROW_HEIGHT);

    let num_row3 = widget::row::with_capacity(4)
        .push(calc_button("4", Message::Number(4), theme::Button::Standard))
        .push(calc_button("5", Message::Number(5), theme::Button::Standard))
        .push(calc_button("6", Message::Number(6), theme::Button::Standard))
        .push(calc_button("−", Message::Operator(Operator::Subtract), theme::Button::Standard))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(NUM_ROW_HEIGHT);

    let num_row4 = widget::row::with_capacity(4)
        .push(calc_button("1", Message::Number(1), theme::Button::Standard))
        .push(calc_button("2", Message::Number(2), theme::Button::Standard))
        .push(calc_button("3", Message::Number(3), theme::Button::Standard))
        .push(calc_button("+", Message::Operator(Operator::Add), theme::Button::Standard))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(NUM_ROW_HEIGHT);

    let num_row5 = widget::row::with_capacity(4)
        .push(calc_button("0", Message::Number(0), theme::Button::Standard))
        .push(calc_button(".", Message::Decimal, theme::Button::Standard))
        .push(calc_button("⌫", Message::Backspace, theme::Button::Standard))
        .push(calc_button("=", Message::Evaluate, theme::Button::Suggested))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(NUM_ROW_HEIGHT);

    widget::column::with_capacity(11)
        .push(angle_row)
        .push(trig_row)
        .push(math_row)
        .push(bitwise_row)
        .push(base_row)
        .push(num_row1)
        .push(num_row2)
        .push(num_row3)
        .push(num_row4)
        .push(num_row5)
        .spacing(row_spacing)
        .width(Length::Fill)
        .into()
}
