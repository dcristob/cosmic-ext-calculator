use app::CosmicCalculator;

mod app;
mod core;
mod engine;
mod ui;

fn main() -> cosmic::iced::Result {
    app::settings::init();
    let (settings, flags) = (app::settings::settings(), app::settings::flags());
    cosmic::app::run::<CosmicCalculator>(settings, flags)
}
