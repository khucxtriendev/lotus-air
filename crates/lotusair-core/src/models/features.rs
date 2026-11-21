use serde::{Deserialize, Serialize};

/// Trạng thái cảm biến đeo tai (In-Ear Detection)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct InEarStatus {
    pub left_in_ear: bool,
    pub right_in_ear: bool,
}

/// Cấu hình các chế độ được luân chuyển khi bóp thân tai nghe
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CycleModesConfig {
    pub anc: bool,
    pub adaptive: bool,
    pub transparency: bool,
    pub off: bool,
}

impl Default for CycleModesConfig {
    fn default() -> Self {
        Self {
            anc: true,
            adaptive: true,
            transparency: true,
            off: false,
        }
    }
}

impl CycleModesConfig {
    pub fn to_bitmask(self) -> u8 {
        let mut mask = 0u8;
        if self.off {
            mask |= 0x01;
        }
        if self.anc {
            mask |= 0x02;
        }
        if self.transparency {
            mask |= 0x04;
        }
        if self.adaptive {
            mask |= 0x08;
        }
        mask
    }
}

/// Trạng thái các tính năng cao cấp giống như trên macOS/iOS
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AirPodsFeaturesState {
    /// Tự động nhận biết cuộc hội thoại (hạ âm lượng khi nói)
    pub conversational_awareness: bool,
    /// Âm lượng cá nhân hóa (Personalized Volume)
    pub personalized_volume: bool,
    /// Giảm tiếng ồn lớn đột ngột (Loud Noise Reduction)
    pub loud_noise_reduction: bool,
    /// Mức độ điều chỉnh Âm thanh thích ứng (0: Ít tiếng ồn hơn, 100: Nhiều tiếng ồn hơn)
    pub adaptive_audio_blend: u8,
    /// Luôn duy trì kết nối Bluetooth (Keep-Alive) để chọn thiết bị đầu ra tức thì
    pub keep_alive_connection: bool,
    /// Ưu tiên cướp tai nghe từ iPhone/iPad khi kết nối
    pub auto_takeover: bool,
    /// Spatial Audio (Âm thanh không gian: Tắt / Cố định / Theo dõi đầu)
    pub spatial_audio_mode: SpatialAudioMode,
    /// Thao tác bóp thân tai (Stem Press Action)
    pub stem_press_left: StemAction,
    pub stem_press_right: StemAction,
    /// Các chế độ nghe luân chuyển khi nhấn giữ thân AirPods
    pub cycle_modes: CycleModesConfig,
    /// Kiểm soát tiếng ồn bằng một AirPod
    pub one_bud_anc: bool,
    /// Tương tác bằng cử chỉ đầu (Gật đầu nhận cuộc gọi, lắc đầu từ chối)
    pub head_gestures: bool,
    /// Tự động phát hiện tai (Phát khi đeo, tạm dừng khi tháo)
    pub auto_ear_detection: bool,
    /// Bật âm thanh hộp sạc
    pub case_charging_sounds: bool,
    /// Âm lượng chuông hộp sạc (0 - 100%)
    pub case_chime_volume: u8,
    /// Mạng Tìm (Find My Network)
    pub find_my_enabled: bool,
    /// Hiệu chuẩn không gian cá nhân hóa
    pub spatial_calibrated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpatialAudioMode {
    Off,
    Fixed,
    HeadTracked,
}

impl Default for SpatialAudioMode {
    fn default() -> Self {
        Self::Fixed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StemAction {
    CycleNoiseModes,
    SiriOrAssistant,
    PlayPause,
}

impl StemAction {
    pub fn from_str_mode(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "siri" | "siriorassistant" | "assistant" => Self::SiriOrAssistant,
            "playpause" | "media" => Self::PlayPause,
            _ => Self::CycleNoiseModes,
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            Self::CycleNoiseModes => 0x01,
            Self::SiriOrAssistant => 0x02,
            Self::PlayPause => 0x03,
        }
    }

    pub fn display_label(self) -> &'static str {
        match self {
            Self::CycleNoiseModes => "Chế độ nghe",
            Self::SiriOrAssistant => "Trợ lý giọng nói",
            Self::PlayPause => "Phát / Tạm dừng",
        }
    }
}

impl Default for StemAction {
    fn default() -> Self {
        Self::CycleNoiseModes
    }
}

impl Default for AirPodsFeaturesState {
    fn default() -> Self {
        Self {
            conversational_awareness: false,
            personalized_volume: true,
            loud_noise_reduction: true,
            adaptive_audio_blend: 50,
            keep_alive_connection: true,
            auto_takeover: false,
            spatial_audio_mode: SpatialAudioMode::Fixed,
            stem_press_left: StemAction::CycleNoiseModes,
            stem_press_right: StemAction::CycleNoiseModes,
            cycle_modes: CycleModesConfig::default(),
            one_bud_anc: true,
            head_gestures: true,
            auto_ear_detection: true,
            case_charging_sounds: true,
            case_chime_volume: 80,
            find_my_enabled: true,
            spatial_calibrated: true,
        }
    }
}
