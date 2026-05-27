// GLSL ES 3.00 shaders ported line-for-line from
// `crates/atom-desktop/shaders/raymarch.wgsl`.
//
// Differences from the desktop WGSL source:
//   * `@vertex` / `@fragment` become GLSL `in`/`out` declarations.
//   * The 1D colormap LUT is a 2D texture with height=1 — WebGL2 has no
//     1D texture target. Sampled at v=0.5 to land in the centre of the
//     single row. Linear filter on `u` matches the desktop's wgpu sampler.
//   * UV convention: GLSL samples `texture(volume, uvw)`; we still map
//     world-space `(p + half) / (2*half)` into [0,1]^3, identical to WGSL.

export const VERT_SRC = /* glsl */ `#version 300 es
precision highp float;

out vec2 v_ndc;

// Standard fullscreen triangle: three NDC verts that cover [-1,1]^2 with
// no vertex buffer. Indices 0,1,2 produce (-1,-1), (3,-1), (-1,3).
void main() {
    vec2 p = vec2(
        (gl_VertexID == 1) ?  3.0 : -1.0,
        (gl_VertexID == 2) ?  3.0 : -1.0
    );
    v_ndc = p;
    gl_Position = vec4(p, 0.0, 1.0);
}
`;

export const FRAG_SRC = /* glsl */ `#version 300 es
precision highp float;
precision highp sampler3D;

in vec2 v_ndc;
out vec4 frag_color;

uniform mat4 u_inv_view_proj;
uniform vec3 u_cam_pos;
uniform float u_box_half;
// (k, exposure, steps, _)
uniform vec4 u_params;
uniform sampler3D u_volume;
uniform sampler2D u_lut;

vec2 slab_intersect(vec3 ro, vec3 rd, vec3 bmin, vec3 bmax) {
    vec3 inv = 1.0 / rd;
    vec3 t0 = (bmin - ro) * inv;
    vec3 t1 = (bmax - ro) * inv;
    vec3 tmin = min(t0, t1);
    vec3 tmax = max(t0, t1);
    float tn = max(max(tmin.x, tmin.y), tmin.z);
    float tf = min(min(tmax.x, tmax.y), tmax.z);
    return vec2(tn, tf);
}

void main() {
    vec4 near_h = u_inv_view_proj * vec4(v_ndc, 0.0, 1.0);
    vec4 far_h  = u_inv_view_proj * vec4(v_ndc, 1.0, 1.0);
    vec3 near_w = near_h.xyz / near_h.w;
    vec3 far_w  = far_h.xyz  / far_h.w;
    vec3 ro = u_cam_pos;
    vec3 rd = normalize(far_w - near_w);

    float half_ext = u_box_half;
    vec3 bmin = vec3(-half_ext);
    vec3 bmax = vec3( half_ext);
    vec2 hit = slab_intersect(ro, rd, bmin, bmax);
    if (hit.y <= max(hit.x, 0.0)) {
        frag_color = vec4(0.0, 0.0, 0.0, 1.0);
        return;
    }
    float t_start = max(hit.x, 0.0);
    float t_end   = hit.y;

    float steps_f = u_params.z;
    int steps = int(steps_f);
    float dt = (t_end - t_start) / steps_f;
    float sum = 0.0;
    float t = t_start + 0.5 * dt;
    for (int i = 0; i < steps; ++i) {
        vec3 p = ro + rd * t;
        vec3 uvw = (p + vec3(half_ext)) / (2.0 * half_ext);
        float d = texture(u_volume, uvw).r;
        sum += d * dt;
        t += dt;
    }

    float k = u_params.x;
    float exposure = u_params.y;
    float intensity = clamp(exposure * (1.0 - exp(-k * sum)), 0.0, 1.0);

    // Sample the 256x1 LUT at (intensity, 0.5). v=0.5 lands in the centre
    // of the only row; CLAMP_TO_EDGE on u handles the inclusive [0,1] range.
    vec3 rgb = texture(u_lut, vec2(intensity, 0.5)).rgb;
    frag_color = vec4(rgb, 1.0);
}
`;
