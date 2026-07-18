# ADR-0004: Primary Systems Language

Status: Accepted
Date: 2026-07-18
Deciders: Chief Architect

## Context

NovaOS's own components (compositor, session manager, services, package manager, SDK,
native apps) need one primary language to keep the codebase coherent at 100,000+ LOC,
memory-safe (no GC pause budget to spare, and no tolerance for use-after-free-class bugs
in a compositor that touches every app), and fast to compile/iterate on.

## Options Considered

1. **C** — what wlroots and most of the Linux userland stack are written in; maximum
   compatibility, minimum memory safety, highest long-term maintenance risk at our target
   scale (100k+ LOC, many contributors).
2. **C++** — more expressive than C, still no memory safety by default, larger surface for
   subtle bugs, heavier build tooling story.
3. **Go** — easy to learn, good tooling, but a GC with pause behavior that's a poor fit for
   a compositor's frame-timing requirements, and a runtime/binary size that works against
   the RAM budget for many small resident processes.
4. **Rust** — memory- and thread-safe without a GC, C-compatible FFI (needed to bind
   wlroots per [ADR-0003](ADR-0003-compositor-and-display-protocol.md)), predictable
   low-level performance, small static binaries, a mature and growing systems/GUI
   ecosystem, and first-class WASM compilation (relevant to tooling, not to the OS-in-
   browser strategy itself — see [ADR-0009](ADR-0009-browser-boot-emulator.md)).

## Decision

**Rust** as the primary language for all NovaOS-authored components: compositor, session
manager, system services, Nova Bus (IPC), package manager, SDK, and native applications.
C is used only at FFI boundaries to existing C libraries we depend on (wlroots, and
narrowly-scoped hardware/codec libraries where no mature Rust equivalent exists) — never
for new NovaOS logic. A small amount of shell/POSIX sh is acceptable for build glue and
early-boot scripts where a process doesn't exist yet to run Rust.

## Rationale

Against the priority order in [00-VISION.md](../00-VISION.md) §6 — simplicity,
maintainability, consistency, performance — Rust is the only option that scores well on
all four simultaneously: memory safety removes an entire class of bugs that dominate
long-lived C/C++ desktop codebases; no GC preserves the low, predictable RAM/latency
profile the compositor needs; one language across the whole stack (compositor → services
→ SDK → apps) means one toolchain, one dependency manager (Cargo), one set of lint/test
conventions, directly serving "consistency" and lowering onboarding cost for
contributors.

## Consequences

- Compile times and toolchain size are larger than C during development — mitigated by
  incremental builds and CI caching (see [10-TESTING-AND-BUILD.md](../10-TESTING-AND-BUILD.md)).
- Every contributor needs Rust fluency; this is an explicit, accepted tradeoff for
  long-term codebase health over short-term contributor ramp-up.
- Extension/plugin authors are not required to write Rust — see
  [ADR](ADR-0006-ipc-mechanism.md) and the SDK plugin story in
  [04-APPLICATION-FRAMEWORK-AND-SDK.md](../04-APPLICATION-FRAMEWORK-AND-SDK.md), which
  offers a scripting surface for extensions specifically so "must know Rust" doesn't
  become a barrier to the extension ecosystem.

## Revisit Triggers

- If Rust GUI/compositor ecosystem maturity regresses relative to alternatives (unlikely
  given current trajectory, but tracked).
- If build-time pain becomes a measured, repeated contributor complaint after v1.
