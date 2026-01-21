use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use crate::frame::SynapseFrame;

// ============================================================================
// Message IDs
// ============================================================================

// System Messages (0x0001 - 0x00FF)
pub const MSG_HEARTBEAT: u16 = 0x0001;
pub const MSG_TIME_SYNC: u16 = 0x0002;
pub const MSG_PING: u16 = 0x0003;
pub const MSG_PONG: u16 = 0x0004;
pub const MSG_ACK: u16 = 0x0005;

// Sensor Data Messages (0x0100 - 0x01FF)
pub const MSG_IMU_DATA: u16 = 0x0100;
pub const MSG_GPS_DATA: u16 = 0x0101;
pub const MSG_TOF_DATA: u16 = 0x0102;

// State Estimation Messages (0x0200 - 0x02FF)
pub const MSG_FUSED_STATE: u16 = 0x0200;

// System Status Messages (0x0300 - 0x03FF)
pub const MSG_SYSTEM_STATUS: u16 = 0x0301;

// Command Messages (0x0400 - 0x04FF)
pub const MSG_ARM_DISARM: u16 = 0x0400;
pub const MSG_SET_MODE: u16 = 0x0401;

// Telemetry/Config (0x0500 - 0x05FF)
pub const MSG_CONFIG_PARAM: u16 = 0x0500;
pub const MSG_PARAMETER_ACK: u16 = 0x0501;
pub const MSG_REQUEST_PARAM: u16 = 0x0502;
pub const MSG_REQUEST_PARAM_LIST: u16 = 0x0503;
pub const MSG_SAVE_PARAMS: u16 = 0x0504;

// HIL Messages (0x0600 - 0x06FF)
pub const MSG_EMERGENCY_STOP: u16 = 0x0613;

// ============================================================================
// Cortex Supervisor Messages (0x0700 - 0x07FF)
// ============================================================================

/// Service registration with supervisor
pub const MSG_REGISTER: u16 = 0x0700;

/// Service heartbeat to supervisor
pub const MSG_SERVICE_HEARTBEAT: u16 = 0x0701;

/// Supervisor command to service
pub const MSG_SUPERVISOR_CMD: u16 = 0x0702;

/// Service acknowledgment of command
pub const MSG_SUPERVISOR_CMD_ACK: u16 = 0x0703;

/// Supervisor state broadcast
pub const MSG_SUPERVISOR_STATE: u16 = 0x0704;

/// Service status report
pub const MSG_SERVICE_STATUS: u16 = 0x0705;

// ============================================================================
// VIO Messages (0x0710 - 0x071F)
// ============================================================================

/// VIO pose output (position, orientation, velocity)
pub const MSG_VIO_POSE: u16 = 0x0710;

// ============================================================================
// Cortex Service IDs
// ============================================================================

pub const SERVICE_VIO: u8 = 0;
pub const SERVICE_PERCEPTION: u8 = 1;
pub const SERVICE_PLANNER: u8 = 2;
pub const SERVICE_SYNAPSE: u8 = 3;
pub const SERVICE_CAMERA: u8 = 4;

// ============================================================================
// Cortex Service Status
// ============================================================================

pub const STATUS_STARTING: u8 = 0;
pub const STATUS_RUNNING: u8 = 1;
pub const STATUS_DEGRADED: u8 = 2;
pub const STATUS_ERROR: u8 = 3;

// ============================================================================
// Cortex Supervisor Commands
// ============================================================================

pub const CMD_SHUTDOWN: u8 = 0;
pub const CMD_RESTART: u8 = 1;
pub const CMD_PAUSE: u8 = 2;
pub const CMD_RESUME: u8 = 3;

// ============================================================================
// Common Data Structures
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EulerAngles {
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
}

// ============================================================================
// Synapse Message Types
// ============================================================================

