use async_trait::async_trait;
use crate::core::pipeline::item::Item;
use crate::core::teon::Value;
use crate::core::pipeline::ctx::Ctx;
use crate::core::result::Result;
#[derive(Debug, Copy, Clone)]
pub struct ReverseModifier {}

impl ReverseModifier {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Item for ReverseModifier {

    async fn call<'a>(&self, ctx: Ctx<'a>) -> Result<Ctx<'a>> {
        match &ctx.value {
            Value::String(s) => ctx.with_value(Value::String(s.chars().rev().collect::<String>())),
            Value::Vec(v) => ctx.with_value(Value::Vec(v.into_iter().rev().map(|v| v.clone()).collect())),
            _ => ctx.internal_server_error("Value cannot be reversed.")
        }
    }
}
