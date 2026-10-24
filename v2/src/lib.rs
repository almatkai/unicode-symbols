//! Standalone, offline Unicode Symbols Search. The host compiles adapter/mod.rs.
mod search;
mod session;
pub use search::{search, sections, SearchResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
pub use session::{acknowledge, fresh_id, Binding, Effect, Mutation, Session};
use std::{collections::HashMap, sync::OnceLock};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Character {
    pub c: u32,
    #[serde(deserialize_with = "decode_symbol", serialize_with = "encode_symbol")]
    pub v: String,
    pub n: String,
    pub o: String,
    pub a: Vec<String>,
    pub u: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub m: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nn: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Block {
    #[serde(rename = "blockName")]
    pub block_name: String,
    #[serde(rename = "startCode")]
    pub start_code: u32,
    #[serde(rename = "endCode")]
    pub end_code: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra: Vec<u32>,
}
impl Block {
    pub fn contains(&self, c: u32) -> bool {
        (self.start_code <= c && c <= self.end_code) || self.extra.contains(&c)
    }
}
#[derive(Deserialize)]
pub struct Dataset {
    pub blocks: Vec<Block>,
    pub characters: Vec<Character>,
    #[serde(skip)]
    codes: HashMap<u32, usize>,
}
impl Dataset {
    fn parse(s: &str) -> Self {
        let mut d: Self = serde_json::from_str(s).expect("bundled dataset");
        d.codes = d
            .characters
            .iter()
            .enumerate()
            .map(|(i, c)| (c.c, i))
            .collect();
        d
    }
    pub fn by_code(&self, c: u32) -> Option<&Character> {
        self.codes.get(&c).map(|i| &self.characters[*i])
    }
}
pub fn dataset(full: bool) -> &'static Dataset {
    static MIN: OnceLock<Dataset> = OnceLock::new();
    static FULL: OnceLock<Dataset> = OnceLock::new();
    if full {
        FULL.get_or_init(|| {
            Dataset::parse(include_str!("../fixtures/own/assets/full-dataset.json"))
        })
    } else {
        MIN.get_or_init(|| Dataset::parse(include_str!("../fixtures/own/assets/dataset.json")))
    }
}
pub fn canonical(c: u32) -> Option<&'static Character> {
    dataset(true).by_code(c)
}
pub fn html_entity(c: u32) -> Option<&'static str> {
    static MAP: OnceLock<HashMap<u32, String>> = OnceLock::new();
    MAP.get_or_init(|| {
        let data: Value = serde_json::from_str(include_str!("../fixtures/own/assets/html.json"))
            .expect("HTML entities");
        data["html_entities"]
            .as_array()
            .expect("entities")
            .iter()
            .map(|v| {
                (
                    v["code"].as_u64().expect("code") as u32,
                    v["value"].as_str().expect("entity").to_owned(),
                )
            })
            .collect()
    })
    .get(&c)
    .map(String::as_str)
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Preferences {
    #[serde(deserialize_with = "deserialize_limit")]
    pub limit: f64,
    pub view: String,
    pub columns: u8,
    pub full: bool,
    pub primary: String,
}
fn js_number(v: &Value) -> f64 {
    match v {
        Value::Null => 0.0,
        Value::Bool(b) => u8::from(*b) as f64,
        Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
        Value::String(s) => {
            let s = js_trim(s);
            if s.is_empty() {
                0.0
            } else if let Some(h) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                radix_number(h, 16)
            } else if let Some(h) = s.strip_prefix("0b").or_else(|| s.strip_prefix("0B")) {
                radix_number(h, 2)
            } else if let Some(h) = s.strip_prefix("0o").or_else(|| s.strip_prefix("0O")) {
                radix_number(h, 8)
            } else if s.to_ascii_lowercase().contains("inf") {
                if s == "Infinity" || s == "+Infinity" {
                    f64::INFINITY
                } else if s == "-Infinity" {
                    f64::NEG_INFINITY
                } else {
                    f64::NAN
                }
            } else {
                s.parse().unwrap_or(f64::NAN)
            }
        }
        _ => f64::NAN,
    }
}
impl Preferences {
    pub fn from_payload(payload: &Value) -> Result<Self, String> {
        let v = &payload["preferences"];
        if !v.is_null() && !v.is_object() {
            return Err("preferences must be an object".into());
        }
        let mut p = json!({"searchResultLimitStr":"100","viewType":"grid","gridItemSize":"medium","dataSet":"minimal","primaryAction":"paste"});
        if let Some(values) = v.as_object() {
            for (key, value) in values {
                p[key] = value.clone();
            }
        }
        for k in ["viewType", "gridItemSize", "dataSet", "primaryAction"] {
            if !p[k].is_string() {
                return Err(format!("{k} must be a string"));
            }
        }
        let n = js_number(&p["searchResultLimitStr"]);
        let n = if n == 0.0 || n.is_nan() { 100.0 } else { n };
        let view = match p["viewType"].as_str().unwrap() {
            "" => "list",
            s => s,
        }
        .to_owned();
        Ok(Self {
            limit: if n <= 1000.0 { n } else { 1000.0 },
            columns: match p["gridItemSize"].as_str() {
                Some("small") => 8,
                Some("large") => 3,
                _ => 5,
            },
            full: p["dataSet"] == "full" && view == "grid",
            view,
            primary: if p["primaryAction"] == "copy" {
                "copy".into()
            } else {
                "paste".into()
            },
        })
    }
    pub fn slice_end(&self, len: usize) -> usize {
        let end = self.limit.trunc();
        if end < 0.0 {
            len.saturating_sub((-end) as usize)
        } else {
            len.min(end as usize)
        }
    }
}
#[derive(Debug, Serialize)]
pub struct Section {
    pub title: String,
    pub lowest: f64,
    pub items: Vec<(u32, Option<f64>, bool)>,
}
pub fn upper_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
        None => String::new(),
    }
}
pub fn hex(c: u32) -> String {
    format!("{c:04X}")
}
pub fn formats(c: &Character) -> Value {
    json!({"hex":hex(c.c),"unicodeEscape":format!("\\u{}",hex(c.c)),"htmlDecimal":format!("&#{};",c.c),"htmlEntity":html_entity(c.c),"name":upper_first(&c.n)})
}
#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ItemMeta {
    pub score: Option<f64>,
    pub extra: bool,
}
#[derive(Clone, Default, Debug)]
pub struct Store {
    pub favorites: Vec<u32>,
    pub recent: Vec<u32>,
    pub favorite_meta: HashMap<u32, ItemMeta>,
    pub recent_meta: HashMap<u32, ItemMeta>,
}
impl Store {
    pub fn read(v: &Value) -> Result<Self, String> {
        if !v.is_object() {
            return Err("Native storage must be an object".into());
        }
        let read = |k: &str, limit: usize| -> Result<Vec<u32>, String> {
            let Some(v) = v.get(k) else {
                return Ok(Vec::new());
            };
            let a = v.as_array().ok_or("Corrupt Unicode storage array")?;
            if a.len() > limit {
                return Err("Corrupt Unicode storage limit".into());
            }
            let mut out = Vec::new();
            for n in a {
                let c = n
                    .as_u64()
                    .and_then(|n| u32::try_from(n).ok())
                    .filter(|n| canonical(*n).is_some())
                    .ok_or("Corrupt Unicode storage record")?;
                if out.contains(&c) {
                    return Err("Duplicate Unicode storage record".into());
                }
                out.push(c);
            }
            Ok(out)
        };
        let favorites = read("favorites-v1", 50)?;
        let recent = read("recently-used-v2", 10)?;
        let meta = |key: &str, codes: &[u32]| -> Result<HashMap<u32, ItemMeta>, String> {
            let Some(value) = v.get(key) else {
                return Ok(HashMap::new());
            };
            let out: HashMap<u32, ItemMeta> = serde_json::from_value(value.clone())
                .map_err(|_| "Corrupt Unicode item metadata")?;
            if out.keys().any(|c| !codes.contains(c)) {
                return Err("Orphan Unicode item metadata".into());
            }
            Ok(out)
        };
        Ok(Self {
            favorite_meta: meta("favorites-meta-v1", &favorites)?,
            recent_meta: meta("recently-used-meta-v2", &recent)?,
            favorites,
            recent,
        })
    }
}

