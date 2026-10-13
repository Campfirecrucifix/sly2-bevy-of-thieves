//! C2ce4d0 (world class 33).
use super::ClassHost;
use crate::engine::physics::driven::GameRng;
use crate::engine::world::props::{Prop, PropValue};

/// The emitter template at +0xe4 (0x100 bytes, copied to the stack by slot 51) and the 0x14-byte
/// block its +0x08 pointer refers to (word 0 = source object, word 4 = +0x10).
#[derive(Clone, Debug)]
pub struct Template {
    pub w: [u32; 64],
    pub block: [u32; 5],
}
impl Default for Template {
    fn default() -> Self {
        Template { w: [0; 64], block: [0; 5] }
    }
}

#[derive(Clone, Debug, Default)]
pub struct C2ce4d0 {
    /// +0xe4
    pub tpl: Template,
    /// +0xf0: sound descriptor (FUN_0015c650 allocates it on first field write); words +0..+0x10
    pub f_f0: Option<[u32; 5]>,
    /// +0xe0 template dirty / class-32 box (K23, classes/k23_32.rs)
    pub f_e0: i32,
    pub bx: [f32; 8],
    /// +0xf4 / +0xf8: random count range
    pub f_f4: f32,
    pub f_f8: f32,
    /// +0xfc
    pub f_fc: i32,
    pub f_100: i32,
    pub f_104: i32,
}

/// slot 5: FUN_00113c20
pub fn ee_sub_113c20() -> u32 {
    0x2bad7c
}

fn f(u: u32) -> f32 {
    f32::from_bits(u)
}

impl C2ce4d0 {
    /// Load-time field writes (field table 0x2ac880): 555 -> +0xf4 (2 floats); 590/591/592/593 -> +0xf0 sound
    /// descriptor +8/+4/+0xc/+0x10 (setter 0x15c650); setter 0x2590f0 fields write the +0xe4 template.
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            let PropValue::Scalar(b) = &p.value else { continue };
            let words: Vec<u32> = b.chunks_exact(4).map(|c| u32::from_le_bytes(c.try_into().unwrap())).collect();
            let tpl_off = match p.field {
                547 => Some(0x18),
                548 => Some(0x50),
                549 => Some(0x5c),
                550 => Some(0x90),
                552 => Some(0x88),
                559 => Some(0x58),
                562 => Some(0x8c),
                _ => None,
            };
            if let Some(o) = tpl_off {
                for (k, w) in words.iter().enumerate() {
                    self.tpl.w[o / 4 + k] = *w;
                }
                continue;
            }
            let sd = match p.field {
                590 => 2,
                591 => 1,
                592 => 3,
                593 => 4,
                _ => {
                    if p.field == 555 && words.len() == 2 {
                        self.f_f4 = f(words[0]);
                        self.f_f8 = f(words[1]);
                    }
                    continue;
                }
            };
            if let Some(w) = words.first() {
                self.f_f0.get_or_insert([0; 5])[sd] = *w;
            }
        }
    }

    /// slot 4 (message handler): FUN_0015c5c8
    pub fn ee_sub_15c5c8(&mut self, obj: usize, msg: i32, p: i32, h: &mut dyn ClassHost) {
        if msg == 1 {
            if p == self.f_100 {
                h.ee_sub_133448(p, obj);
                self.f_100 = 0;
            } else if p == self.f_104 {
                h.ee_sub_133448(p, obj);
                self.f_104 = 0;
            }
        }
    }
    /// slot 50: FUN_0015c628
    pub fn ee_sub_15c628(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.sub_vcall_1a4(obj);
    }

    /// slot 51: FUN_0015c690 — emit with request `r` (15 words, r[0] = flags) from a stack copy of the template.
    /// `obj` doubles as the source-object word (index, not a pointer).
    pub fn ee_sub_15c690(&mut self, obj: usize, r: &[u32; 15], rng: &mut GameRng, h: &mut dyn ClassHost) {
        let mut t = self.tpl.w;
        let mut blk = self.tpl.block;
        let mut count: i32 = if r[0] & 0x80 == 0 {
            rng.FUN_0011d448(self.f_f4, self.f_f8) as u32 as i32
        } else {
            r[14] as i32
        };
        let n0 = count;
        if r[0] & 1 != 0 {
            blk[0] = r[1];
        }
        if r[0] & 2 != 0 {
            t[16..20].copy_from_slice(&r[4..8]);
        }
        if r[0] & 4 != 0 {
            t[12..16].copy_from_slice(&r[8..12]);
            t[4] = 0;
        } else if r[0] & 0x40 != 0 {
            t[7] = r[13];
        }
        if r[0] & 8 != 0 && t[1] == 0 {
            let mut fv = f(r[2]);
            if fv == 0.0 {
                fv = f(r[3]) * 0.01;
            }
            let n = if fv < 1.0 { n0 as f32 * fv } else { n0 as f32 / fv } as i32;
            let half = (self.f_f8 as i32) / 2;
            count = n.max(half);
            let s = h.ee_sub_11d948(fv, 0x2bdec0);
            t[20] = (f(t[20]) * s).to_bits();
            t[21] = (f(t[21]) * s).to_bits();
            // FUN_00256ce8/256d08/256d28/256d18: scale the +0xe0 vector block by max(fv, 0.5) (fv < 0.5 -> 0.5)
            let k = if fv < 0.5 { 0.5 } else { fv };
            h.ee_sub_256ce8(&mut t[56..64], k);
        }
        if t[1] == 1 {
            if r[0] & 0x10 != 0 {
                t[54] = r[2];
                t[55] = r[2];
            }
            if r[0] & 0x20 != 0 {
                blk[4] = r[12];
            }
        }
        let link = self.f_fc != 0;
        if blk[0] == 0 && t[4] != 2 && r[0] & 0x48 == 0 {
            blk[0] = obj as u32;
        }
        h.ee_sub_15bd20(obj, count, &t, &blk, link);
        let Some(sd) = self.f_f0 else { return };
        // position: t[4] mode 1 = midpoint of the two vectors at t[12], 0 = t[12..16], 2 = blk[1]'s +0xf4,
        // 3 = *(t[12] + 0x20); then through the t[7] matrix when set; FUN_001a6f00(blk[0], 0, pos)
        let pos = h.emit_pos_15c690(&t, &blk);
        h.sound_15c690(blk[0], pos, sd);
    }
}
