use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use async_trait::async_trait;
use tracing::debug;

use crate::backends::PackageBackend;
use crate::models::{
    AppInfo, BackendError, BackendResult, BackendStatus, InstallMethod,
};
use crate::system::privilege;
use crate::system::process::{command_exists_sync, run_command};
use crate::util::validation::validate_package_id;

pub struct AptBackend;

impl AptBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AptBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PackageBackend for AptBackend {
    fn id(&self) -> InstallMethod {
        InstallMethod::Apt
    }

    fn is_available(&self) -> bool {
        command_exists_sync("dpkg") && command_exists_sync("dpkg-query")
    }

    async fn detect(&self) -> BackendStatus {
        if !self.is_available() {
            return BackendStatus::unavailable(InstallMethod::Apt);
        }
        let version = run_command("dpkg", &["--version"])
            .await
            .ok()
            .and_then(|o| o.lines().next().map(|l| l.to_string()));
        BackendStatus::available(InstallMethod::Apt, version)
    }

    async fn list_installed(&self) -> BackendResult<Vec<AppInfo>> {
        if !self.is_available() {
            return Err(BackendError::Unavailable);
        }

        // Mapa desktop_path -> package (uma única consulta)
        let desktop_map = collect_desktop_package_map().await;
        let desktop_pkgs: HashSet<String> = desktop_map.values().cloned().collect();

        let output = run_command(
            "dpkg-query",
            &[
                "-W",
                "-f=${Package}\t${Version}\t${Architecture}\t${Installed-Size}\t${Status}\n",
            ],
        )
        .await?;

        let mut apps_by_pkg: HashMap<String, AppInfo> = HashMap::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 5 {
                continue;
            }
            let pkg = parts[0];
            let status = parts[4];
            if !status.contains("installed") || status.contains("not-installed") {
                continue;
            }
            if !desktop_pkgs.contains(pkg) && !looks_like_app_package(pkg) {
                continue;
            }
            if is_library_package(pkg) && !desktop_pkgs.contains(pkg) {
                continue;
            }

            let mut app = AppInfo::new(InstallMethod::Apt, pkg, humanize_name(pkg));
            app.version = Some(parts[1].to_string());
            app.architecture = Some(parts[2].to_string());
            if let Ok(kib) = parts[3].parse::<u64>() {
                app.size_bytes = Some(kib * 1024);
            }
            app.origin = Some("Repositório APT".into());
            app.install_path = Some("/usr".into());
            app.icon_name = Some(pkg.to_string());
            apps_by_pkg.insert(pkg.to_string(), app);
        }

        enrich_from_desktop_map(&mut apps_by_pkg, &desktop_map);
        enrich_from_apt_cache(&mut apps_by_pkg).await;

        let apps: Vec<AppInfo> = apps_by_pkg.into_values().collect();
        debug!(count = apps.len(), "APT: aplicativos listados");
        Ok(apps)
    }

    async fn get_details(&self, id: &str) -> BackendResult<AppInfo> {
        validate_package_id(InstallMethod::Apt, id)?;
        let apps = self.list_installed().await?;
        apps.into_iter()
            .find(|a| a.package_id == id)
            .ok_or_else(|| BackendError::NotFound(id.to_string()))
    }

    async fn uninstall(&self, id: &str) -> BackendResult<()> {
        validate_package_id(InstallMethod::Apt, id)?;
        privilege::uninstall_with_privilege(InstallMethod::Apt, id).await
    }
}

/// Uma consulta: `firefox: /usr/share/applications/firefox.desktop`
async fn collect_desktop_package_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Ok(output) = run_command(
        "dpkg-query",
        &["-S", "/usr/share/applications/*.desktop"],
    )
    .await
    else {
        return map;
    };
    for line in output.lines() {
        let Some((pkgs, path)) = line.split_once(':') else {
            continue;
        };
        let path = path.trim().to_string();
        let pkg = pkgs
            .split(',')
            .next()
            .unwrap_or(pkgs)
            .trim()
            .to_string();
        if !pkg.is_empty() && !path.is_empty() {
            map.insert(path, pkg);
        }
    }
    map
}

