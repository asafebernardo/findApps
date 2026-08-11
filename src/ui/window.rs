use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, Label, ListBox, Orientation, PolicyType, ScrolledWindow,
    SearchEntry, Spinner, Stack, StackTransitionType,
};
use libadwaita as adw;
use adw::prelude::*;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

use crate::models::{AppFilter, AppInfo, InstallMethod, SortBy};
use crate::services::{PackageManager, ScanEvent, SearchService};
use crate::i18n::{t, tf, Language};
use crate::services::AppConfig;
use crate::ui::dashboard::build_dashboard;
use crate::ui::details::show_details;
use crate::util::format::format_size;
use crate::util::icons::{logo_path, APP_ICON_NAME};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ScanUiStatus {
    Pending,
    Running,
    Done,
    Failed,
    Skipped,
}

pub struct UiState {
    pub filter: AppFilter,
    pub search: String,
    pub sort: SortBy,
    pub apps_cache: Vec<AppInfo>,
    pub scan_status: HashMap<InstallMethod, ScanUiStatus>,
    pub scanning: bool,
    pub usable_methods: Vec<InstallMethod>,
}

pub struct FindWindow {
    window: adw::ApplicationWindow,
}

impl FindWindow {
    pub fn new(
        app: &adw::Application,
        manager: Arc<tokio::sync::Mutex<PackageManager>>,
        rt: Arc<Runtime>,
    ) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("FindApps")
            .default_width(1100)
            .default_height(720)
            .icon_name(APP_ICON_NAME)
            .build();
        window.set_icon_name(Some(APP_ICON_NAME));

        let toast_overlay = adw::ToastOverlay::new();
        let split = adw::NavigationSplitView::new();

        let state = Rc::new(RefCell::new(UiState {
            filter: AppFilter::Home,
            search: String::new(),
            sort: SortBy::Name,
            apps_cache: Vec::new(),
            scan_status: HashMap::new(),
            scanning: true,
            usable_methods: Vec::new(),
        }));

        let content_stack = Stack::new();
        content_stack.set_transition_type(StackTransitionType::Crossfade);
        content_stack.set_hexpand(true);
        content_stack.set_vexpand(true);

        let dashboard_box = GtkBox::new(Orientation::Vertical, 0);
        let list_root = GtkBox::new(Orientation::Vertical, 0);
        let settings_host = GtkBox::new(Orientation::Vertical, 0);
        settings_host.set_vexpand(true);
        settings_host.set_hexpand(true);

        let on_language_changed: Rc<RefCell<Option<Rc<dyn Fn()>>>> =
            Rc::new(RefCell::new(None));

        content_stack.add_named(&dashboard_box, Some("dashboard"));
        content_stack.add_named(&list_root, Some("list"));
        content_stack.add_named(&settings_host, Some("settings"));
        content_stack.set_visible_child_name("dashboard");

        let content_toolbar = adw::ToolbarView::new();
        let content_header = adw::HeaderBar::new();
        let title = Label::new(Some(&t("home")));
        title.add_css_class("heading");
        content_header.set_title_widget(Some(&title));
        content_toolbar.add_top_bar(&content_header);
        content_toolbar.set_content(Some(&content_stack));
        let content_nav = adw::NavigationPage::new(&content_toolbar, "Conteúdo");

        let sidebar_toolbar = adw::ToolbarView::new();
        let sidebar_header = adw::HeaderBar::new();
        sidebar_header.set_show_end_title_buttons(false);
        let brand_box = GtkBox::new(Orientation::Horizontal, 8);
        brand_box.set_halign(Align::Center);
        let brand_icon = app_logo_image(28);
        let brand = Label::new(Some("FindApps"));
        brand.add_css_class("heading");
        brand_box.append(&brand_icon);
        brand_box.append(&brand);
        sidebar_header.set_title_widget(Some(&brand_box));
        sidebar_toolbar.add_top_bar(&sidebar_header);

        let sidebar_list = ListBox::new();
        sidebar_list.add_css_class("navigation-sidebar");
        sidebar_list.set_selection_mode(gtk4::SelectionMode::Single);

