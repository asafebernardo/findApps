use regex::Regex;
use once_cell::sync::Lazy;

use crate::models::{BackendError, BackendResult, InstallMethod};

static SAFE_ID: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[A-Za-z0-9][A-Za-z0-9._+:@/-]{0,255}$").expect("regex")
});

/// Valida identificadores de pacote antes de qualquer operação.
pub fn validate_package_id(method: InstallMethod, id: &str) -> BackendResult<()> {
    if id.is_empty() || id.len() > 256 {
        return Err(BackendError::InvalidPackageId(id.to_string()));
    }
    if id.contains('\0') || id.contains(' ') || id.contains(';') || id.contains('|') || id.contains('&')
        || id.contains('`') || id.contains('$') || id.contains('\n') || id.contains('\r')
    {
        return Err(BackendError::InvalidPackageId(id.to_string()));
    }
    if !SAFE_ID.is_match(id) {
        return Err(BackendError::InvalidPackageId(id.to_string()));
    }

    match method {
        InstallMethod::Flatpak => {
            // app id style: org.mozilla.firefox
            if !id.contains('.') && !id.contains('/') {
                // allow short names too, but prefer dotted
            }
        }
        InstallMethod::AppImage => {
            if !std::path::Path::new(id).is_absolute() {
                return Err(BackendError::InvalidPackageId(
                    "AppImage requer caminho absoluto".into(),
                ));
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_apt_id() {
        assert!(validate_package_id(InstallMethod::Apt, "firefox").is_ok());
        assert!(validate_package_id(InstallMethod::Apt, "libgtk-4-1").is_ok());
    }

    #[test]
    fn rejects_injection() {
        assert!(validate_package_id(InstallMethod::Apt, "firefox; rm -rf /").is_err());
        assert!(validate_package_id(InstallMethod::Apt, "foo && bar").is_err());
        assert!(validate_package_id(InstallMethod::Apt, "$(whoami)").is_err());
    }

    #[test]
    fn accepts_flatpak_ref() {
        assert!(validate_package_id(InstallMethod::Flatpak, "org.mozilla.firefox").is_ok());
    }
}
