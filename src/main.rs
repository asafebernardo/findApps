//! FindApps — gerenciador universal de aplicativos Linux.

use tracing::info;

fn main() {
    findapps::util::logging::init_logging();
    info!("Iniciando FindApps {}", env!("CARGO_PKG_VERSION"));

    let app = findapps::app::FindApplication::new();
    app.run();
}
