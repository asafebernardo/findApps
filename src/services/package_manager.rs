use std::collections::HashSet;
use std::sync::Arc;

use futures::future::join_all;
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::backends::appimage::AppImageBackend;
use crate::backends::apt::AptBackend;
use crate::backends::dnf::DnfBackend;
use crate::backends::flatpak::FlatpakBackend;
use crate::backends::manual::ManualBackend;
use crate::backends::snap::SnapBackend;
use crate::backends::PackageBackend;
use crate::models::{
    AppInfo, BackendError, BackendResult, BackendStatus, InstallMethod,
};
use crate::repositories::AppRepository;
use crate::services::normalize::finalize_apps;
use crate::services::AppConfig;
use crate::system::distro::DistroInfo;
use crate::system::privilege;

#[derive(Debug, Clone)]
pub enum ScanEvent {
    BackendStarted(InstallMethod),
    BackendFinished {
        method: InstallMethod,
        status: BackendStatus,
        count: usize,
        error: Option<String>,
    },
    AppsFound(Vec<AppInfo>),
    /// Lista final após deduplicação / classificação / ícones.
    FinalApps(Vec<AppInfo>),
    Completed,
}

pub struct PackageManager {
    backends: Vec<Arc<dyn PackageBackend>>,
    pub repo: Arc<AppRepository>,
    pub distro: DistroInfo,
    pub statuses: Vec<BackendStatus>,
}

impl PackageManager {
    pub fn new(config: &AppConfig) -> Self {
        let distro = DistroInfo::detect();
        let mut backends: Vec<Arc<dyn PackageBackend>> = Vec::new();

        let apt = AptBackend::new();
        let dnf = DnfBackend::new();
        let flatpak = FlatpakBackend::new();
        let snap = SnapBackend::new();
        let appimage = AppImageBackend::with_dirs(config.appimage_dirs.clone());
        let manual = ManualBackend::new();

        if distro.prefers_dnf() {
            backends.push(Arc::new(dnf));
            backends.push(Arc::new(apt));
        } else {
            backends.push(Arc::new(apt));
            backends.push(Arc::new(dnf));
        }
        backends.push(Arc::new(flatpak));
        backends.push(Arc::new(snap));
        backends.push(Arc::new(appimage));
        backends.push(Arc::new(manual));

        Self {
            backends,
            repo: Arc::new(AppRepository::new()),
            distro,
            statuses: Vec::new(),
        }
    }

    pub fn usable_methods(&self) -> Vec<InstallMethod> {
        let mut methods: Vec<InstallMethod> = self
            .statuses
            .iter()
            .filter(|s| s.is_usable())
            .map(|s| s.method)
            .filter(|m| *m != InstallMethod::System)
            .collect();
        // Sistema sempre visível na sidebar
        if !methods.contains(&InstallMethod::System) {
            methods.push(InstallMethod::System);
        }
        methods
    }

    pub async fn detect_all(&mut self) -> Vec<BackendStatus> {
        let futs: Vec<_> = self
            .backends
            .iter()
            .map(|backend| {
                let backend = Arc::clone(backend);
                async move {
                    let status = backend.detect().await;
                    info!(
                        method = %status.method,
                        usable = status.is_usable(),
                        "{}",
                        status.message
                    );
                    status
                }
            })
            .collect();
        let statuses = join_all(futs).await;
        self.statuses = statuses.clone();
        statuses
    }

