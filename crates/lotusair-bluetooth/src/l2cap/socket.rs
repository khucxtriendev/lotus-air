use lotusair_core::aap::{AAPPacket, AAP_L2CAP_PSM};
use lotusair_driver::LotusAirDriverBridge;
use tracing::{debug, info, warn};

pub enum L2capBackend {
    Driver(LotusAirDriverBridge),
    #[cfg(windows)]
    WinSock,
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
            info!("Sử dụng KMDF Driver Backend cho L2CAP PSM 0x1001");
            L2capBackend::Driver(driver_bridge)
        } else {
            warn!("Driver không khả dụng, sử dụng chế độ Mock/Direct L2CAP");
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
            #[cfg(windows)]
            L2capBackend::WinSock => {
                self.is_connected = true;
                Ok(())
            }
            L2capBackend::Mock => {
                debug!("Giả lập kết nối L2CAP PSM 0x1001 thành công!");
                self.is_connected = true;
                Ok(())
            }
        }
    }

    /// Gửi một gói tin AAP tới AirPods
    pub fn send(&self, packet: &AAPPacket) -> Result<(), String> {
        let bytes = packet.to_bytes();
        match &self.backend {
            L2capBackend::Driver(bridge) => {
                bridge.send_packet(&bytes).map_err(|e| format!("{:?}", e))
            }
            #[cfg(windows)]
            L2capBackend::WinSock => Ok(()),
            L2capBackend::Mock => {
                debug!("Mock send AAP packet: Opcode=0x{:04X}", packet.opcode);
                Ok(())
            }
        }
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
            #[cfg(windows)]
            L2capBackend::WinSock => Ok(None),
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