fn looks_like_app_package(pkg: &str) -> bool {
    const KNOWN: &[&str] = &[
        "firefox", "chromium", "thunderbird", "gimp", "vlc", "libreoffice",
        "code", "discord", "spotify", "steam", "inkscape", "blender",
        "audacity", "obs-studio", "transmission", "filezilla",
    ];
    KNOWN
        .iter()
        .any(|k| pkg == *k || pkg.starts_with(&format!("{k}-")))
}

fn is_library_package(pkg: &str) -> bool {
    pkg.starts_with("lib")
        || pkg.starts_with("python3-")
        || pkg.starts_with("gir1.2-")
        || pkg.ends_with("-dev")
        || pkg.ends_with("-doc")
        || pkg.ends_with("-common")
        || pkg.ends_with("-data")
        || pkg.contains("-dbg")
}

fn humanize_name(pkg: &str) -> String {
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

fn enrich_from_desktop_map(
    apps: &mut HashMap<String, AppInfo>,
    desktop_map: &HashMap<String, String>,
) {
    for (path, pkg) in desktop_map {
        let Some(app) = apps.get_mut(pkg) else {
            continue;
        };
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        if content.contains("NoDisplay=true") || content.contains("Hidden=true") {
            continue;
        }
        if let Some(n) = desktop_localized(&content, "Name") {
            app.name = n;
        }
        if let Some(icon) = desktop_key(&content, "Icon") {
            if icon.starts_with('/') {
                app.icon_path = Some(icon);
            } else {
                app.icon_name = Some(icon);
            }
        }
        if app.description.is_none() {
            app.description = desktop_localized(&content, "Comment");
        }
        if app.category.is_none() {
            app.category = desktop_key(&content, "Categories").map(|c| {
                c.split(';')
                    .find(|s| !s.is_empty() && *s != "Application")
                    .unwrap_or("Aplicativo")
                    .to_string()
            });
        }
        app.desktop_id = Path::new(path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string());
    }
}

/// Preenche descrição/desenvolvedor ausentes via apt-cache (lote limitado).
async fn enrich_from_apt_cache(apps: &mut HashMap<String, AppInfo>) {
    let missing: Vec<String> = apps
        .iter()
        .filter(|(_, a)| a.description.is_none() || a.developer.is_none())
        .map(|(k, _)| k.clone())
        .take(80)
        .collect();

    for pkg in missing {
        let Ok(out) = run_command("apt-cache", &["show", &pkg]).await else {
            continue;
        };
        let Some(app) = apps.get_mut(&pkg) else {
            continue;
        };
        let mut desc_lines = Vec::new();
        let mut in_desc = false;
        for line in out.lines() {
            if let Some(v) = line.strip_prefix("Maintainer: ") {
                if app.developer.is_none() {
                    // "Name <email>" -> Name
                    let name = v.split('<').next().unwrap_or(v).trim();
                    app.developer = Some(name.to_string());
                }
            } else if let Some(v) = line.strip_prefix("Description-pt_BR: ") {
                app.description = Some(v.to_string());
                in_desc = false;
            } else if let Some(v) = line.strip_prefix("Description: ") {
                if app.description.is_none() {
                    desc_lines.clear();
                    desc_lines.push(v.to_string());
                    in_desc = true;
                }
            } else if in_desc {
                if line.starts_with(' ') || line.starts_with('\t') {
                    let t = line.trim();
                    if t != "." {
                        desc_lines.push(t.to_string());
                    }
                } else {
                    in_desc = false;
                }
            } else if let Some(v) = line.strip_prefix("Section: ") {
                if app.category.is_none() {
                    app.category = Some(v.to_string());
                }
            } else if let Some(v) = line.strip_prefix("Homepage: ") {
                if app.origin.as_deref() == Some("Repositório APT") {
                    app.origin = Some(format!("APT · {v}"));
                }
            }
        }
        if app.description.is_none() && !desc_lines.is_empty() {
            app.description = Some(desc_lines.join(" "));
        }
    }
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

    #[test]
    fn library_filter() {
        assert!(is_library_package("libgtk-4-1"));
        assert!(is_library_package("python3-gi"));
        assert!(!is_library_package("firefox"));
    }

    #[tokio::test]
    async fn detect_does_not_panic_when_missing() {
        let backend = AptBackend::new();
        let status = backend.detect().await;
        assert!(status.method == InstallMethod::Apt);
    }
}
