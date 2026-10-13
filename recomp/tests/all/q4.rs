//! WP-Q4 tests (breadth-first queue packages).
use sly2_recomp::engine::classes::c2d8c48::C2d8c48;
use sly2_recomp::engine::disc::exe::exe;
use sly2_recomp::engine::disc::exe_scan::scan_thunk;
use sly2_recomp::engine::world::field_table::{field_word, FIELD_COUNT};
use sly2_recomp::engine::world::props::{ArrayItem, Prop, PropValue};

/// run_001dffd8: the class-114 record adders FUN_001e02e0..FUN_001e1b10 (decompiled bodies + disasm of the
/// a2-passthrough ones): (adder, record kind +0, Some(arg index) = +4 from that argument / None = constant).
const ADDERS: &[(u32, u32, Option<usize>, u32)] = &[
    (0x1e02e0, 2, None, 1),
    (0x1e03a0, 4, None, 0),
    (0x1e03e0, 5, None, 1),
    (0x1e0420, 6, None, 1),
    (0x1e0450, 7, None, 1),
    (0x1e0480, 8, None, 1),
    (0x1e04b0, 9, None, 1),
    (0x1e04e0, 10, Some(2), 0),
    (0x1e0530, 11, Some(2), 0),
    (0x1e0570, 12, None, 1),
    (0x1e0610, 14, Some(2), 0),
    (0x1e0650, 15, None, 1),
    (0x1e0680, 16, None, 1),
    (0x1e06b0, 17, None, 1),
    (0x1e06f0, 18, None, 1),
    (0x1e0730, 19, None, 1),
    (0x1e0790, 21, None, 1),
    (0x1e07d0, 22, None, 1),
    (0x1e0820, 23, None, 1),
    (0x1e0840, 24, None, 1),
    (0x1e0880, 25, None, 1),
    (0x1e08c0, 26, Some(2), 0),
    (0x1e0910, 27, None, 0),
    (0x1e0940, 28, Some(2), 0),
    (0x1e0990, 29, None, 0),
    (0x1e09d0, 30, None, 0),
    (0x1e0a20, 31, None, 0),
    (0x1e0a60, 32, None, 0),
    (0x1e0aa0, 33, None, 0),
    (0x1e0ae0, 34, None, 0),
    (0x1e0b40, 35, None, 0),
    (0x1e0c78, 37, None, 1),
    (0x1e0c98, 38, None, 1),
    (0x1e0cb8, 39, None, 0),
    (0x1e0d38, 40, None, 0),
    (0x1e0db8, 41, Some(3), 0),
    (0x1e0e18, 42, Some(3), 0),
    (0x1e0e78, 43, Some(3), 0),
    (0x1e0ed8, 44, Some(2), 0),
    (0x1e0f18, 45, Some(2), 0),
    (0x1e0f58, 46, None, 1),
    (0x1e0f98, 47, None, 1),
    (0x1e0fd8, 48, None, 1),
    (0x1e1018, 49, None, 1),
    (0x1e1058, 50, Some(1), 0),
    (0x1e1088, 51, Some(2), 0),
    (0x1e10c8, 52, None, 1),
    (0x1e1128, 53, None, 1),
    (0x1e1178, 54, None, 1),
    (0x1e1208, 55, None, 1),
    (0x1e1388, 58, None, 1),
    (0x1e13b8, 59, Some(3), 0),
    (0x1e1408, 60, Some(0), 0),
    (0x1e1428, 61, Some(1), 0),
    (0x1e1458, 62, Some(1), 0),
    (0x1e1510, 64, Some(1), 0),
    (0x1e1540, 65, None, 1),
    (0x1e15b0, 67, None, 0),
    (0x1e15f0, 68, None, 0),
    (0x1e1610, 69, Some(2), 0),
    (0x1e1650, 70, None, 1),
    (0x1e1670, 71, None, 1),
    (0x1e16c0, 72, None, 1),
    (0x1e1710, 73, None, 1),
    (0x1e1740, 74, None, 0),
    (0x1e17a0, 75, None, 1),
    (0x1e17e0, 76, None, 1),
    (0x1e1810, 77, None, 1),
    (0x1e1840, 78, None, 0),
    (0x1e18e0, 80, None, 0),
    (0x1e1910, 81, None, 0),
    (0x1e1970, 83, Some(2), 0),
    (0x1e19e0, 85, Some(2), 0),
    (0x1e1a20, 86, None, 1),
    (0x1e1a60, 87, None, 1),
    (0x1e1a90, 88, Some(2), 0),
    (0x1e1ad0, 89, Some(2), 0),
    (0x1e1b10, 90, Some(2), 0),
];

