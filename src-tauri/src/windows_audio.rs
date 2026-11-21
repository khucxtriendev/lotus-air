use serde::Serialize;

#[derive(Serialize)]
pub struct AudioOutput {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

#[derive(Serialize)]
pub struct CurrentMedia {
    pub title: String,
    pub artist: String,
    pub playing: bool,
}

#[tauri::command]
pub async fn get_current_media() -> Result<Option<CurrentMedia>, String> {
    #[cfg(windows)]
    {
        tokio::task::spawn_blocking(|| {
            use windows::Media::Control::{GlobalSystemMediaTransportControlsSessionManager, GlobalSystemMediaTransportControlsSessionPlaybackStatus};
            use windows::Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize};
            unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.map_err(|error| error.to_string())?;
            let result = (|| {
            let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
                .and_then(|operation| operation.get())
                .map_err(|error| error.to_string())?;
            let session = match manager.GetCurrentSession() {
                Ok(session) => session,
                Err(_) => return Ok(None),
            };
            let properties = session.TryGetMediaPropertiesAsync()
                .and_then(|operation| operation.get())
                .map_err(|error| error.to_string())?;
            let info = session.GetPlaybackInfo().map_err(|error| error.to_string())?;
            Ok(Some(CurrentMedia {
                title: properties.Title().map_err(|error| error.to_string())?.to_string(),
                artist: properties.Artist().map_err(|error| error.to_string())?.to_string(),
                playing: info.PlaybackStatus().map_err(|error| error.to_string())?
                    == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing,
            }))
            })();
            unsafe { RoUninitialize() };
            result
        }).await.map_err(|error| error.to_string())?
    }
    #[cfg(not(windows))]
    Ok(None)
}

#[cfg(windows)]
fn owned_wide_string(value: windows::core::PWSTR) -> windows::core::Result<String> {
    use windows::Win32::System::Com::CoTaskMemFree;
    let result = unsafe { value.to_string() };
    unsafe { CoTaskMemFree(Some(value.0 as *const _)) };
    Ok(result?)
}

