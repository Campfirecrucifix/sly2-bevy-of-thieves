//! WP-I5: camera tracks (class-15 C2db6b8 -> C2db858), binocucom zoom, camera collision / Hor+.

use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::world::loads_b4::B4Load;
use sly2_recomp::engine::world::objects::{LoadData, Payload};
use sly2_recomp::engine::world::props::PropValue;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::collections::BTreeMap;
use std::sync::Arc;

fn tables() -> (LevelLoader, Vec<sly2_recomp::engine::disc::descriptor::Descriptor>) {
    let iso = Arc::new(IsoImage::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso")).expect("ISO"));
    let t = sly2_recomp::engine::disc::descriptor::static_table(&iso.read_file("SCUS_973.16").unwrap()).unwrap();
    let l = LevelLoader::new(iso, ModOverlay::default());
    let boot = crate::common::load_blocking(&l, t[25]).unwrap();
    let d = boot.header.level_table.entries.iter().map(|e| e.desc).collect();
    (l, d)
}

#[test]
#[ignore]
fn i5_probe_tracks() {
    let (l, ds) = tables();
    let mut fields: BTreeMap<i16, usize> = BTreeMap::new();
    let mut grf: BTreeMap<String, usize> = BTreeMap::new();
    let mut nested_cls: BTreeMap<u16, usize> = BTreeMap::new();
    let mut n = 0;
    for (i, d) in ds.iter().enumerate() {
        let Ok(lv) = crate::common::load_blocking(&l, *d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        for o in &w.objects {
            let Payload::Loaded(LoadData::B4(b)) = &o.payload else { continue };
            let B4Load::L220fd0 { props, nested, .. } = &**b else { continue };
            n += 1;
            let mut g = String::from("7");
            for p in props {
                *fields.entry(p.field).or_default() += 1;
                if let PropValue::Scalar(b) = &p.value {
                    if p.field == 197 { g = format!("{}", u32::from_le_bytes([b[0], b[1], b[2], b[3]])); }
                }
                if matches!(p.field, 176 | 190 | 198 | 199 | 210) && i < 50 {
                    eprintln!("lv {i} oid {:#x} field {} {:?}", o.oid, p.field, p.value);
                }
            }
            *grf.entry(format!("{g} nested={}", !nested.is_empty())).or_default() += 1;
            for &c in nested { *nested_cls.entry(w.objects[c].class).or_default() += 1; }
        }
        eprintln!("level {i} done");
    }
    eprintln!("tracks {n}\nfields {fields:?}\ngrf {grf:?}\nnested {nested_cls:?}");
}

#[test]
#[ignore]
fn i5_probe_trackset() {
    use sly2_recomp::engine::camera::track::TrackSet;
    for f in [176i16, 190, 197, 200, 201, 214] {
        eprintln!("field {f} {:?}", sly2_recomp::engine::world::props::field(f));
    }
    let (l, ds) = tables();
    for (i, d) in ds.iter().enumerate().take(12) {
        let Ok(lv) = crate::common::load_blocking(&l, *d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        let ts = TrackSet::load(&w);
        let vol = ts.tracks.iter().filter(|t| !t.bsp.nodes.is_empty()).count();
        let f13c: Vec<i32> = ts.tracks.iter().map(|t| t.f_13c).filter(|&x| x != -1).collect();
        let simple = ts.tracks.iter().filter(|t| t.track.simple()).count();
        eprintln!("level {i}: tracks {} vol {vol} simple {simple} f13c!=-1 {f13c:?} n_params>1 {}", ts.tracks.len(), ts.tracks.iter().filter(|t| t.track.n_params > 1).count());
    }
}

fn inside_point(t: &sly2_recomp::engine::camera::track::RegionTrack) -> Option<[f32; 3]> {
    use sly2_recomp::engine::physics::point::FUN_00118fb0;
    let v = &t.geom.verts;
    let n = v.len() as f32;
    let c = [0, 1, 2].map(|j| v.iter().map(|p| p[j]).sum::<f32>() / n);
    let mut cand = vec![c];
    for a in v.iter().take(40) {
        cand.push([0, 1, 2].map(|j| (a[j] + c[j]) * 0.5));
    }
    let l = cand.into_iter().find(|p| FUN_00118fb0(&t.geom, &t.bsp, 0, 0x7fffffff, *p).is_some())?;
    let m = sly2_recomp::engine::gfx::scene::invert(&t.inv);
    Some([0, 1, 2].map(|j| l[0] * m[0][j] + l[1] * m[1][j] + l[2] * m[2][j] + m[3][j]))
}

/// FUN_00221298 / FUN_00221178 on real level-0 tracks: entering a volume pushes the track's follow at
/// +0x128 + 2 with pv = the track, leaving revokes it by pv (grf 8); +0x13c == 0 tracks never run.
#[test]
fn i5_region_tracks_level0() {
    use sly2_recomp::engine::camera::track::{TrackCm, TrackSet};
    let (l, ds) = tables();
    let lv = crate::common::load_blocking(&l, ds[0]).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    let mut ts = TrackSet::load(&w);
    assert_eq!(ts.tracks.len(), 22);
    assert_eq!(ts.tracks.iter().filter(|t| t.f_13c == 0).count(), 5);
    let far = [1.0e7, 1.0e7, 1.0e7];
    assert!(ts.update(7, far, false, 0.0).is_empty(), "outside every volume: state stays 0");
    let mut hit = 0;
    for k in 0..ts.tracks.len() {
        if ts.tracks[k].f_13c != -1 {
            continue;
        }
        let Some(p) = inside_point(&ts.tracks[k]) else { continue };
        let (pv, cpp) = (ts.tracks[k].pv, ts.tracks[k].track.f_128 + 2);
        let a = ts.update(7, p, false, 1.0);
        assert!(a.contains(&TrackCm::Push { cpp, pv }), "{a:?}");
        assert_eq!(ts.tracks[k].f_134, 1);
        assert!(ts.update(7, p, false, 1.1).is_empty(), "no change while inside");
        let a = ts.update(7, far, false, 2.0);
        assert!(a.contains(&TrackCm::Revoke { pv }), "{a:?}");
        assert!(ts.tracks.iter().all(|t| t.f_134 == 0));
        hit += 1;
    }
    eprintln!("volumes entered: {hit}");
    assert!(hit >= 5, "{hit}");
}

/// Field 190 setter 0x00221960 writes the entry's +0x70 = +0x74 = v, +0x78 = 1.
#[test]
fn i5_field_190() {
    use sly2_recomp::engine::world::props::{Prop, PropValue};
    let p = vec![Prop { field: 190, value: PropValue::Scalar(0.4f32.to_le_bytes().to_vec()) }];
    let t = sly2_recomp::engine::camera::follow::follow_track(&p, false);
    let w = t.params.to_words();
    assert_eq!([w[28], w[29], w[30]], [0.4f32.to_bits(), 0.4f32.to_bits(), 1.0f32.to_bits()]);
    assert_eq!(t.n_params, 1);
}

/// Priority stack: the free follow (cpp 1, pv 0) under two tracks; the highest cpp is active and revoking
/// it by pv (grf 8) falls back to the next one.
#[test]
fn i5_track_priority_stack() {
    use sly2_recomp::engine::camera::cm::*;
    use sly2_recomp::engine::camera::follow::*;
    use sly2_recomp::engine::camera::CmClock;
    let clk = |t| CmClock { t, dt: 1.0 / 60.0, dt_real: 1.0 / 60.0 };
    let mut cm = Cm::new();
    let id = add_policy(&mut cm, Box::new(CplcyFollow::new(Box::new(NoFollowWorld))));
    assert!(FUN_0014e328(&mut cm, 1, id, 7, 0, clk(0.0)));
    FUN_0014e328(&mut cm, 2, id, 7, 11, clk(0.1));
    FUN_0014e328(&mut cm, 5, id, 7, 12, clk(0.2));
    assert_eq!(cm.acpr.first().map(|r| (r.cpp, r.pv)), Some((5, 12)));
    FUN_0014e4c8(&mut cm, 8, -1, 0, 0, 12, clk(0.3));
    assert_eq!(cm.acpr.first().map(|r| (r.cpp, r.pv)), Some((2, 11)));
    FUN_0014e4c8(&mut cm, 8, -1, 0, 0, 11, clk(0.4));
    assert_eq!(cm.acpr.first().map(|r| (r.cpp, r.pv)), Some((1, 0)));
}

#[test]
#[ignore]
/// WP-I5 probe: Sly's JT subtree, which objects FUN_00189530 hides.
fn i5_probe_binoc_hide() {
    use sly2_recomp::engine::playerdraw::{find_jt, subtree, vis::JtVis};
    let (l, ds) = tables();
    let lv = crate::common::load_blocking(&l, ds[0]).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    let jt = find_jt(&w).unwrap();
    let mut v = JtVis::load(&w, jt);
    let before: Vec<usize> = subtree(&w, jt).into_iter().filter(|&o| v.drawn(o)).collect();
    v.ee_sub_189530(&w, true);
    let mut depth = std::collections::HashMap::new();
    depth.insert(jt, 0usize);
    for o in subtree(&w, jt) {
        let d = depth[&o];
        for &c in &w.objects[o].children { depth.insert(c, d + 1); }
        let ob = &w.objects[o];
        use sly2_recomp::engine::world::objects::{LoadData, Payload};
        let Payload::Loaded(LoadData::Alo { alo, .. }) = &ob.payload else { continue };
        let recs: Vec<(usize, bool)> = (0..alo.mesh.records.len()).map(|r| (r, v.record_drawn(o, r, None))).collect();
        println!("{}{} oid {:#x} class {} drawn0 {} drawn1 {} words {:x?} recs {} drawn {} r {:.1} c {:?}", "  ".repeat(d), o, ob.oid, ob.class, before.contains(&o), v.drawn(o), alo.mesh.words, recs.len(), recs.iter().filter(|r| r.1).count(), alo.mesh.radius, alo.mesh.center);
    }
}

#[test]
#[ignore]
/// WP-I5 probe: events of the binocucom enter / exit ASEGs (JT +0xf0c / +0xf10).
fn i5_probe_binoc_aseg() {
    use sly2_recomp::engine::playerdraw::{aseg_of, find_jt, jt_anim_members};
    let (l, ds) = tables();
    let lv = crate::common::load_blocking(&l, ds[0]).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    let jt = find_jt(&w).unwrap();
    let m = jt_anim_members(&w, jt);
    for f in [0xf0cu32, 0xf10] {
        let Some(&o) = m.get(&f) else { println!("{f:#x}: none"); continue };
        let a = aseg_of(&w, o).unwrap();
        println!("{f:#x}: obj {o} oid {:#x} len {} ev1 {}", w.objects[o].oid, a.length, a.events1.len());
        for e in &a.events1 { println!("   {:?}", e); }
    }
}

#[test]
/// WP-I5: binocucom look (FUN_00189530 hide) switches off Sly's whole JT subtree (body records included)
/// and leaving restores exactly the previous draw state.
fn i5_binoc_hides_body() {
    use sly2_recomp::engine::playerdraw::{find_jt, subtree, vis::JtVis};
    let (l, ds) = tables();
    let lv = crate::common::load_blocking(&l, ds[0]).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    let jt = find_jt(&w).unwrap();
    let mut v = JtVis::load(&w, jt);
    let all = subtree(&w, jt);
    let before: Vec<bool> = all.iter().map(|&o| v.drawn(o)).collect();
    assert!(v.drawn(jt));
    v.ee_sub_189530(&w, true);
    assert!(all.iter().all(|&o| !v.drawn(o)));
    v.ee_sub_189530(&w, false);
    assert_eq!(all.iter().map(|&o| v.drawn(o)).collect::<Vec<_>>(), before);
}

#[test]
/// WP-I5 item 3: Hor+ holds in every camera mode. Each policy only sets the vertical fov (SetCmFov
/// 0x14f5a0) and near clip (0x14f610); RecalcCmFrustrum 0x14db70 widens x only. Modes: follow (fov 1.0,
/// near 10 default), track fov +0x188 (0.7..1.4 seen in data range), close-up C2cd450 (near 25, fov 1.15),
/// binocucom look C2cd4e0 (near 49, zoomed fov = fov / 7).
fn i5_hor_plus_every_mode() {
    use sly2_recomp::engine::camera::cm::*;
    use sly2_recomp::engine::camera::widescreen::*;
    use sly2_recomp::engine::camera::CmClock;
    let clk = CmClock { t: 0.0, dt: 1.0 / 60.0, dt_real: 1.0 / 60.0 };
    let modes = [(1.0f32, 10.0f32), (0.7, 10.0), (1.4, 10.0), (f32::from_bits(0x3f933333), 25.0), (1.0, 49.0), (1.0 / 7.0, 49.0)];
    for (fov, near) in modes {
        let at = |w: f32, h: f32| {
            let mut cm = Cm::new();
            apply_hor_plus(&mut cm, w, h, clk);
            FUN_0014f610(&mut cm, near, clk);
            FUN_0014f5a0(&mut cm, fov, clk);
            (vertical_fov(&cm), horizontal_fov(&cm), cm.s_near_radius, cm.y_scale, cm.x_scale)
        };
        let (v16, h16, r16, y16, x16) = at(1920.0, 1080.0);
        let (v32, h32, r32, y32, x32) = at(5120.0, 1440.0);
        assert!((v16 - v32).abs() < 1e-6, "vertical fov constant ({fov}, {near})");
        assert!((y16 - y32).abs() < 1e-7);
        assert!((x32 / x16 - 2.0).abs() < 1e-4, "x extent doubles 16:9 -> 32:9");
        assert!(h32 > h16);
        // near-clip rectangle radius (camera push-in probe) covers the wider near plane
        let rect = |y: f32, x: f32| (x * x + y * y).sqrt() * near + 1.0;
        assert!((r16 - rect(y16, x16)).abs() < 1e-3 && (r32 - rect(y32, x32)).abs() < 1e-3);
        println!("fov {fov:.3} near {near}: v {:.2} deg, h 16:9 {:.2} / 32:9 {:.2} deg, near r {r16:.1} / {r32:.1}", v16.to_degrees(), h16.to_degrees(), h32.to_degrees());
    }
}
