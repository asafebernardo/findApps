use std::process::Stdio;

use tokio::process::Command;
use tracing::debug;

use crate::models::{BackendError, BackendResult};

/// Executa um comando com argumentos separados (sem shell).
pub async fn run_command(program: &str, args: &[&str]) -> BackendResult<String> {
    debug!(program, ?args, "executando comando");
    let output = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| BackendError::CommandFailed(format!("falha ao iniciar {program}: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = output.status.code().unwrap_or(-1);
        if code == 126 || code == 127 {
            return Err(BackendError::Unavailable);
        }
        // pkexec cancel / auth failure
        if stderr.contains("not authorized")
            || stderr.contains("Authorization required")
            || stderr.contains("polkit")
            || code == 126
        {
            return Err(BackendError::PermissionDenied(stderr.trim().to_string()));
        }
        return Err(BackendError::CommandFailed(format!(
            "{program} saiu com código {code}: {}",
            stderr.trim()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub async fn command_exists(program: &str) -> bool {
    Command::new("which")
        .arg(program)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn command_exists_sync(program: &str) -> bool {
    std::process::Command::new("which")
        .arg(program)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
