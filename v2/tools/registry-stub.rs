use crate::native_core::common::{CoreContext, CoreResult};
use serde_json::Value;
pub fn installed_command(ctx: &CoreContext, _id: &str, command: &str) -> Option<Value> {
    let path = ctx.paths.data_dir.join("installed.json");
    if !path.exists() {
        return None;
    }
    let value: Value = serde_json::from_slice(&std::fs::read(path).ok()?).ok()?;
    (command == "index").then_some(value)
}
pub fn preferences(value: &Value, _command: Option<&str>) -> CoreResult {
    Ok(value["preferences"].clone())
}
