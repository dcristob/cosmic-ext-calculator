use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget;

use crate::app::Message;
use crate::fl;

/// Settings drawer content. Currently a single tax-rate field used by the
/// Financial mode's Tax+/Tax− quick functions.
pub fn view<'a>(tax_rate_input: &'a str) -> Element<'a, Message> {
    let spacing = cosmic::theme::active().cosmic().spacing;

    let tax_row = widget::row::with_capacity(2)
        .push(widget::text::body(fl!("tax-rate")).width(Length::Fill))
        .push(
            widget::text_input("21", tax_rate_input)
                .on_input(Message::TaxRateChanged)
                .width(Length::Fixed(96.0)),
        )
        .spacing(spacing.space_s)
        .align_y(Alignment::Center)
        .width(Length::Fill);

    widget::column::with_capacity(1)
        .push(tax_row)
        .spacing(spacing.space_s)
        .padding(spacing.space_s)
        .width(Length::Fill)
        .into()
}
