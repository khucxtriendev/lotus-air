use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AirPodsModel {
    AirPods4Anc,
    AirPods4Standard,
    AirPodsPro2,
    AirPodsPro1,
    AirPods3,
    AirPodsMax,
    Unknown(u16),
}

impl AirPodsModel {
    pub fn from_model_id(id: u16) -> Self {
        match id {
            0x1B20 | 0x201B => Self::AirPods4Anc,
            0x1920 | 0x2019 => Self::AirPods4Standard,
            0x1420 | 0x2014 | 0x2420 | 0x2024 => Self::AirPodsPro2,
            0x0E20 | 0x200E => Self::AirPodsPro1,
            0x1320 | 0x2013 => Self::AirPods3,
            0x0A20 | 0x200A => Self::AirPodsMax,
            other => Self::Unknown(other),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::AirPods4Anc => "AirPods 4 (ANC)",
            Self::AirPods4Standard => "AirPods 4",
            Self::AirPodsPro2 => "AirPods Pro 2",
            Self::AirPodsPro1 => "AirPods Pro",
            Self::AirPods3 => "AirPods (3rd gen)",
            Self::AirPodsMax => "AirPods Max",
            Self::Unknown(_) => "Apple AirPods",
        }
    }

    pub fn supports_anc(&self) -> bool {
        matches!(self, Self::AirPods4Anc | Self::AirPodsPro2 | Self::AirPodsPro1 | Self::AirPodsMax)
    }

    pub fn supports_adaptive_audio(&self) -> bool {
        matches!(self, Self::AirPods4Anc | Self::AirPodsPro2)
    }

    pub fn supports_conversational_awareness(&self) -> bool {
        matches!(self, Self::AirPods4Anc | Self::AirPodsPro2)
    }
}
