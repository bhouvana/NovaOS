# Spec 15 — Nova Bus Protocol Specification

Status: Draft v0.1 · Last updated: 2026-07-18

The complete wire protocol for Nova Bus — NovaOS's equivalent of D-Bus's protocol
documentation, per Staff Engineer review. Concretizes
[ADR-0006](../decisions/ADR-0006-ipc-mechanism.md) and
[RFC-0002](../rfcs/RFC-0002-nova-bus.md).

## 1. Transport

Unix domain socket, `SOCK_STREAM`, at the fixed path `/run/nova/bus.sock`
([RFC-0002](../rfcs/RFC-0002-nova-bus.md) Configuration). No TCP transport exists or is
planned — Nova Bus is same-host-only by design (§9's rationale against service
discovery applies equally here: same-host IPC never needs network transport
complexity).

## 2. Framing

Every message on the wire is:

```text
┌──────────────┬─────────────────────────────┐
│ length (u32, │ Envelope (protobuf-encoded,  │
│  LE, 4 bytes)│  `length` bytes)             │
└──────────────┴─────────────────────────────┘
```

`length` is capped at 4 MiB (`MAX_MESSAGE_SIZE`) — a message larger than that is a
protocol violation, connection is dropped. This bound exists so a single oversized
message can't be used to exhaust broker memory; large data (file contents, images)
belongs in `nova-storage`/broker-mediated file handles, never inlined into a Bus
message ([06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md) §4, §8).

## 3. Envelope

```protobuf
message Envelope {
  uint32 protocol_version = 1;   // see §8
  optional uint64 trace_id = 13; // cross-service correlation, see
                                  // 21-OBSERVABILITY-SPEC.md §3 — absent for
                                  // the vast majority of untraced messages
  oneof kind {
    Connect connect = 2;
    ConnectAck connect_ack = 3;
    Call call = 4;
    Response response = 5;
    Subscribe subscribe = 6;
    SubscribeAck subscribe_ack = 7;
    Publish publish = 8;
    Unsubscribe unsubscribe = 9;
    Error error = 10;
    Ping ping = 11;
    Pong pong = 12;
  }
}
```

## 4. Connection Handshake

```text
Client connects to /run/nova/bus.sock
   ↓
Client sends Connect { protocol_version: 1 }
   ↓
Broker resolves peer identity via SO_PEERCRED (§7) — not part of the
   Connect message itself, the kernel-verified credential is read
   directly off the socket
   ↓
Broker sends ConnectAck { protocol_version: 1, connection_id: u64 }
   or Error { code: PROTOCOL_VERSION_UNSUPPORTED } and closes the
   connection
```

`connection_id` is a broker-assigned handle used only for broker-side logging/metrics
correlation ([RFC-0002](../rfcs/RFC-0002-nova-bus.md) Metrics) — never sent back by the
client in later messages (the socket itself is the connection identity for all
subsequent traffic).

## 5. Request/Response (Method Calls)

```protobuf
message Call {
  uint64 correlation_id = 1;   // client-assigned, unique per connection
  string topic = 2;            // e.g. "nova.session.launch"
  bytes payload = 3;           // topic-specific protobuf message, opaque to the broker
  uint32 timeout_ms = 4;       // 0 = use default (§6)
}

message Response {
  uint64 correlation_id = 1;   // echoes the Call's correlation_id
  bytes payload = 2;           // topic-specific response message
}
```

- Exactly one handler may be registered per method-call topic (§9) — the broker routes
  a `Call` to that single handler's connection and relays its `Response` back to the
  caller, matching `correlation_id`.
- The broker itself never deserializes `payload` — it is opaque bytes from the broker's
  perspective (§1's "broker does not inspect payload content" property,
  [RFC-0002](../rfcs/RFC-0002-nova-bus.md) Logging). Topic-specific payload schemas are
  defined per-service (e.g., [06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md)'s API
  descriptions) and compiled into both caller and handler via shared `.proto` files in
  `services/nova-bus/proto/`.

## 6. Timeouts

Default request/response timeout: **5000ms**. Callers may override per-call via
`Call.timeout_ms` (0 means "use default," not "no timeout" — an unbounded-wait call is
not permitted by the protocol; a caller wanting long-running-operation semantics uses
the progress-event pattern instead, §7, as `novapkg-agent`'s install flow does with
`nova.package.install_progress`, [RFC-0004](../rfcs/RFC-0004-package-service.md)). On
timeout, the broker synthesizes `Error { code: TIMEOUT, correlation_id }` to the caller
and does not cancel the in-flight handler-side work (the handler is not notified of the
timeout — this is a caller-side give-up, not a cancellation protocol, which v1
deliberately doesn't have: consistent with
[../00-VISION.md](../00-VISION.md) §6's simplicity priority, since almost no Nova Bus
call in this architecture performs slow, cancellable work — the ones that do
(package install, [RFC-0004](../rfcs/RFC-0004-package-service.md)) already use the
progress-event pattern instead of a single long call).

## 7. Publish/Subscribe (Broadcast Events)

```protobuf
message Subscribe {
  string topic = 1;            // supports a trailing wildcard, e.g. "nova.package.*"
}
message SubscribeAck { string topic = 1; }
message Unsubscribe { string topic = 1; }

message Publish {
  string topic = 1;            // exact topic, no wildcard on publish
  bytes payload = 2;
}
```

