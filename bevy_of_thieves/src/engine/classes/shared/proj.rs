//! C2cb780 (class 79) projectile base and its C2d3aa0 (class 82) layer: init FUN_0013a9e8 / FUN_001b1db8 and
//! the field rows 383-412 (member offsets from FIELD_MEMBER). Raw words are kept for the members the ported
//! slots do not interpret (oids stay oids: the load-time resolve is part of the unported class-79/82 slots).
use super::act::XfActs;
use crate::engine::classes::k4::word;
use crate::engine::classes::ClassHost;
use crate::engine::physics::driver::AloPhys;
use crate::engine::world::props::{Prop, PropValue};

/// field row -> member offset (FIELD_BINDINGS / FIELD_MEMBER, classes 79 / 82 / 81; rows 389 / 390 / 403-405
/// are arrays / method thunks, not members)
pub const ROWS_79: &[(i16, u16)] = &[
    (383, 0x15c), (384, 0x160), (385, 0x150), (386, 0x170), (387, 0x164), (388, 0x16c), (391, 0x1ec), (392, 0x158),
    (393, 0x1f4), (394, 0x168), (395, 0x1f0), (396, 0x1e0), (397, 0x200), (398, 0x1e4), (399, 0x1e8),
];
pub const ROWS_82: &[(i16, u16)] = &[(400, 0x210), (401, 0x220), (402, 0x264), (406, 0x260), (407, 0x26c), (410, 0x270), (411, 0x274), (412, 0x280)];
pub const ROWS_81: &[(i16, u16)] = &[(400, 0x210), (401, 0x220), (402, 0x264), (406, 0x260), (407, 0x26c), (408, 0x274), (409, 0x270)];

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Proj {
    /// K15: launched (FUN_001d6b00's slot 7 put it in the world; the host world flag +0x28 bit 60 is not
    /// modelled for projectiles)
    pub in_play: bool,
    /// ALO +0x40 flag word (tagged: modelled inline)
    pub f_40: u32,
    /// +0x150, +0x154 (FLT_MAX DAT_002a4b08), +0x158, +0x15c / +0x160 (200 DAT_002bc9c8), +0x164, +0x168 (oid),
    /// +0x16c, +0x170
    pub f_150: u32,
    pub f_154: f32,
    pub f_158: u32,
    pub f_15c: f32,
    pub f_160: f32,
    pub f_164: u32,
    pub f_168: u32,
    pub f_16c: f32,
    pub f_170: u32,
    /// +0x1e0 (1), +0x1e4, +0x1e8 (slot-69 speed hold gate), +0x1ec (0x100), +0x1f0 (-10), +0x1f4, +0x200 (self)
    pub f_1e0: u32,
    pub f_1e4: u32,
    pub f_1e8: u32,
    pub f_1ec: u32,
    pub f_1f0: i32,
    pub f_1f4: u32,
    pub f_200: u32,
    /// class 82: +0x210 (launcher), +0x220 vector, +0x230 anim, +0x234 anim handle, +0x260 (800), +0x264 (1 = act),
    /// +0x268 act record id, +0x26c (1.0), +0x270 cruise speed, +0x274 speed rate, +0x280
    pub f_210: u32,
    pub f_220: [f32; 4],
    pub f_230: u32,
    pub f_234: u32,
    pub f_260: f32,
    pub f_264: u32,
    pub f_268: u32,
    pub f_26c: f32,
    pub f_270: f32,
    pub f_274: f32,
    /// class 82: +0x278 arrival time (slot 77), +0x280 arrival velocity (row 412, vector; zero = estimate)
    pub f_278: f32,
    pub f_280: [f32; 3],
    /// K15 (classes/k15_79.rs): +0x174 explosion recorded, +0x180 its position, +0x190 orientation rows,
    /// +0x1c0 / +0x1d0 linear / angular velocity at that moment (FUN_0013b428)
    pub f_174: u32,
    pub f_180: [f32; 3],
    pub f_190: [[f32; 3]; 3],
    pub f_1c0: [f32; 3],
    pub f_1d0: [f32; 3],
    /// +0x1f8 / +0x1fc line-of-sight records (slot 14 allocates 4 when +0x1f4; FUN_0013ac18)
    pub los: Vec<crate::engine::classes::k15_79::Los>,
    /// class 80: +0x238 / +0x23c ignored oids (method 403), +0x24c / +0x250 ignored classes (method 404)
    pub f_23c: Vec<i32>,
    pub f_250: Vec<i32>,
    /// +0x58 xfLin / +0xe8 physics block (E5 `AloPhys`), +0x5c xfAng actuator list
    pub phys: AloPhys,
    pub xf_ang: XfActs,
}

