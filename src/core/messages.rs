use anyhow::{anyhow, Result};

use crate::core::frame::SynapseFrame;
use crate::endpoint::NodeId;

// ============================================================================
// Message IDs
// ============================================================================

// System Messages (0x0001 - 0x00FF)
pub const MSG_HEARTBEAT: u16 = 0x0001;
pub const MSG_TIME_SYNC: u16 = 0x0002;
pub const MSG_PING: u16 = 0x0003;
pub const MSG_PONG: u16 = 0x0004;

// ROUGE Mission / Network Messages (0x0800 - 0x08FF)
pub const MSG_NODE_HEARTBEAT: u16 = 0x0801;
pub const MSG_LINK_STATUS: u16 = 0x0802;
pub const MSG_MISSION_ROLE: u16 = 0x0803;

pub const MSG_TARGET_CUE: u16 = 0x0810;
pub const MSG_HANDOFF_REQUEST: u16 = 0x0811;
pub const MSG_HANDOFF_ACCEPT: u16 = 0x0812;
pub const MSG_HANDOFF_REJECT: u16 = 0x0813;

pub const MSG_ACK_RECEIVED: u16 = 0x08F0;
pub const MSG_ACK_ACCEPTED: u16 = 0x08F1;
pub const MSG_ACK_REJECTED: u16 = 0x08F2;

// Optional Supervisor / Service Messages (0x0700 - 0x07FF)
pub const MSG_REGISTER: u16 = 0x0700;
pub const MSG_SERVICE_HEARTBEAT: u16 = 0x0701;
pub const MSG_SUPERVISOR_CMD: u16 = 0x0702;
pub const MSG_SUPERVISOR_CMD_ACK: u16 = 0x0703;
pub const MSG_SUPERVISOR_STATE: u16 = 0x0704;
pub const MSG_SERVICE_STATUS: u16 = 0x0705;

// ============================================================================
// Optional Supervisor Constants
// ============================================================================

pub const SERVICE_VIO: u8 = 0;
pub const SERVICE_PERCEPTION: u8 = 1;
pub const SERVICE_PLANNER: u8 = 2;
pub const SERVICE_SYNAPSE: u8 = 3;
pub const SERVICE_CAMERA: u8 = 4;

pub const STATUS_STARTING: u8 = 0;
pub const STATUS_RUNNING: u8 = 1;
pub const STATUS_DEGRADED: u8 = 2;
pub const STATUS_ERROR: u8 = 3;

pub const CMD_SHUTDOWN: u8 = 0;
pub const CMD_RESTART: u8 = 1;
pub const CMD_PAUSE: u8 = 2;
pub const CMD_RESUME: u8 = 3;

// ============================================================================
// Common Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRole {
    FixedWing = 1,
    Fpv = 2,
    Gcs = 3,
    Relay = 4,
}

