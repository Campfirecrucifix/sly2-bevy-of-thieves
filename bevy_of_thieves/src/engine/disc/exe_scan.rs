//! WP-PUB2: tiny R5900 code scanners used to recover table data the original keeps in code rather than in
//! .data (the Splice method thunks' argument kinds / callee, the class ctors' vtable). Straight-line
//! symbolic execution: constants, `$sp`-relative addresses and stack words are tracked; everything else
//! is unknown. Verified entry-for-entry against the previous generated tables.

use super::exe::Image;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Val {
    Unknown,
    Const(u128),
    Sp(i64),
}

fn sx(v: u32) -> i64 {
    (v & 0xffff) as u16 as i16 as i64
}

/// Result of [`scan_thunk`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ThunkScan {
    /// the kind array handed to FUN_00201360 (first call), None when not constant
    pub kinds: Option<Vec<u32>>,
    /// jal / j targets in code order
    pub calls: Vec<u32>,
    /// FUN_0012a0b0(out, 0x24) (sets the result to Void)
    pub void_ret: bool,
}

/// FUN_00201360: unpack the CRef arguments by kind.
pub const UNPACK_ARGS: u32 = 0x0020_1360;
/// FUN_0012a0b0: set CRef type.
pub const SET_REF_TYPE: u32 = 0x0012_a0b0;

/// Scans a Splice method thunk `fn(out, obj, ?, argv)` from `start` to its `jr` (max 200 instructions).
pub fn scan_thunk(img: &Image, start: u32) -> ThunkScan {
    let mut r = [Val::Unknown; 32];
    r[0] = Val::Const(0);
    r[29] = Val::Sp(0);
    let mut stack: HashMap<i64, u32> = HashMap::new();
    let mut out = ThunkScan::default();
    let mut seen_unpack = false;

    fn set(r: &mut [Val; 32], d: usize, v: Val) {
        if d != 0 {
            r[d] = v;
        }
    }
    let exe = |r: &mut [Val; 32], stack: &mut HashMap<i64, u32>, i: u32| {
        let (op, rs, rt, rd, imm, fu) = (i >> 26, (i >> 21 & 31) as usize, (i >> 16 & 31) as usize, (i >> 11 & 31) as usize, i & 0xffff, i & 63);
        let store = |r: &[Val; 32], stack: &mut HashMap<i64, u32>, nw: u32| {
            let Val::Sp(b) = r[rs] else { return };
            let o = b + sx(imm);
            for k in 0..nw as i64 {
                match r[rt] {
                    Val::Const(v) => {
                        stack.insert(o + 4 * k, (v >> (32 * k)) as u32);
                    }
                    _ => {
                        stack.remove(&(o + 4 * k));
                    }
                }
            }
        };
        let load = |r: &mut [Val; 32], stack: &HashMap<i64, u32>, nw: u32| {
            // (writes rt even when it is $zero, like the reference generator)
            r[rt] = match r[rs] {
                Val::Const(b) => {
                    let a = (b as i64 + sx(imm)) as u32;
                    Val::Const((0..nw).fold(0u128, |v, k| v | (img.w(a.wrapping_add(4 * k)) as u128) << (32 * k)))
                }
                Val::Sp(b) => {
                    let o = b + sx(imm);
                    let mut v = 0u128;
                    let mut ok = true;
                    for k in 0..nw as i64 {
                        match stack.get(&(o + 4 * k)) {
                            Some(&x) => v |= (x as u128) << (32 * k),
                            None => ok = false,
                        }
                    }
                    if ok { Val::Const(v) } else { Val::Unknown }
                }
                Val::Unknown => Val::Unknown,
            };
        };
        match op {
            0x09 | 0x19 => {
                let v = match r[rs] {
                    Val::Sp(o) => Val::Sp(o + sx(imm)),
                    Val::Const(v) => {
                        let s = (v as i128 + sx(imm) as i128) as u128;
                        Val::Const(if op == 0x19 { s & 0xffff_ffff_ffff_ffff } else { s & 0xffff_ffff })
                    }
                    Val::Unknown => Val::Unknown,
                };
                set(r, rt, v)
            }
            0x0f => set(r, rt, Val::Const((imm << 16) as u128)),
            0x0d => {
                let v = match r[rs] {
                    Val::Const(v) => Val::Const(v | imm as u128),
                    _ => Val::Unknown,
                };
                set(r, rt, v)
            }
            0 if matches!(fu, 0x25 | 0x2d | 0x21) => {
                let v = match (r[rs], r[rt]) {
                    (Val::Sp(o), _) if rt == 0 => Val::Sp(o),
                    (_, Val::Sp(o)) if rs == 0 => Val::Sp(o),
                    (Val::Const(a), Val::Const(b)) => Val::Const(if fu == 0x25 { a | b } else { a.wrapping_add(b) }),
                    _ => Val::Unknown,
                };
                set(r, rd, v)
            }
            0x2b | 0x2e => store(r, stack, 1),
            0x3f | 0x2d => store(r, stack, 2),
            0x1f => store(r, stack, 4),
            0x23 | 0x26 => load(r, stack, 1),
            0x37 | 0x1b => load(r, stack, 2),
            0x1e => load(r, stack, 4),
            // ldl/sdl/lwl/swl (the r half carries the value), sh/sb/swc1/lwc1, cop0/1, sdc1/lqc2/sqc2
            0x1a | 0x2c | 0x22 | 0x2a | 0x29 | 0x28 | 0x39 | 0x31 | 0x11 | 0x10 | 0x35 => {}
            0 | 0x1c | 0x12 => set(r, rd, Val::Unknown),
            0x20 | 0x21 | 0x24 | 0x25 | 0x27 | 0x0a | 0x0b | 0x0c | 0x0e | 0x18 => set(r, rt, Val::Unknown),
            _ => {}
        }
    };

    let mut pc = start;
    for _ in 0..200 {
        let i = img.w(pc);
        let op = i >> 26;
        if op == 3 || op == 2 || (op == 0 && i & 63 == 8) {
            exe(&mut r, &mut stack, img.w(pc + 4));
            if op == 0 {
                break;
            }
            let t = ((pc + 4) & 0xf000_0000) | ((i & 0x03ff_ffff) << 2);
            out.calls.push(t);
            if t == UNPACK_ARGS && !seen_unpack {
                seen_unpack = true;
                let na = match r[5] {
                    Val::Const(v) => Some(v),
                    _ => None,
                };
                if let (Some(na), Val::Sp(base)) = (na, r[7]) {
                    let ks: Option<Vec<u32>> = (0..na as i64).map(|k| stack.get(&(base + 4 * k)).copied()).collect();
                    out.kinds = ks;
                }
                if na == Some(0) {
                    out.kinds = Some(Vec::new());
                }
            }
            if t == SET_REF_TYPE && r[5] == Val::Const(0x24) {
                out.void_ret = true;
            }
            if op == 2 {
                break;
            }
            for k in [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 24, 25, 31] {
                r[k] = Val::Unknown;
            }
            pc += 8;
            continue;
        }
        exe(&mut r, &mut stack, i);
        pc += 4;
    }
    out
}

