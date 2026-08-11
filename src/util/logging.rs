use std::fs;
use std::sync::OnceLock;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

static GUARD: OnceLock<WorkerGuard> = OnceLock::new();

pub fn init_logging() {
    let log_dir = crate::system::paths::log_dir();
    let _ = fs::create_dir_all(&log_dir);

    let file_appender = tracing_appender::rolling::daily(&log_dir, "findapps.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let _ = GUARD.set(guard);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(fmt::layer().with_ansi(false).with_writer(non_blocking))
        .try_init();
}

pub fn read_recent_logs(max_lines: usize) -> String {
    let log_dir = crate::system::paths::log_dir();
    let Ok(entries) = fs::read_dir(&log_dir) else {
        return "Nenhum log disponível.".to_string();
    };

    let mut files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("findapps.log")
        })
        .collect();
    files.sort_by_key(|e| e.file_name());

    let Some(latest) = files.last() else {
        return "Nenhum log disponível.".to_string();
    };

    let Ok(content) = fs::read_to_string(latest.path()) else {
        return "Não foi possível ler os logs.".to_string();
    };

    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(max_lines);
    lines[start..].join("\n")
}
