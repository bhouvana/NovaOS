# Spec 12 — Browser Demo Experience

Status: Draft v0.1 · Last updated: 2026-07-18

The actual UX of landing on novaos.dev/demo, addressing the Staff Engineer note that
this surface is easy to underestimate: it is the primary way most people will ever
touch NovaOS, and it should feel like a designed moment, not a technical proof-of-
concept. Built on top of [08-BROWSER-ARCHITECTURE-SPEC.md](08-BROWSER-ARCHITECTURE-SPEC.md).

## 1. Design Reference

The "Windows 95 in a browser" reference point the Staff Engineer raised is well-chosen:
what made those demos compelling wasn't technical fidelity, it was *immediacy* — no
loading wall, no explanation required, the thing just works and invites clicking. NovaOS
targets the same immediacy with its own visual identity, not a pastiche of another OS.

## 2. Flow

```text
Land on novaos.dev
   ↓
Hero section: one-line pitch + a large "Try it now" button (no
   separate landing → demo-page navigation for the primary CTA —
   the demo starts loading the moment it's requested, minimizing
   clicks-to-value per 13-WEBSITE-INFORMATION-ARCHITECTURE.md §2)
   ↓
Branded loading screen (08-BROWSER-ARCHITECTURE-SPEC.md §2) —
   shows real progress (asset fetch %, then guest boot milestones
   relayed from 03-BOOT-TIMELINE.md's ring buffer once the guest
   is far enough along to publish them), never a fake/generic spinner
   ↓
Desktop appears, guest reaches "Desktop ready"
   ↓
First-run welcome notification (a real nova.notify.publish message
   from a pre-seeded "Welcome" helper, not a browser-side overlay —
   dogfoods the real notification system, 01-INTERACTION-FLOWS.md §3)
   introduces: "This is a real, running OS — try the Terminal, or
   open a game from the Taskbar."
   ↓
Guided-tour highlight sequence (dismissible, shown once per
   IndexedDB-persisted session): Launcher → Terminal → a pinned
   Arcade game → Package Center's (offline, catalog-cached) browse
   view → the persistent "Install NovaOS" CTA (§4)
   ↓
Free exploration — full desktop interactivity, no further
   guardrails; a visitor can open/close/break things freely since
   "Restart Demo" (08-BROWSER-ARCHITECTURE-SPEC.md §8) always
   recovers to a pristine state
```

## 3. Pre-Seeded Desktop State

The `browser-demo` image profile ([11-BUILD-PIPELINE-SPEC.md](11-BUILD-PIPELINE-SPEC.md)
§2) ships with:
- Taskbar pre-pinned: Files, Terminal, one Arcade game (Snake — fastest to understand at
  a glance), Package Center.
- Nova Files pre-populated with a small set of sample files/folders (not an empty,
  unconvincing home directory) — demonstrates the file manager has something to show
  without requiring the visitor to create content first.
- Package Center's catalog cache pre-populated with the full real catalog metadata
  (names, icons, descriptions) even though actual installs are disabled in the demo
  (§5) — browsing feels real, not like a placeholder screen.

## 4. Install CTA

A persistent, low-key affordance (not a modal interrupting exploration) — a taskbar
"Install NovaOS" button that links to the real Downloads page
([13-WEBSITE-INFORMATION-ARCHITECTURE.md](13-WEBSITE-INFORMATION-ARCHITECTURE.md)). This
is deliberately *inside* the emulated desktop's own taskbar (a NovaOS-side affordance,
shown only under the `browser-demo` profile via the same cmdline-flag mechanism as
`nova-browser-bridge`, [08-BROWSER-ARCHITECTURE-SPEC.md](08-BROWSER-ARCHITECTURE-SPEC.md)
§7) rather than page-level browser chrome — reinforces that what the visitor is using
*is* the real product, and the install path leads to more of the same thing, not a
different product.

## 5. What's Disabled in the Demo

- Package Center install/update actions are visible but produce an inline explanation
  ("Installing requires the full NovaOS — try it on real hardware or a VM") rather than
  a broken/hanging action — never a silent no-op.
- OS update checks: `update-agent` does not run under the `browser-demo` profile at all
  (no update channel exists for an ephemeral demo instance).
- Networking: off by default per
  [08-BROWSER-ARCHITECTURE-SPEC.md](08-BROWSER-ARCHITECTURE-SPEC.md) §9; Nova Browser is
  excluded from `preinstalled_apps` in the `browser-demo` profile
  ([11-BUILD-PIPELINE-SPEC.md](11-BUILD-PIPELINE-SPEC.md) §2) specifically because "a
  web browser that can't reach the web" is a worse demo moment than simply not including
  it — visitors are already in a real browser tab if they want that.

## 6. Recruiter/Portfolio Appeal (explicit design goal)

Per the Staff Engineer's framing — this surface is a legitimate part of the project's
value, not just a technical demo:

- Snake is pinned specifically because it's instantly recognizable and playable within
  seconds with no explanation, the single highest "someone clicks and immediately gets
  it" interaction in the whole demo.
- The guided tour (§2) is short (4 highlighted stops) and skippable — optimized for
  someone spending 60–90 seconds, not a multi-minute onboarding a casual visitor won't
  finish.
- Session state (§4/§8 of [08-BROWSER-ARCHITECTURE-SPEC.md](08-BROWSER-ARCHITECTURE-SPEC.md))
  means a visitor who opens a few windows and switches tabs to look at something else
  can come back and find their demo session intact within the same browser tab — no
  "your session expired" friction.

## 7. Success Metric

Tracked the same way as the rest of
[../09-PERFORMANCE-STRATEGY.md](../09-PERFORMANCE-STRATEGY.md): page-load-to-interactive
≤15s (§2 budget) is the release-blocking metric; this doc's UX flow is validated
qualitatively at each Phase 6 milestone review
([../12-ROADMAP-AND-MILESTONES.md](../12-ROADMAP-AND-MILESTONES.md) §8), not by an
automated metric — "does this feel good to click through" is a manual-test-pass
judgment call, not a CI assertion.
