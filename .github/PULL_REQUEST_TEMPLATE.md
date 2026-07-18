## What this changes

## Why

## Which RFC/ADR governs this (if applicable)

## Checklist

Reused from [docs/11-CODING-STANDARDS.md](../docs/11-CODING-STANDARDS.md) §9:

- [ ] Matches the dependency-direction rules ([docs/02-REPOSITORY-STRUCTURE.md](../docs/02-REPOSITORY-STRUCTURE.md) §3)
- [ ] No new resident process without an ADR ([docs/01-SYSTEM-ARCHITECTURE.md](../docs/01-SYSTEM-ARCHITECTURE.md) §3)
- [ ] Errors handled per coding standards §4, no silent failure paths
- [ ] Unit tests present for new logic in `services/`/`sdk/`
- [ ] Public SDK items documented ([docs/11-CODING-STANDARDS.md](../docs/11-CODING-STANDARDS.md) §7)
- [ ] No secrets/PII in logs
- [ ] Performance-sensitive paths checked against [docs/specs/02-MEMORY-BUDGET.md](../docs/specs/02-MEMORY-BUDGET.md) / [docs/specs/03-BOOT-TIMELINE.md](../docs/specs/03-BOOT-TIMELINE.md)
- [ ] New/changed UI uses `nova-ui` tokens/components, not one-off styling
- [ ] A new subsystem/protocol/breaking change has an accompanying RFC or ADR
      ([docs/rfcs/README.md](../docs/rfcs/README.md))

## How this was tested
