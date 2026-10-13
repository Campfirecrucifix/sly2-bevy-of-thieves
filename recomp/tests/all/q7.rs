//! WP-Q7 queue ports: unit tests on FlatMem with values derived from the disassembly.
use sly2_recomp::engine::classes::q::q2mem::{FlatMem, QMem, V4};
use sly2_recomp::engine::classes::q::q7util::CLOCK;
use sly2_recomp::engine::classes::q::{c2cbe68, c2d0320, c2d0728, c2d7698};

fn fv(x: f32) -> (u32, V4) {
    (0, [x, 0.0, 0.0, 0.0])
}

#[test]
fn q7_c2cbe68_small_slots() {
    let mut m = FlatMem::default();
    assert_eq!(c2cbe68::c2cbe68_v5(), 0x2baef8);
    assert_eq!(c2cbe68::c2cbe68_v98(), 0);
    assert_eq!(c2cbe68::c2cbe68_v97(&mut m), u32::MAX);
    m.ret.insert(0x20f0b0, (1, [0.0; 4]));
    assert_eq!(c2cbe68::c2cbe68_v97(&mut m), 0);
    // slot 75 arg: 5 when +0x15c == 0xb, else 3
    m.vtable(0x1000, 0x2000, &[(75, 0x5555)]);
    m.w32(0x1000 + 0x15c, 0xb);
    c2cbe68::ee_sub_2245b8(&mut m, 0x1000);
    assert_eq!(m.calls.last().unwrap().1, vec![0x1000, 5]);
    m.w32(0x1000 + 0x15c, 2);
    c2cbe68::ee_sub_2245b8(&mut m, 0x1000);
    assert_eq!(m.calls.last().unwrap().1, vec![0x1000, 3]);
}

#[test]
fn q7_c2cbe68_query_each() {
    fn route(m: &mut FlatMem, a: u32, x: &[u32], _v: &[V4]) -> Option<(u32, V4)> {
        (a == 0x15e9c8).then(|| {
            m.w32(x[5], 0x111);
            m.w32(x[5] + 4, 0x222);
            (2, [0.0; 4])
        })
    }
    let mut m = FlatMem { route: Some(route), ..Default::default() };
    c2cbe68::ee_sub_19f220(&mut m, 0x9000, 7);
    let c: Vec<_> = m.calls.iter().filter(|c| c.0 == 0x207030).map(|c| c.1.clone()).collect();
    assert_eq!(c, vec![vec![0x111, 7], vec![0x222, 7]]);
    assert_eq!(m.calls[0].1[1..5], [0x105, 0x38, 0, 0x20]);
}

#[test]
fn q7_c2d0728_wrap_and_slot5() {
    let mut m = FlatMem::default();
    assert_eq!(c2d0728::c2d0728_v5(), 0x2bb7e0);
    // open path: plain difference
    assert_eq!(c2d0728::ee_sub_1e7b38(&mut m, 10.0, 90.0, 0x1000), 80.0);
    // closed path of length 100: wrap into (-50, 50)
    m.w32(0x1000 + 0xe4, 1);
    m.ret.insert(0x1e6648, fv(100.0));
    assert_eq!(c2d0728::ee_sub_1e7b38(&mut m, 10.0, 90.0, 0x1000), -20.0);
    assert_eq!(c2d0728::ee_sub_1e7b38(&mut m, 90.0, 10.0, 0x1000), 20.0);
    assert_eq!(c2d0728::ee_sub_1e7b38(&mut m, 10.0, 40.0, 0x1000), 30.0);
}

