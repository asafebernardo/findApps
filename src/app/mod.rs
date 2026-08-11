//! Ciclo de vida da aplicação GTK.

use std::sync::Arc;

use gtk4::prelude::*;
use libadwaita as adw;

use crate::services::{AppConfig, PackageManager};
use crate::ui::window::FindWindow;
use crate::util::icons::{icon_theme_dir, APP_ICON_NAME};

pub struct FindApplication {
    app: adw::Application,
}

impl FindApplication {
    pub fn new() -> Self {
        let app = adw::Application::builder()
            .application_id("br.com.findapps.FindApps")
            .flags(gio::ApplicationFlags::FLAGS_NONE)
            .build();

        app.connect_startup(|_| {
            register_app_icon();
        });

        app.connect_activate(|app| {
            load_css();
            register_app_icon();
            let config = AppConfig::load();
            crate::i18n::init(config.language_enum());
            let manager = Arc::new(tokio::sync::Mutex::new(PackageManager::new(&config)));

            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("tokio runtime");
            let rt = Arc::new(rt);

            let window = FindWindow::new(app, manager, rt);
            window.present();
        });

        Self { app }
    }

    pub fn run(&self) {
        self.app.run();
    }
}

fn register_app_icon() {
    let Some(display) = gtk4::gdk::Display::default() else {
        return;
    };
    let theme = gtk4::IconTheme::for_display(&display);
    let path = icon_theme_dir();
    if path.is_dir() {
        theme.add_search_path(path);
    }
    if let Ok(snap) = std::env::var("SNAP") {
        theme.add_search_path(format!("{snap}/usr/share/icons"));
    }
    // Also try installed system path after package install
    theme.add_search_path("/usr/share/icons");
    let _ = APP_ICON_NAME;
}

fn load_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_string(include_str!("../../resources/style.css"));
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("display"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
