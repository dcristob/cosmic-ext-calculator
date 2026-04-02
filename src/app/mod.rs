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
use cosmic::widget::{self, toaster::ToastId};
use cosmic::widget::wrapper::RcElementWrapper;
use cosmic::{Application, Core};
use cosmic::widget::menu::action::MenuAction as MenuActionTrait;

use crate::core::icons;
use crate::core::keybinds::key_binds;
use crate::fl;
use crate::ui;

type Task = cosmic::app::Task<Message>;

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
            MenuAction::SwitchStandard
            | MenuAction::SwitchEngineering
            | MenuAction::SwitchFinancial
            | MenuAction::Undo => Message::ToggleContextDrawer,
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

    fn init(core: Core, flags: Self::Flags) -> (Self, Task) {
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
                (
                    fl!("support"),
                    "https://github.com/dcristob/cosmic-calculator/issues",
                ),
                (
                    fl!("repository"),
                    "https://github.com/dcristob/cosmic-calculator",
                ),
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

        let task = app.set_window_title(fl!("app-title"));

        (app, task)
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            RcElementWrapper::new(Element::from(menu::root(fl!("view")))),
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

    fn view(&self) -> Element<'_, Self::Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;

        let tab_bar = widget::segmented_button::horizontal(&self.mode_model)
            .on_activate(Message::ModeSelected)
            .width(cosmic::iced::Length::Fill);

        let display = widget::column::with_capacity(2)
            .push(
                widget::text::body(&self.expression)
                    .width(cosmic::iced::Length::Fill)
                    .align_x(cosmic::iced::Alignment::End),
            )
            .push(
                widget::text::title1(&self.display)
                    .width(cosmic::iced::Length::Fill)
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
                    .push(widget::toaster(&self.toasts, widget::Space::new().width(cosmic::iced::Length::Fill))),
            )
            .width(cosmic::iced::Length::Fill)
            .height(cosmic::iced::Length::Fill)
            .spacing(spacing.space_xs)
            .padding(spacing.space_xxs)
            .into()
    }

    fn update(&mut self, message: Self::Message) -> Task {
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
        cosmic::iced::Task::batch(tasks)
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
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
                Event::Keyboard(KeyEvent::KeyPressed {
                    key, modifiers, ..
                }) => match status {
                    event::Status::Ignored => Some(Message::Key(modifiers, key)),
                    event::Status::Captured => None,
                },
                Event::Keyboard(KeyEvent::ModifiersChanged(modifiers)) => {
                    Some(Message::Modifiers(modifiers))
                }
                Event::Window(cosmic::iced::window::Event::Focused) => Some(Message::Window),
                _ => None,
            }),
            cosmic::cosmic_config::config_subscription::<_, CalculatorConfig>(
                TypeId::of::<()>(),
                Self::APP_ID.into(),
                CONFIG_VERSION,
            )
            .map(|_| Message::SystemThemeModeChange),
        ];

        cosmic::iced::Subscription::batch(subscriptions)
    }
}
