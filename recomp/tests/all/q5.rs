//! WP-Q5 tests: queue ports against the FlatMem reference host (values from the decompiled bodies).
use sly2_recomp::engine::classes::q::q2mem::{FlatMem, QMem, V4};

fn qv(x: f32, y: f32, z: f32) -> V4 {
    [x, y, z, 0.0]
}

mod c2cc3c0 {
    use super::*;
    use sly2_recomp::engine::classes::q::c2cc3c0::*;

    #[test]
    fn q5_c2cc3c0_init_and_state() {
        let mut m = FlatMem::default();
        assert_eq!(ee_sub_114630(), 0x2bba10);
        ee_sub_145010(&mut m, 0x1000);
        assert_eq!(m.r32(0x1060), 0xffff_ffff);
        assert_eq!((m.rf(0x1068), m.rf(0x106c), m.rf(0x1070), m.rf(0x1074), m.rf(0x1078)), (0.25, 250.0, 350.0, 1.0, 1.2));
        assert_eq!(m.called(), vec![0x1eef68]);
        // state 1: FUN_0019f540(owner, +0x80); same state again is a no-op
        m.w32(0x1010, 0x5000);
        m.wf(CLOCK, 7.5);
        ee_sub_145b18(&mut m, 0x1000, 1);
        assert_eq!(m.r32(0x1060), 1);
        assert_eq!(m.rf(0x1064), 7.5);
        assert_eq!(m.called(), vec![0x1eef68, 0x1ef0d8, 0x19f540]);
        ee_sub_145b18(&mut m, 0x1000, 1);
        assert_eq!(m.calls.len(), 3);
        // state 2 with owner kind 0x36: behaviour 0xc3 gets the helper
        m.ret.insert(0x132ea0, (0x36, [0.0; 4]));
        m.ret.insert(0x205438, (0x8000, [0.0; 4]));
        m.vtable(0x8000, 0x9000, &[(7, 0x77)]);
        m.w32(0x1090, 0x4444);
        ee_sub_145b18(&mut m, 0x1000, 2);
        assert_eq!(m.calls[5].1, vec![0x5000, 0xc3]);
        assert_eq!(m.r32(0x8038), 0x4444);
        assert_eq!(m.called().last(), Some(&0x77));
        assert!(ee_sub_145da0(&mut m, 0x2000) == false);
        m.w32(0x2030, 0xffff_ffff);
        assert!(ee_sub_146790(&mut m, 0x2000));
    }

    /// FUN_001450e0: excluded-bit candidates are skipped, raw distance from the owner list, the group
    /// list goes through 200 + d
    #[test]
    fn q5_c2cc3c0_pick_helper() {
        fn route(m: &mut FlatMem, a: u32, x: &[u32], _v: &[V4]) -> Option<(u32, V4)> {
            match a {
                0x1a62e8 => Some((0, m.rv(x[0] + 0x400))),
                _ => None,
            }
        }
        let mut m = FlatMem { route: Some(route), ..Default::default() };
        let (s, o, t) = (0x1000, 0x2000, 0x3000);
        m.w32(s + 0x10, o);
        m.w32(s + 0x30, t);
        m.wv(t + 0x400, qv(0.0, 0.0, 0.0));
        let l = 0x6000;
        m.w32(o + 0x1088, l);
        m.w32(l + 0x28, 2);
        let (a, b) = (0x7000, 0x7800);
        m.w32(l + 0x2c, a);
        m.w32(l + 0x30, b);
        m.wv(a + 0x400, qv(30.0, 40.0, 0.0)); // 50
        m.wv(b + 0x400, qv(3.0, 4.0, 0.0)); // 5 but excluded (bit 0x10)
        m.w32(b + 0x2c0, 0x10000);
        // curves 0x2bd300 (prev pick) / 0x2bd310 (group)
        for (at, c) in [(0x2bd300u32, [-200.0f32, 0.8, 0.0]), (0x2bd310, [200.0, 1.0, 0.0]), (0x2bd320, [-400.0, 1.0, 0.0])] {
            for (i, x) in c.iter().enumerate() {
                m.wf(at + 4 * i as u32, *x);
            }
        }
        assert_eq!(ee_sub_1450e0(&mut m, s), a);
        // group candidate at 10 scores 210 > 50
        let g = 0x6800;
        m.ret.insert(0x1f8240, (g, [0.0; 4]));
        m.w32(g + 0x28, 1);
        m.w32(g + 0x2c, 0x7c00);
        m.wv(0x7c00 + 0x400, qv(6.0, 8.0, 0.0));
        assert_eq!(ee_sub_1450e0(&mut m, s), a);
        // ... unless it is the previous pick: -200 + 0.8 * 210 = -32
        m.w32(s + 0x90, 0x7c00);
        assert_eq!(ee_sub_1450e0(&mut m, s), 0x7c00);
    }
}

