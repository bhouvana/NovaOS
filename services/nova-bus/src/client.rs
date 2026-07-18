//! Client library for connecting to a running Nova Bus broker.
//! Implements the caller-side and handler-side halves of
//! docs/specs/15-NOVA-BUS-PROTOCOL-SPEC.md §4–§7.

use crate::error::BusError;
use crate::framing::{read_envelope, write_envelope};
use crate::proto::{
    envelope::Kind, Call, Connect, Publish, RegisterHandler, Response, Subscribe, Unsubscribe,
};

/// Protocol version this client speaks — docs/specs/15-NOVA-BUS-PROTOCOL-SPEC.md §8.
pub const PROTOCOL_VERSION: u32 = 1;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot, Mutex};

type PendingCalls = Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Vec<u8>, BusError>>>>>;
type HandlerTopics = Arc<Mutex<HashMap<String, mpsc::UnboundedSender<IncomingCall>>>>;
type SubscriptionTopics = Arc<Mutex<HashMap<String, mpsc::UnboundedSender<Vec<u8>>>>>;

/// A `Call` delivered to a registered handler, awaiting `respond()`.
pub struct IncomingCall {
    correlation_id: u64,
    pub topic: String,
    pub payload: Vec<u8>,
    responder: mpsc::UnboundedSender<Envelope>,
}

use crate::proto::Envelope;

impl IncomingCall {
    pub fn respond(self, payload: Vec<u8>) {
        let _ = self.responder.send(Envelope {
            protocol_version: PROTOCOL_VERSION,
            trace_id: None,
            kind: Some(Kind::Response(Response {
                correlation_id: self.correlation_id,
                payload,
            })),
        });
    }
}

/// A connected Nova Bus client. Cheap to clone; all clones share the same
/// underlying connection and dispatcher task.
#[derive(Clone)]
pub struct Client {
    connection_id: u64,
    outgoing: mpsc::UnboundedSender<Envelope>,
    next_correlation_id: Arc<std::sync::atomic::AtomicU64>,
    pending_calls: PendingCalls,
    handlers: HandlerTopics,
    subscriptions: SubscriptionTopics,
}

impl Client {
    /// Dev-transport connection over TCP loopback — see lib.rs module doc
    /// for why this isn't a Unix socket in this vertical slice.
    pub async fn connect_tcp(addr: &str, identity: &str) -> Result<Self, BusError> {
        let stream = TcpStream::connect(addr).await?;
        let (read_half, write_half) = stream.into_split();
        Self::connect_over(read_half, write_half, identity).await
    }

    /// Generic connect over any split async stream — used directly by tests
    /// with `tokio::io::duplex`, and by `connect_tcp` above.
    pub async fn connect_over<R, W>(
        mut reader: R,
        mut writer: W,
        identity: &str,
    ) -> Result<Self, BusError>
    where
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        write_envelope(
            &mut writer,
            &Envelope {
                protocol_version: PROTOCOL_VERSION,
                trace_id: None,
                kind: Some(Kind::Connect(Connect {
                    protocol_version: PROTOCOL_VERSION,
                    identity: identity.to_string(),
                })),
            },
        )
        .await?;

        let ack = read_envelope(&mut reader)
            .await?
            .ok_or(BusError::ConnectionClosed)?;
        let connection_id = match ack.kind {
            Some(Kind::ConnectAck(a)) => a.connection_id,
            Some(Kind::Error(e)) => {
                return Err(BusError::from_wire(e.code, "", e.message));
            }
            other => {
                return Err(BusError::Internal(format!(
                    "expected ConnectAck, got {other:?}"
                )))
            }
        };

        let (outgoing_tx, mut outgoing_rx) = mpsc::unbounded_channel::<Envelope>();

        // Writer task: drains outgoing envelopes onto the wire.
        tokio::spawn(async move {
            while let Some(envelope) = outgoing_rx.recv().await {
                if write_envelope(&mut writer, &envelope).await.is_err() {
                    break;
                }
            }
        });

        let pending_calls: PendingCalls = Arc::new(Mutex::new(HashMap::new()));
        let handlers: HandlerTopics = Arc::new(Mutex::new(HashMap::new()));
        let subscriptions: SubscriptionTopics = Arc::new(Mutex::new(HashMap::new()));

        // Reader task: demultiplexes inbound envelopes to whichever
        // waiter/channel is responsible for them.
        {
            let pending_calls = pending_calls.clone();
            let handlers = handlers.clone();
            let subscriptions = subscriptions.clone();
            let outgoing_tx = outgoing_tx.clone();
            tokio::spawn(async move {
                loop {
                    let envelope = match read_envelope(&mut reader).await {
                        Ok(Some(e)) => e,
                        _ => break,
                    };
                    dispatch_inbound(
                        envelope,
                        &pending_calls,
                        &handlers,
                        &subscriptions,
                        &outgoing_tx,
                    )
                    .await;
                }
            });
        }

