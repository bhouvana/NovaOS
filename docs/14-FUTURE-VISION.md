# NovaOS — Future Vision & Extensibility

Status: Draft v0.1 · Owner: Chief Architect · Last updated: 2026-07-18

This doc records deliberately-deferred ideas — things the v1 architecture leaves room
for without building prematurely, per the YAGNI principle. Nothing here is committed;
each item becomes a real ADR/roadmap phase only when there's real signal it's needed.

## 1. Post-v1 Candidates

- **Full multi-user support** — separate home/data partitions, fast user switching, per-
  user secrets stores. The sandboxing/session model ([08-SECURITY-MODEL.md](08-SECURITY-MODEL.md)
  §6) already isolates per-process; multi-user is primarily a login/UX/storage-layout
  project, not a re-architecture.
- **Third-party theme marketplace** — the theme token schema
  ([06-DESIGN-SYSTEM.md](06-DESIGN-SYSTEM.md) §6) is designed to support signed
  third-party themes from day one; only the marketplace/distribution tooling is deferred.
- **Delta/binary-diff updates** — both OS images and `.novapkg` updates currently ship
  full artifacts ([ADR-0008](decisions/ADR-0008-filesystem-and-update-strategy.md)
  Consequences); delta updates are a pure optimization on top of the existing signed-
  artifact model, not a format change.
- **aarch64 hardware support** — the architecture has no x86-specific assumptions above
  the boot/image-build layer; this is primarily a `system/image/` and driver-coverage
  effort, not a redesign.
- **Compatibility "App Runner" for non-native Linux apps** — an explicitly second-class,
  opt-in sandboxed runner for select existing Linux apps, noted as out-of-v1-scope in
  [ADR-0007](decisions/ADR-0007-package-format.md) Consequences, revisited only if user
  demand for "just run my existing app" proves strong enough to justify the sandboxing/
  design-system exceptions it would require.
- **Mobile/tablet form factor** — Nova UI's token-based layout system
  ([06-DESIGN-SYSTEM.md](06-DESIGN-SYSTEM.md)) does not assume desktop-only input, but
  touch-first interaction patterns, a mobile shell, and power/thermal tuning are
  undesigned; explicitly out of scope until the desktop experience is proven.
- **Cloud sync (optional, opt-in)** — settings/file sync across devices, kept strictly
  optional and off by default to preserve [00-VISION.md](00-VISION.md) §7's
  not-cloud-account-gated non-goal; would be additive to `nova-storage`, not a
  replacement for local-first storage.

## 2. Extensibility Surfaces Already Designed In

These aren't speculative — the v1 architecture is deliberately shaped so they attach
cleanly later without a rewrite:

- **Nova Bus** ([ADR-0006](decisions/ADR-0006-ipc-mechanism.md)) is schema-versioned
  (Protobuf), so new services can join the bus without breaking existing clients.
- **App manifests** ([04-APPLICATION-FRAMEWORK-AND-SDK.md](04-APPLICATION-FRAMEWORK-AND-SDK.md)
  §4) declare a `sdk_version` range, so the SDK can evolve major versions with old apps
  continuing to run under a compatibility window, rather than every OS update breaking
  every installed app.
- **Plugin/extension system** ([04-APPLICATION-FRAMEWORK-AND-SDK.md](04-APPLICATION-FRAMEWORK-AND-SDK.md)
  §6) already separates "sandboxed process" extensions from "data-only" extensions
  (themes, search providers) — a scripting host can be added as a third category without
  touching the trust model of the other two.
- **Nova Store catalog format** ([05-PACKAGE-AND-UPDATE-SYSTEM.md](05-PACKAGE-AND-UPDATE-SYSTEM.md)
  §4) is signed and self-describing enough that self-hosted alternate catalogs are an
  ADR-and-tooling exercise, not an architecture change.

## 3. Explicitly Not Planned

- Server/headless NovaOS variant — the entire desktop-shell/compositor layer is core to
  the product identity ([00-VISION.md](00-VISION.md) §1); a headless variant would be a
  different product sharing only the base-system layer, out of scope indefinitely unless
  a future decision explicitly re-opens it.
- Kernel/driver-stack forking — reaffirmed from [00-VISION.md](00-VISION.md) §1 as a
  permanent non-goal, not a "not yet."

## 4. How to Propose Something New

Add a candidate to §1 with a one-paragraph rationale, or open an ADR directly if it's
already well-understood enough to decide. Nothing in this document authorizes work by
itself — it exists so ideas aren't lost and aren't accidentally designed against by a
v1 decision that didn't know they were coming.
