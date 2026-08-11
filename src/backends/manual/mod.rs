use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use tracing::debug;

use crate::backends::PackageBackend;
use crate::models::{
    AppInfo, BackendError, BackendResult, BackendStatus, InstallMethod,
};

/// Programas detectados via .desktop que não pertencem a outros backends.
pub struct ManualBackend {
    claimed_desktop_ids: HashSet<String>,
    claimed_exec_hints: HashSet<String>,
}

impl ManualBackend {
    pub fn new() -> Self {
        Self {
            claimed_desktop_ids: HashSet::new(),
            claimed_exec_hints: HashSet::new(),
        }
    }

    pub fn with_claims(claimed_desktop_ids: HashSet<String>, claimed_exec_hints: HashSet<String>) -> Self {
        Self {
            claimed_desktop_ids,
            claimed_exec_hints,
        }
    }

    pub fn set_claims(&mut self, desktop_ids: HashSet<String>, exec_hints: HashSet<String>) {
        self.claimed_desktop_ids = desktop_ids;
        self.claimed_exec_hints = exec_hints;
    }
}

impl Default for ManualBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PackageBackend for ManualBackend {
    fn id(&self) -> InstallMethod {
        InstallMethod::Manual
    }

    fn is_available(&self) -> bool {
        true
    }

    async fn detect(&self) -> BackendStatus {
        BackendStatus::detectable(InstallMethod::Manual)
    }

    async fn list_installed(&self) -> BackendResult<Vec<AppInfo>> {
        let claimed_desktop = self.claimed_desktop_ids.clone();
        let claimed_exec = self.claimed_exec_hints.clone();
        let apps = tokio::task::spawn_blocking(move || {
            scan_manual_apps(&claimed_desktop, &claimed_exec)
        })
        .await
        .map_err(|e| BackendError::Other(e.to_string()))?;
        debug!(count = apps.len(), "Manual: aplicativos listados");
        Ok(apps)
    }

    async fn get_details(&self, id: &str) -> BackendResult<AppInfo> {
        let apps = self.list_installed().await?;
        apps.into_iter()
            .find(|a| a.package_id == id || a.id == id)
            .ok_or_else(|| BackendError::NotFound(id.to_string()))
    }

    async fn uninstall(&self, id: &str) -> BackendResult<()> {
        let path = PathBuf::from(id);
        let home = dirs::home_dir().ok_or_else(|| {
            BackendError::Other("Não foi possível determinar o diretório home".into())
        })?;

        // Only allow removing desktop entries under ~/.local
        let local_apps = home.join(".local/share/applications");
        if path.starts_with(&local_apps) && path.extension().and_then(|e| e.to_str()) == Some("desktop")
        {
            fs::remove_file(&path)?;
            return Ok(());
        }

        Err(BackendError::PermissionDenied(
            "Aplicativos manuais do sistema não podem ser removidos automaticamente. Remova os arquivos manualmente."
                .into(),
        ))
    }
}

fn scan_manual_apps(
    claimed_desktop: &HashSet<String>,
    claimed_exec: &HashSet<String>,
) -> Vec<AppInfo> {
    let mut apps = Vec::new();
    for dir in desktop_dirs() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                continue;
            }
            let file_name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            if claimed_desktop.contains(&file_name) {
                continue;
            }
            // Flatpak / snap desktop ids
            if file_name.contains("flatpak")
                || file_name.starts_with("snap.")
                || file_name.contains(".flatpak.")
            {
                continue;
            }

            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            if !is_user_facing_app(&content) {
                continue;
            }
            if let Some(exec) = desktop_exec(&content) {
                let exec_base = Path::new(&exec)
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                if claimed_exec.contains(&exec) || claimed_exec.contains(&exec_base) {
                    continue;
                }
                if exec.to_lowercase().contains(".appimage") {
                    continue;
                }
                // Skip package-manager wrappers
                if exec.contains("/snap/") || exec.contains("flatpak run") {
                    continue;
                }
            }

            if let Some(app) = parse_desktop_to_app(&path, &content) {
                apps.push(app);
            }
        }
    }
    apps
}

fn desktop_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/share/applications"));
    }
    dirs.push(PathBuf::from("/usr/local/share/applications"));
    // Também /usr/share para capturar itens não reivindicados (vira Manual ou Sistema)
    dirs.push(PathBuf::from("/usr/share/applications"));
    dirs
}

fn is_user_facing_app(content: &str) -> bool {
    if content.contains("NoDisplay=true") || content.contains("Hidden=true") {
        return false;
    }
    if !content.contains("Type=Application") && !content.contains("[Desktop Entry]") {
        return false;
    }
    if content.contains("OnlyShowIn=") && content.contains("OnlyShowIn=;") {
        return false;
    }
    desktop_key(content, "Name").is_some() && desktop_exec(content).is_some()
}

fn desktop_key(content: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}=");
    for line in content.lines() {
        if let Some(v) = line.strip_prefix(&prefix) {
            return Some(v.trim().to_string());
        }
    }
    None
}

fn desktop_exec(content: &str) -> Option<String> {
    let exec = desktop_key(content, "Exec")?;
    let first = exec.split_whitespace().next()?.to_string();
    Some(first)
}

fn parse_desktop_to_app(path: &Path, content: &str) -> Option<AppInfo> {
    let name = desktop_localized(content, "Name")?;
    let package_id = path.to_string_lossy().to_string();
    let mut app = AppInfo::new(InstallMethod::Manual, package_id, name);
    app.description = desktop_localized(content, "Comment");
    if let Some(icon) = desktop_key(content, "Icon") {
        if icon.starts_with('/') {
            app.icon_path = Some(icon);
        } else {
            app.icon_name = Some(icon);
        }
    }
    if let Some(cats) = desktop_key(content, "Categories") {
        app.category = cats
            .split(';')
            .find(|s| !s.is_empty() && *s != "Application")
            .map(|s| s.to_string());
    }
    app.install_path = desktop_exec(content);
    app.origin = Some("Instalação manual / local".into());
    app.desktop_id = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string());
    app.developer = desktop_key(content, "X-Developer")
        .or_else(|| desktop_key(content, "StartupWMClass"));
    Some(app)
}

fn desktop_localized(content: &str, key: &str) -> Option<String> {
    for locale in ["pt_BR", "pt"] {
        let prefix = format!("{key}[{locale}]=");
        for line in content.lines() {
            if let Some(v) = line.strip_prefix(&prefix) {
                return Some(v.trim().to_string());
            }
        }
    }
    desktop_key(content, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn skips_nodisplay() {
        let content = "[Desktop Entry]\nType=Application\nName=Secret\nExec=secret\nNoDisplay=true\n";
        assert!(!is_user_facing_app(content));
    }

    #[tokio::test]
    async fn finds_manual_desktop() {
        let dir = tempfile::tempdir().unwrap();
        let desktop = dir.path().join("MyCoolApp.desktop");
        let mut f = fs::File::create(&desktop).unwrap();
        writeln!(
            f,
            "[Desktop Entry]\nType=Application\nName=My Cool App\nExec=/opt/mycool/app\nComment=Test\nCategories=Utility;\n"
        )
        .unwrap();

        // Point scan only at temp by using claims empty and patching — call parse directly
        let content = fs::read_to_string(&desktop).unwrap();
        let app = parse_desktop_to_app(&desktop, &content).unwrap();
        assert_eq!(app.name, "My Cool App");
        assert_eq!(app.method, InstallMethod::Manual);
    }
}
