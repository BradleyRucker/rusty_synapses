//! Python bindings for Synapse protocol via PyO3
//!
//! Build with: maturin build --features python
//! Install with: pip install .

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use crate::{cobs, crc, frame, messages};

/// Python wrapper for SynapseFrame
#[pyclass(name = "SynapseFrame")]
#[derive(Clone)]
pub struct PySynapseFrame {
    inner: frame::SynapseFrame,
}

#[pymethods]
impl PySynapseFrame {
    /// Create a new Synapse frame with message ID and payload
    #[new]
    #[pyo3(signature = (msg_id, payload=vec![]))]
    fn new(msg_id: u16, payload: Vec<u8>) -> Self {
        PySynapseFrame {
            inner: frame::SynapseFrame::new(msg_id, payload),
        }
    }

    /// Set sequence number
    fn with_sequence(&self, seq: u16) -> Self {
        PySynapseFrame {
            inner: self.inner.clone().with_sequence(seq),
        }
    }

    /// Set routing endpoints
    fn with_routing(&self, src: u16, dst: u16) -> Self {
        PySynapseFrame {
            inner: self.inner.clone().with_routing(src, dst),
        }
    }

    /// Encode frame to bytes (without COBS)
    fn encode(&self) -> Vec<u8> {
        self.inner.encode()
    }

    /// Encode frame with COBS framing (ready for wire)
    fn encode_with_cobs(&self) -> Vec<u8> {
        self.inner.encode_with_cobs()
    }

