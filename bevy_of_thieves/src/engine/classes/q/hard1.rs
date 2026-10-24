//! WP-HARD1: the hard unstarted functions (VU0 macro math, 128-bit register args, stack records), ported
//! 1:1 from the raw disassembly (the decompiles lose quadword args / lanes). QMem convention: integer args
//! in `a`, quadword (VU0 / 128-bit GPR) and float args in `v`; quadword results are `.1`.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};
use crate::engine::classes::q::q6::{callf, rov, s};
use crate::engine::math::vu0;

/// DAT_002dded4: frame delta time
pub const DT: u32 = 0x002d_ded4;

/// FUN_0018e1e0 (Jt steer toward the +0x10dc target): d = pos(target) - pos(this) (FUN_001a62e8, quadword
/// results); len = |d.xyz|; dir = d / len (all lanes), or DAT_002a2da0 when len < 1e-4; FUN_002046b0(this, dir);
/// speed = PosApproachTrapezoid(len, DAT_002bf414, dt * 0.5, 0x2bf418, &out) (FUN_0011d018, out[0]);
/// +0x190.xyz = (FUN_0011b340(+0x1b0) rows * speed) * dir (w = dir.w).
pub fn ee_sub_18e1e0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let t = m.r32(o + 0x10dc);
    let a = m.call(0x001a_62e8, &[t], &[]).1;
    let b = m.call(0x001a_62e8, &[o], &[]).1;
    let d = vu0::sub(a, b);
    let len = vu0::vsqrt(vu0::dot3(d, d));
    let dir = if len < f32::from_bits(0x38d1_b717) {
        rov(0x002a_2da0)
    } else {
        vu0::scale(d, vu0::vdiv(1.0, len))
    };
    m.call(0x0020_46b0, &[o], &[dir]);
    let sp = stack_push(m, 0x80);
    let k = m.rf(0x002b_f414);
    let h = m.rf(DT) * 0.5;
    m.call(0x0011_d018, &[0x002b_f418, sp + 0x40], &[s(len), s(k), s(h)]);
    let spd = m.rf(sp + 0x40);
    m.call(0x0011_b340, &[sp, o + 0x1b0], &[]);
    let r = [0u32, 0x10, 0x20].map(|i| vu0::scale(m.rv(sp + i), spd));
    stack_pop(m, 0x80);
    let v = vu0::mul3(&r, dir);
    m.wv(o + 0x190, vu0::masked(dir, v, 0b0111));
}

/// FUN_001cf0a0 (C2d7468_v3, particle step): unless |vel +0xd0| is tiny (FUN_001cf010): vel *= 1 + drag(+0xe4) * dt;
/// with len = |vel.xyz| > 1e-4 the basis rows +0x40..+0x60 are rotated (xyz) by MatrixFromAxisAngle(vel / len,
/// len * dt) (FUN_0011b500). previous +0x80 = position +0x70. When spin +0x90 is not small (FUN_001cefc8) or the
/// spin record +0xa0 is not small (FUN_001cf058): M = FUN_0015bfd8(+0xa0); +0x90.xyz = M * (+0x90, 1);
/// +0x70 += +0x90 * dt.
/// Quirks kept (obs: disasm 0x1cf1b8, 0x1cf244): row +0x60 takes w from R row 0 (vf3 reuse); w lanes keep M row 0 w;
/// `VADDA ACC, pos, vf0` adds 1.0 to position.w every step.
pub fn ee_sub_1cf0a0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    use crate::engine::classes::q::c2d7468::{ee_sub_1cefc8, ee_sub_1cf010, ee_sub_1cf058};
    let dt = m.rf(DT);
    let vel = m.rv(o + 0xd0);
    if !ee_sub_1cf010(vel) {
        let k = m.rf(o + 0xe4) * dt + 1.0;
        let v = vu0::scale(vel, k);
        m.wv(o + 0xd0, v);
        let len = vu0::vsqrt(vu0::dot3(v, v));
        if f32::from_bits(0x38d1_b717) < len {
            let axis = vu0::scale(v, vu0::vdiv(1.0, len));
            let sp = stack_push(m, 0x40);
            m.call(0x0011_b500, &[sp], &[axis, s(len * dt)]);
            let r = [m.rv(sp), m.rv(sp + 0x10), m.rv(sp + 0x20)];
            stack_pop(m, 0x40);
            let rows = [m.rv(o + 0x40), m.rv(o + 0x50), m.rv(o + 0x60)];
            let n0 = vu0::masked(rows[0], vu0::mul3(&r, rows[0]), 0b0111);
            let n1 = vu0::masked(rows[1], vu0::mul3(&r, rows[1]), 0b0111);
            let n2 = vu0::masked(r[0], vu0::mul3(&r, rows[2]), 0b0111);
            m.wv(o + 0x40, n0);
            m.wv(o + 0x50, n1);
            m.wv(o + 0x60, n2);
        }
    }
    let p = m.rv(o + 0x70);
    m.wv(o + 0x80, p);
    let spin = m.rv(o + 0x90);
    let rec = m.rv(o + 0xa0);
    if !ee_sub_1cefc8(spin) || !ee_sub_1cf058(rec) {
        let sp = stack_push(m, 0x40);
        m.call(0x0015_bfd8, &[sp, o + 0xa0], &[]);
        let mm = m.rm(sp);
        stack_pop(m, 0x40);
        let t = vu0::mul4(&mm, [spin[0], spin[1], spin[2], 1.0]);
        let ns = vu0::masked(mm[0], t, 0b0111);
        m.wv(o + 0x90, ns);
        let p = m.rv(o + 0x70);
        let np: V4 = std::array::from_fn(|i| vu0::fix(p[i] + [0.0, 0.0, 0.0, 1.0][i] + ns[i] * dt));
        m.wv(o + 0x70, np);
    }
}

/// rows `r` (R * row).xyz for the three basis rows at `b`; the third row takes w from R row 0 (obs: the
/// `VMADDz.xyz vf3, ..` destination reuses R row 0's register, 0x1cf1b8 / 0x1d0acc)
fn rot_rows<M: QMem + ?Sized>(m: &mut M, b: u32, r: &[V4; 3]) -> [V4; 3] {
    let rows = [m.rv(b), m.rv(b + 0x10), m.rv(b + 0x20)];
    [
        vu0::masked(rows[0], vu0::mul3(r, rows[0]), 0b0111),
        vu0::masked(rows[1], vu0::mul3(r, rows[1]), 0b0111),
        vu0::masked(r[0], vu0::mul3(r, rows[2]), 0b0111),
    ]
}

/// FUN_001d09a0 (C2d74b8_v3): d = spin(+0x90) . row0; rate = clamp(c10 + d*(c14 + d*c18), [c20, c24])
/// (DAT_002c2010..); R = MatrixFromAxisAngle(row1 (128-bit a1), rate * dt); spin and the three rows rotated by R;
/// FUN_0011e290(spin, old row0, &par, &perp); dir = par / |par| (DAT_002a2da0 when < 1e-4);
/// spin = dir * (c28 + (c2c * dt + 1) * (|par| - c28)) + perp * (+0xc4 * dt + 1); C2d7468_v3 runs with +0xc4 = 0,
/// then +0xc4 is restored.
pub fn ee_sub_1d09a0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let row0 = m.rv(o + 0x40);
    let spin = m.rv(o + 0x90);
    let d = vu0::dot3(spin, row0);
    let mut f = m.rf(0x002c_2010) + d * (m.rf(0x002c_2014) + d * m.rf(0x002c_2018));
    let (lo, hi) = (m.rf(0x002c_2020), m.rf(0x002c_2024));
    if f < lo {
        f = lo;
    } else if hi < f {
        f = hi;
    }
    let row1 = m.rv(o + 0x50);
    let dt = m.rf(DT);
    let sp = stack_push(m, 0x50);
    m.call(0x0011_b500, &[sp], &[row1, s(f * dt)]);
    let r = [m.rv(sp), m.rv(sp + 0x10), m.rv(sp + 0x20)];
    let ns = vu0::masked(spin, vu0::mul3(&r, spin), 0b0111);
    m.wv(o + 0x90, ns);
    let rows = rot_rows(m, o + 0x40, &r);
    let c4 = m.rf(o + 0xc4);
    m.wv(o + 0x50, rows[1]);
    m.wv(o + 0x60, rows[2]);
    m.w32(o + 0xc4, 0);
    m.wv(o + 0x40, rows[0]);
    // out slots still hold row0' / row1' (stack reuse) when FUN_0011e290 skips one
    m.wv(sp + 0x30, rows[0]);
    m.wv(sp + 0x40, rows[1]);
    m.call(0x0011_e290, &[sp + 0x30, sp + 0x40], &[ns, row0]);
    let par = m.rv(sp + 0x30);
    let perp = m.rv(sp + 0x40);
    stack_pop(m, 0x50);
    let len = vu0::vsqrt(vu0::dot3(par, par));
    let dir = if len < f32::from_bits(0x38d1_b717) { rov(0x002a_2da0) } else { vu0::scale(par, vu0::vdiv(1.0, len)) };
    let c28 = m.rf(0x002c_2028);
    let nl = c28 + (m.rf(0x002c_202c) * dt + 1.0) * (len - c28);
    let kp = c4 * dt + 1.0;
    let v: V4 = std::array::from_fn(|i| vu0::fix(dir[i] * nl + perp[i] * kp));
    m.wv(o + 0x90, v);
    ee_sub_1cf0a0(m, o);
    m.wf(o + 0xc4, c4);
}

