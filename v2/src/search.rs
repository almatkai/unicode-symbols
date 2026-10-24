//! Fuse.js 6.6.2 extended/Bitap search port (Apache-2.0). See notices.
use super::{Character, Dataset, Preferences, Section};
use std::collections::HashMap;

fn units(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}
fn index_of(text: &[u16], pattern: &[u16], start: usize) -> Option<usize> {
    if pattern.is_empty() {
        return (start <= text.len()).then_some(start);
    }
    text.get(start..)?
        .windows(pattern.len())
        .position(|s| s == pattern)
        .map(|i| i + start)
}
fn bitap(text: &[u16], pattern: &[u16], location: usize) -> (bool, f64) {
    let n = pattern.len();
    if n == 0 {
        return (false, 1.0);
    }
    let expected = location.min(text.len());
    let score =
        |errors: usize, at: usize| errors as f64 / n as f64 + expected.abs_diff(at) as f64 / 100.0;
    let mut threshold: f64 = 0.3;
    let mut from = expected;
    while let Some(i) = index_of(text, pattern, from) {
        threshold = threshold.min(score(0, i));
        from = i + n;
    }
    let mut alphabet = HashMap::new();
    for (i, c) in pattern.iter().enumerate() {
        *alphabet.entry(*c).or_insert(0u32) |= 1u32 << (n - i - 1);
    }
    let mut best = None;
    let mut last = Vec::<u32>::new();
    let mut final_score = 1.0f64;
    let mut bin_max = n + text.len();
    let mask = 1u32 << (n - 1);
    for errors in 0..n {
        let mut bin_min = 0;
        let mut mid = bin_max;
        while bin_min < mid {
            if score(errors, expected + mid) <= threshold {
                bin_min = mid;
            } else {
                bin_max = mid;
            }
            mid = (bin_max - bin_min) / 2 + bin_min;
        }
        bin_max = mid;
        let mut start = 1.max(expected.saturating_sub(mid) + 1);
        let finish = (expected + mid).min(text.len()) + n;
        let mut bits = vec![0u32; finish + 2];
        bits[finish + 1] = (1u32 << errors).wrapping_sub(1);
        let mut j = finish;
        while j >= start {
            let at = j - 1;
            let m = text
                .get(at)
                .and_then(|c| alphabet.get(c))
                .copied()
                .unwrap_or(0);
            bits[j] = (bits[j + 1].wrapping_shl(1) | 1) & m;
            if errors > 0 {
                let a = last.get(j + 1).copied().unwrap_or(0);
                let b = last.get(j).copied().unwrap_or(0);
                bits[j] |= (a | b).wrapping_shl(1) | 1 | a;
            }
            if bits[j] & mask != 0 {
                final_score = score(errors, at);
                if final_score <= threshold {
                    threshold = final_score;
                    best = Some(at);
                    if at <= expected {
                        break;
                    }
                    start = 1.max((2 * expected).saturating_sub(at));
                }
            }
            j -= 1;
        }
        if score(errors + 1, expected) > threshold {
            break;
        }
        last = bits;
    }
    (best.is_some(), final_score.max(0.001))
}
fn fuzzy(text: &[u16], pattern: &[u16]) -> Option<f64> {
    if text == pattern {
        return Some(0.0);
    }
    if pattern.is_empty() {
        return None;
    }
    let mut chunks = Vec::new();
    if pattern.len() <= 32 {
        chunks.push((pattern, 0));
    } else {
        let end = pattern.len() - pattern.len() % 32;
        for i in (0..end).step_by(32) {
            chunks.push((&pattern[i..i + 32], i));
        }
        if pattern.len() % 32 != 0 {
            let i = pattern.len() - 32;
            chunks.push((&pattern[i..], i));
        }
    }
    let mut found = false;
    let mut total = 0.0;
    for (p, i) in &chunks {
        let (yes, score) = bitap(text, p, *i);
        found |= yes;
        total += score;
    }
    found.then_some(total / chunks.len() as f64)
}
#[derive(Clone)]
enum Matcher {
    Exact(String),
    Include(String),
    Prefix(String),
    NotPrefix(String),
    NotSuffix(String),
    Suffix(String),
    Not(String),
    Fuzzy(Vec<u16>),
}
fn matcher(token: &str) -> Option<Matcher> {
    // Match the real 6.6.2 regex precedence, including its truthy-capture rule.
    for quoted in [true, false] {
        for kind in 0..8 {
            let (prefix, suffix) = match kind {
                0 => ("=", ""),
                1 => ("'", ""),
                2 => ("^", ""),
                3 => ("!^", ""),
                4 => ("!", "$"),
                5 => ("", "$"),
                6 => ("!", ""),
                _ => ("", ""),
            };
            let a = format!("{prefix}{}", if quoted { "\"" } else { "" });
            let b = format!("{}{suffix}", if quoted { "\"" } else { "" });
            if let Some(p) = token.strip_prefix(&a).and_then(|t| t.strip_suffix(&b)) {
                if p.is_empty() || p.contains(['\n', '\r', '\u{2028}', '\u{2029}']) {
                    continue;
                }
                let p = p.to_owned();
                return Some(match kind {
                    0 => Matcher::Exact(p),
                    1 => Matcher::Include(p),
                    2 => Matcher::Prefix(p),
                    3 => Matcher::NotPrefix(p),
                    4 => Matcher::NotSuffix(p),
                    5 => Matcher::Suffix(p),
                    6 => Matcher::Not(p),
                    _ => Matcher::Fuzzy(units(&p)),
                });
            }
        }
    }
    None
}
fn parse(pattern: &str) -> Vec<Vec<Matcher>> {
    pattern
        .to_lowercase()
        .split('|')
        .map(|branch| {
            let s = super::js_trim(branch);
            let mut tokens = Vec::new();
            let mut begin = 0;
            let bytes = s.as_bytes();
            let mut i = 0;
            while i < bytes.len() {
                if bytes[i] == b' ' {
                    let mut end = i + 1;
                    while end < bytes.len() && bytes[end] == b' ' {
                        end += 1;
                    }
                    if bytes[end..].iter().filter(|b| **b == b'"').count() % 2 == 0 {
                        tokens.push(&s[begin..i]);
                        begin = end;
                    }
                    i = end;
                } else {
                    i += 1;
                }
            }
            tokens.push(&s[begin..]);
            tokens
                .into_iter()
                .filter(|t| !super::js_trim(t).is_empty())
                .filter_map(matcher)
                .collect()
        })
        .collect()
}
fn extended(text: &str, branches: &[Vec<Matcher>]) -> Option<f64> {
    let t = text.to_lowercase();
    let u = units(&t);
    for branch in branches {
        if branch.is_empty() {
            continue;
        }
        let mut total = 0.0;
        let mut yes = true;
        for m in branch {
            let result = match m {
                Matcher::Exact(p) => (t == *p).then_some(0.0),
                Matcher::Include(p) => t.contains(p).then_some(0.0),
                Matcher::Prefix(p) => t.starts_with(p).then_some(0.0),
                Matcher::NotPrefix(p) => (!t.starts_with(p)).then_some(0.0),
                Matcher::Suffix(p) => t.ends_with(p).then_some(0.0),
                Matcher::NotSuffix(p) => (!t.ends_with(p)).then_some(0.0),
                Matcher::Not(p) => (!t.contains(p)).then_some(0.0),
                Matcher::Fuzzy(p) => fuzzy(&u, p),
            };
            if let Some(s) = result {
                total += s;
            } else {
                yes = false;
                break;
            }
        }
        if yes {
            return Some(total / branch.len() as f64);
        }
    }
    None
}
fn norm(s: &str) -> f64 {
    (1000.0 / (s.split(' ').filter(|s| !s.is_empty()).count() as f64).sqrt()).round() / 1000.0
}
fn field_score(s: &str, weight: f64, branches: &[Vec<Matcher>]) -> Option<f64> {
    if super::js_trim(s).is_empty() {
        return None;
    }
    extended(s, branches)
        .map(|s0| (if s0 == 0.0 { f64::EPSILON } else { s0 }).powf(weight * norm(s)))
}
fn parse_int(s: &str, radix: u32) -> Option<u32> {
    let s = s.trim_start_matches(super::js_whitespace);
    let (negative, s) = if let Some(s) = s.strip_prefix('-') {
        (true, s)
    } else {
        (false, s.strip_prefix('+').unwrap_or(s))
    };
    let s = if radix == 16 {
        s.strip_prefix("0x")
            .or_else(|| s.strip_prefix("0X"))
            .unwrap_or(s)
    } else {
        s
    };
    let digits: String = s.chars().take_while(|c| c.is_digit(radix)).collect();
    if negative {
        return None;
    } // Negative codepoints cannot occur in either map.
    u32::from_str_radix(&digits, radix).ok()
}
fn exact<'a>(d: &'a Dataset, q: &str) -> Option<&'a Character> {
    // Preserve parseInt precedence and the source's else-if behavior (0x is decimal 0).
    if let Some(n) = parse_int(q, 10) {
        if let Some(c) = d.by_code(n) {
            return Some(c);
        }
    } else if let Some(n) = parse_int(q, 16) {
        if let Some(c) = d.by_code(n) {
            return Some(c);
        }
    } else if ["\\u", "\\U", "U+", "u+", "0x"]
        .iter()
        .any(|p| q.starts_with(p))
    {
        return parse_int(&q[2..], 16).and_then(|n| d.by_code(n));
    }
    if q.encode_utf16().count() == 1 {
        if let Some(c) = d.by_code(q.encode_utf16().next()? as u32) {
            return Some(c);
        }
    }
    d.characters.iter().rev().find(|c| c.v == q)
}
#[derive(Clone)]
pub struct SearchResult {
    pub characters: Vec<(u32, Option<f64>)>,
    pub selected: Option<String>,
}
pub fn search(d: &Dataset, p: &Preferences, query: &str, filter: Option<&str>) -> SearchResult {
    let block = filter.and_then(|f| d.blocks.iter().find(|b| b.block_name == f));
    let chars: Vec<_> = d
        .characters
        .iter()
        .filter(|c| block.is_none_or(|b| b.contains(c.c)))
        .collect();
    if query.is_empty() {
        let end = if filter.is_some() {
            chars.len()
        } else {
            p.slice_end(chars.len())
        };
        return SearchResult {
            characters: chars[..end].iter().map(|c| (c.c, None)).collect(),
            selected: block.map(|b| b.block_name.clone()),
        };
    }
    let split: Vec<_> = super::js_trim(query).split(' ').collect();
    let pattern = if split.len() == 1 {
        format!("'{} | {}", split[0], split[0])
    } else {
        split
            .iter()
            .map(|s| format!("'{s}"))
            .collect::<Vec<_>>()
            .join(" ")
    };
    let branches = parse(&pattern);
    let mut ranked = Vec::new();
    for (idx, c) in chars.iter().enumerate() {
        let mut score = 1.0;
        let mut matched = false;
        // FuseIndex keys keep the original (unnormalized) weights for string queries.
        for (s, w) in std::iter::once((&c.n, 0.6))
            .chain(c.a.iter().rev().map(|s| (s, 0.4)))
            .chain(std::iter::once((&c.o, 0.2)))
            .chain(c.nn.iter().map(|s| (s, 0.1)))
        {
            if let Some(v) = field_score(s, w, &branches) {
                score *= v;
                matched = true;
            }
        }
        if matched {
            ranked.push((idx, c.c, score));
        }
    }
    ranked.sort_by(|a, b| a.2.total_cmp(&b.2).then(a.0.cmp(&b.0)));
    if p.limit > -1.0 {
        ranked.truncate(p.slice_end(ranked.len()));
    }
    let mut out: Vec<_> = ranked.into_iter().map(|(_, c, s)| (c, Some(s))).collect();
    if split.len() == 1 {
        if let Some(c) = exact(d, split[0]) {
            out.retain(|(n, _)| *n != c.c);
            out.insert(0, (c.c, Some(-1.0)));
        }
    }
    SearchResult {
        characters: out,
        selected: block.map(|b| b.block_name.clone()),
    }
}
pub fn sections(
    d: &Dataset,
    r: &SearchResult,
    query: &str,
    filter: Option<&str>,
    favorites: &[u32],
    recent: &[u32],
) -> Vec<Section> {
    let mut sections: Vec<_> = d
        .blocks
        .iter()
        .filter(|b| r.selected.as_ref().is_none_or(|s| *s == b.block_name))
        .map(|b| {
            let items: Vec<_> = r
                .characters
                .iter()
                .filter(|(c, _)| b.contains(*c))
                .map(|(c, s)| (*c, *s, b.extra.contains(c)))
                .collect();
            let lowest = items
                .iter()
                .map(|(_, s, _)| s.filter(|s| *s != 0.0).unwrap_or(1.0))
                .fold(1.0, f64::min);
            Section {
                title: b.block_name.clone(),
                lowest,
                items,
            }
        })
        .collect();
    for (title, codes, score) in [
        ("Favorites", favorites, -1.0),
        ("Recently Used", recent, 0.0),
    ] {
        if !codes.is_empty() && filter.is_none_or(|s| s.is_empty()) {
            sections.insert(
                0,
                Section {
                    title: title.into(),
                    lowest: score,
                    items: codes
                        .iter()
                        .filter(|c| query.is_empty() || r.characters.iter().any(|(n, _)| n == *c))
                        .map(|c| (*c, None, false))
                        .collect(),
                },
            );
        }
    }
    sections.sort_by(|a, b| a.lowest.total_cmp(&b.lowest));
    sections
}
