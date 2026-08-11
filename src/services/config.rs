use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::i18n::Language;
use crate::system::paths::{config_dir, config_file, default_appimage_dirs};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub appimage_dirs: Vec<PathBuf>,
    pub show_system_components: bool,
    /// Interface language code: en, zh, es, hi, ar, pt, ru
    #[serde(default)]
    pub language: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            appimage_dirs: default_appimage_dirs(),
            show_system_components: false,
            language: Language::English.code().to_string(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let path = config_file();
        match fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap_or_default(),
            Err(_) => {
                let cfg = Self::default();
                let _ = cfg.save();
                cfg
            }
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        fs::create_dir_all(config_dir())?;
        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(config_file(), content)
    }

    pub fn language_enum(&self) -> Language {
        Language::from_code(&self.language)
    }
}
