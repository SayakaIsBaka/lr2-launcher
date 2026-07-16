use std::rc::Rc;
use slint::{ModelRc, SharedString, VecModel};
use anyhow::{Result, bail};
use crate::ApplicationGlobal;

#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub fn get_audio_devices(device_type: i32) -> Result<ModelRc<SharedString>> {
    let mut device_list_vec: Vec<SharedString> = vec![];
    match device_type {
        0 => { // DirectSound
            device_list_vec = windows::directsound::get_devices()?;
        },
        1 => { // WASAPI
            let wasapi_enumerator = windows::wasapi::WasapiDeviceEnumerator::new()?;
            device_list_vec = wasapi_enumerator.get_devices()?;
        },
        2 => { // ASIO
            device_list_vec = windows::asio::get_devices()?;
        },
        _ => {
            bail!("Invalid device type");
        }
    }
    let device_list = Rc::new(VecModel::from(device_list_vec));
    Ok(ModelRc::from(device_list))
}

pub fn reload_devices(app_globals: &ApplicationGlobal, device_type: i32) {
    app_globals.set_drivers(get_audio_devices(device_type).unwrap_or_default());
}