/// Synapse message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SynapseMessage {
    // ---- Flight Controller Messages ----

    #[serde(rename = "imu_data")]
    ImuData {
        gyro: Vector3,
        accel: Vector3,
        mag: Vector3,
        timestamp: u64,
    },

    #[serde(rename = "gps_data")]
    GpsData {
        lat: f64,
        lon: f64,
        altitude: f32,
        speed: f32,
        heading: f32,
        satellites: u8,
        timestamp: u64,
    },

    #[serde(rename = "tof_data")]
    ToFData {
        sensor_id: u8,
        resolution: u8,
        min_distance: u16,
        avg_distance: u16,
        max_distance: u16,
        timestamp: u64,
    },

    #[serde(rename = "fused_state")]
    FusedState {
        euler: EulerAngles,
        timestamp: u64,
    },

    #[serde(rename = "system_status")]
    SystemStatus {
        armed: u8,
        system_state: u8,
        battery_voltage: f32,
        loop_rate: u16,
        timestamp: u64,
    },

    #[serde(rename = "parameter_value")]
    ParameterValue {
        #[serde(rename = "paramId")]
        param_id: u16,
        value: f32,
        timestamp: u64,
    },

    #[serde(rename = "parameter_ack")]
    ParameterAck {
        #[serde(rename = "paramId")]
        param_id: u16,
        success: bool,
        timestamp: u64,
    },

    #[serde(rename = "save_params_ack")]
    SaveParamsAck {
        success: bool,
        timestamp: u64,
    },

    // ---- Cortex Supervisor Messages ----

    #[serde(rename = "register")]
    Register {
        service_id: u8,
        pid: u32,
        version: String,
    },

    #[serde(rename = "service_heartbeat")]
    ServiceHeartbeat {
        service_id: u8,
        status: u8,
        timestamp: u64,
        error_msg: Option<String>,
    },

    #[serde(rename = "supervisor_cmd")]
    SupervisorCmd {
        command_id: u32,
        command_type: u8,
        target_service: u8,
    },

    #[serde(rename = "supervisor_cmd_ack")]
    SupervisorCmdAck {
        command_id: u32,
        success: bool,
        error_msg: Option<String>,
    },

    #[serde(rename = "supervisor_state")]
    SupervisorState {
        state_id: u8,
        services_up: u8,
        services_healthy: u8,
        timestamp: u64,
    },

    #[serde(rename = "service_status")]
    ServiceStatusReport {
        service_id: u8,
        status: u8,
        cpu_percent: f32,
        memory_bytes: u64,
        timestamp: u64,
    },

    // ---- VIO Messages ----

    #[serde(rename = "vio_pose")]
    VioPose {
        timestamp_ns: u64,
        position: [f64; 3],      // x, y, z in meters
        quaternion: [f64; 4],    // w, x, y, z
        velocity: [f64; 3],      // vx, vy, vz in m/s
        initialized: bool,
    },
}

