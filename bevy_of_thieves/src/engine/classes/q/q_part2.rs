//! PART-2: ports of the remaining unported / partial EE rows (1:1 from disassembly, tools/eedis.py).
//! VU0 macro sequences are plain f32 lane math (vf00 = (0,0,0,1), so `vadda ACC, x, vf00` adds 1 to w).
#![allow(clippy::too_many_arguments)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};
use crate::engine::classes::q::q5util::f4;

fn add(a: V4, b: V4) -> V4 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}
fn sub(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}
fn scale(a: V4, s: f32) -> V4 {
    [a[0] * s, a[1] * s, a[2] * s, a[3] * s]
}
/// a + vf00 + b*s (vadda ACC, a, vf00; vmaddx d, b, s)
fn madd1(a: V4, b: V4, s: f32) -> V4 {
    [a[0] + b[0] * s, a[1] + b[1] * s, a[2] + b[2] * s, a[3] + 1.0 + b[3] * s]
}
fn dot3(a: V4, b: V4) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// FUN_001c6fd8(world, kind, pos, _, n): spawn `n` pickups (coins, kind 0x5b/0x5d) around `pos`:
/// ground-probe pos (FUN_001c6de8), find the SW region (FUN_001c4840 / FUN_001c0d70), project
/// within DAT_002c0e80 (xy), allocate n objects (FUN_001c6ce8 into a FUN_001ae318 frame buffer), then
/// spiral up to 50 tries (angle += DAT_002c0e94, radius += DAT_002c0e90 wrapping) placing each object
/// with a clear line (FUN_001c0980 both ways) + ground hit, launching it with FUN_001c6bd0 on a
/// ballistic arc (FUN_00196928 flight time). Unplaced objects get vtable +0x264(.., 2) (removed).
pub fn ee_sub_1c6fd8<M: QMem + ?Sized>(m: &mut M, world: u32, kind: u32, pos: V4, n: u32) {
    let sp = stack_push(m, 0xc0);
    run_1c6fd8(m, sp, world, kind, pos, n);
    stack_pop(m, 0xc0);
}

fn run_1c6fd8<M: QMem + ?Sized>(m: &mut M, sp: u32, world: u32, kind: u32, pos: V4, n: u32) {
    let k = m.rf(0x002a_775c);
    if m.call(0x001c_6de8, &[0, 0, sp], &[pos, f4(0.0), f4(300.0), f4(k)]).0 == 0 {
        return;
    }
    let s0 = m.call(0x001c_4840, &[world], &[]).0;
    let mut sw = 0;
    if s0 != 0 {
        let a = m.r32(s0 + 0xe8);
        let b = if a != 0 { m.r32(a + 0xdc) } else { 0 };
        sw = if b != 0 { m.r32(b + 0x10) } else { 0 };
    }
    if sw == 0 {
        let g = m.rv(sp);
        sw = m.call(0x001c_0d70, &[1], &[g]).0;
    }
    let g = m.rv(sp);
    let z = if m.call(0x001c_0940, &[sw, sp + 0x10, 1], &[g]).0 != 0 {
        m.rf(sp + 8)
    } else {
        let q = m.call(0x001a_62e8, &[s0], &[]).1;
        if m.call(0x001c_0940, &[sw, sp + 0x10, 1], &[q]).0 == 0 {
            return;
        }
        q[2]
    };
    m.wf(sp + 0x18, z);
    let p = m.rv(sp + 0x10);
    let d = sub(g, p);
    if (d[0] * d[0] + d[1] * d[1]).sqrt() > m.rf(0x002c_0e80) {
        return;
    }
    if m.call(0x001c_6de8, &[0, 0, sp + 0x10], &[p, f4(-250.0), f4(1500.0), f4(k)]).0 == 0 {
        return;
    }
    m.call(0x001a_e478, &[sp + 0x20], &[]);
    m.w32(sp + 0x30, n << 2);
    m.w32(sp + 0x34, 0x10);
    let buf = m.call(0x001a_e318, &[sp + 0x30], &[]).0;
    let cnt = m.call(0x001c_6ce8, &[world, kind, n, buf], &[]).0 as i32;
    if cnt != 0 {
        place_1c6fd8(m, sp, sw, buf, cnt);
    }
    m.call(0x001a_e4a0, &[sp + 0x20, 2], &[]);
}

fn place_1c6fd8<M: QMem + ?Sized>(m: &mut M, sp: u32, sw: u32, buf: u32, cnt: i32) {
    let r = m.rf(0x002c_0e84);
    let up = m.rv(0x002a_2de0);
    let p = m.rv(sp + 0x10);
    let hi = madd1(p, up, r);
    let lo = madd1(p, up, -r);
    m.wv(sp + 0x30, lo);
    m.wv(sp + 0x50, f4(r));
    m.wv(sp + 0x40, hi);
    m.call(0x0020_faa0, &[0x003d_d490, 0x001c_5570, 0, sp + 0x70, sp + 0x74], &[lo, hi]);
    let pi = std::f32::consts::PI;
    let mut ang = m.call(0x0011_d448, &[], &[f4(-pi), f4(pi)]).1[0];
    let mut rad = m.rf(0x002c_0e8c);
    if cnt <= 0 {
        return;
    }
    let (mut i, mut placed, mut ptr) = (0i32, 0i32, buf);
    loop {
        let t = i as f32 * 0.02;
        let h = m.rf(0x002c_0ea0) + t * (m.rf(0x002c_0ea4) + t * m.rf(0x002c_0ea8));
        let (lo, hi) = (m.rf(0x002c_0eb0), m.rf(0x002c_0eb4));
        let h = if h < lo { lo } else if hi < h { hi } else { h };
        let w = m.rv(sp + 0x10);
        m.wv(sp + 0x80, w);
        let off = m.call(0x0011_e1a0, &[], &[f4(ang), f4(rad), f4(0.0)]).1;
        let v5 = add(w, off);
        let g = m.rv(sp);
        let v3 = sub(v5, g);
        let upu = m.rv(0x002a_2dc0);
        let dd = dot3(v3, upu);
        let v2 = sub(v3, scale(upu, dd));
        m.wv(sp + 0x50, f4(h));
        let len = dot3(v2, v2).sqrt();
        let dir = if len < 1.0e-4 { m.rv(0x002a_2da0) } else { scale(v2, 1.0 / len) };
        let tgt = madd1(v5, dir, h);
        m.wv(sp + 0xb0, v5);
        m.wv(sp + 0x90, tgt);
        let clear = m.call(0x001c_0980, &[sw, 0], &[v5, tgt]).0 == 0 && m.call(0x001c_0980, &[sw, 0], &[tgt, v5]).0 == 0;
        if clear {
            let (a1, a2) = (m.r32(sp + 0x70), m.r32(sp + 0x74));
            if m.call(0x001c_6de8, &[a1, a2, sp + 0x50], &[v5, f4(-300.0), f4(300.0), f4(0.7)]).0 != 0 {
                let obj = m.r32(ptr);
                let lift = m.rf(obj + 0x1d8) - 1.0;
                let hit = madd1(m.rv(sp + 0x50), upu, lift);
                m.wv(sp + 0x60, f4(lift));
                m.wv(sp + 0xa0, hit);
                let g = m.rv(sp);
                let grav = m.rv(0x002c_0d60);
                let (c0, c1, c2) = (m.rf(0x002c_0eb8), m.rf(0x002c_0ebc), m.rf(0x002c_0ec0));
                let tt = m.call(0x0019_6928, &[], &[f4(c0), f4(c1), f4(c2), f4(c2), sub(hit, g), grav]).1[0];
                let tf = tt + placed as f32 * m.rf(0x002c_0ec4);
                placed += 1;
                let d = sub(hit, g);
                let inv = 1.0 / tf;
                let half = tf * -0.5;
                let vel = add(scale(d, inv), scale(grav, half));
                let f12 = placed as f32 * m.rf(0x002c_0d70);
                let f13 = m.rf(0x002a_7760);
                m.call(0x001c_6bd0, &[obj], &[f4(f12), f4(f13), g, vel]);
                ptr += 4;
            }
        }
        let da = m.rf(0x002c_0e94);
        let a = m.call(0x0011_cf60, &[], &[f4(ang + da)]).1[0];
        rad += m.rf(0x002c_0e90);
        let rmax = m.rf(0x002c_0e84);
        if rmax < rad {
            rad -= rmax - m.rf(0x002c_0e88);
        }
        ang = a;
        i += 1;
        if i >= 0x32 || placed >= cnt {
            break;
        }
    }
    let mut j = placed;
    while j < cnt {
        let obj = m.r32(buf + 4 * j as u32);
        let vt = m.r32(obj + 0xc);
        let off = m.r16(vt + 0x260) as i16 as i32;
        let f = m.r32(vt + 0x264);
        m.call(f, &[(obj as i32 + off) as u32, 2], &[]);
        j += 1;
    }
}

/// rows c0..c3 at `base` (stride 0x10): c0*v.x + c1*v.y + c2*v.z + c3 on xyz, w = `w`
fn xform<M: QMem + ?Sized>(m: &mut M, base: u32, v: V4, w: f32) -> V4 {
    let (c0, c1, c2, c3) = (m.rv(base), m.rv(base + 0x10), m.rv(base + 0x20), m.rv(base + 0x30));
    let f = |i: usize| c0[i] * v[0] + c1[i] * v[1] + c2[i] * v[2] + c3[i];
    [f(0), f(1), f(2), w]
}

/// FUN_001d3450(obj, prm, mesh, .., query, p7): ledge/step search for a vertical cylinder (radius prm+0x20,
/// height prm+0x28 - prm+0x24) at the object's frame: collect BSP faces (FUN_00118a88), clip their shared
/// edges against the frame plane (FUN_00119390) into frame-local hit points, add the cylinder silhouette
/// segment hits (FUN_00118668, 6 segments over the DAT_002c2150 ring), then per edge side pick the highest
/// supported lift and report it with FUN_001a7c18(0x3dd490, obj+0xe4, mesh, pos, plane, 10, 0, 0, p7).
pub fn ee_sub_1d3450<M: QMem + ?Sized>(m: &mut M, obj: u32, prm: u32, mesh: u32, query: u32, p7: u32) {
    let sp = stack_push(m, 0x1010);
    run_1d3450(m, sp, obj, prm, mesh, query, p7);
    stack_pop(m, 0x1010);
}

fn run_1d3450<M: QMem + ?Sized>(m: &mut M, sp: u32, obj: u32, prm: u32, mesh: u32, query: u32, p7: u32) {
    let vt = m.r32(obj + 0xc);
    let off = m.r16(vt + 0xf8) as i16 as i32;
    let f = m.r32(vt + 0xfc);
    let rot = m.call(f, &[(obj as i32 + off) as u32], &[]).0;
    let q = m.rv(prm + 0x50);
    let p = m.call(0x001a_6f00, &[obj, 0], &[q]).1;
    m.call(0x0025_6d18, &[sp + 0x40], &[p]);
    for i in 0..3 {
        let r = m.rv(rot + i * 0x10);
        m.wv(sp + 0x10 + i * 0x10, [r[0], r[1], r[2], 0.0]);
    }
    m.wf(sp + 0x4c, 1.0);
    m.call(0x0011_c318, &[sp + 0x90, sp + 0x10], &[]);
    let h = m.rf(prm + 0x28) - m.rf(prm + 0x24);
    let rr = m.rf(prm + 0x20);
    for i in 0..4 {
        let r = m.rv(sp + 0x90 + i * 0x10);
        m.wv(sp + 0x50 + i * 0x10, r);
    }
    let (r1, r3) = (m.rv(sp + 0x20), m.rv(sp + 0x40));
    m.call(0x0011_9390, &[sp + 0x90], &[r1, r3]);
    let plane = m.rv(sp + 0x90);
    m.wv(sp + 0xf80, plane);
    m.call(0x0023_33b4, &[sp + 0xd0, 0, 0x800], &[]);
    m.call(0x001a_e478, &[sp + 0x8d0], &[]);
    m.w32(sp + 0x90, 0xfffc);
    m.w32(sp + 0x94, 0x10);
    let buf = m.call(0x001a_e318, &[sp + 0x90], &[]).0;
    m.w32(sp + 0xee4, buf);
    m.w32(sp + 0xee0, 0x1555);
    let pos = m.rv(sp + 0x40);
    if m.call(0x0011_8a88, &[query, 0x7fff_ffff, sp + 0xee0, sp + 0xee4], &[f4((rr * rr + h * h).sqrt()), pos]).0 != 2 {
        m.call(0x001a_e4a0, &[sp + 0x8d0, 2], &[]);
        return;
    }
    let cur = m.r32(sp + 0xee4);
    let n = ((cur.wrapping_sub(buf) as i32).wrapping_mul(0xaaaa_aaabu32 as i32)) >> 2;
    let edges = m.r32(mesh + 0x9c);
    m.w32(sp + 0xee4, cur.wrapping_sub(0xc));
    let ent = |i: i32| buf + 8 + 12 * i as u32;
    for i in 0..n {
        let e = m.r32(ent(i));
        let fl = m.r32(e + 0x18);
        m.w32(e + 0x18, fl | 0x400);
    }
    let mut cnt: u32 = 0;
    let slot = |k: u32| sp + 0xd0 + k * 0x20;
    for i in 0..n {
        let face = m.r32(ent(i));
        let fl = m.r32(face + 0x18);
        if fl & 0x800 != 0 {
            continue;
        }
        m.w32(face + 0x18, fl | 0x800);
        let mut ed = m.r32(face + 0x10);
        while ed != 0 {
            let (e4, e6) = (m.r16(ed + 4) as u32, m.r16(ed + 6) as u32);
            let mut other = edges + e4 * 0x20;
            if other == face {
                other = edges + e6 * 0x20;
            }
            if m.r32(other + 0x18) & 0x400 != 0 {
                let vb = m.r32(mesh + 0x94);
                let (ia, ib) = (m.r16(ed) as u32, m.r16(ed + 2) as u32);
                let a = m.rv(vb + ia * 0x10);
                let b = m.rv(vb + ib * 0x10);
                m.wv(sp + 0xf90, a);
                m.wv(sp + 0xfa0, b);
                let da = dot3(plane, a) - plane[3];
                let db = dot3(plane, b) - plane[3];
                let t = da / (da - db);
                if t >= 0.0 && t <= 1.0 {
                    let x = add(scale(a, 1.0 - t), scale(b, t));
                    let w = m.rv(sp + 0x80)[3];
                    let l = xform(m, sp + 0x50, x, w);
                    m.wv(sp + 0xb0, f4(t));
                    m.wv(sp + 0x90, x);
                    m.wv(sp + 0xa0, l);
                    let ok = if 0.0 < l[2] { l[0].abs() <= rr } else { dot3(l, l) <= rr * rr };
                    if ok {
                        let s = slot(cnt);
                        cnt += 1;
                        if dot3(plane, sub(a, b)) < 0.0 {
                            m.w32(s + 0x14, other);
                            m.w32(s + 0x10, face);
                        } else {
                            m.w32(s + 0x10, other);
                            m.w32(s + 0x14, face);
                        }
                        m.wv(s, l);
                    }
                }
            }
            ed = m.r32(ed + 8);
        }
    }
    for i in 0..n {
        let e = m.r32(ent(i));
        let fl = m.r32(e + 0x18);
        m.w32(e + 0x18, fl & !0xc00);
    }
    // cylinder silhouette: 6 segments over the ring at DAT_002c2150 (scaled by R, last lifted by H)
    let up = m.rv(0x002a_2dc0);
    let ring = |i: u32| 0x002c_2150 + i * 0x10;
    let t0 = m.rv(ring(0));
    let mut prev = xform(m, sp + 0x10, add(scale(t0, rr), scale(up, h)), 0.0);
    let res = sp + 0x8e0;
    m.w32(sp + 0xfd4, res);
    for i in 0..6u32 {
        let v = if i < 5 {
            scale(m.rv(ring(i)), rr)
        } else {
            add(scale(m.rv(ring(4)), rr), scale(up, h))
        };
        let nw = xform(m, sp + 0x10, v, 0.0);
        let cur = m.r32(sp + 0xee4);
        let k = m.call(0x0011_8668, &[cur, 0x7fff_ffff, 0x10, res], &[prev, nw]).0 as i32;
        prev = nw;
        for j in 0..k.max(0) as u32 {
            let rec = res + j * 0x60;
            for (fo, so, vo) in [(0x4c, 0x14, 0), (0x50, 0x10, 0x10)] {
                let a = m.r32(rec + fo);
                if a != 0 {
                    let s = slot(cnt);
                    cnt += 1;
                    m.w32(s + so, a);
                    let v = m.rv(rec + vo);
                    let w = m.rv(sp + 0x50)[3];
                    let l = xform(m, sp + 0x50, v, w);
                    m.wv(s, l);
                }
            }
        }
    }
    let _ = prev;
    // per side: highest lift
    let mut best = 0.0f32;
    for pass in 0..2u32 {
        let fo = if pass == 0 { 0x14 } else { 0x10 };
        for i in 0..cnt {
            let s = slot(i);
            let (lo, hi);
            if pass == 0 {
                let a1 = m.r32(s + 0x14);
                if a1 == 0 {
                    continue;
                }
                lo = if m.r32(s + 0x10) != 0 { m.rf(s) } else { m.rf(0x002a_7838) };
                let mut h2 = m.rf(0x002a_783c);
                let mut pick = 0;
                for j in 0..cnt {
                    let e = slot(j);
                    if m.r32(e + 0x10) == a1 {
                        let x = m.rf(e);
                        if lo < x && x < h2 {
                            h2 = x;
                            pick = e;
                        }
                    }
                }
                if pick != 0 {
                    m.w32(pick + 0x18, 1);
                }
                hi = h2;
            } else {
                if m.r32(s + 0x10) == 0 || m.r32(s + 0x18) != 0 {
                    continue;
                }
                lo = m.rf(0x002a_7838);
                hi = if m.r32(s + 0x14) != 0 { m.rf(s) } else { m.rf(0x002a_783c) };
            }
            let fc = m.r32(s + fo);
            let nv = m.rv(fc);
            let r = m.call(0x0011_93d8, &[sp + 0x50], &[nv]).1;
            m.wv(sp + 0x90, r);
            if r[2] < 0.0 {
                continue;
            }
            let f2 = -r[0] / r[2];
            let r2 = rr * rr;
            let mut x = (r2 * f2 * f2 / (f2 * f2 + 1.0)).sqrt();
            if f2 < 0.0 {
                x = -x;
            }
            if x < lo {
                x = lo;
            } else if hi < x {
                x = hi;
            }
            let z = -(r2 - x * x).sqrt();
            m.wf(sp + 0xa0, x);
            m.wf(sp + 0xa4, 0.0);
            m.wf(sp + 0xa8, z);
            let dd = r[0] * x + r[2] * z - r[3];
            let lift = (-dd / r[2]).min(h);
            if best < lift {
                best = lift;
                let top = z + best;
                m.wf(sp + 0xfb0, x);
                m.wf(sp + 0xfb4, 0.0);
                m.wf(sp + 0xfb8, top);
                m.w32(sp + 0xfbc, if top.to_bits() & 0x8000_0000 != 0 { u32::MAX } else { 0 });
                let v = m.rv(sp + 0xfb0);
                m.wv(sp + 0xb0, v);
            }
        }
    }
    if 0.0 < best {
        let v = m.rv(sp + 0xfb0);
        let x = xform(m, sp + 0x10, v, v[3]);
        m.wv(sp + 0xfc0, x);
        m.wv(sp + 0xf60, plane);
        let e4 = m.r32(obj + 0xe4);
        m.call(0x001a_7c18, &[0x003d_d490, e4, mesh, 0xa, 0, 0, p7], &[f4(best), x, plane]);
    }
    m.call(0x001a_e4a0, &[sp + 0x8d0, 2], &[]);
}

/// virtual call through the vtable at obj+0xc: this-adjust (lh vt+off), fn (lw vt+off+4)
fn vcall<M: QMem + ?Sized>(m: &mut M, obj: u32, off: u32, a: &[u32], v: &[V4]) -> (u32, V4) {
    let vt = m.r32(obj.wrapping_add(0xc));
    let adj = m.r16(vt.wrapping_add(off)) as i16 as i32;
    let f = m.r32(vt.wrapping_add(off).wrapping_add(4));
    let mut args = vec![(obj as i32 + adj) as u32];
    args.extend_from_slice(a);
    m.call(f, &args, v)
}

/// heading of `d` (FUN_002458f0(d.y, d.x)) relative to the actor's facing (FUN_00204740), wrapped (FUN_0011cf60)
fn rel_heading<M: QMem + ?Sized>(m: &mut M, actor: u32, d: V4) -> f32 {
    let a = m.call(0x0024_58f0, &[], &[f4(d[1]), f4(d[0])]).1[0];
    let f = m.call(0x0020_4740, &[actor], &[]).1[0];
    m.call(0x0011_cf60, &[], &[f4(a - f)]).1[0]
}

/// FUN_0021f418(p): follow-target state machine (p[0] actor, p[1] state, p[4] target, p+0x58 anim,
/// p+0x12c listener). States 4..6 first update the look point p+0xe0 (blend p+0x100 -> p+0x110 over
/// p+0x120/+0x124, or orbit offset from p+0x1c/0x28), then FUN_0021fb70(p, 1); the switch runs until the
/// state stops changing (FUN_002206c8 sets it). Afterwards actors with +0x1a4 == 1 refresh +0x1e0 from
/// their anim (FUN_0012ff10 kind 6) or DAT_002a2dd0.
pub fn ee_sub_21f418<M: QMem + ?Sized>(m: &mut M, p: u32) {
    let sp = stack_push(m, 0x40);
    if run_21f418(m, sp, p) {
        post_21f418(m, sp, p);
    }
    stack_pop(m, 0x40);
}

fn run_21f418<M: QMem + ?Sized>(m: &mut M, sp: u32, p: u32) -> bool {
    let actor = m.r32(p);
    let s3 = m.r32(actor + 0x150);
    let pos = m.call(0x001a_62e8, &[actor], &[]).1;
    let st = m.r32(p + 4) as i32;
    if (4..7).contains(&st) {
        if m.r32(p + 0x128) != 0 {
            let (f3, f0, f4_) = (m.rf(p + 0x120), m.rf(p + 0x124), m.rf(0x002d_ded0));
            let f2 = f3 + f0;
            if f2 <= f4_ {
                m.w32(p + 0x128, 0);
                let v = m.rv(p + 0x110);
                m.wv(p + 0xe0, v);
            } else {
                let t = crate::engine::math::ps2f::ps2_div(f4_ - f3, f2 - f3);
                let (a, b) = (m.rv(p + 0x100), m.rv(p + 0x110));
                m.wv(p + 0xe0, add(scale(a, 1.0 - t), scale(b, t)));
            }
        } else {
            let (x, y) = (m.r32(p + 0x1c), m.r32(p + 0x20));
            m.w32(sp, x);
            m.w32(sp + 4, y);
            if m.call(0x001d_aea8, &[0x24], &[]).0 != 0 {
                let v = m.rf(sp + 4) * m.rf(0x002c_40e0);
                m.wf(sp + 4, v);
            }
            let mut d = sub(m.rv(p + 0xe0), pos);
            d[2] = 0.0;
            let f13 = m.rf(s3 + 0x1c) * m.rf(0x002d_ded4) * m.rf(p + 0x28);
            let f12 = m.rf(s3 + 0x18);
            let o = m.call(0x0011_e1a0, &[], &[f4(f12), f4(f13), f4(0.0)]).1;
            let v1 = add(d, o);
            m.wv(sp + 0x10, v1);
            m.call(0x0011_ef08, &[sp + 0x20, sp + 0x24, 0], &[v1]);
            let r0 = m.rf(sp + 0x24);
            let r = m.call(0x0011_d948, &[sp], &[f4(r0)]).1[0];
            m.wf(sp + 0x24, r);
            let a0 = m.rf(sp + 0x20);
            let o2 = m.call(0x0011_e1a0, &[], &[f4(a0), f4(r), f4(0.0)]).1;
            m.wv(sp + 0x10, o2);
            m.wv(p + 0xe0, add(pos, o2));
        }
        m.call(0x0021_fb70, &[p, 1], &[]);
    }
    let mut s2 = m.r32(p + 4);
    loop {
        let o = m.r32(p);
        let s0 = m.r32(o + 0x1030);
        let busy = |m: &mut M| s0 != 0 && m.call(0x0013_1968, &[s0], &[]).0 == 0;
        match s2 {
            0 => {
                let t = m.r32(p + 0x10);
                let d = sub(m.call(0x001a_62e8, &[t], &[]).1, pos);
                let a = rel_heading(m, actor, d);
                let lim = m.rf(p + 0x14);
                if lim < a {
                    m.call(0x0022_06c8, &[p, 2], &[]);
                } else if a < -lim {
                    m.call(0x0022_06c8, &[p, 1], &[]);
                } else {
                    let t = crate::engine::math::ps2f::ps2_div(a, lim) * 0.5 + 0.5;
                    m.wf(p + 0x4c, t);
                    m.wf(p + 0x40, 1.0 - t);
                    let an = m.r32(p + 0x58);
                    m.call(0x0013_84c0, &[an], &[]);
                }
            }
            1 | 2 => {
                if !busy(m) {
                    m.call(0x0022_06c8, &[p, 0], &[]);
                }
            }
            3 => {
                if !busy(m) {
                    let pl = m.r32(0x002d_d5bc);
                    let mode = if m.r32(pl + 0x13d0) == 8 { 4 } else { 0 };
                    m.call(0x0022_06c8, &[p, mode], &[]);
                }
            }
            4..=6 => {
                let t = m.r32(p + 0x10);
                if m.call(0x0013_2b10, &[t, 100], &[]).0 != 0 && m.r32(t + 0x158) as i32 != -1 {
                    let o = m.r32(p);
                        vcall(m, o, 0x2c0, &[0], &[]);
                } else if m.call(0x001c_46c0, &[s3, 0xc0], &[f4(0.1)]).0 != 0 {
                    m.call(0x0022_06c8, &[p, 7], &[]);
                } else if m.call(0x001c_46c0, &[s3, 0x30], &[f4(0.1)]).0 != 0 {
                    if m.r32(p + 0x58) != 0 {
                        m.call(0x0022_06c8, &[p, 0], &[]);
                    } else {
                        let o = m.r32(p);
                        vcall(m, o, 0x2c0, &[0], &[]);
                    }
                    let pl = m.r32(0x002d_d5bc);
                    let t = m.r32(p + 0x10);
                    if t == pl && m.r32(t + 0x13d0) == 8 {
                        m.wf(sp, 400.0);
                        let a = m.call(0x001a_62e8, &[t], &[]).1;
                        let o = m.r32(p);
                        let b = m.call(0x001a_62e8, &[o], &[]).1;
                        let d = sub(a, b);
                        let upu = m.rv(0x002a_2dc0);
                        let v3 = sub(d, scale(upu, dot3(d, upu)));
                        let len = dot3(v3, v3).sqrt();
                        let dir = if len < 1.0e-4 { m.rv(0x002a_2da0) } else { scale(v3, 1.0 / len) };
                        let v = add(scale(dir, 400.0), scale(upu, 600.0));
                        m.call(0x0013_6530, &[pl, 1], &[]);
                        vcall(m, pl, 0x118, &[], &[v]);
                    }
                    let l = m.r32(p + 0x12c);
                    if l != 0 {
                        vcall(m, l, 0x48, &[], &[]);
                    }
                    return false;
                } else if busy(m) {
                    if s2 == 4 {
                        let d = sub(m.rv(p + 0xe0), pos);
                        let a = rel_heading(m, actor, d);
                        let lim = m.rf(p + 0x18);
                        if lim < a {
                            m.call(0x0022_06c8, &[p, 6], &[]);
                        } else if a < -lim {
                            m.call(0x0022_06c8, &[p, 5], &[]);
                        }
                    }
                } else {
                    m.call(0x0022_06c8, &[p, 4], &[]);
                }
            }
            7 => {
                if !busy(m) {
                    if m.r32(p + 0x58) != 0 {
                        m.call(0x0022_06c8, &[p, 0], &[]);
                    } else {
                        let o = m.r32(p);
                        vcall(m, o, 0x2c0, &[0], &[]);
                    }
                }
            }
            _ => {}
        }
        let v1 = m.r32(p + 4);
        if v1 == s2 || v1 as i32 == -1 {
            return true;
        }
        s2 = v1;
    }
}

