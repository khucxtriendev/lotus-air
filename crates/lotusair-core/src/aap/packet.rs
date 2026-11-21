use bytes::{Buf, BufMut, BytesMut};
use crate::error::{CoreError, Result};
use crate::aap::opcodes::AAP_HEADER_MAGIC;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AAPPacket {
    pub opcode: u16,
    pub payload: Vec<u8>,
}

impl AAPPacket {
    pub fn new(opcode: u16, payload: Vec<u8>) -> Self {
        Self { opcode, payload }
    }

    /// Serialize packet sang binary buffer để gửi qua socket L2CAP
    /// Format: [Header 4B: 04 00 04 00] [Opcode 2B LittleEndian] [Payload]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(AAP_HEADER_MAGIC.len() + 2 + self.payload.len());
        buf.put_slice(&AAP_HEADER_MAGIC);
        buf.put_u16_le(self.opcode);
        buf.put_slice(&self.payload);
        buf.to_vec()
    }

    /// Parse binary packet nhận được từ L2CAP
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < AAP_HEADER_MAGIC.len() + 2 {
            return Err(CoreError::BufferUnderflow {
                expected: AAP_HEADER_MAGIC.len() + 2,
                actual: data.len(),
            });
        }

        let mut cursor = data;

        // Check header magic nếu có
        if cursor.starts_with(&AAP_HEADER_MAGIC) {
            cursor.advance(AAP_HEADER_MAGIC.len());
        }

        if cursor.len() < 2 {
            return Err(CoreError::BufferUnderflow {
                expected: 2,
                actual: cursor.len(),
            });
        }

        let opcode = cursor.get_u16_le();
        let payload = cursor.to_vec();

        Ok(Self { opcode, payload })
    }
}
