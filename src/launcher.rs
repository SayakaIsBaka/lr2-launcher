use std::{env::current_exe, fs, path::PathBuf};
use bstr::ByteSlice;
use slint::{ComponentHandle, language::ColorScheme};
use crate::{App, ApplicationGlobal, GameType, Palette, lr2::{self, lr2_config}, openlr2::{self, openlr2_config}};

pub mod config;

pub struct Game {
    pub typ: GameType,
    pub version: String
}

fn set_launcher_initial_values(app_globals: &ApplicationGlobal, launcher_config: &config::Config, game_type: &Game) {
    app_globals.set_lr2_path(launcher_config.lr2_path.clone().into_os_string().into_string().unwrap().into());
    app_globals.set_disable_score_save(launcher_config.disable_score);
    app_globals.set_darkmode(launcher_config.dark_mode);
    app_globals.set_language(launcher_config.language.clone() as i32);
    //TODO: actually set the application language to the selected one here

    app_globals.set_gametype(game_type.typ);
    app_globals.set_gameversion(game_type.version.clone().into());
}

pub fn get_binary_type(path: &PathBuf) -> Game {
    struct SearchArgs<'a> {
        typ: GameType,
        search_string: &'a str
    }

    static SEARCH_ARGS: [SearchArgs; 2] = [
        SearchArgs { typ: GameType::LR2, search_string: "LR2 beta3 version " },
        SearchArgs { typ: GameType::OpenLR2, search_string: "OpenLR2 version " },
    ];
    static VERSION_LEN: usize = 6;

    let mut game = Game { typ: GameType::Unknown, version: "".into() };
    let binary = fs::read(path).unwrap();

    for arg in SEARCH_ARGS.iter() {
        match binary.find(arg.search_string) {
            Some(idx) => {
                let ver = binary[idx + arg.search_string.len()..idx + arg.search_string.len() + VERSION_LEN].to_str().unwrap();
                game.version = ver.into();
                game.typ = arg.typ;
                break
            }
            None => {}
        }
    }

    game
}

pub fn init_launcher(app: &App) -> (lr2_config::Config, config::Config, openlr2_config::Config) {
    let app_globals = app.global::<ApplicationGlobal>();
    let launcher_path = current_exe().unwrap();
    let launcher_dir = launcher_path.parent().unwrap();

    let mut launcher_config = config::Config::load(launcher_dir).unwrap_or_else(|_| config::generate_default_config(launcher_dir).unwrap());
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

    let bin_type = get_binary_type(&launcher_config.lr2_path);

    set_launcher_initial_values(&app_globals, &launcher_config, &bin_type);
    let config = lr2::load_lr2_config(&app_globals, &launcher_config.lr2_path).unwrap();

    // We can still call it here even if it's vanilla because it returns a default object (and I'm too lazy to change more code)
    let openlr2_config = openlr2::load_openlr2_config(&app_globals, &launcher_config.lr2_path).unwrap();

    // Init color scheme from Rust (otherwise it is not applied on startup)
    app.global::<Palette>().set_color_scheme(if launcher_config.dark_mode { ColorScheme::Dark } else { ColorScheme::Light });

    (config, launcher_config, openlr2_config)
}
