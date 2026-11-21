use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Packet parsing failed: {0}")]
    PacketParseError(String),

    #[error("Packet buffer too short: expected at least {expected} bytes, got {actual}")]
    BufferUnderflow { expected: usize, actual: usize },

    #[error("Invalid AAP Opcode: 0x{0:04X}")]
    InvalidOpcode(u16),

    #[error("Invalid Noise Control Mode: {0}")]
    InvalidNoiseMode(u8),

    #[error("Decryption failed: {0}")]
    DecryptionError(String),

    #[error("Device not connected")]
    DeviceNotConnected,

    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, CoreError>;