/// FUN_001d0eb8 (C2d7508 update): C2d7468_v3; r = rand(0, 1) (FUN_0011d448); unless spin +0x90 equals
/// DAT_002a2dd0 lane by lane (FUN_0011d4d0 eps 1e-4 on x, then y (PROT3W lane 1), then z (PEXTUW lane 2)) and
/// r < 0.3: (u, v) = FUN_0011e2d8(spin / |spin|, DAT_002a2da0, DAT_002a2dc0); mag = rand(100, (1 - r * 3.333) * 300 + 100);
/// (s, c) = sincos(rand(0, 2pi)); +0x70 += (u * s * mag + v * c * mag) * dt (+1.0 on w, `VADDA ACC, pos, vf0`).
/// obs: no |spin| epsilon before the VDIV (0 -> FLT_MAX); spin lanes moved into $f12/$f13 with PROT3W / PEXTUW.
pub fn ee_sub_1d0eb8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    ee_sub_1cf0a0(m, o);
    let r = m.call(0x0011_d448, &[], &[s(0.0), s(1.0)]).1[0];
    let s1 = vu0::qbits(m.rv(o + 0x90));
    let s0 = vu0::qbits(rov(0x002a_2dd0));
    let eps = s(f32::from_bits(0x38d1_b717));
    let feq = |m: &mut M, a: u32, b: u32| m.call(0x0011_d4d0, &[], &[s(f32::from_bits(a)), s(f32::from_bits(b)), eps]).0 != 0;
    let mut same = false;
    if feq(m, s0[0], s1[0]) && feq(m, vu0::prot3w(s0)[0], vu0::prot3w(s1)[0]) {
        same = feq(m, vu0::pextuw([0; 4], s0)[0], vu0::pextuw([0; 4], s1)[0]);
    }
    if same || !(r < f32::from_bits(0x3e99_999a)) {
        return;
    }
    let spin = vu0::qf(s1);
    let len = vu0::vsqrt(vu0::dot3(spin, spin));
    let dir = vu0::scale(spin, vu0::vdiv(1.0, len));
    let sp = stack_push(m, 0x60);
    m.call(0x0011_e2d8, &[sp + 0x10], &[dir, rov(0x002a_2da0), rov(0x002a_2dc0)]);
    let hi = (1.0 - r * f32::from_bits(0x4055_5555)) * 300.0 + 100.0;
    let mag = m.call(0x0011_d448, &[], &[s(100.0), s(hi)]).1[0];
    let ang = m.call(0x0011_d448, &[], &[s(0.0), s(f32::from_bits(0x40c9_0fdb))]).1[0];
    m.call(0x0011_d608, &[sp + 0x50, sp + 0x54], &[s(ang)]);
    let (sn, cs) = (m.rf(sp + 0x50) * mag, m.rf(sp + 0x54) * mag);
    let (u, v) = (m.rv(sp + 0x10), m.rv(sp + 0x20));
    stack_pop(m, 0x60);
    let d: V4 = std::array::from_fn(|i| vu0::fix(u[i] * sn + v[i] * cs));
    let dt = m.rf(DT);
    let p = m.rv(o + 0x70);
    let np: V4 = std::array::from_fn(|i| vu0::fix(p[i] + [0.0, 0.0, 0.0, 1.0][i] + d[i] * dt));
    m.wv(o + 0x70, np);
}

/// FUN_001db0c8 (C2d8568_v13, select the nearest marker in a direction): on a new choice `sel` (!= +0x28): +0x28 =
/// sel, +0x2c = DAT_002ddf0c; unless sel or +0x48 is -1: cur = FUN_001dae20(this, +0x48); base = (cur.x14,
/// cur.y18 * DAT_002c3280, 0, 0) (PEXTLW / PCPYLD packing); dir = (+0x10 ? (t.x103c, -t.y1040, 0, 0) : 0) / |dir|
/// * DAT_002c3274; for each other entry e of the +0x50 table (+0x4c entries of 0x20, flag +0x1c & 4) with
/// p = (e.x14, e.y18 * k, 0, 0): f = hit.t(+0x40) * c3274 when FUN_0014c748(c3278, p, base, base + dir, 1, &hit) > 0,
/// else (t, d) = FUN_0011ee08(p, base, base + dir) and f = c3274 + d when d < DAT_002c327c and 0 < t; the smallest
/// f below DAT_002a79c8 wins (e[0]) -> FUN_001db390(this, id), FUN_001da9e8(this), FUN_001fb2f8(FUN_001fbb90(0x35), 0).
/// obs: zero dir -> VDIV gives FLT_MAX, times 0 = 0 (kept by vu0::vdiv).
pub fn ee_sub_1db0c8<M: QMem + ?Sized>(m: &mut M, o: u32, sel: u32) {
    if sel == m.r32(o + 0x28) {
        return;
    }
    m.w32(o + 0x28, sel);
    let t = m.r32(0x002d_df0c);
    m.w32(o + 0x2c, t);
    if sel == u32::MAX || m.r32(o + 0x48) == u32::MAX {
        return;
    }
    let a48 = m.r32(o + 0x48);
    let cur = m.call(0x001d_ae20, &[o, a48], &[]).0;
    let k80 = m.rf(0x002c_3280);
    let base = [m.rf(cur + 0x14), m.rf(cur + 0x18) * k80, 0.0, 0.0];
    let tg = m.r32(o + 0x10);
    let d = if tg != 0 { [m.rf(tg + 0x103c), -m.rf(tg + 0x1040), 0.0, 0.0] } else { [0.0; 4] };
    let n = vu0::scale(d, vu0::vdiv(1.0, vu0::vsqrt(vu0::dot3(d, d))));
    let k74 = m.rf(0x002c_3274);
    let off = vu0::scale(n, k74);
    let end = vu0::add(base, off);
    let mut best = m.rf(0x002a_79c8);
    let mut win = u32::MAX;
    let sp = stack_push(m, 0x70);
    let mut i = 0u32;
    while (i as i32) < m.r32(o + 0x4c) as i32 {
        let e = m.r32(o + 0x50).wrapping_add(i << 5);
        i += 1;
        if e == cur || m.r32(e + 0x1c) & 4 == 0 {
            continue;
        }
        let p = [m.rf(e + 0x14), m.rf(e + 0x18) * k80, 0.0, 0.0];
        let c78 = m.rf(0x002c_3278);
        let f = if (m.call(0x0014_c748, &[1, sp], &[s(c78), p, base, end]).0 as i32) > 0 {
            m.rf(sp + 0x40) * m.rf(0x002c_3274)
        } else {
            m.call(0x0011_ee08, &[0, sp + 0x60, sp + 0x64], &[p, base, end]);
            let (tt, dd) = (m.rf(sp + 0x60), m.rf(sp + 0x64));
            if !(dd < m.rf(0x002c_327c)) || !(0.0 < tt) {
                continue;
            }
            m.rf(0x002c_3274) + dd
        };
        if f < best {
            win = m.r32(e);
            best = f;
        }
    }
    stack_pop(m, 0x70);
    if win != u32::MAX {
        m.call(0x001d_b390, &[o, win], &[]);
        m.call(0x001d_a9e8, &[o], &[]);
        let h = m.call(0x001f_bb90, &[0x35], &[]).0;
        m.call(0x001f_b2f8, &[h, 0], &[]);
    }
}

