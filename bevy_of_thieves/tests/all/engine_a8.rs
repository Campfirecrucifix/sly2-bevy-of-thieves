//! WP-A8 (render): RT Blend surfaces use the LDR-clamped alpha (level-0 plant leaves), SLY_NPC_PATROL is
//! a debug-build-only override.
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::gfx::material::{material_from, Alpha};
use sly2_recomp::engine::gfx::rt::{ldr_alpha_k, scale_alpha_rgba8};
use sly2_recomp::engine::gfx::scene::LevelScene;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

/// LDR blend source alpha (unorm target clamps the fragment output) vs the A8 split used by the Hdr camera.
fn ldr(t: u8, v: f32) -> f32 {
    (t as f32 / 255.0 * v).min(1.0)
}
fn split(t: u8, v: f32, k: f32) -> f32 {
    let mut px = [0, 0, 0, t];
    scale_alpha_rgba8(&mut px, k);
    px[3] as f32 / 255.0 * (v / k)
}

#[test]
fn a8_split_matches_ldr_clamp() {
    let v = 255.0 / 128.0; // PS2 vertex alpha 0xFF via ps2_color
    let k = ldr_alpha_k([1.0, v, 0.5, f32::NAN].into_iter());
    assert_eq!(k, v);
    assert_eq!(ldr_alpha_k([0.25, 1.0].into_iter()), 1.0, "alpha <= 1 needs no split");
    for t in 0..=255u8 {
        let (a, b) = (ldr(t, v), split(t, v, k));
        assert!(b <= 1.0 + 1e-6, "split alpha {b} > 1 (t {t})");
        assert!((a - b).abs() <= 1.0 / 255.0 + 1e-6, "t {t}: ldr {a} split {b}");
    }
    // the unsplit Hdr alpha (pre-A8) overshoots 1 for every texel above ~half alpha: the leaf break-up
    assert!(255.0 / 255.0 * v > 1.9);
}

#[test]
fn a8_level0_leaves_need_the_split() {
    let (loader, descs) = levels();
    let l0 = crate::common::load_blocking(&loader, descs[0]).unwrap();
    let lv = crate::common::load_blocking(&loader, l0.header.level_table.entries[0].desc).unwrap();
    let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
    let scene = LevelScene::build(&lv, &w);
    let (mut blend, mut over, mut leaf) = (0, 0, false);
    for so in &scene.objects {
        for p in &so.prims {
            let Some(si) = p.prim.shader else { continue };
            let tex = &scene.textures[so.entry];
            let Some(Ok(dec)) = tex.decode_shader(si as usize, 0) else { continue };
            if material_from(&tex.shaders[si as usize], dec.has_alpha()).alpha != Alpha::Blend || p.prim.colors.is_empty() {
                continue;
            }
            blend += 1;
            let k = ldr_alpha_k(p.prim.colors.iter().map(|c| c[3]));
            if k <= 1.0 {
                continue;
            }
            over += 1;
            // after the split every vertex alpha <= 1 and every (texel, vertex) pair keeps its LDR alpha
            let mut rgba = dec.rgba.clone();
            scale_alpha_rgba8(&mut rgba, k);
            for c in &p.prim.colors {
                let v = c[3];
                assert!(v / k <= 1.0 + 1e-6);
                for (t0, t1) in dec.rgba.iter().skip(3).step_by(4).zip(rgba.iter().skip(3).step_by(4)).step_by(97) {
                    let b = *t1 as f32 / 255.0 * (v / k);
                    if (v - k).abs() < 1e-6 {
                        assert!((ldr(*t0, v) - b).abs() <= 1.0 / 255.0 + 1e-6);
                    }
                    assert!(b <= 1.0 + 1e-6);
                }
            }
            // t0 plant leaves (A8 probe: entry 183 shader 4 / entry 234 shader 1, vertex alpha 0xFF)
            leaf |= (so.entry, si) == (183, 4) || (so.entry, si) == (234, 1);
        }
    }
    println!("level 0: {blend} blend prims, {over} with vertex alpha > 1");
    assert!(over > 0 && leaf, "the plant leaves are Blend prims with vertex alpha > 1");
}

#[test]
fn a8_patrol_override_is_debug_only() {
    // SAFETY: test-local variable, no other test in this binary reads it
    unsafe { std::env::set_var("SLY_NPC_PATROL", "1") };
    let on = sly2_recomp::engine::npcdraw::patrol_override();
    unsafe { std::env::remove_var("SLY_NPC_PATROL") };
    assert_eq!(on, cfg!(debug_assertions), "release builds ignore SLY_NPC_PATROL (J10 scripts start patrols)");
    assert!(!sly2_recomp::engine::npcdraw::patrol_override());
}
