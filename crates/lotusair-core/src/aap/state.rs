use crate::aap::parser::AAPEvent;
use crate::models::AirPodsDeviceState;
use tracing::{debug, info};

pub struct AAPStateMachine {
    pub is_handshake_done: bool,
    pub state: AirPodsDeviceState,
}

impl AAPStateMachine {
    pub fn new() -> Self {
        Self {
            is_handshake_done: false,
            state: AirPodsDeviceState::default(),
        }
    }

    /// Cập nhật state máy trạng thái khi có gói tin/sự kiện AAP từ AirPods
    pub fn handle_event(&mut self, event: AAPEvent) {
        match event {
            AAPEvent::HandshakeAck => {
                self.is_handshake_done = true;
                self.state.connected = true;
                info!("AAP Handshake thành công với AirPods!");
            }
            AAPEvent::FeaturesAck(_data) => {
                debug!("Nhận Features ACK từ AirPods");
            }
            AAPEvent::NoiseModeChanged(mode) => {
                info!("Đổi chế độ Noise Control sang: {:?}", mode);
                self.state.noise_mode = mode;
            }
            AAPEvent::BatteryReport(battery) => {
                debug!("Cập nhật mức pin: {:?}", battery);
                self.state.battery = battery;
            }
            AAPEvent::InEarChanged(in_ear) => {
                debug!("Cập nhật trạng thái đeo tai: {:?}", in_ear);
                self.state.in_ear = in_ear;
            }
            AAPEvent::ConversationalAwarenessChanged(enabled) => {
                debug!("Conversational Awareness: {}", enabled);
                self.state.features.conversational_awareness = enabled;
            }
            AAPEvent::Unknown { opcode, data } => {
                debug!("AAP event không xác định: 0x{:04X}, len: {}", opcode, data.len());
            }
        }
    }
}
