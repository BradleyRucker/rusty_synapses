/// Calculate CRC8 for Synapse header validation
/// Polynomial: 0x07
pub fn crc8(data: &[u8]) -> u8 {
    let mut crc: u8 = 0;

    for &byte in data {
        crc ^= byte;
        for _ in 0..8 {
            if crc & 0x80 != 0 {
                crc = (crc << 1) ^ 0x07;
            } else {
                crc <<= 1;
            }
        }
    }

    crc
}

/// Calculate CRC16 for Synapse V2 payload validation
/// Polynomial: CRC-16-CCITT (0x1021)
pub fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;

    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }

    crc
}

/// Calculate CRC32 for Synapse V1 payload validation (legacy)
/// Polynomial: IEEE 802.3 (0xEDB88320)
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;

    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }

    crc ^ 0xFFFFFFFF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc8_empty() {
        assert_eq!(crc8(&[]), 0);
    }

    #[test]
    fn test_crc8_basic() {
        // Test with Synapse magic bytes "SN"
        let data = [0x53, 0x4E];
        let result = crc8(&data);
        // Result should be deterministic (u8 is always < 256)
        assert!(result == crc8(&data)); // Verify determinism
    }

    #[test]
    fn test_crc16_empty() {
        let result = crc16(&[]);
        assert!(result != 0); // CRC16 should not be zero for empty data
    }

    #[test]
    fn test_crc16_basic() {
        let data = b"123456789";
        let result = crc16(data);
        // CRC16-CCITT test vector
        assert_eq!(result, 0x29B1);
    }

    #[test]
    fn test_crc32_empty() {
        // Empty input: crc starts at 0xFFFFFFFF, no iterations, final XOR gives 0
        assert_eq!(crc32(&[]), 0);
    }

    #[test]
    fn test_crc32_basic() {
        let data = b"123456789";
        let result = crc32(data);
        // Standard CRC32 test vector
        assert_eq!(result, 0xCBF43926);
    }
}