/// The vtable a class ctor installs: the LAST constant stored to `+0xc` of any register before its `jr`
/// (many ctors store the base vtable 0x2d9978 first and then the real subclass vtable). 0 = none.
pub fn ctor_vtable(img: &Image, ctor: u32) -> u32 {
    ctor_vtable_d(img, ctor, 1)
}

/// ANIM-ALL R16: a ctor that stores no vtable itself (FUN_001116d8 (JT): `FUN_002563e8(FUN_001ae4f8(0x2060))`)
/// installs the one of its last `jal` callee that does (0x2d2e98); `depth` bounds that descent.
fn ctor_vtable_d(img: &Image, ctor: u32, depth: u32) -> u32 {
    let mut jals: Vec<u32> = Vec::new();
    let mut r: [Option<u32>; 32] = [None; 32];
    r[0] = Some(0);
    let mut found = 0;
    let mut pc = ctor;
    for _ in 0..400 {
        let i = img.w(pc);
        let (op, rs, rt, rd, imm, fu) = (i >> 26, (i >> 21 & 31) as usize, (i >> 16 & 31) as usize, (i >> 11 & 31) as usize, i & 0xffff, i & 63);
        if op == 0 && fu == 8 {
            let n = img.w(pc + 4);
            if n >> 26 == 0x2b && n & 0xffff == 0xc {
                if let Some(v) = r[(n >> 16 & 31) as usize] {
                    found = v;
                }
            }
            break;
        }
        match op {
            0x0f => r[rt] = Some(imm << 16),
            0x09 | 0x19 => r[rt] = r[rs].map(|v| (v as i64 + sx(imm)) as u32),
            0x0d => r[rt] = r[rs].map(|v| v | imm),
            0x2b if imm == 0xc && rt != 0 && r[rt].is_some() => found = r[rt].unwrap(),
            0x03 => jals.push((pc & 0xf000_0000) | (i & 0x03ff_ffff) << 2),
            0 if matches!(fu, 0x25 | 0x2d | 0x21) => {
                r[rd] = match (r[rs], r[rt]) {
                    (Some(a), Some(b)) => Some(if fu == 0x25 { a | b } else { a.wrapping_add(b) }),
                    _ => None,
                }
            }
            0x23 | 0x37 | 0x1e | 0x20 | 0x24 | 0x21 | 0x25 | 0x0a | 0x0b | 0x0c | 0x0e | 0x1a | 0x1b | 0x22 | 0x26 if rt != 0 => r[rt] = None,
            0 | 0x1c if rd != 0 => r[rd] = None,
            _ => {}
        }
        pc += 4;
    }
    if found == 0 && depth > 0 {
        found = jals.iter().rev().map(|&t| ctor_vtable_d(img, t, depth - 1)).find(|&v| v != 0).unwrap_or(0);
    }
    found
}
