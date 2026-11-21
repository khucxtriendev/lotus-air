use lotusair_core::aap::{AAPCommands, AAPPacket, AAP_L2CAP_PSM};
use lotusair_driver::{find_connected_airpods, LotusAirDriverBridge};
use tracing::{debug, info, warn};

pub enum L2capBackend {
    Driver(LotusAirDriverBridge),
    Mock,
}

pub struct L2capSocket {
    backend: L2capBackend,
    pub is_connected: bool,
}

impl L2capSocket {
    pub fn new() -> Self {
        let mut driver_bridge = LotusAirDriverBridge::new();
        let backend = if driver_bridge.open().is_ok() {
            info!("Sử dụng LotusAir Driver Bridge cho L2CAP PSM 0x1001");
            L2capBackend::Driver(driver_bridge)
        } else {
            warn!("Driver Bridge không khả dụng, sử dụng chế độ Mock");
            L2capBackend::Mock
        };

        Self {
            backend,
            is_connected: false,
        }
    }

    /// Kết nối tới địa chỉ MAC Bluetooth của AirPods
    pub fn connect(&mut self, mac_address: u64) -> Result<(), String> {
        info!("Bắt đầu kết nối L2CAP tới AirPods: 0x{:012X}", mac_address);
        match &mut self.backend {
            L2capBackend::Driver(bridge) => {
                bridge
                    .connect_l2cap(mac_address, AAP_L2CAP_PSM)
                    .map_err(|e| format!("Driver connect error: {:?}", e))?;
                self.is_connected = true;
                Ok(())
            }
            L2capBackend::Mock => {
                debug!("Giả lập kết nối L2CAP PSM 0x1001");
                self.is_connected = true;
                Ok(())
            }
        }
    }

    /// Đảm bảo kết nối L2CAP đã mở tới AirPods đang kết nối trên Windows
    pub fn ensure_connected(&mut self) -> Result<(), String> {
        if self.is_connected {
            return Ok(());
        }
        if let Some(dev) = find_connected_airpods().filter(|d| d.connected) {
            self.connect(dev.bt_address)
        } else {
            Err("Chưa tìm thấy tai nghe AirPods đang kết nối với Windows Bluetooth".into())
        }
    }

    /// Gửi một gói tin AAP tới AirPods (tự động kết nối lại nếu kênh L2CAP bị ngắt sau khi đổi tên/ngủ)
    pub fn send(&mut self, packet: &AAPPacket) -> Result<(), String> {
        if !self.is_connected {
            self.ensure_connected()?;
        }
        let bytes = packet.to_bytes();
        let first_err = match &self.backend {
            L2capBackend::Driver(bridge) => match bridge.send_packet(&bytes) {
                Ok(()) => return Ok(()),
                Err(e) => format!("{:?}", e),
            },
            L2capBackend::Mock => {
                debug!("Mock send AAP packet: Opcode=0x{:04X}", packet.opcode);
                return Ok(());
            }
        };

        // Tự động mở lại kênh L2CAP + Handshake và thử gửi lại 1 lần
        warn!("Gửi AAP lỗi ({}), đang tự động kết nối lại L2CAP...", first_err);
        self.is_connected = false;
        if let Some(dev) = find_connected_airpods().filter(|d| d.connected) {
            if let L2capBackend::Driver(bridge) = &mut self.backend {
                let _ = bridge.disconnect();
                let _ = bridge.open();
            }
            self.connect(dev.bt_address)?;
            if let L2capBackend::Driver(bridge) = &self.backend {
                let _ = bridge.send_packet(&AAPCommands::handshake_init().to_bytes());
                let _ = bridge.send_packet(&AAPCommands::set_specific_features().to_bytes());
                let _ = bridge.send_packet(&AAPCommands::subscribe_notifications().to_bytes());
                return bridge.send_packet(&bytes).map_err(|e| format!("{:?}", e));
            }
        }
        Err(first_err)
    }

    /// Nhận gói tin phản hồi từ AirPods
    pub fn receive(&self) -> Result<Option<AAPPacket>, String> {
        match &self.backend {
            L2capBackend::Driver(bridge) => {
                let bytes = bridge.receive_packet().map_err(|e| format!("{:?}", e))?;
                if bytes.is_empty() {
                    return Ok(None);
                }
                let packet = AAPPacket::from_bytes(&bytes).map_err(|e| format!("{:?}", e))?;
                Ok(Some(packet))
            }
            L2capBackend::Mock => Ok(None),
        }
    }

    pub fn disconnect(&mut self) {
        self.is_connected = false;
        if let L2capBackend::Driver(bridge) = &mut self.backend {
            let _ = bridge.disconnect();
        }
    }
}
