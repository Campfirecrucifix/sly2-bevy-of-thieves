// PERF2: WIRE-C3 colour pass in the (binned, batched) opaque forward phase with additive blending
// (twolayer.rs AddPassExt; ray traced GI only). Same output as the AlphaMode::Add StandardMaterial:
// unlit texture * vertex colour, fog, then rgb * a added (blend One/One, depth test, no depth write).
import bevy_pbr::render::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::main_pass_post_lighting_processing,
};

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    let pbr_input = pbr_input_from_standard_material(in, is_front);
    let c = main_pass_post_lighting_processing(pbr_input, pbr_input.material.base_color);
    // LOOK R25: the PS2 additive colour pass fogs to FOGCOL 0 (black): GS fog f*Cs, not mix(fog, Cs)
    let k = main_pass_post_lighting_processing(pbr_input, vec4<f32>(0.0, 0.0, 0.0, pbr_input.material.base_color.a));
    var out: FragmentOutput;
    out.color = vec4<f32>((c.rgb - k.rgb) * c.a, 0.0);
    return out;
}
