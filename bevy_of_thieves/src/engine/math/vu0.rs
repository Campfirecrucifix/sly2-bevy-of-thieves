//! WP-HARD1: VU0 macro-mode (COP2) and EE MMI lane operations, as used by the game's inline VU0 code.
//! Floats are `[f32; 4]` (x, y, z, w = lanes 0..3), 128-bit GPRs are `[u32; 4]` (lane 0 = low word).
//! PS2 float quirks kept: the VU FPU has no inf / NaN / denormals (results saturate to +-FLT_MAX,
//! denormals flush to 0), SQRT / RSQRT use |x|, DIV by 0 gives +-FLT_MAX (sign = xor of the operand signs).
pub type V4 = [f32; 4];
pub type Q = [u32; 4];
use super::ps2f::{ps2_round as r_, ps2_val as vv};
#[inline]
fn v(x: f32) -> f64 {
    vv(x)
}
#[inline]
fn r(x: f64) -> f32 {
    r_(x)
}

/// VU result normalisation: inf / NaN -> +-FLT_MAX, denormal -> +-0
pub fn fix(x: f32) -> f32 {
    // (inputs here are IEEE results; exp-255 operands are decoded by `v` below, so NaN can't arise from them)
    if x.is_nan() || x.is_infinite() {
        f32::MAX.copysign(x)
    } else if x != 0.0 && x.abs() < f32::MIN_POSITIVE {
        0.0f32.copysign(x)
    } else {
        x
    }
}
/// `VSQRT Q, ft` (sqrt of |ft|)
pub fn vsqrt(x: f32) -> f32 {
    r(v(x).abs().sqrt())
}
/// `VDIV Q, fs, ft` (fs / ft; ft == 0 -> +-FLT_MAX with the xor sign)
pub fn vdiv(n: f32, d: f32) -> f32 {
    if d == 0.0 {
        let neg = n.is_sign_negative() ^ d.is_sign_negative();
        if neg { -f32::MAX } else { f32::MAX }
    } else {
        r(v(n) / v(d))
    }
}
/// `VRSQRT Q, fs, ft` (fs / sqrt(|ft|))
pub fn vrsqrt(n: f32, d: f32) -> f32 {
    vdiv(n, d.abs().sqrt())
}
pub fn add(a: V4, b: V4) -> V4 {
    std::array::from_fn(|i| r(v(a[i]) + v(b[i])))
}
pub fn sub(a: V4, b: V4) -> V4 {
    std::array::from_fn(|i| r(v(a[i]) - v(b[i])))
}
pub fn mul(a: V4, b: V4) -> V4 {
    std::array::from_fn(|i| r(v(a[i]) * v(b[i])))
}
/// `VMULx/VMULq` (broadcast)
pub fn scale(a: V4, s: f32) -> V4 {
    std::array::from_fn(|i| r(v(a[i]) * v(s)))
}
/// write only the lanes of `mask` (bit 0 = x .. bit 3 = w) of `v` into `d`
pub fn masked(d: V4, v: V4, mask: u8) -> V4 {
    std::array::from_fn(|i| if mask >> i & 1 != 0 { v[i] } else { d[i] })
}
/// `VMUL.xyz t,a,b ; VADDAy ; VMADDz` = x*x' + y*y' + z*z' (x lane)
pub fn dot3(a: V4, b: V4) -> f32 {
    r(v(a[0]) * v(b[0]) + v(a[1]) * v(b[1]) + v(a[2]) * v(b[2]))
}
/// `VOPMULA ACC, a, b ; VOPMSUB d, b, a` = a x b (xyz; w of the destination untouched -> 0 here)
pub fn cross(a: V4, b: V4) -> V4 {
    let c = |p: usize, q: usize| r(v(a[p]) * v(b[q]) - v(a[q]) * v(b[p]));
    [c(1, 2), c(2, 0), c(0, 1), 0.0]
}
/// `VMULAx ACC, r0, v ; VMADDAy ACC, r1, v ; VMADDz d, r2, v` (all four lanes): r0*v.x + r1*v.y + r2*v.z
pub fn mul3(r: &[V4; 3], v: V4) -> V4 {
    let w = v;
    std::array::from_fn(|i| r_(vv(r[0][i]) * vv(w[0]) + vv(r[1][i]) * vv(w[1]) + vv(r[2][i]) * vv(w[2])))
}
/// `... ; VMADDw d, r3, v` (VU0 4x4 matrix * vector, rows = columns of the math)
pub fn mul4(r: &[V4; 4], v: V4) -> V4 {
    let w = v;
    std::array::from_fn(|i| r_(vv(r[0][i]) * vv(w[0]) + vv(r[1][i]) * vv(w[1]) + vv(r[2][i]) * vv(w[2]) + vv(r[3][i]) * vv(w[3])))
}
pub fn qbits(v: V4) -> Q {
    v.map(f32::to_bits)
}
pub fn qf(q: Q) -> V4 {
    q.map(f32::from_bits)
}
/// MMI `PEXTLW rd, rs, rt`: interleave the low words -> (rt0, rs0, rt1, rs1)
pub fn pextlw(rs: Q, rt: Q) -> Q {
    [rt[0], rs[0], rt[1], rs[1]]
}
/// MMI `PEXTUW rd, rs, rt`: interleave the high words -> (rt2, rs2, rt3, rs3)
pub fn pextuw(rs: Q, rt: Q) -> Q {
    [rt[2], rs[2], rt[3], rs[3]]
}
/// MMI `PCPYLD rd, rs, rt`: rd.lo = rt.lo, rd.hi = rs.lo
pub fn pcpyld(rs: Q, rt: Q) -> Q {
    [rt[0], rt[1], rs[0], rs[1]]
}
/// MMI `PCPYUD rd, rs, rt`: rd.lo = rs.hi, rd.hi = rt.hi
pub fn pcpyud(rs: Q, rt: Q) -> Q {
    [rs[2], rs[3], rt[2], rt[3]]
}
/// MMI `PROT3W rd, rt`: rotate the low three words -> (rt1, rt2, rt0, rt3)
pub fn prot3w(rt: Q) -> Q {
    [rt[1], rt[2], rt[0], rt[3]]
}
/// MMI `POR rd, rs, rt`
pub fn por(rs: Q, rt: Q) -> Q {
    std::array::from_fn(|i| rs[i] | rt[i])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sqrt_div_quirks() {
        assert_eq!(vsqrt(-4.0), 2.0);
        assert_eq!(vdiv(1.0, 0.0), f32::MAX);
        assert_eq!(vdiv(-1.0, 0.0), -f32::MAX);
        assert_eq!(vdiv(1.0, -0.0), -f32::MAX);
        assert_eq!(vdiv(6.0, 3.0), 2.0);
        assert_eq!(vrsqrt(1.0, -4.0), 0.5);
        assert_eq!(fix(f32::INFINITY), f32::MAX);
        assert_eq!(fix(1e-40), 0.0);
    }
    #[test]
    fn vector_ops() {
        let a = [1.0, 2.0, 3.0, 4.0];
        let b = [5.0, 6.0, 7.0, 8.0];
        assert_eq!(add(a, b), [6.0, 8.0, 10.0, 12.0]);
        assert_eq!(sub(b, a), [4.0; 4]);
        assert_eq!(mul(a, b), [5.0, 12.0, 21.0, 32.0]);
        assert_eq!(scale(a, 2.0), [2.0, 4.0, 6.0, 8.0]);
        assert_eq!(dot3(a, b), 38.0);
        assert_eq!(cross([1.0, 0.0, 0.0, 9.0], [0.0, 1.0, 0.0, 9.0]), [0.0, 0.0, 1.0, 0.0]);
        assert_eq!(masked(a, b, 0b0101), [5.0, 2.0, 7.0, 4.0]);
        let r = [[1.0, 0.0, 0.0, 0.0], [0.0, 2.0, 0.0, 0.0], [0.0, 0.0, 3.0, 0.0]];
        assert_eq!(mul3(&r, a), [1.0, 4.0, 9.0, 0.0]);
        let r4 = [r[0], r[1], r[2], [1.0, 1.0, 1.0, 1.0]];
        assert_eq!(mul4(&r4, a), [5.0, 8.0, 13.0, 4.0]);
    }
    #[test]
    fn mmi_ops() {
        let s = [0, 1, 2, 3];
        let t = [10, 11, 12, 13];
        assert_eq!(pextlw(s, t), [10, 0, 11, 1]);
        assert_eq!(pextuw(s, t), [12, 2, 13, 3]);
        assert_eq!(pcpyld(s, t), [10, 11, 0, 1]);
        assert_eq!(pcpyud(s, t), [2, 3, 12, 13]);
        assert_eq!(prot3w(t), [11, 12, 10, 13]);
        assert_eq!(por([1, 2, 4, 8], [8, 4, 2, 1]), [9, 6, 6, 9]);
        assert_eq!(qf(qbits([1.5, 2.0, 0.0, -1.0])), [1.5, 2.0, 0.0, -1.0]);
    }
}
