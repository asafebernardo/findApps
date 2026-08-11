use async_trait::async_trait;

use crate::models::{
    AppInfo, BackendResult, BackendStatus, InstallMethod, UpdateInfo,
};

/// Interface comum para todos os métodos de instalação.
#[async_trait]
pub trait PackageBackend: Send + Sync {
    fn id(&self) -> InstallMethod;

    fn is_available(&self) -> bool;

    async fn detect(&self) -> BackendStatus;

    async fn list_installed(&self) -> BackendResult<Vec<AppInfo>>;

    async fn get_details(&self, id: &str) -> BackendResult<AppInfo>;

    async fn uninstall(&self, id: &str) -> BackendResult<()>;

    /// Preparado para versões futuras.
    async fn install(&self, _id: &str) -> BackendResult<()> {
        Err(crate::models::BackendError::Unsupported(
            "Instalação será implementada em versões futuras".into(),
        ))
    }

    /// Preparado para versões futuras.
    async fn update(&self, _id: &str) -> BackendResult<()> {
        Err(crate::models::BackendError::Unsupported(
            "Atualização será implementada em versões futuras".into(),
        ))
    }

    /// Preparado para versões futuras.
    async fn check_updates(&self) -> BackendResult<Vec<UpdateInfo>> {
        Ok(Vec::new())
    }
}
