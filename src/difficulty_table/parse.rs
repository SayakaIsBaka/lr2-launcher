use anyhow::{Result, bail};
use quick_xml::XmlVersion;
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::QName;
use quick_xml::reader::Reader;
use regex::Regex;
use serde::Deserialize;
use serde_json::Value;
use std::sync::LazyLock;

#[derive(Debug, Clone, Default)]
pub struct TableHeader {
    pub name: String,
    pub symbol: String,
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

/// Prefer quick-xml (HTML-tolerant). Fall back to regex when XML scan fails/misses.
pub fn extract_bmstable_meta(html: &str) -> Option<String> {
    let mut reader = Reader::from_str(html);
    {
        let cfg = reader.config_mut();
        cfg.check_end_names = false;
        cfg.allow_unmatched_ends = true;
        cfg.allow_dangling_amp = true;
        cfg.trim_text(true);
    }

    loop {
        let decoder = reader.decoder();
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(Event::Empty(ref e)) => {
                if let Some(content) = bmstable_content_from_meta(e, decoder) {
                    return Some(content);
                }
            }
            Ok(Event::Start(ref e)) => {
                let skip_raw = e.local_name().as_ref().eq_ignore_ascii_case(b"script")
                    || e.local_name().as_ref().eq_ignore_ascii_case(b"style");
                let content = bmstable_content_from_meta(e, decoder);
                let raw_name = skip_raw.then(|| e.name().as_ref().to_owned());
                if let Some(content) = content {
                    return Some(content);
                }
                if let Some(name) = raw_name {
                    let _ = reader.read_to_end(QName(&name));
                }
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }
    extract_bmstable_via_regex(html)
}

fn bmstable_content_from_meta(
    e: &BytesStart<'_>,
    decoder: quick_xml::Decoder,
) -> Option<String> {
    if !e.local_name().as_ref().eq_ignore_ascii_case(b"meta") {
        return None;
    }
    let mut name_is_bmstable = false;
    let mut content: Option<String> = None;
    for attr in e.html_attributes().with_checks(false).flatten() {
        let key = attr.key.local_name();
        let Ok(val) = attr.decoded_and_normalized_value(XmlVersion::Implicit1_0, decoder) else {
            continue;
        };
        if key.as_ref().eq_ignore_ascii_case(b"name") && val.eq_ignore_ascii_case("bmstable") {
            name_is_bmstable = true;
        } else if key.as_ref().eq_ignore_ascii_case(b"content") {
            content = Some(val.into_owned());
        }
    }
    if name_is_bmstable {
        content.filter(|c| !c.is_empty())
    } else {
        None
    }
}

static BMSTABLE_META_REGEXES: LazyLock<[Regex; 2]> = LazyLock::new(|| {
    [
        // <meta name="bmstable" content="header.json">
        Regex::new(
            r#"(?i)<meta[^>]+name\s*=\s*["']?bmstable["']?[^>]*content\s*=\s*["']([^"'\s>]+)["']?"#,
        )
        .expect("bmstable name-first regex"),
        // <meta content="header.json" name="bmstable">
        Regex::new(
            r#"(?i)<meta[^>]+content\s*=\s*["']([^"'\s>]+)["']?[^>]*name\s*=\s*["']?bmstable["']?"#,
        )
        .expect("bmstable content-first regex"),
    ]
});

fn extract_bmstable_via_regex(html: &str) -> Option<String> {
    for re in BMSTABLE_META_REGEXES.iter() {
        if let Some(caps) = re.captures(html) {
            let content = caps.get(1)?.as_str();
            if !content.is_empty() {
                return Some(content.to_string());
            }
        }
    }
    None
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
    let symbol = v
        .get("symbol")
        .and_then(|x| x.as_str())
        .unwrap_or("")
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
        symbol,
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

pub fn folder_name_for_level(level: &str, symbol: &str) -> String {
    format!("{symbol}{level}")
}
