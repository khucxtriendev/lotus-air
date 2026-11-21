use serde::{Deserialize, Serialize};
use crate::error::{CoreError, Result};

/// Các chế độ chống ồn và kiểm soát âm thanh của AirPods 4 ANC & AirPods Pro
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NoiseControlMode {
    Off = 0x01,
    NoiseCancellation = 0x02,
    Transparency = 0x03,
    Adaptive = 0x04,
}

impl NoiseControlMode {
    pub fn from_u8(val: u8) -> Result<Self> {
        match val {
            0x01 => Ok(Self::Off),
            0x02 => Ok(Self::NoiseCancellation),
            0x03 => Ok(Self::Transparency),
            0x04 => Ok(Self::Adaptive),
            other => Err(CoreError::InvalidNoiseMode(other)),
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Off => "Tắt",
            Self::NoiseCancellation => "Chống ồn chủ động (ANC)",
            Self::Transparency => "Xuyên âm (Transparency)",
            Self::Adaptive => "Âm thanh thích ứng (Adaptive)",
        }
    }
}

impl Default for NoiseControlMode {
    fn default() -> Self {
        Self::Off
    }
}
