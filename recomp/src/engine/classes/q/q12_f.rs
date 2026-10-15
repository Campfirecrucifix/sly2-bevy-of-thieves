//! WP-Q12 part F: file package run_0020a540 (actor sub-objects: pose-follow helpers, camera-ray / trigger
//! state machines, text buffer at 0x20d5d0..). 1:1 from ghidra_functions(_v2).body on `QMem` (conventions as
//! q12_a / q12_e). Vector (VU0) code reproduces the xyz math; in 20a818 / 20cb28 the lane / argument choices that
//! the decompiler lost (qmfc2 = x lane only, third GPR argument of FUN_0014f850) are documented where used.
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q12_e::{ee_sub_209130, ee_sub_2090d0};
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};
use crate::engine::classes::q::q6::{rof, vcall};
use crate::engine::classes::q::q7util::{f, fr, CLOCK, WORLD};

const NEG1: u32 = u32::MAX;

fn len3(v: V4) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// FUN_0020a540(o, p): +0x2e8 = p; clear +0x2dc (0xc); the 0x9c component (kind 1) gets a -1 / 1 message
pub fn ee_sub_20a540<M: QMem + ?Sized>(m: &mut M, o: u32, p: u32) {
    if m.r32(o + 0x2e8) != p {
        m.w32(o + 0x2e8, p);
        m.call(0x0023_33b4, &[o + 0x2dc, 0, 0xc], &[]);
        let l = m.call(0x001e_eee0, &[o + 0x26c, 0x9c], &[]).0;
        if l != 0 && m.r32(l + 0x30) == 1 {
            let b = stack_push(m, 0x70);
            m.call(0x0023_33b4, &[b, 0, 0x70], &[]);
            m.w32(b, NEG1);
            m.call(0x0017_6750, &[l, b], &[]);
            m.call(0x0023_33b4, &[o + 0x2dc, 0, 0xc], &[]);
            m.w32(b, 1);
            m.call(0x0017_6750, &[l, b], &[]);
            stack_pop(m, 0x70);
        }
    }
}
/// FUN_0020a658(o, id): index of id in the +0x2c list (count +0x28), -1
pub fn ee_sub_20a658<M: QMem + ?Sized>(m: &mut M, o: u32, id: u32) -> i32 {
    let mut i = 0;
    if 0 < m.r32(o + 0x28) as i32 {
        let mut p = o + 0x2c;
        loop {
            if id == m.r32(p) {
                return i;
            }
            i += 1;
            p += 4;
            if !(i < m.r32(o + 0x28) as i32) {
                break;
            }
        }
    }
    -1
}
/// FUN_0020a690(o): kind 0: FUN_002458f0(DAT_002ddfa4, DAT_002ddfa0); kind 1: the same of the host's
/// virtual +0xfc results ({[+4]}, {[0]})
pub fn ee_sub_20a690<M: QMem + ?Sized>(m: &mut M, o: u32) -> f32 {
    let k = m.r32(o + 0x10) as i32;
    if k == 0 {
        let (a, b) = (m.rf(0x002d_dfa4), m.rf(0x002d_dfa0));
        return fr(m.call(0x0024_58f0, &[], &[f(a), f(b)]));
    }
    if 0 < k && k == 1 {
        let h = m.r32(o + 0xc);
        let i2 = vcall(m, h, 0xc, 0xfc, &[], &[]).0;
        let h = m.r32(o + 0xc);
        let p3 = vcall(m, h, 0xc, 0xfc, &[], &[]).0;
        let (a, b) = (m.rf(i2 + 4), m.rf(p3));
        return fr(m.call(0x0024_58f0, &[], &[f(a), f(b)]));
    }
    0.0
}
/// FUN_0020a730(o, id): first entry sets [0] = clock + FUN_0020a9a8(o, 0); append id to the +0x2c list
pub fn ee_sub_20a730<M: QMem + ?Sized>(m: &mut M, o: u32, id: u32) {
    if m.r32(o + 0x28) == 0 {
        let c = m.r32(CLOCK);
        m.w32(o, c);
        ee_sub_20a9a8(m, o, 0);
    }
    let n = m.r32(o + 0x28);
    m.w32(o + 0x2c + n * 4, id);
    m.w32(o + 0x28, n + 1);
}
/// FUN_0020a790(o, id): remove id from the +0x2c list (FUN_001ae5a0 shift)
pub fn ee_sub_20a790<M: QMem + ?Sized>(m: &mut M, o: u32, id: u32) {
    let i = ee_sub_20a658(m, o, id);
    let n = (m.r32(o + 0x28) as i32).wrapping_sub(1);
    let p = o.wrapping_add((i as u32).wrapping_mul(4)).wrapping_add(0x2c);
    m.w32(o + 0x28, n as u32);
    m.call(0x001a_e5a0, &[p, p + 4, (n.wrapping_sub(i) as u32).wrapping_mul(4)], &[]);
}
fn angle_of<M: QMem + ?Sized>(m: &mut M, o: u32, d: V4) -> f32 {
    let buf = stack_push(m, 0x10);
    m.call(0x0011_ef08, &[buf, 0, 0], &[d]);
    let a0 = m.rf(buf);
    stack_pop(m, 0x10);
    let ang = ee_sub_20a690(m, o);
    fr(m.call(0x0011_cf60, &[], &[f(a0 - ang)]))
}
/// FUN_0020a818(o, p2): 1 when the follow condition holds (p2 = the follower: +0xf30 / +0xf34 flags, list
/// +0x2c of hosts whose +0xf3c blocks, mode +0x10: 0 near test through FUN_0014f850 + angle window +0x18, 1 angle
/// window only, other 1). FUN_0014f850 / FUN_0011ef08 take 128-bit vector args (1:1 lanes).
pub fn ee_sub_20a818<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) -> u32 {
    if m.r32(o + 0x1c) != 0 {
        return 0;
    }
    if m.r32(p2 + 0xf30) == 0 {
        let mut i = 0;
        if 0 < m.r32(o + 0x28) as i32 {
            let mut p = o + 0x2c;
            loop {
                i += 1;
                let h = m.r32(p);
                if m.r32(h + 0xf3c) != 0 {
                    return 0;
                }
                p += 4;
                if !(i < m.r32(o + 0x28) as i32) {
                    break;
                }
            }
        }
    }
    let mode = m.r32(o + 0x10) as i32;
    if mode != 0 {
        if 0 < mode {
            if mode != 1 {
                return 1;
            }
            let pos = m.call(0x001a_62e8, &[p2], &[]).1;
            let oc = m.r32(o + 0xc);
            let q = m.call(0x001a_62e8, &[oc], &[]).1;
            let r = angle_of(m, o, [pos[0] - q[0], pos[1] - q[1], pos[2] - q[2], pos[3] - q[3]]);
            if m.rf(o + 0x18) < r.abs() {
                return 0;
            }
        }
        return 1;
    }
    if m.r32(p2 + 0xf34) != 0 {
        return 1;
    }
    let pos = m.call(0x001a_62e8, &[p2], &[]).1;
    let k = m.rv(0x002d_dfd0);
    let d = [pos[0] - k[0], pos[1] - k[1], pos[2] - k[2], pos[3] - k[3]];
    // a2 = mfc1 0.0 over the FUN_001a62e8 result register: low 64 bits zeroed, upper lanes kept (z, w of pos)
    let l = m.call(0x0014_f850, &[0x002d_df40], &[d, [0.0, 0.0, pos[2], pos[3]]]).0;
    if l == 0 {
        return 0;
    }
    let oc = m.r32(o + 0xc);
    let q = m.call(0x001a_62e8, &[oc], &[]).1;
    let r = angle_of(m, o, [pos[0] - q[0], pos[1] - q[1], pos[2] - q[2], pos[3] - q[3]]);
    (r.abs() <= m.rf(o + 0x18)) as u32
}
/// FUN_0020a9a8(o, v): [2] = v, [1] = clock
pub fn ee_sub_20a9a8<M: QMem + ?Sized>(m: &mut M, o: u32, v: u32) {
    let c = m.r32(CLOCK);
    m.w32(o + 8, v);
    m.w32(o + 4, c);
}
/// FUN_0020a9c0(o): lazily allocated 0x78-byte helper (FUN_0013e3c8) registered in +0xedc / +0xeb8 (id 0x2cc)
pub fn ee_sub_20a9c0<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    if m.r32(o + 0xed8) == 0 {
        let u = m.call(0x0013_e3c8, &[0x78, o, 0x2cc, NEG1], &[]).0;
        let i = m.r32(o + 0xefc);
        m.w32(o + i * 4 + 0xedc, u);
        m.w32(o + 0xefc, i + 1);
        let i = m.r32(o + 0xed8);
        m.w32(o + i * 4 + 0xeb8, 0x2cc);
        m.w32(o + 0xed8, i + 1);
        u
    } else {
        m.r32(o + 0xedc)
    }
}
/// FUN_0020ac28(o, p2, p3, p4): drop the old +0xf0c job; +0xf08 = p2; with p2 and its +0xa0: +0xf0c =
/// FUN_0012dc18(o, [p2+0xa0], 5, 8, 3, DAT_002a2dd0), vector +0xf20 = p4, +0xf10 = p3
pub fn ee_sub_20ac28<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, p3: u32, p4: V4) {
    if m.r32(o + 0xf08) != 0 {
        m.call(0x0012_e460, &[o + 0xf0c], &[]);
        m.w32(o + 0xf0c, 0);
        m.w32(o + 0xf08, 0);
    }
    m.w32(o + 0xf08, p2);
    if p2 != 0 {
        let i1 = m.r32(p2 + 0xa0);
        if i1 != 0 {
            let d = m.r32(0x002a_2dd0);
            let u = m.call(0x0012_dc18, &[o, i1, 5, 8, 3, d], &[]).0;
            m.wv(o + 0xf20, p4);
            m.w32(o + 0xf0c, u);
            m.w32(o + 0xf10, p3);
        }
    }
}
/// FUN_0020acc0(o): job +0xf0c vector +0x20 = FUN_0011bff0(FUN_0011ba08(buf, [src+0x90], FUN_0022fc48(+0xf10,
/// [[src+0xa0]+0xa0]) - [[src+0xa0]+0x30].x)); src = [o+0xf08]
pub fn ee_sub_20acc0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let src = m.r32(o + 0xf08);
    let s1 = m.r32(src + 0xa0);
    let a = m.r32(o + 0xf10);
    let b = m.r32(s1 + 0xa0);
    let u = m.call(0x0022_fc48, &[a, b], &[]).0;
    let v = m.rv(s1 + 0x30);
    let d = f32::from_bits(u) - v[0];
    let buf = stack_push(m, 0x30);
    let w = m.r32(src + 0x90);
    m.call(0x0011_ba08, &[buf, w, d.to_bits()], &[]);
    let job = m.r32(o + 0xf0c);
    let r = m.call(0x0011_bff0, &[buf], &[]).1;
    m.wv(job + 0x20, r);
    stack_pop(m, 0x30);
}
/// FUN_0020af38(o, a, b): FUN_002090d0(o + 0xb0); +0xbc = b; +0xb4 = a
pub fn ee_sub_20af38<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32, b: u32) {
    ee_sub_2090d0(m, o + 0xb0);
    m.w32(o + 0xbc, b);
    m.w32(o + 0xb4, a);
}
/// FUN_0020b060(o): FUN_0020aff0(o); FUN_00209130(o + 0xb0, [o+0xac])
pub fn ee_sub_20b060<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0020_aff0, &[o], &[]);
    let id = m.r32(o + 0xac);
    ee_sub_209130(m, o + 0xb0, id);
}
/// FUN_0020b210(o): release the attached object by kind (+0xf80: 0..3 -> FUN_00204f78(o, +0xf84, 0), +0xf84 = 0)
pub fn ee_sub_20b210<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let k = m.r32(o + 0xf80) as i32;
    if (0..=3).contains(&k) {
        let t = m.r32(o + 0xf84);
        m.call(0x0020_4f78, &[o, t, 0], &[]);
    }
    m.w32(o + 0xf84, 0);
    m.w32(o + 0xf80, NEG1);
}
/// FUN_0020b2a8(o, t, p3): attach t (+0xf84, +0xf8c = p3): isa 0xb -> kind 1 (FUN_001e6f60 with pos of +0xf90);
/// isa 0xd -> kind 0 (pos of o); isa 0x13 with +0x124 == 3 -> kind 2; isa 0x27 -> kind 3; else +0xf84 = 0
pub fn ee_sub_20b2a8<M: QMem + ?Sized>(m: &mut M, o: u32, t: u32, p3: u32) {
    m.w32(o + 0xf84, t);
    m.w32(o + 0xf8c, p3);
    if m.call(0x0013_2b10, &[t, 0xb], &[]).0 != 0 {
        m.w32(o + 0xf80, 1);
        let a = m.r32(o + 0xf90);
        let pos = m.call(0x001a_62e8, &[a], &[]).1;
        let t = m.r32(o + 0xf84);
        m.call(0x001e_6f60, &[t, 0, 0, 0, 0, o + 0xf88], &[pos]);
    } else if m.call(0x0013_2b10, &[t, 0xd], &[]).0 == 0 {
        let t = m.r32(o + 0xf84);
        if m.call(0x0013_2b10, &[t, 0x13], &[]).0 != 0 && m.r32(t + 0x124) == 3 {
            m.w32(o + 0xf80, 2);
            m.call(0x0020_4f78, &[o, t, 1], &[]);
            return;
        }
        let t = m.r32(o + 0xf84);
        if m.call(0x0013_2b10, &[t, 0x27], &[]).0 == 0 {
            m.w32(o + 0xf84, 0);
            return;
        }
        m.w32(o + 0xf80, 3);
    } else {
        m.w32(o + 0xf80, 0);
        let pos = m.call(0x001a_62e8, &[o], &[]).1;
        let t = m.r32(o + 0xf84);
        m.call(0x001e_6f60, &[t, 0, 0, 0, 0, o + 0xf88], &[pos]);
    }
    let t = m.r32(o + 0xf84);
    m.call(0x0020_4f78, &[o, t, 1], &[]);
}
/// FUN_0020b438(o): +0xffc 0 -> 1: FUN_001a6ca8(o, 1)
pub fn ee_sub_20b438<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let mut c = m.r32(o + 0xffc);
    if c == 0 {
        m.call(0x001a_6ca8, &[o, 1], &[]);
        c = m.r32(o + 0xffc);
    }
    m.w32(o + 0xffc, c + 1);
}
/// FUN_0020b478(o): +0xffc -= 1; 0 -> FUN_001a6ca8(o, 0)
pub fn ee_sub_20b478<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let c = m.r32(o + 0xffc).wrapping_sub(1);
    m.w32(o + 0xffc, c);
    if c == 0 {
        m.call(0x001a_6ca8, &[o, 0], &[]);
    }
}
/// pools init shared by 20b6e8 / 20ccd0 / 20d478 / 20d878 (identical bodies, each its own copy in the original):
/// six refcounted FUN_0011cca8 pools {0x4520a8, 0x14, 0x2000}, {0x4520d0, 0x18, 0x800}, {0x4520f8, 0x28, 0x200},
/// {0x452120, 0x1c, 0x800}, {0x452148, 0xc, 200}, {0x452170, 0x14, 0x100}, align 0x10
fn pools<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32) {
    if p2 == 0xffff && p1 != 0 {
        for (cnt, pool, sz, n) in [
            (0x002d_d2e0u32, 0x0045_20a8u32, 0x14u32, 0x2000u32),
            (0x002d_d2e4, 0x0045_20d0, 0x18, 0x800),
            (0x002d_d2e8, 0x0045_20f8, 0x28, 0x200),
            (0x002d_d2ec, 0x0045_2120, 0x1c, 0x800),
            (0x002d_d2f0, 0x0045_2148, 0xc, 200),
            (0x002d_d2f4, 0x0045_2170, 0x14, 0x100),
        ] {
            let c = m.r32(cnt).wrapping_add(1);
            m.w32(cnt, c);
            if c == 1 {
                m.call(0x0011_cca8, &[pool, sz, n, 0x10], &[]);
            }
        }
    }
}
pub fn ee_sub_20b6e8<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32) {
    pools(m, p1, p2);
}
pub fn ee_sub_20b820<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_20b6e8(m, 1, 0xffff);
}
pub fn ee_sub_20ccd0<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32) {
    pools(m, p1, p2);
}
pub fn ee_sub_20ce08<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_20ccd0(m, 1, 0xffff);
}
pub fn ee_sub_20d478<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32) {
    pools(m, p1, p2);
}
pub fn ee_sub_20d5b0<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_20d478(m, 1, 0xffff);
}
pub fn ee_sub_20d878<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32) {
    pools(m, p1, p2);
}
pub fn ee_sub_20d9b0<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_20d878(m, 1, 0xffff);
}
/// FUN_0020bb48(o): [[o+4]+0x50]
pub fn ee_sub_20bb48<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    let a = m.r32(o + 4);
    m.r32(a + 0x50)
}
/// FUN_0020bc08(o): for the two +0x9c flags: flag = ([[[o+4]+0xd0]+12i+8].y <= FUN_0011d898(pos.f - ..)); when
/// it falls to 0 an effect (FUN_001fbb90 / 1fb2f8 / 1fb3f0 / 1fb4f0(512, 2000) / 1fb1f8(lerp +0x98, +0x94 by +0x50))
/// is started
pub fn ee_sub_20bc08<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let mut pu = o + 0x9c;
    let mut off = 0;
    let a = m.r32(o + 4);
    let i1 = m.r32(a + 0x50);
    let f12 = m.rf(i1 + 0x1c);
    for _ in 0..2 {
        let a = m.r32(o + 4);
        let tab = m.r32(a + 0xd0);
        let pf2 = m.r32(off + tab + 8);
        let a74 = m.rf(a + 0x74);
        let x = f12 - m.rf(pf2);
        let fv10 = fr(m.call(0x0011_d898, &[x.to_bits()], &[f(a74)]));
        let u7 = (m.rf(pf2 + 4) <= fv10) as u32;
        if u7 != m.r32(pu) {
            if u7 == 0 {
                let f13 = m.rf(o + 0x50);
                let f10 = m.rf(o + 0x98);
                let f11 = m.rf(o + 0x94);
                let e = m.r32(o + 0x90);
                let u3 = m.call(0x001f_bb90, &[e], &[]).0;
                m.call(0x001f_b2f8, &[u3, 0], &[]);
                let t = m.r32(i1 + 0x18);
                let pos = m.call(0x001a_62e8, &[t], &[]).1;
                m.call(0x001f_b3f0, &[u3], &[pos]);
                m.call(0x001f_b4f0, &[u3], &[f(512.0), f(2000.0)]);
                m.call(0x001f_b1f8, &[u3], &[f(f13 * f10 + (1.0 - f13) * f11)]);
                m.w32(pu, 0);
            } else {
                m.w32(pu, u7);
            }
        }
        pu += 4;
        off += 0xc;
    }
}
/// FUN_0020bd88(o, p2): resolve [o+4] = FUN_0015e938(world, 4, id, p2); curve ends from its 0x22 / 0x23 / 0x24
/// properties -> +8, +0xc, +0x10, +0x14 (= p4 - p2), +0x18 (= p4 - p3); path length +0x1c; +0x28 = height
/// gradient; +0x30 = 1
pub fn ee_sub_20bd88<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    if m.r32(o + 0x30) != 0 {
        return;
    }
    if m.r32(o + 4) == 0 {
        let id = m.r32(o);
        let l = m.call(0x0015_e938, &[WORLD, 4, id, p2], &[]).0;
        m.w32(o + 4, l);
        if l == 0 {
            return;
        }
    }
    let u = m.r32(o + 4);
    let l2 = m.call(0x0012_f938, &[u, 0x22], &[]).0;
    if l2 == 0 {
        return;
    }
    let u = m.r32(o + 4);
    let l3 = m.call(0x0012_f938, &[u, 0x23], &[]).0;
    if l3 == 0 {
        return;
    }
    let u = m.r32(o + 4);
    let l4 = m.call(0x0012_f938, &[u, 0x24], &[]).0;
    if l4 == 0 {
        return;
    }
    let (f7, f8, f6) = (m.rf(l2), m.rf(l3), m.rf(l4));
    m.wf(o + 8, f7);
    m.wf(o + 0xc, f8);
    m.wf(o + 0x10, f6);
    m.wf(o + 0x14, f6 - f7);
    m.wf(o + 0x18, f6 - f8);
    let u = m.r32(o + 4);
    let l = m.call(0x0012_ff10, &[u, p2, 1, 1], &[]).0;
    if l != 0 {
        let b = stack_push(m, 0x40);
        let (va, vb, vc) = (b, b + 0x10, b + 0x20);
        let t = m.r32(l + 4);
        let vt = m.r32(t);
        let fnp = m.r32(vt + 0x1c);
        let adj = m.r16(vt + 0x18) as i16 as i32;
        let this = t.wrapping_add(adj as u32);
        let c2 = m.r32(o + 8);
        m.call(fnp, &[c2, 0x3f80_0000, this, va, 0], &[]);
        let c4 = m.r32(o + 0x10);
        m.call(fnp, &[c4, 0x3f80_0000, this, vb, 0], &[]);
        let (a, bq) = (m.rv(va), m.rv(vb));
        let d = [a[0] - bq[0], a[1] - bq[1], a[2] - bq[2], a[3] - bq[3]];
        m.wf(o + 0x1c, len3(d));
        let fv7 = (m.rf(o + 0x10) - m.rf(o + 8)) * 0.5;
        let x = m.rf(o + 8) + fv7;
        m.call(fnp, &[x.to_bits(), 0x3f80_0000, this, vc, 0], &[]);
        m.w32(o + 0x20, 0);
        m.w32(o + 0x24, 0);
        let az = m.rf(va + 8);
        let cz = m.rf(vc + 8);
        m.wf(o + 0x28, (az - cz) / (fv7 * fv7 * 0.5));
        stack_pop(m, 0x40);
    }
    m.w32(o + 0x30, 1);
}
/// FUN_0020bf80(o, p2, p3, buf): with [o+4]: buf (0x70) cleared + FUN_0020c058 init, +0x30 = o, +0x34 = p2,
/// +0x3c = p3, FUN_0020c2f8(buf, 0)
pub fn ee_sub_20bf80<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32, p3: u32, buf: u32) {
    if m.r32(o + 4) != 0 {
        m.call(0x0023_33b4, &[buf, 0, 0x70], &[]);
        m.call(0x0023_33b4, &[buf, 0, 0x70], &[]);
        ee_sub_20c058(m, buf);
        m.w32(buf + 0x30, o);
        m.w32(buf + 0x34, p2);
        m.w32(buf + 0x3c, p3);
        ee_sub_20c2f8(m, buf, 0);
    }
}
/// FUN_0020c010 / FUN_0020c030 / FUN_0020ca00 / FUN_0020ca20: pool alloc / free of 0x3e1ad8 / 0x3e1af0
pub fn ee_sub_20c010<M: QMem + ?Sized>(m: &mut M) -> u32 {
    m.call(0x0011_ccc0, &[0x003e_1ad8], &[]).0
}
pub fn ee_sub_20c030<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.call(0x0011_cd98, &[0x003e_1ad8, p], &[]);
}
pub fn ee_sub_20ca00<M: QMem + ?Sized>(m: &mut M) -> u32 {
    m.call(0x0011_ccc0, &[0x003e_1af0], &[]).0
}
pub fn ee_sub_20ca20<M: QMem + ?Sized>(m: &mut M, p: u32) {
    m.call(0x0011_cd98, &[0x003e_1af0, p], &[]);
}
/// FUN_0020c058 (C2daa10_ctor)
pub fn ee_sub_20c058<M: QMem + ?Sized>(m: &mut M, o: u32) -> u32 {
    m.w32(o + 0x10, NEG1);
    m.w32(o + 0xc, 0x002d_aa10);
    m.w32(o, 0);
    m.w32(o + 4, 0);
    o
}
/// FUN_0020c080(o): not in state 4: FUN_0020c2f8(o, 4); FUN_001305b8(0, 1.0, 0, 0, 0, o + 0x38)
pub fn ee_sub_20c080<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if !ee_sub_20c0d8(m, o) {
        ee_sub_20c2f8(m, o, 4);
        m.call(0x0013_05b8, &[0, 0x3f80_0000, 0, 0, 0, o + 0x38], &[]);
    }
}
pub fn ee_sub_20c0d8<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    m.r32(o + 0x10) == 4
}
/// FUN_0020c0e8(o): state machine on +0x10 (2: wait / end conditions -> 3 or FUN_0020d1a0; 1 -> 3 / 2; 3 ->
/// 4 when FUN_00131968(+0x38)); loops until the state is stable
pub fn ee_sub_20c0e8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let mut st = m.r32(o + 0x10) as i32;
    loop {
        let clock = m.rf(CLOCK);
        let mut to3 = false;
        match st {
            2 => {
                let t = m.r32(o + 0x34);
                if m.r32(t + 0x208) != 0 || rof(0x002c_3de0) < clock - m.rf(o + 0x50) {
                    to3 = true;
                } else if m.rf(o + 0x50) <= clock {
                    if m.r32(o + 0x60) != 0 {
                        to3 = true;
                    } else {
                        let t = m.r32(o + 0x34);
                        if m.call(0x0013_2b10, &[t, 0x38], &[]).0 != 0 {
                            let a = m.r32(t + 0x498);
                            ee_sub_20d1a0(m, a);
                        }
                    }
                }
            }
            1 => {
                let t = m.r32(o + 0x34);
                if m.r32(t + 0x208) != 0 && m.rf(o + 0x14) + 0.15 < clock {
                    to3 = true;
                } else if !(clock < m.rf(o + 0x54)) {
                    ee_sub_20c2f8(m, o, 2);
                }
            }
            3 => {
                let a = m.r32(o + 0x38);
                if m.call(0x0013_1968, &[a], &[]).0 != 0 {
                    ee_sub_20c2f8(m, o, 4);
                }
            }
            _ => {}
        }
        if to3 {
            ee_sub_20c2f8(m, o, 3);
        }
        let now = m.r32(o + 0x10) as i32;
        if st == now {
            return;
        }
        st = now;
    }
}
/// FUN_0020c2f8(o, k): +0x10 = k, +0x14 = clock; k < 5: enter handler PTR_LAB_002a88f0[k] (o, k)
pub fn ee_sub_20c2f8<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    if k != m.r32(o + 0x10) {
        m.w32(o + 0x10, k);
        let c = m.r32(CLOCK);
        m.w32(o + 0x14, c);
        if k < 5 {
            let h = m.r32(0x002a_88f0 + k * 4);
            m.call(h, &[o, k], &[]);
        }
    }
}
/// FUN_0020c6f8(o, v): state 0: vector +0x40 = v; states 1..2 with +0x60: job (+0x68) +0x20 =
/// FUN_0022fc48(0, [[[o+0x34]+0x58]+0xa0], v.x)
pub fn ee_sub_20c6f8<M: QMem + ?Sized>(m: &mut M, o: u32, v: V4) {
    let k = m.r32(o + 0x10) as i32;
    if k == 0 {
        m.wv(o + 0x40, v);
    } else if (0..3).contains(&k) && m.r32(o + 0x60) != 0 {
        let i1 = m.r32(o + 0x68);
        let t = m.r32(o + 0x34);
        let a = m.r32(t + 0x58);
        let b = m.r32(a + 0xa0);
        let r = m.call(0x0022_fc48, &[0, b, v[0].to_bits()], &[]).1;
        m.wv(i1 + 0x20, r);
    }
}
pub fn ee_sub_20c7d8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0xc0, 0);
}
/// FUN_0020c7e0(o, p2): once: for the 4 records (0x30 bytes, [0] = id): resolve the curve / object, its ends:
/// +0x10 = (B - A) / [obj+0x74] (vector), +0x20 = |xyz of that|
pub fn ee_sub_20c7e0<M: QMem + ?Sized>(m: &mut M, o: u32, p2: u32) {
    if m.r32(o + 0xc0) != 0 {
        return;
    }
    let b = stack_push(m, 0x30);
    let (va, vb) = (b, b + 0x10);
    let mut p = o;
    let mut iv4 = m.r32(o);
    loop {
        let l = m.call(0x0015_e938, &[WORLD, 4, iv4, p2], &[]).0;
        iv4 = l;
        m.w32(p, l);
        if l != 0 {
            let i2 = m.call(0x0012_ff10, &[l, p2, 1, 1], &[]).0;
            let t = m.r32(i2 + 4);
            let vt = m.r32(t);
            let fnp = m.r32(vt + 0x1c);
            let adj = m.r16(vt + 0x18) as i16 as i32;
            let this = t.wrapping_add(adj as u32);
            m.call(fnp, &[0, 0x3f80_0000, this, va, 0], &[]);
            let w = m.r32(iv4 + 0x74);
            m.call(fnp, &[w, 0x3f80_0000, this, vb, 0], &[]);
            let x = m.rf(iv4 + 0x74);
            let (a, bq) = (m.rv(va), m.rv(vb));
            let q = 1.0 / x;
            let v: V4 = std::array::from_fn(|i| (bq[i] - a[i]) * q);
            m.wv(p + 0x10, v);
            m.wf(p + 0x20, len3(v));
        }
        p += 0x30;
        if o + 0xc0 <= p {
            break;
        }
        iv4 = m.r32(p);
    }
    stack_pop(m, 0x30);
    m.w32(o + 0xc0, 1);
}
/// FUN_0020c948 (C2da9d0_ctor_20c948)(a1, p2, p3, buf)
pub fn ee_sub_20c948<M: QMem + ?Sized>(m: &mut M, a1: u32, p2: u32, p3: u32, buf: u32) {
    m.call(0x0023_33b4, &[buf, 0, 0x40], &[]);
    m.call(0x0023_33b4, &[buf, 0, 0x40], &[]);
    m.w32(buf + 0x28, a1);
    m.w32(buf + 0x20, p2);
    m.w32(buf + 0xc, 0x002d_a9d0);
    m.w32(buf + 0x30, p3);
    m.w32(buf, 0);
    m.w32(buf + 4, 0);
    let s = m.r32(a1 + 0x30);
    m.call(0x0013_05b8, &[0, 0x3f80_0000, s, p2, p3, buf + 0x24], &[]);
    let h = m.r32(buf + 0x24);
    m.call(0x0013_33c0, &[h, buf], &[]);
    m.w32(buf + 0x2c, a1 + 0x30);
}
pub fn ee_sub_20ca48<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if !ee_sub_20ca98(m, o) {
        m.call(0x0013_05b8, &[0, 0x3f80_0000, 0, 0, 0, o + 0x24], &[]);
    }
}
pub fn ee_sub_20ca98<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    m.r32(o + 0x24) == 0
}
/// FUN_0020cb28(o, p2): direction input p2 (float bits, x lane of the VU register: qmtc2) normalised; picks the
/// best of the 4 records at [o+0x28] (0x30 bytes: id, dir +0x10, speed +0x20) by dot product (x DAT_002c3de4
/// for the current id), starts its sound (FUN_001305b8) and stores dir * speed at +0x10. p2 is the full
/// 128-bit vector (qmtc2 a1): vv = p2 / |p2.xyz| on all four lanes.
pub fn ee_sub_20cb28<M: QMem + ?Sized>(m: &mut M, o: u32, p2: V4) {
    let q = 1.0 / (p2[0] * p2[0] + p2[1] * p2[1] + p2[2] * p2[2]).sqrt();
    let vv: V4 = p2.map(|c| c * q);
    let pi2;
    if m.r32(o + 0x34) == 0 && m.r32(o + 0x38) == 0 {
        pi2 = m.r32(o + 0x2c);
    } else {
        let mut p = m.r32(o + 0x28);
        let mut f8 = m.rf(0x002a_8904);
        let mut pi6 = 0;
        let mut pi7 = 0;
        for _ in 0..4 {
            let mut f9 = f8;
            pi7 = pi6;
            if m.r32(p) != 0 {
                let v13 = m.rv(p + 0x10);
                let il = 1.0 / len3(v13);
                let n: V4 = v13.map(|c| c * il);
                f9 = vv[0] * n[0] + vv[1] * n[1] + vv[2] * n[2];
                let a = m.r32(o + 0x24);
                if m.r32(p) == m.r32(a + 0x14) {
                    f9 *= rof(0x002c_3de4);
                }
                pi7 = p;
                if f9 <= f8 {
                    f9 = f8;
                    pi7 = pi6;
                }
            }
            p += 0x30;
            f8 = f9;
            pi6 = pi7;
        }
        let cur = m.r32(o + 0x2c);
        if pi7 == cur {
            pi2 = cur;
        } else {
            let mut uv10 = 0;
            let mut uv16 = 0x17d;
            let i1 = m.r32(pi7);
            if m.r32(o + 0x34) == 0 {
                uv16 = 0x17f;
            }
            let l = m.call(0x0012_f938, &[i1, uv16], &[]).0;
            if l != 0 {
                uv10 = m.r32(l);
            }
            let (a, b, c) = (m.r32(pi7), m.r32(o + 0x20), m.r32(o + 0x30));
            m.call(0x0013_05b8, &[uv10, 0x3f80_0000, a, b, c, o + 0x24], &[]);
            let h = m.r32(o + 0x24);
            m.call(0x0013_33c0, &[h, o], &[]);
            m.w32(o + 0x2c, pi7);
            pi2 = m.r32(o + 0x2c);
        }
    }
    let s = m.rf(pi2 + 0x20);
    m.wv(o + 0x10, vv.map(|c| c * s));
}
/// FUN_0020ce28(o, p): [0] = p, [3] = 1, [1] = -1
pub fn ee_sub_20ce28<M: QMem + ?Sized>(m: &mut M, o: u32, p: u32) {
    m.w32(o, p);
    m.w32(o + 0xc, 1);
    m.w32(o + 4, NEG1);
}
pub fn ee_sub_20ce40<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 4, NEG1);
    m.w32(o + 0xc, 1);
    ee_sub_20d108(m, o, 0);
}
/// FUN_0020ce70(o): state machine on +4 (0 -> 1 when FUN_0020d020; 1 -> 0 / 2; 2 -> 0)
pub fn ee_sub_20ce70<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0xc) == 0 {
        return;
    }
    let l2 = ee_sub_20d020(m, o);
    let mut st = m.r32(o + 4) as i32;
    loop {
        match st {
            1 => {
                if !l2 {
                    ee_sub_20d108(m, o, 0);
                } else if rof(0x002c_3df0) < m.rf(CLOCK) - m.rf(o + 8) {
                    let h = m.r32(o);
                    if vcall(m, h, 0xc, 0x264, &[0x4000], &[]).0 == 0 {
                        ee_sub_20d108(m, o, 2);
                    }
                }
            }
            0 => {
                if l2 {
                    ee_sub_20d108(m, o, 1);
                }
            }
            2 => {
                if !l2 {
                    ee_sub_20d108(m, o, 0);
                }
            }
            _ => {}
        }
        let now = m.r32(o + 4) as i32;
        if now == st {
            return;
        }
        st = now;
    }
}
/// FUN_0020cfc0(p1, p2): flag bit 48 of [p2+0x10] and isa([p2+4], 0x59) -> 1, else FUN_0020fa50
pub fn ee_sub_20cfc0<M: QMem + ?Sized>(m: &mut M, p1: u32, p2: u32) -> u32 {
    if m.r64(p2 + 0x10) & 0x1_0000_0000_0000 != 0 {
        let t = m.r32(p2 + 4);
        if m.call(0x0013_2b10, &[t, 0x59], &[]).0 != 0 {
            return 1;
        }
    }
    m.call(0x0020_fa50, &[p1, p2], &[]).0
}
/// FUN_0020d020(o): the host ([o]) is free (+0x208 == 0, +0xf80 == -1, +0x4a0 != 0, not state 0x13) and all
/// 5 probe points (DAT_002c3e00 + 16 i, pos by FUN_001a6f00) pass FUN_00173610(DAT_002c3e50, DAT_002c3e54, ..)
pub fn ee_sub_20d020<M: QMem + ?Sized>(m: &mut M, o: u32) -> bool {
    let i1 = m.r32(o);
    let mut b = false;
    if m.r32(i1 + 0x208) == 0 && m.r32(i1 + 0xf80) == NEG1 && m.r32(i1 + 0x4a0) != 0 {
        b = false;
        if m.call(0x0020_9a68, &[i1, 0x13], &[]).0 == 0 {
            let buf = stack_push(m, 0x10);
            let mut u5 = 0;
            let mut p6 = 0x002c_3e00;
            loop {
                let h = m.r32(o);
                let pv = [m.rf(p6), m.rf(p6 + 4), 0.0, 0.0];
                let r = m.call(0x001a_6f00, &[h, 0], &[pv]).1;
                m.wf(buf, r[0]);
                m.wf(buf + 4, r[1]);
                let (c0, c1) = (m.r32(0x002c_3e50), m.r32(0x002c_3e54));
                if m.call(0x0017_3610, &[c0, c1, buf, 1, 0x0020_cfc0], &[]).0 != 0 {
                    break;
                }
                u5 += 1;
                p6 += 0x10;
                if !(u5 < 5) {
                    break;
                }
            }
            stack_pop(m, 0x10);
            b = 4 < u5;
        }
    }
    b
}
/// FUN_0020d108(o, k): +4 = k, +8 = clock; k == 2: FUN_001f7598({[o], +8 = [o], +0xc = 0x4000})
pub fn ee_sub_20d108<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    if m.r32(o + 4) != k {
        m.w32(o + 4, k);
        let c = m.r32(CLOCK);
        m.w32(o + 8, c);
        if k == 2 {
            let b = stack_push(m, 0x60);
            m.call(0x0023_33b4, &[b, 0, 0x60], &[]);
            let h = m.r32(o);
            m.w32(b, h);
            m.w32(b + 0xc, 0x4000);
            m.w32(b + 0x10, 0);
            m.w32(b + 8, h);
            m.call(0x001f_7598, &[b], &[]);
            stack_pop(m, 0x60);
        }
    }
}
pub fn ee_sub_20d178<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0xc, 0);
    ee_sub_20d108(m, o, 0);
}
pub fn ee_sub_20d1a0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0xc, 1);
}
pub fn ee_sub_20d1b0<M: QMem + ?Sized>(m: &mut M, o: u32, p: u32) {
    m.w32(o, p);
    m.w32(o + 0x10, 1);
    m.w32(o + 4, NEG1);
}
pub fn ee_sub_20d1c8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 4, NEG1);
    m.w32(o + 0x10, 1);
    ee_sub_20d3c0(m, o, 0);
}
/// FUN_0020d1f8(o): state machine on +4 (0 -> 1 when the target (+0xc) accepts the host position (virtual +0x24c)
/// and the host's virtual +0x10c result z < 0; 1 -> 0 / 2; 2 -> 0)
pub fn ee_sub_20d1f8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0x10) == 0 {
        return;
    }
    let mut st = m.r32(o + 4) as i32;
    loop {
        match st {
            1 => {
                let t = m.r32(o + 0xc);
                if t == 0 {
                    ee_sub_20d3c0(m, o, 0);
                } else if rof(0x002c_3e58) < m.rf(CLOCK) - m.rf(o + 8) {
                    let a = m.r32(t + 0x188);
                    let h = m.r32(o);
                    if vcall(m, h, 0xc, 0x264, &[a], &[]).0 == 0 {
                        ee_sub_20d3c0(m, o, 2);
                    }
                }
            }
            0 => {
                let t = m.r32(o + 0xc);
                if t != 0 {
                    let mut b = false;
                    let vt = m.r32(t + 0xc);
                    let adj = m.r16(vt + 0x248) as i16 as i32;
                    let h = m.r32(o);
                    let pos = m.call(0x001a_62e8, &[h], &[]).1;
                    let fnp = m.r32(vt + 0x24c);
                    if m.call(fnp, &[t.wrapping_add(adj as u32)], &[pos]).0 != 0 {
                        let h = m.r32(o);
                        let r = vcall(m, h, 0xc, 0x10c, &[], &[]).1;
                        if r[2] < 0.0 {
                            b = true;
                        }
                    }
                    if b {
                        ee_sub_20d3c0(m, o, 1);
                    }
                }
            }
            2 => {
                let t = m.r32(o + 0xc);
                let go = if t == 0 {
                    true
                } else {
                    let a = m.r32(t + 0x188);
                    let h = m.r32(o);
                    vcall(m, h, 0xc, 0x264, &[a], &[]).0 != 0
                };
                if go {
                    ee_sub_20d3c0(m, o, 0);
                }
            }
            _ => {}
        }
        let now = m.r32(o + 4) as i32;
        if now == st {
            return;
        }
        st = now;
    }
}
/// FUN_0020d3c0(o, k): +4 = k, +8 = clock; k == 2: FUN_001f7598({target, +8 = host, +0xc = [t+0x188],
/// +0x10 = [t+0x184], +0x20 = pos(host) xy, +0x30 = host's virtual +0x10c result xy})
pub fn ee_sub_20d3c0<M: QMem + ?Sized>(m: &mut M, o: u32, k: u32) {
    if k != m.r32(o + 4) {
        m.w32(o + 4, k);
        let c = m.r32(CLOCK);
        m.w32(o + 8, c);
        if k == 2 {
            let b = stack_push(m, 0x60);
            m.call(0x0023_33b4, &[b, 0, 0x60], &[]);
            let t = m.r32(o + 0xc);
            let h = m.r32(o);
            m.w32(b, t);
            m.w32(b + 8, h);
            let v = m.r32(t + 0x188);
            m.w32(b + 0xc, v);
            let v = m.r32(t + 0x184);
            m.w32(b + 0x10, v);
            let pos = m.call(0x001a_62e8, &[h], &[]).1;
            m.wf(b + 0x20, pos[0]);
            m.wf(b + 0x24, pos[1]);
            let h = m.r32(o);
            let r = vcall(m, h, 0xc, 0x10c, &[], &[]).1;
            m.wf(b + 0x30, r[0]);
            m.wf(b + 0x34, r[1]);
            m.call(0x001f_7598, &[b], &[]);
            stack_pop(m, 0x60);
        }
    }
}
pub fn ee_sub_20d460<M: QMem + ?Sized>(m: &mut M, o: u32, v: u32) {
    m.w32(o + 0xc, v);
}
/// FUN_0020d5d0(o, p): [3] = p; [0], [1], [2], [4], [5] = 0
pub fn ee_sub_20d5d0<M: QMem + ?Sized>(m: &mut M, o: u32, p: u32) -> u32 {
    m.w32(o + 0xc, p);
    for off in [0, 4, 8, 0x10, 0x14] {
        m.w32(o + off, 0);
    }
    o
}
/// FUN_0020d5f0(o, a, b): [0] = a, [1] = b, [3] = -1, [2] = [4] = [5] = 0
pub fn ee_sub_20d5f0<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32, b: u32) -> u32 {
    m.w32(o, a);
    m.w32(o + 4, b);
    m.w32(o + 0xc, NEG1);
    for off in [8, 0x10, 0x14] {
        m.w32(o + off, 0);
    }
    o
}
/// FUN_0020d618(o, flags): flush; bit 0 -> FUN_001ae520(o)
pub fn ee_sub_20d618<M: QMem + ?Sized>(m: &mut M, o: u32, flags: u32) {
    ee_sub_20d798(m, o);
    if flags & 1 != 0 {
        m.call(0x001a_e520, &[o], &[]);
    }
}
/// FUN_0020d660(o, src, n): append n bytes: into the external buffer ([0], size [1], pos [2]) clamped, else into
/// the 0x400-byte line buffer at +0x18 (flush through FUN_0020d798 when full); +0x10 += written; returns n
pub fn ee_sub_20d660<M: QMem + ?Sized>(m: &mut M, o: u32, src: u32, n: i32) -> i32 {
    let mut n = n;
    let mut iv4: i32 = 0;
    if m.r32(o) != 0 && m.r32(o + 4) != 0 {
        if n == 0 {
            return 0;
        }
        let iv1 = (m.r32(o + 4) as i32).wrapping_sub(m.r32(o + 8) as i32);
        if iv1 < n {
            n = iv1;
        }
    }
    if n != 0 {
        if m.r32(o) == 0 {
            if 0 < n {
                let mut iv1 = m.r32(o + 0x14) as i32;
                loop {
                    let mut iv2 = 0x400 - iv1;
                    if n - iv4 <= 0x400 - iv1 {
                        iv2 = n - iv4;
                    }
                    if iv2 != 0 {
                        m.call(0x001a_e5a0, &[(o as i32 + iv1 + 0x18) as u32, (src as i32 + iv4) as u32, iv2 as u32], &[]);
                        iv4 += iv2;
                        let c = m.r32(o + 0x14).wrapping_add(iv2 as u32);
                        m.w32(o + 0x14, c);
                    }
                    if m.r32(o + 0x14) == 0x400 {
                        ee_sub_20d798(m, o);
                    }
                    if n <= iv4 {
                        break;
                    }
                    iv1 = m.r32(o + 0x14) as i32;
                }
            }
        } else {
            let d = m.r32(o).wrapping_add(m.r32(o + 8));
            m.call(0x001a_e5a0, &[d, src, n as u32], &[]);
            let p = m.r32(o + 8).wrapping_add(n as u32);
            m.w32(o + 8, p);
            iv4 = n;
        }
        let w = m.r32(o + 0x10).wrapping_add(iv4 as u32);
        m.w32(o + 0x10, w);
    }
    n
}
/// FUN_0020d798(o): flush the line buffer (+0x18, length +0x14) when no external buffer: file handle [3] < 0 ->
/// FUN_00236960(0x2a8910, text) in 0x80-byte pieces, else FUN_00238528(handle, buf)
pub fn ee_sub_20d798<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let n = m.r32(o + 0x14) as i32;
    if n != 0 && m.r32(o) == 0 {
        let mut p = o + 0x18;
        if (m.r32(o + 0xc) as i32) < 0 {
            let mut tot: i32 = 0;
            if 0 < n {
                let buf = stack_push(m, 0x90);
                loop {
                    let rem = n - tot;
                    let k = if rem < 0x80 { rem } else { 0x80 };
                    tot += k;
                    m.call(0x001a_e5a0, &[buf, p, k as u32], &[]);
                    p += k as u32;
                    m.w8(buf + k as u32, 0);
                    m.call(0x0023_6960, &[0x002a_8910, buf], &[]);
                    if !(tot < m.r32(o + 0x14) as i32) {
                        break;
                    }
                }
                stack_pop(m, 0x90);
                m.w32(o + 0x14, 0);
                return;
            }
        } else {
            let h = m.r32(o + 0xc);
            m.call(0x0023_8528, &[h, p], &[]);
        }
        m.w32(o + 0x14, 0);
    }
}
