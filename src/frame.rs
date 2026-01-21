use anyhow::{Result, anyhow};
use crate::{crc, cobs};
use std::time::{SystemTime, UNIX_EPOCH};

const SYNAPSE_MAGIC: u16 = 0x534E; // "SN"
const SYNAPSE_VERSION: u8 = 0x02;  // V2 protocol
const HEADER_SIZE: usize = 15;      // V2 header is 15 bytes

// Synapse V2 Flag Bit Layout (WIRE FORMAT - DO NOT CHANGE)
// 0x01 ACK_REQ
// 0x02 IS_ACK
// 0x04 DUP
// 0x08 ENCRYPTED
// 0x10 CHUNKED
// 0x20 PAYLOAD_CRC
pub const FLAG_ACK_REQ: u8 = 0x01;
pub const FLAG_IS_ACK: u8 = 0x02;
pub const FLAG_DUP: u8 = 0x04;
pub const FLAG_ENCRYPTED: u8 = 0x08;
pub const FLAG_CHUNKED: u8 = 0x10;
pub const FLAG_PAYLOAD_CRC: u8 = 0x20;

// Header extension types (TLV)
const EXT_ROUTING: u8 = 0x01;    // Routing: src/dst endpoints (4 bytes)
const EXT_FRAGMENT: u8 = 0x03;   // Fragment info (6 bytes)

/// Synapse V2 frame structure
#[derive(Debug, Clone)]
pub struct SynapseFrame {
    pub magic: u16,
    pub version: u8,
    pub flags: u8,
    pub msg_id: u16,
    pub sequence: u16,
    pub timestamp_ms: u32,         // V2: 32-bit milliseconds (not microseconds)
    pub hdr_ext_len: u8,           // V2: TLV extension length
    pub payload_len: u8,           // V2: 8-bit (max 255 bytes)

    // Routing extension (if present)
    pub src_endpoint: Option<u16>,
    pub dst_endpoint: Option<u16>,

    pub payload: Vec<u8>,
    pub payload_crc: Option<u16>,  // V2: CRC16 (not CRC32)
}

impl SynapseFrame {
    /// Create a new Synapse V2 frame
    pub fn new(msg_id: u16, payload: Vec<u8>) -> Self {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u32;

        SynapseFrame {
            magic: SYNAPSE_MAGIC,
            version: SYNAPSE_VERSION,
            flags: FLAG_PAYLOAD_CRC,  // Enable payload CRC by default
            msg_id,
            sequence: 0,
            timestamp_ms,
            hdr_ext_len: 0,
            payload_len: payload.len() as u8,
            src_endpoint: None,
            dst_endpoint: None,
            payload,
            payload_crc: None,
        }
    }

    /// Set the sequence number
    pub fn with_sequence(mut self, seq: u16) -> Self {
        self.sequence = seq;
        self
    }

    /// Set routing endpoints
    pub fn with_routing(mut self, src: u16, dst: u16) -> Self {
        self.src_endpoint = Some(src);
        self.dst_endpoint = Some(dst);
        self
    }

    /// Set flags
    pub fn with_flags(mut self, flags: u8) -> Self {
        self.flags = flags;
        self
    }

