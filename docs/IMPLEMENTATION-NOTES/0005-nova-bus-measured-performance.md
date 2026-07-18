# 0005: Nova Bus measured latency/throughput (first real numbers)

Date: 2026-07-18
Status: Open — informs Phase 2.5 System Validation, not yet judged against a formal
budget (Nova Bus has no dedicated latency/throughput budget in
[09-PERFORMANCE-STRATEGY.md](../09-PERFORMANCE-STRATEGY.md); one should probably be
added, itself a Phase 2.5 candidate finding)

## Architecture

[09-PERFORMANCE-STRATEGY.md](../09-PERFORMANCE-STRATEGY.md) sets an app-launch budget
(≤500ms) and a compositor frame budget (≤16.6ms) but no explicit Nova Bus per-call
latency or throughput budget — IPC cost was implicitly assumed to be small relative to
those two numbers, not measured.

## Reality

`tools/nova-bus-bench` (real broker, real `Client`s, real TCP/Protobuf wire protocol —
see that tool's module doc for exactly what is and isn't measured) produced, on this
development machine (release build, Windows host, single process):

```
Sequential round-trip latency, n=2000:
  avg:  148.0µs   p50: 139.2µs   p95: 207.1µs   p99: 265.9µs   max: 1.07ms

Throughput, 5000 concurrent calls:
  total wall time: 136.5ms   throughput: 36,618 calls/sec
  timeout rate: 0/5000 (0.000%)   other errors: 0/5000

NO_HANDLER correctness: 126.8µs wait before error (near-instant, as designed —
  does not wait for a timeout)

TIMEOUT correctness: requested 200ms, actual wait 207.7ms (matches, plus small
  scheduling overhead)
```

Real app-launch timing (from a running `hello` process,
`sdk/nova-app/src/lib.rs`'s `LaunchTiming`):

```
bus_connect: 2.64ms   app_new: ~0µs   on_launch: 303µs   total: 2.96ms
```

## Reason

N/A — this is a measurement note, not a divergence-from-spec note. Filed under
IMPLEMENTATION-NOTES anyway (rather than only in `tools/nova-bus-bench`'s output)
because Phase 2.5 explicitly asked for measured, not estimated, numbers
([12-ROADMAP-AND-MILESTONES.md](../12-ROADMAP-AND-MILESTONES.md) §4a), and this is
where that phase should look for them.

## Decision

None required yet — these numbers are reported, not acted on. At a glance: 2.96ms total
app-launch time is comfortably within the 500ms budget in
[09-PERFORMANCE-STRATEGY.md](../09-PERFORMANCE-STRATEGY.md) §2 even before accounting
for the fact that this vertical slice skips sandbox construction (the dominant real
cost per that doc's §6), and Nova Bus's own p99 latency (266µs) is small enough relative
to both budgets that it's unlikely to be the bottleneck once sandbox construction is
added back in Phase 3. These are single-machine, single-process numbers on a Windows
dev host, not the reference VM/hardware [03-BOOT-TIMELINE.md](../specs/03-BOOT-TIMELINE.md)
specifies — directionally useful, not release-gate numbers.

## Future Direction

1. Add an explicit Nova Bus latency/throughput budget to
   [09-PERFORMANCE-STRATEGY.md](../09-PERFORMANCE-STRATEGY.md), informed by these
   numbers, as a Phase 2.5 action item.
2. Re-run `tools/nova-bus-bench` on the reference VM/hardware once one exists
   ([03-BOOT-TIMELINE.md](../specs/03-BOOT-TIMELINE.md)) and once the real Unix-socket
   transport ([0001](0001-nova-bus-dev-transport-tcp.md)) lands, since loopback TCP and
   Unix sockets have different overhead characteristics.
3. Wire `tools/nova-bus-bench` into the CI performance-regression gate
   ([10-TESTING-AND-BUILD.md](../10-TESTING-AND-BUILD.md) §2 stage 5) once a budget
   from step 1 exists to regress against.
