use crate::models::{AppFilter, AppInfo, SortBy};
use crate::repositories::AppRepository;

pub struct SearchService;

impl SearchService {
    pub fn search(repo: &AppRepository, query: &str, filter: &AppFilter, sort: SortBy) -> Vec<AppInfo> {
        repo.query(filter, query, sort)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::InstallMethod;

    #[test]
    fn finds_partial_names() {
        let repo = AppRepository::new();
        repo.extend(vec![
            AppInfo::new(InstallMethod::Apt, "firefox", "Firefox"),
            AppInfo::new(InstallMethod::Apt, "firefox-esr", "Firefox ESR"),
            AppInfo::new(InstallMethod::Flatpak, "org.mozilla.firefox", "Mozilla Firefox"),
        ]);
        let results = SearchService::search(&repo, "firefox", &AppFilter::All, SortBy::Name);
        assert_eq!(results.len(), 3);
    }
}
