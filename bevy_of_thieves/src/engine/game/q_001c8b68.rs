//! WP-Q5: run_001c8b68 queue package: the load-progress bar (FUN_001c8cc0 record {+0 on, +4 total*2,
//! +8 total, +0xc last, +0x10 flag, +0x50 repeat count, +0x54.. four colours}, drawn by FUN_001c8dd8 as
//! two caption_box_quad bars in a 0x400-qword packet) and the loading-screen spinner thread
//! (FUN_001c90d0 creates thread FUN_001c9190 + three semaphores; FUN_001c9058 / FUN_001c9098 start /
//! stop it; FUN_001c93e0 draws the five-part logo as GIF triangles, scaled / pulsing by the elapsed
//! time). Game code: ported 1:1 against `QMem`; the kernel calls (CreateThread 0x234820, StartThread
//! 0x2355e0, WaitSema 0x234a60, SignalSema 0x234a40, ReferSemaStatus 0x234a90, CreateSema
//! 0x21efd0) are routed through `QMem::call` like the other render-thread ports (q_00160260).
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};
use crate::engine::classes::q::q5util::f4;

const WAIT: u32 = 0x0023_4a60;
const SIGNAL: u32 = 0x0023_4a40;
const REFER: u32 = 0x0023_4a90;
/// semaphores: start / stop request / stopped
const S_GO: u32 = 0x003d_3a3c;
const S_STOP: u32 = 0x003d_3a40;
const S_DONE: u32 = 0x003d_3a44;
/// DAT_002dd314: the start-time cell of the running loading screen
const T0: u32 = 0x002d_d314;
/// DAT_002ddf0c: real clock
const CLOCK: u32 = 0x002d_df0c;

/// FUN_001c8cc0: progress bar init: colours from `c0`..`c3`, counters cleared
pub fn ee_sub_1c8cc0<M: QMem + ?Sized>(m: &mut M, p: u32, c0: u32, c1: u32, c2: u32, c3: u32) -> u32 {
    for (o, c) in [(0x54, c0), (0x58, c1), (0x5c, c2), (0x60, c3)] {
        let v = m.r32(c);
        m.w32(p + o, v);
    }
    ee_sub_1c8d20(m, p);
    p
}

/// FUN_001c8d20 / FUN_001c8db8: clear the counters
pub fn ee_sub_1c8d20<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.w8(p + 0x10, 0);
    for o in [0, 0x50, 0xc, 8, 4] {
        m.w32(p + o, 0);
    }
}

/// FUN_001c8d40: progress step `k` (remaining) of an enabled bar: the first step fixes the total
/// (+8 = k, +4 = 2k, FUN_00170348 clears the screen); a repeat of the last value bumps +0x50 else
/// resets it; then the bar is drawn (FUN_001c8dd8)
pub fn ee_sub_1c8d40<M: QMem + ?Sized>(m: &mut M, p: u32, k: u32) {
    if m.r32(p) == 0 {
        return;
    }
    if m.r32(p + 8) == 0 {
        m.w32(p + 8, k);
        m.w32(p + 4, k << 1);
        m.call(0x0017_0348, &[], &[]);
    } else if m.r32(p + 0xc) == k {
        let n = m.r32(p + 0x50).wrapping_add(1);
        m.w32(p + 0x50, n);
    } else {
        m.w32(p + 0x50, 0);
    }
    m.w32(p + 0xc, k);
    ee_sub_1c8dd8(m, p);
}

