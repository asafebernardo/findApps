use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tracing::debug;
use walkdir::WalkDir;

use crate::backends::PackageBackend;
use crate::models::{
    AppInfo, BackendError, BackendResult, BackendStatus, InstallMethod,
};
use crate::system::paths::default_appimage_dirs;
use crate::util::validation::validate_package_id;

pub struct AppImageBackend {
    search_dirs: Vec<PathBuf>,
}

impl AppImageBackend {
    pub fn new() -> Self {
        Self {
            search_dirs: default_appimage_dirs(),
        }
    }

    pub fn with_dirs(dirs: Vec<PathBuf>) -> Self {
        Self { search_dirs: dirs }
    }
}

impl Default for AppImageBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PackageBackend for AppImageBackend {
    fn id(&self) -> InstallMethod {
        InstallMethod::AppImage
    }

    fn is_available(&self) -> bool {
        true // always detectable
    }

    async fn detect(&self) -> BackendStatus {
        BackendStatus::detectable(InstallMethod::AppImage)
    }

    async fn list_installed(&self) -> BackendResult<Vec<AppInfo>> {
        let dirs = self.search_dirs.clone();
        let apps = tokio::task::spawn_blocking(move || scan_appimages(&dirs))
            .await
            .map_err(|e| BackendError::Other(e.to_string()))?;
        debug!(count = apps.len(), "AppImage: aplicativos listados");
        Ok(apps)
    }

    async fn get_details(&self, id: &str) -> BackendResult<AppInfo> {
        validate_package_id(InstallMethod::AppImage, id)?;
        let apps = self.list_installed().await?;
        apps.into_iter()
            .find(|a| a.package_id == id || a.id == id)
            .ok_or_else(|| BackendError::NotFound(id.to_string()))
    }

