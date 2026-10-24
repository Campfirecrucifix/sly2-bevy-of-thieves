// LOOK: PS2 shadow-volume count + CLUT resolve (gfx/shadowvol_pass.rs).

//COUNT_BEGIN
@group(0) @binding(0) var<uniform> clip_from_world: mat4x4<f32>;

struct CountOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) @interpolate(flat) sign: f32,
};

@vertex
fn count_vs(@location(0) p: vec4<f32>) -> CountOut {
    var o: CountOut;
    o.pos = clip_from_world * vec4<f32>(p.xyz, 1.0);
    o.sign = p.w;
    return o;
}

@fragment
fn count_fs(i: CountOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    // GS: +-2 by screen winding (the winding itself carries the volume's sign: an inward-wound
    // volume enters through back faces) -> +1 front / -1 back
    return vec4<f32>(select(-1.0, 1.0, front), 0.0, 0.0, 0.0);
}

//COUNT_END
//RESOLVE_BEGIN
// ---- resolve (FUN_0016dd58 + the FUN_001ea9f0 CLUT) ----

struct Params { rgb_a1: vec4<f32>, eye: vec4<f32>, c0: vec4<f32>, c2: vec4<f32> };
@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var cnt: CNT_TYPE;
@group(0) @binding(2) var<uniform> prm: Params;

// shadowvol::clut_index: n = (fa & 0x7e) | (g' & 0x81), g = 0x3e + 2c, g' = g | bit6 -> bit0
fn clut_index(c: i32, fa: u32) -> u32 {
    var g = u32(0x3e + 2 * c) & 0xffu;
    g = g | ((g >> 6u) & 1u);
    return (fa & 0x7eu) | (g & 0x81u);
}

// shadowvol::clut_alpha
fn clut_alpha(n: u32, a: f32) -> f32 {
    let b = i32(f32(max(i32(n & 0x7fu) - 1, 0)) * 0.007936508 * (a * 0.003921569 * 128.0));
    return f32(b) / 128.0;
}

@fragment
fn resolve_fs(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let p = vec2<i32>(pos.xy);
    let cd = textureLoad(src, p, 0);
    let c = i32(round(textureLoad(cnt, p, 0).r + prm.eye.x));
    // LOOK R24: receiver frame alpha byte (twolayer.wesl writes the PS2 As / 255; 1.0 = a draw that
    // does not carry it (StandardMaterial opaque, RT, clear) -> the old fixed 0x40)
    var fa = u32(round(clamp(cd.a, 0.0, 1.0) * 255.0));
    if (cd.a >= 0.999) { fa = 0x40u; }
    let n = clut_index(c, fa);
    var col = vec4<f32>(0.0);
    if ((n & 0x81u) == 1u) { col = vec4<f32>(prm.rgb_a1.rgb, prm.rgb_a1.w); }
    else if ((n & 0x81u) == 0x80u) { col = prm.c0; }
    else if ((n & 0x81u) == 0x81u) { col = prm.c2; }
    var a = 0.0;
    if ((n & 0x81u) != 0u) { a = clut_alpha(n, col.w); }
    // diag: g = receiver frame alpha
    if (prm.eye.y > 0.5) { return vec4<f32>(0.5 + 0.15 * f32(c), cd.a, a * 4.0, 1.0); }
    return vec4<f32>(mix(cd.rgb, col.rgb, a), cd.a);
}
//RESOLVE_END
