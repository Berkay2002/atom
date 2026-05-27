# Bare-Z toggle

Status: ready-for-agent

## Parent

`.scratch/multi-atom/PRD.md`

## What to build

Surface the `View.use_bare_z` flag (already wired through the bake in issue 02) as a visible UI toggle on both targets. The toggle is the pedagogical hook of the multi-atom direction: flipping it lets a user watch shielding's effect on orbital size by comparison.

Behavior:

- **Off (default — Effective Z):** orbitals use Slater's-rules `z_eff`. Carbon's 2p is its realistic size.
- **On (Bare Z):** orbitals use the bare atomic number `Z`. Carbon's 2p collapses inward to a hilariously tight cloud because all 6 protons pull on the electron with no shielding.

UI placement:

- **Web:** a labeled switch or toggle button in the controls panel, visually grouped near the element picker. Label clearly: "Effective Z" (default) / "Bare Z". A short helper text or tooltip explains what the toggle does (one sentence; full explanation is the job of guided tours in issue 07).
- **Desktop:** an egui checkbox in the side panel with the same label and helper text.

Toggling the switch triggers a re-bake via the existing debounced bake pipeline. The switch state is part of `View` in the Scene, so it's automatically captured by URL serialization (issue 04) and tour steps (issue 07) without extra work here.

No new physics or bake code — the plumbing was done in issue 02. This slice is purely UI exposure plus a paragraph or two of doc-text on each target.

## Acceptance criteria

- [ ] Web: toggle visible in the controls panel; flipping it re-bakes and renders the correct cloud
- [ ] Desktop: egui checkbox present in side panel; same behavior
- [ ] Toggling between modes for Carbon at `(n=2, l=1, m=0)` produces a visibly different, much smaller orbital when Bare Z is on
- [ ] Helper text/tooltip explains the toggle in one sentence
- [ ] Toggle state is preserved on debounced re-bake (no UI flicker)

## Blocked by

`.scratch/multi-atom/issues/02-multi-element-with-effective-z.md`
