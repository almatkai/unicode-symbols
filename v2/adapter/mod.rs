//! Compile as a sibling of the real v2_sdk; never load this module dynamically.
#[allow(dead_code, unused_imports)]
#[path = "../src/lib.rs"]
mod engine;
use super::v2_sdk;
use crate::native_core::common::{CoreContext, CoreResult};
use engine::{Effect, Session, Store};
use serde_json::{json, Value};
const SLUG: &str = "unicode-symbols";
pub fn commands() -> &'static [&'static str] {
    &["index"]
}
fn save(state: &mut Value, session: &Session) -> Result<(), String> {
    *state = serde_json::to_value(session).map_err(|e| e.to_string())?;
    Ok(())
}
pub async fn handle(
    ctx: &CoreContext,
    command: &str,
    operation: &str,
    payload: &Value,
    state: &mut Value,
) -> CoreResult {
    if !commands().contains(&command) {
        return Err("Unknown Unicode command".into());
    }
    if !matches!(
        operation,
        "extension:run-command"
            | "extension:search-text-changed"
            | "extension:invoke-action"
            | "extension:refresh-session"
            | "extension:dispose-session"
    ) {
        return Err("Unsupported Unicode operation".into());
    }
    if operation == "extension:dispose-session" {
        *state = Value::Null;
        return Ok(v2_sdk::unchanged());
    }
    if operation == "extension:refresh-session" {
        return Ok(v2_sdk::unchanged());
    }
    if operation == "extension:run-command" {
        if state.as_object().is_none_or(|v| !v.is_empty()) {
            return Err("Unicode session already initialized or disposed".into());
        }
        let mut session = Session::new(payload)?;
        // index declares no launch arguments. argumentValues is intentionally ignored.
        let store = Store::read(&v2_sdk::read_store(ctx, SLUG).await?)?;
        let view = session.render(&store);
        save(state, &session)?;
        return Ok(view);
    }
    let mut session: Session =
        serde_json::from_value(state.clone()).map_err(|_| "Unicode session is not initialized")?;
    if operation == "extension:search-text-changed" {
        if session.snippet.is_some() {
            return Err("Search is unavailable in the snippet form".into());
        }
        session.query = payload["searchText"]
            .as_str()
            .ok_or("Missing searchText")?
            .into();
    } else {
        let id = payload["actionId"].as_str().ok_or("Missing actionId")?;
        let binding = session.binding(id)?.clone();
        let field = binding.op.starts_with("draft-");
        let (effect, mutation) = session.prepare(id, payload)?;
        if let Some(effect) = effect {
            if binding.phase != "done" {
                session.bindings.get_mut(id).unwrap().phase = "uncertain".into();
                save(state, &session)?;
                let copy = matches!(effect, Effect::Copy(_));
                let snippet = matches!(effect, Effect::Snippet { .. });
                if let Effect::Snippet {
                    ref label,
                    ref trigger,
                    ..
                } = effect
                {
                    session.drafts.insert(
                        binding.code.ok_or("Missing snippet code")?,
                        json!({"label":label,"trigger":trigger}),
                    );
                    save(state, &session)?;
                }
                let response = match effect {
                    Effect::Copy(text) => v2_sdk::copy(ctx, &text).await,
                    Effect::Paste(text) => {
                        ctx.request("core:platform:paste-text", vec![json!(text)])
                            .await
                    }
                    Effect::Open(target) => v2_sdk::open(ctx, &target).await,
                    Effect::Snippet {
                        label,
                        trigger,
                        body,
                    } => {
                        ctx.request(
                            "snippets:add",
                            vec![json!({"label":label,"trigger":trigger,"body":body})],
                        )
                        .await
                    }
                };
                // The pinned snippets:add validator returns ok:false before mutation.
                // A rejected trigger can be corrected without treating it as an OS retry.
                let rejected_snippet = snippet
                    && response.as_ref().is_ok_and(|v| {
                        v.is_object()
                            && v["ok"] == false
                            && v.get("success").is_none_or(|s| s == false)
                    });
                let result = response.and_then(|v| engine::acknowledge(&v));
                if let Err(error) = result {
                    if copy || rejected_snippet {
                        session.bindings.get_mut(id).unwrap().phase = String::new();
                        save(state, &session)?;
                    }
                    return Err(error);
                }
                session.bindings.get_mut(id).unwrap().phase = "done".into();
                save(state, &session)?;
            }
        }
        if let Some(mutation) = mutation {
            v2_sdk::update_store(ctx, SLUG, move |v| mutation.apply(v)).await?;
        }
        session.apply_ui(id, payload)?;
        if field {
            save(state, &session)?;
            return Ok(v2_sdk::unchanged());
        }
    }
    let store = Store::read(&v2_sdk::read_store(ctx, SLUG).await?)?;
    let view = session.render(&store);
    save(state, &session)?;
    Ok(view)
}
