//! WP-Q2: what of Sly's subtree is drawn, 1:1 with the JT post-load bind tables and SetJtJtbs.
//!
//! - FUN_001a64e8(obj, n, table) rows {flags, oid, member}: flags without 0x10/0x20/0x40 look the oid
//!   up with FUN_0015e938(0x3dd490, mode, oid, obj) (mode 4 if flags & 2, else 5 if flags & 1, else 1;
//!   | 0x400 with flags & 0x400) and with flags & 0x200 remove it (vtable +0x4c = FUN_001a5630: unlink,
//!   LO +0x28 bit 56, recursive over the children). 0x40 = mesh record by oid (FUN_001f5ff8), 0x200 hides it
//!   (FUN_001f69f8). 0x20 = collision part (+0xe4, 0xb0 stride, FUN_001f6030; 0x200 -> FUN_001f2ad0): not
//!   drawn. 0x10 = bone by +0x9c.
//!   JT order: FUN_001885b8 (0x2be990, 0xad rows), then post-load FUN_00188608 -> FUN_001400b8
//!   (0x2bc9f0, 0x35 rows; then FUN_0015e650(0x3dd490, 0x401, 0x2ab, jt) removes every oid 0x2ab) and
//!   0x2bf1b0 (0x12 rows).
//! - Property field 26 (0x2acc90): deferred Remove (FUN_00100b10 -> FUN_001a5d58, run by the ALO post-load
//!   FUN_001f36a8 -> FUN_001a5d98): the dome / mannequin / gadget props start removed.
//! - Mesh records: runtime record +0xc bit 0x800 = record u1 & 1 at load (FUN_00167320; instances
//!   FUN_00169ac8 their own u1); drawn only while clear (FUN_00168638 / LOD slot bit 50 in FUN_001d5808).
//!   Show/hide by oid: FUN_001f6978 / FUN_001f69f8 (every record with that word).
//! - FUN_0018e540 SetJtJtbs (JT +0x18d8): entering 2 or 0xb from another value adds JT +0x19a8 (oid 0x13e,
//!   vtable +0x44 FUN_001a5478, recursive) and removes +0x19a0 (oid 0x13c); leaving does the reverse.
//!   (vtable +0x134 hand-over between the two is not ported.)

use super::{ee_sub_15e938_below, subtree, JT_TABLE};
use crate::engine::gfx::attach::alo_of;
use crate::engine::world::World;
use std::collections::HashSet;

pub static JT_BIND_2BC9F0: std::sync::LazyLock<Vec<(u32, i32, u32)>> = std::sync::LazyLock::new(|| crate::engine::disc::exe::rows3(0x002b_c9f0, 53));
/// FUN_001a64e8 table of Bentley's slot 13 FUN_00134770 (DAT_002bc390, 0x76 rows).
pub static PO_BIND_2BC390: std::sync::LazyLock<Vec<(u32, i32, u32)>> = std::sync::LazyLock::new(|| crate::engine::disc::exe::rows3(0x002b_c390, 0x76));
/// FUN_001a64e8 table of Murray's slot 13 (DAT_002c0270, 0x89 rows).
pub static PO_BIND_2C0270: std::sync::LazyLock<Vec<(u32, i32, u32)>> = std::sync::LazyLock::new(|| crate::engine::disc::exe::rows3(0x002c_0270, 0x89));
pub static JT_BIND_2BF1B0: std::sync::LazyLock<Vec<(u32, i32, u32)>> = std::sync::LazyLock::new(|| crate::engine::disc::exe::rows3(0x002b_f1b0, 18));

/// FUN_0015e650(0x3dd490, 0x401, 0x2ab, jt, 0x10, ..) in FUN_001400b8: removed oid.
pub const JT_REMOVE_ALL_OID: i32 = 0x2ab;
/// Property field 26: deferred Remove at post-load (objects start removed until vtable +0x44 Add).
pub const FIELD_REMOVE: i16 = 26;
/// Property fields 44 / 45 (A6): method thunks 0x100d18 / 0x100da0 = FUN_001f6978 show / FUN_001f69f8 hide
/// with the row's s32 mesh oid, run in stream order when the object's property stream is read (load).
pub const FIELD_SHOW_MESH: i16 = 44;
pub const FIELD_HIDE_MESH: i16 = 45;

