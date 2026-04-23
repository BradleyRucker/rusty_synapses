use crate::endpoint::NodeId;

pub const MSG_NODE_HEARTBEAT: u16 = 0x0801;
pub const MSG_LINK_STATUS: u16 = 0x0802;

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

    pub fn from_u8(role: u8) -> Option<NodeRole> {
        match role {
            1 => Some(NodeRole::FixedWing),
            2 => Some(NodeRole::Fpv),
            3 => Some(NodeRole::Gcs),
            4 => Some(NodeRole::Relay),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
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

        let node = NodeId(bytes[0]);
        let role = NodeRole::from_u8(bytes[1])?;
        let uptime_ms = u32::from_le_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);
        let health = bytes[6];
        let capabilities = u16::from_le_bytes([bytes[7], bytes[8]]);

        Some(Self {
            node,
            role,
            uptime_ms,
            health,
            capabilities,
        })
    }
}