fn post_21f418<M: QMem + ?Sized>(m: &mut M, sp: u32, p: u32) {
    let a = m.r32(p);
    if m.r32(a + 0x1a4) != 1 {
        return;
    }
    let s0 = m.r32(a + 0x1030);
    let v = if s0 != 0 {
        let k = m.r32(s0 + 0x14);
        m.call(0x0012_ff10, &[k, 0, 6, 1], &[]).0
    } else {
        0
    };
    if v != 0 {
        let a3 = m.r32(v + 4);
        let vt = m.r32(a3);
        let adj = m.r16(vt + 0x18) as i16 as i32;
        let f = m.r32(vt + 0x1c);
        let (f12, f13) = (m.rf(s0 + 0x1c), m.rf(s0 + 0x40));
        m.call(f, &[(a3 as i32 + adj) as u32, 0, sp], &[f4(f12), f4(f13)]);
        let a = m.r32(p);
        let z = m.rf(sp + 8);
        m.w32(a + 0x1e0, 0);
        m.w32(a + 0x1e4, 0);
        m.wf(a + 0x1e8, z);
    } else {
        let v = m.rv(0x002a_2dd0);
        m.wv(a + 0x1e0, v);
    }
}

fn hdist(d: V4) -> f32 {
    (d[0] * d[0] + d[1] * d[1]).sqrt()
}
fn pdist(pl: V4, v: V4) -> f32 {
    dot3(pl, v) - pl[3]
}

/// FUN_0022a220(o): once per frame stamp (DAT_002de3e4), measure the extra climb needed over world
/// geometry between DAT_002ddfd0 (A) and the object (o+0x168 or o) position (B): build the vertical quad
/// A,B lifted by DAT_002c4490 (FUN_00119308 plane, 4 FUN_00119390 side planes), gather BSP faces in the box
/// (FUN_0020faa0 / FUN_00118d08), clip edges of 0x4000-flagged neighbours against the quad plane and keep the
/// steepest slope from A; o+0x17c = 1, o+0x180 = (max slope - base slope) * |AB|xy.
pub fn ee_sub_22a220<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let stamp = m.r32(0x002d_e3e4);
    if m.r32(o + 0x178) == stamp {
        return;
    }
    let sp = stack_push(m, 0x210);
    m.w32(o + 0x178, stamp);
    let a = m.rv(0x002d_dfd0);
    m.wv(sp + 0x180, a);
    let src = match m.r32(o + 0x168) {
        0 => o,
        x => x,
    };
    let b = m.call(0x001a_62e8, &[src], &[]).1;
    m.wv(sp + 0x190, b);
    m.w32(o + 0x17c, 0);
    m.w32(o + 0x180, 0);
    let h = m.rf(0x002c_4490);
    let up = m.rv(0x002a_2dc0);
    let (al, bl) = (madd1(a, up, h), madd1(b, up, h));
    let q = [b, bl, al, a];
    for (i, v) in q.iter().enumerate() {
        m.wv(sp + 0x10 * i as u32, *v);
    }
    m.call(0x0011_9308, &[sp + 0x80], &[a, b, bl]);
    let n = m.rv(sp + 0x80);
    m.wv(sp + 0x1a0, n);
    let mut last = [0.0; 4];
    for i in 0..4usize {
        let e = sub(q[(i + 1) % 4], q[i]);
        let c = [e[1] * n[2] - n[1] * e[2], e[2] * n[0] - n[2] * e[0], e[0] * n[1] - n[0] * e[1], e[3]];
        last = c;
        let c = scale(c, 1.0 / dot3(c, c).sqrt());
        m.call(0x0011_9390, &[sp + 0x40 + 0x10 * i as u32], &[c, q[i]]);
    }
    let fp = sp + 0x90;
    m.call(0x001a_e478, &[fp], &[]);
    m.w32(sp + 0x80, 0x10000);
    m.w32(sp + 0x84, 0x10);
    let buf = m.call(0x001a_e318, &[sp + 0x80], &[]).0;
    let hi = {
        let mx = [a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2]), last[3]];
        madd1(mx, up, h)
    };
    let lo = [a[0].min(b[0]), a[1].min(b[1]), a[2].min(b[2]), a[3]];
    m.call(0x0020_faa0, &[0x003d_d490, 0x0014_f030, 0, sp + 0xb0, sp + 0xb4], &[lo, hi]);
    let d = sub(b, a);
    let base = d[2] / hdist(d);
    let mut best = base;
    let mut s5 = 0;
    while (s5 as i32) < m.r32(sp + 0xb0) as i32 {
        let lst = m.r32(sp + 0xb4);
        let w = m.r32(lst + 4 * s5);
        s5 += 1;
        m.w32(sp + 0xb8, 0x4000);
        m.w32(sp + 0xbc, buf);
        let fb = m.r32(w + 0x74);
        let fe = m.r32(w + 0x70).wrapping_mul(12).wrapping_add(fb);
        let r = m.call(0x0011_8d08, &[fb, fe, 4, sp, sp + 0xb8, sp + 0xbc, 0xc], &[]).0 as i32;
        if r == 1 {
            continue;
        }
        if r == 0 {
            m.call(0x001a_e4a0, &[fp, 2], &[]);
            stack_pop(m, 0x210);
            return;
        }
        let nf = (m.r32(sp + 0xbc).wrapping_sub(buf) as i32) >> 2;
        for k in 0..nf.max(0) as u32 {
            let face = m.r32(buf + 4 * k);
            let mut ed = m.r32(face + 0x10);
            while ed != 0 {
                let e6 = m.r16(ed + 6) as u32;
                let eb = m.r32(w + 0x9c);
                let fl = m.r32(eb + e6 * 0x20 + 0x18);
                if fl & 0x4000 != 0 {
                    let vb = m.r32(w + 0x94);
                    let (ia, ib) = (m.r16(ed) as u32, m.r16(ed + 2) as u32);
                    let (p, qq) = (m.rv(vb + ia * 0x10), m.rv(vb + ib * 0x10));
                    let (dp, dq) = (pdist(n, p), pdist(n, qq));
                    let cross = !((0.0 < dp && 0.0 < dq) || (dp < 0.0 && dq < 0.0));
                    if cross {
                        let t = dp / (dp - dq);
                        let x = add(scale(p, 1.0 - t), scale(qq, t));
                        let mut inside = true;
                        for j in 0..4 {
                            let pl = m.rv(sp + 0x40 + 0x10 * j);
                            if !(pdist(pl, x) < 0.0) {
                                inside = false;
                                break;
                            }
                        }
                        if inside {
                            let dx = sub(x, a);
                            let s = dx[2] / hdist(dx);
                            if best < s {
                                best = s;
                            }
                        }
                    }
                }
                ed = m.r32(ed + 8);
            }
        }
        for k in 0..nf.max(0) as u32 {
            let face = m.r32(buf + 4 * k);
            let fl = m.r32(face + 0x18);
            m.w32(face + 0x18, fl & !0x4000);
        }
    }
    m.w32(o + 0x17c, 1);
    m.wf(o + 0x180, (best - base) * hdist(d));
    m.call(0x001a_e4a0, &[fp, 2], &[]);
    stack_pop(m, 0x210);
}

/// FUN_00220dd8(prm, src, out, track): camera focus point: out = lerp(src+0x10, src+0x40, prm+0x60);
/// with a track, re-express it in track space (FUN_00221408) and place it through the track offset
/// polynomial (prm+0x70/74/78 constant terms, clamps prm+0x40) at the track parameter (FUN_002215c0 mode 1),
/// with track+0x120 (polar flag) suppressed during the call.
pub fn ee_sub_220dd8<M: QMem + ?Sized>(m: &mut M, prm: u32, src: u32, out: u32, track: u32) {
    let k = m.rf(prm + 0x60);
    let (a, b) = (m.rv(src + 0x10), m.rv(src + 0x40));
    let v = add(scale(a, 1.0 - k), scale(b, k));
    m.wv(out, v);
    if track == 0 {
        return;
    }
    let sp = stack_push(m, 0x50);
    m.call(0x0023_33b4, &[sp + 0x10, 0, 0x30], &[]);
    let s1 = m.r32(track + 0x120);
    m.w32(track + 0x120, 0);
    for i in 0..3 {
        let c = m.rf(prm + 0x70 + 4 * i);
        m.wf(sp + 0x14 + 0x10 * i, c);
    }
    m.call(0x0022_1408, &[track, sp, sp + 0x40], &[v]);
    let u = m.rf(track + 0x140);
    m.call(0x0022_15c0, &[track, sp, sp + 0x10, prm + 0x40, 1, out], &[f4(u), f4(0.0)]);
    m.w32(track + 0x120, s1);
    stack_pop(m, 0x50);
}

/// FUN_00221818(o, key): select (or append, max 4) the 0x90-byte track-params entry for `key` in
/// o+0x114 (count o+0x110), current -> o+0x118. New entries copy the previous entry or the defaults at
/// DAT_002c4120; o+0x11c != 0 pins entry 0. Calls FUN_00221940(o, key) first.
pub fn ee_sub_221818<M: QMem + ?Sized>(m: &mut M, o: u32, key: u32) {
    m.call(0x0022_1940, &[o, key], &[]);
    let n = m.r32(o + 0x110) as i32;
    if n < 4 {
        let arr = m.r32(o + 0x114);
        for i in 0..n.max(0) as u32 {
            let e = arr + i * 0x90;
            if m.r32(e) == key {
                m.w32(o + 0x118, e); // found: no key store (asm b 0x22192c)
                return;
            }
        }
        if m.r32(o + 0x11c) != 0 {
            m.w32(o + 0x118, arr); // pinned: no key store
        } else {
            let e = arr + n as u32 * 0x90;
            m.w32(o + 0x110, (n + 1) as u32);
            m.w32(o + 0x118, e);
            let src = if n + 1 < 2 { 0x002c_4120 } else { e - 0x90 };
            for j in 0..9 {
                let v = m.rv(src + j * 0x10);
                m.wv(e + j * 0x10, v);
            }
            let cur = m.r32(o + 0x118);
            m.w32(cur, key);
        }
    }
}

/// FUN_00221a18(o, tgt, track): CplcyFollow activate: FUN_0014fcc8 base, pick the track-params entry
/// (track+0x114, best of FUN_00132b10-matching entries by FUN_0015e988, else DAT_002c4120 defaults) into
/// o+0x14, compute the focus (FUN_0014e968 / FUN_00220dd8, snapped onto the path FUN_001e6f60 or circle
/// FUN_00221ce8), then the polar camera offset o+0x18..0x20 from DAT_002ddfd0 (FUN_0011efb0) and o+0x70.
pub fn ee_sub_221a18<M: QMem + ?Sized>(m: &mut M, o: u32, tgt: u32, track: u32) {
    let sp = stack_push(m, 0x90);
    m.call(0x0014_fcc8, &[o, tgt, track], &[]);
    m.w32(o + 0x10, track);
    if track == 0 {
        m.w32(o + 0x14, 0x002c_4120);
    } else {
        let mut best = m.r32(track + 0x114);
        let mut i = 1u32;
        loop {
            let t = m.r32(o + 0x10);
            if i as i32 >= m.r32(t + 0x110) as i32 {
                break;
            }
            let e = m.r32(t + 0x114) + i * 0x90;
            let k = m.r32(e);
            if m.call(0x0013_2b10, &[tgt, k], &[]).0 != 0 {
                let kb = m.r32(best);
                if m.call(0x0015_e988, &[k, kb], &[]).0 != 0 {
                    best = e;
                }
            }
            i += 1;
        }
        m.w32(o + 0x14, best);
    }
    m.w32(o + 0x38, 1);
    let prm = m.r32(o + 0x14);
    m.call(0x0022_0d40, &[prm], &[]);
    m.call(0x0014_e968, &[0x002d_df40, tgt, sp], &[]);
    m.call(0x0022_0dd8, &[prm, sp, sp + 0x50, 0], &[]);
    let t = m.r32(o + 0x10);
    if t != 0 {
        let path = m.r32(t + 0x10c);
        if path != 0 {
            let f = m.rv(sp + 0x50);
            m.call(0x001e_6f60, &[path, 0, sp + 0x60, 0, 0, t + 0x140], &[f]);
        } else if m.r32(t + 0x12c) != 0 {
            m.call(0x0022_1ce8, &[o, sp], &[]);
        }
        let prm = m.r32(o + 0x14);
        m.call(0x0022_0dd8, &[prm, sp, sp + 0x50, t], &[]);
    }
    let foc = add(m.rv(sp + 0x50), m.rv(0x002d_e220));
    m.wv(sp + 0x70, foc);
    let d = sub(m.rv(0x002d_dfd0), foc);
    m.call(0x0011_efb0, &[o + 0x18, o + 0x1c, o + 0x20], &[d]);
    let r = m.rf(o + 0x20) - m.rf(0x002d_df8c);
    m.wf(o + 0x20, r);
    let (a, b) = (m.rf(o + 0x18), m.rf(o + 0x1c));
    let v = m.call(0x0011_e208, &[], &[f4(a), f4(b), f4(r)]).1;
    m.wv(o + 0x70, add(foc, v));
    let t = m.r32(o + 0x10);
    let prm = m.r32(o + 0x14);
    let f = m.rf(prm + 0x5c);
    m.w32(o + 0x34, 0);
    m.wf(o + 0x24, f);
    if t != 0 && m.r32(t + 0x184) != 0 {
        m.call(0x0014_d8c0, &[0x002d_df40], &[]);
    }
    let g = m.rf(0x002d_ded0);
    let f2 = m.rf(0x002c_41dc);
    m.wf(o + 0x80, g);
    m.wf(o + 0x60, g);
    m.w32(o + 0x3c, 0);
    m.wf(o + 0x48, g - f2);
    let t = m.r32(o + 0x10);
    if t != 0 {
        let f1 = m.rf(t + 0x188);
        if f1 != 0.0 {
            m.wf(0x002d_df64, f1);
        }
    }
    stack_pop(m, 0x90);
}

/// FUN_00220388(p, out): out+0x10 += DAT_002ddfb0 * DAT_002c4108 (w+1); in states 4..6 out+0x20 = the
/// horizontal direction (z = 0) from out+0x10 to the look point p+0xe0 (minus the same offset), normalized
/// (DAT_002a2dd0 when < 1e-4) and scaled by DAT_002c410c.
pub fn ee_sub_220388<M: QMem + ?Sized>(m: &mut M, p: u32, out: u32) {
    let k = m.rf(0x002c_4108);
    let g = m.rv(0x002d_dfb0);
    let v5 = madd1(m.rv(out + 0x10), g, k);
    m.wv(out + 0x10, v5);
    let st = m.r32(p + 4) as i32;
    if !(4..7).contains(&st) {
        return;
    }
    let e = m.rv(p + 0xe0);
    let mut d = [
        e[0] - v5[0] - g[0] * k,
        e[1] - v5[1] - g[1] * k,
        0.0,
        e[3] + 1.0 - v5[3] - g[3] * k,
    ];
    d[2] = 0.0;
    let s = m.rf(0x002c_410c);
    let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let v = if len < 1.0e-4 { m.rv(0x002a_2dd0) } else { scale(d, 1.0 / len) };
    m.wv(out + 0x20, scale(v, s));
}

/// FUN_0020f710(o): look up the 4 screen objects (oids DAT_002c3e70[0..4], FUN_0015e938 kind 0x405) into
/// o+0x3ba0 (stride 0xc), each set up with FUN_00213c68(obj, {1,0,0,1, 0..}); o+0x4764 = FUN_001c7d08(),
/// o+0x4768 = FUN_001c7d00().
pub fn ee_sub_20f710<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x20);
    m.call(0x0023_33b4, &[sp, 0, 0x20], &[]);
    m.wf(sp, 1.0);
    m.wf(sp + 0xc, 1.0);
    for i in 0..4u32 {
        let oid = m.r32(0x002c_3e70 + 4 * i);
        let s0 = m.call(0x0015_e938, &[0x003d_d490, 0x405, oid, 0], &[]).0;
        if s0 != 0 {
            m.call(0x0021_3c68, &[s0, sp], &[]);
        }
        m.w32(o + 0x3ba0 + 0xc * i, s0);
    }
    let a = m.call(0x001c_7d08, &[], &[]).0;
    m.w32(o + 0x4764, a);
    let b = m.call(0x001c_7d00, &[], &[]).0;
    m.w32(o + 0x4768, b);
    stack_pop(m, 0x20);
}

/// FUN_00209c88(o): guard path follow: FUN_001f4fa8 base, then by mode o+0xf80 aim the guard (vtable
/// +0x290) along its path o+0xf84: 0 -> path point/tangent at o+0xf88 (FUN_001e6f10), kind 0x22 with
/// look target *(o+0x218); 1 -> path point/tangent + position of o+0xf90, kind 7; 2 -> pos/xf row of
/// o+0xf84 + pos of o+0xf90, kind 7; 3 -> pos of o+0xf84, DAT_002a2da0, *(o+0x218), kind 7.
pub fn ee_sub_209c88<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x80);
    m.call(0x001f_4fa8, &[o], &[]);
    let path = m.r32(o + 0xf84);
    match m.r32(o + 0xf80) {
        0 => {
            let u = m.rf(o + 0xf88);
            m.call(0x001e_6f10, &[path, sp, sp + 0x10], &[f4(u)]);
            let (a, b) = (m.rv(sp), m.rv(sp + 0x10));
            let c = { let t_ = m.r32(o + 0x218); m.rv(t_) };
            vcall(m, o, 0x290, &[0x22, path], &[a, b, c]);
        }
        1 => {
            let u = m.rf(o + 0xf88);
            m.call(0x001e_6f10, &[path, sp, sp + 0x10], &[f4(u)]);
            let (a, b) = (m.rv(sp), m.rv(sp + 0x10));
            let t_ = m.r32(o + 0xf90);
            let c = m.call(0x001a_62e8, &[t_], &[]).1;
            vcall(m, o, 0x290, &[7, path], &[a, b, c]);
        }
        2 => {
            let a = m.call(0x001a_62e8, &[path], &[]).1;
            let x = m.call(0x001a_6318, &[path], &[]).0;
            let b = m.rv(x);
            let t_ = m.r32(o + 0xf90);
            let c = m.call(0x001a_62e8, &[t_], &[]).1;
            vcall(m, o, 0x290, &[7, path], &[a, b, c]);
        }
        3 => {
            let a = m.call(0x001a_62e8, &[path], &[]).1;
            let b = m.rv(0x002a_2da0);
            let c = { let t_ = m.r32(o + 0x218); m.rv(t_) };
            vcall(m, o, 0x290, &[7, path], &[a, b, c]);
        }
        _ => {}
    }
    stack_pop(m, 0x80);
}

/// FUN_0020b868(o): build a run blend for block o = {+0 skel, +4 blend out, +0x10 entries[2] stride
/// 0x40 (+4 aseg)}: FUN_00137fb8 label lookups (0x2c3dc0 -> +0x20, 0x2c3dc8 -> +0x30); per entry the
/// root velocity +0x10 = (pose(t = aseg+0x74) - pose(0)) / aseg+0x74 via the channel evaluator
/// (FUN_0012ff10, vtable +0x18); FUN_00137a30 blend; then per blend channel (+0x60 stride 8, count
/// +0x5c) the bound key (FUN_0012fe58) kind 0x10/0x12/0x14 links +8 to blend+0xd0+0xc, 0x11/0x13/0x15
/// keep it, other kinds clear it; FUN_001384c0(blend).
pub fn ee_sub_20b868<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0x14) == 0 {
        return;
    }
    let sp = stack_push(m, 0xd0);
    let e = o + 0x10;
    m.call(0x0013_7fb8, &[0x40, 2, e, 0x20, 2, 0x002c_3dc0], &[]);
    m.call(0x0013_7fb8, &[0x40, 2, e, 0x30, 2, 0x002c_3dc8], &[]);
    for i in 0..2u32 {
        let s2 = e + 0x40 * i;
        let s1 = m.r32(s2 + 4);
        if s1 == 0 {
            continue;
        }
        let sk = m.r32(o);
        let s0 = m.call(0x0012_ff10, &[s1, sk, 1, 1], &[]).0;
        let a3 = m.r32(s0 + 4);
        let vt = m.r32(a3);
        let (adj, f) = (m.r16(vt + 0x18) as i16 as i32, m.r32(vt + 0x1c));
        m.call(f, &[(a3 as i32 + adj) as u32, sp, 0], &[f4(0.0), f4(1.0)]);
        let t = m.rf(s1 + 0x74);
        let a3 = m.r32(s0 + 4);
        let vt = m.r32(a3);
        let (adj, f) = (m.r16(vt + 0x18) as i16 as i32, m.r32(vt + 0x1c));
        m.call(f, &[(a3 as i32 + adj) as u32, sp + 0x10, 0], &[f4(t), f4(1.0)]);
        let d = sub(m.rv(sp + 0x10), m.rv(sp));
        m.wv(s2 + 0x10, scale(d, 1.0 / t));
    }
    let sk = m.r32(o);
    m.call(0x0013_7a30, &[sk, 0x40, 2, e, 2, 0x002c_3dd8, 0x002c_3dd0, o + 4], &[]);
    let mut i = 0u32;
    loop {
        let b = m.r32(o + 4);
        if i as i32 >= m.r32(b + 0x5c) as i32 {
            break;
        }
        let a2 = m.r32(b + 0x60) + i * 8;
        let (k, x) = (m.r32(a2) as i32, m.r32(a2 + 4));
        let (s1, s0) = if k >= 0 { (x, 0) } else { (0, x) };
        let sk = m.r32(o);
        m.call(0x0012_fe58, &[b, sk, a2, sp + 0x30, sp + 0x34], &[]);
        let kind = { let t_ = m.r32(sp + 0x34); m.r32(t_ + 0x9c) };
        match kind {
            0x10 | 0x12 | 0x14 => {
                let v = { let t_ = m.r32(o + 4); m.r32(t_ + 0xd0) }.wrapping_add(0xc);
                if s1 != 0 {
                    m.w32(s1 + 8, v);
                }
                if s0 != 0 {
                    m.w32(s0 + 8, v);
                }
            }
            0x11 | 0x13 | 0x15 => {}
            _ => {
                if s1 != 0 {
                    m.w32(s1 + 8, 0);
                }
                if s0 != 0 {
                    m.w32(s0 + 8, 0);
                }
            }
        }
        i += 1;
    }
    let b = m.r32(o + 4);
    m.call(0x0013_84c0, &[b], &[]);
    stack_pop(m, 0xd0);
}

/// FUN_00210140(l, o): append o to the 8-byte {link, obj} list at l+0x3f20 (count +0x3f14, dead +0x3f18,
/// cursor index ptr +0x3f1c) unless o+0x40 has bit 0x40. When count >= 0x1000 or dead/count > 0.1 the
/// list is compacted first (dropping null objects, re-pointing obj+0x40 to the moved entry and fixing the
/// cursor). The new entry chains the previous o+0x40 and o+0x40 = entry.
pub fn ee_sub_210140<M: QMem + ?Sized>(m: &mut M, l: u32, o: u32) {
    if m.r32(o + 0x40) & 0x40 != 0 {
        return;
    }
    let n = m.r32(l + 0x3f14) as i32;
    let compact = n >= 0x1000 || {
        let r = m.r32(l + 0x3f18) as i32 as f32 / n as f32;
        f32::from_bits(0x3dcc_cccd) < r
    };
    if compact {
        let arr = m.r32(l + 0x3f20);
        let end = arr.wrapping_add((n as u32) << 3);
        let cur = m.r32(l + 0x3f1c);
        let mut t0 = if cur != 0 { arr.wrapping_add(m.r32(cur) << 3) } else { 0 };
        let (mut dst, mut src) = (arr, arr);
        while src < end {
            if src == t0 {
                t0 = dst;
            }
            let ob = m.r32(src + 4);
            if ob != 0 {
                m.w32(ob + 0x40, dst);
                let (a, b) = (m.r32(src), m.r32(src + 4));
                m.w32(dst, a);
                m.w32(dst + 4, b);
                dst += 8;
            }
            src += 8;
        }
        m.w32(l + 0x3f18, 0);
        m.w32(l + 0x3f14, ((dst.wrapping_sub(arr) as i32) >> 3) as u32);
        if cur != 0 {
            m.w32(cur, ((t0.wrapping_sub(arr) as i32) >> 3) as u32);
        }
    }
    let n = m.r32(l + 0x3f14);
    let e = m.r32(l + 0x3f20).wrapping_add(n << 3);
    m.w32(l + 0x3f14, n + 1);
    m.w32(e + 4, o);
    let prev = m.r32(o + 0x40);
    m.w32(e, prev);
    m.w32(o + 0x40, e);
}

