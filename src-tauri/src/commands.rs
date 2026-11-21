use lotusair_bluetooth::BluetoothManager;
use lotusair_core::models::{AirPodsDeviceState, NoiseControlMode, SpatialAudioMode};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::window::{Color, Effect, EffectState, EffectsBuilder};
use tauri::{AppHandle, Manager, State, Theme, WebviewUrl, WebviewWindowBuilder};
use tracing::info;

static TAKEOVER_IN_PROGRESS: AtomicBool = AtomicBool::new(false);
static LAST_TAKEOVER_DONE_MS: AtomicU64 = AtomicU64::new(0);
static POPUP_GENERATION: AtomicU64 = AtomicU64::new(0);
static FLYOUT_GENERATION: AtomicU64 = AtomicU64::new(0);
static LAST_PERSISTED_SNAPSHOT: OnceLock<Mutex<String>> = OnceLock::new();

pub fn bump_flyout_generation() -> u64 {
    FLYOUT_GENERATION.fetch_add(1, Ordering::SeqCst) + 1
}

/// Ẩn Flyout tức thì (giữ ấm 60 giây để bấm lại mở 0ms), sau 60 giây không dùng thì tự hủy WebView2 trả về ~4.5MB RAM!
pub fn hide_flyout_with_idle_destroy(app: &AppHandle) {
    let flyout_gen = bump_flyout_generation();
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    let app_handle = app.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        if FLYOUT_GENERATION.load(Ordering::SeqCst) == flyout_gen {
            if let Some(w) = app_handle.get_webview_window("main") {
                if !w.is_visible().unwrap_or(false) {
                    let _ = w.destroy();
                }
            }
        }
    });
}

pub const LEAN_WEBVIEW_ARGS: &str = "--disable-background-networking --disable-component-update --disable-extensions --disable-sync --disable-gpu-shader-disk-cache --disk-cache-size=1048576 --media-cache-size=1048576 --renderer-process-limit=1 --disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --js-flags=--max-old-space-size=48";

/// Trả lại toàn bộ RAM nhàn rỗi của tiến trình Rust cho Windows sau khi đóng cửa sổ UI
pub fn trim_process_memory() {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::System::Memory::{
            SetProcessWorkingSetSizeEx, SETPROCESSWORKINGSETSIZEEX_FLAGS,
        };
        use windows::Win32::System::Threading::GetCurrentProcess;
        let _ = SetProcessWorkingSetSizeEx(
            GetCurrentProcess(),
            usize::MAX,
            usize::MAX,
            SETPROCESSWORKINGSETSIZEEX_FLAGS(0),
        );
    }
}

fn now_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn is_takeover_active_or_recent() -> bool {
    if TAKEOVER_IN_PROGRESS.load(Ordering::SeqCst) {
        return true;
    }
    let last = LAST_TAKEOVER_DONE_MS.load(Ordering::SeqCst);
    last > 0 && now_epoch_ms().saturating_sub(last) < 4000
}

struct TakeoverGuard;

impl Drop for TakeoverGuard {
    fn drop(&mut self) {
        LAST_TAKEOVER_DONE_MS.store(now_epoch_ms(), Ordering::SeqCst);
        TAKEOVER_IN_PROGRESS.store(false, Ordering::SeqCst);
    }
}

pub struct AppState {
    pub bt_manager: Arc<BluetoothManager>,
}

fn state_file_path() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("APPDATA"))
        .map(PathBuf::from)?;
    Some(base.join("LotusAir").join("device_state.json"))
}

pub fn save_persisted_state(state: &AirPodsDeviceState) {
    // Chỉ ghi xuống ổ đĩa khi cấu hình thực sự thay đổi (không ghi trên mỗi gói tin pin BLE!)
    let key = match serde_json::to_string(&(&state.name, &state.noise_mode, &state.features)) {
        Ok(k) => k,
        Err(_) => return,
    };
    let cache = LAST_PERSISTED_SNAPSHOT.get_or_init(|| Mutex::new(String::new()));
    if let Ok(mut last) = cache.lock() {
        if *last == key {
            return;
        }
        *last = key;
    }

    let Some(path) = state_file_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = std::fs::write(path, json);
    }
}

