use std::{env::current_exe, fs::File, io::{BufReader, BufWriter}, path::{Path, PathBuf}, sync::Mutex};
use quick_xml::events::{BytesDecl, Event};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use anyhow::Result;

use crate::ApplicationGlobal;

#[derive(Serialize, Deserialize, Clone)]
pub enum Language {
    English = 0,
    Japanese,
    Korean
}

impl TryFrom<i32> for Language {
    type Error = ();

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == Language::English as i32 => Ok(Language::English),
            x if x == Language::Japanese as i32 => Ok(Language::Japanese),
            x if x == Language::Korean as i32 => Ok(Language::Korean),
            _ => Err(()),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub lr2_path: PathBuf,
    pub dark_mode: bool,
    pub language: Language,
    pub disable_score: bool,
}

fn write_launcher_config(launcher_dir: &Path, config: &Config) -> Result<()> {
    let launcher_config_path = launcher_dir.join("lr2-launcher.xml");
    let config_file = File::create(launcher_config_path)?;

    let mut writer = quick_xml::Writer::new_with_indent(BufWriter::new(config_file), b' ', 4);
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
    writer.write_serializable("config", config)?;

    Ok(())
}

pub fn generate_default_config(launcher_dir: &Path) -> Result<Config> {
    let default_config = Config {
        lr2_path: launcher_dir.join("LR2body.exe"),
        dark_mode: false,
        language: Language::English,
        disable_score: false,
    };
    write_launcher_config(launcher_dir, &default_config)?;
    Ok(default_config)
}

pub fn save_new_launcher_config(app_globals: &ApplicationGlobal, launcher_config: &Mutex<Config>) {
    let mut launcher_config_new = launcher_config.lock().unwrap();

    launcher_config_new.dark_mode = app_globals.get_darkmode();
    launcher_config_new.disable_score = app_globals.get_disable_score_save();
    launcher_config_new.lr2_path = app_globals.get_lr2_path().to_string().into();
    launcher_config_new.language = app_globals.get_language().try_into().unwrap();

    let launcher_path = current_exe().unwrap();
    let launcher_dir = launcher_path.parent().unwrap();
    write_launcher_config(launcher_dir, &launcher_config_new).unwrap();
}

pub fn load_launcher_config(launcher_dir: &Path) -> Result<Config> {
    let launcher_config_path = launcher_dir.join("lr2-launcher.xml");
    let config_file = File::open(launcher_config_path)?;
    let config: Config = quick_xml::de::from_reader(BufReader::new(config_file))?;

    Ok(config)
}