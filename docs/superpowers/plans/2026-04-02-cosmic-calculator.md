# COSMIC Multi-Mode Calculator Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a three-mode calculator (Standard, Engineering, Financial) for the COSMIC desktop with Rust-native expression evaluation, HP-48S-inspired engineering layout, TVM solver, full keyboard support, and unified history.

**Architecture:** Clean-room libcosmic application with three layers: `engine/` (expression parsing and evaluation per mode), `ui/` (button grids and layouts per mode), `app.rs` (top-level state, COSMIC Application trait, message routing). Config, keybinds, localization, and icons follow standard COSMIC patterns from the reference cosmic-utils/calculator.

**Tech Stack:** Rust (edition 2024), libcosmic (pop-os/libcosmic), serde, cosmic_config, i18n-embed-fl, rust-embed, tracing

---

## Task 1: Project Scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/app.rs`
- Create: `src/app/mod.rs` (re-exports)
- Create: `src/app/config.rs`
- Create: `src/app/settings.rs`
- Create: `src/core/mod.rs`
- Create: `src/core/localization.rs`
- Create: `src/core/icons.rs`
- Create: `src/core/keybinds.rs`
- Create: `src/engine/mod.rs`
- Create: `src/ui/mod.rs`
- Create: `i18n/en/app.ftl`
- Create: `res/icons/bundled/.gitkeep`

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "cosmic-ext-calculator"
version = "0.1.0"
edition = "2024"
license = "GPL-3.0-only"

[dependencies]
i18n-embed-fl = "0.8"
open = "5.3"
rust-embed = "8.3"
serde = { version = "1.0", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[dependencies.libcosmic]
git = "https://github.com/pop-os/libcosmic.git"
default-features = false
features = ["tokio", "winit", "about", "wayland"]

[features]
default = ["i18n"]
i18n = ["dep:i18n-embed"]

[dependencies.i18n-embed]
version = "0.14"
features = ["fluent-system", "desktop-requester"]
optional = true
```

- [ ] **Step 2: Create i18n/en/app.ftl**

```ftl
app-title = Calculator
view = View
standard = Standard
engineering = Engineering
financial = Financial
clear-history = Clear History
about = About
delete = Delete
support = Support
repository = Repository
```

- [ ] **Step 3: Create src/core/localization.rs**

```rust
use std::sync::LazyLock;

use i18n_embed::{
    DefaultLocalizer, LanguageLoader, Localizer,
    fluent::{FluentLanguageLoader, fluent_language_loader},
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader: FluentLanguageLoader = fluent_language_loader!();
    loader
        .load_fallback_language(&Localizations)
        .expect("Error while loading fallback language");
    loader
});

#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::core::localization::LANGUAGE_LOADER, $message_id)
    }};
    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::core::localization::LANGUAGE_LOADER, $message_id, $($args), *)
    }};
}

pub fn localizer() -> Box<dyn Localizer> {
    Box::from(DefaultLocalizer::new(&*LANGUAGE_LOADER, &Localizations))
}

pub fn localize() {
    let localizer = localizer();
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    if let Err(error) = localizer.select(&requested_languages) {
        eprintln!("Error while loading language: {error}");
    }
}
```

- [ ] **Step 4: Create src/core/icons.rs**

```rust
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use cosmic::widget::icon;

pub(crate) static ICON_CACHE: OnceLock<Mutex<IconCache>> = OnceLock::new();

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct IconCacheKey {
    name: &'static str,
    size: u16,
}

pub struct IconCache {
    cache: HashMap<IconCacheKey, icon::Handle>,
}

impl IconCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn get_handle(&mut self, name: &'static str, size: u16) -> icon::Handle {
        self.cache
            .entry(IconCacheKey { name, size })
            .or_insert_with(|| icon::from_name(name).size(size).handle())
            .clone()
    }
}

pub fn get_handle(name: &'static str, size: u16) -> icon::Handle {
    let mut icon_cache = ICON_CACHE.get().unwrap().lock().unwrap();
    icon_cache.get_handle(name, size)
}
```

- [ ] **Step 5: Create src/core/keybinds.rs**

```rust
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
    bind!([Ctrl, Shift], Key::Character("C".into()), ClearHistory);
    bind!([Ctrl], Key::Character("i".into()), About);

    key_binds
}
```

- [ ] **Step 6: Create src/core/mod.rs**

```rust
pub mod icons;
pub mod keybinds;
pub mod localization;
```

- [ ] **Step 7: Create src/app/config.rs**

```rust
use cosmic::{
    Application,
    cosmic_config::{self, Config, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry},
    theme,
};
use serde::{Deserialize, Serialize};

use crate::app::CosmicCalculator;

pub const CONFIG_VERSION: u64 = 1;

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize, CosmicConfigEntry)]
pub struct CalculatorConfig {
    pub app_theme: AppTheme,
    pub history: Vec<HistoryEntry>,
    pub angle_mode: AngleMode,
    pub tax_rate: f64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct HistoryEntry {
    pub expression: String,
    pub result: String,
    pub mode: Mode,
    pub timestamp: i64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum Mode {
    #[default]
    Standard,
    Engineering,
    Financial,
}

impl Mode {
    pub fn label(&self) -> &'static str {
        match self {
            Mode::Standard => "STD",
            Mode::Engineering => "ENG",
            Mode::Financial => "FIN",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum AngleMode {
    #[default]
    Deg,
    Rad,
    Grad,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum AppTheme {
    Dark,
    Light,
    #[default]
    System,
}

impl AppTheme {
    pub fn theme(&self) -> theme::Theme {
        match self {
            Self::Dark => {
                let mut t = theme::system_dark();
                t.theme_type.prefer_dark(Some(true));
                t
            }
            Self::Light => {
                let mut t = theme::system_light();
                t.theme_type.prefer_dark(Some(false));
                t
            }
            Self::System => theme::system_preference(),
        }
    }
}

impl CalculatorConfig {
    pub fn config_handler() -> Option<Config> {
        Config::new(CosmicCalculator::APP_ID, CONFIG_VERSION).ok()
    }

    pub fn config() -> CalculatorConfig {
        match Self::config_handler() {
            Some(config_handler) => {
                CalculatorConfig::get_entry(&config_handler).unwrap_or_else(|(errs, config)| {
                    tracing::info!("errors loading config: {:?}", errs);
                    config
                })
            }
            None => CalculatorConfig::default(),
        }
    }
}
```

- [ ] **Step 8: Create src/app/settings.rs**

```rust
use std::sync::Mutex;

use crate::{
    app::{Flags, config::CalculatorConfig},
    core::{
        icons::{ICON_CACHE, IconCache},
        localization::localize,
    },
};
use cosmic::{
    app::Settings,
    iced::{Limits, Size},
};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() {
    localize();
    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "cosmic_ext_calculator=info");
        }
    }
    tracing_subscriber::registry()
        .with(EnvFilter::from_env("RUST_LOG"))
        .with(tracing_subscriber::fmt::layer())
        .init();
    ICON_CACHE.get_or_init(|| Mutex::new(IconCache::new()));
}

pub fn settings() -> Settings {
    let config = CalculatorConfig::config();
    let mut settings = Settings::default();
    settings = settings.theme(config.app_theme.theme());
    settings = settings.size_limits(Limits::NONE.min_width(320.0).min_height(500.0));
    settings = settings.size(Size::new(320.0, 580.0));
    settings = settings.debug(false);
    settings
}

pub fn flags() -> Flags {
    let (config_handler, config) = (
        CalculatorConfig::config_handler(),
        CalculatorConfig::config(),
    );
    Flags {
        config_handler,
        config,
    }
}
```

- [ ] **Step 9: Create src/engine/mod.rs (stubs)**

```rust
pub mod standard;

#[derive(Debug, Clone)]
pub struct CalcResult {
    pub value: f64,
    pub display: String,
    pub alt_bases: Option<AltBases>,
}

#[derive(Debug, Clone)]
pub struct AltBases {
    pub hex: String,
    pub oct: String,
    pub bin: String,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum CalcError {
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Invalid expression: {0}")]
    InvalidExpression(String),
    #[error("Domain error: {0}")]
    DomainError(String),
    #[error("Overflow")]
    Overflow,
    #[error("Convergence error")]
    ConvergenceError,
}

pub trait Evaluate {
    fn evaluate(&self, expr: &str) -> Result<CalcResult, CalcError>;
}
```

Note: Add `thiserror = "2"` to `[dependencies]` in `Cargo.toml`.

- [ ] **Step 10: Create src/engine/standard.rs (minimal stub)**

```rust
use super::{CalcError, CalcResult, Evaluate};

pub struct StandardEngine;

impl Evaluate for StandardEngine {
    fn evaluate(&self, expr: &str) -> Result<CalcResult, CalcError> {
        Err(CalcError::InvalidExpression("not yet implemented".into()))
    }
}
```

- [ ] **Step 11: Create src/ui/mod.rs (stub)**

```rust
pub mod standard;
```

- [ ] **Step 12: Create src/ui/standard.rs (stub)**

```rust
use cosmic::prelude::*;
use cosmic::widget;
use cosmic::iced::Length;

use crate::app::Message;

pub fn view<'a>() -> Element<'a, Message> {
    widget::text("Standard mode placeholder").into()
}
```

- [ ] **Step 13: Create src/app.rs with minimal Application trait impl**

```rust
pub mod config;
pub mod settings;

use std::any::TypeId;
use std::collections::HashMap;

use config::{CalculatorConfig, HistoryEntry, Mode, CONFIG_VERSION};
use cosmic::app::context_drawer;
use cosmic::iced::keyboard::{Key, Modifiers};
use cosmic::iced::keyboard::Event as KeyEvent;
use cosmic::iced::Event;
use cosmic::iced::event;
use cosmic::prelude::*;
use cosmic::widget::about::About;
use cosmic::widget::menu::{self, ItemHeight, ItemWidth};
use cosmic::widget::segmented_button;
use cosmic::widget::{self, nav_bar, toaster::ToastId};
use cosmic::{Application, Core, Task, theme};
use cosmic::widget::menu::action::MenuAction as MenuActionTrait;

use crate::core::icons;
use crate::core::keybinds::key_binds;
use crate::fl;
use crate::ui;

pub struct CosmicCalculator {
    core: Core,
    about: About,
    context_page: ContextPage,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    modifiers: Modifiers,
    config_handler: Option<cosmic::cosmic_config::Config>,
    config: CalculatorConfig,
    mode: Mode,
    mode_model: segmented_button::SingleSelectModel,
    expression: String,
    display: String,
    history: Vec<HistoryEntry>,
    toasts: widget::Toasts<Message>,
    input_id: widget::Id,
}

#[derive(Debug, Clone)]
pub enum Message {
    ModeSelected(segmented_button::Entity),
    Input(String),
    ToggleContextPage(ContextPage),
    ToggleContextDrawer,
    Key(Modifiers, Key),
    Modifiers(Modifiers),
    SystemThemeModeChange,
    CleanHistory,
    ShowToast(String),
    CloseToast(ToastId),
    Open(String),
    Window,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    History,
}

pub struct Flags {
    pub config_handler: Option<cosmic::cosmic_config::Config>,
    pub config: CalculatorConfig,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    ClearHistory,
    ToggleHistory,
    SwitchStandard,
    SwitchEngineering,
    SwitchFinancial,
    Undo,
}

impl MenuActionTrait for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::ClearHistory => Message::CleanHistory,
            MenuAction::ToggleHistory => Message::ToggleContextPage(ContextPage::History),
            MenuAction::SwitchStandard | MenuAction::SwitchEngineering | MenuAction::SwitchFinancial | MenuAction::Undo => {
                // Handled directly in key event processing
                Message::ToggleContextDrawer
            }
        }
    }
}

