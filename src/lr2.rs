use std::{fs::File, io::{BufReader, Write}, path::{Path, PathBuf}, rc::Rc, sync::Mutex};
use anyhow::{Result, bail};
use encoding_rs::SHIFT_JIS;
use quick_xml::se::EmptyElementHandling;
use serde::Serialize;
use slint::{Model, ModelRc, SharedString, VecModel};
use crate::{ApplicationGlobal, audio::reload_devices, utils::{find_player_in_array, jukebox_paths_to_slint_arr, slint_arr_to_jukebox_paths}};

pub mod lr2_config;

impl lr2_config::Config {
    pub fn load(lr2_folder_path: &PathBuf) -> Result<lr2_config::Config> {
        let config_file_path = lr2_folder_path.join("LR2files\\Config\\config.xml");
        let config_file = File::open(config_file_path)?;
        let config: lr2_config::Config = quick_xml::de::from_reader(BufReader::new(config_file))?;
        Ok(config)
    }

    fn write(&self, lr2_folder_path: &Path) -> Result<()> {
        let config_path = lr2_folder_path.join("LR2files\\Config\\config.xml");
        let mut config_file = File::create(config_path)?;

        let mut buffer = String::from("<?xml version=\"1.0\" encoding=\"shift_jis\"?>\n");
        let mut ser = quick_xml::se::Serializer::with_root(&mut buffer, Some("config"))?;
        ser.empty_element_handling(EmptyElementHandling::Expanded);
        ser.indent('\t', 1);
        self.serialize(ser).unwrap();
        buffer = buffer.replace("\n", "\r\n");

        let config_encoded = SHIFT_JIS.encode(&buffer);
        config_file.write_all(&config_encoded.0)?;

        Ok(())
    }
}

