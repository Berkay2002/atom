# Normal Scene session

Status: ready-for-agent

## Parent

PRD: `.scratch/web-scene-session/PRD.md`

## What to build

Move the normal, non-Tour browser Scene lifecycle behind a web scene session interface. The slice should cover startup hydration from Scene URL, stored state, and defaults; normal Scene mutation; debounced Scene URL writes; debounced storage writes; and decode-error reporting. The page should render through the new session interface for normal mode while preserving behavior.

## Acceptance criteria

- [ ] Startup precedence is preserved for normal mode: Scene URL wins over stored state, stored state wins over defaults.
- [ ] Malformed Scene URLs surface the same readable decode-error behavior and fall back without crashing.
- [ ] Normal Scene changes still debounce Scene URL writes.
- [ ] Normal Scene changes still debounce local storage writes.
- [ ] HUD visibility persistence behavior is unchanged.
- [ ] Tests cover hydration precedence, malformed Scene URL fallback, debounced URL writes, and debounced storage writes through the session interface.

## Blocked by

None - can start immediately
