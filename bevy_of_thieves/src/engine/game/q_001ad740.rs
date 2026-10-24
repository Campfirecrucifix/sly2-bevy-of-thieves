//! WP-Q5: run_001ad740 queue package: the HUD caption builders of the 0x3d9ed0 race / episode-stat
//! tracker (localised strings FUN_0013e368(0x3dd490, id) appended with the engine's strcpy / strcat
//! FUN_0021ed58 / FUN_0021eda0), the C2d36d0 constructor and the TU static initialisers of the
//! 0x2e6550 / 0x2e65a0 globals, the frame-scratch allocator (DAT_002f6384 bottom / DAT_002f6388 top,
//! frame stack 0x2f639c / 0x2f63a0) and the engine memcpy FUN_001ae5a0. 1:1 against `QMem`.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem};

/// race tracker object (DAT_003d9ed0); +0x24 current slot, slots 0x1c bytes from 0x3d9edc
const TRK: u32 = 0x003d_9ed0;
const LOC: u32 = 0x003d_d490;

fn loc<M: QMem + ?Sized>(m: &mut M, id: u32) -> u32 {
    m.call(0x0013_e368, &[LOC, id], &[]).0
}

fn cpy<M: QMem + ?Sized>(m: &mut M, d: u32, s: u32) {
    m.call(0x0021_ed58, &[d, s], &[]);
}

fn cat<M: QMem + ?Sized>(m: &mut M, d: u32, s: u32) {
    m.call(0x0021_eda0, &[d, s], &[]);
}

/// the tracker slot record of the current slot (+0x24 index * 0x1c from 0x3d9edc)
fn slot<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let i = m.r32(TRK + 0x24);
    i.wrapping_mul(0x1c).wrapping_add(0x003d_9edc)
}

/// FUN_001ad740: with DAT_002e6594 == 1 and the tracker running (FUN_001b0108) the current slot's
/// sound (+8) plays FUN_00213b00 cue `p`+8 + 0x5c8 when the slot owner (+0x14) is `p`+8, else +0x5c5;
/// otherwise FUN_001af608 resets the tracker
pub fn ee_sub_1ad740<M: QMem + ?Sized>(m: &mut M, p: u32) {
    if m.r32(0x002e_6594) != 1 {
        m.call(0x001a_f608, &[TRK], &[]);
        return;
    }
    if m.call(0x001b_0108, &[TRK], &[]).0 == 0 {
        return;
    }
    let s = slot(m);
    let a = m.r32(p + 8);
    let snd = m.r32(s + 8);
    let k = if m.r32(s + 0x14) == a { 0x5c8 } else { 0x5c5 };
    m.call(0x0021_3b00, &[snd, a.wrapping_add(k)], &[]);
}

/// FUN_001ad7e0: caption = string DAT_002e6590
pub fn ee_sub_1ad7e0<M: QMem + ?Sized>(m: &mut M, d: u32) {
    let id = m.r32(0x002e_6590);
    let s = loc(m, id);
    cpy(m, d, s);
}

/// FUN_001ad820: two-player score caption by FUN_001acc80(0x2e6550): 1 -> "&2S&.: " + string 0x38;
/// 2 -> "&2S&. : " + string 0x36 + 0x2a6eb0 + string 0x37 + " : &2T&."
pub fn ee_sub_1ad820<M: QMem + ?Sized>(m: &mut M, d: u32) {
    match m.call(0x001a_cc80, &[0x002e_6550], &[]).0 {
        1 => {
            for i in 0..8 {
                let b = m.r8(0x002a_6e98 + i);
                m.w8(d + i, b);
            }
            let s = loc(m, 0x38);
            cat(m, d, s);
        }
        2 => {
            for i in 0..9 {
                let b = m.r8(0x002a_6ea0 + i);
                m.w8(d + i, b);
            }
            let s = loc(m, 0x36);
            cat(m, d, s);
            cat(m, d, 0x002a_6eb0);
            let s = loc(m, 0x37);
            cat(m, d, s);
            cat(m, d, 0x002a_6eb8);
        }
        _ => {}
    }
}

/// FUN_001ad928: caption = string FUN_001b0078(tracker) (the tracker mode)
pub fn ee_sub_1ad928<M: QMem + ?Sized>(m: &mut M, d: u32) {
    let id = m.call(0x001b_0078, &[TRK], &[]).0;
    let s = loc(m, id);
    cpy(m, d, s);
}

