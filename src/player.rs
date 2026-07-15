use std::{fs, path::Path};
use anyhow::{Result, bail};
use rusqlite::Connection;

pub fn create_new_player(username: String, password: String, lr2_folder_path: &Path) -> Result<()> {
    let mut player_db = lr2_folder_path.join("LR2files\\Database\\Score\\").join(&username);
    player_db.add_extension("db");
    if player_db.exists() {
        bail!("User already exists!");
    }
    let conn = Connection::open(player_db.as_os_str().to_str().unwrap())?;
    
    conn.execute(
        r#"CREATE TABLE player(
            id TEXT primary key,
            hash TEXT,
            name TEXT,
            irid INTEGER,
            irname TEXT,
            playcount INTEGER,
            clear INTEGER,
            fail INTEGER,
            perfect INTEGER,
            great INTEGER,
            good INTEGER,
            bad INTEGER,
            poor INTEGER,
            playtime INTEGER,
            combo INTEGER,
            maxcombo INTEGER,
            grade_7 INTEGER,
            grade_5 INTEGER,
            grade_14 INTEGER,
            grade_10 INTEGER,
            grade_9 INTEGER,
            trial INTEGER,
            option INTEGER,
            systemversion INTEGER,
            gradeversion INTEGER,
            trialversion INTEGER,
            scorehash TEXT
        )"#,
        ()
    )?;

    conn.execute(
        r#"CREATE TABLE score(
            hash TEXT primary key,
            clear INTEGER,
            perfect INTEGER,
            great INTEGER,
            good INTEGER,
            bad INTEGER,
            poor INTEGER,
            totalnotes INTEGER,
            maxcombo INTEGER,
            minbp INTEGER,
            playcount INTEGER,
            clearcount INTEGER,
            failcount INTEGER,
            rank INTEGER,
            rate INTEGER,
            clear_db INTEGER,
            op_history INTEGER,
            scorehash TEXT,
            ghost TEXT,
            clear_sd INTEGER,
            clear_ex INTEGER,
            op_best INTEGER,
            rseed INTEGER,
            complete INTEGER
        )"#,
        ()
    )?;

    conn.execute("CREATE INDEX hashidx ON score (hash)", ())?;

    let digest = md5::compute(password);
    let password_hash = format!("{:x}", digest);

    conn.execute(
    r#"INSERT INTO main.player (
            "id", "hash", "name", "irid", "irname", "playcount", "clear",
            "fail", "perfect", "great", "good", "bad", "poor", "playtime",
            "combo", "maxcombo", "grade_7", "grade_5", "grade_14", "grade_10",
            "grade_9", "trial", "option", "systemversion", "gradeversion", "trialversion", "scorehash"
        ) VALUES (
            ?1, ?2, ?1, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,
            NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, NULL, NULL, NULL
        );"#,
        [username, password_hash]
    )?;

    Ok(())
}

pub fn delete_player(username: String, lr2_folder_path: &Path) {
    let mut player_db = lr2_folder_path.join("LR2files\\Database\\Score\\").join(&username);
    player_db.add_extension("db");
    if !player_db.exists() {
        return // Database file doesn't exist anymore for some reason
    }

    let _ = fs::remove_file(player_db);
}