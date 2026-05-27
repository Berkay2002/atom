# "What am I looking at" caption

Status: ready-for-agent

## Parent

`.scratch/multi-atom/PRD.md`

## What to build

Show a short caption near the picker that names what's currently on screen in plain language. Tiny effort, huge value for visitors who don't already know spectroscopic notation.

Examples:

- Hydrogen, `n=1, l=0, m=0` → "Hydrogen 1s — the ground-state orbital, spherical and centered on the nucleus."
- Neon, `n=2, l=1, m=0` → "Neon 2p_z — outermost orbital in Ne's electron cloud."
- Carbon, `n=2, l=1, m=1` → "Carbon 2p_x — one of three 2p orbitals; carbon has two electrons across this subshell."

Add a small table mapping `(l, m)` to a short orbital descriptor (s, p_z, p_x, p_y, d_z², d_xz, …) — it's static, only ~15 entries for slice 1's coverage. Combine with the element's symbol/name and electron-config from `atom-core::element` (issue 02) to produce the caption string.

Where the table lives is a judgment call:

- **In `atom-core`** (recommended): the orbital-label table is shared content, drift-proof, and tiny. Putting it next to `ElementData` keeps the "what does this mean" knowledge in one place.
- In `web/src/lib/` only: avoids touching atom-core, but the desktop target loses the caption — which is a regression in user experience.

Go with `atom-core`. Add a function like `caption(scene: &Scene) -> String` to `atom-core::element` (or a new sibling module if it grows). For single-atom Scenes (slice 1's only case), produce the format `"<Name> <n><label> — <one-sentence description>"`. The descriptive sentence can be a small per-`(n, l)` lookup; if no entry exists for an exotic combination (e.g., Carbon's 4f), fall back to a generic "Carbon 4f orbital".

UI placement:

- **Web:** beneath the picker or above the canvas; a single-line caption that wraps gracefully on narrow viewports. Updates whenever the Scene changes.
- **Desktop:** in the egui side panel, near the picker. Same behavior.

## Acceptance criteria

- [ ] `atom-core` exposes a `caption(&Scene) -> String` function (or equivalent) producing the caption text
- [ ] At least 6 hand-written orbital descriptions are present (1s, 2s, 2p, 3s, 3p, 3d) so common picks read naturally
- [ ] Unknown orbital combinations produce a non-broken generic fallback (e.g., "Carbon 4f orbital")
- [ ] Web: caption visible and updates on any Scene change
- [ ] Desktop: caption visible in side panel and updates on any Scene change
- [ ] Caption uses the existing `ElementData` and orbital-label table; no duplicated string tables per-target

## Blocked by

`.scratch/multi-atom/issues/02-multi-element-with-effective-z.md`
