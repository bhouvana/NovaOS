# Spec 20 — Configuration Strategy Specification

Status: Draft v0.1 · Last updated: 2026-07-18

Makes explicit a decision that was already implicit throughout this doc tree (every
manifest, theme file, and settings schema example used TOML) — per Staff Engineer
review, confirmed here as a deliberate, documented choice rather than an unstated
convention.

## 1. Format Decision: TOML

| Option | Verdict |
|---|---|
| JSON | No comments, no multi-line strings, easy to hand-edit incorrectly (trailing commas, quoting every key) — poor fit for human-maintained config files like app manifests |
| YAML | Comments and readability are good, but significant-whitespace parsing and YAML's notoriously large "surprising behavior" surface (implicit type coercion — the classic Norway-`no`-becomes-`false` class of bug) are a real footgun for a format used in security-relevant files like app manifests |
| Binary (e.g., a custom or Protobuf-based format) | Fast and compact, but not human-editable — wrong for files a developer or user is expected to read/hand-edit (app manifests, theme files, system config); Protobuf is already used where a binary format *is* appropriate (Nova Bus payloads, [15-NOVA-BUS-PROTOCOL-SPEC.md](15-NOVA-BUS-PROTOCOL-SPEC.md)) |
| TOML | Comments, unambiguous types, no significant whitespace, explicit array/table syntax, and already Rust's de facto config format (`Cargo.toml` itself) — no new parsing dependency beyond what the toolchain already pulls in |

**Decision: TOML for every human-authored or human-readable configuration file** —
app manifests ([06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md) §3), theme files
([10-DESIGN-BIBLE.md](10-DESIGN-BIBLE.md)), settings schemas
([06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md) §7), system config
([19-FILESYSTEM-LAYOUT-SPEC.md](19-FILESYSTEM-LAYOUT-SPEC.md) §1). This does not
warrant a numbered ADR (a build-tooling/format convention, not an on-disk format with
compatibility implications beyond what §6 below already covers) but is recorded here so
it's never re-litigated file-by-file.

## 2. Configuration Scopes

| Scope | Location | Written by | Read by |
|---|---|---|---|
| Global (system) | `/nova/config/system.toml` | Nova Settings only ([RFC-0007](../rfcs/RFC-0007-settings-service.md)) | `nova-sessiond`, `update-agent`, `novapkg-agent` |
| Theme | `/nova/config/theme.toml` | Nova Settings only | `nova-themed` |
| Per-user | *(reserved, not populated in v1 — [08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md) §6's single-primary-user model means "global" and "per-user" are currently the same scope; the schema leaves room for a future `/nova/config/users/<user>/` split without a breaking format change)* | — | — |
| Per-app | `/nova/data/<app_id>/settings.toml`, managed transparently by `nova-storage`'s `KvStore` (§06-NOVA-SDK-SPEC §4 — apps never hand-write this file directly, they go through the typed `KvStore` API) | The owning app | The owning app |

## 3. Live Reload

- **Theme**: `nova-themed` watches `theme.toml` for changes (inotify) *only* when the
  write came from itself via the `nova.settings.write` path
  ([RFC-0006](../rfcs/RFC-0006-theme-service.md)) — it does not blindly reload on any
  filesystem change to avoid picking up a half-written file from an external process
  (which shouldn't be writing there anyway, §19-FILESYSTEM-LAYOUT-SPEC §2, but defense
  in depth). The reload path is triggered by the same in-process call that wrote the
  file, not a separate file-watch mechanism — write and reload are one atomic operation
  from the service's perspective.
- **System config**: most `system.toml` keys (update channel, silent-apply preference)
  are read once at the relevant service's startup and take effect on next
  read/operation, not live — a change to the update channel doesn't need to
  retroactively affect an in-progress check. The one live-reload exception is anything
  routed through `nova.settings.write` at all (§RFC-0007), which by construction always
  triggers an immediate, event-driven update rather than a poll — there is no polling
  reload mechanism anywhere in NovaOS's configuration system, live reload is always
  Bus-event-driven.
- **Per-app**: the `KvStore::watch()` API
  ([06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md) §4) gives apps the same event-driven
  reactivity pattern for their own settings.

## 4. Validation

Every config file is validated against its schema **at load time**, before any value is
used — a validation failure never results in a partially-applied config:

- **App manifests**: validated by `novapkg-agent` at install time
  ([RFC-0004](../rfcs/RFC-0004-package-service.md)) and again by `nova-sessiond` at
  every launch (defense in depth — a manifest that was valid at install time but
  somehow corrupted on disk is caught before sandbox construction, not after).
- **Theme files**: validated by `nova-themed`'s schema validator
  ([RFC-0006](../rfcs/RFC-0006-theme-service.md) Failure Modes) — invalid file falls
  back to the compiled-in default, never a partial theme.
- **System config**: validated by each reading service against its own expected keys;
  an unrecognized key is ignored with a logged warning (forward-compatible — a newer
  config file read by an older service degrades gracefully), a malformed *known* key
  (wrong type) is a load failure, falling back to that key's compiled-in default with a
  logged error.

## 5. Schema Evolution

Every top-level config file carries a `schema_version` integer field. A reading service
checks this field first:

- `schema_version` equal to what the service expects: read normally.
- `schema_version` lower than expected: apply an in-memory migration function (a small,
  versioned chain of transformations, one per historical schema version — the same
  "additive, never breaking" philosophy as
  [15-NOVA-BUS-PROTOCOL-SPEC.md](15-NOVA-BUS-PROTOCOL-SPEC.md) §8's payload evolution
  rule, applied to files instead of wire messages) before use, then rewrite the file at
  the new version on next save.
- `schema_version` higher than expected (an older NovaOS build reading a file written by
  a newer one — relevant after a rollback, [ADR-0008](../decisions/ADR-0008-filesystem-and-update-strategy.md)):
  read only the fields the older schema recognizes, per §4's forward-compatible
  unrecognized-key handling; never a hard failure, since a rollback must not brick
  config reading.

## 6. What This Spec Deliberately Does Not Cover

Per-app custom config formats (an app is free to use its own format inside its own
`/nova/data/<app_id>/files/` scope for its own content — e.g., Nova Notes' note files
aren't TOML) — this spec governs NovaOS's *own* configuration surface (manifests,
themes, system/settings), not arbitrary app-private data formats, which are each app's
own concern.
