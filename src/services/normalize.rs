//! Normalização: ícones, deduplicação e classificação Sistema.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::models::{AppInfo, InstallMethod};

/// Chave de deduplicação (nome normalizado).
pub fn normalize_key(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// Resolve caminho de ícone a partir de nome ou path.
pub fn resolve_icon_path(icon_name: &Option<String>, icon_path: &Option<String>) -> Option<String> {
    if let Some(path) = icon_path {
        if Path::new(path).is_file() {
            return Some(path.clone());
        }
    }
    let Some(name) = icon_name else {
        return None;
    };
    if name.starts_with('/') {
        if Path::new(name).is_file() {
            return Some(name.clone());
        }
        return None;
    }
    // Remove extensão se houver
    let base = name
        .trim_end_matches(".png")
        .trim_end_matches(".svg")
        .trim_end_matches(".xpm");

    for candidate in icon_search_paths(base) {
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().to_string());
        }
    }
    None
}

fn icon_search_paths(base: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let sizes = ["256x256", "128x128", "96x96", "64x64", "48x48", "32x32", "scalable"];
    let themes = ["hicolor", "Adwaita", "Yaru", "Papirus", "breeze"];
    let exts = ["png", "svg", "xpm"];

    for theme in themes {
        for size in sizes {
            for ext in exts {
                paths.push(PathBuf::from(format!(
                    "/usr/share/icons/{theme}/{size}/apps/{base}.{ext}"
                )));
                paths.push(PathBuf::from(format!(
                    "/usr/share/icons/{theme}/{size}/legacy/{base}.{ext}"
                )));
            }
        }
    }
    for ext in exts {
        paths.push(PathBuf::from(format!("/usr/share/pixmaps/{base}.{ext}")));
        paths.push(PathBuf::from(format!(
            "/usr/share/icons/hicolor/scalable/apps/{base}.{ext}"
        )));
    }
    if let Some(home) = dirs::home_dir() {
        for ext in exts {
            paths.push(home.join(format!(".local/share/icons/hicolor/48x48/apps/{base}.{ext}")));
            paths.push(home.join(format!(".local/share/icons/{base}.{ext}")));
            paths.push(home.join(format!(
                ".local/share/flatpak/exports/share/icons/hicolor/128x128/apps/{base}.{ext}"
            )));
            paths.push(home.join(format!(
                ".local/share/flatpak/exports/share/icons/hicolor/scalable/apps/{base}.{ext}"
            )));
        }
    }
    // Flatpak system exports
    for ext in exts {
        paths.push(PathBuf::from(format!(
            "/var/lib/flatpak/exports/share/icons/hicolor/128x128/apps/{base}.{ext}"
        )));
        paths.push(PathBuf::from(format!(
            "/var/lib/flatpak/exports/share/icons/hicolor/scalable/apps/{base}.{ext}"
        )));
    }
    paths
}

/// Enriquece apps com caminhos de ícone resolvidos.
pub fn enrich_icons(apps: &mut [AppInfo]) {
    for app in apps.iter_mut() {
        if app.icon_path.as_ref().is_some_and(|p| Path::new(p).is_file()) {
            continue;
        }
        // Flatpak / Snap: id do app costuma ser o nome do ícone
        if app.icon_name.is_none() {
            match app.method {
                InstallMethod::Flatpak | InstallMethod::Snap => {
                    app.icon_name = Some(app.package_id.clone());
                }
                _ => {
                    if let Some(desktop) = &app.desktop_id {
                        let stem = desktop.trim_end_matches(".desktop");
                        app.icon_name = Some(stem.to_string());
                    } else {
                        app.icon_name = Some(app.package_id.clone());
                    }
                }
            }
        }
        if let Some(path) = resolve_icon_path(&app.icon_name, &app.icon_path) {
            app.icon_path = Some(path);
        }
    }
}

fn is_system_category(category: &Option<String>) -> bool {
    let Some(cat) = category else {
        return false;
    };
    let lower = cat.to_lowercase();
    lower.contains("system")
        || lower.contains("settings")
        || lower.contains("desktopsettings")
        || lower.contains("x-gnome-settings")
}