- At-most-once delivery to currently-connected subscribers
  ([RFC-0002](../rfcs/RFC-0002-nova-bus.md) Failure Modes) — no persistence, no replay,
  no delivery acknowledgment from subscriber back to publisher.
- A publish with zero current subscribers is not an error — the message is simply
  dropped (this is the common case for e.g. `nova.wm.window_mapped` when no one happens
  to care at that instant).
- Wildcard subscriptions (`nova.package.*`) match any topic sharing that dot-separated
  prefix — used by broad consumers like Nova Monitor
  ([21-OBSERVABILITY-SPEC.md](21-OBSERVABILITY-SPEC.md)) rather than subscribing to
  every individual metric topic by name.

## 8. Versioning

- `Envelope.protocol_version` is the **framing/handshake protocol** version (currently
  `1`) — bumped only if the envelope structure itself changes (new `oneof` variant
  semantics, framing format change). A client and broker with mismatched major protocol
  versions refuse to connect (§4) rather than attempting best-effort compatibility.
- **Payload schema versioning** (the topic-specific messages inside `Call.payload`/
  `Publish.payload`) follows ordinary Protobuf evolution rules: fields are only ever
  added (with new field numbers), never renumbered or removed — a handler compiled
  against an older schema silently ignores fields it doesn't know about, a caller using
  an older schema simply doesn't send fields it doesn't know about. This is why
  Protobuf was chosen for the payload format specifically
  ([ADR-0006](../decisions/ADR-0006-ipc-mechanism.md) Rationale) — schema evolution
  without a version-negotiation dance per topic.
- A breaking payload schema change (rare — e.g., a field's meaning changes, not just a
  new optional field) requires a new topic name (e.g., `nova.session.launch.v2`) rather
  than mutating the existing one — this is an RFC-required change per
  [../rfcs/README.md](../rfcs/README.md) §"When an RFC Is Required."

## 9. Service Discovery

There is no dynamic service discovery mechanism — topic-to-handler binding is static
and known at compile time via the shared `.proto` definitions in
`services/nova-bus/proto/` (every caller and every handler for a given topic is built
against the same schema file, [../02-REPOSITORY-STRUCTURE.md](../02-REPOSITORY-STRUCTURE.md)).
The broker enforces "exactly one handler per method-call topic" by rejecting a second
`RegisterHandler` (an internal, privileged-only message not exposed to ordinary SDK
clients — only Nova services register handlers, apps only ever call/subscribe) with
`Error { code: HANDLER_ALREADY_REGISTERED }`. This deliberately rules out the
flexibility of runtime service discovery (a new handler for an existing topic can't be
hot-swapped in) in exchange for a much simpler, fully-static, auditable topic-ownership
model — consistent with [RFC-0007](../rfcs/RFC-0007-settings-service.md) Security
Considerations making the same tradeoff for its routing table.

## 10. Authentication

No application-level auth token or handshake credential exists. Peer identity is
resolved once, at accept time, from the kernel-verified `SO_PEERCRED` socket option,
which yields the connecting process's PID/UID/GID — `nova-sessiond` maintains the
authoritative PID → `app_id` mapping (established at sandbox construction,
[01-INTERACTION-FLOWS.md](01-INTERACTION-FLOWS.md) §1) and the broker consults it for
every `Call`/`Publish`/`Subscribe` to resolve the sender's `app_id` before the ACL check
([RFC-0002](../rfcs/RFC-0002-nova-bus.md) Security Considerations). A connection cannot
claim a different identity than the kernel-verified one under any circumstance — there
is no field in `Connect` for a self-reported identity at all.

## 11. Error Codes

```protobuf
enum ErrorCode {
  UNKNOWN = 0;
  PERMISSION_DENIED = 1;       // ACL check failed
  TIMEOUT = 2;                  // §6
  NO_HANDLER = 3;                // Call to a topic with no registered handler
  HANDLER_ALREADY_REGISTERED = 4; // §9
  INVALID_ARGUMENT = 5;          // payload failed to deserialize against the
                                  // expected schema
  TOPIC_NOT_FOUND = 6;           // Call/Subscribe to a topic not in the static
                                  // schema registry at all (typo-class error)
  PROTOCOL_VERSION_UNSUPPORTED = 7; // §4/§8
  INTERNAL = 8;                   // handler-side error, opaque to the broker
}

message Error {
  uint64 correlation_id = 1;   // 0 if not associated with a specific Call
  ErrorCode code = 2;
  string message = 3;          // human-readable, non-sensitive detail
}
```

`PERMISSION_DENIED` is intentionally silent about *why* beyond the code itself in the
wire message (no leaking of other apps' grant state) — the requesting app sees "denied,"
and the reason (missing manifest permission vs. missing runtime grant) is only visible
in `nova-sessiond`'s own logs ([RFC-0008](../rfcs/RFC-0008-session-manager.md) Logging),
not echoed back to the (possibly malicious) caller.

## 12. Keepalive

`Ping`/`Pong` (empty messages) every 30s from broker to each client; a client that
doesn't respond within 10s is considered dead and disconnected (its subscriptions
dropped, any in-flight `Call` from it discarded) — bounds the "zombie connection"
failure mode referenced in [RFC-0002](../rfcs/RFC-0002-nova-bus.md) Failure Modes
("slow/backed-up delivery") to a maximum 40s detection window.
