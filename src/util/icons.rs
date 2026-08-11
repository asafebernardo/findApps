use std::path::PathBuf;

fn snap_root() -> Option<PathBuf> {
    std::env::var_os("SNAP").map(PathBuf::from)
}

/// Directory containing `hicolor/.../apps/br.com.findapps.FindApps.png`.
pub fn icon_theme_dir() -> PathBuf {
    if let Some(snap) = snap_root() {
        let snap_icons = snap.join("usr/share/icons");
        if snap_icons.is_dir() {
            return snap_icons;
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/icons")
}

/// High-resolution logo for About / branding.
pub fn logo_path() -> PathBuf {
    let mut candidates = Vec::new();
    if let Some(snap) = snap_root() {
        candidates.push(snap.join("usr/share/findapps/br.com.findapps.FindApps.png"));
        candidates.push(
            snap.join("usr/share/icons/hicolor/256x256/apps/br.com.findapps.FindApps.png"),
        );
        candidates.push(
            snap.join("usr/share/icons/hicolor/128x128/apps/br.com.findapps.FindApps.png"),
        );
    }
    candidates.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources/icons/br.com.findapps.FindApps.png"),
    );
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/icons/findapps.png"));
    candidates.push(PathBuf::from("/usr/share/findapps/br.com.findapps.FindApps.png"));
    candidates.push(PathBuf::from(
        "/usr/share/icons/hicolor/256x256/apps/br.com.findapps.FindApps.png",
    ));
    candidates.push(PathBuf::from(
        "/usr/share/icons/hicolor/128x128/apps/br.com.findapps.FindApps.png",
    ));

    candidates
        .into_iter()
        .find(|p| p.is_file())
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("resources/icons/br.com.findapps.FindApps.png")
        })
}

pub const APP_ICON_NAME: &str = "br.com.findapps.FindApps";
