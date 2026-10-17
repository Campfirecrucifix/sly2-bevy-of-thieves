//! WP-J13: getter 0x0010f7b0 (class 59 field 1341, vector arg) -> FUN_00155fb0 and getter 0x00101368
//! (field 66, oid arg) -> FUN_001f8198.
//!
//! FUN_00155fb0(this unused, O): aim point from O at the player. P0 = FUN_001a62e8(FUN_001c4840(SW)),
//! d = |P0 - O|, t = FUN_00154c90(0x2de578), P = P0 + (0,0,1) * 50, V = player vt+0x10c (FUN_001f4440,
//! physics velocity, 0 without). |V| <= 60 (DAT_002bdc98): off = FUN_0011e1a0(atan2(m[0][1], m[0][0]), 60,
//! 0) (60 along the facing of the player matrix FUN_001a6318); else off = V * clamp(k, 0.1(1-t), 0.4(1-t))
//! with k = A.x + d (A.y + d A.z), A = FUN_00258480(1-t, (0,1e-4,0) 0x2bdc60, t, 0 0x2bdc80). FUN_0011efb0
//! (P + off - O) -> yaw / pitch / len; the 11 yaw offsets DAT_002bdca0 are cast O -> O + FUN_0011e208(yaw +
//! dy, pitch, len) (FUN_001b44d8 collision entries, FUN_0014cc58 nearest hit): the first free ray wins,
//! else the ray whose hit is farthest from O. Result O + FUN_0011e208(best, pitch, len), a type-0x1a
//! vector (FUN_00129718: 128-bit register -> Vector; `GetterRet::F32x2` is that wrapper).
//! FUN_001f8198(alo, oid): the first element of the ALO's +0xa4 / +0xa8 list whose +0x18 is `oid`. The
//! list is filled at load by FUN_00167320 (ALO load slot 13 FUN_001f31e0, `MeshSet::descs`): u8 count,
//! per element FUN_001ed7a8(1, stream) = u16 class (class table ctor) + 0x20-byte block given to vt+0x3c
//! FUN_001ed8e0 (+0x18 = block word 1). FUN_0016ea80 (clone slot 40) copies flagged elements (not needed:
//! the port has no element instances).
//!
//! 1:1: 0x155fb0 (`ee_sub_155fb0`, inputs passed in), 0x11efb0, 0x11e208, 0x11e1a0, 0x258480, 0x1f8198.
//! REPLACED: FUN_001b44d8 / FUN_0014cc58 -> SO colliders `CollisionWorld::clip_segment` nearest entry
//! (as 0x1249a8); the element objects -> `RtRecord` {class = element class, owner = alo, alo = index,
//! arg = oid}. In the getter: player matrix facing and vt+0x10c velocity are not lent to the script host
//! (facing +x, velocity 0 -> the |V| <= 60 branch); t = FUN_00154c90 with DAT_003e0f9c none and the
//! 0x2de578 save-variable handles unbound = 0.

#![allow(non_snake_case)]

use crate::engine::camera::math::FUN_0011cf60;

pub type V3 = [f32; 3];

/// DAT_002bdca0: yaw offsets of the 11 candidate rays.
pub const DY: [f32; 11] = [0.0, -0.025, 0.025, -0.05, 0.05, -0.1, 0.1, -0.2, 0.2, -0.4, 0.4];

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn len(a: V3) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

/// 0x0011e208: spherical -> cartesian.
pub fn FUN_0011e208(yaw: f32, pitch: f32, r: f32) -> V3 {
    [r * pitch.cos() * yaw.cos(), r * pitch.cos() * yaw.sin(), r * pitch.sin()]
}

/// 0x0011efb0: cartesian -> (yaw, pitch, len).
pub fn FUN_0011efb0(v: V3) -> (f32, f32, f32) {
    crate::engine::camera::math::FUN_0011efb0(v)
}

/// 0x00155fb0 with its world inputs: `p0` player position, `fwd` the player matrix row 0 (x, y), `vel`
/// vt+0x10c, `t` FUN_00154c90, `cast(O, E)` = Some(nearest hit point) / None (free).
pub fn ee_sub_155fb0(o: V3, p0: V3, fwd: [f32; 2], vel: V3, t: f32, cast: &mut dyn FnMut(V3, V3) -> Option<V3>) -> V3 {
    // 0x00155fb0
    let d = len(sub(p0, o));
    let p = add(p0, [0.0, 0.0, 50.0]);
    let off = if len(vel) <= 60.0 {
        // FUN_0011e1a0(FUN_002458f0(m[1], m[0]), 60, 0)
        let fy = fwd[1].atan2(fwd[0]);
        [60.0 * fy.cos(), 60.0 * fy.sin(), 0.0]
    } else {
        let u = 1.0 - t;
        // FUN_00258480(u, (0, 1e-4, 0), t, (0, 0, 0))
        let a = [0.0 * u + 0.0 * t, 1e-4 * u + 0.0 * t, 0.0 * u + 0.0 * t];
        let k = a[0] + d * (a[1] + d * a[2]);
        let (lo, hi) = (0.1 * u + 0.0 * t, 0.4 * u + 0.0 * t);
        let k = if k < lo { lo } else if hi < k { hi } else { k };
        [vel[0] * k, vel[1] * k, vel[2] * k]
    };
    let (yaw, pitch, r) = FUN_0011efb0(sub(add(p, off), o));
    let mut best = f32::MIN; // DAT_002a513c -FLT_MAX
    let mut best_yaw = yaw;
    for dy in DY {
        let a = FUN_0011cf60(yaw + dy);
        let e = add(o, FUN_0011e208(a, pitch, r));
        match cast(o, e) {
            None => {
                best_yaw = a;
                break;
            }
            Some(h) => {
                let dd = len(sub(o, h));
                if best < dd {
                    best = dd;
                    best_yaw = a;
                }
            }
        }
    }
    add(o, FUN_0011e208(best_yaw, pitch, r))
}

/// FUN_001f8198 over `MeshSet::descs`: index of the first element whose +0x18 (block word 1) is `oid`.
pub fn ee_sub_1f8198(descs: &[Option<(u16, [u8; 0x20])>], oid: i32) -> Option<(usize, u16)> {
    // 0x001f8198
    descs.iter().enumerate().find_map(|(i, d)| d.as_ref().filter(|(_, b)| i32::from_le_bytes([b[4], b[5], b[6], b[7]]) == oid).map(|(c, _)| (i, *c)))
}
