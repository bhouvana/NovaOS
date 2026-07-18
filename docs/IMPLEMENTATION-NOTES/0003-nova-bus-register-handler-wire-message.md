# 0003: `RegisterHandler`/`RegisterHandlerAck` added to the wire protocol

Date: 2026-07-18
Status: Resolved — spec updated in the same change that introduced the code

## Architecture

[15-NOVA-BUS-PROTOCOL-SPEC.md](../specs/15-NOVA-BUS-PROTOCOL-SPEC.md) §9 describes
handler registration in prose ("The broker enforces [exactly one handler per topic] by
rejecting a second `RegisterHandler`...") but §3's `Envelope` `oneof` listing — the
actual wire schema — never included a `RegisterHandler` variant. The spec described a
mechanism it never gave a wire representation.

## Reality

`proto/nova_bus.proto` (`services/nova-bus/proto/nova_bus.proto`) defines
`RegisterHandler`/`RegisterHandlerAck` messages and adds them to `Envelope.kind` as
fields 14 and 15.

## Reason

A genuine gap in the Phase 1.5 spec, only visible once someone had to actually write
the `.proto` file and discovered §9 referenced a message type §3 never defined. This is
precisely the class of gap Phase 1.5 aimed to close relative to Phase 1's diagrams-only
level of detail, and precisely the class that no amount of additional reading-and-review
reliably catches — it took writing the schema to surface.

## Decision

Added the two message types to the `.proto` file with a module-doc comment explaining
the gap and pointing back to this note (see `proto/nova_bus.proto`'s header comment).
Also noted: handler registration is not yet ACL-restricted to privileged services (any
connected client can currently register a handler for any unclaimed topic) — the spec's
§9 language "an internal, privileged-only message not exposed to ordinary SDK clients"
describes an intended future restriction, not current behavior, since enforcing it
depends on `nova-sessiond`'s identity system, which doesn't exist in this vertical
slice.

## Future Direction

None needed for the schema gap itself — resolved by adding the fields, done. The
handler-registration privilege restriction remains tracked as a Phase 3 item (depends
on `nova-sessiond` existing to grant/deny that specific capability, same dependency
[0002](0002-nova-bus-identity-self-reported.md) has on real identity).
