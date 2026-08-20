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
    if std::env::var_os("RUST_LOG").is_some() {
        let _ = env_logger::try_init();
    }
    if std::env::var_os("WGPU_BACKEND").is_none() {
        std::env::set_var("WGPU_BACKEND", "gl,vulkan");
    }
    config::load();   // first: define music_dir, language, volume…
    db::init();       // initialize local database
    stats::init();    // initialize listening statistics
    locale::load();
    ui::theme::load_system_theme();
    let res = app::run();
    // Best-effort synchronous flush on normal exit (covers close via window X, SIGTERM, etc.).
    // Does not cover SIGKILL (kill -9) — for that, high-value writes now flush() immediately (see db.rs).
    db::flush_sync();
    stats::flush_sync();
    res
}
