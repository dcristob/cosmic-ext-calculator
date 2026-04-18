use std::sync::Mutex;

use crate::{
    app::{config::CalculatorConfig, Flags},
    core::{
        icons::{IconCache, ICON_CACHE},
        localization::localize,
    },
};
use cosmic::{
    app::Settings,
    iced::{Limits, Size},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

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
    settings = settings.size_limits(
        Limits::NONE
            .min_width(320.0)
            .max_width(320.0)
            .min_height(690.0)
            .max_height(690.0),
    );
    settings = settings.size(Size::new(320.0, 690.0));
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
