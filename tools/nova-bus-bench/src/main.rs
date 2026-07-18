//! Real Nova Bus latency/throughput measurement — per Phase 2.5's
//! "measure, don't estimate" directive
//! (docs/12-ROADMAP-AND-MILESTONES.md §4a). Numbers this tool prints are
//! recorded, with the exact invocation and machine, in
//! docs/IMPLEMENTATION-NOTES/0005-nova-bus-measured-performance.md — this
//! source file is the reproduction recipe for those numbers, not a one-off
//! script whose output has to be taken on faith.
//!
//! What this measures: real Protobuf-over-TCP round trips through a real
//! `nova-bus-broker::Broker` and real `nova_bus::client::Client`s — the
//! actual wire protocol and actor-model routing code, not a mock. What it
//! does NOT measure: cross-process/cross-host latency (broker and clients
//! share this process's address space, connected via real loopback
//! sockets) or anything compositor/GPU-related, since neither exists yet
//! (docs/12-ROADMAP-AND-MILESTONES.md §4 Environment note).

use nova_bus::client::Client;
use nova_bus_broker::broker::{AllowAll, Broker};
use nova_bus_broker::server::{accept_loop, bind_tcp};
use std::time::{Duration, Instant};

const ADDR: &str = "127.0.0.1:47790";
const LATENCY_SAMPLES: usize = 2000;
const THROUGHPUT_CONCURRENT_CALLS: usize = 5000;

#[tokio::main]
async fn main() {
    let broker = Broker::spawn(AllowAll);
    let listener = bind_tcp(ADDR).await.expect("bind");
    tokio::spawn(accept_loop(listener, broker));

    let handler = Client::connect_tcp(ADDR, "bench.echo-handler")
        .await
        .expect("handler connect");
    let mut incoming = handler.register_handler("bench.echo").await.unwrap();
    tokio::spawn(async move {
        while let Some(call) = incoming.recv().await {
            let payload = call.payload.clone();
            call.respond(payload);
        }
    });
    // Let RegisterHandlerAck land before anyone calls the topic.
    tokio::time::sleep(Duration::from_millis(50)).await;

    println!("=== Nova Bus latency (sequential round trips, n={LATENCY_SAMPLES}) ===");
    let caller = Client::connect_tcp(ADDR, "bench.caller")
        .await
        .expect("caller connect");
    let mut latencies = Vec::with_capacity(LATENCY_SAMPLES);
    for i in 0..LATENCY_SAMPLES {
        let payload = format!("ping-{i}").into_bytes();
        let start = Instant::now();
        caller
            .call("bench.echo", payload, 2000)
            .await
            .expect("call");
        latencies.push(start.elapsed());
    }
    report_latency(&mut latencies);

    println!("\n=== Nova Bus throughput ({THROUGHPUT_CONCURRENT_CALLS} concurrent calls) ===");
    let start = Instant::now();
    let mut handles = Vec::with_capacity(THROUGHPUT_CONCURRENT_CALLS);
    for i in 0..THROUGHPUT_CONCURRENT_CALLS {
        let caller = caller.clone();
        handles.push(tokio::spawn(async move {
            caller
                .call("bench.echo", format!("burst-{i}").into_bytes(), 5000)
                .await
        }));
    }
    let mut timeouts = 0usize;
    let mut errors = 0usize;
    for h in handles {
        match h.await.unwrap() {
            Ok(_) => {}
            Err(nova_bus::BusError::Timeout { .. }) => timeouts += 1,
            Err(_) => errors += 1,
        }
    }
    let elapsed = start.elapsed();
    let throughput = THROUGHPUT_CONCURRENT_CALLS as f64 / elapsed.as_secs_f64();
    println!("  total wall time:     {elapsed:?}");
    println!("  throughput:          {throughput:.0} calls/sec");
    println!(
        "  timeout rate:        {timeouts}/{THROUGHPUT_CONCURRENT_CALLS} ({:.3}%)",
        100.0 * timeouts as f64 / THROUGHPUT_CONCURRENT_CALLS as f64
    );
    println!("  other errors:        {errors}/{THROUGHPUT_CONCURRENT_CALLS}");

    println!("\n=== NO_HANDLER correctness (topic with no registered handler) ===");
    let no_handler_start = Instant::now();
    let result = caller.call("bench.nonexistent.topic", vec![], 200).await;
    println!("  wait before error:   {:?} (should be near-instant, not the 200ms timeout)", no_handler_start.elapsed());
    println!("  result:              {result:?}");

    println!("\n=== TIMEOUT correctness (registered handler that never responds) ===");
    let stalling = Client::connect_tcp(ADDR, "bench.stalling-handler")
        .await
        .expect("stalling handler connect");
    let mut stalled_calls = stalling.register_handler("bench.stall").await.unwrap();
    tokio::spawn(async move {
        // Deliberately drain and drop every incoming call without responding.
        while stalled_calls.recv().await.is_some() {}
    });
    tokio::time::sleep(Duration::from_millis(50)).await;

    let requested_timeout_ms = 200u32;
    let timeout_start = Instant::now();
    let result = caller
        .call("bench.stall", vec![], requested_timeout_ms)
        .await;
    println!("  requested timeout:   {requested_timeout_ms}ms");
    println!("  actual wait:         {:?}", timeout_start.elapsed());
    println!("  result:              {result:?}");
}

fn report_latency(latencies: &mut [Duration]) {
    latencies.sort();
    let n = latencies.len();
    let avg: Duration = latencies.iter().sum::<Duration>() / n as u32;
    let p50 = latencies[n * 50 / 100];
    let p95 = latencies[n * 95 / 100];
    let p99 = latencies[(n * 99 / 100).min(n - 1)];
    let max = latencies[n - 1];
    println!("  avg:  {avg:?}");
    println!("  p50:  {p50:?}");
    println!("  p95:  {p95:?}");
    println!("  p99:  {p99:?}");
    println!("  max:  {max:?}");
}
