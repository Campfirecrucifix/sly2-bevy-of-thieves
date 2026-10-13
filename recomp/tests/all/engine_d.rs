//! WP-D tests: ASEG (classes 109/110) and ALO-wrapper loads consume exactly their record on all 43 levels;
//! key decode/interpolation vs values derived from the decompiled code; pose sanity. Needs the ISO at
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



use sly2_recomp::engine::anim::aseg::{ee_sub_12f2a8, Aseg, LOAD_ASEG};
use sly2_recomp::engine::anim::pose::*;
use sly2_recomp::engine::anim::load_alo_wrapper;
use sly2_recomp::engine::disc::level_file::memory_stream;
use sly2_recomp::engine::world::class;
use sly2_recomp::engine::world::objects::{Lo, LoadData, ObjectSink};

struct Sink(Vec<Lo>);
impl ObjectSink for Sink {
    fn objects(&self) -> &[Lo] {
        &self.0
    }
    fn push(&mut self, lo: Lo) -> usize {
        if let Some(o) = lo.owner {
            let i = self.0.len();
            self.0[o].children.push(i);
        }
        self.0.push(lo);
        self.0.len() - 1
    }
    fn exists(&self, oid: i32) -> bool {
        self.0.iter().any(|l| l.oid == oid)
    }
    fn get_mut(&mut self, i: usize) -> &mut Lo {
        &mut self.0[i]
    }
}

fn payload(lv: &LoadedLevel, o: &Lo) -> Vec<u8> {
    lv.data[o.payload_offset as usize..(o.payload_offset as usize + o.payload_size as usize)].to_vec()
}

const WRAPPER_CLASSES: &[u16] = &[55, 80, 81, 82, 83, 85, 91, 92, 93, 94, 95, 96, 98, 103];

#[test]
#[ignore = "heavy"]
fn all_levels_anim_records() {
    let (loader, descs) = levels();
    let mut got = Vec::new();
    let mut orth = (0usize, 0usize);
    for d in descs {
        let (lv, w) = load(&loader, d);
        let (mut n, mut ch, mut keys, mut wr) = (0usize, 0usize, 0usize, 0usize);
        for o in w.objects.iter().filter(|o| o.clone_of.is_none()) {
            let ci = class::class(o.class).unwrap();
            if ci.load == LOAD_ASEG {
                let mut s = memory_stream(payload(&lv, o));
                let a: Aseg = ee_sub_12f2a8(&mut s).unwrap_or_else(|e| panic!("lsn {} class {} @{:#x}: {e}", d.lsn(), o.class, o.payload_offset));
                assert!(!s.at_eof(), "lsn {} @{:#x}: ASEG past record", d.lsn(), o.payload_offset);
                assert_eq!(s.tell(), o.payload_size as u32, "lsn {} @{:#x}: ASEG size", d.lsn(), o.payload_offset);
                let (c, k) = a.stats();
                n += 1;
                ch += c;
                keys += k;
                // every vector curve evaluates to finite values over the segment
                for c in &a.channels {
                    if let Some(v) = &c.vec {
                        for i in 0..=4 {
                            if let Some(p) = v.eval(a.length * i as f32 / 4.0, 1.0) {
                                assert!(p.iter().all(|x| x.is_finite()), "lsn {} non-finite curve", d.lsn());
                            }
                        }
                    }
                }
            } else if WRAPPER_CLASSES.contains(&(o.class as u16)) {
                assert!(sly2_recomp::engine::anim::aseg::alo_wrapper_tail(ci.load).is_some(), "class {} load {:#x}", o.class, ci.load);
                let mut s = memory_stream(payload(&lv, o));
                let mut sink = Sink(vec![o.clone()]);
                let (alo, _tail, _ch) = load_alo_wrapper(ci.load, &mut s, &mut sink, 0, o.entry, -1).unwrap().unwrap_or_else(|e| panic!("lsn {} class {}: {e}", d.lsn(), o.class));
                assert!(!s.at_eof());
                assert_eq!(s.tell(), o.payload_size as u32, "lsn {} class {} @{:#x}: wrapper size", d.lsn(), o.class, o.payload_offset);
                wr += 1;
                for b in &alo.base.bones {
                    if let Some(x) = &b.xform {
                        let m = from_4x3(x);
                        assert!(m.iter().flatten().all(|v| v.is_finite()));
                        orth.1 += 1;
                        if orthonormal_error(&m) < 1e-3 {
                            orth.0 += 1;
                        }
                    }
                }
            }
        }
        // ALO bones of already-loaded ALOs too
        for o in &w.objects {
            if let sly2_recomp::engine::world::objects::Payload::Loaded(LoadData::Alo { alo, .. }) = &o.payload {
                for b in &alo.base.bones {
                    if let Some(x) = &b.xform {
                        let m = from_4x3(x);
                        orth.1 += 1;
                        if orthonormal_error(&m) < 1e-3 {
                            orth.0 += 1;
                        }
                        // pose sanity: inverse round trip
                        let id = ee_sub_11c168(&m, &ee_sub_11c318(&m));
                        if orthonormal_error(&m) < 1e-3 {
                            for i in 0..4 {
                                for j in 0..4 {
                                    assert!((id[i][j] - IDENTITY[i][j]).abs() < 1e-2, "lsn {} inverse {id:?}", d.lsn());
                                }
                            }
                        }
                    }
                }
            }
        }
        got.push((d.lsn(), n, ch, keys, wr));
    }
    println!("GOT {got:?}\nORTH {orth:?}");
    assert_eq!(got.len(), 43);
    let tot = got.iter().fold((0, 0, 0, 0), |a, g| (a.0 + g.1, a.1 + g.2, a.2 + g.3, a.3 + g.4));
    println!("TOTAL {tot:?}");
    for (g, e) in got.iter().zip(EXPECT) {
        assert_eq!(*g, *e);
    }
    // D2: only kinds 6/7 (euler extraction FUN_0011bff0) need an orthonormal basis; the
    // other kinds may carry scale/shear (code-based check in tests/engine_d2.rs,
    // anim::motion::basis_must_be_orthonormal). orth is informational.
    assert!(orth.0 <= orth.1);
}

