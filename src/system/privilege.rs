use tracing::info;

use crate::models::{BackendError, BackendResult, InstallMethod};
use crate::system::process::run_command;
use crate::util::validation::validate_package_id;

/// Eleva privilégios apenas para a operação de desinstalação via pkexec.
pub async fn uninstall_with_privilege(
    method: InstallMethod,
    package_id: &str,
) -> BackendResult<()> {
    validate_package_id(method, package_id)?;

    let (program, args): (&str, Vec<&str>) = match method {
        InstallMethod::Apt => ("apt-get", vec!["remove", "--purge", "-y", package_id]),
        InstallMethod::Dnf => ("dnf", vec!["remove", "-y", package_id]),
        InstallMethod::Snap => ("snap", vec!["remove", package_id]),
        InstallMethod::Flatpak => {
            return uninstall_flatpak(package_id).await;
        }
        InstallMethod::AppImage | InstallMethod::Manual => {
            return Err(BackendError::Unsupported(
                "Use the specific backend to remove local files".into(),
            ));
        }
        InstallMethod::System => {
            return Err(BackendError::Unsupported(
                "System components cannot be removed by FindApps".into(),
            ));
        }
    };

    info!(%method, package_id, "requesting elevation via pkexec");
    let mut pk_args = vec![program];
    pk_args.extend(args);
    run_command("pkexec", &pk_args).await?;
    Ok(())
}

async fn uninstall_flatpak(ref_id: &str) -> BackendResult<()> {
    match run_command("flatpak", &["uninstall", "-y", "--user", ref_id]).await {
        Ok(_) => return Ok(()),
        Err(e) => tracing::debug!("flatpak user uninstall failed: {e}"),
    }
    run_command("pkexec", &["flatpak", "uninstall", "-y", "--system", ref_id]).await?;
    Ok(())
}

/// Human-readable uninstall description (shown inside localized dialog).
pub fn describe_uninstall(method: InstallMethod, package_id: &str, name: &str) -> String {
    match method {
        InstallMethod::Apt => format!(
            "APT (admin authorization):\napt-get remove --purge -y {package_id}\n\nApp: {name}"
        ),
        InstallMethod::Dnf => format!(
            "DNF (admin authorization):\ndnf remove -y {package_id}\n\nApp: {name}"
        ),
        InstallMethod::Snap => format!(
            "Snap (admin authorization):\nsnap remove {package_id}\n\nApp: {name}"
        ),
        InstallMethod::Flatpak => {
            format!("Flatpak:\nflatpak uninstall -y {package_id}\n\nApp: {name}")
        }
        InstallMethod::AppImage => {
            format!("The AppImage file will be removed:\n{package_id}\n\nApp: {name}")
        }
        InstallMethod::Manual => {
            format!("User-local app files will be removed if possible.\n\nApp: {name}")
        }
        InstallMethod::System => "System components cannot be uninstalled by FindApps.".into(),
    }
}
