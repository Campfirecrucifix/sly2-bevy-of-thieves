// LOOK: PS2 display-space frame -> linear (gfx/display.rs). GS COLCLAMP + 8-bit truncation, then the exact sRGB decode.
@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
struct Ps2DisplayDecode { on: f32 }
@group(0) @binding(2) var<uniform> settings: Ps2DisplayDecode;

@fragment
fn fragment(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    var c = textureLoad(screen_texture, vec2<i32>(pos.xy), 0);
    // diag SLY2_LOOK_DISPTEST: top band = byte ramp x/5 (expected screenshot byte = floor(x/5))
    if (settings.on > 1.5 && pos.y < 64.0) { c = vec4<f32>(vec3<f32>(floor(pos.x / 5.0) / 255.0), 1.0); }
    // GS COLCLAMP, then the 8-bit PSMCT32 frame store: the GS modulate/blend/fog results are truncated
    // (>>7 / >>8), never rounded, so the stored byte is floor(x * 255). The +0.02 absorbs the 8-bit sRGB
    // main-texture storage error so exact byte values are not pushed one step down.
    let d = floor(clamp(c.rgb, vec3<f32>(0.0), vec3<f32>(1.0)) * 255.0 + vec3<f32>(0.02)) / 255.0;
    let lin = select(pow((d + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4)), d / 12.92, d <= vec3<f32>(0.04045));
    // LOOK R24: the 3D alpha carries the PS2 frame alpha for the shadow resolve only; output opaque
    return vec4<f32>(mix(c.rgb, lin, min(settings.on, 1.0)), 1.0);
}
