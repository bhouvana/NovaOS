# NovaOS — Coding Standards, Versioning & Release Strategy

Status: Draft v0.1 · Owner: Chief Architect · Last updated: 2026-07-18

## 1. Naming Conventions

- **Crates**: `nova-<subsystem>` (e.g. `nova-ui`, `nova-sessiond`) — the `nova-` prefix
  is mandatory for anything published as part of the SDK or resident services, so a
  `Cargo.toml` dependency list makes the NovaOS-vs-third-party-crate boundary visible at
  a glance.
- **Rust code**: standard Rust conventions (`rustfmt` defaults, `clippy::all` +
  `clippy::pedantic` reviewed case-by-case, not blindly enabled) — we do not invent a
  house style that fights the ecosystem's tooling.
- **App IDs**: reverse-DNS style, `dev.novaos.<app>` for first-party, publisher-owned
  domains for third-party (`com.example.myapp`) — matches
  [04-APPLICATION-FRAMEWORK-AND-SDK.md](04-APPLICATION-FRAMEWORK-AND-SDK.md) §4's
  manifest format and gives collision-free namespacing without a central registry
  authority beyond Nova Store's listing check.
- **Nova Bus topics**: `nova.<domain>.<event>` (e.g. `nova.notify.publish`,
  `nova.settings.changed`) — dotted, lowercase, matching the `.proto` package layout.

## 2. Folder Conventions

Governed by [02-REPOSITORY-STRUCTURE.md](02-REPOSITORY-STRUCTURE.md) — every new crate
must be placed per that doc's dependency-direction rules; a PR adding a crate in the
wrong location is a structural defect, not a style nitpick, and CI's dependency lint
(([10-TESTING-AND-BUILD.md](10-TESTING-AND-BUILD.md)) §2) rejects it.

## 3. Dependency Rules (crate-external)

- Prefer the Rust standard library and already-adopted workspace dependencies over adding
  a new external crate (directly reinforces [00-VISION.md](00-VISION.md) §6's
  "minimalism" and matches the Dependency Management principle: every dependency must be
  justified and documented).
- New external dependencies are declared in the workspace root `Cargo.toml` (not
  per-crate ad hoc versions) so version drift across crates is impossible by
  construction, and are noted in the PR description with a one-line justification.
- No dependency with a copyleft license incompatible with NovaOS's chosen open-source
  license may be added without an explicit ADR.

## 4. Error Handling

