//! WP-A6: the guard's post-load weapon setup, run 1:1 through its level script.
//!
//! Guard classes 57/58/61/62/64..67 (vtables 0x2cf520 0x2c9dc0 0x2d7188 0x2c91c8 0x2cd0a0 0x2c8ef0 0x2c8c18
//! 0x2c8940; db class_slots) share slot 14 FUN_002061d8, which queues, when `*(*(obj + 0x3c) + 0x24) == 0`,
//! the deferred member call FUN_00149a50(0x3dd490, obj, DAT_002a8608, 0); DAT_002a8608 is the PMF of
//! FUN_00206170(obj): vtable +0xd4(obj, 0) (FUN_00204c60: drop the +0x208 sound) then vtable +0x1c =
//! FUN_001a61e0(obj, 0xe, 1, 1, {0}, 0) -> FUN_001331f8: look up symbol 0xe in the object's script frame
//! (FUN_00127db8(*obj, 0xe, 1)) and call that proc with (obj, 0) (`param_3 = 1`: queued through
//! FUN_00133040 for the next world update).
//!
//! The guard entry's script block (lsn 377568 entry 44) defines symbol 14 as
//! `(lambda (self a) (for-each (lambda (x) ((get-o self 111) x)) '(19867 125334 125331 19813))
//!   (for-each (lambda (x) ((get-o self 113) x)) '(19919 19869 9086 19851))
//!   (pick-weapon self (get-o self 1256)) ...)`: get-o 111 (class 57 symid 0x6f = method field 45,
//! thunk 0x100da0 -> FUN_001f69f8) HIDES every weapon mesh (club 0x4d9b, 0x1e996, 0x1e993, 0x4d65), get-o
//! 113 (FUN_001f6b48) stops the effect parts, and `pick-weapon` SHOWS the instance's weapon (get-o 110 =
//! FUN_001f6978) chosen by the user property 1256, which the guard's stream sets through property field 8
//! (method FUN_00132d88 = ensure-o {symid, S32 value}).
//!
//! The port has no per-object script frames or deferred member-call dispatch for classes (classes/,
//! game/ own those); this module runs exactly that call for one guard with a fresh frame holding the
//! guard entry's block, with a host that serves get-o / set-o / ensure-o on the stream's user properties
//! and the mesh show / hide methods (FUN_001f6978 / FUN_001f69f8 -> [`JtVis::set_mesh`]). Other methods
//! the proc calls (attack-record / anim-table setters 0x167 / 0x168 / 0x171, effect parts 0x70 / 0x71)
//! have no drawing effect and are recorded only.

use crate::engine::game::fields::{ObjectFields, SYMID_USER};
use crate::engine::game::methods::find::{ee_sub_15e958, ee_sub_15e9c8};
use crate::engine::splice::value::{list_from, list_items};
use crate::engine::game::methods::{by_thunk, get_o_method};
use crate::engine::playerdraw::vis::JtVis;
use crate::engine::splice::frame::{self, FrameRef};
use crate::engine::splice::serialize::read_script_block;
use crate::engine::splice::value::{Method, Ref};
use crate::engine::splice::{SpliceError, SpliceHost, Vm};
use crate::engine::world::props::{ArrayItem, PropValue};
use crate::engine::world::World;
use std::collections::HashSet;

/// Guard classes whose slot 14 is FUN_002061d8 (sends message 0xe); 59 / 60 / 63 override it.
pub const POST_LOAD_CLASSES: [u16; 8] = [57, 58, 61, 62, 64, 65, 66, 67];
/// FUN_001a5c48's message: script symbol 0.
pub const MSG_LOAD: u32 = 0;
/// FUN_00206170's message: script symbol 0xe.
pub const MSG_POST_LOAD: u32 = 0xe;
/// Property field 8: method row thunk 0x100610 -> FUN_00132d88(obj, symid, value) = ensure-o.
pub const FIELD_ENSURE_O: i16 = 8;
/// FUN_001f6978 (show mesh records by oid word) / FUN_001f69f8 (hide).
pub const FN_SHOW_MESH: u32 = 0x001f_6978;
pub const FN_HIDE_MESH: u32 = 0x001f_69f8;
/// Method thunks 0x00100a90 (field 25, LO Add) / 0x00100b88 (field 26, LO Remove).
pub const METHOD_LO_ADD: u32 = 0x0010_0a90;
pub const METHOD_LO_REMOVE: u32 = 0x0010_0b88;

/// Host of the post-load call: get-o / set-o / ensure-o on [`ObjectFields`] (as `game::vm_state`), the
/// mesh show / hide methods and the object lookup BIFs; other methods are recorded only.
pub struct PostLoadHost<'a> {
    pub w: &'a World,
    /// the guard and the object whose meshes stand for it (its template ALO: clones share its records)
    pub obj: u32,
    pub mesh_obj: usize,
    pub vis: JtVis,
    pub objs: ObjectFields,
    /// BIF 99 (FUN_00210ae0): the mission's level-table slot, -1 without a mission
    pub mission_slot: i32,
    /// (callee, args) of every method without a drawing effect, in call order
    pub calls: Vec<(u32, Vec<Ref>)>,
    /// error that stopped the proc (a BIF this host does not serve)
    pub err: Option<String>,
}

