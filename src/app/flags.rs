use cosmic::cosmic_config;

use crate::{
    core::config::{self, TasksConfig},
    services::store::Store,
};

#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: config::TasksConfig,
    pub storage: Store,
}

pub fn flags(storage: Store) -> Flags {
    let (config_handler, config) = (TasksConfig::config_handler(), TasksConfig::config());

    Flags {
        config_handler,
        config,
        storage,
    }
}
