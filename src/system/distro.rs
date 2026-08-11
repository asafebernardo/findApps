use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistroFamily {
    Debian,
    Fedora,
    Arch,
    Suse,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistroInfo {
    pub id: String,
    pub id_like: Vec<String>,
    pub name: String,
    pub version_id: Option<String>,
    pub family: DistroFamily,
}

impl DistroInfo {
    pub fn detect() -> Self {
        Self::from_os_release(PathBuf::from("/etc/os-release"))
            .unwrap_or_else(|| Self {
                id: "unknown".into(),
                id_like: vec![],
                name: "Linux".into(),
                version_id: None,
                family: DistroFamily::Other,
            })
    }

    pub fn from_os_release(path: PathBuf) -> Option<Self> {
        let content = fs::read_to_string(path).ok()?;
        let mut id = String::new();
        let mut id_like = Vec::new();
        let mut name = String::from("Linux");
        let mut version_id = None;

        for line in content.lines() {
            let line = line.trim();
            if let Some(v) = line.strip_prefix("ID=") {
                id = unquote(v);
            } else if let Some(v) = line.strip_prefix("ID_LIKE=") {
                id_like = unquote(v)
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
            } else if let Some(v) = line.strip_prefix("NAME=") {
                name = unquote(v);
            } else if let Some(v) = line.strip_prefix("VERSION_ID=") {
                version_id = Some(unquote(v));
            }
        }

        let family = classify(&id, &id_like);
        Some(Self {
            id,
            id_like,
            name,
            version_id,
            family,
        })
    }

    pub fn prefers_apt(&self) -> bool {
        matches!(self.family, DistroFamily::Debian)
    }

    pub fn prefers_dnf(&self) -> bool {
        matches!(self.family, DistroFamily::Fedora)
    }
}

fn unquote(s: &str) -> String {
    s.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

fn classify(id: &str, id_like: &[String]) -> DistroFamily {
    let all: Vec<&str> = std::iter::once(id)
        .chain(id_like.iter().map(|s| s.as_str()))
        .collect();

    for token in &all {
        match *token {
            "debian" | "ubuntu" | "linuxmint" | "pop" | "elementary" | "zorin" | "raspbian"
            | "kali" | "mx" => return DistroFamily::Debian,
            "fedora" | "rhel" | "centos" | "rocky" | "alma" | "nobara" => {
                return DistroFamily::Fedora
            }
            "arch" | "manjaro" | "endeavouros" | "garuda" => return DistroFamily::Arch,
            "suse" | "opensuse" | "opensuse-tumbleweed" | "opensuse-leap" => {
                return DistroFamily::Suse
            }
            _ => {}
        }
    }
    DistroFamily::Other
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn detects_ubuntu() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("os-release");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(
            f,
            r#"NAME="Ubuntu"
ID=ubuntu
ID_LIKE=debian
VERSION_ID="24.04""#
        )
        .unwrap();
        let info = DistroInfo::from_os_release(path).unwrap();
        assert_eq!(info.family, DistroFamily::Debian);
        assert!(info.prefers_apt());
    }

    #[test]
    fn detects_fedora() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("os-release");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(
            f,
            r#"NAME="Fedora Linux"
ID=fedora
VERSION_ID=40"#
        )
        .unwrap();
        let info = DistroInfo::from_os_release(path).unwrap();
        assert_eq!(info.family, DistroFamily::Fedora);
        assert!(info.prefers_dnf());
    }
}
