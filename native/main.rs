mod migration;
mod process;
use serde_json::{json, Value};
use tezbar_extension_sdk::{BoxFuture, CoreResult, Extension, ExtensionContext};
#[derive(Default)]
struct Adapter {
    effects: process::Effects,
}
impl Extension for Adapter {
    fn id(&self) -> &'static str {
        "com.tezbar.native.unicode-symbols"
    }
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
    fn commands(&self) -> &'static [&'static str] {
        &["index"]
    }
    fn initialize<'a>(&'a self, ctx: &'a ExtensionContext) -> BoxFuture<'a, CoreResult> {
        Box::pin(async move {
            migration::initialize(ctx).await?;
            Ok(json!({"ready":true}))
        })
    }
    fn handle<'a>(
        &'a self,
        ctx: &'a ExtensionContext,
        command: &'a str,
        operation: &'a str,
        payload: &'a Value,
        state: &'a mut Value,
    ) -> BoxFuture<'a, CoreResult> {
        Box::pin(process::handle(
            &self.effects,
            ctx,
            command,
            operation,
            payload,
            state,
        ))
    }
}
fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("extension runtime");
    let result = runtime.block_on(tezbar_extension_sdk::serve(Adapter::default()));
    runtime.shutdown_timeout(std::time::Duration::from_millis(500));
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests;