/// FUN_001ad980: while FUN_001d9880(0x3da530) is clear: sprintf(d, "%s : %d%%", string of
/// FUN_001ddd40(FUN_00210b28()), FUN_001d82b8(0x3e0ea8, 0x3d4a60))
pub fn ee_sub_1ad980<M: QMem + ?Sized>(m: &mut M, d: u32) {
    if m.call(0x001d_9880, &[0x003d_a530], &[]).0 != 0 {
        return;
    }
    let w = m.call(0x0021_0b28, &[], &[]).0;
    let id = m.call(0x001d_dd40, &[w], &[]).0;
    let s = loc(m, id);
    let pct = m.call(0x001d_82b8, &[0x003e_0ea8, 0x003d_4a60], &[]).0;
    m.call(0x0021_ecd8, &[d, 0x002a_6ec8, s, pct], &[]);
}

/// FUN_001ada20: while the tracker runs: the slot owner's (+0x14) record 0x2e6604 + owner * 0x5464
/// is read through FUN_002581c0(tmp, DAT_003e0fac, ..) (tmp+4 = time) and released (FUN_00129ec8);
/// mode 0x19 -> string 0x29 (+ 0x2a6ed8 + string 0x25 when timed), 0x1e -> 0x25 timed / 0x26,
/// 0x1f -> 0x27 timed / 0x26, others nothing
pub fn ee_sub_1ada20<M: QMem + ?Sized>(m: &mut M, d: u32) {
    if m.call(0x001b_0108, &[TRK], &[]).0 == 0 {
        return;
    }
    let sp = stack_push(m, 0x10);
    let s = slot(m);
    let who = m.r32(s + 0x14);
    let a1 = m.r32(0x003e_0fac);
    m.call(0x0025_81c0, &[sp, a1, who.wrapping_mul(0x5464).wrapping_add(0x002e_6604)], &[]);
    let t = m.rf(sp + 4);
    m.call(0x0012_9ec8, &[sp], &[]);
    stack_pop(m, 0x10);
    let id = match m.call(0x001b_0078, &[TRK], &[]).0 {
        0x1e => {
            if t != 0.0 {
                0x25
            } else {
                0x26
            }
        }
        0x1f => {
            if t != 0.0 {
                0x27
            } else {
                0x26
            }
        }
        0x19 => {
            let s = loc(m, 0x29);
            cpy(m, d, s);
            if t != 0.0 {
                cat(m, d, 0x002a_6ed8);
                let s = loc(m, 0x25);
                cat(m, d, s);
            }
            return;
        }
        _ => return,
    };
    let s = loc(m, id);
    cpy(m, d, s);
}

/// FUN_001adbb8: modes 0x19, 0x1e, 0x1f: string 0x40 + " : &2T&."
pub fn ee_sub_1adbb8<M: QMem + ?Sized>(m: &mut M, d: u32) {
    let k = m.call(0x001b_0078, &[TRK], &[]).0;
    if k == 0x19 || k == 0x1e || k == 0x1f {
        let s = loc(m, 0x40);
        cpy(m, d, s);
        cat(m, d, 0x002a_6eb8);
    }
}

/// C2d36d0_ctor (FUN_001addf0): base C2d85e8 ctor (FUN_001da728), size 0x5d4, vtable 0x2d36d0,
/// FUN_001da7d0
pub fn ee_sub_1addf0<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    m.call(0x001d_a728, &[o], &[]);
    m.w32(o + 0x20, 0x5d4);
    m.w32(o + 0xc, 0x002d_36d0);
    m.call(0x001d_a7d0, &[o], &[]);
    o
}

/// FUN_001adf98: TU static init / teardown of 0x2e6550 (C2d3750, FUN_001aca98) and 0x2e65a0
/// (C2d36d0): (1, 0xffff) constructs both then registers the six shared pools (the FUN_0021c6c8
/// body); (0, 0xffff) destroys them (FUN_00258348(.., 2)).
pub fn ee_sub_1adf98<M: QMem + ?Sized>(m: &mut M, init: u32, prio: u32) {
    if prio != 0xffff {
        return;
    }
    if init == 0 {
        m.call(0x0025_8348, &[0x002e_65a0, 2], &[]);
        m.call(0x0025_8348, &[0x002e_6550, 2], &[]);
        return;
    }
    m.call(0x001a_ca98, &[0x002e_6550], &[]);
    ee_sub_1addf0(m, 0x002e_65a0);
    crate::engine::game::q_0021c6c8::ee_sub_21c6c8(m, 1, 0xffff);
}

