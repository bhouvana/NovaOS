# NovaOS — Build System, CI & Testing Strategy

Status: Draft v0.1 · Owner: Chief Architect · Last updated: 2026-07-18

## 1. Build System

- **Rust workspace**: a single Cargo workspace at repo root
  ([02-REPOSITORY-STRUCTURE.md](02-REPOSITORY-STRUCTURE.md)) covering `services/`,
  `desktop/`, `sdk/`, `apps/` — one dependency lockfile, one compiler version pin, one
  `cargo test`/`cargo clippy` invocation surface for the entire application layer.
- **Image builder** (`tools/`, `system/image/`): a separate, non-Cargo pipeline that
  assembles the base system (Alpine packages per
  [ADR-0001](decisions/ADR-0001-linux-base-distribution.md)) plus the compiled Rust
  binaries into the SquashFS root and A/B-partitioned ISO/image artifacts
  ([ADR-0008](decisions/ADR-0008-filesystem-and-update-strategy.md)), parameterized by
  build profile (full ISO vs. browser-demo image, per
  [07-BROWSER-DEPLOYMENT.md](07-BROWSER-DEPLOYMENT.md) §3).
- **Reproducibility**: image builds pin exact Alpine package versions and the exact Rust
  toolchain version; a given commit + build profile always produces a bit-identical (or
  documented-why-not) artifact — required for the update system's trust model
  ([05-PACKAGE-AND-UPDATE-SYSTEM.md](05-PACKAGE-AND-UPDATE-SYSTEM.md) §4) to be
  auditable.

## 2. CI Pipeline Stages

1. **Fast checks** (every push): `cargo fmt --check`, `cargo clippy -D warnings`,
   dependency-direction lint enforcing
   [02-REPOSITORY-STRUCTURE.md](02-REPOSITORY-STRUCTURE.md) §3.
2. **Unit tests**: co-located with each crate, run per-crate on every push.
3. **Contract tests**: SDK modules (`sdk/*`) carry contract tests validating their public
   API/wire-format stability against the previous release — catches accidental breaking
   changes before they hit semver review
   ([04-APPLICATION-FRAMEWORK-AND-SDK.md](04-APPLICATION-FRAMEWORK-AND-SDK.md) §7).
4. **Integration tests** (`tests/`): boot the produced image in a headless QEMU VM,
   assert boot milestones fire in order and within the budget
   ([09-PERFORMANCE-STRATEGY.md](09-PERFORMANCE-STRATEGY.md) §3), assert the desktop
   shell reaches "ready," assert a representative app launches successfully.
5. **Performance regression gate**: re-runs the §2 budget table from
   [09-PERFORMANCE-STRATEGY.md](09-PERFORMANCE-STRATEGY.md) against the freshly built
   image on reference VM hardware; fails the build if idle RAM, boot time, or frame time
   regress beyond a defined tolerance without an accompanying ADR.
6. **Accessibility/contrast lint**: automated WCAG AA contrast check against the active
   theme tokens ([06-DESIGN-SYSTEM.md](06-DESIGN-SYSTEM.md) §6).
7. **Security lint**: manifest-to-sandbox-rule mapping test suite
   ([ADR-0010](decisions/ADR-0010-app-sandboxing-model.md)) validated against every
   in-tree app manifest — no app ships with an unreviewed permission combination.
8. **Browser demo smoke test**: headless-browser (Playwright-class) run against the v86
   demo build, asserting the desktop becomes interactive within the
   [09-PERFORMANCE-STRATEGY.md](09-PERFORMANCE-STRATEGY.md) §2 browser-boot budget.

A PR cannot merge unless stages 1–3 and 6–7 pass; stages 4–5 and 8 gate the release
branch rather than every PR, since full-image boot tests are too slow to run on every
push at v1 team scale.

Release branch cut/gating rules are defined in
[11-CODING-STANDARDS.md](11-CODING-STANDARDS.md) §Release Strategy.

## 3. Testing Pyramid

- **Unit tests**: the bulk of coverage — pure logic in services, SDK modules, apps.
  Target ≥80% line coverage on `services/` and `sdk/` (the shared, highest-blast-radius
  code); `apps/` held to a lower bar since app-specific bugs are lower blast radius.
- **Integration tests**: cross-crate flows that can't be unit-tested meaningfully — app
  launch through `nova-sessiond`, Nova Bus permission enforcement end-to-end, package
  install → launch → update → remove flow.
- **End-to-end/system tests**: full-image boot tests (§2 stage 4) and the browser demo
  smoke test (§2 stage 8) — few in number, but each one is release-blocking because it's
  the closest proxy to "does NovaOS actually work."
- **Tests as documentation**: SDK contract tests double as executable usage examples,
  referenced from generated SDK docs
  ([04-APPLICATION-FRAMEWORK-AND-SDK.md](04-APPLICATION-FRAMEWORK-AND-SDK.md) §8).

## 4. Manual/Exploratory Testing

Automated coverage does not replace hands-on testing before a release: each milestone
(see [12-ROADMAP-AND-MILESTONES.md](12-ROADMAP-AND-MILESTONES.md)) includes an explicit
manual test pass on real reference hardware (not just VM/CI), since driver-level and
input-latency issues are exactly the class of bug CI-in-a-VM is weakest at catching.

## 5. Dev Environment

- Primary development target is a NovaOS VM (QEMU/KVM) — the same artifact CI builds and
  tests, so "works in my dev VM" and "works in CI" are the same claim, not two different
  environments that can silently diverge.
- `tools/` provides a one-command dev VM launcher (boot the latest local build with
  a shared folder for iterative app development) as a Phase 2 deliverable
  ([12-ROADMAP-AND-MILESTONES.md](12-ROADMAP-AND-MILESTONES.md)).
