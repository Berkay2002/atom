# Web scene session

Status: ready-for-agent

## Problem Statement

The web app's page module owns too many Scene lifecycle rules at once: initial hydration, Scene URL decoding, local storage fallback, Tour mode, URL writes, storage writes, decode errors, and user-driven Scene changes. That makes the module shallow: callers get little leverage from the interface, while maintainers must understand ordering constraints spread through many effects.

## Solution

Create a deep web scene session module that owns the browser-side Scene lifecycle behind a small interface. The page keeps rendering the canvas, controls, Tour bar, and error banner, but it no longer owns the state machine that decides where a Scene came from, where it should be persisted, or how Tour mode suppresses normal Scene URL writes. This PRD is a pure refactor: external behavior must be preserved.

## User Stories

1. As a visitor opening the web app with no URL state, I want my last stored Scene to load, so that returning to the demo preserves my previous exploration.
2. As a visitor opening a Scene URL, I want the encoded Scene to take precedence over stored state, so that shared links show the sender's intended Scene.
3. As a visitor opening a Tour URL, I want the Tour step to take precedence over normal Scene URLs, so that the guided lesson controls the Scene.
4. As a visitor with malformed Scene URL data, I want a readable non-blocking error, so that the app still loads a usable default.
5. As a visitor changing an element, Orbital, colormap, or Bare Z choice, I want the address bar to update after a short quiet period, so that I can copy a stable link.
6. As a visitor rapidly clicking controls, I want URL and storage writes to collapse into the final state, so that the browser history and storage are not spammed.
7. As a visitor in Tour mode, I want the URL to encode the Tour and step instead of a competing Scene URL, so that one source of truth controls the page.
8. As a visitor leaving Tour mode, I want the current Scene to become shareable again, so that I can continue exploring from the final Tour step.
9. As a visitor toggling HUD visibility, I want that preference persisted as before, so that this refactor does not change existing behavior.
10. As a developer modifying Scene persistence, I want one module to hold precedence rules, so that I do not have to reason across unrelated page effects.
11. As a developer modifying Tour behavior, I want Tour URL writes and Scene URL suppression in one place, so that Tour mode cannot fight normal persistence.
12. As a developer changing the Scene URL codec, I want the session module to be the only browser lifecycle caller, so that codec initialization has one owner.
13. As a developer testing hydration, I want to exercise one state machine interface, so that tests describe behavior rather than React effect ordering.
14. As a developer reviewing a future Scene feature, I want page composition to stay obvious, so that new features do not add another parallel effect chain.

## Implementation Decisions

- Build a web scene session module with a small interface that exposes current Scene state, Tour state, decode-error state, and intent handlers.
- Preserve the current precedence order: Tour URL, then Scene URL, then stored state, then defaults.
- Preserve debounced local storage writes for normal Scene mode.
- Preserve debounced Scene URL writes for normal Scene mode.
- Preserve Tour URL writes while Tour mode is active.
- Preserve suppression of normal storage writes while Tour mode is active.
- Preserve the current behavior where Tour step application mutates the active Scene state used by the canvas.
- Keep routing concerns inside the session module: callers should not hand-edit Scene URL or Tour query parameters.
- Keep rendering concerns outside the session module: the module does not draw, bake, or render UI.
- Treat this as behavior-preserving architecture work. Any discovered behavior bug should be documented as follow-up unless it blocks the refactor.

## Testing Decisions

- Good tests drive the scene session interface and assert observable browser lifecycle behavior: selected Scene, Tour mode, URL writes, storage writes, and decode errors.
- Test the hydration precedence order using fake URL, fake storage, and fake Tour loading adapters.
- Test normal Scene changes debounce URL and storage writes.
- Test Tour mode writes Tour parameters and suppresses normal Scene URL writes.
- Test Tour exit hands control back to normal Scene persistence.
- Test malformed Scene URL handling produces a decode error and falls back without crashing.
- Do not test canvas rendering, WebGL, or visual layout here; renderer and UI verification remain visual per project convention.

## Out of Scope

- Changing the Scene URL format.
- Changing Tour JSON schema.
- Changing default Scene values.
- Adding new Tour controls.
- Changing visual UI layout.
- Fixing unrelated hydration or rendering bugs discovered during the refactor.

## Further Notes

This PRD should be implemented before other web-facing architecture work where possible, because it reduces churn around Scene URL, Tour, and storage ownership.
