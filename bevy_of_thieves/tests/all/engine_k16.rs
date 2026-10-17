//! WP-K16: A7 class fixes (class-76 swarm rails, class-81 launch start / target), class-10 path data
//! (classes/shared/path10.rs) driving class 85 in the real engine host, class-55 clusters.
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::{ClassHost, ClassSet, Inst};
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

const DT: f32 = 1.0 / 60.0;

fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// distance from `q` to a polyline rail in its world frame
fn rail_dist(p: &sly2_recomp::engine::player::p5_attach::Path, q: [f32; 3]) -> f32 {
    let wp = |v: [f32; 3]| std::array::from_fn::<f32, 3, _>(|j| v[0] * p.rot[0][j] + v[1] * p.rot[1][j] + v[2] * p.rot[2][j] + p.pos[j]);
    let pts: Vec<[f32; 3]> = p.pts.iter().map(|&v| wp(v)).collect();
    let mut best = f32::MAX;
    for s in pts.windows(2) {
        let d = [s[1][0] - s[0][0], s[1][1] - s[0][1], s[1][2] - s[0][2]];
        let l2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        let t = if l2 > 0.0 { (((q[0] - s[0][0]) * d[0] + (q[1] - s[0][1]) * d[1] + (q[2] - s[0][2]) * d[2]) / l2).clamp(0.0, 1.0) } else { 0.0 };
        best = best.min(dist(q, std::array::from_fn(|j| s[0][j] + d[j] * t)));
    }
    best
}

/// every class-76 swarm rides its own rails (FUN_0015e938 kind 4 = nearest relative, not the first object with
/// the oid) in the rails' world frame (owner chain): riders sit on their rail in world space, away from the
/// origin, and swarms sharing rail oids get different rails
#[test]
fn k16_c76_riders_on_own_rails_in_world() {
    let (loader, descs) = levels();
    let (mut swarms, mut riders, mut shared_oid_distinct) = (0, 0, 0);
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        for _ in 0..60 {
            s.g.dt = DT;
            s.g.t += DT;
            s.frame(&mut h);
        }
        let mut by_oid: std::collections::HashMap<i32, Vec<[f32; 3]>> = Default::default();
        for (i, x) in &s.k14 {
            let Inst::K14(c) = x else { continue };
            swarms += 1;
            for g in &c.groups {
                let p = g.path.as_ref().unwrap_or_else(|| panic!("swarm {i} group oid {} has no rail", g.oid));
                by_oid.entry(g.oid).or_default().push(p.pos);
                for &r in &g.list {
                    let q = c.riders[r].pos;
                    let e = rail_dist(p, q);
                    assert!(e < g.r_20 + 50.0, "swarm {i} rider {r} at {q:?} is {e} from its rail");
                    assert!(dist(q, [0.0; 3]) > 1000.0, "swarm {i} rider {r} near the world origin: {q:?}");
                    riders += 1;
                }
            }
        }
        for v in by_oid.values() {
            if v.len() > 1 && v.iter().any(|a| dist(*a, v[0]) > 1.0) {
                shared_oid_distinct += 1;
            }
        }
    }
    println!("{swarms} swarms, {riders} riders on their rails, {shared_oid_distinct} rail oids shared by swarms with distinct rails");
    assert!(swarms > 0 && riders > 0);
    assert!(shared_oid_distinct > 0, "swarms reusing a rail oid must ride different rails");
}