/// FUN_001ae110: FUN_001adf98(1, 0xffff)
pub fn ee_sub_1ae110<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1adf98(m, 1, 0xffff)
}

/// FUN_001ae130: FUN_001adf98(0, 0xffff)
pub fn ee_sub_1ae130<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1adf98(m, 0, 0xffff)
}

/// FUN_001ae150: free scratch bytes (top - bottom)
pub fn ee_sub_1ae150<M: QMem + ?Sized>(m: &mut M) -> u32 {
    m.r32(0x002f_6388).wrapping_sub(m.r32(0x002f_6384))
}

/// FUN_001ae1d0: bottom-up frame-scratch block for {size +0, align +4} at `req` (FUN_001ae1c8 is an
/// empty stub): bottom DAT_002f6384 aligned up, advanced by the 16-rounded size; 0 for size 0
pub fn ee_sub_1ae1d0<M: QMem + ?Sized>(m: &mut M, req: u32) -> u32 {
    let n = m.r32(req);
    if n == 0 {
        return 0;
    }
    let al = m.r32(req + 4);
    let v = m.r32(0x002f_6384).wrapping_sub(1).wrapping_add(al) & al.wrapping_neg();
    m.w32(0x002f_6384, v.wrapping_add(n.wrapping_add(15) & !15));
    v
}

/// FUN_001ae238: bottom-up scratch block for {size, align} at `req`, filled from `src`
/// (FUN_001ae5a0)
pub fn ee_sub_1ae238<M: QMem + ?Sized>(m: &mut M, req: u32, src: u32) -> u32 {
    let p = m.call(0x001a_e1d0, &[req], &[]).0;
    if p != 0 {
        let n = m.r32(req);
        ee_sub_1ae5a0(m, p, src, n);
    }
    p
}

/// FUN_001ae2a0: bottom-up scratch block for {size, align} at `req`, zeroed (FUN_002333b4)
pub fn ee_sub_1ae2a0<M: QMem + ?Sized>(m: &mut M, req: u32) -> u32 {
    let p = m.call(0x001a_e1d0, &[req], &[]).0;
    if p != 0 {
        let n = m.r32(req);
        m.call(0x0023_33b4, &[p, 0, n], &[]);
    }
    p
}

/// FUN_001ae300: bottom = base (DAT_002f6380)
pub fn ee_sub_1ae300<M: QMem + ?Sized>(m: &mut M) {
    let b = m.r32(0x002f_6380);
    m.w32(0x002f_6384, b);
}

/// FUN_001ae318: top-down scratch block {size, align}: top = (top - round16(size)) & -align
pub fn ee_sub_1ae318<M: QMem + ?Sized>(m: &mut M, req: u32) -> u32 {
    let n = m.r32(req);
    if n == 0 {
        return 0;
    }
    m.call(0x001a_e1c8, &[], &[]);
    let al = m.r32(req + 4);
    let t = m.r32(0x002f_6388).wrapping_sub((n + 0xf) & !0xf) & al.wrapping_neg();
    m.w32(0x002f_6388, t);
    t
}

/// FUN_001ae378: top-down scratch block, zeroed (FUN_002333b4)
pub fn ee_sub_1ae378<M: QMem + ?Sized>(m: &mut M, req: u32) -> u32 {
    let p = ee_sub_1ae318(m, req);
    if p != 0 {
        let n = m.r32(req);
        m.call(0x0023_33b4, &[p, 0, n], &[]);
    }
    p
}

/// FUN_001ae3d8: lock (FUN_0021f060(0x2f6390)), push the top onto the frame stack
pub fn ee_sub_1ae3d8<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x0021_f060, &[0x002f_6390], &[]);
    let i = m.r32(0x002f_639c).wrapping_add(1);
    m.w32(0x002f_639c, i);
    let t = m.r32(0x002f_6388);
    m.w32(i.wrapping_mul(4).wrapping_add(0x002f_63a0), t);
}

/// FUN_001ae428: pop the top from the frame stack, unlock (FUN_0021f0c0)
pub fn ee_sub_1ae428<M: QMem + ?Sized>(m: &mut M) {
    let i = m.r32(0x002f_639c);
    // asm order: the entry is loaded before the count store (matters when i == -1 aliases the count)
    let t = m.r32(i.wrapping_mul(4).wrapping_add(0x002f_63a0));
    m.w32(0x002f_639c, i.wrapping_sub(1));
    m.w32(0x002f_6388, t);
    m.call(0x0021_f0c0, &[0x002f_6390], &[]);
}

