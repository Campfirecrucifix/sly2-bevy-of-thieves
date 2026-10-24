//! WP-K14: the class-81 launcher. FUN_001d6b00(launcher, projectile) is called by the Splice method 468
//! (thunk FUN_001058f0, kind 0x2030 argument) on a class-112 launcher record, by class 69 slot 15
//! (FUN_00225b60 -> FUN_00226250: projectile pulled from spawner +0x1c0 with FUN_001cb250, launcher +0x1c4) and
//! by class 55 (FUN_001357c0). It aims the projectile at the launcher's target (+0x50 in the frame of +0x60),
//! sets its velocity / orientation / spin and, for class-80 projectiles (79 <= 80 < 83 range: 80, 81, 82),
//! calls slot 77 (class 81: FUN_001b1e88, classes/k13.rs).
//! Class-112 launcher (K7 `C2d7f40`) rows 460-467: 460 +0x60 frame object (kind 0x2002), 461 +0x50 target
//! (vector), 462 +0x70 flight-time poly, 463 +0x80 its clamp, 464 +0x90 speed poly, 465 +0xa0 its clamp, 466
//! +0xa8 spin range, 467 +0xb0 ballistic blend.
//! HEIST r17: the frame's world matrix (slot 31 + FUN_0011c168) = vtable +0xfc (rotation included). REPLACED:
//! the moving-frame prediction FUN_002309e8(frame, 0, m, 0) over the flight time = the static frame; the
//! world bookkeeping FUN_0013bc60 (+0x200 owner + collision-ignore refresh FUN_00134320), slot 7 (add to
//! world), FUN_001f5720 / FUN_001f57c0 (xfLin / xfAng act reset), FUN_001a58d0 = not modelled (the projectile
//! is always in the class set) except +0x200 = launcher.
//! K15: the class-82 branch (FUN_001f4898(proj, {+0x260, 0, ?}): the parent-frame velocity; the decompile
//! loses the third word -> 0) and the ballistic branch of classes 79 / 80 (FUN_001d6848 over the projectile's
//! gravity FUN_001f5860 = +0xe8 +0x80) 1:1; a ballistic shot with no solution leaves the velocity at 0 (the
//! original passes an unset register to slot 35).
use super::super::ClassHost;
use super::{len3, scale3};
use crate::engine::camera::looks::FUN_0011b7e0;
use crate::engine::physics::driven::GameRng;
use crate::engine::player::p4::FUN_00132b10;
use super::super::k7::C2d7f40;

/// FUN_001d6b00 on a class-112 launcher record (`k7::C2d7f40`, K7: slot 6 defaults + rows 460-467); +0x60
/// = the resolved frame handle (0 = none)
pub struct Launcher<'a> {
    pub l: &'a C2d7f40,
    pub frame: u32,
}

impl Launcher<'_> {
    /// FUN_001d6b00 aim part: target T = frame * +0x50; d = |T - p0|; flight time t = clamp(poly +0x70 (d),
    /// +0x80) (prediction REPLACED); speed s = clamp(poly +0x90 (d), +0xa0) (lower bound first, then the upper,
    /// as the original: min if below, max if above); aim = p0 + unit(T - p0) * s
    pub fn aim(&self, p0: [f32; 3], h: &mut dyn ClassHost) -> [f32; 3] {
        let l = self.l;
        // HEIST r17: slot 31 world matrix of the frame (FUN_0011c168: T = +0x50 x M), the rotation included (the
        // EP2 turret barrels' 2c4a frames follow the class-69 yaw / pitch pin, FUN_0012d108)
        let t = if self.frame == 0 {
            [l.f_50[0], l.f_50[1], l.f_50[2]]
        } else {
            let m = h.vcall_fc(self.frame);
            std::array::from_fn(|k| l.f_50[0] * m[0][k] + l.f_50[1] * m[1][k] + l.f_50[2] * m[2][k] + m[3][k])
        };
        let d = [t[0] - p0[0], t[1] - p0[1], t[2] - p0[2]];
        let n = len3(d);
        let s = l.f_90[0] + n * (l.f_90[1] + n * l.f_90[2]);
        let mut sp = l.f_a0[0];
        if l.f_a0[0] <= s {
            sp = l.f_a0[1];
            if s <= l.f_a0[1] {
                sp = s;
            }
        }
        let u = if n < 0.0001 { [1.0, 0.0, 0.0] } else { scale3(d, 1.0 / n) };
        [p0[0] + u[0] * sp, p0[1] + u[1] * sp, p0[2] + u[2] * sp]
    }
}

