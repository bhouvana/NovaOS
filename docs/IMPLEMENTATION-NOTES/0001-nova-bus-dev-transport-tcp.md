# 0001: Nova Bus dev transport is TCP loopback, not Unix sockets

Date: 2026-07-18
Status: Open — closes when a Linux build environment exists

## Architecture

[ADR-0006](../decisions/ADR-0006-ipc-mechanism.md) and
[15-NOVA-BUS-PROTOCOL-SPEC.md](../specs/15-NOVA-BUS-PROTOCOL-SPEC.md) §1 specify Nova
Bus running over Unix domain sockets at a fixed path (`/run/nova/bus.sock`).

## Reality

`services/nova-bus/src/client.rs` (`Client::connect_tcp`) and
`services/nova-bus-broker/src/server.rs` (`serve_tcp`, `bind_tcp`) use TCP loopback
(`127.0.0.1:<port>`) end to end.

## Reason

Phase 2 was implemented on a Windows development host. Windows 10+/Server support
`AF_UNIX`, but the ecosystem support (particularly `tokio::net::UnixListener`, which is
Unix-only in `tokio` 1.53) is not available on this platform without additional
platform-specific work. No Linux build environment (see
[0006](0006-process-memory-windows-vs-linux.md)) was available to validate a real
`UnixListener`/`UnixStream` implementation.

## Decision

Built and validated the entire protocol (framing, envelope, request/response routing,
pub/sub, timeouts, ACL) over TCP loopback, since none of that logic depends on which
byte-stream transport carries it — `Client::connect_over` (the transport-agnostic
generic-stream entry point) already isolates the wire-protocol code from the transport,
by design, specifically to make this substitution low-risk. Every place a real
deployment differs is confined to `connect_tcp`/`bind_tcp`/`serve_tcp` — three small
functions, not the protocol implementation.

## Future Direction

Once a Linux build environment exists (WSL2 with a full toolchain, a Linux CI runner,
or physical/VM Linux hardware), add `Client::connect_unix`/`server::serve_unix` using
`tokio::net::UnixListener`/`UnixStream`, gated `#[cfg(unix)]`, and switch
`novabusd`/app defaults to it. This is expected to be a small, low-risk change given the
transport/protocol separation already in place — tracked as a Phase 3 prerequisite, not
a Phase 2.5 finding requiring redesign.
