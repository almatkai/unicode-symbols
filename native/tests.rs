use super::*;
use engine::Session;
use tezbar_extension_sdk::{read_store, update_store, ExtensionPaths, TestContext};
fn fixture(dir: &std::path::Path) -> TestContext {
    TestContext::new(
        ExtensionPaths {
            package_dir: dir.join("package"),
            data_dir: dir.join("data"),
            temp_dir: dir.join("tmp"),
        },
        &["clipboard.write", "clipboard.pasteText", "shell.open"],
    )
}
fn action(state: &Value, op: &str) -> String {
    let session: Session = serde_json::from_value(state.clone()).unwrap();
    session
        .bindings
        .iter()
        .find(|(_, b)| b.code == Some(38) && b.op == op)
        .unwrap()
        .0
        .clone()
}
async fn open(adapter: &Adapter, fixture: &TestContext) -> Value {
    let mut state = json!({});
    adapter.initialize(&fixture.context).await.unwrap();
    adapter
        .handle(
            &fixture.context,
            "index",
            "extension:run-command",
            &json!({"sessionId":"test","preferences":{"viewType":"list","primaryAction":"copy"}}),
            &mut state,
        )
        .await
        .unwrap();
    adapter
        .handle(
            &fixture.context,
            "index",
            "extension:search-text-changed",
            &json!({"sessionId":"test","searchText":"&"}),
            &mut state,
        )
        .await
        .unwrap();
    state
}
#[tokio::test]
async fn failure_is_visible_no_history_or_false_copy_success_and_id_can_retry() {
    let dir = tempfile::tempdir().unwrap();
    let fixture = fixture(dir.path());
    let adapter = Adapter::default();
    let mut state = open(&adapter, &fixture).await;
    let id = action(&state, "copy");
    let payload = json!({"sessionId":"test","actionId":id});
    let before = state.clone();
    fixture.reply(Err("Clipboard denied".into()));
    assert_eq!(
        fixture
            .run(adapter.handle(
                &fixture.context,
                "index",
                "extension:invoke-action",
                &payload,
                &mut state
            ))
            .await
            .unwrap_err(),
        "Clipboard denied"
    );
    assert_eq!(state, before);
    assert!(
        read_store(&fixture.context, process::STORE).await.unwrap()["recently-used-v2"].is_null()
    );
    fixture.reply(Ok(json!({"ok":true})));
    let view = fixture
        .run(adapter.handle(
            &fixture.context,
            "index",
            "extension:invoke-action",
            &payload,
            &mut state,
        ))
        .await
        .unwrap();
    assert_eq!(view["mode"], "view");
    assert!(view["root"].is_object());
    assert_eq!(view["message"], "Copied to clipboard");
    assert_eq!(
        read_store(&fixture.context, process::STORE).await.unwrap()["recently-used-v2"],
        json!([38])
    );
    assert_eq!(fixture.calls().len(), 2);
    assert_eq!(fixture.calls()[0]["method"], "host.clipboard.write");
    assert!(adapter
        .handle(
            &fixture.context,
            "index",
            "extension:invoke-action",
            &payload,
            &mut state
        )
        .await
        .unwrap_err()
        .contains("forged"));
}
#[tokio::test]
async fn malformed_ack_is_separate_from_real_host_shapes_and_paste_is_not_repeated() {
    let dir = tempfile::tempdir().unwrap();
    let fixture = fixture(dir.path());
    let adapter = Adapter::default();
    let mut state = open(&adapter, &fixture).await;
    for invalid in [
        Value::Null,
        json!({"ok":false}),
        json!({"ok":true,"error":"denied"}),
    ] {
        fixture.reply(Ok(invalid));
        let id = action(&state, "copy");
        assert!(fixture
            .run(adapter.handle(
                &fixture.context,
                "index",
                "extension:invoke-action",
                &json!({"sessionId":"test","actionId":id}),
                &mut state
            ))
            .await
            .is_err());
    }
    let id = action(&state, "paste");
    let payload = json!({"sessionId":"test","actionId":id});
    fixture.reply(Err("Transport lost".into()));
    assert!(fixture
        .run(adapter.handle(
            &fixture.context,
            "index",
            "extension:invoke-action",
            &payload,
            &mut state
        ))
        .await
        .is_err());
    let count = fixture.calls().len();
    assert!(adapter
        .handle(
            &fixture.context,
            "index",
            "extension:invoke-action",
            &payload,
            &mut state
        )
        .await
        .unwrap_err()
        .contains("uncertain"));
    assert_eq!(fixture.calls().len(), count);
}
#[tokio::test]
async fn acknowledged_paste_is_not_replayed_after_storage_failure_and_staged_state_rollback() {
    let dir = tempfile::tempdir().unwrap();
    let fixture = fixture(dir.path());
    let adapter = Adapter::default();
    let mut state = open(&adapter, &fixture).await;
    let before = state.clone();
    let id = action(&state, "paste");
    let payload = json!({"sessionId":"test","actionId":id});
    let storage = dir.path().join("data/unicode-symbols/storage.json");
    // Empty-source initialization deliberately no longer creates a private file.
    update_store(&fixture.context, process::STORE, |_| Ok(()))
        .await
        .unwrap();
    std::fs::remove_file(&storage).unwrap();
    std::fs::create_dir(&storage).unwrap();
    fixture.reply(Ok(json!({"ok":true,"message":"Pasted text"})));
    assert!(fixture
        .run(adapter.handle(
            &fixture.context,
            "index",
            "extension:invoke-action",
            &payload,
            &mut state
        ))
        .await
        .is_err());
    state = before; // runner discards staged state after errors
    std::fs::remove_dir(&storage).unwrap();
    let view = fixture
        .run(adapter.handle(
            &fixture.context,
            "index",
            "extension:invoke-action",
            &payload,
            &mut state,
        ))
        .await
        .unwrap();
    assert_eq!(view["mode"], "view");
    assert_eq!(fixture.calls().len(), 1);
}
#[tokio::test]
async fn dropped_paste_future_retains_uncertainty_outside_transactional_state() {
    let dir = tempfile::tempdir().unwrap();
    let fixture = fixture(dir.path());
    let adapter = Adapter::default();
    let mut state = open(&adapter, &fixture).await;
    let before = state.clone();
    let payload = json!({"sessionId":"test","actionId":action(&state,"paste")});
    // No stub pump: drive until reverse RPC waits, then drop as SDK cancellation does.
    assert!(tokio::time::timeout(
        std::time::Duration::from_millis(20),
        adapter.handle(
            &fixture.context,
            "index",
            "extension:invoke-action",
            &payload,
            &mut state
        )
    )
    .await
    .is_err());
    state = before;
    assert!(adapter
        .handle(
            &fixture.context,
            "index",
            "extension:invoke-action",
            &payload,
            &mut state
        )
        .await
        .unwrap_err()
        .contains("uncertain"));
    assert_eq!(fixture.calls().len(), 1);
}
#[tokio::test]
async fn snippet_gap_and_forged_ids_do_not_make_host_requests_or_mutate_state() {
    let dir = tempfile::tempdir().unwrap();
    let fixture = fixture(dir.path());
    let adapter = Adapter::default();
    let mut state = open(&adapter, &fixture).await;
    let before = state.clone();
    for id in [action(&state, "snippet"), "forged".into()] {
        assert!(adapter
            .handle(
                &fixture.context,
                "index",
                "extension:invoke-action",
                &json!({"sessionId":"test","actionId":id}),
                &mut state
            )
            .await
            .is_err());
        assert_eq!(state, before);
    }
    assert!(fixture.calls().is_empty());
}
#[tokio::test]
async fn migration_preserves_order_scores_private_state_sources_and_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let mut fixture = fixture(dir.path());
    let source = dir.path().join("legacy.json");
    let legacy = json!({"favorites-v1":json!([{"c":38,"score":0.25,"isExtra":true},{"c":65}]).to_string(),"recently-used-v2":json!([{"c":65}]).to_string()});
    std::fs::write(&source, serde_json::to_vec(&legacy).unwrap()).unwrap();
    fixture
        .context
        .migration_paths
        .insert("legacyLocalStorage".into(), source.clone());
    update_store(&fixture.context, process::STORE, |s| {
        s["recently-used-v2"] = json!([38]);
        Ok(())
    })
    .await
    .unwrap();
    let store = migration::initialize(&fixture.context).await.unwrap();
    assert_eq!(store["favorites-v1"], json!([38, 65]));
    assert_eq!(store["favorites-meta-v1"]["38"]["score"], 0.25);
    assert_eq!(store["favorites-meta-v1"]["38"]["extra"], true);
    assert_eq!(store["recently-used-v2"], json!([38]));
    assert_eq!(
        serde_json::from_slice::<Value>(&std::fs::read(&source).unwrap()).unwrap(),
        legacy
    );
    std::fs::write(&source, b"bad").unwrap();
    assert_eq!(
        migration::initialize(&fixture.context).await.unwrap(),
        store
    );
    assert!(fixture.calls().is_empty());
}
#[tokio::test]
async fn malformed_migration_does_not_publish_marker_or_partial_import() {
    let dir = tempfile::tempdir().unwrap();
    let mut fixture = fixture(dir.path());
    let path = dir.path().join("legacy");
    std::fs::write(&path, br#"{"favorites-v1":"[{\"c\":99999999}]"}"#).unwrap();
    fixture
        .context
        .migration_paths
        .insert("legacyLocalStorage".into(), path);
    assert!(migration::initialize(&fixture.context).await.is_err());
    assert_eq!(
        read_store(&fixture.context, process::STORE).await.unwrap(),
        json!({})
    );
}

#[tokio::test]
async fn legacy_unpaired_utf16_record_migrates_by_canonical_code_without_glyph_loss() {
    let dir = tempfile::tempdir().unwrap();
    let mut fixture = fixture(dir.path());
    let path = dir.path().join("legacy");
    let code = engine::dataset(true)
        .characters
        .iter()
        .find(|c| (0xd800..=0xdfff).contains(&c.c))
        .unwrap()
        .c;
    let legacy =
        json!({"favorites-v1":format!(r#"[{{"c":{code},"v":"\u{code:04x}","isExtra":true}}]"#)});
    std::fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();
    fixture
        .context
        .migration_paths
        .insert("legacyLocalStorage".into(), path);
    let store = migration::initialize(&fixture.context).await.unwrap();
    assert_eq!(store["favorites-v1"], json!([code]));
    assert_eq!(store["favorites-meta-v1"][code.to_string()]["extra"], true);
}

#[tokio::test]
async fn native_migration_has_precedence_and_does_not_import_orphan_legacy_metadata() {
    let dir = tempfile::tempdir().unwrap();
    let mut fixture = fixture(dir.path());
    for (name, value) in [
        ("nativeStore", json!({"favorites-v1":[38]})),
        (
            "legacyLocalStorage",
            json!({"favorites-v1":json!([{"c":65,"score":0.1}]).to_string(),"recently-used-v2":json!([{"c":65}]).to_string()}),
        ),
    ] {
        let path = dir.path().join(name);
        std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        fixture.context.migration_paths.insert(name.into(), path);
    }
    let store = migration::initialize(&fixture.context).await.unwrap();
    assert_eq!(store["favorites-v1"], json!([38]));
    assert_eq!(store["favorites-meta-v1"], json!({}));
    assert_eq!(store["recently-used-v2"], json!([65]));
}