/// FUN_0011b340 (pure part): inverse of the 3x3 basis `r` (rows): cofactor rows c0 = r1 x r2, c1 = r2 x r0,
/// c2 = r0 x r1 (VOPMULA / VOPMSUB), det = r0 . c0, result rows = transpose(c) * (1 / det) (VDIV: det 0 -> FLT_MAX).
/// unk: the w lanes of the three result rows are whatever vf4..vf6 held before the call (xyz-only writes); 0 here.
pub fn inv3(r: &[V4; 3]) -> [V4; 3] {
    let c = [vu0::cross(r[1], r[2]), vu0::cross(r[2], r[0]), vu0::cross(r[0], r[1])];
    let q = vu0::vdiv(1.0, vu0::dot3(r[0], c[0]));
    std::array::from_fn(|i| vu0::scale([c[0][i], c[1][i], c[2][i], 0.0], q))
}
/// FUN_0011b340(out, m): out rows 0..2 = inverse of the 3x3 basis at `src` (see `inv3`); returns `out`.
pub fn ee_sub_11b340<M: QMem + ?Sized>(m: &mut M, out: u32, src: u32) -> u32 {
    let r = [m.rv(src), m.rv(src + 0x10), m.rv(src + 0x20)];
    for (i, v) in inv3(&r).into_iter().enumerate() {
        m.wv(out + 0x10 * i as u32, v);
    }
    out
}
/// FUN_0011b480(out, sin (128-bit a1, lane x), cos (a2), axis (a3)): Rodrigues rotation, 3 rows with w = 0
/// (`VSUB.w vfN, vf0, vf0`); the math is camera::math::FUN_0011b480 (row i = R e_i). Returns `out`.
pub fn ee_sub_11b480<M: QMem + ?Sized>(m: &mut M, out: u32, sn: V4, cs: V4, axis: V4) -> u32 {
    let r = crate::engine::camera::math::FUN_0011b480(sn[0], cs[0], [axis[0], axis[1], axis[2]]);
    for i in 0..3 {
        m.wv(out + 0x10 * i as u32, r[i]);
    }
    out
}

/// FEEL R86: FUN_00141968 calls made by the live image (jt, target, f12, f13) for the native player JT.
/// FEEL R89: + the issuing App's LiveQ id (live::CUR_QID; 0 = untagged, taken by any player).
pub static LAUNCHES_141968: std::sync::Mutex<Vec<(u64, u32, [f32; 3], f32, f32)>> = std::sync::Mutex::new(Vec::new());

/// FEEL R86: take the pending live-image FUN_00141968 launches. FEEL R89: only those of LiveQ `qid` (and
/// untagged ones); other Apps' launches stay queued for their own player (parallel tests share the queue).
/// SLY2_FEEL_R89QID=0 = R86 (take all).
pub fn take_launches_141968(qid: u64) -> Vec<(u32, [f32; 3], f32, f32)> {
    let all = std::env::var("SLY2_FEEL_R89QID").is_ok_and(|v| v == "0");
    let Ok(mut l) = LAUNCHES_141968.lock() else { return Vec::new() };
    let (mine, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut *l).into_iter().partition(|e| all || e.0 == 0 || e.0 == qid);
    *l = rest;
    mine.into_iter().map(|e| (e.1, e.2, e.3, e.4)).collect()
}

/// FUN_00141968(jt (a0), tgt (128-bit a1), f12, f13): p = pos(jt) (FUN_001a62e8) with z += jt[+0x248];
/// r = FUN_001d6848(p, tgt, *(jt[+0xe8] + 0x80), f12); jt vslot +0x11c(r (128-bit)); when jt[+0xdf4] == -1:
/// FUN_00195a00(jt + 0x2c0, 8, 0, 0), +0x300 = 3, +0x360 = r with +0x368 = 0, +0x2f4 = f13, and f13 == 0.0 clears
/// +0x2e8 / +0x2ec. obs: Ghidra's param order (param_3 = jt, param_a1) is wrong; disasm 0x141970 `move s1, a0`.
pub fn ee_sub_141968<M: QMem + ?Sized>(m: &mut M, jt: u32, tgt: V4, f12: f32, f13: f32) {
    // FEEL R86: hand the launch to the native player JT too (player::plugin::tick_player drains it)
    if std::env::var("SLY2_FEEL_R86PAD").map_or(true, |v| v != "0") {
        if let Ok(mut l) = LAUNCHES_141968.lock() {
            // FEEL R89: bounded: an App that stopped ticking leaves its launches behind -> drop the oldest
            if l.len() >= 64 {
                l.remove(0);
            }
            l.push((crate::engine::classes::q::live::CUR_QID.with(|c| c.get()), jt, [tgt[0], tgt[1], tgt[2]], f12, f13));
        }
    }
    let mut p = m.call(0x001a_62e8, &[jt], &[]).1;
    p[2] += m.rf(jt + 0x248);
    let e8 = m.r32(jt + 0xe8);
    let q = m.rv(e8 + 0x80);
    let r = m.call(0x001d_6848, &[], &[p, tgt, q, s(f12)]).1;
    crate::engine::classes::q::q6::vcall(m, jt, 0xc, 0x11c, &[], &[r]);
    if m.r32(jt + 0xdf4) == u32::MAX {
        m.call(0x0019_5a00, &[jt + 0x2c0, 8, 0, 0], &[]);
        m.w32(jt + 0x300, 3);
        m.wv(jt + 0x360, r);
        m.w32(jt + 0x368, 0);
        m.wf(jt + 0x2f4, f13);
        if f13 == m.rf(jt + 0x368) {
            m.w32(jt + 0x2ec, 0);
            m.w32(jt + 0x2e8, 0);
        }
    }
}

/// FUN_0017d608(&pos): drop pos onto the ground: frame (FUN_001ae478); (n, list) = FUN_001b44d8;
/// hit = FUN_0014cc58(0, pos + up * DAT_002be750, pos + up * DAT_002be754, n, list, &rec) with up = DAT_002a2dc0;
/// on a hit pos = rec[+0x10]; frame closed with 2; returns hit != 0.
/// obs: both endpoints are `VADDA ACC, pos, vf0 ; VMADDx` -> w lane gets +1.0.
pub fn ee_sub_17d608<M: QMem + ?Sized>(m: &mut M, pos: u32) -> bool {
    use crate::engine::classes::q::q8util::{frame_begin, frame_end};
    let f = frame_begin(m);
    let sp = stack_push(m, 0x70);
    m.call(0x001b_44d8, &[sp + 0x60, sp + 0x64], &[]);
    let (lo, hi) = (m.rf(0x002b_e750), m.rf(0x002b_e754));
    let up = rov(0x002a_2dc0);
    let p = m.rv(pos);
    let pw = vu0::add(p, [0.0, 0.0, 0.0, 1.0]);
    let a = vu0::add(pw, vu0::scale(up, lo));
    let b = vu0::add(pw, vu0::scale(up, hi));
    let (n, l) = (m.r32(sp + 0x60), m.r32(sp + 0x64));
    let hit = m.call(0x0014_cc58, &[0, n, l, sp], &[a, b]).0 != 0;
    if hit {
        let v = m.rv(sp + 0x10);
        m.wv(pos, v);
    }
    stack_pop(m, 0x70);
    frame_end(m, f, 2);
    hit
}

