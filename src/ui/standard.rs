use cosmic::prelude::*;
use cosmic::widget;

use crate::app::Message;

pub fn view<'a>() -> Element<'a, Message> {
    widget::text("Standard mode placeholder").into()
}
