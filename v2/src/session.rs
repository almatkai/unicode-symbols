use super::*;
use base64::{engine::general_purpose::STANDARD, Engine};
use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hasher},
    sync::atomic::{AtomicU64, Ordering},
};

pub fn fresh_id() -> String {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    static RANDOM: OnceLock<RandomState> = OnceLock::new();
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let mut h = RANDOM.get_or_init(RandomState::new).build_hasher();
    h.write_u64(n);
    format!("unicode:{:016x}:{n:x}", h.finish())
}
pub fn acknowledge(result: &Value) -> Result<(), String> {
    if result.is_object()
        && result["ok"] == true
        && result.get("error").is_none_or(Value::is_null)
        && result.get("success").is_none_or(|v| v == true)
    {
        Ok(())
    } else {
        Err(format!("Operation did not acknowledge success: {result}"))
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Binding {
    pub op: String,
    pub code: Option<u32>,
    pub section: Option<String>,
    #[serde(default)]
    pub phase: String,
    #[serde(default)]
    pub meta: ItemMeta,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub prefs: Preferences,
    pub query: String,
    pub filter: Option<String>,
    pub bindings: HashMap<String, Binding>,
    pub snippet: Option<u32>,
    pub drafts: HashMap<u32, Value>,
    pub disposed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    Copy(String),
    Paste(String),
    Open(String),
    Snippet {
        label: String,
        trigger: String,
        body: String,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub enum Mutation {
    AddFavorite(u32, ItemMeta),
    RemoveFavorite(u32),
    ClearFavorites,
    AddRecent(u32, ItemMeta),
    RemoveRecent(u32),
    ClearRecent,
}
impl Mutation {
    pub fn apply(&self, value: &mut Value) -> Result<(), String> {
        let mut s = Store::read(value)?;
        let add = |v: &mut Vec<u32>,
                   meta: &mut HashMap<u32, ItemMeta>,
                   c: u32,
                   item_meta: ItemMeta,
                   max: usize| {
            if !v.contains(&c) {
                v.insert(0, c);
                v.truncate(max);
                meta.insert(c, item_meta);
                meta.retain(|c, _| v.contains(c));
            }
        };
        match *self {
            Self::AddFavorite(c, m) => add(&mut s.favorites, &mut s.favorite_meta, c, m, 50),
            Self::RemoveFavorite(c) => {
                s.favorites.retain(|n| *n != c);
                s.favorite_meta.remove(&c);
            }
            Self::ClearFavorites => {
                s.favorites.clear();
                s.favorite_meta.clear();
            }
            Self::AddRecent(c, m) => add(&mut s.recent, &mut s.recent_meta, c, m, 10),
            Self::RemoveRecent(c) => {
                s.recent.retain(|n| *n != c);
                s.recent_meta.remove(&c);
            }
            Self::ClearRecent => {
                s.recent.clear();
                s.recent_meta.clear();
            }
        }
        value["favorites-meta-v1"] = json!(s.favorite_meta);
        value["recently-used-meta-v2"] = json!(s.recent_meta);
        value["favorites-v1"] = json!(s.favorites);
        value["recently-used-v2"] = json!(s.recent);
        Ok(())
    }
}
impl Session {
    pub fn new(payload: &Value) -> Result<Self, String> {
        Ok(Self {
            prefs: Preferences::from_payload(payload)?,
            query: String::new(),
            filter: None,
            bindings: HashMap::new(),
            snippet: None,
            drafts: HashMap::new(),
            disposed: false,
        })
    }
    pub fn binding(&self, id: &str) -> Result<&Binding, String> {
        if self.disposed {
            return Err("Disposed Unicode session".into());
        }
        self.bindings
            .get(id)
            .ok_or_else(|| "Stale or forged Unicode action ID".into())
    }
    pub fn prepare(
        &self,
        id: &str,
        payload: &Value,
    ) -> Result<(Option<Effect>, Option<Mutation>), String> {
        let b = self.binding(id)?;
        let c = b.code.and_then(canonical);
        let c = || c.ok_or_else(|| "Missing canonical character".to_owned());
        let mut mutation = None;
        let effect = match b.op.as_str() {
            "copy" => {
                let c = c()?;
                mutation = Some(Mutation::AddRecent(c.c, b.meta));
                Some(Effect::Copy(character_text(c)?))
            }
            "paste" => {
                let c = c()?;
                mutation = Some(Mutation::AddRecent(c.c, b.meta));
                Some(Effect::Paste(character_text(c)?))
            }
            "hex" => {
                let c = c()?;
                mutation = Some(Mutation::AddRecent(c.c, b.meta));
                Some(Effect::Copy(hex(c.c)))
            }
            "escape" => Some(Effect::Copy(format!("\\u{}", hex(c()?.c)))),
            "decimal" => Some(Effect::Copy(format!("&#{};", c()?.c))),
            "html" => {
                let c = c()?;
                mutation = Some(Mutation::AddRecent(c.c, b.meta));
                Some(Effect::Copy(
                    html_entity(c.c).ok_or("No HTML entity")?.into(),
                ))
            }
            "compart" => Some(Effect::Open(format!(
                "https://www.compart.com/en/unicode/U+{}",
                hex(c()?.c)
            ))),
            "explorer" => Some(Effect::Open(format!(
                "https://unicode-explorer.com/c/{}",
                hex(c()?.c)
            ))),
            "favorite" => {
                mutation = Some(Mutation::AddFavorite(c()?.c, b.meta));
                None
            }
            "unfavorite" => {
                mutation = Some(Mutation::RemoveFavorite(c()?.c));
                None
            }
            "clear-favorites" => {
                mutation = Some(Mutation::ClearFavorites);
                None
            }
            "remove-recent" => {
                mutation = Some(Mutation::RemoveRecent(c()?.c));
                None
            }
            "clear-recent" => {
                mutation = Some(Mutation::ClearRecent);
                None
            }
            "submit-snippet" => {
                let c = c()?;
                let draft = self.drafts.get(&c.c).ok_or("Missing snippet draft")?;
                let values = payload.get("formValues").unwrap_or(draft);
                let label = values["label"].as_str().unwrap_or("").trim();
                let trigger = values["trigger"].as_str().unwrap_or("").trim();
                if label.is_empty()
                    || label.encode_utf16().count() > 200
                    || trigger.is_empty()
                    || trigger.encode_utf16().count() > 48
                    || trigger.contains(['\n', '\r'])
                {
                    return Err("Snippet title and single-line trigger are required (200/48 character limits)".into());
                }
                Some(Effect::Snippet {
                    label: label.into(),
                    trigger: trigger.into(),
                    body: character_text(c)?,
                })
            }
            "filter" | "set-filter" | "clear-filter" | "snippet" | "back" | "draft-label"
            | "draft-trigger" => None,
            _ => return Err("Invalid Unicode operation".into()),
        };
        if b.phase == "uncertain" && !matches!(effect, Some(Effect::Copy(_))) {
            return Err("Prior effect outcome is uncertain; no automatic repeat. Return to search and select the action again deliberately.".into());
        }
        Ok((effect, mutation))
    }
    pub fn apply_ui(&mut self, id: &str, payload: &Value) -> Result<(), String> {
        let b = self.binding(id)?.clone();
        match b.op.as_str() {
            "filter" => {
                let f = payload["formValues"]["value"]
                    .as_str()
                    .ok_or("Missing block filter")?;
                if f != "null"
                    && !dataset(self.prefs.full)
                        .blocks
                        .iter()
                        .any(|b| b.block_name == f)
                {
                    return Err("Unknown character block".into());
                }
                self.filter = (f != "null").then(|| f.into());
            }
            "set-filter" => self.filter = b.section,
            "clear-filter" => self.filter = None,
            "snippet" => {
                let c = b
                    .code
                    .and_then(canonical)
                    .ok_or("Invalid snippet character")?;
                character_text(c)?;
                self.snippet = Some(c.c);
                self.drafts
                    .entry(c.c)
                    .or_insert_with(|| json!({"label":title_case(&c.n),"trigger":""}));
            }
            "back" => self.snippet = None,
            "draft-label" | "draft-trigger" => {
                let value = payload["formValues"]["value"]
                    .as_str()
                    .ok_or("Missing draft value")?;
                let key = if b.op == "draft-label" {
                    "label"
                } else {
                    "trigger"
                };
                self.drafts
                    .get_mut(&b.code.ok_or("Missing draft code")?)
                    .ok_or("Missing draft")?[key] = json!(value);
            }
            "submit-snippet" => self.snippet = None,
            _ => {}
        }
        Ok(())
    }
    fn action(
        &mut self,
        actions: &mut Vec<Value>,
        title: String,
        op: &str,
        code: Option<u32>,
        section: Option<&str>,
    ) -> String {
        let id = fresh_id();
        self.bindings.insert(
            id.clone(),
            Binding {
                op: op.into(),
                code,
                section: section.map(str::to_owned),
                phase: String::new(),
                meta: ItemMeta::default(),
            },
        );
        actions.push(json!({"id":id,"title":title,"kind":if op=="submit-snippet"{"submit-form"}else{"action"}}));
        id
    }
    pub fn render(&mut self, store: &Store) -> Value {
        self.bindings.clear();
        let mut actions = Vec::new();
        if let Some(code) = self.snippet {
            let c = canonical(code).expect("canonical snippet");
            let draft = self.drafts[&code].clone();
            let submit = self.action(
                &mut actions,
                "Save as Snippet".into(),
                "submit-snippet",
                Some(code),
                None,
            );
            let back = self.action(&mut actions, "Back".into(), "back", None, None);
            let label = self.action(
                &mut actions,
                "Edit Title".into(),
                "draft-label",
                Some(code),
                None,
            );
            let trigger = self.action(
                &mut actions,
                "Edit Trigger".into(),
                "draft-trigger",
                Some(code),
                None,
            );
            return json!({"ok":true,"mode":"view","title":"Save as Snippet","root":node("Form",json!({"backActionId":back,"primaryActionId":submit}),vec![node("Form.TextField",json!({"id":"label","title":"Title","value":draft["label"],"actionId":label}),vec![]),node("Form.TextField",json!({"id":"trigger","title":"Trigger","value":draft["trigger"],"actionId":trigger}),vec![]),node("Form.Description",json!({"title":"Character","text":c.v}),vec![])]),"actions":actions});
        }
        let d = dataset(self.prefs.full);
        let result = search(d, &self.prefs, &self.query, self.filter.as_deref());
        let mut list = sections(
            d,
            &result,
            &self.query,
            self.filter.as_deref(),
            &store.favorites,
            &store.recent,
        );
        for section in &mut list {
            let meta = match section.title.as_str() {
                "Favorites" => Some(&store.favorite_meta),
                "Recently Used" => Some(&store.recent_meta),
                _ => None,
            };
            if let Some(meta) = meta {
                for (c, score, extra) in &mut section.items {
                    if let Some(m) = meta.get(c) {
                        *score = m.score;
                        *extra = m.extra;
                    }
                }
            }
        }
        let filter = self.action(
            &mut actions,
            "Select a Character Set".into(),
            "filter",
            None,
            None,
        );
        let mut opts = vec![node(
            "List.Dropdown.Item",
            json!({"title":"<All Characters>","value":"null"}),
            vec![],
        )];
        static ORDER: OnceLock<Vec<Vec<String>>> = OnceLock::new();
        let orders = ORDER.get_or_init(|| {
            let v: Value =
                serde_json::from_str(include_str!("../fixtures/derived/block-order.json"))
                    .expect("block order");
            serde_json::from_value(v["sets"].clone()).expect("block sets")
        });
        let blocks = &orders[usize::from(self.prefs.full)];
        opts.extend(
            blocks
                .iter()
                .map(|b| node("List.Dropdown.Item", json!({"title":b,"value":b}), vec![])),
        );
        let accessory = node(
            "List.Dropdown",
            json!({"actionId":filter,"value":self.filter.as_deref().unwrap_or("null"),"tooltip":"Select a character set"}),
            opts,
        );
        let grid = self.prefs.view != "list";
        let mut children = Vec::new();
        for section in list {
            let mut rows = Vec::new();
            for (code, score, extra) in section.items {
                let Some(c) = d.by_code(code).or_else(|| canonical(code)) else {
                    continue;
                };
                let mut ids = Vec::new();
                let primary = if self.prefs.primary == "copy" {
                    ["copy", "paste"]
                } else {
                    ["paste", "copy"]
                };
                for op in primary {
                    ids.push(self.action(
                        &mut actions,
                        if op == "copy" {
                            "Copy Character to Clipboard".into()
                        } else {
                            "Paste Character to Active App".into()
                        },
                        op,
                        Some(code),
                        None,
                    ));
                }
                ids.push(self.action(
                    &mut actions,
                    "Save as Snippet".into(),
                    "snippet",
                    Some(code),
                    None,
                ));
                for (op, title) in [
                    ("hex", format!("Copy \"{}\" (HEX) to Clipboard", hex(code))),
                    (
                        "escape",
                        format!("Copy \"\\u{}\" (Unicode Escape) to Clipboard", hex(code)),
                    ),
                ] {
                    ids.push(self.action(&mut actions, title, op, Some(code), None));
                }
                if let Some(html) = html_entity(code) {
                    ids.push(self.action(
                        &mut actions,
                        format!("Copy \"{html}\" (HTML Entity) to Clipboard"),
                        "html",
                        Some(code),
                        None,
                    ));
                }
                ids.push(self.action(
                    &mut actions,
                    format!("Copy \"&#{code};\" (HTML Decimal) to Clipboard"),
                    "decimal",
                    Some(code),
                    None,
                ));
                if self.filter.as_deref() != Some(&section.title) {
                    ids.push(self.action(
                        &mut actions,
                        format!("Set Filter to \"{}\"", section.title),
                        "set-filter",
                        Some(code),
                        Some(&section.title),
                    ));
                }
                if self.filter.is_some() {
                    ids.push(self.action(
                        &mut actions,
                        "Clear Filter (Show All Characters)".into(),
                        "clear-filter",
                        Some(code),
                        None,
                    ));
                }
                let favorite = store.favorites.contains(&code);
                ids.push(
                    self.action(
                        &mut actions,
                        if favorite {
                            "Remove from Favorites"
                        } else {
                            "Add to Favorites"
                        }
                        .into(),
                        if favorite { "unfavorite" } else { "favorite" },
                        Some(code),
                        None,
                    ),
                );
                ids.push(self.action(
                    &mut actions,
                    "Clear All Favorites".into(),
                    "clear-favorites",
                    Some(code),
                    None,
                ));
                if store.recent.contains(&code) {
                    for (op, title) in [
                        ("remove-recent", "Remove from Recently Used"),
                        ("clear-recent", "Clear All Recently Used"),
                    ] {
                        ids.push(self.action(&mut actions, title.into(), op, Some(code), None));
                    }
                }
                for (op, title) in [
                    ("compart", "Open Character on Compart"),
                    ("explorer", "Open Character on Unicode Explorer"),
                ] {
                    ids.push(self.action(&mut actions, title.into(), op, Some(code), None));
                }
                for id in &ids {
                    self.bindings.get_mut(id).unwrap().meta = ItemMeta { score, extra };
                }
                let tooltip = tooltip(c, &section.title, self.filter.is_none(), extra);
                if grid {
                    rows.push(node("Grid.Item",json!({"id":format!("{}:{code}",section.title),"title":upper_first(&c.n),"content":{"source":{"light":svg(&c.v,false),"dark":svg(&c.v,true)}},"accessory":{"tooltip":tooltip,"icon":{"source":if extra{"exclamation-outline.svg"}else{"Info"}}},"actionIds":ids}),vec![]));
                } else {
                    let ancient = section.title == "Ancient Symbols";
                    rows.push(node("List.Item",json!({"id":format!("{}:{code}",section.title),"title":if ancient{"?"}else{&c.v},"subtitle":format!("{}{}",upper_first(&c.n),if ancient{" (see in Grid Mode)"}else{""}),"accessories":if c.a.is_empty(){vec![]}else{vec![json!({"icon":"⌨️","text":c.a.join(", ")})]},"actionIds":ids}),vec![]));
                }
            }
            children.push(node(
                if grid { "Grid.Section" } else { "List.Section" },
                json!({"title":section.title,"aspectRatio":"1","fit":"fill"}),
                rows,
            ));
        }
        let root = node(
            if grid { "Grid" } else { "List" },
            json!({"searchText":self.query,"filtering":false,"__hasServerSearch":true,"isLoading":false,"columns":self.prefs.columns,"searchBarAccessory":accessory}),
            children,
        );
        json!({"ok":true,"mode":"view","title":"Search Unicode Symbol","root":root,"actions":actions})
    }
}
fn node(kind: &str, props: Value, children: Vec<Value>) -> Value {
    json!({"type":kind,"props":props,"children":children})
}
fn title_case(s: &str) -> String {
    let mut boundary = true;
    s.to_ascii_lowercase()
        .chars()
        .map(|c| {
            let word = c.is_ascii_alphanumeric() || c == '_';
            let out = if boundary && word {
                c.to_ascii_uppercase()
            } else {
                c
            };
            boundary = !word;
            out
        })
        .collect()
}
fn svg(value: &str, dark: bool) -> String {
    let value = if value.starts_with("\\u") && value.len() == 6 {
        "�"
    } else {
        value
    };
    let val = match value {
        "&" => "&amp;",
        "<" => "&lt;",
        ">" => "&gt;",
        _ => value,
    };
    let color = if dark { "#fff" } else { "#000" };
    let raycast_x = std::env::var("RAYCASTX").as_deref() == Ok("1");
    let y = if raycast_x {
        "100"
    } else {
        "153.84615384615384"
    };
    let font = if raycast_x { "monospace" } else { "mono-space" };
    let s=format!("\n  <svg height=\"200\" width=\"200\" xmlns=\"http://www.w3.org/2000/svg\">\n    <rect fill=\"transparent\" x=\"0\" y=\"0\" width=\"200\" height=\"200\"></rect>\n    <text x=\"100\" y=\"{y}\" fill=\"{color}\" text-anchor=\"middle\" alignment-baseline=\"central\" font-size=\"100\" line-height=\"0\" font-family=\"{font}\">{val}</text>\n  </svg>\n  ");
    format!("data:image/svg+xml;base64,{}", STANDARD.encode(s))
}
fn tooltip(c: &Character, section: &str, unfiltered: bool, extra: bool) -> String {
    let mut v = vec![
        format!("Name: {}", upper_first(&c.n)),
        format!("Dec: {}", c.c),
        format!("Hex: {}", hex(c.c)),
    ];
    if unfiltered {
        v.push(format!("Section: {section}"));
    }
    if let Some(html) = html_entity(c.c) {
        v.push(format!("HTML Entity: {html}"));
    }
    if !c.a.is_empty() {
        v.push(format!(
            "Aliases: \"{}\"",
            c.a.iter()
                .map(|s| upper_first(s))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !c.u.is_empty() {
        v.push(format!("Unicode Version: {}", c.u));
    }
    if let Some(m) = c.m.filter(|m| *m != 0) {
        v.push(format!("Mirror Code: {m}"));
    }
    if extra {
        v.push(" ".into());
        v.push("> Note: This character is actually in a different Character Set".into());
    }
    v.join("\n")
}