    /// Parse a frame from raw bytes (after COBS decoding)
    #[staticmethod]
    fn parse(data: &[u8]) -> PyResult<Self> {
        frame::SynapseFrame::parse(data)
            .map(|f| PySynapseFrame { inner: f })
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Parse a COBS-encoded frame
    #[staticmethod]
    fn parse_cobs(data: &[u8]) -> PyResult<Self> {
        frame::SynapseFrame::parse_cobs(data)
            .map(|f| PySynapseFrame { inner: f })
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    // Properties
    #[getter]
    fn msg_id(&self) -> u16 {
        self.inner.msg_id
    }

    #[getter]
    fn sequence(&self) -> u16 {
        self.inner.sequence
    }

    #[getter]
    fn timestamp_ms(&self) -> u32 {
        self.inner.timestamp_ms
    }

    #[getter]
    fn payload(&self) -> Vec<u8> {
        self.inner.payload.clone()
    }

    #[getter]
    fn flags(&self) -> u8 {
        self.inner.flags
    }

    fn __repr__(&self) -> String {
        format!(
            "SynapseFrame(msg_id=0x{:04X}, seq={}, payload_len={})",
            self.inner.msg_id, self.inner.sequence, self.inner.payload.len()
        )
    }
}

// ============================================================================
// Message Constants
// ============================================================================

/// Cortex message IDs
#[pyclass(name = "MsgId")]
pub struct PyMsgId;

#[pymethods]
impl PyMsgId {
    #[classattr]
    const REGISTER: u16 = messages::MSG_REGISTER;
    #[classattr]
    const SERVICE_HEARTBEAT: u16 = messages::MSG_SERVICE_HEARTBEAT;
    #[classattr]
    const SUPERVISOR_CMD: u16 = messages::MSG_SUPERVISOR_CMD;
    #[classattr]
    const SUPERVISOR_CMD_ACK: u16 = messages::MSG_SUPERVISOR_CMD_ACK;
    #[classattr]
    const SUPERVISOR_STATE: u16 = messages::MSG_SUPERVISOR_STATE;
    #[classattr]
    const SERVICE_STATUS: u16 = messages::MSG_SERVICE_STATUS;
    #[classattr]
    const VIO_POSE: u16 = messages::MSG_VIO_POSE;
}

/// Service IDs
#[pyclass(name = "ServiceId")]
pub struct PyServiceId;

#[pymethods]
impl PyServiceId {
    #[classattr]
    const VIO: u8 = messages::SERVICE_VIO;
    #[classattr]
    const PERCEPTION: u8 = messages::SERVICE_PERCEPTION;
    #[classattr]
    const PLANNER: u8 = messages::SERVICE_PLANNER;
    #[classattr]
    const SYNAPSE: u8 = messages::SERVICE_SYNAPSE;
}

/// Service status codes
#[pyclass(name = "Status")]
pub struct PyStatus;

#[pymethods]
impl PyStatus {
    #[classattr]
    const STARTING: u8 = messages::STATUS_STARTING;
    #[classattr]
    const RUNNING: u8 = messages::STATUS_RUNNING;
    #[classattr]
    const DEGRADED: u8 = messages::STATUS_DEGRADED;
    #[classattr]
    const ERROR: u8 = messages::STATUS_ERROR;
}

/// Supervisor commands
#[pyclass(name = "Cmd")]
pub struct PyCmd;

#[pymethods]
impl PyCmd {
    #[classattr]
    const SHUTDOWN: u8 = messages::CMD_SHUTDOWN;
    #[classattr]
    const RESTART: u8 = messages::CMD_RESTART;
    #[classattr]
    const PAUSE: u8 = messages::CMD_PAUSE;
    #[classattr]
    const RESUME: u8 = messages::CMD_RESUME;
}

// ============================================================================
// Message Builder Functions
// ============================================================================

/// Create a Register message frame
#[pyfunction]
fn make_register(service_id: u8, pid: u32, version: &str) -> PySynapseFrame {
    let msg = messages::SynapseMessage::Register {
        service_id,
        pid,
        version: version.to_string(),
    };
    PySynapseFrame {
        inner: msg.to_frame(),
    }
}

/// Create a ServiceHeartbeat message frame
#[pyfunction]
#[pyo3(signature = (service_id, status, error_msg=None))]
fn make_heartbeat(service_id: u8, status: u8, error_msg: Option<&str>) -> PySynapseFrame {
    let msg = messages::SynapseMessage::ServiceHeartbeat {
        service_id,
        status,
        timestamp: 0, // Will be set by frame
        error_msg: error_msg.map(|s| s.to_string()),
    };
    PySynapseFrame {
        inner: msg.to_frame(),
    }
}

/// Create a SupervisorCmdAck message frame
#[pyfunction]
#[pyo3(signature = (command_id, success, error_msg=None))]
fn make_cmd_ack(command_id: u32, success: bool, error_msg: Option<&str>) -> PySynapseFrame {
    let msg = messages::SynapseMessage::SupervisorCmdAck {
        command_id,
        success,
        error_msg: error_msg.map(|s| s.to_string()),
    };
    PySynapseFrame {
        inner: msg.to_frame(),
    }
}

// ============================================================================
// Low-level utilities
// ============================================================================

/// COBS encode data
#[pyfunction]
fn cobs_encode(data: &[u8]) -> Vec<u8> {
    cobs::encode(data)
}

/// COBS decode data
#[pyfunction]
fn cobs_decode(data: &[u8]) -> PyResult<Vec<u8>> {
    cobs::decode(data).map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Calculate CRC8 (for header validation)
#[pyfunction]
fn calc_crc8(data: &[u8]) -> u8 {
    crc::crc8(data)
}

/// Calculate CRC16 (for payload validation)
#[pyfunction]
fn calc_crc16(data: &[u8]) -> u16 {
    crc::crc16(data)
}

// ============================================================================
// Python Module
// ============================================================================

/// Synapse protocol module for Python
#[pymodule]
#[pyo3(name = "synapse")]
fn synapse_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Frame class
    m.add_class::<PySynapseFrame>()?;

    // Constants
    m.add_class::<PyMsgId>()?;
    m.add_class::<PyServiceId>()?;
    m.add_class::<PyStatus>()?;
    m.add_class::<PyCmd>()?;

    // Message builders
    m.add_function(wrap_pyfunction!(make_register, m)?)?;
    m.add_function(wrap_pyfunction!(make_heartbeat, m)?)?;
    m.add_function(wrap_pyfunction!(make_cmd_ack, m)?)?;

    // Low-level utilities
    m.add_function(wrap_pyfunction!(cobs_encode, m)?)?;
    m.add_function(wrap_pyfunction!(cobs_decode, m)?)?;
    m.add_function(wrap_pyfunction!(calc_crc8, m)?)?;
    m.add_function(wrap_pyfunction!(calc_crc16, m)?)?;

    Ok(())
}