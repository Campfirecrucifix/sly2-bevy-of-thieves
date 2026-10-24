// UPSCALE: FSR1-style RCAS (robust contrast adaptive sharpening), own implementation.
@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> p: vec4<f32>; // w, h, sharpness 0..1, unused
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var src: texture_2d<f32>;

fn tap(c: vec2<i32>) -> vec3<f32> {
    let m = vec2<i32>(i32(p.x) - 1, i32(p.y) - 1);
    return textureLoad(src, clamp(c, vec2<i32>(0), m), 0).rgb;
}

@fragment
fn fragment(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let o = vec2<i32>(pos.xy);
    let b = tap(o + vec2<i32>(0, -1));
    let d = tap(o + vec2<i32>(-1, 0));
    let e = tap(o);
    let f = tap(o + vec2<i32>(1, 0));
    let h = tap(o + vec2<i32>(0, 1));
    if (p.z <= 0.0) { return vec4<f32>(e, 1.0); }
    let mn4 = min(min(b, d), min(f, h));
    let mx4 = max(max(b, d), max(f, h));
    let hit_min = mn4 / (4.0 * mx4 + 1e-5);
    let hit_max = (vec3<f32>(1.0) - mx4) / (4.0 * mn4 - vec3<f32>(4.0) - 1e-5);
    let lobe3 = max(-hit_min, hit_max);
    let lobe = max(-0.1875, min(max(lobe3.r, max(lobe3.g, lobe3.b)), 0.0)) * p.z;
    let c = (lobe * (b + d + f + h) + e) / (4.0 * lobe + 1.0);
    return vec4<f32>(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