impl PostLoadHost<'_> {
    /// Lookups scoped to the guard search its template's subtree (the drawn parts; clones copy its oids).
    fn scope(&self, obj: u32) -> u32 {
        if obj == self.obj { self.mesh_obj as u32 } else { obj }
    }
    fn class_of(&self, obj: u32) -> Option<u16> {
        self.w.objects.get(obj as usize).map(|o| o.class)
    }
}

fn arg(a: &[Ref], i: usize) -> u32 {
    a.get(i).map(|r| r.payload()).unwrap_or(0)
}

impl SpliceHost for PostLoadHost<'_> {
    fn call_method(&mut self, m: &Method, args: &[Ref]) -> Result<Ref, SpliceError> {
        let callee = by_thunk(m.method_id).map(|d| d.callee).unwrap_or(0);
        let oid = arg(args, 0) as i32;
        let o = if m.object == self.obj { self.mesh_obj } else { m.object as usize };
        match (m.method_id, callee) {
            // LO Add / Remove (field 25 / 26 inline thunks: vtable +0x44 FUN_001a5478 / +0x4c FUN_001a5630)
            (METHOD_LO_ADD, _) => self.vis.ee_sub_1a5478(self.w, o),
            (METHOD_LO_REMOVE, _) => self.vis.ee_sub_1a5630(self.w, o),
            (_, FN_SHOW_MESH) => self.vis.set_mesh(self.w, o, oid, true),
            (_, FN_HIDE_MESH) => self.vis.set_mesh(self.w, o, oid, false),
            (_, c) => self.calls.push((c, args.to_vec())),
        }
        Ok(Ref::Void)
    }
    fn ref_op_get_o(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00124048 -> FUN_00200ba0: methods (symid < 0x4d1), else fields / the user property list
        let (obj, sym) = (arg(a, 0), arg(a, 1));
        let class = self.class_of(obj);
        if sym < SYMID_USER {
            if let Some(r) = class.and_then(|c| get_o_method(obj, c, sym as u16)) {
                return Ok(r);
            }
        }
        Ok(self.objs.ee_sub_200ba0(Some(self.w), obj, class, sym))
    }
    fn ref_op_set_o(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00124018 -> FUN_00200c28
        let (obj, sym) = (arg(a, 0), arg(a, 1));
        let class = self.class_of(obj);
        Ok(self.objs.ee_sub_200c28(obj, class, sym, a.get(2).cloned().unwrap_or(Ref::Nil)))
    }
    fn ref_op_ensure_o(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00123fe8 -> FUN_00200d10
        Ok(self.objs.ee_sub_200d10(arg(a, 0), arg(a, 1), a.get(2).cloned().unwrap_or(Ref::Nil)))
    }
    fn bif99_splice_bif_99(&mut self, _: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00123a98: S32 FUN_00210ae0()
        Ok(Ref::S32(self.mission_slot))
    }
    fn bif112_splice_bif_112(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00124150: FUN_0015e958(world, obj ? 1 : 5, oid, obj)
        let obj = self.scope(arg(a, 1));
        Ok(Ref::Basic(ee_sub_15e958(self.w, if obj != 0 { 1 } else { 5 }, arg(a, 0) as i32, obj)))
    }
    fn bif115_splice_bif_115(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x001244e8: FUN_0015e958(world, 4, oid, obj)
        Ok(Ref::Basic(ee_sub_15e958(self.w, 4, arg(a, 0) as i32, self.scope(arg(a, 1)))))
    }
    fn bif114_splice_bif_114(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x001242b8: each oid of the list argv[0] -> FUN_0015e958(world, obj ? 1 : 5, oid, obj = argv[1]+4)
        let Some(l) = a.first().filter(|r| r.pair().is_some()) else { return Ok(a.first().cloned().unwrap_or(Ref::Nil)) };
        let obj = self.scope(arg(a, 1));
        let mode = if obj != 0 { 1 } else { 5 };
        let v = list_items(l.pair()).into_iter().map(|r| Ref::Basic(ee_sub_15e958(self.w, mode, r.payload() as i32, obj))).collect();
        Ok(list_from(v))
    }
    fn ref_op_add_o(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00123fb8 -> FUN_00200ca8
        Ok(self.objs.ee_sub_200ca8(arg(a, 0), arg(a, 1), a.get(2).cloned().unwrap_or(Ref::Nil)))
    }
    fn ref_op_find_class_objects(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00124730: FUN_0015e9c8(world, obj ? 0x401 : 0x405, cid, obj, 0x180) -> list
        let obj = self.scope(arg(a, 1));
        let v = ee_sub_15e9c8(self.w, if obj != 0 { 0x401 } else { 0x405 }, arg(a, 0) as i32, obj, 0x180);
        Ok(list_from(v.into_iter().map(Ref::Basic).collect()))
    }
}

