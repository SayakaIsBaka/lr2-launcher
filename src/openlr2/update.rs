use anyhow::{Result, bail};
use nyquest::{ClientBuilder, Request};
use serde_json::Value;

pub fn check_updates(cur_version: String, is_64bit: bool) -> Result<Option<(String, Option<String>)>> {
    let arch_str = if is_64bit { "x64" } else { "x86" };

    let client = ClientBuilder::default().user_agent("curl/8.4.0").build_blocking()?;
    let body: Value = client.request(Request::get("https://api.github.com/repos/GOMazk/OpenLR2/releases/latest"))?.json()?;
    let latest_version = match body["tag_name"].as_str() {
        Some(s) => s,
        None => bail!("Error parsing tag name")
    };

    // Assuming tag names are all in the format "v123456" (hopefully this doesn't change)
    if latest_version[1..] != cur_version {
        let assets = match body["assets"].as_array() {
            Some(l) => l,
            None => bail!("Error parsing assets list")
        };
        let assets_filtered = assets.iter().filter(|&x| x["name"].as_str().unwrap().contains(arch_str) && x["content_type"] == "application/x-zip-compressed").collect::<Vec<&Value>>();
        if assets_filtered.len() != 1 {
            // Error getting URL but still say that there's an update available
            return Ok(Some((latest_version.to_string(), None)));
        }
        let download_url = match assets_filtered[0]["browser_download_url"].as_str() {
            Some(s) => s,
            None => { return Ok(Some((latest_version.to_string(), None))) }
        };

        Ok(Some((latest_version.to_string(), Some(download_url.to_string()))))
    } else {
        Ok(None)
    }
}