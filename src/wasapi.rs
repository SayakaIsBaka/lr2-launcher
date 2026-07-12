use slint::SharedString;
use windows::Win32::{Devices::FunctionDiscovery::PKEY_Device_FriendlyName, Media::Audio::{DEVICE_STATE_ACTIVE, EDataFlow, ERole, IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator}, System::Com::{CLSCTX_ALL, CoCreateInstance, STGM_READ, StructuredStorage::PropVariantClear}};
use anyhow::Result;

pub struct WasapiDeviceEnumerator {
    enumerator: IMMDeviceEnumerator
}

impl WasapiDeviceEnumerator {
    pub fn new() -> Result<WasapiDeviceEnumerator> {
        let enumerator: IMMDeviceEnumerator =
            unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? };
        Ok(WasapiDeviceEnumerator { enumerator })
    }

    fn push_device_in_vec(device: &IMMDevice, res: &mut Vec<SharedString>) -> Result<()> {
        let store = unsafe { device.OpenPropertyStore(STGM_READ)? };
        let mut name = unsafe { store.GetValue(&PKEY_Device_FriendlyName)? };
        res.push(name.to_string().into());
        unsafe { PropVariantClear(&mut name) }?;
        Ok(())
    }

    pub fn get_devices(&self) -> Result<Vec<SharedString>> {
        let dir: EDataFlow = EDataFlow(0); // Render
        let role: ERole = ERole(0); // Console
        let devices_enum = unsafe {
            self.enumerator.EnumAudioEndpoints(dir, DEVICE_STATE_ACTIVE)?
        };
        let default_device = unsafe {
            self.enumerator.GetDefaultAudioEndpoint(dir, role)?
        };

        let mut devices: Vec<SharedString> = vec![];
        WasapiDeviceEnumerator::push_device_in_vec(&default_device, &mut devices)?; // Put default device at the beginning

        let device_count = unsafe { devices_enum.GetCount()? };
        for i in 0..device_count {
            let device = unsafe { devices_enum.Item(i)? };
            if unsafe { device.GetId()?.to_string()? == default_device.GetId()?.to_string()? } {
                continue;
            }
            WasapiDeviceEnumerator::push_device_in_vec(&device, &mut devices)?;
        };

        Ok(devices)
    }
}