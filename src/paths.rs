use std::path::PathBuf;

fn config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home).join(".config/omatunes")
}

pub fn db() -> PathBuf {
    config_dir().join("db.json")
}

pub fn stats() -> PathBuf {
    config_dir().join("stats.json")
}

pub fn config_toml() -> PathBuf {
    config_dir().join("config.toml")
}
