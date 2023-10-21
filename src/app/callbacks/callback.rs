use std::future::Future;
use futures_util::future::BoxFuture;
use teo_runtime::request::ctx::Ctx;
use teo_result::Result;

pub trait AsyncCallback: Send + Sync {
    fn call(&self, ctx: Ctx) -> BoxFuture<'static, Result<()>>;
}

impl<F, Fut> AsyncCallback for F where
    F: Fn(Ctx) -> Fut + Send + Sync,
    Fut: Future<Output = Result<()>> + Send + 'static {
    fn call(&self, ctx: Ctx) -> BoxFuture<'static, Result<()>> {
        Box::pin(self(ctx))
    }
}