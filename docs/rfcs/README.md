# RFCs

Status: Process document · Last updated: 2026-07-18

## RFC vs. ADR

Both live in this repository; they answer different questions and are both required
going forward per the Staff Engineer review that closed out Phase 1.5 — this is the
"adopt an RFC process" condition of that review.

| | ADR ([../decisions/](../decisions/)) | RFC (this directory) |
|---|---|---|
| Answers | "Which option, and why" — a single point decision | "How does this whole subsystem work" — a complete design |
| Length | Half a page to a page | Several pages: a fixed template covering contract, failure modes, observability, security |
| Triggered by | A choice with alternatives worth recording ([../11-CODING-STANDARDS.md](../11-CODING-STANDARDS.md) §8) | A new subsystem, a new protocol, or a breaking change to an existing subsystem's contract |
| Lifecycle | Accepted, rarely revised — superseded by a new ADR if reversed | Draft → Proposed → Accepted → **implemented against**; amended in place as the subsystem evolves, with a changelog at the bottom |

A new resident service always gets both: an ADR if its *existence* required choosing
between alternatives (most did — see [../decisions/](../decisions/)), and an RFC
describing how it actually behaves once decided. The nine RFCs in this directory are the
second half of that pair for every core Nova service.

## When an RFC Is Required (going forward, post-Phase-1.5)

- A new resident service or subsystem.
- A change to the Nova Bus wire protocol
  ([15-NOVA-BUS-PROTOCOL-SPEC.md](../specs/15-NOVA-BUS-PROTOCOL-SPEC.md)).
- A breaking change to an SDK API's public contract
  ([17-SDK-API-REFERENCE-POLICY.md](../specs/17-SDK-API-REFERENCE-POLICY.md)).
- A change to the `.novapkg` format, filesystem layout, or configuration schema
  versioning strategy.

Not required for: ordinary feature work inside an already-RFC'd subsystem, bug fixes,
or internal refactors that don't change a subsystem's public contract (its Bus topics,
its public API, its on-disk format). The bar is the same spirit as
[../11-CODING-STANDARDS.md](../11-CODING-STANDARDS.md) §8's ADR bar: would a reasonable
engineer, six months from now, need this written down to avoid re-deriving it or
breaking a contract they didn't know existed?

## Process

1. Open a PR adding `RFC-NNNN-<slug>.md` in `Draft` status using the template below.
2. Discussion happens on the PR. At this stage the RFC can change freely.
3. Once the design is settled, status moves to `Proposed` — a last-call period before
   merge (mirrors [../11-CODING-STANDARDS.md](../11-CODING-STANDARDS.md) §11's PR review
   norms, no separate tooling required).
4. Merge moves status to `Accepted`. Implementation may now begin/continue against it.
5. Future changes to an `Accepted` RFC are made in place, with a dated entry added to
   that RFC's `## Changelog` section at the bottom — the RFC is a living contract for its
   subsystem, not a frozen historical record like an ADR.

## Template

```markdown
# RFC-NNNN: <Service Name>

Status: Draft | Proposed | Accepted
Date: YYYY-MM-DD
Owner: <role/name>

## Purpose
One paragraph: what this service exists to do, and what it explicitly does not do.

## Responsibilities
Bulleted, concrete.

## Dependencies
What this service requires to function — other services, kernel features, hardware.

## Public APIs
Every API surface this service exposes: Nova Bus topics (request/response and
pub/sub), SDK client APIs if any, CLI if any. Signatures, not just names.

## Events Published
Nova Bus topics this service publishes to, with payload shape and when.

## Events Consumed
Nova Bus topics this service subscribes to, and what it does on receipt.

## Configuration
What's configurable, where the config lives, what happens on invalid config.

## Startup Order
Where this service sits in the boot/session sequence, and what it depends on being
ready before it can start.

## Failure Modes
Concrete ways this service can fail, and what "failed" looks like from the outside.

## Recovery Strategy
What happens automatically, what surfaces to the user, what requires a restart of what.

## Metrics
What this service exposes to Nova Monitor / the observability system.

## Logging
What gets logged, at what level, and what must never be logged.

## Security Considerations
Attack surface, trust boundaries, what this service must never do regardless of input.

## Changelog
- YYYY-MM-DD: Accepted.
```

## Index

| RFC | Service | Backing process(es) | Status |
|---|---|---|---|
| [0001](RFC-0001-nova-shell.md) | Nova Shell | `nova-shell` | Accepted |
| [0002](RFC-0002-nova-bus.md) | Nova Bus | `novabusd` | Accepted |
| [0003](RFC-0003-nova-wm.md) | Nova WM (Compositor) | `nova-compositor` | Accepted |
| [0004](RFC-0004-package-service.md) | Package Service | `novapkg-agent` | Accepted |
| [0005](RFC-0005-notification-service.md) | Notification Service | `nova-shell` (logical subsystem, not a separate process) | Accepted |
| [0006](RFC-0006-theme-service.md) | Theme Service | `nova-themed` | Accepted |
| [0007](RFC-0007-settings-service.md) | Settings Service | `nova-themed` + `nova-sessiond` (logical subsystem, split by data ownership) | Accepted |
| [0008](RFC-0008-session-manager.md) | Session Manager | `nova-sessiond` | Accepted |
| [0009](RFC-0009-update-service.md) | Update Service | `update-agent` | Accepted |
