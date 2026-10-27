use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use unicode_beta_contract::native_core::{
    common::{CoreContext, CorePaths},
    extensions::*,
};
type Calls = Arc<Mutex<Vec<(String, Vec<Value>)>>>;
fn fixture(response: Result<Value, String>) -> (tempfile::TempDir, CoreContext, Calls) {
    let dir = tempfile::tempdir().unwrap();
    let ctx = CoreContext::new(
        CorePaths {
            data_dir: dir.path().join("data"),
            config_dir: dir.path().join("config"),
            resource_dir: dir.path().join("resources"),
            temp_dir: dir.path().join("tmp"),
            home_dir: dir.path().join("home"),
        },
        None,
    );
    std::fs::create_dir_all(&ctx.paths.data_dir).unwrap();
    std::fs::write(
        ctx.paths.data_dir.join("installed.json"),
        br#"{"preferences":{"primaryAction":"copy","searchResultLimitStr":"3"}}"#,
    )
    .unwrap();
    let calls: Calls = Arc::default();
    let record = calls.clone();
    ctx.set_router(Arc::new(move |c, a| {
        record.lock().unwrap().push((c, a));
        let response = response.clone();
        Box::pin(async move { response })
    }))
    .unwrap();
    (dir, ctx, calls)
}
fn action(view: &Value, title: &str) -> String {
    view["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["title"] == title)
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .into()
}
async fn launch(ctx: &CoreContext, s: &Sessions) -> Value {
    s.call(ctx,"extension:run-command",json!({"extensionId":"com.tezbar.native.unicode-symbols","commandName":"index","argumentValues":{"query":"ignored"}})).await.unwrap()
}
#[tokio::test]
async fn real_beta_session_preferences_registration_refresh_dispose_uninstall() {
    let (_d, ctx, calls) = fixture(Ok(json!({"ok":true})));
    let sessions = Sessions::default();
    for id in [
        "unicode-symbols",
        "raycast.unicode-symbols",
        "com.tezbar.native.unicode-symbols",
    ] {
        assert!(registered(id, "index"));
        assert!(!registered(id, "unknown"));
    }
    let view = launch(&ctx, &sessions).await;
    assert_eq!(view["root"]["type"], "Grid");
    assert_eq!(view["root"]["props"]["searchText"], "");
    let sid = &view["sessionId"];
    let id = action(&view, "Copy Character to Clipboard");
    assert_eq!(
        sessions
            .call(&ctx, "extension:refresh-session", json!({"sessionId":sid}))
            .await
            .unwrap()["mode"],
        "unchanged"
    );
    let result = sessions
        .call(
            &ctx,
            "extension:invoke-action",
            json!({"sessionId":sid,"actionId":id,"text":"forged","url":"https://evil.invalid"}),
        )
        .await
        .unwrap();
    assert_eq!(result["ok"], true);
    assert_eq!(
        *calls.lock().unwrap(),
        vec![("clipboard:write".into(), vec![json!("\t")])]
    );
    assert_eq!(
        sessions
            .call(
                &ctx,
                "extension:invoke-action",
                json!({"sessionId":sid,"actionId":id})
            )
            .await
            .unwrap()["ok"],
        false
    );
    let other = launch(&ctx, &sessions).await;
    assert_eq!(sessions.call(&ctx,"extension:invoke-action",json!({"sessionId":other["sessionId"],"actionId":action(&result,"Copy Character to Clipboard")})).await.unwrap()["ok"],false);
    std::fs::remove_file(ctx.paths.data_dir.join("installed.json")).unwrap();
    assert_eq!(
        sessions
            .call(
                &ctx,
                "extension:invoke-action",
                json!({"sessionId":sid,"actionId":action(&result,"Copy Character to Clipboard")})
            )
            .await
            .unwrap()["ok"],
        false
    );
    assert_eq!(
        sessions
            .call(
                &ctx,
                "extension:dispose-session",
                json!({"sessionId":other["sessionId"]})
            )
            .await
            .unwrap(),
        json!(true)
    );
    assert_eq!(
        sessions
            .call(
                &ctx,
                "extension:refresh-session",
                json!({"sessionId":other["sessionId"]})
            )
            .await
            .unwrap()["ok"],
        false
    );
    sessions.clear().await;
}
#[tokio::test]
async fn strict_acknowledgements_and_safe_copy_retries() {
    for response in [
        Ok(Value::Null),
        Ok(json!(true)),
        Ok(json!({"ok":false})),
        Ok(json!({"ok":true,"error":"denied"})),
        Ok(json!({"ok":true,"success":false})),
        Ok(json!({"ok":true,"success":null})),
        Err("transport failure".into()),
    ] {
        let (_d, ctx, calls) = fixture(response);
        let mut state = json!({});
        let view = adapter(
            &ctx,
            "index",
            "extension:run-command",
            &json!({}),
            &mut state,
        )
        .await
        .unwrap();
        let id = action(&view, "Copy Character to Clipboard");
        for _ in 0..2 {
            assert!(adapter(
                &ctx,
                "index",
                "extension:invoke-action",
                &json!({"actionId":id}),
                &mut state
            )
            .await
            .is_err());
        }
        assert_eq!(calls.lock().unwrap().len(), 2);
        assert!(read_store(&ctx, "unicode-symbols")
            .await
            .unwrap()
            .get("recently-used-v2")
            .is_none());
    }
}
#[tokio::test]
async fn paste_and_open_never_blindly_repeat_uncertain_effects() {
    for title in [
        "Paste Character to Active App",
        "Open Character on Compart",
        "Open Character on Unicode Explorer",
    ] {
        let (_d, ctx, calls) = fixture(Err("transport after partial effect".into()));
        let mut state = json!({});
        let view = adapter(
            &ctx,
            "index",
            "extension:run-command",
            &json!({}),
            &mut state,
        )
        .await
        .unwrap();
        let id = action(&view, title);
        assert!(adapter(
            &ctx,
            "index",
            "extension:invoke-action",
            &json!({"actionId":id}),
            &mut state
        )
        .await
        .is_err());
        assert!(adapter(
            &ctx,
            "index",
            "extension:invoke-action",
            &json!({"actionId":id}),
            &mut state
        )
        .await
        .unwrap_err()
        .contains("uncertain"));
        assert_eq!(calls.lock().unwrap().len(), 1);
    }
}
#[tokio::test]
async fn successful_effect_storage_failure_resumes_without_repeating_os_effect() {
    let (_d, ctx, calls) = fixture(Ok(json!({"ok":true,"error":null,"success":true})));
    let mut state = json!({});
    let view = adapter(
        &ctx,
        "index",
        "extension:run-command",
        &json!({}),
        &mut state,
    )
    .await
    .unwrap();
    let id = action(&view, "Copy Character to Clipboard");
    let dir = ctx
        .paths
        .data_dir
        .join("native-extensions-v2/unicode-symbols");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("storage.json");
    std::fs::write(&path, b"{broken").unwrap();
    assert!(adapter(
        &ctx,
        "index",
        "extension:invoke-action",
        &json!({"actionId":id}),
        &mut state
    )
    .await
    .is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"{broken");
    std::fs::write(&path, br#"{"unknown":{"keep":true}}"#).unwrap();
    adapter(
        &ctx,
        "index",
        "extension:invoke-action",
        &json!({"actionId":id}),
        &mut state,
    )
    .await
    .unwrap();
    assert_eq!(calls.lock().unwrap().len(), 1);
    let store = read_store(&ctx, "unicode-symbols").await.unwrap();
    assert_eq!(store["unknown"]["keep"], true);
    assert_eq!(store["recently-used-v2"], json!([9]));
}
#[tokio::test]
async fn stale_discarded_search_invalid_operations_refresh_and_drafts() {
    let (_d, ctx, calls) = fixture(Ok(json!({"ok":true})));
    let mut state = json!({});
    let first = adapter(
        &ctx,
        "index",
        "extension:run-command",
        &json!({"preferences":{"searchResultLimitStr":"3"}}),
        &mut state,
    )
    .await
    .unwrap();
    let id = action(&first, "Copy Character to Clipboard");
    let committed = state.clone();
    let mut discarded = state.clone();
    let discarded_view = adapter(
        &ctx,
        "index",
        "extension:search-text-changed",
        &json!({"searchText":"arrow"}),
        &mut discarded,
    )
    .await
    .unwrap();
    let mut heartbeat = state.clone();
    assert_eq!(
        adapter(
            &ctx,
            "index",
            "extension:refresh-session",
            &json!({}),
            &mut heartbeat
        )
        .await
        .unwrap()["mode"],
        "unchanged"
    );
    assert_eq!(heartbeat, state);
    for (op, p) in [
        ("unsupported", json!({"actionId":id})),
        ("extension:invoke-action", json!({"actionId":"forged"})),
        ("extension:search-text-changed", json!({"searchText":42})),
        (
            "extension:invoke-action",
            json!({"actionId":action(&discarded_view,"Copy Character to Clipboard")}),
        ),
    ] {
        assert!(adapter(&ctx, "index", op, &p, &mut state).await.is_err());
        assert_eq!(state, committed);
    }
    let v = adapter(
        &ctx,
        "index",
        "extension:search-text-changed",
        &json!({"searchText":"arrow"}),
        &mut state,
    )
    .await
    .unwrap();
    assert_ne!(v["actions"], discarded_view["actions"]);
    let form = adapter(
        &ctx,
        "index",
        "extension:invoke-action",
        &json!({"actionId":action(&v,"Save as Snippet")}),
        &mut state,
    )
    .await
    .unwrap();
    let trigger = action(&form, "Edit Trigger");
    adapter(
        &ctx,
        "index",
        "extension:invoke-action",
        &json!({"actionId":trigger,"formValues":{"value":";arrow"}}),
        &mut state,
    )
    .await
    .unwrap();
    let back = adapter(
        &ctx,
        "index",
        "extension:invoke-action",
        &json!({"actionId":action(&form,"Back")}),
        &mut state,
    )
    .await
    .unwrap();
    let again = adapter(
        &ctx,
        "index",
        "extension:invoke-action",
        &json!({"actionId":action(&back,"Save as Snippet")}),
        &mut state,
    )
    .await
    .unwrap();
    assert_eq!(again["root"]["children"][1]["props"]["value"], ";arrow");
    adapter(
        &ctx,
        "index",
        "extension:invoke-action",
        &json!({"actionId":action(&again,"Save as Snippet")}),
        &mut state,
    )
    .await
    .unwrap();
    assert_eq!(calls.lock().unwrap()[0].0, "snippets:add");
    assert_eq!(calls.lock().unwrap()[0].1[0]["trigger"], ";arrow");
    adapter(
        &ctx,
        "index",
        "extension:dispose-session",
        &json!({}),
        &mut state,
    )
    .await
    .unwrap();
    assert!(adapter(
        &ctx,
        "index",
        "extension:invoke-action",
        &json!({"actionId":id}),
        &mut state
    )
    .await
    .is_err());
}
#[tokio::test]
async fn sdk_atomic_transactions_preserve_unrelated_keys_and_reject_corruption() {
    let (_d, ctx, _) = fixture(Ok(json!({"ok":true})));
    update_store(&ctx, "unicode-symbols", |v| {
        v["foreign"] = json!({"keep":true});
        v["counter"] = json!(0);
        Ok(())
    })
    .await
    .unwrap();
    let mut jobs = Vec::new();
    for _ in 0..20 {
        let ctx = ctx.clone();
        jobs.push(tokio::spawn(async move {
            update_store(&ctx, "unicode-symbols", |v| {
                v["counter"] = json!(v["counter"].as_u64().unwrap() + 1);
                Ok(())
            })
            .await
            .unwrap()
        }));
    }
    for j in jobs {
        j.await.unwrap();
    }
    let value = read_store(&ctx, "unicode-symbols").await.unwrap();
    assert_eq!(value["counter"], 20);
    assert_eq!(value["foreign"]["keep"], true);
}
#[tokio::test]
async fn cancelled_effect_records_uncertainty_and_real_session_disposal_cancels_request() {
    let (_d, original, _) = fixture(Ok(json!({"ok":true})));
    let ctx = CoreContext::new(original.paths.clone(), None);
    let entered = Arc::new(tokio::sync::Notify::new());
    let notify = entered.clone();
    ctx.set_router(Arc::new(move |channel, _| {
        assert_eq!(channel, "core:platform:paste-text");
        let notify = notify.clone();
        Box::pin(async move {
            notify.notify_one();
            std::future::pending().await
        })
    }))
    .unwrap();
    let sessions = Sessions::default();
    let view = launch(&ctx, &sessions).await;
    let call = sessions.clone();
    let cx = ctx.clone();
    let sid = view["sessionId"].clone();
    let body = json!({"sessionId":sid,"actionId":action(&view,"Paste Character to Active App")});
    let task = tokio::spawn(async move {
        call.call(&cx, "extension:invoke-action", body)
            .await
            .unwrap()
    });
    entered.notified().await;
    assert_eq!(
        sessions
            .call(&ctx, "extension:refresh-session", json!({"sessionId":sid}))
            .await
            .unwrap()["mode"],
        "unchanged"
    );
    sessions
        .call(&ctx, "extension:dispose-session", json!({"sessionId":sid}))
        .await
        .unwrap();
    let result = tokio::time::timeout(std::time::Duration::from_secs(2), task)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result["ok"], false);
}
#[tokio::test]
async fn malformed_filter_and_snippet_validation_preserve_legitimate_ids() {
    let (_d, ctx, calls) = fixture(Ok(json!({"ok":true})));
    let mut state = json!({});
    let view = adapter(
        &ctx,
        "index",
        "extension:run-command",
        &json!({"preferences":{"searchResultLimitStr":"3"}}),
        &mut state,
    )
    .await
    .unwrap();
    let filter = action(&view, "Select a Character Set");
    let before = state.clone();
    assert!(adapter(
        &ctx,
        "index",
        "extension:invoke-action",
        &json!({"actionId":filter,"formValues":{"value":"evil"}}),
        &mut state
    )
    .await
    .is_err());
    assert_eq!(state, before);
    let view = adapter(
        &ctx,
        "index",
        "extension:invoke-action",
        &json!({"actionId":filter,"formValues":{"value":"Arrows"}}),
        &mut state,
    )
    .await
    .unwrap();
    let form = adapter(
        &ctx,
        "index",
        "extension:invoke-action",
        &json!({"actionId":action(&view,"Save as Snippet")}),
        &mut state,
    )
    .await
    .unwrap();
    let submit = action(&form, "Save as Snippet");
    let before = state.clone();
    assert!(adapter(
        &ctx,
        "index",
        "extension:invoke-action",
        &json!({"actionId":submit,"formValues":{"label":"x","trigger":""}}),
        &mut state
    )
    .await
    .is_err());
    assert_eq!(state, before);
    assert!(calls.lock().unwrap().is_empty());
    adapter(
        &ctx,
        "index",
        "extension:invoke-action",
        &json!({"actionId":submit,"formValues":{"label":"Arrow","trigger":";a","body":"forged"}}),
        &mut state,
    )
    .await
    .unwrap();
    assert_eq!(
        calls.lock().unwrap()[0],
        (
            "snippets:add".into(),
            vec![json!({"label":"Arrow","trigger":";a","body":"←"})]
        )
    );
}

#[tokio::test]
async fn snippet_definite_rejections_allow_correction_but_malformed_acks_do_not_repeat() {
    for (response, expected_calls) in [
        (
            json!({"ok":false,"message":"Another snippet already uses this trigger"}),
            2,
        ),
        (json!({"ok":true,"success":false}), 1),
        (Value::Null, 1),
    ] {
        let (_d, ctx, calls) = fixture(Ok(response));
        let mut state = json!({});
        adapter(
            &ctx,
            "index",
            "extension:run-command",
            &json!({}),
            &mut state,
        )
        .await
        .unwrap();
        let view = adapter(
            &ctx,
            "index",
            "extension:search-text-changed",
            &json!({"searchText":"65"}),
            &mut state,
        )
        .await
        .unwrap();
        let form = adapter(
            &ctx,
            "index",
            "extension:invoke-action",
            &json!({"actionId":action(&view,"Save as Snippet")}),
            &mut state,
        )
        .await
        .unwrap();
        let id = action(&form, "Save as Snippet");
        for trigger in [";a", ";corrected"] {
            assert!(adapter(
                &ctx,
                "index",
                "extension:invoke-action",
                &json!({"actionId":id,"formValues":{"label":"A","trigger":trigger}}),
                &mut state
            )
            .await
            .is_err());
        }
        assert_eq!(calls.lock().unwrap().len(), expected_calls);
    }
}
