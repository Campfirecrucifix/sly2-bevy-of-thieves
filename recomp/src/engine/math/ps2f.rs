//! VFIX: PS2 EE FPU (COP1) float semantics — see recomp/ENGINE_PORT.md "## FPU semantics".
//! The PS2 FPU has no inf/NaN: overflow and division by zero give +-max float (0x7f7fffff).

/// Clamp an IEEE result to PS2 range: +-inf -> +-max float.
#[inline]
pub fn ps2_clamp(x: f32) -> f32 {
    if x.is_infinite() { f32::MAX.copysign(x) } else { x }
}

/// DIV.S: x / 0 (including 0 / 0) = +-max float with sign = sign(a) ^ sign(b); otherwise clamped.
#[inline]
pub fn ps2_div(a: f32, b: f32) -> f32 {
    if b == 0.0 {
        f32::MAX.copysign(if a.is_sign_negative() != b.is_sign_negative() { -1.0 } else { 1.0 })
    } else {
        ps2_clamp(a / b)
    }
}

/// ADD.S/SUB.S/MUL.S with PS2 overflow clamping.
#[inline]
pub fn ps2_add(a: f32, b: f32) -> f32 { ps2_clamp(a + b) }
#[inline]
pub fn ps2_sub(a: f32, b: f32) -> f32 { ps2_clamp(a - b) }
#[inline]
pub fn ps2_mul(a: f32, b: f32) -> f32 { ps2_clamp(a * b) }

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
    }
}