/// The (mesh oid, show) rows of fields 44 / 45 of `o`'s property stream, in stream order.
pub fn stream_mesh_rows(w: &World, o: usize) -> Vec<(i32, bool)> {
    use crate::engine::world::props::{ArrayItem, PropValue};
    let Some(props) = crate::engine::game::fields::object_props(&w.objects[o]) else { return vec![] };
    let s32 = |b: &[u8]| b.get(..4).map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]]));
    props
        .iter()
        .filter(|p| p.field == FIELD_SHOW_MESH || p.field == FIELD_HIDE_MESH)
        .filter_map(|p| {
            let oid = match &p.value {
                PropValue::Array(items) => match items.first() {
                    Some(ArrayItem::One(_, b)) => s32(b),
                    _ => None,
                },
                PropValue::Scalar(b) => s32(b),
            }?;
            Some((oid, p.field == FIELD_SHOW_MESH))
        })
        .collect()
}

/// FUN_0018e540: JT +0x19a0 / +0x19a8 objects (0x2bf1b0 rows 0 and 2).
pub const OID_19A0: i32 = 0x13c;
pub const OID_19A8: i32 = 0x13e;
/// 0x2bf1b0 row 1: JT +0x19a4.
pub const OID_19A4: i32 = 0x13d;

/// Removed objects (LO +0x28 bit 56) and hidden mesh records (+0xc 0x800) of the JT subtree.
#[derive(Clone, Debug, Default)]
pub struct JtVis {
    pub jt: usize,
    pub removed: HashSet<usize>,
    /// (object, record, instance)
    pub hidden: HashSet<(usize, usize, Option<usize>)>,
    /// JT +0x18d8 (-1 after load)
    pub carry: i32,
    /// WP-P24: objects switched off by vtable +0xac(0) (FUN_00189530 binocucom hide), and the last hide flag
    pub off_ac: HashSet<usize>,
    pub hide_189530: bool,
}

impl JtVis {
    /// vtable +0x4c FUN_001a5630 (recursive).
    pub fn ee_sub_1a5630(&mut self, w: &World, o: usize) {
        self.removed.extend(subtree(w, o));
    }
    /// vtable +0x44 FUN_001a5478 (recursive).
    pub fn ee_sub_1a5478(&mut self, w: &World, o: usize) {
        for i in subtree(w, o) {
            self.removed.remove(&i);
        }
    }
    /// FUN_001f6978 (show) / FUN_001f69f8 (hide): every record / instance of `o` with word `oid`.
    pub fn set_mesh(&mut self, w: &World, o: usize, oid: i32, show: bool) {
        let Some(a) = alo_of(w, o) else { return };
        for (ri, r) in a.mesh.records.iter().enumerate() {
            let mut keys = vec![];
            if r.word == oid {
                keys.push((o, ri, None));
            }
            keys.extend(r.instances.iter().enumerate().filter(|(_, x)| x.word == oid).map(|(ii, _)| (o, ri, Some(ii))));
            for k in keys {
                if show {
                    self.hidden.remove(&k);
                } else {
                    self.hidden.insert(k);
                }
            }
        }
    }

    /// Fields 44 / 45 of `src`'s property stream applied to the meshes of `o` (clones copy the template's
    /// rows: `src` = template, `o` = the template's mesh owner).
    pub fn ee_stream_mesh(&mut self, w: &World, src: usize, o: usize) {
        for (oid, show) in stream_mesh_rows(w, src) {
            self.set_mesh(w, o, oid, show);
        }
    }

    /// FUN_001a64e8(jt, n, table) drawing effects.
    pub fn ee_sub_1a64e8(&mut self, w: &World, table: &[(u32, i32, u32)]) {
        for &(flags, oid, _) in table {
            if flags & 0x200 == 0 || flags & 0x30 != 0 {
                continue;
            }
            if flags & 0x40 != 0 {
                self.set_mesh(w, self.jt, oid, false);
            } else if flags & 3 == 0 {
                // mode 1 (the only mode the JT tables use)
                if let Some(o) = ee_sub_15e938_below(w, self.jt, oid) {
                    self.ee_sub_1a5630(w, o);
                }
            }
        }
    }

    /// Load state: record bits, then the JT bind tables in post-load order.
    pub fn load(w: &World, jt: usize) -> JtVis {
        let mut v = Self::load_bits(w, jt);
        v.ee_sub_1a64e8(w, &JT_TABLE);
        v.ee_sub_1a64e8(w, &JT_BIND_2BC9F0);
        for o in subtree(w, jt).into_iter().filter(|&o| w.objects[o].oid == JT_REMOVE_ALL_OID) {
            v.ee_sub_1a5630(w, o);
        }
        v.ee_sub_1a64e8(w, &JT_BIND_2BF1B0);
        v
    }

