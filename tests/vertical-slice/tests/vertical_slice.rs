//! The Phase 2 vertical slice, proven as real, separate OS processes.
//! docs/12-ROADMAP-AND-MILESTONES.md §4:
//!
//!   Boot(*) -> Nova Bus -> [Nova WM(*) -> Nova UI Toolkit(*) -> Desktop(*)
//!   -> Launcher(*)] -> App starts -> Window appears(*) -> IPC works
//!   -> Window closes
//!
//! (*) not provable here — no compositor/session-manager/boot exists in
//! this environment (docs/12-ROADMAP-AND-MILESTONES.md §4's Environment
//! note). What THIS test proves for real: `novabusd` and the `hello` app
//! run as two independent OS processes (`std::process::Command`, not
//! in-process function calls), perform the real Nova Bus connect handshake,
//! exchange a real `Call`/`Response` and real `Publish`/`Subscribe` events
//! over a real TCP socket, and the app process exits cleanly on command —
//! i.e. everything in the flow above except the parts requiring a Linux
//! graphics stack.

use nova_bus::client::Client;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::time::timeout;

const BUS_ADDR: &str = "127.0.0.1:47780";
const APP_ID: &str = "test.vertical-slice.hello";

fn workspace_binary(name: &str) -> PathBuf {
    // Standard trick: a test binary lives at target/<profile>/deps/<test>-<hash>;
    // every workspace binary lands one directory up, in target/<profile>/.
    let mut path = std::env::current_exe().expect("current_exe");
    path.pop(); // drop the test binary's own file name
    if path.ends_with("deps") {
        path.pop();
    }
    let file_name = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    path.push(file_name);
    path
}

struct ChildGuard(Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

async fn wait_for_port(addr: &str, deadline: Duration) {
    let start = std::time::Instant::now();
    loop {
        if tokio::net::TcpStream::connect(addr).await.is_ok() {
            return;
        }
        if start.elapsed() > deadline {
            panic!("novabusd never became reachable at {addr}");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test]
async fn vertical_slice_boot_bus_app_ipc_shutdown() {
    let novabusd_path = workspace_binary("novabusd");
    let hello_path = workspace_binary("hello");
    assert!(
        novabusd_path.exists(),
        "novabusd binary not found at {novabusd_path:?} — run `cargo build --workspace` first"
    );
    assert!(
        hello_path.exists(),
        "hello binary not found at {hello_path:?} — run `cargo build --workspace` first"
    );

    // --- Boot (as much of it as this environment can prove): start novabusd
    // as a real child process, standing in for OpenRC starting the first
    // Nova service (docs/specs/16-STATE-MACHINES.md §5).
    let novabusd = ChildGuard(
        Command::new(&novabusd_path)
            .env("NOVA_BUS_ADDR", BUS_ADDR)
            .env("RUST_LOG", "info")
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("failed to spawn novabusd"),
    );
    wait_for_port(BUS_ADDR, Duration::from_secs(5)).await;

    // A stand-in for nova-sessiond: observes app lifecycle events over the
    // real bus, the same way the real service would per
    // docs/rfcs/RFC-0008-session-manager.md Events Consumed.
    let harness = Client::connect_tcp(BUS_ADDR, "test.sessiond-harness")
        .await
        .expect("harness failed to connect to novabusd");
    let mut app_started = harness.subscribe("nova.session.app_started").await.unwrap();
    let mut app_exited = harness.subscribe("nova.session.app_exited").await.unwrap();
    // Let both Subscribes land at the broker before the app can publish.
    tokio::time::sleep(Duration::from_millis(100)).await;

    // --- App starts, as a real second OS process.
    let mut hello = ChildGuard(
        Command::new(&hello_path)
            .env("NOVA_BUS_ADDR", BUS_ADDR)
            .env("NOVA_APP_ID", APP_ID)
            .env("RUST_LOG", "info")
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("failed to spawn hello"),
    );

    // --- IPC works: the app's own nova.session.app_started publish,
    // received by our harness over a real socket the app process never
    // touched directly (it went through novabusd).
    let started_payload = timeout(Duration::from_secs(5), app_started.recv())
        .await
        .expect("timed out waiting for nova.session.app_started")
        .expect("app_started channel closed");
    assert_eq!(String::from_utf8(started_payload).unwrap(), APP_ID);

    // --- Window appears (logical proof only — see module doc): the app
    // reaching this point has already run App::on_launch, which calls
    // Window::map() (sdk/nova-app/src/window.rs) and builds the Nova UI
    // widget tree (sdk/nova-ui) before publishing app_started above, so a
    // panic in either would have prevented us ever observing this event.

    // --- Window closes / app shuts down, driven by a real Call this
    // harness makes exactly the way nova-sessiond's
    // nova.session.terminate handler would
    // (docs/rfcs/RFC-0008-session-manager.md Public APIs).
    let shutdown_topic = format!("nova.app.{APP_ID}.shutdown");
    let response = timeout(
        Duration::from_secs(5),
        harness.call(&shutdown_topic, vec![], 3000),
    )
    .await
    .expect("timed out calling shutdown")
    .expect("shutdown call failed");
    assert_eq!(response, b"ok");

    let exited_payload = timeout(Duration::from_secs(5), app_exited.recv())
        .await
        .expect("timed out waiting for nova.session.app_exited")
        .expect("app_exited channel closed");
    assert_eq!(String::from_utf8(exited_payload).unwrap(), APP_ID);

    // The process itself must actually terminate, not just publish the event.
    let status = tokio::task::spawn_blocking(move || hello.0.wait())
        .await
        .unwrap()
        .expect("failed to wait on hello process");
    assert!(status.success(), "hello app exited with {status:?}");

    // novabusd has no remote shutdown call (§Startup Order: system-service
    // supervised, not app-lifecycle supervised) — ChildGuard kills it on
    // drop, matching how a real system would stop it via its supervisor,
    // not a Nova Bus message.
    drop(novabusd);
}
