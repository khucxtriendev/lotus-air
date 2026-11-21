use lotusair_core::aap::{AAPCommands, AAPParser, AAPStateMachine};
use lotusair_core::models::{AirPodsDeviceState, NoiseControlMode};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::info;
use crate::ble::AirPodsBleScanner;
use crate::l2cap::L2capSocket;

pub struct BluetoothManager {
    pub state_machine: Arc<Mutex<AAPStateMachine>>,
    socket: Arc<Mutex<L2capSocket>>,
    pub state_updates_tx: broadcast::Sender<AirPodsDeviceState>,
}

impl BluetoothManager {
    pub fn new() -> (Self, broadcast::Receiver<AirPodsDeviceState>) {
        let (tx, rx) = broadcast::channel(32);
        let sm = Arc::new(Mutex::new(AAPStateMachine::new()));
        let socket = Arc::new(Mutex::new(L2capSocket::new()));

        (
            Self {
                state_machine: sm,
                socket,
                state_updates_tx: tx,
            },
            rx,
        )
    }

    /// Khởi động background tasks (BLE scanning + AAP packet loop)
    pub async fn start(&self) {
        let (ble_scanner, mut ble_rx) = AirPodsBleScanner::new();
        ble_scanner.start_scanning().await;

        let sm_clone = self.state_machine.clone();
        let tx_clone = self.state_updates_tx.clone();

        // Task lắng nghe BLE beacon (mức pin & trạng thái nắp)
        tokio::spawn(async move {
            while let Ok(beacon_info) = ble_rx.recv().await {
                let mut sm = sm_clone.lock().await;
                sm.state.battery = beacon_info.battery;
                sm.state.model = beacon_info.model;
                let _ = tx_clone.send(sm.state.clone());
            }
        });

        // Tự động kết nối mẫu / L2CAP loop
        let sm_clone2 = self.state_machine.clone();
        let socket_clone = self.socket.clone();
        let tx_clone2 = self.state_updates_tx.clone();

        tokio::spawn(async move {
            // Giả lập hoặc khởi động kết nối L2CAP
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            {
                let mut sock = socket_clone.lock().await;
                let _ = sock.connect(0x1234567890AB);
                let _ = sock.send(&AAPCommands::handshake_init());
            }

            {
                let mut sm = sm_clone2.lock().await;
                sm.state.connected = true;
                sm.state.name = "AirPods 4 ANC".to_string();
                let _ = tx_clone2.send(sm.state.clone());
            }

            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                let packet_opt = {
                    let sock = socket_clone.lock().await;
                    sock.receive().unwrap_or(None)
                };

                if let Some(packet) = packet_opt {
                    if let Ok(event) = AAPParser::parse(&packet) {
                        let mut sm = sm_clone2.lock().await;
                        sm.handle_event(event);
                        let _ = tx_clone2.send(sm.state.clone());
                    }
                }
            }
        });
    }

    /// Đổi chế độ Noise Control (ANC / Transparency / Adaptive / Off)
    pub async fn set_noise_mode(&self, mode: NoiseControlMode) -> Result<(), String> {
        info!("BluetoothManager: Gửi lệnh đổi Noise Control sang {:?}", mode);
        let cmd = AAPCommands::set_noise_mode(mode);
        let sock = self.socket.lock().await;
        sock.send(&cmd)?;

        let mut sm = self.state_machine.lock().await;
        sm.state.noise_mode = mode;
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    /// Đổi cấu hình tính năng
    pub async fn toggle_conversational_awareness(&self, enabled: bool) -> Result<(), String> {
        let cmd = AAPCommands::set_conversational_awareness(enabled);
        let sock = self.socket.lock().await;
        sock.send(&cmd)?;

        let mut sm = self.state_machine.lock().await;
        sm.state.features.conversational_awareness = enabled;
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    /// Đổi tên AirPods
    pub async fn rename_device(&self, new_name: String) -> Result<(), String> {
        info!("BluetoothManager: Đổi tên AirPods sang {}", new_name);
        let cmd = AAPCommands::rename_device(&new_name);
        let sock = self.socket.lock().await;
        sock.send(&cmd)?;

        let mut sm = self.state_machine.lock().await;
        sm.state.name = new_name;
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    /// Điều chỉnh thanh trượt Adaptive Audio
    pub async fn set_adaptive_blend(&self, level: u8) -> Result<(), String> {
        let cmd = AAPCommands::set_adaptive_blend(level);
        let sock = self.socket.lock().await;
        sock.send(&cmd)?;

        let mut sm = self.state_machine.lock().await;
        sm.state.features.adaptive_audio_blend = level;
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    /// Cướp tai nghe từ iPhone / Thiết bị khác sang PC
    pub async fn force_takeover_audio(&self) -> Result<(), String> {
        info!("BluetoothManager: Kích hoạt cướp tai nghe từ iPhone sang PC!");
        let cmd = AAPCommands::force_audio_takeover();
        let sock = self.socket.lock().await;
        sock.send(&cmd)?;

        let mut sm = self.state_machine.lock().await;
        sm.state.connected = true;
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    /// Đặt chế độ Spatial Audio
    pub async fn set_spatial_audio_mode(&self, mode: lotusair_core::models::SpatialAudioMode) -> Result<(), String> {
        let cmd = AAPCommands::set_spatial_audio_mode(mode);
        let sock = self.socket.lock().await;
        sock.send(&cmd)?;

        let mut sm = self.state_machine.lock().await;
        sm.state.features.spatial_audio_mode = mode;
        let _ = self.state_updates_tx.send(sm.state.clone());
        Ok(())
    }

    /// Bật/Tắt duy trì kết nối ấm (Keep-Alive)
    pub async fn set_keep_alive(&self, enabled: bool) -> Result<(), String> {
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
