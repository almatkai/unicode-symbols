//! Only explicit host-granted files are read; no profile directory scanning.
use crate::process::STORE;
use engine::{ItemMeta, Store};
use serde_json::{json, Value};
use tezbar_extension_sdk::{read_store, update_store, CoreResult, ExtensionContext};
fn convert_legacy(value: &Value, owned: &Value) -> CoreResult {
    if !value.is_object() {
        return Err("Legacy Unicode storage must be an object".into());
    }
    let mut result = json!({});
    for (key, meta_key, limit) in [
        ("favorites-v1", "favorites-meta-v1", 50),
        ("recently-used-v2", "recently-used-meta-v2", 10),
    ] {
        if owned.get(key).is_some() {
            continue;
        }
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
                // Metadata can already be owned even when the list is absent.
                target
                    .entry(meta)
                    .or_insert_with(|| source.get(meta).cloned().unwrap_or(json!({})));
            }
        }
    }
    Ok(())
}
const MARKER: &str = "_processMigrationV2";
const SOURCES: &[&str] = &[
    "nativeStore",
    "legacyLocalStorage",
    "legacyRaycastLocalStorage",
    "legacyPackageLocalStorage",
    "legacyRegistryLocalStorage",
    "legacyBarePackageLocalStorage",
    "legacyBareRegistryLocalStorage",
    "legacyShimStorage",
    "legacyRaycastShimStorage",
    "legacyPackageShimStorage",
    "legacyRegistryShimStorage",
    "legacyBarePackageShimStorage",
    "legacyBareRegistryShimStorage",
];
fn complete(store: &Value) -> bool {
    store.get("favorites-v1").is_some() && store.get("recently-used-v2").is_some()
}
fn select_native(value: &Value, owned: &Value) -> CoreResult {
    if !value.is_object() {
        return Err("Native storage must be an object".into());
    }
    let mut selected = json!({});
    for (key, meta) in [
        ("favorites-v1", "favorites-meta-v1"),
        ("recently-used-v2", "recently-used-meta-v2"),
    ] {
        if owned.get(key).is_none() {
            if let Some(list) = value.get(key) {
                selected[key] = list.clone();
                if owned.get(meta).is_none() {
                    if let Some(metadata) = value.get(meta) {
                        selected[meta] = metadata.clone();
                    }
                }
            }
        }
    }
    Store::read(&selected)?;
    Ok(selected)
}
fn commit_import(store: &mut Value, incoming: &Value) -> Result<(), String> {
    // Called inside the SDK atomic lane, after rereading the latest private model.
    // Never trust the earlier ownership snapshot or the defective V1 marker.
    Store::read(store)?;
    if complete(store) {
        return Ok(());
    }
    merge(store, incoming)?;
    Store::read(store)?;
    // This means both lists are now owned, not merely "a grant was considered".
    // Partial/absent sources remain retryable; the lists themselves fence reimport.
    if complete(store) && store.get(MARKER).is_none() {
        store[MARKER] = json!(true);
    }
    Ok(())
}
pub async fn initialize(ctx: &ExtensionContext) -> CoreResult {
    let current = read_store(ctx, STORE).await?;
    Store::read(&current)?;
    // A complete private model never needs stale legacy files, even without a marker.
    if complete(&current) {
        return Ok(current);
    }
    let mut candidate = current.clone();
    // Native v2, then ALL view files, then ALL older shim files. Within each
    // shim family: JS resolver order first, then the remaining managed bare roots.
    for name in SOURCES {
        if complete(&candidate) {
            break;
        }
        let Some(path) = ctx.migration_paths.get(*name) else {
            continue;
        };
        let bytes = match tokio::fs::read(path).await {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(format!("Cannot read {name}: {e}")),
        };
        let value: Value =
            serde_json::from_slice(&bytes).map_err(|e| format!("Invalid {name}: {e}"))?;
        // Ignore already-owned domains, including their stale malformed metadata.
        let selected = if *name == "nativeStore" {
            select_native(&value, &candidate)?
        } else {
            convert_legacy(&value, &candidate)?
        };
        merge(&mut candidate, &selected)?;
        Store::read(&candidate)?;
    }
    if candidate == current {
        // No useful source: no file creation or completion marker.
        return Ok(current);
    }
    update_store(ctx, STORE, move |store| commit_import(store, &candidate)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tezbar_extension_sdk::{ExtensionPaths, TestContext};

    #[tokio::test]
    async fn atomic_lane_rechecks_lists_metadata_and_preserves_concurrent_unknown_keys() {
        let dir = tempfile::tempdir().unwrap();
        let fixture = TestContext::new(
            ExtensionPaths {
                package_dir: dir.path().join("package"),
                data_dir: dir.path().join("data"),
                temp_dir: dir.path().join("temp"),
            },
            &[],
        );
        // A migration prepared this snapshot before another SDK writer took ownership.
        let incoming = json!({
            "favorites-v1":[38], "favorites-meta-v1":{"38":{"score":0.5,"extra":true}},
            "recently-used-v2":[65], "recently-used-meta-v2":{"65":{"score":0.1,"extra":true}}
        });
        let latest = json!({
            "_processMigrationV1":true, "favorites-v1":[], "favorites-meta-v1":{},
            "recently-used-meta-v2":{}, "concurrentUnknown":{"preserve":true}
        });
        let seed = latest.clone();
        update_store(&fixture.context, STORE, move |store| {
            *store = seed;
            Ok(())
        })
        .await
        .unwrap();
        let result = update_store(&fixture.context, STORE, move |store| {
            commit_import(store, &incoming)
        })
        .await
        .unwrap();
        for (key, value) in latest.as_object().unwrap() {
            assert_eq!(&result[key], value);
        }
        assert_eq!(result["recently-used-v2"], json!([65]));
        assert_eq!(result[MARKER], true);
        // A second pending import cannot replace even intentionally empty lists.
        let before = result.clone();
        let result = update_store(&fixture.context, STORE, move |store| {
            commit_import(store, &json!({"favorites-v1":[66],"recently-used-v2":[66]}))
        })
        .await
        .unwrap();
        assert_eq!(result, before);
    }
}
