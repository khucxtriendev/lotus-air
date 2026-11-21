#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod tray;
mod windows_audio;
mod windows_bluetooth;

use commands::*;
use lotusair_bluetooth::BluetoothManager;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{Emitter, Listener, Manager};
use tracing::info;
use tracing_subscriber::EnvFilter;

static ALLOW_APP_EXIT: AtomicBool = AtomicBool::new(false);

pub fn request_app_exit(app: &tauri::AppHandle) {
    ALLOW_APP_EXIT.store(true, Ordering::SeqCst);
    app.exit(0);
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("lotusair=debug".parse().unwrap()),
        )
        .init();

    info!("LotusAir đang khởi động...");

    let (bt_manager, mut state_rx) = BluetoothManager::new();

    // Khôi phục cấu hình đã lưu từ lần chạy trước
    if let Some(saved) = commands::load_persisted_state() {
        let mut sm = bt_manager.state_machine.lock().await;
        if !saved.name.is_empty() && saved.name != "AirPods" {
            sm.state.name = saved.name;
        }
        sm.state.noise_mode = saved.noise_mode;
        sm.state.features = saved.features;
    }

    let bt_manager = Arc::new(bt_manager);
    let mut popup_rx = bt_manager.subscribe_popup_events();

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
        .on_window_event(|window, event| {
            if window.label() == "main" && matches!(event, tauri::WindowEvent::Focused(false)) {
                let flyout = window.clone();
                let app_handle = window.app_handle().clone();
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(130)).await;
                    if !flyout.is_focused().unwrap_or(false) {
                        // Ẩn ngay (giữ ấm 60s để mở lại 0ms), sau 60s tự hủy WebView2 trả về ~4.5MB RAM
                        commands::hide_flyout_with_idle_destroy(&app_handle);
                    }
                });
            }
        })
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // Khởi tạo System Tray Icon thuần Rust (không tạo sẵn cửa sổ WebView2 lúc khởi động!)
            tray::setup_tray(&app_handle)?;
            tokio::spawn(async {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                commands::trim_process_memory();
            });

            // Lắng nghe lệnh Cướp kết nối AirPods từ iPhone sang PC từ Tray Menu
            let bt_mgr_takeover = bt_mgr_for_tray.clone();
            let app_handle_takeover = app_handle.clone();
            app.listen("tray-takeover-airpods", move |_event| {
                let mgr = bt_mgr_takeover.clone();
                let handle = app_handle_takeover.clone();
                tokio::spawn(async move {
                    if let Err(err) = commands::perform_airpods_takeover(&handle, &mgr).await {
                        let _ = handle.emit("control-error", err);
                    }
                });
            });

            // Lắng nghe sự kiện đổi noise mode từ Tray Menu
            let bt_mgr_tray = bt_mgr_for_tray.clone();
            let app_handle_for_tray_error = app_handle.clone();
            app.listen("tray-noise-mode", move |event| {
                if let Ok(mode_num) = serde_json::from_str::<u8>(event.payload()) {
                    let mgr = bt_mgr_tray.clone();
                    let app_handle_for_tray_error = app_handle_for_tray_error.clone();
                    tokio::spawn(async move {
                        if let Ok(m) = lotusair_core::models::NoiseControlMode::from_u8(mode_num) {
                            if let Err(error) = mgr.set_noise_mode(m).await {
                                tray::show_flyout(&app_handle_for_tray_error);
                                let _ = app_handle_for_tray_error.emit("control-error", error);
                            }
                        }
                    });
                }
            });

            // Lắng nghe sự kiện BLE mở nắp hộp / lấy tai nghe khỏi hộp -> bật Popup góc phải dưới tức thì (kể cả khi đang nối iPhone!)
            let app_handle_for_ble_popup = app_handle.clone();
            tokio::spawn(async move {
                while let Ok(reason) = popup_rx.recv().await {
                    info!("Nhận sự kiện BLE bật Popup ({})", reason);
                    let _ = commands::show_connection_popup(app_handle_for_ble_popup.clone()).await;
                }
            });

            // Lắng nghe sự kiện từ Bluetooth Manager: chỉ ghi disk khi cấu hình đổi & chỉ emit khi có cửa sổ UI đang mở
            let app_handle_for_events = app_handle.clone();
            tokio::spawn(async move {
                while let Ok(state) = state_rx.recv().await {
                    commands::save_persisted_state(&state);
                    if !app_handle_for_events.webview_windows().is_empty() {
                        let _ = app_handle_for_events.emit("device-state-updated", &state);
                    }
                }
            });

            let app_handle_for_windows = app_handle.clone();
            let bt_mgr_for_output_watch = bt_mgr_for_tray.clone();
            tokio::spawn(async move {
                let mut previous = None;
                let mut prev_default_airpods: Option<bool> = None;
                loop {
                    if let Ok(connection) = windows_bluetooth::get_windows_connection() {
                        if previous != Some(connection.connected) {
                            if !app_handle_for_windows.webview_windows().is_empty() {
                                let _ = app_handle_for_windows
                                    .emit("windows-connection-changed", &connection);
                            }
                            tray::update_tray_connection(
                                &app_handle_for_windows,
                                connection.connected,
                            );
                            if connection.connected && previous == Some(false) {
                                let _ = commands::show_connection_popup(
                                    app_handle_for_windows.clone(),
                                )
                                .await;
                            }
                            previous = Some(connection.connected);
                        }
                    }

                    // Cướp kết nối chủ động khi người dùng chọn đầu ra mặc định trên Windows là AirPods;
                    // ngược lại để AirPods chuyển đổi tự nhiên theo logic của Apple.
                    if let Ok(outputs) = windows_audio::get_audio_outputs() {
                        let known_name = bt_mgr_for_output_watch
                            .get_current_state()
                            .await
                            .name
                            .to_lowercase();
                        let default_is_airpods = outputs.iter().any(|o| {
                            if !o.is_default {
                                return false;
                            }
                            let lower = o.name.to_lowercase();
                            lower.contains("airpod")
                                || (!known_name.is_empty() && lower.contains(&known_name))
                        });
                        if commands::is_takeover_active_or_recent() {
                            // Bỏ qua trong lúc đang cướp hoặc vừa cướp xong để tránh lặp lại khi nảy loa
                            prev_default_airpods = Some(default_is_airpods);
                        } else {
                            if prev_default_airpods == Some(false) && default_is_airpods {
                                info!("Phát hiện người dùng vừa chọn đầu ra Windows là AirPods -> Cướp kết nối chủ động từ iPhone!");
                                let _ = commands::perform_airpods_takeover(
                                    &app_handle_for_windows,
                                    &bt_mgr_for_output_watch,
                                )
                                .await;
                            }
                            prev_default_airpods = Some(default_is_airpods);
                        }
                    }

                    tokio::time::sleep(tokio::time::Duration::from_millis(2500)).await;
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
            set_personalized_volume,
            set_loud_noise_reduction,
            set_one_bud_anc,
            set_head_gestures,
            set_auto_ear_detection,
            set_auto_takeover,
            set_stem_press_action,
            set_cycle_modes,
            set_case_charging_sounds,
            set_case_chime_volume,
            run_acoustic_fit_test,
            disconnect_device,
            close_popup,
            show_connection_popup,
            show_settings_window,
            hide_settings_window,
            windows_audio::get_windows_volume,
            windows_audio::get_audio_outputs,
            windows_audio::set_default_audio_output,
            windows_audio::get_current_media,
            windows_audio::set_windows_volume,
            windows_audio::media_control,
            windows_bluetooth::get_windows_connection,
        ])
        .build(tauri::generate_context!())
        .expect("Lỗi khi khởi tạo ứng dụng LotusAir")
        .run(|app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                if !ALLOW_APP_EXIT.load(Ordering::SeqCst) {
                    // Giữ tiến trình nền thuần Rust ở khay hệ thống khi đã hủy toàn bộ cửa sổ WebView2
                    api.prevent_exit();
                }
            }
            tauri::RunEvent::WindowEvent {
                event: tauri::WindowEvent::Destroyed,
                ..
            } => {
                if app_handle.webview_windows().is_empty() {
                    tokio::spawn(async {
                        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                        commands::trim_process_memory();
                    });
                }
            }
            _ => {}
        });
}
