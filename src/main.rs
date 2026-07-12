#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod lr2_config;
mod wasapi;
mod directsound;
mod launcher_config;

use quick_xml::{events::{BytesDecl, Event}, se::EmptyElementHandling};
use serde::Serialize;
use slint::{CloseRequestResponse, Model, ModelRc, SharedString, VecModel, language::{ColorScheme, StandardListViewItem}};
use std::{env::current_exe, fs::File, io::{BufReader, BufWriter, Write}, path::{Path, PathBuf}, rc::Rc};
use anyhow::{Result, bail};
use encoding_rs::SHIFT_JIS;

use crate::lr2_config::Folders;

slint::include_modules!();

pub fn main() {
    slint::init_translations!(concat!(env!("CARGO_MANIFEST_DIR"), "/lang/"));

    let app = App::new().unwrap();
    let (config, launcher_config) = init_launcher(&app);
    setup_callbacks(&app, config, launcher_config);

    app.run().unwrap();
}

fn setup_callbacks(app: &App, config: lr2_config::Config, launcher_config: launcher_config::Config) {
    app.on_audio_type_change({
        let app_weak = app.as_weak();
        move || {
            let app = app_weak.unwrap();
            let app_globals = app.global::<ApplicationGlobal>();

            app_globals.set_selecteddriver(0);
            reload_devices(&app_globals, app_globals.get_audiooutput());
        }
    });

    // TODO: make buttons do things (including language)

    app.window().on_close_requested({
        let app_weak = app.as_weak();
        move || {
            let app = app_weak.unwrap();
            let app_globals = app.global::<ApplicationGlobal>();

            save_new_lr2_config(&app_globals, &config);
            save_new_launcher_config(&app_globals, &launcher_config);
            
            CloseRequestResponse::HideWindow
        }
    });
}

fn save_new_lr2_config(app_globals: &ApplicationGlobal, config: &lr2_config::Config) {
    let mut config_new = config.clone();

    // Home
    config_new.system.windowsize_x = app_globals.get_window_x();
    config_new.system.windowsize_y = app_globals.get_window_y();
    config_new.system.screenmode = u8::try_from(app_globals.get_screenmode()).unwrap();
    config_new.system.autoreload = u8::try_from(app_globals.get_autoreload()).unwrap();
    config_new.select.preview = app_globals.get_preview();

    // Jukebox
    let mut new_paths: Vec<String> = vec![];
    for path in app_globals.get_jukebox_paths().iter() {
        new_paths.push(path.text.into());
    }
    config_new.jukebox.path = new_paths;

    // Play
    config_new.play.hsmin = app_globals.get_hsmin();
    config_new.play.hsmax = app_globals.get_hsmax();
    config_new.play.hsmargin = app_globals.get_hsmargin();
    config_new.play.shuttermargin = app_globals.get_shuttermargin();
    config_new.play.basespeed = app_globals.get_basespeed();
    config_new.select.folderlamp = app_globals.get_folderlamp();

    let mut folders = Folders::empty();
    folders.set(Folders::Random, app_globals.get_folder_random());
    folders.set(Folders::Favorite, app_globals.get_folder_favorite());
    folders.set(Folders::Ignore, app_globals.get_folder_ignored());
    folders.set(Folders::Top10, app_globals.get_folder_top10());
    folders.set(Folders::Level, app_globals.get_folder_level());
    folders.set(Folders::Clear, app_globals.get_folder_cleartype());
    folders.set(Folders::Playrank, app_globals.get_folder_playrank());
    folders.set(Folders::InsaneBms, app_globals.get_folder_insanebms());
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
    write_lr2_config(lr2_folder_path, &config_new).unwrap();
}

fn write_lr2_config(lr2_folder_path: &Path, config: &lr2_config::Config) -> Result<()> {
    let config_path = lr2_folder_path.join("LR2files\\Config\\config.xml");
    let mut config_file = File::create(config_path)?;

    let mut buffer = String::from("<?xml version=\"1.0\" encoding=\"shift_jis\"?>\n");
    let mut ser = quick_xml::se::Serializer::with_root(&mut buffer, Some("config"))?;
    ser.empty_element_handling(EmptyElementHandling::Expanded);
    ser.indent('\t', 1);
    config.serialize(ser).unwrap();
    buffer = buffer.replace("\n", "\r\n");

    let config_encoded = SHIFT_JIS.encode(&buffer);
    config_file.write_all(&config_encoded.0)?;

    Ok(())
}