/// FUN_0017d8a8 (patrol Goto probe, a0 = goto record): +0xb4 = 0, +0x60 = +0x40, +0x70 = +0x50; unless +0xbc:
/// pos = FUN_001a62e8(+0x10); q = FUN_001c1ec0(&+0x40, pos (128-bit)); own = +0x10 -> +0xe8 -> +0xdc -> +0x10 (null
/// safe); r = FUN_001c0d70(q, 0); r == own -> done; r != 0: n = FUN_001c21d8(DAT_003e1a5c, +0x10, q, 1, 1, &+0xb8)
/// -> +0xb4, n > 0 done; else with +0xc0: n = FUN_001c21d8(.., 0, 1, ..) -> +0xb4; n != 0: target = +0xb8 ?
/// list[n - 1][+0x128] : own (and +0xb4 = 0); +0x60 / +0x64 = 0; FUN_001c0940(target, q, &+0x70, 1). Any other path
/// (r == 0, no hit): +0x70 = pos, +0x60 / +0x64 = 0.
pub fn ee_sub_17d8a8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0xb4, 0);
    let (a, b) = (m.rv(o + 0x40), m.rv(o + 0x50));
    m.wv(o + 0x60, a);
    m.wv(o + 0x70, b);
    if m.r32(o + 0xbc) != 0 {
        return;
    }
    let obj = m.r32(o + 0x10);
    let pos = m.call(0x001a_62e8, &[obj], &[]).1;
    let q = m.call(0x001c_1ec0, &[o + 0x40], &[pos]).1;
    let obj = m.r32(o + 0x10);
    let e8 = m.r32(obj + 0xe8);
    let dc = if e8 != 0 { m.r32(e8 + 0xdc) } else { 0 };
    let own = if dc != 0 { m.r32(dc + 0x10) } else { 0 };
    let r = m.call(0x001c_0d70, &[0], &[q]).0;
    let mut fallback = true;
    if r != 0 {
        if r == own {
            fallback = false;
        } else {
            let g = m.r32(0x003e_1a5c);
            let obj = m.r32(o + 0x10);
            let n = m.call(0x001c_21d8, &[g, obj, 1, 1, o + 0xb8], &[q]).0;
            m.w32(o + 0xb4, n);
            if (n as i32) > 0 {
                fallback = false;
            } else if m.r32(o + 0xc0) != 0 {
                let g = m.r32(0x003e_1a5c);
                let obj = m.r32(o + 0x10);
                let n = m.call(0x001c_21d8, &[g, obj, 0, 1, o + 0xb8], &[q]).0;
                m.w32(o + 0xb4, n);
                if n != 0 {
                    let t = if m.r32(o + 0xb8) == 0 {
                        m.w32(o + 0xb4, 0);
                        own
                    } else {
                        let e = m.r32(o + 0xb8 + (n.wrapping_sub(1) << 2));
                        m.r32(e + 0x128)
                    };
                    m.w32(o + 0x60, 0);
                    m.w32(o + 0x64, 0);
                    m.call(0x001c_0940, &[t, o + 0x70, 1], &[q]);
                    fallback = false;
                }
            }
        }
    }
    if fallback {
        m.wv(o + 0x70, pos);
        m.w32(o + 0x60, 0);
        m.w32(o + 0x64, 0);
    }
}

/// FUN_00192a28(&jt): for every entry s of the DAT_003e1444 chain (+0xfe0): e = FUN_00154180(s[+0xf64], jt); with
/// e, e[4] != 0 and !(s[+0xf64][+0xf8] & 1): d = |pos(s) - pos(jt)|; d > DAT_002bf638 -> FUN_00154080(s[+0xf64], e, 0);
/// else in a frame (FUN_001ae478): (n, list) = FUN_0020faa0(world, pos(s), pos(jt), 0x154568, s, ..);
/// FUN_0014cc58(0, pos(s), pos(jt), n, list, null) != 0 (occluded) -> FUN_00154080(s[+0xf64], e, 0); frame closed (2).
/// obs: the frame record is at sp + 1 (0x192b24, unaligned); emulated by `frame_begin`.
pub fn ee_sub_192a28<M: QMem + ?Sized>(m: &mut M, pj: u32) {
    use crate::engine::classes::q::q6::WORLD;
    use crate::engine::classes::q::q8util::{frame_begin, frame_end};
    let mut s0 = m.r32(0x003e_1444);
    while s0 != 0 {
        let f64_ = m.r32(s0 + 0xf64);
        let jt = m.r32(pj);
        let e = m.call(0x0015_4180, &[f64_, jt], &[]).0;
        let f64p = m.r32(s0 + 0xf64);
        if e != 0 && m.r32(e + 4) != 0 && m.r32(f64p + 0xf8) & 1 == 0 {
            let pa = m.call(0x001a_62e8, &[s0], &[]).1;
            let jt = m.r32(pj);
            let pb = m.call(0x001a_62e8, &[jt], &[]).1;
            let d = vu0::sub(pa, pb);
            let len = vu0::vsqrt(vu0::dot3(d, d));
            if m.rf(0x002b_f638) < len {
                let h = m.r32(s0 + 0xf64);
                m.call(0x0015_4080, &[h, e, 0], &[]);
            } else {
                let f = frame_begin(m);
                let sp = stack_push(m, 0x10);
                m.call(0x0020_faa0, &[WORLD, 0x0015_4568, s0, sp + 4, sp + 8], &[pa, pb]);
                let (n, l) = (m.r32(sp + 4), m.r32(sp + 8));
                stack_pop(m, 0x10);
                if m.call(0x0014_cc58, &[0, n, l, 0], &[pa, pb]).0 != 0 {
                    let h = m.r32(s0 + 0xf64);
                    m.call(0x0015_4080, &[h, e, 0], &[]);
                }
                frame_end(m, f, 2);
            }
        }
        s0 = m.r32(s0 + 0xfe0);
    }
}

/// FUN_001a9b28(a0, b1, b2, a3, t0, t1) (IntersectBodies sphere vs BSP): n = FUN_001a93c8(b1 radius (+8),
/// centre *(b1) + 0x70, b2, 0x7fffffff, a3, t0, 0x40, recs) (0x30-byte contact records {point, plane, +0x20, +0x24});
/// for each record from the last: d = plane.xyz . point - plane.w (VMSUBw); FUN_001a7c18(d, a0, b1, b2, point,
/// plane, rec[+0x20], 0, rec[+0x24], t1 (stack arg)). Two FUN_002564b8 temps are initialised per record and then
/// overwritten (kept as calls). obs: a2 is the int b2 (caller 0x1aa57c `move a2, s0`), not a quadword (Ghidra arg128).
/// QMem order: FUN_001a93c8 v = [radius, centre]; FUN_001a7c18 a = [a0, a1, a2, t1, t2, t3, stack], v = [f12, a3, t0].
pub fn ee_sub_1a9b28<M: QMem + ?Sized>(m: &mut M, a0: u32, b1: u32, b2: u32, a3: u32, t0: u32, t1: u32) {
    let sp = stack_push(m, 0xc40);
    let recs = sp;
    let (t_b0, t_a0) = (sp + 0xc00, sp + 0xc10);
    let p = m.r32(b1);
    let c = m.rv(p + 0x70);
    let r = m.rf(b1 + 8);
    let n = m.call(0x001a_93c8, &[b2, 0x7fff_ffff, a3, t0, 0x40, recs], &[s(r), c]).0 as i32;
    let mut i = n - 1;
    while i >= 0 {
        let rec = recs + 0x30 * i as u32;
        let pt = m.rv(rec);
        let pl = m.rv(rec + 0x10);
        m.call(0x0025_64b8, &[t_b0], &[]);
        m.call(0x0025_64b8, &[t_a0], &[]);
        let d = vu0::fix(vu0::dot3(pl, pt) - pl[3]);
        m.wv(t_a0, [1.0, 0.0, 0.0, 0.0]);
        m.wv(t_b0, [d, 0.0, 0.0, 0.0]);
        let (k20, k24) = (m.r32(rec + 0x20), m.r32(rec + 0x24));
        m.call(0x001a_7c18, &[a0, b1, b2, k20, 0, k24, t1], &[s(d), pt, pl]);
        i -= 1;
    }
    stack_pop(m, 0xc40);
}

