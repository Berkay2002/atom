# Periodic-table-shaped picker UI

Status: ready-for-agent

## Parent

`.scratch/multi-atom/PRD.md`

## What to build

Replace the basic element picker shipped in issue 02 with an 18-cell grid laid out in the shape of the periodic table for periods 1–3. The layout itself is part of the pedagogy: H/He at top, Li–Ne row, Na–Ar row, with the column structure communicating groups.

Web only. The desktop picker stays as-is (egui doesn't render a satisfying periodic table without significant custom drawing work, and the desktop target is the developer/power-user tool — the layout pedagogy matters more on the public web demo).

Layout requirements:

- 18 cells arranged as: row 1 (H, gap×16, He), row 2 (Li, Be, gap×10, B, C, N, O, F, Ne), row 3 (Na, Mg, gap×10, Al, Si, P, S, Cl, Ar). Yes, period 3 doesn't actually need the transition-metal gap chemically, but matching period 2's column structure makes the s-block / p-block visual story clearer.
- Each cell shows the element symbol prominently; optionally atomic number in a small corner.
- The currently-selected element has a visibly distinct background (use the project's existing color palette / theme).
- Clicking a cell selects that element and triggers a re-bake.
- Responsive: on narrow viewports, the layout can collapse the gaps (squeeze together) but should retain row structure. A full-width-per-cell vertical list is acceptable as the smallest fallback.

This slice is **functional layout, not aesthetic polish**. The spec calls out "don't spend a week on visual polish for a picker." Goals are: correct positions, readable symbols, clear selection state. Animations, hover effects, color coding by block, etc. are explicit non-goals — they belong in a future polish slice if anyone wants them.

Replace the picker UI from issue 02 with this grid in the controls panel. Behaviorally identical (selecting an element re-bakes); only the presentation changes.

## Acceptance criteria

- [ ] 18 elements arranged in periodic-table-shaped grid (H/He top, Li–Ne row, Na–Ar row)
- [ ] Symbol clearly visible in each cell
- [ ] Selected element visibly distinct
- [ ] Clicking selects + re-bakes
- [ ] Layout remains usable down to a mobile-portrait viewport (≤ 400px wide)
- [ ] Replaces the issue-02 basic picker entirely (no stale UI left behind)

## Blocked by

`.scratch/multi-atom/issues/02-multi-element-with-effective-z.md`
