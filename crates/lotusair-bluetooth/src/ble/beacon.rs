use lotusair_core::models::{AirPodsBatteryStatus, AirPodsModel, InEarStatus, SingleBattery};
use tracing::debug;

pub const APPLE_COMPANY_ID: u16 = 0x004C;
pub const CONTINUITY_TYPE_PROXIMITY: u8 = 0x07;

#[derive(Debug, Clone)]
pub struct AppleBeaconInfo {
    pub model: AirPodsModel,
    pub battery: AirPodsBatteryStatus,
    pub in_ear: InEarStatus,
    pub lid_open: bool,
    pub lid_counter: u8,
    pub rssi: Option<i16>,
}

pub struct AppleBeaconParser;

impl AppleBeaconParser {
    /// Phân giải Manufacturer Data phát từ AirPods (Continuity Proximity 0x07)
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

        // Model ID nằm tại offset 3-4
        let model_id = if data.len() >= 5 {
            u16::from_be_bytes([data[3], data[4]])
        } else {
            0
        };
        let model = AirPodsModel::from_model_id(model_id);

        // Byte 5: Status / In-Ear / Flip flag
        let status_byte = data.get(5).copied().unwrap_or(0);
        let is_flipped = (status_byte & 0x20) == 0;
        let primary_in_ear = (status_byte & 0x02) != 0 || (status_byte & 0x01) != 0;
        let secondary_in_ear = (status_byte & 0x08) != 0 || (status_byte & 0x01) != 0;
        let in_ear = if is_flipped {
            InEarStatus {
                left_in_ear: secondary_in_ear,
                right_in_ear: primary_in_ear,
            }
        } else {
            InEarStatus {
                left_in_ear: primary_in_ear,
                right_in_ear: secondary_in_ear,
            }
        };

        // Byte 6: Nibble battery
        let (left_raw, right_raw) = if data.len() > 6 {
            let b = data[6];
            let hi = (b >> 4) & 0x0F;
            let lo = b & 0x0F;
            if is_flipped { (lo, hi) } else { (hi, lo) }
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

        let (left_charging, right_charging) = if is_flipped {
            ((charging_byte & 0x20) != 0, (charging_byte & 0x10) != 0)
        } else {
            ((charging_byte & 0x10) != 0, (charging_byte & 0x20) != 0)
        };
        let case_charging = (charging_byte & 0x40) != 0;

        let decode_nibble = |val: u8| -> Option<u8> {
            match val {
                0..=9 => Some(val * 10 + 5), // 5%, 15%, ... 95%
                0x0A => Some(100),           // 100%
                _ => None,                   // Không rõ / hộp sạc đang đóng
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

        // Byte 8 (Apple Continuity Proximity 0x07):
        // - Bit 3 (0x08): 0 = nắp hộp ĐANG MỞ, 1 = nắp hộp ĐANG ĐÓNG
        // - Bits 0..2 (0x07): bộ đếm số lần đóng/mở nắp hộp (tăng mỗi khi mở nắp)
        let lid_byte = data.get(8).copied().unwrap_or(0x08);
        let lid_counter = lid_byte & 0x07;
        let lid_open = (lid_byte & 0x08) == 0 && battery.case.level.is_some();

        debug!(
            "Phân tích Apple BLE beacon thành công: {:?}, in_ear: {:?}, lid_open: {} (counter={}), model: {:?}",
            battery, in_ear, lid_open, lid_counter, model
        );

        Some(AppleBeaconInfo {
            model,
            battery,
            in_ear,
            lid_open,
            lid_counter,
            rssi: None,
        })
    }
}
