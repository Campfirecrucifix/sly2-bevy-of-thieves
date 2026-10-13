//! WP-C6 tests: level geometry census (records in the level vs meshes drawn), particle colours,
//! texture/material gaps. Needs the ISO at "<root>/Sly 2 - Band of Thieves (USA).iso" and
//! db/sly2.sqlite (level list).
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

fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

fn load(loader: &LevelLoader, d: Descriptor) -> (LoadedLevel, World) {
    let lv = crate::common::load_blocking(&loader, d).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap_or_else(|e| panic!("lsn {}: {e}", d.lsn()));
    (lv, w)
}

use sly2_recomp::engine::gfx::mesh::{convert, MeshRole};
use sly2_recomp::engine::world::alo::MeshData;
use sly2_recomp::engine::world::objects::{LoadData, Payload};

#[test]
#[ignore]
fn c6_probe() {
    let (loader, descs) = levels();
    println!("lsn | alo act/inact surfrec | clone(alo-src) act/placed | rec skipped other empty surf ink shadow");
    for d in descs {
        let (_lv, w) = load(&loader, d);
        let (mut a_act, mut a_in, mut surf_rec) = (0, 0, 0);
        let (mut cl, mut cl_act, mut cl_placed) = (0, 0, 0);
        let (mut skipped, mut other, mut empty, mut surf, mut ink, mut sh) = (0, 0, 0, 0, 0, 0);
        let has_surf = |i: usize| -> bool {
            let Payload::Loaded(LoadData::Alo { alo, .. }) = &w.objects[i].payload else { return false };
            alo.mesh.records.iter().any(|r| convert(&r.data).iter().any(|p| p.role == MeshRole::Surface))
        };
        for (i, o) in w.objects.iter().enumerate() {
            match &o.payload {
                Payload::Loaded(LoadData::Alo { alo, .. }) => {
                    let mut any = false;
                    for r in &alo.mesh.records {
                        match &r.data {
                            MeshData::Skipped { .. } => skipped += 1,
                            MeshData::Other => other += 1,
                            dd => {
                                let p = convert(dd);
                                if p.is_empty() {
                                    empty += 1
                                } else if p.iter().any(|p| p.role == MeshRole::Surface) {
                                    surf += 1;
                                    any = true;
                                } else if p[0].role == MeshRole::InkEdges {
                                    ink += 1
                                } else {
                                    sh += 1
                                }
                            }
                        }
                    }
                    if any {
                        surf_rec += 1;
                        if w.is_active(i) { a_act += 1 } else { a_in += 1 }
                    }
                }
                Payload::Clone => {
                    if o.clone_of.is_some_and(has_surf) {
                        cl += 1;
                        if w.is_active(i) { cl_act += 1 }
                        if w.life.placed.contains_key(&i) { cl_placed += 1 }
                    }
                }
                _ => {}
            }
        }
        println!("{} | {a_act}/{a_in} {surf_rec} | {cl} {cl_act}/{cl_placed} | {skipped} {other} {empty} {surf} {ink} {sh}", d.lsn());
    }
}

#[test]
#[ignore]
fn c6_probe_t20() {
    use sly2_recomp::engine::gfx::attach::{affine_from_bytes, BoneWorlds};
    use sly2_recomp::engine::gfx::scene::mul;
    let lsn: u32 = std::env::var("C6_LSN").ok().and_then(|s| s.parse().ok()).unwrap_or(603648);
    let (loader, descs) = levels();
    let d = descs.into_iter().find(|d| d.lsn() == lsn).unwrap();
    let (_lv, w) = load(&loader, d);
    let mut bw = BoneWorlds::default();
    let mut by_class: std::collections::BTreeMap<u16, (usize, usize, usize)> = Default::default();
    for (i, o) in w.objects.iter().enumerate() {
        let src = match (&o.payload, o.clone_of) {
            (Payload::Clone, Some(s)) => s,
            _ => continue,
        };
        let Payload::Loaded(LoadData::Alo { alo, .. }) = &w.objects[src].payload else { continue };
        let ns: usize = alo.mesh.records.iter().map(|r| convert(&r.data).iter().filter(|p| p.role == MeshRole::Surface).count()).sum();
        if ns == 0 {
            continue;
        }
        let e = by_class.entry(o.class).or_default();
        e.0 += 1;
        e.1 += w.is_active(i) as usize;
        e.2 += w.life.placed.contains_key(&i) as usize;
        let m = match w.life.placed.get(&i).map(affine_from_bytes) {
            Some(pm) => mul(&bw.get(&w, i, 0), &pm),
            None => bw.get(&w, i, 0),
        };
        let ms = bw.get(&w, src, 0);
        println!(
            "obj {i} cls {} oid {} entry {} src {src} act {} placed {} owner {:?} prims {ns} r {:.0} pos {:?} srcpos {:?}",
            o.class, o.oid, o.entry, w.is_active(i), w.life.placed.contains_key(&i), o.owner, alo.mesh.radius, m[3].map(|x| x as i32), ms[3].map(|x| x as i32)
        );
    }
    println!("{by_class:?}");
}