    /// WP-P22: Bentley / Murray (class 55 / 54): slot 13 FUN_001a64e8(self, n, `table`) (PO_BIND_2BC390 /
    /// PO_BIND_2C0270), then slot 14's base FUN_001400b8 (0x2bc9f0 table + FUN_0015e650 removal of oid 0x2ab).
    pub fn load_po(w: &World, po: usize, table: &[(u32, i32, u32)]) -> JtVis {
        let mut v = Self::load_bits(w, po);
        v.ee_sub_1a64e8(w, table);
        v.ee_sub_1a64e8(w, &JT_BIND_2BC9F0);
        for o in subtree(w, po).into_iter().filter(|&o| w.objects[o].oid == JT_REMOVE_ALL_OID) {
            v.ee_sub_1a5630(w, o);
        }
        v
    }

    /// Record bits, property-stream mesh rows and field-26 removals of the subtree of `jt`.
    fn load_bits(w: &World, jt: usize) -> JtVis {
        let mut v = JtVis { jt, carry: -1, ..Default::default() };
        for o in subtree(w, jt) {
            let Some(a) = alo_of(w, o) else { continue };
            for (ri, r) in a.mesh.records.iter().enumerate() {
                if r.u1 & 1 != 0 {
                    v.hidden.insert((o, ri, None));
                }
                for (ii, x) in r.instances.iter().enumerate() {
                    if x.u1 & 1 != 0 {
                        v.hidden.insert((o, ri, Some(ii)));
                    }
                }
            }
            // the show / hide rows of the property stream are read after the mesh records
            v.ee_stream_mesh(w, o, o);
        }
        // property field 26 (descriptor 0x2acc90, FUN_00100b10 -> FUN_001a5d58): deferred Remove run by
        // the ALO post-load (FUN_001f36a8 -> FUN_001a5d98 -> FUN_001a5d08 -> vtable +0x4c)
        for o in subtree(w, jt) {
            if alo_of(w, o).is_some_and(|a| a.props.iter().any(|p| p.field == FIELD_REMOVE)) {
                v.ee_sub_1a5630(w, o);
            }
        }
        v
    }

    /// FUN_0018e540 visibility part (JT +0x18d8 := `n`).
    pub fn ee_sub_18e540(&mut self, w: &World, n: i32) {
        if n == self.carry {
            return;
        }
        let glide = |x: i32| x == 0xb || x == 2;
        if glide(self.carry) != glide(n) {
            let a = ee_sub_15e938_below(w, self.jt, OID_19A0);
            let b = ee_sub_15e938_below(w, self.jt, OID_19A8);
            let (add, rem) = if glide(n) { (b, a) } else { (a, b) };
            if let Some(o) = add {
                self.ee_sub_1a5478(w, o);
            }
            if let Some(o) = rem {
                self.ee_sub_1a5630(w, o);
            }
        }
        self.carry = n;
    }

    /// WP-P24: JT vt+0x2bc FUN_00189530(jt, hide): vtable +0xac(!hide) on JT +0x19a0 / +0x19a4 / +0x19a8
    /// (0x2bf1b0 rows 0..2, oids 0x13c / 0x13d / 0x13e) - Sly's body is not drawn while looking through the
    /// binocucom. +0xac is modelled as a subtree draw switch apart from Remove (`removed`).
    ///
    /// WP-I5 REPLACED: those three (cane) objects are all the original switches off; the rest of the body
    /// is kept out of the look view by the camera itself (C2cd4e0: eye = pos + 150 up, camera 50 behind it,
    /// near clip 49 -> the near plane sits at the head). In the port hat, ears and hand still show
    /// (shots/P24/binoc_f200.png; pose / mesh draw differences), so the whole JT subtree is
    /// switched off with them.
    pub fn ee_sub_189530(&mut self, w: &World, hide: bool) {
        self.hide_189530 = hide;
        for i in subtree(w, self.jt) {
            if hide {
                self.off_ac.insert(i);
            } else {
                self.off_ac.remove(&i);
            }
        }
        for oid in [OID_19A0, OID_19A4, OID_19A8] {
            let Some(o) = ee_sub_15e938_below(w, self.jt, oid) else { continue };
            for i in subtree(w, o) {
                if hide {
                    self.off_ac.insert(i);
                } else {
                    self.off_ac.remove(&i);
                }
            }
        }
    }

    pub fn drawn(&self, o: usize) -> bool {
        !self.removed.contains(&o) && !self.off_ac.contains(&o)
    }
    pub fn record_drawn(&self, o: usize, rec: usize, inst: Option<usize>) -> bool {
        self.drawn(o) && !self.hidden.contains(&(o, rec, inst))
    }
}