    async fn uninstall(&self, id: &str) -> BackendResult<()> {
        validate_package_id(InstallMethod::AppImage, id)?;
        let path = PathBuf::from(id);
        if !path.exists() {
            return Err(BackendError::NotFound(id.to_string()));
        }
        // Only remove under home or configured dirs without root
        let home = dirs::home_dir();
        let under_home = home
            .as_ref()
            .map(|h| path.starts_with(h))
            .unwrap_or(false);
        if !under_home {
            return Err(BackendError::PermissionDenied(
                "Remoção de AppImage fora do diretório do usuário requer autorização manual"
                    .into(),
            ));
        }
        fs::remove_file(&path)?;
        // Try remove related desktop entry
        if let Some(home) = home {
            let apps_dir = home.join(".local/share/applications");
            if let Ok(entries) = fs::read_dir(apps_dir) {
                for entry in entries.flatten() {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if content.contains(&path.to_string_lossy().to_string()) {
                            let _ = fs::remove_file(entry.path());
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn scan_appimages(dirs: &[PathBuf]) -> Vec<AppInfo> {
    let mut apps = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // From filesystem scan
    for dir in dirs {
        if !dir.exists() {
            continue;
        }
        let walker = WalkDir::new(dir).max_depth(3).follow_links(false);
        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !is_valid_appimage(path) {
                continue;
            }
            let key = path.to_string_lossy().to_string();
            if !seen.insert(key.clone()) {
                continue;
            }
            apps.push(appinfo_from_path(path));
        }
    }

    // From desktop entries that Exec= an AppImage
    for desktop_dir in desktop_dirs() {
        let Ok(entries) = fs::read_dir(&desktop_dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                continue;
            }
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            if let Some(exec) = extract_appimage_exec(&content) {
                let exec_path = PathBuf::from(&exec);
                if !is_valid_appimage(&exec_path) {
                    continue;
                }
                let key = exec_path.to_string_lossy().to_string();
                if !seen.insert(key) {
                    // enrich existing
                    if let Some(app) = apps.iter_mut().find(|a| a.package_id == exec) {
                        enrich_from_desktop_content(app, &content, &path);
                    }
                    continue;
                }
                let mut app = appinfo_from_path(&exec_path);
                enrich_from_desktop_content(&mut app, &content, &path);
                apps.push(app);
            }
        }
    }

    apps
}

/// Verifica se o arquivo parece um AppImage válido (não qualquer .AppImage).
pub fn is_valid_appimage(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    if !name.ends_with(".appimage") {
        return false;
    }
    if !path.is_file() {
        return false;
    }
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    let mode = meta.permissions().mode();
    if mode & 0o111 == 0 {
        return false;
    }
    let Ok(mut file) = fs::File::open(path) else {
        return false;
    };
    use std::io::Read;
    let mut magic = [0u8; 4];
    if file.read_exact(&mut magic).is_err() {
        return false;
    }
    magic == [0x7f, b'E', b'L', b'F']
}

fn appinfo_from_path(path: &Path) -> AppInfo {
    let file_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("AppImage")
        .to_string();
    let (name, version) = split_name_version(&file_name);
    let mut app = AppInfo::new(
        InstallMethod::AppImage,
        path.to_string_lossy().to_string(),
        name,
    );
    app.version = version;
    app.install_path = Some(path.to_string_lossy().to_string());
    if let Ok(meta) = fs::metadata(path) {
        app.size_bytes = Some(meta.len());
        if let Ok(modified) = meta.modified() {
            app.install_date = system_time_to_utc(modified);
        }
    }
    app.origin = Some("Arquivo local".into());
    app.architecture = Some(std::env::consts::ARCH.to_string());
    app
}

fn split_name_version(stem: &str) -> (String, Option<String>) {
    // e.g. Foo-1.2.3-x86_64
    let cleaned = stem
        .trim_end_matches("-x86_64")
        .trim_end_matches("-aarch64")
        .trim_end_matches("-arm64")
        .trim_end_matches(".AppImage")
        .trim_end_matches(".appimage");
    if let Some((name, ver)) = cleaned.rsplit_once('-') {
        if ver.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            return (name.replace('-', " "), Some(ver.to_string()));
        }
    }
    (cleaned.replace('-', " "), None)
}

fn system_time_to_utc(t: SystemTime) -> Option<DateTime<Utc>> {
    let duration = t.duration_since(SystemTime::UNIX_EPOCH).ok()?;
    DateTime::from_timestamp(duration.as_secs() as i64, duration.subsec_nanos())
}

fn desktop_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/share/applications"));
    }
    dirs.push(PathBuf::from("/usr/share/applications"));
    dirs.push(PathBuf::from("/usr/local/share/applications"));
    dirs
}

fn extract_appimage_exec(content: &str) -> Option<String> {
    for line in content.lines() {
        if let Some(exec) = line.strip_prefix("Exec=") {
            let exec = exec.split_whitespace().next()?;
            if exec.to_lowercase().contains(".appimage") {
                return Some(exec.to_string());
            }
        }
    }
    None
}

fn enrich_from_desktop_content(app: &mut AppInfo, content: &str, desktop_path: &Path) {
    for line in content.lines() {
        if let Some(v) = line.strip_prefix("Name=") {
            app.name = v.to_string();
        } else if let Some(v) = line.strip_prefix("Icon=") {
            if v.starts_with('/') {
                app.icon_path = Some(v.to_string());
            } else {
                app.icon_name = Some(v.to_string());
            }
        } else if let Some(v) = line.strip_prefix("Comment=") {
            app.description = Some(v.to_string());
        } else if let Some(v) = line.strip_prefix("Categories=") {
            app.category = v
                .split(';')
                .find(|s| !s.is_empty())
                .map(|s| s.to_string());
        }
    }
    app.desktop_id = desktop_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    #[test]
    fn rejects_non_elf() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fake.AppImage");
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .mode(0o755)
            .open(&path)
            .unwrap();
        f.write_all(b"not an elf").unwrap();
        assert!(!is_valid_appimage(&path));
    }

    #[test]
    fn accepts_elf_appimage() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Foo-1.0.AppImage");
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .mode(0o755)
            .open(&path)
            .unwrap();
        f.write_all(&[0x7f, b'E', b'L', b'F', 0, 0, 0, 0]).unwrap();
        assert!(is_valid_appimage(&path));
    }

    #[test]
    fn split_version() {
        let (n, v) = split_name_version("Discord-0.0.100-x86_64");
        assert_eq!(n, "Discord");
        assert_eq!(v.as_deref(), Some("0.0.100"));
    }

    #[tokio::test]
    async fn list_from_temp_dir() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("TestApp-2.0.AppImage");
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .mode(0o755)
            .open(&path)
            .unwrap();
        f.write_all(&[0x7f, b'E', b'L', b'F', 2, 1, 1, 0]).unwrap();

        let backend = AppImageBackend::with_dirs(vec![dir.path().to_path_buf()]);
        let apps = backend.list_installed().await.unwrap();
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].method, InstallMethod::AppImage);
    }
}
