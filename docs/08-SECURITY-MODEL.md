# NovaOS — Security Model

Status: Draft v0.1 · Owner: Chief Architect · Last updated: 2026-07-18

Related: [ADR-0006](decisions/ADR-0006-ipc-mechanism.md),
[ADR-0007](decisions/ADR-0007-package-format.md), [ADR-0010](decisions/ADR-0010-app-sandboxing-model.md)

## 1. Principles

Practical, not maximal: NovaOS is a single-user-primary desktop OS, not a multi-tenant
server. The model is scoped to the threats that actually matter for that context —
a malicious or buggy app, a compromised update, a stolen device — not exotic threats
that would demand disproportionate complexity against
[00-VISION.md](00-VISION.md) §6's simplicity-first priority.

## 2. Permission Taxonomy

Declared per-app in the manifest ([04-APPLICATION-FRAMEWORK-AND-SDK.md](04-APPLICATION-FRAMEWORK-AND-SDK.md)
§4), enforced by `nova-sessiond` at sandbox-construction time
([ADR-0010](decisions/ADR-0010-app-sandboxing-model.md)):

| Permission | Grants | Enforcement mechanism |
|---|---|---|
| `filesystem: [home, downloads, ...]` | Read/write to named, pre-defined path scopes only | Mount namespace + Landlock |
| `filesystem: user-selected` | One file/folder at a time, chosen via system file picker | Broker-mediated fd handoff, not a path grant |
| `network` | Outbound network access | Network namespace (absent = no network device at all) |
| `notifications` | Publish to `nova.notify` bus topic | Nova Bus broker ACL |
| `ipc_topics: [...]` | Publish/subscribe to named Nova Bus topics beyond the default set | Nova Bus broker ACL |
| `background` | Continue running when not focused (rare; most apps suspend) | `nova-sessiond` lifecycle policy |

Unlisted permissions are denied by default — an app gets nothing beyond CPU, memory
(bounded by cgroups), and its own `nova-storage` scope unless explicitly declared.

## 3. First-Run & Runtime Prompts

Store-distributed apps' declared permissions are shown at install time in Package Center
(no silent installs of powerful permissions). Some permissions (user-selected file
access, first use of `network` for apps that declared it) additionally prompt at
first-use via a compositor-level trusted surface (`nova-compositor` renders permission
prompts itself, not the requesting app, so a malicious app cannot spoof the prompt) —
this is the concrete reason the Nova protocol extension for a "permission-prompt surface
type" exists ([03-DESKTOP-ARCHITECTURE.md](03-DESKTOP-ARCHITECTURE.md) §2).

## 4. Package & Update Integrity

- Every `.novapkg` and every OS root image is signature-verified before use — detailed
  flow in [05-PACKAGE-AND-UPDATE-SYSTEM.md](05-PACKAGE-AND-UPDATE-SYSTEM.md) §4, §5.
- Verification uses a small, auditable set of trusted public keys shipped in the base
  image (`system/image/`), rotated via the same signed-update mechanism that updates
  everything else — no separate, harder-to-update trust-anchor mechanism.

## 5. Secrets Handling

- A single OS-level secrets store (`nova-sessiond`-adjacent, backed by a small
  encrypted-at-rest store keyed to the user's login credential) — apps request secret
  storage via `nova-storage`'s secret-scoped API, never write credentials to plain files
  in their storage scope.
- No app can read another app's secrets scope; the secrets store itself is not a Nova
  Bus-addressable service directly callable by arbitrary topic names — it's a narrow,
  purpose-built API surface, reducing the chance of an ACL misconfiguration exposing it.

## 6. User Accounts & Sessions

- v1 targets a single primary local account plus an optional guest/limited session — full
  multi-user OS-level isolation (separate home partitions, fast user switching) is a
  post-v1 target (see [14-FUTURE-VISION.md](14-FUTURE-VISION.md)), not because it's
  architecturally hard (the sandboxing/session model already isolates per-process) but
  because the UX (login screen, switch-user flow) isn't in v1 scope.
- Login is credential-gated (local password at minimum; hardware-key/biometric as a
  stretch target where hardware supports it) and unlocks the secrets store (§5).
- Screen lock reuses the same session state machine as login
  ([03-DESKTOP-ARCHITECTURE.md](03-DESKTOP-ARCHITECTURE.md) §6) — not a separate
  lock-screen subsystem.

## 7. Attack Surface Minimization

- Directly reinforces [00-VISION.md](00-VISION.md) §6: every resident process in
  [01-SYSTEM-ARCHITECTURE.md](01-SYSTEM-ARCHITECTURE.md) §3 is also a security surface —
  the same short list defended for RAM reasons is defended for security reasons.
- XWayland ([ADR-0003](decisions/ADR-0003-compositor-and-display-protocol.md)) is
  lazy-started specifically because X11 clients have a much weaker isolation story than
  native Wayland/sandboxed apps — not started means not exploitable.
- No apps run unsandboxed by default; first-party apps use the exact same
  `nova-sessiond` sandboxing path as third-party apps — "we trust our own code more" is
  never encoded as a bypass, both to dogfood the sandbox and to bound the blast radius of
  our own bugs.

## 8. Telemetry & Crash Reporting

- Off by default. If enabled by explicit user opt-in: scoped to crash signatures and
  coarse performance metrics (boot time, idle RAM) needed to validate the budgets in
  [09-PERFORMANCE-STRATEGY.md](09-PERFORMANCE-STRATEGY.md) — never file contents, never
  browsing history, never app-specific user data.
- Reported over the same signed-channel infrastructure as updates, to one documented
  endpoint, with the exact payload schema published in-repo (no undocumented telemetry
  fields ever shipped).

## 9. Out of Scope (v1)

- Full disk encryption is a target-hardware/installer concern
  ([12-ROADMAP-AND-MILESTONES.md](12-ROADMAP-AND-MILESTONES.md) Phase 6), not a core
  architecture requirement for v1's boot/compositor/app model.
- Formal verification of the sandboxing policy compiler — tracked as a future hardening
  step once the manifest-to-sandbox-rule mapping ([ADR-0010](decisions/ADR-0010-app-sandboxing-model.md)
  Consequences) is stable enough to be worth formalizing.