/// FUN_001a9c40(a0, b1, b2, a3) (IntersectBodies sphere vs sphere): d = c1 - c2 (centres *(b) + 0x70), r = r1 + r2
/// (+8); when |d|^2 <= r^2: (len, n) = (0, DAT_002a2dc0) if |d|^2 < 1e-4 else (sqrt.s, d / len);
/// FUN_001a7c18(len - r, a0, b1, b2, c1 + n * -r1 (+1.0 on w: `VADDA ACC, c1, vf0`), n, 7, 0, 0, a3 (stack arg)).
pub fn ee_sub_1a9c40<M: QMem + ?Sized>(m: &mut M, a0: u32, b1: u32, b2: u32, a3: u32) {
    let (p1, p2) = (m.r32(b1), m.r32(b2));
    let c2 = m.rv(p2 + 0x70);
    let r = m.rf(b1 + 8) + m.rf(b2 + 8);
    let c1 = m.rv(p1 + 0x70);
    let d = vu0::sub(c1, c2);
    let l2 = vu0::dot3(d, d);
    if r * r < l2 {
        return;
    }
    let (len, n) = if l2 < f32::from_bits(0x38d1_b717) {
        (0.0, rov(0x002a_2dc0))
    } else {
        let l = l2.sqrt();
        (l, vu0::scale(d, vu0::vdiv(1.0, l)))
    };
    let r1 = m.rf(b1 + 8);
    let pt = vu0::add(vu0::add(c1, [0.0, 0.0, 0.0, 1.0]), vu0::scale(n, -r1));
    m.call(0x001a_7c18, &[a0, b1, b2, 7, 0, 0, a3], &[s(len - r), pt, n]);
}

/// save-data path request for FUN_001d8cc8: `{u32 count; u16 *path}` + the u16 key path (symbol ids / indices)
/// (obs: 0x1c7b68..0x1c7b84 / 0x1c80c8..0x1c80e0: `sw count,0(sp)`, `sw sp+0x10,4(sp)`, `sh key_i,0x10+2i(sp)`).
fn save_lookup<M: QMem + ?Sized>(m: &mut M, ms: u32, keys: &[u16]) -> u32 {
    let req = stack_push(m, 0x20);
    m.w32(req, keys.len() as u32);
    m.w32(req + 4, req + 0x10);
    for (i, &k) in keys.iter().enumerate() {
        m.w16(req + 0x10 + 2 * i as u32, k);
    }
    let r = m.call(0x0021_09b8, &[ms], &[]).0;
    let v = m.call(0x001d_8cc8, &[r, req], &[]).0;
    stack_pop(m, 0x20);
    v
}

/// FUN_001c7b18(i) (afClue[i] save slot): ms = FUN_00210a88 (LoadOptions_mission); -1 -> 0; FUN_0021ecd8(buf,
/// 0x2a7768, i) (formatted name, result unused by the body); key = FUN_001d8720(0x3e0eb0, "afClue" 0x2a7788);
/// FUN_001d8cc8(FUN_002109b8(ms), &{2, [key, (u16)i]}).
pub fn ee_sub_1c7b18<M: QMem + ?Sized>(m: &mut M, i: u32) -> u32 {
    let ms = m.call(0x0021_0a88, &[], &[]).0;
    if ms == u32::MAX {
        return 0;
    }
    let buf = stack_push(m, 0x20);
    m.call(0x0021_ecd8, &[buf, 0x002a_7768, i], &[]);
    stack_pop(m, 0x20);
    let key = m.call(0x001d_8720, &[0x003e_0eb0, 0x002a_7788], &[]).0 as u16;
    save_lookup(m, ms, &[key, i as u16])
}

/// FUN_001c7bb8(i): in a frame: v = FUN_001c7b18(i); 0 -> 0; else FUN_002581c0(&tmp, v, 0x3d4a60), result = tmp[+4],
/// FUN_00129ec8(&tmp) (Release); frame closed (2).
pub fn ee_sub_1c7bb8<M: QMem + ?Sized>(m: &mut M, i: u32) -> u32 {
    use crate::engine::classes::q::q8util::{frame_begin, frame_end};
    let f = frame_begin(m);
    let v = ee_sub_1c7b18(m, i);
    if v == 0 {
        frame_end(m, f, 2);
        return 0;
    }
    let tmp = stack_push(m, 0x10);
    m.call(0x0025_81c0, &[tmp, v, 0x003d_4a60], &[]);
    let r = m.r32(tmp + 4);
    m.call(0x0012_9ec8, &[tmp], &[]);
    stack_pop(m, 0x10);
    frame_end(m, f, 2);
    r
}

/// FUN_001c8090 (fVault save slot): ms = FUN_00210a88; -1 -> 0; FUN_001d8cc8(FUN_002109b8(ms),
/// &{1, [FUN_001d8720(0x3e0eb0, "fVault" 0x2a7790)]}).
pub fn ee_sub_1c8090<M: QMem + ?Sized>(m: &mut M) -> u32 {
    let ms = m.call(0x0021_0a88, &[], &[]).0;
    if ms == u32::MAX {
        return 0;
    }
    let key = m.call(0x001d_8720, &[0x003e_0eb0, 0x002a_7790], &[]).0 as u16;
    save_lookup(m, ms, &[key])
}

/// FUN_001e7d90 (bounding radius of a path in the +0xa0 basis): FUN_00231368(this) (BoneWorlds_xfm_world);
/// B = rows +0xa0/+0xb0/+0xc0. Polyline (+0xe0 == 0): max |B p|^2 over the +0xe8 points at +0xf4. Spline: start
/// with |B p0|^2, then per segment i (knots +0xec, out-tangents +0x144[i], in-tangents +0x140[i + 1]):
/// dk = k[i+1] - k[i]; candidates |B p_i + B tout * dk/3|^2, |B p_{i+1} + B tin * -dk/3|^2, |B p_{i+1}|^2 (Bezier hull).
/// Returns sqrt.s(max).
pub fn ee_sub_1e7d90<M: QMem + ?Sized>(m: &mut M, o: u32) -> f32 {
    m.call(0x0023_1368, &[o], &[]);
    let b = [m.rv(o + 0xa0), m.rv(o + 0xb0), m.rv(o + 0xc0)];
    let l2 = |v: V4| vu0::dot3(v, v);
    let mut mx = 0.0f32;
    let pts = m.r32(o + 0xf4);
    if m.r32(o + 0xe0) == 0 {
        let n = m.r32(o + 0xe8) as i32;
        for i in 0..n.max(0) as u32 {
            let v = l2(vu0::mul3(&b, m.rv(pts + 0x10 * i)));
            if mx < v {
                mx = v;
            }
        }
    } else {
        let mut prev = vu0::mul3(&b, m.rv(pts));
        mx = l2(prev);
        let n1 = m.r32(o + 0xe8) as i32 - 1;
        let (tout, knots) = (m.r32(o + 0x144), m.r32(o + 0xec));
        for i in 0..n1.max(0) as u32 {
            let dk = m.rf(knots + 4 * i + 4) - m.rf(knots + 4 * i);
            let cur = vu0::mul3(&b, m.rv(pts + 0x10 * (i + 1)));
            let t = vu0::mul3(&b, m.rv(tout + 0x10 * i));
            let a = dk * f32::from_bits(0x3eaa_aaab);
            let c1: V4 = std::array::from_fn(|k| prev[k] + t[k] * a);
            let tin_base = m.r32(o + 0x140);
            let t2 = vu0::mul3(&b, m.rv(tin_base + 0x10 * i + 0x10));
            let bb = dk * f32::from_bits(0xbeaa_aaab);
            let c2: V4 = std::array::from_fn(|k| cur[k] + t2[k] * bb);
            for v in [l2(c1), l2(c2), l2(cur)] {
                if mx < v {
                    mx = v;
                }
            }
            prev = cur;
        }
    }
    mx.sqrt()
}

