# Multi-atom direction — slice 1

Extends the visualizer beyond hydrogen toward multi-element rendering, with molecular bonding (LCAO) as the eventual long-arc destination.

Full design and rationale: [`docs/superpowers/specs/2026-05-27-multi-atom-design.md`](../../docs/superpowers/specs/2026-05-27-multi-atom-design.md). Read that first — the spec covers physics-model choice (Tier 2 — Slater's effective Z), architectural approach (Approach 3 staged — `Scene = [Atom]` data model from day one), shape (Sandbox + Guided Tours), per-target boundary (what stays in `atom-core` vs. duplicated per-target), and explicit deferrals.

## Slice 1 scope summary

A visitor can pick any of H–Ar from a periodic-table-shaped picker, see that element's chosen orbital, toggle "Bare Z" to watch shielding's effect, share the view via URL, and step through 2–3 guided tours that walk periodic trends.

## Issues

| # | Title | Depends on |
|---|---|---|
| 01 | Scene-shaped bake API (refactor, no behavior change) | — |
| 02 | Multi-element rendering with Slater's effective Z | 01 |
| 03 | Bare-Z toggle | 02 |
| 04 | Shareable URL state | 01 |
| 05 | Periodic-table-shaped picker UI | 02 |
| 06 | "What am I looking at" caption | 02 |
| 07 | Guided tours v1 | 04 |
| 08 | WASM dev-loop: cargo-watch chore | — |