    /// Encode the frame to bytes (before COBS encoding)
    pub fn encode(&self) -> Vec<u8> {
        // Calculate header extensions size
        let has_routing = self.src_endpoint.is_some() || self.dst_endpoint.is_some();
        let hdr_ext_len = if has_routing { 2 + 4 } else { 0 }; // TLV: type(1) + len(1) + value(4)

        let has_payload_crc = (self.flags & FLAG_PAYLOAD_CRC) != 0;
        let crc_size = if has_payload_crc { 2 } else { 0 };

        let mut frame = Vec::with_capacity(HEADER_SIZE + hdr_ext_len + self.payload.len() + crc_size);

        // Build V2 header (15 bytes) - all little-endian
        frame.extend_from_slice(&self.magic.to_le_bytes());           // 0-1: magic
        frame.push(self.version);                                     // 2: version
        frame.push(self.flags);                                       // 3: flags
        frame.extend_from_slice(&self.msg_id.to_le_bytes());         // 4-5: msg_id
        frame.extend_from_slice(&self.sequence.to_le_bytes());       // 6-7: seq
        frame.extend_from_slice(&self.timestamp_ms.to_le_bytes());   // 8-11: timestamp_ms
        frame.push(hdr_ext_len as u8);                               // 12: hdr_ext_len
        frame.push(self.payload_len);                                // 13: payload_len

        // Calculate and add header CRC8 (first 14 bytes)
        let header_crc = crc::crc8(&frame[0..14]);
        frame.push(header_crc);                                      // 14: hdr_crc8

        // Add header extensions (TLV format)
        if has_routing {
            frame.push(EXT_ROUTING);  // type
            frame.push(4);            // len
            frame.extend_from_slice(&self.src_endpoint.unwrap_or(0).to_le_bytes());
            frame.extend_from_slice(&self.dst_endpoint.unwrap_or(0).to_le_bytes());
        }

        // Add payload
        frame.extend_from_slice(&self.payload);

        // Add optional payload CRC16
        if has_payload_crc {
            let payload_crc = crc::crc16(&self.payload);
            frame.extend_from_slice(&payload_crc.to_le_bytes());
        }

        frame
    }

    /// Encode the frame and apply COBS encoding
    pub fn encode_with_cobs(&self) -> Vec<u8> {
        let raw_frame = self.encode();
        cobs::encode(&raw_frame)
    }

    /// Parse a Synapse V2 frame from raw bytes (after COBS decoding)
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < HEADER_SIZE {
            return Err(anyhow!("Frame too short: {} bytes (need at least {})", data.len(), HEADER_SIZE));
        }

        // Parse V2 header (15 bytes)
        let magic = u16::from_le_bytes([data[0], data[1]]);
        if magic != SYNAPSE_MAGIC {
            return Err(anyhow!("Invalid magic: 0x{:04X}", magic));
        }

        let version = data[2];
        if version != SYNAPSE_VERSION {
            return Err(anyhow!("Unsupported version: 0x{:02X} (expected 0x{:02X})", version, SYNAPSE_VERSION));
        }

        let flags = data[3];
        let msg_id = u16::from_le_bytes([data[4], data[5]]);
        let sequence = u16::from_le_bytes([data[6], data[7]]);
        let timestamp_ms = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let hdr_ext_len = data[12] as usize;
        let payload_len = data[13] as usize;
        let header_crc = data[14];

        // Validate header CRC8
        let calculated_header_crc = crc::crc8(&data[0..14]);
        if calculated_header_crc != header_crc {
            return Err(anyhow!(
                "Header CRC mismatch: expected 0x{:02X}, got 0x{:02X}",
                calculated_header_crc,
                header_crc
            ));
        }

        // Check minimum frame length
        let has_payload_crc = (flags & FLAG_PAYLOAD_CRC) != 0;
        let crc_size = if has_payload_crc { 2 } else { 0 };
        let min_len = HEADER_SIZE + hdr_ext_len + payload_len + crc_size;

        if data.len() < min_len {
            return Err(anyhow!(
                "Frame length mismatch: expected at least {}, got {}",
                min_len,
                data.len()
            ));
        }

        // Parse header extensions (TLV format)
        let mut src_endpoint = None;
        let mut dst_endpoint = None;
        let mut ext_offset = HEADER_SIZE;
        let ext_end = ext_offset + hdr_ext_len;