    pub async fn scan(&self, tx: mpsc::UnboundedSender<ScanEvent>) {
        self.repo.clear();

        let parallel_backends: Vec<Arc<dyn PackageBackend>> = self
            .backends
            .iter()
            .filter(|b| b.id() != InstallMethod::Manual)
            .cloned()
            .collect();

        let mut tasks = Vec::new();
        for backend in parallel_backends {
            let tx = tx.clone();
            tasks.push(tokio::spawn(async move {
                let method = backend.id();
                let _ = tx.send(ScanEvent::BackendStarted(method));
                let status = backend.detect().await;
                if !status.is_usable() {
                    let _ = tx.send(ScanEvent::BackendFinished {
                        method,
                        status: status.clone(),
                        count: 0,
                        error: None,
                    });
                    return (method, status, Vec::new());
                }

                match backend.list_installed().await {
                    Ok(apps) => {
                        let count = apps.len();
                        let _ = tx.send(ScanEvent::AppsFound(apps.clone()));
                        let _ = tx.send(ScanEvent::BackendFinished {
                            method,
                            status: status.clone(),
                            count,
                            error: None,
                        });
                        (method, status, apps)
                    }
                    Err(e) => {
                        warn!(%method, error = %e, "falha ao listar");
                        let _ = tx.send(ScanEvent::BackendFinished {
                            method,
                            status: status.clone(),
                            count: 0,
                            error: Some(e.to_string()),
                        });
                        (method, status, Vec::new())
                    }
                }
            }));
        }

        let mut all_apps = Vec::new();
        let mut claimed_desktop = HashSet::new();
        let mut claimed_exec = HashSet::new();

        for result in join_all(tasks).await {
            match result {
                Ok((_method, _status, apps)) => {
                    for app in &apps {
                        if let Some(d) = &app.desktop_id {
                            claimed_desktop.insert(d.clone());
                        }
                        claimed_exec.insert(app.package_id.clone());
                        if let Some(p) = &app.install_path {
                            claimed_exec.insert(p.clone());
                        }
                        if let Some(icon) = &app.icon_name {
                            claimed_exec.insert(icon.clone());
                        }
                    }
                    all_apps.extend(apps);
                }
                Err(e) => warn!(error = %e, "task de backend falhou"),
            }
        }

        // Manual depende das claims dos outros backends
        let _ = tx.send(ScanEvent::BackendStarted(InstallMethod::Manual));
        let status = BackendStatus::detectable(InstallMethod::Manual);
        let manual = ManualBackend::with_claims(claimed_desktop, claimed_exec);
        match manual.list_installed().await {
            Ok(apps) => {
                let count = apps.len();
                let _ = tx.send(ScanEvent::AppsFound(apps.clone()));
                let _ = tx.send(ScanEvent::BackendFinished {
                    method: InstallMethod::Manual,
                    status,
                    count,
                    error: None,
                });
                all_apps.extend(apps);
            }
            Err(e) => {
                let _ = tx.send(ScanEvent::BackendFinished {
                    method: InstallMethod::Manual,
                    status,
                    count: 0,
                    error: Some(e.to_string()),
                });
            }
        }

        let final_apps = finalize_apps(all_apps);
        info!(count = final_apps.len(), "varredura normalizada");
        self.repo.clear();
        self.repo.extend(final_apps.clone());
        let _ = tx.send(ScanEvent::FinalApps(final_apps));
        let _ = tx.send(ScanEvent::Completed);
    }

    pub async fn uninstall(&self, app: &AppInfo) -> BackendResult<()> {
        if app.method == InstallMethod::System {
            for method in [
                InstallMethod::Apt,
                InstallMethod::Dnf,
                InstallMethod::Manual,
            ] {
                let Some(backend) = self.backends.iter().find(|b| b.id() == method) else {
                    continue;
                };
                if method != InstallMethod::Manual && !backend.is_available() {
                    continue;
                }
                match backend.uninstall(&app.package_id).await {
                    Ok(()) => {
                        self.repo.remove_by_id(&app.id);
                        return Ok(());
                    }
                    Err(BackendError::Unsupported(_))
                    | Err(BackendError::Unavailable)
                    | Err(BackendError::NotFound(_)) => continue,
                    Err(e) => return Err(e),
                }
            }
            return Err(BackendError::Unsupported(
                "Não foi possível remover este componente do sistema pelo FindApps".into(),
            ));
        }

        let backend = self
            .backends
            .iter()
            .find(|b| b.id() == app.method)
            .ok_or(BackendError::Unavailable)?;
        backend.uninstall(&app.package_id).await?;
        self.repo.remove_by_id(&app.id);
        Ok(())
    }

    pub fn describe_uninstall(app: &AppInfo) -> String {
        privilege::describe_uninstall(app.method, &app.package_id, &app.name)
    }
}
