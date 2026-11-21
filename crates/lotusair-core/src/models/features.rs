use serde::{Deserialize, Serialize};

/// Trạng thái cảm biến đeo tai (In-Ear Detection)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct InEarStatus {
    pub left_in_ear: bool,
    pub right_in_ear: bool,
}

/// Trạng thái các tính năng cao cấp giống như trên macOS/iOS
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Spatial Audio (Âm thanh không gian: Tắt / Cố định / Theo dõi đầu)
    pub spatial_audio_mode: SpatialAudioMode,
    /// Thao tác bóp thân tai (Stem Press Action)
    pub stem_press_left: StemAction,
    pub stem_press_right: StemAction,
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

impl Default for StemAction {
    fn default() -> Self {
        Self::CycleNoiseModes
    }
}

impl Default for AirPodsFeaturesState {
    fn default() -> Self {
        Self {
            conversational_awareness: true,
            personalized_volume: true,
            loud_noise_reduction: true,
            adaptive_audio_blend: 50,
            keep_alive_connection: true,
            spatial_audio_mode: SpatialAudioMode::HeadTracked,
            stem_press_left: StemAction::CycleNoiseModes,
            stem_press_right: StemAction::CycleNoiseModes,
        }
    }
}
