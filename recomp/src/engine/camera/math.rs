//! Vector/matrix helpers used by the camera, ported from the game's math module (0x0011bxxx-0x0011exxx).
//! Matrices follow the PS2/VU layout of the original: 4 column vectors (`m[i]` = column i, as stored in
//! memory at +0x10*i), points transform as `M * v = sum m[j] * v[j]`.
//! libm (tanf 0x002450f0, sinf 0x00245000, atan2f 0x002458f0, acosf 0x002456f0) is replaced by Rust `f32`.
#![allow(non_snake_case)]

pub type V3 = [f32; 3];
pub type V4 = [f32; 4];
pub type M4 = [V4; 4];

pub const PI: f32 = 3.1415927;

pub const IDENTITY: M4 = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];

pub fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
pub fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
pub fn scale(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
pub fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub fn len(a: V3) -> f32 {
    dot(a, a).sqrt()
}
/// VU `OPMULA a,b ; OPMSUB b,a` idiom = a x b.
pub fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
pub fn v3(v: V4) -> V3 {
    [v[0], v[1], v[2]]
}
pub fn v4(v: V3, w: f32) -> V4 {
    [v[0], v[1], v[2], w]
}

/// M * v (4-component).
pub fn mul_v4(m: &M4, v: V4) -> V4 {
    let mut r = [0.0; 4];
    for j in 0..4 {
        for i in 0..4 {
            r[i] += m[j][i] * v[j];
        }
    }
    r
}

/// Affine point transform: m[0..3] * p + m[3].
pub fn xform_point(m: &M4, p: V3) -> V3 {
    v3(mul_v4(m, v4(p, 1.0)))
}

/// 0x0011c168: out = a * b (VU micro 0x60 4x4 multiply; b is applied first).
pub fn FUN_0011c168(a: &M4, b: &M4) -> M4 {
    [mul_v4(a, b[0]), mul_v4(a, b[1]), mul_v4(a, b[2]), mul_v4(a, b[3])]
}

/// 0x0011c318: inverse of an affine matrix (3x3 via cross-product cofactors / determinant, then -R^-1 t).
pub fn FUN_0011c318(m: &M4) -> M4 {
    let (a, b, c) = (v3(m[0]), v3(m[1]), v3(m[2]));
    let (bc, ca, ab) = (cross(b, c), cross(c, a), cross(a, b));
    let inv_det = 1.0 / dot(a, bc);
    // rows of the inverse are bc, ca, ab scaled; stored transposed into columns.
    let r = [scale(bc, inv_det), scale(ca, inv_det), scale(ab, inv_det)];
    let t = v3(m[3]);
    let mut out = [[0.0; 4]; 4];
    for i in 0..3 {
        for k in 0..3 {
            out[k][i] = r[i][k];
        }
    }
    let tt = [-dot(r[0], t), -dot(r[1], t), -dot(r[2], t)];
    out[3] = v4(tt, 1.0);
    out
}

/// 0x0011c2f0: general 4x4 inverse (replaced: same result via cofactor expansion).
pub fn FUN_0011c2f0(m: &M4) -> M4 {
    let a: [f32; 16] = std::array::from_fn(|k| m[k / 4][k % 4]);
    let mut inv = [0.0f32; 16];
    inv[0] = a[5] * a[10] * a[15] - a[5] * a[11] * a[14] - a[9] * a[6] * a[15] + a[9] * a[7] * a[14] + a[13] * a[6] * a[11] - a[13] * a[7] * a[10];
    inv[4] = -a[4] * a[10] * a[15] + a[4] * a[11] * a[14] + a[8] * a[6] * a[15] - a[8] * a[7] * a[14] - a[12] * a[6] * a[11] + a[12] * a[7] * a[10];
    inv[8] = a[4] * a[9] * a[15] - a[4] * a[11] * a[13] - a[8] * a[5] * a[15] + a[8] * a[7] * a[13] + a[12] * a[5] * a[11] - a[12] * a[7] * a[9];
    inv[12] = -a[4] * a[9] * a[14] + a[4] * a[10] * a[13] + a[8] * a[5] * a[14] - a[8] * a[6] * a[13] - a[12] * a[5] * a[10] + a[12] * a[6] * a[9];
    inv[1] = -a[1] * a[10] * a[15] + a[1] * a[11] * a[14] + a[9] * a[2] * a[15] - a[9] * a[3] * a[14] - a[13] * a[2] * a[11] + a[13] * a[3] * a[10];
    inv[5] = a[0] * a[10] * a[15] - a[0] * a[11] * a[14] - a[8] * a[2] * a[15] + a[8] * a[3] * a[14] + a[12] * a[2] * a[11] - a[12] * a[3] * a[10];
    inv[9] = -a[0] * a[9] * a[15] + a[0] * a[11] * a[13] + a[8] * a[1] * a[15] - a[8] * a[3] * a[13] - a[12] * a[1] * a[11] + a[12] * a[3] * a[9];
    inv[13] = a[0] * a[9] * a[14] - a[0] * a[10] * a[13] - a[8] * a[1] * a[14] + a[8] * a[2] * a[13] + a[12] * a[1] * a[10] - a[12] * a[2] * a[9];
    inv[2] = a[1] * a[6] * a[15] - a[1] * a[7] * a[14] - a[5] * a[2] * a[15] + a[5] * a[3] * a[14] + a[13] * a[2] * a[7] - a[13] * a[3] * a[6];
    inv[6] = -a[0] * a[6] * a[15] + a[0] * a[7] * a[14] + a[4] * a[2] * a[15] - a[4] * a[3] * a[14] - a[12] * a[2] * a[7] + a[12] * a[3] * a[6];
    inv[10] = a[0] * a[5] * a[15] - a[0] * a[7] * a[13] - a[4] * a[1] * a[15] + a[4] * a[3] * a[13] + a[12] * a[1] * a[7] - a[12] * a[3] * a[5];
    inv[14] = -a[0] * a[5] * a[14] + a[0] * a[6] * a[13] + a[4] * a[1] * a[14] - a[4] * a[2] * a[13] - a[12] * a[1] * a[6] + a[12] * a[2] * a[5];
    inv[3] = -a[1] * a[6] * a[11] + a[1] * a[7] * a[10] + a[5] * a[2] * a[11] - a[5] * a[3] * a[10] - a[9] * a[2] * a[7] + a[9] * a[3] * a[6];
    inv[7] = a[0] * a[6] * a[11] - a[0] * a[7] * a[10] - a[4] * a[2] * a[11] + a[4] * a[3] * a[10] + a[8] * a[2] * a[7] - a[8] * a[3] * a[6];
    inv[11] = -a[0] * a[5] * a[11] + a[0] * a[7] * a[9] + a[4] * a[1] * a[11] - a[4] * a[3] * a[9] - a[8] * a[1] * a[7] + a[8] * a[3] * a[5];
    inv[15] = a[0] * a[5] * a[10] - a[0] * a[6] * a[9] - a[4] * a[1] * a[10] + a[4] * a[2] * a[9] + a[8] * a[1] * a[6] - a[8] * a[2] * a[5];
    let det = a[0] * inv[0] + a[1] * inv[4] + a[2] * inv[8] + a[3] * inv[12];
    let d = if det != 0.0 { 1.0 / det } else { 0.0 };
    std::array::from_fn(|c| std::array::from_fn(|r| inv[c * 4 + r] * d))
}

/// 0x0011c448: projection matrix (column layout): m[0]=sx, m[5]=sy, m[8]=cx, m[9]=cy,
/// m[10]=(n+f)/(n-f), m[11]=1, m[14]=n*(1-(n+f)/(n-f)).
pub fn FUN_0011c448(sx: f32, sy: f32, cx: f32, cy: f32, near: f32, far: f32) -> M4 {
    let a = (near + far) / (near - far);
    [[sx, 0.0, 0.0, 0.0], [0.0, sy, 0.0, 0.0], [cx, cy, a, 1.0], [0.0, 0.0, near * (1.0 - a), 0.0]]
}

/// 0x0011d608: game sin/cos (polynomial, quadrant fold by pi). Returns (sin, cos).
pub fn FUN_0011d608(x: f32) -> (f32, f32) {
    let f = (x + 1.5707964) * 0.31830987;
    let mut n = f as i32 as u32; // (uint)f truncation of the float
    if f < 0.0 {
        n = n.wrapping_sub(1);
    }
    let mut x = x - (n as i32) as f32 * PI;
    if n & 1 != 0 {
        x = -x;
    }
    let x2 = x * x;
    let x5 = x * x2 * x2;
    let x7 = x5 * x2;
    let mut s = x + x * x2 * -0.16666657 + x5 * 0.0083330255 + x7 * -0.00019807414 + x7 * x2 * 2.601887e-06;
    s = s.clamp(-1.0, 1.0);
    let mut c = (1.0 - s * s).sqrt();
    if n & 1 != 0 {
        c = -c;
    }
    (s, c)
}

/// 0x0011cf60: wrap an angle into [-pi, pi] (fmodf 0x0011d898 replaced by rem_euclid).
pub fn FUN_0011cf60(a: f32) -> f32 {
    if a < -PI || PI < a { (a + PI).rem_euclid(2.0 * PI) - PI } else { a }
}

/// 0x0011cfe0: clamp to [-lim, lim].
pub fn FUN_0011cfe0(x: f32, lim: f32) -> f32 {
    if x <= lim { if x < -lim { -lim } else { x } } else { lim }
}

/// 0x0011b480: rotation matrix from (sin, cos, unit axis) (Rodrigues; 3 columns, w = 0).
pub fn FUN_0011b480(s: f32, c: f32, a: V3) -> M4 {
    let t = 1.0 - c;
    let col = |i: usize| -> V4 {
        let e = [(i == 0) as u8 as f32, (i == 1) as u8 as f32, (i == 2) as u8 as f32];
        // R e = c e + s (a x e) + t a (a.e)
        v4(add(add(scale(e, c), scale(cross(a, e), s)), scale(a, t * a[i])), 0.0)
    };
    [col(0), col(1), col(2), [0.0, 0.0, 0.0, 1.0]]
}

/// 0x0011b500: rotation of `angle` around unit `axis` (game sincos).
pub fn FUN_0011b500(angle: f32, axis: V3) -> M4 {
    let (s, c) = FUN_0011d608(angle);
    FUN_0011b480(s, c, axis)
}

/// 0x0011b570: rotation from a rotation vector (axis * angle); identity (0x2a2d00) when |v| < 1e-4.
pub fn FUN_0011b570(v: V3) -> M4 {
    let l = len(v);
    if l < 0.0001 { IDENTITY } else { FUN_0011b500(l, scale(v, 1.0 / l)) }
}

/// 0x0011b720: orientation from yaw (about +Z, 0x2a2dc0) and pitch (about +Y by -pitch, 0x2a2db0):
/// M = Rz(yaw) * Ry(-pitch). Column 0 = forward = (cp*cy, cp*sy, sp).
pub fn FUN_0011b720(yaw: f32, pitch: f32) -> M4 {
    let rz = FUN_0011b500(yaw, [0.0, 0.0, 1.0]);
    let ry = FUN_0011b500(-pitch, [0.0, 1.0, 0.0]);
    let mut m = FUN_0011c168(&rz, &ry);
    m[3] = [0.0, 0.0, 0.0, 1.0];
    m
}

/// 0x0011b628: orientation from Euler angles (x, y, z) (game sincos), columns with w left 0.
pub fn FUN_0011b628(e: V3) -> M4 {
    let (s0, c0) = FUN_0011d608(e[0]);
    let (s1, c1) = FUN_0011d608(e[1]);
    let (s2, c2) = FUN_0011d608(e[2]);
    [
        [c1 * c2, c1 * s2, -s1, 0.0],
        [s0 * s1 * c2 - c0 * s2, s0 * s1 * s2 + c0 * c2, s0 * c1, 0.0],
        [c0 * s1 * c2 + s0 * s2, c0 * s1 * s2 - s0 * c2, c0 * c1, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// 0x0011bcf8: rotation matrix -> (angle, unit axis). Same algorithm (acos of (trace-1)/2, axis magnitudes
/// from the diagonal, signs from the off-diagonals). `m` columns.
pub fn FUN_0011bcf8(m: &M4) -> (f32, V3) {
    let p = |k: usize| m[k / 4][k % 4]; // flat index into the 4x4 as in the original float* walk
    let c = ((p(0) + p(5) + p(10)) - 1.0) * 0.5;
    if 0.999999 < c {
        return (0.0, [0.0, 0.0, 1.0]); // 0x2a2da0
    }
    let mut ax = [0.0f32; 3];
    let mut rem = 1.0f32;
    for i in 0..2 {
        let v = ((p(i * 5) - c) / (1.0 - c)).abs();
        let mut f = rem;
        if v <= rem {
            f = if v < 0.0 { 0.0 } else { v };
        }
        ax[i] = f.sqrt();
        rem -= f;
    }
    ax[2] = rem.sqrt();
    if -0.9999 <= c {
        let ang = c.acos();
        if p(1) < p(4) {
            ax[2] = -ax[2];
        }
        if p(8) < p(2) {
            ax[1] = -ax[1];
        }
        if p(6) > p(9) {
            // original: `if (p[9] <= p[6]) return;` then negate x
        } else {
            ax[0] = -ax[0];
        }
        return (ang, ax);
    }
    let (x, y) = (ax[0], ax[1]);
    if y < x {
        if ax[2] < x {
            if p(1) < 0.0 {
                ax[1] = -y;
            }
            if p(2) < 0.0 {
                ax[2] = -ax[2];
            }
            return (PI, ax);
        }
    } else if ax[2] < y {
        if p(4) < 0.0 {
            ax[0] = -x;
        }
        if p(6) < 0.0 {
            ax[2] = -ax[2];
        }
        return (PI, ax);
    }
    if p(8) < 0.0 {
        ax[0] = -x;
    }
    if p(9) < 0.0 {
        ax[1] = -ax[1];
    }
    (PI, ax)
}

/// 0x0011efb0: decompose a direction into (yaw = atan2(y,x) wrapped, pitch = atan2(z, |xy|) wrapped, length).
pub fn FUN_0011efb0(v: V3) -> (f32, f32, f32) {
    let yaw = FUN_0011cf60(v[1].atan2(v[0]));
    let pitch = FUN_0011cf60(v[2].atan2((v[0] * v[0] + v[1] * v[1]).sqrt()));
    (yaw, pitch, len(v))
}

/// 0x0011e1a0: SetVectorCylind: (cos(a)*r, sin(a)*r, z).
pub fn FUN_0011e1a0(rad: f32, r: f32, z: f32) -> V3 {
    let (s, c) = FUN_0011d608(rad);
    [c * r, s * r, z]
}

/// 0x0011e208: SetVectorSphere: (cos(el)*cos(az)*r, cos(el)*sin(az)*r, sin(el)*r).
pub fn FUN_0011e208(az: f32, el: f32, r: f32) -> V3 {
    let (s0, c0) = FUN_0011d608(az);
    let (s1, c1) = FUN_0011d608(el);
    [c1 * c0 * r, c1 * s0 * r, s1 * r]
}

/// 0x0011d508: real roots of a x^2 + b x + c (ascending). Returns count and roots.
pub fn FUN_0011d508(a: f32, b: f32, c: f32) -> (usize, [f32; 2]) {
    let d = b * b - a * c * 4.0;
    if d < 0.0 {
        return (0, [0.0; 2]);
    }
    let h = d.sqrt() / (a + a);
    let m = -(b / (a + a));
    if h.abs() < 0.0001 {
        return (1, [m, 0.0]);
    }
    let (r1, r0) = (m - h, m + h);
    if r1 < r0 { (2, [r1, r0]) } else { (2, [r0, r1]) }
}

/// Approach parameters (4 floats as laid out in the game's tables, e.g. 0x2bd810, cm+0x2c).
/// p[0] = max speed, p[1] = end speed, p[2] = deceleration distance factor, p[3] = max acceleration
/// (only used by FUN_0011d1c0).
pub type Smp = [f32; 4];

/// 0x0011d018: move `x` towards `target` within `dt` along a speed profile `smp`. Returns (new x, velocity).
pub fn FUN_0011d018(x: f32, target: f32, dt: f32, smp: &Smp) -> (f32, f32) {
    let mut dt = dt;
    let p0 = smp[0];
    if p0 == 0.0 {
        return (x, 0.0);
    }
    let mut d = x - target;
    let neg = d < 0.0;
    if neg {
        d = -d;
    }
    let p2 = smp[2];
    let mut dist = (smp[1] + p0) * p2 * 0.5;
    let moved;
    let vel;
    'done: {
        let p1;
        if dist <= d {
            let tcruise = (d - dist) / p0;
            if dt <= tcruise {
                vel = -p0;
                moved = d - p0 * dt;
                break 'done;
            }
            dt -= tcruise;
            p1 = smp[1];
        } else {
            p1 = smp[1];
            dist = d;
        }
        let (n, roots) = FUN_0011d508((smp[0] - p1) / (p2 + p2), p1, -dist);
        let left = roots[n.max(1) - 1] - dt;
        if 0.0 < left {
            let a1 = smp[1];
            let a2 = smp[2];
            let k = (smp[0] - a1) * left;
            vel = -(a1 + k / a2);
            moved = (k * left) / (a2 + a2) + a1 * left;
        } else {
            vel = -smp[1];
            moved = 0.0;
        }
    }
    if neg { (target - moved, -vel) } else { (moved + target, vel) }
}

/// 0x0011d1c0: FUN_0011d018 with the velocity change limited by smp[3] (max acceleration).
/// Returns (new x, new velocity).
pub fn FUN_0011d1c0(x: f32, v: f32, target: f32, dt: f32, smp: &Smp) -> (f32, f32) {
    let (mut r, mut nv) = FUN_0011d018(x, target, dt, smp);
    let acc = smp[3];
    if 0.0 < acc {
        let hi = v + acc * dt;
        let lo = v - acc * dt;
        if hi < nv {
            r = x + v * dt + acc * dt * 0.5 * dt;
            nv = hi;
        } else if nv < lo {
            r = (x + v * dt) - acc * dt * 0.5 * dt;
            nv = lo;
        }
    }
    (r, nv)
}

/// 0x0011e9f8: limit a vector's length to `max`.
pub fn FUN_0011e9f8(max: f32, v: V3) -> V3 {
    let l = len(v);
    if max < l { scale(v, max / l) } else { v }
}

/// 0x0011bff0: matrix -> Euler (x from atan2(m[6], m[10]), y = asin(-m[2]) clamped, z = atan2(m[1], m[0]));
/// gimbal cases as in the original. Returns (x, y, z).
pub fn FUN_0011bff0(m: &M4) -> V3 {
    let p = |k: usize| m[k / 4][k % 4];
    let y = FUN_0011cfe0(-p(2), 1.0).asin();
    if y < 1.5706964 {
        if -1.5706964 < y {
            [p(6).atan2(p(10)), y, p(1).atan2(p(0))]
        } else {
            [0.0, y, (-p(4)).atan2(-p(8))]
        }
    } else {
        [0.0, y, (-p(4)).atan2(p(8))]
    }
}

/// 0x0011d948: clamp `x` to the range {lo, hi} (lo wins when lo > hi).
pub fn FUN_0011d948(x: f32, r: [f32; 2]) -> f32 {
    let mut f = r[0];
    if f <= x {
        f = r[1];
        if x <= r[1] {
            f = x;
        }
    }
    f
}

/// 0x0011d2d0: angular FUN_0011d018: approach `target` along the shortest wrapped path.
pub fn FUN_0011d2d0(x: f32, target: f32, dt: f32, smp: &Smp) -> f32 {
    let d = FUN_0011cf60(target - x);
    let r = FUN_0011d018(0.0, d, dt, smp).0;
    FUN_0011cf60(x + r)
}

/// 0x0011d340: angular FUN_0011d1c0. Returns (new angle, new velocity).
pub fn FUN_0011d340(x: f32, v: f32, target: f32, dt: f32, smp: &Smp) -> (f32, f32) {
    let d = FUN_0011cf60(target - x);
    let (r, nv) = FUN_0011d1c0(0.0, v, d, dt, smp);
    (FUN_0011cf60(x + r), nv)
}

/// 0x0011ef08: cylindrical decomposition: (wrapped atan2(y,x), |xy|, z).
pub fn FUN_0011ef08(v: V3) -> (f32, f32, f32) {
    (FUN_0011cf60(v[1].atan2(v[0])), (v[0] * v[0] + v[1] * v[1]).sqrt(), v[2])
}

/// 0x0011bc88: (yaw, pitch) of a direction (unwrapped atan2; the column is matrix col 0 for the camera).
pub fn FUN_0011bc88(v: V3) -> (f32, f32) {
    (v[1].atan2(v[0]), v[2].atan2((v[0] * v[0] + v[1] * v[1]).sqrt()))
}