/// the class-81 launcher: the projectile pulled from the pool starts at the launcher (world, owner chain), the
/// FUN_001357c0 tail sets the target (+0x60 = 0, +0x50 = a real object's world origin) and the projectile
/// travels through it
#[test]
#[ignore = "heavy"]
fn k16_c81_launch_from_launcher_hits_real_target() {
    use sly2_recomp::engine::classes::k7::K7;
    let (loader, descs) = levels();
    let mut hits = 0;
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        s.g.dt = DT;
        let Some(pi) = s.k13.iter().find(|x| matches!(x.1, Inst::K13(_))).map(|x| x.0) else { continue };
        let ls: Vec<usize> = s.k7.iter().filter(|x| matches!(x.1, Inst::K7(K7::C2d7f40(_)))).map(|x| x.0).collect();
        // a launcher away from the origin (placed owner chain)
        let Some((li, lp)) = ls.iter().map(|&li| (li, h.ee_sub_1a62e8(li as u32 + 1))).find(|x| dist(x.1, [0.0; 3]) > 500.0) else { continue };
        // a real target: the first placed object 300..3000 from the launcher
        let Some(tp) = w.life.placed.keys().copied().map(|j| h.ee_sub_1a62e8(j as u32 + 1)).filter(|t| (300.0..3000.0).contains(&dist(*t, lp))).next() else { continue };
        assert!(!s.k13_get(pi).unwrap().p.in_play);
        let out = s.k16_fire(li, pi, Some(tp), &mut h).expect("launch");
        let c = s.k13_get(pi).unwrap();
        assert!(dist(c.p.phys.pos, lp) < 1e-3, "starts at the launcher {lp:?}: {:?}", c.p.phys.pos);
        assert_eq!(c.p.f_200, li as u32 + 1);
        let sp = dist(out.v, [0.0; 3]);
        assert!((sp - c.p.f_260).abs() < 1e-2);
        let d0 = dist(lp, tp);
        let step = c.p.f_270.max(c.p.f_260) * DT;
        let slow = c.p.f_260.min(c.p.f_270).max(1.0);
        let mut best = d0;
        for _ in 0..((d0 / (slow * DT)) as usize + 120).min(20000) {
            s.g.t += DT;
            s.frame(&mut h);
            best = best.min(dist(s.k13_get(pi).unwrap().p.phys.pos, tp));
        }
        println!("lsn {} launcher {li} at {lp:?} -> target {tp:?}: d0 {d0} closest {best} (step {step})", d.lsn());
        assert!(best <= step + 1.0, "reaches the target: {best}");
        hits += 1;
    }
    assert!(hits > 0, "no launcher away from the origin");
}

/// a real class-85 beam endpoint moves along its real class-10 path (path10: FUN_001e5df8 / FUN_001e6180 data in
/// the world frame) in the engine host: the drawn endpoint is the path point at the endpoint distance
#[test]
fn k16_c85_beam_moves_along_real_path() {
    let (loader, descs) = levels();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        let Some((i, bi, e)) = s.k15.iter().find_map(|(i, x)| match x {
            Inst::K15e(c) => c.beams.iter().enumerate().find_map(|(bi, b)| {
                b.ends.iter().position(|e| e.kind == 1 && e.path != 0 && (e.prm[1] != 0.0 || e.prm[2] != 0.0)).map(|e| (*i, bi, e))
            }),
            _ => None,
        }) else {
            continue;
        };
        let path = s.k15_85_get(i).unwrap().beams[bi].ends[e].path;
        let p = h.path10(path).expect("class-10 path data");
        let len = p.FUN_001e6648();
        assert!(len > 1.0, "path length {len}");
        let mut pts = Vec::new();
        for _ in 0..240 {
            s.g.dt = DT;
            s.g.t += DT;
            s.frame(&mut h);
            let c = s.k15_85_get(i).unwrap().clone();
            let end = &c.beams[bi].ends[e];
            assert!((-1e-2..=len + 1e-2).contains(&end.d), "d {} len {len}", end.d);
            let segs = c.ee_sub_1df5b0(i, &mut h);
            let q = if e == 0 { segs[bi].0 } else { segs[bi].1 };
            let on = p.FUN_001e6f10(end.d).0;
            if c.beams[bi].clip >= c.beams[bi].len || e == 0 {
                assert!(dist(q, on) < 1e-2, "endpoint {q:?} vs path point {on:?}");
            }
            pts.push(on);
        }
        let moved = pts.iter().map(|q| dist(*q, pts[0])).fold(0.0f32, f32::max);
        println!("lsn {} class 85 obj {i} beam {bi} end {e}: path {path} len {len}, moved {moved}", d.lsn());
        assert!(moved > 1.0, "the endpoint does not move along the path");
        return;
    }
    panic!("no class-85 beam on a class-10 path");
}

