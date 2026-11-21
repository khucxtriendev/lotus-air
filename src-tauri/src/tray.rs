use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::window::{Color, Effect, EffectState, EffectsBuilder};
use tauri::{AppHandle, Emitter, Manager, Theme, WebviewUrl, WebviewWindowBuilder};
use tracing::{info, warn};

pub fn show_flyout(app: &AppHandle) {
    crate::commands::bump_flyout_generation();
    let reused = app.get_webview_window("main").is_some();
    let window = if let Some(existing) = app.get_webview_window("main") {
        existing
    } else {
        #[allow(unused_mut)]
        let mut builder = WebviewWindowBuilder::new(
            app,
            "main",
            WebviewUrl::App("index.html#control".into()),
        )
        .title("LotusAir Control Center")
        .inner_size(360.0, 700.0)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(true)
        .theme(Some(Theme::Dark))
        .effects(
            EffectsBuilder::new()
                .effect(Effect::Acrylic)
                .state(EffectState::Active)
                .color(Color(18, 18, 24, 55))
                .build(),
        )
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false);
        #[cfg(windows)]
        {
            builder = builder.additional_browser_args(crate::commands::LEAN_WEBVIEW_ARGS);
        }
        match builder.build() {
            Ok(w) => w,
            Err(e) => {
                warn!("Không tạo được cửa sổ Control Flyout: {e}");
                return;
            }
        }
    };

    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| app.primary_monitor().ok().flatten());
    if let Some(monitor) = monitor {
        let scale = monitor.scale_factor();
        let screen = monitor.size();
        let origin = monitor.position();
        let margin = (14.0 * scale) as i32;
        let taskbar = (54.0 * scale) as i32;
        let width = ((360.0 * scale) as i32).min(screen.width as i32 - 2 * margin);
        let height = ((700.0 * scale) as i32).min(screen.height as i32 - taskbar - 2 * margin);
        let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: width.max(360) as u32,
            height: height.max(380) as u32,
        }));
        let x = origin.x + screen.width as i32 - width - margin;
        let y = origin.y + screen.height as i32 - height - taskbar - margin;
        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x,
            y: y.max(origin.y + margin),
        }));
    }
    if reused {
        let _ = app.emit("flyout-shown", true);
    }
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let takeover_item = MenuItem::with_id(app, "takeover_airpods", "Cướp AirPods từ iPhone -> PC", true, None::<&str>)?;
    let sep0 = MenuItem::with_id(app, "sep0", "---", false, None::<&str>)?;
    let anc_item = MenuItem::with_id(app, "mode_anc", "Chống ồn chủ động (ANC)", true, None::<&str>)?;
    let trans_item = MenuItem::with_id(app, "mode_trans", "Xuyên âm (Transparency)", true, None::<&str>)?;
    let adapt_item = MenuItem::with_id(app, "mode_adaptive", "Âm thanh thích ứng (Adaptive)", true, None::<&str>)?;
    let off_item = MenuItem::with_id(app, "mode_off", "Tắt chống ồn", true, None::<&str>)?;
    let sep = MenuItem::with_id(app, "sep", "---", false, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "open_settings", "Cài đặt AirPods 4...", true, None::<&str>)?;
    let popup_test_item = MenuItem::with_id(app, "test_popup", "Xem thử Popup kết nối (3D)", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Thoát LotusAir", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &takeover_item,
            &sep0,
            &anc_item,
            &trans_item,
            &adapt_item,
            &off_item,
            &sep,
            &settings_item,
            &popup_test_item,
            &quit_item,
        ],
    )?;

    let mut tray_builder = TrayIconBuilder::with_id("lotusair")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("LotusAir - Apple AirPods 4 ANC Controller");
    tray_builder = tray_builder.icon(render_airpod_tray_icon(false));
    let _tray = tray_builder
        .on_menu_event(|app, event| match event.id().as_ref() {
            "takeover_airpods" => {
                let _ = app.emit("tray-takeover-airpods", true);
            }
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
            "open_settings" => {
                let app_handle = app.clone();
                tokio::spawn(async move {
                    let _ = crate::commands::open_settings_window(&app_handle).await;
                });
            }
            "test_popup" => {
                let app_handle = app.clone();
                tokio::spawn(async move {
                    let _ = crate::commands::show_connection_popup(app_handle).await;
                });
            }
            "quit" => {
                crate::request_app_exit(app);
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
                        crate::commands::hide_flyout_with_idle_destroy(app);
                    } else {
                        show_flyout(app);
                    }
                } else {
                    show_flyout(app);
                }
            }
        })
        .build(app)?;

    info!("System Tray khởi tạo thành công!");
    Ok(())
}

