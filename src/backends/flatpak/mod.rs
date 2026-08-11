use async_trait::async_trait;
use tracing::debug;

use crate::backends::PackageBackend;
use crate::models::{
    AppInfo, BackendError, BackendResult, BackendStatus, InstallMethod,
};
use crate::system::privilege;
use crate::system::process::{command_exists_sync, run_command};
use crate::util::validation::validate_package_id;

pub struct FlatpakBackend;

impl FlatpakBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FlatpakBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PackageBackend for FlatpakBackend {
    fn id(&self) -> InstallMethod {
        InstallMethod::Flatpak
    }

    fn is_available(&self) -> bool {
        command_exists_sync("flatpak")
    }

    async fn detect(&self) -> BackendStatus {
        if !self.is_available() {
            return BackendStatus::unavailable(InstallMethod::Flatpak);
        }
        let version = run_command("flatpak", &["--version"])
            .await
            .ok()
            .map(|s| s.trim().to_string());
        BackendStatus::available(InstallMethod::Flatpak, version)
    }

    async fn list_installed(&self) -> BackendResult<Vec<AppInfo>> {
        if !self.is_available() {
            return Err(BackendError::Unavailable);
        }

        let output = run_command(
            "flatpak",
            &[
                "list",
                "--app",
                "--columns=application,name,version,branch,origin,installation,size,arch",
            ],
        )
        .await?;

        let mut apps = Vec::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.is_empty() || parts[0].is_empty() {
                continue;
            }
            let app_id = parts[0];
            let name = parts.get(1).filter(|s| !s.is_empty()).unwrap_or(&app_id);
            let mut app = AppInfo::new(InstallMethod::Flatpak, app_id, *name);
            if let Some(v) = parts.get(2).filter(|s| !s.is_empty()) {
                app.version = Some((*v).to_string());
            }
            if let Some(origin) = parts.get(4) {
                app.origin = Some((*origin).to_string());
            }
            if let Some(inst) = parts.get(5) {
                app.install_path = Some(format!("flatpak:{inst}"));
            }
            if let Some(size) = parts.get(6) {
                app.size_bytes = parse_flatpak_size(size);
            }
            if let Some(arch) = parts.get(7) {
                app.architecture = Some((*arch).to_string());
            }
            app.icon_name = Some(app_id.to_string());
            app.developer = app_id
                .split('.')
                .take(2)
                .collect::<Vec<_>>()
                .join(".")
                .into();
            apps.push(app);
        }
        debug!(count = apps.len(), "Flatpak: aplicativos listados");
        Ok(apps)
    }

    async fn get_details(&self, id: &str) -> BackendResult<AppInfo> {
        validate_package_id(InstallMethod::Flatpak, id)?;
        let apps = self.list_installed().await?;
        apps.into_iter()
            .find(|a| a.package_id == id)
            .ok_or_else(|| BackendError::NotFound(id.to_string()))
    }

    async fn uninstall(&self, id: &str) -> BackendResult<()> {
        validate_package_id(InstallMethod::Flatpak, id)?;
        privilege::uninstall_with_privilege(InstallMethod::Flatpak, id).await
    }
}

fn parse_flatpak_size(s: &str) -> Option<u64> {
    let s = s.trim().replace(',', ".");
    let (num, mult) = if let Some(n) = s.strip_suffix("GB").or_else(|| s.strip_suffix('G')) {
        (n.trim(), 1_073_741_824u64)
    } else if let Some(n) = s.strip_suffix("MB").or_else(|| s.strip_suffix('M')) {
        (n.trim(), 1_048_576u64)
    } else if let Some(n) = s.strip_suffix("kB").or_else(|| s.strip_suffix('k')) {
        (n.trim(), 1024u64)
    } else if let Some(n) = s.strip_suffix('B') {
        (n.trim(), 1u64)
    } else {
        return s.parse().ok();
    };
    let f: f64 = num.parse().ok()?;
    Some((f * mult as f64) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sizes() {
        assert_eq!(parse_flatpak_size("245 MB"), Some(245 * 1_048_576));
        assert_eq!(parse_flatpak_size("1.5 GB"), Some((1.5 * 1_073_741_824.0) as u64));
    }

    #[tokio::test]
    async fn detect_works() {
        let backend = FlatpakBackend::new();
        let status = backend.detect().await;
        assert_eq!(status.method, InstallMethod::Flatpak);
    }
}
