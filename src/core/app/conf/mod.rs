use std::path::PathBuf;
use crate::core::app::environment::Environment;
use crate::prelude::Value;

pub struct ServerConf {
    pub(crate) bind: (String, u16),
    pub(crate) jwt_secret: Option<String>,
    pub(crate) path_prefix: Option<String>,
}

pub struct DebugConf {
    pub(crate) log_queries: bool,
    pub(crate) log_migrations: bool,
    pub(crate) log_seed_records: bool,
}

pub struct TestConf {
    pub(crate) reset_after_find: Value,
}

#[derive(Clone)]
pub struct EntityGeneratorConf {
    pub(crate) name: Option<String>,
    pub(crate) provider: Environment,
    pub(crate) dest: PathBuf,
}

