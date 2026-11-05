//! Only explicit host-granted files are read; no profile directory scanning.
use crate::process::STORE;
use engine::{ItemMeta, Store};
use serde_json::{json, Value};
use tezbar_extension_sdk::{read_store, update_store, CoreResult, ExtensionContext};
fn convert_legacy(value: &Value) -> CoreResult {
    if !value.is_object() {
        return Err("Legacy Unicode storage must be an object".into());
    }
    let mut result = json!({});
    for (key, meta_key, limit) in [
        ("favorites-v1", "favorites-meta-v1", 50),
        ("recently-used-v2", "recently-used-meta-v2", 10),
    ] {
        let Some(raw) = value.get(key) else {
            continue;
        };
        // Parse raw records so the six source unpaired UTF-16 values can migrate
        // by canonical numeric identity without decoding their invalid UTF-8 glyphs.
        let rows: Vec<std::collections::HashMap<String, Box<serde_json::value::RawValue>>> =
            match raw {
                Value::String(s) => {
                    serde_json::from_str(s).map_err(|e| format!("Invalid legacy {key}: {e}"))?
                }
                _ => return Err(format!("Legacy {key} must be JSON text")),
            };
        if rows.len() > limit {
            return Err(format!("Legacy {key} exceeds original limit"));
        }
        let mut codes = Vec::new();
        let mut meta = serde_json::Map::new();
        for row in rows {
            let code: u32 =
                serde_json::from_str(row.get("c").ok_or("Missing legacy character code")?.get())
                    .map_err(|_| "Invalid legacy Unicode character")?;
            if engine::canonical(code).is_none() {
                return Err("Invalid legacy Unicode character".into());
            }
            if codes.contains(&code) {
                return Err("Duplicate legacy Unicode character".into());
            }
            codes.push(code);
            let score: Option<f64> = row
                .get("score")
                .map(|v| serde_json::from_str(v.get()).map_err(|_| "Invalid legacy Unicode score"))
                .transpose()?
                .flatten();
            let extra: bool = row
                .get("isExtra")
                .map(|v| {
                    serde_json::from_str(v.get()).map_err(|_| "Invalid legacy Unicode isExtra")
                })
                .transpose()?
                .unwrap_or(false);
            meta.insert(code.to_string(), json!(ItemMeta { score, extra }));
        }
        result[key] = json!(codes);
        result[meta_key] = json!(meta);
    }
    Store::read(&result)?;
    Ok(result)
}
fn merge(target: &mut Value, source: &Value) -> Result<(), String> {
    let target = target
        .as_object_mut()
        .ok_or("Private store must be object")?;
    for (key, meta) in [
        ("favorites-v1", "favorites-meta-v1"),
        ("recently-used-v2", "recently-used-meta-v2"),
    ] {
        if !target.contains_key(key) {
            if let Some(value) = source.get(key) {
                target.insert(key.into(), value.clone());
                target.insert(meta.into(), source.get(meta).cloned().unwrap_or(json!({})));
            }
        }
    }
    Ok(())
}
pub async fn initialize(ctx: &ExtensionContext) -> CoreResult {
    let current = read_store(ctx, STORE).await?;
    Store::read(&current)?;
    if current["_processMigrationV1"] == true {
        return Ok(current);
    }
    let mut incoming = json!({});
    // Native v2 wins, then view shim storage, then the older no-view shim.
    for name in [
        "nativeStore",
        "legacyLocalStorage",
        "legacyPackageLocalStorage",
        "legacyShimStorage",
        "legacyPackageShimStorage",
    ] {
        let Some(path) = ctx.migration_paths.get(name) else {
            continue;
        };
        let bytes = match tokio::fs::read(path).await {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(format!("Cannot read {name}: {e}")),
        };
        let value: Value =
            serde_json::from_slice(&bytes).map_err(|e| format!("Invalid {name}: {e}"))?;
        let value = if name == "nativeStore" {
            Store::read(&value)?;
            value
        } else {
            convert_legacy(&value)?
        };
        merge(&mut incoming, &value)?;
    }
    update_store(ctx, STORE, move |store| {
        // Recheck inside the SDK's atomic update lane. Existing private state wins.
        if store["_processMigrationV1"] == true {
            return Ok(());
        }
        merge(store, &incoming)?;
        Store::read(store)?;
        store["_processMigrationV1"] = json!(true);
        Ok(())
    })
    .await
}
