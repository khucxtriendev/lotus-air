use lotusair_bluetooth::BluetoothManager;
use lotusair_core::models::{AirPodsDeviceState, NoiseControlMode, SpatialAudioMode};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tracing::info;

pub struct AppState {
    pub bt_manager: Arc<BluetoothManager>,
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

#[tauri::command]
pub async fn force_takeover_audio(state: State<'_, AppState>) -> Result<(), String> {
    state.bt_manager.force_takeover_audio().await
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
pub async fn close_popup(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("popup") {
        let _ = window.hide();
    }
    Ok(())
}

#[tauri::command]
pub async fn show_connection_popup(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("popup") {
        info!("Hiển thị Popup thông báo AirPods đã kết nối ở góc phải dưới...");
        if let Ok(Some(monitor)) = window.current_monitor() {
            let screen_size = monitor.size();
            let scale_factor = monitor.scale_factor();
            let window_w = (380.0 * scale_factor) as i32;
            let window_h = (410.0 * scale_factor) as i32;
            let margin = (24.0 * scale_factor) as i32;

            let pos_x = (screen_size.width as i32) - window_w - margin;
            let pos_y = (screen_size.height as i32) - window_h - margin - (48.0 * scale_factor) as i32;

            let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x: pos_x, y: pos_y }));
        }
        let _ = window.show();
        let _ = window.set_focus();

        let app_handle = app.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(6500)).await;
            if let Some(w) = app_handle.get_webview_window("popup") {
                let _ = w.hide();
            }
        });
    }
    Ok(())
}
