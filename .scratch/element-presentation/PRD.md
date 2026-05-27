# Element presentation

Status: ready-for-agent

## Problem Statement

Element presentation knowledge is partly centralized and partly duplicated. Rust owns element data, captions, and HOMO rules, while web and desktop UI still carry their own symbol lists, periodic layout slots, and in some cases HOMO snapshots. That weakens locality: a future element-table or caption change can require edits in several modules.

## Solution

Create a deeper element presentation module that returns the UI-ready facts needed by both targets: element identity, symbol, display name, configuration text, HOMO Orbital, periodic-table slot, and caption-related labels. Target-specific UI still decides how to paint those facts, but the domain facts and stable layout metadata live behind one interface.

## User Stories

1. As a visitor, I want element symbols to be consistent between desktop and web, so that the same element means the same thing everywhere.
2. As a visitor selecting an element, I want the app to snap to that element's HOMO, so that the visualization never starts blank.
3. As a visitor reading a caption, I want the element name and Orbital label to match the selected element and Orbital, so that the display teaches the right concept.
4. As a visitor using the web periodic-table picker, I want elements to appear in their expected period positions, so that the picker teaches periodic structure.
5. As a visitor using the desktop element picker, I want the same supported H-Ar element set, so that desktop and web teach the same scope.
6. As a developer adding an element presentation fact, I want one module to own it, so that web and desktop do not grow parallel tables.
7. As a developer changing HOMO rules, I want one source of truth, so that snapping behavior cannot drift between targets.
8. As a developer changing captions, I want tests to cover the presentation interface, so that wording stays intentional.
9. As a developer polishing web UI, I want UI code to consume element presentation facts without knowing electron configuration internals, so that visual work stays local.
10. As a developer polishing desktop UI, I want the same presentation facts without pulling in web layout code, so that target-specific chrome remains target-specific.
11. As a developer reviewing element support, I want H-Ar scope explicit in one interface, so that unsupported elements fail predictably.
12. As a future agent working on multi-Atom comparison, I want element display facts available independently of current single-Atom controls, so that comparison UI can reuse them.

## Implementation Decisions

- Define an element presentation interface in the shared core that returns facts, not UI widgets.
- Include atomic number, symbol, full name, electron configuration display string, HOMO Orbital, and periodic-table position for H-Ar.
- Keep captions in shared core, using the same Orbital language as current code.
- Keep actual UI layout and painting per-target. The web can render a CSS grid; desktop can render egui chips.
- Delete or reduce duplicated web HOMO data if the new interface can satisfy synchronous UI needs without introducing awkward async behavior.
- Preserve the current H-Ar support limit.
- Preserve the convention that HOMO uses `m = 0`.
- Preserve the current fallback behavior for unknown elements where relevant, but make fallback behavior explicit in the interface docs.
- Avoid moving colormap or camera presentation into this module; those are View concerns, not element facts.

## Testing Decisions

- Good tests assert element presentation facts through the shared interface.
- Test H-Ar entries are contiguous and have unique symbols.
- Test each element's presentation HOMO matches the intended table.
- Test periodic-table slots are valid and unique within the H-Ar layout.
- Test caption output for representative elements and fallback cases.
- If any TypeScript snapshot remains, keep or add parity tests against Rust facts.
- Do not unit-test visual placement in egui or React; verify picker layout visually after implementation.

## Out of Scope

- Supporting elements beyond argon.
- Adding transition-metal or lanthanide Slater rules.
- Changing the physics model.
- Redesigning the element picker UI.
- Localizing element names or captions.
- Adding molecule or bonding presentation.

## Further Notes

This PRD should stay focused on element facts and labels. If the implementation starts deciding how controls look, it has crossed into target-specific UI chrome.
