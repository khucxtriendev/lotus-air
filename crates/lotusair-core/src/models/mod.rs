pub mod noise_mode;
pub mod battery;
pub mod device;
pub mod features;

pub use noise_mode::NoiseControlMode;
pub use battery::{AirPodsBatteryStatus, SingleBattery};
pub use device::AirPodsModel;
pub use features::{AirPodsFeaturesState, InEarStatus, SpatialAudioMode, StemAction};

use serde::{Deserialize, Serialize};

/// Toàn bộ trạng thái thiết bị AirPods hiện tại
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirPodsDeviceState {
    pub name: String,
    pub address: String,
    pub model: AirPodsModel,
    pub connected: bool,
    pub noise_mode: NoiseControlMode,
    pub battery: AirPodsBatteryStatus,
    pub in_ear: InEarStatus,
    pub features: AirPodsFeaturesState,
}

impl Default for AirPodsDeviceState {
    fn default() -> Self {
        Self {
            name: "AirPods 4".to_string(),
            address: "".to_string(),
            model: AirPodsModel::AirPods4Anc,
            connected: false,
            noise_mode: NoiseControlMode::NoiseCancellation,
            battery: AirPodsBatteryStatus::default(),
            in_ear: InEarStatus::default(),
            features: AirPodsFeaturesState::default(),
        }
    }
}