/// Field-8 rows of an object's stream (FUN_00132d88 ensure-o {symid (kind 0x15), S32 (kind 6)}).
pub fn stream_user_props(w: &World, obj: usize) -> Vec<(u32, i32)> {
    let Some(props) = crate::engine::game::fields::object_props(&w.objects[obj]) else { return vec![] };
    let mut out = vec![];
    for p in props.iter().filter(|p| p.field == FIELD_ENSURE_O) {
        let PropValue::Array(items) = &p.value else { continue };
        let (Some(ArrayItem::One(0x15, s)), Some(ArrayItem::One(6, v))) = (items.first(), items.get(1)) else { continue };
        let sym = s.get(..2).map(|b| u16::from_le_bytes([b[0], b[1]]) as u32).unwrap_or(0);
        let val = v.get(..4).map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]])).unwrap_or(0);
        out.push((sym, val));
    }
    out
}

/// FUN_00206170 for guard `obj` (template = `tpl`, the clone source holding the stream and meshes):
/// returns the mesh visibility after the script's message 0xe (records' load bits first, as JtVis::load
/// minus the JT tables), or None when the guard's entry has no symbol-0xe proc.
pub fn ee_sub_206170<'a>(w: &'a World, level: &[u8], obj: usize, tpl: usize, removed: &HashSet<usize>, mission_slot: i32) -> Option<PostLoadHost<'a>> {
    // 0x00206170 (vtable +0xd4 FUN_00204c60 only clears the +0x208 sound: no drawing effect)
    // the object's frame (LO +0): FUN_001ffa40(world, parent_slot) = the fresh frame FUN_001ffa70 made for
    // expression `parent_slot` of its entry's FUN_001ff858 block (each expression gets its own frame)
    let lo = &w.objects[tpl];
    let slot = usize::try_from(lo.parent_slot).ok()?;
    let e = w.entries.get(lo.entry)?;
    let data = level.get(e.start_offset as usize + 12..)?;
    let (blk, _) = read_script_block(data, e.sections.ext).ok()?;
    let expr = blk.get(slot)?.clone();
    let mut vm = Vm::new();
    // no BIF-name bindings: the stream's symids are opaque
    let fr: FrameRef = frame::new_frame(None);
    vm.global = fr.clone();
    let mut vis = JtVis { jt: tpl, carry: -1, removed: removed.clone(), ..Default::default() };
    if let Some(a) = crate::engine::gfx::attach::alo_of(w, tpl) {
        for (ri, r) in a.mesh.records.iter().enumerate() {
            if r.u1 & 1 != 0 {
                vis.hidden.insert((tpl, ri, None));
            }
            for (ii, x) in r.instances.iter().enumerate() {
                if x.u1 & 1 != 0 {
                    vis.hidden.insert((tpl, ri, Some(ii)));
                }
            }
        }
    }
    // A6: property fields 44 / 45 (show / hide mesh rows): the template's stream, then the instance's
    vis.ee_stream_mesh(w, tpl, tpl);
    if obj != tpl {
        vis.ee_stream_mesh(w, obj, tpl);
    }
    let mut h = PostLoadHost { w, obj: obj as u32, mesh_obj: tpl, vis, objs: ObjectFields::default(), mission_slot, calls: vec![], err: None };
    // the template's stream rows (clones copy them), then the instance's own
    for o in [tpl, obj] {
        for (sym, v) in stream_user_props(w, o) {
            h.objs.ee_sub_200d10(obj as u32, sym, Ref::S32(v));
        }
    }
    if let Err(e) = vm.run_script_block(&mut h, std::slice::from_ref(&expr)) {
        h.err = Some(format!("block: {e:?}"));
    }
    // FUN_001a5c48 (LO post-load, reached first through FUN_00203a38) queues message 0 (self) before
    // FUN_002061d8 queues the 0xe call: the scheduler runs them in that order
    let self_ = Ref::Basic(obj as u32);
    if let Some(p) = frame::lookup(&fr, MSG_LOAD, true).filter(|p| matches!(p, Ref::Proc(_))) {
        if let Err(e) = vm.call(&mut h, &p, vec![self_.clone()], &fr) {
            h.err = Some(format!("msg 0: {e:?}"));
        }
    }
    // FUN_001331f8: FUN_00127db8(*obj, 0xe, 1) then the proc with (self, 0)
    let proc_ = frame::lookup(&fr, MSG_POST_LOAD, true).filter(|p| matches!(p, Ref::Proc(_)))?;
    if let Err(e) = vm.call(&mut h, &proc_, vec![self_, Ref::S32(0)], &fr) {
        h.err = Some(format!("{e:?}"));
    }
    // break the frame <-> proc cycle
    fr.borrow_mut().bindings.clear();
    Some(h)
}