/// M.xyz * p + M.row3 (vmulax/vmadday/vmaddaz/vmaddw.xyz), w lane kept from `w`
fn xf_pt<M: QMem + ?Sized>(m: &mut M, mat: u32, p: V4, w: f32) -> V4 {
    let r = m.rm(mat);
    let mut o = [0.0; 4];
    for k in 0..3 {
        o[k] = r[0][k] * p[0] + r[1][k] * p[1] + r[2][k] * p[2] + r[3][k];
    }
    o[3] = w;
    o
}

/// FUN_0022b7d8(o): UpdateWrStateVectors. For every WR glob (o+0x8f0, next +0xa0; kind +0x30, range
/// +0x34..+0x40, xf +0x50, owner +0, model +4) and every point set (+0x90, next +0x18; count +0, points
/// +4, outputs +8[]) compute per point a clamped quadratic weight of its distance / axis coordinate in
/// glob space (kind -1: 1; 0: |p|; 1..3: |p| with z/y/x dropped; 4..6: p.x/y/z), then for each WR state
/// (o+0x60 stride 0xd0, count o+0x54) write either a rotation (kind 0: sin/cos of the angle along the
/// normalized axis +0x10 * +0x64 * 2pi into slots +0x60) or a translation (kind 1: point - glob anchor),
/// both scaled by the weight. o+0x8fc = 0.
pub fn ee_sub_22b7d8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x15c0);
    let mut g = m.r32(o + 0x8f0);
    while g != 0 {
        let kind = m.r32(g + 0x30) as i32;
        let axis;
        if kind == -1 {
            m.call(0x0023_33b4, &[sp, 0, 0x40], &[]);
            for k in 0..4 {
                m.wf(sp + 0x14 * k, 1.0);
            }
            axis = m.rv(0x002a_2dd0);
        } else {
            for k in 0..4 {
                let v = m.rv(g + 0x50 + 0x10 * k);
                m.wv(sp + 0x10 * k, v);
            }
            let (s34, s38, s3c, s40) = (m.rf(g + 0x34), m.rf(g + 0x38), m.rf(g + 0x3c), m.rf(g + 0x40));
            m.wf(sp + 0x50, s40.min(s38));
            m.wf(sp + 0x54, s40.max(s38));
            let k1 = (s38 - s40) / (s34 - s3c);
            m.wf(sp + 0x44, k1);
            m.w32(sp + 0x48, 0);
            m.wf(sp + 0x40, s38 - k1 * s34);
            let ax = match kind {
                4 => scale(m.rv(0x002a_2da0), (s34 + s3c) * 0.5),
                5 => scale(m.rv(0x002a_2db0), (s34 + s3c) * 0.5),
                6 => scale(m.rv(0x002a_2dc0), (s34 + s3c) * 0.5),
                _ => m.rv(0x002a_2dd0),
            };
            m.call(0x0011_c318, &[sp + 0xb0, sp], &[]);
            axis = xf_pt(m, sp + 0xb0, ax, ax[3]);
            let a2 = { let t_ = m.r32(g + 4); m.r32(t_ + 8) };
            if a2 != 0 {
                m.call(0x0011_c168, &[sp + 0xb0, g + 0x50, a2], &[]);
                for k in 0..4 {
                    let v = m.rv(sp + 0xb0 + 0x10 * k);
                    m.wv(sp + 0x10 * k, v);
                }
            }
        }
        m.wv(sp + 0x1500, axis);
        let mut s6 = m.r32(g + 0x90);
        while s6 != 0 {
            let n = m.r32(s6);
            let src = m.r32(s6 + 4);
            m.call(0x001a_e5a0, &[sp + 0xf0, src, n << 4], &[]);
            let n = m.r32(s6);
            for i in 0..n {
                let p = m.rv(sp + 0xf0 + 0x10 * i);
                let r0w = m.rf(sp + 0xc);
                let mut q = xf_pt(m, sp, p, r0w);
                let wslot = sp + 0x10f0 + 4 * i;
                let t = match kind {
                    -1 => {
                        m.wf(wslot, 1.0);
                        continue;
                    }
                    0 => Some(dot3(q, q).sqrt()),
                    1 => {
                        q[2] = 0.0;
                        Some(dot3(q, q).sqrt())
                    }
                    2 => {
                        q[1] = 0.0;
                        Some(dot3(q, q).sqrt())
                    }
                    3 => {
                        q[0] = 0.0;
                        Some(dot3(q, q).sqrt())
                    }
                    4 => Some(q[0]),
                    5 => Some(q[1]),
                    6 => Some(q[2]),
                    _ => None,
                };
                if let Some(t) = t {
                    let (c0, c1, c2) = (m.rf(sp + 0x40), m.rf(sp + 0x44), m.rf(sp + 0x48));
                    let (lo, hi) = (m.rf(sp + 0x50), m.rf(sp + 0x54));
                    let mut f = c0 + t * (c1 + t * c2);
                    if f < lo {
                        f = lo;
                    } else if hi < f {
                        f = hi;
                    }
                    m.wf(wslot, f);
                }
            }
            let mdl = { let t_ = m.r32(g + 4); m.r32(t_ + 4) };
            for k in 0..4 {
                let v = m.rv(mdl + 0x40 + 0x10 * k);
                m.wv(sp + 0x60 + 0x10 * k, v);
            }
            let a2 = { let t_ = m.r32(g + 4); m.r32(t_ + 8) };
            if a2 != 0 {
                m.call(0x0011_c168, &[sp + 0xa0, sp + 0x60, a2], &[]);
                for k in 0..4 {
                    let v = m.rv(sp + 0xa0 + 0x10 * k);
                    m.wv(sp + 0x60 + 0x10 * k, v);
                }
            }
            let ax = m.rv(sp + 0x1500);
            let ow = m.r32(g);
            let anchor = m.call(0x001a_6f00, &[ow, 0], &[ax]).1;
            m.wv(sp + 0x1510, anchor);
            let mut i = 0u32;
            while (i as i32) < m.r32(o + 0x54) as i32 {
                let e = o + 0x60 + i * 0xd0;
                let ei = m.r32(e + 4);
                let mut s1 = m.r32(s6 + 8 + 4 * ei);
                for j in 0..n {
                    let p = m.rv(sp + 0xf0 + 0x10 * j);
                    let r3w = m.rf(sp + 0x9c);
                    let q = xf_pt(m, sp + 0x60, p, r3w);
                    m.wv(sp + 0xa0, q);
                    let w = m.rf(sp + 0x10f0 + 4 * j);
                    match m.r32(e) {
                        0 => {
                            let a = m.rv(e + 0x10);
                            let nn = scale(a, 1.0 / dot3(a, a).sqrt());
                            let s0 = m.r32(e + 0x60);
                            let k = m.rf(e + 0x64) * f32::from_bits(0x40c9_0fdb);
                            let ang = m.call(0x0011_cf60, &[], &[f4(dot3(q, nn) * k)]).1[0];
                            m.call(0x0011_d608, &[sp + 0x14f0, sp + 0x14f4], &[f4(ang)]);
                            let (sn, cs) = (m.rf(sp + 0x14f0), m.rf(sp + 0x14f4));
                            m.wf(s1 + 4 * s0, cs * w);
                            m.wf(s1 + 4 * (s0 + 1), sn * w);
                        }
                        1 => {
                            let d = sub(q, m.rv(sp + 0x1510));
                            m.wv(sp + 0xb0, d);
                            m.wv(s1, [d[0] * w, d[1] * w, d[2] * w, 1.0]);
                        }
                        _ => {}
                    }
                    s1 += 0x10;
                }
                i += 1;
            }
            s6 = m.r32(s6 + 0x18);
        }
        g = m.r32(g + 0xa0);
    }
    m.w32(o + 0x8fc, 0);
    stack_pop(m, 0x15c0);
}

/// VU0 micro 0x60 (as in FUN_0011c168): row i of the result = sum_k a[k] * b[i][k]
fn vu_mul60(a: [V4; 4], b: [V4; 4]) -> [V4; 4] {
    let mut o = [[0.0; 4]; 4];
    for i in 0..4 {
        for k in 0..4 {
            o[i] = madd1(o[i], a[k], b[i][k]);
        }
    }
    o
}

/// FUN_0022faa0(o): compose world matrices of the bone array o+0x54 (count o+0x50, stride 0x100):
/// world(+0x40) = micro0x60(parent world, local +0), parent +0xa0 (identity DAT_002a2d30 when null; the
/// just-composed bone when the next bone's +0x1a0 is this bone); +0xa4 = max(+0xa4, parent+0xa4);
/// then FUN_0022fa20(o).
pub fn ee_sub_22faa0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let n = m.r32(o + 0x50) as i32;
    if n > 0 {
        let first = m.r32(o + 0x54);
        let last = first.wrapping_add((n as u32) << 8).wrapping_sub(0x100);
        let p = m.r32(first + 0xa0);
        let mut par = m.rm(if p != 0 { p.wrapping_add(0x40) } else { 0x002a_2d30 });
        let mut b = first;
        while b <= last {
            let w = vu_mul60(par, m.rm(b));
            let p = m.r32(b + 0xa0);
            if p != 0 {
                let pa = m.r32(p.wrapping_add(0xa4)) as i32;
                if (m.r32(b + 0xa4) as i32) < pa {
                    m.w32(b + 0xa4, pa as u32);
                }
            }
            m.wm(b + 0x40, w);
            if b < last {
                let np = m.r32(b + 0x1a0);
                par = if np == b { w } else { m.rm(np.wrapping_add(0x40)) };
            }
            b += 0x100;
        }
    }
    m.call(0x0022_fa20, &[o], &[]);
}

/// FUN_001f21b8(o, par, bone, fl): SetParent. fl&3: capture the current world matrix (o+0x60 world when
/// o+0x28 bit 60, else the chain product of locals up o+0x60 -> +0xa0 via FUN_0011c168); fl&4 / fl&8
/// capture vtable +0x108 / +0x110 vectors. Root bones whose parent lies in the old parent's bone array
/// (o+0x10: +0x54, count +0x50) are re-pointed to `bone` (or par+0x60); FUN_001a5900 relinks, FUN_0022faa0
/// recomposes, then the captured state is restored through vtable +0xe8 (position) / +0xf0 (matrix) /
/// +0x118 / +0x120; FUN_001f2468, FUN_001f26a8, and FUN_001f8668(o, 0) unless the parent has bit 35 of +0x148.
pub fn ee_sub_1f21b8<M: QMem + ?Sized>(m: &mut M, o: u32, par: u32, bone: u32, fl: u32) {
    let sp = stack_push(m, 0x130);
    if fl & 3 != 0 {
        if (m.r64(o + 0x28) >> 60) & 1 != 0 {
            let w = m.r32(o + 0x60);
            let v = m.rm(w + 0x40);
            m.wm(sp, v);
        } else {
            m.call(0x0023_33b4, &[sp, 0, 0x40], &[]);
            for k in 0..4 {
                m.wf(sp + 0x14 * k, 1.0);
            }
            let mut s0 = m.r32(o + 0x60);
            while s0 != 0 {
                m.call(0x0011_c168, &[sp + 0x40, s0, sp], &[]);
                let v = m.rm(sp + 0x40);
                m.wm(sp, v);
                s0 = m.r32(s0 + 0xa0);
            }
        }
    }
    let fp = if fl & 4 != 0 { vcall(m, o, 0x108, &[], &[]).1 } else { [0.0; 4] };
    if fl & 8 != 0 {
        let v = vcall(m, o, 0x110, &[], &[]).1;
        m.wv(sp + 0x80, v);
    }
    let t = m.r32(o + 0x10);
    // BOSS R24: the image's top-level objects carry +0x10 = the world (0x3dd490) where the original has 0
    // (rec11: dispensers / barrel copies +0x10 = 0): no owner bones then
    let (cnt, base) = if t != 0 && t != super::live::WORLD { (m.r32(t + 0x50) as i32, m.r32(t + 0x54)) } else { (1, 0) };
    let np = if bone != 0 {
        bone
    } else if par != 0 {
        m.r32(par + 0x60)
    } else {
        0
    };
    let mut i = 0u32;
    while (i as i32) < m.r32(o + 0x50) as i32 {
        let b = m.r32(o + 0x54) + (i << 8);
        let idx = (m.r32(b + 0xa0).wrapping_sub(base) as i32) >> 8;
        if idx >= 0 && idx < cnt {
            m.w32(b + 0xa0, np);
        }
        i += 1;
    }
    m.call(0x001a_5900, &[o, par, bone, fl], &[]);
    m.call(0x0022_faa0, &[o], &[]);
    if fl & 1 != 0 {
        let p = m.rv(sp + 0x30);
        vcall(m, o, 0xe8, &[], &[p]);
    }
    if fl & 2 != 0 {
        vcall(m, o, 0xf0, &[sp], &[]);
    }
    if fl & 4 != 0 {
        vcall(m, o, 0x118, &[], &[fp]);
    }
    if fl & 8 != 0 {
        let v = m.rv(sp + 0x80);
        vcall(m, o, 0x120, &[], &[v]);
    }
    m.call(0x001f_2468, &[o], &[]);
    m.call(0x001f_26a8, &[o], &[]);
    let t = m.r32(o + 0x10);
    if t != 0 && m.r64(t + 0x148) & (0x8000u64 << 20) == 0 {
        m.call(0x001f_8668, &[o, 0], &[]);
    }
    stack_pop(m, 0x130);
}

/// 3-lane approximate equality via FUN_0011d4d0(a, b, 1e-4) on x, then y (prot3w), then z (pextuw)
fn vec_eq3<M: QMem + ?Sized>(m: &mut M, a: V4, b: V4) -> bool {
    let e = f32::from_bits(0x38d1_b717);
    (0..3).all(|k| m.call(0x0011_d4d0, &[], &[f4(a[k]), f4(b[k]), f4(e)]).0 != 0)
}

/// shared body of FUN_0012c140 (stride 12 keys, index FUN_0012c0e0) / FUN_0012c768 (stride 8, FUN_0012c708)
fn axis_seg<M: QMem + ?Sized>(m: &mut M, c: u32, k: u32, idx_in: u32, stride: u32, find: u32, hi_off: u32) {
    let idx = if m.r32(c + 4) != 0 {
        let tp = m.r32(k + 0x88);
        let t = m.rf(tp);
        let i = m.call(find, &[c], &[f4(t)]).0;
        let tt = m.r32(c + 8);
        let a = m.rf(tt + 4 * i - 4);
        m.wf(k + 0x80, a);
        let tt = m.r32(c + 8);
        let b = m.rf(tt + 4 * i);
        m.wf(k + 0x84, b);
        i
    } else {
        m.w32(k + 0x88, k + 0x80);
        m.wf(k + 0x84, 1.0);
        m.w32(k + 0x8c, k + 0x84);
        m.w32(k + 0x80, 0);
        idx_in
    };
    let t2 = idx.wrapping_mul(stride);
    for j in 0..3u32 {
        let arr = m.r32(c + 0xc + 4 * j);
        if arr != 0 {
            let v = arr.wrapping_add(t2).wrapping_sub(stride);
            let (a, b) = (m.rf(v), m.rf(v + hi_off));
            m.wf(k + 0xc0 + 4 * j, a);
            m.wf(k + 0xd0 + 4 * j, b);
            let v = arr.wrapping_add(t2);
            let (a, b) = (m.rf(v), m.rf(v + 4));
            m.wf(k + 0xe0 + 4 * j, a);
            m.wf(k + 0xf0 + 4 * j, b);
        } else {
            let d = m.rf(c + 0x20 + 4 * j);
            m.wf(k + 0xc0 + 4 * j, d);
            m.w32(k + 0xd0 + 4 * j, 0);
            let d = m.rf(c + 0x20 + 4 * j);
            m.wf(k + 0xe0 + 4 * j, d);
            m.w32(k + 0xf0 + 4 * j, 0);
        }
    }
    let (p0, p1) = (m.rv(k + 0xc0), m.rv(k + 0xe0));
    let z = m.rv(0x002a_2dd0);
    let same = vec_eq3(m, p0, p1) && {
        let t0 = m.rv(k + 0xd0);
        vec_eq3(m, z, t0)
    } && {
        let t1 = m.rv(k + 0xf0);
        vec_eq3(m, z, t1)
    };
    m.w16(k + 0x9a, if same { 2 } else { 1 });
}

/// FUN_0012c140(c, k, i): 3-axis Hermite key segment (12-byte keys {v, ?, tan_in} per axis +0xc..+0x14,
/// times +8, count +4, constants +0x20): fills k+0x80/+0x84 (t0/t1), +0xc0/+0xd0 (v0/tan0), +0xe0/+0xf0
/// (v1/tan1); k+0x9a = 2 when the segment is constant (equal ends, zero tangents), else 1. Out-of-range
/// index reads go through QMem like the EE.
pub fn ee_sub_12c140<M: QMem + ?Sized>(m: &mut M, c: u32, k: u32, i: u32) {
    stack_push(m, 0x70);
    axis_seg(m, c, k, i, 12, 0x0012_c0e0, 8);
    stack_pop(m, 0x70);
}

/// FUN_0012c768(c, k, i): as FUN_0012c140 with 8-byte {v, tan} keys and index search FUN_0012c708.
pub fn ee_sub_12c768<M: QMem + ?Sized>(m: &mut M, c: u32, k: u32, i: u32) {
    stack_push(m, 0x70);
    axis_seg(m, c, k, i, 8, 0x0012_c708, 4);
    stack_pop(m, 0x70);
}

/// FUN_0012cda0(c, a1, a2, t, f13): scalar curve: first key i >= 1 (of +4, 16-byte {t, v, tan_out,
/// tan_in} at +8) with t < key.t (i = count if none) and Hermite FUN_00117030(a1, a2, dt, t - t0, f13,
/// v0, tan_in0, v1, tan_out1).
pub fn ee_sub_12cda0<M: QMem + ?Sized>(m: &mut M, c: u32, a1: u32, a2: u32, t: f32, f13: f32) {
    stack_push(m, 0x10);
    let n = m.r32(c + 4) as i32;
    let mut i = 1i32;
    while i < n {
        let kk = m.r32(c + 8) + 0x10 * i as u32;
        if t < m.rf(kk) {
            break;
        }
        i += 1;
    }
    let kk = m.r32(c + 8).wrapping_add((i as u32) << 4);
    let p = kk.wrapping_sub(0x10);
    let (t0, t1) = (m.rf(p), m.rf(kk));
    let (pv, pt, kv, kt) = (m.rf(p + 4), m.rf(p + 0xc), m.rf(kk + 4), m.rf(kk + 8));
    m.call(0x0011_7030, &[a1, a2], &[f4(t1 - t0), f4(t - t0), f4(f13), f4(pv), f4(pt), f4(kv), f4(kt)]);
    stack_pop(m, 0x10);
}

/// FUN_00130b18(a): ASEG retract/advance: unless FUN_00131968(a) (done), lock (vtable +8), then for the
/// remaining events i = a+0x54 .. aseg(a+0x14)+0xa8 stamp event time a+0x1c (+0xac stride 0x10) and fire
/// FUN_00131290; bail out (unlock +0x10) as soon as FUN_00131968 reports done. Otherwise finish:
/// FUN_00131990(a, 0), unlink from 0x3e1218 / aseg+0x50, free +0x34 (FUN_00230100/FUN_0012e210) and +0x38
/// (owner vtable +0x168, FUN_0012ce70) lists, FUN_00258040(a+0x5c, 0), notify (+0x18 kind 3, parent +0x28
/// kind 2), FUN_00133508(a, 2, a), unlock twice.
pub fn ee_sub_130b18<M: QMem + ?Sized>(m: &mut M, a: u32) {
    if m.call(0x0013_1968, &[a], &[]).0 != 0 {
        return;
    }
    vcall(m, a, 8, &[], &[]);
    let mut i = m.r32(a + 0x54);
    loop {
        let sg = m.r32(a + 0x14);
        if i as i32 >= m.r32(sg + 0xa8) as i32 {
            break;
        }
        let ev = m.r32(sg + 0xac).wrapping_add(i << 4);
        let t = m.rf(a + 0x1c);
        m.wf(ev, t);
        m.call(0x0013_1290, &[a, ev], &[]);
        if m.call(0x0013_1968, &[a], &[]).0 != 0 {
            vcall(m, a, 0x10, &[], &[]);
            return;
        }
        i = m.r32(a + 0x54) + 1;
        m.w32(a + 0x54, i);
    }
    m.call(0x0013_1990, &[a, 0], &[]);
    m.call(0x0011_a2e8, &[0x003e_1218, a], &[]);
    let sg = m.r32(a + 0x14);
    m.call(0x0011_a2e8, &[sg + 0x50, a], &[]);
    let mut s1 = m.r32(a + 0x34);
    while s1 != 0 {
        let nx = m.r32(s1 + 0x28);
        let (x, y) = (m.r32(s1 + 4), m.r32(s1 + 8));
        m.call(0x0023_0100, &[x, y, s1], &[]);
        m.call(0x0012_e210, &[s1], &[]);
        s1 = nx;
    }
    m.w32(a + 0x34, 0);
    let mut s1 = m.r32(a + 0x38);
    while s1 != 0 {
        let (ow, arg, nx) = (m.r32(s1 + 8), m.r32(s1 + 0xc), m.r32(s1 + 0x10));
        vcall(m, ow, 0x168, &[arg], &[]);
        m.call(0x0012_ce70, &[s1], &[]);
        s1 = nx;
    }
    m.w32(a + 0x38, 0);
    m.call(0x0025_8040, &[a + 0x5c, 0], &[]);
    vcall(m, a, 0x18, &[3, 0, 0, 0, 0], &[]);
    let p = m.r32(a + 0x18);
    if p != 0 {
        vcall(m, p, 0x28, &[2, a], &[]);
    }
    m.call(0x0013_3508, &[a, 2, a], &[]);
    vcall(m, a, 0x10, &[], &[]);
    vcall(m, a, 0x10, &[], &[]);
}

/// FUN_00131a20(r, s): rate: r+0x48 = s, r+0x40 = r+0x44 * s * r+0x4c.
pub fn ee_sub_131a20<M: QMem + ?Sized>(m: &mut M, r: u32, s: f32) {
    let f = m.rf(r + 0x44) * s;
    m.wf(r + 0x48, s);
    let g = m.rf(r + 0x4c);
    m.wf(r + 0x40, f * g);
}

/// FUN_001331f8(o, i, now, a3, t0, t1): start SMA transition i of o's state machine (o+0) when
/// FUN_00127db8(sm, i, 1) allows: build the transition name (FUN_00200da0 with label table
/// 0x27a10c[0x27a1d8[i]]), FUN_00132ed8 resolve, then play now (FUN_00133040 on the world at DAT_002dded0)
/// or queue (FUN_00133000(o, seg, sm, t1)); drop the ref (count +0, free +0x10/+8/self to 0x4520a8) and
/// FUN_00129ec8 the name.
pub fn ee_sub_1331f8<M: QMem + ?Sized>(m: &mut M, o: u32, i: u32, now: u32, a3: u32, t0: u32, t1: u32) {
    let sp = stack_push(m, 0x90);
    let sm = m.r32(o);
    if sm != 0 && m.call(0x0012_7db8, &[sm, i, 1], &[]).0 != 0 {
        let lab = m.r32(0x0027_a1d8 + 4 * i);
        m.call(0x0020_0da0, &[sp, a3, 0x0027_a10c + 4 * lab, t0], &[]);
        m.call(0x0013_2ed8, &[sp + 0x10, o, i, sp], &[]);
        let seg = m.r32(sp + 0x10);
        if now != 0 {
            let t = m.rf(0x002d_ded0);
            m.call(0x0013_3040, &[0x003d_d490, o, seg], &[f4(t)]);
        } else {
            let sm = m.r32(o);
            m.call(0x0013_3000, &[o, seg, sm, t1], &[]);
        }
        let s0 = m.r32(sp + 0x10);
        if s0 != 0 {
            let c = m.r32(s0).wrapping_sub(1);
            m.w32(s0, c);
            if c == 0 {
                m.call(0x0025_6d40, &[s0 + 0x10, 2], &[]);
                m.call(0x0012_9ec8, &[s0 + 8], &[]);
                m.call(0x0025_50f8, &[s0], &[]);
                m.call(0x0011_cd98, &[0x0045_20a8, s0], &[]);
            }
        }
        m.call(0x0012_9ec8, &[sp], &[]);
    }
    stack_pop(m, 0x90);
}

/// FUN_00133508(o, a1, a2): notify o (vtable +0x28 (a1, a2)) and every listener in the chain o+8 (node:
/// +0 target, +8 next).
pub fn ee_sub_133508<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32, a2: u32) {
    vcall(m, o, 0x28, &[a1, a2], &[]);
    let mut s0 = m.r32(o + 8);
    while s0 != 0 {
        let t = m.r32(s0);
        s0 = m.r32(s0 + 8);
        vcall(m, t, 0x28, &[a1, a2], &[]);
    }
}

fn clamp_tab<M: QMem + ?Sized>(m: &mut M, tab: u32, s: f32) -> f32 {
    let f = m.rf(tab) + s * (m.rf(tab + 4) + s * m.rf(tab + 8));
    let (lo, hi) = (m.rf(0x002b_bb60), m.rf(0x002b_bb64));
    if f < lo {
        lo
    } else if hi < f {
        hi
    } else {
        f
    }
}

/// FUN_00134da8 tail: o+0x190 = rows(o+0x1b0..0x1d0) . v (w from row 0), o+0x1038 = -1, FUN_001a0450
fn vel_tail_134da8<M: QMem + ?Sized>(m: &mut M, o: u32, v: V4, a1: u32, a2: u32) {
    let (r0, r1, r2) = (m.rv(o + 0x1b0), m.rv(o + 0x1c0), m.rv(o + 0x1d0));
    m.w32(o + 0x1038, u32::MAX);
    m.wv(o + 0x190, [dot3(r0, v), dot3(r1, v), dot3(r2, v), r0[3]]);
    m.call(0x001a_0450, &[o, a1, a2], &[]);
}