impl SynapseMessage {
    /// Parse a Synapse message from a frame
    pub fn from_frame(frame: &SynapseFrame) -> Result<Self> {
        let payload = &frame.payload;
        let timestamp = frame.timestamp_ms as u64;

        match frame.msg_id {
            // System messages - handle silently
            MSG_HEARTBEAT | MSG_TIME_SYNC | MSG_PING | MSG_PONG | MSG_ACK => {
                Err(anyhow!("System message (no telemetry)"))
            }

            MSG_IMU_DATA => {
                if payload.len() < 36 {
                    return Err(anyhow!("IMU data payload too short"));
                }

                Ok(SynapseMessage::ImuData {
                    gyro: Vector3 {
                        x: f32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]),
                        y: f32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]),
                        z: f32::from_le_bytes([payload[8], payload[9], payload[10], payload[11]]),
                    },
                    accel: Vector3 {
                        x: f32::from_le_bytes([payload[12], payload[13], payload[14], payload[15]]),
                        y: f32::from_le_bytes([payload[16], payload[17], payload[18], payload[19]]),
                        z: f32::from_le_bytes([payload[20], payload[21], payload[22], payload[23]]),
                    },
                    mag: Vector3 {
                        x: f32::from_le_bytes([payload[24], payload[25], payload[26], payload[27]]),
                        y: f32::from_le_bytes([payload[28], payload[29], payload[30], payload[31]]),
                        z: f32::from_le_bytes([payload[32], payload[33], payload[34], payload[35]]),
                    },
                    timestamp,
                })
            }

            MSG_GPS_DATA => {
                if payload.len() < 29 {
                    return Err(anyhow!("GPS data payload too short"));
                }

                Ok(SynapseMessage::GpsData {
                    lat: f64::from_le_bytes([
                        payload[0], payload[1], payload[2], payload[3],
                        payload[4], payload[5], payload[6], payload[7],
                    ]),
                    lon: f64::from_le_bytes([
                        payload[8], payload[9], payload[10], payload[11],
                        payload[12], payload[13], payload[14], payload[15],
                    ]),
                    altitude: f32::from_le_bytes([payload[16], payload[17], payload[18], payload[19]]),
                    speed: f32::from_le_bytes([payload[20], payload[21], payload[22], payload[23]]),
                    heading: f32::from_le_bytes([payload[24], payload[25], payload[26], payload[27]]),
                    satellites: payload[28],
                    timestamp,
                })
            }

            MSG_TOF_DATA => {
                if payload.len() < 8 {
                    return Err(anyhow!("ToF data payload too short"));
                }

                Ok(SynapseMessage::ToFData {
                    sensor_id: payload[0],
                    resolution: payload[1],
                    min_distance: u16::from_le_bytes([payload[2], payload[3]]),
                    avg_distance: u16::from_le_bytes([payload[4], payload[5]]),
                    max_distance: u16::from_le_bytes([payload[6], payload[7]]),
                    timestamp,
                })
            }

            MSG_FUSED_STATE => {
                if payload.len() < 12 {
                    return Err(anyhow!("Fused state payload too short"));
                }

                Ok(SynapseMessage::FusedState {
                    euler: EulerAngles {
                        roll: f32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]),
                        pitch: f32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]),
                        yaw: f32::from_le_bytes([payload[8], payload[9], payload[10], payload[11]]),
                    },
                    timestamp,
                })
            }

            MSG_SYSTEM_STATUS => {
                if payload.len() < 8 {
                    return Err(anyhow!("System status payload too short"));
                }

                Ok(SynapseMessage::SystemStatus {
                    armed: payload[0],
                    system_state: payload[1],
                    battery_voltage: f32::from_le_bytes([payload[2], payload[3], payload[4], payload[5]]),
                    loop_rate: u16::from_le_bytes([payload[6], payload[7]]),
                    timestamp,
                })
            }

            MSG_CONFIG_PARAM => {
                if payload.len() < 6 {
                    return Err(anyhow!("Config param payload too short"));
                }

                Ok(SynapseMessage::ParameterValue {
                    param_id: u16::from_le_bytes([payload[0], payload[1]]),
                    value: f32::from_le_bytes([payload[2], payload[3], payload[4], payload[5]]),
                    timestamp,
                })
            }

            MSG_PARAMETER_ACK => {
                if payload.len() >= 3 {
                    Ok(SynapseMessage::ParameterAck {
                        param_id: u16::from_le_bytes([payload[0], payload[1]]),
                        success: payload[2] != 0,
                        timestamp,
                    })
                } else if payload.len() == 1 {
                    Ok(SynapseMessage::SaveParamsAck {
                        success: payload[0] != 0,
                        timestamp,
                    })
                } else {
                    Err(anyhow!("Parameter ACK payload invalid length"))
                }
            }

            // ---- Cortex Supervisor Messages ----

            MSG_REGISTER => {
                if payload.len() < 5 {
                    return Err(anyhow!("Register payload too short"));
                }

                let service_id = payload[0];
                let pid = u32::from_le_bytes([payload[1], payload[2], payload[3], payload[4]]);
                let version = if payload.len() > 5 {
                    String::from_utf8_lossy(&payload[5..]).to_string()
                } else {
                    String::new()
                };

                Ok(SynapseMessage::Register { service_id, pid, version })
            }

            MSG_SERVICE_HEARTBEAT => {
                if payload.len() < 2 {
                    return Err(anyhow!("Service heartbeat payload too short"));
                }

                let service_id = payload[0];
                let status = payload[1];
                let error_msg = if payload.len() > 2 {
                    Some(String::from_utf8_lossy(&payload[2..]).to_string())
                } else {
                    None
                };

                Ok(SynapseMessage::ServiceHeartbeat {
                    service_id,
                    status,
                    timestamp,
                    error_msg,
                })
            }

            MSG_SUPERVISOR_CMD => {
                if payload.len() < 6 {
                    return Err(anyhow!("Supervisor command payload too short"));
                }

                Ok(SynapseMessage::SupervisorCmd {
                    command_id: u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]),
                    command_type: payload[4],
                    target_service: payload[5],
                })
            }

            MSG_SUPERVISOR_CMD_ACK => {
                if payload.len() < 5 {
                    return Err(anyhow!("Supervisor command ACK payload too short"));
                }

                let command_id = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
                let success = payload[4] != 0;
                let error_msg = if payload.len() > 5 {
                    Some(String::from_utf8_lossy(&payload[5..]).to_string())
                } else {
                    None
                };

                Ok(SynapseMessage::SupervisorCmdAck { command_id, success, error_msg })
            }

            MSG_SUPERVISOR_STATE => {
                if payload.len() < 3 {
                    return Err(anyhow!("Supervisor state payload too short"));
                }

                Ok(SynapseMessage::SupervisorState {
                    state_id: payload[0],
                    services_up: payload[1],
                    services_healthy: payload[2],
                    timestamp,
                })
            }

            MSG_SERVICE_STATUS => {
                if payload.len() < 14 {
                    return Err(anyhow!("Service status payload too short"));
                }

                Ok(SynapseMessage::ServiceStatusReport {
                    service_id: payload[0],
                    status: payload[1],
                    cpu_percent: f32::from_le_bytes([payload[2], payload[3], payload[4], payload[5]]),
                    memory_bytes: u64::from_le_bytes([
                        payload[6], payload[7], payload[8], payload[9],
                        payload[10], payload[11], payload[12], payload[13],
                    ]),
                    timestamp,
                })
            }

            MSG_VIO_POSE => {
                // 8 (timestamp) + 3*8 (pos) + 4*8 (quat) + 3*8 (vel) + 1 (init) = 89 bytes
                if payload.len() < 89 {
                    return Err(anyhow!("VIO pose payload too short: {} < 89", payload.len()));
                }

                let timestamp_ns = u64::from_le_bytes([
                    payload[0], payload[1], payload[2], payload[3],
                    payload[4], payload[5], payload[6], payload[7],
                ]);

                let position = [
                    f64::from_le_bytes(payload[8..16].try_into().unwrap()),
                    f64::from_le_bytes(payload[16..24].try_into().unwrap()),
                    f64::from_le_bytes(payload[24..32].try_into().unwrap()),
                ];

                let quaternion = [
                    f64::from_le_bytes(payload[32..40].try_into().unwrap()),  // w
                    f64::from_le_bytes(payload[40..48].try_into().unwrap()),  // x
                    f64::from_le_bytes(payload[48..56].try_into().unwrap()),  // y
                    f64::from_le_bytes(payload[56..64].try_into().unwrap()),  // z
                ];

                let velocity = [
                    f64::from_le_bytes(payload[64..72].try_into().unwrap()),
                    f64::from_le_bytes(payload[72..80].try_into().unwrap()),
                    f64::from_le_bytes(payload[80..88].try_into().unwrap()),
                ];

                let initialized = payload[88] != 0;

                Ok(SynapseMessage::VioPose {
                    timestamp_ns,
                    position,
                    quaternion,
                    velocity,
                    initialized,
                })
            }

            _ => Err(anyhow!("Unknown message ID: 0x{:04X}", frame.msg_id)),
        }
    }

    /// Encode a message to a payload (bytes)
    pub fn to_payload(&self) -> Vec<u8> {
        match self {
            SynapseMessage::Register { service_id, pid, version } => {
                let mut payload = vec![*service_id];
                payload.extend_from_slice(&pid.to_le_bytes());
                payload.extend_from_slice(version.as_bytes());
                payload
            }

            SynapseMessage::ServiceHeartbeat { service_id, status, error_msg, .. } => {
                let mut payload = vec![*service_id, *status];
                if let Some(msg) = error_msg {
                    payload.extend_from_slice(msg.as_bytes());
                }
                payload
            }

            SynapseMessage::SupervisorCmd { command_id, command_type, target_service } => {
                let mut payload = Vec::with_capacity(6);
                payload.extend_from_slice(&command_id.to_le_bytes());
                payload.push(*command_type);
                payload.push(*target_service);
                payload
            }

            SynapseMessage::SupervisorCmdAck { command_id, success, error_msg } => {
                let mut payload = Vec::with_capacity(5);
                payload.extend_from_slice(&command_id.to_le_bytes());
                payload.push(if *success { 1 } else { 0 });
                if let Some(msg) = error_msg {
                    payload.extend_from_slice(msg.as_bytes());
                }
                payload
            }

            SynapseMessage::SupervisorState { state_id, services_up, services_healthy, .. } => {
                vec![*state_id, *services_up, *services_healthy]
            }

            SynapseMessage::ServiceStatusReport { service_id, status, cpu_percent, memory_bytes, .. } => {
                let mut payload = vec![*service_id, *status];
                payload.extend_from_slice(&cpu_percent.to_le_bytes());
                payload.extend_from_slice(&memory_bytes.to_le_bytes());
                payload
            }

            SynapseMessage::VioPose {
                timestamp_ns,
                position,
                quaternion,
                velocity,
                initialized,
            } => {
                let mut payload = Vec::with_capacity(89);
                payload.extend_from_slice(&timestamp_ns.to_le_bytes());
                payload.extend_from_slice(&position[0].to_le_bytes());
                payload.extend_from_slice(&position[1].to_le_bytes());
                payload.extend_from_slice(&position[2].to_le_bytes());
                payload.extend_from_slice(&quaternion[0].to_le_bytes());
                payload.extend_from_slice(&quaternion[1].to_le_bytes());
                payload.extend_from_slice(&quaternion[2].to_le_bytes());
                payload.extend_from_slice(&quaternion[3].to_le_bytes());
                payload.extend_from_slice(&velocity[0].to_le_bytes());
                payload.extend_from_slice(&velocity[1].to_le_bytes());
                payload.extend_from_slice(&velocity[2].to_le_bytes());
                payload.push(if *initialized { 1 } else { 0 });
                payload
            }

            // Other message types not yet implemented for encoding
            _ => vec![],
        }
    }

    /// Get the message ID for this message type
    pub fn msg_id(&self) -> u16 {
        match self {
            SynapseMessage::ImuData { .. } => MSG_IMU_DATA,
            SynapseMessage::GpsData { .. } => MSG_GPS_DATA,
            SynapseMessage::ToFData { .. } => MSG_TOF_DATA,
            SynapseMessage::FusedState { .. } => MSG_FUSED_STATE,
            SynapseMessage::SystemStatus { .. } => MSG_SYSTEM_STATUS,
            SynapseMessage::ParameterValue { .. } => MSG_CONFIG_PARAM,
            SynapseMessage::ParameterAck { .. } => MSG_PARAMETER_ACK,
            SynapseMessage::SaveParamsAck { .. } => MSG_PARAMETER_ACK,
            SynapseMessage::Register { .. } => MSG_REGISTER,
            SynapseMessage::ServiceHeartbeat { .. } => MSG_SERVICE_HEARTBEAT,
            SynapseMessage::SupervisorCmd { .. } => MSG_SUPERVISOR_CMD,
            SynapseMessage::SupervisorCmdAck { .. } => MSG_SUPERVISOR_CMD_ACK,
            SynapseMessage::SupervisorState { .. } => MSG_SUPERVISOR_STATE,
            SynapseMessage::ServiceStatusReport { .. } => MSG_SERVICE_STATUS,
            SynapseMessage::VioPose { .. } => MSG_VIO_POSE,
        }
    }

    /// Convert message to a SynapseFrame ready for sending
    pub fn to_frame(&self) -> SynapseFrame {
        SynapseFrame::new(self.msg_id(), self.to_payload())
    }
}
