//! WP-K5: the actor core. `alo` = the ALO/SO core slots shared by every class-48 descendant; class 86
//! (C2caf88) and class 98 (C2d4b78) on top. Instances live in `ClassSet::k5`.
pub mod alo;
pub mod c2caf88;
pub mod c2d4260;
pub mod c2d4b78;

use super::*;
use crate::engine::game::fields::object_props;

/// K5 world class id -> vtable (kept in `ClassSet::k5`).
pub const PORTED_K5: &[(ClassId, u32)] = &[(86, 0x2caf88), (98, 0x2d4b78)];

impl ClassSet {
    /// +0x148 bit 35 of a K5 object (owner lookups of slot 6)
    fn bit35_k5(&self, obj: usize) -> Option<bool> {
        self.k5.iter().find(|(i, _)| *i == obj).and_then(|(_, x)| match x {
            Inst::C2caf88(c) => Some(c.alo.so.bit35()),
            Inst::C2d4b78(c) => Some(c.alo.so.bit35()),
            _ => None,
        }).or_else(|| self.k21c.iter().find(|(i, _)| *i == obj).and_then(|(_, x)| match x {
            Inst::C2d4260(c) => Some(c.alo.so.bit35()),
            _ => None,
        }))
    }
    /// K5: construct + init (slot 6) + load / feed the `PORTED_K5` classes. Owners are initialised first
    /// (objects are in stream order, owners precede their children).
    pub(super) fn build_k5(&mut self, w: &World, h: &mut dyn ClassHost) {
        for (i, o) in w.objects.iter().enumerate() {
            if !matches!(o.class, 86 | 98) {
                continue;
            }
            let src = o.clone_of.unwrap_or(i);
            let props = w.objects.get(src).and_then(object_props).unwrap_or(&[]);
            let owner_bit = o.owner.map(|ow| self.bit35_of(ow).or_else(|| self.bit35_k5(ow)).unwrap_or_else(|| h.owner_bit35(ow)));
            let g = self.g.g_2c3b90;
            let inst = if o.class == 86 {
                let mut c = Box::<c2caf88::C2caf88>::default();
                c.alo.so.owner = o.owner;
                c.ee_sub_1320c8(i, owner_bit, g, h);
                c.feed(props);
                Inst::C2caf88(c)
            } else {
                let mut c = Box::<c2d4b78::C2d4b78>::default();
                c.alo.so.owner = o.owner;
                c.oid = o.oid;
                c.ee_sub_1bab50(i, owner_bit, g, h);
                c.ee_sub_1baba0(i, props, h);
                Inst::C2d4b78(c)
            };
            self.k5.push((i, inst));
        }
    }
    /// K5 load-post: slot 14, then slot 22 (add to world) right after, as K3/K4.
    pub(super) fn post_load_k5(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &mut self.k5 {
            match x {
                Inst::C2caf88(c) => {
                    c.ee_sub_132100(*i, h);
                    h.base(0x1f3ea8, *i); // core slot 22
                }
                Inst::C2d4b78(c) => {
                    h.base(0x1f36a8, *i); // core slot 14
                    c.ee_sub_1badb8(*i, &mut self.g.l_3e1450, h);
                }
                _ => {}
            }
        }
    }
    /// K5 per frame: core slot 15 (update + fade), slot 16 (UpdateAlo, E5), slot 18 (draw); class 98 slot 57
    /// (contact latch) after them (assumption: the physics contact pass runs after the update).
    ///
    /// WP-ARCH1 pass 5 (class 86 flip): class 86 (C2caf88) is now store-driven. The LiveQ store
    /// runs its slot-15 ObjectUpdate (vtable 0x2caf88 target is a queue port in
    /// `classes/q/c2caf88.rs`), so the shared ALO/SO slot targets execute on the guest image. The
    /// hand per-frame tick is skipped for class 86 instances; `build_k5` + `post_load_k5` still
    /// seed `Inst::C2caf88` state (filters, bit 34, +0x140) because downstream renderers and
    /// K5 tests read those initial values. Class 98 keeps running through the hand path
    /// (slot 57 contact latch is class-98-specific and not yet replayed through the store).
    pub(super) fn frame_k5(&mut self, h: &mut dyn ClassHost) {
        let mut peers: Vec<(u32, u32)> = Vec::new();
        let lists: Vec<(u32, Vec<(i32, u32)>)> = self
            .k5
            .iter()
            .filter_map(|(i, x)| match x {
                Inst::C2d4b78(c) => Some((*i as u32 + 1, c.f_154.clone())),
                _ => None,
            })
            .collect();
        let peer = |o: u32| lists.iter().find(|x| x.0 == o).map(|x| x.1.clone());
        for (i, x) in &mut self.k5 {
            // ARCH1: class 86 (Inst::C2caf88) is driven by the LiveQ store, not here.
            let Inst::C2d4b78(c) = x else { continue };
            let alo = &mut c.alo;
            alo.ee_sub_1f36e0(*i, self.g.dt, h);
            h.base(0x1f3980, *i);
            alo.so.ee_sub_1f3c70(*i, self.g.g_2ddf54, h);
            for p in c.ee_sub_1bb250(*i, &peer, h) {
                peers.push((p, *i as u32 + 1));
            }
        }
        // FUN_001baed8(peer, obj, 0) on the touched class-98 peers
        for (p, me) in peers {
            if let Some((i, Inst::C2d4b78(c))) = self.k5.iter_mut().find(|x| x.0 as u32 + 1 == p) {
                c.ee_sub_1baed8(*i, me, 0, h);
            }
        }
    }
}
