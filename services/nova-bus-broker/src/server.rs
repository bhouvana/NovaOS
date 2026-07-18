//! TCP accept loop wiring raw connections into the broker. Dev transport —
//! see lib.rs module doc; production is Unix domain sockets per
//! docs/specs/15-NOVA-BUS-PROTOCOL-SPEC.md §1.

use crate::broker::Broker;
use nova_bus::framing::{read_envelope, write_envelope};
use nova_bus::proto::envelope::Kind;
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, warn};

pub async fn serve_tcp(broker: Broker, addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    info!(addr, "novabusd listening");
    accept_loop(listener, broker).await
}

/// Split out from `serve_tcp` so tests can bind an ephemeral port (`:0`),
/// read back the OS-assigned real address, and only then start accepting —
/// avoids hardcoded-port test flakiness under parallel test execution.
pub async fn bind_tcp(addr: &str) -> std::io::Result<TcpListener> {
    TcpListener::bind(addr).await
}

pub async fn accept_loop(listener: TcpListener, broker: Broker) -> std::io::Result<()> {
    loop {
        let (stream, peer) = listener.accept().await?;
        let broker = broker.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(broker, stream).await {
                warn!(%peer, error = %e, "connection ended with error");
            }
        });
    }
}

async fn handle_connection(broker: Broker, stream: TcpStream) -> std::io::Result<()> {
    let (mut read_half, mut write_half) = stream.into_split();

    let first = read_envelope(&mut read_half)
        .await?
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "no Connect received"))?;
    let identity = match first.kind {
        Some(Kind::Connect(c)) => c.identity,
        other => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("expected Connect as first message, got {other:?}"),
            ))
        }
    };

    let (connection_id, mut outgoing_rx) = broker.connect(identity).await;

    // Writer task: everything the broker wants delivered to this
    // connection (including the ConnectAck already queued by `connect()`).
    let writer_task = tokio::spawn(async move {
        while let Some(envelope) = outgoing_rx.recv().await {
            if write_envelope(&mut write_half, &envelope).await.is_err() {
                break;
            }
        }
    });

    while let Ok(Some(envelope)) = read_envelope(&mut read_half).await {
        broker.dispatch(connection_id, envelope);
    }

    broker.disconnect(connection_id);
    writer_task.abort();
    Ok(())
}