impl Application for CosmicCalculator {
    type Executor = cosmic::executor::Default;
    type Flags = Flags;
    type Message = Message;
    const APP_ID: &'static str = "dev.dcristob.Calculator";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let mode_model = segmented_button::Model::builder()
            .insert(|b| b.text(fl!("standard")).data(Mode::Standard).activate())
            .insert(|b| b.text(fl!("engineering")).data(Mode::Engineering))
            .insert(|b| b.text(fl!("financial")).data(Mode::Financial))
            .build();

        let about = About::default()
            .name(fl!("app-title"))
            .icon(widget::icon::from_name(Self::APP_ID))
            .version("0.1.0")
            .license("GPL-3.0-only")
            .links([
                (fl!("support"), "https://github.com/dcristob/cosmic-calculator/issues"),
                (fl!("repository"), "https://github.com/dcristob/cosmic-calculator"),
            ]);

        let history = flags.config.history.clone();

        let mut app = CosmicCalculator {
            core,
            about,
            context_page: ContextPage::default(),
            key_binds: key_binds(),
            modifiers: Modifiers::empty(),
            config_handler: flags.config_handler,
            config: flags.config,
            mode: Mode::Standard,
            mode_model,
            expression: String::new(),
            display: String::from("0"),
            history,
            toasts: widget::toaster::Toasts::new(Message::CloseToast),
            input_id: widget::Id::unique(),
        };

        let mut tasks = vec![];
        tasks.push(app.set_window_title(fl!("app-title")));

        (app, Task::batch(tasks))
    }

    fn header_start<'a>(&'a self) -> Vec<Element<'a, Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")),
            menu::items(
                &self.key_binds,
                vec![
                    menu::Item::Button(
                        fl!("clear-history"),
                        Some(icons::get_handle("edit-clear-all-symbolic", 14)),
                        MenuAction::ClearHistory,
                    ),
                    menu::Item::Button(
                        fl!("about"),
                        Some(icons::get_handle("help-about-symbolic", 14)),
                        MenuAction::About,
                    ),
                ],
            ),
        )])
        .item_height(ItemHeight::Dynamic(40))
        .item_width(ItemWidth::Uniform(240))
        .spacing(4.0);

        vec![menu_bar.into()]
    }

    fn view<'a>(&'a self) -> Element<'a, Self::Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;

        let tab_bar = widget::segmented_button::horizontal(&self.mode_model)
            .on_activate(Message::ModeSelected)
            .width(Length::Fill);

        let display = widget::column::with_capacity(2)
            .push(
                widget::text::body(&self.expression)
                    .width(Length::Fill)
                    .align_x(cosmic::iced::Alignment::End),
            )
            .push(
                widget::text::title1(&self.display)
                    .width(Length::Fill)
                    .align_x(cosmic::iced::Alignment::End),
            )
            .padding(spacing.space_s);

        let button_grid = match self.mode {
            Mode::Standard => ui::standard::view(),
            Mode::Engineering => widget::text("Engineering mode - coming soon").into(),
            Mode::Financial => widget::text("Financial mode - coming soon").into(),
        };

        widget::column::with_capacity(4)
            .push(tab_bar)
            .push(display)
            .push(button_grid)
            .push(
                widget::row::with_capacity(1)
                    .push(widget::toaster(&self.toasts, widget::horizontal_space())),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(spacing.space_xs)
            .padding(spacing.space_xxs)
            .into()
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        let mut tasks = vec![];
        match message {
            Message::ModeSelected(entity) => {
                self.mode_model.activate(entity);
                if let Some(mode) = self.mode_model.data::<Mode>(entity) {
                    self.mode = *mode;
                }
            }
            Message::Input(value) => {
                self.expression = value;
            }
            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }
            Message::ToggleContextDrawer => {
                self.core.window.show_context = !self.core.window.show_context;
            }
            Message::Key(modifiers, key) => {
                for (key_bind, action) in &self.key_binds {
                    if key_bind.matches(modifiers, &key) {
                        return self.update(action.message());
                    }
                }
            }
            Message::Modifiers(modifiers) => {
                self.modifiers = modifiers;
            }
            Message::SystemThemeModeChange => {}
            Message::CleanHistory => {
                self.history.clear();
                if let Some(config_handler) = &self.config_handler {
                    let _ = self.config.set_history(config_handler, Vec::new());
                }
            }
            Message::ShowToast(message) => {
                tasks.push(
                    self.toasts
                        .push(widget::toaster::Toast::new(message))
                        .map(cosmic::Action::App),
                );
            }
            Message::CloseToast(id) => {
                self.toasts.remove(id);
            }
            Message::Open(url) => {
                if let Err(err) = open::that_detached(&url) {
                    tracing::error!("Failed to open {url}: {err}");
                }
            }
            Message::Window => {}
        }
        Task::batch(tasks)
    }

    fn context_drawer<'a>(&'a self) -> Option<context_drawer::ContextDrawer<'a, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::about(
                &self.about,
                |url| Message::Open(url.to_string()),
                Message::ToggleContextDrawer,
            ),
            ContextPage::History => {
                let content = widget::text("History - coming soon");
                context_drawer::context_drawer(content, Message::ToggleContextDrawer)
            }
        })
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        let subscriptions = vec![
            event::listen_with(|event, status, _id| match event {
                Event::Keyboard(KeyEvent::KeyPressed { key, modifiers, .. }) => match status {
                    event::Status::Ignored => Some(Message::Key(modifiers, key)),
                    event::Status::Captured => None,
                },
                Event::Keyboard(KeyEvent::ModifiersChanged(modifiers)) => {
                    Some(Message::Modifiers(modifiers))
                }
                Event::Window(cosmic::iced::window::Event::Focused) => Some(Message::Window),
                _ => None,
            }),
            cosmic::cosmic_config::config_subscription(
                TypeId::of::<()>(),
                Self::APP_ID.into(),
                CONFIG_VERSION,
            )
            .map(|_| Message::SystemThemeModeChange),
        ];

        cosmic::iced::Subscription::batch(subscriptions)
    }
}
```

- [ ] **Step 14: Create src/main.rs**

```rust
use app::CosmicCalculator;

mod app;
mod core;

fn main() -> cosmic::iced::Result {
    app::settings::init();
    let (settings, flags) = (app::settings::settings(), app::settings::flags());
    cosmic::app::run::<CosmicCalculator>(settings, flags)
}
```

- [ ] **Step 15: Build and verify the scaffold compiles**

Run: `cargo build 2>&1`
Expected: Successful compilation (warnings about unused code are fine)

- [ ] **Step 16: Commit**

```bash
git add -A
git commit -m "feat: scaffold COSMIC calculator with mode tabs and app shell"
```

---

## Task 2: Expression Parser

**Files:**
- Create: `src/engine/parser.rs`
- Create: `tests/parser_tests.rs`

- [ ] **Step 1: Add test file with basic arithmetic tests**

Create `tests/parser_tests.rs`:

```rust
use cosmic_ext_calculator::engine::parser::Parser;

#[test]
fn test_integer_literal() {
    let mut p = Parser::new();
    assert_eq!(p.parse("42").unwrap(), 42.0);
}

#[test]
fn test_decimal_literal() {
    let mut p = Parser::new();
    assert_eq!(p.parse("3.14").unwrap(), 3.14);
}

#[test]
fn test_addition() {
    let mut p = Parser::new();
    assert_eq!(p.parse("2+3").unwrap(), 5.0);
}

#[test]
fn test_subtraction() {
    let mut p = Parser::new();
    assert_eq!(p.parse("10-4").unwrap(), 6.0);
}

#[test]
fn test_multiplication() {
    let mut p = Parser::new();
    assert_eq!(p.parse("3*7").unwrap(), 21.0);
}

#[test]
fn test_division() {
    let mut p = Parser::new();
    assert_eq!(p.parse("15/3").unwrap(), 5.0);
}

#[test]
fn test_division_by_zero() {
    let mut p = Parser::new();
    assert!(p.parse("1/0").is_err());
}

#[test]
fn test_precedence_mul_over_add() {
    let mut p = Parser::new();
    assert_eq!(p.parse("2+3*4").unwrap(), 14.0);
}

#[test]
fn test_parentheses() {
    let mut p = Parser::new();
    assert_eq!(p.parse("(2+3)*4").unwrap(), 20.0);
}

#[test]
fn test_nested_parentheses() {
    let mut p = Parser::new();
    assert_eq!(p.parse("((2+3)*(4+1))").unwrap(), 25.0);
}

#[test]
fn test_unary_minus() {
    let mut p = Parser::new();
    assert_eq!(p.parse("-5").unwrap(), -5.0);
}

#[test]
fn test_unary_minus_in_expression() {
    let mut p = Parser::new();
    assert_eq!(p.parse("3+-2").unwrap(), 1.0);
}

#[test]
fn test_power() {
    let mut p = Parser::new();
    assert_eq!(p.parse("2^10").unwrap(), 1024.0);
}

#[test]
fn test_power_right_associative() {
    let mut p = Parser::new();
    assert_eq!(p.parse("2^3^2").unwrap(), 512.0); // 2^(3^2) = 2^9
}

#[test]
fn test_modulus() {
    let mut p = Parser::new();
    assert_eq!(p.parse("17%5").unwrap(), 2.0);
}

#[test]
fn test_percentage() {
    let mut p = Parser::new();
    // 200 + 10% should mean 200 + (10% of 200) = 220
    // But as standalone: 50% = 0.5
    assert_eq!(p.parse("50%").unwrap(), 0.5);
}

#[test]
fn test_implicit_mul_paren() {
    let mut p = Parser::new();
    assert_eq!(p.parse("3(4+1)").unwrap(), 15.0);
}

#[test]
fn test_complex_expression() {
    let mut p = Parser::new();
    let result = p.parse("(3+4)*2-1").unwrap();
    assert_eq!(result, 13.0);
}

#[test]
fn test_invalid_expression() {
    let mut p = Parser::new();
    assert!(p.parse("2++").is_err());
}

