# NovaOS Engineering Principles

Status: Capstone document · Last updated: 2026-07-18

This is the short list every future contribution — code, RFC, ADR, or PR review — is
judged against. Where a decision elsewhere in this doc tree seems to conflict with one
of these, the more specific document wins for its own domain, but the rationale should
trace back to one of these principles or explain why it doesn't. This document doesn't
introduce new rules; it distills principles already argued for, case by case, across
[00-VISION.md](00-VISION.md) §6, every ADR's Rationale section, and every RFC's design
choices, into one page worth reading before writing the first line of Rust.

## The List

**Memory before convenience.** A feature that costs resident RAM must earn that cost
against [../docs/specs/02-MEMORY-BUDGET.md](specs/02-MEMORY-BUDGET.md)'s ledger, not
assume the budget will stretch. Convenience for a developer (a background cache, a
"just in case" resident helper) is not sufficient justification on its own.

**Zero unnecessary background services.** Every entry in
[01-SYSTEM-ARCHITECTURE.md](01-SYSTEM-ARCHITECTURE.md) §3's "Always resident" column
required an ADR to justify. This isn't a one-time gate — it's a standing bar every new
service proposal (now formalized as an RFC requirement,
[rfcs/README.md](rfcs/README.md)) must clear.

**Native-first applications.** Every first-party app is built with Nova UI against the
real SDK ([04-APPLICATION-FRAMEWORK-AND-SDK.md](04-APPLICATION-FRAMEWORK-AND-SDK.md)),
sandboxed identically to third-party apps
([../08-SECURITY-MODEL.md](08-SECURITY-MODEL.md) §7). No first-party shortcut, no
"just this once" unsandboxed process, no web-view escape hatch outside the one
documented exception (Nova Browser's embedded engine,
[ADR-0005](decisions/ADR-0005-ui-toolkit.md) Consequences).

**One design language.** [06-DESIGN-SYSTEM.md](06-DESIGN-SYSTEM.md) and
[specs/10-DESIGN-BIBLE.md](specs/10-DESIGN-BIBLE.md) are not suggestions — `nova-ui`'s
type signatures make bypassing them a compile error, not a style-guide violation
([specs/05-NOVA-UI-TOOLKIT-SPEC.md](specs/05-NOVA-UI-TOOLKIT-SPEC.md) §6). If a design
need doesn't fit the current token set, the token set gets a new entry through review —
it doesn't get bypassed locally.

**Stable public APIs.** The Nova SDK is a product with its own versioning and
deprecation contract ([specs/17-SDK-API-REFERENCE-POLICY.md](specs/17-SDK-API-REFERENCE-POLICY.md)),
independent of how fast the rest of the OS moves internally. An `#[internal]` item can
change on a whim; a `#[stable]` item cannot, ever, without the deprecation cycle that
policy defines.

**Backward compatibility where practical, never at the cost of a security boundary.**
`sdk_version` ranges let old apps keep working across SDK minor versions
([specs/06-NOVA-SDK-SPEC.md](specs/06-NOVA-SDK-SPEC.md) §10). The sandboxing model
([ADR-0010](decisions/ADR-0010-app-sandboxing-model.md)) is the one place compatibility
is never the deciding factor — an app that needs a permission it wasn't granted asks for
it and waits for the user, it does not get a compatibility exception.

**Measured performance over assumptions.** Every budget in this doc tree —
[specs/02-MEMORY-BUDGET.md](specs/02-MEMORY-BUDGET.md),
[specs/03-BOOT-TIMELINE.md](specs/03-BOOT-TIMELINE.md),
[09-PERFORMANCE-STRATEGY.md](09-PERFORMANCE-STRATEGY.md) — is a number to be measured in
CI and on reference hardware, not asserted and left unchecked
([10-TESTING-AND-BUILD.md](10-TESTING-AND-BUILD.md) §2 stage 5). "It feels fast" is not
evidence.

**Simplicity over cleverness.** When two designs solve the same problem, the one with
fewer moving parts wins even if the clever one is more efficient on paper — this is
[00-VISION.md](00-VISION.md) §6's priority order made concrete, and it's the reasoning
behind choosing OpenRC over systemd
([ADR-0002](decisions/ADR-0002-init-and-service-supervision.md)), a custom minimal Nova
Bus over D-Bus ([ADR-0006](decisions/ADR-0006-ipc-mechanism.md)), and static
compile-time topic routing over dynamic service discovery
([specs/15-NOVA-BUS-PROTOCOL-SPEC.md](specs/15-NOVA-BUS-PROTOCOL-SPEC.md) §9).

**Every non-trivial architectural change gets an RFC or an ADR before code.** Formalized
in [rfcs/README.md](rfcs/README.md). This is the standing discipline the Staff Engineer
review that closed out Phase 1.5 made a condition of approving implementation — it does
not expire once Phase 2 starts.

## How to Use This Document

When reviewing a PR or an RFC draft and something feels off but you can't immediately
point to which specific doc it violates, check it against this list first — it's usually
one of these nine. When writing a new RFC, its Rationale should be traceable to at least
one of these; if it isn't, that's a sign the RFC is solving a problem NovaOS doesn't
actually have.
