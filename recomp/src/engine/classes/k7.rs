//! WP-K7: the next most-instanced unported classes after the guards (class_port.md ranking): six slot-5-only
//! classes (2, 22, 24, 41, 42, 100), class 23 (vision blocker: slot 5 + slots 21/22 in guard::los::Blockers),
//! and the small-init classes 5, 18, 112, 128 and the object-group highlighter 45 (`ClassSet::k7`).
use super::k4::{f32_of, vec3_of, word};
use super::*;
use crate::engine::game::fields::object_props;
use crate::engine::world::props::{ArrayItem, Prop, PropValue};

/// K7 world class id -> vtable (kept in `ClassSet::k7`).
pub const PORTED_K7: &[(ClassId, u32)] = &[(5, 0x2dc070), (18, 0x2db518), (45, 0x2d4f80), (112, 0x2d7f40), (128, 0x2daa50)];

/// K7 classes whose only own overrides are slot 5 (descriptor getter) or state held elsewhere (23): no `Inst`.
pub const PORTED_K7_STATELESS: &[(ClassId, u32)] = &[(2, 0x2d3540), (22, 0x2dc3b0), (23, 0x2dc210), (24, 0x2cc7b0), (41, 0x2dc890), (42, 0x2dc550), (100, 0x2dbb10)];

/// slot 5 (class descriptor getter) of every K7 class: FUN_00113a30 (2), 113a60 (5), 113b30 (18), 113b70 (22),
/// 113b80 (23), 113b90 (24), 113ca0 (41), 113cb0 (42), 113ce0 (45), 114050 (100), 114110 (112), 114210 (128)
pub fn slot5(class: ClassId) -> Option<u32> {
    Some(match class {
        2 => 0x2bab10,
        5 => 0x2bab4c,
        18 => 0x2bac50,
        22 => 0x2baca0,
        23 => 0x2bacb4,
        24 => 0x2bacc8,
        41 => 0x2bae1c,
        42 => 0x2bae30,
        45 => 0x2bae6c,
        100 => 0x2bb2b8,
        112 => 0x2bb3a8,
        128 => 0x2bb4e8,
        _ => return None,
    })
}

fn scalar(p: &Prop) -> Option<&[u8]> {
    match &p.value {
        PropValue::Scalar(b) => Some(b),
        _ => None,
    }
}
/// every oid word of an object / object-list property
fn oids(p: &Prop) -> Vec<i32> {
    let w = |b: &Vec<u8>| b.chunks_exact(4).map(|x| i32::from_le_bytes([x[0], x[1], x[2], x[3]])).collect::<Vec<_>>();
    match &p.value {
        PropValue::Scalar(b) => w(b),
        PropValue::Array(items) => items
            .iter()
            .flat_map(|i| match i {
                ArrayItem::One(_, b) => w(b),
                ArrayItem::List(_, l) => l.iter().flat_map(w).collect(),
            })
            .collect(),
    }
}

/// C2dc070 (class 5, ALO child; load FUN_00227380 already in world): slot 6 FUN_00227350.
#[derive(Clone, Debug, Default)]
pub struct C2dc070 {
    /// +0xe4 (field 175), +0x110
    pub f_e4: i32,
    pub f_110: i32,
}
impl C2dc070 {
    /// slot 6 FUN_00227350: ALO init FUN_00231100, +0x110 = -1
    pub fn ee_sub_227350(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x231100, obj);
        self.f_110 = -1;
    }
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            if let (175, Some(b)) = (p.field, scalar(p)) {
                self.f_e4 = word(b) as i32;
            }
        }
    }
}

/// C2db518 (class 18, ALO child): slot 6 FUN_0021d278.
#[derive(Clone, Debug, Default)]
pub struct C2db518 {
    /// +0xe8 (field 215), +0xec (216), +0xf0 (217), +0xf4 (218)
    pub f_e8: i32,
    pub f_ec: f32,
    pub f_f0: bool,
    pub f_f4: i32,
}
impl C2db518 {
    /// slot 6 FUN_0021d278: FUN_00231100, FUN_001a6ca8(obj, 3), +0xe8 = 0x17e, +0xf4 = -1, +0xec = 25.0
    pub fn ee_sub_21d278(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x231100, obj);
        h.ee_sub_1a6ca8(obj, 3);
        self.f_e8 = 0x17e;
        self.f_f4 = -1;
        self.f_ec = 25.0;
    }
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            let Some(b) = scalar(p) else { continue };
            match p.field {
                215 => self.f_e8 = word(b) as i32,
                216 => self.f_ec = f32_of(b),
                217 => self.f_f0 = word(b) != 0,
                218 => self.f_f4 = word(b) as i32,
                _ => {}
            }
        }
    }
}

