use crate::{core::config::TasksConfig, services::store::Store};
use cosmic::cosmic_config::Config;

#[derive(Clone, Debug)]
pub struct Flags {
    pub handler: Config,
    pub config: TasksConfig,
    pub store: Store,
}