/// FUN_001c8dd8: draw the bar: frame width w = int(+8 * 620 / +4), fill = int(w * (+8 - +0xc) / +8);
/// box (10, y0) - (fill + 10, y1) in colour +0x54, then (fill + 10) - (w + 10) in +0x58 / +0x5c /
/// +0x60 by the repeat count (<1 / 1 / more); y0 = FUN_0011d7f0(412.9) * 1.1, y1 = y0' + 6.6; the
/// packet (0x400 qwords on the stack) is closed and sent (FUN_00157348) then synced (FUN_00170278)
pub fn ee_sub_1c8dd8<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let sp = stack_push(m, 0x4050);
    let pk = sp + 0x4000;
    m.call(0x0016_6920, &[pk], &[]);
    m.call(0x0015_7330, &[pk, 0x400, sp], &[]);
    m.call(0x0015_74b8, &[pk], &[]);
    let tot = m.r32(p + 8) as i32 as f32;
    let c = m.r32(p + 0x54);
    m.w32(sp + 0x4030, c);
    let w = (tot * 620.0 / m.r32(p + 4) as i32 as f32) as i32;
    let d = (m.r32(p + 8).wrapping_sub(m.r32(p + 0xc))) as i32 as f32 / tot;
    let fill = (w as f32 * d) as i32;
    let k = f32::from_bits(0x43ce_745d);
    let y0 = m.call(0x0011_d7f0, &[], &[f4(k)]).1[0] * 1.1;
    let x1 = fill.wrapping_add(10) as f32;
    let y1 = m.call(0x0011_d7f0, &[], &[f4(k)]).1[0] * 1.1 + 6.6000004;
    m.call(0x001c_d5f0, &[sp + 0x4030, pk], &[f4(10.0), f4(y0), f4(x1), f4(y1)]);
    let n = m.r32(p + 0x50) as i32;
    let o = if n < 1 {
        0x58
    } else if n == 1 {
        0x5c
    } else {
        0x60
    };
    let c = m.r32(p + o);
    m.w32(sp + 0x4030, c);
    m.w32(sp + 0x4040, c);
    let y0 = m.call(0x0011_d7f0, &[], &[f4(k)]).1[0] * 1.1;
    let x2 = w.wrapping_add(10) as f32;
    let y1 = m.call(0x0011_d7f0, &[], &[f4(k)]).1[0] * 1.1 + 6.6000004;
    m.call(0x001c_d5f0, &[sp + 0x4040, pk], &[f4(x1), f4(y0), f4(x2), f4(y1)]);
    m.call(0x0016_6968, &[pk], &[]);
    m.call(0x0015_76b0, &[pk], &[]);
    m.call(0x0015_76f0, &[pk], &[]);
    m.call(0x0015_7348, &[pk, 0, 0], &[]);
    m.call(0x0017_0278, &[sp], &[]);
    stack_pop(m, 0x4050);
}

/// FUN_001c9058: start the spinner: *t = clock, DAT_002dd314 = t, SignalSema(go)
/// VER5: ABI per asm: t = a0; FUN_00234a40(s, t) (a1 = t)
pub fn ee_sub_1c9058<M: QMem + ?Sized>(m: &mut M, t: u32) {
    let s = m.r32(S_GO);
    let c = m.r32(CLOCK);
    m.w32(t, c);
    m.w32(T0, t);
    m.call(SIGNAL, &[s, t], &[]);
}

/// FUN_001c9090: empty
pub fn ee_sub_1c9090() {}

/// FUN_001c9098: stop the spinner: SignalSema(stop), WaitSema(done), DAT_002dd314 = 0
pub fn ee_sub_1c9098<M: QMem + ?Sized>(m: &mut M) {
    let s = m.r32(S_STOP);
    m.call(SIGNAL, &[s], &[]);
    let s = m.r32(S_DONE);
    m.call(WAIT, &[s], &[]);
    m.w32(T0, 0);
}

/// FUN_001c90d0: spinner thread {entry FUN_001c9190, stack 0x3d3a50 / 0x1000, gp 0x2e5270, prio 4}
/// (CreateThread -> DAT_004425e0), the three semaphores (FUN_0021efd0(0, 1)), StartThread
pub fn ee_sub_1c90d0<M: QMem + ?Sized>(m: &mut M) {
    let sp = stack_push(m, 0x30);
    for k in 0..12 {
        m.w32(sp + 4 * k, 0);
    }
    m.w32(sp + 8, 0x003d_3a50);
    m.w32(sp + 0x10, 0x002e_5270);
    m.w32(sp + 0xc, 0x1000);
    m.w32(sp + 0x14, 4);
    m.w32(sp + 4, 0x001c_9190);
    let th = m.call(0x0023_4820, &[sp], &[]).0;
    stack_pop(m, 0x30);
    m.w32(0x0044_25e0, th);
    for s in [S_GO, S_STOP, S_DONE] {
        let x = m.call(0x0021_efd0, &[0, 1], &[]).0;
        m.w32(s, x);
    }
    m.call(0x0023_55e0, &[th, 0], &[]);
}