/// FUN_00203e08 (patrol rig: per-point approach velocities): with src = +0x258 and dst = +0x25c:
/// M = FUN_0011c318(src[0] + 0x40)^-1 composed with dst[0] + 0x40 (FUN_0011c168); for each of dst[+0x1c] points
/// p (dst[+0x7c], quads): t = M (p, 1) (xyz); per lane j < 3: v[j] = (PosApproachTrapezoid(cur[j], t[j], dt,
/// 0x2c3c68, null) - cur[j]) / dt with cur = src[+0x7c] and v = *src[+0x10] (both stride 0x10 per point).
pub fn ee_sub_203e08<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let src = m.r32(o + 0x258);
    if src == 0 {
        return;
    }
    let dst = m.r32(o + 0x25c);
    if dst == 0 {
        return;
    }
    let mut pts = m.r32(dst + 0x7c);
    let mut cur = m.r32(src + 0x7c);
    let v10 = m.r32(src + 0x10);
    let mut vel = m.r32(v10);
    let sp = stack_push(m, 0x80);
    let s0 = m.r32(src);
    m.call(0x0011_c318, &[sp + 0x40, s0 + 0x40], &[]);
    let d = m.r32(o + 0x25c);
    let d0 = m.r32(d);
    m.call(0x0011_c168, &[sp, sp + 0x40, d0 + 0x40], &[]);
    let mm = m.rm(sp);
    stack_pop(m, 0x80);
    let d = m.r32(o + 0x25c);
    let n = m.r32(d + 0x1c) as i32;
    for _ in 0..n.max(0) {
        let p = m.rv(pts);
        let t = vu0::mul4(&mm, [p[0], p[1], p[2], 1.0]);
        for j in 0..3u32 {
            let c = m.rf(cur + 4 * j);
            let dt = m.rf(DT);
            let f = m.call(0x0011_d018, &[0x002c_3c68, 0], &[s(c), s(t[j as usize]), s(dt)]).1[0];
            let c = m.rf(cur + 4 * j);
            let dt = m.rf(DT);
            m.wf(vel + 4 * j, (f - c) / dt);
        }
        pts += 0x10;
        cur += 0x10;
        vel += 0x10;
    }
}

/// FUN_0020f1c8 (fog/tint color refresh): hsv = (+0x3e10, +0x3e14/255, +0x3e18/255) -> rgb via FUN_0011dd90
/// (stack quad, w lane stale); rgb *= +0x3e20/255 (vmulx all lanes); FUN_001ed2d8(&rgb) in place, then
/// +0x3e28..+0x3e30 = rgb.xyz; +0x3e34 = +0x3e24/255 (clamp(that + 0.3, 0x2bbb60) when +0x3e38); ++(+0x3d5c).
pub fn ee_sub_20f1c8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    let k = f32::from_bits(0x3b80_8081);
    let sp = stack_push(m, 0x30);
    let h = m.r32(o + 0x3e10);
    let sat = m.rf(o + 0x3e14) * k;
    let val = m.rf(o + 0x3e18) * k;
    m.w32(sp, h);
    m.wf(sp + 4, sat);
    m.wf(sp + 8, val);
    m.call(0x0011_dd90, &[sp, sp + 0x10], &[]);
    let a = m.rf(o + 0x3e20) * k;
    let c = vu0::scale(m.rv(sp + 0x10), a);
    m.wf(sp + 0x20, a); // qmtc2 scratch (only x defined)
    m.wv(sp + 0x10, c);
    m.call(0x001e_d2d8, &[sp + 0x10], &[]);
    for i in 0..3 {
        let w = m.r32(sp + 0x10 + 4 * i);
        m.w32(o + 0x3e28 + 4 * i, w);
    }
    stack_pop(m, 0x30);
    let f = m.rf(o + 0x3e24) * k;
    m.wf(o + 0x3e34, f);
    if m.r32(o + 0x3e38) != 0 {
        let r = callf(m, 0x0011_d948, &[0x002b_bb60], &[s(f + f32::from_bits(0x3e99_999a))]);
        m.wf(o + 0x3e34, r);
    }
    let n = m.r32(o + 0x3d5c).wrapping_add(1);
    m.w32(o + 0x3d5c, n);
}

/// FUN_0020f2b8: `sq a1, 0x3e10(a0)` (128-bit arg = v[0], raw lanes) then FUN_0020f1c8(a0).
pub fn ee_sub_20f2b8<M: QMem + ?Sized>(m: &mut M, o: u32, q: V4) {
    m.wv(o + 0x3e10, q);
    ee_sub_20f1c8(m, o);
}

/// FUN_002124f0 (HUD label save-stream load; the decompile's u128 param is just `this` in a0): mx = max of
/// node+0x48 over the DAT_003e106c list (+0x4c next, +0x40 label) whose label+0x58 is nonzero and, when it
/// is 1, whose label+0x60 is within 2 frames of DAT_002be538; FUN_00212a70(label, 0) for every node with
/// node+0x44 >= mx and label+0x58 != 0; DAT_003e1c7c = mx. Then opens the stream described by `this`+0xc0
/// (FileLocation {p[0] ^ p[0x34] + 0xca0, p[4] ^ p[0x3c], 2}; skipped when that size is 0) through a 0x10000
/// heap BinaryInputStream with lz (0x4000, 0x70000000); on magic == DAT_002a8ae8 (4 bytes) and u16 version 6:
/// p+0x44 = DAT_003e1c7c, FUN_00214398(this, s, this, this+0x50, this+0x54), FUN_00214460(this, s, this),
/// p+0x48 = DAT_003e1c7c. Always closes the stream (FUN_00184950(s, 2)) and the heap (FUN_001ae4a0(sp, 2)).
pub fn ee_sub_2124f0<M: QMem + ?Sized>(m: &mut M, o: u32) {
    const LIST: u32 = 0x003e_106c;
    const MX: u32 = 0x003e_1c7c;
    let mut mx = 0i32;
    let frame = m.r32(0x002b_e538) as i32;
    let mut n = m.r32(LIST);
    while n != 0 {
        let l = m.r32(n + 0x40);
        let st = m.r32(l + 0x58);
        let fresh = st != 1 || frame.wrapping_sub(m.r32(l + 0x60) as i32) < 2;
        if st != 0 && fresh {
            mx = mx.max(m.r32(n + 0x48) as i32);
        }
        n = m.r32(n + 0x4c);
    }
    let mut n = m.r32(LIST);
    while n != 0 {
        let l = m.r32(n + 0x40);
        if (m.r32(n + 0x44) as i32) >= mx && m.r32(l + 0x58) != 0 {
            m.call(0x0021_2a70, &[l, 0], &[]);
        }
        n = m.r32(n + 0x4c);
    }
    m.w32(MX, mx as u32);
    let sp = stack_push(m, 0x1a0);
    let (heap, req, strm, loc) = (sp, sp + 0x10, sp + 0x20, sp + 0x90);
    m.call(0x001a_e478, &[heap], &[]);
    m.w32(req, 0x10000);
    m.w32(req + 4, 0x40);
    let buf = m.call(0x001a_e318, &[req], &[]).0;
    m.call(0x0018_48a8, &[strm, 0x10000, buf, 0], &[]);
    m.w32(loc + 8, 2);
    m.call(0x0025_8248, &[loc], &[]);
    let p = m.r32(o + 0xc0);
    let ok = if p == 0 {
        false
    } else {
        let off = (m.r32(p) ^ m.r32(p + 0x34)).wrapping_add(0xca0);
        let size = m.r32(p + 4) ^ m.r32(p + 0x3c);
        m.w32(loc + 8, 2);
        m.w32(loc, off);
        m.w32(loc + 4, size);
        size != 0
    };
    if ok && m.call(0x0018_4d60, &[strm, loc], &[]).0 != 0 {
        m.call(0x0011_79d8, &[strm, 0x4000, 0x7000_0000], &[]);
        for i in 0..4 {
            let b = m.call(0x0011_7bd0, &[strm], &[]).0;
            m.w8(req + i, b as u8);
        }
        let ver = m.call(0x0011_7c20, &[strm], &[]).0;
        if m.call(0x0023_3470, &[req, 0x002a_8ae8, 4], &[]).0 == 0 && ver == 6 {
            let v = m.r32(MX);
            let p = m.r32(o + 0xc0);
            m.w32(p + 0x44, v);
            m.call(0x0021_4398, &[o, strm, o, o + 0x50, o + 0x54], &[]);
            m.call(0x0021_4460, &[o, strm, o], &[]);
            let v = m.r32(MX);
            let p = m.r32(o + 0xc0);
            m.w32(p + 0x48, v);
        }
    }
    m.call(0x0018_4950, &[strm, 2], &[]);
    m.call(0x001a_e4a0, &[heap, 2], &[]);
    stack_pop(m, 0x1a0);
}

