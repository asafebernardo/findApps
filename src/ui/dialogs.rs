use gtk4::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use gtk4::glib::object::IsA;

use crate::i18n::{t, tf};

pub fn show_text_dialog(title: &str, body: &str) {
    let dialog = adw::AlertDialog::builder()
        .heading(title)
        .body(body)
        .build();
    dialog.add_response("ok", &t("close"));
    dialog.set_default_response(Some("ok"));

    let app = gtk4::Application::default();
    if let Some(win) = app.active_window() {
        dialog.present(Some(&win));
        return;
    }
    tracing::info!(%title, "dialog without active window");
}

pub fn confirm_uninstall<F>(parent: &impl IsA<gtk4::Widget>, name: &str, body: &str, on_confirm: F)
where
    F: Fn() + 'static,
{
    let dialog = adw::AlertDialog::builder()
        .heading(&tf("uninstall_confirm_title", &[("name", name)]))
        .body(body)
        .build();
    dialog.add_response("cancel", &t("cancel"));
    dialog.add_response("uninstall", &t("uninstall"));
    dialog.set_response_appearance("uninstall", adw::ResponseAppearance::Destructive);
    dialog.set_default_response(Some("cancel"));
    dialog.set_close_response("cancel");

    dialog.connect_response(None, move |_, response| {
        if response == "uninstall" {
            on_confirm();
        }
    });
    dialog.present(Some(parent));
}
