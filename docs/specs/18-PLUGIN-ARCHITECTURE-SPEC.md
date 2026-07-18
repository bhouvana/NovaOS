# Spec 18 — Plugin Architecture Specification

Status: Draft v0.1 · Last updated: 2026-07-18

Expands [../04-APPLICATION-FRAMEWORK-AND-SDK.md](../04-APPLICATION-FRAMEWORK-AND-SDK.md)
§6 and [06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md) §9 into a full spec, per Staff Engineer
review's "decide now, not later" flag.

## 1. Scope Recap

Two distinct extensibility surfaces, kept separate because they have different trust
models (unchanged from Phase 1, restated here as the anchor for the rest of this spec):

1. **App plugins** — sandboxed process, app-defined API, this document's primary focus.
2. **System extensions** (themes, search providers) — data-only, no code execution,
   already fully specified ([10-DESIGN-BIBLE.md](10-DESIGN-BIBLE.md) for themes) — not
   revisited here.

## 2. Discovery

A host app declares a plugin directory convention in its own manifest-adjacent config
(not the app manifest itself — plugins are a host-app-specific concept, not something
`nova-sessiond` needs to know about at the system level):

```text
/nova/data/<host_app_id>/plugins/<plugin_id>/
  plugin.toml       # PluginManifest, see §3
  bin/<plugin_id>    # plugin binary
```

`PluginHost::discover()` ([06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md) §9) scans this
directory at host-app startup and on-demand (e.g., Nova Paint's "Manage Filters" UI
triggers a re-scan) — no background filesystem watcher, no daemon; discovery is always
triggered by the host app's own lifecycle events, consistent with the no-unnecessary-
background-service principle.

## 3. Plugin Manifest

```toml
# plugin.toml
[plugin]
id = "com.example.paint-filter.sepia"
name = "Sepia Filter"
version = "1.0.0"
host_app_id = "dev.novaos.paint"        # must match the installing host — a plugin
                                          # built for Nova Paint cannot silently load
                                          # into Nova Notes
plugin_api_version = "^1.0"              # versioned against the HOST APP's plugin
                                          # trait version, not the Nova SDK version
                                          # (§6) — these are independent numbers

[permissions]
# subset of the host app's own granted permissions — see §5
filesystem = []
network = false
```

## 4. Loading

```text
Host app calls PluginHost::load(&manifest)
   ↓
Validate: plugin.host_app_id matches the calling host app's own app_id
   (a plugin cannot be loaded by any app other than the one it declares)
   ↓
Validate: plugin_api_version range satisfied by the host's declared
   trait version (§6)
   ↓
nova-sessiond constructs a NESTED sandbox for the plugin process — see
   §5 — and execve()s bin/<plugin_id>
   ↓
Plugin process connects to a HOST-APP-SCOPED Nova Bus channel (not the
   host app's own bus connection — a distinct connection with its own
   ACL, so a compromised plugin cannot make Nova Bus calls the plugin
   API surface doesn't cover, even ones the host app itself could make)
   ↓
Host app's PluginHandle<P> becomes a typed proxy for calling into the
   plugin's app-defined Plugin trait methods over that channel
```

A plugin is always a separate OS process — there is no in-process/dynamically-loaded-
library plugin model in NovaOS, which would violate
[ADR-0010](../decisions/ADR-0010-app-sandboxing-model.md)'s "no unsandboxed native code
loaded into a running Nova process" rule from
[../04-APPLICATION-FRAMEWORK-AND-SDK.md](../04-APPLICATION-FRAMEWORK-AND-SDK.md) §6,
restated here as the single non-negotiable constraint this entire spec is designed
around.

## 5. Permissions & Sandboxing

- A plugin's sandbox is constructed by `nova-sessiond` using the same primitives as an
  ordinary app's sandbox ([ADR-0010](../decisions/ADR-0010-app-sandboxing-model.md)),
  but **nested**: the plugin's effective permission set is the intersection of its own
  declared `[permissions]` (§3) and its host app's granted permissions — a plugin can
  never gain a capability its host app doesn't already have, regardless of what its own
  manifest claims. This is enforced at sandbox-construction time (the same stage that
  enforces the ordinary app manifest-to-sandbox-rule mapping,
  [../08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md) §2), not by trusting the plugin.
- Plugin process lifetime is tied to its host app's lifetime — `nova-sessiond` terminates
  all of a host app's loaded plugins when the host app itself exits or crashes (no
  orphaned plugin processes).
- Resource accounting: a plugin's cgroup is nested under its host app's cgroup slice, so
  Nova Monitor can show "Nova Paint (+2 plugins)" as a rolled-up figure as well as
  per-plugin detail.

## 6. API Compatibility & Version Negotiation

- Each host app defines its own `Plugin` trait (§1) and assigns it a semver version
  independent of the Nova SDK's own version (§17-SDK-API-REFERENCE-POLICY doesn't apply
  to app-defined plugin traits — that policy governs the Nova SDK itself, not every
  app's own extension surface).
- `PluginHost::load` performs the same class of range check as
  `nova-sessiond` does for `sdk_version`
  ([06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md) §10) but scoped to `plugin_api_version`
  against the host's currently-running trait version — a plugin built against an
  incompatible version fails to load with a clear error, never a silent
  method-not-found failure at call time.
- No automatic plugin update mechanism in v1 — a plugin is installed the same way an
  app is conceptually (files placed under the host's plugin directory, §2), but there is
  no v1 requirement for Nova Store to distribute plugins through the same signed-catalog
  flow as apps ([RFC-0004](../rfcs/RFC-0004-package-service.md)); this is an explicit
  [../14-FUTURE-VISION.md](../14-FUTURE-VISION.md)-class deferral — v1 plugins are a
  power-user/developer feature (manually placed files), not a mainstream distribution
  channel.

## 7. What This Spec Deliberately Does Not Cover

- A general-purpose, host-app-agnostic plugin marketplace — out of v1 scope per §6.
- Cross-plugin communication (one plugin calling another) — not supported; a plugin only
  ever talks to its host app, never to sibling plugins or other apps' plugins.
- Hot-reload of a running plugin without restarting it — `PluginHost::load` is a
  point-in-time operation; picking up a changed plugin binary requires unload + reload,
  which is acceptable given plugins are expected to be small, fast-starting processes
  (matching the app-launch performance budget in
  [../09-PERFORMANCE-STRATEGY.md](../09-PERFORMANCE-STRATEGY.md) §2, scaled down for a
  typically-smaller plugin binary).