/// Classifica apps de sistema (Settings/System) para o filtro Sistema.
pub fn classify_system_apps(apps: &mut Vec<AppInfo>) {
    for app in apps.iter_mut() {
        if app.method == InstallMethod::System {
            continue;
        }
        // Não mover Flatpak/Snap/AppImage — só APT/DNF/Manual com cara de sistema
        if !matches!(
            app.method,
            InstallMethod::Apt | InstallMethod::Dnf | InstallMethod::Manual
        ) {
            continue;
        }
        if is_system_category(&app.category) {
            app.method = InstallMethod::System;
            app.id = format!("system:{}", app.package_id);
            if app.origin.is_none() {
                app.origin = Some("Componente do sistema".into());
            }
        }
    }
}

/// Remove duplicatas entre backends (prioriza Flatpak > Snap > APT/DNF > AppImage > Manual).
pub fn deduplicate_apps(apps: Vec<AppInfo>) -> Vec<AppInfo> {
    fn priority(method: InstallMethod) -> u8 {
        match method {
            InstallMethod::Flatpak => 0,
            InstallMethod::Snap => 1,
            InstallMethod::Apt | InstallMethod::Dnf => 2,
            InstallMethod::AppImage => 3,
            InstallMethod::Manual => 4,
            InstallMethod::System => 5,
        }
    }

    let mut by_key: HashMap<String, AppInfo> = HashMap::new();
    let mut by_desktop: HashMap<String, String> = HashMap::new();

    for app in apps {
        // Dedup por desktop_id
        if let Some(desktop) = &app.desktop_id {
            if let Some(existing_id) = by_desktop.get(desktop) {
                if let Some(existing) = by_key.values().find(|a| &a.id == existing_id) {
                    if priority(app.method) >= priority(existing.method) {
                        continue;
                    }
                }
                // Remove inferior
                by_key.retain(|_, a| &a.id != existing_id);
            }
            by_desktop.insert(desktop.clone(), app.id.clone());
        }

        let key = normalize_key(&app.name);
        if key.len() < 2 {
            by_key.insert(app.id.clone(), app);
            continue;
        }

        match by_key.get(&key) {
            Some(existing) if priority(app.method) >= priority(existing.method) => {
                // Keep existing; maybe enrich missing fields
                continue;
            }
            Some(existing) => {
                let mut merged = app;
                if merged.icon_name.is_none() {
                    merged.icon_name = existing.icon_name.clone();
                }
                if merged.icon_path.is_none() {
                    merged.icon_path = existing.icon_path.clone();
                }
                if merged.description.is_none() {
                    merged.description = existing.description.clone();
                }
                by_key.insert(key, merged);
            }
            None => {
                by_key.insert(key, app);
            }
        }
    }

    let mut out: Vec<AppInfo> = by_key.into_values().collect();
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

/// Pipeline completo pós-scan.
pub fn finalize_apps(mut apps: Vec<AppInfo>) -> Vec<AppInfo> {
    enrich_icons(&mut apps);
    classify_system_apps(&mut apps);
    let mut apps = deduplicate_apps(apps);
    enrich_icons(&mut apps);
    apps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedup_prefers_flatpak() {
        let mut apt = AppInfo::new(InstallMethod::Apt, "firefox", "Firefox");
        apt.desktop_id = Some("firefox.desktop".into());
        let mut flat = AppInfo::new(InstallMethod::Flatpak, "org.mozilla.firefox", "Firefox");
        flat.desktop_id = Some("firefox.desktop".into());
        let result = deduplicate_apps(vec![apt, flat]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].method, InstallMethod::Flatpak);
    }

    #[test]
    fn classify_settings_as_system() {
        let mut app = AppInfo::new(InstallMethod::Apt, "gnome-control-center", "Configurações");
        app.category = Some("Settings".into());
        let mut apps = vec![app];
        classify_system_apps(&mut apps);
        assert_eq!(apps[0].method, InstallMethod::System);
    }

    #[test]
    fn normalize_strips_noise() {
        assert_eq!(normalize_key("Fire-fox!"), "firefox");
    }
}