#[test]
fn test_empty_expression() {
    let mut p = Parser::new();
    assert!(p.parse("").is_err());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test parser_tests 2>&1`
Expected: Compilation error — `parser` module does not exist

- [ ] **Step 3: Create the parser with a lib.rs for test access**

Create `src/lib.rs`:

```rust
pub mod engine;
```

Create `src/engine/parser.rs`:

```rust
use super::CalcError;

#[derive(Debug)]
enum Token {
    Number(f64),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    LParen,
    RParen,
    Func(String),
    Eof,
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            tokens: Vec::new(),
            pos: 0,
        }
    }

    pub fn parse(&mut self, input: &str) -> Result<f64, CalcError> {
        self.tokens = self.tokenize(input)?;
        self.pos = 0;
        if self.tokens.is_empty() || matches!(self.tokens[0], Token::Eof) {
            return Err(CalcError::InvalidExpression("empty expression".into()));
        }
        let result = self.parse_expression()?;
        if !matches!(self.peek(), Token::Eof) {
            return Err(CalcError::InvalidExpression("unexpected token after expression".into()));
        }
        Ok(result)
    }

    pub fn parse_with_functions<F>(&mut self, input: &str, func_eval: F) -> Result<f64, CalcError>
    where
        F: Fn(&str, &[f64]) -> Result<f64, CalcError>,
    {
        self.tokens = self.tokenize(input)?;
        self.pos = 0;
        if self.tokens.is_empty() || matches!(self.tokens[0], Token::Eof) {
            return Err(CalcError::InvalidExpression("empty expression".into()));
        }
        let result = self.parse_expression_with_funcs(&func_eval)?;
        if !matches!(self.peek(), Token::Eof) {
            return Err(CalcError::InvalidExpression("unexpected token after expression".into()));
        }
        Ok(result)
    }

    fn tokenize(&self, input: &str) -> Result<Vec<Token>, CalcError> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            match chars[i] {
                ' ' | '\t' => { i += 1; }
                '+' => { tokens.push(Token::Plus); i += 1; }
                '-' => { tokens.push(Token::Minus); i += 1; }
                '*' | '×' => { tokens.push(Token::Star); i += 1; }
                '/' | '÷' => { tokens.push(Token::Slash); i += 1; }
                '%' => { tokens.push(Token::Percent); i += 1; }
                '^' => { tokens.push(Token::Caret); i += 1; }
                '(' => { tokens.push(Token::LParen); i += 1; }
                ')' => { tokens.push(Token::RParen); i += 1; }
                c if c.is_ascii_digit() || c == '.' => {
                    let start = i;
                    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                        i += 1;
                    }
                    let num_str: String = chars[start..i].iter().collect();
                    let num = num_str.parse::<f64>().map_err(|_| {
                        CalcError::InvalidExpression(format!("invalid number: {num_str}"))
                    })?;
                    tokens.push(Token::Number(num));
                }
                c if c.is_alphabetic() || c == 'π' => {
                    let start = i;
                    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                        i += 1;
                    }
                    let name: String = chars[start..i].iter().collect();
                    match name.as_str() {
                        "π" | "pi" => tokens.push(Token::Number(std::f64::consts::PI)),
                        "e" if !matches!(chars.get(i), Some('(')) => {
                            tokens.push(Token::Number(std::f64::consts::E));
                        }
                        _ => tokens.push(Token::Func(name)),
                    }
                }
                c => {
                    return Err(CalcError::InvalidExpression(format!("unexpected character: {c}")));
                }
            }
        }
        tokens.push(Token::Eof);
        // Insert implicit multiplication
        let mut result = Vec::new();
        for i in 0..tokens.len() {
            result.push(std::mem::replace(&mut tokens[0], Token::Eof));
            tokens.remove(0);
            // Removed — we'll handle this below
        }
        // Redo: build with implicit multiplication
        let mut final_tokens = Vec::new();
        let result = tokens; // already consumed above incorrectly, let's redo

        // Actually, let's redo tokenization properly with implicit mul
        Ok(self.insert_implicit_mul(self.raw_tokenize(input)?))
    }

    fn raw_tokenize(&self, input: &str) -> Result<Vec<Token>, CalcError> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            match chars[i] {
                ' ' | '\t' => { i += 1; }
                '+' => { tokens.push(Token::Plus); i += 1; }
                '-' => { tokens.push(Token::Minus); i += 1; }
                '*' | '×' => { tokens.push(Token::Star); i += 1; }
                '/' | '÷' => { tokens.push(Token::Slash); i += 1; }
                '%' => { tokens.push(Token::Percent); i += 1; }
                '^' => { tokens.push(Token::Caret); i += 1; }
                '(' => { tokens.push(Token::LParen); i += 1; }
                ')' => { tokens.push(Token::RParen); i += 1; }
                c if c.is_ascii_digit() || c == '.' => {
                    let start = i;
                    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                        i += 1;
                    }
                    let num_str: String = chars[start..i].iter().collect();
                    let num = num_str.parse::<f64>().map_err(|_| {
                        CalcError::InvalidExpression(format!("invalid number: {num_str}"))
                    })?;
                    tokens.push(Token::Number(num));
                }
                c if c.is_alphabetic() || c == 'π' => {
                    if c == 'π' {
                        tokens.push(Token::Number(std::f64::consts::PI));
                        i += c.len_utf8();
                        continue;
                    }
                    let start = i;
                    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                        i += 1;
                    }
                    let name: String = chars[start..i].iter().collect();
                    match name.as_str() {
                        "pi" => tokens.push(Token::Number(std::f64::consts::PI)),
                        "e" => tokens.push(Token::Number(std::f64::consts::E)),
                        _ => tokens.push(Token::Func(name)),
                    }
                }
                c => {
                    return Err(CalcError::InvalidExpression(format!("unexpected character: {c}")));
                }
            }
        }
        tokens.push(Token::Eof);
        Ok(tokens)
    }

    fn insert_implicit_mul(&self, tokens: Vec<Token>) -> Vec<Token> {
        let mut result = Vec::new();
        for (i, token) in tokens.iter().enumerate() {
            if i > 0 {
                let prev = &result[result.len() - 1];
                let needs_mul = match (prev, token) {
                    (Token::Number(_), Token::LParen) => true,
                    (Token::Number(_), Token::Func(_)) => true,
                    (Token::Number(_), Token::Number(_)) => false, // shouldn't happen
                    (Token::RParen, Token::LParen) => true,
                    (Token::RParen, Token::Number(_)) => true,
                    (Token::RParen, Token::Func(_)) => true,
                    _ => false,
                };
                if needs_mul {
                    result.push(Token::Star);
                }
            }
            // Clone token
            result.push(match token {
                Token::Number(n) => Token::Number(*n),
                Token::Plus => Token::Plus,
                Token::Minus => Token::Minus,
                Token::Star => Token::Star,
                Token::Slash => Token::Slash,
                Token::Percent => Token::Percent,
                Token::Caret => Token::Caret,
                Token::LParen => Token::LParen,
                Token::RParen => Token::RParen,
                Token::Func(s) => Token::Func(s.clone()),
                Token::Eof => Token::Eof,
            });
        }
        result
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> &Token {
        let token = self.tokens.get(self.pos).unwrap_or(&Token::Eof);
        self.pos += 1;
        token
    }

    // Grammar:
    // expression = term (('+' | '-') term)*
    // term       = power (('*' | '/' | '%') power)*
    // power      = unary ('^' power)?        (right-associative)
    // unary      = ('-' | '+') unary | postfix
    // postfix    = primary '%'?
    // primary    = NUMBER | '(' expression ')' | FUNC '(' args ')'

    fn parse_expression(&mut self) -> Result<f64, CalcError> {
        let mut left = self.parse_term()?;
        loop {
            match self.peek() {
                Token::Plus => { self.advance(); left += self.parse_term()?; }
                Token::Minus => { self.advance(); left -= self.parse_term()?; }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_expression_with_funcs<F>(&mut self, func_eval: &F) -> Result<f64, CalcError>
    where
        F: Fn(&str, &[f64]) -> Result<f64, CalcError>,
    {
        let mut left = self.parse_term_with_funcs(func_eval)?;
        loop {
            match self.peek() {
                Token::Plus => { self.advance(); left += self.parse_term_with_funcs(func_eval)?; }
                Token::Minus => { self.advance(); left -= self.parse_term_with_funcs(func_eval)?; }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<f64, CalcError> {
        let mut left = self.parse_power()?;
        loop {
            match self.peek() {
                Token::Star => { self.advance(); left *= self.parse_power()?; }
                Token::Slash => {
                    self.advance();
                    let right = self.parse_power()?;
                    if right == 0.0 {
                        return Err(CalcError::DivisionByZero);
                    }
                    left /= right;
                }
                Token::Percent if !matches!(self.tokens.get(self.pos + 1), Some(Token::Eof) | None) => {
                    // modulus operator when followed by more tokens
                    // but only if next token is not Eof — handled in postfix for standalone %
                    self.advance();
                    let right = self.parse_power()?;
                    if right == 0.0 {
                        return Err(CalcError::DivisionByZero);
                    }
                    left = left % right;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_term_with_funcs<F>(&mut self, func_eval: &F) -> Result<f64, CalcError>
    where
        F: Fn(&str, &[f64]) -> Result<f64, CalcError>,
    {
        let mut left = self.parse_power_with_funcs(func_eval)?;
        loop {
            match self.peek() {
                Token::Star => { self.advance(); left *= self.parse_power_with_funcs(func_eval)?; }
                Token::Slash => {
                    self.advance();
                    let right = self.parse_power_with_funcs(func_eval)?;
                    if right == 0.0 {
                        return Err(CalcError::DivisionByZero);
                    }
                    left /= right;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_power(&mut self) -> Result<f64, CalcError> {
        let base = self.parse_unary()?;
        if matches!(self.peek(), Token::Caret) {
            self.advance();
            let exp = self.parse_power()?; // right-associative recursion
            Ok(base.powf(exp))
        } else {
            Ok(base)
        }
    }

    fn parse_power_with_funcs<F>(&mut self, func_eval: &F) -> Result<f64, CalcError>
    where
        F: Fn(&str, &[f64]) -> Result<f64, CalcError>,
    {
        let base = self.parse_unary_with_funcs(func_eval)?;
        if matches!(self.peek(), Token::Caret) {
            self.advance();
            let exp = self.parse_power_with_funcs(func_eval)?;
            Ok(base.powf(exp))
        } else {
            Ok(base)
        }
    }

    fn parse_unary(&mut self) -> Result<f64, CalcError> {
        match self.peek() {
            Token::Minus => {
                self.advance();
                Ok(-self.parse_unary()?)
            }
            Token::Plus => {
                self.advance();
                self.parse_unary()
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_unary_with_funcs<F>(&mut self, func_eval: &F) -> Result<f64, CalcError>
    where
        F: Fn(&str, &[f64]) -> Result<f64, CalcError>,
    {
        match self.peek() {
            Token::Minus => {
                self.advance();
                Ok(-self.parse_unary_with_funcs(func_eval)?)
            }
            Token::Plus => {
                self.advance();
                self.parse_unary_with_funcs(func_eval)
            }
            _ => self.parse_postfix_with_funcs(func_eval),
        }
    }

    fn parse_postfix(&mut self) -> Result<f64, CalcError> {
        let value = self.parse_primary()?;
        if matches!(self.peek(), Token::Percent) {
            // Check if % is used as postfix (standalone percentage) vs modulus
            // Postfix when followed by Eof, RParen, Plus, Minus, Star, Slash, Caret
            let next = self.tokens.get(self.pos + 1);
            let is_postfix = matches!(next, None | Some(Token::Eof) | Some(Token::RParen) | Some(Token::Plus) | Some(Token::Minus) | Some(Token::Star) | Some(Token::Slash) | Some(Token::Caret));
            if is_postfix {
                self.advance();
                return Ok(value / 100.0);
            }
        }
        Ok(value)
    }

    fn parse_postfix_with_funcs<F>(&mut self, func_eval: &F) -> Result<f64, CalcError>
    where
        F: Fn(&str, &[f64]) -> Result<f64, CalcError>,
    {
        let value = self.parse_primary_with_funcs(func_eval)?;
        if matches!(self.peek(), Token::Percent) {
            let next = self.tokens.get(self.pos + 1);
            let is_postfix = matches!(next, None | Some(Token::Eof) | Some(Token::RParen) | Some(Token::Plus) | Some(Token::Minus) | Some(Token::Star) | Some(Token::Slash) | Some(Token::Caret));
            if is_postfix {
                self.advance();
                return Ok(value / 100.0);
            }
        }
        Ok(value)
    }

    fn parse_primary(&mut self) -> Result<f64, CalcError> {
        match self.peek() {
            Token::Number(_) => {
                if let Token::Number(n) = self.advance() {
                    Ok(*n)
                } else {
                    unreachable!()
                }
            }
            Token::LParen => {
                self.advance();
                let result = self.parse_expression()?;
                if !matches!(self.peek(), Token::RParen) {
                    return Err(CalcError::InvalidExpression("missing closing parenthesis".into()));
                }
                self.advance();
                Ok(result)
            }
            Token::Func(_) => {
                Err(CalcError::InvalidExpression("functions not available in standard mode".into()))
            }
            _ => {
                Err(CalcError::InvalidExpression("unexpected token".into()))
            }
        }
    }

    fn parse_primary_with_funcs<F>(&mut self, func_eval: &F) -> Result<f64, CalcError>
    where
        F: Fn(&str, &[f64]) -> Result<f64, CalcError>,
    {
        match self.peek() {
            Token::Number(_) => {
                if let Token::Number(n) = self.advance() {
                    Ok(*n)
                } else {
                    unreachable!()
                }
            }
            Token::LParen => {
                self.advance();
                let result = self.parse_expression_with_funcs(func_eval)?;
                if !matches!(self.peek(), Token::RParen) {
                    return Err(CalcError::InvalidExpression("missing closing parenthesis".into()));
                }
                self.advance();
                Ok(result)
            }
            Token::Func(_) => {
                let name = if let Token::Func(s) = self.advance() {
                    s.clone()
                } else {
                    unreachable!()
                };
                if !matches!(self.peek(), Token::LParen) {
                    return Err(CalcError::InvalidExpression(format!("expected '(' after function '{name}'")));
                }
                self.advance(); // consume '('
                let mut args = Vec::new();
                if !matches!(self.peek(), Token::RParen) {
                    args.push(self.parse_expression_with_funcs(func_eval)?);
                    while matches!(self.peek(), Token::Func(ref s) if s == ",") {
                        self.advance();
                        args.push(self.parse_expression_with_funcs(func_eval)?);
                    }
                    // Also handle comma as separate token if needed
                }
                if !matches!(self.peek(), Token::RParen) {
                    return Err(CalcError::InvalidExpression(format!("missing ')' after function '{name}' arguments")));
                }
                self.advance(); // consume ')'
                func_eval(&name, &args)
            }
            _ => {
                Err(CalcError::InvalidExpression("unexpected token".into()))
            }
        }
    }
}
```

Update `src/engine/mod.rs` to add:
```rust
pub mod parser;
```

- [ ] **Step 4: Run tests**

Run: `cargo test --test parser_tests 2>&1`
Expected: All 20 tests pass

- [ ] **Step 5: Fix any failing tests and iterate**

If the `%` disambiguation between modulus and postfix-percentage causes failures, adjust the `parse_term` and `parse_postfix` logic. The rule: `%` is postfix-percentage when the next token is Eof, `)`, `+`, `-`, `*`, `/`, or `^`. Otherwise it's modulus.

- [ ] **Step 6: Commit**

```bash
git add src/engine/parser.rs src/engine/mod.rs src/lib.rs tests/parser_tests.rs
git commit -m "feat: add recursive descent expression parser with tests"
```

---

## Task 3: Standard Engine

**Files:**
- Modify: `src/engine/standard.rs`
- Create: `tests/standard_engine_tests.rs`

- [ ] **Step 1: Write tests**

Create `tests/standard_engine_tests.rs`:

```rust
use cosmic_ext_calculator::engine::{Evaluate, standard::StandardEngine};

#[test]
fn test_basic_arithmetic() {
    let engine = StandardEngine;
    assert_eq!(engine.evaluate("2+3").unwrap().value, 5.0);
    assert_eq!(engine.evaluate("10-4").unwrap().value, 6.0);
    assert_eq!(engine.evaluate("3*7").unwrap().value, 21.0);
    assert_eq!(engine.evaluate("15/3").unwrap().value, 5.0);
}

#[test]
fn test_precedence() {
    let engine = StandardEngine;
    assert_eq!(engine.evaluate("2+3*4").unwrap().value, 14.0);
    assert_eq!(engine.evaluate("(2+3)*4").unwrap().value, 20.0);
}

#[test]
fn test_percentage() {
    let engine = StandardEngine;
    assert_eq!(engine.evaluate("50%").unwrap().value, 0.5);
}

#[test]
fn test_modulus() {
    let engine = StandardEngine;
    assert_eq!(engine.evaluate("17%5").unwrap().value, 2.0);
}

#[test]
fn test_division_by_zero() {
    let engine = StandardEngine;
    assert!(engine.evaluate("1/0").is_err());
}

#[test]
fn test_display_formatting() {
    let engine = StandardEngine;
    let result = engine.evaluate("1/3").unwrap();
    // Should display reasonable precision
    assert!(result.display.contains("0.333"));
    assert!(result.alt_bases.is_none());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test standard_engine_tests 2>&1`
Expected: FAIL — StandardEngine returns Err for everything

- [ ] **Step 3: Implement StandardEngine**

Replace `src/engine/standard.rs`:

```rust
use super::{CalcError, CalcResult, Evaluate};
use super::parser::Parser;

pub struct StandardEngine;

impl StandardEngine {
    fn format_display(value: f64) -> String {
        if value.fract() == 0.0 && value.abs() < 1e15 {
            format!("{}", value as i64)
        } else {
            // Remove trailing zeros
            let s = format!("{:.10}", value);
            let s = s.trim_end_matches('0');
            let s = s.trim_end_matches('.');
            s.to_string()
        }
    }
}

impl Evaluate for StandardEngine {
    fn evaluate(&self, expr: &str) -> Result<CalcResult, CalcError> {
        let mut parser = Parser::new();
        let value = parser.parse(expr)?;

        if value.is_infinite() {
            return Err(CalcError::Overflow);
        }
        if value.is_nan() {
            return Err(CalcError::DomainError("result is not a number".into()));
        }

        Ok(CalcResult {
            value,
            display: Self::format_display(value),
            alt_bases: None,
        })
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test --test standard_engine_tests 2>&1`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add src/engine/standard.rs tests/standard_engine_tests.rs
git commit -m "feat: implement standard evaluation engine"
```

---

## Task 4: Standard Mode UI

**Files:**
- Modify: `src/ui/standard.rs`
- Modify: `src/app.rs` (add button messages, wire up evaluation)

- [ ] **Step 1: Define button messages in app.rs**

Add these variants to the `Message` enum in `src/app.rs`:

```rust
pub enum Message {
    // ... existing variants ...
    Number(u8),
    Operator(Operator),
    Evaluate,
    Clear,
    Backspace,
    Decimal,
    Percent,
    ParenOpen,
    ParenClose,
    Undo,
    CopyResult,
}
```

Add the `Operator` enum (can go in `src/app.rs` or a submodule):

```rust
#[derive(Debug, Clone, Copy)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl Operator {
    pub fn display(&self) -> &'static str {
        match self {
            Operator::Add => "+",
            Operator::Subtract => "−",
            Operator::Multiply => "×",
            Operator::Divide => "÷",
        }
    }

    pub fn expression(&self) -> &'static str {
        match self {
            Operator::Add => "+",
            Operator::Subtract => "-",
            Operator::Multiply => "*",
            Operator::Divide => "/",
        }
    }
}
```

- [ ] **Step 2: Implement the standard button grid in src/ui/standard.rs**

```rust
use cosmic::prelude::*;
use cosmic::widget;
use cosmic::iced::Length;
use cosmic::theme;

use crate::app::{Message, Operator};

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

fn num_btn<'a>(n: u8) -> Element<'a, Message> {
    calc_button(&n.to_string(), Message::Number(n), theme::Button::Standard)
}

fn op_btn<'a>(op: Operator) -> Element<'a, Message> {
    calc_button(op.display(), Message::Operator(op), theme::Button::Standard)
}

pub fn view<'a>() -> Element<'a, Message> {
    let spacing = cosmic::theme::active().cosmic().spacing;
    let std = theme::Button::Standard;
    let accent = theme::Button::Suggested;

    widget::column::with_capacity(6)
        // Row 1: C ( ) %
        .push(widget::row::with_capacity(4)
            .push(calc_button("C", Message::Clear, std))
            .push(calc_button("(", Message::ParenOpen, std))
            .push(calc_button(")", Message::ParenClose, std))
            .push(calc_button("%", Message::Percent, std))
            .spacing(spacing.space_xxs)
            .height(Length::Fill))
        // Row 2: 7 8 9 ÷
        .push(widget::row::with_capacity(4)
            .push(num_btn(7))
            .push(num_btn(8))
            .push(num_btn(9))
            .push(op_btn(Operator::Divide))
            .spacing(spacing.space_xxs)
            .height(Length::Fill))
        // Row 3: 4 5 6 ×
        .push(widget::row::with_capacity(4)
            .push(num_btn(4))
            .push(num_btn(5))
            .push(num_btn(6))
            .push(op_btn(Operator::Multiply))
            .spacing(spacing.space_xxs)
            .height(Length::Fill))
        // Row 4: 1 2 3 −
        .push(widget::row::with_capacity(4)
            .push(num_btn(1))
            .push(num_btn(2))
            .push(num_btn(3))
            .push(op_btn(Operator::Subtract))
            .spacing(spacing.space_xxs)
            .height(Length::Fill))
        // Row 5: 0 . ⌫ +
        .push(widget::row::with_capacity(4)
            .push(num_btn(0))
            .push(calc_button(".", Message::Decimal, std))
            .push(calc_button("⌫", Message::Backspace, std))
            .push(op_btn(Operator::Add))
            .spacing(spacing.space_xxs)
            .height(Length::Fill))
        // Row 6: = (full width)
        .push(calc_button("=", Message::Evaluate, accent))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
```

- [ ] **Step 3: Wire up message handling in app.rs update()**

Add these match arms in `update()`:

```rust
Message::Number(n) => {
    self.expression.push_str(&n.to_string());
}
Message::Operator(op) => {
    self.expression.push_str(op.expression());
}
Message::Evaluate => {
    use crate::engine::Evaluate;
    let engine = crate::engine::standard::StandardEngine;
    match engine.evaluate(&self.expression) {
        Ok(result) => {
            let entry = config::HistoryEntry {
                expression: self.expression.clone(),
                result: result.display.clone(),
                mode: self.mode,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
            };
            self.history.push(entry);
            if let Some(config_handler) = &self.config_handler {
                let _ = self.config.set_history(config_handler, self.history.clone());
            }
            self.display = result.display;
            self.expression = result.value.to_string();
        }
        Err(e) => {
            return self.update(Message::ShowToast(e.to_string()));
        }
    }
}
Message::Clear => {
    self.expression.clear();
    self.display = "0".into();
}
Message::Backspace => {
    self.expression.pop();
    if self.expression.is_empty() {
        self.display = "0".into();
    }
}
Message::Decimal => {
    self.expression.push('.');
}
Message::Percent => {
    self.expression.push('%');
}
Message::ParenOpen => {
    self.expression.push('(');
}
Message::ParenClose => {
    self.expression.push(')');
}
Message::Undo => {
    self.expression.pop();
}
Message::CopyResult => {
    // Will implement with clipboard API
}
```

- [ ] **Step 4: Build and verify**

Run: `cargo build 2>&1`
Expected: Compiles successfully

- [ ] **Step 5: Commit**

```bash
git add src/ui/standard.rs src/app.rs
git commit -m "feat: implement standard mode UI with button grid and evaluation"
```

---

## Task 5: Keyboard Input for Standard Mode

**Files:**
- Modify: `src/app.rs`

- [ ] **Step 1: Add keyboard handling in the Key message handler**

Replace the `Message::Key` handler in `update()`:

```rust
Message::Key(modifiers, key) => {
    // First check menu keybinds (Ctrl+combos)
    for (key_bind, action) in &self.key_binds {
        if key_bind.matches(modifiers, &key) {
            return self.update(action.message());
        }
    }
    // Then handle calculator keys (only when no modifier or just shift)
    if modifiers.is_empty() || modifiers == Modifiers::SHIFT {
        match &key {
            Key::Character(c) => match c.as_str() {
                "0" => return self.update(Message::Number(0)),
                "1" => return self.update(Message::Number(1)),
                "2" => return self.update(Message::Number(2)),
                "3" => return self.update(Message::Number(3)),
                "4" => return self.update(Message::Number(4)),
                "5" => return self.update(Message::Number(5)),
                "6" => return self.update(Message::Number(6)),
                "7" => return self.update(Message::Number(7)),
                "8" => return self.update(Message::Number(8)),
                "9" => return self.update(Message::Number(9)),
                "+" => return self.update(Message::Operator(Operator::Add)),
                "-" => return self.update(Message::Operator(Operator::Subtract)),
                "*" => return self.update(Message::Operator(Operator::Multiply)),
                "/" => return self.update(Message::Operator(Operator::Divide)),
                "." => return self.update(Message::Decimal),
                "(" => return self.update(Message::ParenOpen),
                ")" => return self.update(Message::ParenClose),
                "=" => return self.update(Message::Evaluate),
                "%" => return self.update(Message::Percent),
                _ => {}
            },
            Key::Named(named) => {
                use cosmic::iced::keyboard::key::Named;
                match named {
                    Named::Enter => return self.update(Message::Evaluate),
                    Named::Escape => return self.update(Message::Clear),
                    Named::Backspace => return self.update(Message::Backspace),
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
```

- [ ] **Step 2: Update keybinds.rs MenuAction handling**

Fix the `MenuAction::message()` method to properly route mode switches and undo:

```rust
impl MenuActionTrait for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::ClearHistory => Message::CleanHistory,
            MenuAction::ToggleHistory => Message::ToggleContextPage(ContextPage::History),
            MenuAction::SwitchStandard => Message::SwitchMode(Mode::Standard),
            MenuAction::SwitchEngineering => Message::SwitchMode(Mode::Engineering),
            MenuAction::SwitchFinancial => Message::SwitchMode(Mode::Financial),
            MenuAction::Undo => Message::Undo,
        }
    }
}
```

Add `SwitchMode(Mode)` to the `Message` enum and handle it in `update()`:

```rust
Message::SwitchMode(mode) => {
    self.mode = mode;
    // Activate the corresponding tab in the segmented button model
    for entity in self.mode_model.iter() {
        if self.mode_model.data::<Mode>(entity) == Some(&mode) {
            self.mode_model.activate(entity);
            break;
        }
    }
}
```

- [ ] **Step 3: Build and verify**

Run: `cargo build 2>&1`
Expected: Compiles successfully

- [ ] **Step 4: Commit**

```bash
git add src/app.rs src/core/keybinds.rs
git commit -m "feat: add full keyboard input for standard calculator mode"
```

---

## Task 6: Engineering Engine

**Files:**
- Create: `src/engine/engineering.rs`
- Create: `tests/engineering_engine_tests.rs`
- Modify: `src/engine/mod.rs`

- [ ] **Step 1: Write tests**

Create `tests/engineering_engine_tests.rs`:

```rust
use cosmic_ext_calculator::engine::{Evaluate, engineering::EngineeringEngine};
use cosmic_ext_calculator::app::config::AngleMode;

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn test_sin_deg() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("sin(90)").unwrap().value, 1.0));
}

#[test]
fn test_cos_rad() {
    let engine = EngineeringEngine::new(AngleMode::Rad);
    let result = engine.evaluate("cos(0)").unwrap().value;
    assert!(approx_eq(result, 1.0));
}

#[test]
fn test_tan() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("tan(45)").unwrap().value, 1.0));
}

#[test]
fn test_asin() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("asin(1)").unwrap().value, 90.0));
}

#[test]
fn test_log() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("log(100)").unwrap().value, 2.0));
}

#[test]
fn test_ln() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    let result = engine.evaluate("ln(1)").unwrap().value;
    assert!(approx_eq(result, 0.0));
}

#[test]
fn test_sqrt() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("sqrt(144)").unwrap().value, 12.0));
}

#[test]
fn test_factorial() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("fact(5)").unwrap().value, 120.0));
}

#[test]
fn test_abs() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("abs(-42)").unwrap().value, 42.0));
}

#[test]
fn test_floor_ceil() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("floor(3.7)").unwrap().value, 3.0));
    assert!(approx_eq(engine.evaluate("ceil(3.2)").unwrap().value, 4.0));
}

#[test]
fn test_power_in_expression() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("2^10").unwrap().value, 1024.0));
}

#[test]
fn test_constants() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    let result = engine.evaluate("pi").unwrap().value;
    assert!(approx_eq(result, std::f64::consts::PI));
}

#[test]
fn test_complex_expression() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    // sin(30) = 0.5, so 2*sin(30) + 1 = 2.0
    let result = engine.evaluate("2*sin(30)+1").unwrap().value;
    assert!(approx_eq(result, 2.0));
}

#[test]
fn test_alt_bases() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    let result = engine.evaluate("255").unwrap();
    let bases = result.alt_bases.unwrap();
    assert_eq!(bases.hex, "FF");
    assert_eq!(bases.oct, "377");
    assert_eq!(bases.bin, "11111111");
}

#[test]
fn test_sqrt_negative_domain_error() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(engine.evaluate("sqrt(-1)").is_err());
}

#[test]
fn test_log_zero_domain_error() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(engine.evaluate("log(0)").is_err());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test engineering_engine_tests 2>&1`
Expected: FAIL — module doesn't exist

- [ ] **Step 3: Implement EngineeringEngine**

Create `src/engine/engineering.rs`:

```rust
use crate::app::config::AngleMode;
use super::{AltBases, CalcError, CalcResult, Evaluate};
use super::parser::Parser;

pub struct EngineeringEngine {
    angle_mode: AngleMode,
}

impl EngineeringEngine {
    pub fn new(angle_mode: AngleMode) -> Self {
        Self { angle_mode }
    }

    fn to_radians(&self, value: f64) -> f64 {
        match self.angle_mode {
            AngleMode::Rad => value,
            AngleMode::Deg => value.to_radians(),
            AngleMode::Grad => value * std::f64::consts::PI / 200.0,
        }
    }

    fn from_radians(&self, value: f64) -> f64 {
        match self.angle_mode {
            AngleMode::Rad => value,
            AngleMode::Deg => value.to_degrees(),
            AngleMode::Grad => value * 200.0 / std::f64::consts::PI,
        }
    }

    fn eval_function(&self, name: &str, args: &[f64]) -> Result<f64, CalcError> {
        let arg = |i: usize| -> Result<f64, CalcError> {
            args.get(i)
                .copied()
                .ok_or_else(|| CalcError::InvalidExpression(format!("{name}: missing argument {i}")))
        };

        match name {
            "sin" => Ok(self.to_radians(arg(0)?).sin()),
            "cos" => Ok(self.to_radians(arg(0)?).cos()),
            "tan" => Ok(self.to_radians(arg(0)?).tan()),
            "asin" => {
                let v = arg(0)?;
                if v < -1.0 || v > 1.0 {
                    return Err(CalcError::DomainError("asin argument must be in [-1, 1]".into()));
                }
                Ok(self.from_radians(v.asin()))
            }
            "acos" => {
                let v = arg(0)?;
                if v < -1.0 || v > 1.0 {
                    return Err(CalcError::DomainError("acos argument must be in [-1, 1]".into()));
                }
                Ok(self.from_radians(v.acos()))
            }
            "atan" => Ok(self.from_radians(arg(0)?.atan())),
            "log" => {
                let v = arg(0)?;
                if v <= 0.0 {
                    return Err(CalcError::DomainError("log argument must be positive".into()));
                }
                Ok(v.log10())
            }
            "ln" => {
                let v = arg(0)?;
                if v <= 0.0 {
                    return Err(CalcError::DomainError("ln argument must be positive".into()));
                }
                Ok(v.ln())
            }
            "sqrt" => {
                let v = arg(0)?;
                if v < 0.0 {
                    return Err(CalcError::DomainError("cannot take square root of negative number".into()));
                }
                Ok(v.sqrt())
            }
            "abs" => Ok(arg(0)?.abs()),
            "floor" => Ok(arg(0)?.floor()),
            "ceil" => Ok(arg(0)?.ceil()),
            "fact" => {
                let v = arg(0)?;
                if v < 0.0 || v.fract() != 0.0 {
                    return Err(CalcError::DomainError("factorial requires non-negative integer".into()));
                }
                let n = v as u64;
                if n > 170 {
                    return Err(CalcError::Overflow);
                }
                let mut result: f64 = 1.0;
                for i in 2..=n {
                    result *= i as f64;
                }
                Ok(result)
            }
            _ => Err(CalcError::InvalidExpression(format!("unknown function: {name}"))),
        }
    }

    fn format_display(value: f64) -> String {
        if value.fract() == 0.0 && value.abs() < 1e15 {
            format!("{}", value as i64)
        } else {
            let s = format!("{:.10}", value);
            let s = s.trim_end_matches('0');
            let s = s.trim_end_matches('.');
            s.to_string()
        }
    }

    fn compute_alt_bases(value: f64) -> Option<AltBases> {
        if value.fract() == 0.0 && value >= 0.0 && value <= u64::MAX as f64 {
            let n = value as u64;
            Some(AltBases {
                hex: format!("{:X}", n),
                oct: format!("{:o}", n),
                bin: format!("{:b}", n),
            })
        } else {
            None
        }
    }
}

impl Evaluate for EngineeringEngine {
    fn evaluate(&self, expr: &str) -> Result<CalcResult, CalcError> {
        let mut parser = Parser::new();
        let value = parser.parse_with_functions(expr, |name, args| {
            self.eval_function(name, args)
        })?;

        if value.is_infinite() {
            return Err(CalcError::Overflow);
        }
        if value.is_nan() {
            return Err(CalcError::DomainError("result is not a number".into()));
        }

        Ok(CalcResult {
            value,
            display: Self::format_display(value),
            alt_bases: Self::compute_alt_bases(value),
        })
    }
}
```

Add to `src/engine/mod.rs`:
```rust
pub mod engineering;
```

- [ ] **Step 4: Run tests**

Run: `cargo test --test engineering_engine_tests 2>&1`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add src/engine/engineering.rs src/engine/mod.rs tests/engineering_engine_tests.rs
git commit -m "feat: implement engineering evaluation engine with trig, log, bitwise"
```

---

## Task 7: Engineering Mode UI

**Files:**
- Create: `src/ui/engineering.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/app.rs`

- [ ] **Step 1: Add engineering-specific messages to app.rs**

Add to the `Message` enum:

```rust
pub enum Message {
    // ... existing ...
    EngFunction(String),          // insert function name like "sin("
    AngleModeChanged(AngleMode),
    BitwiseOp(BitwiseOp),
    BaseDisplay(BaseDisplay),
}

#[derive(Debug, Clone, Copy)]
pub enum BitwiseOp { And, Or, Xor, Not, Shl, Shr }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseDisplay { Hex, Dec, Oct, Bin }
```

- [ ] **Step 2: Create src/ui/engineering.rs**

```rust
use cosmic::prelude::*;
use cosmic::widget;
use cosmic::iced::Length;
use cosmic::theme;

use crate::app::{Message, Operator};
use crate::app::config::AngleMode;

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

fn func_button<'a>(label: &str, func_name: &str) -> Element<'a, Message> {
    widget::button::custom(
        widget::container(widget::text(label.to_string()).size(13.0))
            .center(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .class(theme::Button::Standard)
    .width(Length::Fill)
    .height(Length::Fill)
    .on_press(Message::EngFunction(format!("{}(", func_name)))
    .into()
}

fn num_btn<'a>(n: u8) -> Element<'a, Message> {
    calc_button(&n.to_string(), Message::Number(n), theme::Button::Standard)
}

fn op_btn<'a>(op: Operator) -> Element<'a, Message> {
    calc_button(op.display(), Message::Operator(op), theme::Button::Standard)
}

pub fn view<'a>(angle_mode: AngleMode) -> Element<'a, Message> {
    let spacing = cosmic::theme::active().cosmic().spacing;
    let std = theme::Button::Standard;
    let accent = theme::Button::Suggested;

    // Angle mode segmented button
    let angle_bar = widget::row::with_capacity(3)
        .push(calc_button("DEG",
            Message::AngleModeChanged(AngleMode::Deg),
            if angle_mode == AngleMode::Deg { accent } else { std }))
        .push(calc_button("RAD",
            Message::AngleModeChanged(AngleMode::Rad),
            if angle_mode == AngleMode::Rad { accent } else { std }))
        .push(calc_button("GRAD",
            Message::AngleModeChanged(AngleMode::Grad),
            if angle_mode == AngleMode::Grad { accent } else { std }))
        .spacing(spacing.space_xxs)
        .height(36);

    // Trig row
    let trig_row = widget::row::with_capacity(6)
        .push(func_button("sin", "sin"))
        .push(func_button("cos", "cos"))
        .push(func_button("tan", "tan"))
        .push(func_button("sin⁻¹", "asin"))
        .push(func_button("cos⁻¹", "acos"))
        .push(func_button("tan⁻¹", "atan"))
        .spacing(spacing.space_xxs)
        .height(Length::Fill);

    // Math functions row
    let math_row = widget::row::with_capacity(6)
        .push(func_button("log", "log"))
        .push(func_button("ln", "ln"))
        .push(calc_button("x²", Message::EngFunction("^2".into()), std))
        .push(func_button("√x", "sqrt"))
        .push(calc_button("xʸ", Message::Operator(Operator::Add), std)) // Will use ^ via expression
        .push(func_button("n!", "fact"))
        .spacing(spacing.space_xxs)
        .height(Length::Fill);

    // Note: xʸ should insert ^, not Add. Fix:
    let math_row = widget::row::with_capacity(6)
        .push(func_button("log", "log"))
        .push(func_button("ln", "ln"))
        .push(calc_button("x²", Message::EngFunction("^2".into()), std))
        .push(func_button("√x", "sqrt"))
        .push(calc_button("xʸ", Message::EngFunction("^".into()), std))
        .push(func_button("n!", "fact"))
        .spacing(spacing.space_xxs)
        .height(Length::Fill);

    // Bitwise row
    let bitwise_row = widget::row::with_capacity(6)
        .push(calc_button("AND", Message::BitwiseOp(crate::app::BitwiseOp::And), std))
        .push(calc_button("OR", Message::BitwiseOp(crate::app::BitwiseOp::Or), std))
        .push(calc_button("XOR", Message::BitwiseOp(crate::app::BitwiseOp::Xor), std))
        .push(calc_button("NOT", Message::BitwiseOp(crate::app::BitwiseOp::Not), std))
        .push(calc_button("≪", Message::BitwiseOp(crate::app::BitwiseOp::Shl), std))
        .push(calc_button("≫", Message::BitwiseOp(crate::app::BitwiseOp::Shr), std))
        .spacing(spacing.space_xxs)
        .height(Length::Fill);

    // Base / constants row
    let base_row = widget::row::with_capacity(6)
        .push(calc_button("HEX", Message::BaseDisplay(crate::app::BaseDisplay::Hex), std))
        .push(calc_button("DEC", Message::BaseDisplay(crate::app::BaseDisplay::Dec), std))
        .push(calc_button("OCT", Message::BaseDisplay(crate::app::BaseDisplay::Oct), std))
        .push(calc_button("BIN", Message::BaseDisplay(crate::app::BaseDisplay::Bin), std))
        .push(calc_button("π", Message::EngFunction("pi".into()), std))
        .push(calc_button("e", Message::EngFunction("e".into()), std))
        .spacing(spacing.space_xxs)
        .height(Length::Fill);

    // Standard numpad
    let numpad = widget::column::with_capacity(6)
        .push(widget::row::with_capacity(4)
            .push(calc_button("C", Message::Clear, std))
            .push(calc_button("(", Message::ParenOpen, std))
            .push(calc_button(")", Message::ParenClose, std))
            .push(op_btn(Operator::Divide))
            .spacing(spacing.space_xxs)
            .height(Length::Fill))
        .push(widget::row::with_capacity(4)
            .push(num_btn(7)).push(num_btn(8)).push(num_btn(9))
            .push(op_btn(Operator::Multiply))
            .spacing(spacing.space_xxs).height(Length::Fill))
        .push(widget::row::with_capacity(4)
            .push(num_btn(4)).push(num_btn(5)).push(num_btn(6))
            .push(op_btn(Operator::Subtract))
            .spacing(spacing.space_xxs).height(Length::Fill))
        .push(widget::row::with_capacity(4)
            .push(num_btn(1)).push(num_btn(2)).push(num_btn(3))
            .push(op_btn(Operator::Add))
            .spacing(spacing.space_xxs).height(Length::Fill))
        .push(widget::row::with_capacity(4)
            .push(num_btn(0))
            .push(calc_button(".", Message::Decimal, std))
            .push(calc_button("⌫", Message::Backspace, std))
            .push(calc_button("=", Message::Evaluate, accent))
            .spacing(spacing.space_xxs).height(Length::Fill))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(Length::Fill);

    widget::column::with_capacity(6)
        .push(angle_bar)
        .push(trig_row)
        .push(math_row)
        .push(bitwise_row)
        .push(base_row)
        .push(numpad)
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
```

- [ ] **Step 3: Update src/ui/mod.rs**

```rust
pub mod standard;
pub mod engineering;
```

- [ ] **Step 4: Wire up in app.rs view() and update()**

In `view()`, change the engineering branch:

```rust
Mode::Engineering => ui::engineering::view(self.config.angle_mode),
```

In `update()`, add handlers:

```rust
Message::EngFunction(func) => {
    self.expression.push_str(&func);
}
Message::AngleModeChanged(mode) => {
    self.config.angle_mode = mode;
    if let Some(config_handler) = &self.config_handler {
        let _ = self.config.set_angle_mode(config_handler, mode);
    }
}
Message::BitwiseOp(op) => {
    let symbol = match op {
        BitwiseOp::And => " AND ",
        BitwiseOp::Or => " OR ",
        BitwiseOp::Xor => " XOR ",
        BitwiseOp::Not => "NOT ",
        BitwiseOp::Shl => "<<",
        BitwiseOp::Shr => ">>",
    };
    self.expression.push_str(symbol);
}
Message::BaseDisplay(_base) => {
    // Will be used to toggle which base is shown in display
    // For now, alt_bases are computed by the engine automatically
}
```

In `update()` `Message::Evaluate`, route to the correct engine based on mode:

```rust
Message::Evaluate => {
    let result = match self.mode {
        Mode::Standard => {
            crate::engine::standard::StandardEngine.evaluate(&self.expression)
        }
        Mode::Engineering => {
            crate::engine::engineering::EngineeringEngine::new(self.config.angle_mode)
                .evaluate(&self.expression)
        }
        Mode::Financial => {
            crate::engine::standard::StandardEngine.evaluate(&self.expression)
        }
    };
    match result {
        Ok(result) => {
            let entry = config::HistoryEntry {
                expression: self.expression.clone(),
                result: result.display.clone(),
                mode: self.mode,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
            };
            self.history.push(entry);
            if let Some(config_handler) = &self.config_handler {
                let _ = self.config.set_history(config_handler, self.history.clone());
            }
            self.display = result.display;
            self.expression = result.value.to_string();
        }
        Err(e) => {
            return self.update(Message::ShowToast(e.to_string()));
        }
    }
}
```

- [ ] **Step 5: Update window size for engineering mode**

In `src/app/settings.rs`, increase the default size to accommodate the wider grid:

```rust
settings = settings.size_limits(Limits::NONE.min_width(380.0).min_height(580.0));
settings = settings.size(Size::new(420.0, 650.0));
```

- [ ] **Step 6: Build and verify**

Run: `cargo build 2>&1`
Expected: Compiles successfully

- [ ] **Step 7: Commit**

```bash
git add src/ui/engineering.rs src/ui/mod.rs src/app.rs src/app/settings.rs
git commit -m "feat: implement engineering mode UI with trig, log, bitwise, base display"
```

---

## Task 8: Financial Engine (TVM Solver)

**Files:**
- Create: `src/engine/financial.rs`
- Create: `tests/financial_engine_tests.rs`
- Modify: `src/engine/mod.rs`

- [ ] **Step 1: Write tests**

Create `tests/financial_engine_tests.rs`:

```rust
use cosmic_ext_calculator::engine::financial::{FinancialEngine, TvmParams, TvmSolveFor};

fn approx_eq(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() < tolerance
}

#[test]
fn test_solve_fv() {
    // $1000 at 5% for 10 years, no payments
    let engine = FinancialEngine;
    let params = TvmParams {
        n: Some(10.0),
        rate: Some(5.0),
        pv: Some(-1000.0),
        pmt: Some(0.0),
        fv: None,
    };
    let result = engine.solve_tvm(params, TvmSolveFor::Fv).unwrap();
    assert!(approx_eq(result, 1628.89, 0.01));
}

#[test]
fn test_solve_pv() {
    // What PV gives FV of $10000 at 6% for 5 years, no payments?
    let engine = FinancialEngine;
    let params = TvmParams {
        n: Some(5.0),
        rate: Some(6.0),
        pv: None,
        pmt: Some(0.0),
        fv: Some(10000.0),
    };
    let result = engine.solve_tvm(params, TvmSolveFor::Pv).unwrap();
    assert!(approx_eq(result, -7472.58, 0.01));
}

#[test]
fn test_solve_pmt() {
    // 30-year mortgage, $200000, 4% annual (monthly: 360 periods, 0.333% rate)
    let engine = FinancialEngine;
    let params = TvmParams {
        n: Some(360.0),
        rate: Some(4.0 / 12.0),
        pv: Some(200000.0),
        pmt: None,
        fv: Some(0.0),
    };
    let result = engine.solve_tvm(params, TvmSolveFor::Pmt).unwrap();
    assert!(approx_eq(result, -954.83, 0.01));
}

#[test]
fn test_solve_n() {
    // How many periods to grow $1000 to $2000 at 7%, no payments?
    let engine = FinancialEngine;
    let params = TvmParams {
        n: None,
        rate: Some(7.0),
        pv: Some(-1000.0),
        pmt: Some(0.0),
        fv: Some(2000.0),
    };
    let result = engine.solve_tvm(params, TvmSolveFor::N).unwrap();
    assert!(approx_eq(result, 10.245, 0.01));
}

#[test]
fn test_solve_rate() {
    // What rate doubles $1000 in 10 years, no payments?
    let engine = FinancialEngine;
    let params = TvmParams {
        n: Some(10.0),
        rate: None,
        pv: Some(-1000.0),
        pmt: Some(0.0),
        fv: Some(2000.0),
    };
    let result = engine.solve_tvm(params, TvmSolveFor::Rate).unwrap();
    assert!(approx_eq(result, 7.177, 0.01));
}

#[test]
fn test_margin() {
    let engine = FinancialEngine;
    // Cost $80, sell $100 => margin = 20%
    assert!(approx_eq(engine.margin(80.0, 100.0), 20.0, 0.01));
}

#[test]
fn test_markup() {
    let engine = FinancialEngine;
    // Cost $80, sell $100 => markup = 25%
    assert!(approx_eq(engine.markup(80.0, 100.0), 25.0, 0.01));
}

#[test]
fn test_tax_add() {
    let engine = FinancialEngine;
    assert!(approx_eq(engine.add_tax(100.0, 21.0), 121.0, 0.01));
}

#[test]
fn test_tax_remove() {
    let engine = FinancialEngine;
    assert!(approx_eq(engine.remove_tax(121.0, 21.0), 100.0, 0.01));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test financial_engine_tests 2>&1`
Expected: FAIL — module doesn't exist

- [ ] **Step 3: Implement FinancialEngine**

Create `src/engine/financial.rs`:

```rust
use super::CalcError;

pub struct FinancialEngine;

#[derive(Debug, Clone)]
pub struct TvmParams {
    pub n: Option<f64>,
    pub rate: Option<f64>,  // per-period rate as percentage (e.g., 5.0 = 5%)
    pub pv: Option<f64>,
    pub pmt: Option<f64>,
    pub fv: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
pub enum TvmSolveFor {
    N,
    Rate,
    Pv,
    Pmt,
    Fv,
}

impl FinancialEngine {
    /// TVM equation: FV + PV*(1+i)^n + PMT*((1+i)^n - 1)/i = 0
    /// When i=0: FV + PV + PMT*n = 0
    pub fn solve_tvm(&self, params: TvmParams, solve_for: TvmSolveFor) -> Result<f64, CalcError> {
        match solve_for {
            TvmSolveFor::Fv => {
                let n = params.n.ok_or(CalcError::InvalidExpression("N required".into()))?;
                let rate = params.rate.ok_or(CalcError::InvalidExpression("rate required".into()))? / 100.0;
                let pv = params.pv.ok_or(CalcError::InvalidExpression("PV required".into()))?;
                let pmt = params.pmt.ok_or(CalcError::InvalidExpression("PMT required".into()))?;

                if rate == 0.0 {
                    Ok(-(pv + pmt * n))
                } else {
                    let factor = (1.0 + rate).powf(n);
                    Ok(-(pv * factor + pmt * (factor - 1.0) / rate))
                }
            }
            TvmSolveFor::Pv => {
                let n = params.n.ok_or(CalcError::InvalidExpression("N required".into()))?;
                let rate = params.rate.ok_or(CalcError::InvalidExpression("rate required".into()))? / 100.0;
                let pmt = params.pmt.ok_or(CalcError::InvalidExpression("PMT required".into()))?;
                let fv = params.fv.ok_or(CalcError::InvalidExpression("FV required".into()))?;

                if rate == 0.0 {
                    Ok(-(fv + pmt * n))
                } else {
                    let factor = (1.0 + rate).powf(n);
                    Ok(-(fv + pmt * (factor - 1.0) / rate) / factor)
                }
            }
            TvmSolveFor::Pmt => {
                let n = params.n.ok_or(CalcError::InvalidExpression("N required".into()))?;
                let rate = params.rate.ok_or(CalcError::InvalidExpression("rate required".into()))? / 100.0;
                let pv = params.pv.ok_or(CalcError::InvalidExpression("PV required".into()))?;
                let fv = params.fv.ok_or(CalcError::InvalidExpression("FV required".into()))?;

                if rate == 0.0 {
                    Ok(-(pv + fv) / n)
                } else {
                    let factor = (1.0 + rate).powf(n);
                    Ok(-(pv * factor + fv) / ((factor - 1.0) / rate))
                }
            }
            TvmSolveFor::N => {
                let rate = params.rate.ok_or(CalcError::InvalidExpression("rate required".into()))? / 100.0;
                let pv = params.pv.ok_or(CalcError::InvalidExpression("PV required".into()))?;
                let pmt = params.pmt.ok_or(CalcError::InvalidExpression("PMT required".into()))?;
                let fv = params.fv.ok_or(CalcError::InvalidExpression("FV required".into()))?;

                if rate == 0.0 {
                    if pmt == 0.0 {
                        return Err(CalcError::DivisionByZero);
                    }
                    Ok(-(pv + fv) / pmt)
                } else {
                    let numerator = (-fv * rate + pmt).ln() - (pv * rate + pmt).ln();
                    let denominator = (1.0 + rate).ln();
                    if denominator == 0.0 || numerator.is_nan() {
                        return Err(CalcError::DomainError("cannot solve for N with given parameters".into()));
                    }
                    Ok(numerator / denominator)
                }
            }
            TvmSolveFor::Rate => {
                // Newton-Raphson iteration
                let n = params.n.ok_or(CalcError::InvalidExpression("N required".into()))?;
                let pv = params.pv.ok_or(CalcError::InvalidExpression("PV required".into()))?;
                let pmt = params.pmt.ok_or(CalcError::InvalidExpression("PMT required".into()))?;
                let fv = params.fv.ok_or(CalcError::InvalidExpression("FV required".into()))?;

                let mut rate = 0.1; // initial guess: 10%
                let max_iter = 1000;
                let tolerance = 1e-10;

                for _ in 0..max_iter {
                    let factor = (1.0 + rate).powf(n);
                    let f = fv + pv * factor + pmt * (factor - 1.0) / rate;

                    // Derivative
                    let dfactor = n * (1.0 + rate).powf(n - 1.0);
                    let df = pv * dfactor + pmt * (dfactor * rate - (factor - 1.0)) / (rate * rate);

                    if df.abs() < 1e-20 {
                        return Err(CalcError::ConvergenceError);
                    }

                    let new_rate = rate - f / df;

                    if (new_rate - rate).abs() < tolerance {
                        return Ok(new_rate * 100.0); // return as percentage
                    }
                    rate = new_rate;

                    if rate.is_nan() || rate.is_infinite() {
                        return Err(CalcError::ConvergenceError);
                    }
                }

                Err(CalcError::ConvergenceError)
            }
        }
    }

    /// Margin = (price - cost) / price * 100
    pub fn margin(&self, cost: f64, price: f64) -> f64 {
        (price - cost) / price * 100.0
    }

    /// Markup = (price - cost) / cost * 100
    pub fn markup(&self, cost: f64, price: f64) -> f64 {
        (price - cost) / cost * 100.0
    }

    /// Add tax: net * (1 + rate/100)
    pub fn add_tax(&self, net: f64, tax_rate: f64) -> f64 {
        net * (1.0 + tax_rate / 100.0)
    }

    /// Remove tax: gross / (1 + rate/100)
    pub fn remove_tax(&self, gross: f64, tax_rate: f64) -> f64 {
        gross / (1.0 + tax_rate / 100.0)
    }
}
```

Add to `src/engine/mod.rs`:
```rust
pub mod financial;
```

- [ ] **Step 4: Run tests**

Run: `cargo test --test financial_engine_tests 2>&1`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add src/engine/financial.rs src/engine/mod.rs tests/financial_engine_tests.rs
git commit -m "feat: implement financial engine with TVM solver, margin, markup, tax"
```

---

## Task 9: Financial Mode UI

**Files:**
- Create: `src/ui/financial.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/app.rs`

- [ ] **Step 1: Add financial-specific messages**

Add to `Message` enum in `src/app.rs`:

```rust
pub enum Message {
    // ... existing ...
    TvmFieldChanged(TvmField, String),
    TvmSolve(TvmField),
    QuickFinancial(QuickFinancial),
    ToggleSign,
}

#[derive(Debug, Clone, Copy)]
pub enum TvmField { N, Rate, Pv, Pmt, Fv }

#[derive(Debug, Clone, Copy)]
pub enum QuickFinancial { Margin, Markup, TaxAdd, TaxRemove }
```

Add TVM state to `CosmicCalculator`:

```rust
pub struct CosmicCalculator {
    // ... existing ...
    tvm_n: String,
    tvm_rate: String,
    tvm_pv: String,
    tvm_pmt: String,
    tvm_fv: String,
}
```

Initialize them as empty strings in `init()`.

- [ ] **Step 2: Create src/ui/financial.rs**

```rust
use cosmic::prelude::*;
use cosmic::widget;
use cosmic::iced::Length;
use cosmic::theme;

use crate::app::{Message, Operator, TvmField, QuickFinancial};

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

fn num_btn<'a>(n: u8) -> Element<'a, Message> {
    calc_button(&n.to_string(), Message::Number(n), theme::Button::Standard)
}

fn op_btn<'a>(op: Operator) -> Element<'a, Message> {
    calc_button(op.display(), Message::Operator(op), theme::Button::Standard)
}

fn tvm_row<'a>(label: &str, value: &str, field: TvmField) -> Element<'a, Message> {
    let spacing = cosmic::theme::active().cosmic().spacing;

    widget::row::with_capacity(3)
        .push(
            widget::text(label.to_string())
                .size(14.0)
                .width(50),
        )
        .push(
            widget::text_input("", value)
                .on_input(move |s| Message::TvmFieldChanged(field, s))
                .size(14.0)
                .width(Length::Fill),
        )
        .push(
            widget::button::custom(
                widget::container(widget::text("Solve").size(12.0))
                    .center(Length::Fill),
            )
            .class(theme::Button::Suggested)
            .width(60)
            .on_press(Message::TvmSolve(field)),
        )
        .spacing(spacing.space_xxs)
        .align_y(cosmic::iced::Alignment::Center)
        .into()
}

pub fn view<'a>(
    tvm_n: &str,
    tvm_rate: &str,
    tvm_pv: &str,
    tvm_pmt: &str,
    tvm_fv: &str,
) -> Element<'a, Message> {
    let spacing = cosmic::theme::active().cosmic().spacing;
    let std = theme::Button::Standard;
    let accent = theme::Button::Suggested;

    let tvm_section = widget::column::with_capacity(5)
        .push(tvm_row("N", tvm_n, TvmField::N))
        .push(tvm_row("I/Y %", tvm_rate, TvmField::Rate))
        .push(tvm_row("PV", tvm_pv, TvmField::Pv))
        .push(tvm_row("PMT", tvm_pmt, TvmField::Pmt))
        .push(tvm_row("FV", tvm_fv, TvmField::Fv))
        .spacing(spacing.space_xxs);

    let quick_row = widget::row::with_capacity(4)
        .push(calc_button("Margin", Message::QuickFinancial(QuickFinancial::Margin), std))
        .push(calc_button("Markup", Message::QuickFinancial(QuickFinancial::Markup), std))
        .push(calc_button("Tax+", Message::QuickFinancial(QuickFinancial::TaxAdd), std))
        .push(calc_button("Tax−", Message::QuickFinancial(QuickFinancial::TaxRemove), std))
        .spacing(spacing.space_xxs)
        .height(Length::Fill);

    let numpad = widget::column::with_capacity(5)
        .push(widget::row::with_capacity(4)
            .push(calc_button("C", Message::Clear, std))
            .push(calc_button("±", Message::ToggleSign, std))
            .push(calc_button("%", Message::Percent, std))
            .push(op_btn(Operator::Divide))
            .spacing(spacing.space_xxs).height(Length::Fill))
        .push(widget::row::with_capacity(4)
            .push(num_btn(7)).push(num_btn(8)).push(num_btn(9))
            .push(op_btn(Operator::Multiply))
            .spacing(spacing.space_xxs).height(Length::Fill))
        .push(widget::row::with_capacity(4)
            .push(num_btn(4)).push(num_btn(5)).push(num_btn(6))
            .push(op_btn(Operator::Subtract))
            .spacing(spacing.space_xxs).height(Length::Fill))
        .push(widget::row::with_capacity(4)
            .push(num_btn(1)).push(num_btn(2)).push(num_btn(3))
            .push(op_btn(Operator::Add))
            .spacing(spacing.space_xxs).height(Length::Fill))
        .push(widget::row::with_capacity(4)
            .push(num_btn(0))
            .push(calc_button(".", Message::Decimal, std))
            .push(calc_button("⌫", Message::Backspace, std))
            .push(calc_button("=", Message::Evaluate, accent))
            .spacing(spacing.space_xxs).height(Length::Fill))
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .height(Length::Fill);

    widget::column::with_capacity(3)
        .push(tvm_section)
        .push(quick_row)
        .push(numpad)
        .spacing(spacing.space_xs)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
```

- [ ] **Step 3: Update src/ui/mod.rs**

```rust
pub mod standard;
pub mod engineering;
pub mod financial;
```

- [ ] **Step 4: Wire up financial messages in app.rs**

In `view()`:

```rust
Mode::Financial => ui::financial::view(
    &self.tvm_n, &self.tvm_rate, &self.tvm_pv, &self.tvm_pmt, &self.tvm_fv,
),
```

In `update()`:

```rust
Message::TvmFieldChanged(field, value) => {
    match field {
        TvmField::N => self.tvm_n = value,
        TvmField::Rate => self.tvm_rate = value,
        TvmField::Pv => self.tvm_pv = value,
        TvmField::Pmt => self.tvm_pmt = value,
        TvmField::Fv => self.tvm_fv = value,
    }
}
Message::TvmSolve(field) => {
    use crate::engine::financial::{FinancialEngine, TvmParams, TvmSolveFor};

    let parse = |s: &str| -> Option<f64> {
        if s.is_empty() { None } else { s.parse().ok() }
    };

    let params = TvmParams {
        n: if matches!(field, TvmField::N) { None } else { parse(&self.tvm_n) },
        rate: if matches!(field, TvmField::Rate) { None } else { parse(&self.tvm_rate) },
        pv: if matches!(field, TvmField::Pv) { None } else { parse(&self.tvm_pv) },
        pmt: if matches!(field, TvmField::Pmt) { None } else { parse(&self.tvm_pmt) },
        fv: if matches!(field, TvmField::Fv) { None } else { parse(&self.tvm_fv) },
    };

    let solve_for = match field {
        TvmField::N => TvmSolveFor::N,
        TvmField::Rate => TvmSolveFor::Rate,
        TvmField::Pv => TvmSolveFor::Pv,
        TvmField::Pmt => TvmSolveFor::Pmt,
        TvmField::Fv => TvmSolveFor::Fv,
    };

    let engine = FinancialEngine;
    match engine.solve_tvm(params, solve_for) {
        Ok(result) => {
            let formatted = format!("{:.2}", result);
            match field {
                TvmField::N => self.tvm_n = formatted,
                TvmField::Rate => self.tvm_rate = formatted,
                TvmField::Pv => self.tvm_pv = formatted,
                TvmField::Pmt => self.tvm_pmt = formatted,
                TvmField::Fv => self.tvm_fv = formatted,
            }
            self.display = format!("{:.2}", result);
        }
        Err(e) => {
            return self.update(Message::ShowToast(e.to_string()));
        }
    }
}
Message::QuickFinancial(func) => {
    use crate::engine::financial::FinancialEngine;
    let engine = FinancialEngine;
    // Use the current expression as the input value
    if let Ok(value) = self.expression.parse::<f64>() {
        let result = match func {
            QuickFinancial::TaxAdd => engine.add_tax(value, self.config.tax_rate),
            QuickFinancial::TaxRemove => engine.remove_tax(value, self.config.tax_rate),
            _ => {
                return self.update(Message::ShowToast(
                    "Enter cost and price separated by comma".into(),
                ));
            }
        };
        self.display = format!("{:.2}", result);
        self.expression = result.to_string();
    } else {
        return self.update(Message::ShowToast("Enter a valid number first".into()));
    }
}
Message::ToggleSign => {
    if self.expression.starts_with('-') {
        self.expression = self.expression[1..].to_string();
    } else if !self.expression.is_empty() {
        self.expression = format!("-{}", self.expression);
    }
}
```

- [ ] **Step 5: Build and verify**

Run: `cargo build 2>&1`
Expected: Compiles successfully

- [ ] **Step 6: Commit**

```bash
git add src/ui/financial.rs src/ui/mod.rs src/app.rs
git commit -m "feat: implement financial mode UI with TVM solver and quick functions"
```

---

## Task 10: History Panel

**Files:**
- Create: `src/ui/history.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/app.rs`

- [ ] **Step 1: Add history messages**

Add to `Message` enum in `src/app.rs`:

```rust
pub enum Message {
    // ... existing ...
    HistorySelect(usize),
    HistoryDelete(usize),
}
```

- [ ] **Step 2: Create src/ui/history.rs**

```rust
use cosmic::prelude::*;
use cosmic::widget;
use cosmic::iced::Length;
use cosmic::theme;

use crate::app::Message;
use crate::app::config::HistoryEntry;

pub fn view<'a>(history: &[HistoryEntry]) -> Element<'a, Message> {
    let spacing = cosmic::theme::active().cosmic().spacing;

    if history.is_empty() {
        return widget::container(
            widget::text("No history yet")
                .size(14.0)
                .width(Length::Fill)
                .align_x(cosmic::iced::Alignment::Center),
        )
        .center(Length::Fill)
        .width(Length::Fill)
        .into();
    }

    let mut list = widget::column::with_capacity(history.len());

    for (i, entry) in history.iter().enumerate().rev() {
        let mode_color = match entry.mode {
            crate::app::config::Mode::Standard => cosmic::iced::Color::from_rgb(0.3, 0.6, 1.0),
            crate::app::config::Mode::Engineering => cosmic::iced::Color::from_rgb(0.3, 0.8, 0.5),
            crate::app::config::Mode::Financial => cosmic::iced::Color::from_rgb(1.0, 0.7, 0.3),
        };

        let row = widget::button::custom(
            widget::column::with_capacity(2)
                .push(
                    widget::row::with_capacity(2)
                        .push(
                            widget::text(entry.mode.label().to_string())
                                .size(10.0)
                                .color(mode_color),
                        )
                        .push(
                            widget::text(&entry.expression)
                                .size(12.0)
                                .width(Length::Fill)
                                .align_x(cosmic::iced::Alignment::End),
                        )
                        .spacing(spacing.space_xxs),
                )
                .push(
                    widget::text(format!("= {}", &entry.result))
                        .size(16.0)
                        .width(Length::Fill)
                        .align_x(cosmic::iced::Alignment::End),
                )
                .padding(spacing.space_xxs),
        )
        .class(theme::Button::Standard)
        .width(Length::Fill)
        .on_press(Message::HistorySelect(i));

        list = list.push(row);
    }

    widget::scrollable(list.spacing(spacing.space_xxs).width(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
```

- [ ] **Step 3: Update src/ui/mod.rs**

```rust
pub mod standard;
pub mod engineering;
pub mod financial;
pub mod history;
```

- [ ] **Step 4: Wire up in app.rs**

In `context_drawer()`:

```rust
ContextPage::History => {
    let content = ui::history::view(&self.history);
    context_drawer::context_drawer(content, Message::ToggleContextDrawer)
}
```

In `update()`:

```rust
Message::HistorySelect(index) => {
    if let Some(entry) = self.history.get(index) {
        self.expression = entry.result.clone();
        self.display = entry.result.clone();
    }
}
Message::HistoryDelete(index) => {
    if index < self.history.len() {
        self.history.remove(index);
        if let Some(config_handler) = &self.config_handler {
            let _ = self.config.set_history(config_handler, self.history.clone());
        }
    }
}
```

- [ ] **Step 5: Build and verify**

Run: `cargo build 2>&1`
Expected: Compiles successfully

- [ ] **Step 6: Commit**

```bash
git add src/ui/history.rs src/ui/mod.rs src/app.rs
git commit -m "feat: implement unified history panel with mode badges"
```

---

## Task 11: Desktop Integration

**Files:**
- Create: `res/dev.dcristob.Calculator.desktop`
- Create: `res/dev.dcristob.Calculator.metainfo.xml`

- [ ] **Step 1: Create .desktop file**

```ini
[Desktop Entry]
Name=Calculator
Comment=A multi-mode calculator for the COSMIC desktop
Exec=cosmic-ext-calculator
Icon=dev.dcristob.Calculator
Terminal=false
Type=Application
Categories=Utility;Calculator;
Keywords=calculator;math;finance;engineering;
```

- [ ] **Step 2: Create metainfo.xml**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>dev.dcristob.Calculator</id>
  <name>Calculator</name>
  <summary>A multi-mode calculator for the COSMIC desktop</summary>
  <metadata_license>CC0-1.0</metadata_license>
  <project_license>GPL-3.0-only</project_license>
  <description>
    <p>
      A calculator application for the COSMIC desktop with three modes:
      Standard, Engineering, and Financial. Features a Rust-native expression
      parser, HP-48S-inspired engineering layout, TVM solver for financial
      calculations, full keyboard support, and unified calculation history.
    </p>
  </description>
  <categories>
    <category>Utility</category>
    <category>Calculator</category>
  </categories>
  <url type="homepage">https://github.com/dcristob/cosmic-calculator</url>
  <url type="bugtracker">https://github.com/dcristob/cosmic-calculator/issues</url>
  <content_rating type="oars-1.1" />
</component>
```

- [ ] **Step 3: Commit**

```bash
git add res/
git commit -m "feat: add desktop entry and metainfo for COSMIC integration"
```

---

## Task 12: Final Build & Smoke Test

**Files:**
- No new files

- [ ] **Step 1: Full build**

Run: `cargo build --release 2>&1`
Expected: Compiles successfully with no errors

- [ ] **Step 2: Run all tests**

Run: `cargo test 2>&1`
Expected: All tests pass (parser, standard engine, engineering engine, financial engine)

- [ ] **Step 3: Run clippy**

Run: `cargo clippy 2>&1`
Expected: No errors (warnings are acceptable for now)

- [ ] **Step 4: Fix any issues found**

Address any compilation errors, test failures, or clippy errors.

- [ ] **Step 5: Final commit if fixes were needed**

```bash
git add -A
git commit -m "fix: address clippy warnings and build issues"
```
