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
