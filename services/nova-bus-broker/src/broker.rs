//! The Nova Bus broker: registers handlers, routes `Call`/`Response` pairs,
//! fans out `Publish` to subscribers, and enforces the permission ACL.
//! Implements docs/specs/15-NOVA-BUS-PROTOCOL-SPEC.md §5, §7, §9, §11 and the
//! operational contract in docs/rfcs/RFC-0002-nova-bus.md.
//!
//! Identity note: RFC-0002 §Security Considerations requires peer identity
//! resolved from the kernel-verified `SO_PEERCRED`, never self-reported. This
//! vertical-slice implementation accepts a self-reported `identity` string in
//! `Connect` instead, because the TCP dev transport (see lib.rs module doc)
//! has no equivalent of `SO_PEERCRED`. This is a real, tracked security gap
//! for this implementation, not a silent deviation — it is closed when the
//! production Unix-socket transport lands (Phase 3+, once a Linux build
//! environment exists) and is listed as a required Phase 2.5 finding.

use nova_bus::client::PROTOCOL_VERSION;
use nova_bus::matching::topic_matches;
use nova_bus::proto::{
    envelope::Kind, Call, ConnectAck, Envelope, Error as WireError, ErrorCode, Publish, Response,
    SubscribeAck,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, warn};

pub const DEFAULT_TIMEOUT_MS: u32 = 5000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AclAction {
    Call,
    RegisterHandler,
    Publish,
    Subscribe,
}

/// Permission enforcement point — RFC-0002 §Security Considerations: this is
/// the single place a message is allowed or denied, mirroring
/// docs/08-SECURITY-MODEL.md §2's "one enforcement point" design.
pub trait Acl: Send + Sync + 'static {
    fn allow(&self, identity: &str, action: AclAction, topic: &str) -> bool;
}

/// Default policy: everything allowed. Used until a real manifest-derived
/// ACL (depending on nova-sessiond, which doesn't exist in this slice) is
/// wired in.
pub struct AllowAll;
impl Acl for AllowAll {
    fn allow(&self, _identity: &str, _action: AclAction, _topic: &str) -> bool {
        true
    }
}

/// Denies exactly the (identity, topic) pairs listed — used by tests and by
/// anything wanting to exercise the PERMISSION_DENIED path deliberately.
#[derive(Default)]
pub struct DenyList {
    denied: HashSet<(String, String)>,
}
impl DenyList {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn deny(mut self, identity: impl Into<String>, topic: impl Into<String>) -> Self {
        self.denied.insert((identity.into(), topic.into()));
        self
    }
}
impl Acl for DenyList {
    fn allow(&self, identity: &str, _action: AclAction, topic: &str) -> bool {
        !self.denied.contains(&(identity.to_string(), topic.to_string()))
    }
}

struct PendingCall {
    caller_connection: u64,
    caller_correlation_id: u64,
    handler_connection: u64,
    topic: String,
}

struct BrokerState {
    next_connection_id: u64,
    next_request_id: u64,
    connections: HashMap<u64, mpsc::UnboundedSender<Envelope>>,
    identities: HashMap<u64, String>,
    handlers: HashMap<String, u64>,
    subscriptions: HashMap<u64, HashSet<String>>,
    pending: HashMap<u64, PendingCall>,
}

impl BrokerState {
    fn new() -> Self {
        Self {
            next_connection_id: 1,
            next_request_id: 1,
            connections: HashMap::new(),
            identities: HashMap::new(),
            handlers: HashMap::new(),
            subscriptions: HashMap::new(),
            pending: HashMap::new(),
        }
    }

    fn send_to(&self, connection_id: u64, envelope: Envelope) {
        if let Some(tx) = self.connections.get(&connection_id) {
            // A closed receiver just means the connection is tearing down
            // concurrently; nothing to escalate.
            let _ = tx.send(envelope);
        }
    }