        let sidebar_scroll = ScrolledWindow::builder()
            .child(&sidebar_list)
            .hscrollbar_policy(PolicyType::Never)
            .vexpand(true)
            .build();
        sidebar_toolbar.set_content(Some(&sidebar_scroll));
        let sidebar_nav = adw::NavigationPage::new(&sidebar_toolbar, "FindApps");
        sidebar_nav.set_width_request(240);

        split.set_sidebar(Some(&sidebar_nav));
        split.set_content(Some(&content_nav));
        toast_overlay.set_child(Some(&split));
        window.set_content(Some(&toast_overlay));

        // List page
        let search = SearchEntry::new();
        search.set_placeholder_text(Some(&t("search_placeholder")));
        search.set_hexpand(true);

        let sort_label = Label::new(Some(&t("sort")));
        let sort_name = t("sort_name");
        let sort_size = t("sort_size");
        let sort_date = t("sort_date");
        let sort_method = t("sort_method");
        let sort_update = t("sort_update");
        let sort_btn = gtk4::DropDown::from_strings(&[
            sort_name.as_str(),
            sort_size.as_str(),
            sort_date.as_str(),
            sort_method.as_str(),
            sort_update.as_str(),
        ]);

        let list_header = GtkBox::new(Orientation::Horizontal, 8);
        list_header.set_margin_top(8);
        list_header.set_margin_bottom(8);
        list_header.set_margin_start(12);
        list_header.set_margin_end(12);
        list_header.append(&search);
        list_header.append(&sort_label);
        list_header.append(&sort_btn);

        let apps_list = ListBox::new();
        apps_list.add_css_class("boxed-list");
        apps_list.set_selection_mode(gtk4::SelectionMode::None);
        apps_list.set_margin_start(12);
        apps_list.set_margin_end(12);
        apps_list.set_margin_bottom(12);

        let list_scroll = ScrolledWindow::builder()
            .child(&apps_list)
            .vexpand(true)
            .hscrollbar_policy(PolicyType::Never)
            .build();

        let empty_page = adw::StatusPage::builder()
            .icon_name("system-search-symbolic")
            .title(&t("no_apps"))
            .description(&t("no_apps_filter"))
            .vexpand(true)
            .hexpand(true)
            .build();

        let list_stack = Stack::new();
        list_stack.set_vexpand(true);
        list_stack.set_hexpand(true);
        list_stack.set_transition_type(StackTransitionType::Crossfade);
        list_stack.add_named(&list_scroll, Some("apps"));
        list_stack.add_named(&empty_page, Some("empty"));
        list_stack.set_visible_child_name("apps");

        let status_bar = GtkBox::new(Orientation::Horizontal, 8);
        status_bar.set_margin_start(12);
        status_bar.set_margin_end(12);
        status_bar.set_margin_bottom(8);
        let spinner = Spinner::new();
        spinner.start();
        let status_label = Label::new(Some(&t("searching_apps")));
        status_label.set_halign(Align::Start);
        status_label.set_hexpand(true);
        status_bar.append(&spinner);
        status_bar.append(&status_label);

        list_root.append(&status_bar);
        list_root.append(&list_header);
        list_root.append(&list_stack);

