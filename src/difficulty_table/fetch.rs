use anyhow::{Result, bail};
use nyquest::{ClientBuilder, Request};

pub fn http_get_text(url: &str) -> Result<String> {
    let client = ClientBuilder::default()
        .user_agent("lr2-launcher/alpha")
        .build_blocking()?;
    let resp = client.request(Request::get(url.to_string()))?;
    let bytes = resp.bytes().map_err(|e| anyhow::anyhow!("HTTP body error for {url}: {e}"))?;
    if bytes.is_empty() {
        bail!("Empty response from {url}");
    }

    Ok(decode_text(&bytes))
}

fn decode_text(bytes: &[u8]) -> String {
    // Some bmstable hosts ship UTF-8 JSON with a BOM (serde_json rejects it).
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    match String::from_utf8(bytes.to_vec()) {
        Ok(s) => s,
        Err(_) => {
            let (cow, _, _) = encoding_rs::SHIFT_JIS.decode(bytes);
            cow.into_owned()
        }
    }
}

pub fn resolve_url(base: &str, relative: &str) -> Result<String> {
    if relative.starts_with("http://") || relative.starts_with("https://") {
        return Ok(relative.to_string());
    }
    let relative = relative.trim_start_matches("./");

    let base_dir = if base.ends_with('/') {
        base.to_string()
    } else if let Some(idx) = base.rfind('/') {
        let after = &base[idx + 1..];
        let after_path = after.split('?').next().unwrap_or(after);
        if after_path.contains('.') {
            base[..idx + 1].to_string()
        } else {
            format!("{base}/")
        }
    } else {
        bail!("Invalid base URL: {base}");
    };

    if relative.starts_with('/') {
        let scheme_end = base
            .find("://")
            .map(|i| i + 3)
            .ok_or_else(|| anyhow::anyhow!("Invalid base URL: {base}"))?;
        let host_end = base[scheme_end..]
            .find('/')
            .map(|i| scheme_end + i)
            .unwrap_or(base.len());
        Ok(format!("{}{}", &base[..host_end], relative))
    } else {
        Ok(format!("{base_dir}{relative}"))
    }
}
