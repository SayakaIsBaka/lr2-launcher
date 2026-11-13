use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use rusqlite::{OptionalExtension, params};

use super::{
    db::{self, PlaylistEntryRow, PlaylistRow},
    fetch, lr2folder, parse,
};

pub fn normalize_jukebox_path(path: &Path) -> String {
    let mut s = path.to_string_lossy().replace('/', "\\");
    if !s.ends_with('\\') {
        s.push('\\');
    }
    s
}

pub fn ensure_jukebox_has_path(paths: &mut Vec<String>, output_base: &Path) -> bool {
    let want = normalize_jukebox_path(output_base);
    let want_trim = want.trim_end_matches('\\').to_ascii_lowercase();
    for p in paths.iter() {
        let existing = p.trim_end_matches('\\').to_ascii_lowercase();
        if existing == want_trim {
            return false;
        }
    }
    paths.push(want);
    true
}

pub fn load_tablelist(url_or_path: &str) -> Result<Vec<parse::TableListItem>> {
    let text = if url_or_path.starts_with("http://") || url_or_path.starts_with("https://") {
        fetch::http_get_text(url_or_path)?
    } else {
        let path = PathBuf::from(url_or_path.trim().trim_matches('"'));
        fs::read_to_string(&path).map_err(|e| anyhow::anyhow!("Read {}: {e}", path.display()))?
    };
    parse::parse_tablelist_json(&text)
}

pub fn add_or_refresh_table(
    lr2_folder_path: &Path,
    output_base: &Path,
    page_url: &str,
) -> Result<PlaylistRow> {
    if !output_base.exists() {
        fs::create_dir_all(output_base)?;
    }

    let page_html = fetch::http_get_text(page_url)?;
    let header_rel = parse::extract_bmstable_meta(&page_html)
        .ok_or_else(|| anyhow::anyhow!("No bmstable meta on page: {page_url}"))?;
    let header_url = fetch::resolve_url(page_url, &header_rel)?;
    let header_text = fetch::http_get_text(&header_url)?;
    let header = parse::parse_header_json(&header_text)?;
    let data_url = fetch::resolve_url(&header_url, &header.data_url)?;
    let data_text = fetch::http_get_text(&data_url)?;
    let charts = parse::parse_data_json(&data_text)?;

    let level_order = if header.level_order.is_empty() {
        let mut order = Vec::new();
        for c in &charts {
            if !order.iter().any(|x| x == &c.level) {
                order.push(c.level.clone());
            }
        }
        order
    } else {
        header.level_order.clone()
    };
    let level_folders: Vec<String> = level_order
        .iter()
        .map(|l| parse::folder_name_for_level(l))
        .collect();

    let mut safe_name: String = header
        .name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    safe_name = safe_name.trim().trim_matches('.').to_string();
    if safe_name.is_empty() {
        safe_name = "table".into();
    }
    if safe_name.len() > 80 {
        safe_name.truncate(80);
    }

    let table_dir = output_base.join(&safe_name);
    let table_dir_str = table_dir.to_string_lossy().to_string();
    let adddate = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let conn = db::open_song_db(lr2_folder_path)?;
    let tx = conn.unchecked_transaction()?;

    let existing_id: Option<i64> = tx
        .query_row(
            "SELECT playlist_id FROM ll_playlist WHERE page_url = ?1",
            params![page_url],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(existing_id) = existing_id {
        if let Ok(Some(old_path)) = db::get_output_dir(&tx, existing_id) {
            let p = PathBuf::from(&old_path);
            if p.exists() && p != table_dir {
                let _ = fs::remove_dir_all(&p);
            }
        }
        db::delete_playlist(&tx, existing_id)?;
    }

    let playlist_id = db::insert_playlist(&tx, &header.name, page_url, &table_dir_str)?;

    let mut entries = Vec::with_capacity(charts.len());
    for c in &charts {
        entries.push(PlaylistEntryRow {
            playlist_id,
            md5: c.md5.clone(),
            title: c.title.clone(),
            folder: parse::folder_name_for_level(&c.level),
            adddate,
        });
    }
    db::insert_entries(&tx, &entries)?;
    tx.commit()?;

    lr2folder::emit_table_folders(&table_dir, &header.name, playlist_id, &level_folders)?;

    Ok(PlaylistRow {
        playlist_id,
        name: header.name,
        page_url: page_url.to_string(),
        output_dir: table_dir_str,
    })
}

pub fn remove_table(lr2_folder_path: &Path, playlist_id: i64) -> Result<()> {
    let conn = db::open_song_db(lr2_folder_path)?;
    let output_dir = db::get_output_dir(&conn, playlist_id)?;
    db::delete_playlist(&conn, playlist_id)?;
    if let Some(dir) = output_dir {
        let p = PathBuf::from(dir);
        if p.exists() {
            fs::remove_dir_all(p)?;
        }
    }
    Ok(())
}

pub fn refresh_all(lr2_folder_path: &Path, output_base: &Path) -> Result<Vec<String>> {
    let conn = db::open_song_db(lr2_folder_path)?;
    let list = db::list_playlists(&conn)?;
    drop(conn);

    let mut errors = Vec::new();
    for p in list {
        if let Err(e) = add_or_refresh_table(lr2_folder_path, output_base, &p.page_url) {
            errors.push(format!("{}: {e}", p.name));
        }
    }
    Ok(errors)
}
