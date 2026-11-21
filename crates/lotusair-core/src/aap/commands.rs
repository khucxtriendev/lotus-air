use crate::aap::opcodes::*;
use crate::aap::packet::AAPPacket;
use crate::models::{CycleModesConfig, NoiseControlMode, SpatialAudioMode, StemAction};

pub struct AAPCommands;

impl AAPCommands {
    /// Tạo gói lệnh điều khiển chuẩn Apple AACP Control Command (11 bytes tổng cộng):
    /// `[04 00 04 00] [09 00] [identifier] [d1] [d2] [d3] [d4]`
    pub fn control_command(identifier: u8, data: [u8; 4]) -> AAPPacket {
        AAPPacket::new(
            OPCODE_CONTROL_COMMAND,
            vec![identifier, data[0], data[1], data[2], data[3]],
        )
    }

    /// Gói tin bắt tay khởi tạo phiên kết nối AAP với AirPods (16 bytes):
    /// `00 00 04 00 01 00 02 00 00 00 00 00 00 00 00 00`
    pub fn handshake_init() -> AAPPacket {
        let payload = vec![
            0x01, 0x00, 0x02, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        AAPPacket::new(OPCODE_HANDSHAKE_INIT, payload)
    }

    /// Bật các tính năng nâng cao cho AirPods Pro 2 / AirPods 4 ANC (14 bytes):
    /// `04 00 04 00 4D 00 D7 00 00 00 00 00 00 00`
    /// (Bắt buộc phải gửi sau Handshake để Adaptive Audio & Conversational Awareness hoạt động)
    pub fn set_specific_features() -> AAPPacket {
        AAPPacket::new(
            OPCODE_FEATURE_NEGOTIATION,
            vec![0xD7, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        )
    }

    /// Đăng ký nhận thông báo thời gian thực (Pin, Đeo tai, Cảm biến, Thay đổi trạng thái) (10 bytes):
    /// `04 00 04 00 0F 00 FF FF FF FF`
    pub fn subscribe_notifications() -> AAPPacket {
        let payload = vec![0xFF, 0xFF, 0xFF, 0xFF];
        AAPPacket::new(OPCODE_NOTIFY_SUBSCRIBE, payload)
    }

    /// Gói tin duy trì kết nối L2CAP (Keep-Alive / Request Notifications)
    pub fn keep_alive_ping() -> AAPPacket {
        Self::subscribe_notifications()
    }

    /// Cho phép chọn chế độ Off trong Listening Mode trên AirPods 4 ANC / Pro 2:
    /// `04 00 04 00 09 00 34 [01|02] 00 00 00`
    pub fn set_allow_off_option(enabled: bool) -> AAPPacket {
        let val = if enabled { 0x01 } else { 0x02 };
        Self::control_command(CTRL_ALLOW_OFF_OPTION, [val, 0x00, 0x00, 0x00])
    }

    /// Đặt chế độ chống ồn (ANC=0x02 / Xuyên âm=0x03 / Thích ứng=0x04 / Tắt=0x01):
    /// `04 00 04 00 09 00 0D [mode] 00 00 00`
    pub fn set_noise_mode(mode: NoiseControlMode) -> AAPPacket {
        Self::control_command(CTRL_LISTENING_MODE, [mode.to_u8(), 0x00, 0x00, 0x00])
    }

    /// Điều chỉnh mức độ thanh kéo thích ứng (0: Ít tiếng ồn, 100: Nhiều tiếng ồn):
    /// `04 00 04 00 09 00 2E [level] 00 00 00`
    pub fn set_adaptive_blend(blend_level: u8) -> AAPPacket {
        Self::control_command(
            CTRL_AUTO_ANC_STRENGTH,
            [blend_level.min(100), 0x00, 0x00, 0x00],
        )
    }

    /// Đổi tên AirPods trực tiếp (Opcode 0x001E: `04 00 04 00 1E 00 01 [len] 00 [utf8...]`)
    pub fn rename_device(new_name: &str) -> AAPPacket {
        let bytes = new_name.as_bytes();
        let len = bytes.len().min(64) as u8;
        let mut payload = Vec::with_capacity(3 + len as usize);
        payload.push(0x01);
        payload.push(len);
        payload.push(0x00);
        payload.extend_from_slice(&bytes[..len as usize]);
        AAPPacket::new(OPCODE_RENAME_DEVICE, payload)
    }

    /// Cướp tai nghe từ iPhone sang PC (OwnsConnection: `04 00 04 00 09 00 06 01 00 00 00`)
    pub fn force_audio_takeover() -> AAPPacket {
        Self::control_command(CTRL_OWNS_CONNECTION, [0x01, 0x00, 0x00, 0x00])
    }

    /// Bật/Tắt Tự động kết nối (Auto Connection Config: `04 00 04 00 09 00 20 [01|02] 00 00 00`)
    pub fn set_auto_connection_config(enabled: bool) -> AAPPacket {
        let val = if enabled { 0x01 } else { 0x02 };
        Self::control_command(CTRL_AUTO_CONNECTION_CONFIG, [val, 0x00, 0x00, 0x00])
    }

    /// Bật/Tắt Conversational Awareness (Nhận biết cuộc hội thoại):
    /// `04 00 04 00 09 00 28 [01=Bật | 02=Tắt] 00 00 00`
    pub fn set_conversational_awareness(enabled: bool) -> AAPPacket {
        let val = if enabled { 0x01 } else { 0x02 };
        Self::control_command(CTRL_CONVERSATION_DETECT_CONFIG, [val, 0x00, 0x00, 0x00])
    }

    /// Bật/Tắt Personalized Volume (Âm lượng được cá nhân hóa):
    /// `04 00 04 00 09 00 26 [01=Bật | 02=Tắt] 00 00 00`
    pub fn set_personalized_volume(enabled: bool) -> AAPPacket {
        let val = if enabled { 0x01 } else { 0x02 };
        Self::control_command(CTRL_ADAPTIVE_VOLUME_CONFIG, [val, 0x00, 0x00, 0x00])
    }

    /// Bật/Tắt Loud Noise Reduction (Giảm âm thanh lớn):
    /// `04 00 04 00 09 00 37 [01=Bật | 02=Tắt] 00 00 00`
    pub fn set_loud_noise_reduction(enabled: bool) -> AAPPacket {
        let val = if enabled { 0x01 } else { 0x02 };
        Self::control_command(CTRL_PPE_TOGGLE_CONFIG, [val, 0x00, 0x00, 0x00])
    }

    /// Bật/Tắt Kiểm soát tiếng ồn bằng một AirPod (One-Bud ANC):
    /// `04 00 04 00 09 00 1B [01=Bật | 02=Tắt] 00 00 00`
    pub fn set_one_bud_anc(enabled: bool) -> AAPPacket {
        let val = if enabled { 0x01 } else { 0x02 };
        Self::control_command(CTRL_ONE_BUD_ANC_MODE, [val, 0x00, 0x00, 0x00])
    }

    /// Bật/Tắt Tương tác bằng cử chỉ đầu (Head Gestures / Call Management):
    /// `04 00 04 00 09 00 24 [01=Bật | 02=Tắt] 00 00 00`
    pub fn set_head_gestures(enabled: bool) -> AAPPacket {
        let val = if enabled { 0x01 } else { 0x02 };
        Self::control_command(CTRL_CALL_MANAGEMENT_CONFIG, [val, 0x00, 0x00, 0x00])
    }

    /// Bật/Tắt Tự động phát hiện tai (Auto Ear Detection):
    /// `04 00 04 00 09 00 0A [01=Bật | 02=Tắt] 00 00 00`
    pub fn set_auto_ear_detection(enabled: bool) -> AAPPacket {
        let val = if enabled { 0x01 } else { 0x02 };
        Self::control_command(CTRL_EAR_DETECTION_CONFIG, [val, 0x00, 0x00, 0x00])
    }

    /// Cấu hình thao tác nhấn và giữ thân AirPods (`04 00 04 00 09 00 16 [d1] [d2] 00 00`)
    pub fn set_stem_press_action(is_right: bool, action: StemAction) -> AAPPacket {
        let code = match action {
            StemAction::CycleNoiseModes => 0x01,
            StemAction::SiriOrAssistant => 0x05,
            StemAction::PlayPause => 0x02,
        };
        if is_right {
            Self::control_command(CTRL_CLICK_HOLD_MODE, [code, 0x00, 0x00, 0x00])
        } else {
            Self::control_command(CTRL_CLICK_HOLD_MODE, [0x00, code, 0x00, 0x00])
        }
    }

    /// Cấu hình danh sách các chế độ nghe được luân chuyển khi bóp thân AirPods:
    /// `04 00 04 00 09 00 1A [bitmask] 00 00 00`
    pub fn set_cycle_modes(config: CycleModesConfig) -> AAPPacket {
        Self::control_command(
            CTRL_LISTENING_MODE_CONFIGS,
            [config.to_bitmask(), 0x00, 0x00, 0x00],
        )
    }

    /// Bật/Tắt âm thanh hộp sạc:
    /// `04 00 04 00 09 00 31 [01=Bật | 02=Tắt] 00 00 00`
    pub fn set_case_charging_sounds(enabled: bool) -> AAPPacket {
        let val = if enabled { 0x01 } else { 0x02 };
        Self::control_command(CTRL_IN_CASE_TONE_CONFIG, [val, 0x00, 0x00, 0x00])
    }

    /// Đặt âm lượng chuông hộp sạc (0 - 100%):
    /// `04 00 04 00 09 00 1F [volume] 00 00 00`
    pub fn set_case_chime_volume(volume: u8) -> AAPPacket {
        Self::control_command(CTRL_CHIME_VOLUME, [volume.clamp(1, 100), 0x00, 0x00, 0x00])
    }

    /// Chạy kiểm tra độ vừa vặn âm học (Acoustic Fit Test)
    pub fn run_acoustic_fit_test() -> AAPPacket {
        AAPPacket::new(OPCODE_ACOUSTIC_FIT_TEST, vec![0x01, 0x00, 0x00, 0x00])
    }

    /// Đặt chế độ Spatial Audio / Head Tracking (Opcode 0x0017)
    pub fn set_spatial_audio_mode(mode: SpatialAudioMode) -> AAPPacket {
        let payload = match mode {
            SpatialAudioMode::HeadTracked => vec![
                0x00, 0x00, 0x10, 0x00, 0x10, 0x00, 0x08, 0xA1, 0x02, 0x42, 0x0B, 0x08, 0x0E, 0x10,
                0x02, 0x1A, 0x05, 0x01, 0x40, 0x9C, 0x00, 0x00,
            ],
            SpatialAudioMode::Fixed | SpatialAudioMode::Off => vec![
                0x00, 0x00, 0x10, 0x00, 0x11, 0x00, 0x08, 0x7E, 0x10, 0x02, 0x42, 0x0B, 0x08, 0x4E,
                0x10, 0x02, 0x1A, 0x05, 0x01, 0x00, 0x00, 0x00, 0x00,
            ],
        };
        AAPPacket::new(OPCODE_HEAD_TRACKING, payload)
    }
}