/// (lsn, ASEG objects, channels, keys, wrapper objects) per level. Regression snapshot of this
/// port's output; the code-derived check is the exact record-size consumption above.
/// WP-B4: re-blessed (ASEGs inside class-6 nested lists and classes 53/54 ALOs are now loaded).
const EXPECT: &[(u32, usize, usize, usize, usize)] = &[(335488, 997, 38340, 107435, 60), (350144, 0, 0, 0, 0), (353856, 1104, 43116, 132810, 66), (377568, 829, 35984, 106467, 41), (388768, 857, 36536, 108007, 50), (402976, 784, 34610, 98976, 33), (409312, 829, 36013, 106559, 25), (417504, 815, 35800, 102092, 21), (427328, 980, 38840, 121284, 52), (444896, 873, 38223, 117068, 33), (453760, 896, 38257, 117222, 28), (465056, 1321, 50641, 147195, 19), (482752, 1358, 50718, 159389, 34), (509056, 957, 40210, 121562, 51), (521344, 1087, 40533, 127370, 40), (540160, 955, 38330, 115253, 43), (552768, 725, 33040, 92174, 7), (561312, 1223, 42833, 133931, 49), (578848, 778, 34510, 100713, 7), (589920, 1088, 42172, 120652, 22), (603648, 810, 35662, 105951, 14), (615360, 811, 34489, 100664, 13), (626688, 777, 34458, 100394, 11), (637920, 780, 34482, 100505, 11), (649056, 778, 34458, 100402, 10), (660320, 779, 34457, 100407, 9), (671392, 1067, 42273, 131621, 36), (687488, 907, 38094, 113008, 19), (698912, 923, 38136, 116966, 31), (710528, 1045, 40326, 118899, 32), (720608, 885, 38122, 117120, 32), (730464, 1079, 41142, 127803, 43), (749312, 790, 34394, 98670, 17), (761664, 787, 34405, 98744, 22), (773120, 896, 36793, 106642, 17), (784544, 742, 33726, 94950, 21), (795296, 864, 36752, 106668, 11), (808192, 1053, 38732, 112572, 47), (823200, 989, 38019, 106009, 27), (834976, 735, 33093, 92382, 14), (845728, 818, 35380, 102825, 18), (856896, 734, 33049, 92214, 33), (867648, 1055, 38757, 108628, 46)];

use sly2_recomp::engine::anim::keys::*;
use sly2_recomp::engine::disc::lz::BinaryInputStream;

fn stream(b: Vec<u8>) -> BinaryInputStream {
    memory_stream(b)
}