use sly2_recomp::engine::gfx::scene::{copy_drawn_elsewhere, LevelScene};

/// C6 census: every clone with surface prims in its template is classified: drawn as a scene copy
/// (template copy), drawn by npcdraw / npcdraw::objects (class), or a pool copy (FUN_0013e510 copies
/// 2..n, placed by its class logic). t20 (603648) must draw its placed floor / wall copies.
#[test]
#[ignore = "heavy"]
fn c6_census_template_copies() {
    let (loader, descs) = levels();
    let (mut tot_drawn, mut tot_else, mut tot_pool) = (0, 0, 0);
    println!("lsn | scene objs surf-prims | clones-with-surf: copy-drawn elsewhere pool | copy prims");
    for d in descs {
        let (lv, w) = load(&loader, d);
        let sc = LevelScene::build(&lv, &w);
        let surf = |si: usize| sc.objects[si].prims.iter().filter(|p| p.prim.role == MeshRole::Surface).count();
        let obj_prims: usize = (0..sc.objects.len()).map(surf).sum();
        let scene_of: std::collections::HashMap<usize, usize> = sc.objects.iter().enumerate().map(|(i, so)| (so.object, i)).collect();
        let copies: std::collections::HashSet<usize> = sc.copies.iter().map(|c| c.object).collect();
        let tis: std::collections::HashSet<usize> = w.template_instances.iter().flat_map(|t| t.objects.iter().copied()).collect();
        let (mut drawn, mut elsewhere, mut pool) = (0, 0, 0);
        for (i, o) in w.objects.iter().enumerate() {
            let (Payload::Clone, Some(src)) = (&o.payload, o.clone_of) else { continue };
            let Some(&si) = scene_of.get(&src) else { continue };
            if surf(si) == 0 {
                continue;
            }
            if copies.contains(&i) {
                drawn += 1
            } else if copy_drawn_elsewhere(o.class) {
                elsewhere += 1
            } else {
                assert!(!tis.contains(&i) || !w.life.placed.contains_key(&i), "lsn {} copy {i} class {} placed but not drawn", d.lsn(), o.class);
                pool += 1
            }
        }
        let copy_prims: usize = sc.copies.iter().map(|c| surf(c.source)).sum();
        for c in &sc.copies {
            assert!(c.world.iter().flatten().all(|x| x.is_finite()));
        }
        println!("{} | {} {obj_prims} | {drawn} {elsewhere} {pool} | {copy_prims}", d.lsn(), sc.objects.len());
        if d.lsn() == 603648 {
            let cls: std::collections::BTreeSet<u16> = sc.copies.iter().map(|c| w.objects[c.object].class).collect();
            assert!(drawn >= 39 && cls.contains(&48) && cls.contains(&98), "t20 copies {drawn} {cls:?}");
        }
        tot_drawn += drawn;
        tot_else += elsewhere;
        tot_pool += pool;
    }
    println!("total copies drawn {tot_drawn}, drawn elsewhere {tot_else}, pool {tot_pool}");
    assert!(tot_drawn > 1000);
}

