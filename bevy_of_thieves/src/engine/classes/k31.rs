//! WP-K31: class 37 C2cb9f8 (vtable 0x2cb9f8, size 0x1a0, child of the class-35 interaction point): the guard
//! loot holder of the pickpocket game (player/loot.rs `Guard`). Own slots 5 / 6 / 14 / 22 (+ slot 21
//! FUN_00183ff0 shared with 35 / 36, classes/k10.rs); slot 50 FUN_0013d020 is `LootSys::ee_sub_13d020`.
use super::k10::{args, handle, scalar, C2d1ee8};
use super::k4::{f32_of, word};
use super::*;
use crate::engine::game::fields::object_props;

/// K31 world class id -> vtable (kept in `ClassSet::k31`).
pub const PORTED_K31: &[(ClassId, u32)] = &[(37, 0x2cb9f8)];

/// slot 5 FUN_00113c60: class descriptor 0x2badcc
pub const SLOT5_37: u32 = 0x2badcc;
/// DAT_002a4bb0: slot 22 FUN_00149ab8 member function
pub const PMF_2A4BB0: u32 = 0x2a4bb0;

/// C2cb9f8 (class 37): +0x180 guard index (DAT_003e1acc counter), +0x184 default loot id (field 1167),
/// +0x188 kind (1171), +0x18c (1173), +0x190 alert (1169), +0x194 loot (1170, object).
#[derive(Clone, Debug)]
pub struct C2cb9f8 {
    pub base: C2d1ee8,
    pub f_180: i32,
    pub f_184: i32,
    pub f_188: i32,
    pub f_18c: f32,
    pub f_190: i32,
    pub f_194: u32,
    pub oid_194: Option<i32>,
    /// method 1168 FUN_0013d328(this, id) arguments (guard loot save; run by the loot system)
    pub m_1168: Vec<i32>,
}
impl Default for C2cb9f8 {
    fn default() -> Self {
        C2cb9f8 { base: C2d1ee8::default(), f_180: 0, f_184: 0, f_188: 0, f_18c: 0.0, f_190: 0, f_194: 0, oid_194: None, m_1168: Vec::new() }
    }
}
impl C2cb9f8 {
    /// slot 6 FUN_0013ceb8: FUN_00183f08, +0x184 = 700, +0x18c = 15.0, +0x188 = -1
    pub fn ee_sub_13ceb8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        self.base.ee_sub_183f08(obj, h);
        self.f_184 = 700;
        self.f_18c = 15.0;
        self.f_188 = -1;
    }
    pub fn feed(&mut self, props: &[Prop]) {
        self.base.feed(props);
        for p in props {
            match p.field {
                1167 => self.f_184 = scalar(p).map(|b| word(b) as i32).unwrap_or(self.f_184),
                1168 => self.m_1168.extend(args(p).into_iter().map(|a| a as i32)),
                1169 => self.f_190 = scalar(p).map(|b| word(b) as i32).unwrap_or(self.f_190),
                1170 => self.oid_194 = scalar(p).map(|b| word(b) as i32),
                1171 => self.f_188 = scalar(p).map(|b| word(b) as i32).unwrap_or(self.f_188),
                1173 => self.f_18c = scalar(p).map(f32_of).unwrap_or(self.f_18c),
                _ => {}
            }
        }
    }
    /// load-post oid resolution (+0x194 loot object)
    pub fn resolve(&mut self, obj: usize, h: &mut dyn ClassHost) {
        self.base.resolve(obj, h);
        if let Some(o) = self.oid_194.take() {
            self.f_194 = h.ee_sub_15e938(4, o, handle(obj));
        }
    }
    /// slot 14 FUN_0013cef8: FUN_00183f50, +0x180 = DAT_003e1acc++
    pub fn ee_sub_13cef8(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        self.base.ee_sub_183f50(obj, h);
        self.f_180 = g.g_3e1acc;
        g.g_3e1acc += 1;
    }
    /// slot 21 FUN_00183ff0 (inherited from class 35): leave DAT_003e1414
    pub fn ee_sub_183ff0(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        self.base.ee_sub_183ff0(obj, g, h);
    }
    /// slot 22 FUN_0013cfd8: FUN_00183fb8, FUN_00149ab8(0x3dd490, this, DAT_002a4bb0, 0)
    pub fn ee_sub_13cfd8(&mut self, obj: usize, g: &mut ClassGlobals, h: &mut dyn ClassHost) {
        self.base.ee_sub_183fb8(obj, g, h);
        h.ee_sub_149ab8(obj, PMF_2A4BB0, 0);
    }
}

impl ClassSet {
    /// K31: construct + init (slot 6) + feed the class-37 objects.
    pub(super) fn build_k31(&mut self, w: &World, h: &mut dyn ClassHost) {
        for (i, o) in w.objects.iter().enumerate() {
            if !PORTED_K31.iter().any(|x| x.0 == o.class) {
                continue;
            }
            let props = w.objects.get(o.clone_of.unwrap_or(i)).and_then(object_props).unwrap_or(&[]);
            let props = &*super::props_ov(w, i, props);
            let mut c = C2cb9f8::default();
            c.ee_sub_13ceb8(i, h);
            c.feed(props);
            self.k31.push((i, c));
        }
    }
    /// K31 load-post: oid resolution, slot 14, slot 22 (list entry + deferred call).
    pub(super) fn post_load_k31(&mut self, h: &mut dyn ClassHost) {
        for (i, c) in &mut self.k31 {
            c.resolve(*i, h);
            c.ee_sub_13cef8(*i, &mut self.g, h);
            c.ee_sub_13cfd8(*i, &mut self.g, h);
        }
    }
    /// K31: slot 21 of a class-37 object (level unload / object removal).
    pub fn unload_k31(&mut self, obj: usize, h: &mut dyn ClassHost) {
        if let Some(k) = self.k31.iter().position(|x| x.0 == obj) {
            let (i, mut c) = self.k31.remove(k);
            c.ee_sub_183ff0(i, &mut self.g, h);
        }
    }
}