        let refresh_list = {
            let apps_list = apps_list.clone();
            let list_stack = list_stack.clone();
            let empty_page = empty_page.clone();
            let state = state.clone();
            let manager = manager.clone();
            let rt = rt.clone();
            let toast_overlay = toast_overlay.clone();
            let window = window.clone();
            // Placeholders filled after both refresh closures exist — use RefCell trick
            let refresh_dashboard_slot: Rc<RefCell<Option<Rc<dyn Fn()>>>> =
                Rc::new(RefCell::new(None));
            let refresh_list_slot: Rc<RefCell<Option<Rc<dyn Fn()>>>> =
                Rc::new(RefCell::new(None));
            let refresh_dashboard_slot_c = refresh_dashboard_slot.clone();
            let refresh_list_slot_c = refresh_list_slot.clone();
            let refresh_fn: Rc<dyn Fn()> = Rc::new(move || {
                while let Some(child) = apps_list.first_child() {
                    apps_list.remove(&child);
                }
                let st = state.borrow();
                let filter = match &st.filter {
                    AppFilter::Home | AppFilter::Settings => AppFilter::All,
                    other => other.clone(),
                };
                let empty_title = match &filter {
                    AppFilter::Method(m) => tf("no_apps_method", &[("method", &m.as_str())]),
                    _ => t("no_apps"),
                };
                let apps = SearchService::search(
                    &{
                        let repo = crate::repositories::AppRepository::new();
                        repo.extend(st.apps_cache.clone());
                        repo
                    },
                    &st.search,
                    &filter,
                    st.sort,
                );
                let has_query = !st.search.is_empty();
                drop(st);

                if apps.is_empty() {
                    let description = if has_query {
                        t("no_apps_search")
                    } else {
                        t("no_apps_filter")
                    };
                    empty_page.set_title(&empty_title);
                    empty_page.set_description(Some(description.as_str()));
                    list_stack.set_visible_child_name("empty");
                    return;
                }

                list_stack.set_visible_child_name("apps");
                let refresh_list = refresh_list_slot_c
                    .borrow()
                    .clone()
                    .unwrap_or_else(|| Rc::new(|| {}) as Rc<dyn Fn()>);
                let refresh_dashboard = refresh_dashboard_slot_c
                    .borrow()
                    .clone()
                    .unwrap_or_else(|| Rc::new(|| {}) as Rc<dyn Fn()>);
                for app in apps {
                    let row = build_app_row(
                        &app,
                        manager.clone(),
                        rt.clone(),
                        toast_overlay.clone(),
                        window.clone(),
                        state.clone(),
                        refresh_list.clone(),
                        refresh_dashboard.clone(),
                    );
                    apps_list.append(&row);
                }
            });
            *refresh_list_slot.borrow_mut() = Some(refresh_fn.clone());
            (refresh_fn, refresh_dashboard_slot, refresh_list_slot)
        };
        let (refresh_list, refresh_dashboard_slot, _refresh_list_slot) = refresh_list;

        let refresh_dashboard = {
            let dashboard_box = dashboard_box.clone();
            let state = state.clone();
            let f: Rc<dyn Fn()> = Rc::new(move || {
                while let Some(child) = dashboard_box.first_child() {
                    dashboard_box.remove(&child);
                }
                let st = state.borrow();
                let page = build_dashboard(&st.apps_cache, &st.scan_status, st.scanning);
                dashboard_box.append(&page);
            });
            *refresh_dashboard_slot.borrow_mut() = Some(f.clone());
            f
        };

        let rebuild_sidebar = {
            let sidebar_list = sidebar_list.clone();
            let state = state.clone();
            let content_stack = content_stack.clone();
            let title = title.clone();
            let refresh_list = refresh_list.clone();
            let refresh_dashboard = refresh_dashboard.clone();
            Rc::new(move || {
                let current_filter = state.borrow().filter.clone();
                while let Some(child) = sidebar_list.first_child() {
                    sidebar_list.remove(&child);
                }
                let methods = state.borrow().usable_methods.clone();
                let mut items: Vec<(AppFilter, &'static str, String)> = vec![
                    (AppFilter::Home, "user-home-symbolic", t("home")),
                    (AppFilter::All, "view-grid-symbolic", t("all")),
                ];
                for m in &methods {
                    if *m == InstallMethod::System {
                        continue;
                    }
                    items.push((AppFilter::Method(*m), m.icon_name(), m.as_str()));
                }
                items.push((
                    AppFilter::Method(InstallMethod::System),
                    "computer-symbolic",
                    t("system"),
                ));
                items.push((
                    AppFilter::Settings,
                    "emblem-system-symbolic",
                    t("settings"),
                ));

                let mut select_idx: i32 = 0;
                for (i, (filter, icon, label)) in items.into_iter().enumerate() {
                    if filter == current_filter {
                        select_idx = i as i32;
                    }
                    let row = adw::ActionRow::builder().title(&label).build();
                    row.add_prefix(&gtk4::Image::from_icon_name(icon));
                    row.set_activatable(true);
                    let f = filter.clone();
                    let state = state.clone();
                    let content_stack = content_stack.clone();
                    let title = title.clone();
                    let refresh_list = refresh_list.clone();
                    let refresh_dashboard = refresh_dashboard.clone();
                    row.connect_activated(move |_| {
                        state.borrow_mut().filter = f.clone();
                        match &f {
                            AppFilter::Home => {
                                title.set_text(&t("home"));
                                content_stack.set_visible_child_name("dashboard");
                                refresh_dashboard();
                            }
                            AppFilter::Settings => {
                                title.set_text(&t("settings"));
                                content_stack.set_visible_child_name("settings");
                            }
                            other => {
                                title.set_text(&other.title());
                                content_stack.set_visible_child_name("list");
                                refresh_list();
                            }
                        }
                    });
                    sidebar_list.append(&row);
                }
                if let Some(row) = sidebar_list.row_at_index(select_idx) {
                    sidebar_list.select_row(Some(&row));
                }
            })
        };