/// C2daa50 (class 128, child of the UI screen class 126): slot 6 FUN_00211e40.
#[derive(Clone, Debug, Default)]
pub struct C2daa50 {
    pub f_b0: i32,
}
impl C2daa50 {
    /// slot 6 FUN_00211e40: FUN_001a5270, +0xb0 = 1
    pub fn ee_sub_211e40(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5270, obj);
        self.f_b0 = 1;
    }
}

/// C2d7f40 (class 112, LO child): slot 6 FUN_001d6a90 defaults; fields 460-467.
#[derive(Clone, Debug, Default)]
pub struct C2d7f40 {
    /// +0x60 object (field 460, oid until resolved), +0x50 vector (461)
    pub f_60: i32,
    pub f_50: [f32; 4],
    /// +0x70 / +0x90 clq (462 / 464), +0x80 / +0xa0 / +0xa8 lm (463 / 465 / 466), +0xb0 (467)
    pub f_70: [f32; 4],
    pub f_80: [f32; 2],
    pub f_90: [f32; 4],
    pub f_a0: [f32; 2],
    pub f_a8: [f32; 2],
    pub f_b0: f32,
}
impl C2d7f40 {
    /// slot 6 FUN_001d6a90: FUN_001a5270, clq +0x70 = 0, lm +0x80 = (0, FLT_MAX DAT_002a7874), clq +0x90 =
    /// (0, 1, 0), lm +0xa0 = (0, FLT_MAX), +0xb0 = 0.5
    pub fn ee_sub_1d6a90(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5270, obj);
        self.f_70[..3].copy_from_slice(&[0.0; 3]);
        self.f_80 = [0.0, f32::MAX];
        self.f_90[..3].copy_from_slice(&[0.0, 1.0, 0.0]);
        self.f_b0 = 0.5;
        self.f_a0 = [0.0, f32::MAX];
    }
    pub fn feed(&mut self, props: &[Prop]) {
        let f4 = |b: &[u8]| [0, 1, 2, 3].map(|i| b.get(i * 4..i * 4 + 4).map(f32_of).unwrap_or(0.0));
        for p in props {
            let Some(b) = scalar(p) else { continue };
            match p.field {
                460 => self.f_60 = word(b) as i32,
                461 => self.f_50 = vec3_of(b),
                462 => self.f_70 = f4(b),
                463 => self.f_80 = [f4(b)[0], f4(b)[1]],
                464 => self.f_90 = f4(b),
                465 => self.f_a0 = [f4(b)[0], f4(b)[1]],
                466 => self.f_a8 = [f4(b)[0], f4(b)[1]],
                467 => self.f_b0 = f32_of(b),
                _ => {}
            }
        }
    }
}