/// Every adder is reached through exactly one Splice field thunk, and `k22_seq::add_record` on that
/// field writes the adder's kind, its +4 value and the first data word at +0x10.
#[test]
fn q4_seq_record_adders() {
    let img = exe();
    for &(adder, kind, barg, bconst) in ADDERS {
        let fields: Vec<usize> = (0..FIELD_COUNT)
            .filter(|&f| field_word(f, 0) == 0x12 && field_word(f, 4) != 0)
            .filter(|&f| scan_thunk(img, field_word(f, 4)).calls.contains(&adder))
            .collect();
        assert_eq!(fields.len(), 1, "adder {adder:#x}: fields {fields:?}");
        let f = fields[0];
        let n = field_word(f, 6) as usize;
        let items = (0..n).map(|i| ArrayItem::One(1, (100 + i as u32).to_le_bytes().to_vec())).collect();
        let mut c = C2d8c48 { f_58: -1, ..Default::default() };
        sly2_recomp::engine::classes::k22_seq::add_record(&mut c, &Prop { field: f as i16, value: PropValue::Array(items) });
        assert_eq!(c.f_54.len(), 1, "adder {adder:#x} field {f}: no record");
        let r = c.f_54[0];
        assert_eq!(r[0], kind, "adder {adder:#x} field {f}");
        let b = barg.map_or(bconst, |i| 100 + i as u32);
        assert_eq!(r[1], b, "adder {adder:#x} field {f} +4");
        let first = (0..n).find(|&i| Some(i) != barg);
        if let (Some(i), false) = (first, matches!(f, 628 | 682 | 683 | 689 | 705)) {
            assert_eq!(r[4], 100 + i as u32, "adder {adder:#x} field {f} +0x10");
        }
    }
}

/// run_001285d8: CRef equality FUN_00129ac0 (tag first, then payload / floats bitwise).
#[test]
fn q4_ref_equality() {
    use sly2_recomp::engine::splice::q_run_001285d8::ee_sub_129ac0 as eq;
    use sly2_recomp::engine::splice::value::Ref;
    assert!(eq(&Ref::S32(5), &Ref::S32(5)));
    assert!(!eq(&Ref::S32(5), &Ref::S32(6)));
    assert!(!eq(&Ref::S32(1), &Ref::F32(f32::from_bits(1))));
    assert!(eq(&Ref::F32(1.5), &Ref::F32(1.5)));
    assert!(!eq(&Ref::F32(0.0), &Ref::F32(-0.0)));
    assert!(eq(&Ref::F32(f32::NAN), &Ref::F32(f32::NAN)));
    assert!(eq(&Ref::Bool(true), &Ref::Bool(true)));
    assert!(!eq(&Ref::Bool(true), &Ref::Bool(false)));
}

mod run_1f6838 {
    use sly2_recomp::engine::classes::q::q2mem::{FlatMem, QMem};
    use sly2_recomp::engine::game::q_001f6838 as q;

    /// bump allocator for FUN_001ae1d0 / 2a0 / 378 ({size, align} block in a0)
    fn route(m: &mut FlatMem, a: u32, x: &[u32], _v: &[[f32; 4]]) -> Option<(u32, [f32; 4])> {
        match a {
            0x1ae1d0 | 0x1ae2a0 | 0x1ae378 => {
                let n = m.r32(x[0]);
                let top = m.r32(0x10_0000).max(0x20_0000);
                m.w32(0x10_0000, top + ((n + 15) & !15));
                Some((top, [0.0; 4]))
            }
            _ => None,
        }
    }