fn save_new_launcher_config(app_globals: &ApplicationGlobal, launcher_config: &launcher_config::Config) {
    let mut launcher_config_new = launcher_config.clone();

    launcher_config_new.dark_mode = app_globals.get_darkmode();
    launcher_config_new.disable_score = app_globals.get_disable_score_save();
    launcher_config_new.lr2_path = app_globals.get_lr2_path().to_string().into();
    launcher_config_new.language = app_globals.get_language().try_into().unwrap();

    let launcher_path = current_exe().unwrap();
    let launcher_dir = launcher_path.parent().unwrap();
    write_launcher_config(launcher_dir, &launcher_config_new).unwrap();
}

fn load_launcher_config(launcher_dir: &Path) -> Result<launcher_config::Config> {
    let launcher_config_path = launcher_dir.join("lr2-launcher.xml");
    let config_file = File::open(launcher_config_path)?;
    let config: launcher_config::Config = quick_xml::de::from_reader(BufReader::new(config_file))?;

    Ok(config)
}

fn write_launcher_config(launcher_dir: &Path, config: &launcher_config::Config) -> Result<()> {
    let launcher_config_path = launcher_dir.join("lr2-launcher.xml");
    let config_file = File::create(launcher_config_path)?;

    let mut writer = quick_xml::Writer::new_with_indent(BufWriter::new(config_file), b' ', 4);
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
    writer.write_serializable("config", config)?;

    Ok(())
}

fn generate_default_config(launcher_dir: &Path) -> Result<launcher_config::Config> {
    let default_config = launcher_config::Config {
        lr2_path: launcher_dir.join("LR2body.exe"),
        dark_mode: false,
        language: launcher_config::Language::English,
        disable_score: false,
    };
    write_launcher_config(launcher_dir, &default_config)?;
    Ok(default_config)
}

fn init_launcher(app: &App) -> (lr2_config::Config, launcher_config::Config) {
    let app_globals = app.global::<ApplicationGlobal>();
    let launcher_path = current_exe().unwrap();
    let launcher_dir = launcher_path.parent().unwrap();

    let mut launcher_config = load_launcher_config(launcher_dir).unwrap_or_else(|_| generate_default_config(launcher_dir).unwrap());
    let lr2body_exists = launcher_config.lr2_path.try_exists().unwrap_or(false);

    if !lr2body_exists {
        rfd::MessageDialog::new()
            .set_buttons(rfd::MessageButtons::Ok)
            .set_level(rfd::MessageLevel::Info)
            .set_title(app_globals.get_window_title())
            .set_description("LR2body.exe not found in current directory, please select LR2body.exe / OpenLR2 binary in the following window.")
            .show();
        launcher_config.lr2_path = match rfd::FileDialog::new()
            .add_filter("LR2 executable", &["exe"])
            .set_title("Pick LR2 executable...")
            .pick_file() {
                Some(path) => path,
                None => panic!("No LR2 executable path given, exiting")
            };
    }
    
    let mut lr2_folder_path = launcher_config.lr2_path.clone();
    lr2_folder_path.pop();
    let players = parse_players(&lr2_folder_path).unwrap_or_else(|_| {panic!("Error reading scores, folder structure is probably wrong") });

    let users = Rc::new(VecModel::from(players));
    app_globals.set_players(ModelRc::from(users));
    app_globals.set_lr2_path(launcher_config.lr2_path.clone().into_os_string().into_string().unwrap().into());

    let config = parse_lr2_config(&lr2_folder_path).unwrap_or_else(|e| {panic!("{}", e) });
    set_initial_values(&app_globals, &launcher_config, &config);

    // Init color scheme from Rust (otherwise it is not applied on startup)
    app.global::<Palette>().set_color_scheme(if launcher_config.dark_mode { ColorScheme::Dark } else { ColorScheme::Light });

    (config, launcher_config)
}

