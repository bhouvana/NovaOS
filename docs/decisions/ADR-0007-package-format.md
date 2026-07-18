# ADR-0007: Package Format & Package Center

Status: Accepted
Date: 2026-07-18
Deciders: Chief Architect

## Context

NovaOS needs a package format and install/update flow for two distinct concerns that must
not be conflated: (a) base-system packages assembled at image-build time (kernel, musl,
compositor, core services), and (b) end-user-installable applications distributed and
updated post-install via **Nova Package Center**. This ADR concerns (b), the user-facing
format — (a) is covered by [ADR-0001](ADR-0001-linux-base-distribution.md)/
[ADR-0008](ADR-0008-filesystem-and-update-strategy.md).

## Options Considered

1. **Reuse `apk` directly for user apps** — least new code, but exposes Alpine's package
   identity to end users and conflates "base system package" with "Nova app," which we
   explicitly want to keep separate (see [ADR-0001](ADR-0001-linux-base-distribution.md)).
2. **Flatpak** — strong sandboxing and a large existing app catalog, but its runtime model
   (shared runtimes, OSTree-backed repo, bundled dependency trees per runtime) carries
   more disk/complexity than a single-vendor curated app set needs, and its GNOME/Freedesktop
   runtime assumptions don't match Nova UI ([ADR-0005](ADR-0005-ui-toolkit.md)).
3. **AppImage** — zero-install-step simplicity, but no real dependency resolution, no
   central catalog/discovery, no update mechanism beyond the app polling for itself —
   insufficient for a first-class "Package Center" experience.
4. **Custom format ("`.novapkg`"): a signed, compressed archive (SquashFS) + manifest,
   installed into an isolated per-app directory, resolved against a small set of shared
   Nova runtime libraries (Nova UI, SDK) via a central catalog server** — purpose-built,
   matches our sandboxing model ([ADR-0010](ADR-0010-app-sandboxing-model.md)), matches
   our update model ([ADR-0008](ADR-0008-filesystem-and-update-strategy.md)).

## Decision

**`.novapkg`**: a signed SquashFS image containing the app binary/assets plus a manifest
(name, version, declared permissions, SDK version dependency, icon, metadata), mounted
read-only at install time under `/nova/apps/<id>/<version>/` and resolved against the
shared Nova runtime (Nova UI, Nova Bus client libs) already present in the base system —
apps do not bundle their own copy of the toolkit. **Nova Package Center** is the one GUI
(and CLI: `novapkg`) for install/update/remove/search, backed by a signed package
repository ("Nova Store" catalog service, hostable by us or self-hosted by others).

## Rationale

A SquashFS-per-app-version layout gives atomic installs/rollbacks (swap a mount point),
small download deltas are achievable since apps don't bundle the shared toolkit, and
signature verification at install time gives a simple, understandable security story
(see [08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md)) without adopting Flatpak's heavier
runtime model. This keeps the "every app looks and feels native" goal intact because
every app is required to link the one shared Nova UI, rather than each bundling its own
GTK/Qt/Electron stack.

## Consequences

- Third-party apps must target the Nova SDK to be distributable through Nova Store; there
  is no "just package your existing GTK app" fast path — an accepted tradeoff for
  consistency (see [ADR-0005](ADR-0005-ui-toolkit.md) consequences).
- A compatibility "App Runner" for select non-native Linux apps (e.g., via a narrowly
  scoped container/AppImage-runner) may exist later as an explicitly second-class,
  opt-in path — not part of v1 scope.
- Nova Store catalog is a separate service (see
  [05-PACKAGE-AND-UPDATE-SYSTEM.md](../05-PACKAGE-AND-UPDATE-SYSTEM.md)) with its own
  signing keys, independent of the OS image update channel
  ([ADR-0008](ADR-0008-filesystem-and-update-strategy.md)).

## Revisit Triggers

- If the "no bundled dependencies" constraint proves too restrictive for real third-party
  apps once the SDK stabilizes and an ecosystem starts forming.
