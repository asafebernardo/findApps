use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::backends::PackageBackend;
use crate::models::{
    AppInfo, BackendError, BackendResult, BackendStatus, InstallMethod, UpdateInfo,
};

/// Backend em memória para testes.
pub struct MockBackend {
    method: InstallMethod,
    available: bool,
    apps: Mutex<Vec<AppInfo>>,
    fail_uninstall: bool,
    permission_denied: bool,
}

impl MockBackend {
    pub fn new(method: InstallMethod, available: bool) -> Self {
        Self {
            method,
            available,
            apps: Mutex::new(Vec::new()),
            fail_uninstall: false,
            permission_denied: false,
        }
    }

    pub fn with_apps(method: InstallMethod, apps: Vec<AppInfo>) -> Self {
        Self {
            method,
            available: true,
            apps: Mutex::new(apps),
            fail_uninstall: false,
            permission_denied: false,
        }
    }

    pub fn with_permission_denied(mut self) -> Self {
        self.permission_denied = true;
        self
    }

    pub fn add_app(&self, app: AppInfo) {
        self.apps.lock().unwrap().push(app);
    }
}

#[async_trait]
impl PackageBackend for MockBackend {
    fn id(&self) -> InstallMethod {
        self.method
    }

    fn is_available(&self) -> bool {
        self.available
    }

    async fn detect(&self) -> BackendStatus {
        if self.available {
            BackendStatus::available(self.method, Some("mock".into()))
        } else {
            BackendStatus::unavailable(self.method)
        }
    }

    async fn list_installed(&self) -> BackendResult<Vec<AppInfo>> {
        if !self.available {
            return Err(BackendError::Unavailable);
        }
        Ok(self.apps.lock().unwrap().clone())
    }

    async fn get_details(&self, id: &str) -> BackendResult<AppInfo> {
        self.apps
            .lock()
            .unwrap()
            .iter()
            .find(|a| a.package_id == id || a.id == id)
            .cloned()
            .ok_or_else(|| BackendError::NotFound(id.to_string()))
    }

    async fn uninstall(&self, id: &str) -> BackendResult<()> {
        if self.permission_denied {
            return Err(BackendError::PermissionDenied("mock".into()));
        }
        if self.fail_uninstall {
            return Err(BackendError::CommandFailed("mock fail".into()));
        }
        let mut apps = self.apps.lock().unwrap();
        let before = apps.len();
        apps.retain(|a| a.package_id != id && a.id != id);
        if apps.len() == before {
            return Err(BackendError::NotFound(id.to_string()));
        }
        Ok(())
    }

    async fn check_updates(&self) -> BackendResult<Vec<UpdateInfo>> {
        Ok(Vec::new())
    }
}

/// Registro de mocks por método (útil em testes de integração).
pub fn mock_registry() -> HashMap<InstallMethod, MockBackend> {
    HashMap::new()
}
