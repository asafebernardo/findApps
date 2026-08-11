//! Internationalization for FindApps.
//! Default language: English.
//! Supported: English, Chinese, Spanish, Hindi, Arabic, Portuguese, Russian.

use std::collections::HashMap;
use std::sync::RwLock;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

static CURRENT: Lazy<RwLock<Language>> = Lazy::new(|| RwLock::new(Language::English));
static STRINGS: Lazy<HashMap<Language, HashMap<&'static str, &'static str>>> =
    Lazy::new(build_catalog);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Language {
    #[default]
    English,
    Chinese,
    Spanish,
    Hindi,
    Arabic,
    Portuguese,
    Russian,
}

impl Language {
    pub fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Chinese => "zh",
            Self::Spanish => "es",
            Self::Hindi => "hi",
            Self::Arabic => "ar",
            Self::Portuguese => "pt",
            Self::Russian => "ru",
        }
    }

    /// Native name for the language picker.
    pub fn native_name(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Chinese => "中文",
            Self::Spanish => "Español",
            Self::Hindi => "हिन्दी",
            Self::Arabic => "العربية",
            Self::Portuguese => "Português",
            Self::Russian => "Русский",
        }
    }

    pub fn all() -> &'static [Language] {
        &[
            Self::English,
            Self::Chinese,
            Self::Spanish,
            Self::Hindi,
            Self::Arabic,
            Self::Portuguese,
            Self::Russian,
        ]
    }

    pub fn from_code(code: &str) -> Self {
        match code.to_lowercase().as_str() {
            "zh" | "zh-cn" | "zh_cn" | "chinese" => Self::Chinese,
            "es" | "es-es" | "spanish" => Self::Spanish,
            "hi" | "hi-in" | "hindi" => Self::Hindi,
            "ar" | "ar-sa" | "arabic" => Self::Arabic,
            "pt" | "pt-br" | "pt_br" | "portuguese" => Self::Portuguese,
            "ru" | "ru-ru" | "ru_ru" | "russian" => Self::Russian,
            _ => Self::English,
        }
    }
}

pub fn init(lang: Language) {
    if let Ok(mut g) = CURRENT.write() {
        *g = lang;
    }
}

pub fn current() -> Language {
    CURRENT.read().map(|g| *g).unwrap_or_default()
}

pub fn set_language(lang: Language) {
    init(lang);
}

/// Translate a key using the current language (falls back to English).
pub fn t(key: &str) -> String {
    let lang = current();
    if let Some(map) = STRINGS.get(&lang) {
        if let Some(s) = map.get(key) {
            return (*s).to_string();
        }
    }
    STRINGS
        .get(&Language::English)
        .and_then(|m| m.get(key).copied())
        .unwrap_or(key)
        .to_string()
}

pub fn tf(key: &str, args: &[(&str, &str)]) -> String {
    let mut s = t(key);
    for (k, v) in args {
        s = s.replace(&format!("{{{k}}}"), v);
    }
    s
}

fn entry(map: &mut HashMap<&'static str, &'static str>, key: &'static str, value: &'static str) {
    map.insert(key, value);
}