fn parse_players(lr2_folder_path: &PathBuf) -> Result<Vec<SharedString>> {
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

fn parse_lr2_config(lr2_folder_path: &PathBuf) -> Result<lr2_config::Config> {
    let config_file_path = lr2_folder_path.join("LR2files\\Config\\config.xml");
    let config_file = File::open(config_file_path)?;
    let config: lr2_config::Config = quick_xml::de::from_reader(BufReader::new(config_file))?;
    Ok(config)
}

fn jukebox_paths_to_slint_arr(config: &lr2_config::Config) -> Result<ModelRc<StandardListViewItem>> {
    let mut standard_list_view_vec: Vec<StandardListViewItem> = vec![];
    for path in &config.jukebox.path {
        standard_list_view_vec.push(StandardListViewItem::from(path.as_str()));
    }
    Ok(VecModel::from_slice(standard_list_view_vec.as_slice()))
}

fn get_audio_devices(device_type: i32) -> Result<ModelRc<SharedString>> {
    let mut device_list_vec: Vec<SharedString> = vec![];
    match device_type {
        0 => { // DirectSound
            device_list_vec = directsound::get_devices()?;
        },
        1 => { // WASAPI
            let wasapi_enumerator = wasapi::WasapiDeviceEnumerator::new()?;
            device_list_vec = wasapi_enumerator.get_devices()?;
        },
        2 => { // ASIO
            let asio_key = windows_registry::LOCAL_MACHINE.open("SOFTWARE\\ASIO")?;
            for key in asio_key.keys()? {
                device_list_vec.push(key.into());
            }
        },
        _ => {
            bail!("Invalid device type");
        }
    }
    let device_list = Rc::new(VecModel::from(device_list_vec));
    Ok(ModelRc::from(device_list))
}

fn reload_devices(app_globals: &ApplicationGlobal, device_type: i32) {
    app_globals.set_drivers(get_audio_devices(device_type).unwrap_or_default());
}

fn set_initial_values(app_globals: &ApplicationGlobal, launcher_config: &launcher_config::Config, config: &lr2_config::Config) {
    // Home
    app_globals.set_window_x(config.system.windowsize_x);
    app_globals.set_window_y(config.system.windowsize_y);
    app_globals.set_screenmode(config.system.screenmode.into());
    app_globals.set_autoreload(config.system.autoreload.into());
    app_globals.set_preview(config.select.preview);
    app_globals.set_disable_score_save(launcher_config.disable_score);

    // Jukebox
    app_globals.set_jukebox_paths(jukebox_paths_to_slint_arr(config).unwrap());

    // Play
    app_globals.set_hsmin(config.play.hsmin);
    app_globals.set_hsmax(config.play.hsmax);
    app_globals.set_hsmargin(config.play.hsmargin);
    app_globals.set_shuttermargin(config.play.shuttermargin);
    app_globals.set_basespeed(config.play.basespeed);
    app_globals.set_folderlamp(config.select.folderlamp);

    app_globals.set_folder_random(config.system.customfolder.contains(Folders::Random));
    app_globals.set_folder_favorite(config.system.customfolder.contains(Folders::Favorite));
    app_globals.set_folder_ignored(config.system.customfolder.contains(Folders::Ignore));
    app_globals.set_folder_top10(config.system.customfolder.contains(Folders::Top10));
    app_globals.set_folder_level(config.system.customfolder.contains(Folders::Level));
    app_globals.set_folder_cleartype(config.system.customfolder.contains(Folders::Clear));
    app_globals.set_folder_playrank(config.system.customfolder.contains(Folders::Playrank));
    app_globals.set_folder_insanebms(config.system.customfolder.contains(Folders::InsaneBms));

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

    // Launcher settings
    app_globals.set_darkmode(launcher_config.dark_mode);
    app_globals.set_language(launcher_config.language.clone() as i32);
    //TODO: actually set the application language to the selected one here
}