    /// FUN_001f6c08: faces collect their vertices from the edges; a face whose winding opposes its plane
    /// normal swaps p1 / p2
    #[test]
    fn q4_cloth_faces() {
        let mut m = FlatMem { route: Some(route), ..Default::default() };
        let o = 0x1000;
        let pts = 0x3000;
        for (i, p) in [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]].iter().enumerate() {
            m.wv(pts + 0x10 * i as u32, [p[0], p[1], p[2], 1.0]);
        }
        m.w32(o + 0x7c, pts);
        m.w32(o + 0x90, 4);
        m.w32(o + 0x98, 2);
        // planes: both normal +z
        m.w32(o + 0x84, 0x4000);
        m.wv(0x4000, [0.0, 0.0, 1.0, 0.0]);
        m.wv(0x4020, [0.0, 0.0, 1.0, 0.0]);
        // edges {v0, v1, f0, f1}: face 0 = {0,1,2}, face 1 = {1,3,2}
        let edges = [[0u16, 1, 0, 0], [1, 2, 0, 1], [2, 0, 0, 0], [1, 3, 1, 1], [3, 2, 1, 1]];
        m.w32(o + 0xa0, edges.len() as u32);
        m.w32(o + 0xa4, 0x5000);
        for (i, e) in edges.iter().enumerate() {
            for (k, v) in e.iter().enumerate() {
                m.w16(0x5000 + 0x10 * i as u32 + 2 * k as u32, *v);
            }
        }
        q::ee_sub_1f6c08(&mut m, o);
        let w = m.r32(o + 0x10);
        assert_ne!(w, 0);
        let t = m.r32(w + 8);
        let tri = |m: &mut FlatMem, f: u32| [m.r32(t + f * 12), m.r32(t + f * 12 + 4), m.r32(t + f * 12 + 8)];
        // face 0: 0,1,2 is counter-clockwise around +z -> kept
        assert_eq!(tri(&mut m, 0), [0, 1, 2]);
        // face 1: collected 1,2,3; (2-1)x(3-1) = -z against normal +z -> swapped
        assert_eq!(tri(&mut m, 1), [1, 3, 2]);
        // second call: buffers exist -> nothing
        let n = m.calls.len();
        q::ee_sub_1f6c08(&mut m, o);
        assert_eq!(m.calls.len(), n);
    }

    /// FUN_001f6ef8: trapezoid step, +0xa8 reset on a big move, +8 = max |p|, one FUN_00119308 per plane
    #[test]
    fn q4_cloth_step() {
        let mut m = FlatMem::default();
        let o = 0x1000;
        m.w32(o + 0x10, 0x2000);
        m.w32(0x2000, 0x3000); // vel
        m.w32(0x2004, 0x3100); // acc
        m.w32(0x2008, 0x3200); // tris
        m.w32(o + 0x7c, 0x3300);
        m.w32(o + 0x78, 1);
        m.wv(0x3000, [1.0, 0.0, 0.0, 0.0]);
        m.wv(0x3100, [0.0, 2.0, 0.0, 0.0]);
        m.wv(0x3300, [3.0, 4.0, 0.0, 0.0]);
        m.w32(o + 0xa8, 5);
        m.w32(o + 0x80, 1);
        m.w32(o + 0x84, 0x4000);
        q::ee_sub_1f6ef8(&mut m, o, 0.5);
        // v' = v + a dt = (1, 1, 0, 1 [vf0.w])
        assert_eq!(m.rv(0x3000), [1.0, 1.0, 0.0, 1.0]);
        // p' = p + (v + v') dt / 2 = (3.5, 4.25, 0, ..)
        let p = m.rv(0x3300);
        assert_eq!(&p[..3], &[3.5, 4.25, 0.0]);
        assert_eq!(m.r32(o + 0xa8), 5, "moved 0.56 < 1: kept");
        assert!((m.rf(o + 8) - (3.5f32 * 3.5 + 4.25 * 4.25).sqrt()).abs() < 1e-5);
        assert_eq!(m.called(), vec![0x119308]);
        q::ee_sub_1f6ef8(&mut m, o, 2.0);
        assert_eq!(m.r32(o + 0xa8), u32::MAX, "big move resets +0xa8");
    }

    /// FUN_001f7a68: fade kind bits and the collision-record swap (kind 2 / kinds 0,1,3)
    #[test]
    fn q4_fade_kind() {
        let mut m = FlatMem::default();
        let o = 0x1000;
        m.w32(o + 0xe0, 2);
        m.w32(o + 0xe4, 0x2000);
        m.w32(0x2000 + 0x18, 0);
        m.w32(0x20b0 + 0x18, 0x10000);
        m.w8(o + 0x148, 7);
        q::ee_sub_1f7a68(&mut m, o, 2, 0, 0, 0);
        assert_eq!((m.r64(o + 0x148) >> 24) & 3, 2);
        assert_eq!(m.r32(0x2018), 0x10000);
        assert_eq!(m.r16(0x2014), 7);
        assert_eq!(m.r32(0x20c8), 0x10000, "record not matching `from` 0 untouched");
        q::ee_sub_1f7a68(&mut m, o, 3, 0, 0, 0);
        assert_eq!(m.r32(0x2018), 0);
        assert_eq!(m.r16(0x2014) as i16, -10);
        assert_eq!(m.r32(0x20c8), 0);
        // same kind again: no change
        m.w32(0x2018, 0x10000);
        q::ee_sub_1f7a68(&mut m, o, 3, 0, 0, 0);
        assert_eq!(m.r32(0x2018), 0x10000);
    }

    /// FUN_001f7bf8 / 7d10 / 7d90: fade rate -1/t, alpha, slot 7 / 8 adds and removes
    #[test]
    fn q4_fade_rate() {
        let mut m = FlatMem::default();
        let o = 0x1000;
        m.vtable(o, 0x8000, &[(7, 0x111), (8, 0x222)]);
        m.w32(o + 0x40, 0x1);
        q::ee_sub_1f7bf8(&mut m, o, 2.0);
        assert_eq!(m.rf(o + 0x130), -0.5);
        assert_eq!(m.r32(o + 0x40), 3);
        assert_eq!(m.called(), vec![0x111]);
        m.calls.clear();
        q::ee_sub_1f7bf8(&mut m, o, 0.0);
        assert_eq!(m.rf(o + 0x130), 0.0);
        assert_eq!(m.rf(o + 300), 0.0);
        assert_eq!(m.called(), vec![0x132b10, 0x222]);
    }

    /// FUN_001f81e0 / 8238: +0xb4 = -(index + 3)
    #[test]
    fn q4_select_index() {
        let mut m = FlatMem::default();
        let o = 0x1000;
        m.w32(o + 0xac, 3);
        m.w32(o + 0xb0, 0x2000);
        for (i, v) in [10, 20, 30].iter().enumerate() {
            m.w32(0x2000 + 4 * i as u32, *v);
        }
        q::ee_sub_1f81e0(&mut m, o, 30);
        assert_eq!(m.r32(o + 0xb4) as i32, -5);
        q::ee_sub_1f81e0(&mut m, o, 99);
        assert_eq!(m.r32(o + 0xb4), 0);
        q::ee_sub_1f81e0(&mut m, o, 10);
        assert_eq!(m.r32(o + 0xb4) as i32, -3);
        q::ee_sub_1f8238(&mut m, o);
        assert_eq!(m.r32(o + 0xb4), 0);
    }

    /// FUN_001f8300 + setters: light record created once, range 75 (150 for class 0x34)
    #[test]
    fn q4_light_record() {
        let mut m = FlatMem { route: Some(route), ..Default::default() };
        let o = 0x1000;
        assert_eq!(q::ee_sub_1f8300(&mut m, o), 0);
        m.w32(o + 0xe8, 0x2000);
        let p = q::ee_sub_1f8300(&mut m, o);
        assert_ne!(p, 0);
        assert_eq!(m.r32(p), o);
        assert_eq!(m.rf(p + 8), 75.0);
        assert_eq!(m.rf(p + 0xc), 1.0);
        q::ee_sub_1f83b0(&mut m, o, 9.0);
        q::ee_sub_1f8410(&mut m, o, 2.0);
        q::ee_sub_1f8470(&mut m, o, 3.0);
        assert_eq!(q::ee_sub_1f8300(&mut m, o), p);
        assert_eq!([m.rf(p + 4), m.rf(p + 8), m.rf(p + 0xc)], [3.0, 9.0, 2.0]);
        let mut m2 = FlatMem { route: Some(route), ..Default::default() };
        m2.ret.insert(0x132b10, (1, [0.0; 4]));
        m2.w32(o + 0xe8, 0x2000);
        let p = q::ee_sub_1f8300(&mut m2, o);
        assert_eq!(m2.rf(p + 8), 150.0);
    }

    /// FUN_001f8910 / 89e0 / 8b58: emitter entry lookup, update, remove, create
    #[test]
    fn q4_emitter_entries() {
        let mut m = FlatMem::default();
        let h = 0x1000;
        m.w32(h + 0x20, 0x77);
        m.w32(h + 0x24, 0x2000);
        m.w32(0x2000 + 0x90, 0x3000);
        m.w32(0x3000, 4);
        m.w32(0x3010, 0x3100);
        m.w32(0x3100, 1);
        // kind 1 not head: key change -> FUN_002301d0 only
        q::ee_sub_1f89e0(&mut m, h, 5, 9, [1.0, 2.0, 3.0, 0.0]);
        assert_eq!(m.r32(0x3100 + 0xc), 9);
        assert_eq!(m.r32(0x3100 + 0x14), 5);
        assert_eq!(m.called(), vec![0x2301d0, 0x12d108]);
        m.calls.clear();
        // kind 4 head, same key: FUN_002303a0 + FUN_00230598, record fields set
        q::ee_sub_1f8b58(&mut m, 0xa, 0xb, h, 6, 0);
        assert_eq!(m.called(), vec![0x2303a0, 0x230598]);
        assert_eq!([m.r32(0x3020), m.r32(0x3014), m.r32(0x301c)], [0xb, 6, 0xa]);
        m.calls.clear();
        // val -1 removes
        assert_eq!(q::ee_sub_1f8910(&mut m, h, 4, -1, 0), 0);
        assert_eq!(m.called(), vec![0x230100, 0x12e210]);
        m.calls.clear();
        // missing kind 1: created through FUN_0012dc18 unless val is -1
        m.w32(0x2000 + 0x90, 0);
        q::ee_sub_1f89e0(&mut m, h, 3, 1, [0.0; 4]);
        assert_eq!(m.calls[0].0, 0x12dc18);
        assert_eq!(m.calls[0].1, vec![0x77, 0x2000, 1, 3, 2]);
        m.calls.clear();
        q::ee_sub_1f89e0(&mut m, h, -1, 1, [0.0; 4]);
        assert!(m.calls.is_empty());
    }

    /// FUN_001f8668: bit 35 propagates to class-0x30 children; FUN_001d6378 / 6310 with DAT_003e12ec set
    #[test]
    fn q4_hide_tree() {
        let mut m = FlatMem::default();
        m.ret.insert(0x132b10, (1, [0.0; 4]));
        let (o, c1, c2) = (0x1000, 0x2000, 0x3000);
        m.w32(o + 0x64, c1);
        m.w32(c1 + 0x24, c2);
        m.w32(q::RENDER_REG, 1);
        q::ee_sub_1f8668(&mut m, o, 1);
        for x in [o, c1, c2] {
            assert_eq!((m.r64(x + 0x148) >> 35) & 1, 1);
        }
        assert_eq!(m.called().iter().filter(|&&a| a == 0x1d6378).count(), 3);
        m.calls.clear();
        q::ee_sub_1f8668(&mut m, o, 1);
        assert!(m.calls.is_empty(), "no change, no recursion");
    }

    /// FUN_001f8c40: descriptor + six pools registered once
    #[test]
    fn q4_static_init() {
        let mut m = FlatMem::default();
        m.wv(0x2a2dc0, [1.0, 2.0, 3.0, 4.0]);
        q::ee_sub_1f8db8(&mut m);
        q::ee_sub_1f8db8(&mut m);
        assert_eq!(m.rv(0x3dd020), [1.0, 2.0, 3.0, 4.0]);
        assert_eq!(m.r32(0x3dd030), 1);
        assert_eq!(m.called().iter().filter(|&&a| a == 0x11cca8).count(), 6);
        assert_eq!(m.calls[2].1, vec![0x4520f8, 0x28, 0x200, 0x10]);
        assert_eq!(m.r32(0x2dd2f4), 2);
        q::ee_sub_1f8c40(&mut m, 1, 0);
        assert_eq!(m.r32(0x2dd2f4), 2);
    }

    /// FUN_001f8510: C2ca2c0 handle for the matching entry
    #[test]
    fn q4_c2ca2c0_ctor() {
        let mut m = FlatMem::default();
        m.ret.insert(0x11ccc0, (0x9000, [0.0; 4]));
        let o = 0x1000;
        m.w32(o + 0x50, 2);
        m.w32(o + 0x54, 0x2000);
        m.w32(0x2094, 5);
        m.w32(0x209c, 1);
        m.w32(0x2194, 5);
        m.w32(0x219c, 2);
        assert_eq!(q::ee_sub_1f8510(&mut m, o, 3, 5), 0);
        assert_eq!(q::ee_sub_1f8510(&mut m, o, 2, 5), 0x9000);
        assert_eq!([m.r32(0x900c), m.r32(0x9020), m.r32(0x9024)], [0x2ca2c0, o, 0x2100]);
        m.w32(0x9024, 0);
        assert_eq!(q::ee_sub_1f8510(&mut m, o, -1, 5), 0x9000);
        assert_eq!(m.r32(0x9024), 0x2000);
    }
}