        Ok(Client {
            connection_id,
            outgoing: outgoing_tx,
            next_correlation_id: Arc::new(std::sync::atomic::AtomicU64::new(1)),
            pending_calls,
            handlers,
            subscriptions,
        })
    }

    pub fn connection_id(&self) -> u64 {
        self.connection_id
    }

    /// §5/§6: request/response with the protocol's default (or an
    /// explicit) timeout.
    pub async fn call(
        &self,
        topic: &str,
        payload: Vec<u8>,
        timeout_ms: u32,
    ) -> Result<Vec<u8>, BusError> {
        let correlation_id = self
            .next_correlation_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        self.pending_calls.lock().await.insert(correlation_id, tx);

        let _ = self.outgoing.send(Envelope {
            protocol_version: PROTOCOL_VERSION,
            trace_id: None,
            kind: Some(Kind::Call(Call {
                correlation_id,
                topic: topic.to_string(),
                payload,
                timeout_ms,
            })),
        });

        rx.await.unwrap_or(Err(BusError::ConnectionClosed))
    }

    /// §9: registers this connection as the sole handler for `topic`.
    /// Returns a channel of `IncomingCall`s to respond to.
    pub async fn register_handler(
        &self,
        topic: &str,
    ) -> Result<mpsc::UnboundedReceiver<IncomingCall>, BusError> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.handlers.lock().await.insert(topic.to_string(), tx);
        let _ = self.outgoing.send(Envelope {
            protocol_version: PROTOCOL_VERSION,
            trace_id: None,
            kind: Some(Kind::RegisterHandler(RegisterHandler {
                topic: topic.to_string(),
            })),
        });
        Ok(rx)
    }

    /// §7: subscribe to a topic (exact or trailing-wildcard).
    pub async fn subscribe(&self, topic: &str) -> Result<mpsc::UnboundedReceiver<Vec<u8>>, BusError> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.subscriptions.lock().await.insert(topic.to_string(), tx);
        let _ = self.outgoing.send(Envelope {
            protocol_version: PROTOCOL_VERSION,
            trace_id: None,
            kind: Some(Kind::Subscribe(Subscribe {
                topic: topic.to_string(),
            })),
        });
        Ok(rx)
    }

    pub async fn unsubscribe(&self, topic: &str) {
        self.subscriptions.lock().await.remove(topic);
        let _ = self.outgoing.send(Envelope {
            protocol_version: PROTOCOL_VERSION,
            trace_id: None,
            kind: Some(Kind::Unsubscribe(Unsubscribe {
                topic: topic.to_string(),
            })),
        });
    }

    pub fn publish(&self, topic: &str, payload: Vec<u8>) {
        let _ = self.outgoing.send(Envelope {
            protocol_version: PROTOCOL_VERSION,
            trace_id: None,
            kind: Some(Kind::Publish(Publish {
                topic: topic.to_string(),
                payload,
            })),
        });
    }
}

async fn dispatch_inbound(
    envelope: Envelope,
    pending_calls: &PendingCalls,
    handlers: &HandlerTopics,
    subscriptions: &SubscriptionTopics,
    outgoing: &mpsc::UnboundedSender<Envelope>,
) {
    match envelope.kind {
        Some(Kind::Response(r)) => {
            if let Some(tx) = pending_calls.lock().await.remove(&r.correlation_id) {
                let _ = tx.send(Ok(r.payload));
            }
        }
        Some(Kind::Error(e)) => {
            if let Some(tx) = pending_calls.lock().await.remove(&e.correlation_id) {
                let _ = tx.send(Err(BusError::from_wire(e.code, "", e.message)));
            }
        }
        Some(Kind::Call(c)) => {
            // A Call arriving here means the broker is delivering it to us
            // because we're the registered handler for `c.topic`.
            if let Some(tx) = handlers.lock().await.get(&c.topic) {
                let _ = tx.send(IncomingCall {
                    correlation_id: c.correlation_id,
                    topic: c.topic,
                    payload: c.payload,
                    responder: outgoing.clone(),
                });
            }
        }
        Some(Kind::Publish(p)) => {
            let subs = subscriptions.lock().await;
            for (pattern, tx) in subs.iter() {
                if crate::matching::topic_matches(pattern, &p.topic) {
                    let _ = tx.send(p.payload.clone());
                }
            }
        }
        Some(Kind::ConnectAck(_))
        | Some(Kind::RegisterHandlerAck(_))
        | Some(Kind::SubscribeAck(_))
        | Some(Kind::Ping(_))
        | Some(Kind::Pong(_)) => {
            // Acks are fire-and-forget for this vertical slice (no
            // caller currently awaits them); Ping/Pong keepalive per §12
            // is not yet implemented over the dev TCP transport.
        }
        _ => {}
    }
}
