/// Apple Accessory Protocol (AAP / AACP) Opcodes
/// Đảo ngược từ Apple bluetooth logs & LibrePods / Tyalie spec

pub const AAP_HEADER_MAGIC: [u8; 4] = [0x04, 0x00, 0x04, 0x00];
pub const AAP_L2CAP_PSM: u16 = 0x1001; // 4097 decimal
pub const APPLE_VENDOR_ID: u16 = 0x004C;

// Opcodes
pub const OPCODE_HANDSHAKE_INIT: u16 = 0x0000;
pub const OPCODE_HANDSHAKE_ACK: u16 = 0x0001;
pub const OPCODE_LISTEN_MODE_CONTROL: u16 = 0x000D; // ANC / Transparency / Adaptive / Off
pub const OPCODE_NOTIFY_SUBSCRIBE: u16 = 0x000F;
pub const OPCODE_HEAD_TRACKING: u16 = 0x0017;
pub const OPCODE_TIMESTAMP_SYNC: u16 = 0x001B;
pub const OPCODE_FEATURES_ACK: u16 = 0x002B;
pub const OPCODE_BATTERY_REPORT: u16 = 0x0032;
pub const OPCODE_IN_EAR_STATUS: u16 = 0x0035;
pub const OPCODE_CONVERSATIONAL_AWARENESS: u16 = 0x003F;
pub const OPCODE_ADAPTIVE_AUDIO_BLEND: u16 = 0x0044; // Thanh kéo tăng giảm thích ứng
pub const OPCODE_RENAME_DEVICE: u16 = 0x0048; // Đổi tên AirPods
pub const OPCODE_FORCE_AUDIO_TAKEOVER: u16 = 0x004A; // Cướp kết nối từ iPhone sang PC
pub const OPCODE_FEATURE_NEGOTIATION: u16 = 0x004D;
