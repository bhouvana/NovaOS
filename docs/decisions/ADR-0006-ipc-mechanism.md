# ADR-0006: IPC Mechanism (Nova Bus)

Status: Accepted
Date: 2026-07-18
Deciders: Chief Architect

## Context

Apps, the compositor, the session manager, and system services (notifications, settings,
package manager) all need to talk to each other: launch requests, notification delivery,
settings-changed events, permission prompts, window-management requests. We need one IPC
mechanism used everywhere, per the consistency principle.

## Options Considered

1. **D-Bus** — the de facto Linux desktop IPC standard; huge existing ecosystem
   (NetworkManager, UPower, etc. all speak it), but the daemon plus its XML introspection
   and type system is more machinery than a from-scratch, single-vendor OS needs, and
   pulls in a dependency (`dbus-daemon` or `dbus-broker`) we'd rather not carry as a
   baseline resident process if we don't need its ecosystem interop.
2. **Raw Unix domain sockets, ad hoc per-service protocols** — minimal, but "ad hoc"
   directly violates consistency; every service reinventing framing/serialization is a
   maintenance trap at 100k+ LOC.
3. **gRPC / HTTP-based local IPC** — good tooling, but HTTP framing and TLS-capable
   stacks are unnecessary weight for same-host IPC and add resident RAM per connection.
4. **Custom minimal bus over Unix domain sockets, length-prefixed Protobuf messages,
   single broker process ("Nova Bus")** — purpose-built: one small broker, one wire
   format, one client library (part of the SDK), capability-scoped topics/methods for the
   permission model ([08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md)).

## Decision

**Nova Bus**: a single lightweight broker process (`novabusd`, Rust) over Unix domain
sockets, using Protobuf for message framing, with a request/response (method call) mode
and a publish/subscribe (event) mode. It is the only IPC mechanism used by NovaOS-authored
components. A narrow, optional **D-Bus compatibility shim** may be added later, isolated
to specific hardware-integration daemons (e.g., a Bluetooth stack) that only speak D-Bus,
started on demand rather than resident by default.

## Rationale

A single small broker with one wire format is the smallest thing that satisfies "every
component talks the same way," which is what the permission model, the notification
system, and the settings system all depend on. Protobuf gives us schema evolution
(services can add fields without breaking old clients) without hand-rolling a wire format,
and Rust's protobuf tooling is mature. Keeping D-Bus as an optional, on-demand shim rather
than the default preserves ecosystem compatibility as an escape hatch without paying its
baseline cost.

## Consequences

- Third-party Linux daemons that assume `dbus-daemon` is always running at a well-known
  address won't work unmodified unless the compat shim is active — acceptable since
  NovaOS minimizes reliance on such daemons by design.
- The permission model ([08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md)) is enforced at
  the Nova Bus broker: every method call/topic subscription is checked against the
  calling app's granted capabilities, giving us one enforcement point instead of many.
- SDK exposes Nova Bus only through typed, generated client stubs (from `.proto`
  schemas) — app authors never hand-roll wire messages.

## Revisit Triggers

- If a hardware-integration dependency we must ship only works well via D-Bus and the
  shim proves consistently necessary rather than exceptional, promote it from optional to
  default-on for affected hardware profiles.
