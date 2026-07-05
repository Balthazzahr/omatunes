mod app;
mod audio;
mod config;
mod db;
mod library;
mod locale;
mod stats;
mod ui;

fn main() -> iced::Result {
    config::load();   // primeiro: define music_dir, language, volume…
    db::init();       // inicializa o banco de dados local
    stats::init();    // inicializa estatísticas de audição
    locale::load();   // usa config.language
    ui::theme::load_system_theme();
    app::run()
}