#[test]
fn q7_c2d0728_segment_pick() {
    // 2 segments; first passes with zero entry fraction 0 -> skipped... entry != 0 picks immediately
    fn route(m: &mut FlatMem, a: u32, x: &[u32], _v: &[V4]) -> Option<(u32, V4)> {
        match a {
            0x1c1700 => {
                m.w32(x[3], 0xa0);
                m.w32(x[3] + 4, 0xb0);
                Some((2, [0.0; 4]))
            }
            0x1bc688 => {
                // exit fractions 0.7 / 0.3, entry 0
                m.wf(x[2] + 0x40, 0.0);
                m.wf(x[2] + 0x44, if x[0] == 0xa0 { 0.7 } else { 0.3 });
                Some((1, [0.0; 4]))
            }
            _ => None,
        }
    }
    let mut m = FlatMem { route: Some(route), ..Default::default() };
    let z = [0.0; 4];
    assert_eq!(c2d0728::ee_sub_1bd310(&mut m, 1, 2, z, z), 0xb0);
    // nothing found
    let mut m = FlatMem::default();
    assert_eq!(c2d0728::ee_sub_1bd310(&mut m, 1, 2, z, z), 0);
}

#[test]
fn q7_c2d0728_search_outward() {
    // FUN_001c0920 accepts only the point with x == 30 (path point = parameter)
    fn route(m: &mut FlatMem, a: u32, x: &[u32], v: &[V4]) -> Option<(u32, V4)> {
        match a {
            0x1e66c8 => Some((0, v[0])),
            0x1e6f10 => {
                m.wv(x[1], v[0]);
                Some((0, [0.0; 4]))
            }
            0x1c0920 => Some(((v[0][0] == 30.0) as u32, [0.0; 4])),
            _ => None,
        }
    }
    let mut m = FlatMem { route: Some(route), ..Default::default() };
    m.wf(0x2c0abc, 10.0);
    assert_eq!(c2d0728::ee_sub_1bcf18(&mut m, 0.0, 50.0, 0x1000, 0x2000, 0x3000, 0), 1);
    assert_eq!(m.rf(0x3000), 30.0);
    assert_eq!(c2d0728::ee_sub_1bcf18(&mut m, 0.0, 25.0, 0x1000, 0x2000, 0x3000, 0), 0);
    assert_eq!(c2d0728::ee_sub_1bcf18(&mut m, 0.0, 0.0, 0x1000, 0x2000, 0x3000, 0), 0);
}

#[test]
fn q7_c2d0728_update_idle_vs_home() {
    // state 0, held: query skipped; outside radius +0x68 of home -> slot 14 (0), inside -> 2
    let mut m = FlatMem::default();
    let o = 0x1000;
    m.vtable(o, 0x2000, &[(14, 0x7777)]);
    m.w32(o + 0x40, 1);
    m.wf(o + 0x68, 50.0);
    m.wv(o + 0x80, [100.0, 0.0, 0.0, 0.0]);
    m.ret.insert(0x1a62e8, (0, [0.0, 0.0, 0.0, 0.0]));
    c2d0728::c2d0728_v9(&mut m, o);
    let s: Vec<_> = m.calls.iter().filter(|c| c.0 == 0x7777).map(|c| c.1[1]).collect();
    assert_eq!(s, vec![0]);
    m.wv(o + 0x80, [30.0, 40.0, 0.0, 0.0]); // distance exactly 50 -> not outside
    c2d0728::c2d0728_v9(&mut m, o);
    assert_eq!(m.calls.last().unwrap().1, vec![o, 2]);
}

#[test]
fn q7_c2d7698_fade_and_slots() {
    let mut m = FlatMem::default();
    let p = 0x1000;
    m.wf(CLOCK, 10.0);
    m.wf(p + 0x14, 8.0);
    m.wf(p + 0x18, 6.0); // remaining 4, fraction 2/3
    c2d7698::ee_sub_1ceaa8(&mut m, 2.0, p);
    assert!((m.rf(p + 0x18) - 3.0).abs() < 1e-6);
    assert!((m.rf(p + 0x14) - 9.0).abs() < 1e-6);
    // already shorter: untouched
    c2d7698::ee_sub_1ceaa8(&mut m, 5.0, p);
    assert!((m.rf(p + 0x18) - 3.0).abs() < 1e-6);
    // slot 7: flag 0x32 -> 0 without bounce
    let ctx = 0x3000;
    m.w32(ctx + 4, 0x4000);
    m.w32(0x4000 + 0xec, 0x5000);
    m.w32(0x5000 + 0x60, 0x6000);
    m.ret.insert(0x132b10, (1, [0.0; 4]));
    assert_eq!(c2d7698::c2d7698_v7(&mut m, p, ctx, [0.0; 4], [0.0; 4]), 0);
}