/// recording host for the class-55 cluster tests
#[derive(Default)]
struct Rec55 {
    calls: Vec<(u32, usize, i32)>,
    veto: bool,
    query: (i32, i32),
    focus: u32,
    v: [f32; 3],
    pos: [f32; 3],
    accel: Vec<[f32; 3]>,
}
impl ClassHost for Rec55 {
    fn ee_sub_135248(&mut self, obj: usize, state: i32) {
        self.calls.push((0x135248, obj, state));
    }
    fn ee_sub_1f9b20(&mut self, obj: usize) {
        self.calls.push((0x1f9b20, obj, 0));
    }
    fn ee_sub_204108(&mut self, obj: usize) {
        self.calls.push((0x204108, obj, 0));
    }
    fn ee_sub_1fac30(&mut self, obj: usize, a: [f32; 3]) {
        self.calls.push((0x1fac30, obj, 0));
        self.accel.push(a);
    }
    fn po_part(&mut self, addr: u32, obj: usize, off: u32) {
        self.calls.push((addr, obj, off as i32));
    }
    fn vcall_10c(&mut self, _obj: usize) -> [f32; 3] {
        self.v
    }
    fn vcall_1c(&mut self, obj: usize, a: i32, _b: i32) {
        self.calls.push((0x1c, obj, a));
    }
    fn ee_sub_1619a0(&mut self, k: i32, obj: u32) {
        self.calls.push((0x1619a0, obj as usize, k));
    }
    fn ee_sub_1a62e8(&mut self, _obj: u32) -> [f32; 3] {
        self.pos
    }
    fn ee_sub_204250(&mut self, obj: usize, rec: &mut sly2_recomp::engine::player::p5_hull::ContactRec, side: usize) {
        self.calls.push((0x204250, obj, side as i32));
        rec.f94 = 7.0;
    }
    fn ee_sub_140ed8(&mut self, obj: usize, kind: i32) -> bool {
        self.calls.push((0x140ed8, obj, kind));
        self.veto
    }
    fn ee_sub_221268(&mut self, vol: u32, s: i32) {
        self.calls.push((0x221268, vol as usize, s));
    }
    fn ee_sub_14e328(&mut self, k: i32, desc: u32, target: u32, obj: u32) {
        self.calls.push((0x14e328, target as usize, k));
        assert_eq!((desc, obj), (0x2de2f0, 0));
    }
    fn ee_sub_14e4c8_k(&mut self, k: i32, a: i32, desc: u32, b: i32, obj: u32) {
        self.calls.push((0x14e4c8, b as usize, k));
        assert_eq!((a, desc, obj), (1, 0x2de2f0, 0));
    }
    fn vcall(&mut self, obj: u32, off: u32) {
        self.calls.push((off, obj as usize, 0));
    }
    fn ee_sub_130b18(&mut self, handle: u32) {
        self.calls.push((0x130b18, handle as usize, 0));
    }
    fn vcall_1c_query(&mut self, obj: usize, k: i32, msg: (i32, i32)) -> (i32, i32) {
        self.calls.push((0x1c41b8, obj, k));
        assert_eq!(msg, (0x17, 1));
        self.query
    }
    fn g_2de1a0(&mut self) -> u32 {
        self.focus
    }
    fn path_point_1e6f10(&mut self, path: u32, d: f32) -> [f32; 3] {
        [path as f32, d, -1.0]
    }
}