// JSON.parse accepts six unpaired UTF-16 boundary records in the original data.
// Retain their exact escapes for record serialization. UTF-8 OS routes cannot copy
// these values; actions reject explicitly. Original Buffer.from renders U+FFFD.
fn decode_symbol<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let raw = Box::<serde_json::value::RawValue>::deserialize(d)?;
    match serde_json::from_str::<String>(raw.get()) {
        Ok(s) => Ok(s),
        Err(e) => {
            let s = raw.get();
            if s.len() == 8
                && s.starts_with("\"\\u")
                && s.ends_with('"')
                && u16::from_str_radix(&s[3..7], 16).is_ok_and(|c| (0xd800..=0xdfff).contains(&c))
            {
                Ok(s[1..7].to_owned())
            } else {
                Err(serde::de::Error::custom(e))
            }
        }
    }
}
fn encode_symbol<S: serde::Serializer>(s: &String, serializer: S) -> Result<S::Ok, S::Error> {
    if s.len() == 6
        && s.starts_with("\\u")
        && u16::from_str_radix(&s[2..], 16).is_ok_and(|c| (0xd800..=0xdfff).contains(&c))
    {
        serde_json::value::RawValue::from_string(format!("\"{s}\""))
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    } else {
        s.serialize(serializer)
    }
}
pub fn character_text(c: &Character) -> Result<String, String> {
    if (0xd800..=0xdfff).contains(&c.c) {
        Err("This original record is an unpaired UTF-16 surrogate; the beta UTF-8 OS router cannot represent it".into())
    } else {
        Ok(c.v.clone())
    }
}

fn deserialize_limit<'de, D: serde::Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
    Ok(Option::<f64>::deserialize(d)?.unwrap_or(f64::NEG_INFINITY))
}

pub(crate) fn js_whitespace(c: char) -> bool {
    matches!(c,'\u{0009}'..='\u{000d}'|'\u{0020}'|'\u{00a0}'|'\u{1680}'|'\u{2000}'..='\u{200a}'|'\u{2028}'|'\u{2029}'|'\u{202f}'|'\u{205f}'|'\u{3000}'|'\u{feff}')
}
pub(crate) fn js_trim(s: &str) -> &str {
    s.trim_matches(js_whitespace)
}
fn radix_number(s: &str, radix: u32) -> f64 {
    if s.is_empty() {
        return f64::NAN;
    }
    let mut n = 0.0;
    for c in s.chars() {
        let Some(v) = c.to_digit(radix) else {
            return f64::NAN;
        };
        n = n * radix as f64 + v as f64;
    }
    n
}
