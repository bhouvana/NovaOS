# Spec 13 — Website Information Architecture

Status: Draft v0.1 · Last updated: 2026-07-18

Full site map for `web/`, addressing the Staff Engineer note that novaos.dev should read
as an open-source OS project's home, not a single download page.

## 1. Site Map

```text
novaos.dev
├── /                    Landing — hero, "Try it now" CTA, feature highlights,
│                        architecture teaser, footer nav to everything below
├── /demo                The browser demo (12-BROWSER-DEMO-EXPERIENCE.md)
├── /architecture         Renders this docs/ tree (00-VISION.md through
│                        14-FUTURE-VISION.md + docs/specs/) as a navigable doc site
├── /docs                 Nova SDK reference — generated from doc-comments
│                        (04-APPLICATION-FRAMEWORK-AND-SDK.md §8), plus a
│                        hand-written "Getting Started" (nova-cli new walkthrough)
├── /packages              Package Browser — read-only view of the Nova Store
│                        catalog (§3), no auth/install from the web (installs only
│                        happen from within a running NovaOS via Package Center)
├── /roadmap               Renders 12-ROADMAP-AND-MILESTONES.md, kept current
│                        automatically from the same source (§4)
├── /downloads             Signed ISO download + checksums + signing key
│                        fingerprint (08-SECURITY-MODEL.md §4) + installer
│                        instructions
└── /community              GitHub link, contribution guide pointer
                           (11-CODING-STANDARDS.md), 14-ECOSYSTEM-VISION.md's
                           community-facing framing (once Phase 6+ opens
                           external contribution)
```

## 2. Landing Page (`/`) Structure

1. **Hero**: one-line positioning — *"NovaOS: a lightweight desktop platform built on
   Linux"* (the framing correction from the Staff Engineer review, applied at
   [../README.md](../README.md) and here consistently — not "another Linux distro").
   Primary CTA: "Try it now" → begins loading `/demo` immediately, no intermediate page.
2. **Feature highlights**: 3–4 cards — Fast boot, Low RAM, Runs in your browser, Built
   for developers — each linking to the relevant `/architecture` doc for anyone who
   wants depth, keeping the landing page itself skimmable.
3. **Architecture teaser**: the layer diagram from
   [../01-SYSTEM-ARCHITECTURE.md](../01-SYSTEM-ARCHITECTURE.md) §1, rendered visually,
   linking into `/architecture`.
4. **Footer**: links to every top-level site section (§1), GitHub, license.

## 3. Package Browser (`/packages`)

- Statically renders the same Nova Store catalog data
  ([../05-PACKAGE-AND-UPDATE-SYSTEM.md](../05-PACKAGE-AND-UPDATE-SYSTEM.md) §2) that
  Nova Package Center consumes inside the OS — one catalog source, two consumers (the
  in-OS app and this static page), never a second hand-maintained listing.
- Read-only by design (§1): reinforces that Nova Store is a real package registry with
  real signed artifacts, not a marketing gallery, while keeping the web tier free of any
  install/auth backend, consistent with
  [../07-BROWSER-DEPLOYMENT.md](../07-BROWSER-DEPLOYMENT.md)'s static-hosting
  constraint.

## 4. Docs-as-Source-of-Truth

`/architecture`, `/docs`, and `/roadmap` are all generated at build time
([11-BUILD-PIPELINE-SPEC.md](11-BUILD-PIPELINE-SPEC.md) §1) directly from this
repository's `docs/` tree and SDK doc-comments — never hand-copied into `web/`'s own
content files. This is the same principle already stated in
[../07-BROWSER-DEPLOYMENT.md](../07-BROWSER-DEPLOYMENT.md) §5, made concrete: a
markdown-to-site renderer (part of the React site build,
[08-BROWSER-ARCHITECTURE-SPEC.md](08-BROWSER-ARCHITECTURE-SPEC.md) §1a) ingests
`docs/**/*.md` and cross-reference links resolve to in-site routes automatically —
meaning every relative link used throughout this entire `docs/` tree
(`[../00-VISION.md](../00-VISION.md)`-style paths) doubles as the site's real navigation
structure with no separate link-mapping step.

## 5. Navigation Model

Persistent top nav across all pages except `/demo` (which goes chrome-free/fullscreen-
capable per [08-BROWSER-ARCHITECTURE-SPEC.md](08-BROWSER-ARCHITECTURE-SPEC.md) §6, to
avoid browser-chrome-within-site-chrome-within-emulated-OS-chrome nesting): Demo ·
Architecture · Docs · Packages · Roadmap · Downloads · Community (GitHub icon, external
link).

## 6. What's Explicitly Not Here

- No blog/news section in v1 — the roadmap and GitHub releases are the source of "what's
  new," avoiding a second content type that needs ongoing authorship before there's a
  team to sustain it.
- No account system/sign-in anywhere on the site — consistent with
  [../00-VISION.md](../00-VISION.md) §7's non-goal; the Package Browser (§3), Docs, and
  Demo are all usable with zero authentication.
