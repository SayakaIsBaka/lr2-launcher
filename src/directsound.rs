use slint::SharedString;
use anyhow::Result;
use windows::{Win32::Media::Audio::DirectSound::{DSDEVID_DefaultPlayback, DirectSoundEnumerateA}, core::{BOOL, GUID, PCSTR}};

static DEVICES: std::sync::RwLock<Vec<SharedString>> = std::sync::RwLock::new(vec![]);

unsafe extern "system" fn ds_enum_callback(guid: *mut GUID, lpcstr_description: PCSTR, _: PCSTR, _: *mut std::ffi::c_void) -> BOOL {
    // Don't add the meta "Default playback" device (LR2 doesn't show it)
    if !guid.is_null() && unsafe { *guid } != DSDEVID_DefaultPlayback {
        let mut devices = DEVICES.write().unwrap();
        devices.push(unsafe { lpcstr_description.to_string().unwrap().into() });
    };
    return BOOL(1);
}

pub fn get_devices() -> Result<Vec<SharedString>> {
    DEVICES.write().unwrap().clear();

    unsafe {
        DirectSoundEnumerateA(Some(ds_enum_callback), None)?;
    };

    Ok(DEVICES.read().unwrap().to_vec())
}
