use serde::{Deserialize, Serialize};

/// Thông tin trạng thái pin của từng thành phần tai nghe
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SingleBattery {
    /// Mức pin từ 0 -> 100%, hoặc None nếu không rõ
    pub level: Option<u8>,
    /// Đang cắm sạc hay không
    pub is_charging: bool,
}

impl Default for SingleBattery {
    fn default() -> Self {
        Self {
            level: None,
            is_charging: false,
        }
    }
}

/// Trạng thái pin tổng hợp của AirPods (Tai trái, Tai phải, Hộp sạc)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AirPodsBatteryStatus {
    pub left: SingleBattery,
    pub right: SingleBattery,
    pub case: SingleBattery,
}

impl AirPodsBatteryStatus {
    pub fn new(left: (Option<u8>, bool), right: (Option<u8>, bool), case: (Option<u8>, bool)) -> Self {
        Self {
            left: SingleBattery { level: left.0, is_charging: left.1 },
            right: SingleBattery { level: right.0, is_charging: right.1 },
            case: SingleBattery { level: case.0, is_charging: case.1 },
        }
    }
}