/// FUN_00134da8(o): player per-frame movement update: FUN_001403d0, then (o+0x28 bit 60) FUN_00194a40 /
/// FUN_00134d30 / FUN_00134850 and the state switch on o+0x13d0 (heading FUN_00204660, sub-controllers
/// +0x1320/+0x1600 FUN_0020bb58/FUN_0020bc08, +0x2c0 / +0x4c0 updates, run/walk velocity from the stick
/// (FUN_0011e1a0 polar, speed tables 0x2bc940/0x2bc950 clamped by 0x2bbb60), reverse states via
/// FUN_00204e38), then FUN_00135eb0 and FUN_001a1358(o+0xdf0).
pub fn ee_sub_134da8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x0014_03d0, &[o], &[]);
    if (m.r64(o + 0x28) >> 60) & 1 == 0 {
        return;
    }
    m.call(0x0019_4a40, &[0x003d_d490, o, 0xf8fe], &[]);
    m.call(0x0013_4d30, &[o], &[]);
    m.call(0x0013_4850, &[o], &[]);
    let p = m.r32(o + 0x150);
    let pi = f32::from_bits(0x4049_0fdb);
    match m.r32(o + 0x13d0) {
        0 => {
            if m.r32(o + 0x181c) != 0 {
                let f = m.rf(0x002b_c934);
                m.call(0x0020_4660, &[o], &[f4(f)]);
            } else {
                let v = m.call(0x0014_30c8, &[o], &[]).0;
                let x = m.r32(o + 0x1034);
                m.call(0x001f_1350, &[x, v], &[]);
            }
        }
        1 | 7 => {
            let h = m.rf(p + 0x18);
            m.call(0x0020_4660, &[o], &[f4(h)]);
            let (s0, f) = if m.r32(o + 0x13d0) == 1 {
                (o + 0x1320, 1.0)
            } else {
                let p = m.r32(o + 0x150);
                (o + 0x1600, m.rf(p + 0x1c))
            };
            m.call(0x0020_bb58, &[s0], &[f4(f)]);
            m.call(0x0020_bc08, &[s0], &[]);
        }
        2 => {
            m.call(0x0019_7f98, &[o + 0x2c0], &[]);
        }
        3 => {
            m.call(0x0019_1a10, &[o + 0x4c0], &[]);
        }
        5 => {
            let x = m.r32(o + 0x10dc);
            m.call(0x0020_46f0, &[o, x], &[]);
            let d = m.rv(o + 0x1410);
            let sp_ = (d[0] * d[0] + d[1] * d[1]).sqrt();
            let p = m.r32(o + 0x150);
            let r_ = m.r32(o + 0x1030);
            let k = m.rf(r_ + 0x40);
            let f13 = m.rf(p + 0x1c) * sp_ * k;
            let f12 = m.rf(p + 0x18) + pi;
            let v = m.call(0x0011_e1a0, &[], &[f4(f12), f4(f13), f4(0.0)]).1;
            m.wv(o + 0x190, v);
            let a2 = m.r32(o + 0x14e0);
            vel_tail_134da8(m, o, v, o + 0x13e0, a2);
        }
        6 => {
            if m.r32(o + 0x16b0) == 1 {
                let s = m.rf(p + 0x1c);
                let f21 = clamp_tab(m, 0x002b_c940, s);
                let f20 = clamp_tab(m, 0x002b_c950, s);
                let p = m.r32(o + 0x150);
                if 0.0 < m.rf(p + 0x1c) {
                    let a = m.rf(p + 0x18) - m.rf(o + 0x1a8);
                    let w = m.call(0x0011_cf60, &[], &[f4(a)]).1[0];
                    let f13 = f20 * m.rf(0x002b_c960) * m.rf(0x002d_ded4);
                    let r = m.call(0x0011_cfe0, &[], &[f4(w), f4(f13)]).1[0];
                    let h = m.rf(o + 0x1a8) + r;
                    m.call(0x0020_4660, &[o], &[f4(h)]);
                }
                let d = m.rv(o + 0x1520);
                let sp_ = (d[0] * d[0] + d[1] * d[1]).sqrt();
                let p = m.r32(o + 0x150);
                let r_ = m.r32(o + 0x1030);
            let k = m.rf(r_ + 0x40);
                let f13 = f21 * sp_ * k;
                let f12 = m.rf(p + 0x18) + pi;
                let v = m.call(0x0011_e1a0, &[], &[f4(f12), f4(f13), f4(0.0)]).1;
                m.wv(o + 0x190, v);
                let a2 = m.r32(o + 0x15f0);
                vel_tail_134da8(m, o, v, o + 0x14f0, a2);
            }
        }
        st @ (10 | 11 | 14) => {
            let a1 = m.r32(o + 0x1030);
            let f = m.rf(a1 + 0x1c);
            let v = m.call(0x0020_4e38, &[o, a1], &[f4(f)]).1;
            m.wv(o + 0x190, scale(v, -1.0));
            if st == 11 {
                let r = m.r32(o + 0x1030);
                let p = m.r32(o + 0x150);
                let s = m.rf(p + 0x1c);
                m.call(0x0013_1a00, &[r], &[f4(s)]);
            }
            if st != 10 {
                let p = m.r32(o + 0x150);
                let h = m.rf(p + 0x18);
                m.call(0x0020_4660, &[o], &[f4(h)]);
            }
        }
        _ => {}
    }
    m.call(0x0013_5eb0, &[o], &[]);
    m.call(0x001a_1358, &[o + 0xdf0], &[]);
}

/// FUN_001357c0(o, tgt): fire: FUN_001356e8(o, tgt); spawn the projectile template (o+0x16dc / +0x16e0
/// by o+0x16d8 = 0 / 1) via FUN_001cb250, aim it at tgt (+0xe4 -> +0x18), launch from the emitter o+0x16e4
/// (+0x60 = 0, +0x50 = o+0x16f0, FUN_001d6b00), zero velocity (vtable +0x120), FUN_0013b5c8(p, 4.0),
/// FUN_001362f8(o, 1), o+0x1700 = now (DAT_002dded0).
pub fn ee_sub_1357c0<M: QMem + ?Sized>(m: &mut M, o: u32, tgt: u32) {
    m.call(0x0013_56e8, &[o, tgt], &[]);
    let t = match m.r32(o + 0x16d8) {
        0 => m.r32(o + 0x16dc),
        1 => m.r32(o + 0x16e0),
        _ => return,
    };
    if t == 0 {
        return;
    }
    let s0 = m.call(0x001c_b250, &[t, 0], &[]).0;
    if s0 == 0 {
        return;
    }
    let x = m.r32(s0 + 0xe4);
    m.w32(x + 0x18, tgt);
    let e = m.r32(o + 0x16e4);
    m.w32(e + 0x60, 0);
    let v = m.rv(o + 0x16f0);
    m.wv(e + 0x50, v);
    m.call(0x001d_6b00, &[e, s0], &[]);
    let z = m.rv(0x002a_2dd0);
    vcall(m, s0, 0x120, &[], &[z]);
    m.call(0x0013_b5c8, &[s0], &[f4(4.0)]);
    m.call(0x0013_62f8, &[o, 1], &[]);
    let now = m.rf(0x002d_ded0);
    m.wf(o + 0x1700, now);
}

/// FUN_00136530(o, mode): switch the carry/hold mode o+0x1800: leave the old mode (0: FUN_001a58d0 detach
/// keeping +0x9e4, release +0x1810/+0x1814 FUN_0012e460, FUN_002303a0 on +0x58/+0x5c; 1: re-face
/// FUN_00204740 -> FUN_00204660, +0x1a4 = 0, FUN_00204ca8), stamp +0x1804; enter: 0 -> attach to the player
/// (DAT_002f7900) bone (+0x191c vtable +0x100, FUN_0022ff40 frame, vtable +0x50/+0x118/+0x120, FUN_0012dc18
/// constraints into +0x1810/+0x1814); -1 -> FUN_002206c8(player+0x1910, 0) when the player is in state 15
/// and not sub-state 7 (then return); 1 -> +0x1a4 = 1. Then FUN_001f15b8(+0x1034), FUN_001305b8(anim
/// +0x1808[mode], o, 9, +0x1030, 0, 1), vtable +0x2d0.
pub fn ee_sub_136530<M: QMem + ?Sized>(m: &mut M, o: u32, mode: u32) {
    let old = m.r32(o + 0x1800);
    if mode == old {
        return;
    }
    let sp = stack_push(m, 0xd0);
    match old {
        0 => {
            let s0 = m.r32(o + 0x9e4);
            m.call(0x001a_58d0, &[o, 0, 0xf], &[]);
            m.w32(o + 0x9e4, s0);
            m.call(0x0012_e460, &[o + 0x1810], &[]);
            m.call(0x0012_e460, &[o + 0x1814], &[]);
            let a = m.r32(o + 0x58);
            m.call(0x0023_03a0, &[o, a, 2], &[]);
            let a = m.r32(o + 0x5c);
            m.call(0x0023_03a0, &[o, a, 2], &[]);
        }
        1 => {
            let f = m.call(0x0020_4740, &[o], &[]).1[0];
            m.call(0x0020_4660, &[o], &[f4(f)]);
            m.w32(o + 0x1a4, 0);
            m.call(0x0020_4ca8, &[o], &[]);
        }
        _ => {}
    }
    m.w32(o + 0x1800, mode);
    let now = m.rf(0x002d_ded0);
    m.wf(o + 0x1804, now);
    let s3 = m.r32(o + 0x1808 + 4 * mode);
    match mode as i32 {
        0 => {
            let pl = m.r32(0x002f_7900);
            let b = m.r32(pl + 0x191c);
            let s0 = vcall(m, b, 0x100, &[sp], &[]).0;
            let x = m.r32(o + 0x54);
            let x = m.r32(x + 0xa0);
            let y = m.r32(o + 0x58);
            let y = m.r32(y + 0xa0);
            m.call(0x0022_ff40, &[sp + 0x40, x, y, sp], &[]);
            let f = m.rm(sp + 0x40);
            m.wm(sp, f);
            let pl = m.r32(0x002f_7900);
            let s1 = m.r32(o + 0x9e4);
            vcall(m, o, 0x50, &[pl, s0, 3], &[]);
            m.w32(o + 0x9e4, s1);
            let z = m.rv(0x002a_2dd0);
            vcall(m, o, 0x118, &[], &[z]);
            let z = m.rv(0x002a_2dd0);
            vcall(m, o, 0x120, &[], &[z]);
            let t = m.rv(sp + 0x30);
            let a = m.r32(o + 0x58);
            let r = m.call(0x0012_dc18, &[o, a, 1, 7, 2], &[t]).0;
            m.w32(o + 0x1810, r);
            let v = m.call(0x0011_bff0, &[sp], &[]).1;
            let a = m.r32(o + 0x5c);
            let r = m.call(0x0012_dc18, &[o, a, 1, 7, 2], &[v]).0;
            m.w32(o + 0x1814, r);
        }
        -1 => {
            let pl = m.r32(0x002f_7900);
            if m.r32(pl + 0x13d0) == 15 && m.r32(pl + 0x1914) != 7 {
                m.call(0x0022_06c8, &[pl + 0x1910, 0], &[]);
            }
            stack_pop(m, 0xd0);
            return;
        }
        1 => m.w32(o + 0x1a4, u32::MAX), // PART r17: asm 0x136620 beq delay slot li v0, -1
        _ => {}
    }
    m.call(0x001f_15b8, &[o + 0x1034], &[]);
    m.call(0x0013_05b8, &[s3, o, 9, o + 0x1030], &[f4(0.0), f4(1.0)]);
    vcall(m, o, 0x2d0, &[], &[]);
    stack_pop(m, 0xd0);
}

/// cfront pointer-to-member call stored at `pmf` (+0 short delta, +2 short index, +4 fn / short vdelta):
/// index < 0 -> fn(obj + delta); else the vtable entry (index-1) of obj+vdelta (short adj, fn).
fn pmf_call<M: QMem + ?Sized>(m: &mut M, obj: u32, pmf: u32, a: &[u32]) -> u32 {
    let d = m.r16(pmf) as i16 as i32;
    let i = m.r16(pmf + 2) as i16 as i32;
    let (f, this) = if i < 0 {
        (m.r32(pmf + 4), (obj as i32 + d) as u32)
    } else {
        let vd = m.r16(pmf + 4) as i16 as i32;
        let vt = m.r32((obj as i32 + vd) as u32);
        let e = vt.wrapping_add((i as u32) * 8).wrapping_sub(8);
        let adj = m.r16(e) as i16 as i32;
        (m.r32(e + 4), (obj as i32 + adj + d) as u32)
    };
    let mut args = vec![this];
    args.extend_from_slice(a);
    m.call(f, &args, &[]).0
}

/// FUN_00149c80(vm): drain the deferred member-call queue (o+0x3ca8, FUN_0011a1e8 empty / FUN_0011a060 pop):
/// each chain node {obj, pmf +4..+0xb, arg +0xc, next +0x10} is called then freed to pool o+0x3c90.
/// Then fire due timers in o+0x39f0 (time +0 <= DAT_002dded0): +0xc = -1, FUN_00133000(target or vm,
/// script +8, ctx, 0) (no target: temp context FUN_00256dd0/FUN_00257058, released via FUN_00257078),
/// unlink (FUN_0011a118), drop the script ref (last ref: FUN_00256d40/FUN_00129ec8/FUN_002550f8 and free to
/// 0x4520a8), free the node to o+0x39fc.
pub fn ee_sub_149c80<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let sp = stack_push(m, 0x10);
    let q = o + 0x3ca8;
    while m.call(0x0011_a1e8, &[q], &[]).0 == 0 {
        let mut s1 = m.r32(q);
        m.call(0x0011_a060, &[q], &[]);
        while s1 != 0 {
            let obj = m.r32(s1);
            let arg = m.r32(s1 + 0xc);
            pmf_call(m, obj, s1 + 4, &[arg]);
            let nx = m.r32(s1 + 0x10);
            m.call(0x0011_cd98, &[o + 0x3c90, s1], &[]);
            s1 = nx;
        }
    }
    loop {
        let s1 = m.r32(o + 0x39f0);
        if s1 == 0 || m.rf(0x002d_ded0) < m.rf(s1) {
            break;
        }
        m.w32(s1 + 0xc, u32::MAX);
        let t = m.r32(s1 + 4);
        if t != 0 {
            let ctx = m.r32(t);
            let sc = m.r32(s1 + 8);
            m.call(0x0013_3000, &[t, sc, ctx, 0], &[]);
        } else {
            let v = m.call(0x0025_6dd0, &[], &[]).0;
            m.call(0x0025_7058, &[sp, v], &[]);
            let (sc, ctx) = (m.r32(s1 + 8), m.r32(sp));
            m.call(0x0013_3000, &[o, sc, ctx, 0], &[]);
            let a = m.r32(sp);
            if a != 0 {
                let n = m.r32(a).wrapping_sub(1);
                m.w32(a, n);
                if n == 0 {
                    m.call(0x0025_7078, &[a, 3], &[]);
                }
            }
        }
        m.call(0x0011_a118, &[o + 0x39f0, s1], &[]);
        let s0 = m.r32(s1 + 8);
        let n = m.r32(s0).wrapping_sub(1);
        m.w32(s0, n);
        if n == 0 && s0 != 0 {
            m.call(0x0025_6d40, &[s0 + 0x10, 2], &[]);
            m.call(0x0012_9ec8, &[s0 + 8], &[]);
            m.call(0x0025_50f8, &[s0], &[]);
            m.call(0x0011_cd98, &[0x0045_20a8, s0], &[]);
        }
        m.call(0x0011_cd98, &[o + 0x39fc, s1], &[]);
    }
    stack_pop(m, 0x10);
}

/// FUN_001375d0(jt, mode): jump-type change (jt+4 = mode, +8 = DAT_002dded0). mode -1: actor velocity
/// (vtable +0x108) scaled by DAT_002bc9b8 -> vtable +0x118. mode 0/1: anim jt+0x10 / +0x14 (FUN_002046f0
/// with actor+0x10dc), root velocity from its FUN_0012ff10 channel (vtable +0x18, t 0, rate anim+0x78)
/// rotated by the actor matrix +0x1b0 -> vtable +0x118, actor+0x190 = -root. All modes != -1: jt+0xc = mode,
/// FUN_001f15b8(actor+0x1034), FUN_001305b8(anim, actor, 1, actor+0x1030, 0.0, 1.0).
pub fn ee_sub_1375d0<M: QMem + ?Sized>(m: &mut M, s0: u32, mode: u32) {
    if mode == m.r32(s0 + 4) {
        return;
    }
    m.w32(s0 + 4, mode);
    let now = m.rf(0x002d_ded0);
    m.wf(s0 + 8, now);
    let actor = m.r32(s0);
    if mode == u32::MAX {
        let k = m.rf(0x002b_c9b8);
        let v = vcall(m, actor, 0x108, &[], &[]).1;
        vcall(m, actor, 0x118, &[], &[scale(v, k)]);
        return;
    }
    let sp = stack_push(m, 0x20);
    let mi = mode as i32;
    let mut s2 = 0;
    if (0..2).contains(&mi) {
        s2 = m.r32(s0 + if mi == 0 { 0x10 } else { 0x14 });
        let a5 = m.r32(actor + 0x10dc);
        m.call(0x0020_46f0, &[actor, a5], &[]);
        let ch = m.call(0x0012_ff10, &[s2, actor, 1, 1], &[]).0;
        let a3 = m.r32(ch + 4);
        let vt = m.r32(a3);
        let adj = m.r16(vt + 0x18) as i16 as i32;
        let f = m.r32(vt + 0x1c);
        let rate = m.rf(s2 + 0x78);
        m.call(f, &[(a3 as i32 + adj) as u32, 0, sp], &[f4(0.0), f4(rate)]);
        let r = m.rv(sp);
        let (r0, r1, r2) = (m.rv(actor + 0x1b0), m.rv(actor + 0x1c0), m.rv(actor + 0x1d0));
        let w = [
            r0[0] * r[0] + r1[0] * r[1] + r2[0] * r[2],
            r0[1] * r[0] + r1[1] * r[1] + r2[1] * r[2],
            r0[2] * r[0] + r1[2] * r[1] + r2[2] * r[2],
            0.0,
        ];
        vcall(m, actor, 0x118, &[], &[w]);
        m.wv(actor + 0x190, scale(r, -1.0));
        m.w32(s0 + 0xc, mode); // VER3: only modes 0 / 1 store +0xc (asm 0x137738; others branch to 0x137740)
    }
    m.call(0x001f_15b8, &[actor + 0x1034], &[]);
    let a = m.r32(s0);
    m.call(0x0013_05b8, &[s2, a, 1, a + 0x1030], &[f4(0.0), f4(1.0)]);
    stack_pop(m, 0x20);
}

fn alloc_req<M: QMem + ?Sized>(m: &mut M, sp: u32, f: u32, size: u32, a: &[u32]) -> u32 {
    m.w32(sp + 0x10, size);
    m.w32(sp + 0x14, 0x10);
    let mut args = vec![sp + 0x10];
    args.extend_from_slice(a);
    m.call(f, &args, &[]).0
}

/// FUN_00137a30(ctx, stride, n, entries, nch, ch_a, ch_b, out): build a jt blend (FUN_0013e3c8 kind 0x6e,
/// vtable +0x48 init) from n weighted entries {w +0, anim +4}: channel table +0xd0 (nch x {a, b, buf b*8}),
/// first anim with +0x98/+0xa8/+0xa0 data copied into +0x98..+0xac (+0xa4 duplicated), merged track list
/// +0x5c/+0x60 keyed by (word & 0x3fffffff) skipping 0x40000000 tracks; new track nodes are FUN_0012cb98(2)
/// (bit 31) or FUN_0012b3b0(5) seeded via FUN_0012fe58/FUN_0022ca18, each with an n-slot per-entry table.
/// Weights become 1.0 for entries of the first anim, else 0; FUN_00256ee0 with the first anim having a
/// name, FUN_001384c0, *out = blend. No anim at all: *out = 0.
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_137a30<M: QMem + ?Sized>(m: &mut M, ctx: u32, stride: u32, n: u32, ents: u32, nch: u32, cha: u32, chb: u32, out: u32) {
    let n = n as i32;
    let (mut s7, mut first, mut named) = (0u32, 0u32, 0u32);
    for i in 0..n.max(0) as u32 {
        let e = ents + i * stride;
        let v1 = m.r32(e + 4);
        if v1 == 0 {
            continue;
        }
        if first == 0 {
            first = v1;
        }
        if s7 == 0 && (m.r32(v1 + 0x98) != 0 || m.r32(v1 + 0xa8) != 0 || m.r32(v1 + 0xa0) != 0) {
            s7 = v1;
        }
        if named == 0 {
            named = if m.r32(v1) == 0 { 0 } else { v1 };
        }
    }
    if first == 0 {
        m.w32(out, 0);
        return;
    }
    let sp = stack_push(m, 0x40);
    m.call(0x001a_e478, &[sp], &[]);
    let s2 = m.call(0x0013_e3c8, &[0x6e, ctx, u32::MAX, u32::MAX], &[]).0;
    vcall(m, s2, 0x48, &[], &[]);
    m.w32(s2 + 0xc0, stride);
    m.w32(s2 + 0xc4, n as u32);
    m.w32(s2 + 0xc8, ents);
    m.w32(s2 + 0xcc, nch);
    let t = alloc_req(m, sp, 0x001a_e1d0, nch.wrapping_mul(12), &[]);
    m.w32(s2 + 0xd0, t);
    let mut s5 = cha;
    if (nch as i32) > 0 {
        s5 = 0;
        for i in 0..nch {
            let rec = m.r32(s2 + 0xd0) + s5;
            let a = m.r32(cha + 4 * i);
            m.w32(rec, a);
            s5 += 0xc;
            let b = m.r32(chb + 4 * i);
            m.w32(rec + 4, b);
            let buf = alloc_req(m, sp, 0x001a_e1d0, b << 3, &[]);
            m.w32(rec + 8, buf);
        }
    }
    if s7 != 0 {
        m.w32(s2 + 0xd8, s7);
        for o in [0x98, 0x9c, 0xa0] {
            let v = m.r32(s7 + o);
            m.w32(s2 + o, v);
        }
        let (c, src) = (m.r32(s7 + 0xa0), m.r32(s7 + 0xa4));
        let v = alloc_req(m, sp, 0x001a_e238, c << 4, &[src]);
        m.w32(s2 + 0xa4, v);
        for o in [0xa8, 0xac] {
            let v = m.r32(s7 + o);
            m.w32(s2 + o, v);
        }
    }
    let mut total = 0u32;
    for i in 0..n.max(0) as u32 {
        let a = m.r32(ents + i * stride + 4);
        if a != 0 {
            total = total.wrapping_add(m.r32(a + 0x5c));
        }
    }
    let tl = alloc_req(m, sp, 0x001a_e378, total << 3, &[]);
    m.w32(s2 + 0x5c, 0);
    m.w32(s2 + 0x60, tl);
    for s3 in 0..n.max(0) as u32 {
        let e = ents + s3 * stride;
        let mut k = 0u32;
        loop {
            let an = m.r32(e + 4);
            if an == 0 || (k as i32) >= m.r32(an + 0x5c) as i32 {
                break;
            }
            let s4 = m.r32(an + 0x60) + k * 8;
            k += 1;
            let w = m.r32(s4);
            if w & 0x4000_0000 != 0 {
                continue;
            }
            let key = w & 0x3fff_ffff;
            let cnt = m.r32(s2 + 0x5c) as i32;
            let base = m.r32(s2 + 0x60);
            let mut s1 = 0;
            for j in 0..cnt.max(0) as u32 {
                if m.r32(base + 8 * j) & 0x3fff_ffff == key {
                    s1 = base + 8 * j;
                    break;
                }
            }
            let mut bit31 = false;
            if s1 == 0 {
                s1 = base + cnt as u32 * 8;
                m.w32(s2 + 0x5c, (cnt + 1) as u32);
                let v = (m.r32(s1) & 0xfff0_0000) | (m.r32(s4) & 0x000f_ffff);
                m.w32(s1, v);
                let v = (v & 0xc00f_ffff) | (m.r32(s4) & 0x3ff0_0000);
                m.w32(s1, v);
                s5 = 0x8000_0000;
                bit31 = (m.r32(s4) as i32) < 0;
            }
            let mut s0 = m.r32(s1 + 4);
            if s0 == 0 {
                if bit31 {
                    s0 = m.call(0x0012_cb98, &[2], &[]).0;
                } else {
                    s0 = m.call(0x0012_b3b0, &[5], &[]).0;
                    m.call(0x0012_fe58, &[s2, ctx, s1, sp + 0x20, sp + 0x24], &[]);
                    let a = m.r32(sp + 0x24);
                    let v = m.call(0x0022_ca18, &[a, a], &[]).1;
                    m.wv(s0 + 0x10, v);
                }
                m.w32(s0 + 4, s2);
                let c = m.r32(s2 + 0xd0);
                m.w32(s0 + 8, c);
                let tb = alloc_req(m, sp, 0x001a_e2a0, (n as u32) << 2, &[]);
                m.w32(s0 + 0xc, tb);
                m.w32(s1 + 4, s0);
            }
            let v = m.r32(s1) | s5;
            m.w32(s1, v);
            let slot = m.r32(s0 + 0xc) + s3 * 4;
            let x = m.r32(s4 + 4);
            m.w32(slot, x);
        }
    }
    let (c, tl) = (m.r32(s2 + 0x5c), m.r32(s2 + 0x60));
    let v = alloc_req(m, sp, 0x001a_e238, c << 3, &[tl]);
    m.w32(s2 + 0x60, v);
    let v = m.r32(first + 0x6c);
    m.w32(s2 + 0x6c, v);
    let f = m.rf(first + 0x78);
    m.w32(s2 + 0xd4, u32::MAX);
    m.wf(s2 + 0x78, f);
    if named != 0 {
        let nm = m.r32(named);
        m.call(0x0025_6ee0, &[s2, nm], &[]);
    }
    for i in 0..n.max(0) as u32 {
        let e = ents + i * stride;
        let w = if m.r32(e + 4) == first { 1.0 } else { 0.0 };
        m.wf(e, w);
    }
    m.call(0x0013_84c0, &[s2], &[]);
    m.w32(out, s2);
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0x40);
}

