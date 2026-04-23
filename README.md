# Rusty Synapses

`rusty_synapses` is a compact binary protocol crate for ROUGE mission coordination and network state.

The crate is no longer centered on being a full communications layer for a custom flight controller. Its current role is:

- companion-to-companion messaging
- GCS-to-companion messaging
- compact routed binary messages
- acknowledgment semantics
- a foundation for later relay and mesh behavior

The main design goal is a small, explicit, manually encoded wire format that can survive constrained links and be shared across Linux services, companion computers, and ground systems.

## Status

Current protocol foundation is working.

- `SynapseFrame` supports:
  - message IDs
  - sequence numbers
  - timestamps
  - COBS framing
  - CRC validation
  - routed endpoints via `src_endpoint` and `dst_endpoint`
- typed message families exist in `src/core/messages.rs`
- a first-pass runtime/session layer exists in `src/runtime.rs`
- unit tests currently pass across the protocol and runtime layer

Validated locally with:

```powershell
cargo test -- --nocapture
```

## Architecture

The crate is split into two layers.

### Protocol Core

The protocol core defines the wire format and typed messages.

- `src/core/frame.rs`
  - transport envelope
  - routing fields
  - COBS-ready encode/decode
- `src/core/messages.rs`
  - manual fixed-size payload definitions
  - message ID dispatch
  - ACK helpers and reason codes
- `src/core/cobs.rs`
  - frame-safe zero-delimited transport encoding
- `src/core/crc.rs`
  - integrity checks

### Runtime Layer

The runtime layer is intentionally small and mechanical.

- `src/runtime.rs`
  - `SynapseSession`
  - stream byte ingestion
  - pending ACK tracking
  - peer state tracking
  - automatic receipt ACK generation
  - typed runtime events

This layer is transport/session plumbing, not mission policy.

## Message Families

Current message organization:

- `SystemMessage`
  - generic control/system messages
- `RougeMessage`
  - mission/network protocol messages
- `SupervisorMessage`
  - optional local supervisor/service messages

### Working ROUGE messages

- `NodeHeartbeat`
- `LinkStatus`
- `MissionRole`
- `TargetCue`
- `HandoffRequest`
- `HandoffAccept`
- `HandoffReject`
- `AckReceived`
- `AckAccepted`
- `AckRejected`

### ACK model

The protocol supports a simple staged acknowledgment flow:

1. `AckReceived`
   receiver confirms the frame arrived and parsed
2. `AckAccepted`
   receiver accepted the message for handling
3. `AckRejected`
   receiver rejected the message with a reason code

Current rejection/decision reasons:

- `Accepted`
- `Busy`
- `Invalid`
- `Unauthorized`
- `Unsupported`

## Routing Model

Routing is endpoint-based.

Each frame may carry:

- `src_endpoint`
- `dst_endpoint`

An endpoint is a compact `(node, service)` identifier. This makes the protocol work for:

- local Linux service routing
- companion-to-companion mission traffic
- GCS-to-companion traffic
- future relay forwarding

## Design Constraints

The crate intentionally keeps a hard boundary around protocol concerns.

- payloads are manual binary payloads, not serde-driven
- `SynapseFrame` remains the transport envelope
- ROUGE mission traffic lives under `RougeMessage`
- payloads prefer fixed-size layouts where practical
- supervisor/service messages are allowed, but they are not the center of Synapse

What this crate should not become:

- a Linux supervisor
- a process lifecycle manager
- a full service bus with policy baked in
- a PX4-specific control stack

## Example Shape

Building a typed message into a routed frame:

```rust
use synapse::core::{RougeMessage, SynapseMessage, TargetCue};
use synapse::endpoint::{nodes, services, EndpointId, NodeId};

let src = EndpointId::new(nodes::FPV, services::MISSION);
let dst = EndpointId::new(nodes::GCS, services::MISSION);

let cue = TargetCue::new(
    NodeId(2),
    0x1234,
    12.5,
    -7.25,
    103.0,
    92,
    3,
    88_001,
);

let frame = SynapseMessage::Rouge(RougeMessage::TargetCue(cue))
    .to_frame()
    .with_sequence(103)
    .with_endpoint_ids(src, dst);

let wire_bytes = frame.encode_with_cobs();
```

Using the runtime/session layer on a byte stream:

```rust
use synapse::endpoint::{nodes, services, EndpointId};
use synapse::runtime::{RuntimeConfig, SynapseSession};

let local = EndpointId::new(nodes::GCS, services::MISSION);
let mut session = SynapseSession::new(RuntimeConfig::new(Some(local)));

let events = session.ingest_bytes(&incoming_bytes);
```

## Repository Layout

```text
src/
├── core.rs
├── endpoint.rs
├── lib.rs
├── rouge_protocol.rs
├── runtime.rs
└── core/
    ├── cobs.rs
    ├── crc.rs
    ├── ffi.rs
    ├── frame.rs
    ├── messages.rs
    └── python.rs
```

## Current Direction

The near-term objective is straightforward:

- make mission-bearing ROUGE messages reliable
- keep the wire format small and stable
- let Linux systems such as Cortex consume Synapse cleanly
- keep routing and policy separate

## Next Steps

### Cortex Integration Plan

The right integration model for Cortex is a central router, not peer-to-peer service coupling.

Cortex should own:

- Unix socket listeners
- connection lifecycle
- service registration
- route resolution
- ACK policy
- authorization and rejection decisions
- local service forwarding vs external link forwarding

Synapse should remain:

- protocol and message definitions
- encode/decode logic
- lightweight session mechanics

### Recommended Cortex shape

Implement a central `SynapseRouter` inside Cortex with:

1. `ConnectionId`
   maps a live Unix socket connection to internal router state
2. `ServiceRegistration`
   maps service identity to endpoint ownership
3. `RouteTable`
   resolves `dst_endpoint` to:
   - a local service connection
   - an external network link
   - a reject path
4. `ConnectionSession`
   per-connection byte buffer and `SynapseSession` state
5. `RouterDispatch`
   consumes runtime events and decides:
   - forward local
   - forward external
   - emit `AckAccepted`
   - emit `AckRejected`

### Suggested flow inside Cortex

1. service connects over Unix socket
2. service sends `Register`
3. Cortex binds that service to an endpoint
4. incoming bytes feed a per-connection `SynapseSession`
5. router inspects `dst_endpoint`
6. router forwards to:
   - another local service
   - the Synapse external link
   - a reject response if no route or permission exists

### Immediate Cortex implementation order

1. build a router-owned service/endpoint registry
2. give each socket connection a per-connection `SynapseSession`
3. route local supervisor and mission traffic by `dst_endpoint`
4. centralize ACK generation in the router
5. add timeout and retry policy for ACK-requested messages
6. add dedup and stale-peer handling

### Integration boundary

A useful rule for Cortex integration:

- if it is bytes, framing, message IDs, or payload encoding, it belongs in Synapse
- if it is sockets, routing, permissions, health, or process ownership, it belongs in Cortex

That boundary keeps the protocol reusable and keeps Cortex in charge of real system behavior.