/// FUN_001ae478: scratch frame enter; returns `a`
pub fn ee_sub_1ae478<M: QMem + ?Sized>(m: &mut M, a: u32) -> u32 {
    ee_sub_1ae3d8(m);
    a
}

/// FUN_001ae4a0: scratch frame leave (bit 0 of `fl`: FUN_001ae520, empty)
pub fn ee_sub_1ae4a0<M: QMem + ?Sized>(m: &mut M, _a: u32, _fl: u32) {
    ee_sub_1ae428(m);
}

/// FUN_001ae4e8: 0
pub fn ee_sub_1ae4e8() -> u32 {
    0
}

/// FUN_001ae4f0 / FUN_001ae520: empty
pub fn ee_sub_1ae4f0() {}

/// FUN_001ae4f8: zeroed bottom-up block of `n` bytes, 16-aligned (FUN_001ae2a0(&{n, 0x10}))
pub fn ee_sub_1ae4f8<M: QMem + ?Sized>(m: &mut M, n: u32) -> u32 {
    let sp = stack_push(m, 0x10);
    m.w32(sp, n);
    m.w32(sp + 4, 0x10);
    let p = ee_sub_1ae2a0(m, sp);
    stack_pop(m, 0x10);
    p
}

/// FUN_001ae528: copy `n` quadwords
/// VFIX 1:1 with the asm: n & 3 single quads first, then 4-quad blocks (all 4 loaded before storing)
/// while the signed remainder (n - (n & 3)) > 0; lq/sq ignore the low 4 address bits.
pub fn ee_sub_1ae528<M: QMem + ?Sized>(m: &mut M, d: u32, s: u32, n: u32) {
    let (mut d, mut s) = (d, s);
    let rem = n & 3;
    let blk = n.wrapping_sub(rem) as i32;
    for _ in 0..rem {
        let q = m.rv(s & !0xf);
        m.wv(d & !0xf, q);
        s = s.wrapping_add(0x10);
        d = d.wrapping_add(0x10);
    }
    let mut i = 0i32;
    if blk > 0 {
        loop {
            i += 4;
            let q: [_; 4] = std::array::from_fn(|k| m.rv(s.wrapping_add(0x10 * k as u32) & !0xf));
            for (k, q) in q.into_iter().enumerate() {
                m.wv(d.wrapping_add(0x10 * k as u32) & !0xf, q);
            }
            s = s.wrapping_add(0x40);
            d = d.wrapping_add(0x40);
            if i >= blk {
                break;
            }
        }
    }
}

/// FUN_001ae5a0: memcpy: quadwords when all of d / s / n are 16-aligned, words when 4-aligned,
/// else bytes
pub fn ee_sub_1ae5a0<M: QMem + ?Sized>(m: &mut M, d: u32, s: u32, n: u32) {
    let a = d | s | n;
    if a & 3 == 0 {
        if a & 0xf == 0 {
            ee_sub_1ae528(m, d, s, n >> 4);
        } else {
            // HEIST r42: EE addu wraps (a bad caller pointer near 0xffffffff must not panic the debug build)
            for k in 0..n >> 2 {
                let w = m.r32(s.wrapping_add(4 * k));
                m.w32(d.wrapping_add(4 * k), w);
            }
        }
    } else {
        for k in 0..n {
            let b = m.r8(s.wrapping_add(k));
            m.w8(d.wrapping_add(k), b);
        }
    }
}

/// FUN_001ae928: +0x20 = FUN_0015e938(0x3dd490, 0x405, 0x370, 0), +0x1ec = -1, +0x1e8 = 0
pub fn ee_sub_1ae928<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let x = m.call(0x0015_e938, &[LOC, 0x405, 0x370, 0], &[]).0;
    m.w32(o + 0x20, x);
    m.w32(o + 0x1ec, 0xffff_ffff);
    m.w32(o + 0x1e8, 0);
}

/// FUN_001ae970: FUN_001af660(o), FUN_001aeaa8(o, 0), +0x20 = 0
pub fn ee_sub_1ae970<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_f660, &[o], &[]);
    m.call(0x001a_eaa8, &[o, 0], &[]);
    m.w32(o + 0x20, 0);
}