pub fn parse_players(lr2_folder_path: &PathBuf) -> Result<Vec<SharedString>> {
    let score_folder = lr2_folder_path.join("LR2files\\Database\\Score");
    let score_folder_exists = score_folder.try_exists().unwrap_or(false);
    if !score_folder_exists {
        bail!("Score folder does not exist");
    }

    let players = std::fs::read_dir(score_folder)?
        .filter_map(|res| res.ok())
        .map(|dir_entry| dir_entry.path())
        .filter_map(|path| {
            if path.extension().map_or(false, |ext| ext == "db") {
                let mut path2 = path.clone();
                path2.set_extension("");
                Some(path2.file_name()?.to_str()?.to_string().into())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(players)
}

pub fn load_lr2_config(app_globals: &ApplicationGlobal, lr2_path: &PathBuf) -> Result<lr2_config::Config> {
    let mut lr2_folder_path = lr2_path.clone();
    lr2_folder_path.pop();

    // TODO: handle case where config does not exist (aka very first launch)
    let players = parse_players(&lr2_folder_path).unwrap_or_else(|_| {panic!("Error reading scores, folder structure is probably wrong") });
    let config = lr2_config::Config::load(&lr2_folder_path).unwrap_or_else(|e| {panic!("{}", e) });

    // Home
    app_globals.set_players(ModelRc::from(Rc::new(VecModel::from(players.clone()))));
    match find_player_in_array(&players, &config.player.id) {
        Some(i) => {
            app_globals.set_selected_player(i32::try_from(i).unwrap());
            app_globals.set_password(config.player.pass.clone().into());
        }
        None => ()
    }
    app_globals.set_window_x(config.system.windowsize_x);
    app_globals.set_window_y(config.system.windowsize_y);
    app_globals.set_screenmode(config.system.screenmode.into());
    app_globals.set_autoreload(config.system.autoreload.into());
    app_globals.set_preview(config.select.preview);

    // Jukebox
    app_globals.set_jukebox_paths(jukebox_paths_to_slint_arr(&config.jukebox.path).unwrap());

    // Play
    app_globals.set_hsmin(config.play.hsmin);
    app_globals.set_hsmax(config.play.hsmax);
    app_globals.set_hsmargin(config.play.hsmargin);
    app_globals.set_shuttermargin(config.play.shuttermargin);
    app_globals.set_basespeed(config.play.basespeed);
    app_globals.set_folderlamp(config.select.folderlamp);

    app_globals.set_folder_random(config.system.customfolder.contains(lr2_config::Folders::Random));
    app_globals.set_folder_favorite(config.system.customfolder.contains(lr2_config::Folders::Favorite));
    app_globals.set_folder_ignored(config.system.customfolder.contains(lr2_config::Folders::Ignore));
    app_globals.set_folder_top10(config.system.customfolder.contains(lr2_config::Folders::Top10));
    app_globals.set_folder_level(config.system.customfolder.contains(lr2_config::Folders::Level));
    app_globals.set_folder_cleartype(config.system.customfolder.contains(lr2_config::Folders::Clear));
    app_globals.set_folder_playrank(config.system.customfolder.contains(lr2_config::Folders::Playrank));
    app_globals.set_folder_insanebms(config.system.customfolder.contains(lr2_config::Folders::InsaneBms));

    app_globals.set_searchmax(config.select.searchmax);
    app_globals.set_poorbga(config.play.poorbga);
    app_globals.set_inputinterval(config.system.inputinterval);

    // System
    app_globals.set_vsync(config.system.vsync);
    app_globals.set_audiooutput(config.sound.output);
    reload_devices(app_globals, config.sound.output);
    app_globals.set_selecteddriver(config.sound.driver);
    app_globals.set_bufferlength(config.sound.bufferlength);
    app_globals.set_disablefmod(config.sound.disablefmod);

    Ok(config)
}

pub fn save_new_lr2_config(app_globals: &ApplicationGlobal, config: &Mutex<lr2_config::Config>) {
    let mut config_new = config.lock().unwrap();

    // Home
    match app_globals.get_players().row_data(usize::try_from(app_globals.get_selected_player()).unwrap()) {
        Some(player) => { config_new.player.id = player.to_string() },
        None => { panic!("Player doesn't exist anymore") }
    };
    config_new.player.pass = app_globals.get_password().into();
    config_new.system.windowsize_x = app_globals.get_window_x();
    config_new.system.windowsize_y = app_globals.get_window_y();
    config_new.system.screenmode = u8::try_from(app_globals.get_screenmode()).unwrap();
    config_new.system.autoreload = u8::try_from(app_globals.get_autoreload()).unwrap();
    config_new.select.preview = app_globals.get_preview();

    // Jukebox
    config_new.jukebox.path = Some(slint_arr_to_jukebox_paths(app_globals));

    // Play
    config_new.play.hsmin = app_globals.get_hsmin();
    config_new.play.hsmax = app_globals.get_hsmax();
    config_new.play.hsmargin = app_globals.get_hsmargin();
    config_new.play.shuttermargin = app_globals.get_shuttermargin();
    config_new.play.basespeed = app_globals.get_basespeed();
    config_new.select.folderlamp = app_globals.get_folderlamp();

    let mut folders = lr2_config::Folders::empty();
    folders.set(lr2_config::Folders::Random, app_globals.get_folder_random());
    folders.set(lr2_config::Folders::Favorite, app_globals.get_folder_favorite());
    folders.set(lr2_config::Folders::Ignore, app_globals.get_folder_ignored());
    folders.set(lr2_config::Folders::Top10, app_globals.get_folder_top10());
    folders.set(lr2_config::Folders::Level, app_globals.get_folder_level());
    folders.set(lr2_config::Folders::Clear, app_globals.get_folder_cleartype());
    folders.set(lr2_config::Folders::Playrank, app_globals.get_folder_playrank());
    folders.set(lr2_config::Folders::InsaneBms, app_globals.get_folder_insanebms());
    config_new.system.customfolder = folders;

    config_new.select.searchmax = app_globals.get_searchmax();
    config_new.play.poorbga = app_globals.get_poorbga();
    config_new.system.inputinterval = app_globals.get_inputinterval();

    // System
    config_new.system.vsync = app_globals.get_vsync();
    config_new.sound.output = app_globals.get_audiooutput();
    config_new.sound.driver = app_globals.get_selecteddriver();
    config_new.sound.bufferlength = app_globals.get_bufferlength();
    config_new.sound.disablefmod = app_globals.get_disablefmod();

    let lr2_path: PathBuf = app_globals.get_lr2_path().to_string().into();
    let lr2_folder_path = lr2_path.parent().unwrap();
    config_new.write(lr2_folder_path).unwrap();
}
