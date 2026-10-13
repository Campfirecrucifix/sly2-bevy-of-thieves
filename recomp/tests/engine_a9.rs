//! WP-A9: kind-3 env-only level volumes -> point light (table 14 red box), kinds 1/7 two-layer
//! albedo (grey lit pass + additive colour pass folded, raster = RT), hit star parameters.
use sly2_recomp::engine::gfx::fx::{fx_params, FxKind, HIT_LIFE, HIT_PULL};
use sly2_recomp::engine::gfx::material::{env_glow, env_volume_light, layers, two_layer_albedo};
use sly2_recomp::engine::gfx::texture::Shader;

fn shader(kind: u8, flags: u8, c: [u8; 4]) -> Shader {
    Shader { id: 0, kind, flags, colors: [c; 3], frames: vec![] }
}

#[test]
fn a9_two_layer_albedo_adds_colour_weighted_by_alpha_and_saturates() {
    // grey 0x20 / 0x40, colour (200, 100, 0) at full alpha / half alpha, grey 0xc0 saturating
    let mut base = vec![0x20, 0x20, 0x20, 0xff, 0x40, 0x40, 0x40, 0x80, 0xc0, 0xc0, 0xc0, 0xff];
    let colour = vec![200, 100, 0, 0xff, 200, 100, 0, 0x80, 200, 100, 0, 0xff];
    two_layer_albedo(&mut base, &colour);
    assert_eq!(&base[0..4], &[0x20 + 200, 0x20 + 100, 0x20, 0xff]);
    // colour * 128/255, base alpha kept
    assert_eq!(&base[4..8], &[0x40 + 100, 0x40 + 50, 0x40, 0x80]);
    assert_eq!(&base[8..12], &[255, 255, 0xc0, 0xff]);
}

#[test]
fn a9_kind1_has_the_colour_layer_that_gets_folded() {
    // FUN_001eb960 case 1: second pass = additive (0x48) CLUT set 0 with the VU 0x41f2 colour slot
    let ly = layers(&shader(1, 0, [0x80; 4]));
    assert_eq!(ly.emissive_set, Some(0));
    assert_eq!(ly.base_set, 1);
    assert_eq!(layers(&shader(0, 0, [0x80; 4])).emissive_set, None);
}

#[test]
fn a9_env_volume_light_from_the_t14_box_shader() {
    // table 14 class-48 volumes: kind 3, flags 1 (additive), tint (255, 74, 11, 102)
    let sh = shader(3, 1, [255, 74, 11, 102]);
    assert!(env_glow(&sh).is_some());
    let pos = [[0.0, 0.0, 0.0], [200.0, 100.0, 50.0], [100.0, 0.0, 50.0]];
    let (c, r, col, k) = env_volume_light(&sh, &pos).unwrap();
    assert_eq!(c, [100.0, 50.0, 25.0]);
    assert!((r - (100.0f32 * 100.0 + 50.0 * 50.0 + 25.0 * 25.0).sqrt()).abs() < 1e-3);
    assert_eq!(col[0], 1.0);
    assert!(col[1] < 0.5 && col[2] < 0.1);
    assert!(k > 0.0);
    // not an env-only shader (textured kind 3 / kind 1) -> no volume light; no positions -> none
    assert!(env_volume_light(&shader(3, 3, [255, 74, 11, 102]), &pos).is_none());
    assert!(env_volume_light(&shader(1, 0, [255, 74, 11, 102]), &pos).is_none());
    assert!(env_volume_light(&sh, &[]).is_none());
}

#[test]
fn a9_hit_star_is_visible_at_spawn_and_fades() {
    let (s0, a0, _, _) = fx_params(FxKind::Hit, 0.0, HIT_LIFE, 300.0);
    let (s1, a1, _, _) = fx_params(FxKind::Hit, HIT_LIFE * 0.9, HIT_LIFE, 300.0);
    assert!(s0 > 0.0 && a0 > 0.9);
    assert!(s1 > s0 && a1 < a0);
    assert!(HIT_PULL > 0.0);
}
