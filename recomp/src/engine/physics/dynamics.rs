//! WP-E4: SO dynamics chain (per-frame driver, gravity/velocity/impulse, contact response, ground, platforms).
//! 1:1: FUN_0011d508 (quadratic roots), FUN_0011d018 (bounded-accel approach), FUN_0014f040 (camera sweep callback).
//! WP-E5: `update_body` delegates to the real chain in driver.rs; still replaced: platform carry, ground flag. See so_vtable_map.md "E4".
#![allow(non_snake_case)]
use super::geom::{dot, V3};
use super::driver::{AloPhys, FUN_001f8dd8};
use super::sphere::Sbi;
use super::CollisionWorld;

/// 0x0011d508: roots of a x^2 + b x + c, ascending; returns count (0, 1 when |disc| tiny, 2).
pub fn FUN_0011d508(a: f32, b: f32, c: f32, out: &mut [f32; 2]) -> usize {
    let d = b * b - a * c * 4.0;
    if d < 0.0 {
        return 0;
    }
    let h = d.sqrt() / (a + a);
    let m = -(b / (a + a));
    if h.abs() < 0.0001 {
        out[0] = m;
        return 1;
    }
    let (lo, hi) = (m - h, m + h);
    *out = if lo < hi { [lo, hi] } else { [hi, lo] };
    2
}

/// 0x0011d018(x, target, dt, {vmax, v0, accel}, &vel_out): moves x towards target within dt under the
/// trapezoidal speed profile; writes the signed speed when `vel` is Some.
pub fn FUN_0011d018(x: f32, target: f32, dt: f32, p: [f32; 3], vel: Option<&mut f32>) -> f32 {
    let mut dt = dt;
    let vmax = p[0];
    if vmax == 0.0 {
        if let Some(v) = vel {
            *v = 0.0;
        }
        return x;
    }
    let mut d = x - target;
    let neg = d < 0.0;
    if neg {
        d = -d;
    }
    let acc = p[2];
    let ramp = (p[1] + vmax) * acc * 0.5;
    let (step, sp);
    'blk: {
        let (v0, dist);
        if ramp <= d {
            let t = (d - ramp) / vmax;
            if dt <= t {
                sp = -vmax;
                step = d - vmax * dt;
                break 'blk;
            }
            dt -= t;
            v0 = p[1];
            dist = ramp;
        } else {
            v0 = p[1];
            dist = d;
        }
        let mut r = [0.0f32; 2];
        let n = FUN_0011d508((vmax - v0) / (acc + acc), v0, -dist, &mut r);
        let rem = if n > 0 { r[n - 1] } else { 0.0 } - dt;
        if 0.0 < rem {
            let k = (vmax - p[1]) * rem;
            sp = -(p[1] + k / acc);
            step = (k * rem) / (acc + acc) + p[1] * rem;
        } else {
            sp = -p[1];
            step = 0.0;
        }
    }
    if neg {
        if let Some(v) = vel {
            *v = -sp;
        }
        target - step
    } else {
        if let Some(v) = vel {
            *v = sp;
        }
        step + target
    }
}

/// DAT_002bd8d8 (125.0) and the profile at 0x002bd8e0 {2500, 0, 0.25} used by the camera callback.
pub const CAM_GAP_BIAS: f32 = 125.0;
pub const CAM_PROFILE: [f32; 3] = [2500.0, 0.0, 0.25];

/// 0x0014f040(cam, centre, vel, sbi): b = n.vel - approach(n.pos (+d), radius(+0x48) - 125, dt DAT_002ddee0).
/// conf 0.5 on the gap term (Ghidra drops the plane w; we use the signed plane distance of the contact point).
/// Wiring: camera/ should pass `|c, v, s| FUN_0014f040(radius, dt, c, v, s)` to FUN_0014eda0 (not done here).
pub fn FUN_0014f040(radius: f32, dt: f32, _c: V3, vel: V3, s: &Sbi) -> f32 {
    let n = [s.plane[0], s.plane[1], s.plane[2]];
    let gap = dot(n, s.pos) + s.plane[3];
    dot(vel, n) - FUN_0011d018(gap, radius - CAM_GAP_BIAS, dt, CAM_PROFILE, None)
}

