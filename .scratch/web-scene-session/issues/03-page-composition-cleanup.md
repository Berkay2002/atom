# Page composition cleanup

Status: ready-for-agent

## Parent

PRD: `.scratch/web-scene-session/PRD.md`

## What to build

Finish the web scene session refactor by removing residual lifecycle ownership from the page module. The page should compose canvas, controls, Tour bar, eye toggle, and decode-error banner from session state and intent handlers, without hand-editing Scene URL, storage, Tour params, or codec lifecycle directly.

## Acceptance criteria

- [ ] The page no longer owns URL hydration, local storage hydration, URL writes, storage writes, or Tour URL writes directly.
- [ ] The page uses session intent handlers for element changes, Orbital changes, colormap changes, Bare Z changes, HUD visibility changes, Tour selection, Tour navigation, Tour exit, and decode-error dismissal.
- [ ] Rendering behavior remains unchanged for normal mode and Tour mode.
- [ ] Existing web tests pass, and session tests remain the behavioral coverage for lifecycle rules.
- [ ] Manual verification confirms normal load, shared Scene URL load, Tour load, Tour step navigation, Tour exit, and HUD toggle still work.

## Blocked by

- `.scratch/web-scene-session/issues/01-normal-scene-session.md`
- `.scratch/web-scene-session/issues/02-tour-scene-session.md`
