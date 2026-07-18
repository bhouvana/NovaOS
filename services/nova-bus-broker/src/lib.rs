//! The Nova Bus broker implementation. Deliberately a separate crate from
//! `nova-bus` (which holds only the client/proto "stubs") so that nothing in
//! `sdk/*` can depend on broker internals — see
//! docs/02-REPOSITORY-STRUCTURE.md §3 rule 2 and the module doc in
//! `services/nova-bus/src/lib.rs`.

pub mod broker;
pub mod server;
