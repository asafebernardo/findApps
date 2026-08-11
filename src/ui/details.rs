use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, Orientation, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;
use tokio::runtime::Runtime;

use crate::i18n::{t, tf};
use crate::models::AppInfo;
use crate::services::PackageManager;
use crate::system::privilege::describe_uninstall;
use crate::ui::dialogs;
use crate::ui::window::UiState;
use crate::util::format::format_size;

pub fn show_details(
    parent: &adw::ApplicationWindow,
    app: &AppInfo,
    manager: Arc<tokio::sync::Mutex<PackageManager>>,
    rt: Arc<Runtime>,
    toast: adw::ToastOverlay,
    state: Rc<RefCell<UiState>>,
    refresh_list: Rc<dyn Fn()>,
    refresh_dashboard: Rc<dyn Fn()>,
) {
    let dialog = adw::Dialog::new();
    dialog.set_title(&app.name);
    dialog.set_content_width(480);

    let toolbar = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    toolbar.add_top_bar(&header);

    let root = GtkBox::new(Orientation::Vertical, 12);
    root.set_margin_top(12);
    root.set_margin_bottom(18);
    root.set_margin_start(18);
    root.set_margin_end(18);

    let header_box = GtkBox::new(Orientation::Horizontal, 14);
    let icon = if let Some(path) = &app.icon_path {
        if std::path::Path::new(path).is_file() {
            gtk4::Image::from_file(path)
        } else {
            gtk4::Image::from_icon_name(
                app.icon_name
                    .as_deref()
                    .unwrap_or("application-x-executable"),
            )
        }
    } else {
        gtk4::Image::from_icon_name(
            app.icon_name
                .as_deref()
                .unwrap_or("application-x-executable"),
        )
    };
    icon.set_pixel_size(64);
    header_box.append(&icon);

    let titles = GtkBox::new(Orientation::Vertical, 4);
    titles.set_valign(Align::Center);
    let title = Label::new(Some(&app.name));
    title.add_css_class("title-1");
    title.set_halign(Align::Start);
    titles.append(&title);

    if let Some(dev) = &app.developer {
        let d = Label::new(Some(dev));
        d.add_css_class("dim-label");
        d.set_halign(Align::Start);
        titles.append(&d);
    }
    header_box.append(&titles);
    root.append(&header_box);

    let info = adw::PreferencesGroup::new();
    info.set_title(&t("information"));
    add_row(&info, &t("version"), app.version.as_deref().unwrap_or("—"));
    add_row(&info, &t("type"), &t("type_app"));
    add_row(
        &info,
        &t("architecture"),
        app.architecture.as_deref().unwrap_or("—"),
    );
    add_row(&info, &t("status"), &app.status.as_str());
    if let Some(cat) = &app.category {
        add_row(&info, &t("category"), cat);
    }
    if let Some(desc) = &app.description {
        add_row(&info, &t("description"), desc);
    }
    root.append(&info);

    let install = adw::PreferencesGroup::new();
    install.set_title(&t("installation"));
    add_row(&install, &t("method"), &app.method.as_str());
    add_row(
        &install,
        &t("origin"),
        app.origin.as_deref().unwrap_or("—"),
    );
    add_row(
        &install,
        &t("location"),
        app.install_path.as_deref().unwrap_or("—"),
    );
    add_row(
        &install,
        &t("size"),
        &app.size_bytes
            .map(format_size)
            .unwrap_or_else(|| "—".into()),
    );
    if let Some(date) = &app.install_date {
        add_row(
            &install,
            &t("date"),
            &date.format("%Y-%m-%d %H:%M").to_string(),
        );
    }
    add_row(&install, &t("package"), &app.package_id);
    root.append(&install);

    let uninstall_btn = gtk4::Button::with_label(&t("uninstall"));
    uninstall_btn.add_css_class("destructive-action");
    uninstall_btn.add_css_class("pill");
    uninstall_btn.set_halign(Align::Center);
    uninstall_btn.set_margin_top(8);

    let app_c = app.clone();
    let manager_c = manager.clone();
    let rt_c = rt.clone();
    let toast_c = toast.clone();
    let state_c = state.clone();
    let dialog_c = dialog.clone();
    let parent_c = parent.clone();
    let refresh_list_c = refresh_list.clone();
    let refresh_dashboard_c = refresh_dashboard.clone();

    uninstall_btn.connect_clicked(move |_| {
        let description = describe_uninstall(app_c.method, &app_c.package_id, &app_c.name);
        let body = tf(
            "uninstall_confirm_body",
            &[
                ("name", &app_c.name),
                ("method", &app_c.method.as_str()),
                ("details", &description),
            ],
        );

        let app_name = app_c.name.clone();
        let app_for_cb = app_c.clone();
        let manager_c = manager_c.clone();
        let rt_c = rt_c.clone();
        let toast_c = toast_c.clone();
        let state_c = state_c.clone();
        let dialog_c = dialog_c.clone();
        let refresh_list_c = refresh_list_c.clone();
        let refresh_dashboard_c = refresh_dashboard_c.clone();

        dialogs::confirm_uninstall(&parent_c, &app_name, &body, move || {
            let app = app_for_cb.clone();
            let manager = manager_c.clone();
            let toast = toast_c.clone();
            let state = state_c.clone();
            let dialog = dialog_c.clone();
            let refresh_list = refresh_list_c.clone();
            let refresh_dashboard = refresh_dashboard_c.clone();
            let (tx, rx) = async_channel::bounded::<Result<(String, String), String>>(1);
            rt_c.spawn(async move {
                let mg = manager.lock().await;
                let result = match mg.uninstall(&app).await {
                    Ok(()) => Ok((app.id.clone(), app.name.clone())),
                    Err(e) => Err(e.to_string()),
                };
                let _ = tx.send(result).await;
            });
            glib::spawn_future_local(async move {
                match rx.recv().await {
                    Ok(Ok((id, name))) => {
                        state.borrow_mut().apps_cache.retain(|a| a.id != id);
                        refresh_list();
                        refresh_dashboard();
                        toast.add_toast(adw::Toast::new(&tf(
                            "uninstalled",
                            &[("name", &name)],
                        )));
                        dialog.close();
                    }
                    Ok(Err(e)) => {
                        toast.add_toast(adw::Toast::new(&tf("failure", &[("error", &e)])));
                    }
                    Err(_) => {
                        toast.add_toast(adw::Toast::new(&t("interrupted")));
                    }
                }
            });
        });
    });

    root.append(&uninstall_btn);

    let scroll = ScrolledWindow::builder()
        .child(&root)
        .propagate_natural_height(true)
        .build();
    toolbar.set_content(Some(&scroll));
    dialog.set_child(Some(&toolbar));
    dialog.present(Some(parent));
}

fn add_row(group: &adw::PreferencesGroup, title: &str, subtitle: &str) {
    let row = adw::ActionRow::builder()
        .title(title)
        .subtitle(subtitle)
        .build();
    group.add(&row);
}
