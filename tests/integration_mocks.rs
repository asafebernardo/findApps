//! Testes de integração com mocks (sem alterar o sistema).

use std::sync::Arc;

use findapps::backends::mock::MockBackend;
use findapps::backends::PackageBackend;
use findapps::models::{AppFilter, AppInfo, BackendError, InstallMethod, SortBy};
use findapps::repositories::AppRepository;
use findapps::services::SearchService;
use findapps::system::distro::{DistroFamily, DistroInfo};
use findapps::util::validation::validate_package_id;

fn sample_apps() -> Vec<AppInfo> {
    let mut firefox = AppInfo::new(InstallMethod::Apt, "firefox", "Firefox");
    firefox.developer = Some("Mozilla".into());
    firefox.description = Some("Navegador web".into());
    firefox.version = Some("141.0".into());
    firefox.size_bytes = Some(245 * 1024 * 1024);

    let mut discord = AppInfo::new(InstallMethod::Flatpak, "com.discordapp.Discord", "Discord");
    discord.developer = Some("Discord Inc.".into());
    discord.version = Some("0.0.100".into());

    let mut vlc = AppInfo::new(InstallMethod::Snap, "vlc", "VLC");
    vlc.developer = Some("VideoLAN".into());

    vec![firefox, discord, vlc]
}

#[tokio::test]
async fn mock_detect_available_and_unavailable() {
    let ok = MockBackend::new(InstallMethod::Apt, true);
    let status = ok.detect().await;
    assert!(status.is_usable());

    let missing = MockBackend::new(InstallMethod::Dnf, false);
    let status = missing.detect().await;
    assert!(!status.is_usable());
    assert!(!missing.is_available());
}

#[tokio::test]
async fn mock_list_and_get_details() {
    let backend = MockBackend::with_apps(InstallMethod::Apt, sample_apps());
    let apps = backend.list_installed().await.unwrap();
    assert_eq!(apps.len(), 3);

    let details = backend.get_details("firefox").await.unwrap();
    assert_eq!(details.name, "Firefox");
}

#[tokio::test]
async fn mock_uninstall_and_permission_error() {
    let backend = MockBackend::with_apps(
        InstallMethod::Apt,
        vec![AppInfo::new(InstallMethod::Apt, "firefox", "Firefox")],
    );
    backend.uninstall("firefox").await.unwrap();
    assert!(backend.list_installed().await.unwrap().is_empty());

    let denied = MockBackend::with_apps(
        InstallMethod::Apt,
        vec![AppInfo::new(InstallMethod::Apt, "vim", "Vim")],
    )
    .with_permission_denied();
    let err = denied.uninstall("vim").await.unwrap_err();
    assert!(matches!(err, BackendError::PermissionDenied(_)));
}

#[tokio::test]
async fn missing_backend_does_not_panic() {
    let backend = MockBackend::new(InstallMethod::Snap, false);
    let err = backend.list_installed().await.unwrap_err();
    assert!(matches!(err, BackendError::Unavailable));
}

#[test]
fn search_and_filters() {
    let repo = AppRepository::new();
    repo.extend(sample_apps());

    let found = SearchService::search(&repo, "firefox", &AppFilter::All, SortBy::Name);
    assert_eq!(found.len(), 1);

    let mozilla = SearchService::search(&repo, "mozilla", &AppFilter::All, SortBy::Name);
    assert_eq!(mozilla.len(), 1);

    let apt = SearchService::search(
        &repo,
        "",
        &AppFilter::Method(InstallMethod::Apt),
        SortBy::Name,
    );
    assert_eq!(apt.len(), 1);

    let flatpak = SearchService::search(
        &repo,
        "discord",
        &AppFilter::Method(InstallMethod::Flatpak),
        SortBy::Name,
    );
    assert_eq!(flatpak.len(), 1);
}

#[test]
fn sort_by_size() {
    let repo = AppRepository::new();
    repo.extend(sample_apps());
    let sorted = repo.query(&AppFilter::All, "", SortBy::Size);
    assert_eq!(sorted[0].package_id, "firefox");
}

#[test]
fn package_id_validation_blocks_injection() {
    assert!(validate_package_id(InstallMethod::Apt, "firefox").is_ok());
    assert!(validate_package_id(InstallMethod::Apt, "firefox;rm -rf /").is_err());
    assert!(validate_package_id(InstallMethod::Flatpak, "org.mozilla.firefox").is_ok());
}

#[test]
fn distro_classification() {
    use std::fs;
    use std::io::Write;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("os-release");
    let mut f = fs::File::create(&path).unwrap();
    writeln!(f, "ID=ubuntu\nID_LIKE=debian\nNAME=Ubuntu").unwrap();
    let info = DistroInfo::from_os_release(path).unwrap();
    assert_eq!(info.family, DistroFamily::Debian);
}

#[tokio::test]
async fn install_and_update_are_stubbed() {
    let backend = MockBackend::new(InstallMethod::Apt, true);
    let err = backend.install("firefox").await.unwrap_err();
    assert!(matches!(err, BackendError::Unsupported(_)));
    let err = backend.update("firefox").await.unwrap_err();
    assert!(matches!(err, BackendError::Unsupported(_)));
}

#[tokio::test]
async fn backend_identification() {
    let backends: Vec<Arc<dyn PackageBackend>> = vec![
        Arc::new(MockBackend::new(InstallMethod::Apt, true)),
        Arc::new(MockBackend::new(InstallMethod::Flatpak, true)),
        Arc::new(MockBackend::new(InstallMethod::Snap, false)),
    ];
    let ids: Vec<_> = backends.iter().map(|b| b.id()).collect();
    assert_eq!(
        ids,
        vec![
            InstallMethod::Apt,
            InstallMethod::Flatpak,
            InstallMethod::Snap
        ]
    );
    assert!(backends[0].is_available());
    assert!(!backends[2].is_available());
}
