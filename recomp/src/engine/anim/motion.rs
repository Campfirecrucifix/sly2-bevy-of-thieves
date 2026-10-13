//! WP-D2: bone motion = per-bone velocity matrices (FUN_0022cdc0 family) and the VU0 trig
//! helpers they use. Matrix conventions as in [`super::pose`].

use super::keys::{Segment, V3};
use super::pose::{ee_sub_11c168, ee_sub_11c318, ee_sub_22cc00, Mat4};

/// FUN_0011d608(a, &sin, &cos): polynomial sine (range-reduced by pi, clamped to [-1,1]),
/// cosine = sqrt(1 - s^2) with the reduction's sign.
pub fn ee_sub_11d608(a: f32) -> (f32, f32) {
    // 0x0011d608
    let f = (a + 1.5707964) * 0.31830987;
    let mut q = f as i32 as u32; // cvt.w.s truncation
    if f < 0.0 {
        q = q.wrapping_sub(1);
    }
    let mut x = a - (q as i32) as f32 * 3.1415927;
    if q & 1 != 0 {
        x = -x;
    }
    let x2 = x * x;
    let x5 = x * x2 * x2;
    let x7 = x5 * x2;
    let s = (x + x * x2 * -0.16666657 + x5 * 0.0083330255 + x7 * -0.00019807414 + x7 * x2 * 2.601887e-06).clamp(-1.0, 1.0);
    let mut c = (1.0 - s * s).sqrt();
    if q & 1 != 0 {
        c = -c;
    }
    (s, c)
}

/// FUN_0011cfe0(x, lim): clamp to [-lim, lim].
pub fn ee_sub_11cfe0(x: f32, lim: f32) -> f32 {
    // 0x0011cfe0
    if x <= lim {
        if x < -lim { -lim } else { x }
    } else {
        lim
    }
}

/// FUN_0011bff0: rotation matrix -> euler (x, y, z), y = asin(-m[0][2]); gimbal lock at
/// |y| >= 1.5706964 gives x = 0. Assumes an orthonormal basis.
pub fn ee_sub_11bff0(m: &Mat4) -> V3 {
    // 0x0011bff0
    let y = ee_sub_11cfe0(-m[0][2], 1.0).asin();
    if y < 1.5706964 {
        if -1.5706964 < y {
            [m[1][2].atan2(m[2][2]), y, m[0][1].atan2(m[0][0])]
        } else {
            [0.0, y, (-m[1][0]).atan2(-m[2][0])]
        }
    } else {
        [0.0, y, (-m[1][0]).atan2(m[2][0])]
    }
}

/// Runtime bone (FUN_0022d7a0 entry, 0x100 bytes): the fields the motion code reads.
#[derive(Clone, Debug, PartialEq)]
pub struct BoneState {
    /// +0x00: local matrix (rows 0-2 basis, row 3 translation)
    pub local: Mat4,
    /// +0x40: world matrix
    pub world: Mat4,
    /// +0x94: bone kind (which matrix part the channel drives)
    pub kind: i32,
    /// +0x98: motion mode
    pub motion: u16,
    /// +0x80..+0xfc channel segment (+0x9a = seg.mode)
    pub seg: Segment,
    /// **(+0x88): channel time; **(+0x8c): rate
    pub t: f32,
    pub rate: f32,
    /// +0xa0
    pub parent: Option<usize>,
    /// +0xa8 (kinds 8/0xe)
    pub attach: Option<usize>,
    /// +0xb0: stored velocity (translation or angular, see FUN_002303a0)
    pub vel: V3,
}

impl BoneState {
    pub fn new(kind: i32, local: Mat4) -> Self {
        let seg = Segment { t0: 0.0, t1: 1.0, p0: [0.0; 3], v0: [0.0; 3], p1: [0.0; 3], v1: [0.0; 3], mode: 0, rewired_time: false };
        BoneState { local, world: local, kind, motion: 0, seg, t: 0.0, rate: 1.0, parent: None, attach: None, vel: [0.0; 3] }
    }
}

