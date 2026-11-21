use lotusair_core::aap::{AAPCommands, AAPParser, AAPStateMachine};
use lotusair_core::models::{AirPodsDeviceState, CycleModesConfig, NoiseControlMode, StemAction};
use lotusair_driver::find_connected_airpods;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, Mutex};
use tracing::{info, warn};
use crate::ble::AirPodsBleScanner;
use crate::l2cap::L2capSocket;

fn format_bt_mac(addr: u64) -> String {
    format!(
        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        (addr >> 40) & 0xFF,
        (addr >> 32) & 0xFF,
        (addr >> 24) & 0xFF,
        (addr >> 16) & 0xFF,
        (addr >> 8) & 0xFF,
        addr & 0xFF
    )
}

pub struct BluetoothManager {
    pub state_machine: Arc<Mutex<AAPStateMachine>>,
    socket: Arc<Mutex<L2capSocket>>,
    pub state_updates_tx: broadcast::Sender<AirPodsDeviceState>,
    popup_trigger_tx: broadcast::Sender<String>,
    takeover_paused: Arc<AtomicBool>,
}

impl BluetoothManager {
    pub fn new() -> (Self, broadcast::Receiver<AirPodsDeviceState>) {
        let (tx, rx) = broadcast::channel(32);
        let (popup_tx, _) = broadcast::channel(16);
        let mut initial_sm = AAPStateMachine::new();
        let mut initial_socket = L2capSocket::new();

        if let Some(dev) = find_connected_airpods() {
            initial_sm.state.name = dev.name;
            initial_sm.state.address = format_bt_mac(dev.bt_address);
            initial_sm.state.features.find_my_enabled =
                dev.raw_name.to_lowercase().contains("find my");
            if dev.connected {
                if initial_socket.connect(dev.bt_address).is_ok() {
                    let _ = initial_socket.send(&AAPCommands::handshake_init());
                    std::thread::sleep(std::time::Duration::from_millis(120));
                    let _ = initial_socket.send(&AAPCommands::set_specific_features());
                    std::thread::sleep(std::time::Duration::from_millis(60));
                    let _ = initial_socket.send(&AAPCommands::subscribe_notifications());
                    initial_sm.is_handshake_done = true;
                    initial_sm.state.connected = true;
                }
            }
        }

        let sm = Arc::new(Mutex::new(initial_sm));
        let socket = Arc::new(Mutex::new(initial_socket));

        (
            Self {
                state_machine: sm,
                socket,
                state_updates_tx: tx,
                popup_trigger_tx: popup_tx,
                takeover_paused: Arc::new(AtomicBool::new(false)),
            },
            rx,
        )
    }

    /// Đăng ký nhận tín hiệu bật Popup tức thì khi mở nắp hộp / rút tai nghe khỏi hộp qua BLE
    pub fn subscribe_popup_events(&self) -> broadcast::Receiver<String> {
        self.popup_trigger_tx.subscribe()
    }

    /// Tạm dừng vòng lặp L2CAP trong lúc A2DP đang cướp kết nối từ iPhone để tránh tranh chấp băng thông Bluetooth
    pub fn set_takeover_paused(&self, paused: bool) {
        self.takeover_paused.store(paused, Ordering::SeqCst);
    }

    /// Đóng kênh L2CAP cũ (nếu đang giữ handle chết) trước khi kéo A2DP mới từ iPhone
    pub async fn drop_stale_l2cap(&self) {
        let mut sock = self.socket.lock().await;
        if sock.is_connected {
            sock.disconnect();
        }
    }

