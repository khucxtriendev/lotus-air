use crate::aap::opcodes::*;
use crate::aap::packet::AAPPacket;
use crate::models::{NoiseControlMode, SpatialAudioMode};

pub struct AAPCommands;

impl AAPCommands {
    /// Gói tin bắt tay khởi tạo phiên kết nối AAP với AirPods
    pub fn handshake_init() -> AAPPacket {
        let payload = vec![
            0x01, 0x00, 0x02, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        AAPPacket::new(OPCODE_HANDSHAKE_INIT, payload)
    }

    /// Đăng ký nhận thông báo thời gian thực (Pin, Đeo tai, Cảm biến, Thay đổi trạng thái)
    pub fn subscribe_notifications() -> AAPPacket {
        let payload = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        AAPPacket::new(OPCODE_NOTIFY_SUBSCRIBE, payload)
    }

    /// Đặt chế độ chống ồn (ANC / Xuyên âm / Thích ứng / Tắt)
    pub fn set_noise_mode(mode: NoiseControlMode) -> AAPPacket {
        let payload = vec![mode.to_u8()];
        AAPPacket::new(OPCODE_LISTEN_MODE_CONTROL, payload)
    }

    /// Điều chỉnh mức độ thanh kéo thích ứng (0: Ít tiếng ồn, 100: Nhiều tiếng ồn)
    pub fn set_adaptive_blend(blend_level: u8) -> AAPPacket {
        let payload = vec![blend_level.min(100)];
        AAPPacket::new(OPCODE_ADAPTIVE_AUDIO_BLEND, payload)
    }

    /// Đổi tên AirPods trực tiếp
    pub fn rename_device(new_name: &str) -> AAPPacket {
        let mut payload = new_name.as_bytes().to_vec();
        payload.push(0x00); // null-terminated
        AAPPacket::new(OPCODE_RENAME_DEVICE, payload)
    }

    /// Cướp tai nghe từ iPhone sang PC (Chuyển luồng A2DP/HFP về máy tính)
    pub fn force_audio_takeover() -> AAPPacket {
        let payload = vec![0x01, 0x00, 0x01];
        AAPPacket::new(OPCODE_FORCE_AUDIO_TAKEOVER, payload)
    }

    /// Bật/Tắt Conversational Awareness
    pub fn set_conversational_awareness(enabled: bool) -> AAPPacket {
        let payload = vec![if enabled { 0x01 } else { 0x00 }];
        AAPPacket::new(OPCODE_CONVERSATIONAL_AWARENESS, payload)
    }

    /// Đặt chế độ Spatial Audio: Off / Fixed / HeadTracked
    pub fn set_spatial_audio_mode(mode: SpatialAudioMode) -> AAPPacket {
        let flag = match mode {
            SpatialAudioMode::Off => 0x00,
            SpatialAudioMode::Fixed => 0x01,
            SpatialAudioMode::HeadTracked => 0x10,
        };
        let payload = vec![0x00, 0x00, flag];
        AAPPacket::new(OPCODE_HEAD_TRACKING, payload)
    }
}