/// FUN_0022ca18: channel value read back from the local matrix (inverse of FUN_0022cc00;
/// kinds 6/7 = euler via FUN_0011bff0).
pub fn ee_sub_22ca18(b: &BoneState) -> V3 {
    // 0x0022ca18
    let m = &b.local;
    match b.kind {
        1 | 4 | 5 | 9 | 10 => [m[3][0], m[3][1], m[3][2]],
        6 | 7 => ee_sub_11bff0(m),
        0xb => [m[1][0], m[2][0], m[2][1]],
        0xc => [m[0][0], m[1][1], m[2][2]],
        _ => [0.0; 3], // original returns whatever a2 held
    }
}

/// FUN_0022cad0(euler, euler rate) -> angular velocity.
pub fn ee_sub_22cad0(e: V3, r: V3) -> V3 {
    // 0x0022cad0
    let (sy, cy) = ee_sub_11d608(e[1]);
    let (sz, cz) = ee_sub_11d608(e[2]);
    [cy * cz * r[0] - sz * r[1], cy * sz * r[0] + cz * r[1], -sy * r[0] + r[2]]
}

/// FUN_0022ccd0(out, bone, m) with angular velocity `w` (a3): skew(w) * m on the 3x3 part,
/// translation row and w column zero. skew rows: (0, z, -y), (-z, 0, x), (y, -x, 0).
pub fn ee_sub_22ccd0(w: V3, m: &Mat4) -> Mat4 {
    // 0x0022ccd0
    let mut s = [[0.0f32; 4]; 4];
    s[0][1] = w[2];
    s[2][0] = w[1];
    s[2][1] = -w[0];
    s[1][0] = -w[2];
    s[1][2] = w[0];
    s[0][2] = -w[1];
    let mut o = [[0.0f32; 4]; 4];
    for j in 0..3 {
        for r in 0..3 {
            o[j][r] = (0..3).map(|i| m[j][i] * s[i][r]).sum();
        }
    }
    o
}

/// FUN_0022cb90(out, bone, m, euler, euler rate) = FUN_0022ccd0(FUN_0022cad0(e, r), m).
pub fn ee_sub_22cb90(m: &Mat4, e: V3, r: V3) -> Mat4 {
    // 0x0022cb90
    ee_sub_22ccd0(ee_sub_22cad0(e, r), m)
}

fn zero() -> Mat4 {
    [[0.0; 4]; 4]
}

/// FUN_00117120 value / velocity of the bone's mode-1 segment.
fn herm(b: &BoneState) -> (V3, V3) {
    let s = &b.seg;
    let (p, v, _) = super::keys::ee_sub_117120(s.t1 - s.t0, b.t - s.t0, b.rate, s.p0, s.v0, s.p1, s.v1);
    (p, v)
}

/// Motion modes (+0x98) with no motion (FUN_0022cdc0 first switch).
pub fn motion_is_static(m: u16) -> bool {
    matches!(m, 0 | 3 | 10 | 0xc..=0x10 | 0x13)
}

/// FUN_0022cdc0(out, bone): the bone's local VELOCITY matrix (d local / dt), not its pose:
/// FUN_002303a0 stores it back into +0xb0 when the motion mode changes and FUN_0022d148 sums
/// it along a chain. Static modes -> zero. Kinds 1/4/5/9/10/0xb/0xc: FUN_0022cc00 of the
/// vector velocity (modes 2,3,8,9,0xb,0x11,0x12,0x14,0x15 = stored +0xb0; modes 5/6 = from
/// the segment: seg mode 1 Hermite velocity, 3/4 +0xd0, else 0). Kinds 6/7: skew(angular
/// velocity) * local (stored +0xb0, or euler + euler rate from the segment; seg mode 5 ->
/// zero). Kinds 8/0xe: the attached bone's velocity with the translation row negated.
/// Cases the original leaves uninitialised (motion 4, unknown kinds) return zero here.
pub fn ee_sub_22cdc0(bones: &[BoneState], i: usize) -> Mat4 {
    ee_sub_22cdc0_depth(bones, i, 0)
}