/// Gửi lệnh KSPROPERTY_ONESHOT_RECONNECT (cướp kết nối A2DP từ iPhone/iPad về PC)
/// hoặc KSPROPERTY_ONESHOT_DISCONNECT trực tiếp tới driver BthA2dp.sys của Windows.
#[cfg(windows)]
pub fn trigger_airpods_a2dp_control(reconnect: bool) -> Result<String, String> {
    use windows::core::{GUID, Interface};
    use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
    use windows::Win32::Media::Audio::{
        DEVICE_STATE, DEVICE_STATE_ACTIVE, DEVICE_STATE_UNPLUGGED, IConnector, IDeviceTopology,
        IMMDeviceEnumerator, IPart, MMDeviceEnumerator, eRender,
    };
    use windows::Win32::Media::KernelStreaming::{IKsControl, KSIDENTIFIER, KSIDENTIFIER_0, KSIDENTIFIER_0_0};
    use windows::Win32::System::Com::StructuredStorage::PropVariantToStringAlloc;
    use windows::Win32::System::Com::{
        CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
        STGM_READ,
    };

    // KSPROPSETID_BtAudio = {7FA06C40-B8F6-4C7E-8556-E8C33A12E54D}
    const KSPROPSETID_BT_AUDIO: GUID = GUID::from_u128(0x7FA06C40_B8F6_4C7E_8556_E8C33A12E54D);
    const KSPROPERTY_ONESHOT_RECONNECT: u32 = 0;
    const KSPROPERTY_ONESHOT_DISCONNECT: u32 = 1;
    const KSPROPERTY_TYPE_GET: u32 = 0x00000001;

    let known_name = lotusair_driver::find_connected_airpods().map(|d| d.name.to_lowercase());

    let initialized = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok();
    let result = (|| -> Result<String, String> {
        let enumerator: IMMDeviceEnumerator =
            unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }
                .map_err(|e| format!("MMDeviceEnumerator lỗi: {e}"))?;
        let devices = unsafe {
            enumerator.EnumAudioEndpoints(
                eRender,
                DEVICE_STATE(DEVICE_STATE_ACTIVE.0 | DEVICE_STATE_UNPLUGGED.0),
            )
        }
        .map_err(|e| format!("EnumAudioEndpoints lỗi: {e}"))?;

        let count = unsafe { devices.GetCount() }.map_err(|e| e.to_string())?;
        let mut target_device = None;
        let mut target_name = String::new();

        for i in 0..count {
            let dev = unsafe { devices.Item(i) }.map_err(|e| e.to_string())?;
            let store = unsafe { dev.OpenPropertyStore(STGM_READ) }.map_err(|e| e.to_string())?;
            let val = unsafe { store.GetValue(&PKEY_Device_FriendlyName) }
                .map_err(|e| e.to_string())?;
            let name = owned_wide_string(
                unsafe { PropVariantToStringAlloc(&val) }.map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            let lower = name.to_lowercase();

            let matches_airpods = lower.contains("airpod")
                || known_name
                    .as_ref()
                    .map(|kn| !kn.is_empty() && lower.contains(kn.as_str()))
                    .unwrap_or(false);

            if matches_airpods {
                let is_stereo = lower.contains("headphones");
                if is_stereo || target_device.is_none() {
                    target_device = Some(dev);
                    target_name = name;
                    if is_stereo {
                        break;
                    }
                }
            }
        }

        let Some(dev) = target_device else {
            return Err("Không tìm thấy thiết bị âm thanh AirPods đã ghép đôi trên Windows".into());
        };

        let topo: IDeviceTopology = unsafe { dev.Activate(CLSCTX_ALL, None) }
            .map_err(|e| format!("Activate IDeviceTopology lỗi: {e}"))?;
        let conn: IConnector = unsafe { topo.GetConnector(0) }
            .map_err(|e| format!("GetConnector(0) lỗi: {e}"))?;
        let peer_conn: IConnector = unsafe { conn.GetConnectedTo() }
            .map_err(|e| format!("GetConnectedTo lỗi: {e}"))?;
        let peer_part: IPart = peer_conn
            .cast()
            .map_err(|e| format!("Cast IPart lỗi: {e}"))?;
        let peer_topo: IDeviceTopology = unsafe { peer_part.GetTopologyObject() }
            .map_err(|e| format!("GetTopologyObject lỗi: {e}"))?;
        let adapter_id = owned_wide_string(
            unsafe { peer_topo.GetDeviceId() }.map_err(|e| format!("GetDeviceId lỗi: {e}"))?,
        )
        .map_err(|e| e.to_string())?;
        let adapter_id_w: Vec<u16> = adapter_id.encode_utf16().chain(std::iter::once(0)).collect();
        let adapter_dev = unsafe {
            enumerator.GetDevice(windows::core::PCWSTR(adapter_id_w.as_ptr()))
        }
        .map_err(|e| format!("GetDevice(adapter) lỗi: {e}"))?;
        let ks_ctrl: IKsControl = unsafe { adapter_dev.Activate(CLSCTX_ALL, None) }
            .map_err(|e| format!("Activate IKsControl lỗi: {e}"))?;

        let mut prop = KSIDENTIFIER {
            Anonymous: KSIDENTIFIER_0 {
                Anonymous: KSIDENTIFIER_0_0 {
                    Set: KSPROPSETID_BT_AUDIO,
                    Id: if reconnect {
                        KSPROPERTY_ONESHOT_RECONNECT
                    } else {
                        KSPROPERTY_ONESHOT_DISCONNECT
                    },
                    Flags: KSPROPERTY_TYPE_GET,
                },
            },
        };
        let mut bytes_returned = 0u32;
        unsafe {
            ks_ctrl.KsProperty(
                &mut prop,
                std::mem::size_of::<KSIDENTIFIER>() as u32,
                std::ptr::null_mut(),
                0,
                &mut bytes_returned,
            )
        }
        .map_err(|e| format!("BthA2dp KSPROPERTY_ONESHOT lỗi: {e}"))?;

        Ok(target_name)
    })();

    if initialized {
        unsafe { CoUninitialize() };
    }
    result
}

#[cfg(not(windows))]
pub fn trigger_airpods_a2dp_control(_reconnect: bool) -> Result<String, String> {
    Err("Chỉ hỗ trợ Windows".into())
}

