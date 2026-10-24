// PERF2: WIRE-C3 two-layer prim in one draw (twolayer.rs). COLOR = colour-pass vertex colour,
// UV_1 = (grey, alpha) of the lit pass.
import bevy_pbr::render::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
};

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var<uniform> two_layer_mode: vec4<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var colour_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(102) var colour_sampler: sampler;

@fragment
fn fragment(vertex_output: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var in = vertex_output;
    var ov_vc = vertex_output.color;
    if two_layer_mode.y > 0.5 {
        ov_vc.a = 1.0;
    }
    in.color = vec4<f32>(vertex_output.uv_b.x, vertex_output.uv_b.x, vertex_output.uv_b.x, vertex_output.uv_b.y);
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(pbr_input.material.flags, pbr_input.material.alpha_cutoff, pbr_input.material.base_color);
    var lit = vec4<f32>(0.0);
    if two_layer_mode.y > 0.5 {
        // LOOK: raster-baked grey pass (unlit): grey rgb = (UV_1.x, UV_1.y, COLOR.a); COLOR.rgb = colour rgb * alpha
        let gb = vertex_output.color.a;
        // LOOK R26: blended base (mode 1): the grey pass's GS vertex alpha rides in the tangent w
        var ga = 1.0;
        if two_layer_mode.x > 0.5 && two_layer_mode.x < 1.5 {
            @if(VERTEX_TANGENTS) {
                ga = abs(vertex_output.world_tangent.w);
            }
        }
        var bin = vertex_output;
        bin.color = vec4<f32>(vertex_output.uv_b.x, vertex_output.uv_b.y, gb, ga);
        var b_input = pbr_input_from_standard_material(bin, is_front);
        lit = main_pass_post_lighting_processing(b_input, b_input.material.base_color);
    } else {
        lit = main_pass_post_lighting_processing(pbr_input, apply_pbr_lighting(pbr_input));
    }
    // colour pass: unlit, texture * vertex colour * k, additive (Cs*As + Cd)
    var ov_input = pbr_input;
    ov_input.material.base_color = textureSample(colour_texture, colour_sampler, vertex_output.uv) * ov_vc;
    let ov = main_pass_post_lighting_processing(ov_input, ov_input.material.base_color);
    // LOOK R25: the PS2 colour pass is fogged to FOGCOL 0 (black; L3 GS dump: grey pass FOGCOL =
    // the level fog colour, colour pass FOGCOL 0, both FGE 1): f * Cs, not mix(Cs, fog colour, 1 - f).
    // Bevy's fog is a linear mix, so subtracting the fogged black leaves f * Cs.
    let ovk = main_pass_post_lighting_processing(ov_input, vec4<f32>(0.0, 0.0, 0.0, ov_input.material.base_color.a));
    var add = (ov.rgb - ovk.rgb) * ov.a;
    // diag SLY2_LOOK_TLPASS (twolayer::look_tlpass): 1 = base pass only, 2 = colour pass only
    if two_layer_mode.z > 0.5 && two_layer_mode.z < 1.5 { add = vec3<f32>(0.0); }
    if two_layer_mode.z > 1.5 { lit = vec4<f32>(0.0, 0.0, 0.0, lit.a); }
    var out: FragmentOutput;
    let mode = two_layer_mode.x;
    if mode < 0.5 {
        // LOOK R24: opaque base: the frame alpha the PS2 colour pass leaves (ALPHA 0x48, no FBA):
        // As = Ta * Av >> 7 (texel alpha 0x80 = 1.0, Av = the colour-pass vertex alpha, carried in the
        // merged mesh's tangent w, `twolayer::merged_mesh*`); the shadow-volume resolve indexes its
        // CLUT with it (`shadowvol::clut_index`). Stored as byte / 255, kept below 1.0 (1.0 = none).
        var va = 1.0;
        @if(VERTEX_TANGENTS) {
            va = abs(vertex_output.world_tangent.w);
        }
        let ta = round(clamp(textureSample(colour_texture, colour_sampler, vertex_output.uv).a, 0.0, 1.0) * 128.0);
        let fa = min(floor(ta * round(va * 128.0) / 128.0), 254.0);
        out.color = vec4<f32>(lit.rgb + add, fa / 255.0);
    } else if mode < 1.5 {
        out.color = vec4<f32>(lit.rgb * lit.a + add, lit.a);
    } else {
        out.color = vec4<f32>(lit.rgb * lit.a + add, 0.0);
    }
    return out;
}