#[test]
fn q7_c2d7698_bounce_floor() {
    // particle falling straight down onto a static floor (normal +z): velocity flips with restitution
    let mut m = FlatMem::default();
    let p = 0x1000;
    m.w32(p, 8);
    m.wf(p + 0xf0, 1.0);
    m.wf(p + 0xf4, 0.5);
    m.wf(p + 0xf8, 0.0);
    m.wf(p + 0x18, 1.0);
    m.wf(p + 0x20, 1.0); // radius 1 -> clamped to 10
    m.wv(p + 0x90, [0.0, 0.0, -10.0, 0.0]);
    m.wv(p + 0x70, [0.0, 0.0, 5.0, 0.0]);
    m.wv(0x2a2da0, [1.0, 0.0, 0.0, 0.0]);
    let n = [0.0, 0.0, 1.0, 0.0];
    assert_eq!(c2d7698::ee_sub_1ceb08(&mut m, p, 0x3000, [0.0; 4], n), 1);
    let v = m.rv(p + 0x90);
    assert!((v[2] - 5.0).abs() < 1e-4, "{v:?}");
    assert!(v[0].abs() < 1e-4 && v[1].abs() < 1e-4);
    // pushed out: pos + n * (max(0, (cp - pos).n) + 1) = z 6
    assert!((m.rv(p + 0x70)[2] - 6.0).abs() < 1e-5);
}

#[test]
fn q7_c2d0320_slots() {
    let mut m = FlatMem::default();
    assert_eq!(c2d0320::c2d0320_v5(), 0x2bb858);
    let o = 0x1000;
    c2d0320::c2d0320_v6(&mut m, o, &[5]);
    assert_eq!(m.r32(o + 0x3c), 0xf);
    assert_eq!(m.calls[0], (0x174920, vec![o, 5], vec![]));
    // slot 9: frame push only with a goal frame, idle and record count > 1
    m.w32(o + 0x6c, 1);
    m.w32(o + 0x40, 0x2000);
    m.w32(0x2000 + 8, 0x3000);
    m.w32(0x3000 + 0x10, 2);
    m.w32(o + 0x10, 0x4000);
    c2d0320::c2d0320_v9(&mut m, o, &[]);
    assert_eq!(m.calls.last().unwrap().1, vec![0x4000 + 0xfd0, o + 0x70]);
    let n = m.calls.len();
    m.w32(0x3000 + 0x10, 1);
    c2d0320::c2d0320_v9(&mut m, o, &[]);
    assert_eq!(m.calls.len(), n + 1);
}

#[test]
fn q7_c2d0320_goal_offset() {
    let mut m = FlatMem::default();
    let o = 0x1000;
    m.w32(o + 0x10, 0x4000);
    m.w32(0x4000 + 0xf90, 0x5000);
    m.w32(0x4000 + 0x60, 0x6000);
    m.wv(0x5000 + 0x90, [1.0, 2.0, 3.0, 0.0]);
    m.wv(0x6000 + 0x30, [10.0, 0.0, 0.0, 0.0]);
    m.wv(o + 0x50, [100.0, 100.0, 100.0, 0.0]);
    // identity goal frame
    m.wv(o + 0x70, [1.0, 0.0, 0.0, 0.0]);
    m.wv(o + 0x80, [0.0, 1.0, 0.0, 0.0]);
    m.wv(o + 0x90, [0.0, 0.0, 1.0, 0.0]);
    c2d0320::ee_sub_17a468(&mut m, o);
    let g = m.rv(o + 0x50);
    assert_eq!([g[0], g[1], g[2]], [89.0, 98.0, 97.0]);
}
