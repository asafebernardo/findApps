use serde::{Deserialize, Serialize};

use super::InstallMethod;

/// Informação de atualização disponível (arquitetura futura).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub package_id: String,
    pub name: String,
    pub method: InstallMethod,
    pub current_version: String,
    pub available_version: String,
}
