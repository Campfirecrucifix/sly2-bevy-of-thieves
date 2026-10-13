//! C2dad90 (world class 129, parent = the sequencer class of FUN_002123c8 / FUN_00212c50): counts how many of
//! its listed class-133 objects are done and finishes (vtable +0x194) or fails (+0x18c) its own sequence.
//! Lists +0x110 / +0x194 are filled by the load-time method calls of fields 1150 / 1151. Handles = index + 1.
use super::c2cf378::{ee_sub_164030, C133View};
use super::ClassHost;
use crate::engine::world::props::{ArrayItem, Prop, PropValue};

/// One list slot: (oid, resolved object; 0 = unresolved)
pub type Slot = (i32, u32);

#[derive(Clone, Debug, Default)]
pub struct C2dad90 {
    /// +0x110 count + (oid, object) pairs at +0x114 (max 16; FUN_00211e70)
    pub f_110: Vec<Slot>,
    /// +0x194 count + pairs at +0x198 (max 16; FUN_00211e98)
    pub f_194: Vec<Slot>,
}

/// slot 5: FUN_00114220
pub fn ee_sub_114220() -> u32 {
    0x2bb4fc
}

/// 0x2c3ec0 / 0x2c3ef0 / 0x2c3f20 counter text buffers (FUN_0021ecd8 with "%d")
pub const TXT_DONE: u32 = 0x2c3ec0;
pub const TXT_TOTAL: u32 = 0x2c3ef0;
pub const TXT_LEFT: u32 = 0x2c3f20;

impl C2dad90 {
    /// FUN_00211e70
    pub fn ee_sub_211e70(&mut self, oid: i32) {
        if self.f_110.len() < 0x10 {
            self.f_110.push((oid, 0));
        }
    }

    /// FUN_00211e98
    pub fn ee_sub_211e98(&mut self, oid: i32) {
        if self.f_194.len() < 0x10 {
            self.f_194.push((oid, 0));
        }
    }

    /// Load-time method calls: field 1150 (thunk 0x10def8 -> FUN_00211e70), 1151 (0x10df80 -> FUN_00211e98).
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            if p.field != 1150 && p.field != 1151 {
                continue;
            }
            let b = match &p.value {
                PropValue::Array(items) => match items.first() {
                    Some(ArrayItem::One(_, b)) => b.clone(),
                    Some(ArrayItem::List(_, v)) => v.first().cloned().unwrap_or_default(),
                    None => continue,
                },
                PropValue::Scalar(b) => b.clone(),
            };
            let oid = b.get(..4).map(|x| i32::from_le_bytes([x[0], x[1], x[2], x[3]])).unwrap_or(0);
            if p.field == 1150 {
                self.ee_sub_211e70(oid);
            } else {
                self.ee_sub_211e98(oid);
            }
        }
    }

    /// slot 14: FUN_00211ec0
    pub fn ee_sub_211ec0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x2123c8, obj);
        for s in self.f_194.iter_mut().chain(self.f_110.iter_mut()) {
            let o = h.ee_sub_15e938(5, s.0, 0);
            if o != 0 {
                s.1 = o;
            }
        }
    }

    /// slot 15: FUN_00211f90
    pub fn ee_sub_211f90(&mut self, obj: usize, c133: &dyn Fn(u32) -> Option<C133View>, h: &mut dyn ClassHost) {
        let me = obj as u32 + 1;
        h.base(0x212c50, obj);
        let (mut n_one, mut n_done, mut n_req) = (0, 0, 0);
        for &(_, o) in &self.f_194 {
            if o == 0 {
                continue;
            }
            let v = c133(o).unwrap_or_default();
            if v.f_54 == 1 {
                n_one += 1;
            } else if ee_sub_164030(&v, h) {
                n_done += 1;
                n_req += 1;
            }
        }
        for &(_, o) in &self.f_110 {
            if o == 0 {
                continue;
            }
            let v = c133(o).unwrap_or_default();
            if v.f_54 == 1 {
                n_one += 1;
            } else if ee_sub_164030(&v, h) {
                n_done += 1;
            }
        }
        if h.ee_sub_2121e8(me, 0x314) {
            let total = self.f_194.len() as i32;
            h.ee_sub_21ecd8(TXT_DONE, n_req);
            h.ee_sub_21ecd8(TXT_TOTAL, total);
            h.ee_sub_21ecd8(TXT_LEFT, total - n_req);
        }
        if n_done != self.f_194.len() + self.f_110.len() && n_one != 0 {
            h.vcall_arg(me, 0x18c, -1);
            return;
        }
        h.vcall(me, 0x194);
    }
}
