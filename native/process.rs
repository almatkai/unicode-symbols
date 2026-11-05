//! Reuses the v2 engine; only transport, persistence and unsupported snippet routing differ.
use engine::{Effect, Session, Store};
use serde_json::{json, Value};
use std::{collections::HashMap, sync::Mutex};
use tezbar_extension_sdk::{read_store, update_store, CoreResult, ExtensionContext};
pub const STORE: &str = "unicode-symbols";
// SDK state is transactional: errors/cancellation discard it. External effects cannot
// roll back, so retain acknowledgement/uncertainty separately until bindings expire.
#[derive(Default)]
pub struct Effects(Mutex<HashMap<String, HashMap<String, bool>>>);
impl Effects {
    fn begin(&self, session: &str, id: &str, copy: bool) -> Result<bool, String> {
        let mut all = self.0.lock().map_err(|e| e.to_string())?;
        let lane = all.entry(session.into()).or_default();
        match lane.get(id) {
            Some(true) => return Ok(false),
            Some(false) if !copy => return Err("Prior effect outcome is uncertain; return to search and deliberately select a new action to retry.".into()),
            _ => {}
        }
        lane.insert(id.into(), false);
        Ok(true)
    }
    fn done(&self, session: &str, id: &str) {
        self.0
            .lock()
            .unwrap()
            .entry(session.into())
            .or_default()
            .insert(id.into(), true);
    }
    fn prune(&self, session: &str, bindings: &HashMap<String, engine::Binding>) {
        if let Some(lane) = self.0.lock().unwrap().get_mut(session) {
            lane.retain(|id, _| bindings.contains_key(id));
        }
    }
    fn clear(&self, session: &str) {
        self.0.lock().unwrap().remove(session);
    }
}
fn save(state: &mut Value, session: &Session) -> Result<(), String> {
    *state = serde_json::to_value(session).map_err(|e| e.to_string())?;
    Ok(())
}
pub async fn handle(
    effects: &Effects,
    ctx: &ExtensionContext,
    command: &str,
    operation: &str,
    payload: &Value,
    state: &mut Value,
) -> CoreResult {
    if command != "index" {
        return Err("Unknown Unicode command".into());
    }
    let sid = payload["sessionId"].as_str().ok_or("Missing sessionId")?;
    if operation == "extension:dispose-session" {
        effects.clear(sid);
        *state = Value::Null;
        return Ok(tezbar_extension_sdk::unchanged());
    }
    if operation == "extension:refresh-session" {
        return Ok(tezbar_extension_sdk::unchanged());
    }
    let mut session = if operation == "extension:run-command" {
        if state.as_object().is_none_or(|s| !s.is_empty()) {
            return Err("Unicode session already initialized".into());
        }
        Session::new(payload)?
    } else {
        serde_json::from_value::<Session>(state.clone())
            .map_err(|_| "Unicode session is not initialized")?
    };
    effects.prune(sid, &session.bindings);
    let mut page = state["processPage"].as_u64().unwrap_or(0) as usize;
    let mut message = None;
    match operation {
        "extension:run-command" => {}
        "extension:search-text-changed" => {
            if session.snippet.is_some() {
                return Err("Search unavailable in snippet form".into());
            }
            page = 0;
            session.query = payload["searchText"]
                .as_str()
                .ok_or("Missing searchText")?
                .into();
        }
        "extension:invoke-action" => {
            let id = payload["actionId"].as_str().ok_or("Missing actionId")?;
            let binding = session.binding(id)?.clone();
            // The current closed process allowlist has no snippet creation API.
            if matches!(binding.op.as_str(), "snippet" | "submit-snippet") {
                return Err("Save as Snippet is unsupported: Tezbar's process host does not expose a snippet creation API. Copy the character and create the snippet in Tezbar.".into());
            }
            if binding.op == "process-next-page" {
                page = page.saturating_add(1);
            } else if binding.op == "process-previous-page" {
                page = page.saturating_sub(1);
            } else {
                if matches!(
                    binding.op.as_str(),
                    "filter" | "set-filter" | "clear-filter"
                ) {
                    page = 0;
                }
                let (effect, mutation) = session.prepare(id, payload)?;
                if let Some(effect) = effect {
                    let copy = matches!(effect, Effect::Copy(_));
                    if effects.begin(sid, id, copy)? {
                        let response = match effect {
                            Effect::Copy(text) => {
                                ctx.host("host.clipboard.write", json!({"text":text})).await
                            }
                            Effect::Paste(text) => {
                                ctx.host("host.clipboard.pasteText", json!({"text":text}))
                                    .await
                            }
                            Effect::Open(target) => {
                                ctx.host("host.shell.open", json!({"target":target})).await
                            }
                            Effect::Snippet { .. } => unreachable!(),
                        }?;
                        engine::acknowledge(&response)?;
                        effects.done(sid, id);
                    }
                    message = Some(match binding.op.as_str() {
                        "paste" => "Pasted character",
                        "compart" | "explorer" => "Opened character reference",
                        _ => "Copied to clipboard",
                    });
                }
                if let Some(mutation) = mutation {
                    update_store(ctx, STORE, move |v| mutation.apply(v)).await?;
                }
                session.apply_ui(id, payload)?;
            }
        }
        _ => return Err("Unsupported Unicode operation".into()),
    }
    let store = Store::read(&read_store(ctx, STORE).await?)?;
    let mut view = session.render_window(&store, Some((page, 150)));
    page = view
        .as_object_mut()
        .unwrap()
        .remove("_processPage")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    view["refreshMs"] = Value::Null;
    if let Some(message) = message {
        view["message"] = json!(match view["message"].as_str() {
            Some(status) => format!("{message} · {status}"),
            None => message.into(),
        });
    }
    save(state, &session)?;
    state["processPage"] = json!(page);
    Ok(view)
}
