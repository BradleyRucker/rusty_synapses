use anyhow::{Result, anyhow};

/// COBS (Consistent Overhead Byte Stuffing)
/// Wire-compatible with Synapse C++ / Python codec
///
/// Encoding rules:
/// - No 0x00 bytes in encoded payload
/// - Exactly one 0x00 delimiter at end
/// - Empty input => [0x01, 0x00]
pub fn encode(data: &[u8]) -> Vec<u8> {
    // Empty frame special-case
    if data.is_empty() {
        return vec![0x01, 0x00];
    }

    let mut out = Vec::with_capacity(data.len() + 2);

    let mut code_index = 0;
    let mut code: u8 = 1;

    // Reserve first code byte
    out.push(0);

    for &byte in data {
        if byte == 0 {
            // Flush current block
            out[code_index] = code;
            code_index = out.len();
            out.push(0); // new code placeholder
            code = 1;
        } else {
            out.push(byte);
            code += 1;

            if code == 0xFF {
                // Max block reached
                out[code_index] = code;
                code_index = out.len();
                out.push(0);
                code = 1;
            }
        }
    }

    // Final block
    out[code_index] = code;

    // Frame delimiter
    out.push(0x00);

    out
}

pub fn decode(encoded: &[u8]) -> Result<Vec<u8>> {
    if encoded.is_empty() {
        return Err(anyhow!("Empty COBS input"));
    }

    // Must end with delimiter
    if *encoded.last().unwrap() != 0x00 {
        return Err(anyhow!("COBS frame missing 0x00 delimiter"));
    }

    let mut decoded = Vec::with_capacity(encoded.len());
    let mut i = 0;
    let end = encoded.len() - 1; // exclude delimiter

    while i < end {
        let code = encoded[i];
        if code == 0 {
            return Err(anyhow!("Invalid COBS code byte"));
        }

        i += 1;
        let copy_len = (code - 1) as usize;

        if i + copy_len > end {
            return Err(anyhow!("COBS block overruns frame"));
        }

        // Copy data bytes
        for _ in 0..copy_len {
            let b = encoded[i];
            if b == 0 {
                return Err(anyhow!("Zero byte in COBS payload"));
            }
            decoded.push(b);
            i += 1;
        }

        // Insert implicit zero when code < 0xFF
        if code < 0xFF {
            decoded.push(0x00);
        }
    }

    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let encoded = encode(&[]);
        assert_eq!(encoded, vec![0x01, 0x00]);
    }

    #[test]
    fn test_no_zeros() {
        let data = vec![0x01, 0x02, 0x03];
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        // COBS adds trailing zero on decode
        assert_eq!(&decoded[..3], &data[..]);
    }

    #[test]
    fn test_with_zeros() {
        let data = vec![0x00, 0x01, 0x00, 0x02];
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(&decoded[..4], &data[..]);
    }

    #[test]
    fn test_roundtrip() {
        let data = vec![0x11, 0x22, 0x00, 0x33];
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(&decoded[..4], &data[..]);
    }
}