/// FUN_0014dc58(cm): camera matrices. While shaking (now - cm+0x618 < DAT_002bd850) the view (cm+0x60) is
/// multiplied by a shake transform (FUN_0011b570 rot + FUN_00256d18 offset, amplitude = FUN_00245000 wave *
/// clamped quadratic decay) and the projection (FUN_0011c448) is offset by the shake; FUN_0014d8f8 composes
/// cm+0x140, FUN_0011c2f0 inverts to +0x180; clip planes FUN_0014da08 (+0x1c0 and +0x200 with widened fov)
/// are transposed into +0xa0 / +0xd0; FUN_0014e078.
pub fn ee_sub_14dc58<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let dt = m.rf(0x002d_ded0) - m.rf(o + 0x618);
    let k = m.rf(0x002b_d850);
    if !(k < dt) {
        let sp = stack_push(m, 0xd0);
        let t = dt / k;
        let c = |m: &mut M, a| m.rf(a);
        let f3 = c(m, 0x002b_d860) + t * (c(m, 0x002b_d864) + t * c(m, 0x002b_d868));
        let (lo, hi) = (c(m, 0x002b_bb60), c(m, 0x002b_bb64));
        let f20 = if f3 < lo { lo } else if hi < f3 { hi } else { f3 };
        let ph = dt * c(m, 0x002b_d854);
        let w = m.call(0x0024_5000, &[], &[f4(ph)]).1[0];
        let amp = w * f20;
        let r = scale(m.rv(0x002b_d880), amp);
        m.call(0x0011_b570, &[sp + 0x40], &[r]);
        let r = scale(m.rv(0x002b_d870), amp);
        m.call(0x0025_6d18, &[sp + 0x30], &[r]);
        for i in 0..3 {
            let mut v = m.rv(sp + 0x40 + 0x10 * i);
            v[3] = 0.0;
            m.wv(sp + 0x10 * i, v);
        }
        m.wf(sp + 0x3c, 1.0);
        m.call(0x0011_c168, &[sp + 0x40, o + 0x60, sp], &[]);
        let row0 = m.rv(sp + 0x40);
        let row1 = m.rv(sp + 0x50);
        let a = madd1(m.rv(0x002a_2de0), m.rv(0x002b_d890), amp);
        let b = scale(m.rv(0x002b_d8a0), amp);
        let f12 = a[0] * 512.0 / (m.rf(o + 0x40) * 4096.0);
        let f13 = a[1] * 224.0 / (m.rf(o + 0x44) * 4096.0);
        let (f16, f17) = (m.rf(o + 0x4c), m.rf(o + 0x50));
        let (r2, r3) = (m.rv(sp + 0x60), m.rv(sp + 0x70));
        m.wv(sp, row0);
        m.wv(sp + 0x10, row1);
        m.wv(sp + 0x20, r2);
        m.wv(sp + 0x30, r3);
        m.wv(sp + 0x40, a);
        m.wv(sp + 0x50, b);
        m.call(0x0011_c448, &[sp + 0x80], &[f4(f12), f4(f13), f4(b[0]), f4(b[1]), f4(f16), f4(f17)]);
        m.call(0x0014_d8f8, &[sp, sp + 0x80, o + 0x140], &[]);
        stack_pop(m, 0xd0);
    } else {
        m.call(0x0014_d8f8, &[o + 0x60, o + 0x100, o + 0x140], &[]);
    }
    m.call(0x0011_c2f0, &[o + 0x140, o + 0x180], &[]);
    for (dst, src, ks) in [(0xa0u32, 0x1c0u32, (1.0f32, 1.0f32)), (0xd0, 0x200, (f32::from_bits(0x40fd_c28f), f32::from_bits(0x4190_cccd)))] {
        let (fx, fy) = (m.rf(o + 0x40) * ks.0, m.rf(o + 0x44) * ks.1);
        m.call(0x0014_da08, &[o + 0x60, o + src], &[f4(fx), f4(fy)]);
        for i in 0..3 {
            for j in 0..4 {
                let v = m.r32(o + src + 0x10 * j + 4 * i);
                m.w32(o + dst + 0x10 * i + 4 * j, v);
            }
        }
    }
    m.call(0x0014_e078, &[o], &[]);
}

/// FUN_0014ea08(n, worlds, p0, p1, lo, hi, gap): collect the blocked parameter intervals of segment p0-p1
/// against each world's BSP (FUN_00118668 over faces +0x74/+0x70, merged by FUN_0014c9f8, max 0x40 x 0x60)
/// and return 1 with the first free interval [lo, hi] wider than `gap` (from 0 to 1), else 0.
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_14ea08<M: QMem + ?Sized>(m: &mut M, n: u32, list: u32, p0: V4, p1: V4, lo: u32, hi: u32, gap: f32) -> u32 {
    let sp = stack_push(m, 0x1810);
    let mut cnt = 0u32;
    for i in 0..(n as i32).max(0) as u32 {
        let w = m.r32(list + 4 * i);
        let fb = m.r32(w + 0x74);
        let fe = m.r32(w + 0x70).wrapping_mul(12).wrapping_add(fb);
        let dst = sp + cnt * 0x60;
        let r = m.call(0x0011_8668, &[fb, fe, 0x40 - cnt, dst], &[p0, p1]).0;
        if r != 0 {
            cnt = m.call(0x0014_c9f8, &[cnt, sp, r, dst, 0x40, sp, 0], &[]).0;
        }
    }
    let mut ret = 0;
    for i in 0..=(cnt as i32).max(0) as u32 {
        let f2 = if i > 0 { m.rf(sp + 0x60 * (i - 1) + 0x44) } else { 0.0 };
        let f1 = if i < cnt { m.rf(sp + 0x60 * i + 0x40) } else { 1.0 };
        if gap < f1 - f2 {
            if lo != 0 {
                m.wf(lo, f2);
            }
            if hi != 0 {
                m.wf(hi, f1);
            }
            ret = 1;
            break;
        }
    }
    stack_pop(m, 0x1810);
    ret
}

/// FUN_00139370(o, src): particle look setup from FUN_001ec8d0(src): blend table o+0x438 (flag bit 0:
/// 0x2dd600 else 0x2dd5c0), first time halve the 4 colour bytes into o+0x3b4; frame count t = +0x10 rounded
/// up to a power of two (max 16) -> o+0x1a0; frames (8-byte refs from +0x14, tag 6) written to o+0x1b0 with
/// the first (pow2 - t) frames duplicated to pad.
pub fn ee_sub_139370<M: QMem + ?Sized>(m: &mut M, o: u32, src: u32) {
    let v = m.call(0x001e_c8d0, &[src], &[]).0;
    let tbl = if m.r32(v + 8) & 1 != 0 { 0x002d_d600 } else { 0x002d_d5c0 };
    m.w32(o + 0x438, tbl);
    if m.r32(o + 0x3b0) == 0 {
        m.w32(o + 0x3b0, 1);
        for i in 0..4 {
            let b = ((m.r8(v + 0xc + i) as u32 + 1) >> 1) as u8;
            m.w8(o + 0x3b4 + i, b);
        }
    }
    let t = m.r32(v + 0x10) as i32;
    let mut a: i32 = 1;
    let mut x = t >> 1;
    while x != 0 {
        x >>= 1;
        a <<= 1;
    }
    a <<= (a < t) as i32;
    while a >= 0x20 {
        a >>= 1;
    }
    let pad = if t < a { a - t } else { 0 };
    m.w32(o + 0x1a0, a as u32);
    let mut d = o + 0x1b0;
    for i in 0..t.max(0) {
        let s = m.r32(v + 0x14) + 8 * i as u32;
        let q = m.r64(s);
        m.w32(d, q as u32);
        m.w32(d + 4, (q >> 32) as u32);
        m.w32(d + 8, 6);
        if i < pad {
            let e = m.rv(d);
            m.wv(d + 0x10, e);
            d += 0x10;
        }
        d += 0x10;
    }
}

/// FUN_00162250(_, e, state, ev): job anim-cue record {state +0 (-1 any), oid +4, sub +8, cue +0xc,
/// needs-event +0x10, level +0x14 (-1 any, else == FUN_00210a88())}: find the 0x205 object (FUN_0015e650,
/// exactly one), with flag 0x30 (FUN_00132b10) and sub-object FUN_001f67e0; cue = +0xc, or for state -1 the
/// per-state cue 0x612..0x615 (states 0..3). ev 1/4 -> FUN_001f0948, else FUN_001f0660.
pub fn ee_sub_162250<M: QMem + ?Sized>(m: &mut M, _a0: u32, e: u32, state: u32, ev: u32) {
    let st = m.r32(e);
    if st != u32::MAX && st != state {
        return;
    }
    if m.r32(e + 0x10) != 0 && ev == 0 {
        return;
    }
    if m.r32(e + 0x14) != u32::MAX && m.r32(e + 0x14) != m.call(0x0021_0a88, &[], &[]).0 {
        return;
    }
    let sp = stack_push(m, 0x10);
    let oid = m.r32(e + 4);
    let n = m.call(0x0015_e650, &[0x003d_d490, 0x205, oid, 0, 1, sp], &[]).0;
    let obj = m.r32(sp);
    stack_pop(m, 0x10);
    if n != 1 || m.call(0x0013_2b10, &[obj, 0x30], &[]).0 == 0 {
        return;
    }
    let sub_ = m.r32(e + 8);
    let a0 = m.call(0x001f_67e0, &[obj, sub_], &[]).0;
    if a0 == 0 {
        return;
    }
    let mut cue = m.r32(e + 0xc);
    if m.r32(e) == u32::MAX {
        cue = match state {
            0 => 0x612,
            1 => 0x613,
            2 => 0x614,
            3 => 0x615,
            _ => cue,
        };
    }
    if cue == u32::MAX {
        return;
    }
    let f = if ev == 1 || ev == 4 { 0x001f_0948 } else { 0x001f_0660 };
    m.call(f, &[a0, cue], &[]);
}

/// FUN_001623d0(j, state, n, recs, ev): FUN_00162250 over n 0x18-byte cue records.
pub fn ee_sub_1623d0<M: QMem + ?Sized>(m: &mut M, j: u32, state: u32, n: u32, recs: u32, ev: u32) {
    for i in 0..(n as i32).max(0) as u32 {
        ee_sub_162250(m, j, recs + 0x18 * i, state, ev);
    }
}

/// FUN_001629a0(j): job start: FUN_00165dc0(world, j); if FUN_00162670(j): +0x7c++, FUN_00163460,
/// event 0x21 to 0x2deb30 (FUN_001619a0), start lists (FUN_001620b0 +0x8c/+0x90, cues +0x94/+0x98 ev +0xf0)
/// for state 4, then vtable +0x18 message 0x13.
pub fn ee_sub_1629a0<M: QMem + ?Sized>(m: &mut M, j: u32) {
    m.call(0x0016_5dc0, &[0x003d_d490, j], &[]);
    if m.call(0x0016_2670, &[j], &[]).0 == 0 {
        return;
    }
    let c = m.r32(j + 0x7c).wrapping_add(1);
    m.w32(j + 0x7c, c);
    m.call(0x0016_3460, &[j], &[]);
    m.call(0x0016_19a0, &[0x002d_eb30, 0x21, j], &[]);
    let (a, b) = (m.r32(j + 0x8c), m.r32(j + 0x90));
    m.call(0x0016_20b0, &[j, 4, a, b], &[]);
    let (a, b, ev) = (m.r32(j + 0x94), m.r32(j + 0x98), m.r32(j + 0xf0));
    ee_sub_1623d0(m, j, 4, a, b, ev);
    vcall(m, j, 0x18, &[0x13, 0, 0, 0, 0], &[]);
}

/// FUN_0014bea8(list, e): add e to list unless already present (FUN_0011a1b8 / FUN_0011a070); e+0x10 =
/// (e's descriptor id **e == DAT_002e0cb4).
pub fn ee_sub_14bea8<M: QMem + ?Sized>(m: &mut M, list: u32, e: u32) {
    if m.call(0x0011_a1b8, &[list, e], &[]).0 != 0 {
        return;
    }
    m.call(0x0011_a070, &[list, e], &[]);
    let d = m.r32(e);
    let v = (m.r32(d) == m.r32(0x002e_0cb4)) as u32;
    m.w32(e + 0x10, v);
}

fn len3(a: V4) -> f32 {
    dot3(a, a).sqrt()
}

/// FUN_00155fb0(t): aim/camera vantage point for target t: offset from the player (+50 up) either along the
/// player velocity (fast: |v| > DAT_002bdc98, scaled by a blended quadratic of the distance, clamp from
/// DAT_002bdc70/90 blended by FUN_00154c90) or the facing (FUN_002458f0 of the xf, radius DAT_002bdc98);
/// then 11 yaw offsets (DAT_002bdca0) are tried keeping the polar pitch/radius (FUN_0011efb0/FUN_0011e208):
/// the first clear one (FUN_0014cc58) wins, else the one whose hit lies farthest from t. Returns t + offset.
pub fn ee_sub_155fb0<M: QMem + ?Sized>(m: &mut M, t: V4) -> V4 {
    let sp = stack_push(m, 0x160);
    let p = m.call(0x001c_4840, &[0x003d_d490], &[]).0;
    let pp = m.call(0x001a_62e8, &[p], &[]).1;
    let d = len3(sub(pp, t));
    let f21 = m.call(0x0015_4c90, &[0x002d_e578], &[]).1[0];
    let pp2 = m.call(0x001a_62e8, &[p], &[]).1;
    let up = m.rv(0x002a_2dc0);
    let b = madd1(pp2, up, 50.0);
    let vel = vcall(m, p, 0x108, &[], &[]).1;
    let off;
    if m.rf(0x002b_dc98) < len3(vel) {
        let f20 = 1.0 - f21;
        m.call(0x0025_6ce8, &[sp + 0xb0], &[f4(f20)]);
        let s3 = m.rv(sp + 0xb0);
        m.call(0x0025_6d08, &[sp + 0xc0, 0x002b_dc60], &[]);
        let s2 = m.rv(sp + 0xc0);
        m.call(0x0025_6ce8, &[sp + 0xd0], &[f4(f21)]);
        let s0 = m.rv(sp + 0xd0);
        m.call(0x0025_6d08, &[sp + 0xe0, 0x002b_dc80], &[]);
        let a3 = m.rv(sp + 0xe0);
        let r = m.call(0x0025_8480, &[], &[s3, s2, s0, a3]).1;
        m.call(0x0025_6d18, &[sp + 0x10], &[r]);
        let q = m.rv(sp + 0x10);
        let f2 = q[0] + d * (q[1] + d * q[2]);
        let lo = f20 * m.rf(0x002b_dc70) + f21 * m.rf(0x002b_dc90);
        let hi = f20 * m.rf(0x002b_dc74) + f21 * m.rf(0x002b_dc94);
        let k = if f2 < lo { lo } else if hi < f2 { hi } else { f2 };
        off = scale(vel, k);
    } else {
        let xf0 = m.call(0x001a_6318, &[p], &[]).0;
        let xf1 = m.call(0x001a_6318, &[p], &[]).0;
        let (x, y) = (m.rf(xf1), m.rf(xf0 + 4));
        let a = m.call(0x0024_58f0, &[], &[f4(y), f4(x)]).1[0];
        let r = m.rf(0x002b_dc98);
        off = m.call(0x0011_e1a0, &[], &[f4(a), f4(r), f4(0.0)]).1;
    }
    let rel = sub(add(b, off), t);
    m.call(0x0011_efb0, &[sp + 0xf0, sp + 0xf4, sp + 0xf8], &[rel]);
    m.call(0x001a_e478, &[sp + 0x40], &[]);
    m.call(0x001b_44d8, &[sp + 0xfc, sp + 0x100], &[]);
    let mut far = m.rf(0x002a_513c);
    let mut best = 0.0;
    for i in 0..11u32 {
        let y = m.rf(sp + 0xf0) + m.rf(0x002b_dca0 + 4 * i);
        let f20 = m.call(0x0011_cf60, &[], &[f4(y)]).1[0];
        let (pi, r) = (m.rf(sp + 0xf4), m.rf(sp + 0xf8));
        let v = m.call(0x0011_e208, &[], &[f4(f20), f4(pi), f4(r)]).1;
        let c = add(t, v);
        let (a3, t0) = (m.r32(sp + 0xfc), m.r32(sp + 0x100));
        if m.call(0x0014_cc58, &[0, a3, t0, sp + 0x50], &[t, c]).0 == 0 {
            best = f20;
            break;
        }
        let h = len3(sub(t, m.rv(sp + 0x50)));
        if far < h {
            far = h;
            best = f20;
        }
    }
    let (pi, r) = (m.rf(sp + 0xf4), m.rf(sp + 0xf8));
    let v = m.call(0x0011_e208, &[], &[f4(best), f4(pi), f4(r)]).1;
    m.call(0x001a_e4a0, &[sp + 0x40, 2], &[]);
    stack_pop(m, 0x160);
    add(t, v)
}

/// FUN_00152c58(o): rhythm-game HUD draw (slot 18): text objects (FUN_00215ac0 / width FUN_00215ed8 / draw
/// FUN_002163b0 flags 0xc) on style sp+0x140 (FUN_00258438..78: 640 x 492.8, colour 0xff808080). For each
/// beat o+0x70+0x30*i (o+0x60 count; only while o+0x50 in 1..2): far-off beats draw text DAT_002bdb60 at the
/// beat point, near/active ones draw the 4 button glyphs ("&2?&." from DAT_002bdb70, matching one larger and
/// centred). Then the beat cursor o+0x190/+0x194 (0.4 s window DAT_002bdbb0, start DAT_002bdbc0) is
/// spring-moved (FUN_00117120) to the next beat and DAT_002bdb68 drawn at it.
pub fn ee_sub_152c58<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001a_5ed8, &[o], &[]);
    let now = m.call(0x001f_e3f8, &[0x003d_d040], &[]).1[0];
    let cnt = m.r32(o + 0x50) as i32;
    if !(cnt < 3) || cnt <= 0 || m.r32(o + 0x60) == 0 {
        return;
    }
    let sp = stack_push(m, 0x2c0);
    let (t0, t1, s, str_, t2, t3) = (sp, sp + 0xa0, sp + 0x140, sp + 0x160, sp + 0x170, sp + 0x210);
    let txt = |m: &mut M, t: u32, s_: u32, sz: u32| {
        let f = m.rf(sz);
        m.call(0x0021_5ac0, &[t, s_], &[f4(f)]);
    };
    let width = |m: &mut M, t: u32| m.call(0x0021_5ed8, &[t], &[]).1[0];
    txt(m, t0, 0x002b_db60, 0x002b_db58);
    txt(m, t1, 0x002b_db68, 0x002b_db5c);
    m.call(0x0025_8438, &[s], &[f4(0.0), f4(0.0)]);
    m.call(0x0025_8448, &[s], &[f4(640.0), f4(f32::from_bits(0x43f6_6667))]);
    m.w32(str_, 0xff80_8080);
    m.call(0x0025_8458, &[s, str_], &[]);
    m.call(0x0025_8470, &[s, 0], &[]);
    m.call(0x0025_8478, &[s, 0], &[]);
    for i in 0..6 {
        let b = m.r8(0x002a_5018 + i);
        m.w8(str_ + i, b);
    }
    txt(m, t2, str_, 0x002b_db50);
    let f26 = width(m, t2);
    let f25 = m.rf(sp + 0x1f4);
    txt(m, t3, str_, 0x002b_db54);
    let f24 = width(m, t3);
    let f23 = m.rf(sp + 0x294);
    let f29 = width(m, t0);
    let f28 = m.rf(sp + 0x84);
    let f31 = width(m, t1);
    let h1 = m.rf(sp + 0x124);
    let draw = |m: &mut M, t: u32, x: f32, y: f32| {
        m.wf(s, x);
        m.wf(s + 4, y);
        m.call(0x0021_63b0, &[t, s, 0xc], &[]);
    };
    let mut i = 0u32;
    while (i as i32) < m.r32(o + 0x60) as i32 {
        let s1 = o + 0x70 + 0x30 * i;
        i += 1;
        let far = cnt != 1 && m.r32(s1 + 0x20) == 0 && !(m.rf(s1) + m.rf(o + 0x5c) < now);
        if far {
            let (x, y) = (m.rf(s1 + 0x10) - f29 * 0.5, m.rf(s1 + 0x14) - f28);
            draw(m, t0, x, y);
            continue;
        }
        for k in 0..4u32 {
            let s0 = 0x002b_db70 + 0x10 * k;
            let mut x = m.rf(s1 + 0x10) + m.rf(s0 + 8) * f24;
            let mut y = m.rf(s1 + 0x14) + m.rf(s0 + 0xc) * f23;
            let mut sz = 0x002b_db54;
            let mut ch = 0x6f;
            if m.r32(s0) == m.r32(s1 + 4) {
                ch = m.r8(s0 + 4);
                sz = 0x002b_db50;
                x -= (f26 - f24) * 0.5;
                y -= (f25 - f23) * 0.5;
            }
            for (j, b) in [0x26u8, 0x32, ch, 0x26, 0x2e, 0].iter().enumerate() {
                m.w8(str_ + j as u32, *b);
            }
            txt(m, t2, str_, sz);
            draw(m, t2, x, y);
        }
    }
    let v1 = m.r32(o + 0x194);
    if (v1 as i32) < m.r32(o + 0x60) as i32 {
        let a1 = o + 0x70 + 0x30 * v1;
        match m.r32(o + 0x190) {
            0 => {
                if m.rf(a1) - now < m.rf(0x002b_dbb0) {
                    m.w32(o + 0x190, 1);
                    let v = m.rv(0x002b_dbc0);
                    m.wv(o + 0x1b0, v);
                }
            }
            1 => {
                if m.rf(a1) < now {
                    m.w32(o + 0x190, 0);
                    m.w32(o + 0x194, v1 + 1);
                }
            }
            _ => {}
        }
        if m.r32(o + 0x190) == 1 {
            let dt = m.rf(0x002d_ded4);
            let t = (m.rf(a1) - now).max(dt);
            let (p, v, g, k) = (m.rv(o + 0x1a0), m.rv(o + 0x1b0), m.rv(a1 + 0x10), m.rv(0x002b_dbd0));
            m.call(0x0011_7120, &[o + 0x1a0, o + 0x1b0, 0], &[f4(t), f4(dt), f4(1.0), p, v, g, k]);
        }
    }
    let x = m.rf(o + 0x1a0) - f31 * 0.5;
    let y = m.rf(o + 0x1a4) - f23 * 3.0 - h1 - 7.0;
    draw(m, t1, x, y);
    stack_pop(m, 0x2c0);
}

/// shared body of the script method thunks: unpack `n` args of `argv` by type list (FUN_00201360) into
/// out slots, reset/set the return value (FUN_00129ec8 / FUN_0012a0b0 kind 0x24 = void), call `f(obj, args)`,
/// copy it to `ret` (FUN_001291a8) and release the temp.
fn script_thunk<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, argv: u32, types: &[u32], f: u32) -> u32 {
    let n = types.len() as u32;
    let sp = stack_push(m, 0x40);
    let (tmp, ty, outs) = (sp + 0x10, sp, sp + 0x20);
    for (i, t) in types.iter().enumerate() {
        m.w32(ty + 4 * i as u32, *t);
    }
    let mut a = vec![tmp, n, argv, ty];
    a.extend((0..n).map(|i| outs + 4 * i));
    m.call(0x0020_1360, &a, &[]);
    m.call(0x0012_9ec8, &[tmp], &[]);
    m.call(0x0012_a0b0, &[tmp, 0x24], &[]);
    let fa: Vec<u32> = std::iter::once(obj).chain((0..n).map(|i| m.r32(outs + 4 * i))).collect();
    m.call(f, &fa, &[]);
    m.call(0x0012_91a8, &[ret, tmp], &[]);
    m.call(0x0012_9ec8, &[tmp], &[]);
    stack_pop(m, 0x40);
    ret
}

/// FUN_001097c0(ret, obj, _, argv): script thunk: 2 args (types DAT_002a3e38) -> FUN_001e1560(obj, a, b).
pub fn ee_sub_1097c0<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let t: Vec<u32> = (0..2).map(|i| m.r32(0x002a_3e38 + 4 * i)).collect();
    script_thunk(m, ret, obj, argv, &t, 0x001e_1560)
}

/// FUN_0010ccf8(ret, obj, _, argv): property setter thunk: 1 int arg (type 6) -> FUN_001bbc08(obj, a).
pub fn ee_sub_10ccf8<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    script_thunk(m, ret, obj, argv, &[6], 0x001b_bc08)
}

/// FUN_0010e008(ret, obj, _, argv): script thunk: 3 args (types DAT_002a3f78) -> FUN_0013c3b8(obj, a, b, c).
pub fn ee_sub_10e008<M: QMem + ?Sized>(m: &mut M, ret: u32, obj: u32, _a2: u32, argv: u32) -> u32 {
    let t: Vec<u32> = (0..3).map(|i| m.r32(0x002a_3f78 + 4 * i)).collect();
    script_thunk(m, ret, obj, argv, &t, 0x0013_c3b8)
}

