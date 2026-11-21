use lotusair_core::models::{AirPodsBatteryStatus, AirPodsModel, SingleBattery};
use tracing::debug;

pub const APPLE_COMPANY_ID: u16 = 0x004C;
pub const CONTINUITY_TYPE_PROXIMITY: u8 = 0x07;

#[derive(Debug, Clone)]
pub struct AppleBeaconInfo {
    pub model: AirPodsModel,
    pub battery: AirPodsBatteryStatus,
    pub lid_open: bool,
}

pub struct AppleBeaconParser;

impl AppleBeaconParser {
    /// Phân giải Manufacturer Data phát từ AirPods
    pub fn parse_manufacturer_data(company_id: u16, data: &[u8]) -> Option<AppleBeaconInfo> {
        if company_id != APPLE_COMPANY_ID || data.len() < 8 {
            return None;
        }

        let msg_type = data[0];
        if msg_type != CONTINUITY_TYPE_PROXIMITY {
            return None;
        }

        let length = data.get(1).copied().unwrap_or(0);
        if (length as usize) > data.len() {
            return None;
        }

        // Model ID nằm tại offset 3-4 (little-endian hoặc big-endian tùy phiên bản)
        let model_id = if data.len() >= 5 {
            u16::from_be_bytes([data[3], data[4]])
        } else {
            0
        };
        let model = AirPodsModel::from_model_id(model_id);

        // Byte 6: Nibble battery (High: Left, Low: Right)
        let (left_raw, right_raw) = if data.len() > 6 {
            let b = data[6];
            ((b >> 4) & 0x0F, b & 0x0F)
        } else {
            (0x0F, 0x0F)
        };

        // Byte 7: Case battery & charging flags
        let (case_raw, charging_byte) = if data.len() > 7 {
            let b = data[7];
            (b & 0x0F, b)
        } else {
            (0x0F, 0)
        };

        let left_charging = (charging_byte & 0x10) != 0;
        let right_charging = (charging_byte & 0x20) != 0;
        let case_charging = (charging_byte & 0x40) != 0;

        let decode_nibble = |val: u8| -> Option<u8> {
            match val {
                0..=9 => Some(val * 10 + 5), // 5%, 15%, ... 95%
                0x0A => Some(100),           // 100%
                _ => None,                  // Đang ngắt kết nối / không rõ
            }
        };

        let battery = AirPodsBatteryStatus {
            left: SingleBattery {
                level: decode_nibble(left_raw),
                is_charging: left_charging,
            },
            right: SingleBattery {
                level: decode_nibble(right_raw),
                is_charging: right_charging,
            },
            case: SingleBattery {
                level: decode_nibble(case_raw),
                is_charging: case_charging,
            },
        };

        let lid_open = (charging_byte & 0x80) != 0;

        debug!("Phân tích Apple BLE beacon thành công: {:?}, model: {:?}", battery, model);

        Some(AppleBeaconInfo {
            model,
            battery,
            lid_open,
        })
    }
}
