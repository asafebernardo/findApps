use serde::{Deserialize, Serialize};

use super::InstallMethod;
use crate::i18n::tf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendAvailability {
    Available,
    Unavailable,
    Detectable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendStatus {
    pub method: InstallMethod,
    pub availability: BackendAvailability,
    pub message: String,
    pub version: Option<String>,
}

impl BackendStatus {
    pub fn available(method: InstallMethod, version: Option<String>) -> Self {
        Self {
            method,
            availability: BackendAvailability::Available,
            message: tf("backend_available", &[("method", &method.as_str())]),
            version,
        }
    }

    pub fn unavailable(method: InstallMethod) -> Self {
        Self {
            method,
            availability: BackendAvailability::Unavailable,
            message: tf("backend_unavailable", &[("method", &method.as_str())]),
            version: None,
        }
    }

    pub fn detectable(method: InstallMethod) -> Self {
        Self {
            method,
            availability: BackendAvailability::Detectable,
            message: tf("backend_detectable", &[("method", &method.as_str())]),
            version: None,
        }
    }

    pub fn is_usable(&self) -> bool {
        matches!(
            self.availability,
            BackendAvailability::Available | BackendAvailability::Detectable
        )
    }
}
