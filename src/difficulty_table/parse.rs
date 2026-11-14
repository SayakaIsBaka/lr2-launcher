use anyhow::{Result, bail};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct TableHeader {
    pub name: String,
    pub data_url: String,
    pub level_order: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TableChart {
    pub md5: String,
    pub level: String,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct TableListItem {
    pub name: String,
    pub url: String,
    pub tag1: String,
    pub tag2: String,
    pub symbol: String,
}

pub fn extract_bmstable_meta(html: &str) -> Option<String> {
    // Prefer name="bmstable" ... content="..."
    let lower = html.to_ascii_lowercase();
    let mut search_from = 0;
    while let Some(rel) = lower[search_from..].find("bmstable") {
        let abs = search_from + rel;
        let window_start = html.floor_char_boundary(abs.saturating_sub(200));
        let abs_end = abs.min(html.len());
        let window = &html[window_start..abs_end];
        if !window.to_ascii_lowercase().contains("<meta") {
            search_from = abs + 8;
            continue;
        }
        let after = &html[abs_end..];
        if let Some(content) = extract_attr(after, "content") {
            return Some(content);
        }
        let before_end = html.ceil_char_boundary((abs + 80).min(html.len()));
        let before = &html[window_start..before_end];
        if let Some(content) = extract_attr(before, "content") {
            return Some(content);
        }
        search_from = abs + 8;
    }
    None
}

fn extract_attr(s: &str, attr: &str) -> Option<String> {
    let lower = s.to_ascii_lowercase();
    let key = format!("{attr}=");
    let idx = lower.find(&key)?;
    let rest = &s[idx + key.len()..];
    let rest = rest.trim_start();
    let quote = rest.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let end = rest[1..].find(quote)?;
    Some(rest[1..1 + end].to_string())
}

fn strip_bom(text: &str) -> &str {
    text.strip_prefix('\u{feff}').unwrap_or(text)
}

pub fn parse_header_json(text: &str) -> Result<TableHeader> {
    let v: Value = serde_json::from_str(strip_bom(text))?;
    let name = v
        .get("name")
        .and_then(|x| x.as_str())
        .unwrap_or("Untitled")
        .to_string();
    let data_url = v
        .get("data_url")
        .and_then(|x| x.as_str())
        .unwrap_or("data.json")
        .to_string();
    let mut level_order = Vec::new();
    if let Some(arr) = v.get("level_order").and_then(|x| x.as_array()) {
        for item in arr {
            if let Some(s) = item.as_str() {
                level_order.push(s.to_string());
            } else if let Some(n) = item.as_i64() {
                level_order.push(n.to_string());
            } else if let Some(n) = item.as_f64() {
                level_order.push(n.to_string());
            }
        }
    }
    Ok(TableHeader {
        name,
        data_url,
        level_order,
    })
}

#[derive(Deserialize)]
struct ChartRaw {
    md5: Option<String>,
    level: Option<Value>,
    title: Option<String>,
}

pub fn parse_data_json(text: &str) -> Result<Vec<TableChart>> {
    let raw: Vec<ChartRaw> = serde_json::from_str(strip_bom(text))?;
    let mut out = Vec::new();
    for c in raw {
        let md5 = match c.md5 {
            Some(m) if !m.is_empty() => m.to_ascii_lowercase(),
            _ => continue,
        };
        let level = match c.level {
            Some(Value::String(s)) => s,
            Some(Value::Number(n)) => n.to_string(),
            _ => continue,
        };
        out.push(TableChart {
            md5,
            level,
            title: c.title.unwrap_or_default(),
        });
    }
    if out.is_empty() {
        bail!("data.json has no usable entries (need md5 + level)");
    }
    Ok(out)
}

pub fn parse_tablelist_json(text: &str) -> Result<Vec<TableListItem>> {
    let v: Value = serde_json::from_str(strip_bom(text))?;
    let arr = match v {
        Value::Array(a) => a,
        Value::Object(o) => o
            .get("table")
            .or_else(|| o.get("tables"))
            .or_else(|| o.get("data"))
            .and_then(|x| x.as_array())
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("tablelist JSON is not an array"))?,
        _ => bail!("Unexpected tablelist JSON root"),
    };
    let mut out = Vec::new();
    for item in arr {
        let name = item
            .get("name")
            .or_else(|| item.get("title"))
            .and_then(|x| x.as_str())
            .unwrap_or("Untitled")
            .to_string();
        let url = item
            .get("url")
            .or_else(|| item.get("site"))
            .or_else(|| item.get("page_url"))
            .or_else(|| item.get("page"))
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        if url.is_empty() {
            continue;
        }
        let tag1 = item
            .get("tag1")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let tag2 = item
            .get("tag2")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let symbol = item
            .get("symbol")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        out.push(TableListItem {
            name,
            url,
            tag1,
            tag2,
            symbol,
        });
    }
    Ok(out)
}

pub fn unique_sorted_tags(items: &[TableListItem], which: u8) -> Vec<String> {
    let mut tags: Vec<String> = items
        .iter()
        .map(|i| if which == 1 { i.tag1.clone() } else { i.tag2.clone() })
        .filter(|t| !t.is_empty())
        .collect();
    tags.sort();
    tags.dedup();
    tags
}

pub fn filter_tablelist_items(
    items: &[TableListItem],
    tag1: Option<&str>,
    tag2: Option<&str>,
    search: &str,
) -> Vec<TableListItem> {
    let search = search.trim().to_ascii_lowercase();
    items
        .iter()
        .filter(|i| {
            if let Some(t1) = tag1 {
                if i.tag1 != t1 {
                    return false;
                }
            }
            if let Some(t2) = tag2 {
                if i.tag2 != t2 {
                    return false;
                }
            }
            if !search.is_empty() {
                let hay = format!(
                    "{} {} {} {}",
                    i.name, i.symbol, i.tag1, i.tag2
                )
                .to_ascii_lowercase();
                if !hay.contains(&search) {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect()
}

pub fn folder_name_for_level(level: &str) -> String {
    format!("LEVEL {level}")
}
