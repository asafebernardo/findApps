use async_trait::async_trait;
use tracing::debug;

use crate::backends::PackageBackend;
use crate::models::{
    AppInfo, BackendError, BackendResult, BackendStatus, InstallMethod,
};
use crate::system::privilege;
use crate::system::process::{command_exists_sync, run_command};
use crate::util::validation::validate_package_id;

pub struct SnapBackend;

impl SnapBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SnapBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PackageBackend for SnapBackend {
    fn id(&self) -> InstallMethod {
        InstallMethod::Snap
    }

    fn is_available(&self) -> bool {
        command_exists_sync("snap")
    }

    async fn detect(&self) -> BackendStatus {
        if !self.is_available() {
            return BackendStatus::unavailable(InstallMethod::Snap);
        }
        let version = run_command("snap", &["version"])
            .await
            .ok()
            .and_then(|o| {
                o.lines()
                    .find(|l| l.starts_with("snap "))
                    .map(|l| l.trim().to_string())
                    .or_else(|| o.lines().next().map(|l| l.to_string()))
            });
        BackendStatus::available(InstallMethod::Snap, version)
    }

    async fn list_installed(&self) -> BackendResult<Vec<AppInfo>> {
        if !self.is_available() {
            return Err(BackendError::Unavailable);
        }

        let output = run_command("snap", &["list"]).await?;
        let mut apps = Vec::new();
        for (i, line) in output.lines().enumerate() {
            if i == 0 {
                continue; // header
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                continue;
            }
            let name = parts[0];
            if name == "core" || name.starts_with("core") || name == "snapd" || name == "bare" {
                continue;
            }
            let mut app = AppInfo::new(InstallMethod::Snap, name, humanize(name));
            app.version = Some(parts[1].to_string());
            if parts.len() > 3 {
                app.origin = Some(parts[3].to_string());
            }
            if parts.len() > 4 {
                // publisher
                app.developer = Some(parts[4..].join(" ").replace('*', ""));
            }
            app.install_path = Some(format!("/snap/{name}"));
            app.icon_name = Some(name.to_string());
            apps.push(app);
        }
        debug!(count = apps.len(), "Snap: aplicativos listados");
        Ok(apps)
    }

    async fn get_details(&self, id: &str) -> BackendResult<AppInfo> {
        validate_package_id(InstallMethod::Snap, id)?;
        let apps = self.list_installed().await?;
        apps.into_iter()
            .find(|a| a.package_id == id)
            .ok_or_else(|| BackendError::NotFound(id.to_string()))
    }

    async fn uninstall(&self, id: &str) -> BackendResult<()> {
        validate_package_id(InstallMethod::Snap, id)?;
        privilege::uninstall_with_privilege(InstallMethod::Snap, id).await
    }
}

fn humanize(pkg: &str) -> String {
    pkg.replace('-', " ")
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn detect_works() {
        let backend = SnapBackend::new();
        let status = backend.detect().await;
        assert_eq!(status.method, InstallMethod::Snap);
    }
}