/// Gravity magnitude: ALO slot63 0x1f3590 default e8+0x80 = (0,0,-1750) (WP-E5; replaces the E4 2000 guess).
pub const GRAVITY: f32 = -super::driver::GRAVITY_Z;
/// Replaced: ground when contact normal z >= cos(slope limit) (Sly 1 SO uses ~45 deg).
pub const GROUND_COS: f32 = 0.7071;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Body {
    pub pos: V3,
    pub vel: V3,
    pub radius: f32,
    pub mass: f32,
    /// ALO +0xf0 pending linear impulse (velocity delta, consumed by 0x1f9e70)
    pub impulse: V3,
    pub grounded: bool,
    pub ground_normal: V3,
    /// moving platform carrying this body (index into `platforms`)
    pub platform: Option<usize>,
    /// WP-E5: full ALO physics state (rotation, angular velocity, sleep, ...) driven by driver::FUN_001f8dd8.
    pub alo: AloPhys,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Platform {
    /// top plane height and xy rect
    pub z: f32,
    pub min: [f32; 2],
    pub max: [f32; 2],
    pub vel: V3,
}

/// 0x1f9e70 linear half (WP-E5: the impulse is a velocity delta; FUN_0022fcc8 only changes frame, no mass).
pub fn apply_impulse(b: &mut Body) {
    if b.impulse != [0.0; 3] {
        for k in 0..3 {
            b.vel[k] += b.impulse[k];
        }
        b.impulse = [0.0; 3];
    }
}

/// Per-frame update. WP-E5: delegates to the real driver chain driver::FUN_001f8dd8 (slot16 integrate, contacts,
/// slot64 impulse, sleep). Still replaced: platform attach/carry and the grounded flag (game-side ALO state).
pub fn update_body(cw: &CollisionWorld, plats: &mut [Platform], b: &mut Body, dt: f32) {
    b.platform = plats.iter().position(|p| {
        b.pos[0] >= p.min[0] && b.pos[0] <= p.max[0] && b.pos[1] >= p.min[1] && b.pos[1] <= p.max[1] && (b.pos[2] - b.radius - p.z).abs() <= 1.0
    });
    let carry = b.platform.map(|i| plats[i].vel).unwrap_or([0.0; 3]);
    let a = &mut b.alo;
    a.pos = b.pos;
    a.v = b.vel;
    a.imp_lin = [a.imp_lin[0] + b.impulse[0], a.imp_lin[1] + b.impulse[1], a.imp_lin[2] + b.impulse[2]];
    b.impulse = [0.0; 3];
    if b.platform.is_some() {
        a.ckind = 3; // locked against the platform top (e8+0xc0 = 3: no gravity)
        a.v[2] = 0.0;
    } else if a.ckind == 3 {
        a.ckind = 0;
        a.kind = 1;
    }
    let normals = FUN_001f8dd8(cw, a, b.radius, dt);
    b.pos = a.pos;
    b.vel = a.v;
    let ground = normals.iter().copied().find(|n| n[2] >= GROUND_COS);
    // asleep (kind 3): no contacts are gathered, keep the previous ground state
    b.grounded = ground.is_some() || b.platform.is_some() || (a.kind == 3 && b.grounded);
    if ground.is_some() || b.platform.is_some() {
        b.ground_normal = ground.unwrap_or([0.0, 0.0, 1.0]);
    }
    for k in 0..3 {
        b.pos[k] += carry[k] * dt;
    }
    for p in plats.iter_mut() {
        for k in 0..2 {
            p.min[k] += p.vel[k] * dt;
            p.max[k] += p.vel[k] * dt;
        }
        p.z += p.vel[2] * dt;
    }
}
