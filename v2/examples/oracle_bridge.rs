use serde::Serialize;
use serde_json::{json, value::RawValue, Value};
use std::io::{self, BufRead};
use tezbar_unicode_symbols::*;
#[derive(Serialize)]
struct Output {
    preferences: Preferences,
    characters: Vec<Box<RawValue>>,
    sections: Vec<Section>,
    view: Value,
}
fn main() {
    let mut cache = std::collections::HashMap::<String, SearchResult>::new();
    for line in io::stdin().lock().lines() {
        let p: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let prefs = Preferences::from_payload(&p).unwrap();
        let d = dataset(prefs.full);
        let q = p["query"].as_str().unwrap_or("");
        let filter = p["filter"].as_str();
        let key = format!("{}:{}:{q:?}:{filter:?}", prefs.full, prefs.limit);
        let result = if p["dumpAll"] == true {
            SearchResult {
                characters: d.characters.iter().map(|c| (c.c, None)).collect(),
                selected: None,
            }
        } else {
            cache
                .entry(key)
                .or_insert_with(|| search(d, &prefs, q, filter))
                .clone()
        };
        let favorites: Vec<u32> =
            serde_json::from_value(p.get("favorites").cloned().unwrap_or(json!([]))).unwrap();
        let recent: Vec<u32> =
            serde_json::from_value(p.get("recent").cloned().unwrap_or(json!([]))).unwrap();
        let sections = sections(d, &result, q, filter, &favorites, &recent);
        let characters = result
            .characters
            .iter()
            .map(|(c, score)| {
                let mut s = serde_json::to_string(d.by_code(*c).unwrap()).unwrap();
                if let Some(score) = score {
                    s.pop();
                    s.push_str(&format!(",\"score\":{score}}}"));
                }
                RawValue::from_string(s).unwrap()
            })
            .collect();
        let mut session = Session::new(&p).unwrap();
        session.query = q.into();
        session.filter = filter.map(str::to_owned);
        let view = if p["emitView"] == true {
            session.render(&Store {
                favorites,
                recent,
                ..Default::default()
            })
        } else {
            Value::Null
        };
        println!(
            "{}",
            serde_json::to_string(&Output {
                preferences: prefs,
                characters,
                sections,
                view
            })
            .unwrap()
        );
    }
}