        // Live language switch: refresh all visible UI without restarting.
        let apply_ui_language = {
            let search = search.clone();
            let sort_label = sort_label.clone();
            let sort_btn = sort_btn.clone();
            let status_label = status_label.clone();
            let empty_page = empty_page.clone();
            let title = title.clone();
            let state = state.clone();
            let rebuild_sidebar = rebuild_sidebar.clone();
            let refresh_list = refresh_list.clone();
            let refresh_dashboard = refresh_dashboard.clone();
            let settings_host = settings_host.clone();
            let toast_overlay = toast_overlay.clone();
            let on_language_changed = on_language_changed.clone();
            Rc::new(move || {
                search.set_placeholder_text(Some(&t("search_placeholder")));
                sort_label.set_text(&t("sort"));

                let sort_name = t("sort_name");
                let sort_size = t("sort_size");
                let sort_date = t("sort_date");
                let sort_method = t("sort_method");
                let sort_update = t("sort_update");
                let selected = sort_btn.selected();
                let model = gtk4::StringList::new(&[
                    sort_name.as_str(),
                    sort_size.as_str(),
                    sort_date.as_str(),
                    sort_method.as_str(),
                    sort_update.as_str(),
                ]);
                sort_btn.set_model(Some(&model));
                sort_btn.set_selected(selected);

                {
                    let st = state.borrow();
                    if st.scanning {
                        status_label.set_text(&t("searching_apps"));
                    } else {
                        status_label.set_text(&tf(
                            "apps_found",
                            &[("n", &st.apps_cache.len().to_string())],
                        ));
                    }
                    let title_text = match &st.filter {
                        AppFilter::Home => t("home"),
                        AppFilter::Settings => t("settings"),
                        other => other.title(),
                    };
                    title.set_text(&title_text);
                }

                empty_page.set_title(&t("no_apps"));
                empty_page.set_description(Some(&t("no_apps_filter")));

                rebuild_sidebar();
                refresh_list();
                refresh_dashboard();

                while let Some(child) = settings_host.first_child() {
                    settings_host.remove(&child);
                }
                let page = build_settings(toast_overlay.clone(), on_language_changed.clone());
                settings_host.append(&page);
            })
        };
        *on_language_changed.borrow_mut() = Some(apply_ui_language.clone());
        // Initial settings page
        settings_host.append(&build_settings(
            toast_overlay.clone(),
            on_language_changed.clone(),
        ));

        {
            let state = state.clone();
            let refresh_list = refresh_list.clone();
            search.connect_search_changed(move |entry| {
                state.borrow_mut().search = entry.text().to_string();
                refresh_list();
            });
        }
        {
            let state = state.clone();
            let refresh_list = refresh_list.clone();
            sort_btn.connect_selected_notify(move |dropdown| {
                state.borrow_mut().sort = match dropdown.selected() {
                    1 => SortBy::Size,
                    2 => SortBy::InstallDate,
                    3 => SortBy::Method,
                    4 => SortBy::UpdateAvailable,
                    _ => SortBy::Name,
                };
                refresh_list();
            });
        }