    fn send_error(&self, connection_id: u64, correlation_id: u64, code: ErrorCode, message: String) {
        self.send_to(
            connection_id,
            Envelope {
                protocol_version: PROTOCOL_VERSION,
                trace_id: None,
                kind: Some(Kind::Error(WireError {
                    correlation_id,
                    code: code as i32,
                    message,
                })),
            },
        );
    }
}

pub struct Broker {
    tx: mpsc::UnboundedSender<Command>,
}

enum Command {
    Connect {
        identity: String,
        outgoing: mpsc::UnboundedSender<Envelope>,
        reply: oneshot::Sender<u64>,
    },
    Disconnect {
        connection_id: u64,
    },
    RegisterHandler {
        connection_id: u64,
        topic: String,
    },
    Call {
        connection_id: u64,
        correlation_id: u64,
        topic: String,
        payload: Vec<u8>,
        timeout_ms: u32,
    },
    Response {
        connection_id: u64,
        correlation_id: u64,
        payload: Vec<u8>,
    },
    Subscribe {
        connection_id: u64,
        topic: String,
    },
    Unsubscribe {
        connection_id: u64,
        topic: String,
    },
    Publish {
        connection_id: u64,
        topic: String,
        payload: Vec<u8>,
    },
    TimeoutFired {
        request_id: u64,
    },
}

impl Broker {
    /// Spawns the broker's single-threaded actor task and returns a cheap,
    /// cloneable handle. Serializing all state mutation through one task
    /// (rather than a shared `Mutex<BrokerState>`) makes the routing logic
    /// trivially free of races by construction — see §7's Publish fan-out
    /// and §5's Call/Response correlation, both of which read-then-write
    /// state that would otherwise need careful lock discipline.
    pub fn spawn(acl: impl Acl) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<Command>();
        let acl = Arc::new(acl);
        let actor_tx = tx.clone();

        tokio::spawn(async move {
            let mut state = BrokerState::new();
            while let Some(cmd) = rx.recv().await {
                handle_command(&mut state, &acl, &actor_tx, cmd);
            }
        });

        Broker { tx }
    }

    /// Registers a new connection with the broker and returns its
    /// broker-assigned `connection_id` plus a channel of envelopes destined
    /// for it (the caller is responsible for writing those to the wire).
    pub async fn connect(
        &self,
        identity: impl Into<String>,
    ) -> (u64, mpsc::UnboundedReceiver<Envelope>) {
        let (outgoing_tx, outgoing_rx) = mpsc::unbounded_channel();
        let (reply_tx, reply_rx) = oneshot::channel();
        let _ = self.tx.send(Command::Connect {
            identity: identity.into(),
            outgoing: outgoing_tx,
            reply: reply_tx,
        });
        let connection_id = reply_rx.await.expect("broker actor task died");
        (connection_id, outgoing_rx)
    }

    pub fn disconnect(&self, connection_id: u64) {
        let _ = self.tx.send(Command::Disconnect { connection_id });
    }

    /// Feeds one parsed `Envelope` received from `connection_id` into the
    /// broker. This is the single entry point a per-connection reader task
    /// calls for every inbound message.
    pub fn dispatch(&self, connection_id: u64, envelope: Envelope) {
        let Some(kind) = envelope.kind else {
            warn!(connection_id, "received envelope with no kind, ignoring");
            return;
        };
        let cmd = match kind {
            Kind::RegisterHandler(m) => Command::RegisterHandler {
                connection_id,
                topic: m.topic,
            },
            Kind::Call(Call {
                correlation_id,
                topic,
                payload,
                timeout_ms,
            }) => Command::Call {
                connection_id,
                correlation_id,
                topic,
                payload,
                timeout_ms,
            },
            Kind::Response(Response {
                correlation_id,
                payload,
            }) => Command::Response {
                connection_id,
                correlation_id,
                payload,
            },
            Kind::Subscribe(m) => Command::Subscribe {
                connection_id,
                topic: m.topic,
            },
            Kind::Unsubscribe(m) => Command::Unsubscribe {
                connection_id,
                topic: m.topic,
            },
            Kind::Publish(Publish { topic, payload }) => Command::Publish {
                connection_id,
                topic,
                payload,
            },
            Kind::Ping(_) => {
                // Pong handling lives at the transport layer (see client.rs)
                // for the dev transport; kept minimal here.
                return;
            }
            other => {
                debug!(connection_id, ?other, "unhandled envelope kind at broker");
                return;
            }
        };
        let _ = self.tx.send(cmd);
    }
}

