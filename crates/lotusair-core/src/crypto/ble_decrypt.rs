use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
use cbc::Decryptor;
use crate::error::{CoreError, Result};

type Aes128CbcDec = Decryptor<aes::Aes128>;

/// Giải mã dữ liệu quảng bá BLE mã hoá của AirPods sử dụng khóa chia sẻ (accKey)
pub fn decrypt_ble_payload(key: &[u8; 16], iv: &[u8; 16], ciphertext: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes128CbcDec::new(key.into(), iv.into());
    let mut buf = ciphertext.to_vec();
    let res = cipher
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .map_err(|e| CoreError::DecryptionError(format!("AES-128 CBC decrypt failed: {:?}", e)))?;
    Ok(res.to_vec())
}