#[tauri::command]
pub fn get_audio_outputs() -> Result<Vec<AudioOutput>, String> {
    #[cfg(windows)]
    {
        use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
        use windows::Win32::Media::Audio::{DEVICE_STATE_ACTIVE, IMMDeviceEnumerator, MMDeviceEnumerator, eMultimedia, eRender};
        use windows::Win32::System::Com::StructuredStorage::PropVariantToStringAlloc;
        use windows::Win32::System::Com::{CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize, STGM_READ};

        let initialized = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok();
        let result = (|| -> windows::core::Result<Vec<AudioOutput>> {
            let enumerator: IMMDeviceEnumerator = unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? };
            let default = unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia)? };
            let default_id = owned_wide_string(unsafe { default.GetId()? })?;
            let devices = unsafe { enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)? };
            let mut outputs = Vec::new();
            for index in 0..unsafe { devices.GetCount()? } {
                let device = unsafe { devices.Item(index)? };
                let id = owned_wide_string(unsafe { device.GetId()? })?;
                let store = unsafe { device.OpenPropertyStore(STGM_READ)? };
                let value = unsafe { store.GetValue(&PKEY_Device_FriendlyName)? };
                let name = owned_wide_string(unsafe { PropVariantToStringAlloc(&value)? })?;
                outputs.push(AudioOutput { is_default: id == default_id, id, name });
            }
            Ok(outputs)
        })();
        if initialized { unsafe { CoUninitialize() }; }
        result.map_err(|error| format!("Windows Audio: {error}"))
    }
    #[cfg(not(windows))]
    Ok(Vec::new())
}

#[tauri::command]
pub async fn set_default_audio_output(id: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        if !get_audio_outputs()?.iter().any(|output| output.id == id) {
            return Err("Thiết bị đầu ra không còn hoạt động".into());
        }
        tokio::task::spawn_blocking(move || {
            use com_policy_config::{IPolicyConfig, PolicyConfigClient};
            use windows061::core::PCWSTR;
            use windows061::Win32::Media::Audio::{eCommunications, eConsole, eMultimedia};
            use windows061::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED};
            unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.ok().map_err(|error| error.to_string())?;
            let result = (|| -> windows061::core::Result<()> {
                let policy: IPolicyConfig = unsafe { CoCreateInstance(&PolicyConfigClient, None, CLSCTX_ALL)? };
                let wide: Vec<u16> = id.encode_utf16().chain(std::iter::once(0)).collect();
                let endpoint = PCWSTR(wide.as_ptr());
                unsafe {
                    policy.SetDefaultEndpoint(endpoint, eConsole)?;
                    policy.SetDefaultEndpoint(endpoint, eMultimedia)?;
                    policy.SetDefaultEndpoint(endpoint, eCommunications)?;
                }
                Ok(())
            })();
            unsafe { CoUninitialize() };
            result.map_err(|error| format!("Windows không đổi được đầu ra âm thanh: {error}"))
        }).await.map_err(|error| error.to_string())?
    }
    #[cfg(not(windows))]
    Err("Chỉ hỗ trợ Windows".into())
}

#[cfg(windows)]
fn with_endpoint<T>(operation: impl FnOnce(&windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume) -> windows::core::Result<T>) -> Result<T, String> {
    use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
    use windows::Win32::Media::Audio::{IMMDeviceEnumerator, MMDeviceEnumerator, eMultimedia, eRender};
    use windows::Win32::System::Com::{CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize};

    let initialized = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok();
    let result = (|| -> windows::core::Result<T> {
        let enumerator: IMMDeviceEnumerator = unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? };
        let device = unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia)? };
        let volume: IAudioEndpointVolume = unsafe { device.Activate(CLSCTX_ALL, None)? };
        operation(&volume)
    })();
    if initialized {
        unsafe { CoUninitialize() };
    }
    result.map_err(|error| format!("Windows Audio: {error}"))
}

#[tauri::command]
pub fn get_windows_volume() -> Result<u8, String> {
    #[cfg(windows)]
    {
        with_endpoint(|volume| {
            let value = unsafe { volume.GetMasterVolumeLevelScalar()? };
            Ok((value * 100.0).round().clamp(0.0, 100.0) as u8)
        })
    }
    #[cfg(not(windows))]
    Err("Chỉ hỗ trợ Windows".into())
}

