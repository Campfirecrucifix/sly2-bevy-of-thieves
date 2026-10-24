// UPSCALE: FSR1-style EASU (edge adaptive spatial upsampling), own implementation.
// 12-tap (4x4 minus corners) Lanczos-2 approximation whose kernel is rotated along the local
// luma gradient and stretched by edge strength; result clamped to the centre 2x2 (deringing).
@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> p: vec4<f32>; // in_w, in_h, out_w, out_h
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var src: texture_2d<f32>;

fn tap(c: vec2<i32>) -> vec3<f32> {
    let m = vec2<i32>(i32(p.x) - 1, i32(p.y) - 1);
    return textureLoad(src, clamp(c, vec2<i32>(0), m), 0).rgb;
}
fn luma(c: vec3<f32>) -> f32 { return c.g + 0.5 * (c.r + c.b); }

// direction/length contribution of one centre texel (FSR FsrEasuSetF), weight w
fn dir_len(w: f32, lA: f32, lB: f32, lC: f32, lD: f32, lE: f32, d: ptr<function, vec2<f32>>, len: ptr<function, f32>) {
    // lA up, lB left, lC centre, lD right, lE down
    let dc = lD - lC; let cb = lC - lB;
    var lx = max(abs(dc), abs(cb)); lx = select(0.0, 1.0 / lx, lx > 0.0);
    let dx = lD - lB;
    var lenx = clamp(abs(dx) * lx, 0.0, 1.0); lenx = lenx * lenx;
    let ec = lE - lC; let ca = lC - lA;
    var ly = max(abs(ec), abs(ca)); ly = select(0.0, 1.0 / ly, ly > 0.0);
    let dy = lE - lA;
    var leny = clamp(abs(dy) * ly, 0.0, 1.0); leny = leny * leny;
    *d = *d + vec2<f32>(dx, dy) * w;
    *len = *len + (lenx + leny) * w;
}

fn kernel(off: vec2<f32>, dir: vec2<f32>, len2: vec2<f32>, lob: f32, clp: f32, c: vec3<f32>, acc: ptr<function, vec4<f32>>) {
    var v = vec2<f32>(off.x * dir.x + off.y * dir.y, off.x * -dir.y + off.y * dir.x);
    v = v * len2;
    let d2 = min(dot(v, v), clp);
    var wb = 0.4 * d2 - 1.0; var wa = lob * d2 - 1.0;
    wb = wb * wb; wa = wa * wa;
    wb = 1.5625 * wb - 0.5625;
    let w = wb * wa;
    *acc = *acc + vec4<f32>(c * w, w);
}

@fragment
fn fragment(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let pp = pos.xy * p.xy / p.zw - vec2<f32>(0.5);
    let fp = floor(pp);
    let f = pp - fp;
    let o = vec2<i32>(fp);
    //    b c
    //  e f g h
    //  i j k l
    //    n o
    let b = tap(o + vec2<i32>(0, -1)); let c = tap(o + vec2<i32>(1, -1));
    let e = tap(o + vec2<i32>(-1, 0)); let ff = tap(o); let g = tap(o + vec2<i32>(1, 0)); let h = tap(o + vec2<i32>(2, 0));
    let i = tap(o + vec2<i32>(-1, 1)); let j = tap(o + vec2<i32>(0, 1)); let k = tap(o + vec2<i32>(1, 1)); let l = tap(o + vec2<i32>(2, 1));
    let n = tap(o + vec2<i32>(0, 2)); let oo = tap(o + vec2<i32>(1, 2));
    let bL = luma(b); let cL = luma(c); let eL = luma(e); let fL = luma(ff); let gL = luma(g); let hL = luma(h);
    let iL = luma(i); let jL = luma(j); let kL = luma(k); let lL = luma(l); let nL = luma(n); let oL = luma(oo);
    var dir = vec2<f32>(0.0); var len = 0.0;
    dir_len((1.0 - f.x) * (1.0 - f.y), bL, eL, fL, gL, jL, &dir, &len);
    dir_len(f.x * (1.0 - f.y), cL, fL, gL, hL, kL, &dir, &len);
    dir_len((1.0 - f.x) * f.y, fL, iL, jL, kL, nL, &dir, &len);
    dir_len(f.x * f.y, gL, jL, kL, lL, oL, &dir, &len);
    let d2 = dot(dir, dir);
    let zero = d2 < (1.0 / 32768.0);
    dir = select(dir * inverseSqrt(max(d2, 1e-12)), vec2<f32>(1.0, 0.0), zero);
    len = len * 0.5; len = len * len;
    let stretch = dot(dir, dir) / max(abs(dir.x), abs(dir.y));
    let len2 = vec2<f32>(1.0 + (stretch - 1.0) * len, 1.0 - 0.5 * len);
    let lob = 0.5 + (0.25 - 0.04 - 0.5) * len;
    let clp = 1.0 / lob;
    var acc = vec4<f32>(0.0);
    kernel(vec2<f32>(0.0, -1.0) - f, dir, len2, lob, clp, b, &acc);
    kernel(vec2<f32>(1.0, -1.0) - f, dir, len2, lob, clp, c, &acc);
    kernel(vec2<f32>(-1.0, 1.0) - f, dir, len2, lob, clp, i, &acc);
    kernel(vec2<f32>(0.0, 1.0) - f, dir, len2, lob, clp, j, &acc);
    kernel(vec2<f32>(0.0, 0.0) - f, dir, len2, lob, clp, ff, &acc);
    kernel(vec2<f32>(-1.0, 0.0) - f, dir, len2, lob, clp, e, &acc);
    kernel(vec2<f32>(1.0, 1.0) - f, dir, len2, lob, clp, k, &acc);
    kernel(vec2<f32>(2.0, 1.0) - f, dir, len2, lob, clp, l, &acc);
    kernel(vec2<f32>(2.0, 0.0) - f, dir, len2, lob, clp, h, &acc);
    kernel(vec2<f32>(1.0, 0.0) - f, dir, len2, lob, clp, g, &acc);
    kernel(vec2<f32>(1.0, 2.0) - f, dir, len2, lob, clp, oo, &acc);
    kernel(vec2<f32>(0.0, 2.0) - f, dir, len2, lob, clp, n, &acc);
    let mn = min(min(ff, g), min(j, k));
    let mx = max(max(ff, g), max(j, k));
    let col = clamp(acc.rgb / max(acc.w, 1e-5), mn, mx);
    return vec4<f32>(col, 1.0);
}
