//! WP-Q6 tests: values / branches from the decompiled bodies (FlatMem host), trivial overrides checked
//! against the ELF image.
use sly2_recomp::engine::classes::q::q2mem::{FlatMem, QMem};
use sly2_recomp::engine::classes::q::q6::{CONST_RET, NOPS};
use sly2_recomp::engine::classes::q::q6_runs::*;

/// `jr ra` + delay slot: (fn, value) — CONST_RET bodies are `jr ra; addiu v0, zero, imm` (or lui/ori pairs)
#[test]
fn q6_trivial_overrides_match_elf() {
    let exe = sly2_recomp::engine::disc::exe::exe();
    for &(f, name, _v) in NOPS {
        // body ends within 2 instructions: jr ra
        let w0 = exe.w(f);
        let w1 = exe.w(f + 4);
        assert!(w0 == 0x03e0_0008 || w1 == 0x03e0_0008, "{name} {f:#x}: {w0:08x} {w1:08x}");
    }
    for &(f, name, v) in CONST_RET {
        let ws: Vec<u32> = (0..3).map(|i| exe.w(f + 4 * i)).collect();
        assert!(ws.contains(&0x03e0_0008), "{name} {f:#x}: {ws:08x?}");
        // value: low 16 bits appear in an addiu/ori v0 immediate
        let lo = v & 0xffff;
        assert!(ws.iter().any(|w| (w >> 16) & 0x1f == 2 && w & 0xffff == lo) || v == 0, "{name} {f:#x} {v:#x}: {ws:08x?}");
    }
}

#[test]
fn q6_guard_sentinels() {
    let mut m = FlatMem::default();
    m.w32(0x1030, u32::MAX);
    assert!(ee_sub_17de40(&mut m, 0x1000));
    assert!(!ee_sub_17e570(&mut m, 0x1000));
    m.w32(0x1034, u32::MAX);
    assert!(ee_sub_175748(&mut m, 0x1000) && ee_sub_175810(&mut m, 0x1000));
    ee_sub_1e6330(&mut m, 0x2000);
    assert_eq!((m.r32(0x2178), m.r32(0x2148)), (0, u32::MAX));
}

#[test]
fn q6_131270_163430() {
    let mut m = FlatMem::default();
    m.w32(0x1014, 7);
    assert_eq!(ee_sub_131270(&mut m, 0x1000), 7);
    m.w32(0x1018, 9);
    assert_eq!(ee_sub_131270(&mut m, 0x1000), 9);
    // n = 3 > 1, k = 1 < 2 -> 1; k = 5 -> 5; n = 1 -> 1
    m.w32(0x2064, 0x3000);
    m.w32(0x2068, 0x4000);
    m.w32(0x3054, 3);
    m.w32(0x4054, 1);
    assert_eq!(ee_sub_163430(&mut m, 0x2000), 1);
    m.w32(0x4054, 5);
    assert_eq!(ee_sub_163430(&mut m, 0x2000), 5);
    m.w32(0x3054, 0);
    assert_eq!(ee_sub_163430(&mut m, 0x2000), 0);
}

#[test]
fn q6_clamp_1e6728() {
    let mut m = FlatMem::default();
    m.ret.insert(0x1e6620, (0, [1.0; 4]));
    m.ret.insert(0x1e6648, (0, [3.0; 4]));
    assert_eq!(ee_sub_1e6728(&mut m, 0.5, 0x1000), 1.0);
    assert_eq!(ee_sub_1e6728(&mut m, 2.0, 0x1000), 2.0);
    assert_eq!(ee_sub_1e6728(&mut m, 9.0, 0x1000), 3.0);
    m.w32(0x10e4, 1);
    m.ret.insert(0x1e66c8, (0, [7.0; 4]));
    assert_eq!(ee_sub_1e6728(&mut m, 2.0, 0x1000), 7.0);
}

