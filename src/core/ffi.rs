//! C FFI bindings for Synapse protocol.
//!
//! These functions provide a C-compatible interface for building and parsing
//! Synapse frames from C/C++ code.

use crate::core::frame::SynapseFrame;
use crate::core::messages::{
    SupervisorMessage, SynapseMessage, SERVICE_VIO, STATUS_RUNNING,
};
use libc::{c_char, c_int, size_t};
use std::ffi::CStr;
use std::slice;

/// Build a Register message and write it to the output buffer.
/// Returns the number of bytes written, or -1 on error.
#[no_mangle]
pub extern "C" fn synapse_build_register(
    service_id: u8,
    pid: u32,
    version: *const c_char,
    out_buf: *mut u8,
    out_buf_len: size_t,
) -> c_int {
    if out_buf.is_null() || version.is_null() {
        return -1;
    }

    let version_str = unsafe {
        match CStr::from_ptr(version).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return -1,
        }
    };

    let msg = SynapseMessage::Supervisor(SupervisorMessage::Register {
        service_id,
        pid,
        version: version_str,
    });

    let frame = msg.to_frame();
    let encoded = frame.encode_with_cobs();

    if encoded.len() > out_buf_len {
        return -1;
    }

    unsafe {
        std::ptr::copy_nonoverlapping(encoded.as_ptr(), out_buf, encoded.len());
    }

    encoded.len() as c_int
}

/// Build a ServiceHeartbeat message and write it to the output buffer.
/// Returns the number of bytes written, or -1 on error.
#[no_mangle]
pub extern "C" fn synapse_build_heartbeat(
    service_id: u8,
    status: u8,
    out_buf: *mut u8,
    out_buf_len: size_t,
) -> c_int {
    if out_buf.is_null() {
        return -1;
    }

    let msg = SynapseMessage::Supervisor(SupervisorMessage::ServiceHeartbeat {
        service_id,
        status,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
        error_msg: None,
    });

    let frame = msg.to_frame();
    let encoded = frame.encode_with_cobs();

    if encoded.len() > out_buf_len {
        return -1;
    }

    unsafe {
        std::ptr::copy_nonoverlapping(encoded.as_ptr(), out_buf, encoded.len());
    }

    encoded.len() as c_int
}

/// Build a raw frame with given msg_id and payload.
/// Returns the number of bytes written, or -1 on error.
#[no_mangle]
pub extern "C" fn synapse_build_frame(
    msg_id: u16,
    payload: *const u8,
    payload_len: size_t,
    out_buf: *mut u8,
    out_buf_len: size_t,
) -> c_int {
    if out_buf.is_null() || (payload.is_null() && payload_len > 0) {
        return -1;
    }

    let payload_slice = if payload_len > 0 {
        unsafe { slice::from_raw_parts(payload, payload_len) }
    } else {
        &[]
    };

    let frame = SynapseFrame::new(msg_id, payload_slice.to_vec());
    let encoded = frame.encode_with_cobs();

    if encoded.len() > out_buf_len {
        return -1;
    }

    unsafe {
        std::ptr::copy_nonoverlapping(encoded.as_ptr(), out_buf, encoded.len());
    }

    encoded.len() as c_int
}

// Service ID constants for C
#[no_mangle]
pub static SYNAPSE_SERVICE_VIO: u8 = SERVICE_VIO;

#[no_mangle]
pub static SYNAPSE_STATUS_RUNNING: u8 = STATUS_RUNNING;

#[no_mangle]
pub static SYNAPSE_STATUS_INIT: u8 = 0;