        // Bridge tokio → glib via async-channel (Send) + spawn_future_local
        let (event_tx, event_rx) = async_channel::unbounded::<ScanEvent>();

        {
            let manager = manager.clone();
            let event_tx = event_tx.clone();
            rt.spawn(async move {
                {
                    let mut mg = manager.lock().await;
                    let _ = mg.detect_all().await;
                }
                let (tx, mut rx) = mpsc::unbounded_channel::<ScanEvent>();
                let manager2 = manager.clone();
                let scan_task = tokio::spawn(async move {
                    let mg = manager2.lock().await;
                    mg.scan(tx).await;
                });
                while let Some(ev) = rx.recv().await {
                    if event_tx.send(ev).await.is_err() {
                        break;
                    }
                }
                let _ = scan_task.await;
            });
        }

        {
            let state = state.clone();
            let refresh_list = refresh_list.clone();
            let refresh_dashboard = refresh_dashboard.clone();
            let rebuild_sidebar = rebuild_sidebar.clone();
            let spinner = spinner.clone();
            let status_label = status_label.clone();
            let toast_overlay = toast_overlay.clone();
            let manager = manager.clone();
            let rt = rt.clone();

            glib::spawn_future_local(async move {
                // Initial usable methods
                {
                    let (tx, rx) = async_channel::bounded(1);
                    let manager = manager.clone();
                    rt.spawn(async move {
                        let mg = manager.lock().await;
                        let usable = mg.usable_methods();
                        let statuses = mg.statuses.clone();
                        let _ = tx.send((usable, statuses)).await;
                    });
                    if let Ok((usable, statuses)) = rx.recv().await {
                        let mut st = state.borrow_mut();
                        st.usable_methods = usable;
                        for s in &statuses {
                            st.scan_status.insert(s.method, ScanUiStatus::Pending);
                        }
                        drop(st);
                        rebuild_sidebar();
                        refresh_dashboard();
                    }
                }

                while let Ok(ev) = event_rx.recv().await {
                    match ev {
                        ScanEvent::BackendStarted(method) => {
                            state
                                .borrow_mut()
                                .scan_status
                                .insert(method, ScanUiStatus::Running);
                            status_label.set_text(&tf(
                                "searching_method",
                                &[("method", &method.as_str())],
                            ));
                            refresh_dashboard();
                        }
                        ScanEvent::BackendFinished {
                            method,
                            status,
                            count,
                            error,
                        } => {
                            let ui = if error.is_some() {
                                ScanUiStatus::Failed
                            } else if status.is_usable() {
                                ScanUiStatus::Done
                            } else {
                                ScanUiStatus::Skipped
                            };
                            state.borrow_mut().scan_status.insert(method, ui);
                            if status.is_usable()
                                && !state.borrow().usable_methods.contains(&method)
                            {
                                state.borrow_mut().usable_methods.push(method);
                                rebuild_sidebar();
                            }
                            status_label.set_text(&tf(
                                "method_count",
                                &[
                                    ("method", &method.as_str()),
                                    ("n", &count.to_string()),
                                ],
                            ));
                            refresh_dashboard();
                        }
                        ScanEvent::AppsFound(apps) => {
                            {
                                let mut st = state.borrow_mut();
                                st.apps_cache.extend(apps);
                                let mut seen = HashSet::new();
                                st.apps_cache.retain(|a| seen.insert(a.id.clone()));
                            }
                            refresh_list();
                            refresh_dashboard();
                        }
                        ScanEvent::FinalApps(apps) => {
                            state.borrow_mut().apps_cache = apps;
                            // Garantir Sistema na sidebar após classificação
                            if !state
                                .borrow()
                                .usable_methods
                                .contains(&InstallMethod::System)
                            {
                                state.borrow_mut().usable_methods.push(InstallMethod::System);
                                rebuild_sidebar();
                            }
                            refresh_list();
                            refresh_dashboard();
                        }
                        ScanEvent::Completed => {
                            state.borrow_mut().scanning = false;
                            spinner.stop();
                            spinner.set_visible(false);
                            let n = state.borrow().apps_cache.len();
                            let n_str = n.to_string();
                            status_label.set_text(&tf("apps_found", &[("n", &n_str)]));
                            toast_overlay.add_toast(adw::Toast::new(&tf(
                                "scan_done_toast",
                                &[("n", &n_str)],
                            )));
                            refresh_dashboard();
                            refresh_list();
                        }
                    }
                }
            });
        }

