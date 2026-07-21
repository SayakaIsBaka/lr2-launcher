use std::{ffi::{CStr, c_char}, path::{Path, PathBuf}, rc::Rc, sync::Mutex};
use anyhow::{Result, bail};
use libloading::{Library, Symbol};
use slint::{ModelRc, SharedString, VecModel};
use walkdir::WalkDir;
use crate::{ApplicationGlobal, GameType};

pub mod openlr2_config;
pub mod update;

#[repr(C)]
struct MethodTable {
    get_name: Option<unsafe extern "C" fn() -> *const c_char>,
    dummy: [usize; 64] // Hopefully this is enough to not have out of bounds writes
}

fn get_customir_name(module_path: &Path) -> Result<String> {
    let lib = unsafe { Library::new(module_path)? };
    let get_method_table: Symbol<unsafe extern "C" fn(*mut MethodTable)> = unsafe { lib.get(b"GetMethodTable")? };

    let mut method_table = MethodTable { get_name: None, dummy: [0; 64] };
    let method_table_ptr: *mut MethodTable = &mut method_table;
    unsafe { get_method_table(method_table_ptr) };
    
    match method_table.get_name {
        Some(get_name) => {
            let name = unsafe { get_name() };
            let name_cstr = unsafe { CStr::from_ptr(name) };
            Ok(name_cstr.to_string_lossy().to_string())
        }
        None => bail!("Error calling GetMethodTable")
    }
}

fn list_available_customirs(lr2_folder_path: &PathBuf) -> Result<Vec<SharedString>> {
    let mut customirs: Vec<SharedString> = vec![];
    let customirs_folder = lr2_folder_path.join("LR2files\\CustomIRs");

    #[cfg(target_os = "windows")]
    static EXTENSION: &'static str = ".dll";

    #[cfg(target_os = "linux")]
    static EXTENSION: &'static str = ".so";

    #[cfg(target_os = "macos")]
    static EXTENSION: &'static str = ".dylib";

    for entry in WalkDir::new(customirs_folder)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok()) {
        let f_name = entry.file_name().to_string_lossy();

        if f_name.ends_with(EXTENSION) {
            match get_customir_name(entry.path()) {
                Ok(res) => customirs.push(res.into()),
                Err(_) => {} // Skip library and continue if an error occured during loading (arch mismatch, GetMethodTable not found, etc.)
            };
        }
    }

    Ok(customirs)
}

pub fn load_openlr2_config(app_globals: &ApplicationGlobal, lr2_path: &PathBuf) -> Result<openlr2_config::Config> {
    let mut lr2_folder_path = lr2_path.clone();
    lr2_folder_path.pop();

    let config = openlr2_config::Config::load(&lr2_folder_path).unwrap_or_default();
    let customirs = list_available_customirs(&lr2_folder_path).unwrap_or_default();
    
    if app_globals.get_gametype() == GameType::OpenLR2 { // Only load here if binary is OpenLR2
        app_globals.set_screenmode(config.system.screenmode.into());
    }
    app_globals.set_resolution(config.system.resolution.into());
    app_globals.set_fullscreenfilter(config.system.fullscreenfilter.into());
    app_globals.set_fullscreenfitstretch(config.system.fullscreenfitstretch);
    app_globals.set_gaugeautoshift(config.play.gaugeautoshift);
    app_globals.set_customirs(ModelRc::from(Rc::new(VecModel::from(customirs.clone()))));
    match customirs.iter().position(|x| x.as_str() == config.network.display_ir) {
        Some(idx) => app_globals.set_selectedir_index(i32::try_from(idx).unwrap()),
        None => {}
    };
    app_globals.set_display_ir(config.network.display_ir.clone().into());

    Ok(config)
}

pub fn save_new_openlr2_config(app_globals: &ApplicationGlobal, config: &Mutex<openlr2_config::Config>) {
    let mut config_new = config.lock().unwrap();

    config_new.system.screenmode = u8::try_from(app_globals.get_screenmode()).unwrap(); 
    config_new.system.resolution = u8::try_from(app_globals.get_resolution()).unwrap();
    config_new.system.fullscreenfilter = u8::try_from(app_globals.get_fullscreenfilter()).unwrap();
    config_new.system.fullscreenfitstretch = app_globals.get_fullscreenfitstretch();
    config_new.play.gaugeautoshift = app_globals.get_gaugeautoshift();
    config_new.network.display_ir = app_globals.get_display_ir().to_string();

    let lr2_path: PathBuf = app_globals.get_lr2_path().to_string().into();
    let lr2_folder_path = lr2_path.parent().unwrap();
    config_new.write(lr2_folder_path).unwrap();
}
