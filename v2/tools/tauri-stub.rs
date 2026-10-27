#[derive(Clone)]
pub struct AppHandle;
pub trait Emitter {
    fn emit(&self, channel: &str, payload: serde_json::Value) -> Result<(), String>;
}
impl Emitter for AppHandle {
    fn emit(&self, _: &str, _: serde_json::Value) -> Result<(), String> {
        Err("Headless fixture forbids OS events".into())
    }
}
