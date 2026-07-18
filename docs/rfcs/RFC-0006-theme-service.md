# RFC-0006: Theme Service

Status: Accepted
Date: 2026-07-18
Owner: Chief Architect

## Purpose

`nova-themed` owns the active theme's token set and is the single source of truth every
Nova UI instance renders against. Concrete token values in
[10-DESIGN-BIBLE.md](../specs/10-DESIGN-BIBLE.md).

## Responsibilities

- Load the active theme (Nova Light/Dark, or a signed third-party theme file) at startup.
- Validate a theme file against the token schema before activating it (reject malformed/
  incomplete themes rather than partially applying them).
- Publish the resolved token set on change; support live switching with no app restart.

## Dependencies

`novabusd` only.

## Public APIs

`nova.settings.write {key: "nova.theme.mode", value: "light"|"dark"|"system"}` (routed
here from `nova-settings-api`, [RFC-0007](RFC-0007-settings-service.md) — theme is the
one settings category `nova-themed` itself owns, per that RFC's data-ownership split).
No separate "set custom theme" call in v1 (third-party themes are a
[../14-FUTURE-VISION.md](../14-FUTURE-VISION.md) deferral,
[../06-DESIGN-SYSTEM.md](../06-DESIGN-SYSTEM.md) §6).

## Events Published

`nova.theme.changed {tokens}` — full resolved token set, published on: startup, mode
change (light/dark/system), and OS-level "system" mode following a
day/night-schedule or system-appearance signal (a `nova-themed`-internal timer, not an
externally triggered event).

## Events Consumed

`nova.settings.write` (for the `nova.theme.mode` key only — the broker ACL restricts
which keys route here vs. to `nova-sessiond`, [RFC-0007](RFC-0007-settings-service.md)
Public APIs).

## Configuration

Active theme token file at `/nova/config/theme.toml`
([19-FILESYSTEM-LAYOUT-SPEC.md](../specs/19-FILESYSTEM-LAYOUT-SPEC.md)),
schema-versioned ([20-CONFIGURATION-STRATEGY-SPEC.md](../specs/20-CONFIGURATION-STRATEGY-SPEC.md)
§6). Default token values for Nova Light/Dark are compiled into `nova-themed` itself
(not loaded from disk) so the desktop always has a valid theme even before any user
config exists — `/nova/config/theme.toml` only needs to exist to *override* the mode
selection, not to define the token values.

## Startup Order

Starts alongside `nova-sessiond`
([../01-SYSTEM-ARCHITECTURE.md](../01-SYSTEM-ARCHITECTURE.md) §3 table) — must be ready
before `nova-shell`/apps request their first theme token set, though in practice every
Nova UI instance has compiled-in fallback tokens (matching Nova Light) so a slow
`nova-themed` start degrades to "briefly shows default light theme, then re-themes" 
rather than blocking rendering.

## Failure Modes

- **Crash**: every running app keeps its last-received token set (Nova UI holds a
  cached `Arc<ThemeTokens>`, [05-NOVA-UI-TOOLKIT-SPEC.md](../specs/05-NOVA-UI-TOOLKIT-SPEC.md)
  §6) — theme changes stop working, but the desktop doesn't visually break.
- **Malformed theme file on disk**: rejected at load time, falls back to the compiled-in
  Nova Light default with a logged error — never a broken/partial theme applied.

## Recovery Strategy

`nova-sessiond` restarts `nova-themed` on crash (bounded retry, same policy family as
[RFC-0001](RFC-0001-nova-shell.md)); on restart it re-publishes
`nova.theme.changed`, and every live app picks up the current tokens again.

## Metrics

Theme-switch count, token-file load failures, subscriber count (how many processes are
currently holding a live token subscription).

## Logging

Theme mode changes (info), token file validation failures (warn), crash/restart (error).

## Security Considerations

Theme files are data, not code (§Purpose,
[../06-DESIGN-SYSTEM.md](../06-DESIGN-SYSTEM.md) §6) — the schema validator
(§Responsibilities) is the security-relevant piece here: it must reject any field that
isn't a recognized token (color/spacing/radius/elevation/type value), so a theme file
can never be used to inject arbitrary content or exceed its declared surface. Only
`nova-settings` (the app) is permitted to write `nova.theme.mode`
([RFC-0007](RFC-0007-settings-service.md) Public APIs), enforced at the Nova Bus ACL —
no other app can change the system theme out from under the user.

## Changelog

- 2026-07-18: Accepted.