        while ext_offset < ext_end {
            if ext_offset + 2 > ext_end {
                return Err(anyhow!("Incomplete TLV at offset {}", ext_offset));
            }

            let ext_type = data[ext_offset];
            let ext_len = data[ext_offset + 1] as usize;
            ext_offset += 2;

            if ext_offset + ext_len > ext_end {
                return Err(anyhow!("TLV value overrun: type=0x{:02X}, len={}", ext_type, ext_len));
            }

            match ext_type {
                EXT_ROUTING if ext_len == 4 => {
                    src_endpoint = Some(u16::from_le_bytes([data[ext_offset], data[ext_offset + 1]]));
                    dst_endpoint = Some(u16::from_le_bytes([data[ext_offset + 2], data[ext_offset + 3]]));
                }
                _ => {
                    // Unknown or unsupported TLV - skip
                }
            }

            ext_offset += ext_len;
        }

        // Extract payload
        let payload_offset = HEADER_SIZE + hdr_ext_len;
        let payload = data[payload_offset..payload_offset + payload_len].to_vec();

        // Validate optional payload CRC16
        let payload_crc = if has_payload_crc {
            let crc_offset = payload_offset + payload_len;
            if data.len() < crc_offset + 2 {
                return Err(anyhow!("Missing payload CRC"));
            }

            let received_crc = u16::from_le_bytes([data[crc_offset], data[crc_offset + 1]]);
            let calculated_crc = crc::crc16(&payload);

            if calculated_crc != received_crc {
                return Err(anyhow!(
                    "Payload CRC mismatch: expected 0x{:04X}, got 0x{:04X}",
                    calculated_crc,
                    received_crc
                ));
            }

            Some(received_crc)
        } else {
            None
        };

        Ok(SynapseFrame {
            magic,
            version,
            flags,
            msg_id,
            sequence,
            timestamp_ms,
            hdr_ext_len: hdr_ext_len as u8,
            payload_len: payload_len as u8,
            src_endpoint,
            dst_endpoint,
            payload,
            payload_crc,
        })
    }

    /// Parse a COBS-encoded frame
    pub fn parse_cobs(cobs_data: &[u8]) -> Result<Self> {
        let raw_frame = cobs::decode(cobs_data)?;
        Self::parse(&raw_frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_v2() {
        let payload = vec![0x01, 0x02, 0x03];
        let frame = SynapseFrame::new(0x0400, payload.clone())
            .with_routing(0x0002, 0x0001);

        let encoded = frame.encode();
        let decoded = SynapseFrame::parse(&encoded).unwrap();

        assert_eq!(decoded.magic, SYNAPSE_MAGIC);
        assert_eq!(decoded.version, SYNAPSE_VERSION);
        assert_eq!(decoded.msg_id, 0x0400);
        assert_eq!(decoded.payload, payload);
        assert_eq!(decoded.src_endpoint, Some(0x0002));
        assert_eq!(decoded.dst_endpoint, Some(0x0001));
    }

    #[test]
    fn test_cobs_roundtrip_v2() {
        let payload = vec![0x00, 0x01, 0x00, 0x02];
        let frame = SynapseFrame::new(0x0613, payload.clone());

        let cobs_encoded = frame.encode_with_cobs();
        let decoded = SynapseFrame::parse_cobs(&cobs_encoded).unwrap();

        assert_eq!(decoded.version, SYNAPSE_VERSION);
        assert_eq!(decoded.msg_id, 0x0613);
        assert_eq!(decoded.payload, payload);
    }

    #[test]
    fn test_v2_header_size() {
        assert_eq!(HEADER_SIZE, 15, "V2 header must be 15 bytes");
    }

    #[test]
    fn test_no_routing() {
        let payload = vec![0x01, 0x02];
        let frame = SynapseFrame::new(0x0001, payload.clone());

        let encoded = frame.encode();
        let decoded = SynapseFrame::parse(&encoded).unwrap();

        assert_eq!(decoded.src_endpoint, None);
        assert_eq!(decoded.dst_endpoint, None);
        assert_eq!(decoded.payload, payload);
    }
}