/// FUN_00229468 (water/attachment probe: downward ray from `other` against `this`'s collision set):
/// p = FUN_001a62e8(other) (quad); a = p with a.z = (other+0xec)+8, or, when FUN_00132b10(other, 0x33),
/// p.z + other+0x248 (max'd with other+0x238 when other+0x208); b = p with b.z = (other+0xec)+0x18; heap of
/// (this+0xe0) * 4 entries x 0x10; cb = FUN_001f7e08(this, 0x229d60, 0, buf); hit = FUN_0014cdb0(a, b, cb, buf,
/// 0x10, &rec) (quads in v). Miss: *out_n = DAT_002a2dc0 (if out_n), 0.0. Hit: *out_p = rec+0x10 (if out_p);
/// *out_n = rec+0x30, reset to DAT_002a2dc0 when it is FFloatsNear (1e-4) DAT_002a2dd0 lane-wise in x, y
/// (prot3w) and z (pextuw); returns rec+0x44 - rec+0x40. Heap freed (FUN_001ae4a0(heap, 2)) on both paths.
pub fn ee_sub_229468<M: QMem + ?Sized>(m: &mut M, o: u32, other: u32, out_p: u32, out_n: u32) -> f32 {
    let p = m.call(0x001a_62e8, &[other], &[]).1;
    let sp = stack_push(m, 0x640);
    m.wv(sp, p);
    m.wv(sp + 0x10, p);
    if m.call(0x0013_2b10, &[other, 0x33], &[]).0 != 0 {
        let z = m.rf(sp + 8) + m.rf(other + 0x248);
        m.wf(sp + 8, z);
        if m.r32(other + 0x208) != 0 {
            let lo = m.rf(other + 0x238);
            m.wf(sp + 8, lo.max(z));
        }
    } else {
        let e = m.r32(other + 0xec);
        let z = m.rf(e + 8);
        m.wf(sp + 8, z);
    }
    let e = m.r32(other + 0xec);
    let z = m.rf(e + 0x18);
    m.wf(sp + 0x18, z);
    let heap = sp + 0x20;
    m.call(0x001a_e478, &[heap], &[]);
    let n = m.r32(o + 0xe0) << 2;
    m.w32(sp + 0x30, n);
    m.w32(sp + 0x34, 0x10);
    let buf = m.call(0x001a_e318, &[sp + 0x30], &[]).0;
    let cb = m.call(0x001f_7e08, &[o, 0x0022_9d60, 0, buf], &[]).0;
    let (a, b) = (m.rv(sp), m.rv(sp + 0x10));
    let rec = sp + 0x40;
    let hit = m.call(0x0014_cdb0, &[cb, buf, 0x10, rec], &[a, b]).0;
    let r = if hit == 0 {
        if out_n != 0 {
            let d = m.rv(0x002a_2dc0);
            m.wv(out_n, d);
        }
        0.0
    } else {
        if out_p != 0 {
            let v = m.rv(rec + 0x10);
            m.wv(out_p, v);
        }
        if out_n != 0 {
            let nrm = m.rv(rec + 0x30);
            m.wv(out_n, nrm);
            let rf = m.rv(0x002a_2dd0);
            let mut near = true;
            for i in 0..3 {
                if m.call(0x0011_d4d0, &[], &[s(rf[i]), s(nrm[i]), s(f32::from_bits(0x38d1_b717))]).0 == 0 {
                    near = false;
                    break;
                }
            }
            if near {
                let d = m.rv(0x002a_2dc0);
                m.wv(out_n, d);
            }
        }
        m.rf(rec + 0x44) - m.rf(rec + 0x40)
    };
    m.call(0x001a_e4a0, &[heap, 2], &[]);
    stack_pop(m, 0x640);
    r
}

/// FUN_00158728 (motor pgs: per-constraint body deltas): for both bodies b = (z+0x50)+8+0x40i: vslot 67
/// (b, y, z, x, i); z+0x54 = heap((g+0x50) x 0x10); per constraint c = (g+0x54)[i]: for both ends e = c+0x40j,
/// rec_j = vslot 68 (e+8, z or z+0x20 by which body e+8 is, q(e+0x10) (quad arg), &rec_j), or rec_j.xyz = 0
/// when e+8 is null / neither body (w left stale); z+0x54[i] = rec_0 - rec_1 (vsub.xyzw). y+0x18 =
/// heap((g+4) x 4): per row r = g+0x60 + 0x60k: (r+0x40) . z+0x54[(r+0x50)+0xa8] (vmul.xyz, vadday, vmaddz).
pub fn ee_sub_158728<M: QMem + ?Sized>(m: &mut M, g: u32, x: u32, y: u32, z: u32) {
    let bodies = m.r32(z + 0x50);
    for i in 0..2u32 {
        let b = m.r32(bodies + 8 + 0x40 * i);
        if b != 0 {
            vslot(m, b, 67, &[y, z, x, i], &[]);
        }
    }
    let sp = stack_push(m, 0x30);
    let n = m.r32(g + 0x50);
    m.w32(sp, n << 4);
    m.w32(sp + 4, 0x10);
    let mut out = m.call(0x001a_e318, &[sp], &[]).0;
    m.w32(z + 0x54, out);
    let mut i = 0;
    while (i as i32) < m.r32(g + 0x50) as i32 {
        let cl = m.r32(g + 0x54);
        let mut c = m.r32(cl + 4 * i);
        for j in 0..2u32 {
            let rec = sp + 0x10 + 0x10 * j;
            let e = m.r32(c + 8);
            let side = if e == 0 {
                None
            } else if e == m.r32(bodies + 8) {
                Some(z)
            } else if e == m.r32(bodies + 0x48) {
                Some(z + 0x20)
            } else {
                None
            };
            match side {
                Some(a1) => {
                    let q = m.rv(c + 0x10);
                    vslot(m, e, 68, &[a1, rec], &[q]);
                }
                None => {
                    for k in 0..3 {
                        m.w32(rec + 4 * k, 0);
                    }
                }
            }
            c += 0x40;
        }
        let d = vu0::sub(m.rv(sp + 0x10), m.rv(sp + 0x20));
        m.wv(out, d);
        out += 0x10;
        i += 1;
    }
    let n = m.r32(g + 4);
    m.w32(sp, n << 2);
    m.w32(sp + 4, 0x10);
    let mut dst = m.call(0x001a_e318, &[sp], &[]).0;
    stack_pop(m, 0x30);
    m.w32(y + 0x18, dst);
    let mut k = 0;
    while (k as i32) < m.r32(g + 4) as i32 {
        let r = m.r32(g + 0x60) + 0x60 * k;
        let v = m.rv(r + 0x40);
        let rb = m.r32(r + 0x50);
        let idx = m.r32(rb + 0xa8);
        let dl = m.r32(z + 0x54);
        let d = m.rv(dl + (idx << 4));
        let t = vu0::mul(v, d);
        m.wf(dst, vu0::fix(t[0] + t[1] + t[2]));
        dst += 4;
        k += 1;
    }
}

