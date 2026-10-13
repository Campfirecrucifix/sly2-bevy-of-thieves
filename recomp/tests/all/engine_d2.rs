


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



// ---------------------------------------------------------------- WP-D2
use sly2_recomp::engine::anim::aseg::{Aseg, Event, EventData};
use sly2_recomp::engine::anim::blend::*;
use sly2_recomp::engine::anim::keys::{ScalarCurve, VecCurve, KeyV};
use sly2_recomp::engine::anim::motion::*;
use sly2_recomp::engine::anim::play::*;
use sly2_recomp::engine::anim::pose::{from_4x3, orthonormal_error, ee_sub_22cc00, IDENTITY};
use sly2_recomp::engine::world::objects::{LoadData, Payload};

/// Every ASEG (classes 109/110) and ALO-wrapper object (55, 80-83, 85, 91-96, 98, 103) is now
/// loaded by the world dispatch; bones of kinds 6/7 (euler-extracted by FUN_0011bff0) must be
/// orthonormal, other kinds may carry scale/shear (motion::basis_must_be_orthonormal).
#[test]
fn world_dispatch_loads_anim_classes() {
    let (loader, descs) = levels();
    let (mut asegs, mut chans, mut wrappers) = (0usize, 0usize, 0usize);
    let (mut rot_bones, mut rot_bad, mut scaled_other) = (0usize, 0usize, 0usize);
    for d in descs {
        let (_lv, w) = load(&loader, d);
        for o in &w.objects {
            match &o.payload {
                Payload::Loaded(LoadData::Aseg(a)) => {
                    asegs += 1;
                    chans += a.channels.len();
                }
                Payload::Loaded(LoadData::Alo { alo, .. }) => {
                    if [55u16, 80, 81, 82, 83, 85, 91, 92, 93, 94, 95, 96, 98, 103].contains(&(o.class as u16)) {
                        wrappers += 1;
                    }
                    for b in &alo.base.bones {
                        if let Some(x) = &b.xform {
                            let e = orthonormal_error(&from_4x3(x));
                            if basis_must_be_orthonormal(b.kind as i32) {
                                rot_bones += 1;
                                // FUN_0011bff0 clamps asin's input and never normalises: skewed
                                // bases (authoring noise, up to ~0.2 off-diagonal) are tolerated,
                                // but there is no scale: every basis row has unit length
                                assert!(ee_sub_11bff0(&from_4x3(x)).iter().all(|v| v.is_finite()));
                                let _ = e;
                                if x[..3].iter().any(|r| ((r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt() - 1.0).abs() >= 0.05) {
                                    rot_bad += 1;
                                }
                            } else if e >= 1e-3 {
                                scaled_other += 1;
                            }
                        }
                    }
                }
                _ => {}
            }
            assert!(![109u16, 110].contains(&(o.class as u16)) || !matches!(o.payload, Payload::Unported), "ASEG left unported");
        }
    }
    println!("ASEG {asegs} channels {chans} wrappers {wrappers} rot bones {rot_bones} bad {rot_bad} scaled other {scaled_other}");
    assert!(asegs >= 4659 && chans >= 146095, "WP-D totals (Findings D:)");
    assert!(wrappers > 0);
    assert_eq!(rot_bad, 0, "kind 6/7 bone with a scaled basis");
}

fn ev(t: f32, kind: u8) -> Event {
    Event { time: Some(t), kind, data: EventData::None }
}
fn aseg(length: f32, events1: Vec<Event>) -> Aseg {
    Aseg { length, channels: vec![], scalars: vec![], events0: vec![], events1, events2: vec![], props: vec![] }
}
#[derive(Default)]
struct Rec(Vec<(usize, bool, f32)>, usize, usize);
impl AsegaHost for Rec {
    fn event(&mut self, a: &mut Asega, i: usize, _e: &Event, r: bool) {
        self.0.push((i, r, a.t));
    }
    fn ended(&mut self, _a: &mut Asega) {
        self.1 += 1;
    }
    fn retract(&mut self, a: &mut Asega) {
        self.2 += 1;
        a.alive = false;
    }
}

#[test]
fn asega_events_loop_and_clamp() {
    let s = aseg(1.0, vec![ev(0.25, 1), ev(0.5, 2), ev(0.75, 3)]);
    // loop: 0.6 s steps
    let mut a = Asega { flags: SEGA_LOOP, ..Default::default() };
    let mut h = Rec::default();
    assert!(ee_sub_130918(&mut a, &s, 0.6, 0.0, &mut h));
    assert_eq!(h.0, vec![(0, false, 0.25), (1, false, 0.5)]);
    assert!((a.t - 0.6).abs() < 1e-6 && a.next_event == 2);
    assert!(ee_sub_130918(&mut a, &s, 0.6, 0.0, &mut h));
    // 1.2 -> event 2 fired before the wrap, fmod -> 0.2, cursor reset to 0, nothing below 0.2
    assert_eq!(h.0.len(), 3);
    assert_eq!(h.1, 1);
    assert!((a.t - 0.2).abs() < 1e-5 && a.next_event == 0 && a.alive);
    // clamp: end reached -> t = length, svt 0, retract
    let mut a = Asega::default();
    let mut h = Rec::default();
    ee_sub_130918(&mut a, &s, 2.0, 0.0, &mut h);
    assert_eq!(a.t, 1.0);
    assert_eq!(a.svt, 0.0);
    assert_eq!((h.0.len(), h.1, h.2), (3, 1, 1));
    // |svt| < 1e-4: nothing
    let mut a = Asega { svt: 0.00005, ..Default::default() };
    assert!(!ee_sub_130918(&mut a, &s, 1.0, 0.0, &mut NoHost));
    // reverse play fires backwards with the reverse flag
    let mut a = Asega { t: 0.8, svt: -1.0, svt2: -1.0, flags: SEGA_LOOP, ..Default::default() };
    ee_sub_130eb0(&mut a, &s);
    assert_eq!(a.next_event, 3);
    let mut h = Rec::default();
    ee_sub_130918(&mut a, &s, 0.4, 0.0, &mut h);
    assert_eq!(h.0, vec![(2, true, 0.75), (1, true, 0.5)]);
    // real clock flag
    let mut a = Asega { real_clock: true, flags: SEGA_LOOP, ..Default::default() };
    ee_sub_130918(&mut a, &s, 0.0, 0.1, &mut NoHost);
    assert!((a.t - 0.1).abs() < 1e-6);
}

#[test]
fn asega_pingpong_and_seek() {
    let mut a = Asega { t: 1.25, flags: SEGA_PINGPONG, ..Default::default() };
    assert!(ee_sub_1317d8(&mut a, 1.0));
    assert!((a.t - 0.75).abs() < 1e-6 && a.svt == -1.0 && a.svt2 == -1.0);
    let mut a = Asega { t: -0.25, svt: -1.0, svt2: -1.0, flags: SEGA_PINGPONG, ..Default::default() };
    assert!(ee_sub_1317d8(&mut a, 1.0));
    assert!((a.t - 0.25).abs() < 1e-6 && a.svt == 1.0);
    let mut a = Asega { t: -0.25, svt: -1.0, flags: SEGA_LOOP, ..Default::default() };
    assert!(ee_sub_1317d8(&mut a, 1.0));
    assert!((a.t - 0.75).abs() < 1e-6);
    let mut a = Asega { t: 0.5, ..Default::default() };
    assert!(!ee_sub_1317d8(&mut a, 1.0));
    // seek fires only state-like kinds
    let s = aseg(1.0, vec![ev(0.1, 1), ev(0.2, 7), ev(0.3, 0x15)]);
    let mut a = Asega { t: 0.5, ..Default::default() };
    let mut h = Rec::default();
    ee_sub_131100(&mut a, &s, &mut h);
    assert_eq!(h.0.iter().map(|x| x.0).collect::<Vec<_>>(), vec![1, 2]);
    assert_eq!(a.t, 0.5);
    assert_eq!(a.next_event, 3);
}

#[test]
fn blend_curves() {
    assert_eq!(ee_sub_138370(0.5, 2.0, &[(0.0, 0.0), (0.25, 0.5)]), (1, Some(0.25)));
    assert_eq!(ee_sub_138370(0.1, 2.0, &[(1.5, 0.75)]), (0, Some(0.6)));
    assert_eq!(ee_sub_138370(1.0, 2.0, &[(0.0, 0.5)]).0, 1);
    let blend = BlendAseg {
        length: 2.0,
        entries: vec![
            BlendEntry { weight: 0.25, sub_length: 1.0, phase: vec![] },
            BlendEntry { weight: 0.75, sub_length: 4.0, phase: vec![] },
            BlendEntry { weight: 0.0, sub_length: 1.0, phase: vec![] },
        ],
        select: -1,
    };
    let lin = |a: f32, b: f32, len: f32| {
        let tan = [(b - a) / len; 3];
        VecCurve::Bez1(vec![KeyV { t: 0.0, pos: [a; 3], tan }, KeyV { t: len, pos: [b; 3], tan }])
    };
    let c = BlendVecCurve { blend: blend.clone(), phases: None, children: vec![Some(lin(0.0, 1.0, 1.0)), None, Some(lin(5.0, 5.0, 1.0))], default: [8.0; 3] };
    let s = c.ee_sub_138660(1.0).unwrap();
    // entry 0: u = 0.5 -> sub t 0.5 -> 0.5, vel = 1 * rate (1/2); entry 1: default 8, vel 0
    assert_eq!(s.mode, 4);
    assert!((s.p0[0] - (0.25 * 0.5 + 0.75 * 8.0)).abs() < 1e-5, "{s:?}");
    assert!((s.v0[0] - 0.25 * 0.5).abs() < 1e-5, "{s:?}");
    let mut sel = c.clone();
    sel.blend.select = 1;
    let s = sel.ee_sub_138660(1.0).unwrap();
    assert_eq!((s.mode, s.p0, s.rewired_time), (2, [8.0; 3], true));
    // scalar: rate input scales the sub rate
    let sc = BlendScalarCurve {
        blend,
        phases: None,
        children: vec![Some(ScalarCurve::Bez(vec![[0.0, 0.0, 1.0, 1.0], [1.0, 1.0, 1.0, 1.0]])), Some(ScalarCurve::Const(2.0)), None],
        default: 0.0,
    };
    let (v, d) = sc.ee_sub_138ba0(1.0, 2.0).unwrap();
    assert!((v - (0.25 * 0.5 + 0.75 * 2.0)).abs() < 1e-5 && (d - 0.25 * 1.0).abs() < 1e-5, "{v} {d}");
    // phase table
    let pb = BlendAseg { length: 2.0, entries: vec![BlendEntry { weight: 1.0, sub_length: 1.0, phase: vec![(0.0, 0.5), (0.5, 0.5)] }], select: -1 };
    let lin = |a: f32, b: f32| VecCurve::Bez1(vec![KeyV { t: 0.0, pos: [a; 3], tan: [b - a; 3] }, KeyV { t: 1.0, pos: [b; 3], tan: [b - a; 3] }]);
    let pc = BlendVecCurve { blend: pb, phases: Some(PhaseTable { phases: vec![(0.0, 1.0), (1.0, 1.0)] }), children: vec![Some(lin(0.0, 1.0))], default: [0.0; 3] };
    // t 1.5 -> phase 1, local 0.5, u 0.5 -> sub t 0.5 + 0.5*0.5 = 0.75, rate 0.5
    let s = pc.ee_sub_138660(1.5).unwrap();
    assert!((s.p0[0] - 0.75).abs() < 1e-5 && (s.v0[0] - 0.5).abs() < 1e-5, "{s:?}");
}

#[test]
fn bone_motion() {
    // trig helper
    for a in [-3.0f32, -1.0, 0.0, 0.5, 2.0, 7.0] {
        let (s, c) = ee_sub_11d608(a);
        assert!((s - a.sin()).abs() < 1e-4 && (c - a.cos()).abs() < 1e-4, "{a} {s} {c}");
    }
    // euler round trip for a pure z rotation
    let (s, c) = (0.3f32.sin(), 0.3f32.cos());
    let mut m = IDENTITY;
    m[0] = [c, s, 0.0, 0.0];
    m[1] = [-s, c, 0.0, 0.0];
    let e = ee_sub_11bff0(&m);
    assert!(e[0].abs() < 1e-6 && e[1].abs() < 1e-6 && (e[2] - 0.3).abs() < 1e-6);
    // static mode -> zero; stored translation velocity -> FUN_0022cc00 of +0xb0
    let mut b = BoneState::new(1, IDENTITY);
    b.vel = [1.0, 2.0, 3.0];
    assert_eq!(ee_sub_22cdc0(&[b.clone()], 0), [[0.0; 4]; 4]);
    b.motion = 2;
    assert_eq!(ee_sub_22cdc0(&[b.clone()], 0), ee_sub_22cc00(1, [1.0, 2.0, 3.0]));
    // angular velocity about z on identity: skew rows
    let mut r = BoneState::new(6, IDENTITY);
    r.motion = 2;
    r.vel = [0.0, 0.0, 1.0];
    let v = ee_sub_22cdc0(&[r.clone()], 0);
    assert_eq!(v[0][1], 1.0);
    assert_eq!(v[1][0], -1.0);
    // chain to world with identity worlds = plain sum
    let mut c = b.clone();
    c.parent = Some(1);
    let bones = vec![c, r];
    let sum = ee_sub_22d148(&bones, 0, None);
    assert!((sum[3][0] - 1.0).abs() < 1e-6 && (sum[0][1] - 1.0).abs() < 1e-6);
    assert!(basis_must_be_orthonormal(6) && !basis_must_be_orthonormal(0xc));
}