#[test]
#[ignore]
fn c6_probe_empty() {
    let (loader, descs) = levels();
    for d in descs.into_iter().filter(|d| [603648u32, 427328].contains(&d.lsn())) {
        let (lv, w) = load(&loader, d);
        let sc = LevelScene::build(&lv, &w);
        let e = sc.objects.iter().flat_map(|o| &o.prims).filter(|p| p.prim.role == MeshRole::Surface && (p.prim.positions.is_empty() || p.prim.indices.is_empty())).count();
        let ec = sc.copies.iter().flat_map(|c| &sc.objects[c.source].prims).filter(|p| p.prim.role == MeshRole::Surface && (p.prim.positions.is_empty() || p.prim.indices.is_empty())).count();
        println!("{} empty surface prims {e}, in copies {ec}", d.lsn());
    }
}

/// C6 census (item 3): every drawn surface prim of all 43 levels has a shader whose base pass decodes
/// (no missing shader / frame / CLUT); prints per level [no-shader, bad-shader, no-frame, decode-err, ok].
#[test]
#[ignore = "heavy"]
fn c6_census_textures() {
    let (loader, descs) = levels();
    println!("lsn | surf prims: no-shader bad-shader no-frame decode-err ok | worst classes");
    for d in descs {
        let (lv, w) = load(&loader, d);
        let sc = LevelScene::build(&lv, &w);
        let mut n = [0usize; 5];
        let mut bad: std::collections::BTreeMap<(u16, usize), usize> = Default::default();
        for so in &sc.objects {
            for p in so.prims.iter().filter(|p| p.prim.role == MeshRole::Surface) {
                let t = &sc.textures[so.entry];
                let k = match p.prim.shader {
                    None => 0,
                    Some(s) if t.shaders.get(s as usize).is_none() => 1,
                    Some(s) => match t.decode_shader(s as usize, 0) {
                        None => 2,
                        Some(Err(_)) => 3,
                        Some(Ok(_)) => 4,
                    },
                };
                n[k] += 1;
                if k < 4 {
                    *bad.entry((w.objects[so.object].class, k)).or_default() += 1;
                }
            }
        }
        println!("{} | {:?} | {:?}", d.lsn(), n, bad);
        assert!(n[4] > 0 && n[..4].iter().all(|&x| x == 0), "lsn {}: {n:?} {bad:?}", d.lsn());
    }
}

use sly2_recomp::engine::classes::k21_28::C2cb5d8;
use sly2_recomp::engine::gfx::particles::{ee_sub_1394c8_look, frames_padded, particle_color, GS_SET_ADD, GS_SET_BLEND};

/// C6 item 2: FUN_001394c8 / FUN_00139370 look constants and the colour of a particle over its life.
#[test]
fn c6_particle_look() {
    assert_eq!([1, 2, 3, 4, 5, 9, 16, 17, 40].map(frames_padded), [1, 2, 4, 4, 8, 16, 16, 16, 16]);
    // no look set: white
    let s = C2cb5d8::default();
    assert_eq!(particle_color(&s, 2.0, 1.0), [1.0; 4]);
    // two keys (red at birth, transparent blue at death), interpolated (+0xcc bit 0), blend set
    let mut s = C2cb5d8::default();
    ee_sub_1394c8_look(&mut s, Some(&[[255, 0, 0, 255], [0, 0, 255, 0]]), 1, 0, [0; 4], 1, 2.0);
    assert_eq!(s.m.u(0x438), GS_SET_BLEND);
    assert_eq!(s.m.i(0x3b0), 2);
    assert_eq!(s.m.u(0x3b4), u32::from_le_bytes([0, 0, 128, 0]));
    assert_eq!(particle_color(&s, 3.0, 1.0), [1.0, 0.0, 0.0, 1.0]); // 2 s left of 2: birth
    assert_eq!(particle_color(&s, 1.0, 1.0), [0.0, 0.0, 1.0, 0.0]); // death
    assert_eq!(particle_color(&s, 2.0, 1.0), [0.5, 0.0, 0.5, 0.5]);
    // no template keys: the shader's RGBA, additive set (shader +8 bit 0)
    let mut s = C2cb5d8::default();
    ee_sub_1394c8_look(&mut s, None, 0, 1, [128, 64, 0, 128], 3, 1.0);
    assert_eq!(s.m.u(0x438), GS_SET_ADD);
    assert_eq!(s.m.i(0x1a0), 4);
    assert_eq!(particle_color(&s, 1.0, 0.5), [0.5, 0.25, 0.0, 0.5]);
}
