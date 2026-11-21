/// Apple Accessory Protocol (AAP / AACP) Opcodes & Control Command Identifiers
/// Chuẩn giao thức Apple H1/H2 qua L2CAP PSM 0x1001

pub const AAP_HEADER_MAGIC: [u8; 4] = [0x04, 0x00, 0x04, 0x00];
pub const AAP_HANDSHAKE_MAGIC: [u8; 4] = [0x00, 0x00, 0x04, 0x00];
pub const AAP_HANDSHAKE_ACK_MAGIC: [u8; 4] = [0x01, 0x00, 0x04, 0x00];
pub const AAP_L2CAP_PSM: u16 = 0x1001; // 4097 decimal
pub const APPLE_VENDOR_ID: u16 = 0x004C;

// Top-level AAP Opcodes (sau 04 00 04 00)
pub const OPCODE_HANDSHAKE_INIT: u16 = 0x0000;
pub const OPCODE_HANDSHAKE_ACK: u16 = 0x0001;
pub const OPCODE_BATTERY_REPORT: u16 = 0x0004;
pub const OPCODE_IN_EAR_STATUS: u16 = 0x0006;
pub const OPCODE_CONTROL_COMMAND: u16 = 0x0009;
pub const OPCODE_LISTEN_MODE_CONTROL: u16 = 0x0009; // Alias cho Control Command (0x09, sub-id 0x0D)
pub const OPCODE_AUDIO_SOURCE_REQ: u16 = 0x000D;
pub const OPCODE_NOTIFY_SUBSCRIBE: u16 = 0x000F;
pub const OPCODE_HEAD_TRACKING: u16 = 0x0017; // Spatial Audio / Head Tracking
pub const OPCODE_STEM_PRESS: u16 = 0x0019;
pub const OPCODE_RENAME_DEVICE_1A: u16 = 0x001A;
pub const OPCODE_TIMESTAMP_SYNC: u16 = 0x001B; // Keep-Alive / Timestamp Sync
pub const OPCODE_DEVICE_METADATA: u16 = 0x001D;
pub const OPCODE_RENAME_DEVICE: u16 = 0x001E; // Đổi tên AirPods
pub const OPCODE_FEATURES_ACK: u16 = 0x002B;
pub const OPCODE_CONVERSATIONAL_AWARENESS_DATA: u16 = 0x004B; // Sự kiện đang nói (Conversational Awareness)
pub const OPCODE_FEATURE_NEGOTIATION: u16 = 0x004D; // SET_SPECIFIC_FEATURES (bắt buộc cho AirPods Pro 2 / AirPods 4 ANC)
pub const OPCODE_ACOUSTIC_FIT_TEST: u16 = 0x004E;

// Control Command Identifiers (Byte thứ 7 trong gói OPCODE_CONTROL_COMMAND 0x0009, tổng 11 bytes)
pub const CTRL_MIC_MODE: u8 = 0x01;
pub const CTRL_BUTTON_SEND_MODE: u8 = 0x05;
pub const CTRL_OWNS_CONNECTION: u8 = 0x06; // Chiếm quyền điều khiển âm thanh (Audio Takeover)
pub const CTRL_VOICE_TRIGGER: u8 = 0x07;
pub const CTRL_EAR_DETECTION_CONFIG: u8 = 0x0A; // Tự động phát hiện tai (0x01 = Bật, 0x02 = Tắt)
pub const CTRL_LISTENING_MODE: u8 = 0x0D; // Chế độ chống ồn (0x01 = Off, 0x02 = ANC, 0x03 = Transparency, 0x04 = Adaptive)
pub const CTRL_ALLOW_AUTO_CONNECT: u8 = 0x14;
pub const CTRL_DOUBLE_TAP_MODE: u8 = 0x15;
pub const CTRL_CLICK_HOLD_MODE: u8 = 0x16; // Nhấn giữ thân AirPods
pub const CTRL_CLICK_SPEED: u8 = 0x17;
pub const CTRL_HOLD_DURATION: u8 = 0x18;
pub const CTRL_LISTENING_MODE_CONFIGS: u8 = 0x1A; // Bitmask luân chuyển chế độ nghe (0x01=Off, 0x02=ANC, 0x04=Trans, 0x08=Adaptive)
pub const CTRL_ONE_BUD_ANC_MODE: u8 = 0x1B; // ANC với 1 tai nghe (0x01 = Bật, 0x02 = Tắt)
pub const CTRL_CHIME_VOLUME: u8 = 0x1F; // Âm lượng chuông hộp sạc
pub const CTRL_AUTO_CONNECTION_CONFIG: u8 = 0x20; // Kết nối tự động (0x01 = Bật, 0x02 = Tắt)
pub const CTRL_CALL_MANAGEMENT_CONFIG: u8 = 0x24; // Cử chỉ đầu / Quản lý cuộc gọi
pub const CTRL_VOLUME_SWIPE_LENGTH: u8 = 0x25;
pub const CTRL_ADAPTIVE_VOLUME_CONFIG: u8 = 0x26; // Âm lượng cá nhân hóa (Personalized Volume: 0x01 = Bật, 0x02 = Tắt)
pub const CTRL_CONVERSATION_DETECT_CONFIG: u8 = 0x28; // Nhận biết cuộc hội thoại (0x01 = Bật, 0x02 = Tắt)
pub const CTRL_AUTO_ANC_STRENGTH: u8 = 0x2E; // Mức độ Adaptive Audio (0..=100)
pub const CTRL_IN_CASE_TONE_CONFIG: u8 = 0x31; // Âm thanh hộp sạc (0x01 = Bật, 0x02 = Tắt)
pub const CTRL_SIRI_MULTITONE_CONFIG: u8 = 0x32;
pub const CTRL_HEARING_ASSIST_CONFIG: u8 = 0x33;
pub const CTRL_ALLOW_OFF_OPTION: u8 = 0x34; // Cho phép chọn chế độ Off trên AirPods Pro 2 / AirPods 4 ANC (0x01 = Bật, 0x02 = Tắt)
pub const CTRL_STEM_CONFIG: u8 = 0x35;
pub const CTRL_SLEEP_DETECTION_CONFIG: u8 = 0x36;
pub const CTRL_PPE_TOGGLE_CONFIG: u8 = 0x37; // Giảm âm thanh lớn (Loud Noise Reduction: 0x01 = Bật, 0x02 = Tắt)
pub const CTRL_PPE_CAP_LEVEL_CONFIG: u8 = 0x38;
pub const CTRL_IN_CASE_TONE_VOLUME: u8 = 0x40; // Âm lượng âm báo hộp sạc