#[tauri::command]
pub fn set_windows_volume(level: u8) -> Result<(), String> {
    if level > 100 {
        return Err("Âm lượng phải từ 0 đến 100".into());
    }
    #[cfg(windows)]
    {
        with_endpoint(|volume| unsafe {
            volume.SetMasterVolumeLevelScalar(level as f32 / 100.0, std::ptr::null())
        })
    }
    #[cfg(not(windows))]
    Err("Chỉ hỗ trợ Windows".into())
}

#[tauri::command]
pub async fn media_control(action: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        tokio::task::spawn_blocking(move || {
            use windows::Media::Control::{GlobalSystemMediaTransportControlsSessionManager, GlobalSystemMediaTransportControlsSessionPlaybackStatus};
            use windows::Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize};
            unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.map_err(|error| error.to_string())?;
            let result = (|| -> Result<(), String> {
                let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
                    .and_then(|operation| operation.get())
                    .map_err(|error| error.to_string())?;
                let session = manager.GetCurrentSession().map_err(|_| "Không có phiên phát nhạc đang hoạt động".to_string())?;
                let operation = match action.as_str() {
                    "previous" => session.TrySkipPreviousAsync(),
                    "toggle" => {
                        let info = session.GetPlaybackInfo().map_err(|error| error.to_string())?;
                        if info.PlaybackStatus().map_err(|error| error.to_string())?
                            == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing {
                            session.TryPauseAsync()
                        } else {
                            session.TryPlayAsync()
                        }
                    },
                    "next" => session.TrySkipNextAsync(),
                    _ => return Err("Lệnh phát nhạc không hợp lệ".into()),
                }.map_err(|error| error.to_string())?;
                if operation.get().map_err(|error| error.to_string())? {
                    Ok(())
                } else {
                    Err("Ứng dụng phát nhạc không chấp nhận lệnh".into())
                }
            })();
            unsafe { RoUninitialize() };
            result
        }).await.map_err(|error| error.to_string())?
    }
    #[cfg(not(windows))]
    Err("Chỉ hỗ trợ Windows".into())
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[ignore = "Temporarily switches the Windows default playback output"]
    async fn switch_audio_output_round_trip() {
        let outputs = super::get_audio_outputs().expect("list outputs");
        let original = outputs.iter().find(|output| output.is_default).expect("default output");
        let other = outputs.iter().find(|output| !output.is_default).expect("second output");
        super::set_default_audio_output(other.id.clone()).await.expect("switch output");
        let switched = super::get_audio_outputs().expect("list switched outputs");
        let worked = switched.iter().any(|output| output.id == other.id && output.is_default);
        super::set_default_audio_output(original.id.clone()).await.expect("restore output");
        assert!(worked, "Windows did not select the requested output");
        let restored = super::get_audio_outputs().expect("list restored outputs");
        assert!(restored.iter().any(|output| output.id == original.id && output.is_default));
    }
    #[tokio::test]
    async fn reads_current_media_session() {
        println!("Current media: {:?}", super::get_current_media().await.map(|media| media.map(|item| (item.title, item.artist, item.playing))));
    }

    #[tokio::test]
    #[ignore = "Changes the active Windows media session during a hardware smoke test"]
    async fn media_toggle_round_trip() {
        let before = super::get_current_media().await.expect("read media");
        if let Some(before) = before {
            super::media_control("toggle".into()).await.expect("toggle playback");
            tokio::time::sleep(std::time::Duration::from_millis(700)).await;
            super::media_control("toggle".into()).await.expect("restore playback");
            let mut restored = false;
            for _ in 0..12 {
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                let after = super::get_current_media().await.expect("read media after toggles").expect("media session still available");
                if before.playing == after.playing {
                    restored = true;
                    break;
                }
            }
            assert!(restored, "playback did not return to its original state");
        }
    }
    #[test]
    fn lists_current_audio_outputs() {
        let outputs = super::get_audio_outputs().expect("list Windows outputs");
        for output in outputs {
            println!("{}: {}", if output.is_default { "default" } else { "output" }, output.name);
        }
    }
    #[test]
    fn test_airpods_a2dp_reconnect_property() {
        let res = super::trigger_airpods_a2dp_control(true);
        println!("A2DP KSPROPERTY_ONESHOT_RECONNECT result: {:?}", res);
        assert!(res.is_ok(), "A2DP reconnect property failed: {:?}", res);
    }
}
