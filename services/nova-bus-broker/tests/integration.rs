//! Integration test: real TCP sockets, real broker, two independent
//! `Client` instances — proves the wire protocol end-to-end, not just the
//! in-process `Broker::dispatch` unit tests in src/broker.rs.
//!
//! Exercises the Phase 2 vertical-slice Nova Bus exit criteria
//! (docs/12-ROADMAP-AND-MILESTONES.md §4): "services register, messages
//! send, replies work, timeouts work, error paths work."

use nova_bus::client::Client;
use nova_bus_broker::broker::{AllowAll, Broker};
use nova_bus_broker::server::{accept_loop, bind_tcp};

async fn spawn_test_broker() -> String {
    let broker = Broker::spawn(AllowAll);
    let listener = bind_tcp("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    tokio::spawn(accept_loop(listener, broker));
    addr
}

#[tokio::test]
async fn hello_service_registers_and_replies() {
    let addr = spawn_test_broker().await;

    let handler = Client::connect_tcp(&addr, "dev.novaos.hello-service")
        .await
        .unwrap();
    let mut incoming = handler.register_handler("nova.hello.greet").await.unwrap();

    tokio::spawn(async move {
        while let Some(call) = incoming.recv().await {
            let name = String::from_utf8(call.payload.clone()).unwrap();
            call.respond(format!("Hello, {name}!").into_bytes());
        }
    });

    let caller = Client::connect_tcp(&addr, "dev.novaos.hello")
        .await
        .unwrap();
    let reply = caller
        .call("nova.hello.greet", b"Nova".to_vec(), 2000)
        .await
        .unwrap();

    assert_eq!(String::from_utf8(reply).unwrap(), "Hello, Nova!");
}

#[tokio::test]
async fn call_to_unregistered_topic_returns_no_handler() {
    let addr = spawn_test_broker().await;
    let caller = Client::connect_tcp(&addr, "dev.novaos.hello").await.unwrap();

    let result = caller.call("nova.does.not.exist", vec![], 1000).await;
    assert!(matches!(result, Err(nova_bus::BusError::NoHandler { .. })));
}

#[tokio::test]
async fn call_times_out_when_handler_is_unresponsive() {
    let addr = spawn_test_broker().await;

    let handler = Client::connect_tcp(&addr, "slow.handler").await.unwrap();
    let mut incoming = handler.register_handler("nova.slow").await.unwrap();
    // Deliberately never respond — the incoming call is just dropped.
    tokio::spawn(async move {
        let _ = incoming.recv().await;
    });

    let caller = Client::connect_tcp(&addr, "impatient").await.unwrap();
    let result = caller.call("nova.slow", vec![], 100).await;
    assert!(matches!(result, Err(nova_bus::BusError::Timeout { .. })));
}

#[tokio::test]
async fn publish_subscribe_round_trip() {
    let addr = spawn_test_broker().await;

    let subscriber = Client::connect_tcp(&addr, "nova.monitor").await.unwrap();
    let mut events = subscriber.subscribe("nova.package.*").await.unwrap();
    // Give the Subscribe message time to reach the broker before publishing.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let publisher = Client::connect_tcp(&addr, "novapkg-agent").await.unwrap();
    publisher.publish(
        "nova.package.install_complete",
        b"dev.novaos.files".to_vec(),
    );

    let payload = tokio::time::timeout(std::time::Duration::from_secs(2), events.recv())
        .await
        .expect("timed out waiting for publish")
        .expect("channel closed");
    assert_eq!(payload, b"dev.novaos.files");
}

#[tokio::test]
async fn full_vertical_slice_bus_flow() {
    // Mirrors docs/specs/01-INTERACTION-FLOWS.md §1's app-launch flow at the
    // Nova Bus layer: nova-shell calls nova-sessiond, which acks; separately
    // a status event is published and observed by a subscriber (standing in
    // for nova-shell's window_list_changed subscription).
    let addr = spawn_test_broker().await;

    let sessiond = Client::connect_tcp(&addr, "nova.sessiond").await.unwrap();
    let mut launch_requests = sessiond.register_handler("nova.session.launch").await.unwrap();
    let sessiond_events = sessiond.clone();
    tokio::spawn(async move {
        while let Some(call) = launch_requests.recv().await {
            let app_id = String::from_utf8(call.payload.clone()).unwrap();
            sessiond_events.publish("nova.session.app_started", app_id.clone().into_bytes());
            call.respond(b"launched".to_vec());
        }
    });

    let shell = Client::connect_tcp(&addr, "nova.shell").await.unwrap();
    let mut app_started = shell.subscribe("nova.session.app_started").await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let ack = shell
        .call("nova.session.launch", b"dev.novaos.hello".to_vec(), 2000)
        .await
        .unwrap();
    assert_eq!(ack, b"launched");

    let event = tokio::time::timeout(std::time::Duration::from_secs(2), app_started.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(event, b"dev.novaos.hello");
}