/// FUN_001db870(o): ThiefNet shop screen per-frame input (pad o+0x10, FUN_00187d80 button masks 0x10
/// triangle / 0x40 cross / 0x80 square; page o+0x40: 0 buy, 1 sell; o+0x30 dialog). Buy: item o+0x254 (price
/// DAT_002bcd00[item].+8): dialog open -> FUN_001da950(0, 0x34) confirm or (1, 0x35) no coins; else pay
/// (FUN_00161798/FUN_001617e0 on 0x2deb30), FUN_001daf38(item, 1), refresh, sound 0x29, FUN_0019f0a8. Sell:
/// loot o+0x258 removed from the save variable (FUN_001d8a38 / FUN_001d8b20, count - 1) and its value
/// DAT_002c3288[loot*3] added to the coins. Invalid selection: sound 0x36. Triangle closes the dialog or the
/// screen (FUN_001dc260(-1)); square swaps pages (FUN_001dc260(page == 0)).
pub fn ee_sub_1db870<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.call(0x001d_a808, &[o], &[]);
    if m.r32(o + 0x40) == u32::MAX {
        return;
    }
    let pad = m.r32(o + 0x10);
    if pad == 0 {
        return;
    }
    let btn = |m: &mut M, b: u32| {
        let p = m.r32(o + 0x10);
        m.call(0x0018_7d80, &[p, b], &[]).0 != 0
    };
    let sound = |m: &mut M, id: u32| {
        let s = m.call(0x001f_bb90, &[id], &[]).0;
        m.call(0x001f_b2f8, &[s, 0], &[]);
    };
    if m.r32(o + 0x30) != 0 && btn(m, 0x10) {
        m.call(0x001d_c260, &[o, u32::MAX], &[]);
        return;
    }
    let coins = 0x002d_eb30;
    match m.r32(o + 0x40) {
        0 => {
            if btn(m, 0x40) {
                let item = m.r32(o + 0x254);
                if item == u32::MAX {
                    sound(m, 0x36);
                } else {
                    let price = m.r32(0x002b_cd00 + (item << 5) + 8) as i32;
                    let have = m.call(0x0016_1798, &[coins], &[]).0 as i32;
                    if m.r32(o + 0x30) == 0 {
                        m.call(0x001d_a950, &[o, u32::MAX, u32::MAX], &[]);
                        m.call(0x0016_17e0, &[coins, (have - price) as u32], &[]);
                        let it = m.r32(o + 0x254);
                        m.call(0x001d_af38, &[it, 1], &[]);
                        m.call(0x001d_bd10, &[o], &[]);
                        sound(m, 0x29);
                        m.call(0x0019_f0a8, &[0x003d_d490], &[]);
                    } else if have < price {
                        m.call(0x001d_a950, &[o, 1, 0x35], &[]);
                    } else {
                        m.call(0x001d_a950, &[o, 0, 0x34], &[]);
                    }
                }
            }
        }
        1 => {
            if btn(m, 0x40) {
                let loot = m.r32(o + 0x258);
                if loot == u32::MAX {
                    sound(m, 0x36);
                } else {
                    let sp = stack_push(m, 0x50);
                    m.w32(sp, u32::MAX);
                    m.call(0x0021_ecd8, &[sp + 0x10, 0x002a_79a0], &[]);
                    m.call(0x001d_8a38, &[sp + 0x30, 0x002a_79d0, 0x002a_79d8, sp + 0x10, 0], &[]);
                    m.call(0x0012_9658, &[sp, sp + 0x30], &[]);
                    m.call(0x0012_9ec8, &[sp + 0x30], &[]);
                    let n = m.r32(sp + 4) as i32;
                    if n > 0 {
                        m.w32(sp + 0x40, 0x15);
                        m.w32(sp + 0x44, (n - 1) as u32);
                        m.call(0x001d_8b20, &[sp + 0x30, sp + 0x40, 0x002a_79d0, 0x002a_79d8, sp + 0x10, 0], &[]);
                        m.call(0x0012_9ec8, &[sp + 0x30], &[]);
                        m.call(0x0012_9ec8, &[sp + 0x40], &[]);
                        let have = m.call(0x0016_1798, &[coins], &[]).0;
                        let l = m.r32(o + 0x258);
                        let val = m.r32(0x002c_3288 + l.wrapping_mul(12));
                        m.call(0x0016_17e0, &[coins, have.wrapping_add(val)], &[]);
                        m.call(0x001d_bd10, &[o], &[]);
                        sound(m, 0x29);
                    }
                    m.call(0x0012_9ec8, &[sp], &[]);
                    stack_pop(m, 0x50);
                }
            }
        }
        _ => {}
    }
    if m.r32(o + 0x30) == 0 && btn(m, 0x10) {
        m.call(0x001d_a950, &[o, u32::MAX, u32::MAX], &[]);
    }
    if btn(m, 0x80) {
        m.call(0x001d_a950, &[o, u32::MAX, u32::MAX], &[]);
        let pg = (m.r32(o + 0x40) == 0) as u32;
        m.call(0x001d_c260, &[o, pg], &[]);
    }
    if m.r32(o + 0x30) != 0 {
        m.call(0x001d_a858, &[o], &[]);
    }
    m.call(0x001d_bef0, &[o], &[]);
}

/// FUN_0015e650(a0..t1): forwards to FUN_0015e678 with t0 = 0 (t1 -> t2, t0 -> t1).
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_15e650<M: QMem + ?Sized>(m: &mut M, a0: u32, a1: u32, a2: u32, a3: u32, t0: u32, t1: u32) -> u32 {
    m.call(0x0015_e678, &[a0, a1, a2, a3, 0, t0, t1], &[]).0
}

/// FUN_001613a8(): FUN_001a2da0(FUN_0024a860()) after FUN_001a2c20(0); post event (0x2deb30, 0, 0)
/// via FUN_001619a0, then FUN_001a4e08(0x2e1e88).
pub fn ee_sub_1613a8<M: QMem + ?Sized>(m: &mut M) {
    m.call(0x001a_2c20, &[0], &[]);
    let v = m.call(0x0024_a860, &[], &[]).0;
    m.call(0x001a_2da0, &[v], &[]);
    m.call(0x0016_19a0, &[0x002d_eb30, 0, 0], &[]);
    m.call(0x001a_4e08, &[0x002e_1e88], &[]);
}

/// FUN_0013e510(src, parent): clone `src` (class from vtable +0x30 -> +4) into `parent` via
/// FUN_0013e3c8(cls, parent, src+0x18, -1), then new->vtable +0x140(new, src). Returns new or 0.
pub fn ee_sub_13e510<M: QMem + ?Sized>(m: &mut M, src: u32, parent: u32) -> u32 {
    if src == 0 {
        return 0;
    }
    let vt = m.r32(src + 0xc);
    let adj = m.r16(vt + 0x30) as i16 as i32;
    let f = m.r32(vt + 0x34);
    let cls = m.call(f, &[(src as i32 + adj) as u32], &[]).0;
    let c = m.r32(cls + 4);
    let n = m.r32(src + 0x18);
    let new = m.call(0x0013_e3c8, &[c, parent, n, u32::MAX], &[]).0;
    vcall(m, new, 0x140, &[src], &[]);
    new
}

/// FUN_001426e8(o, a1): find bone FUN_001f4c58(o, a1, 6); o+0x1150..0x1170 = rows of that bone's
/// rotation (+0x40) expressed in the frame of FUN_0011b340(o+0x60 -> +0x40) (3x3, w kept);
/// o+0x10f4 = FUN_0012dc18(o, bone, 3, 7, 2, FUN_0011bff0(o+0x1150)); FUN_0012d020(that).
pub fn ee_sub_1426e8<M: QMem + ?Sized>(m: &mut M, o: u32, a1: u32) {
    let sp = stack_push(m, 0xc0);
    let b = m.call(0x001f_4c58, &[o, a1, 6], &[]).0;
    if b != 0 {
        let src = m.r32(o + 0x60) + 0x40;
        m.call(0x0011_b340, &[sp + 0x30, src], &[]);
        let (r0, r1, r2) = (m.rv(sp + 0x30), m.rv(sp + 0x40), m.rv(sp + 0x50));
        for i in 0..3u32 {
            let v = m.rv(b + 0x40 + 0x10 * i);
            let mut o4 = v;
            for k in 0..3 {
                o4[k] = r0[k] * v[0] + r1[k] * v[1] + r2[k] * v[2];
            }
            m.wv(o + 0x1150 + 0x10 * i, o4);
        }
        let q = m.call(0x0011_bff0, &[o + 0x1150], &[]).1;
        let r = m.call(0x0012_dc18, &[o, b, 3, 7, 2], &[q]).0;
        m.w32(o + 0x10f4, r);
        m.call(0x0012_d020, &[r], &[]);
    }
    stack_pop(m, 0xc0);
}

/// FUN_0013e3c8(cls, parent, oid, a3): construct class `cls` via the class table DAT_002baae8
/// (stride 0x14, ctor +0x10), set +0x10 parent / +0x18 oid, register in the world (FUN_001ffa40 ->
/// FUN_00256ee0, oid/class lists FUN_0015e380 / FUN_0015e3a8 -> FUN_0011a070), vtable +0x38 init,
/// +0x58 when flag bit 60 of +0x28, +0xb8 when FUN_001a61a0; append to the world's pending list +0x39e4.
pub fn ee_sub_13e3c8<M: QMem + ?Sized>(m: &mut M, cls: u32, parent: u32, oid: u32, a3: u32) -> u32 {
    let w = 0x003d_d490;
    let ctor = m.r32(cls.wrapping_mul(0x14).wrapping_add(0x002b_aae8) + 0x10);
    let o = m.call(ctor, &[cls, parent], &[]).0;
    m.w32(o + 0x10, parent);
    m.w32(o + 0x18, oid);
    let v = m.call(0x001f_fa40, &[w, a3, o], &[]).0;
    m.call(0x0025_6ee0, &[o, v], &[]);
    let l = m.call(0x0015_e380, &[w, oid], &[]).0;
    m.call(0x0011_a070, &[l, o], &[]);
    let l = m.call(0x0015_e3a8, &[w, cls], &[]).0;
    m.call(0x0011_a070, &[l, o], &[]);
    vcall(m, o, 0x38, &[], &[]);
    if (m.r64(o + 0x28) >> 60) & 1 != 0 {
        vcall(m, o, 0x58, &[], &[]);
    }
    if m.call(0x001a_61a0, &[o], &[]).0 != 0 {
        vcall(m, o, 0xb8, &[], &[]);
    }
    let p = m.r32(w + 0x39e4);
    if p != 0 {
        m.w32(p, o);
        m.w32(w + 0x39e4, p + 4);
    }
    o
}

/// FUN_0013e690(out, kind, stream): read a typed value from `stream` into `out` by property kind:
/// 0 bool (0x17), 1 float (0x16), 2/9/10 vectors (FUN_001291e0 / FUN_00129330 3 floats, FUN_00129398 2),
/// 5/6 FUN_00117db8, 7/0xe..0x17 FUN_00117d50 (0x15), kind>>12 == 1 FUN_00117c80 (0x15), else FUN_0012a0b0(out, 0x24).
pub fn ee_sub_13e690<M: QMem + ?Sized>(m: &mut M, out: u32, kind: u32, s: u32) -> u32 {
    let sp = stack_push(m, 0x60);
    let int = |m: &mut M, tag: u32, v: u32| {
        m.w32(out + 4, v);
        m.w32(out, tag);
    };
    match kind {
        0 => {
            let v = (m.call(0x0011_7bd0, &[s], &[]).0 != 0) as u32;
            int(m, 0x17, v);
        }
        1 => {
            let v = m.call(0x0011_7e38, &[s], &[]).1[0];
            int(m, 0x16, v.to_bits());
        }
        2 | 9 | 10 => {
            let n = if kind == 10 { 2 } else { 3 };
            for i in 0..n {
                let v = m.call(0x0011_7e38, &[s], &[]).1[0];
                m.wf(sp + 4 * i, v);
            }
            let f = match kind {
                2 => 0x0012_91e0,
                9 => 0x0012_9330,
                _ => 0x0012_9398,
            };
            m.call(f, &[out, sp], &[]);
        }
        5 | 6 => {
            let v = m.call(0x0011_7db8, &[s], &[]).0;
            int(m, 0x15, v);
        }
        7 | 0xe..=0x11 | 0x13..=0x15 | 0x17 => {
            let v = m.call(0x0011_7d50, &[s], &[]).0;
            int(m, 0x15, v);
        }
        _ if ((kind & 0xffff_7fff) as i32) >> 12 == 1 => {
            let v = m.call(0x0011_7c80, &[s], &[]).0;
            int(m, 0x15, v);
        }
        _ => {
            m.call(0x0012_a0b0, &[out, 0x24], &[]);
        }
    }
    stack_pop(m, 0x60);
    out
}

/// FUN_0013d020(o, actor): pickup touch: only for actors of class 0x34. No actor+0x11a8 and no o+0x194
/// -> nothing. Locked (o+0x194 with DAT_003e0f9c and its +0x158 +0x10 == 0): sound FUN_001da4b8(0x3da0c0,
/// 0x124, 15.0) + hint FUN_001fb2f8(FUN_001fbb90(0x36), 0). Unless o+0x190 or the held item's kind matches
/// o+0x188 (else event 0x32 via FUN_001619a0): hint 0x27 when o+0x194, FUN_00143678(actor, 0, o), link
/// actor+0x11a8 +0x15c = o (FUN_0013c560 3) and o+0x194 +0x160 = actor (FUN_0013c560 2), FUN_00184130(o, actor).
pub fn ee_sub_13d020<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32) {
    if m.call(0x0013_2b10, &[a, 0x34], &[]).0 == 0 {
        return;
    }
    let held = m.r32(a + 0x11a8);
    let lk = m.r32(o + 0x194);
    if held == 0 && lk == 0 {
        return;
    }
    if lk != 0 && m.r32(0x003e_0f9c) != 0 && { let t = m.r32(lk + 0x158); m.r32(t + 0x10) == 0 } {
        m.call(0x001d_a4b8, &[0x003d_a0c0, 0x124], &[f4(15.0)]);
        let h = m.call(0x001f_bb90, &[0x36], &[]).0;
        m.call(0x001f_b2f8, &[h, 0], &[]);
        return;
    }
    if m.r32(o + 0x190) == 0 && held != 0 && { let t = m.r32(held + 0x158); m.r32(t + 0x14) != m.r32(o + 0x188) } {
        m.call(0x0016_19a0, &[0x002d_eb30, 0x32, o], &[]);
        return;
    }
    if m.r32(o + 0x194) != 0 {
        let h = m.call(0x001f_bb90, &[0x27], &[]).0;
        m.call(0x001f_b2f8, &[h, 0], &[]);
    }
    m.call(0x0014_3678, &[a, 0, o], &[]);
    let held = m.r32(a + 0x11a8);
    if held != 0 {
        m.w32(held + 0x15c, o);
        m.call(0x0013_c560, &[held, 3], &[]);
    }
    let lk = m.r32(o + 0x194);
    if lk != 0 {
        m.w32(lk + 0x160, a);
        m.call(0x0013_c560, &[lk, 2], &[]);
    }
    m.call(0x0018_4130, &[o, a], &[]);
}

/// FUN_0015e3e8(w, obj, key, flags, anchor, chain, max, count*, out[], best*): find-objects filter.
/// Unless flags & 0x400, the obj's 4-bit signed field (+0x28 bits 48..51) gates: 0 -> some chain node must
/// match `key` by FUN_001c9950; -1 -> `key` must be a chain node (flags 0x800 stops at the first miss).
/// Then by flags & 0xff: 1 nearest marked ancestor depth == 1, 2 parent == anchor, 3 depth >= 2, 4 keep the
/// shallowest marked ancestor depth (resets count), 5 any. 0x100 requires +0x28 bit 60. Appends to out (< max).
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_15e3e8<M: QMem + ?Sized>(
    m: &mut M, _w: u32, obj: u32, key: u32, flags: u32, anchor: u32, chain: u32, max: u32, cnt: u32, out: u32, best: u32,
) {
    let bit60 = |m: &mut M| (m.r64(obj + 0x28) >> 60) & 1 != 0;
    if flags & 0x400 == 0 {
        let k = ((m.r64(obj + 0x28) << 12) as i64) >> 60;
        let mut p = chain;
        if k == 0 {
            loop {
                if p == 0 {
                    return;
                }
                let a = m.r32(p);
                let a = m.call(0x001c_9950, &[a], &[]).0;
                let b = m.r32(key);
                if a == m.call(0x001c_9950, &[b], &[]).0 {
                    break;
                }
                if flags & 0x800 != 0 {
                    return;
                }
                p = m.r32(p + 4);
            }
        } else if k == -1 {
            while p != key {
                if p == 0 || flags & 0x800 != 0 {
                    return;
                }
                p = m.r32(p + 4);
            }
        }
    }
    let depth = |m: &mut M, o: u32| m.r8(o + 0x2c) as i8 as i32;
    match (flags & 0xff).wrapping_sub(1) {
        0 => {
            if obj == 0 {
                return;
            }
            let mut o = obj;
            let mut c = depth(m, o);
            while c == 0 {
                o = m.r32(o + 0x10);
                if o == 0 {
                    return;
                }
                c = depth(m, o);
            }
            if c != 1 {
                return;
            }
        }
        1 => {
            if m.r32(obj + 0x10) != anchor {
                return;
            }
        }
        2 => {
            if depth(m, obj) < 2 {
                return;
            }
        }
        3 => {
            let mut o = obj;
            let mut found = false;
            while obj != 0 && o != 0 {
                let c = depth(m, o);
                if c != 0 && c < m.r32(best) as i32 {
                    if flags & 0x100 != 0 && !bit60(m) {
                        return;
                    }
                    m.w32(best, c as u32);
                    m.w32(cnt, 0);
                    found = true;
                    break;
                }
                o = m.r32(o + 0x10);
            }
            if !found && m.r32(best) != 0x7fff_ffff {
                return;
            }
            if found {
                store_15e3e8(m, obj, max, cnt, out);
                return;
            }
        }
        4 => {}
        _ => return,
    }
    if flags & 0x100 != 0 && !bit60(m) {
        return;
    }
    store_15e3e8(m, obj, max, cnt, out);
}

fn store_15e3e8<M: QMem + ?Sized>(m: &mut M, obj: u32, max: u32, cnt: u32, out: u32) {
    let i = m.r32(cnt) as i32;
    if i < max as i32 {
        m.w32(out + 4 * i as u32, obj);
    }
    m.w32(cnt, (i + 1) as u32);
}

/// FUN_0015e678(w, flags, key, anchor, chain, max, out): collect objects with oid `key` (oid hash list
/// FUN_0015e380 by +0x18/+0x1c, then alias list FUN_0015e3c0 by +8/+0xc skipping already-found objects)
/// through FUN_0015e3e8. Kinds 1/3/4 temporarily number the anchor's ancestor chain (+0x2c depth).
/// Returns the count (clamped to max unless flags & 0x200).
#[allow(clippy::too_many_arguments)]
pub fn ee_sub_15e678<M: QMem + ?Sized>(m: &mut M, w: u32, flags: u32, key: u32, anchor: u32, chain: u32, max: u32, out: u32) -> u32 {
    let kind = flags & 0xff;
    if key == u32::MAX || (anchor == 0 && kind != 5) {
        return 0;
    }
    let mut chain = chain;
    if chain == 0 {
        chain = m.r32(0x003e_139c);
        if anchor != 0 {
            chain = m.r32(anchor + 0x3c);
        }
    }
    let ctx = stack_push(m, 0x10);
    m.w32(ctx, 0);
    let mut mark = 0;
    if (kind == 1 || kind == 3 || kind == 4) && anchor != 0 {
        let (mut o, mut c) = (anchor, 1u8);
        while o != 0 {
            m.w8(o + 0x2c, c);
            o = m.r32(o + 0x10);
            c = c.wrapping_add(1);
        }
        m.w32(ctx + 4, 0x7fff_ffff);
        mark = anchor;
    } else if kind == 1 || kind == 3 || kind == 4 {
        m.w32(ctx + 4, 0x7fff_ffff);
    }
    m.call(0x0015_e380, &[w, key], &[]);
    let l = m.call(0x0015_e380, &[w, key], &[]).0;
    let mut o = m.r32(l);
    while o != 0 {
        if m.r32(o + 0x18) == key {
            let k = m.r32(o + 0x3c);
            ee_sub_15e3e8(m, w, o, k, flags, anchor, chain, max, ctx, out, ctx + 4);
        }
        o = m.r32(o + 0x1c);
    }
    m.call(0x0015_e3c0, &[w, key], &[]);
    let l = m.call(0x0015_e3c0, &[w, key], &[]).0;
    let mut n = m.r32(l);
    while n != 0 {
        if m.r32(n + 8) == key {
            let obj = m.r32(n + 0x10);
            if obj != 0 {
                let c = m.r32(ctx) as i32;
                let mut dup = false;
                if m.r32(obj + 0x18) == key && c > 0 {
                    for i in 0..c as u32 {
                        if m.r32(out + 4 * i) == obj {
                            dup = true;
                            break;
                        }
                    }
                }
                if !dup {
                    let k = m.r32(n + 4);
                    ee_sub_15e3e8(m, w, obj, k, flags, anchor, chain, max, ctx, out, ctx + 4);
                }
            }
        }
        n = m.r32(n + 0xc);
    }
    let mut o = mark;
    while o != 0 {
        m.w8(o + 0x2c, 0);
        o = m.r32(o + 0x10);
    }
    let mut c = m.r32(ctx);
    if flags & 0x200 == 0 && (max as i32) < c as i32 {
        c = max;
    }
    stack_pop(m, 0x10);
    c
}

/// FUN_0015e9c8(w, flags, key, anchor, max, out): find objects by class/oid `key` (FUN_00132b10 /
/// FUN_0015e988 ranges < 199 over class lists FUN_0015e3a8): kind 3 ancestors of anchor, kind 2 children
/// (+0x64 / +0x24, up to +0x70 with 0x100), kind 5 everything, others through FUN_0015e3e8 (flags | 0x400)
/// with the anchor chain numbered. 0x100 requires +0x28 bit 60. Returns count (clamped unless 0x200).
pub fn ee_sub_15e9c8<M: QMem + ?Sized>(m: &mut M, w: u32, flags: u32, key: u32, anchor: u32, max: u32, out: u32) -> u32 {
    let kind = flags & 0xff;
    if key == u32::MAX || (anchor == 0 && kind != 5) {
        return 0;
    }
    let ctx = stack_push(m, 0x10);
    m.w32(ctx, 0);
    let ok = |m: &mut M, o: u32| flags & 0x100 == 0 || (m.r64(o + 0x28) >> 60) & 1 != 0;
    let push = |m: &mut M, o: u32| store_15e3e8(m, o, max, ctx, out);
    let next_key = |m: &mut M, k: u32| (k as i32) < 199 && m.call(0x0015_e988, &[k, key], &[]).0 != 0;
    match kind {
        3 => {
            let mut o = m.r32(anchor + 0x10);
            while o != 0 {
                if m.call(0x0013_2b10, &[o, key], &[]).0 != 0 && ok(m, o) {
                    push(m, o);
                }
                o = m.r32(o + 0x10);
            }
        }
        2 => {
            let end = if flags & 0x100 != 0 { m.r32(anchor + 0x70) } else { 0 };
            let mut o = m.r32(anchor + 0x64);
            while o != end {
                if m.call(0x0013_2b10, &[o, key], &[]).0 != 0 {
                    push(m, o);
                }
                o = m.r32(o + 0x24);
            }
        }
        5 => {
            let mut k = key;
            while next_key(m, k) {
                m.call(0x0015_e3a8, &[w, k], &[]);
                let l = m.call(0x0015_e3a8, &[w, k], &[]).0;
                k += 1;
                let mut o = m.r32(l);
                while o != 0 {
                    if ok(m, o) {
                        push(m, o);
                    }
                    o = m.r32(o + 0x20);
                }
            }
        }
        _ => {
            let mut mark = 0;
            if kind == 1 || kind == 4 {
                let (mut o, mut c) = (anchor, 1u8);
                while o != 0 {
                    m.w8(o + 0x2c, c);
                    o = m.r32(o + 0x10);
                    c = c.wrapping_add(1);
                }
                m.w32(ctx + 4, 0x7fff_ffff);
                mark = anchor;
            }
            let mut k = key;
            while next_key(m, k) {
                m.call(0x0015_e3a8, &[w, k], &[]);
                let l = m.call(0x0015_e3a8, &[w, k], &[]).0;
                k += 1;
                let mut o = m.r32(l);
                while o != 0 {
                    ee_sub_15e3e8(m, w, o, 0, flags | 0x400, anchor, 0, max, ctx, out, ctx + 4);
                    o = m.r32(o + 0x20);
                }
            }
            let mut o = mark;
            while o != 0 {
                m.w8(o + 0x2c, 0);
                o = m.r32(o + 0x10);
            }
        }
    }
    let mut c = m.r32(ctx);
    if flags & 0x200 == 0 && (max as i32) < c as i32 {
        c = max;
    }
    stack_pop(m, 0x10);
    c
}

/// FUN_001400b8(o): Sly/player-actor post-load: base init (FUN_00203a38, FUN_001a64e8 kind 0x35), sub-object
/// inits at +0x2c0/+0x4c0/+0x9e0/+0xdf0/+0xf00/+0x11b0/+0xf50; gather up to 16 oid-0x2ab children
/// (FUN_0015e650 0x401) into an 8-byte-stride array +0x1090 (count +0x1094, FUN_001ae2a0) and init each
/// (vtable +0x48); vtable +0x278(0), +0x340; FUN_00141ef8, FUN_001440d0(DAT_003e1084 & 4), +0x1088 =
/// FUN_001f8278, FUN_001f35c8(0.25). If (+0x54)+0x94 == 0: reset that matrix to identity, bump DAT_003e11e8
/// into +0xa4, re-apply pos (vtable +0xe8) and xf (+0xf0); finally FUN_00149a50(world, o, DAT_002a4d10, 0).
pub fn ee_sub_1400b8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let w = 0x003d_d490;
    let sp = stack_push(m, 0xf0);
    m.call(0x0020_3a38, &[o], &[]);
    m.call(0x001a_64e8, &[o, 0x35, 0x002b_c9f0], &[]);
    for (f, off) in [
        (0x0019_59c0, 0x2c0),
        (0x0019_10f8, 0x4c0),
        (0x0019_f3f0, 0x9e0),
        (0x001a_1098, 0xdf0),
        (0x0019_cc30, 0xf00),
        (0x0021_f388, 0x11b0),
        (0x0019_de58, 0xf50),
    ] {
        m.call(f, &[o + off], &[]);
    }
    let n = ee_sub_15e650(m, w, 0x401, 0x2ab, o, 0x10, sp);
    if n != 0 {
        m.w32(o + 0x1094, n);
        m.w32(sp + 0x40, n << 3);
        m.w32(sp + 0x44, 0x10);
        let arr = m.call(0x001a_e2a0, &[sp + 0x40], &[]).0;
        m.w32(o + 0x1090, arr);
        for i in 0..n {
            let base = m.r32(o + 0x1090);
            let c = m.r32(sp + 4 * i);
            m.w32(base + 8 * i, c);
            vcall(m, c, 0x48, &[], &[]);
        }
    }
    vcall(m, o, 0x278, &[0], &[]);
    vcall(m, o, 0x340, &[], &[]);
    m.call(0x0014_1ef8, &[o], &[]);
    let fl = m.r32(w + 0x3bf4) & 4;
    m.call(0x0014_40d0, &[o, fl], &[]);
    let r = m.call(0x001f_8278, &[o], &[]).0;
    m.w32(o + 0x1088, r);
    m.call(0x001f_35c8, &[o], &[f4(0.25)]);
    let mt0 = m.r32(o + 0x54);
    if m.r32(mt0 + 0x94) == 0 {
        let pos = m.call(0x001a_62e8, &[o], &[]).1;
        let xf = m.call(0x001a_6318, &[o], &[]).0;
        for i in 0..3 {
            let v = m.rv(xf + 0x10 * i);
            m.wv(sp + 0x50 + 0x10 * i, v);
        }
        let mt = m.r32(o + 0x54);
        m.call(0x0023_33b4, &[mt, 0, 0x40], &[]);
        for i in [0u32, 5, 10, 15] {
            m.wf(mt + 4 * i, 1.0);
        }
        let s = m.r32(0x003e_11e8).wrapping_add(1);
        m.w32(0x003e_11e8, s);
        let mt = m.r32(o + 0x54);
        m.w32(mt + 0xa4, s);
        vcall(m, o, 0x90, &[], &[]);
        vcall(m, o, 0xe8, &[], &[pos]);
        vcall(m, o, 0xf0, &[sp + 0x50], &[]);
    }
    let k = m.r32(0x002a_4d10);
    m.call(0x0014_9a50, &[w, o, k, 0], &[]);
    stack_pop(m, 0xf0);
}


