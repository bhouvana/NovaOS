# Spec 07 — `.novapkg` Package Format Specification

Status: Draft v0.1 · Last updated: 2026-07-18

Concretizes [ADR-0007](../decisions/ADR-0007-package-format.md) into a byte-level
layout.

## 1. File Layout

A `.novapkg` file is three concatenated regions:

```text
Offset 0
┌─────────────────────────────────────────────┐
│ HEADER (fixed 8192 bytes = 2 × 4KB pages)    │
├─────────────────────────────────────────────┤
│ SQUASHFS PAYLOAD (variable length,           │
│   4KB-aligned, length given in header)       │
├─────────────────────────────────────────────┤
│ SIGNATURE FOOTER (fixed 128 bytes)           │
└─────────────────────────────────────────────┘
```

Rationale for this three-part layout: `novapkg-agent` can read the fixed-size header
and footer without mounting or even fully downloading the SquashFS payload — enabling
the fast-reject checksum/signature checks in
[01-INTERACTION-FLOWS.md](01-INTERACTION-FLOWS.md) §5 before the (potentially large)
payload is fully committed to disk, and enabling Package Center to show manifest
metadata (name, version, permissions) for a catalog listing by fetching only the header
via an HTTP range request, not the whole file. The header is sized to comfortably inline
a full manifest (§3's `manifest.toml` is expected to be well under 4KB) while staying
4KB-page-aligned so the SquashFS payload that follows starts on a clean page boundary.

## 2. Header (8192 bytes, little-endian)

| Offset | Size | Field | Description |
|---|---|---|---|
| 0 | 8 | `magic` | ASCII `"NOVAPKG1"` — format identifier + version |
| 8 | 4 | `format_version` | `u32`, currently `1` — distinct from `magic`'s version digit to allow minor header revisions without changing the magic string |
| 12 | 4 | `header_len` | `u32`, always `8192` in v1 — future-proofs a longer header without breaking readers that only need fields they know |
| 16 | 8 | `manifest_offset` | `u64`, byte offset of `manifest.toml` *within the SquashFS payload* — duplicated as `manifest_inline_len`/`manifest_inline` below so it's readable pre-mount |
| 24 | 4 | `manifest_inline_len` | `u32`, length of the inlined manifest copy (≤ 4096 bytes) |
| 28 | 4096 | `manifest_inline` | Verbatim copy of `manifest.toml`'s bytes, UTF-8, zero-padded — lets Package Center read app metadata without mounting SquashFS; the *authoritative* copy used at runtime is still the one inside the mounted SquashFS at `manifest_offset`, and `novapkg-agent` verifies the two are byte-identical at install time (a mismatch is a corrupt/tampered package, rejected) |
| 4124 | 8 | `payload_offset` | `u64`, always `8192` (== `header_len`, immediately after header) in v1 |
| 4132 | 8 | `payload_len` | `u64`, byte length of the SquashFS region |
| 4140 | 32 | `payload_sha256` | SHA-256 of the SquashFS payload bytes — checked before signature verification (§4, cheap corruption check) |
| 4172 | 4 | `sig_algorithm` | `u32` enum, `1 = Ed25519` in v1 |
| 4176 | 16 | `signer_key_id` | First 16 bytes of the signer's Ed25519 public key's SHA-256 — a reference into the trusted keyring, not the key itself (§4) |
| 4192 | 4000 | reserved | Zero-filled, reserved for future fields, pads the header to the 8192-byte page-aligned total |

The header struct is `#[repr(C)]` with a build-time assertion that
`size_of::<Header>() == header_len`, keeping this table and the code from silently
drifting.

## 3. SquashFS Payload — Internal Layout

```text
/manifest.toml           authoritative manifest (06-NOVA-SDK-SPEC §3)
/assets/
  icon.svg
  i18n/<locale>.ftl       (06-NOVA-SDK-SPEC §6)
  settings-schema.toml    (06-NOVA-SDK-SPEC §7, optional)
/bin/<app_id_last_segment>   the app's compiled binary, e.g. /bin/files for dev.novaos.files
/lib/                      app-private libraries, rare (06-NOVA-SDK-SPEC exception apps
                            like Nova Browser's embedded engine)
```

Mounted read-only at `/nova/apps/<id>/<version>/` per
[01-INTERACTION-FLOWS.md](01-INTERACTION-FLOWS.md) §5; `nova-sessiond` resolves
`/bin/<binary>` at launch using the manifest's `app.id` last DNS segment as a fixed
convention (no separate "entrypoint" manifest field needed for the common case; an
explicit `entrypoint` override field is reserved but unused in v1).

## 4. Signature Footer (128 bytes)

| Offset (from footer start) | Size | Field | Description |
|---|---|---|---|
| 0 | 8 | `magic` | ASCII `"NOVASIG1"` |
| 8 | 64 | `signature` | Ed25519 signature over `header_bytes \|\| payload_bytes` (the entire file except this footer) |
| 72 | 56 | reserved | Zero-filled |

**Verification procedure** (`novapkg-agent`, matching
[01-INTERACTION-FLOWS.md](01-INTERACTION-FLOWS.md) §5):

1. Read header (256 bytes) — validate `magic` and `format_version`.
2. Stream-hash the payload region as it downloads; compare to `payload_sha256` —
   reject on mismatch without needing the signature check (cheap, catches truncated/
   corrupted downloads immediately).
3. Look up `signer_key_id` in `/nova/config/trusted-keys/` (populated at image build
   time from [../08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md) §4's trust anchors,
   updatable only via the signed OS update channel).
4. Verify `signature` against `header_bytes || payload_bytes` using the resolved public
   key — reject if the key isn't in the trusted keyring or the signature doesn't
   verify.
5. Only after all four checks pass: mount the SquashFS payload.

## 5. Version & Dependency Fields (recap from manifest, §06-NOVA-SDK-SPEC §3)

- `app.version`: publisher-controlled string, must be monotonically increasing per
  [../11-CODING-STANDARDS.md](../11-CODING-STANDARDS.md) §10 — `novapkg-agent` compares
  using semver ordering when the string parses as semver, else falls back to a
  publish-timestamp field from the catalog entry (not the package itself) to break ties.
- `app.sdk_version`: a semver *range* (e.g. `^1.0`), checked at launch time
  ([06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md) §10), not at install time — a package may
  be installed ahead of an SDK upgrade that will satisfy it.
- No separate app-to-app dependency graph in v1 (an app depends only on the SDK version
  installed with the OS, never on another app being present) — keeps install order
  irrelevant and avoids a dependency-resolution algorithm entirely, consistent with
  [../00-VISION.md](../00-VISION.md) §6's simplicity priority.

## 6. Compression

SquashFS payload uses `zstd` at a level tuned for fast decompression over maximal ratio
(per [../09-PERFORMANCE-STRATEGY.md](../09-PERFORMANCE-STRATEGY.md) §7), specifically
`zstd -3` as the v1 default — revisited per-release if the size/speed tradeoff needs
adjusting, tracked as a build-tooling config value, not a format constant (changing
compression level doesn't change anything in §1–§4's layout).

## 7. Build Tooling

`novapkg-builder` (part of `tools/`, [../02-REPOSITORY-STRUCTURE.md](../02-REPOSITORY-STRUCTURE.md))
takes an app's build output directory + `manifest.toml`, produces the SquashFS payload,
computes `payload_sha256`, writes the header (including the inlined manifest copy,
verified byte-identical to the SquashFS copy), and — for release builds — signs it using
a publisher key held outside the repository (CI secret, never committed); local
development builds use the `--allow-unsigned` path noted in
[../05-PACKAGE-AND-UPDATE-SYSTEM.md](../05-PACKAGE-AND-UPDATE-SYSTEM.md) §4.