/// C2d4f80 (class 45, ALO child): object-group highlighter. Activation takes one of the two global slots
/// DAT_003e1a78 (stride 0x28, +0x20 owner) and shows / highlights the +0xe4 group.
#[derive(Clone, Debug, Default)]
pub struct C2d4f80 {
    /// +0x118 object (field 1063), +0x11c / +0x120 / +0x124 (1060-1062)
    pub f_118: i32,
    pub f_11c: f32,
    pub f_120: f32,
    pub f_124: f32,
    /// +0xe0 count / +0xe4 group objects (field 1064 setter FUN_001beaa8), +0xf4 per-entry flags
    /// (1 = added to the world by the activation, 2 = highlight off -> on)
    pub f_e4: Vec<u32>,
    pub f_f4: Vec<u32>,
    /// +0x104 count / +0x108 objects (field 1065 setter FUN_001bead0)
    pub f_108: Vec<u32>,
    /// +0x128 global slot (index into ClassGlobals::g_3e1a78) while active
    pub f_128: Option<usize>,
    /// unresolved oids of fields 1064 / 1065 (resolved at load-post, FUN_0015e938)
    pub oids_e4: Vec<i32>,
    pub oids_108: Vec<i32>,
}
impl C2d4f80 {
    /// slot 6 FUN_001bea40: FUN_00231100, +0x11c = 1.0, +0x124 = 1000.0, +0x120 = 4.0
    pub fn ee_sub_1bea40(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x231100, obj);
        self.f_11c = 1.0;
        self.f_124 = 1000.0;
        self.f_120 = 4.0;
    }
    pub fn feed(&mut self, props: &[Prop]) {
        for p in props {
            match p.field {
                1060 => self.f_11c = scalar(p).map(f32_of).unwrap_or(self.f_11c),
                1061 => self.f_120 = scalar(p).map(f32_of).unwrap_or(self.f_120),
                1062 => self.f_124 = scalar(p).map(f32_of).unwrap_or(self.f_124),
                1063 => self.f_118 = scalar(p).map(|b| word(b) as i32).unwrap_or(0),
                1064 => self.oids_e4.extend(oids(p)),
                1065 => self.oids_108.extend(oids(p)),
                _ => {}
            }
        }
    }
    /// field 1064 setter FUN_001beaa8(g, o): append a non-null object to +0xe4
    pub fn ee_sub_1beaa8(&mut self, o: u32) {
        if o != 0 {
            self.f_e4.push(o);
            self.f_f4.push(0);
        }
    }
    /// field 1065 setter FUN_001bead0(g, o): append to +0x108, o+0x15c = o+0x160 = 1, o vt+0x4c (remove)
    pub fn ee_sub_1bead0(&mut self, o: u32, h: &mut dyn ClassHost) {
        if o != 0 {
            self.f_108.push(o);
            // +0x15c = 1, +0x160 = 1 (part visibility/active flags)
            h.obj_set_u32(o, 0x15c, 1);
            h.obj_set_u32(o, 0x160, 1);
            h.vcall(o, 0x4c);
        }
    }
    /// load-post: resolve the load-time setter arguments (FUN_0015e938 kind 4)
    pub fn resolve(&mut self, obj: usize, h: &mut dyn ClassHost) {
        for oid in std::mem::take(&mut self.oids_e4) {
            let o = h.ee_sub_15e938(4, oid, obj as u32 + 1);
            self.ee_sub_1beaa8(o);
        }
        for oid in std::mem::take(&mut self.oids_108) {
            let o = h.ee_sub_15e938(4, oid, obj as u32 + 1);
            self.ee_sub_1bead0(o, h);
        }
    }

    /// field 1066 method FUN_001becf8 -> FUN_001beb28(g): take a free global slot (+0x20 owner = g); every
    /// group member not in the world (+0x28 bit 60) is added (vt+0x44, FUN_001a61a0 / FUN_0011a220 0x3dd4bc +
    /// root vt+0xbc as hooks) and flagged 1; members with +0x40 bit 3 get their highlight on (vt+0xac(0)),
    /// flag 2; then g vt+0xac(1), g vt+0x1c(2). Returns the slot (None: both busy / already active).
    pub fn ee_sub_1beb28(&mut self, obj: usize, slots: &mut [u32; 2], h: &mut dyn ClassHost) -> Option<usize> {
        if self.f_128.is_some() {
            return self.f_128;
        }
        let k = slots.iter().position(|&o| o == 0)?;
        self.f_128 = Some(k);
        slots[k] = obj as u32 + 1;
        for (o, fl) in self.f_e4.iter().zip(self.f_f4.iter_mut()) {
            *fl = 0;
            if h.obj_i32(*o, 0x2c) >> 28 & 1 == 0 {
                *fl = 1;
                h.vcall(*o, 0x44);
                if !h.ee_sub_1a61a0(*o) {
                    h.base(0x11a220, *o as usize - 1);
                }
            }
            if h.obj_i32(*o, 0x40) & 8 != 0 {
                *fl |= 2;
                h.vcall_arg(*o, 0xac, 0);
            }
        }
        h.vcall_arg(obj as u32 + 1, 0xac, 1);
        h.vcall_1c(obj, 2, 0);
        self.f_128
    }

    /// field 1067 method FUN_001bed18(g): release the slot, undo the activation flags (1 -> vt+0x4c remove,
    /// 2 -> vt+0xac(1)), g vt+0xac(0), g vt+0x1c(3)
    pub fn ee_sub_1bed18(&mut self, obj: usize, slots: &mut [u32; 2], h: &mut dyn ClassHost) {
        let Some(k) = self.f_128.take() else { return };
        slots[k] = 0;
        for (o, fl) in self.f_e4.iter().zip(self.f_f4.iter()) {
            if fl & 1 != 0 {
                h.vcall(*o, 0x4c);
            }
            if fl & 2 != 0 {
                h.vcall_arg(*o, 0xac, 1);
            }
        }
        h.vcall_arg(obj as u32 + 1, 0xac, 0);
        h.vcall_1c(obj, 3, 0);
    }

    /// slot 18 FUN_001bee60: while active, FUN_00169198(member + 0x80, g) for every group member (highlight
    /// draw, hook)
    pub fn ee_sub_1bee60(&self, obj: usize, h: &mut dyn ClassHost) {
        if self.f_128.is_none() {
            return;
        }
        for &o in &self.f_e4 {
            h.ee_sub_169198(o, obj);
        }
    }
}

