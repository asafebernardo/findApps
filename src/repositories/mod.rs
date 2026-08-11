//! Repositório de aplicativos em memória.

use std::sync::RwLock;

use crate::models::{AppFilter, AppInfo, InstallMethod, SortBy};

pub struct AppRepository {
    apps: RwLock<Vec<AppInfo>>,
}

impl AppRepository {
    pub fn new() -> Self {
        Self {
            apps: RwLock::new(Vec::new()),
        }
    }

    pub fn clear(&self) {
        self.apps.write().unwrap().clear();
    }

    pub fn extend(&self, new_apps: Vec<AppInfo>) {
        let mut apps = self.apps.write().unwrap();
        for app in new_apps {
            if let Some(existing) = apps.iter_mut().find(|a| a.id == app.id) {
                *existing = app;
            } else {
                apps.push(app);
            }
        }
    }

    pub fn remove_by_id(&self, id: &str) {
        self.apps.write().unwrap().retain(|a| a.id != id);
    }

    pub fn all(&self) -> Vec<AppInfo> {
        self.apps.read().unwrap().clone()
    }

    pub fn get(&self, id: &str) -> Option<AppInfo> {
        self.apps
            .read()
            .unwrap()
            .iter()
            .find(|a| a.id == id)
            .cloned()
    }

    pub fn count(&self) -> usize {
        self.apps.read().unwrap().len()
    }

    pub fn count_by_method(&self, method: InstallMethod) -> usize {
        self.apps
            .read()
            .unwrap()
            .iter()
            .filter(|a| a.method == method)
            .count()
    }

    pub fn size_by_method(&self, method: InstallMethod) -> u64 {
        self.apps
            .read()
            .unwrap()
            .iter()
            .filter(|a| a.method == method)
            .filter_map(|a| a.size_bytes)
            .sum()
    }

    pub fn query(&self, filter: &AppFilter, search: &str, sort: SortBy) -> Vec<AppInfo> {
        let mut result: Vec<AppInfo> = self
            .apps
            .read()
            .unwrap()
            .iter()
            .filter(|a| match filter {
                AppFilter::All | AppFilter::Applications => true,
                AppFilter::Method(m) => a.method == *m,
                AppFilter::Home | AppFilter::Settings => false,
            })
            .filter(|a| a.matches_query(search))
            .cloned()
            .collect();

        match sort {
            SortBy::Name => result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
            SortBy::Size => result.sort_by(|a, b| {
                b.size_bytes
                    .unwrap_or(0)
                    .cmp(&a.size_bytes.unwrap_or(0))
            }),
            SortBy::InstallDate => result.sort_by(|a, b| b.install_date.cmp(&a.install_date)),
            SortBy::Method => result.sort_by(|a, b| a.method.cmp(&b.method).then_with(|| a.name.cmp(&b.name))),
            SortBy::UpdateAvailable => result.sort_by(|a, b| {
                b.update_available
                    .is_some()
                    .cmp(&a.update_available.is_some())
                    .then_with(|| a.name.cmp(&b.name))
            }),
        }
        result
    }
}

impl Default for AppRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> AppInfo {
        let mut a = AppInfo::new(InstallMethod::Apt, "firefox", "Firefox");
        a.developer = Some("Mozilla".into());
        a.description = Some("Web browser".into());
        a.size_bytes = Some(1000);
        a
    }

    #[test]
    fn search_by_name_and_developer() {
        let repo = AppRepository::new();
        repo.extend(vec![sample()]);
        assert_eq!(repo.query(&AppFilter::All, "fire", SortBy::Name).len(), 1);
        assert_eq!(repo.query(&AppFilter::All, "mozilla", SortBy::Name).len(), 1);
        assert_eq!(repo.query(&AppFilter::All, "apt", SortBy::Name).len(), 1);
        assert!(repo.query(&AppFilter::All, "zzz", SortBy::Name).is_empty());
    }

    #[test]
    fn filter_by_method() {
        let repo = AppRepository::new();
        repo.extend(vec![
            sample(),
            AppInfo::new(InstallMethod::Flatpak, "org.foo", "Foo"),
        ]);
        assert_eq!(
            repo.query(&AppFilter::Method(InstallMethod::Flatpak), "", SortBy::Name)
                .len(),
            1
        );
    }
}