mod c2cce90 {
    use super::*;
    use sly2_recomp::engine::classes::q::c2cce90::*;

    #[test]
    fn q5_c2cce90_stops() {
        let mut m = FlatMem::default();
        assert_eq!(ee_sub_114570(), 0x2bb920);
        let s = 0x1000;
        ee_sub_14a1b0(&mut m, s);
        assert_eq!((m.r32(s + 0x30), m.r32(s + 0x3c), m.r32(s + 0x40)), (!0, !0, !0));
        // stops: kinds 0 (walk), 1 (wait); +8 objects
        let tr = 0x2000;
        m.w32(s + 0x38, tr);
        m.w32(tr + 0x58, 0x3000);
        m.w32(0x3000, 0);
        m.w32(0x300c, 1);
        m.w32(0x3008, 0xaaaa);
        m.w32(0x3014, 0xbbbb);
        m.w32(s + 0x3c, 0);
        m.w32(s + 0x40, 1);
        m.w32(s + 0x30, 5); // unknown old state: no exit action
        ee_sub_14ad70(&mut m, s);
        assert_eq!(m.r32(s + 0x3c), 1);
        assert_eq!(m.r32(s + 0x30), 2);
        assert_eq!(ee_sub_14ae18(&mut m, s), 0xbbbb);
        m.w32(s + 0x40, 0);
        m.w32(s + 0x30, 5);
        ee_sub_14ad70(&mut m, s);
        // walking back: kind of the stop left (1) -> state 2, index 0
        assert_eq!((m.r32(s + 0x3c), m.r32(s + 0x30)), (0, 2));
    }

    /// FUN_0014aa30 scoring: 0.5 track band + 0.15 depth band + 0.35 centre falloff
    #[test]
    fn q5_c2cce90_best_stop() {
        fn route(m: &mut FlatMem, a: u32, x: &[u32], v: &[V4]) -> Option<(u32, V4)> {
            match a {
                // FUN_0011d910: lo <= x <= hi
                0x11d910 => {
                    let (lo, hi) = (m.rf(x[0]), m.rf(x[0] + 4));
                    Some(((lo <= v[0][0] && v[0][0] <= hi) as u32, [0.0; 4]))
                }
                0x1e6f60 => {
                    m.wf(x[5], 1000.0);
                    Some((0, [0.0; 4]))
                }
                0x14f308 => Some((0, v[0])),
                _ => None,
            }
        }
        let mut m = FlatMem { route: Some(route), ..Default::default() };
        for (i, x) in [900.0f32, 1400.0, 1400.0, 4000.0, 600.0, 1000.0, 1000.0, 4000.0, 0.0, 10.0].iter().enumerate() {
            m.wf(0x2bd480 + 4 * i as u32, *x);
        }
        let (s, tr, t) = (0x1000, 0x2000, 0x2800);
        m.w32(s + 0x38, tr);
        m.w32(tr + 0x54, 3);
        m.w32(tr + 0x58, 0x3000);
        m.w32(tr + 0x5c, t);
        m.w32(t + 0xf0, 0x4000);
        m.w32(t + 0xf4, 0x5000);
        for i in 0..3u32 {
            m.w32(0x3000 + 0xc * i + 8, 1);
        }
        // stop 0: track 1000 - (-150) = 1150 -> 0.5; depth 800 -> 0.5; r 5 -> 0.5  => 0.5
        m.wf(0x4000, -150.0);
        m.wv(0x5000, qv(3.0, 4.0, 800.0));
        // stop 1: track 2000 -> 1 - 600/2600; depth 2500 -> 0.5; r 0 -> 1
        m.wf(0x4004, -1000.0);
        m.wv(0x5010, qv(0.0, 0.0, 2500.0));
        // stop 2: no object
        m.w32(0x3000 + 0x18 + 8, 0);
        m.wf(0x4008, 1000.0);
        assert_eq!(ee_sub_14aa30(&mut m, s), 1);
        m.w32(0x3000 + 0xc + 8, 0);
        assert_eq!(ee_sub_14aa30(&mut m, s), 0);
        m.w32(0x3000 + 8, 0);
        assert_eq!(ee_sub_14aa30(&mut m, s), -1);
    }
}

mod run_00141ef8 {
    use super::*;
    use sly2_recomp::engine::game::q_00141ef8::*;

