# NovaOS — Application Framework & SDK

Status: Draft v0.1 · Owner: Chief Architect · Last updated: 2026-07-18

Related: [ADR-0004](decisions/ADR-0004-systems-language.md),
[ADR-0005](decisions/ADR-0005-ui-toolkit.md), [ADR-0006](decisions/ADR-0006-ipc-mechanism.md)

## 1. Purpose

The SDK is the *only* thing a Nova app — first-party or third-party — is allowed to
depend on (see [02-REPOSITORY-STRUCTURE.md](02-REPOSITORY-STRUCTURE.md) §3, Dependency
Rules). It is treated as a product with its own stability/versioning guarantees, not an
internal implementation detail exposed by accident.

## 2. App Lifecycle

Every Nova app implements a single trait/entrypoint contract provided by `sdk/nova-app`:

```
App::new(context) -> Self
App::on_launch(&mut self)
App::on_window_event(&mut self, event)
App::on_suspend(&mut self)      // sandboxed pause: app is backgrounded
App::on_resume(&mut self)
App::on_shutdown(&mut self)     // graceful termination, save state
```

`nova-sessiond` ([03-DESKTOP-ARCHITECTURE.md](03-DESKTOP-ARCHITECTURE.md) §6) drives this
state machine; apps never manage their own process lifecycle or daemonize. An app is
exactly one sandboxed process (or a small declared set for apps that need a helper
process, e.g. Nova Browser's renderer — an explicit, manifest-declared exception, not the
default).

## 3. Core SDK Modules

| Module | Provides |
|---|---|
| `nova-ui` | Widgets, layout, theming, input, accessibility tree — see [06-DESIGN-SYSTEM.md](06-DESIGN-SYSTEM.md) |
| `nova-app` | Entrypoint trait, window creation/management, event loop |
| `nova-storage` | Per-app sandboxed key-value + file storage, scoped by app ID, survives updates |
| `nova-notify` | Publish notifications via Nova Bus (permission-gated) |
| `nova-settings-api` | Read/subscribe to system + own-app settings |
| `nova-clipboard` | Clipboard read/write, drag-and-drop source/target |
| `nova-plugin` | Host and author extensions (see §6) |

Each module is a versioned crate published to an internal registry mirror; SDK-wide
semantic versioning rules are in
[11-CODING-STANDARDS.md](11-CODING-STANDARDS.md) §Versioning.

## 4. Manifest

Every app ships a manifest (packaged inside the `.novapkg`,
[ADR-0007](decisions/ADR-0007-package-format.md)):

```toml
[app]
id = "dev.novaos.files"
name = "Nova Files"
version = "1.0.0"
sdk_version = "^1.0"
icon = "icon.svg"

[permissions]
filesystem = ["home", "downloads"]
notifications = true
network = false
ipc_topics = []
```

The manifest is the single source of truth consumed by: Nova Package Center (display),
`nova-sessiond` (sandbox construction), the Launcher (indexing), and the Permission
Broker (grant enforcement). No permission is ever inferred from code — only from the
declared manifest, so the security review surface is the manifest, not the binary.

## 5. Windowing & Graphics

`nova-app` window creation talks Wayland directly (client-side), rendered through
`nova-ui`'s GPU 2D layer. Apps never touch wlroots or raw Wayland protocol objects —
those are compositor-internal ([ADR-0003](decisions/ADR-0003-compositor-and-display-protocol.md)).
A software-rendering fallback path exists for constrained targets (old GPUs, the browser
demo — [ADR-0009](decisions/ADR-0009-browser-boot-emulator.md)), selected automatically
at startup based on available GPU capability, transparent to app code.

## 6. Plugin / Extension System

Two distinct extensibility surfaces, kept separate because they have different trust
models:

1. **App plugins** — an app (e.g., Nova Notes, Nova Paint) may define its own narrow
   plugin API for its domain (e.g., a Paint filter plugin). Scoped entirely to that app;
   `nova-plugin` provides the common hosting mechanics (discovery, sandboxed execution)
   but the API surface is app-defined.
2. **System extensions** — theme packs, launcher search providers, file-type handlers.
   Registered via manifest declarations, invoked through Nova Bus, never given direct
   code-loading access to a Nova Service process.

Neither surface allows loading unsandboxed native code into a running Nova process — an
extension is either a sandboxed process of its own (like an app) or a data-only
contribution (like a theme). This keeps [ADR-0010](decisions/ADR-0010-app-sandboxing-model.md)'s
guarantees intact regardless of how many extensions are installed. A scripting host
(embedded Lua, evaluated at implementation time) is the sanctioned way to add
lightweight logic-bearing extensions without granting native-code trust.

## 7. Backward Compatibility & Versioning

- SDK follows semver; a major version bump is the only thing allowed to break an app
  manifest's `sdk_version` compatibility.
- `nova-sessiond` refuses to launch an app whose declared `sdk_version` range doesn't
  match the running SDK — a clear error, never a silent crash.
- Deprecations are announced at least one minor version ahead, tracked in the SDK
  crate's `CHANGELOG.md` (see [11-CODING-STANDARDS.md](11-CODING-STANDARDS.md)).

## 8. Developer Experience

- `nova-cli new <app-name>` scaffolds a manifest + minimal `App` impl (Phase 5 tooling,
  see [12-ROADMAP-AND-MILESTONES.md](12-ROADMAP-AND-MILESTONES.md)).
- Apps run and hot-reload inside a dev-mode sandbox on the developer's own NovaOS VM —
  no separate "simulator," since the target *is* a VM-friendly OS
  ([00-VISION.md](00-VISION.md) §4).
- SDK documentation is generated from doc-comments (`cargo doc`-equivalent) and published
  alongside novaos.dev ([07-BROWSER-DEPLOYMENT.md](07-BROWSER-DEPLOYMENT.md)).