/// FUN_001477d8(o): draw the two-sided overlay sprite set (state +0x150 1..3; 1 first runs FUN_001475c0 +
/// FUN_00147410(o, 2)) into the GIF packet 0x2dea00: setup regs (ALPHA 0x47, TEX0 6 from DAT_002dece0 /
/// DAT_002decd0, 0x14, 0x3f, 8), the bone matrices of +0x158/+0x15c through FUN_0022ff40(o+0x60), then up
/// to 3 strips (gated by +0x164/+0x168/+0x16c) of 4 vertices: corner DAT_002bd3b0[j] -> FUN_0011c1a8 by the
/// bone matrix and the view DAT_002bd410, perspective divide, screen xy (x+.5)*512, (y+.5)*224 + 0.5,
/// STQ = UV DAT_002bd3f0[j] + DAT_002bd398[side] times 1/w, RGBA 0x80808080, XYZ ((x+0x700)<<4, (y+0x790)<<4).
pub fn ee_sub_1477d8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    const G: u32 = 0x002d_ea00;
    let st = m.r32(o + 0x150) as i32;
    if st == 1 {
        m.call(0x0014_75c0, &[o], &[]);
        m.call(0x0014_7410, &[o, 2], &[]);
    } else if !(1..=3).contains(&st) {
        return;
    }
    let d = m.r32(0x002d_ea4c);
    let f = m.r32(d + 0x30) & !1;
    m.w32(d + 0x30, f);
    if m.r32(d + 0x24) == 0 {
        let v = m.r32(0x002d_ea34);
        m.w32(d + 0x24, v);
    }
    m.call(0x0015_76f0, &[G], &[]);
    m.call(0x0015_7500, &[G, 0x25, 0x002d_f4b0], &[]);
    m.call(0x0015_74b8, &[G], &[]);
    m.call(0x0016_6bd8, &[G, 0, 1, 0xe], &[]);
    let reg = |m: &mut M, r: u32, v: u64| {
        crate::engine::game::q_00166008::ee_sub_166e90(m, G, r as u64, v); // VER3: full 64-bit data
    };
    reg(m, 0x47, 0x31001);
    let tex = ((m.r64(0x002d_ece0) & 0x1ff) << 5) | (((m.r64(0x002d_ecd0) & 0xffff_ffff) >> 16 & 0x3f) << 14) | 0x6_a800_0000;
    reg(m, 6, tex);
    reg(m, 0x14, 0x61);
    reg(m, 0x3f, 0);
    reg(m, 8, 0x380_0080_000a);
    let sp = stack_push(m, 0x100);
    let tmp = sp + 0x80;
    for i in 0..2u32 {
        let b = m.r32(o + 0x158 + 4 * i);
        let a = m.r32(o + 0x60);
        m.call(0x0022_ff40, &[tmp, 0, a, b + 0x40], &[]);
        for k in 0..4 {
            let v = m.rv(tmp + 0x10 * k);
            m.wv(sp + 0x40 * i + 0x10 * k, v);
        }
    }
    let mut k = 0u32;
    while k < 3 {
        let draw = match k {
            0 => m.r32(o + 0x164) != 0 && m.r32(o + 0x16c) != 0,
            1 => m.r32(o + 0x168) != 0,
            _ => m.r32(o + 0x164) != 0 && m.r32(o + 0x16c) == 0,
        };
        if draw {
            m.call(0x0016_6bd8, &[G, 0x14, 3, 0x512], &[]);
            let side = k & 1;
            for j in 0..4u32 {
                let c = m.rv(0x002b_d3b0 + 0x10 * j);
                let v = m.call(0x0011_c1a8, &[sp + (side << 6)], &[c]).1;
                let v = m.call(0x0011_c1a8, &[0x002b_d410], &[v]).1;
                m.wv(tmp, v);
                m.call(0x0011_d9d8, &[tmp], &[]);
                let w = crate::engine::math::ps2f::ps2_div(1.0, v[3]); // DIV.S: x/0 = max float
                m.wf(sp + 0xc0, w);
                m.wv(sp + 0x90, [w, 0.0, 0.0, 0.0]);
                m.wv(tmp, v.map(|e| crate::engine::math::ps2f::ps2_mul(e, w)));
                m.call(0x0011_d9d8, &[sp + 0xc0], &[]);
                m.call(0x0011_d9d8, &[tmp], &[]);
                let x = (m.rf(tmp) + 0.5) * 512.0 + 0.5;
                let y = (m.rf(tmp + 4) + 0.5) * 224.0 + 0.5;
                m.wf(tmp, x);
                m.wf(tmp + 4, y);
                m.call(0x0011_d9d8, &[tmp], &[]);
                let s = crate::engine::math::ps2f::ps2_mul(w, m.rf(0x002b_d3f0 + 8 * j) + m.rf(0x002b_d398 + 8 * side));
                let t = crate::engine::math::ps2f::ps2_mul(w, m.rf(0x002b_d3f4 + 8 * j) + m.rf(0x002b_d39c + 8 * side));
                m.call(0x0016_6d38, &[G], &[f4(s), f4(t), f4(w)]);
                m.call(0x0016_6c70, &[G, 0x8080_8080], &[]);
                let (xi, yi) = (x as i32, y as i32);
                m.call(0x0016_6da0, &[G, ((xi + 0x700) << 4) as u32, ((yi + 0x790) << 4) as u32, 0], &[]);
            }
        }
        k += 1;
    }
    stack_pop(m, 0x100);
}

/// FUN_00147f40(o): proximity-list update. Camera DAT_002ddfd0 and focus (FUN_001c4840 actor pos, else the
/// camera) are compared with the cached +0x39b0/+0x39c0; moving more than DAT_002bd450 or a new mask
/// DAT_002ddf54 resets the cursor +0x39a4. Walks list +0x20 (link offset +0x28, safe-iterator frame pushed on
/// DAT_002dd2fc) up to the cursor: kind (+0x28 bits 52..55) 1/2 -> FUN_00147ec8, 0 (camera) / 3 (focus)
/// distance vs radius +0x38 with mask byte +0x2d -> inside flag, FUN_00147ee8 re-sort; inside-set +0x2c
/// membership (FUN_0011a380) toggled with vtable +0xb8 enter / +0xb0 leave (FUN_0011a220 / FUN_0011a2e8).
pub fn ee_sub_147f40<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let lim = m.rf(0x002b_d450);
    let mask = m.r32(0x002d_df54);
    let cam = m.rv(0x002d_dfd0);
    let a = m.call(0x001c_4840, &[], &[]).0;
    let foc = if a == 0 { cam } else { m.call(0x001a_62e8, &[a], &[]).1 };
    let dist = |p: V4, q: V4| {
        let d = sub(p, q);
        dot3(d, d).sqrt()
    };
    let dc = dist(cam, m.rv(o + 0x39b0));
    let df = dist(foc, m.rv(o + 0x39c0));
    if !(dc <= lim && df <= lim && mask == m.r32(o + 0x39a8)) {
        m.w32(o + 0x39a4, 0);
        m.w32(o + 0x39a8, mask);
        m.wv(o + 0x39b0, cam);
        m.wv(o + 0x39c0, foc);
    }
    let fr = stack_push(m, 0x10);
    let off = m.r32(o + 0x28);
    let mut n = m.r32(o + 0x20);
    m.w32(fr, o + 0x20);
    m.w32(fr + 4, n.wrapping_add(off));
    m.w32(fr + 8, off);
    let prev = m.r32(0x002d_d2fc);
    m.w32(fr + 0xc, prev);
    m.w32(0x002d_d2fc, fr);
    let mut inside = 0u32;
    while n != m.r32(o + 0x39a4) {
        let it = m.r32(n);
        let kind = ((m.r64(it + 0x28) << 8) as i64) >> 60;
        let near = |m: &mut M, c: V4, r: f32| -> Option<u32> {
            let p = m.call(0x001a_62e8, &[it], &[]).1;
            let f = dist(p, c) - m.rf(it + 0x38);
            Some(((f < 0.0) as u32) | (((f.abs() - r) > lim) as u32) << 1)
        };
        match kind {
            1 | 2 => {
                inside = (kind == 1) as u32;
                m.call(0x0014_7ec8, &[o, n], &[]);
            }
            0 => {
                if mask & m.r8(it + 0x2d) as u32 == 0 {
                    inside = 0;
                    m.call(0x0014_7ee8, &[o, n], &[]);
                } else {
                    let r = near(m, cam, dc).unwrap();
                    inside = r & 1;
                    if r & 2 != 0 {
                        m.call(0x0014_7ee8, &[o, n], &[]);
                    }
                }
            }
            3 => {
                if mask & m.r8(it + 0x2d) as u32 == 0 {
                    inside = 0;
                } else {
                    let r = near(m, foc, df).unwrap();
                    inside = r & 1;
                    if r & 2 != 0 {
                        m.call(0x0014_7ee8, &[o, n], &[]);
                    }
                }
            }
            _ => {}
        }
        let set = o + 0x2c;
        let was = m.call(0x0011_a380, &[set, n], &[]).0;
        if inside != was {
            if inside == 0 {
                vcall(m, it, 0xb0, &[], &[]);
                m.call(0x0011_a2e8, &[set, n], &[]);
            } else {
                m.call(0x0011_a220, &[set, n], &[]);
                vcall(m, it, 0xb8, &[], &[]);
            }
        }
        let link = m.r32(fr + 4);
        n = m.r32(link);
        let off = m.r32(fr + 8);
        m.w32(fr + 4, n.wrapping_add(off));
    }
    m.w32(0x002d_d2fc, prev);
    stack_pop(m, 0x10);
}

/// FUN_00161a58(o, ev): level/game event handler. 1: when the save checksum pair matches
/// (DAT_003e1090 ^ DAT_003e10c0 == DAT_002a37f0 ^ DAT_002a3820) re-apply held loot of up to 3 class-0x34
/// actors (FUN_0013c478 -> FUN_00143af0). 3: FUN_001613f8 (flag 1) / FUN_00166008 (flag 0x40), FUN_001ab630,
/// then (no FUN_00210ae0, DAT_002e65e8 == 6, no pending) re-check the stored level var DAT_002f6334 ->
/// FUN_001d8ff8. 4: FUN_00165280. 5: load-time apply: resolve +0x10/+0x14/+0x18 objects into
/// FUN_00165dc0/de8/e10, FUN_001629a0 / FUN_00163d18 / FUN_001657a8, wrapping the FUN_00165c98 list with
/// FUN_00164270 / FUN_001643b0 (item +0xf4/+0xf8 forced then restored). 0x10: FUN_0022b490 respawn request.
pub fn ee_sub_161a58<M: QMem + ?Sized>(m: &mut M, o: u32, ev: u32) {
    const W: u32 = 0x003d_d490;
    let sp = stack_push(m, 0xa0);
    match ev {
        1 => {
            let a = m.r32(0x003e_1090) ^ m.r32(0x003e_10c0);
            let b = m.r32(0x002a_37f0) ^ m.r32(0x002a_3820);
            if a == b {
                let n = ee_sub_15e9c8(m, W, 5, 0x34, 0, 3, sp) as i32;
                for i in 0..n.max(0) as u32 {
                    let a = m.r32(sp + 4 * i);
                    let h = m.r32(a + 0x11a8);
                    if h != 0 {
                        let l = m.call(0x0013_c478, &[h], &[]).0;
                        m.call(0x0014_3af0, &[a, l], &[]);
                    }
                }
            }
        }
        3 => 'c: {
            let fl = m.r32(W + 0x3bf4);
            if fl & 1 != 0 {
                m.call(0x0016_13f8, &[o], &[]);
                m.w32(0x002f_6330, 0);
            } else if fl & 0x40 != 0 {
                m.call(0x0016_6008, &[], &[]);
            }
            m.call(0x001a_b630, &[0x002e_65e0], &[]);
            if m.call(0x0021_0ae0, &[], &[]).0 != 0 || m.r32(0x002e_65e8) != 6 || m.r32(0x002f_6330) != 0 {
                break 'c;
            }
            let v = m.r32(0x002f_6334);
            if v == 0 {
                break 'c;
            }
            let src = m.r32(W + 0x3b1c);
            m.call(0x0025_81c0, &[sp + 0x10, src, 0x003d_4a60], &[]);
            let f = m.rf(sp + 0x14);
            m.call(0x0012_9ec8, &[sp + 0x10], &[]);
            if f == 0.0 {
                let v = m.r32(0x002f_6334);
                m.call(0x0025_81c0, &[sp, src, v], &[]);
                let f = m.rf(sp + 4);
                m.call(0x0012_9ec8, &[sp], &[]);
                if f != 0.0 {
                    let v = m.r32(0x002f_6334);
                    m.call(0x001d_8ff8, &[v], &[]);
                }
            }
        }
        4 => {
            m.call(0x0016_5280, &[o], &[]);
        }
        5 => {
            m.call(0x001a_e478, &[sp + 0x20], &[]);
            m.call(0x0016_5c98, &[sp + 0x24, sp + 0x28], &[]);
            let wrap = |m: &mut M, f: u32, en: u32| {
                let n = m.r32(sp + 0x24) as i32;
                for i in 0..n.max(0) as u32 {
                    let arr = m.r32(sp + 0x28);
                    let it = m.r32(arr + 4 * i);
                    let (mut s4, mut s8) = (0, 0);
                    if it != 0 {
                        s4 = m.r32(it + 0xf4);
                        s8 = m.r32(it + 0xf8);
                        m.w32(it + 0xf4, en);
                        m.w32(it + 0xf8, 0);
                    }
                    m.call(f, &[it], &[]);
                    if it != 0 {
                        m.w32(it + 0xf4, s4);
                        m.w32(it + 0xf8, s8);
                    }
                }
            };
            wrap(m, 0x0016_4270, 1);
            for (off, f) in [(0x10, 0x0016_5dc0), (0x14, 0x0016_5de8), (0x18, 0x0016_5e10)] {
                let id = m.r32(o + off);
                let x = m.call(0x0015_e938, &[W, 0x105, id, 0], &[]).0;
                m.call(f, &[W, x], &[]);
            }
            let a = m.r32(W + 0x3b0c);
            if a != 0 {
                m.call(0x0016_29a0, &[a], &[]);
            }
            let b = m.r32(W + 0x3b10);
            if b != 0 {
                m.call(0x0016_3d18, &[b], &[]);
            }
            if m.r32(W + 0x3bf4) & 4 != 0 {
                let c = m.r32(W + 0x3b14);
                if c != 0 {
                    m.call(0x0016_57a8, &[c], &[]);
                } else if m.r32(W + 0x3b0c) != 0 {
                    let a = m.r32(W + 0x3b0c);
                    let x = m.r32(a + 0x64);
                    m.call(0x0016_57a8, &[x], &[]);
                }
            }
            wrap(m, 0x0016_43b0, 0);
            m.call(0x001a_e4a0, &[sp + 0x20, 2], &[]);
        }
        0x10 => {
            if m.r32(W + 0x3b0c) == 0 {
                let t0 = if m.r32(W + 0x3e38) != 0 { 0x20 } else { 0 };
                let (a2, a3) = (m.r32(W + 0x3c88), m.r32(W + 0x3c8c));
                m.call(0x0022_b490, &[W, 0, a2, a3, t0, 0x30a], &[]);
            }
        }
        _ => {}
    }
    stack_pop(m, 0xa0);
}

