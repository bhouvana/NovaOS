//! `novabusd` — the Nova Bus broker binary.
//! docs/rfcs/RFC-0002-nova-bus.md §Startup Order: first Nova process after
//! the base system's minimal runlevel; everything else blocks on this
//! socket existing.
//!
//! Dev transport: binds TCP loopback, not a Unix socket — see
//! services/nova-bus/src/lib.rs module doc.

use nova_bus_broker::broker::{AllowAll, Broker};
use nova_bus_broker::server::serve_tcp;

const DEFAULT_ADDR: &str = "127.0.0.1:7780";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let addr = std::env::var("NOVA_BUS_ADDR").unwrap_or_else(|_| DEFAULT_ADDR.to_string());
    let broker = Broker::spawn(AllowAll);
    serve_tcp(broker, &addr).await
}
