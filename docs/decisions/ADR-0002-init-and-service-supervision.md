# ADR-0002: Init System & Service Supervision

Status: Accepted
Date: 2026-07-18
Deciders: Chief Architect

## Context

We need something to be PID 1, bring up services in order, supervise/restart them, and
hand off to the Nova session. The RAM budget and "avoid unnecessary daemons" principle
([00-VISION.md](../00-VISION.md) §6) argue strongly against a maximal init system.

## Options Considered

1. **systemd** — de facto standard, huge feature set (logind, udev integration, timers,
   sockets, cgroups), but pulls in dozens of MB of resident services and a large surface
   area we would spend more time disabling than using. Directly conflicts with the "every
   daemon needs a measurable reason to exist" principle.
2. **runit** — extremely small, simple supervise-tree model, minimal RAM, no built-in
   dependency graph (ordering is done via numbered symlinks/explicit `sv` scripts) —
   simplicity comes with more manual wiring for complex dependency chains.
3. **s6 / s6-rc** — smaller and more correct than runit in some respects, steeper learning
   curve, smaller community, harder to onboard contributors quickly.
4. **OpenRC** — Alpine's native init, dependency-based service ordering, still lightweight
   (no cgroup-manager/logind/udev-replacement baggage by default), well documented, and
   already the natural fit given [ADR-0001](ADR-0001-linux-base-distribution.md).

## Decision

**OpenRC** as PID 1 and service supervisor for the base system. A small, purpose-built
**Nova Session Manager** (our own component, see
[03-DESKTOP-ARCHITECTURE.md](../03-DESKTOP-ARCHITECTURE.md) §Session Management) takes
over supervision of the *user session* (compositor, Nova services, apps) once OpenRC has
brought up the minimal system runlevel — OpenRC does not supervise per-app processes.

## Rationale

OpenRC gives dependency-ordered service startup (needed for a handful of real system
services: udev/mdev-style device management, network, D-Bus-compat shim if required) at a
fraction of systemd's resident cost, and it pairs naturally with the Alpine base, avoiding
a second package/config ecosystem. Session-level supervision is deliberately *not* pushed
into OpenRC — that's product-layer logic (restart a crashed app, not restart a system
service) and belongs to Nova Session Manager, keeping the boundary between "OS init" and
"Nova product" clean.

## Consequences

- We do not get systemd's unit-file ecosystem; third-party software shipping only
  `.service` files needs a small translation layer or a hand-written OpenRC init script —
  acceptable, since NovaOS ships almost no third-party system daemons.
- No `journald` — logging is handled by a minimal Nova logging convention
  (see future `LOGGING.md`, referenced from [00-VISION.md](../00-VISION.md)'s subsystem
  list; tracked for a follow-up doc, not blocking v1 architecture).
- No `systemd-logind`/`udev` full replacement — we take Alpine's `mdev` or `eudev` as
  shipped, minimally configured.

## Revisit Triggers

- If we need cgroup-v2-based resource limiting per-app at the *system* level (as opposed
  to Nova Session Manager doing it in userspace) at a granularity OpenRC can't express.
- If hardware-support software we must package assumes systemd unit files with no
  reasonable OpenRC equivalent.