/// FUN_00117120 at u=0 gives p0 and velocity v0*rate, at u=dt gives p1 and v1*rate
/// (h00(0)=1, h10'=1; h01(1)=1, h11'(1)=1 from the coefficient code).
#[test]
fn hermite_endpoints() {
    let (p0, v0, p1, v1) = ([1.0, 2.0, 3.0], [0.5, 0.0, -1.0], [4.0, -2.0, 0.0], [1.0, 1.0, 1.0]);
    let (p, v, _) = ee_sub_117120(2.0, 0.0, 3.0, p0, v0, p1, v1);
    assert_eq!(p, p0);
    assert_eq!(v, [1.5, 0.0, -3.0]);
    let (p, v, _) = ee_sub_117120(2.0, 2.0, 1.0, p0, v0, p1, v1);
    for i in 0..3 {
        assert!((p[i] - p1[i]).abs() < 1e-5 && (v[i] - v1[i]).abs() < 1e-5);
    }
    // midpoint, zero tangents: h00=h01=0.5
    let (p, _, _) = ee_sub_117120(1.0, 0.5, 1.0, [0.0; 3], [0.0; 3], [2.0; 3], [0.0; 3]);
    assert_eq!(p, [1.0; 3]);
    assert_eq!(ee_sub_117030(1.0, 0.5, 1.0, 0.0, 0.0, 2.0, 0.0).0, 1.0);
}

#[test]
fn key_search_and_frames() {
    assert_eq!(frame_time(60), 60.0 * 0.016666668);
    let t = [0.0, 1.0, 2.0, 3.0];
    // FUN_0012b6d8: k with t[k-1] <= x < t[k], clamped to [1, n-1]
    assert_eq!(key_search(-1.0, |i| t[i], 4), 1);
    assert_eq!(key_search(1.5, |i| t[i], 4), 2);
    assert_eq!(key_search(9.0, |i| t[i], 4), 3);
    assert!(ee_sub_11d4d0(1000.0, 1000.05, SEG_EPS));
    assert!(!ee_sub_11d4d0(1.0, 1.001, SEG_EPS));
}

/// FUN_0012b5e0 layout: u16 n, then n x {u16 frame, 9 f32}; segment setup 0x0012b738 picks
/// out-tangent of key k-1 and in-tangent of key k.
#[test]
fn bez_decode_and_segment() {
    let mut b = vec![2u8, 0];
    for (f, base) in [(0u16, 0.0f32), (30, 10.0)] {
        b.extend(f.to_le_bytes());
        for j in 0..9 {
            b.extend((base + j as f32).to_le_bytes());
        }
    }
    let c = load_vec_curve(1, &mut stream(b)).unwrap();
    let VecCurve::Bez(k) = &c else { panic!() };
    assert_eq!(k[1].t, 30.0 * 0.016666668);
    assert_eq!(k[1].pos, [10.0, 11.0, 12.0]);
    let s = c.segment(0.1).unwrap();
    assert_eq!((s.p0, s.v0, s.p1, s.v1, s.mode), ([0.0, 1.0, 2.0], [6.0, 7.0, 8.0], [10.0, 11.0, 12.0], [13.0, 14.0, 15.0], 1));
    assert_eq!(c.eval(0.0, 1.0).unwrap(), [0.0, 1.0, 2.0]);
    // constant: mode 2 (0x0012b520)
    let mut b = Vec::new();
    for x in [1.0f32, 2.0, 3.0] {
        b.extend(x.to_le_bytes());
    }
    let c = load_vec_curve(0, &mut stream(b)).unwrap();
    assert_eq!(c.segment(5.0).unwrap().mode, 2);
    assert_eq!(c.eval(5.0, 1.0).unwrap(), [1.0, 2.0, 3.0]);
    assert!(load_vec_curve(6, &mut stream(vec![])).is_none());
}

#[test]
fn pose_inverse() {
    let m: Mat4 = [[0.0, 1.0, 0.0, 0.0], [-1.0, 0.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [5.0, 6.0, 7.0, 1.0]];
    let id = ee_sub_11c168(&m, &ee_sub_11c318(&m));
    for i in 0..4 {
        for j in 0..4 {
            assert!((id[i][j] - IDENTITY[i][j]).abs() < 1e-6);
        }
    }
    let w = world_matrices(&IDENTITY, &[m, m], &[None, Some(0)]);
    assert_eq!(w[1], ee_sub_11c168(&m, &m));
}