pub fn load_persisted_state() -> Option<AirPodsDeviceState> {
    let path = state_file_path()?;
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

#[tauri::command]
pub async fn get_device_state(state: State<'_, AppState>) -> Result<AirPodsDeviceState, String> {
    Ok(state.bt_manager.get_current_state().await)
}

#[tauri::command]
pub async fn set_noise_mode(mode: u8, state: State<'_, AppState>) -> Result<(), String> {
    let noise_mode = NoiseControlMode::from_u8(mode).map_err(|e| e.to_string())?;
    state.bt_manager.set_noise_mode(noise_mode).await
}

#[tauri::command]
pub async fn set_adaptive_blend(level: u8, state: State<'_, AppState>) -> Result<(), String> {
    state.bt_manager.set_adaptive_blend(level).await
}

#[tauri::command]
pub async fn rename_device(new_name: String, state: State<'_, AppState>) -> Result<(), String> {
    state.bt_manager.rename_device(new_name).await
}

pub async fn perform_airpods_takeover(
    app: &AppHandle,
    bt_manager: &Arc<BluetoothManager>,
) -> Result<(), String> {
    use tauri::Emitter;

    if TAKEOVER_IN_PROGRESS
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        info!("Đang có tiến trình cướp AirPods khác chạy, bỏ qua lệnh trùng lặp.");
        return Ok(());
    }
    let _guard = TakeoverGuard;

    // 0. Tạm dừng vòng lặp L2CAP trong lúc kéo A2DP để BthA2dp.sys toàn quyền dùng sóng Bluetooth,
    //    tránh xung đột ACL/AVDTP gây chậm hoặc giật âm thanh.
    bt_manager.set_takeover_paused(true);
    let bt_mgr_resume = bt_manager.clone();
    struct ResumeL2capGuard(Arc<BluetoothManager>);
    impl Drop for ResumeL2capGuard {
        fn drop(&mut self) {
            let mgr = self.0.clone();
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(1200)).await;
                mgr.set_takeover_paused(false);
            });
        }
    }
    let _l2cap_resume = ResumeL2capGuard(bt_mgr_resume);

    let known_name = lotusair_driver::find_connected_airpods().map(|d| d.name.to_lowercase());

    let is_airpods_output = |o: &crate::windows_audio::AudioOutput| -> bool {
        let lower = o.name.to_lowercase();
        lower.contains("airpod")
            || known_name
                .as_ref()
                .map(|kn| !kn.is_empty() && lower.contains(kn.as_str()))
                .unwrap_or(false)
    };

    let (was_stereo_active_before, was_default_before) = crate::windows_audio::get_audio_outputs()
        .map(|outs| {
            let stereo = outs
                .iter()
                .find(|o| is_airpods_output(o) && o.name.to_lowercase().contains("headphones"));
            (stereo.is_some(), stereo.map(|o| o.is_default).unwrap_or(false))
        })
        .unwrap_or((false, false));

    // Nếu AirPods đang ở iPhone (endpoint Stereo chưa ACTIVE trên Win), đóng kênh L2CAP cũ trước
    if !was_stereo_active_before {
        bt_manager.drop_stale_l2cap().await;
    }

    // 1. Gửi ĐÚNG 1 lệnh KSPROPERTY_ONESHOT_RECONNECT xuống BthA2dp.sys (không bắn lặp gây giật sóng)
    let a2dp_res =
        tokio::task::spawn_blocking(|| crate::windows_audio::trigger_airpods_a2dp_control(true))
            .await
            .map_err(|e| e.to_string())?;
    info!(
        "Kết quả kéo A2DP AirPods từ iPhone về PC (was_active={}, was_default={}): {:?}",
        was_stereo_active_before, was_default_before, a2dp_res
    );

    if let Err(err) = a2dp_res {
        return Err(format!("Không gửi được lệnh kéo AirPods về PC: {err}"));
    }

    // 2. Chờ endpoint Stereo Headphones của AirPods chuyển sang ACTIVE
    //    Poll nhanh mỗi 100ms, tối đa 4.5 giây để phản hồi tức thì và thoát gọn nếu iPhone đang giữ kết nối.
    let mut found_airpods_id: Option<String> = None;

    if was_stereo_active_before {
        // Endpoint đã ACTIVE sẵn -> chỉ chờ ngắn cho AVDTP ổn định
        let wait_ms = if was_default_before { 450 } else { 250 };
        tokio::time::sleep(tokio::time::Duration::from_millis(wait_ms)).await;
        if let Ok(outputs) = crate::windows_audio::get_audio_outputs() {
            found_airpods_id = outputs
                .iter()
                .find(|o| is_airpods_output(o) && o.name.to_lowercase().contains("headphones"))
                .or_else(|| outputs.iter().find(|o| is_airpods_output(o)))
                .map(|o| o.id.clone());
        }
    } else {
        for attempt in 0..45 {
            if let Ok(outputs) = crate::windows_audio::get_audio_outputs() {
                let stereo = outputs
                    .iter()
                    .find(|o| is_airpods_output(o) && o.name.to_lowercase().contains("headphones"));
                let any_airpods = stereo.or_else(|| {
                    if attempt >= 30 {
                        outputs.iter().find(|o| is_airpods_output(o))
                    } else {
                        None
                    }
                });

                if let Some(ep) = any_airpods {
                    found_airpods_id = Some(ep.id.clone());
                    break;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    // Nếu không kéo được AirPods lên ACTIVE -> dừng gọn, KHÔNG nảy loa và KHÔNG đụng L2CAP gây giật!
    let Some(mut airpods_id) = found_airpods_id else {
        let _ = app.emit("audio-outputs-changed", true);
        return Err(
            "Không kéo được AirPods về PC (iPhone đang giữ kết nối hoặc tai nghe ở ngoài vùng phủ sóng)."
                .into(),
        );
    };

    // 3. Nếu vừa mới nổi từ UNPLUGGED -> ACTIVE, chờ 250ms cho AVDTP stream mở xong rồi định tuyến ngay
    if !was_stereo_active_before {
        tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
    }

    let mut is_default_now = false;
    let mut fallback_other_id: Option<String> = None;
    if let Ok(fresh_outputs) = crate::windows_audio::get_audio_outputs() {
        fallback_other_id = fresh_outputs
            .iter()
            .find(|o| !is_airpods_output(o))
            .map(|o| o.id.clone());

        if let Some(ep) = fresh_outputs
            .iter()
            .find(|o| is_airpods_output(o) && o.name.to_lowercase().contains("headphones"))
            .or_else(|| fresh_outputs.iter().find(|o| o.id == airpods_id))
        {
            airpods_id = ep.id.clone();
            is_default_now = ep.is_default;
        }
    }

    // Chỉ nảy nhẹ 150ms qua loa dự phòng khi Windows đã tự đặt AirPods làm default quá sớm
    // (để ép WASAPI rebind); nếu đang ở Speakers thì chuyển thẳng sang AirPods luôn (không giật)!
    if is_default_now {
        if let Some(other_id) = fallback_other_id {
            info!("Windows đã giữ AirPods default từ trước -> nảy nhanh 150ms để khơi thông WASAPI");
            let _ = crate::windows_audio::set_default_audio_output(other_id).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        }
    }
    let _ = crate::windows_audio::set_default_audio_output(airpods_id).await;

    // 4. Đẩy sự kiện cập nhật ngay xuống toàn bộ cửa sổ UI (không chặn chờ L2CAP!)
    if let Ok(conn) = crate::windows_bluetooth::get_windows_connection() {
        let _ = app.emit("windows-connection-changed", &conn);
        crate::tray::update_tray_connection(app, conn.connected);
    }
    let _ = app.emit("audio-outputs-changed", true);

    Ok(())
}

#[tauri::command]
pub async fn force_takeover_audio(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    perform_airpods_takeover(&app, &state.bt_manager).await
}

#[tauri::command]
pub async fn set_spatial_audio_mode(mode: String, state: State<'_, AppState>) -> Result<(), String> {
    let m = match mode.as_str() {
        "fixed" => SpatialAudioMode::Fixed,
        "headtracked" => SpatialAudioMode::HeadTracked,
        _ => SpatialAudioMode::Off,
    };
    state.bt_manager.set_spatial_audio_mode(m).await
}

#[tauri::command]
pub async fn set_keep_alive(enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    state.bt_manager.set_keep_alive(enabled).await
}

#[tauri::command]
pub async fn toggle_conversational_awareness(enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    state.bt_manager.toggle_conversational_awareness(enabled).await
}

#[tauri::command]
pub async fn set_personalized_volume(enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    state.bt_manager.set_personalized_volume(enabled).await
}

#[tauri::command]
pub async fn set_loud_noise_reduction(enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    state.bt_manager.set_loud_noise_reduction(enabled).await
}

#[tauri::command]
pub async fn set_one_bud_anc(enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    state.bt_manager.set_one_bud_anc(enabled).await
}

#[tauri::command]
pub async fn set_head_gestures(enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    state.bt_manager.set_head_gestures(enabled).await
}

#[tauri::command]
pub async fn set_auto_ear_detection(enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    state.bt_manager.set_auto_ear_detection(enabled).await
}

#[tauri::command]
pub async fn set_auto_takeover(
    app: AppHandle,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let _ = state.bt_manager.set_auto_takeover(enabled).await;
    if enabled {
        let _ = perform_airpods_takeover(&app, &state.bt_manager).await;
    }
    Ok(())
}

#[tauri::command]
pub async fn set_stem_press_action(side: String, action: String, state: State<'_, AppState>) -> Result<(), String> {
    state.bt_manager.set_stem_press_action(side, action).await
}

#[tauri::command]
pub async fn set_cycle_modes(
    anc: bool,
    adaptive: bool,
    transparency: bool,
    off: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .bt_manager
        .set_cycle_modes(anc, adaptive, transparency, off)
        .await
}

#[tauri::command]
pub async fn set_case_charging_sounds(enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    state.bt_manager.set_case_charging_sounds(enabled).await
}

#[tauri::command]
pub async fn set_case_chime_volume(volume: u8, state: State<'_, AppState>) -> Result<(), String> {
    state.bt_manager.set_case_chime_volume(volume).await
}

#[tauri::command]
pub async fn run_acoustic_fit_test(state: State<'_, AppState>) -> Result<String, String> {
    state.bt_manager.run_acoustic_fit_test().await
}

#[tauri::command]
pub async fn disconnect_device(state: State<'_, AppState>) -> Result<(), String> {
    let _ = tokio::task::spawn_blocking(|| crate::windows_audio::trigger_airpods_a2dp_control(false)).await;
    state.bt_manager.disconnect_device().await
}

#[tauri::command]
pub async fn close_popup(app: AppHandle) -> Result<(), String> {
    POPUP_GENERATION.fetch_add(1, Ordering::SeqCst);
    if let Some(window) = app.get_webview_window("popup") {
        let _ = window.destroy();
    }
    Ok(())
}

pub async fn open_settings_window(app: &AppHandle) -> Result<(), String> {
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.destroy();
    }

    let window = if let Some(existing) = app.get_webview_window("settings") {
        existing
    } else {
        #[allow(unused_mut)]
        let mut builder = WebviewWindowBuilder::new(
            app,
            "settings",
            WebviewUrl::App("index.html#settings".into()),
        )
        .title("LotusAir AirPods Settings")
        .inner_size(540.0, 820.0)
        .resizable(true)
        .decorations(false)
        .transparent(true)
        .shadow(true)
        .theme(Some(Theme::Dark))
        .effects(
            EffectsBuilder::new()
                .effect(Effect::Acrylic)
                .state(EffectState::Active)
                .color(Color(18, 18, 24, 65))
                .build(),
        )
        .always_on_top(false)
        .skip_taskbar(false)
        .visible(false);
        #[cfg(windows)]
        {
            builder = builder.additional_browser_args(LEAN_WEBVIEW_ARGS);
        }
        builder.build().map_err(|e| e.to_string())?
    };

    if let Some(state) = app.try_state::<AppState>() {
        let current = state.bt_manager.get_current_state().await;
        let _ = tauri::Emitter::emit(app, "device-state-updated", &current);
    }
    let _ = window.center();
    window.show().map_err(|error| error.to_string())?;
    let _ = window.set_focus();
    Ok(())
}

#[tauri::command]
pub async fn show_settings_window(app: AppHandle) -> Result<(), String> {
    open_settings_window(&app).await
}

#[tauri::command]
pub async fn hide_settings_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.destroy();
    }
    crate::tray::show_flyout(&app);
    Ok(())
}

