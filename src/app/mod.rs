pub mod config;
pub mod settings;

use std::any::TypeId;
use std::collections::HashMap;

use config::{CalculatorConfig, HistoryEntry, Mode, CONFIG_VERSION};
use cosmic::app::context_drawer;
use cosmic::iced::event;
use cosmic::iced::keyboard::Event as KeyEvent;
use cosmic::iced::keyboard::{Key, Modifiers};
use cosmic::iced::Event;
use cosmic::prelude::*;
use cosmic::widget::about::About;
use cosmic::widget::menu::action::MenuAction as MenuActionTrait;
use cosmic::widget::menu::{self, ItemHeight, ItemWidth};
use cosmic::widget::wrapper::RcElementWrapper;
use cosmic::widget::{self, toaster::ToastId};
use cosmic::{Application, Core};

use crate::core::icons;
use crate::core::keybinds::key_binds;
use crate::engine::CalcResult;
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
    expression: String,
    display: String,
    /// Set after a successful `Evaluate`. While true, the next digit/decimal
    /// starts a fresh expression instead of appending to the carried-over
    /// result, while a following operator continues chaining from that result.
    just_evaluated: bool,
    /// Holds the first value (cost) and chosen function while a two-step
    /// Margin/Markup entry is in progress; `None` when not mid-operation.
    pending_quick: Option<(QuickFinancial, f64)>,
    /// Text buffer backing the Settings tax-rate input, so partial entries
    /// like "8." survive without round-tripping through the stored f64.
    tax_rate_input: String,
    /// Last evaluated result, kept so the HEX/DEC/OCT/BIN buttons can reformat
    /// it (it already bundles the decimal display and the alt-base strings).
    last_result: Option<CalcResult>,
    /// Base the last result is currently shown in (drives button highlight).
    current_base: BaseDisplay,
    tvm_n: String,
    tvm_rate: String,
    tvm_pv: String,
    tvm_pmt: String,
    tvm_fv: String,
    active_tvm_field: TvmField,
    history: Vec<HistoryEntry>,
    toasts: widget::Toasts<Message>,
}

#[derive(Debug, Clone, Copy)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl Operator {
    pub fn expression(&self) -> &'static str {
        match self {
            Operator::Add => "+",
            Operator::Subtract => "-",
            Operator::Multiply => "*",
            Operator::Divide => "/",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BitwiseOp {
    And,
    Or,
    Xor,
    Not,
    Shl,
    Shr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseDisplay {
    Hex,
    Dec,
    Oct,
    Bin,
}

#[derive(Debug, Clone, Copy)]
pub enum TvmField {
    N,
    Rate,
    Pv,
    Pmt,
    Fv,
}

#[derive(Debug, Clone, Copy)]
pub enum QuickFinancial {
    Margin,
    Markup,
    TaxAdd,
    TaxRemove,
}

#[derive(Debug, Clone)]
pub enum Message {
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
    SwitchMode(Mode),
    EngFunction(String),
    AngleModeChanged(config::AngleMode),
    BitwiseOp(BitwiseOp),
    BaseDisplay(BaseDisplay),
    TvmFieldChanged(TvmField, String),
    TvmSolve(TvmField),
    TvmSelectField(TvmField),
    QuickFinancial(QuickFinancial),
    ToggleSign,
    HistorySelect(usize),
    TaxRateChanged(String),
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    History,
    Settings,
}

pub struct Flags {
    pub config_handler: Option<cosmic::cosmic_config::Config>,
    pub config: CalculatorConfig,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    Settings,
    ClearHistory,
    ToggleHistory,
    SwitchStandard,
    SwitchEngineering,
    SwitchFinancial,
    Undo,
    CopyResult,
}

impl MenuActionTrait for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::Settings => Message::ToggleContextPage(ContextPage::Settings),
            MenuAction::ClearHistory => Message::CleanHistory,
            MenuAction::ToggleHistory => Message::ToggleContextPage(ContextPage::History),
            MenuAction::SwitchStandard => Message::SwitchMode(Mode::Standard),
            MenuAction::SwitchEngineering => Message::SwitchMode(Mode::Engineering),
            MenuAction::SwitchFinancial => Message::SwitchMode(Mode::Financial),
            MenuAction::Undo => Message::Undo,
            MenuAction::CopyResult => Message::CopyResult,
        }
    }
}

