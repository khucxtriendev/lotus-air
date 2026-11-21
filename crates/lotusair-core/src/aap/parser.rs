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
    Unknown { opcode: u16, data: Vec<u8> },
}

pub struct AAPParser;

impl AAPParser {
    pub fn parse(packet: &AAPPacket) -> Result<AAPEvent> {
        match packet.opcode {
            OPCODE_HANDSHAKE_ACK => Ok(AAPEvent::HandshakeAck),
            OPCODE_FEATURES_ACK => Ok(AAPEvent::FeaturesAck(packet.payload.clone())),
            OPCODE_LISTEN_MODE_CONTROL => {
                if packet.payload.is_empty() {
                    return Err(CoreError::PacketParseError("Empty noise mode payload".into()));
                }
                let mode = NoiseControlMode::from_u8(packet.payload[0])?;
                Ok(AAPEvent::NoiseModeChanged(mode))
            }
            OPCODE_BATTERY_REPORT => {
                // Pin trả về theo AAP format: Left, Right, Case và cờ charging
                if packet.payload.len() < 3 {
                    return Err(CoreError::PacketParseError("Battery payload too short".into()));
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
                let status_byte = packet.payload[0];
                let left_in_ear = (status_byte & 0x01) != 0;
                let right_in_ear = (status_byte & 0x02) != 0;
                Ok(AAPEvent::InEarChanged(InEarStatus {
                    left_in_ear,
                    right_in_ear,
                }))
            }
            OPCODE_CONVERSATIONAL_AWARENESS => {
                let enabled = packet.payload.first().map(|&b| b != 0).unwrap_or(false);
                Ok(AAPEvent::ConversationalAwarenessChanged(enabled))
            }
            other => Ok(AAPEvent::Unknown {
                opcode: other,
                data: packet.payload.clone(),
            }),
        }
    }
}