/// what FUN_001d6b00 did to the projectile
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Launch {
    pub aim: [f32; 3],
    pub v: [f32; 3],
    pub rot: Option<[[f32; 3]; 3]>,
    pub w: Option<[f32; 3]>,
    /// slot 77 (launcher frame +0x60, target +0x50) for class-80 projectiles
    pub slot77: Option<(u32, [f32; 4])>,
}

/// FUN_001d6848(blend, p0, aim, g): ballistic launch velocity. d = aim - p0, horizontal length dh; elevation
/// a = max(atan2(d.z, dh), 0) lifted toward 90 degrees by sin(blend); k = g.z / (2 (d.z - dh tan a)); k > 0 ->
/// v = unit({d.x, d.y, dh tan a}) * dh sqrt(k) / cos a, else None (no solution)
#[allow(non_snake_case)]
pub fn FUN_001d6848(blend: f32, p0: [f32; 3], aim: [f32; 3], g: [f32; 3]) -> Option<[f32; 3]> {
    let s = blend.sin();
    let d = [aim[0] - p0[0], aim[1] - p0[1], aim[2] - p0[2]];
    let dh = len3([d[0], d[1], 0.0]);
    let mut a = d[2].atan2(dh);
    if a < 0.0 {
        a = 0.0;
    }
    let gz = g[2];
    let a = a + (std::f32::consts::FRAC_PI_2 - a) * s;
    let t = a.tan();
    let e = d[2] - dh * t;
    let k = gz / (e + e);
    if 0.0 < k {
        let c = a.cos();
        let u = [d[0], d[1], dh * t];
        let n = len3(u);
        let sp = dh * k.sqrt() / c;
        Some(scale3(u, sp / n))
    } else {
        None
    }
}

/// FUN_001d6b00 for a projectile of world class `cls` at p0 with launch speed +0x260 and gravity `g`: class 82
/// -> v = {+0x260, 0, 0} (FUN_001f4898, top-level = world frame); class 81 -> v = unit(aim - p0) * +0x260
/// (slot 35), orientation FUN_0011b7e0(v) (slot 30); 79 / 80 -> ballistic FUN_001d6848(+0xb0, p0, aim, g)
/// (slot 35); spin rand(+0xa8, +0xac) * unit(rand(-1, 1) x3) (slot 36) when +0xac > 0; class 80-82 -> slot
/// 77(+0x60, +0x50).
pub fn ee_sub_1d6b00(l: &Launcher, cls: i32, p0: [f32; 3], f_260: f32, g: [f32; 3], rng: &mut GameRng, h: &mut dyn ClassHost) -> Option<Launch> {
    if !FUN_00132b10(cls, 79) {
        return None;
    }
    let aim = l.aim(p0, h);
    let mut out = Launch { aim, ..Default::default() };
    if FUN_00132b10(cls, 82) {
        out.v = [f_260, 0.0, 0.0];
    } else if FUN_00132b10(cls, 81) {
        let d = [aim[0] - p0[0], aim[1] - p0[1], aim[2] - p0[2]];
        let n = len3(d);
        let u = if n < 0.0001 { [1.0, 0.0, 0.0] } else { scale3(d, 1.0 / n) };
        out.v = scale3(u, f_260);
        out.rot = Some(FUN_0011b7e0(out.v));
    } else {
        out.v = FUN_001d6848(l.l.f_b0, p0, aim, g).unwrap_or([0.0; 3]);
    }
    if 0.0 < l.l.f_a8[1] {
        let m = rng.FUN_0011d448(l.l.f_a8[0], l.l.f_a8[1]);
        let x = rng.FUN_0011d448(-1.0, 1.0);
        let y = rng.FUN_0011d448(-1.0, 1.0);
        let z = rng.FUN_0011d448(-1.0, 1.0);
        let r = [x, y, z];
        let k = len3(r);
        let a = if 0.0001 <= k { scale3(r, 1.0 / k) } else { [1.0, 0.0, 0.0] };
        out.w = Some(scale3(a, m));
    }
    if FUN_00132b10(cls, 80) {
        out.slot77 = Some((l.frame, l.l.f_50));
    }
    Some(out)
}