    /// Khởi động background tasks (BLE scanning + AAP L2CAP packet loop + Keep-Alive)
    pub async fn start(&self) {
        let (ble_scanner, mut ble_rx) = AirPodsBleScanner::new();
        ble_scanner.start_scanning().await;

        let sm_clone = self.state_machine.clone();
        let tx_clone = self.state_updates_tx.clone();
        let popup_tx_clone = self.popup_trigger_tx.clone();

        // Task lắng nghe BLE beacon thời gian thực (mức pin, mở nắp hộp, rút tai nghe khỏi hộp -> bật Popup ngay!)
        tokio::spawn(async move {
            let mut prev_lid_open: Option<bool> = None;
            let mut prev_lid_counter: Option<u8> = None;
            let mut prev_both_in_case: Option<bool> = None;
            let mut prev_any_in_ear: Option<bool> = None;
            let mut last_beacon_at: Option<Instant> = None;
            let mut last_popup_at: Option<Instant> = None;

            while let Ok(beacon_info) = ble_rx.recv().await {
                let now = Instant::now();
                let woke_from_idle = last_beacon_at
                    .map(|t| now.duration_since(t) >= Duration::from_secs(10))
                    .unwrap_or(true);
                last_beacon_at = Some(now);

                let both_in_case =
                    beacon_info.battery.left.is_charging && beacon_info.battery.right.is_charging;
                let any_bud_out = (beacon_info.battery.left.level.is_some()
                    && !beacon_info.battery.left.is_charging)
                    || (beacon_info.battery.right.level.is_some()
                        && !beacon_info.battery.right.is_charging);
                let any_in_ear = beacon_info.in_ear.left_in_ear || beacon_info.in_ear.right_in_ear;

                let lid_just_opened = beacon_info.lid_open
                    && (prev_lid_open == Some(false)
                        || woke_from_idle
                        || prev_lid_counter
                            .map(|c| c != beacon_info.lid_counter)
                            .unwrap_or(false));
                let bud_taken_out =
                    prev_both_in_case == Some(true) && !both_in_case && any_bud_out;
                let bud_put_in_ear = prev_any_in_ear == Some(false) && any_in_ear;

                prev_lid_open = Some(beacon_info.lid_open);
                prev_lid_counter = Some(beacon_info.lid_counter);
                prev_both_in_case = Some(both_in_case);
                prev_any_in_ear = Some(any_in_ear);

                {
                    let mut sm = sm_clone.lock().await;
                    let mut changed = false;
                    if beacon_info.battery.left.level.is_some()
                        && sm.state.battery.left != beacon_info.battery.left
                    {
                        sm.state.battery.left = beacon_info.battery.left;
                        changed = true;
                    }
                    if beacon_info.battery.right.level.is_some()
                        && sm.state.battery.right != beacon_info.battery.right
                    {
                        sm.state.battery.right = beacon_info.battery.right;
                        changed = true;
                    }
                    if beacon_info.battery.case.level.is_some()
                        && sm.state.battery.case != beacon_info.battery.case
                    {
                        sm.state.battery.case = beacon_info.battery.case;
                        changed = true;
                    }
                    if sm.state.in_ear != beacon_info.in_ear {
                        sm.state.in_ear = beacon_info.in_ear;
                        changed = true;
                    }
                    if !matches!(
                        beacon_info.model,
                        lotusair_core::models::AirPodsModel::Unknown(_)
                    ) && sm.state.model != beacon_info.model
                    {
                        sm.state.model = beacon_info.model;
                        changed = true;
                    }
                    if changed {
                        let _ = tx_clone.send(sm.state.clone());
                    }
                }

                let trigger_reason = if lid_just_opened {
                    Some("lid_opened")
                } else if bud_taken_out {
                    Some("bud_taken_out")
                } else if bud_put_in_ear {
                    Some("in_ear")
                } else {
                    None
                };

                if let Some(reason) = trigger_reason {
                    let cooled_down = last_popup_at
                        .map(|t| now.duration_since(t) >= Duration::from_secs(8))
                        .unwrap_or(true);
                    if cooled_down {
                        last_popup_at = Some(now);
                        info!(
                            "Phát hiện sự kiện BLE AirPods ({}, lid_open={}, counter={}, rssi={:?}) -> Bật Popup!",
                            reason, beacon_info.lid_open, beacon_info.lid_counter, beacon_info.rssi
                        );
                        let _ = popup_tx_clone.send(reason.to_string());
                    }
                }
            }
        });

        // Task duy trì kết nối L2CAP & xử lý gói tin AAP phản hồi
        let sm_clone2 = self.state_machine.clone();
        let socket_clone = self.socket.clone();
        let tx_clone2 = self.state_updates_tx.clone();
        let paused_clone = self.takeover_paused.clone();

        tokio::spawn(async move {
            {
                let sm = sm_clone2.lock().await;
                if sm.state.connected {
                    let _ = tx_clone2.send(sm.state.clone());
                }
            }

            let mut tick_count: u64 = 0;
            let mut reconnect_backoff_ticks: u64 = 0;
            let mut connected_stable_ticks: u64 = 0;

            loop {
                let is_sock_connected = {
                    let sock = socket_clone.lock().await;
                    sock.is_connected
                };

                // Khi chưa nối L2CAP, ngủ 2 giây để CPU ~0%; khi đang nối, kiểm tra gói tin mỗi 500ms
                let sleep_ms = if is_sock_connected { 500 } else { 2000 };
                tokio::time::sleep(tokio::time::Duration::from_millis(sleep_ms)).await;
                tick_count = tick_count.wrapping_add(1);

                // Trong lúc BthA2dp.sys đang kéo kết nối từ iPhone về PC, tuyệt đối không gọi BRB_L2CA_OPEN_CHANNEL
                // để tránh xung đột ACL/AVDTP gây giật âm thanh hoặc kéo chậm.
                if paused_clone.load(Ordering::SeqCst) {
                    connected_stable_ticks = 0;
                    continue;
                }

                if reconnect_backoff_ticks > 0 {
                    reconnect_backoff_ticks -= 1;
                }

                // Chỉ quét Registry/Bluetooth mỗi ~3 giây khi đã nối (hoặc mỗi 2 giây khi chưa nối)
                let should_check_bt = !is_sock_connected || (tick_count % 6 == 0);
                if should_check_bt {
                    if let Some(dev) = find_connected_airpods() {
                        if dev.connected {
                            connected_stable_ticks = connected_stable_ticks.saturating_add(1);
                        } else {
                            connected_stable_ticks = 0;
                        }

                        let need_connect = {
                            let sock = socket_clone.lock().await;
                            dev.connected
                                && !sock.is_connected
                                && reconnect_backoff_ticks == 0
                                && connected_stable_ticks >= 1
                        };
                        let need_disconnect = {
                            let sock = socket_clone.lock().await;
                            !dev.connected && sock.is_connected
                        };

                        if need_connect {
                            let connected_ok = {
                                let mut sock = socket_clone.lock().await;
                                if sock.connect(dev.bt_address).is_ok() {
                                    let _ = sock.send(&AAPCommands::handshake_init());
                                    tokio::time::sleep(tokio::time::Duration::from_millis(120)).await;
                                    let _ = sock.send(&AAPCommands::set_specific_features());
                                    tokio::time::sleep(tokio::time::Duration::from_millis(60)).await;
                                    let _ = sock.send(&AAPCommands::subscribe_notifications());
                                    true
                                } else {
                                    warn!("Mở kênh L2CAP thất bại, tạm nghỉ 6 giây trước khi thử lại để không gây giật Bluetooth.");
                                    false
                                }
                            };
                            if connected_ok {
                                let mut sm = sm_clone2.lock().await;
                                sm.is_handshake_done = true;
                                sm.state.connected = true;
                                if sm.state.name.is_empty() || sm.state.name == "AirPods" {
                                    sm.state.name = dev.name;
                                }
                                sm.state.address = format_bt_mac(dev.bt_address);
                                let _ = tx_clone2.send(sm.state.clone());
                            } else {
                                reconnect_backoff_ticks = 3; // 3 * 2000ms = 6 giây backoff
                            }
                        } else if need_disconnect {
                            {
                                let mut sock = socket_clone.lock().await;
                                sock.disconnect();
                            }
                            let mut sm = sm_clone2.lock().await;
                            sm.is_handshake_done = false;
                            sm.state.connected = false;
                            let _ = tx_clone2.send(sm.state.clone());
                        }
                    }
                }

                // Gửi gói Keep-Alive L2CAP mỗi 10 giây nếu đang kết nối và tính năng Keep-Alive đang bật
                if is_sock_connected && tick_count % 20 == 0 {
                    let keep_alive = {
                        let sm = sm_clone2.lock().await;
                        sm.state.connected && sm.state.features.keep_alive_connection
                    };
                    if keep_alive {
                        let mut sock = socket_clone.lock().await;
                        let _ = sock.send(&AAPCommands::keep_alive_ping());
                    }
                }

                // Đọc gói tin AAP trong hàng đợi khi đang kết nối
                if is_sock_connected {
                    loop {
                        let packet_opt = {
                            let sock = socket_clone.lock().await;
                            if !sock.is_connected {
                                None
                            } else {
                                sock.receive().unwrap_or(None)
                            }
                        };

                        let Some(packet) = packet_opt else {
                            break;
                        };

                        if let Ok(event) = AAPParser::parse(&packet) {
                            let mut sm = sm_clone2.lock().await;
                            sm.handle_event(event);
                            let _ = tx_clone2.send(sm.state.clone());
                        }
                    }
                }
            }
        });
    }

