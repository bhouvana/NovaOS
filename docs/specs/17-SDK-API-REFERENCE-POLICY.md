# Spec 17 — SDK Public API Reference Policy

Status: Draft v0.1 · Last updated: 2026-07-18

Treats the Nova SDK as a public product with its own stability contract, per Staff
Engineer review — extends
[../04-APPLICATION-FRAMEWORK-AND-SDK.md](../04-APPLICATION-FRAMEWORK-AND-SDK.md) §7 and
[../11-CODING-STANDARDS.md](../11-CODING-STANDARDS.md) §10 beyond "we follow semver"
into the concrete policy a third-party app author can rely on.

## 1. Stability Tiers

Every public SDK item (struct, trait, function) carries one of three stability
markers, expressed as a doc-comment annotation checked by CI
([../10-TESTING-AND-BUILD.md](../10-TESTING-AND-BUILD.md) §2 stage 3's contract tests):

| Tier | Meaning | Breaking-change policy |
|---|---|---|
| `#[stable]` | Safe to build on indefinitely | Only ever broken in a new SDK major version, with a deprecation cycle first (§2) |
| `#[experimental]` | Working, but the shape may still change | May break in a minor version bump, always called out in release notes |
| `#[internal]` | Not part of the public SDK surface despite being technically reachable | No stability guarantee at all; using it is unsupported |

At SDK `1.0.0` ([../12-ROADMAP-AND-MILESTONES.md](../12-ROADMAP-AND-MILESTONES.md) §6 exit
criteria), every module listed in
[06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md) §1–§9 is `#[stable]` except `nova-plugin`
(§5 of that doc), which ships `#[experimental]` for its first release since the plugin
model is expected to need at least one iteration once real plugins exist.

## 2. Deprecation Policy

1. A `#[stable]` item marked for removal gets `#[deprecated(since = "X.Y", note =
   "...")]` — compiler warning on use, item still fully functional.
2. Deprecation must ship at least **one minor version** before removal, and the release
   notes for the deprecating version must include a migration note (what to use
   instead, not just "this is going away").
3. Removal only happens at a **major version** bump ([../11-CODING-STANDARDS.md](../11-CODING-STANDARDS.md)
   §10) — never in a minor/patch release, matching
   [06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md) §10's `sdk_version` range-check guarantee:
   an app pinned to `^1.0` is never broken by anything short of a `2.0` release, and even
   then it saw the deprecation warning at least one `1.x` release earlier if it was
   rebuilt.

## 3. Semantic Versioning Applied

Restates and sharpens [../11-CODING-STANDARDS.md](../11-CODING-STANDARDS.md) §10 with
concrete examples:

| Change | Version bump |
|---|---|
| New `#[stable]` method added to an existing trait | Minor |
| New optional manifest field | Minor |
| New `Call`/`Publish` Nova Bus topic | Minor |
| Existing `#[stable]` method signature changed | Major |
| Existing `#[stable]` item removed | Major (after deprecation, §2) |
| Bug fix, no public signature change | Patch |
| `#[experimental]` item's shape changed | Minor, called out in release notes (§1) |
| `#[internal]` item changed | Not versioned at all — not part of the contract |

## 4. Extension Points

The SDK's explicit points of controlled extensibility (as opposed to accidental surface
that happens to be reachable):

- `sdk/nova-plugin`'s `Plugin` trait, defined per-app
  ([06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md) §9, full model in
  [18-PLUGIN-ARCHITECTURE-SPEC.md](18-PLUGIN-ARCHITECTURE-SPEC.md)).
- App manifest's `[settings]` schema
  ([06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md) §7) — a declarative extension point, not
  code.
- Theme token files ([10-DESIGN-BIBLE.md](10-DESIGN-BIBLE.md), `nova-themed`'s schema
  validator) — also declarative.

Everything else in the SDK is a fixed API surface, not designed for subclassing/
overriding — `Widget` implementations (§05-NOVA-UI-TOOLKIT-SPEC §2) are the one
exception, being an open trait any app can implement for custom widgets, but doing so
opts out of the shared design-system guarantee for that specific widget
([../06-DESIGN-SYSTEM.md](../06-DESIGN-SYSTEM.md) §1) and is expected to be rare (used by
Nova Paint/Arcade's `Canvas` node, [09-APPLICATION-SPECS.md](09-APPLICATION-SPECS.md)),
not a general app-building pattern.

## 5. Error Handling Conventions

- Every fallible SDK call returns `Result<T, NovaError>`. `NovaError` is a single,
  SDK-wide enum (not a per-module error type) so app code doesn't need N different
  error-handling patterns depending on which SDK module it's calling:

```rust
pub enum NovaError {
    PermissionDenied { permission: String },
    Timeout,
    InvalidArgument(String),
    NotFound,
    Internal(String),   // opaque — see below
}
```

- `NovaError` variants map directly onto the Nova Bus `ErrorCode` enum
  ([15-NOVA-BUS-PROTOCOL-SPEC.md](15-NOVA-BUS-PROTOCOL-SPEC.md) §11) — the SDK does not
  invent a second error taxonomy layered on top of the protocol's, it's a thin typed
  wrapper.
- `NovaError::Internal` is deliberately opaque (a string, not a structured variant) —
  it represents "something failed inside a Nova service and the detail isn't part of the
  public contract" (mirrors [15-NOVA-BUS-PROTOCOL-SPEC.md](15-NOVA-BUS-PROTOCOL-SPEC.md)
  §11's `INTERNAL` code being "opaque to the broker"). App code should never match on
  the string content of an `Internal` error to drive logic — only on the other, stable
  variants.
- Every `#[stable]` fallible function's doc-comment enumerates which `NovaError`
  variants it can actually return (not just "returns `Result`") — checked by the same
  `#![warn(missing_docs)]` discipline as
  [../11-CODING-STANDARDS.md](../11-CODING-STANDARDS.md) §7.

## 6. Examples Requirement

Every `#[stable]` SDK module's doc-comments include at least one compiled, tested
example (Rust doc-tests, run as part of
[../10-TESTING-AND-BUILD.md](../10-TESTING-AND-BUILD.md) §2 stage 2) — an example that
doesn't compile is a CI failure, not just a documentation nicety, so published examples
can never silently rot out of sync with the real API (the failure mode
[../04-APPLICATION-FRAMEWORK-AND-SDK.md](../04-APPLICATION-FRAMEWORK-AND-SDK.md) §8's
"tests as documentation" principle exists specifically to prevent).

## 7. Where This Is Published

The generated SDK reference site
([13-WEBSITE-INFORMATION-ARCHITECTURE.md](13-WEBSITE-INFORMATION-ARCHITECTURE.md) §1
`/docs`) renders the stability tier (§1) and deprecation status (§2) inline on every
item's page — a visitor never has to guess whether something is safe to depend on.
