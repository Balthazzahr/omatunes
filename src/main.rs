mod app;
mod audio;
mod config;
mod db;
mod library;
mod locale;
mod paths;
mod stats;
mod ui;

fn main() -> iced::Result {
    config::load();   // first: define music_dir, language, volume…
    db::init();       // initialize local database
    stats::init();    // initialize listening statistics
    locale::load();   // usa config.language
    ui::theme::load_system_theme();
    app::run()
}
