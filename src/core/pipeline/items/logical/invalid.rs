use async_trait::async_trait;
use crate::core::pipeline::item::Item;
use crate::core::pipeline::ctx::Ctx;
use crate::core::result::Result;

#[derive(Debug, Copy, Clone)]
pub struct InvalidModifier { }

impl InvalidModifier {
    pub fn new() -> Self {
        Self { }
    }
}

#[async_trait]
impl Item for InvalidModifier {
    async fn call<'a>(&self, ctx: Ctx<'a>) -> Result<Ctx<'a>> {
        ctx.internal_server_error("Value is invalid.")
    }
}
