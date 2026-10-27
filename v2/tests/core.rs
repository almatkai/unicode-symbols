use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use tezbar_unicode_symbols::*;
fn id(session: &Session, code: u32, op: &str) -> String {
    session
        .bindings
        .iter()
        .find(|(_, b)| b.code == Some(code) && b.op == op)
        .unwrap()
        .0
        .clone()
}
#[test]
fn executed_original_oracle_cases() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("oracle-ranks.json")).unwrap();
    assert_eq!(cases.len(), 499);
    let mut cache = HashMap::<String, SearchResult>::new();
    for case in cases {
        let p = &case["request"];
        let prefs = Preferences::from_payload(p).unwrap();
        assert_eq!(
            prefs,
            serde_json::from_value::<Preferences>(case["preferences"].clone()).unwrap()
        );
        let d = dataset(prefs.full);
        let q = p["query"].as_str().unwrap_or("");
        let filter = p["filter"].as_str();
        let key = format!("{}:{}:{q}:{filter:?}", prefs.full, prefs.limit);
        let result = cache
            .entry(key)
            .or_insert_with(|| search(d, &prefs, q, filter));
        let expected = case["characters"].as_array().unwrap();
        assert_eq!(result.characters.len(), expected.len(), "{p}");
        for ((c, score), e) in result.characters.iter().zip(expected) {
            assert_eq!(*c as u64, e[0].as_u64().unwrap(), "{p}");
            match (score, e[1].as_f64()) {
                (Some(a), Some(b)) => assert!((a - b).abs() < 1e-11, "{p}: {a} != {b}"),
                (None, None) => {}
                _ => panic!("score shape"),
            }
        }
        let codes = |key: &str| {
            serde_json::from_value::<Vec<u32>>(p.get(key).cloned().unwrap_or(json!([]))).unwrap()
        };
        let sections = sections(d, result, q, filter, &codes("favorites"), &codes("recent"));
        let expected: Vec<SectionFixture> =
            serde_json::from_value(case["sections"].clone()).unwrap();
        for (a, b) in sections.iter().zip(expected) {
            assert_eq!(a.title, b.title);
            assert_eq!(a.items.len(), b.items.len());
            assert!((a.lowest - b.lowest).abs() < 1e-11);
            for (x, y) in a.items.iter().zip(b.items) {
                assert_eq!(x.0, y.0);
                assert_eq!(x.2, y.2);
            }
        }
    }
}
#[derive(serde::Deserialize)]
struct SectionFixture {
    title: String,
    lowest: f64,
    items: Vec<(u32, Option<f64>, bool)>,
}
#[test]
fn data_surrogates_and_last_html_entity_are_preserved() {
    assert_eq!(dataset(false).characters.len(), 5811);
    assert_eq!(dataset(true).characters.len(), 40510);
    assert_eq!(dataset(false).blocks.len(), 41);
    assert_eq!(dataset(true).blocks.len(), 346);
    let mut bad = 0;
    for c in &dataset(true).characters {
        let encoded = serde_json::to_string(c).unwrap();
        let decoded: Character = serde_json::from_str(&encoded).unwrap();
        assert_eq!(c.c, decoded.c);
        assert_eq!(c.v, decoded.v);
        if (0xd800..=0xdfff).contains(&c.c) {
            bad += 1;
            assert!(character_text(c).is_err());
            assert!(encoded.contains(&format!("\\u{:04x}", c.c)));
        }
    }
    assert_eq!(bad, 6);
    assert_eq!(html_entity(38), Some("&amp;"));
    assert_eq!(html_entity(34), Some("&quot;"));
}
#[test]
fn original_action_effects_and_recent_callbacks() {
    let fixture: Value = serde_json::from_str(include_str!("original-actions.json")).unwrap();
    for panel in fixture["actionFixtures"].as_array().unwrap() {
        let c = panel["code"].as_u64().unwrap() as u32;
        let mut session =
            Session::new(&json!({"preferences":{"dataSet":"full","primaryAction":"copy"}}))
                .unwrap();
        session.query = c.to_string();
        let view = session.render(&Store::default());
        let row = view["root"]["children"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|s| s["children"].as_array().unwrap())
            .find(|r| {
                r["props"]["id"]
                    .as_str()
                    .unwrap()
                    .ends_with(&format!(":{c}"))
            })
            .unwrap();
        assert_eq!(
            row["props"]["content"]["source"]["light"],
            panel["formatting"][if std::env::var("RAYCASTX").as_deref() == Ok("1") {
                "1"
            } else {
                "0"
            }]["light"]
        );
        assert_eq!(
            row["props"]["content"]["source"]["dark"],
            panel["formatting"][if std::env::var("RAYCASTX").as_deref() == Ok("1") {
                "1"
            } else {
                "0"
            }]["dark"]
        );
        let mut copies = Vec::new();
        let mut opens = Vec::new();
        for action in panel["actions"].as_array().unwrap() {
            match action["type"].as_str().unwrap() {
                "copy" => copies.push((action["content"].as_str().unwrap(), action.get("onCopy"))),
                "open" => opens.push(action["url"].as_str().unwrap()),
                _ => {}
            }
        }
        let ops: Vec<_> = session
            .bindings
            .iter()
            .filter(|(_, b)| b.code == Some(c))
            .map(|(id, b)| (id, b.op.as_str()))
            .collect();
        for content in copies {
            assert!(ops.iter().any(|(id, _)| session
                .prepare(id, &json!({}))
                .is_ok_and(|(e, _)| e == Some(Effect::Copy(content.0.into())))));
        }
        for url in opens {
            assert!(ops.iter().any(|(id, _)| session
                .prepare(id, &json!({"url":"https://forged.invalid"}))
                .is_ok_and(|(e, _)| e == Some(Effect::Open(url.into())))));
        }
        for op in ["copy", "paste", "hex", "html"] {
            if let Some((id, _)) = ops.iter().find(|(_, o)| *o == op) {
                assert!(session.prepare(id, &json!({})).unwrap().1.is_some());
            }
        }
        for op in ["escape", "decimal", "compart", "explorer"] {
            let action = id(&session, c, op);
            assert!(session.prepare(&action, &json!({})).unwrap().1.is_none());
        }
    }
}
#[test]
fn favorites_recent_order_limits_and_metadata_match_source_hooks() {
    let mut value = json!({"unrelated":{"keep":true}});
    let meta = ItemMeta {
        score: Some(-1.0),
        extra: true,
    };
    for c in [65, 66, 65, 67] {
        Mutation::AddFavorite(c, meta).apply(&mut value).unwrap();
    }
    assert_eq!(Store::read(&value).unwrap().favorites, vec![67, 66, 65]);
    for c in 65..200 {
        if canonical(c).is_some() {
            Mutation::AddFavorite(c, meta).apply(&mut value).unwrap();
            Mutation::AddRecent(c, meta).apply(&mut value).unwrap();
        }
    }
    let store = Store::read(&value).unwrap();
    assert_eq!(store.favorites.len(), 50);
    assert_eq!(store.recent.len(), 10);
    assert_eq!(store.favorite_meta[&199], meta);
    assert_eq!(value["unrelated"]["keep"], true);
    Mutation::RemoveFavorite(199).apply(&mut value).unwrap();
    assert!(!Store::read(&value)
        .unwrap()
        .favorite_meta
        .contains_key(&199));
    Mutation::ClearFavorites.apply(&mut value).unwrap();
    Mutation::ClearRecent.apply(&mut value).unwrap();
    assert!(Store::read(&value).unwrap().favorites.is_empty());
    for corrupt in [
        json!({"favorites-v1":"bad"}),
        json!({"favorites-v1":[65,65]}),
        json!({"favorites-v1":[99999999]}),
        json!({"favorites-meta-v1":{"65":{"score":null,"extra":true}}}),
    ] {
        let before = corrupt.clone();
        let mut v = corrupt;
        assert!(Mutation::AddFavorite(65, meta).apply(&mut v).is_err());
        assert_eq!(v, before);
    }
}
#[test]
fn session_bound_ids_do_not_revive_and_invalid_events_preserve_bindings() {
    let mut s = Session::new(&json!({})).unwrap();
    let view = s.render(&Store::default());
    let ids: HashSet<_> = s.bindings.keys().cloned().collect();
    let mut discarded = s.clone();
    discarded.query = "arrow".into();
    discarded.render(&Store::default());
    let next = s.render(&Store::default());
    assert_ne!(view["actions"], next["actions"]);
    assert!(s
        .bindings
        .keys()
        .all(|i| !ids.contains(i) && !discarded.bindings.contains_key(i)));
    let real = s.bindings.keys().next().unwrap().clone();
    assert!(s.prepare("forged", &json!({})).is_err());
    assert!(s.binding(&real).is_ok());
    let filter = s
        .bindings
        .iter()
        .find(|(_, b)| b.op == "filter")
        .unwrap()
        .0
        .clone();
    assert!(s
        .apply_ui(&filter, &json!({"formValues":{"value":"forged"}}))
        .is_err());
    assert!(s.binding(&filter).is_ok());
}
#[test]
fn strict_effect_ack_contract() {
    for good in [
        json!({"ok":true}),
        json!({"ok":true,"error":null,"success":true}),
    ] {
        assert!(acknowledge(&good).is_ok());
    }
    for bad in [
        Value::Null,
        json!(true),
        json!([]),
        json!({"ok":"true"}),
        json!({"ok":true,"error":false}),
        json!({"ok":true,"success":null}),
        json!({"ok":true,"success":false}),
        json!({"success":true}),
    ] {
        assert!(acknowledge(&bad).is_err());
    }
}
#[test]
fn surrogate_actions_reject_without_consuming_valid_format_actions() {
    let mut s = Session::new(&json!({"preferences":{"dataSet":"full"}})).unwrap();
    s.query = "U+D800".into();
    s.render(&Store::default());
    assert!(s.prepare(&id(&s, 55296, "copy"), &json!({})).is_err());
    assert_eq!(
        s.prepare(&id(&s, 55296, "escape"), &json!({})).unwrap().0,
        Some(Effect::Copy("\\uD800".into()))
    );
}
#[test]
fn negative_infinity_preference_survives_state_roundtrip() {
    let s = Session::new(&json!({"preferences":{"searchResultLimitStr":"-Infinity"}})).unwrap();
    let state = serde_json::to_value(s).unwrap();
    let decoded: Session = serde_json::from_value(state).unwrap();
    assert_eq!(decoded.prefs.limit, f64::NEG_INFINITY);
    assert_eq!(decoded.prefs.slice_end(100), 0);
}
