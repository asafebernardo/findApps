use async_trait::async_trait;
use tracing::debug;

use crate::backends::PackageBackend;
use crate::models::{
    AppInfo, BackendError, BackendResult, BackendStatus, InstallMethod,
};
use crate::system::privilege;
use crate::system::process::{command_exists_sync, run_command};
use crate::util::validation::validate_package_id;

pub struct DnfBackend;

impl DnfBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DnfBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PackageBackend for DnfBackend {
    fn id(&self) -> InstallMethod {
        InstallMethod::Dnf
    }

    fn is_available(&self) -> bool {
        // DNF only — presence of `rpm` alone (e.g. on Ubuntu) is not enough.
        command_exists_sync("dnf")
    }

    async fn detect(&self) -> BackendStatus {
        if !self.is_available() {
            return BackendStatus::unavailable(InstallMethod::Dnf);
        }
        let version = run_command("dnf", &["--version"])
            .await
            .ok()
            .and_then(|o| o.lines().next().map(|l| l.to_string()));
        BackendStatus::available(InstallMethod::Dnf, version)
    }

    async fn list_installed(&self) -> BackendResult<Vec<AppInfo>> {
        if !self.is_available() {
            return Err(BackendError::Unavailable);
        }

        let output = run_command(
            "rpm",
            &[
                "-qa",
                "--qf",
                "%{NAME}\t%{VERSION}-%{RELEASE}\t%{ARCH}\t%{SIZE}\t%{SUMMARY}\n",
            ],
        )
        .await?;

        let desktop_names = collect_desktop_package_names().await;
        let mut apps = Vec::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 5 {
                continue;
            }
            let name = parts[0];
            if is_rpm_library(name) && !desktop_names.contains(name) {
                continue;
            }
            if !desktop_names.contains(name) && !looks_like_app(name) {
                continue;
            }
            let mut app = AppInfo::new(InstallMethod::Dnf, name, humanize(name));
            app.version = Some(parts[1].to_string());
            app.architecture = Some(parts[2].to_string());
            if let Ok(size) = parts[3].parse::<u64>() {
                app.size_bytes = Some(size);
            }
            app.description = Some(parts[4].to_string());
            app.origin = Some("Repositório DNF/RPM".into());
            app.icon_name = Some(name.to_string());
            apps.push(app);
        }
        debug!(count = apps.len(), "DNF: aplicativos listados");
        Ok(apps)
    }

    async fn get_details(&self, id: &str) -> BackendResult<AppInfo> {
        validate_package_id(InstallMethod::Dnf, id)?;
        let apps = self.list_installed().await?;
        apps.into_iter()
            .find(|a| a.package_id == id)
            .ok_or_else(|| BackendError::NotFound(id.to_string()))
    }

    async fn uninstall(&self, id: &str) -> BackendResult<()> {
        validate_package_id(InstallMethod::Dnf, id)?;
        privilege::uninstall_with_privilege(InstallMethod::Dnf, id).await
    }
}

async fn collect_desktop_package_names() -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    let dir = std::path::Path::new("/usr/share/applications");
    let Ok(entries) = std::fs::read_dir(dir) else {
        return set;
    };
    for entry in entries.flatten().take(400) {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
            continue;
        }
        let path_str = path.to_string_lossy().to_string();
        if let Ok(out) = run_command("rpm", &["-qf", "--qf", "%{NAME}\n", &path_str]).await {
            let name = out.trim();
            if !name.is_empty() && !name.contains("not owned") {
                set.insert(name.to_string());
            }
        }
    }
    set
}

fn is_rpm_library(name: &str) -> bool {
    name.starts_with("lib")
        || name.starts_with("python3-")
        || name.ends_with("-devel")
        || name.ends_with("-libs")
        || name.ends_with("-debuginfo")
}

fn looks_like_app(name: &str) -> bool {
    const KNOWN: &[&str] = &[
        "firefox", "chromium", "thunderbird", "gimp", "vlc", "libreoffice",
        "code", "discord", "spotify", "steam", "inkscape", "blender",
    ];
    KNOWN
        .iter()
        .any(|k| name == *k || name.starts_with(&format!("{k}-")))
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
    async fn detect_unavailable_without_dnf() {
        let backend = DnfBackend::new();
        if !command_exists_sync("dnf") {
            let status = backend.detect().await;
            assert!(!status.is_usable());
        }
    }

    #[test]
    fn library_filter() {
        assert!(is_rpm_library("libfoo"));
        assert!(!is_rpm_library("firefox"));
    }
}