/// One K7 instance.
#[derive(Clone, Debug)]
pub enum K7 {
    C2dc070(C2dc070),
    C2db518(C2db518),
    C2d4f80(Box<C2d4f80>),
    C2d7f40(C2d7f40),
    C2daa50(C2daa50),
}

impl ClassSet {
    /// K7: construct + init (slot 6) + feed the `PORTED_K7` classes.
    pub(super) fn build_k7(&mut self, w: &World, h: &mut dyn ClassHost) {
        for (i, o) in w.objects.iter().enumerate() {
            if !PORTED_K7.iter().any(|x| x.0 == o.class) {
                continue;
            }
            let props = w.objects.get(o.clone_of.unwrap_or(i)).and_then(object_props).unwrap_or(&[]);
            let k = match o.class {
                5 => {
                    let mut c = C2dc070::default();
                    c.ee_sub_227350(i, h);
                    c.feed(props);
                    K7::C2dc070(c)
                }
                18 => {
                    let mut c = C2db518::default();
                    c.ee_sub_21d278(i, h);
                    c.feed(props);
                    K7::C2db518(c)
                }
                45 => {
                    let mut c = Box::<C2d4f80>::default();
                    c.ee_sub_1bea40(i, h);
                    c.feed(props);
                    K7::C2d4f80(c)
                }
                112 => {
                    let mut c = C2d7f40::default();
                    c.ee_sub_1d6a90(i, h);
                    c.feed(props);
                    K7::C2d7f40(c)
                }
                _ => {
                    let mut c = C2daa50::default();
                    c.ee_sub_211e40(i, h);
                    K7::C2daa50(c)
                }
            };
            self.k7.push((i, Inst::K7(k)));
        }
    }
    /// K7 load-post: class 45 setter arguments
    pub(super) fn post_load_k7(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &mut self.k7 {
            if let Inst::K7(K7::C2d4f80(c)) = x {
                c.resolve(*i, h);
            }
        }
    }
    /// K7 per frame: class 45 slot 18
    pub(super) fn frame_k7_classes(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &self.k7 {
            if let Inst::K7(K7::C2d4f80(c)) = x {
                c.ee_sub_1bee60(*i, h);
            }
        }
    }
    /// class-45 methods 1066 (activate, FUN_001beb28) / 1067 (deactivate, FUN_001bed18) on object `obj`
    pub fn k7_highlight(&mut self, obj: usize, on: bool, h: &mut dyn ClassHost) -> Option<usize> {
        let slots = &mut self.g.g_3e1a78;
        let c = self.k7.iter_mut().find_map(|(i, x)| match x {
            Inst::K7(K7::C2d4f80(c)) if *i == obj => Some(c),
            _ => None,
        })?;
        if on {
            c.ee_sub_1beb28(obj, slots, h)
        } else {
            c.ee_sub_1bed18(obj, slots, h);
            None
        }
    }
}
