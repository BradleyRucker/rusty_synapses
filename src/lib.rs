pub mod crc;
pub mod cobs;
pub mod frame;
pub mod messages;
pub mod ffi;

#[cfg(feature = "python")]
mod python;

pub use frame::SynapseFrame;
pub use frame::{FLAG_ACK_REQ, FLAG_IS_ACK, FLAG_DUP, FLAG_ENCRYPTED, FLAG_CHUNKED, FLAG_PAYLOAD_CRC};
pub use messages::{SynapseMessage, Vector3, EulerAngles};
pub use messages::*;