fn ee_sub_22cdc0_depth(bones: &[BoneState], i: usize, depth: usize) -> Mat4 {
    // 0x0022cdc0
    let b = &bones[i];
    if motion_is_static(b.motion) {
        return zero();
    }
    let m1 = b.motion.wrapping_sub(1);
    let stored = matches!(m1, 0 | 1 | 6 | 7 | 8 | 10 | 0x10 | 0x11 | 0x13 | 0x14);
    let from_seg = matches!(m1, 4 | 5);
    match b.kind {
        1 | 4 | 5 | 9 | 10 | 0xb | 0xc => {
            let v = if stored {
                b.vel
            } else if from_seg {
                match b.seg.mode {
                    1 => herm(b).1,
                    3 | 4 => b.seg.v0,
                    _ => [0.0; 3], // DAT_002a2dd0
                }
            } else {
                return zero();
            };
            ee_sub_22cc00(b.kind, v)
        }
        6 | 7 => {
            if stored {
                return ee_sub_22ccd0(b.vel, &b.local);
            }
            if !from_seg || b.seg.mode == 5 {
                return zero();
            }
            let e = match b.seg.mode {
                0 => ee_sub_22ca18(b),
                1 => herm(b).0,
                2..=4 => b.seg.p0,
                _ => [0.0; 3],
            };
            let r = match b.seg.mode {
                1 => herm(b).1,
                3 | 4 => b.seg.v0,
                _ => [0.0; 3],
            };
            ee_sub_22cb90(&b.local, e, r)
        }
        8 | 0xe => match b.attach {
            // recursion guard: attach cycles would recurse forever in the original
            Some(a) if a < bones.len() && depth < bones.len() => {
                let mut m = ee_sub_22cdc0_depth(bones, a, depth + 1);
                m[3][0] = -m[3][0];
                m[3][1] = -m[3][1];
                m[3][2] = -m[3][2];
                m
            }
            _ => zero(),
        },
        _ => zero(),
    }
}

/// FUN_0022d148(out, from, to): velocity of bone `from` relative to ancestor `to` (None =
/// world): sum over the chain of parent_world * V_local * inverse(world) (no parent: V *
/// inverse(world)), skipping motion modes 0/3/10/0xc/0xf/0x13, then inverse(world_to) * sum.
pub fn ee_sub_22d148(bones: &[BoneState], from: usize, to: Option<usize>) -> Mat4 {
    // 0x0022d148
    let mut acc = zero();
    let mut cur = Some(from);
    let mut guard = 0;
    while cur != to {
        let Some(i) = cur else { break };
        guard += 1;
        if guard > bones.len() {
            break;
        }
        let b = &bones[i];
        if !matches!(b.motion, 0 | 3 | 10 | 0xc | 0xf | 0x13) {
            let v = ee_sub_22cdc0(bones, i);
            let inv = ee_sub_11c318(&b.world);
            let d = match b.parent {
                None => ee_sub_11c168(&v, &inv),
                Some(p) => ee_sub_11c168(&ee_sub_11c168(&bones[p].world, &v), &inv),
            };
            for (ar, dr) in acc.iter_mut().zip(d.iter()) {
                for (a, x) in ar.iter_mut().zip(dr.iter()) {
                    *a += *x;
                }
            }
        }
        cur = b.parent;
    }
    if let Some(t) = to {
        acc = ee_sub_11c168(&ee_sub_11c318(&bones[t].world), &acc);
    }
    acc
}

/// Code-based replacement for the observed orthonormality floor: only FUN_0011bff0 (euler
/// extraction for kinds 6/7, via FUN_0022ca18) assumes an orthonormal basis. Kind 0xc drives
/// the diagonal (scale) and 0xb the off-diagonal (shear) through FUN_0022cc00, and no loader
/// (FUN_00117f18 / FUN_0022d7a0) normalises a stored matrix, so scale/shear is intended for
/// every other kind.
pub fn basis_must_be_orthonormal(kind: i32) -> bool {
    matches!(kind, 6 | 7)
}
