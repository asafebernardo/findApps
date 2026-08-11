use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, Orientation, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;

use crate::i18n::{t, tf};
use crate::models::{AppInfo, InstallMethod};
use crate::ui::window::ScanUiStatus;
use crate::util::format::format_size;
use std::collections::HashMap;

pub fn build_dashboard(
    apps: &[AppInfo],
    scan_status: &HashMap<InstallMethod, ScanUiStatus>,
    scanning: bool,
) -> ScrolledWindow {
    let root = GtkBox::new(Orientation::Vertical, 16);
    root.set_margin_top(24);
    root.set_margin_bottom(24);
    root.set_margin_start(24);
    root.set_margin_end(24);

    let heading = Label::new(Some(&t("apps_installed")));
    heading.add_css_class("title-1");
    heading.set_halign(Align::Start);
    root.append(&heading);

    let total = Label::new(Some(&tf(
        "apps_count",
        &[("n", &apps.len().to_string())],
    )));
    total.add_css_class("dashboard-stat");
    total.set_halign(Align::Start);
    root.append(&total);

    if scanning {
        let scan_group = adw::PreferencesGroup::new();
        scan_group.set_title(&t("searching_apps"));
        for method in [
            InstallMethod::Apt,
            InstallMethod::Dnf,
            InstallMethod::Flatpak,
            InstallMethod::Snap,
            InstallMethod::AppImage,
            InstallMethod::Manual,
        ] {
            let status = scan_status
                .get(&method)
                .copied()
                .unwrap_or(ScanUiStatus::Pending);
            let (mark, css) = match status {
                ScanUiStatus::Done => ("✓", "scan-ok"),
                ScanUiStatus::Running => ("⏳", "scan-pending"),
                ScanUiStatus::Failed => ("✗", "scan-fail"),
                ScanUiStatus::Skipped => ("✗", "dim-label"),
                ScanUiStatus::Pending => ("…", "dim-label"),
            };
            let subtitle = match status {
                ScanUiStatus::Done => t("done"),
                ScanUiStatus::Running => t("in_progress"),
                ScanUiStatus::Failed => t("error"),
                ScanUiStatus::Skipped => t("unavailable"),
                ScanUiStatus::Pending => t("pending"),
            };
            let row = adw::ActionRow::builder()
                .title(method.as_str())
                .subtitle(&subtitle)
                .build();
            let badge = Label::new(Some(mark));
            badge.add_css_class(css);
            row.add_prefix(&badge);
            scan_group.add(&row);
        }
        root.append(&scan_group);
    }

    let counts = adw::PreferencesGroup::new();
    counts.set_title(&t("by_method"));
    for method in InstallMethod::all() {
        if *method == InstallMethod::System {
            continue;
        }
        let n = apps.iter().filter(|a| a.method == *method).count();
        if n == 0 && !scan_status.get(method).is_some_and(|s| *s == ScanUiStatus::Done) {
            continue;
        }
        let row = adw::ActionRow::builder()
            .title(method.as_str())
            .subtitle(&tf("apps_count_paren", &[("n", &n.to_string())]))
            .build();
        let badge = Label::new(Some(&method.as_str()));
        badge.add_css_class(method.css_class());
        row.add_suffix(&badge);
        counts.add(&row);
    }
    root.append(&counts);

    let sizes = adw::PreferencesGroup::new();
    sizes.set_title(&t("space_used"));
    for method in InstallMethod::all() {
        if *method == InstallMethod::System {
            continue;
        }
        let bytes: u64 = apps
            .iter()
            .filter(|a| a.method == *method)
            .filter_map(|a| a.size_bytes)
            .sum();
        if bytes == 0 {
            continue;
        }
        let row = adw::ActionRow::builder()
            .title(method.as_str())
            .subtitle(&format_size(bytes))
            .build();
        sizes.add(&row);
    }
    root.append(&sizes);

    ScrolledWindow::builder()
        .child(&root)
        .vexpand(true)
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .build()
}
