use std::collections::HashMap;

use crate::core::{
    AckDecision, AckReason, AckReceipt, RougeMessage, SynapseFrame, SynapseMessage, FLAG_ACK_REQ,
    FLAG_IS_ACK,
};
use crate::endpoint::EndpointId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PendingMessageKey {
    pub msg_id: u16,
    pub sequence: u16,
}

impl PendingMessageKey {
    pub fn new(msg_id: u16, sequence: u16) -> Self {
        Self { msg_id, sequence }
    }

    pub fn from_frame(frame: &SynapseFrame) -> Self {
        Self::new(frame.msg_id, frame.sequence)
    }
}

#[derive(Debug, Clone)]
pub struct PendingTx {
    pub frame: SynapseFrame,
    pub receipt: Option<AckReceipt>,
    pub decision: Option<AckDecision>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeerState {
    pub endpoint: Option<EndpointId>,
    pub last_timestamp_ms: Option<u32>,
    pub frames_received: u64,
    pub decode_errors: u64,
}

impl PeerState {
    pub fn new() -> Self {
        Self {
            endpoint: None,
            last_timestamp_ms: None,
            frames_received: 0,
            decode_errors: 0,
        }
    }
}

impl Default for PeerState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub local_endpoint: Option<EndpointId>,
    pub auto_ack_receipts: bool,
}

