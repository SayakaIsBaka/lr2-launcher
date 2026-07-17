use std::{fs, path::{Path, PathBuf}, rc::Rc};
use anyhow::{Result, bail};
use rusqlite::{Connection, OpenFlags};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, language::ColorScheme};

use crate::{App, ApplicationGlobal, NewUser, Palette, lr2, utils};

pub fn show_new_player_window(app: &App) {
    let new_user_window = NewUser::new().unwrap();
    let is_dark = app.global::<ApplicationGlobal>().get_darkmode();
    new_user_window.global::<Palette>().set_color_scheme(if is_dark { ColorScheme::Dark } else { ColorScheme::Light });

    new_user_window.on_user_create_ok({
        let new_user_window_weak = new_user_window.as_weak();
        let app_weak = app.as_weak();

        move |username: SharedString, password: SharedString| {
            let app = app_weak.unwrap();
            let app_globals = app.global::<ApplicationGlobal>();
            let lr2_path: PathBuf = app_globals.get_lr2_path().to_string().clone().into();
            let lr2_folder_path = lr2_path.parent().unwrap();

            let new_user_window = new_user_window_weak.unwrap();
            match create_new_player(username.clone().into(), password.clone().into(), lr2_folder_path) {
                Ok(()) => {
                    let players = lr2::parse_players(&lr2_folder_path.to_path_buf()).unwrap();
                    app_globals.set_players(ModelRc::from(Rc::new(VecModel::from(players.clone()))));
                    match utils::find_player_in_array(&players, &username.to_string()) {
                        Some(i) => {
                            app_globals.set_selected_player(i32::try_from(i).unwrap());
                            app_globals.set_password(password.clone());
                        }
                        None => () // This isn't really supposed to happen
                    }
                    app.show().unwrap();
                    new_user_window.hide().unwrap()
                }
                Err(e) => { new_user_window.set_error_text(e.to_string().into()) }
            };
        }
    });

    new_user_window.on_user_create_cancel({
        let new_user_window_weak = new_user_window.as_weak();

        move || {
            new_user_window_weak.unwrap().hide().unwrap();
        }
    });

    new_user_window.show().unwrap();
}

pub fn create_new_player(username: String, password: String, lr2_folder_path: &Path) -> Result<()> {
    let mut player_db = lr2_folder_path.join("LR2files\\Database\\Score\\").join(&username);
    player_db.add_extension("db");
    if player_db.exists() {
        bail!("User already exists!");
    }
    let conn = Connection::open(&player_db)?;
    
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

    let _ = fs::remove_file(player_db);
}

pub fn are_credentials_valid(username: &String, password: &String, lr2_folder_path: &Path) -> Result<bool> {
    let mut player_db = lr2_folder_path.join("LR2files\\Database\\Score\\").join(&username);
    player_db.add_extension("db");

    let conn = Connection::open_with_flags(&player_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut stmt = conn.prepare("SELECT hash FROM player")?;
    let res = stmt.query_one([], |row| {
        let ref_hash: String = row.get(0)?;
        let digest = md5::compute(password);
        let password_hash = format!("{:x}", digest);

        Ok(ref_hash == password_hash)
    })?;

    Ok(res)
}