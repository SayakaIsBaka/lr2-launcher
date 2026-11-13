use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel, language::StandardListViewItem};

use crate::{
    App, ApplicationGlobal,
    difficulty_table::{
        self,
        db::{self, PlaylistRow},
        parse::TableListItem,
    },
    launcher,
    lr2::{self, lr2_config},
    utils,
};

type InstalledList = Arc<Mutex<Vec<PlaylistRow>>>;
type TableList = Arc<Mutex<Vec<TableListItem>>>;

pub struct State {
    pub installed: InstalledList,
    pub tablelist: TableList,
    pub tablelist_filtered: TableList,
}

impl State {
    pub fn new() -> Self {
        Self {
            installed: Arc::new(Mutex::new(Vec::new())),
            tablelist: Arc::new(Mutex::new(Vec::new())),
            tablelist_filtered: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

fn selected_filter_tag(options: &ModelRc<SharedString>, index: i32) -> Option<String> {
    if index <= 0 {
        return None;
    }
    options.row_data(usize::try_from(index).ok()?).map(|s| s.to_string())
}

fn apply_tablelist_filter(app_globals: &ApplicationGlobal, full: &TableList, filtered: &TableList) {
    let full_items = full.lock().unwrap();
    let tag1 = selected_filter_tag(
        &app_globals.get_difficulty_tablelist_tag1_options(),
        app_globals.get_difficulty_tablelist_tag1_index(),
    );
    let tag2 = selected_filter_tag(
        &app_globals.get_difficulty_tablelist_tag2_options(),
        app_globals.get_difficulty_tablelist_tag2_index(),
    );
    let search = app_globals.get_difficulty_tablelist_search().to_string();
    let view = difficulty_table::parse::filter_tablelist_items(
        &full_items,
        tag1.as_deref(),
        tag2.as_deref(),
        &search,
    );
    let items: Vec<StandardListViewItem> = view
        .iter()
        .map(|r| {
            let tags = match (r.tag1.is_empty(), r.tag2.is_empty()) {
                (true, true) => String::new(),
                (false, true) => format!("[{}] ", r.tag1),
                (true, false) => format!("[{}] ", r.tag2),
                (false, false) => format!("[{}/{}] ", r.tag1, r.tag2),
            };
            StandardListViewItem::from(format!("{tags}{}", r.name).as_str())
        })
        .collect();
    app_globals.set_difficulty_tablelist_items(VecModel::from_slice(&items));
    app_globals.set_difficulty_tablelist_status(format!("{}/{}", view.len(), full_items.len()).into());
    *filtered.lock().unwrap() = view;
}

fn reload_installed_ui(app_globals: &ApplicationGlobal, lr2_folder_path: &std::path::Path, installed: &InstalledList) {
    match db::open_song_db(lr2_folder_path).and_then(|conn| db::list_playlists(&conn)) {
        Ok(rows) => {
            let items: Vec<StandardListViewItem> = rows
                .iter()
                .map(|r| StandardListViewItem::from(format!("{}  ({})", r.name, r.page_url).as_str()))
                .collect();
            app_globals.set_difficulty_tables(VecModel::from_slice(&items));
            *installed.lock().unwrap() = rows;
        }
        Err(_) => {
            app_globals.set_difficulty_tables(VecModel::from_slice(&[]));
            installed.lock().unwrap().clear();
        }
    }
}

fn lr2_folder_from_globals(app_globals: &ApplicationGlobal) -> PathBuf {
    let lr2_path: PathBuf = app_globals.get_lr2_path().to_string().into();
    lr2_path.parent().unwrap().to_path_buf()
}

fn show_error(app_globals: &ApplicationGlobal, msg: impl ToString) {
    rfd::MessageDialog::new()
        .set_buttons(rfd::MessageButtons::Ok)
        .set_level(rfd::MessageLevel::Error)
        .set_title(app_globals.get_window_title())
        .set_description(msg.to_string())
        .show();
}

// MessageBox inside invoke_from_event_loop can ding before the dialog paints.
fn show_error_later(app_weak: slint::Weak<App>, msg: impl ToString) {
    let msg = msg.to_string();
    slint::Timer::single_shot(std::time::Duration::from_millis(50), move || {
        let Some(app) = app_weak.upgrade() else {
            return;
        };
        show_error(&app.global::<ApplicationGlobal>(), msg);
    });
}

fn network_busy(app_globals: &ApplicationGlobal) -> bool {
    app_globals.get_difficulty_table_adding() || app_globals.get_difficulty_tablelist_loading()
}

fn ensure_output_in_jukebox(app_globals: &ApplicationGlobal, config: &Arc<Mutex<lr2_config::Config>>) {
    let output = app_globals.get_difficulty_table_output_dir().to_string();
    if output.trim().is_empty() {
        return;
    }
    let path = PathBuf::from(&output);
    let mut paths = utils::slint_arr_to_jukebox_paths(app_globals);
    if difficulty_table::ensure_jukebox_has_path(&mut paths, &path) {
        let paths_slint = utils::jukebox_paths_to_slint_arr(&Some(paths)).unwrap();
        app_globals.set_jukebox_paths(paths_slint);
        lr2::save_new_lr2_config(app_globals, config);
    }
}

fn require_output(app_globals: &ApplicationGlobal) -> Option<PathBuf> {
    let path = app_globals.get_difficulty_table_output_dir().to_string();
    if path.trim().is_empty() {
        show_error(app_globals, "Set an output folder first");
        return None;
    }
    Some(PathBuf::from(path))
}

enum TableJobKind {
    AddOrRefresh { url: String, save_launcher: bool, ok_status: String },
    RefreshAll,
}

fn spawn_table_job(
    app_weak: slint::Weak<App>,
    lr2_folder_path: PathBuf,
    output: PathBuf,
    installed: InstalledList,
    launcher_config: Option<Arc<Mutex<launcher::config::Config>>>,
    kind: TableJobKind,
) {
    std::thread::spawn(move || {
        let result = match &kind {
            TableJobKind::AddOrRefresh { url, .. } => {
                difficulty_table::add_or_refresh_table(&lr2_folder_path, &output, url)
                    .map(|row| (Some(row), Vec::new()))
            }
            TableJobKind::RefreshAll => {
                difficulty_table::refresh_all(&lr2_folder_path, &output).map(|errors| (None, errors))
            }
        };

        let _ = slint::invoke_from_event_loop(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let app_globals = app.global::<ApplicationGlobal>();
            app_globals.set_difficulty_table_adding(false);

            match result {
                Ok((row, errors)) => {
                    reload_installed_ui(&app_globals, &lr2_folder_path, &installed);
                    match kind {
                        TableJobKind::AddOrRefresh { save_launcher, ok_status, .. } => {
                            if let Some(row) = row {
                                app_globals.set_difficulty_table_status(
                                    format!("{ok_status}{}", row.name).into(),
                                );
                            } else {
                                app_globals.set_difficulty_table_status(ok_status.into());
                            }
                            if save_launcher {
                                if let Some(cfg) = launcher_config {
                                    launcher::config::save_new_launcher_config(&app_globals, &cfg);
                                }
                            }
                        }
                        TableJobKind::RefreshAll => {
                            if errors.is_empty() {
                                app_globals.set_difficulty_table_status("All tables refreshed".into());
                            } else {
                                app_globals.set_difficulty_table_status(
                                    format!("Some tables failed ({})", errors.len()).into(),
                                );
                                show_error_later(
                                    app_weak.clone(),
                                    format!("Some tables failed:\n{}", errors.join("\n")),
                                );
                            }
                        }
                    }
                }
                Err(e) => show_error_later(app_weak.clone(), e.to_string()),
            }
        });
    });
}

fn start_add_or_refresh(
    app_globals: &ApplicationGlobal,
    app_weak: slint::Weak<App>,
    config: &Arc<Mutex<lr2_config::Config>>,
    launcher_config: Option<Arc<Mutex<launcher::config::Config>>>,
    installed: InstalledList,
    url: String,
    ok_prefix: &str,
    save_launcher: bool,
) {
    if network_busy(app_globals) {
        return;
    }
    let Some(output) = require_output(app_globals) else {
        return;
    };
    ensure_output_in_jukebox(app_globals, config);
    let lr2_folder_path = lr2_folder_from_globals(app_globals);
    app_globals.set_difficulty_table_status("".into());
    app_globals.set_difficulty_table_adding(true);
    spawn_table_job(
        app_weak,
        lr2_folder_path,
        output,
        installed,
        launcher_config,
        TableJobKind::AddOrRefresh {
            url,
            save_launcher,
            ok_status: ok_prefix.to_string(),
        },
    );
}

pub fn load_installed(app: &App, state: &State) {
    let app_globals = app.global::<ApplicationGlobal>();
    let lr2_path: PathBuf = app_globals.get_lr2_path().to_string().into();
    if let Some(lr2_folder_path) = lr2_path.parent() {
        reload_installed_ui(&app_globals, lr2_folder_path, &state.installed);
    }
}

pub fn setup_callbacks(
    app: &App,
    config: Arc<Mutex<lr2_config::Config>>,
    launcher_config: Arc<Mutex<launcher::config::Config>>,
    state: &State,
) {
    let installed = state.installed.clone();
    let tablelist = state.tablelist.clone();
    let tablelist_filtered = state.tablelist_filtered.clone();

    app.on_difficulty_table_pick_output_dir({
        let app_weak = app.as_weak();
        let config_clone = config.clone();
        let launcher_config_clone = launcher_config.clone();

        move || {
            let app = app_weak.unwrap();
            let app_globals = app.global::<ApplicationGlobal>();
            let folder = match rfd::FileDialog::new()
                .set_title("Pick difficulty table output folder...")
                .pick_folder()
            {
                Some(p) => p,
                None => return,
            };
            let normalized = difficulty_table::normalize_jukebox_path(&folder);
            app_globals.set_difficulty_table_output_dir(normalized.into());
            ensure_output_in_jukebox(&app_globals, &config_clone);
            launcher::config::save_new_launcher_config(&app_globals, &launcher_config_clone);
        }
    });

    app.on_difficulty_table_reload_installed({
        let app_weak = app.as_weak();
        let installed = installed.clone();
        move || {
            let app = app_weak.unwrap();
            let app_globals = app.global::<ApplicationGlobal>();
            let lr2_folder_path = lr2_folder_from_globals(&app_globals);
            reload_installed_ui(&app_globals, &lr2_folder_path, &installed);
        }
    });

    app.on_difficulty_table_add_by_url({
        let app_weak = app.as_weak();
        let config_clone = config.clone();
        let launcher_config_clone = launcher_config.clone();
        let installed = installed.clone();

        move |url| {
            let app = app_weak.unwrap();
            let app_globals = app.global::<ApplicationGlobal>();
            let url = url.trim().to_string();
            if url.is_empty() {
                show_error(&app_globals, "Enter a table page URL");
                return;
            }
            start_add_or_refresh(
                &app_globals,
                app_weak.clone(),
                &config_clone,
                Some(launcher_config_clone.clone()),
                installed.clone(),
                url,
                "Added/updated: ",
                true,
            );
        }
    });

    app.on_difficulty_table_load_tablelist({
        let app_weak = app.as_weak();
        let tablelist = tablelist.clone();
        let filtered = tablelist_filtered.clone();
        let launcher_config_clone = launcher_config.clone();

        move || {
            let app = app_weak.unwrap();
            let app_globals = app.global::<ApplicationGlobal>();
            if network_busy(&app_globals) {
                return;
            }
            let url = app_globals.get_difficulty_tablelist_url().to_string();
            app_globals.set_difficulty_tablelist_loading(true);

            let app_weak = app_weak.clone();
            let tablelist = tablelist.clone();
            let filtered = filtered.clone();
            let launcher_config_clone = launcher_config_clone.clone();
            std::thread::spawn(move || {
                let result = difficulty_table::load_tablelist(&url);
                let _ = slint::invoke_from_event_loop(move || {
                    let Some(app) = app_weak.upgrade() else {
                        return;
                    };
                    let app_globals = app.global::<ApplicationGlobal>();
                    app_globals.set_difficulty_tablelist_loading(false);
                    match result {
                        Ok(items) => {
                            let mut tag1 = vec!["(All)".to_string()];
                            tag1.extend(difficulty_table::parse::unique_sorted_tags(&items, 1));
                            let mut tag2 = vec!["(All)".to_string()];
                            tag2.extend(difficulty_table::parse::unique_sorted_tags(&items, 2));
                            let tag1_model: Vec<SharedString> =
                                tag1.iter().map(|s| SharedString::from(s.as_str())).collect();
                            let tag2_model: Vec<SharedString> =
                                tag2.iter().map(|s| SharedString::from(s.as_str())).collect();
                            app_globals.set_difficulty_tablelist_tag1_options(ModelRc::new(VecModel::from(tag1_model)));
                            app_globals.set_difficulty_tablelist_tag2_options(ModelRc::new(VecModel::from(tag2_model)));
                            app_globals.set_difficulty_tablelist_tag1_index(0);
                            app_globals.set_difficulty_tablelist_tag2_index(0);
                            app_globals.set_difficulty_tablelist_search("".into());
                            *tablelist.lock().unwrap() = items;
                            apply_tablelist_filter(&app_globals, &tablelist, &filtered);
                            launcher::config::save_new_launcher_config(&app_globals, &launcher_config_clone);
                        }
                        Err(e) => show_error_later(app_weak.clone(), e.to_string()),
                    }
                });
            });
        }
    });

    app.on_difficulty_table_filter_tablelist({
        let app_weak = app.as_weak();
        let tablelist = tablelist.clone();
        let filtered = tablelist_filtered.clone();

        move || {
            let app = app_weak.unwrap();
            let app_globals = app.global::<ApplicationGlobal>();
            apply_tablelist_filter(&app_globals, &tablelist, &filtered);
        }
    });

    app.on_difficulty_table_add_tablelist_selected({
        let app_weak = app.as_weak();
        let config_clone = config.clone();
        let launcher_config_clone = launcher_config.clone();
        let installed = installed.clone();
        let filtered = tablelist_filtered.clone();

        move |selected| {
            let app = app_weak.unwrap();
            let app_globals = app.global::<ApplicationGlobal>();
            let items = filtered.lock().unwrap();
            let idx = usize::try_from(selected).unwrap_or(usize::MAX);
            let Some(item) = items.get(idx) else {
                show_error(&app_globals, "Invalid selection");
                return;
            };
            let url = item.url.clone();
            drop(items);

            start_add_or_refresh(
                &app_globals,
                app_weak.clone(),
                &config_clone,
                Some(launcher_config_clone.clone()),
                installed.clone(),
                url,
                "Added/updated: ",
                true,
            );
        }
    });

    app.on_difficulty_table_refresh_selected({
        let app_weak = app.as_weak();
        let config_clone = config.clone();
        let installed = installed.clone();

        move |selected| {
            let app = app_weak.unwrap();
            let app_globals = app.global::<ApplicationGlobal>();
            let rows = installed.lock().unwrap();
            let idx = usize::try_from(selected).unwrap_or(usize::MAX);
            let Some(row) = rows.get(idx) else {
                show_error(&app_globals, "Invalid selection");
                return;
            };
            let page_url = row.page_url.clone();
            drop(rows);

            start_add_or_refresh(
                &app_globals,
                app_weak.clone(),
                &config_clone,
                None,
                installed.clone(),
                page_url,
                "Refreshed: ",
                false,
            );
        }
    });

    app.on_difficulty_table_refresh_all({
        let app_weak = app.as_weak();
        let config_clone = config.clone();
        let installed = installed.clone();

        move || {
            let app = app_weak.unwrap();
            let app_globals = app.global::<ApplicationGlobal>();
            if network_busy(&app_globals) {
                return;
            }
            if installed.lock().unwrap().is_empty() {
                return;
            }
            let Some(output) = require_output(&app_globals) else {
                return;
            };
            ensure_output_in_jukebox(&app_globals, &config_clone);
            let lr2_folder_path = lr2_folder_from_globals(&app_globals);
            app_globals.set_difficulty_table_status("".into());
            app_globals.set_difficulty_table_adding(true);
            spawn_table_job(
                app_weak.clone(),
                lr2_folder_path,
                output,
                installed.clone(),
                None,
                TableJobKind::RefreshAll,
            );
        }
    });

    app.on_difficulty_table_remove_selected({
        let app_weak = app.as_weak();
        let installed = installed.clone();

        move |selected| {
            let app = app_weak.unwrap();
            let app_globals = app.global::<ApplicationGlobal>();
            let rows = installed.lock().unwrap();
            let idx = usize::try_from(selected).unwrap_or(usize::MAX);
            let Some(row) = rows.get(idx) else {
                show_error(&app_globals, "Invalid selection");
                return;
            };
            let id = row.playlist_id;
            let name = row.name.clone();
            drop(rows);

            match rfd::MessageDialog::new()
                .set_buttons(rfd::MessageButtons::YesNo)
                .set_level(rfd::MessageLevel::Warning)
                .set_title(app_globals.get_window_title())
                .set_description(format!("Remove difficulty table \"{name}\"?")).show()
            {
                rfd::MessageDialogResult::Yes => {
                    let lr2_folder_path = lr2_folder_from_globals(&app_globals);
                    match difficulty_table::remove_table(&lr2_folder_path, id) {
                        Ok(()) => reload_installed_ui(&app_globals, &lr2_folder_path, &installed),
                        Err(e) => show_error(&app_globals, e),
                    }
                }
                _ => {}
            }
        }
    });
}
