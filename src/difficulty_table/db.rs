use std::path::Path;
use anyhow::{Result, bail};
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Debug, Clone)]
pub struct PlaylistRow {
    pub playlist_id: i64,
    pub name: String,
    pub page_url: String,
    #[allow(dead_code)]
    pub output_dir: String,
}

#[derive(Debug, Clone)]
pub struct PlaylistEntryRow {
    pub playlist_id: i64,
    pub md5: String,
    pub title: String,
    pub folder: String,
    pub adddate: i64,
}

pub fn open_song_db(game_dir: &Path) -> Result<Connection> {
    let path = game_dir.join("LR2files").join("Database").join("song.db");
    if !path.exists() {
        bail!("song.db not found at {}", path.display());
    }
    let conn = Connection::open(path)?;

    // Recreate if leftover wide-schema prototype under ll_* names.
    let needs_reset = conn
        .prepare("SELECT symbol FROM ll_playlist LIMIT 1")
        .is_ok()
        || conn
            .prepare("SELECT is_removed FROM ll_playlist_entry LIMIT 1")
            .is_ok();
    if needs_reset {
        conn.execute_batch(
            r#"
            DROP TABLE IF EXISTS ll_playlist_entry;
            DROP TABLE IF EXISTS ll_playlist;
            "#,
        )?;
    }

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS ll_playlist (
            playlist_id integer primary key autoincrement not null,
            name text not null,
            page_url text not null unique,
            output_dir text not null
        );

        CREATE TABLE IF NOT EXISTS ll_playlist_entry (
            playlist_id integer not null,
            md5 text not null,
            title text not null default '',
            folder text not null,
            adddate integer not null default 0,
            primary key (playlist_id, md5, folder)
        );

        CREATE INDEX IF NOT EXISTS ll_playlist_entry_idx_id
            on ll_playlist_entry(playlist_id);
        CREATE INDEX IF NOT EXISTS ll_playlist_entry_idx_folder
            on ll_playlist_entry(playlist_id, folder);
        CREATE INDEX IF NOT EXISTS ll_playlist_entry_idx_title
            on ll_playlist_entry(playlist_id, title);
        CREATE INDEX IF NOT EXISTS ll_playlist_entry_idx_adddate
            on ll_playlist_entry(playlist_id, adddate);
        "#,
    )?;
    Ok(conn)
}

pub fn list_playlists(conn: &Connection) -> Result<Vec<PlaylistRow>> {
    let mut stmt = conn.prepare(
        r#"SELECT playlist_id, name, page_url, output_dir
           FROM ll_playlist ORDER BY playlist_id"#,
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(PlaylistRow {
            playlist_id: row.get(0)?,
            name: row.get(1)?,
            page_url: row.get(2)?,
            output_dir: row.get(3)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn get_output_dir(conn: &Connection, playlist_id: i64) -> Result<Option<String>> {
    let dir: Option<String> = conn
        .query_row(
            "SELECT output_dir FROM ll_playlist WHERE playlist_id = ?1",
            params![playlist_id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(dir)
}

pub fn delete_playlist(conn: &Connection, playlist_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM ll_playlist_entry WHERE playlist_id = ?1",
        params![playlist_id],
    )?;
    conn.execute(
        "DELETE FROM ll_playlist WHERE playlist_id = ?1",
        params![playlist_id],
    )?;
    Ok(())
}

pub fn insert_playlist(
    conn: &Connection,
    name: &str,
    page_url: &str,
    output_dir: &str,
) -> Result<i64> {
    conn.execute(
        r#"INSERT INTO ll_playlist (name, page_url, output_dir)
           VALUES (?1, ?2, ?3)"#,
        params![name, page_url, output_dir],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn insert_entries(conn: &Connection, entries: &[PlaylistEntryRow]) -> Result<()> {
    let mut stmt = conn.prepare(
        r#"INSERT OR IGNORE INTO ll_playlist_entry (
            playlist_id, md5, title, folder, adddate
        ) VALUES (?1, ?2, ?3, ?4, ?5)"#,
    )?;
    for e in entries {
        stmt.execute(params![
            e.playlist_id,
            e.md5,
            e.title,
            e.folder,
            e.adddate,
        ])?;
    }
    Ok(())
}
