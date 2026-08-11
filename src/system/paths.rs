use std::path::PathBuf;

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from(".").join(".config"))
        .join("findapps")
}

pub fn data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from(".").join(".local").join("share"))
        .join("findapps")
}

pub fn log_dir() -> PathBuf {
    data_dir().join("logs")
}

pub fn config_file() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn default_appimage_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join("Applications"));
        dirs.push(home.join("AppImages"));
        dirs.push(home.join("Downloads"));
        dirs.push(home.join(".local").join("bin"));
    }
    dirs.push(PathBuf::from("/opt"));
    dirs
}