        rebuild_sidebar();
        refresh_dashboard();

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn build_app_row(
    app: &AppInfo,
    manager: Arc<tokio::sync::Mutex<PackageManager>>,
    rt: Arc<Runtime>,
    toast: adw::ToastOverlay,
    window: adw::ApplicationWindow,
    state: Rc<RefCell<UiState>>,
    refresh_list: Rc<dyn Fn()>,
    refresh_dashboard: Rc<dyn Fn()>,
) -> adw::ActionRow {
    let subtitle = format!(
        "{} · {} · {}",
        app.developer.as_deref().unwrap_or("—"),
        app.method.as_str(),
        app.version.as_deref().unwrap_or("—")
    );
    let row = adw::ActionRow::builder()
        .title(&app.name)
        .subtitle(&subtitle)
        .activatable(true)
        .build();

    let icon = app_icon_image(app);
    icon.set_pixel_size(40);
    row.add_prefix(&icon);

    let method_label = app.method.as_str();
    let method = Label::new(Some(&method_label));
    method.add_css_class(app.method.css_class());
    method.add_css_class("caption");
    row.add_suffix(&method);

    if let Some(size) = app.size_bytes {
        let size_l = Label::new(Some(&format_size(size)));
        size_l.add_css_class("dim-label");
        row.add_suffix(&size_l);
    }

    let details_btn = Button::from_icon_name("info-outline-symbolic");
    details_btn.set_tooltip_text(Some(&t("details")));
    details_btn.add_css_class("flat");

    let open = {
        let app = app.clone();
        let window = window.clone();
        let manager = manager.clone();
        let rt = rt.clone();
        let toast = toast.clone();
        let state = state.clone();
        let refresh_list = refresh_list.clone();
        let refresh_dashboard = refresh_dashboard.clone();
        move || {
            show_details(
                &window,
                &app,
                manager.clone(),
                rt.clone(),
                toast.clone(),
                state.clone(),
                refresh_list.clone(),
                refresh_dashboard.clone(),
            );
        }
    };

    {
        let open = open.clone();
        details_btn.connect_clicked(move |_| open());
    }
    {
        let open = open.clone();
        row.connect_activated(move |_| open());
    }
    row.add_suffix(&details_btn);
    row
}

fn app_logo_image(pixel_size: i32) -> gtk4::Image {
    let path = logo_path();
    let image = if path.is_file() {
        gtk4::Image::from_file(&path)
    } else {
        gtk4::Image::from_icon_name(APP_ICON_NAME)
    };
    image.set_pixel_size(pixel_size);
    image
}

fn app_icon_image(app: &AppInfo) -> gtk4::Image {
    if let Some(path) = &app.icon_path {
        if std::path::Path::new(path).is_file() {
            return gtk4::Image::from_file(path);
        }
    }
    if let Some(name) = &app.icon_name {
        let img = gtk4::Image::from_icon_name(name);
        // Se o tema não tiver o ícone, GTK mostra placeholder — ok
        return img;
    }
    gtk4::Image::from_icon_name("application-x-executable")
}

fn build_settings(
    toast: adw::ToastOverlay,
    on_language_changed: Rc<RefCell<Option<Rc<dyn Fn()>>>>,
) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_vexpand(true);
    root.set_hexpand(true);

    let content = GtkBox::new(Orientation::Vertical, 18);
    content.set_margin_top(20);
    content.set_margin_bottom(24);
    content.set_margin_start(20);
    content.set_margin_end(20);
    content.set_vexpand(true);

    // Language
    let lang_group = adw::PreferencesGroup::new();
    lang_group.set_title(&t("language"));
    lang_group.set_description(Some(&t("language_desc")));

