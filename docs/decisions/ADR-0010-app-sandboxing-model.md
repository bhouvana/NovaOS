# ADR-0010: Application Sandboxing Model

Status: Accepted
Date: 2026-07-18
Deciders: Chief Architect

## Context

Nova apps come from a curated store but must still be sandboxed: NovaOS should not depend
on every app author being trustworthy or bug-free, and the permission model referenced
throughout ([08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md)) needs an enforcement
mechanism at the OS level, not just at the IPC broker.

## Options Considered

1. **No sandboxing, trust the store's review process** — simplest, but a single review
   miss or supply-chain compromise becomes full-system compromise; unacceptable given
   [00-VISION.md](../00-VISION.md) §2's security-by-default expectations.
2. **Full container runtime per app (e.g., a general container engine)** — strong
   isolation, but a heavyweight, general-purpose container stack is disproportionate
   machinery (and RAM/startup-time cost) for single-user desktop apps, and duplicates
   functionality we can get more cheaply.
3. **Direct Linux primitives (namespaces + seccomp-bpf + Landlock/cgroups), applied per
   app process at launch by the Nova Session Manager, no separate daemon** — the same
   underlying primitives containers use, applied directly and minimally: a new mount/PID/
   user namespace, a seccomp filter restricting syscalls, Landlock for filesystem-path
   scoping, driven entirely by the app manifest's declared permissions
   ([ADR-0007](ADR-0007-package-format.md)).

## Decision

**Direct Linux sandboxing primitives** (namespaces, seccomp-bpf, Landlock, cgroups-v2 for
resource limits), applied by Nova Session Manager at app launch time based on the app's
manifest-declared permissions, with no general container runtime/daemon in the stack.

## Rationale

This gets the real isolation properties (filesystem scoping, syscall restriction, resource
limits) without adopting a general container engine's abstraction layer, image format, or
daemon — directly serving the "no unnecessary daemon" and RAM-budget principles while
still meeting the security bar. It also keeps enforcement co-located with Nova Session
Manager, which already owns app lifecycle (see
[04-APPLICATION-FRAMEWORK-AND-SDK.md](../04-APPLICATION-FRAMEWORK-AND-SDK.md)), rather
than introducing a second process-management authority.

## Consequences

- Every declared permission in an app manifest (filesystem paths, network, camera/mic if
  ever supported, IPC topics via Nova Bus) must map to a concrete namespace/seccomp/
  Landlock rule — this mapping is a security-critical, carefully tested piece of code
  (see [08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md) for the permission taxonomy).
- Apps cannot assume a full traditional Linux filesystem view; Nova UI/SDK file pickers
  are the sanctioned way for an app to gain access to a user-chosen file outside its
  granted paths (broker-mediated, not a raw path grant).
- Debugging a misbehaving sandboxed app requires sandbox-aware tooling in Nova Terminal/
  Monitor — tracked as an SDK/devtools requirement, not an afterthought.

## Revisit Triggers

- If manifest-to-sandbox-rule mapping complexity grows unmanageable as permission types
  expand, consider extracting it into a dedicated policy-compiler component rather than
  inline Session Manager logic.