impl NodeRole {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(role: u8) -> Option<Self> {
        match role {
            1 => Some(Self::FixedWing),
            2 => Some(Self::Fpv),
            3 => Some(Self::Gcs),
            4 => Some(Self::Relay),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionRole {
    Scout = 1,
    Interceptor = 2,
    Relay = 3,
    Overwatch = 4,
    GroundControl = 5,
}

impl MissionRole {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(role: u8) -> Option<Self> {
        match role {
            1 => Some(Self::Scout),
            2 => Some(Self::Interceptor),
            3 => Some(Self::Relay),
            4 => Some(Self::Overwatch),
            5 => Some(Self::GroundControl),
            _ => None,
        }
    }
}

// ============================================================================
// System Messages
// ============================================================================

#[derive(Debug, Clone, )]
pub enum SystemMessage {
    Heartbeat,
    TimeSync { time_ms: u64 },
    Ping,
    Pong,
}

// ============================================================================
// ROUGE Messages
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeHeartbeat {
    pub node: NodeId,
    pub role: NodeRole,
    pub uptime_ms: u32,
    pub health: u8,
    pub capabilities: u16,
}

impl NodeHeartbeat {
    pub const PAYLOAD_LEN: usize = 9;

    pub fn new(
        node: NodeId,
        role: NodeRole,
        uptime_ms: u32,
        health: u8,
        capabilities: u16,
    ) -> Self {
        Self {
            node,
            role,
            uptime_ms,
            health,
            capabilities,
        }
    }

    pub fn to_payload_bytes(&self) -> [u8; Self::PAYLOAD_LEN] {
        let mut buf = [0u8; Self::PAYLOAD_LEN];
        buf[0] = self.node.0;
        buf[1] = self.role.as_u8();
        buf[2..6].copy_from_slice(&self.uptime_ms.to_le_bytes());
        buf[6] = self.health;
        buf[7..9].copy_from_slice(&self.capabilities.to_le_bytes());
        buf
    }

    pub fn from_payload_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::PAYLOAD_LEN {
            return None;
        }

        Some(Self {
            node: NodeId(bytes[0]),
            role: NodeRole::from_u8(bytes[1])?,
            uptime_ms: u32::from_le_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]),
            health: bytes[6],
            capabilities: u16::from_le_bytes([bytes[7], bytes[8]]),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinkStatus {
    pub node: NodeId,
    pub rssi_dbm: i16,
    pub lq_percent: u8,
    pub snr_db: i8,
    pub timestamp_ms: u32,
}

impl LinkStatus {
    pub const PAYLOAD_LEN: usize = 9;

    pub fn new(
        node: NodeId,
        rssi_dbm: i16,
        lq_percent: u8,
        snr_db: i8,
        timestamp_ms: u32,
    ) -> Self {
        Self{
            node,
            rssi_dbm,
            lq_percent,
            snr_db,
            timestamp_ms,
        }
    }
    pub fn to_payload_bytes(&self) -> [u8; Self::PAYLOAD_LEN] {
        let mut buf = [0u8; Self::PAYLOAD_LEN];
        buf[0] = self.node.0;
        buf[1..3].copy_from_slice(&self.rssi_dbm.to_le_bytes());
        buf[3] = self.lq_percent;
        buf[4] = self.snr_db as u8;
        buf[5..9].copy_from_slice(&self.timestamp_ms.to_le_bytes());
        buf
    }

    pub fn from_payload_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::PAYLOAD_LEN {
            return None;
        }

        Some(Self {
            node: NodeId(bytes[0]),
            rssi_dbm: i16::from_le_bytes([bytes[1], bytes[2]]),
            lq_percent: bytes[3],
            snr_db: bytes[4] as i8,
            timestamp_ms: u32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissionRoleState {
    pub node: NodeId,
    pub role: MissionRole,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TargetCue {
    pub source_node: NodeId,
    pub target_id: u16,
    pub x_m: f32,
    pub y_m: f32,
    pub z_m: f32,
    pub confidence: u8,
    pub urgency: u8,
    pub timestamp_ms: u32,
}

impl TargetCue {
    pub const PAYLOAD_LEN: usize = 21;

    pub fn new(
        source_node: NodeId,
        target_id: u16,
        x_m: f32,
        y_m: f32,
        z_m: f32,
        confidence: u8,
        urgency: u8,
        timestamp_ms: u32,
    ) -> Self {
        Self {
            source_node,
            target_id,
            x_m,
            y_m,
            z_m,
            confidence,
            urgency,
            timestamp_ms,
        }
    }

    pub fn to_payload_bytes(&self) -> [u8; Self::PAYLOAD_LEN] {
        let mut buf = [0u8; Self::PAYLOAD_LEN];

        buf[0] = self.source_node.0;
        buf[1..3].copy_from_slice(&self.target_id.to_le_bytes());
        buf[3..7].copy_from_slice(&self.x_m.to_le_bytes());
        buf[7..11].copy_from_slice(&self.y_m.to_le_bytes());
        buf[11..15].copy_from_slice(&self.z_m.to_le_bytes());
        buf[15] = self.confidence;
        buf[16] = self.urgency;
        buf[17..21].copy_from_slice(&self.timestamp_ms.to_le_bytes());

        buf
    }

    pub fn from_payload_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::PAYLOAD_LEN {
            return None;
        }

        Some(Self {
            source_node: NodeId(bytes[0]),
            target_id: u16::from_le_bytes([bytes[1], bytes[2]]),
            x_m: f32::from_le_bytes([bytes[3], bytes[4], bytes[5], bytes[6]]),
            y_m: f32::from_le_bytes([bytes[7], bytes[8], bytes[9], bytes[10]]),
            z_m: f32::from_le_bytes([bytes[11], bytes[12], bytes[13], bytes[14]]),
            confidence: bytes[15],
            urgency: bytes[16],
            timestamp_ms: u32::from_le_bytes([bytes[17], bytes[18], bytes[19], bytes[20]]),
        })
    }
}

impl MissionRoleState {
    pub const PAYLOAD_LEN: usize = 2;

    pub fn to_payload_bytes(&self) -> [u8; Self::PAYLOAD_LEN] {
        [self.node.0, self.role.as_u8()]
    }

    pub fn from_payload_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::PAYLOAD_LEN {
            return None;
        }

        Some(Self {
            node: NodeId(bytes[0]),
            role: MissionRole::from_u8(bytes[1])?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AckReceipt {
    pub original_msg_id: u16,
    pub original_seq: u16,
    pub from_node: NodeId,
}

impl AckReceipt {
    pub const PAYLOAD_LEN: usize = 5;

    pub fn to_payload_bytes(&self) -> [u8; Self::PAYLOAD_LEN] {
        let mut buf = [0u8; Self::PAYLOAD_LEN];
        buf[0..2].copy_from_slice(&self.original_msg_id.to_le_bytes());
        buf[2..4].copy_from_slice(&self.original_seq.to_le_bytes());
        buf[4] = self.from_node.0;
        buf
    }

    pub fn from_payload_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::PAYLOAD_LEN {
            return None;
        }

        Some(Self {
            original_msg_id: u16::from_le_bytes([bytes[0], bytes[1]]),
            original_seq: u16::from_le_bytes([bytes[2], bytes[3]]),
            from_node: NodeId(bytes[4]),
        })
    }
    pub fn for_frame(frame: &SynapseFrame, from_node: NodeId) -> Self {
        Self {
            original_msg_id: frame.msg_id,
            original_seq: frame.sequence,
            from_node,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AckReason {
    Accepted = 0,
    Busy = 1,
    Invalid = 2,
    Unauthorized = 3,
    Unsupported = 4,
}

impl AckReason {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Accepted),
            1 => Some(Self::Busy),
            2 => Some(Self::Invalid),
            3 => Some(Self::Unauthorized),
            4 => Some(Self::Unsupported),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AckDecision {
    pub original_msg_id: u16,
    pub original_seq: u16,
    pub from_node: NodeId,
    pub reason: AckReason,
}

impl AckDecision {
    pub const PAYLOAD_LEN: usize = 6;

    pub fn to_payload_bytes(&self) -> [u8; Self::PAYLOAD_LEN] {
        let mut buf = [0u8; Self::PAYLOAD_LEN];
        buf[0..2].copy_from_slice(&self.original_msg_id.to_le_bytes());
        buf[2..4].copy_from_slice(&self.original_seq.to_le_bytes());
        buf[4] = self.from_node.0;
        buf[5] = self.reason.as_u8();
        buf
    }

    pub fn from_payload_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::PAYLOAD_LEN {
            return None;
        }

        Some(Self {
            original_msg_id: u16::from_le_bytes([bytes[0], bytes[1]]),
            original_seq: u16::from_le_bytes([bytes[2], bytes[3]]),
            from_node: NodeId(bytes[4]),
            reason: AckReason::from_u8(bytes[5])?,
        })
    }

    pub fn accept_for_frame(frame: &SynapseFrame, from_node: NodeId) -> Self {
        Self {
            original_msg_id: frame.msg_id,
            original_seq: frame.sequence,
            from_node,
            reason: AckReason::Accepted,
        }
    }

    pub fn reject_for_frame(frame: &SynapseFrame, from_node: NodeId, reason: AckReason) -> Self {
        Self {
            original_msg_id: frame.msg_id,
            original_seq: frame.sequence,
            from_node,
            reason,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandoffRequest {
    pub source_node: NodeId,
    pub target_node: NodeId,
    pub target_id: u16,
    pub reason: u8,
    pub timestamp_ms: u32,
}

impl HandoffRequest {
    pub const PAYLOAD_LEN: usize = 9;

    pub fn new(
        source_node: NodeId,
        target_node: NodeId,
        target_id: u16,
        reason: u8,
        timestamp_ms: u32,
    ) -> Self {
        Self {
            source_node,
            target_node,
            target_id,
            reason,
            timestamp_ms,
        }
    }

    pub fn to_payload_bytes(&self) -> [u8; Self::PAYLOAD_LEN] {
        let mut buf = [0u8; Self::PAYLOAD_LEN];
        buf[0] = self.source_node.0;
        buf[1] = self.target_node.0;
        buf[2..4].copy_from_slice(&self.target_id.to_le_bytes());
        buf[4] = self.reason;
        buf[5..9].copy_from_slice(&self.timestamp_ms.to_le_bytes());
        buf
    }

    pub fn from_payload_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::PAYLOAD_LEN {
            return None;
        }

        Some(Self {
            source_node: NodeId(bytes[0]),
            target_node: NodeId(bytes[1]),
            target_id: u16::from_le_bytes([bytes[2], bytes[3]]),
            reason: bytes[4],
            timestamp_ms: u32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandoffAccept {
    pub source_node: NodeId,
    pub target_node: NodeId,
    pub target_id: u16,
    pub timestamp_ms: u32,
}

impl HandoffAccept {
    pub const PAYLOAD_LEN: usize = 8;

    pub fn new(source_node: NodeId, target_node: NodeId, target_id: u16, timestamp_ms: u32) -> Self {
        Self {
            source_node,
            target_node,
            target_id,
            timestamp_ms,
        }
    }

    pub fn to_payload_bytes(&self) -> [u8; Self::PAYLOAD_LEN] {
        let mut buf = [0u8; Self::PAYLOAD_LEN];
        buf[0] = self.source_node.0;
        buf[1] = self.target_node.0;
        buf[2..4].copy_from_slice(&self.target_id.to_le_bytes());
        buf[4..8].copy_from_slice(&self.timestamp_ms.to_le_bytes());
        buf
    }

    pub fn from_payload_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::PAYLOAD_LEN {
            return None;
        }

        Some(Self {
            source_node: NodeId(bytes[0]),
            target_node: NodeId(bytes[1]),
            target_id: u16::from_le_bytes([bytes[2], bytes[3]]),
            timestamp_ms: u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandoffReject {
    pub source_node: NodeId,
    pub target_node: NodeId,
    pub target_id: u16,
    pub reason: AckReason,
    pub timestamp_ms: u32,
}

impl HandoffReject {
    pub const PAYLOAD_LEN: usize = 9;

    pub fn new(
        source_node: NodeId,
        target_node: NodeId,
        target_id: u16,
        reason: AckReason,
        timestamp_ms: u32,
    ) -> Self {
        Self {
            source_node,
            target_node,
            target_id,
            reason,
            timestamp_ms,
        }
    }

    pub fn to_payload_bytes(&self) -> [u8; Self::PAYLOAD_LEN] {
        let mut buf = [0u8; Self::PAYLOAD_LEN];
        buf[0] = self.source_node.0;
        buf[1] = self.target_node.0;
        buf[2..4].copy_from_slice(&self.target_id.to_le_bytes());
        buf[4] = self.reason.as_u8();
        buf[5..9].copy_from_slice(&self.timestamp_ms.to_le_bytes());
        buf
    }

    pub fn from_payload_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::PAYLOAD_LEN {
            return None;
        }

        Some(Self {
            source_node: NodeId(bytes[0]),
            target_node: NodeId(bytes[1]),
            target_id: u16::from_le_bytes([bytes[2], bytes[3]]),
            reason: AckReason::from_u8(bytes[4])?,
            timestamp_ms: u32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]),
        })
    }
}

#[derive(Debug, Clone)]
pub enum RougeMessage {
    NodeHeartbeat(NodeHeartbeat),
    LinkStatus(LinkStatus),
    MissionRole(MissionRoleState),
    TargetCue(TargetCue),
    HandoffRequest(HandoffRequest),
    HandoffAccept(HandoffAccept),
    HandoffReject(HandoffReject),
    AckReceived(AckReceipt),
    AckAccepted(AckDecision),
    AckRejected(AckDecision),
}

// ============================================================================
// Optional Supervisor Messages
// ============================================================================

#[derive(Debug, Clone, )]
pub enum SupervisorMessage {
    Register {
        service_id: u8,
        pid: u32,
        version: String,
    },
    ServiceHeartbeat {
        service_id: u8,
        status: u8,
        timestamp: u64,
        error_msg: Option<String>,
    },
    SupervisorCmd {
        command_id: u32,
        command_type: u8,
        target_service: u8,
    },
    SupervisorCmdAck {
        command_id: u32,
        success: bool,
        error_msg: Option<String>,
    },
    SupervisorState {
        state_id: u8,
        services_up: u8,
        services_healthy: u8,
        timestamp: u64,
    },
    ServiceStatusReport {
        service_id: u8,
        status: u8,
        cpu_percent: f32,
        memory_bytes: u64,
        timestamp: u64,
    },
}

// ============================================================================
// Top-Level Message Type
// ============================================================================

#[derive(Debug, Clone, )]
pub enum SynapseMessage {
    System(SystemMessage),
    Rouge(RougeMessage),
    Supervisor(SupervisorMessage),
}

impl SynapseMessage {
    pub fn from_frame(frame: &SynapseFrame) -> Result<Self> {
        let payload = &frame.payload;
        let timestamp = frame.timestamp_ms as u64;

        match frame.msg_id {
            MSG_HEARTBEAT => Ok(Self::System(SystemMessage::Heartbeat)),
            MSG_TIME_SYNC => {
                if payload.len() < 8 {
                    return Err(anyhow!("Time sync payload too short"));
                }
                Ok(Self::System(SystemMessage::TimeSync {
                    time_ms: u64::from_le_bytes(payload[0..8].try_into().unwrap()),
                }))
            }
            MSG_PING => Ok(Self::System(SystemMessage::Ping)),
            MSG_PONG => Ok(Self::System(SystemMessage::Pong)),

            MSG_NODE_HEARTBEAT => Ok(Self::Rouge(
                RougeMessage::NodeHeartbeat(
                    NodeHeartbeat::from_payload_bytes(payload)
                        .ok_or_else(|| anyhow!("Invalid node heartbeat payload"))?,
                ),
            )),

            MSG_LINK_STATUS => Ok(Self::Rouge(
                RougeMessage::LinkStatus(
                    LinkStatus::from_payload_bytes(payload)
                        .ok_or_else(|| anyhow!("Invalid link status payload"))?,
                ),
            )),

            MSG_MISSION_ROLE => Ok(Self::Rouge(
                RougeMessage::MissionRole(
                    MissionRoleState::from_payload_bytes(payload)
                        .ok_or_else(|| anyhow!("Invalid mission role payload"))?,
                ),
            )),

            MSG_TARGET_CUE => Ok(Self::Rouge(
                RougeMessage::TargetCue(
                    TargetCue::from_payload_bytes(payload)
                        .ok_or_else(|| anyhow!("Invalid target cue payload"))?,
                ),
            )),

            MSG_HANDOFF_REQUEST => Ok(Self::Rouge(
                RougeMessage::HandoffRequest(
                    HandoffRequest::from_payload_bytes(payload)
                        .ok_or_else(|| anyhow!("Invalid handoff request payload"))?,
                ),
            )),

            MSG_HANDOFF_ACCEPT => Ok(Self::Rouge(
                RougeMessage::HandoffAccept(
                    HandoffAccept::from_payload_bytes(payload)
                        .ok_or_else(|| anyhow!("Invalid handoff accept payload"))?,
                ),
            )),

            MSG_HANDOFF_REJECT => Ok(Self::Rouge(
                RougeMessage::HandoffReject(
                    HandoffReject::from_payload_bytes(payload)
                        .ok_or_else(|| anyhow!("Invalid handoff reject payload"))?,
                ),
            )),

            MSG_ACK_RECEIVED => Ok(Self::Rouge(
                RougeMessage::AckReceived(
                    AckReceipt::from_payload_bytes(payload)
                        .ok_or_else(|| anyhow!("Invalid ACK_RECEIVED payload"))?,
                ),
            )),

            MSG_ACK_ACCEPTED => Ok(Self::Rouge(
                RougeMessage::AckAccepted(
                    AckDecision::from_payload_bytes(payload)
                        .ok_or_else(|| anyhow!("Invalid ACK_ACCEPTED payload"))?,
                ),
            )),

            MSG_ACK_REJECTED => Ok(Self::Rouge(
                RougeMessage::AckRejected(
                    AckDecision::from_payload_bytes(payload)
                        .ok_or_else(|| anyhow!("Invalid ACK_REJECTED payload"))?,
                ),
            )),

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

                Ok(Self::Supervisor(SupervisorMessage::Register {
                    service_id,
                    pid,
                    version,
                }))
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

                Ok(Self::Supervisor(SupervisorMessage::ServiceHeartbeat {
                    service_id,
                    status,
                    timestamp,
                    error_msg,
                }))
            }

            MSG_SUPERVISOR_CMD => {
                if payload.len() < 6 {
                    return Err(anyhow!("Supervisor command payload too short"));
                }

                Ok(Self::Supervisor(SupervisorMessage::SupervisorCmd {
                    command_id: u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]),
                    command_type: payload[4],
                    target_service: payload[5],
                }))
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

                Ok(Self::Supervisor(SupervisorMessage::SupervisorCmdAck {
                    command_id,
                    success,
                    error_msg,
                }))
            }

            MSG_SUPERVISOR_STATE => {
                if payload.len() < 3 {
                    return Err(anyhow!("Supervisor state payload too short"));
                }

                Ok(Self::Supervisor(SupervisorMessage::SupervisorState {
                    state_id: payload[0],
                    services_up: payload[1],
                    services_healthy: payload[2],
                    timestamp,
                }))
            }

            MSG_SERVICE_STATUS => {
                if payload.len() < 14 {
                    return Err(anyhow!("Service status payload too short"));
                }

                Ok(Self::Supervisor(SupervisorMessage::ServiceStatusReport {
                    service_id: payload[0],
                    status: payload[1],
                    cpu_percent: f32::from_le_bytes([payload[2], payload[3], payload[4], payload[5]]),
                    memory_bytes: u64::from_le_bytes([
                        payload[6], payload[7], payload[8], payload[9],
                        payload[10], payload[11], payload[12], payload[13],
                    ]),
                    timestamp,
                }))
            }

            _ => Err(anyhow!("Unknown message ID: 0x{:04X}", frame.msg_id)),
        }
    }

    pub fn to_payload(&self) -> Vec<u8> {
        match self {
            Self::System(SystemMessage::Heartbeat) => vec![],
            Self::System(SystemMessage::TimeSync { time_ms }) => time_ms.to_le_bytes().to_vec(),
            Self::System(SystemMessage::Ping) => vec![],
            Self::System(SystemMessage::Pong) => vec![],

            Self::Rouge(RougeMessage::NodeHeartbeat(msg)) => msg.to_payload_bytes().to_vec(),
            Self::Rouge(RougeMessage::LinkStatus(msg)) => msg.to_payload_bytes().to_vec(),
            Self::Rouge(RougeMessage::MissionRole(msg)) => msg.to_payload_bytes().to_vec(),
            Self::Rouge(RougeMessage::TargetCue(msg)) => msg.to_payload_bytes().to_vec(),
            Self::Rouge(RougeMessage::HandoffRequest(msg)) => msg.to_payload_bytes().to_vec(),
            Self::Rouge(RougeMessage::HandoffAccept(msg)) => msg.to_payload_bytes().to_vec(),
            Self::Rouge(RougeMessage::HandoffReject(msg)) => msg.to_payload_bytes().to_vec(),
            Self::Rouge(RougeMessage::AckReceived(msg)) => msg.to_payload_bytes().to_vec(),
            Self::Rouge(RougeMessage::AckAccepted(msg)) => msg.to_payload_bytes().to_vec(),
            Self::Rouge(RougeMessage::AckRejected(msg)) => msg.to_payload_bytes().to_vec(),

            Self::Supervisor(SupervisorMessage::Register { service_id, pid, version }) => {
                let mut payload = vec![*service_id];
                payload.extend_from_slice(&pid.to_le_bytes());
                payload.extend_from_slice(version.as_bytes());
                payload
            }

            Self::Supervisor(SupervisorMessage::ServiceHeartbeat { service_id, status, error_msg, .. }) => {
                let mut payload = vec![*service_id, *status];
                if let Some(msg) = error_msg {
                    payload.extend_from_slice(msg.as_bytes());
                }
                payload
            }

            Self::Supervisor(SupervisorMessage::SupervisorCmd { command_id, command_type, target_service }) => {
                let mut payload = Vec::with_capacity(6);
                payload.extend_from_slice(&command_id.to_le_bytes());
                payload.push(*command_type);
                payload.push(*target_service);
                payload
            }

            Self::Supervisor(SupervisorMessage::SupervisorCmdAck { command_id, success, error_msg }) => {
                let mut payload = Vec::with_capacity(5);
                payload.extend_from_slice(&command_id.to_le_bytes());
                payload.push(if *success { 1 } else { 0 });
                if let Some(msg) = error_msg {
                    payload.extend_from_slice(msg.as_bytes());
                }
                payload
            }

            Self::Supervisor(SupervisorMessage::SupervisorState { state_id, services_up, services_healthy, .. }) => {
                vec![*state_id, *services_up, *services_healthy]
            }

            Self::Supervisor(SupervisorMessage::ServiceStatusReport { service_id, status, cpu_percent, memory_bytes, .. }) => {
                let mut payload = vec![*service_id, *status];
                payload.extend_from_slice(&cpu_percent.to_le_bytes());
                payload.extend_from_slice(&memory_bytes.to_le_bytes());
                payload
            }
        }
    }

    pub fn msg_id(&self) -> u16 {
        match self {
            Self::System(SystemMessage::Heartbeat) => MSG_HEARTBEAT,
            Self::System(SystemMessage::TimeSync { .. }) => MSG_TIME_SYNC,
            Self::System(SystemMessage::Ping) => MSG_PING,
            Self::System(SystemMessage::Pong) => MSG_PONG,

            Self::Rouge(RougeMessage::NodeHeartbeat(_)) => MSG_NODE_HEARTBEAT,
            Self::Rouge(RougeMessage::LinkStatus(_)) => MSG_LINK_STATUS,
            Self::Rouge(RougeMessage::MissionRole(_)) => MSG_MISSION_ROLE,
            Self::Rouge(RougeMessage::TargetCue(_)) => MSG_TARGET_CUE,
            Self::Rouge(RougeMessage::HandoffRequest(_)) => MSG_HANDOFF_REQUEST,
            Self::Rouge(RougeMessage::HandoffAccept(_)) => MSG_HANDOFF_ACCEPT,
            Self::Rouge(RougeMessage::HandoffReject(_)) => MSG_HANDOFF_REJECT,
            Self::Rouge(RougeMessage::AckReceived(_)) => MSG_ACK_RECEIVED,
            Self::Rouge(RougeMessage::AckAccepted(_)) => MSG_ACK_ACCEPTED,
            Self::Rouge(RougeMessage::AckRejected(_)) => MSG_ACK_REJECTED,

            Self::Supervisor(SupervisorMessage::Register { .. }) => MSG_REGISTER,
            Self::Supervisor(SupervisorMessage::ServiceHeartbeat { .. }) => MSG_SERVICE_HEARTBEAT,
            Self::Supervisor(SupervisorMessage::SupervisorCmd { .. }) => MSG_SUPERVISOR_CMD,
            Self::Supervisor(SupervisorMessage::SupervisorCmdAck { .. }) => MSG_SUPERVISOR_CMD_ACK,
            Self::Supervisor(SupervisorMessage::SupervisorState { .. }) => MSG_SUPERVISOR_STATE,
            Self::Supervisor(SupervisorMessage::ServiceStatusReport { .. }) => MSG_SERVICE_STATUS,
        }
    }

    pub fn to_frame(&self) -> SynapseFrame {
        SynapseFrame::new(self.msg_id(), self.to_payload())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::endpoint::{nodes, services, EndpointId};

    #[test]
    fn test_node_heartbeat_roundtrip() {
        let heartbeat = NodeHeartbeat::new(NodeId(1), NodeRole::FixedWing, 12_345, 100, 0x0003);

        let src = EndpointId::new(nodes::FIXED_WING, services::COMPANION);
        let dst = EndpointId::new(nodes::FPV, services::COMPANION);

        let msg = SynapseMessage::Rouge(RougeMessage::NodeHeartbeat(heartbeat));
        let frame = msg
            .to_frame()
            .with_sequence(42)
            .with_routing(src.raw(), dst.raw());

        let encoded = frame.encode_with_cobs();
        let decoded_frame = SynapseFrame::parse_cobs(&encoded).unwrap();

        assert_eq!(decoded_frame.msg_id, MSG_NODE_HEARTBEAT);
        assert_eq!(decoded_frame.sequence, 42);
        assert_eq!(decoded_frame.src_endpoint, Some(src.raw()));
        assert_eq!(decoded_frame.dst_endpoint, Some(dst.raw()));

        let decoded_msg = SynapseMessage::from_frame(&decoded_frame).unwrap();

        match decoded_msg {
            SynapseMessage::Rouge(RougeMessage::NodeHeartbeat(decoded)) => assert_eq!(decoded, heartbeat),
            _ => panic!("Expected NodeHeartbeat message"),
        }
    }

    #[test]
    fn test_link_status_roundtrip() {
        let link = LinkStatus::new(NodeId(2), -72, 95, 7, 55_000);

        let src = EndpointId::new(nodes::FPV, services::LINK_MONITOR);
        let dst = EndpointId::new(nodes::GCS, services::MISSION);

        let msg = SynapseMessage::Rouge(RougeMessage::LinkStatus(link));
        let frame = msg
            .to_frame()
            .with_sequence(77)
            .with_routing(src.raw(), dst.raw());

        let encoded = frame.encode_with_cobs();
        let decoded_frame = SynapseFrame::parse_cobs(&encoded).unwrap();

        assert_eq!(decoded_frame.msg_id, MSG_LINK_STATUS);
        assert_eq!(decoded_frame.sequence, 77);
        assert_eq!(decoded_frame.src_endpoint, Some(src.raw()));
        assert_eq!(decoded_frame.dst_endpoint, Some(dst.raw()));

        let decoded_msg = SynapseMessage::from_frame(&decoded_frame).unwrap();

        match decoded_msg {
            SynapseMessage::Rouge(RougeMessage::LinkStatus(decoded)) => assert_eq!(decoded, link),
            _ => panic!("Expected LinkStatus message"),
        }
    }

    #[test]
    fn test_target_cue_roundtrip() {
        let cue = TargetCue::new(NodeId(2), 0x1234, 12.5, -7.25, 103.0, 92, 3, 88_001);
        let expected_payload = [
            2,
            0x34, 0x12,
            0x00, 0x00, 0x48, 0x41,
            0x00, 0x00, 0xE8, 0xC0,
            0x00, 0x00, 0xCE, 0x42,
            92,
            3,
            0xC1, 0x57, 0x01, 0x00,
        ];

        assert_eq!(cue.to_payload_bytes(), expected_payload);
        assert_eq!(TargetCue::from_payload_bytes(&expected_payload), Some(cue));

        let src = EndpointId::new(nodes::FPV, services::MISSION);
        let dst = EndpointId::new(nodes::GCS, services::MISSION);
        let msg = SynapseMessage::Rouge(RougeMessage::TargetCue(cue));
        let frame = msg
            .to_frame()
            .with_sequence(103)
            .with_routing(src.raw(), dst.raw());

        let encoded = frame.encode_with_cobs();
        let decoded_frame = SynapseFrame::parse_cobs(&encoded).unwrap();

        assert_eq!(decoded_frame.msg_id, MSG_TARGET_CUE);
        assert_eq!(decoded_frame.sequence, 103);
        assert_eq!(decoded_frame.src_endpoint, Some(src.raw()));
        assert_eq!(decoded_frame.dst_endpoint, Some(dst.raw()));
        assert_eq!(decoded_frame.payload.as_slice(), &expected_payload);

        let decoded_msg = SynapseMessage::from_frame(&decoded_frame).unwrap();

        match decoded_msg {
            SynapseMessage::Rouge(RougeMessage::TargetCue(decoded)) => assert_eq!(decoded, cue),
            _ => panic!("Expected TargetCue message"),
        }
    }

    #[test]
    fn test_ack_helpers_and_reason_codes() {
        assert_eq!(AckReason::from_u8(0), Some(AckReason::Accepted));
        assert_eq!(AckReason::from_u8(1), Some(AckReason::Busy));
        assert_eq!(AckReason::from_u8(2), Some(AckReason::Invalid));
        assert_eq!(AckReason::from_u8(3), Some(AckReason::Unauthorized));
        assert_eq!(AckReason::from_u8(4), Some(AckReason::Unsupported));
        assert_eq!(AckReason::from_u8(99), None);

        let frame = SynapseMessage::Rouge(RougeMessage::TargetCue(TargetCue::new(
            NodeId(2),
            7,
            1.0,
            2.0,
            3.0,
            80,
            1,
            55,
        )))
        .to_frame()
        .with_sequence(222);

        let receipt = AckReceipt::for_frame(&frame, NodeId(3));
        assert_eq!(receipt.original_msg_id, MSG_TARGET_CUE);
        assert_eq!(receipt.original_seq, 222);
        assert_eq!(receipt.from_node, NodeId(3));

        let accepted = AckDecision::accept_for_frame(&frame, NodeId(1));
        assert_eq!(accepted.original_msg_id, MSG_TARGET_CUE);
        assert_eq!(accepted.original_seq, 222);
        assert_eq!(accepted.from_node, NodeId(1));
        assert_eq!(accepted.reason, AckReason::Accepted);

        let rejected = AckDecision::reject_for_frame(&frame, NodeId(1), AckReason::Busy);
        assert_eq!(rejected.original_msg_id, MSG_TARGET_CUE);
        assert_eq!(rejected.original_seq, 222);
        assert_eq!(rejected.from_node, NodeId(1));
        assert_eq!(rejected.reason, AckReason::Busy);
    }

    #[test]
    fn test_ack_received_roundtrip() {
        let ack = AckReceipt {
            original_msg_id: MSG_NODE_HEARTBEAT,
            original_seq: 42,
            from_node: NodeId(2),
        };

        let src = EndpointId::new(nodes::FPV, services::COMPANION);
        let dst = EndpointId::new(nodes::FIXED_WING, services::COMPANION);
        let msg = SynapseMessage::Rouge(RougeMessage::AckReceived(ack));
        let frame = msg
            .to_frame()
            .with_sequence(100)
            .with_routing(src.raw(), dst.raw());

        let encoded = frame.encode_with_cobs();
        let decoded_frame = SynapseFrame::parse_cobs(&encoded).unwrap();

        assert_eq!(decoded_frame.msg_id, MSG_ACK_RECEIVED);
        assert_eq!(decoded_frame.sequence, 100);
        assert_eq!(decoded_frame.src_endpoint, Some(src.raw()));
        assert_eq!(decoded_frame.dst_endpoint, Some(dst.raw()));

        let decoded_msg = SynapseMessage::from_frame(&decoded_frame).unwrap();

        match decoded_msg {
            SynapseMessage::Rouge(RougeMessage::AckReceived(decoded)) => assert_eq!(decoded, ack),
            _ => panic!("Expected AckReceived message"),
        }
    }

    #[test]
    fn test_ack_accepted_roundtrip() {
        let ack = AckDecision {
            original_msg_id: MSG_LINK_STATUS,
            original_seq: 77,
            from_node: NodeId(3),
            reason: AckReason::Accepted,
        };

        let src = EndpointId::new(nodes::GCS, services::MISSION);
        let dst = EndpointId::new(nodes::FPV, services::COMPANION);
        let msg = SynapseMessage::Rouge(RougeMessage::AckAccepted(ack));
        let frame = msg
            .to_frame()
            .with_sequence(101)
            .with_routing(src.raw(), dst.raw());

        let encoded = frame.encode_with_cobs();
        let decoded_frame = SynapseFrame::parse_cobs(&encoded).unwrap();

        assert_eq!(decoded_frame.msg_id, MSG_ACK_ACCEPTED);
        assert_eq!(decoded_frame.sequence, 101);
        assert_eq!(decoded_frame.src_endpoint, Some(src.raw()));
        assert_eq!(decoded_frame.dst_endpoint, Some(dst.raw()));

        let decoded_msg = SynapseMessage::from_frame(&decoded_frame).unwrap();

        match decoded_msg {
            SynapseMessage::Rouge(RougeMessage::AckAccepted(decoded)) => assert_eq!(decoded, ack),
            _ => panic!("Expected AckAccepted message"),
        }
    }

    #[test]
    fn test_ack_rejected_roundtrip() {
        let ack = AckDecision {
            original_msg_id: MSG_MISSION_ROLE,
            original_seq: 88,
            from_node: NodeId(2),
            reason: AckReason::Invalid,
        };

        let src = EndpointId::new(nodes::FPV, services::COMPANION);
        let dst = EndpointId::new(nodes::GCS, services::MISSION);
        let msg = SynapseMessage::Rouge(RougeMessage::AckRejected(ack));
        let frame = msg
            .to_frame()
            .with_sequence(102)
            .with_routing(src.raw(), dst.raw());

        let encoded = frame.encode_with_cobs();
        let decoded_frame = SynapseFrame::parse_cobs(&encoded).unwrap();

        assert_eq!(decoded_frame.msg_id, MSG_ACK_REJECTED);
        assert_eq!(decoded_frame.sequence, 102);
        assert_eq!(decoded_frame.src_endpoint, Some(src.raw()));
        assert_eq!(decoded_frame.dst_endpoint, Some(dst.raw()));

        let decoded_msg = SynapseMessage::from_frame(&decoded_frame).unwrap();

        match decoded_msg {
            SynapseMessage::Rouge(RougeMessage::AckRejected(decoded)) => assert_eq!(decoded, ack),
            _ => panic!("Expected AckRejected message"),
        }
    }

    #[test]
    fn test_handoff_request_roundtrip() {
        let request = HandoffRequest::new(NodeId(2), NodeId(1), 0x0102, 4, 90_000);

        let src = EndpointId::new(nodes::FPV, services::MISSION);
        let dst = EndpointId::new(nodes::FIXED_WING, services::MISSION);
        let msg = SynapseMessage::Rouge(RougeMessage::HandoffRequest(request));
        let frame = msg
            .to_frame()
            .with_sequence(140)
            .with_routing(src.raw(), dst.raw());

        let encoded = frame.encode_with_cobs();
        let decoded_frame = SynapseFrame::parse_cobs(&encoded).unwrap();

        assert_eq!(decoded_frame.msg_id, MSG_HANDOFF_REQUEST);
        assert_eq!(decoded_frame.src_endpoint, Some(src.raw()));
        assert_eq!(decoded_frame.dst_endpoint, Some(dst.raw()));

        let decoded_msg = SynapseMessage::from_frame(&decoded_frame).unwrap();

        match decoded_msg {
            SynapseMessage::Rouge(RougeMessage::HandoffRequest(decoded)) => assert_eq!(decoded, request),
            _ => panic!("Expected HandoffRequest message"),
        }
    }

    #[test]
    fn test_handoff_accept_roundtrip() {
        let accept = HandoffAccept::new(NodeId(1), NodeId(2), 0x0102, 90_010);

        let src = EndpointId::new(nodes::FIXED_WING, services::MISSION);
        let dst = EndpointId::new(nodes::FPV, services::MISSION);
        let msg = SynapseMessage::Rouge(RougeMessage::HandoffAccept(accept));
        let frame = msg
            .to_frame()
            .with_sequence(141)
            .with_routing(src.raw(), dst.raw());

        let encoded = frame.encode_with_cobs();
        let decoded_frame = SynapseFrame::parse_cobs(&encoded).unwrap();

        assert_eq!(decoded_frame.msg_id, MSG_HANDOFF_ACCEPT);

        let decoded_msg = SynapseMessage::from_frame(&decoded_frame).unwrap();

        match decoded_msg {
            SynapseMessage::Rouge(RougeMessage::HandoffAccept(decoded)) => assert_eq!(decoded, accept),
            _ => panic!("Expected HandoffAccept message"),
        }
    }

    #[test]
    fn test_handoff_reject_roundtrip() {
        let reject = HandoffReject::new(NodeId(1), NodeId(2), 0x0102, AckReason::Busy, 90_020);

        let src = EndpointId::new(nodes::FIXED_WING, services::MISSION);
        let dst = EndpointId::new(nodes::FPV, services::MISSION);
        let msg = SynapseMessage::Rouge(RougeMessage::HandoffReject(reject));
        let frame = msg
            .to_frame()
            .with_sequence(142)
            .with_routing(src.raw(), dst.raw());

        let encoded = frame.encode_with_cobs();
        let decoded_frame = SynapseFrame::parse_cobs(&encoded).unwrap();

        assert_eq!(decoded_frame.msg_id, MSG_HANDOFF_REJECT);

        let decoded_msg = SynapseMessage::from_frame(&decoded_frame).unwrap();

        match decoded_msg {
            SynapseMessage::Rouge(RougeMessage::HandoffReject(decoded)) => assert_eq!(decoded, reject),
            _ => panic!("Expected HandoffReject message"),
        }
    }
}
