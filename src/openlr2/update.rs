use anyhow::{Result, bail};
use nyquest::{ClientBuilder, Request};
use serde_json::Value;

pub fn check_updates(cur_version: String) -> Result<Option<String>> {
    let client = ClientBuilder::default().user_agent("curl/8.4.0").build_blocking()?;
    let body: Value = client.request(Request::get("https://api.github.com/repos/GOMazk/OpenLR2/releases/latest"))?.json()?;
    let latest_version = match body["tag_name"].as_str() {
        Some(s) => s,
        None => bail!("Error parsing tag name")
    };

    // Assuming tag names are all in the format "v123456" (hopefully this doesn't change)
    if latest_version[1..] != cur_version {
        Ok(Some(latest_version.to_string()))
    } else {
        Ok(None)
    }
}