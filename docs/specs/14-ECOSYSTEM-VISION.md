# Spec 14 — Ecosystem Vision

Status: Draft v0.1 · Last updated: 2026-07-18

Expands [../14-FUTURE-VISION.md](../14-FUTURE-VISION.md) into the explicit platform
chain the Staff Engineer review asked for, while reconciling each link against v1's
non-goals — this is where "design for the possibility" and "don't cloud-account-gate
the OS" ([../00-VISION.md](../00-VISION.md) §7) get checked against each other
explicitly, link by link, rather than left as a tension to notice later.

## 1. The Chain

```text
Nova SDK       → Nova Store → Nova Cloud → Nova Sync → Nova Browser Demo
   → Nova Docs → Nova Community → Nova Packages
```

## 2. Per-Link Assessment

| Link | What it is | v1 status | Already accommodated by | New work required |
|---|---|---|---|---|
| **Nova SDK** | The app platform | Shipping in v1 | [06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md) | — (this is v1 core) |
| **Nova Store** | Signed catalog + distribution | Shipping in v1 | [../05-PACKAGE-AND-UPDATE-SYSTEM.md](../05-PACKAGE-AND-UPDATE-SYSTEM.md), [07-PACKAGE-FORMAT-SPEC.md](07-PACKAGE-FORMAT-SPEC.md) | — (this is v1 core) |
| **Nova Cloud** | Optional backend: account, storage quota | Post-v1 | Nothing in v1 *requires* it — `nova-storage` is local-first ([06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md) §4) | An entire backend service, an account system, and a client-side opt-in flow — the single largest net-new build in this chain |
| **Nova Sync** | Settings/file sync across devices, built on Nova Cloud | Post-v1, depends on Nova Cloud | `nova-settings-api`'s key-value model ([06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md) §5) is shaped so a future sync layer could subscribe to changes without a settings-API redesign | Conflict resolution, sync protocol, and — critically — must remain fully optional per §3 |
| **Nova Browser Demo** | Marketing/onboarding, novaos.dev | Shipping in v1 (Phase 6) | [08-BROWSER-ARCHITECTURE-SPEC.md](08-BROWSER-ARCHITECTURE-SPEC.md), [12-BROWSER-DEMO-EXPERIENCE.md](12-BROWSER-DEMO-EXPERIENCE.md) | — (this is v1 core) |
| **Nova Docs** | SDK reference + architecture site | Shipping in v1 (Phase 6) | [13-WEBSITE-INFORMATION-ARCHITECTURE.md](13-WEBSITE-INFORMATION-ARCHITECTURE.md) §4 | — (this is v1 core) |
| **Nova Community** | External contribution, forums/discussion | Post-v1 | [../11-CODING-STANDARDS.md](../11-CODING-STANDARDS.md) (contribution standards already exist, just not yet opened externally) | A moderation/discussion venue decision (GitHub Discussions vs. a dedicated forum) — deliberately not decided yet, low cost to defer |
| **Nova Packages** (third-party ecosystem) | External publishers on Nova Store | Post-v1 | Publisher key registration model sketched in [../05-PACKAGE-AND-UPDATE-SYSTEM.md](../05-PACKAGE-AND-UPDATE-SYSTEM.md) §4 | Publisher onboarding flow, review policy, abuse handling — real product/policy work, not architecture |

## 3. The Hard Constraint: Optionality

Every link right of "Nova Store" in §1 touches the one non-goal most likely to be
violated by ecosystem ambition creeping into the core: **NovaOS must never require a
cloud account to be fully functional**
([../00-VISION.md](../00-VISION.md) §7,
[../08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md) §6's local-account-first model).
Concretely, this means, as binding constraints on any future Nova Cloud/Sync design
(not just aspirational language):

1. `nova-storage`'s `FileStore`/`KvStore` APIs ([06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md)
   §4) never change signature to require a cloud identity — sync, if built, is an
   additive background replicator observing the same local-first store, not a
   replacement storage backend apps must opt into.
2. Package install/update ([../05-PACKAGE-AND-UPDATE-SYSTEM.md](../05-PACKAGE-AND-UPDATE-SYSTEM.md))
   never requires a Nova Cloud account — Nova Store's trust model
   ([../08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md) §4) is signature-based, not
   account-based, precisely so this stays true even after Nova Cloud exists for those
   who opt in to it for other reasons.
3. Nova Settings' account section (§06-NOVA-SDK-SPEC, [09-APPLICATION-SPECS.md](09-APPLICATION-SPECS.md)
   "Nova Settings") treats "Sign in to Nova Cloud" as one settings entry among many, off
   by default, never a first-run gate.

## 4. Why Document This Now Instead of Waiting

Per [../14-FUTURE-VISION.md](../14-FUTURE-VISION.md) §4, nothing here authorizes work —
but recording the chain and its constraints now means Phase 2–6 implementation of the v1
core (SDK, Store, storage APIs) can be checked against "does this design choice foreclose
or complicate the optional-cloud future" as a lightweight, ongoing review question,
rather than discovering after the fact that a v1 API shape assumed a single-device,
account-free world so completely that adding optional sync later requires a breaking
change. The three constraints in §3 are the concrete artifact of that check.

## 5. Explicit Sequencing

If pursued at all, this chain is pursued strictly after v1 ships
([../12-ROADMAP-AND-MILESTONES.md](../12-ROADMAP-AND-MILESTONES.md) Phase 6 exit), and
in the order listed in §1 — Nova Cloud before Nova Sync (Sync has no backend without
it), Nova Community before Nova Packages (a third-party publisher ecosystem needs a
place for publishers to show up and ask questions before it needs a review pipeline).
This is a sequencing note, not a commitment to build any of it.