/// class 55 (Bentley) cluster 55a: the 1-function slots on every real Bentley instance
#[test]
fn k16_c55a_bentley_leaf_slots() {
    use sly2_recomp::engine::classes::k16_55::{self, C2cb230};
    let (loader, descs) = levels();
    let mut n = 0;
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        let objs: Vec<usize> = w.objects.iter().enumerate().filter(|x| x.1.class == 55).map(|x| x.0).collect();
        assert_eq!(objs.len(), s.k16.len());
        for i in objs {
            n += 1;
            assert_eq!(k16_55::slot5(55), Some(0x2baf34));
            let c: C2cb230 = s.k16_get(i).unwrap().clone();
            assert_eq!(c.ee_sub_136cb0(), "bentley");
            assert_eq!(c.ee_sub_135e38(), 0);
            // slot 76 on the load-time damage block: no damage state, no flash
            assert_eq!(c.ee_sub_140f70(), 0);
            // 87 / 88 are inverse over the 9 indexed states; 93 is true for 0, 1, 5, 7
            let mut r = Rec55::default();
            for k in 0..10u32 {
                c.ee_sub_135c20(i, k, &mut r);
            }
            assert_eq!(r.calls.len(), 9);
            for (k, &(a, o, st)) in r.calls.iter().enumerate() {
                assert_eq!((a, o), (0x135248, i));
                let mut b = c.clone();
                b.state = st;
                assert_eq!(b.ee_sub_135d08(), k as i32);
            }
            for st in -1..16 {
                let mut b = c.clone();
                b.state = st;
                assert_eq!(b.ee_sub_135ff0(), [0, 1, 5, 7].contains(&st) as i32, "state {st}");
                if !k16_55::STATE_OF_INDEX.contains(&st) {
                    assert_eq!(b.ee_sub_135d08(), -1);
                }
            }
            // slot 69 per state
            for (st, f16b0, want) in [
                (2, 0, vec![(0x204108, 0), (0x196df8, 0x2c0)]),
                (3, 0, vec![(0x204108, 0), (0x194170, 0x4c0)]),
                (4, 0, vec![(0x204108, 0), (0x1a1148, 0xdf0)]),
                (6, 0, vec![(0x204108, 0)]),
                (6, 2, vec![(0x204108, 0), (0x1fac30, 0)]),
                (0xd, 0, vec![(0x1f9b20, 0), (0x19df70, 0xf50)]),
                (0, 0, vec![(0x204108, 0)]),
            ] {
                let mut b = c.clone();
                b.state = st;
                b.f_16b0 = f16b0;
                let mut r = Rec55 { v: [10.0, -4.0, 2.0], ..Default::default() };
                b.ee_sub_135940(i, &mut r);
                assert_eq!(r.calls.iter().map(|x| (x.0, x.2)).collect::<Vec<_>>(), want, "state {st}");
                if f16b0 == 2 {
                    assert_eq!(r.accel, vec![[-25.0, 10.0, -5.0]]);
                }
            }
            // slot 95 lead: horizontal velocity only
            let mut r = Rec55 { v: [100.0, 50.0, 300.0], pos: [1.0, 2.0, 3.0], ..Default::default() };
            let l = c.ee_sub_143220(i, 0.5, &mut r);
            assert_eq!((l.kind, l.r_4, l.r_8), (0, 150.0, 75.0));
            assert_eq!(l.vel, [100.0, 50.0, 0.0]);
            assert_eq!(l.pos, [51.0, 27.0, 3.0]);
            // 92 / 103
            let mut r = Rec55::default();
            c.ee_sub_140eb0(i, &mut r);
            c.ee_sub_141e30(i, &mut r);
            assert_eq!(r.calls, vec![(0x1619a0, i + 1, 0x10), (0x1c, i, 0x1a)]);
        }
    }
    println!("{n} Bentley instances");
    assert!(n > 0);
}

