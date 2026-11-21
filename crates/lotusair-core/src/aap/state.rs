use crate::aap::opcodes::*;
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
            AAPEvent::ControlCommandChanged { id, data } => {
                let d0 = data[0];
                let bool_opt = match d0 {
                    0x01 => Some(true),
                    0x02 => Some(false),
                    _ => None,
                };
                match id {
                    CTRL_ADAPTIVE_VOLUME_CONFIG => {
                        if let Some(v) = bool_opt {
                            self.state.features.personalized_volume = v;
                        }
                    }
                    CTRL_AUTO_ANC_STRENGTH => {
                        if d0 <= 100 {
                            self.state.features.adaptive_audio_blend = d0;
                        }
                    }
                    CTRL_ONE_BUD_ANC_MODE => {
                        if let Some(v) = bool_opt {
                            self.state.features.one_bud_anc = v;
                        }
                    }
                    CTRL_CALL_MANAGEMENT_CONFIG => {
                        if let Some(v) = bool_opt {
                            self.state.features.head_gestures = v;
                        }
                    }
                    CTRL_EAR_DETECTION_CONFIG => {
                        if let Some(v) = bool_opt {
                            self.state.features.auto_ear_detection = v;
                        }
                    }
                    CTRL_IN_CASE_TONE_CONFIG => {
                        if let Some(v) = bool_opt {
                            self.state.features.case_charging_sounds = v;
                        }
                    }
                    CTRL_CHIME_VOLUME | CTRL_IN_CASE_TONE_VOLUME => {
                        if d0 <= 100 {
                            self.state.features.case_chime_volume = d0;
                        }
                    }
                    CTRL_PPE_TOGGLE_CONFIG => {
                        if let Some(v) = bool_opt {
                            self.state.features.loud_noise_reduction = v;
                        }
                    }
                    _ => {
                        debug!("Control command update id=0x{:02X} data={:02X?}", id, data);
                    }
                }
            }
            AAPEvent::Unknown { opcode, data } => {
                debug!("AAP event không xác định: 0x{:04X}, len: {}", opcode, data.len());
            }
        }
    }
}
