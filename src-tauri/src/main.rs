#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod tray;

use commands::*;
use lotusair_bluetooth::BluetoothManager;
use std::sync::Arc;
use tauri::{Emitter, Listener, Manager};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("lotusair=debug".parse().unwrap()))
        .init();

    info!("LotusAir đang khởi động...");

    let (bt_manager, mut state_rx) = BluetoothManager::new();
    let bt_manager = Arc::new(bt_manager);

    let bt_mgr_clone = bt_manager.clone();
    tokio::spawn(async move {
        bt_mgr_clone.start().await;
    });

    let bt_mgr_for_tray = bt_manager.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            bt_manager: bt_manager.clone(),
        })
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // Khởi tạo System Tray Icon
            tray::setup_tray(&app_handle)?;

            // Kích hoạt hiệu ứng Mica / Acrylic Liquid Glass trên Windows
            if let Some(_main_window) = app.get_webview_window("main") {
                #[cfg(target_os = "windows")]
                {
                    use window_vibrancy::{apply_acrylic, apply_mica};
                    let _ = apply_mica(&_main_window, Some(true))
                        .or_else(|_| apply_acrylic(&_main_window, Some((20, 20, 20, 160))));
                }
            }

            if let Some(_popup_window) = app.get_webview_window("popup") {
                #[cfg(target_os = "windows")]
                {
                    use window_vibrancy::{apply_acrylic, apply_mica};
                    let _ = apply_mica(&_popup_window, Some(true))
                        .or_else(|_| apply_acrylic(&_popup_window, Some((20, 20, 20, 160))));
                }
            }

            // Lắng nghe sự kiện đổi noise mode từ Tray Menu
            let bt_mgr_tray = bt_mgr_for_tray.clone();
            app.listen("tray-noise-mode", move |event| {
                if let Ok(mode_num) = serde_json::from_str::<u8>(event.payload()) {
                    let mgr = bt_mgr_tray.clone();
                    tokio::spawn(async move {
                        if let Ok(m) = lotusair_core::models::NoiseControlMode::from_u8(mode_num) {
                            let _ = mgr.set_noise_mode(m).await;
                        }
                    });
                }
            });

            // Lắng nghe sự kiện từ Bluetooth Manager: khi trạng thái đổi, push xuống frontend
            let app_handle_for_events = app_handle.clone();
            tokio::spawn(async move {
                let mut first_connect = true;
                while let Ok(state) = state_rx.recv().await {
                    let _ = app_handle_for_events.emit("device-state-updated", &state);

                    // Khi tai nghe kết nối -> tự động bật Popup góc phải bên dưới
                    if state.connected && first_connect {
                        first_connect = false;
                        tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
                        let _ = commands::show_connection_popup(app_handle_for_events.clone()).await;
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_device_state,
            set_noise_mode,
            set_adaptive_blend,
            rename_device,
            force_takeover_audio,
            set_spatial_audio_mode,
            set_keep_alive,
            toggle_conversational_awareness,
            close_popup,
            show_connection_popup,
        ])
        .run(tauri::generate_context!())
        .expect("Lỗi khi chạy ứng dụng LotusAir");
}
