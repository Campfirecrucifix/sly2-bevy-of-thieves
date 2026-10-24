//! VFIX: PS2 EE FPU (COP1) float semantics — see recomp/ENGINE_PORT.md "## FPU semantics".
//! The PS2 FPU has no inf/NaN: overflow and division by zero give +-max float (0x7f7fffff).

/// Clamp an IEEE result to PS2 range: +-inf -> +-max float; denormal results flush to signed zero
/// (the EE FPU has no denormals).
#[inline]
pub fn ps2_clamp(x: f32) -> f32 {
    if x.is_infinite() { f32::MAX.copysign(x) } else if x.is_subnormal() { 0.0f32.copysign(x) } else { x }
}

/// PS2 FPU / VU operand decode: there is no inf / NaN, exponent 255 is an ordinary (larger) exponent, so
/// such bit patterns are big finite numbers (up to ~2^129). Returned as f64 so the arithmetic can be done
/// without IEEE inf / NaN and clamped once (`ps2_round`).
#[inline]
pub fn ps2_val(x: f32) -> f64 {
    let b = x.to_bits();
    let e = (b >> 23) & 0xff;
    if e == 0xff {
        let m = 1.0 + (b & 0x7f_ffff) as f64 / 8_388_608.0;
        let v = m * 2f64.powi(128);
        if b >> 31 != 0 { -v } else { v }
    } else {
        x as f64
    }
}
/// f64 result -> PS2 f32: |x| > FLT_MAX saturates (sign kept), denormals flush to signed zero.
#[inline]
pub fn ps2_round(x: f64) -> f32 {
    if x.is_nan() {
        return f32::MAX;
    }
    if x.abs() > f32::MAX as f64 {
        return f32::MAX.copysign(x as f32);
    }
    ps2_clamp(x as f32)
}

/// DIV.S: x / 0 (including 0 / 0) = +-max float with sign = sign(a) ^ sign(b); otherwise clamped.
#[inline]
pub fn ps2_div(a: f32, b: f32) -> f32 {
    if b == 0.0 {
        f32::MAX.copysign(if a.is_sign_negative() != b.is_sign_negative() { -1.0 } else { 1.0 })
    } else {
        ps2_round(ps2_val(a) / ps2_val(b))
    }
}

/// ADD.S/SUB.S/MUL.S with PS2 overflow clamping.
#[inline]
pub fn ps2_add(a: f32, b: f32) -> f32 { ps2_round(ps2_val(a) + ps2_val(b)) }
#[inline]
pub fn ps2_sub(a: f32, b: f32) -> f32 { ps2_round(ps2_val(a) - ps2_val(b)) }
#[inline]
pub fn ps2_mul(a: f32, b: f32) -> f32 { ps2_round(ps2_val(a) * ps2_val(b)) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn div_by_zero_is_max() {
        assert_eq!(ps2_div(1.0, 0.0), f32::MAX);
        assert_eq!(ps2_div(-1.0, 0.0), -f32::MAX);
        assert_eq!(ps2_div(0.0, 0.0), f32::MAX);
        assert_eq!(ps2_div(ps2_div(1.0, 0.0), ps2_div(1.0, 0.0)), 1.0);
        assert_eq!(ps2_mul(f32::MAX, 2.0), f32::MAX);
        assert_eq!(ps2_mul(1e-20, 1e-20), 0.0);
        // exponent-255 operands are finite on the PS2: 0 * "NaN bits" = 0, not NaN
        assert_eq!(ps2_mul(f32::from_bits(0xffff_ffff), -0.0).to_bits() & 0x7fff_ffff, 0);
        assert_eq!(ps2_add(f32::from_bits(0x7f80_0000), -f32::MAX), ps2_round(2f64.powi(128) - f32::MAX as f64));
        assert_eq!(ps2_add(f32::from_bits(0x0080_0001), -f32::MIN_POSITIVE).to_bits(), 0);
    }
}
