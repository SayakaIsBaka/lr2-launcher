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

    // Prefer UTF-8, fall back to Shift_JIS
    match String::from_utf8(bytes.clone()) {
        Ok(s) => Ok(s),
        Err(_) => {
            let (cow, _, _) = encoding_rs::SHIFT_JIS.decode(&bytes);
            Ok(cow.into_owned())
        }
    }
}

pub fn resolve_url(base: &str, relative: &str) -> Result<String> {
    if relative.starts_with("http://") || relative.starts_with("https://") {
        return Ok(relative.to_string());
    }

    let base = base.trim_end_matches('/');
    let base_dir = if let Some(idx) = base.rfind('/') {
        let after = &base[idx + 1..];
        if after.contains('.') && !after.contains('?') {
            &base[..idx + 1]
        } else if base.ends_with('/') {
            base
        } else {
            &base[..idx + 1]
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