/// FUN_001c9190: spinner thread body. The original loops forever; this is one start / stop session
/// per call: WaitSema(go); frames (FUN_0014d0b8 clock, FUN_00160828 frame, chain 0x2dea28 opened
/// with the 0x2df260 clear, FUN_002270c8) until the stop semaphore has a count, each drawing the logo
/// at scale 1 + DAT_002c10b0 * FUN_00245000(t * DAT_002c10b4) (t = clock - start) with alpha
/// clamp(DAT_002c10c0 + t (DAT_002c10c4 + t DAT_002c10c8)) and submitting (FUN_00160a98); the last
/// frame drops the frame's bit 0; then waits for the GS (DAT_002deb18 status), consumes stop and
/// signals done.
pub fn ee_sub_1c9190<M: QMem + ?Sized>(m: &mut M) {
    let sp = stack_push(m, 0x40);
    let st = sp;
    let mat = sp + 0x20;
    let s = m.r32(S_GO);
    m.call(WAIT, &[s], &[]);
    m.w32(0x002d_edbc, 0);
    loop {
        m.call(0x0014_d0b8, &[0x002d_ded0, 1], &[]);
        m.call(0x0016_0828, &[], &[]);
        let f = m.r32(0x002d_ea4c);
        let c = m.r32(0x002d_ea34);
        m.w32(f + 0x24, c);
        m.w32(0x002d_ea48, 1);
        m.call(0x0015_74b8, &[0x002d_ea28], &[]);
        m.call(0x0022_70c8, &[0x002d_ea28, 0x25, 0x002d_f260, 0], &[]);
        m.call(0x0015_76f0, &[0x002d_ea28], &[]);
        let s = m.r32(S_STOP);
        m.call(REFER, &[s, st], &[]);
        if m.r32(st) != 0 {
            break;
        }
        let p0 = m.r32(T0);
        let t = m.rf(0x002d_df0c) - m.rf(p0);
        let k = m.rf(0x002c_10b4);
        let w = m.call(0x0024_5000, &[], &[f4(t * k)]).1[0];
        let sc = m.rf(0x002c_10b0) * w + 1.0;
        for k in 0..8 {
            m.w32(mat + 4 * k, 0);
        }
        let a = m.r32(0x002c_10a0) as i32 as f32 * sc;
        m.wf(mat, a);
        let d = m.r32(0x002c_10a4) as i32 as f32 * sc;
        m.wf(mat + 0xc, d);
        let e = m.r32(0x002c_10a8) as i32 as f32;
        m.wf(mat + 0x10, e);
        let ff = m.r32(0x002c_10ac) as i32 as f32;
        m.wf(mat + 0x14, ff);
        let al = m.rf(0x002c_10c0) + t * (m.rf(0x002c_10c4) + t * m.rf(0x002c_10c8));
        let (lo, hi) = (m.rf(0x002b_bb60), m.rf(0x002b_bb64));
        let al = if al < lo {
            lo
        } else if hi < al {
            hi
        } else {
            al
        };
        ee_sub_1c93e0(m, al, 0x002d_ea00, mat);
        m.call(0x0016_0a98, &[], &[]);
        let n = m.r32(0x002b_e538).wrapping_add(1);
        m.w32(0x002b_e538, n);
    }
    let f = m.r32(0x002d_ea4c);
    let x = m.r32(f + 0x30) & !1;
    m.w32(f + 0x30, x);
    m.call(0x0016_0a98, &[], &[]);
    let n = m.r32(0x002b_e538).wrapping_add(1);
    m.w32(0x002b_e538, n);
    loop {
        let s = m.r32(0x002d_eb18);
        m.call(REFER, &[s, st], &[]);
        if m.r32(st) != 0 {
            break;
        }
    }
    let s = m.r32(S_STOP);
    m.call(WAIT, &[s], &[]);
    let s = m.r32(S_DONE);
    m.call(SIGNAL, &[s], &[]);
    stack_pop(m, 0x40);
}

