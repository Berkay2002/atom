# Atom

Interactive orbital visualizer for teaching how atomic orbitals change across quantum numbers, elements, shielding, and guided scenes.

## Language

**Scene**:
The shareable state of what is being visualized: one or more Atoms plus the View that explains how the scene is presented. A Scene is the unit encoded into URLs and consumed by the volume bake.
_Avoid_: app state, render state

**Atom**:
An element placed at a position with one selected Orbital. In the current learning path, a Scene has exactly one Atom at the origin; multi-Atom Scenes are reserved for comparison and bonding.
_Avoid_: particle, nucleus

**Orbital**:
A single `(n, l, m)` quantum-number triple for one Atom. It is not a list of orbitals; in-Atom composition would require a separate model because true superposition density is not the sum of independent densities.
_Avoid_: shell when the quantum numbers matter, cloud

**View**:
Presentation choices that apply to a Scene as a whole, such as bare-vs-effective nuclear charge, colormap, exposure, and camera. View choices are global to the lesson being shown, not per-Atom facts.
_Avoid_: settings when the value must round-trip with a Scene

**Effective Z**:
The Slater-screened effective nuclear charge used to size and evaluate hydrogen-like orbitals for elements beyond hydrogen. Effective Z is the default teaching mode for multi-element rendering.
_Avoid_: shielded charge, Slater mode

**Bare Z**:
The element's raw atomic number used as the nuclear charge, ignoring electron shielding. Bare Z is a comparison View, not a physically preferred default.
_Avoid_: actual Z, real charge

**Tour**:
A guided sequence of Scenes plus captions. A Tour teaches a relationship, such as shielding across a period, by stepping through pre-authored Scene states.
_Avoid_: tutorial, slideshow

**Scene URL**:
A versioned string representation of a Scene used for shareable links and Tour steps. The Scene URL is the cross-target contract; desktop and web must agree on it byte-for-byte.
_Avoid_: query state, deep link payload

## Example Dialogue

Dev: "Should Bare Z live on each Atom?"

Domain expert: "No. Bare Z is a View choice for the whole Scene. A Scene comparing Carbon with Bare Z and Oxygen with Effective Z would teach the wrong lesson."

Dev: "Can an Orbital hold multiple `(n, l, m)` triples?"

Domain expert: "No. An Orbital is one triple. If we ever teach superposition, that gets its own term because the density math changes."

Dev: "Can a Tour step use a different encoding from a shared link?"

Domain expert: "No. A Tour step stores a Scene URL. If a user can share it, a Tour can use it."
