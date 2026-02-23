use cosmic::{
    app::Settings,
    iced::{Limits, Size},
    Application,
};
use directories::ProjectDirs;
use std::sync::Mutex;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    app::Tasks,
    core::{
        config::TasksConfig,
        icons::{IconCache, ICON_CACHE},
        localize::localize,
    },
    services::store::Store,
};

pub fn init() {
    localize();
    icons();
    tracing();
}

pub fn storage() -> Result<Store, crate::Error> {
    let project = ProjectDirs::from("dev", "edfloreshz", "Tasks")
        .expect("Failed to determine project directories");

    Store::open(project.data_dir())
}

pub fn settings() -> Settings {
    Settings::default()
        .antialiasing(true)
        .client_decorations(true)
        .theme(TasksConfig::config().app_theme.theme())
        .size_limits(Limits::NONE.min_width(350.0).min_height(180.0))
        .size(Size::new(850.0, 700.0))
        .debug(false)
}

pub fn tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

pub fn icons() {
    ICON_CACHE.get_or_init(|| Mutex::new(IconCache::new()));
    crate::core::icons::cache_all_icons_in_background(vec![14, 16, 18, 20, 32]);
}
