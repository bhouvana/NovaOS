# NovaOS — Browser Deployment Strategy

Status: Draft v0.1 · Owner: Chief Architect · Last updated: 2026-07-18

Related: [ADR-0009](decisions/ADR-0009-browser-boot-emulator.md)

## 1. Goal

A visitor to novaos.dev boots the real NovaOS within seconds, in-tab, with no install,
no account, and no backend VM per visitor.

## 2. Architecture

```
novaos.dev (static site, CDN/Render-hostable)
├── index.html + app shell (marketing/docs pages)
├── /demo
│   ├── v86 runtime (WASM + JS glue, vendored, ADR-0009)
│   └── nova-browser-demo.img  (SquashFS root image, browser-optimized variant)
├── /docs                      (SDK docs, architecture docs rendered as a doc site)
└── /downloads
    └── novaos-<version>.iso   (full installable ISO)
```

No application server, no database, no per-session backend process. The entire demo is
static assets served to the browser, which is what makes "instantly accessible" and
cheap-to-host compatible with each other (see
[ADR-0009](decisions/ADR-0009-browser-boot-emulator.md) Rationale).

## 3. Image Variants

Two image build targets from one source tree (not two forks):

| | Full ISO | Browser Demo Image |
|---|---|---|
| Target | Real hardware, general VM | v86 in-browser |
| Apps preloaded | Full first-party suite | Reduced set (enough to demonstrate the
  desktop shell, Files, Terminal, one or two apps, one Arcade game) |
| Rendering | GPU-accelerated Nova UI | Software-rendering fallback path
  ([04-APPLICATION-FRAMEWORK-AND-SDK.md](04-APPLICATION-FRAMEWORK-AND-SDK.md) §5) |
| Persistence | Full A/B + data partition | In-browser save-state only (v86 state
  snapshot), no network sync |
| Size target | Optimized for install media | Optimized for first-load download size |

Both are produced by the same `tools/` image-builder (see
[10-TESTING-AND-BUILD.md](10-TESTING-AND-BUILD.md)) from the same `system/image/`
recipes, parameterized by a build profile — this is a build-config difference, not a
maintained fork, so the browser demo never silently drifts from what the real OS does.

## 4. Performance Budget

- First interactive boot (page load → NovaOS desktop visible) target defined and tracked
  in [09-PERFORMANCE-STRATEGY.md](09-PERFORMANCE-STRATEGY.md) — this is the single most
  important number for the "instantly accessible" goal and is treated as a release-
  blocking metric for the demo, the same way idle RAM is release-blocking for the OS.
- Asset delivery: the demo image is compressed and served with CDN caching + range
  requests so v86 can begin execution before the full image finishes downloading, where
  the emulator supports progressive/streamed disk loading.

## 5. Hosting

- Static hosting (Render static site or equivalent) — no server-side compute requirement,
  per [ADR-0009](decisions/ADR-0009-browser-boot-emulator.md) Rationale.
- Documentation (architecture docs, SDK reference) is generated from this `docs/` tree
  and SDK doc-comments and published to `/docs` on the same site, so the source of truth
  for docs is always this repository, never a hand-maintained copy.

## 6. Constraints This Places on the Rest of the System

- Nova UI must have a functioning software-rendering path (no hard GPU dependency) — a
  requirement that also benefits low-end real hardware, so this isn't purely a
  browser-demo tax.
- No component may assume network reachability at boot (the demo may run fully offline
  once loaded) — reinforces [05-PACKAGE-AND-UPDATE-SYSTEM.md](05-PACKAGE-AND-UPDATE-SYSTEM.md)
  §6's offline-mode requirement rather than introducing a browser-only special case.
- Image size discipline (§3) is a standing constraint on what ships preloaded in the demo
  variant, reviewed at every milestone that adds a new default app.

## 7. Explicit Non-Goals

- No attempt to make the in-browser instance persist or sync across visits/devices —
  that would require backend infrastructure this strategy specifically avoids.
- No mobile-browser optimization target for v1 (desktop browser only) — revisit in
  [14-FUTURE-VISION.md](14-FUTURE-VISION.md) once the desktop experience is solid.
