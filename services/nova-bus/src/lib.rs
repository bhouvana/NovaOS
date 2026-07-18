//! Nova Bus client/protocol crate: the "generated client stubs" per
//! docs/02-REPOSITORY-STRUCTURE.md §3 rule 2 — this is what `sdk/*` and
//! apps are allowed to depend on. The broker implementation
//! (`novabusd`) lives in the separate `nova-bus-broker` crate specifically
//! so nothing in `sdk/*` can accidentally reach broker internals; only this
//! crate's public API (`proto`, `framing`, `client`, `error`, `matching`) is
//! reachable from `sdk/nova-app` and friends.
//!
//! See docs/specs/15-NOVA-BUS-PROTOCOL-SPEC.md for the protocol this
//! implements, and docs/rfcs/RFC-0002-nova-bus.md for the broker's
//! operational contract.
//!
//! Transport note: production is Unix domain sockets
//! (docs/specs/15-NOVA-BUS-PROTOCOL-SPEC.md §1). `client::Client::connect_tcp`
//! uses TCP loopback instead, for this vertical slice's Windows development
//! environment — see docs/12-ROADMAP-AND-MILESTONES.md §4's Environment
//! note. `client::Client::connect_over` is transport-agnostic and is where a
//! real `UnixStream` would plug in on Linux.

pub mod client;
pub mod error;
pub mod framing;
pub mod matching;

pub mod proto {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/nova.bus.rs"));
}

pub use error::BusError;
