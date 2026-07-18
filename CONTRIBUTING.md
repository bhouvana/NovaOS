# Contributing to NovaOS

NovaOS is currently in Phase 2 (Vertical Slice) —
see [docs/12-ROADMAP-AND-MILESTONES.md](docs/12-ROADMAP-AND-MILESTONES.md). The project
is not yet seeking broad external contribution (there isn't enough working code yet to
onboard around), but the process below is real and in effect for anyone working on it,
core team included.

## Before You Start

1. Read [docs/00-VISION.md](docs/00-VISION.md) and
   [docs/ENGINEERING-PRINCIPLES.md](docs/ENGINEERING-PRINCIPLES.md) — every review
   checks against these.
2. Read [docs/02-REPOSITORY-STRUCTURE.md](docs/02-REPOSITORY-STRUCTURE.md) to find
   where your change belongs.
3. Check [docs/decisions/](docs/decisions/) and [docs/rfcs/](docs/rfcs/) — your change
   may already be governed by an existing ADR or RFC.

## Do I Need an RFC or ADR First?

See [docs/rfcs/README.md](docs/rfcs/README.md) §"When an RFC Is Required" and
[docs/decisions/README.md](docs/decisions/README.md) §"When to write one." Short
version: a new subsystem, a protocol change, or a breaking SDK/format change needs one
written and merged *before* the implementing code. An ordinary feature, bug fix, or
internal refactor inside an already-specified subsystem does not.

## Development Setup

```sh
git clone <repo-url> && cd NovaOS
cargo build --workspace   # requires protoc on PATH, or PROTOC=<path to protoc>
cargo test --workspace
```

Full build/CI pipeline detail: [docs/10-TESTING-AND-BUILD.md](docs/10-TESTING-AND-BUILD.md).
Some Phase 2 exit criteria (compositor, boot, browser demo) require a Linux graphics
toolchain (wlroots, QEMU) not needed for Nova Bus/SDK/app-level work — see
[docs/12-ROADMAP-AND-MILESTONES.md](docs/12-ROADMAP-AND-MILESTONES.md) §4's Environment
note.

## Code Review Checklist

Reused verbatim from [docs/11-CODING-STANDARDS.md](docs/11-CODING-STANDARDS.md) §9 —
every PR is checked against it, not just described by it.

## Commit / PR Conventions

- Conventional-commit-style PR titles (`feat:`, `fix:`, `perf:`, `docs:`, `refactor:`) —
  feeds automated changelog generation
  ([docs/specs/22-RELEASE-PROCESS-SPEC.md](docs/specs/22-RELEASE-PROCESS-SPEC.md) §4).
- One PR, one logical change. Linear history, rebase not merge
  ([docs/11-CODING-STANDARDS.md](docs/11-CODING-STANDARDS.md) §11).
- Use the PR template ([.github/PULL_REQUEST_TEMPLATE.md](.github/PULL_REQUEST_TEMPLATE.md)).

## Where Things Live

| I want to... | Look at |
|---|---|
| Report a bug | [.github/ISSUE_TEMPLATE/bug_report.md](.github/ISSUE_TEMPLATE/bug_report.md) |
| Propose a feature | [.github/ISSUE_TEMPLATE/feature_request.md](.github/ISSUE_TEMPLATE/feature_request.md) |
| Propose a new subsystem/protocol/breaking change | [.github/ISSUE_TEMPLATE/rfc_proposal.md](.github/ISSUE_TEMPLATE/rfc_proposal.md), then [docs/rfcs/README.md](docs/rfcs/README.md) |
| Understand the label taxonomy | [.github/LABELS.md](.github/LABELS.md) |

## Code of Conduct

[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) applies to all project spaces.
