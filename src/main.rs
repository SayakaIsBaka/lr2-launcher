#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod lr2;
mod audio;
mod launcher;
mod player;
mod process;
mod utils;
mod openlr2;

use slint::{CloseRequestResponse, Model, ModelRc, VecModel, run_event_loop};
use std::{path::PathBuf, rc::Rc, sync::{Arc, Mutex}};

use crate::{lr2::lr2_config, openlr2::openlr2_config};
use crate::launcher::config;

slint::include_modules!();

pub fn main() {
    slint::init_translations!(concat!(env!("CARGO_MANIFEST_DIR"), "/lang/"));

    let app = App::new().unwrap();
    let (config, launcher_config, openlr2_config) = launcher::init_launcher(&app);
    let config_arc = Arc::new(Mutex::new(config));
    let launcher_config_arc = Arc::new(Mutex::new(launcher_config));
    let openlr2_config_arc = Arc::new(Mutex::new(openlr2_config));

    setup_callbacks(&app, config_arc, launcher_config_arc, openlr2_config_arc);

    if app.global::<ApplicationGlobal>().get_players().row_count() == 0 {
        player::show_new_player_window(&app);
    } else {
        app.show().unwrap();
    }
    run_event_loop().unwrap();
    app.hide().unwrap();
}

fn setup_callbacks(app: &App, config: Arc<Mutex<lr2_config::Config>>, launcher_config: Arc<Mutex<config::Config>>, openlr2_config: Arc<Mutex<openlr2_config::Config>>) {
    app.on_audio_type_change({
        let app_weak = app.as_weak();

        move || {
            let app = app_weak.unwrap();
            let app_globals = app.global::<ApplicationGlobal>();

            app_globals.set_selecteddriver(0);
            audio::reload_devices(&app_globals, app_globals.get_audiooutput());
        }
    });

    // TODO: language change callback
    app.on_launch_game({
        let app_weak = app.as_weak();
        let config_clone = config.clone();
        let launcher_config_clone = launcher_config.clone();
        let openlr2_config_clone = openlr2_config.clone();

        move || {
            let app = app_weak.unwrap();
            let app_globals = app.global::<ApplicationGlobal>();

            let username = match app_globals.get_players().row_data(usize::try_from(app_globals.get_selected_player()).unwrap()) {
                Some(player) => { player.to_string() },
                None => { panic!("Player doesn't exist anymore") }
            };
            let password = app_globals.get_password().to_string();
            let lr2_path: PathBuf = app_globals.get_lr2_path().to_string().clone().into();
            let lr2_folder_path = lr2_path.parent().unwrap();

            let valid = player::are_credentials_valid(&username, &password, lr2_folder_path).unwrap_or(false);
            if !valid {
                rfd::MessageDialog::new()
                    .set_buttons(rfd::MessageButtons::Ok)
                    .set_level(rfd::MessageLevel::Error)
                    .set_title(app_globals.get_window_title())
                    .set_description("Invalid password, ensure you typed it correctly!")
                    .show();
            } else {
                lr2::save_new_lr2_config(&app_globals, &config_clone);
                launcher::config::save_new_launcher_config(&app_globals, &launcher_config_clone);
                openlr2::save_new_openlr2_config(&app_globals, &openlr2_config_clone);
                process::launch_game(&lr2_path, app_globals.get_disable_score_save());

                app.hide().unwrap();
            }
        }
    });

    app.on_player_change({
        let app_weak = app.as_weak();
        let config_clone = config.clone();

        move |current_player| {
            let app = app_weak.unwrap();
            let app_globals = app.global::<ApplicationGlobal>();
            let config = config_clone.lock().unwrap();

            if current_player.to_string() == config.player.id {
                app_globals.set_password(config.player.pass.clone().into());
            } else {
                app_globals.set_password("".into());
            }
        }
    });

    app.on_delete_player({
        let app_weak = app.as_weak();
        let config_clone = config.clone();

        move |player| {
            let app = app_weak.unwrap();
            let app_globals = app.global::<ApplicationGlobal>();
            let config = config_clone.lock().unwrap();

            match rfd::MessageDialog::new()
                .set_buttons(rfd::MessageButtons::YesNo)
                .set_level(rfd::MessageLevel::Warning)
                .set_title(app_globals.get_window_title())
                .set_description(format!("Confirm deleting player {}? This operation cannot be undone and will delete all associated scores!", player))
                .show() {
                    rfd::MessageDialogResult::Yes => {
                        let lr2_path: PathBuf = app_globals.get_lr2_path().to_string().clone().into();
                        let lr2_folder_path = lr2_path.parent().unwrap();
                        player::delete_player(player.to_string(), &lr2_folder_path);

                        let players = lr2::parse_players(&lr2_folder_path.to_path_buf()).unwrap();
                        app_globals.set_players(ModelRc::from(Rc::new(VecModel::from(players.clone()))));
                        app_globals.set_selected_player(0);
                        app_globals.set_password("".into());
                        if !players.is_empty() {
                            let player_name = players[0].to_string();
                            if player_name.to_string() == config.player.id {
                                app_globals.set_password(config.player.pass.clone().into());
                            }
                        }
                    }
                    _ => {}
                };
        }
    });

    app.on_show_new_player_window({
        let app_weak = app.as_weak();

        move || {
            // Show second window with username / password input
            player::show_new_player_window(&app_weak.unwrap());
        }
    });

    app.on_jukebox_add({
        let app_weak = app.as_weak();

        move || {
            let app = app_weak.unwrap();
            let app_globals = app.global::<ApplicationGlobal>();

            let folder_to_add = match rfd::FileDialog::new()
                .set_title("Add jukebox folder...")
                .pick_folder() {
                    Some(path) => path,
                    None => return
                };
            let mut paths = utils::slint_arr_to_jukebox_paths(&app_globals);
            paths.push(folder_to_add.as_os_str().to_os_string().into_string().unwrap() + "\\"); // Doesn't actually matter but for consistency with the original launcher
            let paths_slint = utils::jukebox_paths_to_slint_arr(&Some(paths)).unwrap();
            app_globals.set_jukebox_paths(paths_slint);
        }
    });

    app.on_jukebox_del({
        let app_weak = app.as_weak();

        move |selected_path| {
            let app = app_weak.unwrap();
            let app_globals = app.global::<ApplicationGlobal>();

            let mut paths = utils::slint_arr_to_jukebox_paths(&app_globals);
            paths.remove(usize::try_from(selected_path).unwrap());
            let paths_slint = utils::jukebox_paths_to_slint_arr(&Some(paths)).unwrap();
            app_globals.set_jukebox_paths(paths_slint);
        }
    });

    app.on_set_lr2_path({
        let app_weak = app.as_weak();
        let config_clone = config.clone();
        let openlr2_config_clone = openlr2_config.clone();

        move || {
            let app = app_weak.unwrap();
            let app_globals = app.global::<ApplicationGlobal>();

            let new_lr2_path = match rfd::FileDialog::new()
                .add_filter("LR2 executable", &["exe"])
                .set_title("Pick LR2 executable...")
                .pick_file() {
                    Some(path) => path,
                    None => return
                };
            
            app_globals.set_lr2_path(new_lr2_path.clone().into_os_string().into_string().unwrap().into());
            let config_new = lr2::load_lr2_config(&app_globals, &new_lr2_path).unwrap();
            let openlr2_config_new = openlr2::load_openlr2_config(&app_globals, &new_lr2_path).unwrap();

            let mut config_ref = config_clone.lock().unwrap();
            *config_ref = config_new;

            let mut openlr2_config_ref = openlr2_config_clone.lock().unwrap();
            *openlr2_config_ref = openlr2_config_new;
        }
    });

    app.window().on_close_requested({
        let app_weak = app.as_weak();
        let config_clone = config.clone();
        let launcher_config_clone = launcher_config.clone();
        let openlr2_config_clone = openlr2_config.clone();

        move || {
            let app = app_weak.unwrap();
            let app_globals = app.global::<ApplicationGlobal>();

            lr2::save_new_lr2_config(&app_globals, &config_clone);
            openlr2::save_new_openlr2_config(&app_globals, &openlr2_config_clone);
            launcher::config::save_new_launcher_config(&app_globals, &launcher_config_clone);
            
            CloseRequestResponse::HideWindow
        }
    });
}
