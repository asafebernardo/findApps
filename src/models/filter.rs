use super::InstallMethod;
use crate::i18n::t;

/// Filtro da lista de aplicativos.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AppFilter {
    #[default]
    Home,
    All,
    Method(InstallMethod),
    Applications,
    Settings,
}

impl AppFilter {
    pub fn matches_method(&self, method: InstallMethod) -> bool {
        match self {
            Self::Home | Self::Settings => false,
            Self::All | Self::Applications => true,
            Self::Method(m) => *m == method,
        }
    }

    pub fn title(&self) -> String {
        match self {
            Self::Home => t("home"),
            Self::All => t("all_apps"),
            Self::Method(m) => m.as_str(),
            Self::Applications => t("all"),
            Self::Settings => t("settings"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortBy {
    #[default]
    Name,
    Size,
    InstallDate,
    Method,
    UpdateAvailable,
}

impl SortBy {
    pub fn label(&self) -> String {
        match self {
            Self::Name => t("sort_name"),
            Self::Size => t("sort_size"),
            Self::InstallDate => t("sort_date"),
            Self::Method => t("sort_method"),
            Self::UpdateAvailable => t("sort_update"),
        }
    }
}
