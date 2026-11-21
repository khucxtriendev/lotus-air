use bytes::{Buf, BufMut, BytesMut};
use crate::error::{CoreError, Result};
use crate::aap::opcodes::{
    AAP_HANDSHAKE_ACK_MAGIC, AAP_HANDSHAKE_MAGIC, AAP_HEADER_MAGIC, OPCODE_HANDSHAKE_ACK,
    OPCODE_HANDSHAKE_INIT,
};

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
    /// - Handshake Init (opcode 0x0000): [00 00 04 00] + [Payload 12B] = 16 bytes
    /// - Standard AAP Packet: [Header 4B: 04 00 04 00] [Opcode 2B LittleEndian] [Payload]
    pub fn to_bytes(&self) -> Vec<u8> {
        if self.opcode == OPCODE_HANDSHAKE_INIT {
            let mut buf = BytesMut::with_capacity(AAP_HANDSHAKE_MAGIC.len() + self.payload.len());
            buf.put_slice(&AAP_HANDSHAKE_MAGIC);
            buf.put_slice(&self.payload);
            return buf.to_vec();
        }

        let mut buf = BytesMut::with_capacity(AAP_HEADER_MAGIC.len() + 2 + self.payload.len());
        buf.put_slice(&AAP_HEADER_MAGIC);
        buf.put_u16_le(self.opcode);
        buf.put_slice(&self.payload);
        buf.to_vec()
    }

    /// Parse binary packet nhận được từ L2CAP
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 4 {
            return Err(CoreError::BufferUnderflow {
                expected: 4,
                actual: data.len(),
            });
        }

        // Gói Handshake ACK thực tế từ AirPods bắt đầu bằng [01 00 04 00 ...]
        if data.starts_with(&AAP_HANDSHAKE_ACK_MAGIC) {
            return Ok(Self {
                opcode: OPCODE_HANDSHAKE_ACK,
                payload: data[AAP_HANDSHAKE_ACK_MAGIC.len()..].to_vec(),
            });
        }

        // Gói Handshake Init bắt đầu bằng [00 00 04 00 ...]
        if data.starts_with(&AAP_HANDSHAKE_MAGIC) {
            return Ok(Self {
                opcode: OPCODE_HANDSHAKE_INIT,
                payload: data[AAP_HANDSHAKE_MAGIC.len()..].to_vec(),
            });
        }

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
