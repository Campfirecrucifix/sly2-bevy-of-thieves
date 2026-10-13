//! WP-K23: class 32 C2ca098 (parent class 30, child class 33 C2ce4d0; 52 instances): the emitter-template holder.
//! Load FUN_00259030 (ported in world::loads_b4) allocates the shared 0x100-byte template at +0xe4
//! (FUN_00159118 defaults), the field setter FUN_002590f0 copy-on-writes it; slot 14 FUN_00259170 resolves the
//! template's object references into the 0x14-byte block at template +8 (FUN_00233178 compare, FUN_002590f0 +
//! FUN_001ae238 copy when changed); slot 40 FUN_002590b8 (FUN_002311b0 + reference count) and slot 50
//! FUN_0015c1e0 (empty). Class 33 inherits slots 14 / 40 and the template fields, so the same functions run on
//! C2ce4d0's template (`ClassSet::post_load_k23`).
//! REPLACED: the shared, reference-counted allocation (FUN_001ae1d0 / FUN_001ae238 ActPool blocks) -> every
//! instance owns its `Template`; w[0] still counts references the way the game does.
use super::c2ce4d0::Template;
use super::c2ceb78::ee_sub_159118;
use super::{ClassHost, ClassSet, Inst};
use crate::engine::world::loads_b4::B4Load;
use crate::engine::world::props::{Prop, PropValue};
use super::{LoadData, Payload};
use crate::engine::world::World;

/// slot 5 FUN_00113c10
pub fn ee_sub_113c10() -> u32 {
    0x2bad68
}

/// slot 50 FUN_0015c1e0: empty
pub fn ee_sub_15c1e0() {}

#[derive(Clone, Debug)]
pub struct C2ca098 {
    /// +0xe0: template dirty
    pub f_e0: i32,
    /// *(+0xe4)
    pub tpl: Template,
    /// fields 538 / 539: shape-1 box min / max (template +0x30 block)
    pub bx: [f32; 8],
}
impl Default for C2ca098 {
    fn default() -> Self {
        C2ca098 { f_e0: 0, tpl: tpl_new(), bx: [0.0; 8] }
    }
}

/// FUN_00259030's template: FUN_00159118 defaults
pub fn tpl_new() -> Template {
    Template { w: ee_sub_159118(), block: [0; 5] }
}

/// Template field writes of field table 0x2ac880 classes 32 / 33 (setter FUN_002590f0 then the member store):
/// 537 / 540 -> +0x30, 541 -> +0x20 (source oid), 542 -> +0x14, 543..553 / 558..564 at their member offsets
/// (FIELD_MEMBER); 538 / 539 the box. `kind` = template +0x10 (load, field 536).
pub fn feed_tpl(t: &mut Template, f_e0: &mut i32, bx: &mut [f32; 8], props: &[Prop], kind: i32) {
    use crate::engine::game::fieldbind::FIELD_MEMBER;
    t.w[4] = kind as u32;
    for p in props {
        let PropValue::Scalar(b) = &p.value else { continue };
        let ws: Vec<u32> = b.chunks_exact(4).map(|c| u32::from_le_bytes(c.try_into().unwrap())).collect();
        let o = match p.field {
            537 | 540 => 0x30,
            541 => 0x20,
            542 => 0x14,
            538 | 539 => {
                let k = if p.field == 538 { 0 } else { 4 };
                for (j, w) in ws.iter().take(3).enumerate() {
                    bx[k + j] = f32::from_bits(*w);
                }
                *f_e0 = 1;
                continue;
            }
            543..=553 | 558..=564 => FIELD_MEMBER[p.field as usize] as usize,
            _ => continue,
        };
        if o >= 0x100 {
            continue;
        }
        for (k, w) in ws.iter().enumerate() {
            if o / 4 + k < 64 {
                t.w[o / 4 + k] = *w;
            }
        }
        *f_e0 = 1;
    }
}

/// FUN_002590f0(obj, 1): copy-on-write the shared template (the old copy keeps refcount - 1)
pub fn ee_sub_2590f0(t: &mut Template, f_e0: &mut i32) {
    *f_e0 = 1;
    if (t.w[0] as i32) < 2 {
        return;
    }
    t.w[0] = 1;
}

/// slot 40 FUN_002590b8: FUN_002311b0 + one more template reference
pub fn ee_sub_2590b8(t: &mut Template, obj: usize, h: &mut dyn ClassHost) {
    h.base(0x2311b0, obj);
    t.w[0] = t.w[0].wrapping_add(1);
}

/// slot 14 FUN_00259170: FUN_001a5c48, then the template's object references (+0x20, kind 2 +0x30; emitter
/// kind +4 == 0: +0xd4, == 1: +0xc8 / +0xcc / +0xd0) through FUN_0015e938(4, oid, obj) into the block
pub fn ee_sub_259170(t: &mut Template, f_e0: &mut i32, obj: usize, h: &mut dyn ClassHost) {
    h.base(0x1a5c48, obj);
    let me = obj as u32 + 1;
    let w = t.w;
    let a = w[8] as i32;
    let b = if w[4] == 2 { w[12] as i32 } else { -1 };
    let mut blk = [0u32; 5];
    if a != -1 {
        blk[0] = h.ee_sub_15e938(4, a, me);
    }
    if b != -1 {
        blk[1] = h.ee_sub_15e938(4, b, me);
        blk[0] = 0;
    }
    if w[1] == 0 {
        if w[53] as i32 != -1 {
            blk[2] = h.ee_sub_15e938(4, w[53] as i32, me);
        }
    } else if w[1] == 1 {
        for (k, x) in [w[50], w[51], w[52]].into_iter().enumerate() {
            if x as i32 != -1 {
                blk[2 + k] = h.ee_sub_15e938(4, x as i32, me);
            }
        }
    }
    // FUN_00233178 against the current block (DAT_002ce4b8 zeros when none); changed -> private copy
    if blk != t.block {
        ee_sub_2590f0(t, f_e0);
        t.block = blk;
    }
}

impl ClassSet {
    /// class 32 instances (class 33 is built in `ClassSet::build`, its template fed here too)
    pub(super) fn build_k23(&mut self, w: &World, _h: &mut dyn ClassHost) {
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 32 {
                continue;
            }
            let mut c = C2ca098::default();
            if let Some(Payload::Loaded(LoadData::B4(b))) = w.objects.get(o.clone_of.unwrap_or(i)).map(|x| &x.payload) {
                if let B4Load::L258598 { props, kind, .. } = &**b {
                    feed_tpl(&mut c.tpl, &mut c.f_e0, &mut c.bx, props, *kind);
                }
            }
            self.k23c.push((i, Inst::K23C32(Box::new(c))));
        }
    }
    /// slot 14 of classes 32 / 33
    pub(super) fn post_load_k23(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in self.k23c.iter_mut().chain(self.insts.iter_mut()) {
            match x {
                Inst::K23C32(c) => ee_sub_259170(&mut c.tpl, &mut c.f_e0, *i, h),
                Inst::C2ce4d0(c) => ee_sub_259170(&mut c.tpl, &mut c.f_e0, *i, h),
                _ => {}
            }
        }
    }
    pub fn k23_32_get(&self, obj: usize) -> Option<&C2ca098> {
        self.k23c.iter().find(|x| x.0 == obj).and_then(|x| match &x.1 {
            Inst::K23C32(c) => Some(&**c),
            _ => None,
        })
    }
}
