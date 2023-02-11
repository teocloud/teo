use async_trait::async_trait;
use cuid::cuid;
use crate::core::pipeline::item::Item;
use crate::core::teon::Value;
use crate::core::result::Result;
use crate::core::pipeline::ctx::Ctx;

#[derive(Debug, Copy, Clone)]
pub struct CUIDModifier {}

impl CUIDModifier {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Item for CUIDModifier {
    async fn call<'a>(&self, ctx: Ctx<'a>) -> Result<Ctx<'a>> {
        context.with_value(Value::String(cuid().unwrap()))
    }
}