/// class 55 (Bentley) cluster 55b: the 2-function slots on every real Bentley instance
#[test]
fn k16_c55b_bentley_two_fn_slots() {
    use sly2_recomp::engine::classes::k16_55::{self, C2cb230};
    use sly2_recomp::engine::player::p5_hull::{ContactRec, CtSide, GeomRef};
    let (loader, descs) = levels();
    let side = CtSide { geom: GeomRef::Jt, point: [0.0; 3], local: [0.0; 3], vel: [0.0; 3] };
    let rec0 = ContactRec { side: [side, side], n: [0.0, 0.0, 1.0], gap: 0.0, f94: 0.5, f98: 0.5, kind: 0, feat_a: 0, feat_b: 0, relvel: [0.0; 3] };
    let mut n = 0;
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        let objs: Vec<usize> = s.k16.iter().map(|x| x.0).collect();
        for i in objs {
            n += 1;
            let c: C2cb230 = s.k16_get(i).unwrap().clone();
            // slot 54
            for (st, bit, want, calls) in [(4, false, (1.0, 0.3), 0), (0xd, false, (0.0, 0.0), 0), (0, false, (7.0, 0.5), 1), (4, true, (0.5, 0.5), 0)] {
                let mut b = c.clone();
                b.state = st;
                b.dmg.state = 3;
                let mut r = Rec55::default();
                let mut rec = rec0;
                b.ee_sub_135a48(i, &mut rec, 1, bit, &mut r);
                assert_eq!((rec.f94, rec.f98), want, "state {st}");
                assert_eq!(r.calls.len(), calls);
            }
            // slot 75 (+ FUN_00194640)
            for (st, k8, kc, kind, veto, want, base) in [
                (3, 8, 9, 0, false, true, false),
                (3, 8, 11, 0, false, false, true),
                (3, 0xc, 0, 0, false, true, false),
                (0xd, 0, 0, 0x4000, false, true, false),
                (0xd, 0, 0, 0x2000, true, true, true),
                (0, 0, 0, 0, false, false, true),
            ] {
                let mut b = c.clone();
                b.state = st;
                b.att_4c8 = k8;
                b.att_4cc = kc;
                let mut r = Rec55 { veto, ..Default::default() };
                assert_eq!(b.ee_sub_135f78(i, kind, &mut r), want);
                assert_eq!(r.calls == vec![(0x140ed8, i, kind)], base);
            }
            // slot 84 (+ FUN_00143c78)
            let mut b = c.clone();
            b.f_2dc = -1;
            for (st, att, f204, dst, f2dc, want) in [
                (8, 0, 0, 0, -1, 5),
                (0xe, 0, 0, 0, -1, 0xf),
                (0xd, 0, 0, 0, -1, 0),
                (0, 0, 1, 0, -1, 0),
                (0, 3, 0, 0, -1, 4),
                (0, 0, 0, 7, -1, 1),
                (0, 0, 0, -1, -1, 7),
                (0, 0, 0, 2, -1, 0),
                (0, 0, 0, 2, 5, 1),
            ] {
                b.state = st;
                b.att_4c8 = att;
                b.f_204 = f204;
                b.dmg.state = dst;
                b.f_2dc = f2dc;
                assert_eq!(b.ee_sub_136cc0(), want);
            }
            // slot 100: 0x1d drains 2/s, 0x2a 1/s from the counter, floor 0, fraction kept
            let mut b = c.clone();
            b.ctr_12fc = Some(5);
            b.f_1300 = 9;
            for _ in 0..30 {
                b.ee_sub_141dd0(0x1d, 1.0 / 60.0);
            }
            assert_eq!(b.ctr_12fc, Some(4));
            for _ in 0..600 {
                b.ee_sub_141dd0(0x2a, 1.0 / 60.0);
                b.ee_sub_141dd0(0x99, 1.0 / 60.0);
            }
            assert_eq!(b.ctr_12fc, Some(0));
            assert_eq!(b.ee_sub_1441a0(), 0);
            b.ctr_12fc = None;
            assert_eq!(b.ee_sub_1441a0(), 9);
            // slot 77 / FUN_001c3ff0
            let mut b = c.clone();
            b.f_1310 = 77;
            let mut r = Rec55::default();
            b.ee_sub_144480(i, true, &mut r);
            assert_eq!(r.calls, vec![(0x14e328, i + 1, 1), (0x34c, i + 1, 0)]);
            assert_eq!(b.f_1310, 77);
            let mut r = Rec55::default();
            b.ee_sub_144480(i, false, &mut r);
            assert_eq!(r.calls, vec![(0x14e4c8, i + 1, 0xf), (0x34c, i + 1, 0), (0x130b18, 77, 0), (0x14, 77, 0)]);
            assert_eq!((b.f_1310, b.f_1314), (0, -1));
            b.f_16c = 5;
            let mut r = Rec55::default();
            b.ee_sub_1c3ff0(i, true, &mut r);
            assert_eq!(r.calls, vec![(0x221268, 5, 1)]);
            // slot 79
            for (q, dst, want) in [((0x17, 1), -1, true), ((0x17, 0), -1, false), ((3, 0), -1, true)] {
                let mut b = c.clone();
                b.dmg.state = dst;
                let mut r = Rec55 { query: q, ..Default::default() };
                assert_eq!(b.ee_sub_144090(i, &mut r), want);
            }
            // slot 96
            let g = s.g.clone();
            let mut b = c.clone();
            b.f_f54 = 0;
            b.f_f5c = 12;
            b.f_f60 = 34.0;
            let mut r = Rec55 { pos: [1.0, 2.0, 100.0], ..Default::default() };
            assert_eq!(b.ee_sub_141588(i, &g, &mut r), [12.0, 34.0, -1.0]);
            b.f_f54 = -1;
            b.f_248 = 10.0;
            b.f_1048 = 20.0;
            b.f_104c = 5.0;
            b.f_238 = 3.0;
            assert_eq!(b.ee_sub_141588(i, &g, &mut r), [1.0, 2.0, 130.0]);
            r.focus = i as u32 + 1;
            assert_eq!(b.ee_sub_141588(i, &g, &mut r), [1.0, 2.0, 5.0]);
            b.f_208 = 1;
            assert_eq!(b.ee_sub_141588(i, &g, &mut r), [1.0, 2.0, 23.0]);
            assert!(k16_55::ee_sub_194640(8, 10) && !k16_55::ee_sub_194640(8, 8));
        }
    }
    println!("{n} Bentley instances");
    assert!(n > 0);
}
