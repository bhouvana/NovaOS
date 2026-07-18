# Implementation Notes

Status: Process document · Last updated: 2026-07-18

Every time Phase 2+ implementation diverges from what Phase 1/1.5's architecture or
specs said, that divergence gets a numbered entry here — per Staff Engineer review,
added specifically because "why is the code different from the spec" is exactly the
kind of question that's cheap to answer the day it happens and expensive to
reconstruct six months later from commit archaeology.

## Relationship to ADRs and RFCs

| | ADR | RFC | Implementation Note |
|---|---|---|---|
| Answers | Which option, and why (a point decision) | How a whole subsystem works (a complete design) | Why the *running code* differs from what the ADR/RFC/spec said |
| Written | Before implementation | Before implementation | During or immediately after implementation |
| Triggers | A choice with alternatives worth recording | A new subsystem/protocol/breaking change | Reality (an environment constraint, a bug found under test, a design that didn't survive contact with real code) |

An Implementation Note that reveals the original ADR/RFC was simply wrong (not just
incomplete) should conclude by filing a superseding ADR or amending the RFC in place —
this directory is where the divergence is *first recorded*, not where it's meant to
live forever unresolved.

## Template

```markdown
# NNNN: Title

Date: YYYY-MM-DD
Status: Open | Resolved (superseded by ADR-XXXX / RFC-XXXX amendment)

## Architecture
What the spec/ADR/RFC said would happen.

## Reality
What actually happened when it was implemented/tested.

## Reason
Why the gap exists — environment constraint, bug, wrong assumption, etc.

## Decision
What was done about it right now.

## Future Direction
What closes this gap permanently, and when.
```

## Index

| # | Title | Status |
|---|---|---|
| [0001](0001-nova-bus-dev-transport-tcp.md) | Nova Bus dev transport is TCP loopback, not Unix sockets | Open — closes when a Linux build environment exists |
| [0002](0002-nova-bus-identity-self-reported.md) | Nova Bus peer identity is self-reported, not `SO_PEERCRED` | Open — same trigger as 0001 |
| [0003](0003-nova-bus-register-handler-wire-message.md) | `RegisterHandler`/`RegisterHandlerAck` added to the wire protocol | Resolved — spec updated |
| [0004](0004-nova-bus-response-authorization.md) | Response-spoofing vulnerability found and fixed | Resolved — code fixed, regression test added |
| [0005](0005-nova-bus-measured-performance.md) | Nova Bus measured latency/throughput (first real numbers) | Open — informs Phase 2.5 System Validation |
| [0006](0006-process-memory-windows-vs-linux.md) | Process memory measured on Windows dev host, not the Linux target | Open — closes when Linux build environment exists |
