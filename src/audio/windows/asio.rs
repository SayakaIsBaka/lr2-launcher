use anyhow::Result;
use slint::SharedString;

pub fn get_devices() -> Result<Vec<SharedString>> {
    let mut devices: Vec<SharedString> = vec![];

    let asio_key = windows_registry::LOCAL_MACHINE.open("SOFTWARE\\ASIO")?;
    for key in asio_key.keys()? {
        devices.push(key.into());
    }

    Ok(devices)
}