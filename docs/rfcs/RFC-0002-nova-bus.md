# RFC-0002: Nova Bus

Status: Accepted
Date: 2026-07-18
Owner: Chief Architect

## Purpose

Nova Bus (`novabusd`) is the single IPC broker every Nova process communicates through
([ADR-0006](../decisions/ADR-0006-ipc-mechanism.md)). It routes request/response calls
and pub/sub events, and enforces the permission ACL for every message
([../08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md) §2). Full wire-level detail is in
[15-NOVA-BUS-PROTOCOL-SPEC.md](../specs/15-NOVA-BUS-PROTOCOL-SPEC.md) — this RFC covers
the service's operational contract.

## Responsibilities

- Accept Unix domain socket connections from every Nova process.
- Route method calls (request/response) to the registered handler for a topic.
- Fan out pub/sub messages to all current subscribers of a topic.
- Resolve the calling process's `app_id`/service identity from `SO_PEERCRED` and enforce
  the permission ACL before routing ([15-NOVA-BUS-PROTOCOL-SPEC.md](../specs/15-NOVA-BUS-PROTOCOL-SPEC.md)
  §7).
- Nothing else: no message persistence, no delivery guarantees beyond "delivered to
  currently-connected subscribers" (§Failure Modes).

## Dependencies

None among Nova services — it is the first Nova process to start
([../01-SYSTEM-ARCHITECTURE.md](../01-SYSTEM-ARCHITECTURE.md) §2), depending only on the
kernel providing Unix domain sockets.

## Public APIs

Nova Bus itself exposes no application-level topics — its "API" is the wire protocol
(connect, call, subscribe, publish) specified in full in
[15-NOVA-BUS-PROTOCOL-SPEC.md](../specs/15-NOVA-BUS-PROTOCOL-SPEC.md). One
broker-internal topic exists for observability: `nova.bus.stats` (pub/sub, §Metrics).

## Events Published

`nova.bus.stats` — periodic (5s interval), connection count, message throughput,
per-topic subscriber counts. Consumed by Nova Monitor
([21-OBSERVABILITY-SPEC.md](../specs/21-OBSERVABILITY-SPEC.md)).

## Events Consumed

None — the broker routes, it does not act on message content.

## Configuration

Socket path (`/run/nova/bus.sock`, fixed, not configurable — a fixed well-known path is
simpler than service discovery, per
[15-NOVA-BUS-PROTOCOL-SPEC.md](../specs/15-NOVA-BUS-PROTOCOL-SPEC.md) §9), per-topic ACL
rules loaded from each installed app's manifest at `nova-sessiond` registration time
(pushed to `novabusd` via a privileged internal call, not read directly from disk by
the broker — keeps `novabusd` from needing filesystem-wide read access).

## Startup Order

First Nova process after OpenRC's minimal runlevel
([03-BOOT-TIMELINE.md](../specs/03-BOOT-TIMELINE.md) §1) — everything else blocks on its
socket existing.

## Failure Modes

- **Crash**: catastrophic — every Nova service loses IPC simultaneously. This is why
  `novabusd` is held to the highest code-quality/test-coverage bar of any Nova service
  (mirrors [../10-TESTING-AND-BUILD.md](../10-TESTING-AND-BUILD.md) §3's "shared,
  highest-blast-radius code" ≥80% coverage target, applied here as the strictest
  instance of that rule).
- **Slow/backed-up delivery**: a misbehaving subscriber that never drains its socket
  buffer is disconnected after a bounded backlog threshold (its subscription is dropped,
  not the whole broker slowed) — one slow consumer never blocks delivery to others.
- **Subscriber disconnected at publish time**: message is dropped for that subscriber,
  not queued (§Recovery Strategy) — Nova Bus is at-most-once, not a durable queue.

## Recovery Strategy

OpenRC restarts `novabusd` on crash (it's a supervised system service, not a
Session-Manager-supervised app — [../01-SYSTEM-ARCHITECTURE.md](../01-SYSTEM-ARCHITECTURE.md)
§3/[ADR-0002](../decisions/ADR-0002-init-and-service-supervision.md)). Every other Nova
process reconnects on socket-error with exponential backoff (capped at 5 retries within
2s before surfacing a fatal error to that process's own supervisor). No message replay
on reconnect — clients that need durable state re-fetch it via a fresh request after
reconnecting, rather than the bus attempting to buffer/replay missed events.

## Metrics

Connection count, messages/sec (by topic), permission-denial count (ACL rejections —
a spike here is a security-relevant signal, not just a debugging one), p50/p99 message
routing latency.

## Logging

Connection/disconnection events (info), ACL denials (warn — these are potential
security events per §Security Considerations), broker-internal errors (error). Message
*payloads* are never logged by the broker itself (it doesn't inspect payload content at
all beyond the envelope needed for routing, [15-NOVA-BUS-PROTOCOL-SPEC.md](../specs/15-NOVA-BUS-PROTOCOL-SPEC.md)
§2) — this is a structural privacy property, not just a logging policy.

## Security Considerations

`novabusd` is the single enforcement point for the entire permission model
([../08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md) §2) — a bug in its ACL check logic
is equivalent in severity to a bug in the sandboxing code
([ADR-0010](../decisions/ADR-0010-app-sandboxing-model.md)). Peer identity is resolved
from the kernel-verified `SO_PEERCRED` credential of the connecting socket, never from a
self-reported field in the connection handshake — an app cannot claim to be a different
`app_id` regardless of what it sends. ACL check happens before routing, not after
(a denied message is never delivered to any subscriber, even partially).

**Response authorization** (added after a Phase 2 implementation finding —
[IMPLEMENTATION-NOTES/0004](../IMPLEMENTATION-NOTES/0004-nova-bus-response-authorization.md)):
the broker must verify that a `Response` for a given broker-assigned `request_id`
originates from the same connection the corresponding `Call` was forwarded to, not
merely that the `request_id` is currently pending. Request IDs are sequential and
therefore guessable; without this check, any connected client could forge a `Response`
for another client's in-flight call. This is a caller-authenticity check distinct from
the topic-level ACL check above — a connection can be fully authorized to call a topic
and still be an unauthorized *responder* for someone else's unrelated pending call.

## Changelog

- 2026-07-18: Accepted.
- 2026-07-18: Added Response authorization requirement to Security Considerations,
  following a vulnerability found and fixed during Phase 2 implementation
  ([IMPLEMENTATION-NOTES/0004](../IMPLEMENTATION-NOTES/0004-nova-bus-response-authorization.md)).