- Fail fast, fail explicitly: no silently swallowed `Result`s (`clippy::unwrap_used` and
  `clippy::expect_used` denied outside test code and truly-unreachable invariants, which
  must carry a comment explaining why they're unreachable).
- Errors carry context (which app, which permission, which file) sufficient to act on
  without re-running with extra logging — reinforces the Observability principle from the
  user's global engineering principles.
- User-facing errors (a failed app launch, a failed install) always surface a
  notification or dialog with a human-readable cause — never a silent failure at the UI
  layer, even if the underlying error is logged.

## 5. Logging

- Structured, leveled (`error`/`warn`/`info`/`debug`/`trace`), one log line per event —
  no multi-line stack-trace dumps to the primary log stream (full backtraces go to a
  secondary crash artifact, referenced by ID from the log line).
- Never log secrets ([08-SECURITY-MODEL.md](08-SECURITY-MODEL.md) §5), full file
  contents, or unredacted user content.

## 6. Testing Strategy

Covered in full in [10-TESTING-AND-BUILD.md](10-TESTING-AND-BUILD.md) §3; the standard
applied at code-review time is: new logic in `services/` or `sdk/` without an
accompanying unit test is not mergeable, no exceptions without an explicit, reviewed
justification in the PR.

## 7. Documentation Standards

- Every public SDK item (`sdk/*`) carries a doc-comment — this is enforced by CI
  (`#![warn(missing_docs)]` at minimum on `sdk/*` crates) because these doc-comments are
  the literal source of the published SDK reference
  ([04-APPLICATION-FRAMEWORK-AND-SDK.md](04-APPLICATION-FRAMEWORK-AND-SDK.md) §8).
- Every new subsystem gets a short `README.md` in its top-level folder pointing back to
  the relevant `docs/` architecture doc — code and architecture docs must never diverge
  silently; a PR that changes architecture-relevant behavior updates the doc in the same
  PR.

## 8. Architecture Decision Records & RFCs

ADR process defined in [decisions/README.md](decisions/README.md). ADRs are required
for: new resident processes, new external dependencies with license implications,
changes to the on-disk package/image format, changes to the IPC wire protocol, and
anything listed as a "Revisit Trigger" in an existing ADR.

**RFCs** are the companion process, defined in [rfcs/README.md](rfcs/README.md), and are
a **standing condition of entering implementation** (adopted at the close of Phase 1.5,
[12-ROADMAP-AND-MILESTONES.md](12-ROADMAP-AND-MILESTONES.md) §3) — required for: a new
resident service or subsystem, a change to the Nova Bus wire protocol
([specs/15-NOVA-BUS-PROTOCOL-SPEC.md](specs/15-NOVA-BUS-PROTOCOL-SPEC.md)), a breaking
change to an SDK API's public contract
([specs/17-SDK-API-REFERENCE-POLICY.md](specs/17-SDK-API-REFERENCE-POLICY.md)), or a
change to the `.novapkg` format, filesystem layout, or configuration schema versioning
strategy. Where both are triggered by the same change (most new-service proposals), both
are required: the ADR for the point decision, the RFC for the resulting service's full
contract — see [rfcs/README.md](rfcs/README.md) §"RFC vs. ADR" for the full distinction.

## 9. Code Review Checklist

- [ ] Matches the dependency-direction rules ([02-REPOSITORY-STRUCTURE.md](02-REPOSITORY-STRUCTURE.md) §3)
- [ ] No new resident process without an ADR ([01-SYSTEM-ARCHITECTURE.md](01-SYSTEM-ARCHITECTURE.md) §3)
- [ ] Errors handled per §4, no silent failure paths
- [ ] Unit tests present for new logic in `services/`/`sdk/`
- [ ] Public SDK items documented (§7)
- [ ] No secrets/PII in logs (§5, [08-SECURITY-MODEL.md](08-SECURITY-MODEL.md) §5)
- [ ] Performance-sensitive paths (compositor, boot, app launch) checked against budgets
      in [09-PERFORMANCE-STRATEGY.md](09-PERFORMANCE-STRATEGY.md)
- [ ] New/changed UI uses `nova-ui` tokens/components, not one-off styling
      ([06-DESIGN-SYSTEM.md](06-DESIGN-SYSTEM.md))

## 10. Versioning Strategy

- **SDK crates** (`sdk/*`): semver, independently versioned per crate but released in
  lockstep as a single "SDK version" (`sdk_version` in app manifests,
  [04-APPLICATION-FRAMEWORK-AND-SDK.md](04-APPLICATION-FRAMEWORK-AND-SDK.md) §7) —
  simpler for app authors to reason about than N independently-versioned SDK crates.
- **OS releases**: `MAJOR.MINOR.PATCH` — MAJOR for breaking changes to the on-disk
  format or SDK, MINOR for new features/apps within a stable SDK major version, PATCH for
  fixes shipped via the standard A/B update flow
  ([05-PACKAGE-AND-UPDATE-SYSTEM.md](05-PACKAGE-AND-UPDATE-SYSTEM.md) §5).
- **Apps** (`.novapkg`): independently versioned by their own publisher, semver
  recommended but not enforced beyond "must be monotonically increasing," per
  [ADR-0007](decisions/ADR-0007-package-format.md).

## 11. Branching Strategy

- `main`: always releasable at PATCH granularity for the current stable MINOR; every
  merge passes the fast-check + unit + contract + lint CI stages
  ([10-TESTING-AND-BUILD.md](10-TESTING-AND-BUILD.md) §2).
- `release/X.Y`: cut from `main` at MINOR freeze; only fixes backported, gated by the
  full CI pipeline including integration/E2E stages.
- Feature branches: short-lived, one per PR, rebased (not merged) onto `main` to keep
  history linear and bisectable — bisectability matters given how much of this project's
  correctness is validated by "does the boot sequence still hit its milestones."

## 12. Release Strategy

1. Freeze `release/X.Y` from `main`.
2. Full CI pipeline (all 8 stages, [10-TESTING-AND-BUILD.md](10-TESTING-AND-BUILD.md) §2)
   plus the manual reference-hardware pass
   ([10-TESTING-AND-BUILD.md](10-TESTING-AND-BUILD.md) §4).
3. Publish signed ISO, signed browser-demo image, and update the update channel manifest
   ([05-PACKAGE-AND-UPDATE-SYSTEM.md](05-PACKAGE-AND-UPDATE-SYSTEM.md) §5) — existing
   installs discover the release through the normal A/B update flow, not a separate
   "reinstall" path.
4. Publish release notes generated from `CHANGELOG.md`, itself built from conventional
   commit messages / PR labels (mechanism finalized in Phase 2 tooling work, see
   [12-ROADMAP-AND-MILESTONES.md](12-ROADMAP-AND-MILESTONES.md)).
5. Deploy the corresponding `web/` build to novaos.dev
   ([07-BROWSER-DEPLOYMENT.md](07-BROWSER-DEPLOYMENT.md)) so the browser demo and the
   downloadable ISO are always the same release, never mismatched.