fn build_catalog() -> HashMap<Language, HashMap<&'static str, &'static str>> {
    let mut all = HashMap::new();

    // ——— English (default) ———
    let mut en = HashMap::new();
    entry(&mut en, "home", "Home");
    entry(&mut en, "all", "All");
    entry(&mut en, "all_apps", "All apps");
    entry(&mut en, "settings", "Settings");
    entry(&mut en, "system", "System");
    entry(&mut en, "manual", "Manual");
    entry(&mut en, "apps_installed", "Installed apps");
    entry(&mut en, "apps_count", "{n} apps");
    entry(&mut en, "apps_count_paren", "{n} app(s)");
    entry(&mut en, "searching_apps", "Searching for apps...");
    entry(&mut en, "searching_method", "Searching: {method}...");
    entry(&mut en, "method_count", "{method}: {n} app(s)");
    entry(&mut en, "scan_done_toast", "Scan complete: {n} apps");
    entry(&mut en, "apps_found", "{n} apps found");
    entry(&mut en, "by_method", "By installation method");
    entry(&mut en, "space_used", "Space used by apps");
    entry(&mut en, "search_placeholder", "Search apps...");
    entry(&mut en, "sort", "Sort:");
    entry(&mut en, "sort_name", "Name");
    entry(&mut en, "sort_size", "Size");
    entry(&mut en, "sort_date", "Install date");
    entry(&mut en, "sort_method", "Method");
    entry(&mut en, "sort_update", "Update available");
    entry(&mut en, "no_apps", "No apps");
    entry(&mut en, "no_apps_method", "No {method} apps");
    entry(&mut en, "no_apps_filter", "No apps for this installation method.");
    entry(&mut en, "no_apps_search", "No results for the current search.");
    entry(&mut en, "done", "Done");
    entry(&mut en, "in_progress", "In progress...");
    entry(&mut en, "error", "Error");
    entry(&mut en, "unavailable", "Not available");
    entry(&mut en, "pending", "Waiting");
    entry(&mut en, "details", "Details");
    entry(&mut en, "information", "Information");
    entry(&mut en, "installation", "Installation");
    entry(&mut en, "version", "Version");
    entry(&mut en, "type", "Type");
    entry(&mut en, "type_app", "Application");
    entry(&mut en, "architecture", "Architecture");
    entry(&mut en, "status", "Status");
    entry(&mut en, "category", "Category");
    entry(&mut en, "description", "Description");
    entry(&mut en, "method", "Method");
    entry(&mut en, "origin", "Origin");
    entry(&mut en, "location", "Location");
    entry(&mut en, "size", "Size");
    entry(&mut en, "date", "Date");
    entry(&mut en, "package", "Package");
    entry(&mut en, "uninstall", "Uninstall");
    entry(&mut en, "uninstall_confirm_title", "Uninstall {name}?");
    entry(
        &mut en,
        "uninstall_confirm_body",
        "Uninstall {name}?\n\nThis app was installed via {method}.\n\n{details}",
    );
    entry(&mut en, "cancel", "Cancel");
    entry(&mut en, "close", "Close");
    entry(&mut en, "uninstalled", "{name} uninstalled");
    entry(&mut en, "failure", "Failed: {error}");
    entry(&mut en, "interrupted", "Operation interrupted");
    entry(&mut en, "about", "About");
    entry(&mut en, "about_desc", "FindApps information");
    entry(&mut en, "identifier", "Identifier");
    entry(&mut en, "license", "License");
    entry(&mut en, "language", "Language");
    entry(&mut en, "language_desc", "Interface language");
    entry(
        &mut en,
        "language_updated",
        "Language updated",
    );
    entry(
        &mut en,
        "tagline",
        "Universal Linux application manager",
    );
    entry(
        &mut en,
        "about_body",
        "FindApps detects and organizes programs installed on your system, \
regardless of how they were installed — APT, DNF, Flatpak, Snap, AppImage, \
or manual installation.\n\n\
With a simple interface, you can search, filter, view details, and \
uninstall apps safely — without using the terminal.\n\n\
Installing and updating packages is planned for future versions.",
    );
    entry(
        &mut en,
        "about_features",
        "Backends · Search · Filters · Details · Safe uninstall",
    );
    entry(&mut en, "status_installed", "Installed");
    entry(&mut en, "status_update", "Update available");
    entry(&mut en, "status_broken", "Broken");
    entry(&mut en, "status_unknown", "Unknown");
    entry(&mut en, "backend_available", "{method} available");
    entry(&mut en, "backend_unavailable", "{method} not available");
    entry(&mut en, "backend_detectable", "{method} detectable");
    all.insert(Language::English, en);

    // ——— Chinese ———
    let mut zh = HashMap::new();
    entry(&mut zh, "home", "主页");
    entry(&mut zh, "all", "全部");
    entry(&mut zh, "all_apps", "全部应用");
    entry(&mut zh, "settings", "设置");
    entry(&mut zh, "system", "系统");
    entry(&mut zh, "manual", "手动");
    entry(&mut zh, "apps_installed", "已安装的应用");
    entry(&mut zh, "apps_count", "{n} 个应用");
    entry(&mut zh, "apps_count_paren", "{n} 个应用");
    entry(&mut zh, "searching_apps", "正在查找应用...");
    entry(&mut zh, "searching_method", "正在查找：{method}...");
    entry(&mut zh, "method_count", "{method}：{n} 个应用");
    entry(&mut zh, "scan_done_toast", "扫描完成：{n} 个应用");
    entry(&mut zh, "apps_found", "找到 {n} 个应用");
    entry(&mut zh, "by_method", "按安装方式");
    entry(&mut zh, "space_used", "应用占用空间");
    entry(&mut zh, "search_placeholder", "搜索应用...");
    entry(&mut zh, "sort", "排序：");
    entry(&mut zh, "sort_name", "名称");
    entry(&mut zh, "sort_size", "大小");
    entry(&mut zh, "sort_date", "安装日期");
    entry(&mut zh, "sort_method", "方式");
    entry(&mut zh, "sort_update", "有可用更新");
    entry(&mut zh, "no_apps", "没有应用");
    entry(&mut zh, "no_apps_method", "没有 {method} 应用");
    entry(&mut zh, "no_apps_filter", "此安装方式下没有应用。");
    entry(&mut zh, "no_apps_search", "当前搜索没有结果。");
    entry(&mut zh, "done", "完成");
    entry(&mut zh, "in_progress", "进行中...");
    entry(&mut zh, "error", "错误");
    entry(&mut zh, "unavailable", "不可用");
    entry(&mut zh, "pending", "等待中");
    entry(&mut zh, "details", "详情");
    entry(&mut zh, "information", "信息");
    entry(&mut zh, "installation", "安装");
    entry(&mut zh, "version", "版本");
    entry(&mut zh, "type", "类型");
    entry(&mut zh, "type_app", "应用程序");
    entry(&mut zh, "architecture", "架构");
    entry(&mut zh, "status", "状态");
    entry(&mut zh, "category", "类别");
    entry(&mut zh, "description", "描述");
    entry(&mut zh, "method", "方式");
    entry(&mut zh, "origin", "来源");
    entry(&mut zh, "location", "位置");
    entry(&mut zh, "size", "大小");
    entry(&mut zh, "date", "日期");
    entry(&mut zh, "package", "软件包");
    entry(&mut zh, "uninstall", "卸载");
    entry(&mut zh, "uninstall_confirm_title", "卸载 {name}？");
    entry(
        &mut zh,
        "uninstall_confirm_body",
        "卸载 {name}？\n\n此应用通过 {method} 安装。\n\n{details}",
    );
    entry(&mut zh, "cancel", "取消");
    entry(&mut zh, "close", "关闭");
    entry(&mut zh, "uninstalled", "已卸载 {name}");
    entry(&mut zh, "failure", "失败：{error}");
    entry(&mut zh, "interrupted", "操作已中断");
    entry(&mut zh, "about", "关于");
    entry(&mut zh, "about_desc", "FindApps 信息");
    entry(&mut zh, "identifier", "标识符");
    entry(&mut zh, "license", "许可证");
    entry(&mut zh, "language", "语言");
    entry(&mut zh, "language_desc", "界面语言");
    entry(
        &mut zh,
        "language_updated",
        "语言已更新",
    );
    entry(&mut zh, "tagline", "通用 Linux 应用管理器");
    entry(
        &mut zh,
        "about_body",
        "FindApps 可检测并整理系统中已安装的程序，\
无论通过 APT、DNF、Flatpak、Snap、AppImage 还是手动安装。\n\n\
界面简洁，可搜索、筛选、查看详情并安全卸载应用，无需使用终端。\n\n\
安装与更新功能将在后续版本中提供。",
    );
    entry(
        &mut zh,
        "about_features",
        "后端 · 搜索 · 筛选 · 详情 · 安全卸载",
    );
    entry(&mut zh, "status_installed", "已安装");
    entry(&mut zh, "status_update", "有可用更新");
    entry(&mut zh, "status_broken", "已损坏");
    entry(&mut zh, "status_unknown", "未知");
    entry(&mut zh, "backend_available", "{method} 可用");
    entry(&mut zh, "backend_unavailable", "{method} 不可用");
    entry(&mut zh, "backend_detectable", "{method} 可检测");
    all.insert(Language::Chinese, zh);

    // ——— Spanish ———
    let mut es = HashMap::new();
    entry(&mut es, "home", "Inicio");
    entry(&mut es, "all", "Todos");
    entry(&mut es, "all_apps", "Todas las apps");
    entry(&mut es, "settings", "Configuración");
    entry(&mut es, "system", "Sistema");
    entry(&mut es, "manual", "Manual");
    entry(&mut es, "apps_installed", "Aplicaciones instaladas");
    entry(&mut es, "apps_count", "{n} aplicaciones");
    entry(&mut es, "apps_count_paren", "{n} aplicación(es)");
    entry(&mut es, "searching_apps", "Buscando aplicaciones...");
    entry(&mut es, "searching_method", "Buscando: {method}...");
    entry(&mut es, "method_count", "{method}: {n} aplicación(es)");
    entry(&mut es, "scan_done_toast", "Escaneo completado: {n} apps");
    entry(&mut es, "apps_found", "{n} aplicaciones encontradas");
    entry(&mut es, "by_method", "Por método de instalación");
    entry(&mut es, "space_used", "Espacio usado por apps");
    entry(&mut es, "search_placeholder", "Buscar aplicaciones...");
    entry(&mut es, "sort", "Ordenar:");
    entry(&mut es, "sort_name", "Nombre");
    entry(&mut es, "sort_size", "Tamaño");
    entry(&mut es, "sort_date", "Fecha de instalación");
    entry(&mut es, "sort_method", "Método");
    entry(&mut es, "sort_update", "Actualización disponible");
    entry(&mut es, "no_apps", "Sin aplicaciones");
    entry(&mut es, "no_apps_method", "Sin apps {method}");
    entry(&mut es, "no_apps_filter", "No hay apps con este método de instalación.");
    entry(&mut es, "no_apps_search", "Sin resultados para la búsqueda actual.");
    entry(&mut es, "done", "Listo");
    entry(&mut es, "in_progress", "En curso...");
    entry(&mut es, "error", "Error");
    entry(&mut es, "unavailable", "No disponible");
    entry(&mut es, "pending", "Esperando");
    entry(&mut es, "details", "Detalles");
    entry(&mut es, "information", "Información");
    entry(&mut es, "installation", "Instalación");
    entry(&mut es, "version", "Versión");
    entry(&mut es, "type", "Tipo");
    entry(&mut es, "type_app", "Aplicación");
    entry(&mut es, "architecture", "Arquitectura");
    entry(&mut es, "status", "Estado");
    entry(&mut es, "category", "Categoría");
    entry(&mut es, "description", "Descripción");
    entry(&mut es, "method", "Método");
    entry(&mut es, "origin", "Origen");
    entry(&mut es, "location", "Ubicación");
    entry(&mut es, "size", "Tamaño");
    entry(&mut es, "date", "Fecha");
    entry(&mut es, "package", "Paquete");
    entry(&mut es, "uninstall", "Desinstalar");
    entry(&mut es, "uninstall_confirm_title", "¿Desinstalar {name}?");
    entry(
        &mut es,
        "uninstall_confirm_body",
        "¿Desinstalar {name}?\n\nEsta app se instaló vía {method}.\n\n{details}",
    );
    entry(&mut es, "cancel", "Cancelar");
    entry(&mut es, "close", "Cerrar");
    entry(&mut es, "uninstalled", "{name} desinstalada");
    entry(&mut es, "failure", "Error: {error}");
    entry(&mut es, "interrupted", "Operación interrumpida");
    entry(&mut es, "about", "Acerca de");
    entry(&mut es, "about_desc", "Información de FindApps");
    entry(&mut es, "identifier", "Identificador");
    entry(&mut es, "license", "Licencia");
    entry(&mut es, "language", "Idioma");
    entry(&mut es, "language_desc", "Idioma de la interfaz");
    entry(
        &mut es,
        "language_updated",
        "Idioma actualizado",
    );
    entry(&mut es, "tagline", "Gestor universal de aplicaciones Linux");
    entry(
        &mut es,
        "about_body",
        "FindApps detecta y organiza los programas instalados en tu sistema, \
sin importar el método — APT, DNF, Flatpak, Snap, AppImage o instalación manual.\n\n\
Con una interfaz simple puedes buscar, filtrar, ver detalles y \
desinstalar de forma segura, sin usar la terminal.\n\n\
La instalación y actualización de paquetes están previstas para versiones futuras.",
    );
    entry(
        &mut es,
        "about_features",
        "Backends · Búsqueda · Filtros · Detalles · Desinstalación segura",
    );
    entry(&mut es, "status_installed", "Instalada");
    entry(&mut es, "status_update", "Actualización disponible");
    entry(&mut es, "status_broken", "Rota");
    entry(&mut es, "status_unknown", "Desconocido");
    entry(&mut es, "backend_available", "{method} disponible");
    entry(&mut es, "backend_unavailable", "{method} no disponible");
    entry(&mut es, "backend_detectable", "{method} detectable");
    all.insert(Language::Spanish, es);

    // ——— Hindi ———
    let mut hi = HashMap::new();
    entry(&mut hi, "home", "होम");
    entry(&mut hi, "all", "सभी");
    entry(&mut hi, "all_apps", "सभी ऐप्स");
    entry(&mut hi, "settings", "सेटिंग्स");
    entry(&mut hi, "system", "सिस्टम");
    entry(&mut hi, "manual", "मैनुअल");
    entry(&mut hi, "apps_installed", "इंस्टॉल किए गए ऐप्स");
    entry(&mut hi, "apps_count", "{n} ऐप्स");
    entry(&mut hi, "apps_count_paren", "{n} ऐप");
    entry(&mut hi, "searching_apps", "ऐप्स खोजे जा रहे हैं...");
    entry(&mut hi, "searching_method", "खोज रहे हैं: {method}...");
    entry(&mut hi, "method_count", "{method}: {n} ऐप");
    entry(&mut hi, "scan_done_toast", "स्कैन पूरा: {n} ऐप्स");
    entry(&mut hi, "apps_found", "{n} ऐप्स मिले");
    entry(&mut hi, "by_method", "इंस्टॉल विधि के अनुसार");
    entry(&mut hi, "space_used", "ऐप्स द्वारा उपयोग स्थान");
    entry(&mut hi, "search_placeholder", "ऐप्स खोजें...");
    entry(&mut hi, "sort", "क्रमबद्ध करें:");
    entry(&mut hi, "sort_name", "नाम");
    entry(&mut hi, "sort_size", "आकार");
    entry(&mut hi, "sort_date", "इंस्टॉल तिथि");
    entry(&mut hi, "sort_method", "विधि");
    entry(&mut hi, "sort_update", "अपडेट उपलब्ध");
    entry(&mut hi, "no_apps", "कोई ऐप नहीं");
    entry(&mut hi, "no_apps_method", "कोई {method} ऐप नहीं");
    entry(&mut hi, "no_apps_filter", "इस इंस्टॉल विधि में कोई ऐप नहीं है।");
    entry(&mut hi, "no_apps_search", "वर्तमान खोज के लिए कोई परिणाम नहीं।");
    entry(&mut hi, "done", "पूर्ण");
    entry(&mut hi, "in_progress", "प्रगति में...");
    entry(&mut hi, "error", "त्रुटि");
    entry(&mut hi, "unavailable", "उपलब्ध नहीं");
    entry(&mut hi, "pending", "प्रतीक्षा");
    entry(&mut hi, "details", "विवरण");
    entry(&mut hi, "information", "जानकारी");
    entry(&mut hi, "installation", "इंस्टॉलेशन");
    entry(&mut hi, "version", "संस्करण");
    entry(&mut hi, "type", "प्रकार");
    entry(&mut hi, "type_app", "एप्लिकेशन");
    entry(&mut hi, "architecture", "आर्किटेक्चर");
    entry(&mut hi, "status", "स्थिति");
    entry(&mut hi, "category", "श्रेणी");
    entry(&mut hi, "description", "विवरण");
    entry(&mut hi, "method", "विधि");
    entry(&mut hi, "origin", "स्रोत");
    entry(&mut hi, "location", "स्थान");
    entry(&mut hi, "size", "आकार");
    entry(&mut hi, "date", "तिथि");
    entry(&mut hi, "package", "पैकेज");
    entry(&mut hi, "uninstall", "अनइंस्टॉल");
    entry(&mut hi, "uninstall_confirm_title", "{name} अनइंस्टॉल करें?");
    entry(
        &mut hi,
        "uninstall_confirm_body",
        "{name} अनइंस्टॉल करें?\n\nयह ऐप {method} से इंस्टॉल हुआ था।\n\n{details}",
    );
    entry(&mut hi, "cancel", "रद्द करें");
    entry(&mut hi, "close", "बंद करें");
    entry(&mut hi, "uninstalled", "{name} अनइंस्टॉल हो गया");
    entry(&mut hi, "failure", "विफल: {error}");
    entry(&mut hi, "interrupted", "ऑपरेशन बाधित");
    entry(&mut hi, "about", "परिचय");
    entry(&mut hi, "about_desc", "FindApps जानकारी");
    entry(&mut hi, "identifier", "पहचानकर्ता");
    entry(&mut hi, "license", "लाइसेंस");
    entry(&mut hi, "language", "भाषा");
    entry(&mut hi, "language_desc", "इंटरफ़ेस भाषा");
    entry(
        &mut hi,
        "language_updated",
        "भाषा अपडेट हो गई",
    );
    entry(&mut hi, "tagline", "यूनिवर्सल Linux ऐप मैनेजर");
    entry(
        &mut hi,
        "about_body",
        "FindApps आपके सिस्टम पर इंस्टॉल प्रोग्रामों का पता लगाता और व्यवस्थित करता है — \
APT, DNF, Flatpak, Snap, AppImage या मैन्युअल इंस्टॉल से।\n\n\
सरल इंटरफ़ेस से आप खोज, फ़िल्टर, विवरण देख और सुरक्षित रूप से अनइंस्टॉल कर सकते हैं, \
टर्मिनल की आवश्यकता के बिना।\n\n\
इंस्टॉल और अपडेट भविष्य के संस्करणों में आएंगे।",
    );
    entry(
        &mut hi,
        "about_features",
        "बैकएंड · खोज · फ़िल्टर · विवरण · सुरक्षित अनइंस्टॉल",
    );
    entry(&mut hi, "status_installed", "इंस्टॉल");
    entry(&mut hi, "status_update", "अपडेट उपलब्ध");
    entry(&mut hi, "status_broken", "क्षतिग्रस्त");
    entry(&mut hi, "status_unknown", "अज्ञात");
    entry(&mut hi, "backend_available", "{method} उपलब्ध");
    entry(&mut hi, "backend_unavailable", "{method} उपलब्ध नहीं");
    entry(&mut hi, "backend_detectable", "{method} पता लगाने योग्य");
    all.insert(Language::Hindi, hi);

    // ——— Arabic ———
    let mut ar = HashMap::new();
    entry(&mut ar, "home", "الرئيسية");
    entry(&mut ar, "all", "الكل");
    entry(&mut ar, "all_apps", "كل التطبيقات");
    entry(&mut ar, "settings", "الإعدادات");
    entry(&mut ar, "system", "النظام");
    entry(&mut ar, "manual", "يدوي");
    entry(&mut ar, "apps_installed", "التطبيقات المثبتة");
    entry(&mut ar, "apps_count", "{n} تطبيقات");
    entry(&mut ar, "apps_count_paren", "{n} تطبيق(ات)");
    entry(&mut ar, "searching_apps", "جاري البحث عن التطبيقات...");
    entry(&mut ar, "searching_method", "جاري البحث: {method}...");
    entry(&mut ar, "method_count", "{method}: {n} تطبيق(ات)");
    entry(&mut ar, "scan_done_toast", "اكتمل الفحص: {n} تطبيقات");
    entry(&mut ar, "apps_found", "تم العثور على {n} تطبيقات");
    entry(&mut ar, "by_method", "حسب طريقة التثبيت");
    entry(&mut ar, "space_used", "المساحة المستخدمة للتطبيقات");
    entry(&mut ar, "search_placeholder", "البحث عن تطبيقات...");
    entry(&mut ar, "sort", "ترتيب:");
    entry(&mut ar, "sort_name", "الاسم");
    entry(&mut ar, "sort_size", "الحجم");
    entry(&mut ar, "sort_date", "تاريخ التثبيت");
    entry(&mut ar, "sort_method", "الطريقة");
    entry(&mut ar, "sort_update", "يتوفر تحديث");
    entry(&mut ar, "no_apps", "لا توجد تطبيقات");
    entry(&mut ar, "no_apps_method", "لا توجد تطبيقات {method}");
    entry(&mut ar, "no_apps_filter", "لا توجد تطبيقات لهذه الطريقة.");
    entry(&mut ar, "no_apps_search", "لا نتائج للبحث الحالي.");
    entry(&mut ar, "done", "تم");
    entry(&mut ar, "in_progress", "قيد التقدم...");
    entry(&mut ar, "error", "خطأ");
    entry(&mut ar, "unavailable", "غير متاح");
    entry(&mut ar, "pending", "في الانتظار");
    entry(&mut ar, "details", "التفاصيل");
    entry(&mut ar, "information", "معلومات");
    entry(&mut ar, "installation", "التثبيت");
    entry(&mut ar, "version", "الإصدار");
    entry(&mut ar, "type", "النوع");
    entry(&mut ar, "type_app", "تطبيق");
    entry(&mut ar, "architecture", "المعمارية");
    entry(&mut ar, "status", "الحالة");
    entry(&mut ar, "category", "الفئة");
    entry(&mut ar, "description", "الوصف");
    entry(&mut ar, "method", "الطريقة");
    entry(&mut ar, "origin", "المصدر");
    entry(&mut ar, "location", "الموقع");
    entry(&mut ar, "size", "الحجم");
    entry(&mut ar, "date", "التاريخ");
    entry(&mut ar, "package", "الحزمة");
    entry(&mut ar, "uninstall", "إلغاء التثبيت");
    entry(&mut ar, "uninstall_confirm_title", "إلغاء تثبيت {name}؟");
    entry(
        &mut ar,
        "uninstall_confirm_body",
        "إلغاء تثبيت {name}؟\n\nتم تثبيت هذا التطبيق عبر {method}.\n\n{details}",
    );
    entry(&mut ar, "cancel", "إلغاء");
    entry(&mut ar, "close", "إغلاق");
    entry(&mut ar, "uninstalled", "تم إلغاء تثبيت {name}");
    entry(&mut ar, "failure", "فشل: {error}");
    entry(&mut ar, "interrupted", "تم إيقاف العملية");
    entry(&mut ar, "about", "حول");
    entry(&mut ar, "about_desc", "معلومات FindApps");
    entry(&mut ar, "identifier", "المعرّف");
    entry(&mut ar, "license", "الترخيص");
    entry(&mut ar, "language", "اللغة");
    entry(&mut ar, "language_desc", "لغة الواجهة");
    entry(
        &mut ar,
        "language_updated",
        "تم تحديث اللغة",
    );
    entry(&mut ar, "tagline", "مدير تطبيقات لينكس الشامل");
    entry(
        &mut ar,
        "about_body",
        "يكتشف FindApps وينظّم البرامج المثبتة على نظامك، \
بغض النظر عن طريقة التثبيت — APT أو DNF أو Flatpak أو Snap أو AppImage أو التثبيت اليدوي.\n\n\
بواجهة بسيطة يمكنك البحث والتصفية وعرض التفاصيل وإلغاء التثبيت بأمان دون الطرفية.\n\n\
التثبيت والتحديث مخططان للإصدارات القادمة.",
    );
    entry(
        &mut ar,
        "about_features",
        "الخوادم · البحث · التصفية · التفاصيل · إلغاء تثبيت آمن",
    );
    entry(&mut ar, "status_installed", "مثبت");
    entry(&mut ar, "status_update", "يتوفر تحديث");
    entry(&mut ar, "status_broken", "تالف");
    entry(&mut ar, "status_unknown", "غير معروف");
    entry(&mut ar, "backend_available", "{method} متاح");
    entry(&mut ar, "backend_unavailable", "{method} غير متاح");
    entry(&mut ar, "backend_detectable", "{method} قابل للاكتشاف");
    all.insert(Language::Arabic, ar);

    // ——— Portuguese ———
    let mut pt = HashMap::new();
    entry(&mut pt, "home", "Início");
    entry(&mut pt, "all", "Todos");
    entry(&mut pt, "all_apps", "Todos os aplicativos");
    entry(&mut pt, "settings", "Configurações");
    entry(&mut pt, "system", "Sistema");
    entry(&mut pt, "manual", "Manual");
    entry(&mut pt, "apps_installed", "Aplicativos instalados");
    entry(&mut pt, "apps_count", "{n} aplicativos");
    entry(&mut pt, "apps_count_paren", "{n} aplicativo(s)");
    entry(&mut pt, "searching_apps", "Procurando aplicativos...");
    entry(&mut pt, "searching_method", "Procurando: {method}...");
    entry(&mut pt, "method_count", "{method}: {n} aplicativo(s)");
    entry(&mut pt, "scan_done_toast", "Varredura concluída: {n} apps");
    entry(&mut pt, "apps_found", "{n} aplicativos encontrados");
    entry(&mut pt, "by_method", "Por método de instalação");
    entry(&mut pt, "space_used", "Espaço utilizado por aplicativos");
    entry(&mut pt, "search_placeholder", "Pesquisar aplicativos...");
    entry(&mut pt, "sort", "Ordenar:");
    entry(&mut pt, "sort_name", "Nome");
    entry(&mut pt, "sort_size", "Tamanho");
    entry(&mut pt, "sort_date", "Data de instalação");
    entry(&mut pt, "sort_method", "Método");
    entry(&mut pt, "sort_update", "Atualização disponível");
    entry(&mut pt, "no_apps", "Nenhum aplicativo");
    entry(&mut pt, "no_apps_method", "Nenhum aplicativo {method}");
    entry(&mut pt, "no_apps_filter", "Não há aplicativos neste método de instalação.");
    entry(&mut pt, "no_apps_search", "Nenhum resultado para a pesquisa atual.");
    entry(&mut pt, "done", "Concluído");
    entry(&mut pt, "in_progress", "Em andamento...");
    entry(&mut pt, "error", "Erro");
    entry(&mut pt, "unavailable", "Não disponível");
    entry(&mut pt, "pending", "Aguardando");
    entry(&mut pt, "details", "Detalhes");
    entry(&mut pt, "information", "Informações");
    entry(&mut pt, "installation", "Instalação");
    entry(&mut pt, "version", "Versão");
    entry(&mut pt, "type", "Tipo");
    entry(&mut pt, "type_app", "Aplicativo");
    entry(&mut pt, "architecture", "Arquitetura");
    entry(&mut pt, "status", "Status");
    entry(&mut pt, "category", "Categoria");
    entry(&mut pt, "description", "Descrição");
    entry(&mut pt, "method", "Método");
    entry(&mut pt, "origin", "Origem");
    entry(&mut pt, "location", "Local");
    entry(&mut pt, "size", "Tamanho");
    entry(&mut pt, "date", "Data");
    entry(&mut pt, "package", "Pacote");
    entry(&mut pt, "uninstall", "Desinstalar");
    entry(&mut pt, "uninstall_confirm_title", "Desinstalar {name}?");
    entry(
        &mut pt,
        "uninstall_confirm_body",
        "Desinstalar {name}?\n\nEste aplicativo foi instalado via {method}.\n\n{details}",
    );
    entry(&mut pt, "cancel", "Cancelar");
    entry(&mut pt, "close", "Fechar");
    entry(&mut pt, "uninstalled", "{name} desinstalado");
    entry(&mut pt, "failure", "Falha: {error}");
    entry(&mut pt, "interrupted", "Operação interrompida");
    entry(&mut pt, "about", "Sobre");
    entry(&mut pt, "about_desc", "Informações do FindApps");
    entry(&mut pt, "identifier", "Identificador");
    entry(&mut pt, "license", "Licença");
    entry(&mut pt, "language", "Idioma");
    entry(&mut pt, "language_desc", "Idioma da interface");
    entry(
        &mut pt,
        "language_updated",
        "Idioma atualizado",
    );
    entry(&mut pt, "tagline", "Gerenciador universal de aplicativos Linux");
    entry(
        &mut pt,
        "about_body",
        "O FindApps detecta e organiza programas instalados no seu sistema, \
independentemente do método de instalação — APT, DNF, Flatpak, Snap, AppImage \
ou instalação manual.\n\n\
Com uma interface simples, você pode pesquisar, filtrar, ver detalhes e \
desinstalar aplicativos com segurança, sem precisar usar o terminal.\n\n\
A instalação e as atualizações de pacotes estão previstas para versões futuras.",
    );
    entry(
        &mut pt,
        "about_features",
        "Backends · Pesquisa · Filtros · Detalhes · Desinstalação segura",
    );
    entry(&mut pt, "status_installed", "Instalado");
    entry(&mut pt, "status_update", "Atualização disponível");
    entry(&mut pt, "status_broken", "Quebrado");
    entry(&mut pt, "status_unknown", "Desconhecido");
    entry(&mut pt, "backend_available", "{method} disponível");
    entry(&mut pt, "backend_unavailable", "{method} não disponível");
    entry(&mut pt, "backend_detectable", "{method} detectável");
    all.insert(Language::Portuguese, pt);

    // ——— Russian ———
    let mut ru = HashMap::new();
    entry(&mut ru, "home", "Главная");
    entry(&mut ru, "all", "Все");
    entry(&mut ru, "all_apps", "Все приложения");
    entry(&mut ru, "settings", "Настройки");
    entry(&mut ru, "system", "Система");
    entry(&mut ru, "manual", "Вручную");
    entry(&mut ru, "apps_installed", "Установленные приложения");
    entry(&mut ru, "apps_count", "{n} приложений");
    entry(&mut ru, "apps_count_paren", "{n} прилож.");
    entry(&mut ru, "searching_apps", "Поиск приложений...");
    entry(&mut ru, "searching_method", "Поиск: {method}...");
    entry(&mut ru, "method_count", "{method}: {n} прилож.");
    entry(&mut ru, "scan_done_toast", "Сканирование завершено: {n} прилож.");
    entry(&mut ru, "apps_found", "Найдено приложений: {n}");
    entry(&mut ru, "by_method", "По способу установки");
    entry(&mut ru, "space_used", "Место, занятое приложениями");
    entry(&mut ru, "search_placeholder", "Поиск приложений...");
    entry(&mut ru, "sort", "Сортировка:");
    entry(&mut ru, "sort_name", "Имя");
    entry(&mut ru, "sort_size", "Размер");
    entry(&mut ru, "sort_date", "Дата установки");
    entry(&mut ru, "sort_method", "Способ");
    entry(&mut ru, "sort_update", "Доступно обновление");
    entry(&mut ru, "no_apps", "Нет приложений");
    entry(&mut ru, "no_apps_method", "Нет приложений {method}");
    entry(&mut ru, "no_apps_filter", "Нет приложений с этим способом установки.");
    entry(&mut ru, "no_apps_search", "Нет результатов по текущему поиску.");
    entry(&mut ru, "done", "Готово");
    entry(&mut ru, "in_progress", "Выполняется...");
    entry(&mut ru, "error", "Ошибка");
    entry(&mut ru, "unavailable", "Недоступно");
    entry(&mut ru, "pending", "Ожидание");
    entry(&mut ru, "details", "Сведения");
    entry(&mut ru, "information", "Информация");
    entry(&mut ru, "installation", "Установка");
    entry(&mut ru, "version", "Версия");
    entry(&mut ru, "type", "Тип");
    entry(&mut ru, "type_app", "Приложение");
    entry(&mut ru, "architecture", "Архитектура");
    entry(&mut ru, "status", "Статус");
    entry(&mut ru, "category", "Категория");
    entry(&mut ru, "description", "Описание");
    entry(&mut ru, "method", "Способ");
    entry(&mut ru, "origin", "Источник");
    entry(&mut ru, "location", "Расположение");
    entry(&mut ru, "size", "Размер");
    entry(&mut ru, "date", "Дата");
    entry(&mut ru, "package", "Пакет");
    entry(&mut ru, "uninstall", "Удалить");
    entry(&mut ru, "uninstall_confirm_title", "Удалить {name}?");
    entry(
        &mut ru,
        "uninstall_confirm_body",
        "Удалить {name}?\n\nЭто приложение установлено через {method}.\n\n{details}",
    );
    entry(&mut ru, "cancel", "Отмена");
    entry(&mut ru, "close", "Закрыть");
    entry(&mut ru, "uninstalled", "{name} удалено");
    entry(&mut ru, "failure", "Ошибка: {error}");
    entry(&mut ru, "interrupted", "Операция прервана");
    entry(&mut ru, "about", "О программе");
    entry(&mut ru, "about_desc", "Сведения о FindApps");
    entry(&mut ru, "identifier", "Идентификатор");
    entry(&mut ru, "license", "Лицензия");
    entry(&mut ru, "language", "Язык");
    entry(&mut ru, "language_desc", "Язык интерфейса");
    entry(
        &mut ru,
        "language_updated",
        "Язык обновлён",
    );
    entry(&mut ru, "tagline", "Универсальный менеджер приложений Linux");
    entry(
        &mut ru,
        "about_body",
        "FindApps обнаруживает и упорядочивает программы, установленные в системе, \
независимо от способа — APT, DNF, Flatpak, Snap, AppImage или ручная установка.\n\n\
В простом интерфейсе можно искать, фильтровать, смотреть сведения и \
безопасно удалять приложения без терминала.\n\n\
Установка и обновление пакетов запланированы в будущих версиях.",
    );
    entry(
        &mut ru,
        "about_features",
        "Бэкенды · Поиск · Фильтры · Сведения · Безопасное удаление",
    );
    entry(&mut ru, "status_installed", "Установлено");
    entry(&mut ru, "status_update", "Доступно обновление");
    entry(&mut ru, "status_broken", "Повреждено");
    entry(&mut ru, "status_unknown", "Неизвестно");
    entry(&mut ru, "backend_available", "{method} доступен");
    entry(&mut ru, "backend_unavailable", "{method} недоступен");
    entry(&mut ru, "backend_detectable", "{method} обнаруживается");
    all.insert(Language::Russian, ru);

    all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_english() {
        assert_eq!(Language::default(), Language::English);
        assert_eq!(Language::from_code("en").code(), "en");
    }

    #[test]
    fn translates_home() {
        set_language(Language::English);
        assert_eq!(t("home"), "Home");
        set_language(Language::Portuguese);
        assert_eq!(t("home"), "Início");
        set_language(Language::Chinese);
        assert_eq!(t("home"), "主页");
        set_language(Language::Russian);
        assert_eq!(t("home"), "Главная");
        set_language(Language::English);
    }
}
