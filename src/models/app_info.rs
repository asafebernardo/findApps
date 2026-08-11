use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::InstallMethod;

/// Status de um aplicativo instalado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AppStatus {
    #[default]
    Installed,
    UpdateAvailable,
    Broken,
    Unknown,
}

impl AppStatus {
    pub fn as_str(&self) -> String {
        match self {
            Self::Installed => crate::i18n::t("status_installed"),
            Self::UpdateAvailable => crate::i18n::t("status_update"),
            Self::Broken => crate::i18n::t("status_broken"),
            Self::Unknown => crate::i18n::t("status_unknown"),
        }
    }
}

/// Informações normalizadas de um aplicativo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    /// Identificador único interno: `{method}:{package_id}`
    pub id: String,
    pub name: String,
    pub icon_name: Option<String>,
    pub icon_path: Option<String>,
    pub developer: Option<String>,
    pub version: Option<String>,
    pub method: InstallMethod,
    pub architecture: Option<String>,
    pub install_date: Option<DateTime<Utc>>,
    pub size_bytes: Option<u64>,
    pub install_path: Option<String>,
    pub status: AppStatus,
    pub category: Option<String>,
    pub description: Option<String>,
    pub origin: Option<String>,
    /// Versão disponível para atualização (futuro).
    pub update_available: Option<String>,
    /// ID nativo do backend (nome do pacote, ref flatpak, etc.).
    pub package_id: String,
    pub desktop_id: Option<String>,
}

impl AppInfo {
    pub fn new(method: InstallMethod, package_id: impl Into<String>, name: impl Into<String>) -> Self {
        let package_id = package_id.into();
        let id = format!("{}:{}", method.id_key(), package_id);
        Self {
            id,
            name: name.into(),
            icon_name: None,
            icon_path: None,
            developer: None,
            version: None,
            method,
            architecture: None,
            install_date: None,
            size_bytes: None,
            install_path: None,
            status: AppStatus::Installed,
            category: None,
            description: None,
            origin: None,
            update_available: None,
            package_id,
            desktop_id: None,
        }
    }

    pub fn matches_query(&self, query: &str) -> bool {
        let q = query.to_lowercase();
        if q.is_empty() {
            return true;
        }
        self.name.to_lowercase().contains(&q)
            || self
                .developer
                .as_ref()
                .map(|d| d.to_lowercase().contains(&q))
                .unwrap_or(false)
            || self.package_id.to_lowercase().contains(&q)
            || self
                .description
                .as_ref()
                .map(|d| d.to_lowercase().contains(&q))
                .unwrap_or(false)
            || self.method.as_str().to_lowercase().contains(&q)
    }
}