impl Clone for Broker {
    fn clone(&self) -> Self {
        Broker { tx: self.tx.clone() }
    }
}

fn handle_command(
    state: &mut BrokerState,
    acl: &Arc<impl Acl + ?Sized>,
    actor_tx: &mpsc::UnboundedSender<Command>,
    cmd: Command,
) {
    match cmd {
        Command::Connect {
            identity,
            outgoing,
            reply,
        } => {
            let id = state.next_connection_id;
            state.next_connection_id += 1;
            state.connections.insert(id, outgoing.clone());
            state.identities.insert(id, identity);
            let _ = outgoing.send(Envelope {
                protocol_version: PROTOCOL_VERSION,
                trace_id: None,
                kind: Some(Kind::ConnectAck(ConnectAck {
                    protocol_version: PROTOCOL_VERSION,
                    connection_id: id,
                })),
            });
            let _ = reply.send(id);
        }

        Command::Disconnect { connection_id } => {
            state.connections.remove(&connection_id);
            state.identities.remove(&connection_id);
            state.subscriptions.remove(&connection_id);
            state.handlers.retain(|_, v| *v != connection_id);
            // Pending calls whose caller disconnected are left to time out
            // naturally rather than tearing down more state here — keeps
            // this transition simple, matching RFC-0002's "no message
            // persistence" no-durable-queue philosophy applied to cleanup.
        }

        Command::RegisterHandler { connection_id, topic } => {
            let identity = state.identities.get(&connection_id).cloned().unwrap_or_default();
            if !acl.allow(&identity, AclAction::RegisterHandler, &topic) {
                state.send_error(connection_id, 0, ErrorCode::PermissionDenied, topic);
                return;
            }
            if state.handlers.contains_key(&topic) {
                state.send_error(
                    connection_id,
                    0,
                    ErrorCode::HandlerAlreadyRegistered,
                    topic,
                );
                return;
            }
            state.handlers.insert(topic.clone(), connection_id);
            state.send_to(
                connection_id,
                Envelope {
                    protocol_version: PROTOCOL_VERSION,
                    trace_id: None,
                    kind: Some(Kind::RegisterHandlerAck(
                        nova_bus::proto::RegisterHandlerAck { topic },
                    )),
                },
            );
        }

        Command::Call {
            connection_id,
            correlation_id,
            topic,
            payload,
            timeout_ms,
        } => {
            let identity = state.identities.get(&connection_id).cloned().unwrap_or_default();
            if !acl.allow(&identity, AclAction::Call, &topic) {
                state.send_error(connection_id, correlation_id, ErrorCode::PermissionDenied, topic);
                return;
            }
            let Some(&handler_conn) = state.handlers.get(&topic) else {
                state.send_error(connection_id, correlation_id, ErrorCode::NoHandler, topic);
                return;
            };

            let request_id = state.next_request_id;
            state.next_request_id += 1;
            state.pending.insert(
                request_id,
                PendingCall {
                    caller_connection: connection_id,
                    caller_correlation_id: correlation_id,
                    handler_connection: handler_conn,
                    topic: topic.clone(),
                },
            );

            state.send_to(
                handler_conn,
                Envelope {
                    protocol_version: PROTOCOL_VERSION,
                    trace_id: None,
                    kind: Some(Kind::Call(Call {
                        correlation_id: request_id,
                        topic,
                        payload,
                        timeout_ms,
                    })),
                },
            );

            // §6: default timeout applies when the caller didn't override it.
            let effective_timeout = if timeout_ms == 0 {
                DEFAULT_TIMEOUT_MS
            } else {
                timeout_ms
            };
            let timeout_tx = actor_tx.clone();
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(effective_timeout as u64)).await;
                let _ = timeout_tx.send(Command::TimeoutFired { request_id });
            });
        }

        Command::Response {
            connection_id,
            correlation_id: request_id,
            payload,
        } => {
            // §5: a Response's correlation_id echoes the broker-assigned
            // request_id we sent the handler in the forwarded Call. Found
            // during Phase 2 implementation (not originally called out in
            // the spec): without checking that the RESPONDING connection is
            // the same one the broker actually forwarded the Call to, any
            // connected client could guess/observe a request_id and inject
            // a forged Response for someone else's in-flight call — a
            // spoofing vector RFC-0002's Security Considerations didn't
            // enumerate. Peek (not remove) so a mismatched sender doesn't
            // consume/invalidate the real pending call.
            let is_authorized = matches!(
                state.pending.get(&request_id),
                Some(p) if p.handler_connection == connection_id
            );
            if !is_authorized {
                warn!(
                    connection_id,
                    request_id, "Response from a connection that wasn't the forwarded call's handler — dropped"
                );
                return;
            }
            if let Some(pending) = state.pending.remove(&request_id) {
                state.send_to(
                    pending.caller_connection,
                    Envelope {
                        protocol_version: PROTOCOL_VERSION,
                        trace_id: None,
                        kind: Some(Kind::Response(Response {
                            correlation_id: pending.caller_correlation_id,
                            payload,
                        })),
                    },
                );
            }
            // If `pending` is already gone, the request already timed out —
            // a late Response is silently dropped, matching §6's "caller-side
            // give-up, handler not notified" semantics.
        }

        Command::TimeoutFired { request_id } => {
            if let Some(pending) = state.pending.remove(&request_id) {
                state.send_error(
                    pending.caller_connection,
                    pending.caller_correlation_id,
                    ErrorCode::Timeout,
                    pending.topic,
                );
            }
            // Already-completed requests: no-op, this is the race §6 expects.
        }

        Command::Subscribe { connection_id, topic } => {
            let identity = state.identities.get(&connection_id).cloned().unwrap_or_default();
            if !acl.allow(&identity, AclAction::Subscribe, &topic) {
                state.send_error(connection_id, 0, ErrorCode::PermissionDenied, topic);
                return;
            }
            state
                .subscriptions
                .entry(connection_id)
                .or_default()
                .insert(topic.clone());
            state.send_to(
                connection_id,
                Envelope {
                    protocol_version: PROTOCOL_VERSION,
                    trace_id: None,
                    kind: Some(Kind::SubscribeAck(SubscribeAck { topic })),
                },
            );
        }

        Command::Unsubscribe { connection_id, topic } => {
            if let Some(set) = state.subscriptions.get_mut(&connection_id) {
                set.remove(&topic);
            }
        }

        Command::Publish {
            connection_id,
            topic,
            payload,
        } => {
            let identity = state.identities.get(&connection_id).cloned().unwrap_or_default();
            if !acl.allow(&identity, AclAction::Publish, &topic) {
                state.send_error(connection_id, 0, ErrorCode::PermissionDenied, topic);
                return;
            }
            // §7: at-most-once fan-out to currently-connected subscribers;
            // zero subscribers is not an error.
            for (&sub_conn, patterns) in state.subscriptions.iter() {
                if patterns.iter().any(|p| topic_matches(p, &topic)) {
                    state.send_to(
                        sub_conn,
                        Envelope {
                            protocol_version: PROTOCOL_VERSION,
                            trace_id: None,
                            kind: Some(Kind::Publish(Publish {
                                topic: topic.clone(),
                                payload: payload.clone(),
                            })),
                        },
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn connect_assigns_increasing_connection_ids() {
        let broker = Broker::spawn(AllowAll);
        let (id_a, _rx_a) = broker.connect("app.a").await;
        let (id_b, _rx_b) = broker.connect("app.b").await;
        assert_ne!(id_a, id_b);
    }

    #[tokio::test]
    async fn call_with_no_handler_returns_no_handler_error() {
        let broker = Broker::spawn(AllowAll);
        let (caller_id, mut caller_rx) = broker.connect("app.caller").await;
        let _ = caller_rx.recv().await; // ConnectAck

        broker.dispatch(
            caller_id,
            Envelope {
                protocol_version: PROTOCOL_VERSION,
                trace_id: None,
                kind: Some(Kind::Call(Call {
                    correlation_id: 42,
                    topic: "nova.nonexistent".into(),
                    payload: vec![],
                    timeout_ms: 1000,
                })),
            },
        );

        let reply = caller_rx.recv().await.unwrap();
        match reply.kind {
            Some(Kind::Error(e)) => {
                assert_eq!(e.correlation_id, 42);
                assert_eq!(e.code, ErrorCode::NoHandler as i32);
            }
            other => panic!("expected Error(NoHandler), got {other:?}"),
        }
    }

    #[tokio::test]
    async fn call_routes_to_handler_and_response_routes_back() {
        let broker = Broker::spawn(AllowAll);

        let (handler_id, mut handler_rx) = broker.connect("nova.sessiond").await;
        let _ = handler_rx.recv().await; // ConnectAck
        broker.dispatch(
            handler_id,
            Envelope {
                protocol_version: PROTOCOL_VERSION,
                trace_id: None,
                kind: Some(Kind::RegisterHandler(nova_bus::proto::RegisterHandler {
                    topic: "nova.session.launch".into(),
                })),
            },
        );
        let ack = handler_rx.recv().await.unwrap();
        assert!(matches!(ack.kind, Some(Kind::RegisterHandlerAck(_))));

        let (caller_id, mut caller_rx) = broker.connect("nova.shell").await;
        let _ = caller_rx.recv().await; // ConnectAck
        broker.dispatch(
            caller_id,
            Envelope {
                protocol_version: PROTOCOL_VERSION,
                trace_id: None,
                kind: Some(Kind::Call(Call {
                    correlation_id: 7,
                    topic: "nova.session.launch".into(),
                    payload: b"hello".to_vec(),
                    timeout_ms: 2000,
                })),
            },
        );

        // Handler receives a forwarded Call (broker-assigned correlation_id,
        // per §5 — not the caller's original 7).
        let forwarded = handler_rx.recv().await.unwrap();
        let (broker_request_id, payload) = match forwarded.kind {
            Some(Kind::Call(c)) => (c.correlation_id, c.payload),
            other => panic!("expected forwarded Call, got {other:?}"),
        };
        assert_eq!(payload, b"hello");
        assert_ne!(broker_request_id, 7);

        // Handler replies.
        broker.dispatch(
            handler_id,
            Envelope {
                protocol_version: PROTOCOL_VERSION,
                trace_id: None,
                kind: Some(Kind::Response(Response {
                    correlation_id: broker_request_id,
                    payload: b"ok".to_vec(),
                })),
            },
        );

        // Caller receives a Response with ITS OWN original correlation_id.
        let response = caller_rx.recv().await.unwrap();
        match response.kind {
            Some(Kind::Response(r)) => {
                assert_eq!(r.correlation_id, 7);
                assert_eq!(r.payload, b"ok");
            }
            other => panic!("expected Response, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn duplicate_handler_registration_is_rejected() {
        let broker = Broker::spawn(AllowAll);
        let (a, mut rx_a) = broker.connect("a").await;
        let _ = rx_a.recv().await;
        let (b, mut rx_b) = broker.connect("b").await;
        let _ = rx_b.recv().await;

        let register = |topic: &str| {
            Envelope {
                protocol_version: PROTOCOL_VERSION,
                trace_id: None,
                kind: Some(Kind::RegisterHandler(nova_bus::proto::RegisterHandler {
                    topic: topic.to_string(),
                })),
            }
        };
        broker.dispatch(a, register("nova.dup"));
        assert!(matches!(
            rx_a.recv().await.unwrap().kind,
            Some(Kind::RegisterHandlerAck(_))
        ));

        broker.dispatch(b, register("nova.dup"));
        match rx_b.recv().await.unwrap().kind {
            Some(Kind::Error(e)) => {
                assert_eq!(e.code, ErrorCode::HandlerAlreadyRegistered as i32)
            }
            other => panic!("expected HANDLER_ALREADY_REGISTERED, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn call_times_out_when_handler_never_responds() {
        let broker = Broker::spawn(AllowAll);
        let (handler_id, mut handler_rx) = broker.connect("slow.handler").await;
        let _ = handler_rx.recv().await;
        broker.dispatch(
            handler_id,
            Envelope {
                protocol_version: PROTOCOL_VERSION,
                trace_id: None,
                kind: Some(Kind::RegisterHandler(nova_bus::proto::RegisterHandler {
                    topic: "nova.slow".into(),
                })),
            },
        );
        let _ = handler_rx.recv().await; // ack

        let (caller_id, mut caller_rx) = broker.connect("impatient.caller").await;
        let _ = caller_rx.recv().await;
        broker.dispatch(
            caller_id,
            Envelope {
                protocol_version: PROTOCOL_VERSION,
                trace_id: None,
                kind: Some(Kind::Call(Call {
                    correlation_id: 99,
                    topic: "nova.slow".into(),
                    payload: vec![],
                    timeout_ms: 50, // never overridden by DEFAULT_TIMEOUT_MS
                })),
            },
        );
        let _forwarded = handler_rx.recv().await; // handler gets it, never replies

        let reply = caller_rx.recv().await.unwrap();
        match reply.kind {
            Some(Kind::Error(e)) => {
                assert_eq!(e.correlation_id, 99);
                assert_eq!(e.code, ErrorCode::Timeout as i32);
            }
            other => panic!("expected Error(Timeout), got {other:?}"),
        }
    }

    #[tokio::test]
    async fn a_response_from_the_wrong_connection_is_dropped_not_routed() {
        let broker = Broker::spawn(AllowAll);

        let (handler_id, mut handler_rx) = broker.connect("real.handler").await;
        let _ = handler_rx.recv().await;
        broker.dispatch(
            handler_id,
            Envelope {
                protocol_version: PROTOCOL_VERSION,
                trace_id: None,
                kind: Some(Kind::RegisterHandler(nova_bus::proto::RegisterHandler {
                    topic: "nova.protected.op".into(),
                })),
            },
        );
        let _ = handler_rx.recv().await;

        let (caller_id, mut caller_rx) = broker.connect("caller").await;
        let _ = caller_rx.recv().await;
        broker.dispatch(
            caller_id,
            Envelope {
                protocol_version: PROTOCOL_VERSION,
                trace_id: None,
                kind: Some(Kind::Call(Call {
                    correlation_id: 5,
                    topic: "nova.protected.op".into(),
                    payload: vec![],
                    timeout_ms: 5000,
                })),
            },
        );
        let forwarded = handler_rx.recv().await.unwrap();
        let broker_request_id = match forwarded.kind {
            Some(Kind::Call(c)) => c.correlation_id,
            other => panic!("expected forwarded Call, got {other:?}"),
        };

        // An unrelated, malicious third connection guesses/observes the
        // broker_request_id and tries to answer on the real handler's
        // behalf.
        let (attacker_id, _attacker_rx) = broker.connect("attacker").await;
        broker.dispatch(
            attacker_id,
            Envelope {
                protocol_version: PROTOCOL_VERSION,
                trace_id: None,
                kind: Some(Kind::Response(Response {
                    correlation_id: broker_request_id,
                    payload: b"forged".to_vec(),
                })),
            },
        );

        // The caller must NOT receive the forged response...
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(200), caller_rx.recv())
                .await
                .is_err(),
            "caller should not have received anything from the forged Response"
        );

        // ...and the real handler's genuine, later Response still works,
        // proving the forged attempt didn't consume/invalidate the pending
        // call entry.
        broker.dispatch(
            handler_id,
            Envelope {
                protocol_version: PROTOCOL_VERSION,
                trace_id: None,
                kind: Some(Kind::Response(Response {
                    correlation_id: broker_request_id,
                    payload: b"genuine".to_vec(),
                })),
            },
        );
        let reply = caller_rx.recv().await.unwrap();
        match reply.kind {
            Some(Kind::Response(r)) => assert_eq!(r.payload, b"genuine"),
            other => panic!("expected genuine Response, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn permission_denied_call_never_reaches_a_handler() {
        let acl = DenyList::new().deny("untrusted.app", "nova.protected");
        let broker = Broker::spawn(acl);

        let (handler_id, mut handler_rx) = broker.connect("trusted.handler").await;
        let _ = handler_rx.recv().await;
        broker.dispatch(
            handler_id,
            Envelope {
                protocol_version: PROTOCOL_VERSION,
                trace_id: None,
                kind: Some(Kind::RegisterHandler(nova_bus::proto::RegisterHandler {
                    topic: "nova.protected".into(),
                })),
            },
        );
        let _ = handler_rx.recv().await;

        let (caller_id, mut caller_rx) = broker.connect("untrusted.app").await;
        let _ = caller_rx.recv().await;
        broker.dispatch(
            caller_id,
            Envelope {
                protocol_version: PROTOCOL_VERSION,
                trace_id: None,
                kind: Some(Kind::Call(Call {
                    correlation_id: 1,
                    topic: "nova.protected".into(),
                    payload: vec![],
                    timeout_ms: 1000,
                })),
            },
        );

        let reply = caller_rx.recv().await.unwrap();
        assert!(matches!(
            reply.kind,
            Some(Kind::Error(e)) if e.code == ErrorCode::PermissionDenied as i32
        ));
        // Handler must never have seen this call.
        assert!(handler_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn publish_fans_out_to_subscribers_including_wildcard() {
        let broker = Broker::spawn(AllowAll);

        let (sub_id, mut sub_rx) = broker.connect("nova.monitor").await;
        let _ = sub_rx.recv().await;
        broker.dispatch(
            sub_id,
            Envelope {
                protocol_version: PROTOCOL_VERSION,
                trace_id: None,
                kind: Some(Kind::Subscribe(nova_bus::proto::Subscribe {
                    topic: "nova.package.*".into(),
                })),
            },
        );
        let _ = sub_rx.recv().await; // SubscribeAck

        let (pub_id, mut pub_rx) = broker.connect("novapkg-agent").await;
        let _ = pub_rx.recv().await;
        broker.dispatch(
            pub_id,
            Envelope {
                protocol_version: PROTOCOL_VERSION,
                trace_id: None,
                kind: Some(Kind::Publish(Publish {
                    topic: "nova.package.install_complete".into(),
                    payload: b"dev.novaos.files".to_vec(),
                })),
            },
        );

        let received = sub_rx.recv().await.unwrap();
        match received.kind {
            Some(Kind::Publish(p)) => {
                assert_eq!(p.topic, "nova.package.install_complete");
                assert_eq!(p.payload, b"dev.novaos.files");
            }
            other => panic!("expected Publish, got {other:?}"),
        }
    }

    #[test]
    fn topic_matches_exact_and_wildcard() {
        assert!(topic_matches("nova.theme.changed", "nova.theme.changed"));
        assert!(!topic_matches("nova.theme.changed", "nova.theme.other"));
        assert!(topic_matches("nova.package.*", "nova.package.install_complete"));
        assert!(!topic_matches("nova.package.*", "nova.session.launch"));
        assert!(!topic_matches("nova.package.*", "nova.package"));
    }
}