impl RuntimeConfig {
    pub fn new(local_endpoint: Option<EndpointId>) -> Self {
        Self {
            local_endpoint,
            auto_ack_receipts: true,
        }
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self::new(None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SendOptions {
    pub src: Option<EndpointId>,
    pub dst: Option<EndpointId>,
    pub ack_requested: bool,
    pub extra_flags: u8,
}

impl SendOptions {
    pub fn routed(src: EndpointId, dst: EndpointId) -> Self {
        Self {
            src: Some(src),
            dst: Some(dst),
            ack_requested: false,
            extra_flags: 0,
        }
    }
}

impl Default for SendOptions {
    fn default() -> Self {
        Self {
            src: None,
            dst: None,
            ack_requested: false,
            extra_flags: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MessageEvent {
    pub frame: SynapseFrame,
    pub message: SynapseMessage,
    pub auto_responses: Vec<SynapseFrame>,
}

#[derive(Debug, Clone)]
pub enum RuntimeEvent {
    Message(MessageEvent),
    AckReceived(AckReceipt),
    AckAccepted(AckDecision),
    AckRejected(AckDecision),
    DecodeError(String),
}

pub struct SynapseSession {
    config: RuntimeConfig,
    next_sequence: u16,
    rx_buffer: Vec<u8>,
    pending: HashMap<PendingMessageKey, PendingTx>,
    peer: PeerState,
}

impl SynapseSession {
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config,
            next_sequence: 1,
            rx_buffer: Vec::new(),
            pending: HashMap::new(),
            peer: PeerState::new(),
        }
    }

    pub fn config(&self) -> RuntimeConfig {
        self.config
    }

    pub fn peer_state(&self) -> PeerState {
        self.peer
    }

    pub fn pending(&self) -> &HashMap<PendingMessageKey, PendingTx> {
        &self.pending
    }

    pub fn pending_tx(&self, key: PendingMessageKey) -> Option<&PendingTx> {
        self.pending.get(&key)
    }

    pub fn clear_pending(&mut self, key: PendingMessageKey) -> Option<PendingTx> {
        self.pending.remove(&key)
    }

    pub fn send_message(&mut self, message: SynapseMessage, options: SendOptions) -> SynapseFrame {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.wrapping_add(1);

        let mut frame = message.to_frame().with_sequence(sequence);

        let src = options.src.or(self.config.local_endpoint);
        if let (Some(src), Some(dst)) = (src, options.dst) {
            frame = frame.with_endpoint_ids(src, dst);
        }

        let mut flags = frame.flags | options.extra_flags;
        if options.ack_requested {
            flags |= FLAG_ACK_REQ;
        }
        frame = frame.with_flags(flags);

        if options.ack_requested && (frame.flags & FLAG_IS_ACK) == 0 {
            self.pending.insert(
                PendingMessageKey::from_frame(&frame),
                PendingTx {
                    frame: frame.clone(),
                    receipt: None,
                    decision: None,
                },
            );
        }

        frame
    }

    pub fn ingest_bytes(&mut self, bytes: &[u8]) -> Vec<RuntimeEvent> {
        self.rx_buffer.extend_from_slice(bytes);

        let mut events = Vec::new();
        while let Some(pos) = self.rx_buffer.iter().position(|&byte| byte == 0x00) {
            let mut encoded_frame: Vec<u8> = self.rx_buffer.drain(..=pos).collect();
            if encoded_frame.len() == 1 {
                continue;
            }

            match SynapseFrame::parse_cobs(&encoded_frame) {
                Ok(frame) => {
                    self.peer.frames_received += 1;
                    self.peer.last_timestamp_ms = Some(frame.timestamp_ms);
                    self.peer.endpoint = frame.src_endpoint.map(EndpointId);
                    events.extend(self.handle_inbound_frame(frame));
                }
                Err(err) => {
                    self.peer.decode_errors += 1;
                    events.push(RuntimeEvent::DecodeError(err.to_string()));
                }
            }

            encoded_frame.clear();
        }

        events
    }

    pub fn build_receipt_ack(&mut self, frame: &SynapseFrame) -> Option<SynapseFrame> {
        let from_node = self.config.local_endpoint?.node();
        let dst = frame.src_endpoint.map(EndpointId)?;
        let ack = SynapseMessage::Rouge(RougeMessage::AckReceived(AckReceipt::for_frame(frame, from_node)));

        Some(
            self.send_message(
                ack,
                SendOptions {
                    src: self.config.local_endpoint,
                    dst: Some(dst),
                    ack_requested: false,
                    extra_flags: FLAG_IS_ACK,
                },
            ),
        )
    }

    pub fn build_accept_ack(&mut self, frame: &SynapseFrame) -> Option<SynapseFrame> {
        let from_node = self.config.local_endpoint?.node();
        let dst = frame.src_endpoint.map(EndpointId)?;
        let ack = SynapseMessage::Rouge(RougeMessage::AckAccepted(AckDecision::accept_for_frame(
            frame, from_node,
        )));

        Some(
            self.send_message(
                ack,
                SendOptions {
                    src: self.config.local_endpoint,
                    dst: Some(dst),
                    ack_requested: false,
                    extra_flags: FLAG_IS_ACK,
                },
            ),
        )
    }

    pub fn build_reject_ack(
        &mut self,
        frame: &SynapseFrame,
        reason: AckReason,
    ) -> Option<SynapseFrame> {
        let from_node = self.config.local_endpoint?.node();
        let dst = frame.src_endpoint.map(EndpointId)?;
        let ack = SynapseMessage::Rouge(RougeMessage::AckRejected(AckDecision::reject_for_frame(
            frame, from_node, reason,
        )));

        Some(
            self.send_message(
                ack,
                SendOptions {
                    src: self.config.local_endpoint,
                    dst: Some(dst),
                    ack_requested: false,
                    extra_flags: FLAG_IS_ACK,
                },
            ),
        )
    }

    fn handle_inbound_frame(&mut self, frame: SynapseFrame) -> Vec<RuntimeEvent> {
        let mut events = Vec::new();
        match SynapseMessage::from_frame(&frame) {
            Ok(message) => {
                match &message {
                    SynapseMessage::Rouge(RougeMessage::AckReceived(ack)) => {
                        self.apply_receipt(*ack);
                        events.push(RuntimeEvent::AckReceived(*ack));
                    }
                    SynapseMessage::Rouge(RougeMessage::AckAccepted(ack)) => {
                        self.apply_decision(*ack);
                        events.push(RuntimeEvent::AckAccepted(*ack));
                    }
                    SynapseMessage::Rouge(RougeMessage::AckRejected(ack)) => {
                        self.apply_decision(*ack);
                        events.push(RuntimeEvent::AckRejected(*ack));
                    }
                    _ => {}
                }

                let mut auto_responses = Vec::new();
                if self.config.auto_ack_receipts && self.should_auto_ack(&frame) {
                    if let Some(ack_frame) = self.build_receipt_ack(&frame) {
                        auto_responses.push(ack_frame);
                    }
                }

                events.push(RuntimeEvent::Message(MessageEvent {
                    frame,
                    message,
                    auto_responses,
                }));
            }
            Err(err) => {
                self.peer.decode_errors += 1;
                events.push(RuntimeEvent::DecodeError(err.to_string()));
            }
        }

        events
    }

    fn should_auto_ack(&self, frame: &SynapseFrame) -> bool {
        (frame.flags & FLAG_ACK_REQ) != 0 && (frame.flags & FLAG_IS_ACK) == 0
    }

    fn apply_receipt(&mut self, ack: AckReceipt) {
        let key = PendingMessageKey::new(ack.original_msg_id, ack.original_seq);
        if let Some(pending) = self.pending.get_mut(&key) {
            pending.receipt = Some(ack);
        }
    }

    fn apply_decision(&mut self, ack: AckDecision) {
        let key = PendingMessageKey::new(ack.original_msg_id, ack.original_seq);
        if let Some(pending) = self.pending.get_mut(&key) {
            pending.decision = Some(ack);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{LinkStatus, TargetCue};
    use crate::endpoint::{nodes, services, NodeId};

    #[test]
    fn test_send_message_assigns_sequence_and_tracks_pending() {
        let local = EndpointId::new(nodes::FIXED_WING, services::MISSION);
        let remote = EndpointId::new(nodes::FPV, services::MISSION);
        let mut session = SynapseSession::new(RuntimeConfig::new(Some(local)));

        let msg = SynapseMessage::Rouge(RougeMessage::TargetCue(TargetCue::new(
            NodeId(1),
            7,
            10.0,
            20.0,
            30.0,
            90,
            2,
            42,
        )));

        let frame = session.send_message(
            msg,
            SendOptions {
                src: None,
                dst: Some(remote),
                ack_requested: true,
                extra_flags: 0,
            },
        );

        assert_eq!(frame.sequence, 1);
        assert_eq!(frame.src_endpoint, Some(local.raw()));
        assert_eq!(frame.dst_endpoint, Some(remote.raw()));
        assert_ne!(frame.flags & FLAG_ACK_REQ, 0);

        let key = PendingMessageKey::from_frame(&frame);
        let pending = session.pending_tx(key).unwrap();
        assert_eq!(pending.frame.sequence, 1);
        assert!(pending.receipt.is_none());
        assert!(pending.decision.is_none());
    }

    #[test]
    fn test_ingest_bytes_reassembles_stream_and_auto_generates_receipt_ack() {
        let local = EndpointId::new(nodes::GCS, services::MISSION);
        let remote = EndpointId::new(nodes::FPV, services::MISSION);
        let mut session = SynapseSession::new(RuntimeConfig::new(Some(local)));

        let inbound = SynapseMessage::Rouge(RougeMessage::LinkStatus(LinkStatus::new(
            NodeId(2),
            -70,
            90,
            6,
            1_000,
        )))
        .to_frame()
        .with_sequence(55)
        .with_endpoint_ids(remote, local)
        .with_flags(crate::core::FLAG_PAYLOAD_CRC | FLAG_ACK_REQ);

        let encoded = inbound.encode_with_cobs();
        let split = 5;

        let mut first_events = session.ingest_bytes(&encoded[..split]);
        assert!(first_events.is_empty());

        let second_events = session.ingest_bytes(&encoded[split..]);
        first_events.extend(second_events);

        assert_eq!(first_events.len(), 1);
        match &first_events[0] {
            RuntimeEvent::Message(event) => {
                assert_eq!(event.frame.sequence, 55);
                assert_eq!(event.frame.src_endpoint, Some(remote.raw()));
                assert_eq!(event.auto_responses.len(), 1);

                let ack_frame = &event.auto_responses[0];
                assert_eq!(ack_frame.src_endpoint, Some(local.raw()));
                assert_eq!(ack_frame.dst_endpoint, Some(remote.raw()));
                assert_ne!(ack_frame.flags & FLAG_IS_ACK, 0);

                let ack_msg = SynapseMessage::from_frame(ack_frame).unwrap();
                match ack_msg {
                    SynapseMessage::Rouge(RougeMessage::AckReceived(ack)) => {
                        assert_eq!(ack.original_msg_id, inbound.msg_id);
                        assert_eq!(ack.original_seq, inbound.sequence);
                        assert_eq!(ack.from_node, nodes::GCS);
                    }
                    _ => panic!("Expected AckReceived auto response"),
                }
            }
            _ => panic!("Expected message event"),
        }

        let peer = session.peer_state();
        assert_eq!(peer.endpoint, Some(remote));
        assert_eq!(peer.frames_received, 1);
        assert_eq!(peer.decode_errors, 0);
    }

    #[test]
    fn test_ingest_ack_updates_pending_state() {
        let local = EndpointId::new(nodes::FIXED_WING, services::MISSION);
        let remote = EndpointId::new(nodes::GCS, services::MISSION);
        let mut session = SynapseSession::new(RuntimeConfig::new(Some(local)));

        let outbound = session.send_message(
            SynapseMessage::Rouge(RougeMessage::TargetCue(TargetCue::new(
                nodes::FIXED_WING,
                99,
                5.0,
                6.0,
                7.0,
                88,
                1,
                500,
            ))),
            SendOptions {
                src: None,
                dst: Some(remote),
                ack_requested: true,
                extra_flags: 0,
            },
        );

        let receipt_inbound = SynapseMessage::Rouge(RougeMessage::AckReceived(
            AckReceipt::for_frame(&outbound, remote.node()),
        ))
        .to_frame()
        .with_sequence(76)
        .with_endpoint_ids(remote, local)
        .with_flags(crate::core::FLAG_PAYLOAD_CRC | FLAG_IS_ACK);

        let receipt_events = session.ingest_bytes(&receipt_inbound.encode_with_cobs());
        assert!(matches!(receipt_events[0], RuntimeEvent::AckReceived(_)));

        let key = PendingMessageKey::from_frame(&outbound);
        let pending_after_receipt = session.pending_tx(key).unwrap();
        assert!(pending_after_receipt.receipt.is_some());
        assert!(pending_after_receipt.decision.is_none());

        let accept = SynapseMessage::Rouge(RougeMessage::AckAccepted(AckDecision::accept_for_frame(
            &outbound,
            remote.node(),
        )))
        .to_frame()
        .with_sequence(77)
        .with_endpoint_ids(remote, local)
        .with_flags(crate::core::FLAG_PAYLOAD_CRC | FLAG_IS_ACK);

        let accept_events = session.ingest_bytes(&accept.encode_with_cobs());
        assert!(matches!(accept_events[0], RuntimeEvent::AckAccepted(_)));

        let pending_after_accept = session.pending_tx(key).unwrap();
        assert!(pending_after_accept.receipt.is_some());
        assert_eq!(
            pending_after_accept.decision.as_ref().unwrap().reason,
            AckReason::Accepted
        );
    }

    #[test]
    fn test_invalid_frame_surfaces_decode_error() {
        let mut session = SynapseSession::new(RuntimeConfig::default());
        let events = session.ingest_bytes(&[0x02, 0x01, 0x00]);

        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], RuntimeEvent::DecodeError(_)));
        assert_eq!(session.peer_state().decode_errors, 1);
    }
}