mod run_1e9090 {
    use sly2_recomp::engine::classes::q::q2mem::{FlatMem, QMem};
    use sly2_recomp::engine::game::q_001e9090 as q;

    const ID: [[f32; 4]; 4] = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];

    fn route(m: &mut FlatMem, a: u32, x: &[u32], v: &[[f32; 4]]) -> Option<(u32, [f32; 4])> {
        match a {
            // FUN_0011c318(dst, src): inverse of an identity = copy
            0x11c318 => {
                let s = m.rm(x[1]);
                m.wm(x[0], s);
                Some((0, [0.0; 4]))
            }
            0x1e6620 => Some((0, [0.0; 4])),
            0x1e6648 => Some((0, [1.0, 0.0, 0.0, 0.0])),
            0x1e6f10 => {
                m.wv(x[1], [v[0][0], 0.0, 0.0, 1.0]);
                Some((0, [0.0; 4]))
            }
            0x1c0d70 => Some((if v[0][0] < 0.5 { 7 } else { 0 }, [0.0; 4])),
            0x1c0920 => Some((1, [0.0; 4])),
            0x11dfa0 => {
                m.wv(x[1], [1.0, 2.0, 3.0, 0.0]);
                Some((0, [0.0; 4]))
            }
            0x1ae238 | 0x1ae2a0 | 0x1ae1d0 => Some((0x9000, [0.0; 4])),
            _ => None,
        }
    }

    /// p = 0x1000: start object 0x2000, rope 0x3000, points 0x4000, velocities 0x5000, arcs 0x6000
    fn rope(m: &mut FlatMem, n: u32, arcs: &[f32]) -> u32 {
        let p = 0x1000;
        m.route = Some(route);
        m.w32(p + 0x40, 0x2000);
        m.wm(0x2040, ID);
        m.w32(p + 0x48, 0x3000);
        m.w32(0x30e8, n);
        m.w32(0x30f0, 0x3800);
        m.w32(0x30f4, 0x3a00);
        m.vtable(0x3000, 0x7000, &[(30, 0x7777)]);
        m.ret.insert(0x7777, (0x2040, [0.0; 4]));
        m.w32(p + 0x54, 0x4000);
        m.w32(p + 0x58, 0x5000);
        m.w32(p + 0x5c, 0x6000);
        for (i, a) in arcs.iter().enumerate() {
            m.wf(0x6000 + 4 * i as u32, *a);
            m.wv(0x4000 + 0x10 * i as u32, [*a, 0.0, 0.0, 0.0]);
        }
        p
    }

    /// FUN_001e9090: gravity pulls the free points down, every segment keeps its rest length, the rope
    /// points are copied back through the (identity) inverse
    #[test]
    fn q4_rope_step() {
        let mut m = FlatMem::default();
        let p = rope(&mut m, 3, &[0.0, 1.0, 2.0]);
        m.wf(q::DT, 1.0 / 30.0);
        m.wv(p, [0.0, -10.0, 0.0, 0.0]);
        m.wv(q::DEFAULT_DIR, [1.0, 0.0, 0.0, 0.0]);
        for _ in 0..10 {
            q::ee_sub_1e9090(&mut m, p);
        }
        let pts: Vec<_> = (0..3).map(|i| m.rv(0x4000 + 0x10 * i)).collect();
        assert_eq!(&pts[0][..3], &[0.0, 0.0, 0.0]);
        for i in 1..3 {
            let d: f32 = (0..3).map(|k| (pts[i][k] - pts[i - 1][k]).powi(2)).sum::<f32>().sqrt();
            assert!((d - 1.0).abs() < 1e-4, "segment {i} = {d}");
        }
        assert!(pts[2][1] < -0.1, "{:?}", pts[2]);
        assert_eq!(&m.rv(0x3a20)[..3], &pts[2][..3]);
        assert_eq!(*m.called().last().unwrap(), 0x1e6330);
    }

    /// FUN_001e95b0: growing inserts points one segment apart, shrinking drops them
    #[test]
    fn q4_rope_length() {
        let mut m = FlatMem::default();
        let p = rope(&mut m, 2, &[0.0, 1.0]);
        m.wf(p + 0x88, 1.0);
        m.w32(p + 0x84, 10);
        m.wf(q::SLACK, 1.5);
        m.w32(p + 0x4c, 2);
        q::ee_sub_1e95b0(&mut m, 3.0, p);
        assert_eq!(m.r32(0x30e8), 4);
        let s: Vec<f32> = (0..4).map(|i| m.rf(0x6000 + 4 * i)).collect();
        assert_eq!(s, vec![0.0, 1.0, 2.0, 3.0]);
        assert_eq!(m.r32(p + 0x4c), 2);
        q::ee_sub_1e95b0(&mut m, 1.5, p);
        assert_eq!(m.r32(0x30e8), 3);
        assert_eq!(m.rf(0x6008), 1.5);
        // at the maximum the mode is cleared
        m.w32(p + 0x84, 3);
        q::ee_sub_1e95b0(&mut m, 9.0, p);
        assert_eq!(m.r32(p + 0x4c), 0);
    }

    /// FUN_001e9c88: one range from the last inside sample to the path end, one volume id
    #[test]
    fn q4_rail_ranges() {
        let mut m = FlatMem::default();
        m.route = Some(route);
        m.wf(q::PATH_STEP, 0.25);
        let (rec, path) = (0x1000, 0x2000);
        q::ee_sub_1e9c88(&mut m, rec, path);
        assert_eq!([m.r32(rec + 4), m.r32(rec + 0xc), m.r32(rec + 0x90), m.r32(rec + 0x98)], [1, 1, 1, u32::MAX]);
        let c = m.calls.iter().find(|c| c.0 == 0x1ae5a0).unwrap().clone();
        assert_eq!(c.1[2], 8);
        assert_eq!([m.rf(c.1[1]), m.rf(c.1[1] + 4)], [0.25, 1.0]);
        // stamp: nothing rebuilt while no volume changed
        m.w32(rec + 0x94, 0x9100);
        m.w32(0x9100, 0x9200);
        m.w32(0x9200 + 0x114, 5);
        m.w32(rec + 0x98, 5);
        m.calls.clear();
        q::ee_sub_1ea000(&mut m, rec);
        assert!(m.calls.is_empty());
    }

    /// FUN_001ea9f0's CLUT words
    #[test]
    fn q4_glow_clut() {
        let mut m = FlatMem::default();
        m.route = Some(route);
        for k in 0..3 {
            m.wf(0x3e12f4 + 4 * k, 255.0);
        }
        let t = q::ee_sub_1ea9f0_table(&mut m);
        assert_eq!(t.len(), 256);
        assert_eq!([t[0], t[1], t[2], t[3]], [0, 0x030201, 0, 0x02030201]);
        assert_eq!(t[0x80], 0x030201);
        assert_eq!(t[0xff], 0x80030201); // 126 / 126 rounds to 1.0 -> 128
        // no CLUT address and no texture: nothing uploaded
        m.calls.clear();
        q::ee_sub_1ea9f0(&mut m, 0x1000, 0);
        assert!(m.calls.is_empty());
    }

    /// FUN_001ea568 / FUN_001ead98 / FUN_001eb050
    #[test]
    fn q4_load_fixups() {
        let mut m = FlatMem::default();
        m.w32(0x3dcff8, 2);
        m.w32(0x3dcffc, 0x1000);
        m.w8(0x1004, 1);
        m.w16(0x1008, 3);
        m.w32(0x100c, 0x100);
        m.w32(0x3dcff0, 1);
        m.w32(0x3dcff4, 0x2000);
        m.w8(0x2004, 1);
        m.w32(0x2014, 0x40);
        q::ee_sub_1ea568(&mut m, 0x8000);
        assert_eq!(m.r32(0x100c), 0x8100);
        assert_eq!(m.r32(0x2014), 0x8040);
        assert_eq!(m.called(), vec![0x1ed278; 3]);
        assert_eq!(m.calls[2].1, vec![0x8108]);
        m.w16(0x1008, 5);
        q::ee_sub_1ead98(&mut m, 0x3000, 0x1000);
        assert_eq!(m.r32(0x3010), 2);
        m.w32(0x1010, 0x21);
        q::ee_sub_1eb050(&mut m, 0x4000, 0x1000);
        assert_eq!(m.r32(0x4018), 3);
    }
}