impl Proj {
    /// FUN_0013a9e8 (class 79 slot 6): base FUN_001f2108; +0x160 = +0x15c = 200, +0x154 = FLT_MAX, +0x1e0 = 1,
    /// +0x1ec = 0x100, +0x1f0 = -10, +0x150 = 0, +0x40 |= 2, +0x200 = self
    pub fn ee_sub_13a9e8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f2108, obj);
        self.f_160 = 200.0;
        self.f_154 = f32::MAX;
        self.f_1e0 = 1;
        self.f_1ec = 0x100;
        self.f_1f0 = -10;
        self.f_15c = 200.0;
        self.f_150 = 0;
        self.f_40 |= 2;
        self.f_200 = obj as u32 + 1;
    }
    /// FUN_001b1db8 (class 80 slot 6; the class-81 / 82 init core): FUN_0013a9e8; +0x264 = 1, +0x26c = 1.0, +0x260 = 800
    pub fn ee_sub_1b1db8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        self.ee_sub_13a9e8(obj, h);
        self.f_264 = 1;
        self.f_26c = 1.0;
        self.f_260 = 800.0;
    }
    /// member write by offset (raw prop word)
    pub fn set(&mut self, off: u16, v: u32) {
        let f = f32::from_bits(v);
        match off {
            0x150 => self.f_150 = v,
            0x154 => self.f_154 = f,
            0x158 => self.f_158 = v,
            0x15c => self.f_15c = f,
            0x160 => self.f_160 = f,
            0x164 => self.f_164 = v,
            0x168 => self.f_168 = v,
            0x16c => self.f_16c = f,
            0x170 => self.f_170 = v,
            0x1e0 => self.f_1e0 = v,
            0x1e4 => self.f_1e4 = v,
            0x1e8 => self.f_1e8 = v,
            0x1ec => self.f_1ec = v,
            0x1f0 => self.f_1f0 = v as i32,
            0x1f4 => self.f_1f4 = v,
            0x200 => self.f_200 = v,
            0x210 => self.f_210 = v,
            0x220 => self.f_220[0] = f,
            0x260 => self.f_260 = f,
            0x264 => self.f_264 = v,
            0x26c => self.f_26c = f,
            0x270 => self.f_270 = f,
            0x274 => self.f_274 = f,
            0x280 => self.f_280[0] = f,
            _ => {}
        }
    }
    /// load-time property rows of the class's table (`ROWS_79` + `ROWS_82` / `ROWS_81`)
    pub fn feed(&mut self, props: &[Prop], rows: &[(i16, u16)]) {
        for p in props {
            // K15: load-time methods 403 / 404 (FUN_001b2020 / FUN_001b2048, flags 0x400) of classes 80-82
            if p.field == 403 || p.field == 404 {
                for a in crate::engine::classes::k15_79::method_args(&p.value) {
                    let l = if p.field == 403 { &mut self.f_23c } else { &mut self.f_250 };
                    if l.len() < 4 {
                        l.push(a as i32);
                    }
                }
                continue;
            }
            let PropValue::Scalar(b) = &p.value else { continue };
            if let Some(&(_, off)) = ROWS_79.iter().chain(rows).find(|r| r.0 == p.field) {
                self.set(off, word(b));
                // +0x280 (row 412) is a vector
                if off == 0x280 {
                    for k in 1..3 {
                        if let Some(w) = b.get(k * 4..k * 4 + 4) {
                            self.f_280[k] = f32::from_bits(word(w));
                        }
                    }
                }
            }
        }
    }
}
