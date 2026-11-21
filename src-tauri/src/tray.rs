use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};
use tracing::info;

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let anc_item = MenuItem::with_id(app, "mode_anc", "Chống ồn chủ động (ANC)", true, None::<&str>)?;
    let trans_item = MenuItem::with_id(app, "mode_trans", "Xuyên âm (Transparency)", true, None::<&str>)?;
    let adapt_item = MenuItem::with_id(app, "mode_adaptive", "Âm thanh thích ứng (Adaptive)", true, None::<&str>)?;
    let off_item = MenuItem::with_id(app, "mode_off", "Tắt chống ồn", true, None::<&str>)?;
    let sep = MenuItem::with_id(app, "sep", "---", false, None::<&str>)?;
    let popup_test_item = MenuItem::with_id(app, "test_popup", "Xem thử Popup kết nối (3D)", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Thoát LotusAir", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &anc_item,
            &trans_item,
            &adapt_item,
            &off_item,
            &sep,
            &popup_test_item,
            &quit_item,
        ],
    )?;

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("LotusAir - Apple AirPods 4 ANC Controller")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "mode_anc" => {
                let _ = app.emit("tray-noise-mode", 2);
            }
            "mode_trans" => {
                let _ = app.emit("tray-noise-mode", 3);
            }
            "mode_adaptive" => {
                let _ = app.emit("tray-noise-mode", 4);
            }
            "mode_off" => {
                let _ = app.emit("tray-noise-mode", 1);
            }
            "test_popup" => {
                let app_handle = app.clone();
                tokio::spawn(async move {
                    let _ = crate::commands::show_connection_popup(app_handle).await;
                });
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        // Căn vị trí cửa sổ Flyout cạnh System Tray
                        if let Ok(Some(monitor)) = window.current_monitor() {
                            let screen_size = monitor.size();
                            let scale_factor = monitor.scale_factor();
                            let window_w = (380.0 * scale_factor) as i32;
                            let window_h = (560.0 * scale_factor) as i32;
                            let margin = (16.0 * scale_factor) as i32;

                            let pos_x = (screen_size.width as i32) - window_w - margin;
                            let pos_y = (screen_size.height as i32) - window_h - margin - (48.0 * scale_factor) as i32;

                            let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x: pos_x, y: pos_y }));
                        }
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    info!("System Tray khởi tạo thành công!");
    Ok(())
}