/// FUN_001c93e0: logo into GIF packet `g`: alpha-blend setup (ALPHA_1 0x31001, TEST 0x44), then
/// for the five parts {count, triangle list, rgba} at 0x3d3a00 (stride 0xc): colour with alpha *
/// `al`, PRIM 0x43 (0x83 when alpha >= 0x80), and per triangle three vertices from the 0x2c10d0
/// (x, y) table through the 2x3 matrix `mt` to screen ((x' * 0.8 + 1792) * 16, (y' * 0.4545 +
/// 1936) * 16)
pub fn ee_sub_1c93e0<M: QMem + ?Sized>(m: &mut M, al: f32, g: u32, mt: u32) {
    m.call(0x0016_6bd8, &[g, 0, 1, 0xe], &[]);
    m.call(0x0016_6e90, &[g, 0x47, 0x0003_1001], &[]);
    m.call(0x0016_6e90, &[g, 0x42, 0x44], &[]);
    let mx: Vec<f32> = (0..6).map(|i| m.rf(mt + 4 * i)).collect();
    for part in 0..5u32 {
        let e = 0x003d_3a00 + part * 0xc;
        m.call(0x0016_6bd8, &[g, 0, 2, 0xe1], &[]);
        let c = m.r32(e + 8);
        let a = (((c >> 24) as i32 as f32) * al) as i32 as u32 & 0xff;
        let c = (c & 0x00ff_ffff) | a << 24;
        m.call(0x0016_6c70, &[g, c], &[]);
        let prim = if a & 0x80 != 0 { 0x83 } else { 0x43 };
        m.call(0x0016_6e90, &[g, 0, prim], &[]);
        m.call(0x0016_6ef0, &[g, 1, 4], &[]);
        let n = m.r32(e) as i32;
        for i in 0..n.max(0) as u32 {
            let l = m.r32(e + 4).wrapping_add(i * 0xc);
            for k in 0..3 {
                let v = m.r32(l + 4 * k);
                let vt = 0x002c_10d0u32.wrapping_add(v << 3);
                let (x, y) = (m.rf(vt), m.rf(vt + 4));
                let sx = mx[0] * x + mx[2] * y + mx[4];
                let sy = mx[1] * x + mx[3] * y + mx[5];
                let px = ((sx * 0.8 + 1792.0) * 16.0) as i32 as u32;
                let py = ((sy * f32::from_bits(0x3ee8_ba2f) + 1936.0) * 16.0) as i32 as u32;
                m.call(0x0016_70b8, &[g, px, py, 0, 0], &[]);
            }
        }
    }
}

/// FUN_001c9688: TU static init (1, 0xffff): the five logo parts {count, triangles, rgba}
/// (0x35 @ 0x2c1740 2f3a7f7f, 0x16 @ 0x2c19c0 2f3a7f7f, 0xb @ 0x2c1ac8 5f5f5f7f, 0xb @ 0x2c1b50
/// 5f5f5f7f, 0x3e @ 0x2c1bd8 5f5f5f7f), then the six shared pools (FUN_0021c6c8 body)
pub fn ee_sub_1c9688<M: QMem + ?Sized>(m: &mut M, init: u32, prio: u32) {
    if prio != 0xffff || init == 0 {
        return;
    }
    let parts: [(u32, u32, [u8; 4]); 5] = [
        (0x35, 0x002c_1740, [0x2f, 0x3a, 0x7f, 0x7f]),
        (0x16, 0x002c_19c0, [0x2f, 0x3a, 0x7f, 0x7f]),
        (0xb, 0x002c_1ac8, [0x5f, 0x5f, 0x5f, 0x7f]),
        (0xb, 0x002c_1b50, [0x5f, 0x5f, 0x5f, 0x7f]),
        (0x3e, 0x002c_1bd8, [0x5f, 0x5f, 0x5f, 0x7f]),
    ];
    for (i, (n, l, c)) in parts.iter().enumerate() {
        let e = 0x003d_3a00 + 0xc * i as u32;
        m.call(0x0023_33b4, &[e, 0, 0xc], &[]);
        m.w32(e, *n);
        m.w32(e + 4, *l);
        for (k, b) in c.iter().enumerate() {
            m.w8(e + 8 + k as u32, *b);
        }
    }
    crate::engine::game::q_0021c6c8::ee_sub_21c6c8(m, 1, 0xffff);
}

/// FUN_001c9920: FUN_001c9688(1, 0xffff)
pub fn ee_sub_1c9920<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1c9688(m, 1, 0xffff)
}