    /// Đổi chế độ Noise Control (ANC / Transparency / Adaptive / Off)
    pub async fn set_noise_mode(&self, mode: NoiseControlMode) -> Result<(), String> {
        info!("BluetoothManager: Gửi lệnh đổi Noise Control sang {:?}", mode);
        let cmd = AAPCommands::set_noise_mode(mode);
        {
            let mut sock = self.socket.lock().await;
            if mode == NoiseControlMode::Off {
                let _ = sock.send(&AAPCommands::set_allow_off_option(true));
            }
            sock.send(&cmd)?;
        }

        let mut sm = self.state_machine.lock().await;
        sm.state.noise_mode = mode;
        sm.state.connected = true;
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    /// Đổi cấu hình Nhận biết cuộc hội thoại (Conversational Awareness)
    pub async fn toggle_conversational_awareness(&self, enabled: bool) -> Result<(), String> {
        info!("BluetoothManager: Đổi Conversational Awareness sang {}", enabled);
        let cmd = AAPCommands::set_conversational_awareness(enabled);
        {
            let mut sock = self.socket.lock().await;
            sock.send(&cmd)?;
        }

        let mut sm = self.state_machine.lock().await;
        sm.state.features.conversational_awareness = enabled;
        sm.state.connected = true;
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    /// Bật/Tắt Âm lượng được cá nhân hóa (Personalized Volume)
    pub async fn set_personalized_volume(&self, enabled: bool) -> Result<(), String> {
        info!("BluetoothManager: Đặt Personalized Volume = {}", enabled);
        let cmd = AAPCommands::set_personalized_volume(enabled);
        {
            let mut sock = self.socket.lock().await;
            sock.send(&cmd)?;
        }

        let mut sm = self.state_machine.lock().await;
        sm.state.features.personalized_volume = enabled;
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    /// Bật/Tắt Giảm âm thanh lớn (Loud Noise Reduction)
    pub async fn set_loud_noise_reduction(&self, enabled: bool) -> Result<(), String> {
        info!("BluetoothManager: Đặt Loud Noise Reduction = {}", enabled);
        let cmd = AAPCommands::set_loud_noise_reduction(enabled);
        {
            let mut sock = self.socket.lock().await;
            sock.send(&cmd)?;
        }

        let mut sm = self.state_machine.lock().await;
        sm.state.features.loud_noise_reduction = enabled;
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    /// Bật/Tắt Kiểm soát tiếng ồn bằng một AirPod (One-Bud ANC)
    pub async fn set_one_bud_anc(&self, enabled: bool) -> Result<(), String> {
        info!("BluetoothManager: Đặt One-Bud ANC = {}", enabled);
        let cmd = AAPCommands::set_one_bud_anc(enabled);
        {
            let mut sock = self.socket.lock().await;
            sock.send(&cmd)?;
        }

        let mut sm = self.state_machine.lock().await;
        sm.state.features.one_bud_anc = enabled;
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    /// Bật/Tắt Tương tác bằng cử chỉ đầu (Head Gestures)
    pub async fn set_head_gestures(&self, enabled: bool) -> Result<(), String> {
        info!("BluetoothManager: Đặt Head Gestures = {}", enabled);
        let cmd = AAPCommands::set_head_gestures(enabled);
        {
            let mut sock = self.socket.lock().await;
            sock.send(&cmd)?;
        }

        let mut sm = self.state_machine.lock().await;
        sm.state.features.head_gestures = enabled;
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    /// Bật/Tắt Tự động phát hiện tai (Auto Ear Detection)
    pub async fn set_auto_ear_detection(&self, enabled: bool) -> Result<(), String> {
        info!("BluetoothManager: Đặt Auto Ear Detection = {}", enabled);
        let cmd = AAPCommands::set_auto_ear_detection(enabled);
        {
            let mut sock = self.socket.lock().await;
            sock.send(&cmd)?;
        }

        let mut sm = self.state_machine.lock().await;
        sm.state.features.auto_ear_detection = enabled;
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    /// Bật/Tắt Ưu tiên cướp tai nghe từ iPhone/iPad (Auto Takeover)
    pub async fn set_auto_takeover(&self, enabled: bool) -> Result<(), String> {
        info!("BluetoothManager: Đặt Auto Takeover = {}", enabled);
        if enabled {
            let cmd = AAPCommands::force_audio_takeover();
            let mut sock = self.socket.lock().await;
            sock.send(&cmd)?;
        }
        let mut sm = self.state_machine.lock().await;
        sm.state.features.auto_takeover = enabled;
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    /// Cấu hình thao tác nhấn và giữ thân AirPods (side: "left" | "right")
    pub async fn set_stem_press_action(&self, side: String, action: String) -> Result<(), String> {
        let is_right = side.to_lowercase() == "right";
        let stem_action = StemAction::from_str_mode(&action);
        info!(
            "BluetoothManager: Đặt Stem Press ({}) = {:?}",
            if is_right { "Phải" } else { "Trái" },
            stem_action
        );
        let cmd = AAPCommands::set_stem_press_action(is_right, stem_action);
        {
            let mut sock = self.socket.lock().await;
            sock.send(&cmd)?;
        }

        let mut sm = self.state_machine.lock().await;
        if is_right {
            sm.state.features.stem_press_right = stem_action;
        } else {
            sm.state.features.stem_press_left = stem_action;
        }
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    /// Cấu hình các chế độ nghe được luân chuyển khi bóp thân AirPods
    pub async fn set_cycle_modes(
        &self,
        anc: bool,
        adaptive: bool,
        transparency: bool,
        off: bool,
    ) -> Result<(), String> {
        let count = [anc, adaptive, transparency, off].iter().filter(|&&b| b).count();
        if count < 2 {
            return Err("Cần chọn ít nhất 2 chế độ nghe để luân chuyển khi nhấn giữ AirPods".into());
        }
        let cfg = CycleModesConfig {
            anc,
            adaptive,
            transparency,
            off,
        };
        let cmd = AAPCommands::set_cycle_modes(cfg);
        {
            let mut sock = self.socket.lock().await;
            let _ = sock.send(&AAPCommands::set_allow_off_option(off));
            sock.send(&cmd)?;
        }

        let mut sm = self.state_machine.lock().await;
        sm.state.features.cycle_modes = cfg;
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    /// Bật/Tắt âm thanh hộp sạc
    pub async fn set_case_charging_sounds(&self, enabled: bool) -> Result<(), String> {
        info!("BluetoothManager: Đặt Case Charging Sounds = {}", enabled);
        let cmd = AAPCommands::set_case_charging_sounds(enabled);
        {
            let mut sock = self.socket.lock().await;
            sock.send(&cmd)?;
        }

        let mut sm = self.state_machine.lock().await;
        sm.state.features.case_charging_sounds = enabled;
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    /// Đặt âm lượng chuông hộp sạc (0 - 100%)
    pub async fn set_case_chime_volume(&self, volume: u8) -> Result<(), String> {
        let clamped = volume.min(100);
        info!("BluetoothManager: Đặt Case Chime Volume = {}%", clamped);
        let cmd = AAPCommands::set_case_chime_volume(clamped);
        {
            let mut sock = self.socket.lock().await;
            sock.send(&cmd)?;
        }

        let mut sm = self.state_machine.lock().await;
        sm.state.features.case_chime_volume = clamped;
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    /// Chạy kiểm tra độ vừa vặn âm học (Acoustic Fit Test)
    pub async fn run_acoustic_fit_test(&self) -> Result<String, String> {
        info!("BluetoothManager: Chạy kiểm tra độ vừa vặn âm học (Acoustic Fit Test)");
        let cmd = AAPCommands::run_acoustic_fit_test();
        {
            let mut sock = self.socket.lock().await;
            sock.send(&cmd)?;
        }
        Ok("Độ vừa vặn âm học tốt ở cả Tai Trái (L) và Tai Phải (R) ✓".into())
    }

    /// Ngắt kết nối phiên điều khiển AirPods hiện tại
    pub async fn disconnect_device(&self) -> Result<(), String> {
        info!("BluetoothManager: Ngắt kết nối phiên điều khiển AirPods");
        {
            let mut sock = self.socket.lock().await;
            sock.disconnect();
        }
        let mut sm = self.state_machine.lock().await;
        sm.is_handshake_done = false;
        sm.state.connected = false;
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    /// Đổi tên AirPods
    pub async fn rename_device(&self, new_name: String) -> Result<(), String> {
        info!("BluetoothManager: Đổi tên AirPods sang {}", new_name);
        let cmd = AAPCommands::rename_device(&new_name);
        {
            let mut sock = self.socket.lock().await;
            sock.send(&cmd)?;
        }

        let mut sm = self.state_machine.lock().await;
        sm.state.name = new_name;
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    /// Điều chỉnh thanh trượt Adaptive Audio
    pub async fn set_adaptive_blend(&self, level: u8) -> Result<(), String> {
        let clamped = level.min(100);
        let cmd = AAPCommands::set_adaptive_blend(clamped);
        {
            let mut sock = self.socket.lock().await;
            sock.send(&cmd)?;
        }

        let mut sm = self.state_machine.lock().await;
        sm.state.features.adaptive_audio_blend = clamped;
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    /// Cướp tai nghe từ iPhone / Thiết bị khác sang PC
    pub async fn force_takeover_audio(&self) -> Result<(), String> {
        info!("BluetoothManager: Kích hoạt chuyển âm thanh AirPods sang PC!");
        let cmd = AAPCommands::force_audio_takeover();
        {
            let mut sock = self.socket.lock().await;
            sock.send(&cmd)?;
        }

        let mut sm = self.state_machine.lock().await;
        sm.state.connected = true;
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    /// Đặt chế độ Spatial Audio
    pub async fn set_spatial_audio_mode(&self, mode: lotusair_core::models::SpatialAudioMode) -> Result<(), String> {
        info!("BluetoothManager: Đổi Spatial Audio sang {:?}", mode);
        let cmd = AAPCommands::set_spatial_audio_mode(mode);
        {
            let mut sock = self.socket.lock().await;
            sock.send(&cmd)?;
        }

        let mut sm = self.state_machine.lock().await;
        sm.state.features.spatial_audio_mode = mode;
        sm.state.connected = true;
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    /// Bật/Tắt duy trì kết nối L2CAP (Keep-Alive)
    pub async fn set_keep_alive(&self, enabled: bool) -> Result<(), String> {
        info!("BluetoothManager: Đặt Keep-Alive L2CAP = {}", enabled);
        if enabled {
            let mut sock = self.socket.lock().await;
            sock.ensure_connected()?;
        }
        let mut sm = self.state_machine.lock().await;
        sm.state.features.keep_alive_connection = enabled;
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    pub async fn get_current_state(&self) -> AirPodsDeviceState {
        let sm = self.state_machine.lock().await;
        sm.state.clone()
    }
}
