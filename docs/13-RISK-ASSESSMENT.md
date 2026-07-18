# NovaOS — Risk Assessment

Status: Draft v0.1 · Owner: Chief Architect · Last updated: 2026-07-18

Each risk is scored Impact × Likelihood (High/Medium/Low) at current understanding, with
a concrete mitigation and an owning doc where the mitigation is architecturally encoded.

## 1. Technical Risks

| Risk | Impact | Likelihood | Mitigation |
|---|---|---|---|
| Building a custom compositor (wlroots-based) takes materially longer than planned | High | Medium | Scoped tightly in Phase 2/3 as the critical path; wlroots absorbs the hardest 80% (DRM/KMS, input) — see [ADR-0003](decisions/ADR-0003-compositor-and-display-protocol.md) |
| Building a custom UI toolkit (Nova UI) from scratch is a large undertaking that could stall the whole app layer | High | Medium | Explicit revisit trigger in [ADR-0005](decisions/ADR-0005-ui-toolkit.md): if it blocks Phase 3/4 by more than one milestone, fall back to a GTK4 escape hatch for select apps |
| Idle RAM budget (64–100 MB) proves unreachable once real services/apps are running | High | Medium | Budget is enforced continuously via CI regression gate, not checked only at release ([09-PERFORMANCE-STRATEGY.md](09-PERFORMANCE-STRATEGY.md) §4), so drift is caught early and cheaply, not late and expensively |
| v86-based browser boot is too slow or too limited (no GPU accel) to be a compelling demo | Medium | Medium | Browser-demo image is explicitly allowed to be a reduced variant optimized for this constraint ([07-BROWSER-DEPLOYMENT.md](07-BROWSER-DEPLOYMENT.md) §3); software-rendering fallback is a shared investment that also benefits low-end real hardware |
| musl libc (via Alpine base) causes compatibility friction with some hardware/driver firmware tooling | Medium | Low | NovaOS's own stack is built from source against musl; explicit ADR revisit trigger if friction becomes disproportionate ([ADR-0001](decisions/ADR-0001-linux-base-distribution.md)) |
| Sandboxing (namespaces/seccomp/Landlock) manifest-to-rule mapping has a security-relevant bug | High | Low–Medium | Dedicated CI security-lint stage tests every in-tree manifest against the mapping ([10-TESTING-AND-BUILD.md](10-TESTING-AND-BUILD.md) §2 stage 7); formal verification tracked as future hardening ([08-SECURITY-MODEL.md](08-SECURITY-MODEL.md) §9) |
| A/B update mechanism has a bug that bricks devices | High | Low | Automatic rollback-on-failed-health-check is architecturally mandatory, not optional ([ADR-0008](decisions/ADR-0008-filesystem-and-update-strategy.md)); integration tests boot-test every release image before publishing |

## 2. Scope & Execution Risks

| Risk | Impact | Likelihood | Mitigation |
|---|---|---|---|
| Scope creep — attempting all 40+ subsystems listed in the original brief simultaneously | High | High | Phased roadmap ([12-ROADMAP-AND-MILESTONES.md](12-ROADMAP-AND-MILESTONES.md)) with hard exit criteria per phase; no phase N+1 work starts before phase N's exit criteria are met |
| Small team, one-language stack (Rust) — bus factor / hiring pool narrower than a multi-language project | Medium | Medium | Deliberate, documented tradeoff ([ADR-0004](decisions/ADR-0004-systems-language.md) Consequences); mitigated long-term by strong docs/onboarding (this tree) and a plugin/scripting surface that doesn't require Rust for extension authors |
| SDK designed without real external users, ships an API that doesn't fit real app needs | Medium | Medium | Phase 5 exit criteria explicitly requires a real external-style app built against public docs alone before declaring the SDK done ([12-ROADMAP-AND-MILESTONES.md](12-ROADMAP-AND-MILESTONES.md) §6) |
| Documentation drifts from implementation as code changes accelerate | Medium | Medium | Code review checklist requires doc updates in the same PR as architecture-relevant changes ([11-CODING-STANDARDS.md](11-CODING-STANDARDS.md) §7, §9) |

## 3. Ecosystem & Adoption Risks

| Risk | Impact | Likelihood | Mitigation |
|---|---|---|---|
| No third-party apps materialize even after SDK ships | Medium | Medium | Not a v1 blocker — success criteria ([00-VISION.md](00-VISION.md) §5) requires the SDK to be usable and documented, not that an ecosystem has already formed; tracked in [14-FUTURE-VISION.md](14-FUTURE-VISION.md) |
| Hardware driver coverage gaps on real devices (Wi-Fi, GPU) since we rely on upstream Linux only | Medium | Medium | Explicit non-goal to fork/extend the kernel ([00-VISION.md](00-VISION.md) §1); reference hardware list kept deliberately modest and well-supported for v1, broadened over time |
| Licensing/compliance issues from a dependency (e.g., a component with an incompatible license) | Medium | Low | Every new external dependency requires a one-line license justification in review ([11-CODING-STANDARDS.md](11-CODING-STANDARDS.md) §3); copyleft-incompatible deps require an ADR |

## 4. Risks Explicitly Accepted, Not Mitigated

- **2x disk usage from A/B partitioning** — accepted for update-safety ([ADR-0008](decisions/ADR-0008-filesystem-and-update-strategy.md)).
- **No native Linux desktop app compatibility (GTK/Qt apps don't just run)** — accepted for design-system cohesion ([ADR-0005](decisions/ADR-0005-ui-toolkit.md), [ADR-0007](decisions/ADR-0007-package-format.md)).
- **Single-primary-user model in v1** — accepted; full multi-user is post-v1 ([08-SECURITY-MODEL.md](08-SECURITY-MODEL.md) §6).

## 5. Review Cadence

This assessment is reviewed at every phase gate ([12-ROADMAP-AND-MILESTONES.md](12-ROADMAP-AND-MILESTONES.md)
§8) — risks are re-scored, mitigations checked for whether they actually fired, and new
risks discovered during the phase are added rather than only being written once at
project start.