    let langs = Language::all();
    let names: Vec<&str> = langs.iter().map(|l| l.native_name()).collect();
    let lang_dropdown = gtk4::DropDown::from_strings(&names);
    let current = crate::i18n::current();
    if let Some(idx) = langs.iter().position(|l| *l == current) {
        lang_dropdown.set_selected(idx as u32);
    }
    let lang_row = adw::ActionRow::builder()
        .title(&t("language"))
        .activatable(false)
        .build();
    lang_row.add_suffix(&lang_dropdown);
    lang_group.add(&lang_row);
    content.append(&lang_group);

    {
        let toast = toast.clone();
        let on_language_changed = on_language_changed.clone();
        // Avoid re-entrancy when rebuilding settings after language change
        let applying = Rc::new(RefCell::new(false));
        let applying_c = applying.clone();
        lang_dropdown.connect_selected_notify(move |dropdown| {
            if *applying_c.borrow() {
                return;
            }
            let idx = dropdown.selected() as usize;
            let Some(lang) = Language::all().get(idx).copied() else {
                return;
            };
            if lang == crate::i18n::current() {
                return;
            }
            crate::i18n::set_language(lang);
            let mut cfg = AppConfig::load();
            cfg.language = lang.code().to_string();
            let _ = cfg.save();

            *applying_c.borrow_mut() = true;
            if let Some(refresh) = on_language_changed.borrow().clone() {
                refresh();
            }
            *applying_c.borrow_mut() = false;
            toast.add_toast(adw::Toast::new(&t("language_updated")));
        });
    }

    // About
    let about_group = adw::PreferencesGroup::new();
    about_group.set_title(&t("about"));
    about_group.set_description(Some(&t("about_desc")));

    let version_row = adw::ActionRow::builder()
        .title(&t("version"))
        .subtitle(env!("CARGO_PKG_VERSION"))
        .build();
    about_group.add(&version_row);

    let id_row = adw::ActionRow::builder()
        .title(&t("identifier"))
        .subtitle("br.com.findapps.FindApps")
        .build();
    about_group.add(&id_row);

    let license_row = adw::ActionRow::builder()
        .title(&t("license"))
        .subtitle("GPL-3.0-or-later")
        .build();
    about_group.add(&license_row);
    content.append(&about_group);

    let about_fill = GtkBox::new(Orientation::Vertical, 12);
    about_fill.set_vexpand(true);
    about_fill.set_valign(Align::Fill);
    about_fill.set_halign(Align::Fill);
    about_fill.add_css_class("card");
    about_fill.set_margin_top(4);

    let about_inner = GtkBox::new(Orientation::Vertical, 10);
    about_inner.set_margin_top(28);
    about_inner.set_margin_bottom(28);
    about_inner.set_margin_start(28);
    about_inner.set_margin_end(28);
    about_inner.set_vexpand(true);
    about_inner.set_valign(Align::Center);
    about_inner.set_halign(Align::Center);

    let icon = app_logo_image(64);
    icon.set_halign(Align::Center);
    about_inner.append(&icon);

    let title = Label::new(Some("FindApps"));
    title.add_css_class("title-1");
    title.set_halign(Align::Center);
    about_inner.append(&title);

    let tagline = Label::new(Some(&t("tagline")));
    tagline.add_css_class("heading");
    tagline.set_halign(Align::Center);
    about_inner.append(&tagline);

    let description = Label::new(Some(&t("about_body")));
    description.set_wrap(true);
    description.set_justify(gtk4::Justification::Center);
    description.set_max_width_chars(56);
    description.add_css_class("dim-label");
    description.set_halign(Align::Center);
    about_inner.append(&description);

    let features = Label::new(Some(&t("about_features")));
    features.add_css_class("caption");
    features.set_halign(Align::Center);
    features.set_margin_top(8);
    about_inner.append(&features);

    about_fill.append(&about_inner);
    content.append(&about_fill);

    let scroll = ScrolledWindow::builder()
        .child(&content)
        .vexpand(true)
        .hexpand(true)
        .hscrollbar_policy(PolicyType::Never)
        .build();
    root.append(&scroll);
    root
}