#[test]
fn q6_stream_readers() {
    let mut m = FlatMem::default();
    m.wf(0x2000 + 0x3c, 5.0);
    ee_sub_117f18(&mut m, 0x1000, 0x2000);
    let sizes: Vec<u32> = m.calls.iter().map(|c| c.1[1]).collect();
    assert_eq!(sizes, vec![0xc; 4]);
    assert_eq!(m.calls.iter().map(|c| c.1[2]).collect::<Vec<_>>(), vec![0x2000, 0x2010, 0x2020, 0x2030]);
    assert_eq!(m.rf(0x203c), 1.0);
    ee_sub_117fb0(&mut m, 0x1000, 0x3000);
    assert_eq!(m.calls.last().unwrap().1, vec![0x1000, 8, 0x3000]);
}

#[test]
fn q6_pool_refcounts() {
    let mut m = FlatMem::default();
    ee_sub_1ba9f8(&mut m, 1, 0x1234); // wrong key: nothing
    ee_sub_1ba9f8(&mut m, 0, 0xffff); // off: nothing
    assert!(m.calls.is_empty());
    ee_sub_1bab30(&mut m);
    assert_eq!(m.calls.len(), 6);
    assert_eq!(m.calls[0].1, vec![0x4520a8, 0x14, 0x2000, 0x10]);
    assert_eq!(m.calls[4].1, vec![0x452148, 0xc, 200, 0x10]);
    ee_sub_1bab30(&mut m);
    assert_eq!(m.calls.len(), 6, "second ref does not re-create");
    assert_eq!(m.r32(0x2dd2e0), 2);
}

#[test]
fn q6_target_filter_175820() {
    let mut m = FlatMem::default();
    let (o, c, x, e) = (0x1000, 0x2000, 0x3000, 0x4000);
    m.w32(c + 4, x);
    m.w32(x + 0xec, e);
    assert!(ee_sub_175820(&mut m, o, c));
    m.ret.insert(0x132b10, (1, [0.0; 4]));
    assert!(!ee_sub_175820(&mut m, o, c), "class 0x33 rejected");
    m.ret.clear();
    m.w32(c + 0x14, 0x1_0000);
    assert!(!ee_sub_175820(&mut m, o, c), "bit 48 rejects");
    m.w32(c + 0x14, 0);
    m.w32(e + 0x70, 4);
    assert!(!ee_sub_175820(&mut m, o, c));
}

#[test]
fn q6_curve_keys_12ccb8() {
    let mut m = FlatMem::default();
    // stream: count 2, frames 60 / 30, floats 1.5
    m.route = Some(|m, a, _x, _v| match a {
        0x117c20 => {
            let n = m.r32(0x9000);
            m.w32(0x9000, n + 1);
            Some(([2, 60, 30][n as usize], [0.0; 4]))
        }
        0x117e38 => Some((0, [1.5; 4])),
        0x1ae2a0 => Some((0x5000, [0.0; 4])),
        _ => None,
    });
    ee_sub_12ccb8(&mut m, 0x1000, 0x8000);
    assert_eq!((m.r32(0x1004), m.r32(0x1008)), (2, 0x5000));
    assert!((m.rf(0x5000) - 1.0).abs() < 1e-6);
    assert!((m.rf(0x5010) - 0.5).abs() < 1e-6);
    assert_eq!(m.rf(0x501c), 1.5);
}

#[test]
fn q6_current_channel_12cee0() {
    let mut m = FlatMem::default();
    let (c, h, cur) = (0x1000, 0x2000, 0x3000);
    m.w32(c + 8, h);
    assert_eq!(ee_sub_12cee0(&mut m, c), 0);
    m.w32(h + 0x90, c);
    assert_eq!(ee_sub_12cee0(&mut m, c), 1);
    m.w32(h + 0x90, cur);
    assert_eq!(ee_sub_12cee0(&mut m, c), 0);
    m.w32(cur, 5);
    assert_eq!(ee_sub_12cee0(&mut m, c), 0);
    m.w32(cur + 0x1c, c);
    assert_eq!(ee_sub_12cee0(&mut m, c), 1);
}
