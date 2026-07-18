# services/

Status: **Not yet implemented** — Phase 1 (architecture only). Planned for Phase 2–3
([../docs/12-ROADMAP-AND-MILESTONES.md](../docs/12-ROADMAP-AND-MILESTONES.md) §3–§4).

Resident Nova Services — the "Nova Services" layer in
[../docs/01-SYSTEM-ARCHITECTURE.md](../docs/01-SYSTEM-ARCHITECTURE.md) §1. Each is a
Rust crate ([ADR-0004](../docs/decisions/ADR-0004-systems-language.md)); every process
listed here appears in the resident-process ledger in
[../docs/01-SYSTEM-ARCHITECTURE.md](../docs/01-SYSTEM-ARCHITECTURE.md) §3 and is
accountable to the RAM budget in
[../docs/09-PERFORMANCE-STRATEGY.md](../docs/09-PERFORMANCE-STRATEGY.md).

| Crate | Purpose | Doc |
|---|---|---|
| `nova-bus/` | IPC broker (`novabusd`) + `.proto` wire schemas | [ADR-0006](../docs/decisions/ADR-0006-ipc-mechanism.md) |
| `nova-sessiond/` | App lifecycle, sandboxing, login/lock/session state machine | [03-DESKTOP-ARCHITECTURE.md](../docs/03-DESKTOP-ARCHITECTURE.md) §6, [ADR-0010](../docs/decisions/ADR-0010-app-sandboxing-model.md) |
| `nova-themed/` | Theme token engine, live theme switching | [03-DESKTOP-ARCHITECTURE.md](../docs/03-DESKTOP-ARCHITECTURE.md) §7 |
| `novapkg/` | Package manager agent + CLI | [05-PACKAGE-AND-UPDATE-SYSTEM.md](../docs/05-PACKAGE-AND-UPDATE-SYSTEM.md) |
| `permission-broker/` | Permission grant/prompt logic used by `nova-sessiond` | [08-SECURITY-MODEL.md](../docs/08-SECURITY-MODEL.md) §3 |

Dependency direction: may depend on each other (see
[../docs/02-REPOSITORY-STRUCTURE.md](../docs/02-REPOSITORY-STRUCTURE.md) §3 rule 4);
never depended on directly by `apps/*`.
