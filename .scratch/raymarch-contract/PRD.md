# Ray-march contract

Status: ready-for-agent

## Problem Statement

Desktop and web intentionally use separate renderer adapters: wgpu with WGSL on desktop, raw WebGL2 with GLSL in the browser. That decision is correct, but the shared ray-march behavior currently lives as duplicated shader logic plus comments. The project needs a named contract for what both renderer implementations must preserve, without creating a shared renderer abstraction that would contradict ADR-0002.

## Solution

Document and lightly test a ray-march contract: volume texture semantics, coordinate space, uniforms, slab intersection, step count, accumulation formula, intensity mapping, LUT sampling, and expected parity constraints. The contract should be enough for future shader edits to keep desktop and web aligned while leaving each graphics adapter independent.

## User Stories

1. As a desktop user, I want the wgpu renderer to keep producing the same orbital appearance, so that visual behavior stays stable.
2. As a web user, I want the WebGL2 renderer to match the desktop algorithm, so that the browser demo is not a separate interpretation.
3. As a developer editing the WGSL shader, I want to know which algorithm details must be mirrored in GLSL, so that I do not introduce cross-target drift.
4. As a developer editing the GLSL shader, I want to know which desktop behavior is canonical, so that web fixes stay aligned.
5. As a developer changing volume bake output, I want to know the texture layout contract the renderer expects, so that bake and render remain compatible.
6. As a developer changing colormap interpolation, I want to know whether LUT sampling is part of the contract, so that colors do not drift silently.
7. As a developer changing camera math, I want to know how inverse view-projection and camera position are interpreted by the shader, so that rays stay correct.
8. As a developer reviewing ADR-0002, I want confidence this contract is not a renderer abstraction, so that raw WebGL2 remains the web choice.
9. As a future agent optimizing ray marching, I want to know which changes require parity checks, so that optimizations do not alter the teaching visual.
10. As a maintainer debugging a visual mismatch, I want a checklist of contract points, so that diagnosis starts from shared semantics.

## Implementation Decisions

- Create a ray-march contract document or module-level contract that names the required renderer semantics.
- Keep desktop wgpu and web WebGL2 adapters separate.
- Keep WGSL and GLSL shaders separate.
- Define volume data layout: normalized scalar density, row-major x-fastest grid, cubic box half-extent, texture coordinate mapping.
- Define camera inputs: camera world position and inverse view-projection matrix.
- Define ray-box intersection against the volume cube.
- Define march count behavior and relation to volume resolution or renderer params.
- Define accumulation: density multiplied by step length, then emission-only saturation.
- Define intensity mapping: exposure times `1 - exp(-k * sum)`, clamped before LUT lookup.
- Define LUT behavior and interpolation expectations.
- Include explicit ADR note: this contract aligns adapters; it does not introduce a shared renderer module.

## Testing Decisions

- Good tests assert small deterministic contract facts, not full GPU rendering.
- Add CPU-side golden calculations for scalar formulas where practical, such as intensity mapping and LUT interpolation.
- Add shader smoke validation only if the existing toolchain can do it cheaply without mocking full graphics APIs.
- Keep visual parity checks manual: run desktop and web with representative Scenes and inspect output.
- Do not mock wgpu or WebGL2.
- Do not add broad screenshot-diff infrastructure in this PRD.

## Out of Scope

- Rewriting shaders.
- Generating GLSL from WGSL or WGSL from GLSL.
- Introducing Three.js, WebGPU, or a shared renderer abstraction.
- Adding early termination, jittered starts, empty-space skipping, or other renderer optimizations.
- Changing colormap tables.
- Changing volume bake normalization.

## Further Notes

This is the most speculative improvement. Keep it small: a clear contract and narrow tests have value; a large renderer framework would be the wrong direction.