/// FUN_0018f958 (3-axis point constraint impulse solve between body `a` and `b`): pos = [FUN_001a62e8(a),
/// *pq]; for each axis k (DAT_002a2d00 + 0x10k, negated for the second body) and body i: J[k][i] =
/// (FUN_001faa00(body, axis), FUN_001faa48(body, pos_i, axis)) (quad pairs at sp+0x40+0x20k+0x60i) and
/// vel_i = vslot 68 (a, &J[k][i], *pq (quad), &vel_i); d_k = vel_0 - vel_1. K[k][j] = axis_j . d_k (3x3,
/// stride 0xc), r = rhs[0..3]; x = FUN_001198f8(3, FUN_001195e0(3, K) (inverse), r); out[2i + h] =
/// sum_k J[k][i].h * x_k (all lanes) for i, h in 0..2.
pub fn ee_sub_18f958<M: QMem + ?Sized>(m: &mut M, a: u32, b: u32, pq: u32, rhs: u32, out: u32) {
    const AX: u32 = 0x002a_2d00;
    let p = m.call(0x001a_62e8, &[a], &[]).1;
    let sp = stack_push(m, 0x1e0);
    m.w32(sp, a);
    m.w32(sp + 4, b);
    let q = m.rv(pq);
    m.wv(sp + 0x10, p);
    m.wv(sp + 0x20, q);
    let (kmat, kinv, rv, xv) = (sp + 0x150, sp + 0x180, sp + 0x1b0, sp + 0x1c0);
    for k in 0..3u32 {
        for i in 0..2u32 {
            let body = m.r32(sp + 4 * i);
            let mut axis = m.rv(AX + 0x10 * k);
            m.wv(sp + 0x30, axis);
            if i == 1 {
                axis = vu0::scale(axis, -1.0);
                m.wf(sp + 0x1d0, -1.0); // qmtc2 scratch (only x defined)
                m.wv(sp + 0x30, axis);
            }
            let jr = sp + 0x40 + 0x20 * k + 0x60 * i;
            let r = m.call(0x001f_aa00, &[body], &[axis]).1;
            m.wv(jr, r);
            let pos = m.rv(sp + 0x10 + 0x10 * i);
            let axis = m.rv(sp + 0x30);
            let r = m.call(0x001f_aa48, &[body], &[pos, axis]).1;
            m.wv(jr + 0x10, r);
            let q = m.rv(pq);
            vslot(m, a, 68, &[jr, sp + 0x100 + 0x10 * i], &[q]);
        }
        let d = vu0::sub(m.rv(sp + 0x100), m.rv(sp + 0x110));
        m.wv(sp + 0x120 + 0x10 * k, d);
    }
    for k in 0..3u32 {
        for j in 0..3u32 {
            let t = vu0::mul(m.rv(AX + 0x10 * j), m.rv(sp + 0x120 + 0x10 * k));
            m.wf(kmat + 0xc * k + 4 * j, vu0::fix(t[0] + t[1] + t[2]));
        }
        let r = m.r32(rhs + 4 * k);
        m.w32(rv + 4 * k, r);
    }
    m.call(0x0011_95e0, &[3, kmat, kinv], &[]);
    m.call(0x0011_98f8, &[3, kinv, rv, xv], &[]);
    let x = [m.rf(xv), m.rf(xv + 4), m.rf(xv + 8), 0.0];
    for i in 0..2u32 {
        for h in 0..2u32 {
            let base = sp + 0x40 + 0x60 * i + 0x10 * h;
            let rows = [m.rv(base), m.rv(base + 0x20), m.rv(base + 0x40)];
            let v = vu0::mul3(&rows, x);
            m.wv(out + 0x20 * i + 0x10 * h, v);
        }
    }
    stack_pop(m, 0x1e0);
}

/// project the Jt slot-32 vector off the FUN_001a6318 normal (+0x10) and hand it to vslot 34 (quad arg)
fn jt_flatten_1a0720<M: QMem + ?Sized>(m: &mut M, jt: u32, sp: u32) {
    let v = vslot(m, jt, 32, &[], &[]).1;
    m.wv(sp + 0x20, v);
    let n = m.call(0x001a_6318, &[jt], &[]).0;
    let nn = m.rv(n + 0x10);
    let v = m.rv(sp + 0x20);
    let t = vu0::mul(v, nn);
    let d = vu0::fix(t[0] + t[1] + t[2]);
    let r = vu0::sub(v, vu0::scale(nn, d));
    m.wv(sp + 0x10, r);
    vslot(m, jt, 34, &[], &[r]);
}

/// FUN_001a0720 (Jt stick input -> +0x190 move velocity): when `flag`: FUN_002046f0(jt, +0x10dc target, or its
/// +0x494 when +0x10e4 == 3); (x, y) = FUN_0011e1a0(FUN_0011cf60(ang - +0x1a8), mag, 0) (quad result);
/// state s: y > DAT_002bf9f0 -> 1 (from 0: y = 0 and the slot-32 vector is flattened against the ground
/// normal into vslot 34); y < -DAT_002bf9f0 -> 0 (same flatten when coming from 1); else s = +0x1038 (-1 ->
/// 0). +0x190 = s ? tbl.30 * y + tbl.70 * max(x, 0) + tbl.b0 * -min(x, 0) : tbl.10 * -y + tbl.50 * max(x, 0)
/// + tbl.90 * -min(x, 0); +0x198 = 0; +0x190 *= (+0x1030)+0x40 (all lanes); +0x1038 = s.
pub fn ee_sub_1a0720<M: QMem + ?Sized>(m: &mut M, jt: u32, tbl: u32, flag: u32, ang: f32, mag: f32) {
    if flag == 0 {
        return;
    }
    let t = m.r32(jt + 0x10dc);
    if t != 0 {
        let t = if m.r32(jt + 0x10e4) == 3 { m.r32(t + 0x494) } else { t };
        m.call(0x0020_46f0, &[jt, t], &[]);
    }
    let h = m.rf(jt + 0x1a8);
    let a = callf(m, 0x0011_cf60, &[], &[s(ang - h)]);
    let r = m.call(0x0011_e1a0, &[], &[s(a), s(mag), s(0.0)]).1;
    let sp = stack_push(m, 0x30);
    m.wv(sp, r);
    let th = m.rf(0x002b_f9f0);
    let y = m.rf(sp + 4);
    let st = if th < y {
        if m.r32(jt + 0x1038) == 0 {
            m.wf(sp + 4, 0.0);
            jt_flatten_1a0720(m, jt, sp);
        }
        1
    } else if y < -th {
        if m.r32(jt + 0x1038) == 1 {
            m.wf(sp + 4, 0.0);
            jt_flatten_1a0720(m, jt, sp);
        }
        0
    } else {
        let v = m.r32(jt + 0x1038);
        if v == u32::MAX {
            0
        } else {
            v
        }
    };
    let x = m.rf(sp);
    let y = m.rf(sp + 4);
    let (px, nx) = (0f32.max(x), -(0f32.min(x)));
    let v = if st == 1 {
        let rows = [m.rv(tbl + 0x30), m.rv(tbl + 0x70), m.rv(tbl + 0xb0)];
        vu0::mul3(&rows, [y, px, nx, 0.0])
    } else {
        let rows = [m.rv(tbl + 0x10), m.rv(tbl + 0x50), m.rv(tbl + 0x90)];
        vu0::mul3(&rows, [-y, px, nx, 0.0])
    };
    m.wv(jt + 0x190, v);
    stack_pop(m, 0x30);
    m.w32(jt + 0x198, 0);
    let kp = m.r32(jt + 0x1030);
    let k = m.rf(kp + 0x40);
    let v = vu0::scale(m.rv(jt + 0x190), k);
    m.w32(jt + 0x1038, st);
    m.wv(jt + 0x190, v);
}

/// FUN_00199298 (when `this`+0x1c == 0x11: pull the contacts with the player's ground object out of every
/// contact bucket): g = (((this+0xc8)+4)+0xec)+0x60 (none when +0xec is null); any = vslot 61 (g); for each
/// bucket b of ((*this)+0xec)+0x64 (b[0] head, b[4] next, *b[8] = b[0] afterwards) walk the +0xac chain:
/// type 0xb (+0x9c) contacts stay; otherwise the other side o = c+0x48 if *this == c+8 else c+8 decides:
/// with `any`, vslot 61 (o) != 0 unlinks; without, o == g unlinks. Unlinked contacts are pushed (via +0xac)
/// onto a list handed to FUN_001a7e10(0x3dd490, list).
pub fn ee_sub_199298<M: QMem + ?Sized>(m: &mut M, o: u32) {
    if m.r32(o + 0x1c) != 0x11 {
        return;
    }
    let c8 = m.r32(o + 0xc8);
    let c8 = m.r32(c8 + 4);
    let q = m.r32(c8 + 0xec);
    if q == 0 {
        return;
    }
    let g = m.r32(q + 0x60);
    let any = vslot(m, g, 61, &[], &[]).0;
    let mut out = 0u32;
    let me = m.r32(o);
    let me = m.r32(me + 0xec);
    let mut b = m.r32(me + 0x64);
    while b != 0 {
        let mut link = b;
        let mut c = m.r32(b);
        while c != 0 {
            let keep = if m.r32(c + 0x9c) == 0xb {
                true
            } else {
                let side = if m.r32(o) == m.r32(c + 8) { 0x48 } else { 8 };
                let other = m.r32(c + side);
                if any != 0 {
                    vslot(m, other, 61, &[], &[]).0 == 0
                } else {
                    other != g
                }
            };
            if keep {
                link = c + 0xac;
            } else {
                let nx = m.r32(c + 0xac);
                m.w32(link, nx);
                m.w32(c + 0xac, out);
                out = c;
            }
            c = m.r32(link);
        }
        let tail = m.r32(b + 8);
        let h = m.r32(b);
        m.w32(tail, h);
        b = m.r32(b + 4);
    }
    m.call(0x001a_7e10, &[0x003d_d490, out], &[]);
}
