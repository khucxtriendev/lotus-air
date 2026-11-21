use crate::aap::opcodes::*;
use crate::aap::packet::AAPPacket;
use crate::error::{CoreError, Result};
use crate::models::{AirPodsBatteryStatus, InEarStatus, NoiseControlMode, SingleBattery};

#[derive(Debug, Clone, PartialEq)]
pub enum AAPEvent {
    HandshakeAck,
    FeaturesAck(Vec<u8>),
    NoiseModeChanged(NoiseControlMode),
    BatteryReport(AirPodsBatteryStatus),
    InEarChanged(InEarStatus),
    ConversationalAwarenessChanged(bool),
    ControlCommandChanged { id: u8, data: [u8; 4] },
    Unknown { opcode: u16, data: Vec<u8> },
}

pub struct AAPParser;

impl AAPParser {
    pub fn parse(packet: &AAPPacket) -> Result<AAPEvent> {
        match packet.opcode {
            OPCODE_HANDSHAKE_ACK => Ok(AAPEvent::HandshakeAck),
            OPCODE_FEATURES_ACK => Ok(AAPEvent::FeaturesAck(packet.payload.clone())),
            OPCODE_CONTROL_COMMAND => {
                if packet.payload.is_empty() {
                    return Err(CoreError::PacketParseError(
                        "Empty control command payload".into(),
                    ));
                }
                // Hỗ trợ cả định dạng chuẩn AACP [id, d0, d1, d2, d3] và định dạng rút gọn [mode]
                if packet.payload.len() == 1 {
                    let mode = NoiseControlMode::from_u8(packet.payload[0])?;
                    return Ok(AAPEvent::NoiseModeChanged(mode));
                }

                let id = packet.payload[0];
                let d0 = *packet.payload.get(1).unwrap_or(&0);
                let d1 = *packet.payload.get(2).unwrap_or(&0);
                let d2 = *packet.payload.get(3).unwrap_or(&0);
                let d3 = *packet.payload.get(4).unwrap_or(&0);

                match id {
                    CTRL_LISTENING_MODE => {
                        let mode = NoiseControlMode::from_u8(d0)?;
                        Ok(AAPEvent::NoiseModeChanged(mode))
                    }
                    CTRL_CONVERSATION_DETECT_CONFIG => {
                        if d0 == 0x01 || d0 == 0x02 {
                            Ok(AAPEvent::ConversationalAwarenessChanged(d0 == 0x01))
                        } else {
                            Ok(AAPEvent::ControlCommandChanged {
                                id,
                                data: [d0, d1, d2, d3],
                            })
                        }
                    }
                    _ => Ok(AAPEvent::ControlCommandChanged {
                        id,
                        data: [d0, d1, d2, d3],
                    }),
                }
            }
            OPCODE_BATTERY_REPORT => {
                if packet.payload.is_empty() {
                    return Err(CoreError::PacketParseError(
                        "Battery payload too short".into(),
                    ));
                }

                // Định dạng chuẩn Apple AACP: [count] ([component] [0x01] [level] [status] [0x01])*
                // component: 0x02 = Right, 0x04 = Left, 0x08 = Case
                // status: 0x01 = Charging, 0x02 = Discharging, 0x04 = Disconnected
                let count = packet.payload[0] as usize;
                if count <= 3 && packet.payload.len() >= 1 + count * 5 {
                    let mut battery = AirPodsBatteryStatus::default();
                    for i in 0..count {
                        let base = 1 + i * 5;
                        let comp = packet.payload[base];
                        let level = packet.payload[base + 2];
                        let status = packet.payload[base + 3];
                        let single = SingleBattery {
                            level: if status != 0x04 && level <= 100 {
                                Some(level)
                            } else {
                                None
                            },
                            is_charging: status == 0x01,
                        };
                        match comp {
                            0x04 => battery.left = single,
                            0x02 => battery.right = single,
                            0x08 => battery.case = single,
                            _ => {}
                        }
                    }
                    return Ok(AAPEvent::BatteryReport(battery));
                }

                // Fallback định dạng rút gọn 3-4 bytes
                if packet.payload.len() < 3 {
                    return Err(CoreError::PacketParseError(
                        "Battery payload too short".into(),
                    ));
                }
                let left_val = packet.payload[0];
                let right_val = packet.payload[1];
                let case_val = packet.payload[2];

                let left_charging = packet.payload.get(3).map(|&b| (b & 0x01) != 0).unwrap_or(false);
                let right_charging = packet.payload.get(3).map(|&b| (b & 0x02) != 0).unwrap_or(false);
                let case_charging = packet.payload.get(3).map(|&b| (b & 0x04) != 0).unwrap_or(false);

                let battery = AirPodsBatteryStatus {
                    left: SingleBattery {
                        level: if left_val <= 100 { Some(left_val) } else { None },
                        is_charging: left_charging,
                    },
                    right: SingleBattery {
                        level: if right_val <= 100 { Some(right_val) } else { None },
                        is_charging: right_charging,
                    },
                    case: SingleBattery {
                        level: if case_val <= 100 { Some(case_val) } else { None },
                        is_charging: case_charging,
                    },
                };
                Ok(AAPEvent::BatteryReport(battery))
            }
            OPCODE_IN_EAR_STATUS => {
                if packet.payload.is_empty() {
                    return Err(CoreError::PacketParseError("In-ear payload empty".into()));
                }
                // Định dạng chuẩn Apple AACP (2 bytes): [primary_status, secondary_status]
                // 0x00 = In Ear, 0x01 = Out of Ear, 0x02 = In Case
                if packet.payload.len() >= 2 && packet.payload[0] <= 0x02 && packet.payload[1] <= 0x02 {
                    let primary_in_ear = packet.payload[0] == 0x00;
                    let secondary_in_ear = packet.payload[1] == 0x00;
                    return Ok(AAPEvent::InEarChanged(InEarStatus {
                        left_in_ear: primary_in_ear,
                        right_in_ear: secondary_in_ear,
                    }));
                }
                let status_byte = packet.payload[0];
                let left_in_ear = (status_byte & 0x01) != 0;
                let right_in_ear = (status_byte & 0x02) != 0;
                Ok(AAPEvent::InEarChanged(InEarStatus {
                    left_in_ear,
                    right_in_ear,
                }))
            }
            OPCODE_CONVERSATIONAL_AWARENESS_DATA => {
                // 0x004B là sự kiện người dùng đang nói / ngừng nói (KHÔNG phải bật/tắt cài đặt)
                Ok(AAPEvent::Unknown {
                    opcode: OPCODE_CONVERSATIONAL_AWARENESS_DATA,
                    data: packet.payload.clone(),
                })
            }
            other => Ok(AAPEvent::Unknown {
                opcode: other,
                data: packet.payload.clone(),
            }),
        }
    }
}
