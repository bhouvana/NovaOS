# Spec 22 — Release Process Specification

Status: Draft v0.1 · Last updated: 2026-07-18

Expands [../11-CODING-STANDARDS.md](../11-CODING-STANDARDS.md) §11–§12 into a full
channel model, per Staff Engineer review's "think beyond code" note.

## 1. Channels

| Channel | Source | Signing | Audience | Cadence |
|---|---|---|---|---|
| **Nightly** | `main`, every merge | Dev-signed (a separate, clearly-lower-trust key never accepted by a release/stable install's trusted keyring, [19-FILESYSTEM-LAYOUT-SPEC.md](19-FILESYSTEM-LAYOUT-SPEC.md) §2) | Core contributors, CI itself (dogfooding) | Every merge to `main` that passes fast-check + unit + contract CI ([../10-TESTING-AND-BUILD.md](../10-TESTING-AND-BUILD.md) §2 stages 1–3) |
| **Beta** | `release/X.Y` branch, pre-freeze | Beta signing key (distinct from stable — an update channel switch from beta→stable is a one-way trust upgrade, never silent) | Contributors + opted-in early testers via Nova Settings' update-channel selector | Once per `release/X.Y` cut, updated as fixes land pre-freeze |
| **Stable** | `release/X.Y` branch, post-freeze | Production signing key ([../08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md) §4) | Everyone (default channel) | Per [../11-CODING-STANDARDS.md](../11-CODING-STANDARDS.md) §12's release steps |

Debug overlays and other debug-build-only features
([21-OBSERVABILITY-SPEC.md](21-OBSERVABILITY-SPEC.md) §4) are compiled into Nightly and
Beta but stripped from Stable builds at the image-builder level
([11-BUILD-PIPELINE-SPEC.md](11-BUILD-PIPELINE-SPEC.md) §2) — a separate build profile
dimension from `full-iso`/`browser-demo`, orthogonal to it (a Nightly `browser-demo`
build is a real, buildable combination, used for testing the demo experience against
in-progress work).

## 2. Release Signing Key Hierarchy

```text
Root key (offline, never touches CI, used only to sign the two
   channel-signing keys below — rotated on a long cycle, e.g. yearly,
   via a manual, documented ceremony, not automated)
   ├── Stable signing key (used by CI's release pipeline, held as a
   │     CI secret, rotatable independently of the root)
   └── Beta signing key (separate CI secret, compromise of this key
         cannot be used to forge a Stable-channel update)

Dev/Nightly signing key: entirely separate hierarchy, not derived
   from the root key at all — a compromised Nightly key has zero
   ability to forge a Beta or Stable artifact, by construction, not
   just by policy
```

This mirrors [RFC-0009](../rfcs/RFC-0009-update-service.md) Security Considerations'
"OS-image signing key and app publisher keys are cryptographically distinct entries"
principle, extended one level further: the three OS-image channels are also mutually
non-forgeable.

## 3. Rollback

- **Single-device rollback**: the A/B mechanism already specified in
  [ADR-0008](../decisions/ADR-0008-filesystem-and-update-strategy.md) and
  [16-STATE-MACHINES.md](16-STATE-MACHINES.md) §4 — automatic on failed post-boot health
  check, or manual via a "Roll back to previous version" action in Nova Settings that
  simply re-triggers the same slot-switch mechanism in reverse.
- **Fleet-wide rollback** (a released Stable version turns out to be broken after
  reaching real users): the update channel manifest
  ([../05-PACKAGE-AND-UPDATE-SYSTEM.md](../05-PACKAGE-AND-UPDATE-SYSTEM.md) §5) is
  updated to point back at the previous version's signed image — devices that haven't
  yet updated simply never see the bad version; devices that already applied it are
  offered the previous version as a new "update" (an update *to* an older version is
  permitted specifically for this recovery case, an explicit exception to
  [07-PACKAGE-FORMAT-SPEC.md](07-PACKAGE-FORMAT-SPEC.md) §5's "monotonically
  increasing" version rule for *ordinary* updates — a fleet-rollback publishes a new,
  higher-numbered release whose *content* matches the last-known-good version, rather
  than literally serving an old version number, keeping the monotonic-version invariant
  intact everywhere except this one documented, deliberate exception).

## 4. Changelog Generation

- Every merge to `main` is a single PR (per
  [../11-CODING-STANDARDS.md](../11-CODING-STANDARDS.md) §11's linear-history norm) with
  a conventional-commit-style title (`feat:`, `fix:`, `perf:`, etc. — the same taxonomy
  many open-source Rust projects use, no NovaOS-specific invention needed here).
- `CHANGELOG.md` is generated automatically from merged PR titles/labels at each
  `release/X.Y` cut (Phase 2 tooling, [../12-ROADMAP-AND-MILESTONES.md](../12-ROADMAP-AND-MILESTONES.md)
  §3) — grouped by type (Features, Fixes, Performance, Breaking Changes), with
  Breaking Changes cross-linking to the relevant RFC/ADR when one exists (§5).
- Release notes published alongside each Stable release
  ([../11-CODING-STANDARDS.md](../11-CODING-STANDARDS.md) §12 step 4) are the same
  generated changelog, not a separately hand-written summary — avoids the two documents
  drifting apart.

## 5. RFC/ADR Cross-Linking in Releases

Per the RFC process adoption ([../rfcs/README.md](../rfcs/README.md)), any release whose
changelog includes a breaking SDK change, a new resident service, or a protocol version
bump must link the originating RFC/ADR directly in that changelog entry — makes "why did
this break" always one click from the release notes rather than requiring a git-log
archaeology session.

## 6. What This Spec Deliberately Does Not Cover

Package-level (`.novapkg`) release channels for third-party apps — that's Nova Store's
own publisher-facing concern
([../05-PACKAGE-AND-UPDATE-SYSTEM.md](../05-PACKAGE-AND-UPDATE-SYSTEM.md)), tracked
separately from the OS release process this document governs, and out of v1 scope per
[14-ECOSYSTEM-VISION.md](14-ECOSYSTEM-VISION.md) §2's "Nova Packages" row.