fn row_spacing_for(mode: Mode) -> u16 {
    match mode {
        Mode::Standard => 32,
        Mode::Engineering => 6,
        Mode::Financial => 4,
    }
}

impl CosmicCalculator {
    /// If the last action was an evaluation, discard the carried-over result so
    /// the incoming operand starts a brand new expression. Called before
    /// appending a digit, decimal, opening paren, or function.
    fn start_fresh_if_evaluated(&mut self) {
        if self.just_evaluated {
            self.expression.clear();
            self.just_evaluated = false;
        }
    }

    /// Apply a prefix function like "cos(" to the current operand, wrapping it
    /// as the argument (`34` -> `cos(34)`) instead of leaving a dangling
    /// `34cos(`. Right after an evaluation the whole result is wrapped,
    /// including a leading sign (`-7` -> `cos(-7)`). With no trailing operand
    /// the function is appended open, ready for input (`2+` -> `2+cos(`).
    fn apply_prefix_function(&mut self, func: &str) {
        if self.just_evaluated {
            self.expression = format!("{}{})", func, self.expression);
            self.just_evaluated = false;
            return;
        }
        let trailing_start = self
            .expression
            .rfind(|c: char| !c.is_ascii_digit() && c != '.')
            .map_or(0, |i| i + 1);
        if trailing_start < self.expression.len() {
            let number = self.expression.split_off(trailing_start);
            self.expression.push_str(func);
            self.expression.push_str(&number);
            self.expression.push(')');
        } else {
            self.expression.push_str(func);
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
        let about = About::default()
            .name(fl!("app-title"))
            .icon(widget::icon::from_name(Self::APP_ID))
            .version("0.1.0")
            .license("GPL-3.0-only")
            .links([
                (
                    fl!("repository"),
                    "https://github.com/dcristob/cosmic-ext-calculator",
                ),
                (
                    fl!("support"),
                    "https://github.com/dcristob/cosmic-ext-calculator/issues",
                ),
            ]);

        let history = flags.config.history.clone();
        let tax_rate_input = flags.config.tax_rate.to_string();

        let mut app = CosmicCalculator {
            core,
            about,
            context_page: ContextPage::default(),
            key_binds: key_binds(),
            modifiers: Modifiers::empty(),
            config_handler: flags.config_handler,
            config: flags.config,
            mode: Mode::Standard,
            expression: String::new(),
            display: String::from("0"),
            just_evaluated: false,
            pending_quick: None,
            tax_rate_input,
            last_result: None,
            current_base: BaseDisplay::Dec,
            tvm_n: String::new(),
            tvm_rate: String::new(),
            tvm_pv: String::new(),
            tvm_pmt: String::new(),
            tvm_fv: String::new(),
            active_tvm_field: TvmField::N,
            history,
            toasts: widget::toaster::Toasts::new(Message::CloseToast),
        };

        let task = app.set_window_title(fl!("app-title"));

        (app, task)
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            RcElementWrapper::new(Element::from(menu::root(fl!("menu-root")))),
            menu::items(
                &self.key_binds,
                vec![
                    menu::Item::Button(
                        fl!("clear-history"),
                        Some(icons::get_handle("edit-clear-all-symbolic", 14)),
                        MenuAction::ClearHistory,
                    ),
                    menu::Item::Button(
                        fl!("settings"),
                        Some(icons::get_handle("emblem-system-symbolic", 14)),
                        MenuAction::Settings,
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

        let tab = |label: &str, mode: Mode| -> Element<'_, Message> {
            let style = if self.mode == mode {
                cosmic::theme::Button::Suggested
            } else {
                cosmic::theme::Button::Standard
            };
            widget::button::custom(
                widget::container(widget::text(label.to_string()).size(11.0))
                    .center(cosmic::iced::Length::Fill)
                    .width(cosmic::iced::Length::Fill)
                    .height(cosmic::iced::Length::Fill),
            )
            .class(style)
            .width(cosmic::iced::Length::Fill)
            .height(cosmic::iced::Length::Fixed(28.0))
            .on_press(Message::SwitchMode(mode))
            .into()
        };

        let tab_bar = widget::row::with_capacity(3)
            .push(tab(&fl!("standard"), Mode::Standard))
            .push(tab(&fl!("engineering"), Mode::Engineering))
            .push(tab(&fl!("financial"), Mode::Financial))
            .spacing(spacing.space_xxs)
            .width(cosmic::iced::Length::Fill);

        // While a two-step Margin/Markup is pending, the small line carries
        // the cost prompt and the large line shows the percentage being typed;
        // otherwise the small line is the expression and the large line the
        // result.
        let (top_line, bottom_line) = match self.pending_quick {
            Some((func, cost)) => {
                let label = match func {
                    QuickFinancial::Markup => "markup",
                    _ => "margin",
                };
                let entry = if self.expression.is_empty() {
                    "0".to_string()
                } else {
                    self.expression.clone()
                };
                (format!("cost {cost} · {label} %?"), entry)
            }
            None => (self.expression.clone(), self.display.clone()),
        };

        let display = widget::column::with_capacity(2)
            .push(
                widget::text::body(top_line)
                    .width(cosmic::iced::Length::Fill)
                    .align_x(cosmic::iced::Alignment::End),
            )
            .push(
                widget::text::title1(bottom_line)
                    .width(cosmic::iced::Length::Fill)
                    .align_x(cosmic::iced::Alignment::End),
            )
            .padding(spacing.space_s);

        let row_spacing = row_spacing_for(self.mode);
        let button_grid = match self.mode {
            Mode::Standard => ui::standard::view(row_spacing),
            Mode::Engineering => {
                ui::engineering::view(self.config.angle_mode, self.current_base, row_spacing)
            }
            Mode::Financial => ui::financial::view(
                self.active_tvm_field,
                &self.tvm_n,
                &self.tvm_rate,
                &self.tvm_pv,
                &self.tvm_pmt,
                &self.tvm_fv,
                row_spacing,
            ),
        };

        widget::column::with_capacity(4)
            .push(tab_bar)
            .push(display)
            .push(button_grid)
            .push(widget::row::with_capacity(1).push(widget::toaster(
                &self.toasts,
                widget::Space::new().width(cosmic::iced::Length::Fill),
            )))
            .width(cosmic::iced::Length::Fill)
            .height(cosmic::iced::Length::Fill)
            .spacing(spacing.space_xs)
            .padding(spacing.space_xxs)
            .into()
    }

    fn update(&mut self, message: Self::Message) -> Task {
        let mut tasks = vec![];
        match message {
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
            Message::Number(n) => {
                self.start_fresh_if_evaluated();
                self.expression.push_str(&n.to_string());
            }
            Message::Operator(op) => {
                // Chain from the previous result rather than starting over.
                self.just_evaluated = false;
                self.expression.push_str(op.expression());
            }
            Message::Evaluate => {
                // Finish a pending two-step Margin/Markup rather than
                // evaluating the price entry as a bare expression.
                if let Some((func, _)) = self.pending_quick {
                    return self.update(Message::QuickFinancial(func));
                }
                use crate::engine::Evaluate;
                let result = match self.mode {
                    Mode::Standard => {
                        crate::engine::standard::StandardEngine.evaluate(&self.expression)
                    }
                    Mode::Engineering => {
                        let angle = match self.config.angle_mode {
                            config::AngleMode::Deg => crate::engine::AngleMode::Deg,
                            config::AngleMode::Rad => crate::engine::AngleMode::Rad,
                            config::AngleMode::Grad => crate::engine::AngleMode::Grad,
                        };
                        crate::engine::engineering::EngineeringEngine::new(angle)
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
                            let _ = self
                                .config
                                .set_history(config_handler, self.history.clone());
                        }
                        self.display = result.display.clone();
                        self.expression = result.value.to_string();
                        self.just_evaluated = true;
                        // A fresh result is shown in decimal; keep it around so
                        // the base buttons can reformat it.
                        self.current_base = BaseDisplay::Dec;
                        self.last_result = Some(result);
                    }
                    Err(e) => {
                        return self.update(Message::ShowToast(e.to_string()));
                    }
                }
            }
            Message::Clear => {
                self.expression.clear();
                self.display = String::from("0");
                self.just_evaluated = false;
                self.pending_quick = None;
                self.last_result = None;
                self.current_base = BaseDisplay::Dec;
            }
            Message::Backspace => {
                self.just_evaluated = false;
                self.expression.pop();
            }
            Message::Decimal => {
                self.start_fresh_if_evaluated();
                self.expression.push('.');
            }
            Message::Percent => {
                self.just_evaluated = false;
                self.expression.push('%');
            }
            Message::ParenOpen => {
                // An opening paren begins a new sub-expression, so start fresh
                // after a result instead of turning "7" into "7(".
                self.start_fresh_if_evaluated();
                self.expression.push('(');
            }
            Message::ParenClose => {
                self.just_evaluated = false;
                self.expression.push(')');
            }
            Message::Undo => {
                self.expression.pop();
            }
            Message::CopyResult => {
                // Clipboard support comes later
            }
            Message::EngFunction(func) => {
                if func.ends_with('(') {
                    // Prefix function (sin/cos/log/sqrt...): apply to the
                    // current operand, wrapping it as the argument.
                    self.apply_prefix_function(&func);
                } else if func.starts_with('^') || func == "!" {
                    // Postfix/infix operator (x², xʸ, n!): operates on the
                    // preceding number or result, so keep it and append.
                    self.just_evaluated = false;
                    self.expression.push_str(&func);
                } else {
                    // Constant (π, e): behaves like an entered value.
                    self.start_fresh_if_evaluated();
                    self.expression.push_str(&func);
                }
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
                self.just_evaluated = false;
                self.expression.push_str(symbol);
            }
            Message::BaseDisplay(base) => {
                let Some(result) = &self.last_result else {
                    return self.update(Message::ShowToast("Evaluate something first".into()));
                };
                let repr = match base {
                    BaseDisplay::Dec => Some(result.display.clone()),
                    BaseDisplay::Hex => result.alt_bases.as_ref().map(|a| format!("0x{}", a.hex)),
                    BaseDisplay::Oct => result.alt_bases.as_ref().map(|a| format!("0o{}", a.oct)),
                    BaseDisplay::Bin => result.alt_bases.as_ref().map(|a| format!("0b{}", a.bin)),
                };
                match repr {
                    Some(s) => {
                        self.display = s;
                        self.current_base = base;
                    }
                    None => {
                        return self.update(Message::ShowToast(
                            "Only non-negative integers have hex/oct/bin".into(),
                        ));
                    }
                }
            }
            Message::TvmSelectField(field) => {
                self.active_tvm_field = field;
            }
            Message::TvmFieldChanged(field, value) => match field {
                TvmField::N => self.tvm_n = value,
                TvmField::Rate => self.tvm_rate = value,
                TvmField::Pv => self.tvm_pv = value,
                TvmField::Pmt => self.tvm_pmt = value,
                TvmField::Fv => self.tvm_fv = value,
            },
            Message::TvmSolve(field) => {
                use crate::engine::financial::{FinancialEngine, TvmParams, TvmSolveFor};

                let parse = |s: &str| -> Option<f64> {
                    if s.is_empty() {
                        None
                    } else {
                        s.parse().ok()
                    }
                };

                let params = TvmParams {
                    n: if matches!(field, TvmField::N) {
                        None
                    } else {
                        parse(&self.tvm_n)
                    },
                    rate: if matches!(field, TvmField::Rate) {
                        None
                    } else {
                        parse(&self.tvm_rate)
                    },
                    pv: if matches!(field, TvmField::Pv) {
                        None
                    } else {
                        parse(&self.tvm_pv)
                    },
                    pmt: if matches!(field, TvmField::Pmt) {
                        None
                    } else {
                        parse(&self.tvm_pmt)
                    },
                    fv: if matches!(field, TvmField::Fv) {
                        None
                    } else {
                        parse(&self.tvm_fv)
                    },
                };

                let solve_for = match field {
                    TvmField::N => TvmSolveFor::N,
                    TvmField::Rate => TvmSolveFor::Rate,
                    TvmField::Pv => TvmSolveFor::Pv,
                    TvmField::Pmt => TvmSolveFor::Pmt,
                    TvmField::Fv => TvmSolveFor::Fv,
                };

                match FinancialEngine.solve_tvm(params, solve_for) {
                    Ok(result) => {
                        let formatted = format!("{:.2}", result);
                        match field {
                            TvmField::N => self.tvm_n = formatted.clone(),
                            TvmField::Rate => self.tvm_rate = formatted.clone(),
                            TvmField::Pv => self.tvm_pv = formatted.clone(),
                            TvmField::Pmt => self.tvm_pmt = formatted.clone(),
                            TvmField::Fv => self.tvm_fv = formatted.clone(),
                        }
                        self.display = formatted;
                    }
                    Err(e) => {
                        return self.update(Message::ShowToast(e.to_string()));
                    }
                }
            }
            Message::QuickFinancial(func) => {
                use crate::engine::financial::FinancialEngine;
                let engine = FinancialEngine;
                let Ok(value) = self.expression.parse::<f64>() else {
                    return self.update(Message::ShowToast("Enter a valid number first".into()));
                };
                match func {
                    QuickFinancial::TaxAdd | QuickFinancial::TaxRemove => {
                        // Single operand: operate immediately on the entry.
                        self.pending_quick = None;
                        let result = match func {
                            QuickFinancial::TaxAdd => engine.add_tax(value, self.config.tax_rate),
                            _ => engine.remove_tax(value, self.config.tax_rate),
                        };
                        self.display = format!("{result:.2}");
                        self.expression = result.to_string();
                        self.just_evaluated = true;
                    }
                    QuickFinancial::Margin | QuickFinancial::Markup => {
                        match self.pending_quick.take() {
                            // First press: stash the cost. The prompt for the
                            // percentage is rendered from `pending_quick` in
                            // the view (small line).
                            None => {
                                self.pending_quick = Some((func, value));
                                self.expression.clear();
                            }
                            // Second value is the percentage; compute the
                            // selling price with the first-press function.
                            Some((pending_func, cost)) => {
                                let result = match pending_func {
                                    QuickFinancial::Markup => {
                                        engine.price_from_markup(cost, value)
                                    }
                                    // Only Margin/Markup are ever stashed.
                                    _ => engine.price_from_margin(cost, value),
                                };
                                match result {
                                    Ok(price) => {
                                        self.display = format!("{price:.2}");
                                        self.expression = price.to_string();
                                        self.just_evaluated = true;
                                    }
                                    Err(e) => {
                                        return self.update(Message::ShowToast(e.to_string()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Message::ToggleSign => {
                if self.expression.starts_with('-') {
                    self.expression = self.expression[1..].to_string();
                } else if !self.expression.is_empty() {
                    self.expression = format!("-{}", self.expression);
                }
            }
            Message::SwitchMode(mode) => {
                self.mode = mode;
            }
            Message::HistorySelect(index) => {
                if let Some(entry) = self.history.get(index) {
                    self.expression = entry.result.clone();
                    self.display = entry.result.clone();
                }
            }
            Message::TaxRateChanged(value) => {
                // Keep the raw text so partial entries survive; mirror valid
                // numbers into the persisted config for Tax+/Tax−.
                if let Ok(rate) = value.parse::<f64>() {
                    self.config.tax_rate = rate;
                    if let Some(config_handler) = &self.config_handler {
                        let _ = self.config.set_tax_rate(config_handler, rate);
                    }
                }
                self.tax_rate_input = value;
            }
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
                let content = ui::history::view(&self.history);
                context_drawer::context_drawer(content, Message::ToggleContextDrawer)
            }
            ContextPage::Settings => {
                let content = ui::settings::view(&self.tax_rate_input);
                context_drawer::context_drawer(content, Message::ToggleContextDrawer)
                    .title(fl!("settings"))
            }
        })
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        let subscriptions = vec![
            event::listen_with(|event, status, _id| match event {
                Event::Keyboard(KeyEvent::KeyPressed {
                    modified_key,
                    modifiers,
                    ..
                }) => match status {
                    // Use `modified_key` (Shift/AltGr applied) rather than the raw
                    // `key`, otherwise on layouts where symbols require Shift the
                    // base key leaks through: e.g. on a Spanish keyboard `*` is
                    // Shift+`+` and `/` is Shift+`7`, so `key` would report `+`/`7`.
                    event::Status::Ignored => Some(Message::Key(modifiers, modified_key)),
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
