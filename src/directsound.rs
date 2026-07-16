use std::ffi::c_void;
use slint::SharedString;
use anyhow::Result;
use windows::{Win32::Media::Audio::DirectSound::{DSDEVID_DefaultPlayback, DirectSoundEnumerateA}, core::{BOOL, GUID, PCSTR}};

unsafe extern "system" fn ds_enum_callback(guid: *mut GUID, lpcstr_description: PCSTR, _: PCSTR, user_ptr: *mut std::ffi::c_void) -> BOOL {
    // Don't add the meta "Default playback" device (LR2 doesn't show it)
    if !guid.is_null() && unsafe { *guid } != DSDEVID_DefaultPlayback {
        let ptr2 = user_ptr as *mut Vec<SharedString>;
        let devices = unsafe { &mut *ptr2 };
        devices.push(unsafe { lpcstr_description.to_string().unwrap().into() });
    };
    return BOOL(1);
}

pub fn get_devices() -> Result<Vec<SharedString>> {
    let mut devices: Vec<SharedString> = vec![];
    let myptr: *mut Vec<SharedString> = &mut devices;
    let voidptr = myptr as *mut c_void;

    unsafe {
        DirectSoundEnumerateA(Some(ds_enum_callback), Some(voidptr))?;
    };

    Ok(devices)
}