    #[test]
    fn q5_141ef8_state_requests() {
        let mut m = FlatMem::default();
        let s = 0x1000;
        m.vtable(s, 0x9000, &[(87, 0x87)]);
        m.w32(s + 0x4c8, 5);
        ee_sub_142140(&mut m, s, 0);
        assert_eq!(m.calls[0].0, 0x87);
        assert_eq!(m.calls[0].1, vec![s, 0]);
        ee_sub_142340(&mut m, s, 0); // not in 9: nothing
        assert_eq!(m.calls.len(), 1);
        ee_sub_1421f8(&mut m, s, 0x55);
        assert_eq!(m.called()[1..], [0x1928f8, 0x1cd308]);
        assert_eq!(m.calls[1].1, vec![s + 0x4c0, 5, 0x55, 0]);
        ee_sub_1424e0(&mut m, s, 0, 0x66, 0x77);
        assert_eq!((m.r32(s + 0x9cc), m.r32(s + 0x1318)), (0, 0x77));
        assert_eq!(m.calls[3].1, vec![s + 0x4c0, 0xc, 0x66, 0]);
        assert!(ee_sub_1424d0(&mut m, s) && ee_sub_142530(&mut m, s) && !ee_sub_1424c0(&mut m, s));
        // look target: same target + priority is a no-op; new one with armed timer starts the glance
        m.wf(0x2a4d60, -1.0);
        m.w32(s + 0x1100, 1);
        ee_sub_1427c8(&mut m, s, 2, 0x4444);
        assert_eq!((m.r32(s + 0x10fc), m.r32(s + 0x10f8), m.r32(s + 0x1148), m.r32(s + 0x1100)), (2, 0x4444, 1, 0));
        ee_sub_142830(&mut m, s, 2, 0x4444);
        assert_eq!(m.r32(s + 0x1100), 1);
        m.wf(s + 0x114c, -1.0);
        m.w32(s + 0x1148, 0);
        ee_sub_1427c8(&mut m, s, 3, 0x5555);
        assert_eq!(m.r32(s + 0x1148), 0);
        // throw params
        m.w32(0x2000, 0x1111);
        m.w32(0x2004, 0x2222);
        ee_sub_142310(&mut m, s, 0x2000, [1.0, 2.0, 3.0, 4.0], [5.0; 4], [6.0, 7.0, 8.0, 9.0]);
        assert_eq!((m.r32(s + 0x11cc), m.r32(s + 0x11d0)), (0x1111, 0x2222));
        assert_eq!((m.rf(s + 0x11c4), m.rf(s + 0x11d8), m.rf(s + 0x1288), m.rf(s + 0x11ec)), (6.0, 9.0, 3.0, 5.0));
    }

    /// FUN_00142af8 target choice: list entry with higher priority in range wins over the current one
    #[test]
    fn q5_142af8_pick_target() {
        fn route(m: &mut FlatMem, a: u32, x: &[u32], _v: &[V4]) -> Option<(u32, V4)> {
            match a {
                // FUN_001428d8(s, obj, .., out_dist, out_angle): distance = obj +0x100
                0x1428d8 => {
                    let d = m.r32(x[1] + 0x100);
                    m.w32(x[3], d);
                    if x[4] != 0 {
                        m.w32(x[4], 0);
                    }
                    Some((0, [0.0; 4]))
                }
                0x11d910 => Some((1, [0.0; 4])),
                _ => None,
            }
        }
        let mut m = FlatMem { route: Some(route), ..Default::default() };
        let s = 0x1000;
        m.w32(s + 0x10f4, 0x8000);
        m.w32(s + 0xf04, 0xffff_ffff);
        m.wf(s + 0x114c, 1e9);
        m.w32(s + 0x10e4, 1);
        let (a, b) = (0x3000, 0x3400);
        m.w32(0x3e1474, a);
        m.w32(a + 0xe0, b);
        m.w32(a + 0xf4, 1);
        m.wf(a + 0xe8, 100.0);
        m.wf(a + 0x100, 50.0);
        m.w32(b + 0xf4, 2);
        m.wf(b + 0xe8, 10.0);
        m.wf(b + 0x100, 50.0); // out of range (10 < 50)
        m.wf(0x2a4d68, 1e9);
        ee_sub_142af8(&mut m, s);
        assert_eq!((m.r32(s + 0x10f8), m.r32(s + 0x10fc)), (a, 1));
        // no yaw / pitch range -> nothing to look at: rig disabled
        assert!(m.called().contains(&0x12d020));
        m.wf(b + 0xe8, 100.0);
        ee_sub_142af8(&mut m, s);
        assert_eq!((m.r32(s + 0x10f8), m.r32(s + 0x10fc)), (b, 2));
    }
}
