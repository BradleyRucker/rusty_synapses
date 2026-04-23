#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ServiceId(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EndpointId(pub u16);

impl EndpointId {
    pub const fn new(node: NodeId, service: ServiceId) -> Self {
        Self(((node.0 as u16) << 8) | (service.0 as u16))
    }

    pub const fn node(self) -> NodeId {
        NodeId((self.0 >> 8) as u8)
    }

    pub const fn service(self) -> ServiceId {
        ServiceId((self.0 & 0xFF) as u8)
    }

    pub const fn raw(self) -> u16 {
        self.0
    }
}

pub mod nodes {
    use super::NodeId;
    pub const FIXED_WING: NodeId = NodeId(1);
    pub const FPV: NodeId = NodeId(2);
    pub const GCS: NodeId = NodeId(3);
}

pub mod services {
    use super::ServiceId;
    pub const COMPANION: ServiceId = ServiceId(1);
    pub const FC_BRIDGE: ServiceId = ServiceId(2);
    pub const MISSION: ServiceId = ServiceId(3);
    pub const LINK_MONITOR: ServiceId = ServiceId(4);
}