/// FUN_00143678(a, kind, o): loot save-variable sync for actor `a` (+0x11a8 held loot, +0x18 oid).
/// kind 0 (pick up from `o`): set the actor's held var (vtable +0x338 path, DAT_002a4db0) to o+0x194's id
/// (FUN_0013c478) or -1, clear/assign the source slot var (FUN_0013d1a8(o) <- held id or the actor oid) and,
/// when o has no lock, clear the actor slot (FUN_00143410). kind 1 (drop): clear both and bump the counter
/// var (DAT_002a4dc0/4dc8 under the held loot's name DAT_002a4d50). kind 2 (restore): re-resolve the held
/// loot (FUN_00143af0) and, when the current level name matches, relink +0x15c to the class-0x25 pedestal
/// whose +0x180 equals the loot's +0xc id.
pub fn ee_sub_143678<M: QMem + ?Sized>(m: &mut M, a: u32, kind: u32, o: u32) {
    const P: u32 = 0x003d_4a60;
    let sp = stack_push(m, 0xf0);
    let val = |m: &mut M, at: u32, v: u32| {
        m.w32(at, 0x15);
        m.w32(at + 4, v);
    };
    let ec8 = |m: &mut M, at: u32| {
        m.call(0x0012_9ec8, &[at], &[]);
    };
    // set var `name` := (0x15, v) through FUN_001291a8 / FUN_00258200 with temporaries at t, t+0x10, t+0x20
    let setv = |m: &mut M, name: u32, v: u32, t: u32| {
        val(m, t, v);
        m.call(0x0012_91a8, &[t + 0x10, t], &[]);
        m.call(0x0025_8200, &[t + 0x20, name, t + 0x10, P], &[]);
        ec8(m, t + 0x20);
        ec8(m, t + 0x10);
        ec8(m, t);
    };
    m.call(0x001a_e478, &[sp], &[]);
    match kind {
        0 => {
            let lk = m.r32(o + 0x194);
            let v = if lk != 0 { m.call(0x0013_c478, &[lk], &[]).0 } else { u32::MAX };
            val(m, sp + 0x20, v);
            let path = vcall(m, a, 0x338, &[], &[]).0;
            m.call(0x001d_8b20, &[sp + 0x10, sp + 0x20, path, 0x002a_4db0, 0], &[]);
            ec8(m, sp + 0x20);
            let held = m.r32(a + 0x11a8);
            if held == 0 {
                let n = m.call(0x0013_d1a8, &[o], &[]).0;
                let id = m.r32(a + 0x18);
                setv(m, n, id, sp + 0x20);
            } else {
                if m.r32(o + 0x194) == 0 {
                    let id = m.r32(a + 0x18);
                    let n = m.call(0x0014_3410, &[id], &[]).0;
                    setv(m, n, u32::MAX, sp + 0x30);
                }
                let n = m.call(0x0013_d1a8, &[o], &[]).0;
                let h = m.r32(a + 0x11a8);
                let v = m.call(0x0013_c478, &[h], &[]).0;
                setv(m, n, v, sp + 0x20);
            }
            ec8(m, sp + 0x10);
        }
        1 => {
            let id = m.r32(a + 0x18);
            let n = m.call(0x0014_3410, &[id], &[]).0;
            if n != 0 {
                setv(m, n, u32::MAX, sp + 0x10);
            }
            val(m, sp + 0x20, u32::MAX);
            let path = vcall(m, a, 0x338, &[], &[]).0;
            m.call(0x001d_8b20, &[sp + 0x10, sp + 0x20, path, 0x002a_4db0, 0], &[]);
            ec8(m, sp + 0x20);
            let h = m.r32(a + 0x11a8);
            let lp = m.r32(h + 0x158);
            let nm = m.r32(lp + 4);
            if nm != u32::MAX {
                m.call(0x0021_ecd8, &[sp + 0x20, 0x002a_4d50, nm], &[]);
                m.call(0x001d_8a38, &[sp + 0x40, 0x002a_4dc0, 0x002a_4dc8, sp + 0x20, 0], &[]);
                m.call(0x0012_9658, &[sp + 0x10, sp + 0x40], &[]);
                ec8(m, sp + 0x40);
                let c = m.r32(sp + 0x14).wrapping_add(1);
                val(m, sp + 0x60, c);
                m.call(0x001d_8b20, &[sp + 0x40, sp + 0x60, 0x002a_4dc0, 0x002a_4dc8, sp + 0x20, 0], &[]);
                m.call(0x0012_9658, &[sp + 0x10, sp + 0x40], &[]);
                ec8(m, sp + 0x40);
                ec8(m, sp + 0x60);
            }
            ec8(m, sp + 0x10);
        }
        2 => {
            let h = m.r32(a + 0x11a8);
            let l = m.call(0x0013_c478, &[h], &[]).0;
            let r = m.call(0x0014_3af0, &[a, l], &[]).0;
            let x = m.call(0x0021_0a88, &[], &[]).0;
            if r != 0 && x != u32::MAX {
                let v = m.call(0x0021_09b8, &[x], &[]).0;
                if m.r32(v) as i32 >= 4 {
                    let (s, t) = (m.r32(r + 4), m.r32(v + 4));
                    if m.call(0x0023_3178, &[s, t, 8], &[]).0 == 0 {
                        let n = ee_sub_15e9c8(m, 0x003d_d490, 5, 0x25, 0, 0xf, sp + 0x10) as i32;
                        for i in 0..n.max(0) as u32 {
                            let p = m.r32(sp + 0x10 + 4 * i);
                            let rr = m.r32(r + 4);
                            if m.r32(p + 0x180) == m.r16(rr + 0xc) as u32 {
                                let h = m.r32(a + 0x11a8);
                                m.w32(h + 0x15c, p);
                                break;
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    m.call(0x001a_e4a0, &[sp, 2], &[]);
    stack_pop(m, 0xf0);
}

/// FUN_0013c3b8(o, a, b, c): FUN_0013c440(o), then append a 0x2c-byte entry (max 0x20) to o+0x154
/// (count o+0x150), current -> o+0x158: {a, b, .., +0x14 c, +0x18 -1, +0x1c 0}.
pub fn ee_sub_13c3b8<M: QMem + ?Sized>(m: &mut M, o: u32, a: u32, b: u32, c: u32) {
    m.call(0x0013_c440, &[o], &[]);
    let n = m.r32(o + 0x150);
    if (n as i32) < 0x20 {
        let e = m.r32(o + 0x154).wrapping_add(n.wrapping_mul(0x2c));
        m.w32(o + 0x150, n + 1);
        m.w32(o + 0x158, e);
        m.w32(e, a);
        m.w32(e + 4, b);
        m.w32(e + 0x14, c);
        m.w32(e + 0x18, u32::MAX);
        m.w32(e + 0x1c, 0);
    }
}

/// FUN_0014f6f0(o, p): o+0x24c = *p (unaligned word copy), FUN_001ed278(o+0x24c), FUN_0014db70(o).
pub fn ee_sub_14f6f0<M: QMem + ?Sized>(m: &mut M, o: u32, p: u32) {
    let v = m.r32(p);
    m.w32(o + 0x24c, v);
    m.call(0x001e_d278, &[o + 0x24c], &[]);
    m.call(0x0014_db70, &[o], &[]);
}

/// FUN_0013da18(w, s): level stream load. Reads the directory (count FUN_00117d50, 0x6c-byte entries at
/// w+0x3f08: name 0x40 @+0x1c, +0x5c/+0x60/+0x10/+0x14 words, +8 count -> +0x18 table filled with -1),
/// then the global sections (FUN_001fd0e8 .. FUN_0020f1c8, sound/var setup FUN_0016f960), allocates the
/// three render scratch pools (+0x39e0/+0x39d0/+0x39d8), and per directory entry loads its object block
/// (FUN_001848a8 .. FUN_0013fb40), duplicates the first instance row via FUN_0013e510 for every extra copy
/// and runs post-load (vtable +0x48) on all instances of non-deferred entries (+0x5c == 0).
pub fn ee_sub_13da18<M: QMem + ?Sized>(m: &mut M, w: u32, s: u32) {
    const F: u32 = 0x4f20;
    let sp = stack_push(m, F);
    let at = |x: u32| sp + F - x;
    let (c0, b0, bc, b8) = (at(0xc0), sp + 0x20, sp + 0x650, sp + 0x10);
    m.call(0x0012_ae18, &[], &[]);
    let n = m.call(0x0011_7d50, &[s], &[]).0;
    m.w32(sp, n.wrapping_mul(0x6c));
    m.w32(sp + 4, 0x10);
    m.w32(w + 0x3f04, n);
    let t = m.call(0x001a_e2a0, &[sp], &[]).0;
    m.w32(w + 0x3f08, t);
    m.call(0x001c_9058, &[c0], &[]);
    m.call(0x001c_9090, &[c0, 0x002a_4bd0], &[]);
    let mut i = 0u32;
    while (i as i32) < m.r32(w + 0x3f04) as i32 {
        let e = m.r32(w + 0x3f08) + i * 0x6c;
        i += 1;
        m.call(0x0011_7968, &[s, 0x40, e + 0x1c], &[]);
        for off in [0x5c, 0x60, 0x10, 0x14] {
            let v = m.call(0x0011_7db8, &[s], &[]).0;
            m.w32(e + off, v);
        }
        let k = m.call(0x0011_7db8, &[s], &[]).0;
        m.w32(sp, k << 2);
        m.w32(e + 8, k);
        m.w32(sp + 4, 0x10);
        let tb = m.call(0x001a_e1d0, &[sp], &[]).0;
        m.w32(e + 0x18, tb);
        m.call(0x0023_33b4, &[tb, u32::MAX, k << 2], &[]);
    }
    m.call(0x001f_d0e8, &[0x003d_d040, s], &[]);
    m.call(0x001d_7b88, &[w + 0x3a18, s], &[]);
    for f in [0x0020_f318, 0x0013_df80, 0x0013_e0e0] {
        m.call(f, &[w, s], &[]);
    }
    m.call(0x0013_f810, &[0x003d_d490, s], &[]);
    m.call(0x0013_f810, &[0x002d_df40, s], &[]);
    m.call(0x0011_7988, &[s, 0x10], &[]);
    for f in [0x001f_f7e8, 0x0020_e3c0, 0x0020_f1c8] {
        m.call(f, &[w], &[]);
    }
    if m.r32(w + 0x3e38) != 0 {
        m.w32(w + 0x3e00, 0x8000_0000);
    }
    let v = m.r32(w + 0x3e00);
    m.w32(sp, v);
    m.call(0x0016_f960, &[0x002d_edc0, sp, 3], &[]);
    for (size, a, b) in [(0x10000, 0x39e0, Some(0x39e4)), (0x6000, 0x39d0, Some(0x39d4)), (0x3000, 0x39d8, Some(0x39dc))] {
        m.w32(sp, size);
        m.w32(sp + 4, 0x10);
        let p = m.call(0x001a_e318, &[sp], &[]).0;
        m.w32(w + a, p);
        if let Some(b) = b {
            m.w32(w + b, if size == 0x10000 { p } else { 0 });
        }
    }
    m.call(0x001a_e478, &[b8], &[]);
    m.call(0x0017_03f8, &[b0, 0x1800, 0x4000], &[]);
    m.call(0x001e_d3a8, &[], &[]);
    m.call(0x0017_03f8, &[bc, 0x1000, 0x1800], &[]);
    m.call(0x001e_a9f0, &[w, b0], &[]);
    let (e0, s150, d0) = (at(0xe0), at(0x150), at(0xd0));
    let mut i = 0u32;
    while (i as i32) < m.r32(w + 0x3f04) as i32 {
        let e = m.r32(w + 0x3f08) + i * 0x6c;
        m.w32(at(0x4298), 2);
        m.call(0x0025_8248, &[at(0x42a0)], &[]);
        let nm = m.call(0x0014_b690, &[e + 0x1c], &[]).0;
        m.call(0x001c_9090, &[c0, nm], &[]);
        let deferred = m.r32(e + 0x5c) != 0;
        m.w32(w + 0x3f10, if deferred { u32::MAX } else { i });
        m.call(0x0018_48a8, &[s150, 0x403f, at(0x4190), 0], &[]);
        m.call(0x0020_f5d0, &[1, s], &[]);
        let g = m.call(0x0011_7db8, &[s], &[]).0;
        m.w32(0x003e_1c34, g);
        let e10 = m.r32(e + 0x10);
        m.call(0x001f_f858, &[w, e10, s], &[]);
        m.call(0x001e_b748, &[b0, bc, s], &[]);
        m.call(0x001c_0ec0, &[w, s], &[]);
        m.call(0x001a_e478, &[e0], &[]);
        m.call(0x0013_f888, &[sp], &[]);
        m.call(0x0013_fb40, &[0, s, sp], &[]);
        m.call(0x0011_7988, &[s, 0x10], &[]);
        m.call(0x0014_b690, &[e + 0x1c], &[]);
        m.call(0x001e_c7a0, &[], &[]);
        m.w32(w + 0x39e8, 0);
        let cnt = m.r32(sp + 8);
        m.w32(e, cnt);
        let copies = m.r32(e + 8);
        m.w32(d0, cnt.wrapping_mul(copies).wrapping_mul(4));
        m.w32(d0 + 4, 0x10);
        let tbl = m.call(0x001a_e2a0, &[d0], &[]).0;
        m.w32(e + 4, tbl);
        let src = m.r32(sp + 0xc);
        m.call(0x001a_e5a0, &[tbl, src, cnt << 2], &[]);
        let mut out = tbl + cnt * 4;
        for _ in 1..(copies as i32).max(1) {
            for j in 0..cnt {
                let o = m.r32(tbl + 4 * j);
                let c = m.call(0x0013_e510, &[o, 0], &[]).0;
                m.w32(out, c);
                out += 4;
            }
        }
        if !deferred {
            let tot = (cnt as i32).wrapping_mul(copies as i32);
            for j in 0..tot.max(0) as u32 {
                let o = m.r32(tbl + 4 * j);
                let vt = m.r32(o + 0xc);
                let adj = m.r16(vt + 0x48) as i16 as i32;
                let f = m.r32(vt + 0x4c);
                m.call(f, &[(o as i32 + adj) as u32], &[]);
            }
        } else {
            let v = m.r32(e + 0xc);
            m.w32(e + 0xc, v.wrapping_add(1));
        }
        m.call(0x001a_e4a0, &[e0, 2], &[]);
        m.call(0x0018_4950, &[s150, 2], &[]);
        i += 1;
    }
    m.call(0x001e_d448, &[], &[]);
    m.w32(w + 0x3f10, 0xffff_fffe);
    m.call(0x001f_d5e8, &[0x003d_d040], &[]);
    m.call(0x001c_9090, &[c0, 0], &[]);
    m.call(0x001c_9098, &[c0], &[]);
    m.call(0x001a_e4a0, &[b8, 2], &[]);
    stack_pop(m, F);
}

/// FUN_0013c560 helper: arm the throw/arc helper `h` (FUN_001ce888 result) from `from` toward `to`:
/// h's vtable(+0xec) +0xc (1.0, pos(from), 0), FUN_001cf578(h, o), +0x18 = max(|from - to|, 250) * 0.002,
/// +0x1c = tgt1c, +0xa0/+0xa4 = 0, +0xa8 = -3000, +0x104 = t104, FUN_001cfd30(h), o's vtable +0x40.
fn arm_13c560<M: QMem + ?Sized>(m: &mut M, o: u32, h: u32, from: u32, to: u32, t1c: u32, t104: u32) {
    let vt = m.r32(h + 0xec);
    let adj = m.r16(vt + 8) as i16 as i32;
    let f = m.r32(vt + 0xc);
    let p = m.call(0x001a_62e8, &[from], &[]).1;
    m.call(f, &[(h as i32 + adj) as u32, 0], &[p, f4(1.0)]);
    m.call(0x001c_f578, &[h, o], &[]);
    let a = m.call(0x001a_62e8, &[from], &[]).1;
    let b = m.call(0x001a_62e8, &[to], &[]).1;
    let d = sub(a, b);
    let dist = dot3(d, d).sqrt();
    m.w32(h + 0x1c, t1c);
    m.wf(h + 0x18, if dist < 250.0 { 250.0 } else { dist } * 0.002);
    m.w32(h + 0xa0, 0);
    m.wf(h + 0xa8, -3000.0);
    m.w32(h + 0xa4, 0);
    m.w32(h + 0x104, t104);
    m.call(0x001c_fd30, &[h], &[]);
    vcall(m, o, 0x40, &[], &[]);
}

/// FUN_0013c560(o, st): carried-prop (loot) state change o+0x184 -> st (0 free, 1 held by +0x15c,
/// 2/6 flying to the carrier +0x160's +0x11a4, 3 flying from +0x15c, 4 carried, 5 dropped/tossed).
/// Leaves the old state (registry FUN_001cb488/4b8 on DAT_003e1ac8 when crossing 1|2, release grip,
/// collision restore, hand-off), stamps +0x188 = clock, then enters the new one (attach FUN_001a58d0 /
/// FUN_001a6be8, arc helper FUN_001ce888 kind 6, toss with a random spin, events 0x2f/0x30 on DAT_002deb30).
pub fn ee_sub_13c560<M: QMem + ?Sized>(m: &mut M, o: u32, st: u32) {
    const W: u32 = 0x003d_d490;
    let old = m.r32(o + 0x184);
    if old == st {
        return;
    }
    let sp = stack_push(m, 0x110);
    let reg = m.r32(0x003e_1ac8);
    if reg != 0 {
        if (old as i32) < 2 {
            if (st as i32) > 1 {
                m.call(0x001c_b488, &[reg, o], &[]);
            }
        } else if (st as i32) <= 1 {
            m.call(0x001c_b4b8, &[reg, o], &[]);
        }
    }
    let carrier = m.r32(o + 0x160);
    let zero = m.rv(0x002a_2dd0);
    match old {
        0 => {
            m.call(0x001f_6a68, &[o], &[]);
            let mut i = 0u32;
            while (i as i32) < r158(m, o, 0x1c) as i32 {
                let d = m.r32(o + 0x158);
                let x = m.r32(d + 0x20 + 4 * i);
                m.call(0x001f_6978, &[o, x], &[]);
                i += 1;
            }
            let mut i = 0u32;
            while (i as i32) < m.r32(o + 0xe0) as i32 {
                let e = m.r32(o + 0xe4) + i * 0xb0;
                m.call(0x001f_2ad0, &[o, e], &[]);
                i += 1;
            }
            if r158(m, o, 0x18) != u32::MAX {
                m.call(0x001f_6ac8, &[o], &[]);
            }
        }
        1 => {
            if r158(m, o, 0xc) != 0 {
                let h = m.r32(o + 0x15c);
                let t = m.rf(h + 0x18c) + 0.99;
                m.call(0x0016_18f8, &[W], &[f4(t)]);
            }
            let h = m.r32(o + 0x15c);
            let p = m.call(0x001a_62e8, &[h], &[]).1;
            m.call(0x0015_6c80, &[W, 1, h, h + 0x198], &[p]);
            if st == 2 {
                m.call(0x0016_19a0, &[0x002d_eb30, 0x2f, o], &[]);
            }
            let h = m.r32(o + 0x15c);
            m.w32(h + 0x194, 0);
            m.w32(o + 0x15c, 0);
            vcall(m, o, 0x18, &[5, 0, 0, 0, 0], &[]);
        }
        2 => {
            m.w32(o + 0x18c, 0);
            if st == 4 {
                m.wf(o + 0x180, 0.3);
            }
        }
        3 => m.w32(o + 0x18c, 0),
        4 => {
            m.w32(o + 0x160, 0);
            let g = m.r32(carrier + 0x11a4);
            vcall(m, g, 0x48, &[], &[]);
            m.w32(carrier + 0x11a8, 0);
            if r158(m, o, 0x18) != u32::MAX {
                m.call(0x001f_6ac8, &[o], &[]);
            }
            if r158(m, o, 0xc) != 0 {
                m.call(0x0016_1958, &[W], &[]);
            }
        }
        5 => {
            let t = m.r32(o + 0x164);
            if t != 0 {
                m.w32(sp, 1);
                m.w32(sp + 4, t);
                vcall(m, t, 0x198, &[sp], &[]);
            }
            m.call(0x001f_5720, &[o, 1], &[]);
            m.call(0x001f_57c0, &[o, 1], &[]);
            vcall(m, o, 0x118, &[], &[zero]);
            vcall(m, o, 0x120, &[], &[zero]);
            m.call(0x001f_5880, &[o], &[zero]);
        }
        6 => m.w32(o + 0x180, 0),
        _ => {}
    }
    m.w32(o + 0x184, st);
    let clk = m.r32(0x002d_ded0);
    m.w32(o + 0x188, clk);
    match st {
        0 => {
            m.w32(o + 0x158, 0);
            m.call(0x001a_58d0, &[o, 0, 0], &[]);
            vcall(m, o, 0x48, &[], &[]);
        }
        1 => {
            let h = m.r32(o + 0x15c);
            m.call(0x0015_6c80, &[W, u32::MAX, 0, h + 0x198], &[zero]);
            let h = m.r32(o + 0x15c);
            let x = vcall(m, h, 0xf8, &[], &[]).0;
            m.call(0x001a_6be8, &[o, x], &[]);
            let h10 = m.r32(h + 0x10);
            m.call(0x001a_58d0, &[o, h10, 3], &[]);
            m.w32(h + 0x194, o);
            if old != 3 {
                vcall(m, o, 0x48, &[], &[]);
                m.call(0x001f_7c68, &[o], &[f4(0.5)]);
            }
            vcall(m, o, 0x18, &[6, 0, 0, 0, 0], &[]);
        }
        2 | 6 => {
            let g = m.r32(carrier + 0x11a4);
            vcall(m, g, 0x40, &[], &[]);
            if old == 2 {
                let h = m.call(0x001c_e888, &[W, 6, 0], &[]).0;
                m.w32(o + 0x18c, h);
            } else {
                m.w32(o + 0x18c, 0);
                vcall(m, o, 0x18, &[6, 0, 0, 0, 0], &[]);
                vcall(m, o, 0x18, &[5, 0, 0, 0, 0], &[]);
            }
            let h = m.r32(o + 0x18c);
            if h != 0 {
                let c = m.r32(o + 0x160);
                let g = m.r32(c + 0x11a4);
                arm_13c560(m, o, h, o, g, g, o);
            }
            let c = m.r32(o + 0x160);
            let g = m.r32(c + 0x11a4);
            let x = vcall(m, g, 0xf8, &[], &[]).0;
            m.call(0x001a_6be8, &[o, x], &[]);
            m.call(0x001a_58d0, &[o, g, 3], &[]);
            if m.r32(o + 0x18c) == 0 {
                ee_sub_13c560(m, o, 4);
            }
        }
        3 => {
            let h = m.r32(o + 0x15c);
            let x = vcall(m, h, 0xf8, &[], &[]).0;
            m.call(0x001a_6be8, &[o, x], &[]);
            let h10 = m.r32(h + 0x10);
            m.call(0x001a_58d0, &[o, h10, 3], &[]);
            let a = m.call(0x001c_e888, &[W, 6, 0], &[]).0;
            m.w32(o + 0x18c, a);
            if a != 0 {
                let g = m.r32(carrier + 0x11a4);
                arm_13c560(m, o, a, g, o, o, g);
                if m.r32(o + 0x18c) != 0 {
                    stack_pop(m, 0x110);
                    return;
                }
            }
            ee_sub_13c560(m, o, 1);
        }
        4 => {
            let c = m.r32(o + 0x160);
            m.w32(c + 0x11a8, o);
            if r158(m, o, 0x18) != u32::MAX {
                m.call(0x001f_6b48, &[o], &[]);
            }
        }
        5 => {
            m.call(0x001a_58d0, &[o, 0, 3], &[]);
            m.call(0x001f_5720, &[o, 0], &[]);
            m.call(0x001f_57c0, &[o, 0], &[]);
            let pv = vcall(m, o, 0xf8, &[], &[]).0;
            let v = scale(m.rv(pv), 1000.0);
            vcall(m, o, 0x118, &[], &[v]);
            let tau = std::f32::consts::TAU;
            let mut r = m.rv(sp + 0x10);
            for k in 0..3 {
                r[k] = m.call(0x0011_d448, &[], &[f4(-tau), f4(tau)]).1[0];
            }
            m.wv(sp + 0x10, r);
            vcall(m, o, 0x120, &[], &[r]);
            let z = m.rv(0x002a_4b60);
            m.call(0x001f_5880, &[o], &[z]);
            m.call(0x0016_19a0, &[0x002d_eb30, 0x30, o], &[]);
        }
        _ => {}
    }
    stack_pop(m, 0x110);
}

fn r158<M: QMem + ?Sized>(m: &mut M, o: u32, off: u32) -> u32 {
    let d = m.r32(o + 0x158);
    m.r32(d + off)
}

/// resolve a 8-byte C++ member-function pointer at `pmf` for `obj` -> (fn, adjusted this)
fn pmf_res<M: QMem + ?Sized>(m: &mut M, obj: u32, pmf: u32) -> (u32, u32) {
    let d = m.r16(pmf) as i16 as i32;
    let i = m.r16(pmf + 2) as i16 as i32;
    if i < 0 {
        (m.r32(pmf + 4), (obj as i32 + d) as u32)
    } else {
        let vd = m.r16(pmf + 4) as i16 as i32;
        let vt = m.r32((obj as i32 + vd) as u32);
        let e = vt.wrapping_add((i as u32) * 8).wrapping_sub(8);
        let adj = m.r16(e) as i16 as i32;
        (m.r32(e + 4), (obj as i32 + adj + d) as u32)
    }
}

/// FUN_0013e818(obj, prop, flags, s): deserialize one class property from stream `s`.
/// flags & 0x401: array property - prop+0x18 elements, each a value object (FUN_0013e690, or for 0x8000
/// kinds a byte count of sub-values combined by FUN_00200da0) gathered in 8-byte slots at sp, then handed to
/// prop+0x10 (or prop+8 with 0x400) as (tmp, obj, n, slots); slots released via FUN_00129ec8.
/// flags & 0x90: direct member at obj (or member-getter pmf prop+0x20 (this, 1) with 0x2000) + prop+0x18,
/// stored per kind (*prop: 0 u8, 1 f32, 2/9/b vec3, 3/c vec4, 4 euler -> 3x4 matrix FUN_0011b570, 5/6 / 7.. ints,
/// a vec2, d string id FUN_001ed278, 0x16 u16 buffer, 0x1xxx id, 0x2xxx object ref: resolved now
/// FUN_0013e590 or deferred (0x80) through FUN_00210120/FUN_001a5d58).
/// otherwise: the same kinds passed to the setter pmf prop+0x10 (prop+0x18 with 0x100), deferred refs via
/// FUN_0013e5e0.
pub fn ee_sub_13e818<M: QMem + ?Sized>(m: &mut M, obj: u32, prop: u32, flags: u32, s: u32) {
    const F: u32 = 0x350;
    let sp = stack_push(m, F);
    let rd_f = |m: &mut M| m.call(0x0011_7e38, &[s], &[]).1[0].to_bits();
    let kind = m.r32(prop);
    let high = ((kind & 0xffff_7fff) as i32) >> 0xc;
    if flags & 0x401 != 0 {
        let tmp = sp + 0x100;
        for k in 0..32 {
            m.w32(sp + 8 * k, u32::MAX);
        }
        let n = m.r32(prop + 0x18);
        let mut i = 0u32;
        while (i as i32) < n as i32 {
            let kd = m.call(0x0011_7c20, &[s], &[]).0;
            if kd & 0x8000 == 0 {
                m.call(0x0013_e690, &[tmp, kd, s], &[]);
                m.call(0x0012_9658, &[sp + 8 * i, tmp], &[]);
                m.call(0x0012_9ec8, &[tmp], &[]);
            } else {
                let (dc, e0, e4) = (sp + 0x200, sp + 0x1c0, sp + 0x180);
                for k in 0..16 {
                    m.w32(tmp + 8 * k, u32::MAX);
                }
                let kd = kd & 0xffff_7fff;
                let c = m.call(0x0011_7bd0, &[s], &[]).0;
                for j in 0..(c as i32).max(0) as u32 {
                    m.call(0x0013_e690, &[dc, kd, s], &[]);
                    m.call(0x0012_9658, &[tmp + 8 * j, dc], &[]);
                    m.call(0x0012_9ec8, &[dc], &[]);
                    m.w32(e0 + 4 * j, tmp + 4 + 8 * j);
                    m.w32(e4 + 4 * j, kd);
                }
                m.call(0x0020_0da0, &[dc, c, e4, e0], &[]);
                m.call(0x0012_9658, &[sp + 8 * i, dc], &[]);
                m.call(0x0012_9ec8, &[dc], &[]);
                let mut p = e4;
                while p != tmp {
                    p -= 8;
                    m.call(0x0012_9ec8, &[p], &[]);
                }
            }
            i += 1;
        }
        let f = m.r32(prop + if flags & 0x400 != 0 { 8 } else { 0x10 });
        m.call(f, &[tmp, obj, n, sp], &[]);
        m.call(0x0012_9ec8, &[tmp], &[]);
        let mut p = tmp;
        while p != sp {
            p -= 8;
            m.call(0x0012_9ec8, &[p], &[]);
        }
        stack_pop(m, F);
        return;
    }
    let mut obj = obj;
    if flags & 0x90 != 0 {
        let base = if flags & 0x2000 == 0 { obj } else { pmf_call(m, obj, prop + 0x20, &[1]) };
        let fld = base.wrapping_add(m.r32(prop + 0x18));
        match kind {
            0 => {
                let v = m.call(0x0011_7bd0, &[s], &[]).0;
                m.w8(fld, v as u8);
            }
            1 | 2 | 9 | 0xb | 3 | 0xc | 10 => {
                let n = match kind {
                    1 => 1,
                    10 => 2,
                    3 | 0xc => 4,
                    _ => 3,
                };
                for k in 0..n {
                    let v = rd_f(m);
                    m.w32(fld + 4 * k, v);
                }
            }
            4 => {
                for k in 0..3 {
                    let v = rd_f(m);
                    m.w32(sp + 4 * k, v);
                }
                let v = m.rv(sp);
                m.call(0x0011_b570, &[sp + 0x10], &[v]);
                for k in 0..12 {
                    let w = m.r32(sp + 0x10 + 4 * k);
                    m.w32(fld + 4 * k, w);
                }
            }
            5 | 6 => {
                let v = m.call(0x0011_7db8, &[s], &[]).0;
                m.w32(fld, v);
            }
            7 | 0xe..=0x11 | 0x13..=0x15 | 0x17 => {
                let v = m.call(0x0011_7d50, &[s], &[]).0;
                m.w32(fld, v);
            }
            0xd => {
                let v = m.call(0x0011_7c80, &[s], &[]).0;
                m.w32(sp, v);
                m.call(0x001e_d278, &[sp], &[]);
                let v = m.r32(sp);
                m.w32(fld, v);
            }
            0x16 => {
                let n = m.call(0x0011_7c20, &[s], &[]).0;
                m.w32(fld, n);
                m.w32(sp, n << 1);
                m.w32(sp + 4, 0x10);
                let b = m.call(0x001a_e2a0, &[sp], &[]).0;
                m.w32(fld + 4, b);
                m.call(0x0011_7968, &[s, n << 1, b], &[]);
            }
            _ if high == 1 => {
                let v = m.call(0x0011_7c80, &[s], &[]).0;
                m.w32(fld, v);
            }
            _ if high == 2 => {
                let v = m.call(0x0011_7db8, &[s], &[]).0;
                if flags & 0x80 != 0 {
                    let p = m.call(0x0021_0120, &[0x003d_d490], &[]).0;
                    m.w32(p, v);
                    m.w32(p + 4, fld.wrapping_sub(obj));
                    let q = m.call(0x001a_5d58, &[obj], &[]).0;
                    m.w32(q + 4, p);
                    m.w32(q, 0x0013_e590);
                } else {
                    let w = m.r32(prop + 0x10);
                    m.w32(sp, v);
                    m.w32(sp + 4, w);
                    m.call(0x0013_e590, &[obj, sp], &[]);
                }
            }
            _ => {}
        }
        stack_pop(m, F);
        return;
    }
    let orig = obj;
    if flags & 0x2000 != 0 {
        obj = pmf_call(m, obj, prop + 0x20, &[1]);
    }
    let pmf = prop + if flags & 0x100 != 0 { 0x18 } else { 0x10 };
    let a = sp + 0x10;
    match kind {
        0 | 5 | 6 | 7 | 0xe..=0x11 | 0x13..=0x15 | 0x17 => {
            let rd = match kind {
                0 => 0x0011_7bd0,
                5 | 6 => 0x0011_7db8,
                _ => 0x0011_7d50,
            };
            let (f, t) = pmf_res(m, obj, pmf);
            let v = m.call(rd, &[s], &[]).0;
            m.call(f, &[t, v], &[]);
        }
        1 => {
            let (f, t) = pmf_res(m, obj, pmf);
            let v = m.call(0x0011_7e38, &[s], &[]).1;
            m.call(f, &[t], &[v]);
        }
        2 | 3 => {
            let n = if kind == 2 { 3 } else { 4 };
            for k in 0..n {
                let v = rd_f(m);
                m.w32(a + 4 * k, v);
            }
            let v = m.rv(a);
            let (f, t) = pmf_res(m, obj, pmf);
            m.call(f, &[t], &[v]);
        }
        4 => {
            for k in 0..3 {
                let v = rd_f(m);
                m.w32(a + 4 * k, v);
            }
            let v = m.rv(a);
            m.call(0x0011_b570, &[sp + 0x50], &[v]);
            for k in 0..12 {
                let w = m.r32(sp + 0x50 + 4 * k);
                m.w32(sp + 0x20 + 4 * k, w);
            }
            let (f, t) = pmf_res(m, obj, pmf);
            m.call(f, &[t, sp + 0x20], &[]);
        }
        9 | 10 | 0xb | 0xc => {
            let n = match kind {
                10 => 2,
                0xc => 4,
                _ => 3,
            };
            for k in 0..n {
                let v = rd_f(m);
                m.w32(a + 4 * k, v);
            }
            let (f, t) = pmf_res(m, obj, pmf);
            m.call(f, &[t, a], &[]);
        }
        0xd => {
            let v = m.call(0x0011_7c80, &[s], &[]).0;
            m.w32(a, v);
            m.call(0x001e_d278, &[a], &[]);
            let v = m.r32(a);
            m.w32(sp + 0x20, v);
            let (f, t) = pmf_res(m, obj, pmf);
            m.call(f, &[t, sp + 0x20], &[]);
        }
        0x16 => {
            let pool = sp + 0x210;
            m.call(0x001a_e478, &[pool], &[]);
            let n = m.call(0x0011_7c20, &[s], &[]).0;
            m.w32(a, n);
            m.w32(sp + 0x20, n << 1);
            m.w32(sp + 0x24, 0x10);
            let b = m.call(0x001a_e378, &[sp + 0x20], &[]).0;
            m.w32(a + 4, b);
            m.call(0x0011_7968, &[s, n << 1, b], &[]);
            let (f, t) = pmf_res(m, obj, pmf);
            m.call(f, &[t, a], &[]);
            m.call(0x001a_e4a0, &[pool, 2], &[]);
        }
        _ if high == 1 => {
            let (f, t) = pmf_res(m, obj, pmf);
            let v = m.call(0x0011_7c80, &[s], &[]).0;
            m.call(f, &[t, v], &[]);
        }
        _ if high == 2 => {
            let v = m.call(0x0011_7db8, &[s], &[]).0;
            if flags & 0x100 != 0 {
                let p = m.call(0x0021_0120, &[0x003d_d490], &[]).0;
                m.w32(p, v);
                let (x, y) = (m.r32(prop + 0x18), m.r32(prop + 0x1c));
                m.w32(p + 4, x);
                m.w32(p + 8, y);
                let q = m.call(0x001a_5d58, &[orig], &[]).0;
                m.w32(q + 4, p);
                m.w32(q, 0x0013_e5e0);
            } else {
                let (x, y) = (m.r32(prop + 0x10), m.r32(prop + 0x14));
                m.w32(a, v);
                m.w32(a + 4, x);
                m.w32(a + 8, y);
                m.call(0x0013_e5e0, &[orig, a], &[]);
            }
        }
        _ => {}
    }
    stack_pop(m, F);
}
