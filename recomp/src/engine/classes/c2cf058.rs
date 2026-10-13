//! C2cf058 (world class 116): timed full-screen colour fade. FUN_0015e1d8 (script method, field 979) stamps
//! the start time (+0x68) and adds the object (vtable +0x44); slot 15 removes it (vtable +0x4c) once the
//! three phases +0x5c / +0x60 / +0x64 are over; slot 18 draws the screen quad in the interpolated colour
//! (+0x50 -> +0x54 hold -> +0x58). Handles in hooks = index + 1.
use super::k4::{f32_of, word};
use super::{ClassGlobals, ClassHost};
use crate::engine::world::props::{Prop, PropValue};

#[derive(Clone, Debug, Default)]
pub struct C2cf058 {
    /// +0x28 bit 60: in the world (vtable +0x44 add / +0x4c remove)
    pub active: bool,
    /// +0x40 flags (slot 6 |= 10)
    pub f_40: u32,
    /// +0x50 / +0x54 / +0x58 RGBA colours (fields 980-982)
    pub f_50: u32,
    pub f_54: u32,
    pub f_58: u32,
    /// +0x5c fade-in, +0x60 hold, +0x64 fade-out durations (fields 983 / 985 / 984)
    pub f_5c: f32,
    pub f_60: f32,
    pub f_64: f32,
    /// +0x68 start time
    pub f_68: f32,
    /// +0x6c blend mode (field 986): 0 -> draw record kind 1, 1 -> 0xe
    pub f_6c: i32,
}

/// slot 5: FUN_00114150
pub fn ee_sub_114150() -> u32 {
    0x2bb3f8
}

/// One channel step of FUN_0015dd48: from `a` toward `b` by `f` (+0.999 so the end value is reached)
fn chan(a: u8, b: u8, f: f32) -> u8 {
    if a == b {
        b
    } else if a < b {
        a.wrapping_add((f * ((b as f32 + 0.999) - a as f32)) as i32 as u8)
    } else {
        b.wrapping_add(((1.0 - f) * ((a as f32 + 0.999) - b as f32)) as i32 as u8)
    }
}

impl C2cf058 {
    /// Load fields 980-986; the field-979 method call (thunk 0x10c6a8 -> FUN_0015e1d8) if present.
    pub fn feed(&mut self, obj: usize, props: &[Prop], g: &ClassGlobals, h: &mut dyn ClassHost) {
        for p in props {
            if p.field == 979 {
                self.ee_sub_15e1d8(obj, g, h);
                continue;
            }
            let PropValue::Scalar(b) = &p.value else { continue };
            match p.field {
                980 => self.f_50 = word(b),
                981 => self.f_54 = word(b),
                982 => self.f_58 = word(b),
                983 => self.f_5c = f32_of(b),
                984 => self.f_64 = f32_of(b),
                985 => self.f_60 = f32_of(b),
                986 => self.f_6c = word(b) as i32,
                _ => {}
            }
        }
    }

    /// slot 6: FUN_0015dc80
    pub fn ee_sub_15dc80(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5270, obj);
        h.ee_sub_1a6ca8(obj, 1);
        self.f_40 |= 10;
    }

    /// FUN_0015e1d8: start (once)
    pub fn ee_sub_15e1d8(&mut self, obj: usize, g: &ClassGlobals, h: &mut dyn ClassHost) {
        if !self.active {
            self.f_68 = g.t;
            self.active = true;
            h.vcall(obj as u32 + 1, 0x44);
        }
    }

    /// slot 15: FUN_0015dcd8
    pub fn ee_sub_15dcd8(&mut self, obj: usize, g: &ClassGlobals, h: &mut dyn ClassHost) {
        h.ee_sub_1a5e28(obj);
        if self.f_5c + self.f_60 + self.f_64 < g.t - self.f_68 {
            self.active = false;
            h.vcall(obj as u32 + 1, 0x4c);
        }
    }

    /// FUN_0015dd48 colour part: (phase start colour, end colour, fraction) -> drawn RGBA
    pub fn colour(&self, t: f32) -> u32 {
        let mut f = t - self.f_68;
        let (a, b, d);
        if f < self.f_5c {
            (a, b, d) = (self.f_50, self.f_54, self.f_5c);
        } else {
            f -= self.f_5c;
            if f < self.f_60 {
                (a, b, d) = (self.f_54, self.f_54, self.f_60);
            } else {
                // past the fade-out the original reads uninitialised locals; slot 15 removes the
                // object first, so this clamps to the fade-out end
                f -= self.f_60;
                (a, b, d) = (self.f_54, self.f_58, self.f_64);
                f = f.min(d);
            }
        }
        let f = f / d;
        let (a, b) = (a.to_le_bytes(), b.to_le_bytes());
        u32::from_le_bytes([chan(a[0], b[0], f), chan(a[1], b[1], f), chan(a[2], b[2], f), chan(a[3], b[3], f)])
    }

    /// slot 18: FUN_0015dd48 (quad build FUN_0016f960 + DMA list 0x2dea28 + draw record = hook)
    pub fn ee_sub_15dd48(&mut self, _obj: usize, g: &ClassGlobals, h: &mut dyn ClassHost) {
        let c = self.colour(g.t);
        let kind = match self.f_6c {
            0 => 1,
            1 => 0xe,
            _ => -1,
        };
        h.ee_sub_15dd48_draw(c, kind);
    }
}