#[tauri::command]
pub async fn show_connection_popup(app: AppHandle) -> Result<(), String> {
    use tauri::Emitter;

    let popup_gen = POPUP_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    info!("Hiển thị Popup AirPods (lazy WebView2, gen={})...", popup_gen);

    let window = if let Some(existing) = app.get_webview_window("popup") {
        existing
    } else {
        #[allow(unused_mut)]
        let mut builder = WebviewWindowBuilder::new(
            &app,
            "popup",
            WebviewUrl::App("index.html#popup".into()),
        )
        .title("LotusAir Connected")
        .inner_size(320.0, 300.0)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(true)
        .theme(Some(Theme::Light))
        .effects(
            EffectsBuilder::new()
                .effect(Effect::Acrylic)
                .state(EffectState::Active)
                .color(Color(246, 248, 252, 115))
                .build(),
        )
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .visible(false);
        #[cfg(windows)]
        {
            builder = builder.additional_browser_args(LEAN_WEBVIEW_ARGS);
        }
        builder.build().map_err(|e| e.to_string())?
    };

    if let Some(state) = app.try_state::<AppState>() {
        let current = state.bt_manager.get_current_state().await;
        let _ = app.emit("device-state-updated", &current);
    }
    let _ = app.emit("popup-shown", popup_gen);

    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| app.primary_monitor().ok().flatten());
    if let Some(monitor) = monitor {
        let screen_size = monitor.size();
        let scale_factor = monitor.scale_factor();
        let window_w = (320.0 * scale_factor) as i32;
        let window_h = (300.0 * scale_factor) as i32;
        let margin_x = (10.0 * scale_factor) as i32;
        let taskbar_h = (48.0 * scale_factor) as i32;
        let margin_y = (6.0 * scale_factor) as i32;

        let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: window_w as u32,
            height: window_h as u32,
        }));

        let origin = monitor.position();
        let pos_x = origin.x + (screen_size.width as i32) - window_w - margin_x;
        let pos_y = origin.y + (screen_size.height as i32) - window_h - taskbar_h - margin_y;

        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: pos_x,
            y: pos_y,
        }));
    }
    let _ = window.show();

    let app_handle = app.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(6500)).await;
        if POPUP_GENERATION.load(Ordering::SeqCst) == popup_gen {
            if let Some(w) = app_handle.get_webview_window("popup") {
                let _ = w.destroy();
            }
        }
    });
    Ok(())
}
