struct Uniforms {
    inv_view_proj: mat4x4<f32>,
    cam_pos: vec4<f32>,
    box_half: vec4<f32>,    // x = half-extent in world units, y..w unused
    params: vec4<f32>,      // x = k, y = exposure, z = res (steps), w = unused
};

@group(0) @binding(0) var<uniform> U: Uniforms;
@group(0) @binding(1) var volume_tex: texture_3d<f32>;
@group(0) @binding(2) var volume_smp: sampler;
@group(0) @binding(3) var lut_tex: texture_1d<f32>;
@group(0) @binding(4) var lut_smp: sampler;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
    var p = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0),
    );
    let xy = p[vid];
    var out: VsOut;
    out.pos = vec4<f32>(xy, 0.0, 1.0);
    out.ndc = xy;
    return out;
}

fn slab_intersect(ro: vec3<f32>, rd: vec3<f32>, bmin: vec3<f32>, bmax: vec3<f32>) -> vec2<f32> {
    let inv = 1.0 / rd;
    let t0 = (bmin - ro) * inv;
    let t1 = (bmax - ro) * inv;
    let tmin = min(t0, t1);
    let tmax = max(t0, t1);
    let tn = max(max(tmin.x, tmin.y), tmin.z);
    let tf = min(min(tmax.x, tmax.y), tmax.z);
    return vec2<f32>(tn, tf);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let near_h = U.inv_view_proj * vec4<f32>(in.ndc, 0.0, 1.0);
    let far_h  = U.inv_view_proj * vec4<f32>(in.ndc, 1.0, 1.0);
    let near_w = near_h.xyz / near_h.w;
    let far_w  = far_h.xyz  / far_h.w;
    let ro = U.cam_pos.xyz;
    let rd = normalize(far_w - near_w);

    let half = U.box_half.x;
    let bmin = vec3<f32>(-half);
    let bmax = vec3<f32>( half);
    let hit = slab_intersect(ro, rd, bmin, bmax);
    if (hit.y <= max(hit.x, 0.0)) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    let t_start = max(hit.x, 0.0);
    let t_end   = hit.y;

    let steps_f = U.params.z;
    let steps = i32(steps_f);
    let dt = (t_end - t_start) / steps_f;
    var sum = 0.0;
    var t = t_start + 0.5 * dt;
    for (var i: i32 = 0; i < steps; i = i + 1) {
        let p = ro + rd * t;
        let uvw = (p + vec3<f32>(half)) / (2.0 * half);
        let d = textureSampleLevel(volume_tex, volume_smp, uvw, 0.0).r;
        sum = sum + d * dt;
        t = t + dt;
    }

    let k = U.params.x;
    let exposure = U.params.y;
    let intensity = clamp(exposure * (1.0 - exp(-k * sum)), 0.0, 1.0);
    let rgb = textureSampleLevel(lut_tex, lut_smp, intensity, 0.0).rgb;
    return vec4<f32>(rgb, 1.0);
}
