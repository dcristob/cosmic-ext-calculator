use std::collections::HashMap;

use cosmic::iced::keyboard::Key;
use cosmic::widget::menu::key_bind::{KeyBind, Modifier};

use crate::app::MenuAction;

pub fn key_binds() -> HashMap<KeyBind, MenuAction> {
    let mut key_binds = HashMap::new();

    macro_rules! bind {
        ([$($modifier:ident),* $(,)?], $key:expr, $action:ident) => {{
            key_binds.insert(
                KeyBind {
                    modifiers: vec![$(Modifier::$modifier),*],
                    key: $key,
                },
                MenuAction::$action,
            );
        }};
    }

    bind!([Ctrl], Key::Character("1".into()), SwitchStandard);
    bind!([Ctrl], Key::Character("2".into()), SwitchEngineering);
    bind!([Ctrl], Key::Character("3".into()), SwitchFinancial);
    bind!([Ctrl], Key::Character("h".into()), ToggleHistory);
    bind!([Ctrl], Key::Character("z".into()), Undo);
    bind!([Ctrl], Key::Character("c".into()), CopyResult);
    bind!([Ctrl, Shift], Key::Character("C".into()), ClearHistory);
    bind!([Ctrl], Key::Character("i".into()), About);

    key_binds
}
