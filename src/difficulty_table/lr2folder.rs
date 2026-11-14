use std::{fs, path::Path};

use anyhow::Result;
use encoding_rs::SHIFT_JIS;

struct FolderSpec {
    title: String,
    command: String,
}

fn level_folder(playlist_id: i64, folder: &str) -> FolderSpec {
    let folder_esc = folder.replace('\'', "''");
    FolderSpec {
        title: folder.to_string(),
        command: format!(
            "song.hash in (SELECT md5 FROM ll_playlist_entry WHERE playlist_id = {playlist_id} AND folder = '{folder_esc}')"
        ),
    }
}

fn alphabet_and_rest(playlist_id: i64) -> Vec<FolderSpec> {
    let ranges = [
        ("A.B.C.D.", "BETWEEN 'A' AND 'E' AND UPPER(title) != 'E'"),
        ("E.F.G.H.", "BETWEEN 'E' AND 'I' AND UPPER(title) != 'I'"),
        ("I.J.K.L.", "BETWEEN 'I' AND 'M' AND UPPER(title) != 'M'"),
        ("M.N.O.P.", "BETWEEN 'M' AND 'Q' AND UPPER(title) != 'Q'"),
        ("Q.R.S.T.", "BETWEEN 'Q' AND 'U' AND UPPER(title) != 'U'"),
        ("U.V.W.X.Y.Z.", "BETWEEN 'U' AND '[' AND UPPER(title) != '['"),
    ];
    let mut out = Vec::new();
    for (title, cond) in ranges {
        out.push(FolderSpec {
            title: title.into(),
            command: format!(
                "song.hash in (SELECT md5 FROM ll_playlist_entry WHERE playlist_id = {playlist_id} AND UPPER(title) {cond}) ORDER BY UPPER(song.title) ASC"
            ),
        });
    }
    out.push(FolderSpec {
        title: "OTHERS".into(),
        command: format!(
            "song.hash in (SELECT md5 FROM ll_playlist_entry WHERE playlist_id = {playlist_id} AND (UPPER(title) NOT BETWEEN 'A' AND '[' OR UPPER(title) = '[')) ORDER BY UPPER(song.title) ASC"
        ),
    });

    let base = format!(
        "song.hash in (SELECT md5 FROM ll_playlist_entry WHERE playlist_id = {playlist_id})"
    );
    let clears = [
        (
            "PERFECT ATTACK CLEAR",
            "(clear = 5 AND op_history & 16 != 0) ORDER BY minbp ASC",
        ),
        ("FULL COMBO CLEAR", "(clear = 5) ORDER BY minbp ASC"),
        ("HARD CLEAR", "(clear = 4) ORDER BY minbp ASC"),
        ("CLEAR", "(clear = 3) ORDER BY minbp ASC"),
        ("EASY CLEAR", "(clear = 2 AND rank != 0) ORDER BY minbp ASC"),
        ("ASSIST CLEAR", "(clear >= 2 AND rank = 0) ORDER BY minbp ASC"),
        ("FAILED", "(clear = 1) ORDER BY minbp ASC"),
        ("NO PLAY", "(clear IS NULL) ORDER BY minbp ASC"),
    ];
    for (title, cond) in clears {
        out.push(FolderSpec {
            title: title.into(),
            command: format!("{base} AND {cond}"),
        });
    }
    let ranks = [
        ("DJ LEVEL AAA", 8),
        ("DJ LEVEL AA", 7),
        ("DJ LEVEL A", 6),
        ("DJ LEVEL B", 5),
        ("DJ LEVEL C", 4),
        ("DJ LEVEL D", 3),
        ("DJ LEVEL E", 2),
        ("DJ LEVEL F", 1),
    ];
    for (title, rank) in ranks {
        out.push(FolderSpec {
            title: title.into(),
            command: format!("{base} AND (rank = {rank}) ORDER BY rate DESC"),
        });
    }
    out.push(FolderSpec {
        title: "NO SCORE".into(),
        command: format!("{base} AND (rank = 0 OR rank IS NULL ) ORDER BY rate DESC"),
    });
    out.push(FolderSpec {
        title: "ALL LONG NOTES".into(),
        command: format!("{base} AND (longnote = 1)"),
    });
    out.push(FolderSpec {
        title: "ALL VERY HARD JUDGES".into(),
        command: format!("{base} AND (judge = 0)"),
    });
    out.push(FolderSpec {
        title: "ALL HARD JUDGES".into(),
        command: format!("{base} AND (judge = 1)"),
    });
    out.push(FolderSpec {
        title: "ALL NORMAL JUDGES".into(),
        command: format!("{base} AND (judge = 2)"),
    });
    out.push(FolderSpec {
        title: "ALL EASY JUDGES".into(),
        command: format!("{base} AND (judge >= 3)"),
    });
    out.push(FolderSpec {
        title: "MY BEST".into(),
        command: format!("{base} ORDER BY playcount DESC"),
    });
    out.push(FolderSpec {
        title: "NEW SONGS".into(),
        command: format!(
            "{base} ORDER BY (SELECT adddate FROM ll_playlist_entry WHERE md5 = song.hash AND playlist_id = {playlist_id}) DESC"
        ),
    });
    out
}

fn write_lr2folder(path: &Path, category: &str, spec: &FolderSpec) -> Result<()> {
    let body = format!(
        "#COMMAND {}\r\n#MAXTRACKS 0\r\n#CATEGORY {}\r\n#TITLE {}\r\n#INFORMATION_A \r\n#INFORMATION_B \r\n\r\n",
        spec.command, category, spec.title
    );
    let (encoded, _, had_errors) = SHIFT_JIS.encode(&body);
    if had_errors {
        fs::write(path, body.as_bytes())?;
    } else {
        fs::write(path, encoded.as_ref())?;
    }
    Ok(())
}

/// Emit full folder set under `table_dir`. Replaces directory contents.
pub fn emit_table_folders(
    table_dir: &Path,
    category: &str,
    playlist_id: i64,
    level_folders: &[String],
) -> Result<()> {
    if table_dir.exists() {
        fs::remove_dir_all(table_dir)?;
    }
    fs::create_dir_all(table_dir)?;

    let mut specs: Vec<FolderSpec> = level_folders
        .iter()
        .map(|f| level_folder(playlist_id, f))
        .collect();
    specs.extend(alphabet_and_rest(playlist_id));

    for (i, spec) in specs.iter().enumerate() {
        let filename = format!("{i:04}.lr2folder");
        write_lr2folder(&table_dir.join(filename), category, spec)?;
    }
    Ok(())
}
