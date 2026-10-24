//! WP-K21: Murray (class 54 C2d44c8) FUN_001b93e8 finished: the attach (sub-state 1) / throw (2) setup of the
//! held object (+0x13f0 +0x180).
//! 1:1: the call order, the record words +0x1404 / +0x1408 (bone ids), +0x13fc / +0x1400 (attach acts),
//! +0x13f8 / +0x13f4 (throw pins), +0x1610 / +0x1614 grip records (+0 / +0xcc / +0xa0..), +0x1618..+0x1624,
//! +0x1604 / +0x160c, the +0x190..+0x198 clear and the anim phase fStack_70 = (label 0x1c7 - +0x1c) / +0x40.
//! REPLACED: the matrix callees (FUN_002309e8 stack matrix, FUN_001a71b0, FUN_001b72b0 hand frames,
//! FUN_0022fcc8 of the bone frame) and the act builders (FUN_0012de58 / 12dc18 / 12dd28 / 12ddb0) are
//! `ee_call(addr, args)` with object handles; their vector arguments are not modelled (0).
#![allow(non_snake_case)]
use super::k18_54::C2d44c8;
use super::k18_55::Cx;

/// DAT_002c0a3c / DAT_002c0a40: grip record +0xcc (left / right)
pub const F_2C0A3C: f32 = 4.75;
pub const F_2C0A40: f32 = 1.5;

impl C2d44c8 {
    /// FUN_001b93e8 sub-states 1 / 2 with the held object `x` in the world
    pub fn k21_1b93e8_setup(&mut self, obj: usize, s: i32, x: u32, cx: &mut Cx) {
        let o = obj as u32 + 1;
        let h = &mut *cx.h;
        let m = &mut self.b.m;
        if s == 1 {
            h.ee_call(0x1f5720, &[x, 0]);
            let b58 = h.obj_i32(x, 0x58) as u32;
            let b5c = h.obj_i32(x, 0x5c) as u32;
            m.set_i(0x1404, h.obj_i32(b58, 0x98) as i16 as i32);
            m.set_i(0x1408, h.obj_i32(b5c, 0x98) as i16 as i32);
            h.ee_call(0x1a58d0, &[x, 0, 0xf]);
            h.ee_call(0x1f6860, &[x, 1]);
            let anim = m.u(0x1030);
            let a14 = h.obj_i32(anim, 0x14) as u32;
            let lt = f32::from_bits(h.ee_call(0x12f938, &[a14, 0x1c7]) as u32);
            let t70 = (lt - h.obj_f32(anim, 0x1c)) / h.obj_f32(anim, 0x40);
            let (c60, s60) = (h.obj_i32(m.u(0x13ec), 0x60) as u32, h.obj_i32(o, 0x60) as u32);
            h.ee_call(0x2309e8, &[t70.to_bits(), c60, s60]);
            h.ee_call(0x1a71b0, &[o, 0]);
            h.ee_call(0x1b72b0, &[o, 0]);
            if h.ee_call(0x132b10, &[x, 0x38]) != 0 {
                h.ee_call(0x209e28, &[x, o]);
            }
            h.ee_call(0x1b7700, &[o]);
            m.set_u(0x13fc, h.ee_call(0x12de58, &[(cx.g.t + t70).to_bits(), x, b58, 5]) as u32);
            h.vcall(x, 0x11c);
            m.set_u(0x1400, h.ee_call(0x12dc18, &[x, b5c, 5, 7, 2]) as u32);
            // TAKEN BY MURRAY: the attach acts' effect (REPLACED): x follows Murray's hands
            h.so_carry(x, o);
        } else {
            let carrier = m.u(0x13ec);
            h.ee_call(0x1a58d0, &[x, carrier, 0xf]);
            h.ee_call(0x1f6860, &[x, 1]);
            let c60 = h.obj_i32(carrier, 0x60) as u32;
            h.ee_call(0x1b72b0, &[o, c60, 0x2a2d30]);
            let b58 = h.obj_i32(x, 0x58) as u32;
            let b5c = h.obj_i32(x, 0x5c) as u32;
            m.set_u(0x13f8, h.ee_call(0x12dc18, &[x, b5c, 5, 7, 2]) as u32);
            m.set_u(0x13f4, h.ee_call(0x12dc18, &[x, b58, 5, 7, 2]) as u32);
            // TAKEN BY MURRAY: the throw pins' effect (REPLACED): x stays in Murray's hands until released
            h.so_carry(x, o);
            let c = m.u(0x13f0);
            // the two grips: (+0x190 hand, record +0x1610, acts +0x1618 / 161c, pin +0x15f4 -> +0x1604) and
            // (+0x198, +0x1614, +0x1620 / 1624, +0x15fc -> +0x160c)
            for (src, rec, a, b, pin, out, f) in
                [(0x190, 0x1610, 0x1618, 0x161c, 0x15f4, 0x1604, F_2C0A3C), (0x198, 0x1614, 0x1620, 0x1624, 0x15fc, 0x160c, F_2C0A40)]
            {
                let g = h.obj_i32(c, src) as u32;
                if g == 0 {
                    continue;
                }
                let r = m.u(rec);
                let v = h.vcall_ret(g, 0x104) as u32;
                h.obj_set_u32(r, 0, v);
                h.obj_set_u32(r, 0xcc, f.to_bits());
                let r1 = h.obj_i32(r, 4) as u32;
                let r2 = h.obj_i32(r, 8) as u32;
                m.set_u(a, h.ee_call(0x12dd28, &[o, r1, 5, r]) as u32);
                m.set_u(b, h.ee_call(0x12dd28, &[o, r2, 5, r]) as u32);
                m.set_u(out, h.ee_call(0x12ddb0, &[o, m.u(pin), 5, 5, v]) as u32);
            }
            if h.ee_call(0x132b10, &[x, 0x38]) != 0 {
                h.ee_call(0x209e88, &[x]);
            }
            m.set_v3(0x190, [0.0; 3]);
        }
    }
}
