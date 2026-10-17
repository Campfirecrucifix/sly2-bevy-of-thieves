//! WP-C2 tests: VU1 cull flag counts over all 43 levels, fog/colour path unit tests. Needs the ISO at
//! "<root>/Sly 2 - Band of Thieves (USA).iso" (level list from the boot level table).
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::{LevelLoader, LoadedLevel};
use sly2_recomp::engine::world::{LoadOptions, World};
use std::path::PathBuf;
use std::sync::Arc;

fn real_root() -> PathBuf {
    let r = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    if r.join("Sly 2 - Band of Thieves (USA).iso").exists() { r } else { PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/..")) }
}

/// The 43 level descriptors (db disc_files + level_table, as in engine_a).
fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

fn load(loader: &LevelLoader, d: Descriptor) -> (LoadedLevel, World) {
    let lv = crate::common::load_blocking(&loader, d).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap_or_else(|e| panic!("lsn {}: {e}", d.lsn()));
    (lv, w)
}



use sly2_recomp::engine::gfx::attach::{alo_bone_refs, alo_of, xfm_bone_ref, BoneRef};
use sly2_recomp::engine::gfx::light::*;
use sly2_recomp::engine::gfx::scene::IDENTITY;
use sly2_recomp::engine::world::objects::{LoadData, Payload};

#[test]
fn light_math_from_code() {
    // FUN_001a3730: FitLinear through (x0,y0),(x1,y1); near-equal x -> (y0, 0)
    assert_eq!(ee_sub_1a3730(200.0, 1.0, 2000.0, 0.0), (1.0 + 200.0 / 1800.0, -1.0 / 1800.0));
    assert_eq!(ee_sub_1a3730(5.0, 3.0, 5.0, 9.0), (3.0, 0.0));
    // FUN_001a38a8 at deg 0: cos(0)=1 -> x = 2
    let (du, ru) = ee_sub_1a38a8(180.0, 0.0, 0.0, 255.0);
    assert!((du + ru * 2.0 - 1.0).abs() < 1e-5);
    // FUN_001a3360 defaults
    let p = ee_sub_1a3360();
    assert_eq!((p.kind, p.angle, p.falloff, p.b158), (0, 240.0, [200.0, 2000.0], 1));
    // point light, white V=255: colour (255,255,255)/g, g = 255, distance term g at near, 0 at far
    let mut p = p;
    p.kind = 1;
    p.hsv = [0.0, 0.0, 255.0];
    let mut m = IDENTITY;
    m[3] = [1.0, 2.0, 3.0];
    let d = ee_sub_1a3968(&p, &m, false);
    assert_eq!(d.g, 255.0);
    assert_eq!(d.color, [1.0; 3]);
    assert_eq!(d.vec, [1.0, 2.0, 3.0]);
    assert!((d.du_fall + d.ru_fall * 200.0 - 255.0).abs() < 1e-2);
    assert!((d.du_fall + d.ru_fall * 2000.0).abs() < 1e-2);
    // directional: row 0, constant distance term g
    p.kind = 0;
    let d = ee_sub_1a3968(&p, &m, false);
    assert_eq!((d.vec, d.du_fall, d.ru_fall), ([1.0, 0.0, 0.0], 255.0, 0.0));
}

/// All 43 levels: every class-4 object becomes a light; kinds / flags as the setters allow;
/// FUN_0022d6b0 refs never dangle; child-ALO roots hang off an owner bone.
#[test]
#[ignore = "heavy"]
fn lights_and_attachments_all_levels() {
    let (ld, descs) = levels();
    let (mut lights, mut dir, mut point, mut excl, mut dynamic, mut owner_bone_roots, mut attach, mut xfm_bones) = (0, 0, 0, 0, 0, 0, 0, 0);
    let (mut dir_dyn, mut owned_world_roots) = (0, 0);
    for d in descs {
        let (_lv, w) = load(&ld, d);
        let n4 = w.objects.iter().filter(|o| o.class == CID_LIGHT && matches!(o.payload, Payload::Loaded(LoadData::L231170 { .. }))).count();
        let mut bw = sly2_recomp::engine::gfx::attach::BoneWorlds::default();
        let ls = level_lights(&w, &mut bw);
        assert_eq!(ls.len(), n4, "lsn {}", d.lsn());
        let mut ndir = 0;
        for l in &ls {
            assert!(l.params.kind <= 1, "lsn {} light kind {}", d.lsn(), l.params.kind);
            assert!(!(l.params.excluded && l.params.dynamic), "157/158 clear each other");
            assert!(l.derived.g >= 0.0 && l.world.iter().flatten().all(|v| v.is_finite()));
            if l.derived.g > 0.0 {
                // angular term at theta 0 = g/255 (FitLinear end point)
                assert!((l.angular(0.0) - l.derived.g / 255.0).abs() < 1e-3 * l.derived.g.max(1.0));
            }
            match l.params.kind {
                // WP-B4: dynamic (158) directional lights on prefab/actor ALOs exist; only the
                // static sun is unique per level
                0 if !l.params.dynamic => ndir += 1,
                0 => dir_dyn += 1,
                _ => point += 1,
            }
            excl += l.params.excluded as usize;
            dynamic += l.params.dynamic as usize;
        }
        // at most one static directional (sun) light per level
        assert!(ndir <= 1, "lsn {} {} directional", d.lsn(), ndir);
        dir += ndir;
        lights += ls.len();
        for i in 0..w.objects.len() {
            let refs = alo_bone_refs(&w, i);
            for (bi, (p, a)) in refs.iter().enumerate() {
                assert_ne!(*p, BoneRef::Dangling, "lsn {} obj {} bone {}", d.lsn(), i, bi);
                if let Some(a) = a {
                    attach += 1;
                    assert_ne!(*a, BoneRef::Dangling);
                }
                if bi == 0 {
                    match (w.objects[i].owner.and_then(|o| alo_of(&w, o).map(|_| o)), p) {
                        (Some(o), BoneRef::Bone(t, _)) => {
                            assert_eq!(*t, o);
                            owner_bone_roots += 1;
                        }
                        (None, BoneRef::None) => {}
                        // WP-B4: owned ALOs from class-6 nested lists may use -1 (world space)
                        (Some(_), BoneRef::None) => owned_world_roots += 1,
                        x => panic!("lsn {} obj {} root ref {:?}", d.lsn(), i, x),
                    }
                }
            }
            if let Some(r) = xfm_bone_ref(&w, i) {
                assert_ne!(r, BoneRef::Dangling, "lsn {} xfm obj {}", d.lsn(), i);
                xfm_bones += matches!(r, BoneRef::Bone(..)) as usize;
            }
        }
    }
    // totals measured on the disc (regression values)
    println!("TOTALS lights {lights} dir {dir} dir_dyn {dir_dyn} point {point} excl {excl} dynamic {dynamic} owner_bone_roots {owner_bone_roots} attach {attach} xfm_bones {xfm_bones} owned_world_roots {owned_world_roots}");
    // WP-B4: +126 dynamic directional lights, +1st owner-bone roots and XFM bones from the
    // objects now created by loads_b4 (class-6 nested lists, classes 53/54 ALOs)
    assert_eq!((lights, dir, dir_dyn, point), (2855, 41, 126, 2688));
    assert_eq!((excl, dynamic), (545, 168));
    // WP-B5: FUN_001ca370 reparents template copies (root ref -1, placed by the template
    // matrix via vtable +0x12c) to the class-6 record's owner: 3 -> 813 owned world-space roots
    assert_eq!((owner_bone_roots, owned_world_roots), (6629, 813));
    assert_eq!(attach, 0, "no kind 8/0xe bones on the disc");
    assert_eq!(xfm_bones, 4302);
}
