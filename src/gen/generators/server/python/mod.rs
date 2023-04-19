
use crate::gen::internal::file_util::FileUtil;
use crate::gen::interface::server::EntityGenerator;
use crate::prelude::Graph;
use async_trait::async_trait;
use crate::gen::interface::server::conf::EntityGeneratorConf;
use crate::core::result::Result;

pub(crate) struct PythonEntityGenerator {}

impl PythonEntityGenerator {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl EntityGenerator for PythonEntityGenerator {
    async fn generate_entity_files(&self, _graph: &Graph, _conf: &EntityGeneratorConf, _generator: &FileUtil) -> Result<()> {
        Ok(())
    }
}
