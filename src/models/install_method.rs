use serde::{Deserialize, Serialize};
use std::fmt;

use crate::i18n::t;

/// Método de instalação / origem do pacote.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum InstallMethod {
    Apt,
    Dnf,
    Flatpak,
    Snap,
    AppImage,
    Manual,
    System,
}

impl InstallMethod {
    /// Stable technical key (never translated — used in app ids).
    pub fn id_key(&self) -> &'static str {
        match self {
            Self::Apt => "apt",
            Self::Dnf => "dnf",
            Self::Flatpak => "flatpak",
            Self::Snap => "snap",
            Self::AppImage => "appimage",
            Self::Manual => "manual",
            Self::System => "system",
        }
    }

    /// Localized display name.
    pub fn as_str(&self) -> String {
        match self {
            Self::Apt => "APT".to_string(),
            Self::Dnf => "DNF".to_string(),
            Self::Flatpak => "Flatpak".to_string(),
            Self::Snap => "Snap".to_string(),
            Self::AppImage => "AppImage".to_string(),
            Self::Manual => t("manual"),
            Self::System => t("system"),
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Apt => "method-apt",
            Self::Dnf => "method-dnf",
            Self::Flatpak => "method-flatpak",
            Self::Snap => "method-snap",
            Self::AppImage => "method-appimage",
            Self::Manual => "method-manual",
            Self::System => "method-system",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Apt => "package-x-generic-symbolic",
            Self::Dnf => "package-x-generic-symbolic",
            Self::Flatpak => "application-x-addon-symbolic",
            Self::Snap => "snap-symbolic",
            Self::AppImage => "application-x-executable-symbolic",
            Self::Manual => "folder-symbolic",
            Self::System => "computer-symbolic",
        }
    }

    pub fn all() -> &'static [InstallMethod] {
        &[
            Self::Apt,
            Self::Dnf,
            Self::Flatpak,
            Self::Snap,
            Self::AppImage,
            Self::Manual,
            Self::System,
        ]
    }
}

impl fmt::Display for InstallMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
