# Tour Scene session

Status: ready-for-agent

## Parent

PRD: `.scratch/web-scene-session/PRD.md`

## What to build

Extend the web scene session so Tour mode is owned by the same interface as normal Scene mode. Tour URL hydration should load the Tour, apply the selected Tour step's Scene, suppress normal Scene URL and storage writes while active, write Tour URL params as steps change, and return to normal Scene persistence when the visitor exits the Tour.

## Acceptance criteria

- [ ] Tour URL startup takes precedence over normal Scene URL and stored state.
- [ ] A loaded Tour step applies its Scene to the canvas-facing state.
- [ ] While Tour mode is active, the URL contains Tour parameters and does not fight with a competing Scene URL writer.
- [ ] While Tour mode is active, normal storage writes are suppressed as before.
- [ ] Exiting a Tour restores normal Scene URL persistence for the current Scene.
- [ ] Tests cover Tour URL precedence, step navigation, Scene URL suppression, storage suppression, and Tour exit.

## Blocked by

- `.scratch/web-scene-session/issues/01-normal-scene-session.md`