/// Vẽ icon 1 bên tai nghe AirPod nét mảnh màu đen (anti-aliased 32x32),
/// kèm dấu tích màu xanh lá ở góc dưới phải khi đang kết nối.
fn render_airpod_tray_icon(connected: bool) -> Image<'static> {
    const SIZE: u32 = 32;
    let mut pixels = vec![0u8; (SIZE * SIZE * 4) as usize];

    fn blend_pixel(pixels: &mut [u8], x: u32, y: u32, r: u8, g: u8, b: u8, a: f32) {
        if x >= SIZE || y >= SIZE || a <= 0.001 {
            return;
        }
        let idx = ((y * SIZE + x) * 4) as usize;
        let sa = a.clamp(0.0, 1.0);
        let da = pixels[idx + 3] as f32 / 255.0;
        let out_a = sa + da * (1.0 - sa);
        if out_a <= 0.0001 {
            return;
        }
        let dr = pixels[idx] as f32;
        let dg = pixels[idx + 1] as f32;
        let db = pixels[idx + 2] as f32;
        pixels[idx] = ((r as f32 * sa + dr * da * (1.0 - sa)) / out_a).round() as u8;
        pixels[idx + 1] = ((g as f32 * sa + dg * da * (1.0 - sa)) / out_a).round() as u8;
        pixels[idx + 2] = ((b as f32 * sa + db * da * (1.0 - sa)) / out_a).round() as u8;
        pixels[idx + 3] = (out_a * 255.0).round() as u8;
    }

    fn dist_segment(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
        let vx = bx - ax;
        let vy = by - ay;
        let wx = px - ax;
        let wy = py - ay;
        let len_sq = vx * vx + vy * vy;
        let t = if len_sq > 0.0 {
            ((wx * vx + wy * vy) / len_sq).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let cx = ax + t * vx;
        let cy = ay + t * vy;
        ((px - cx).powi(2) + (py - cy).powi(2)).sqrt()
    }

    // Hình dáng 1 bên AirPod (đầu tai nghe bo tròn bên trái + thân mảnh kéo xuống bên phải)
    // Dùng SDF hợp nhất của phần đầu (capsule ngang nhẹ) và phần thân (capsule dọc mảnh)
    let airpod_sdf = |x: f32, y: f32| -> f32 {
        // Phần đầu tai nghe (nghiêng nhẹ theo kiểu AirPods 4)
        let d_head = dist_segment(x, y, 10.5, 9.8, 16.2, 10.5) - 5.2;
        // Phần thân tai nghe mảnh kéo xuống
        let d_stem = dist_segment(x, y, 17.8, 12.2, 17.8, 26.8) - 2.35;
        // Smooth-min nhẹ để cổ tai nghe nối mượt tự nhiên
        let k = 1.6f32;
        let h = (0.5 + 0.5 * (d_stem - d_head) / k).clamp(0.0, 1.0);
        d_stem * (1.0 - h) + d_head * h - k * h * (1.0 - h)
    };

    let stroke_half = 0.85f32; // Nét mảnh ~1.7px sắc nét

    for y in 0..SIZE {
        for x in 0..SIZE {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            // Viền nét mảnh màu đen của 1 bên AirPod
            let d = airpod_sdf(px, py);
            let edge = (d.abs() - stroke_half).max(0.0);
            let stroke_alpha = (1.0 - edge / 0.95).clamp(0.0, 1.0);
            if stroke_alpha > 0.0 {
                blend_pixel(&mut pixels, x, y, 18, 18, 20, stroke_alpha);
            }

            // Khe loa nhỏ (accent mảnh bên trong đầu tai nghe)
            let d_grill = dist_segment(px, py, 9.8, 8.6, 9.8, 11.2) - 0.75;
            let grill_alpha = (0.8 - d_grill).clamp(0.0, 1.0);
            if grill_alpha > 0.0 {
                blend_pixel(&mut pixels, x, y, 18, 18, 20, grill_alpha);
            }

            // Nếu đang kết nối -> vẽ badge xanh lá (#34C759) + dấu tích trắng (✓) ở góc dưới phải
            if connected {
                let bx = 24.5f32;
                let by = 24.5f32;
                let br = 6.3f32;
                let db = ((px - bx).powi(2) + (py - by).powi(2)).sqrt() - br;

                // Nền vòng tròn xanh lá
                let badge_alpha = (0.6 - db).clamp(0.0, 1.0);
                if badge_alpha > 0.0 {
                    blend_pixel(&mut pixels, x, y, 52, 199, 89, badge_alpha);
                }

                // Dấu tích trắng mảnh (✓) bên trong vòng tròn xanh
                let d_chk1 = dist_segment(px, py, 21.6, 24.7, 23.6, 26.8) - 0.85;
                let d_chk2 = dist_segment(px, py, 23.6, 26.8, 27.6, 22.3) - 0.85;
                let d_chk = d_chk1.min(d_chk2);
                let chk_alpha = (0.65 - d_chk).clamp(0.0, 1.0);
                if chk_alpha > 0.0 {
                    blend_pixel(&mut pixels, x, y, 255, 255, 255, chk_alpha);
                }
            }
        }
    }

    Image::new_owned(pixels, SIZE, SIZE)
}

pub fn update_tray_connection(app: &AppHandle, connected: bool) {
    let Some(tray) = app.tray_by_id("lotusair") else {
        warn!("Không tìm thấy biểu tượng LotusAir trong khay hệ thống");
        return;
    };
    if let Err(error) = tray.set_icon(Some(render_airpod_tray_icon(connected))) {
        warn!("Không cập nhật được biểu tượng kết nối: {error}");
    }
    let status = if connected { "đã kết nối Windows" } else { "chưa kết nối Windows" };
    if let Err(error) = tray.set_tooltip(Some(format!("LotusAir • AirPods {status}"))) {
        warn!("Không cập nhật được chú thích biểu tượng: {error}");
    }
    info!("Biểu tượng LotusAir: AirPods {status}");
}
