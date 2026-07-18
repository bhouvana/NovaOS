# 0002: Nova Bus peer identity is self-reported, not `SO_PEERCRED`

Date: 2026-07-18
Status: Open — same trigger and close condition as
[0001](0001-nova-bus-dev-transport-tcp.md)

## Architecture

[15-NOVA-BUS-PROTOCOL-SPEC.md](../specs/15-NOVA-BUS-PROTOCOL-SPEC.md) §10 and
[RFC-0002](../rfcs/RFC-0002-nova-bus.md) §Security Considerations require peer identity
resolved from the kernel-verified `SO_PEERCRED` socket option — "a connection cannot
claim a different identity than the kernel-verified one under any circumstance."

## Reality

`Connect { identity: String }` in `proto/nova_bus.proto` carries a self-reported
identity string, trusted as-is by the broker
(`services/nova-bus-broker/src/broker.rs`'s `Command::Connect` handler).

## Reason

Direct consequence of [0001](0001-nova-bus-dev-transport-tcp.md): `SO_PEERCRED` is a
Unix-domain-socket-specific mechanism with no TCP equivalent (TCP loopback connections
carry no kernel-verified process-identity metadata the way a Unix socket credential
does).

## Decision

Documented prominently in `services/nova-bus-broker/src/broker.rs`'s module doc as "a
real, tracked security gap for this implementation, not a silent deviation." The ACL
enforcement mechanism itself (`Acl` trait, `AllowAll`/`DenyList`, the permission-denial
tests) is fully implemented and tested against whatever identity string arrives — the
gap is specifically "any process can claim any identity," not "permissions aren't
enforced once an identity is established." This means the vertical slice proves the ACL
*mechanism* works correctly (see the permission-denied and response-spoofing tests in
`services/nova-bus-broker/src/broker.rs`) without yet proving the *identity* feeding
into it is trustworthy.

## Future Direction

Bundled with [0001](0001-nova-bus-dev-transport-tcp.md)'s Unix-socket transport work:
once `UnixListener` is in use, resolve identity via `SO_PEERCRED` (available through
`tokio::net::unix::UCred` or the `nix` crate) and drop the self-reported `identity`
field from `Connect` entirely — a wire-protocol breaking change requiring an RFC
amendment to [RFC-0002](../rfcs/RFC-0002-nova-bus.md), not a silent code change, per
[../rfcs/README.md](../rfcs/README.md)'s "protocol change" trigger.
