//! WP-J5: get-o / set-o for ordinary fields and user properties, and the world+0x3ca8 deferred
//! member-call list + BIF 107 (RefOpDeferObjectUpdate).
//!
//! get-o (BIF 111 0x00124048 -> FUN_00123eb0 kind 0x6f -> FUN_00200ba0(obj, symid)):
//!  - symid < 0x4d1: FUN_001ffcf8 finds the field row of the object's class (`fieldbind.rs`, the
//!    0x00279ad0 tables replayed with the original binary search), FUN_001ffd98 converts the member by
//!    field kind: 0 Bool(!=0), 1 F32, 2 Vector, 5..8/0xe..0x11/0x13/0x14/0x17 S32, 9 Clq, 10 Lm, 0xb Smp,
//!    kind & 0x1000 / 0x2000 = object (Basic), 0xd / unknown = Nil (out stays tag -1).
//!  - symid >= 0x4d1: FUN_0012ab28 user property list at object +8 ({+8 symid, +0xc CRef, +0x14 next});
//!    missing -> Nil.
//! set-o (BIF 110 -> FUN_00200c28): < 0x4d1 FUN_00200598 member write, else FUN_0012aa70 list set.
//!
//! Replaced: object memory. The member value of a field is the property the level stream wrote
//! (`world::props::Prop` of the object's load) until a script writes it (`ObjectFields::members`); the
//! setter side effects of FUN_00200598 (vtable setters) are not run. WP-J6: kind 4 = matrix (see
//! `member_to_ref`); kinds 3 / 0xc fall to the default of FUN_001ffd98's switch (Nil).

use super::fieldbind::FIELD_BINDINGS;
use crate::engine::splice::value::Ref;
use crate::engine::world::loads_b4::B4Load;
use crate::engine::world::objects::{LoadData, Lo, Payload};
use crate::engine::world::props::{Prop, PropValue};
use crate::engine::world::World;
use std::collections::HashMap;

/// First user-property symid (FUN_00200ba0 / FUN_00200c28 split).
pub const SYMID_USER: u32 = 0x4d1;

/// FUN_001ffcf8 for every field kind: the field row bound to (class, symid).
pub fn ee_sub_1ffcf8_field(class: u16, symid: u16) -> Option<u16> {
    // 0x001ffcf8 (bindings already resolved by the original binary search in the generator)
    let i = FIELD_BINDINGS.partition_point(|b| (b.0, b.1) < (class, symid));
    FIELD_BINDINGS.get(i).filter(|b| b.0 == class && b.1 == symid).map(|b| b.2)
}

/// Property list of an object's load (the members the stream wrote).
pub fn object_props(o: &Lo) -> Option<&[Prop]> {
    let Payload::Loaded(d) = &o.payload else { return None };
    Some(match d {
        LoadData::Lo(p) => p,
        LoadData::L231170 { props, .. } | LoadData::L227380 { props, .. } | LoadData::L231170Blob { props, .. } => props,
        LoadData::Solid(s) => &s.props,
        LoadData::L2282b0 { solid, .. } => &solid.props,
        LoadData::Alo { alo, .. } => &alo.props,
        LoadData::Aseg(a) => &a.props,
        LoadData::B4(b) => match &**b {
            B4Load::L185150 { props, .. }
            | B4Load::L1e5df8 { props, .. }
            | B4Load::L220fd0 { props, .. }
            | B4Load::L1c03c0 { props, .. }
            | B4Load::L258598 { props, .. }
            | B4Load::L15c1e8 { props, .. }
            | B4Load::L231170 { props, .. }
            | B4Load::L1efac0 { props, .. }
            | B4Load::L212298 { props, .. }
            | B4Load::L1624a8 { props, .. }
            | B4Load::L1635f8 { props, .. } => props,
            B4Load::Lo(p) => p,
            _ => return None,
        },
    })
}

/// WIRE-EP7: the field-8 rows (FUN_00132d88 ensure-o {symid (kind 0x15), S32 (kind 6)}) FUN_0013f810 ran
/// at load for `obj`: its own stream props, then the FUN_001c9b08 override rows, in order.
pub fn load_user_props(w: &World, obj: usize) -> Vec<(u32, i32)> {
    use crate::engine::world::props::ArrayItem;
    let own = w.objects.get(obj).and_then(object_props).unwrap_or(&[]);
    let ov = w.override_props.get(&obj).map(|v| v.as_slice()).unwrap_or(&[]);
    let mut out = vec![];
    for p in own.iter().chain(ov).filter(|p| p.field == 8) {
        let PropValue::Array(items) = &p.value else { continue };
        let (Some(ArrayItem::One(0x15, s)), Some(ArrayItem::One(6, v))) = (items.first(), items.get(1)) else { continue };
        let sym = s.get(..2).map(|b| u16::from_le_bytes([b[0], b[1]]) as u32).unwrap_or(0);
        let val = v.get(..4).map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]])).unwrap_or(0);
        out.push((sym, val));
    }
    out
}

fn f(b: &[u8], i: usize) -> f32 {
    b.get(i * 4..i * 4 + 4).map(|x| f32::from_le_bytes([x[0], x[1], x[2], x[3]])).unwrap_or(0.0)
}

/// FUN_001ffd98 kind switch (flags & 2 member path) on the raw member bytes.
pub fn member_to_ref(kind: u32, b: &[u8], w: Option<&World>) -> Ref {
    // 0x001ffd98
    let word = || b.get(..4).map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]])).unwrap_or(0);
    let short = || b.get(..2).map(|x| i16::from_le_bytes([x[0], x[1]]) as i32).unwrap_or(0);
    match kind {
        0 => Ref::Bool(b.first().is_some_and(|&v| v != 0)), // FUN_00129598(*p != 0)
        1 => Ref::F32(f(b, 0)),                                // FUN_00129558
        2 => Ref::Vector([f(b, 0), f(b, 1), f(b, 2)]),         // FUN_001296a8
        5 | 6 => Ref::S32(word() as i32),                      // FUN_00129518
        7 | 0xe..=0x11 | 0x13 | 0x14 | 0x17 => Ref::S32(short()),
        8 => Ref::S32(word() as i32),
        4 => {
            // WP-J6: case 4 copies the 3x4 member matrix into a Matrix ref (tag 0x1b) with words 3/7/11
            // cleared. The stream value of a kind-4 field is the rotation vector FUN_0011b570 expands at
            // load; a 0x30-byte value is the matrix itself. Words 12..15 (no member data) = 0, 0, 0, 1.
            let m = if b.len() >= 0x30 {
                [[f(b, 0), f(b, 1), f(b, 2), 0.0], [f(b, 4), f(b, 5), f(b, 6), 0.0], [f(b, 8), f(b, 9), f(b, 10), 0.0]]
            } else {
                let r = crate::engine::camera::math::FUN_0011b570([f(b, 0), f(b, 1), f(b, 2)]);
                [[r[0][0], r[0][1], r[0][2], 0.0], [r[1][0], r[1][1], r[1][2], 0.0], [r[2][0], r[2][1], r[2][2], 0.0]]
            };
            let mut o = [0.0f32; 16];
            for (i, row) in m.iter().enumerate() {
                o[i * 4..i * 4 + 4].copy_from_slice(row);
            }
            o[15] = 1.0;
            Ref::Matrix(o)
        }
        9 => Ref::Clq([f(b, 0), f(b, 1), f(b, 2)]), // FUN_00129808
        10 => Ref::Lm([f(b, 0), f(b, 1)]),          // FUN_00129878
        0xb => Ref::Smp([f(b, 0), f(b, 1), f(b, 2)]), // FUN_001298f0
        k if k & 0x1000 != 0 || k & 0x2000 != 0 => {
            // FUN_00129a10: object pointer. Stream value = s32 oid (resolved after the load).
            let oid = word() as i32;
            Ref::Basic(w.and_then(|w| w.find_oid(oid).first().copied()).map(|i| i as u32).unwrap_or(0))
        }
        _ => Ref::Nil,
    }
}

/// WP-J8: field descriptor 19 (0x002ac880 + 19 * 0x28): kind 0x2030, flags 2, member offset 0x10.
pub const FIELD_LO_OWNER: u16 = 19;

/// WP-J11: field descriptor 27 (kind 0, flags 4): getter FUN_00259ae8 = LO +0x28 bit 60 ("in the world").
pub const FIELD_IN_WORLD: u16 = 27;
/// WP-J11: field descriptor 21 (kind 2, flags 0x24): getter FUN_001a62e8 = world matrix (vtable +0xfc)
/// +0x30, the world position. Set-o keeps going through `members` (read first by get-o).
pub const FIELD_ALO_POS: u16 = 21;

/// WP-J12: field descriptor 15 (kind 6, flags 2, member offset 0x18, symid 0x1c on every class): the
/// LO +0x18 oid, read directly (FUN_00129518(*(obj + 0x18))), not a stream property.
pub const FIELD_LO_OID: u16 = 15;
/// WP-J13: SW field 926 (getter FUN_001c4840, the current player).
pub const FIELD_SW_PLAYER: u16 = 926;
/// WP-J12: field descriptor 22 (kind 4, flags 0x24): getter FUN_001a6318 = the world matrix (vtable
/// +0xfc) itself, converted by the kind-4 path (rotation rows, no translation).
pub const FIELD_ALO_MAT: u16 = 22;

/// WP-J12: FUN_001a6318(alo): the object's world matrix (vtable +0xfc), as `ee_sub_1a62e8`.
pub fn ee_sub_1a6318(w: &World, i: usize) -> crate::engine::gfx::scene::Affine {
    // 0x001a6318
    let mut bw = crate::engine::gfx::attach::BoneWorlds::default();
    match crate::engine::gfx::attach::alo_of(w, i) {
        Some(al) if !al.base.bones.is_empty() => bw.get(w, i, 0),
        _ => bw.xfm_world(w, i).unwrap_or(crate::engine::gfx::scene::IDENTITY),
    }
}

/// WP-J11: FUN_001a62e8(alo): translation row of the object's world matrix. Replaced: world matrices are
/// the rest-pose root joints (`gfx::attach::BoneWorlds`, joint 0) / class-3 XFM matrices, identity
/// otherwise (as `ref_op_convert_object_position`).
pub fn ee_sub_1a62e8(w: &World, i: usize) -> [f32; 3] {
    // 0x001a62e8
    let mut bw = crate::engine::gfx::attach::BoneWorlds::default();
    let m = match crate::engine::gfx::attach::alo_of(w, i) {
        Some(al) if !al.base.bones.is_empty() => bw.get(w, i, 0),
        _ => bw.xfm_world(w, i).unwrap_or(crate::engine::gfx::scene::IDENTITY),
    };
    m[3]
}

/// WP-J11: LO +0x28 bit 60 after the level load. FUN_001a5270 sets it on the root at init and children
/// inherit it, so every loaded object starts in the world; the ALO post-load then runs the deferred
/// Remove of property field 26 (FUN_001f36a8 -> FUN_001a5d98 -> FUN_001a5d08 -> vtable +0x4c FUN_001a5630,
/// recursive -> FUN_001a5848 clears the bit) on the object and its subtree. Out = the object or an
/// ancestor carries field 26 (as `playerdraw::vis::JtVis::load` for the JT subtree).
pub fn out_of_world_at_load(w: &World, obj: usize) -> bool {
    let mut o = Some(obj);
    while let Some(i) = o.filter(|&i| i < w.objects.len()) {
        if removed_self_at_load(w, i) {
            return true;
        }
        o = w.objects[i].owner;
    }
    false
}

/// POOL1 R99: the object's OWN load Remove (LO +0x28 bit 56 at load: it sits on its parent's removed-child list
/// +0x70), without the ancestors of `out_of_world_at_load` (rec11 005: guard 11539 @0xefba60 in the world, its
/// child 13144 @0x785c00 b56 1 out, 13144's child 13145 b56 0 out under it)
pub fn removed_self_at_load(w: &World, i: usize) -> bool {
    if i >= w.objects.len() {
        return false;
    }
    if unplaced_copy(w, i) {
        return true;
    }
    // POOL1 R95: a player load bind-table Remove (World::bind_removed; e.g. Sly's class-29 emitters oid 385 =
    // 0x2bc9f0 row 52 {0x300, 385, +0x12f8} and oid 255 = 0x2be990 row 111 {0x300, 255, +0x1ab0}: rec11 001 has
    // both Removed, LO +0x28 bit 56). Default since R96 (opt-out SLY2_R96_NO_BINDRM=1): with it (and the rec5 052
    // load rand) feel_inworld_live_saved_matches_original_ep1_hub passes with R64 on and off
    if w.bind_removed.contains(&i) && std::env::var_os("SLY2_R96_NO_BINDRM").is_none() {
        return true;
    }
    // FEEL R34: a FUN_0013e510 copy runs its source's deferred list: FUN_001a6730 (the LO part of vtable
    // +0x14c) copies LO +0x34 (`puVar13[0xd] = param_2[0xd]`), so the field-26 Remove applies to the copy too
    let mut s = Some(i);
    while let Some(j) = s.filter(|&j| j < w.objects.len()) {
        if object_props(&w.objects[j]).is_some_and(|p| p.iter().any(|p| p.field == crate::engine::playerdraw::vis::FIELD_REMOVE)) {
            return true;
        }
        // POOL1 R94: any FUN_001c9b08 placement override row with field 26 (FUN_0013f810 on the copy runs the
        // field-26 setter FUN_00100b10 -> deferred Remove, as World::has_field_26): rec11 001 has the hub's entry-14
        // emitters 100712 / entry-18 copies Removed (LO +0x28 bit 56) at load. Default on since R95 (opt-out
        // SLY2_R94_NO_OVRREMOVE=1); the guard tests that depend on the draw order run the one shared rand (R95)
        // (class-6 template records are not objects in the original: their rows are the in-place case below)
        // POOL1 R96: only the copy's OWN row (FUN_001c9b08 runs per placement; the EP1 hub entry 4/5/6 copies 1 =
        // clones of the rowed copy 0 are in the original's world in rec5 210; opt-out SLY2_R96_OVRCLONE=1)
        if std::env::var_os("SLY2_R94_NO_OVRREMOVE").is_none() && (j == i || std::env::var_os("SLY2_R96_OVRCLONE").is_some()) && w.objects[j].class != 6 && w.override_props.get(&j).is_some_and(|p| p.iter().any(|p| p.field == crate::engine::playerdraw::vis::FIELD_REMOVE)) {
            return true;
        }
        // CRAWL R90: the FUN_001c9b08 kind-5 rows ran FUN_0013f810 on the in-place spawner itself
        if matches!(w.objects[j].payload, crate::engine::world::objects::Payload::Loaded(crate::engine::world::objects::LoadData::B4(ref b)) if matches!(**b, crate::engine::world::loads_b4::B4Load::Class6 { in_place: true, .. }))
            && w.override_props.get(&j).is_some_and(|p| p.iter().any(|p| p.field == crate::engine::playerdraw::vis::FIELD_REMOVE))
        {
            return true;
        }
        s = w.objects[j].clone_of;
    }
    false
}

/// CRAWL R109: LO +0x28 bit 56 at load as the R99 Add skip uses it: the object's (or its copy source's, FEEL R34:
/// FUN_001a6730 copies LO +0x34) OWN field-26 property = its deferred Remove FUN_001a5d30 at post-load ->
/// FUN_001a5630 sets bit 56, so a later Add of the parent skips it (FUN_001a5800 walks the in-world list only).
/// Default: class-44 cameras only (the gang's owned cameras 7563 / 4332 are out of the original's world while the
/// safehouse script's Remove+Add of the gang re-adds the gang). Staged: for every class (SLY2_R109_B56ALL=1) the
/// ep1 hub guard pool picks change through the rand draw order (feel_inworld ep1_hub: 125317 copy 1 hidden;
/// own-only without copies: wired_guards_hear_search guard recycled), as POOL1's whole `removed_self_at_load`
/// (SLY2_R99_B56=1; ep1 hub 88309 copy 0 has an override field-26 row but is in rec5 210's world).
/// SLY2_NO_R109_B56OWN=1 = no skip (old whole-subtree Add).
pub fn b56_at_load(w: &World, i: usize) -> bool {
    if std::env::var_os("SLY2_R99_B56").is_some() {
        return removed_self_at_load(w, i);
    }
    if i >= w.objects.len() || (w.objects[i].class != 44 && std::env::var_os("SLY2_R109_B56ALL").is_none()) {
        return false;
    }
    let mut s = Some(i);
    while let Some(j) = s.filter(|&j| j < w.objects.len()) {
        if object_props(&w.objects[j]).is_some_and(|p| p.iter().any(|p| p.field == crate::engine::playerdraw::vis::FIELD_REMOVE)) {
            return true;
        }
        s = w.objects[j].clone_of;
    }
    false
}

fn b56_skip_on() -> bool {
    std::env::var_os("SLY2_R99_B56").is_some() || std::env::var_os("SLY2_NO_R109_B56OWN").is_none()
}

/// CRAWL R109: the Add walk skips child `c` (LO +0x28 bit 56): `rt` = its run-time bit (last direct Add / Remove),
/// else `b56_at_load`. Staged like `b56_at_load`: class 44 only unless SLY2_R99_B56 / SLY2_R109_B56ALL (the run-time
/// skip of a directly Removed child for every class alone hides ep1 hub guard 125317 copy 1, feel_inworld)
pub fn b56_skips(w: &World, c: usize, rt: Option<bool>) -> bool {
    b56_skips_in(w, c, rt, true)
}

/// ANIM-ALL R87: the script path (method 0x100a90 -> vt +0x44, `ObjectFields::set_in_world`) keeps the whole-subtree
/// Add for the class-29 emitters (`emit_rule` false): rec5 101 -> 102 the deferred FUN_001a5d08 Removes ran for every
/// node holder (709 of 725 b56 1), but the EP1 hub holder 141539 copies (class 48, entry 152), Added after that, have
/// their 88309 emitters / 6390 children back in with bit 56 clear (as FEEL R36's boss parts); rec_ep1b 060 -> 061 the
/// holders were Added before the batch and their 88309 stay out (b56 1). Opt-out SLY2_R87_SCRIPTEMIT=1 (rule there too).
/// ANIM-ALL R88: only a class-48 holder's Add (`root`) re-adds them: decomp FUN_001a5478 (Add) clears bit 56 on the
/// object alone and never walks a removed child, and FUN_001a5630 (Remove) sets it unconditionally, so 88309's b56 0
/// at rec5 102 is its deferred Remove never running: the 141539 holder copies are detached pool members (p 0, b56 1 at
/// rec5 101 / rec_ep1b 059) when the post-load batch runs, claimed (attached + Added) later. A char's Add (Sly /
/// Bentley / Murray, attached at load) keeps its emitters out (PCSX2 002: 385 / 3742 / 4624 / 7462 b56 1 under the
/// in-world chars). EP2 table 8 draws/f with B56EMIT + B56HOLD: 470 -> 125. Opt-out SLY2_NO_R88_HOLDADD=1 (R87: every Add).
pub fn b56_skips_script(w: &World, root: usize, c: usize, rt: Option<bool>) -> bool {
    let holder = std::env::var_os("SLY2_NO_R88_HOLDADD").is_some() || w.objects.get(root).is_some_and(|o| o.class == 48);
    b56_skips_in(w, c, rt, std::env::var_os("SLY2_R87_SCRIPTEMIT").is_some() || !holder)
}

fn b56_skips_in(w: &World, c: usize, rt: Option<bool>, emit_rule: bool) -> bool {
    let emit = emit_rule && r86_b56_emitter(w, c);
    if !emit && std::env::var_os("SLY2_R99_B56").is_none() && std::env::var_os("SLY2_R109_B56ALL").is_none() && w.objects.get(c).is_none_or(|o| o.class != 44) {
        return false;
    }
    rt.unwrap_or_else(|| if emit { removed_self_at_load(w, c) || w.has_field_26(c) } else { b56_at_load(w, c) })
}

/// ANIM-ALL R86: class-29 emitters (C2ceb78) take the whole POOL1 `removed_self_at_load` as their LO +0x28 bit 56
/// for the Add skip (FUN_001a5478 / FUN_001a5800 walk only the in-world child list +0x64; a child on the removed
/// list +0x70 stays out with its subtree). PCSX2 cap_anim_ep2_guard_walk 002: the emitters under the in-world
/// guards (C2cf520 / C2c9dc0), Sly, Bentley, Murray, class 99 are out with bit 56 set. Emitters never feed the
/// guard pool picks except through the shared rand stream. A FUN_001f66f0 kid clone inherits its source's LO +0x34
/// deferred list (FUN_001a6730), override field-26 rows included (`World::has_field_26`; PCSX2 002: every 19724 /
/// 19726 copy under the 11539 guard copies has the FUN_001a5d08 node and bit 56). DEFAULT since ANIM-ALL R88 (with the R88 holder-only script re-add: feel_inworld ep1 hub + wired_guards green), opt-out SLY2_NO_R86_B56EMIT=1 (EP2 table 8
/// draws/f 657 -> 245, original ~55): EP1 hub emitter 88309 (own override field-26 row, FUN_001a5d08 node) is in the
/// original's world with bit 56 clear at rec5 052 / 210, re-added by something not modelled -> feel_inworld ep1 hub;
/// opt-in SLY2_R86_B56HOLD=1 also class-48 holders (PCSX2 002: the 4299 / 13079 holders under guards are out).
fn r86_b56_emitter(w: &World, c: usize) -> bool {
    w.objects.get(c).is_some_and(|o| o.class == 29 || (o.class == 48 && std::env::var_os("SLY2_R86_B56HOLD").is_some())) && b56_skip_on() && std::env::var_os("SLY2_NO_R86_B56EMIT").is_none()
}

/// FEEL R35: `obj` is one of its directory entry's instances (entry +4, m * n) whose copy no FUN_001ca370
/// placement handed out at load (copy index >= entry +0x0c). Only FUN_001ca370 Adds a copy at load; the
/// rest start out of the world until FUN_001caec8 (spawner slot 22 / FUN_001cb250) Adds them.
pub fn unplaced_copy(w: &World, obj: usize) -> bool {
    let Some(e) = w.entries.get(w.objects[obj].entry) else { return false };
    let m = e.objects.len();
    m > 0 && e.instances.iter().position(|&x| x == obj).is_some_and(|k| (k / m) as i64 >= e.load_count as i64)
}

/// FEEL R32: LO +0x28 bit 60 of `obj` (in the world's update list) as one shared check: the scripts' object
/// state (`ObjectFields::in_world`: script / job-toggle Add 0x44 / Remove 0x4c) when there is one, else the
/// load-time state (`out_of_world_at_load`, the deferred field-26 Remove).
pub fn lo_in_world(objs: Option<&ObjectFields>, w: &World, obj: usize) -> bool {
    match objs {
        Some(o) => o.in_world(Some(w), obj as u32),
        None => !out_of_world_at_load(w, obj),
    }
}

/// Run-time object state scripts reach through get-o / set-o (object memory in the original).
#[derive(Clone, Debug, Default)]
pub struct ObjectFields {
    /// member writes by FUN_00200598: (object, field) -> value
    pub members: HashMap<(u32, u16), Ref>,
    /// FUN_0012aa70 user property lists (object +8), in insertion order
    pub user: HashMap<u32, Vec<(u32, Ref)>>,
    /// BIF 107: object +0x30 (next update time)
    pub next_update: HashMap<u32, f32>,
    /// world+0x3ca8 deferred member calls (FUN_00149a50 / FUN_00149ab8 / FUN_00149b58)
    pub deferred: Vec<DeferredCall>,
    /// nodes FUN_00149c80 dispatched (the member functions behind the PMFs are owned by other
    /// subsystems; the port records the calls in order for them)
    pub dispatched: Vec<DeferredCall>,
    /// method 0x00101d20 (FUN_00130b18 retract) requests on ASEG objects, consumed by `anim_play`
    pub retract: Vec<u32>,
    /// WP-J6: LO Add/Remove (vtable +0x44 / +0x4c, methods 0x00100a90 / 0x00100b88) and mesh record
    /// show/hide (FUN_001f6978 / FUN_001f69f8) done by scripts, on any object (`jt` unused here)
    pub vis: crate::engine::playerdraw::vis::JtVis,
    /// WP-J6: the oid list at object +0x84 (count) / +0x88 (8 slots), FUN_001e0020 / FUN_001e0070
    pub oid_list: HashMap<u32, Vec<i32>>,
    /// WP-J8: ALO +0x110 byte per part (26), FUN_001f7918 (method 0x00100fe0)
    pub alo_110: HashMap<u32, [u8; 26]>,
    /// WP-J8: 0x3dd040 +0x204 count / +0x208 stack (<= 8) of music instances, each kept as the +0x40
    /// key of its definition (instance +0x60, a +0xf0 record). Pushed by FUN_001fe150 (not ported).
    pub music_stack: Vec<i32>,
    /// WP-J8: 0x3dd040 +0x5c / +0x60 counters (FUN_001fee90 / 001feea0 / 001feeb0 / 001feec0)
    pub music_5c: i32,
    pub music_60: i32,
    /// WP-J9: 0x3dd040 +0x200 current music instance, kept as the +0x40 key of its +0xec definition
    /// (FUN_001fe5f0 sets it, FUN_001fe738 clears it, FUN_001fe848 tests it)
    pub music_cur: Option<i32>,
    /// PORT2: methods 0x001025a8 / 0x00102630 / getter 0x00102538 -> MusicMgr (`music_link`)
    pub music_link: super::music_link::MusicLink,
    /// WP-J8: global 0x3da0c0 (class 154)
    pub g3da0c0: G3da0c0,
    /// WP-J9: run-time records getter thunks create (Basic = `BASIC_RT | index`), in creation order
    /// (FUN_0011a220 appends at the tail; the list searches walk from the head)
    pub rt: Vec<RtRecord>,
    /// WP-J10: `rt` indices FUN_001f0598 took off their list (0x3e126c links); list searches skip them
    pub rt_unlisted: std::collections::HashSet<u32>,
    /// WP-J9: FUN_0012f9b8 play requests from getter 0x00101910 (ASEG, svt), consumed by `anim_play`
    pub aseg_play: Vec<(u32, f32)>,
    /// WP-J11: objects out of the world (LO +0x28 bit 60 clear); None until the first script Add / Remove
    /// (until then [`out_of_world_at_load`] answers). See [`ObjectFields::in_world`].
    pub out_of_world: Option<std::collections::HashSet<u32>>,
    /// POOL1 R99: script-path LO +0x28 bit 56 set by a direct Add (false) / Remove (true); else `removed_self_at_load`
    pub lo_b56: HashMap<u32, bool>,
    /// WP-C8: member writes on LIGHT (class 4) objects, (object, field, value); FUN_00200598 calls the
    /// LIGHT setter (0x1a35b0.., ends with FUN_001a3968). Drained by `gfx::screen::drain_light_writes`.
    pub light_writes: Vec<(u32, u16, Ref)>,
    /// WP-EP2F: gadget slot state (jt+0x1180) of level JT objects (class 53), keyed by object index; absent =
    /// constructor state (all slots -1). Read by getter 0x0010b490 (FUN_00141e78).
    pub gad: HashMap<u32, crate::engine::player::p18_gadget::Gadgets>,
    /// WP-EP2F: class-0x8f element words +0x60 / +0x64 (raw f32 bits) set by FUN_001ee678 through method
    /// 0x00101218 (FUN_001f7fa0), keyed (ALO object, index in `MeshSet::descs`)
    pub desc_60: HashMap<(u32, usize), (u32, u32)>,
}

/// WP-J9: Basic payloads of `ObjectFields::rt` records (bit 29; emitters use bit 30, objects are indices).
pub const BASIC_RT: u32 = 0x2000_0000;
/// HEIST r16: Basic payload of a usable object's embedded JOY tracker (class 137, member +0x150 of the
/// C2d5898 family, field 888 / sym 810 kind 0x2089): `BASIC_JOY | object index` (bit 28).
pub const BASIC_JOY: u32 = 0x1000_0000;

/// WP-J9: a run-time record (pool object the original allocates with FUN_0011ccc0).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RtRecord {
    /// class id of the record's vtable (+0x34 class info +4): 134 ASEGA (vtable 0x2caf48), 136 link
    /// (0x2d9768), 147 (0x2ce2b8)
    pub class: u16,
    /// ASEGA +0x14 ASEG / link +0x24 template / class 147 +0x14 owner
    pub owner: u32,
    /// ASEGA +0x18 ALO / link +0x28 ALO / class 147 +0x40 object
    pub alo: u32,
    /// ASEGA +0x1c svt / class 147 +0x20 vector x (raw f32 bits)
    pub f: f32,
    /// ASEGA +0x3c flags / link +0x2c (FUN_001f01b8 param_4)
    pub arg: i32,
}

impl ObjectFields {
    /// WP-J9: append a record to the run-time list, returning its Basic payload.
    pub fn rt_push(&mut self, r: RtRecord) -> u32 {
        self.rt.push(r);
        BASIC_RT | (self.rt.len() - 1) as u32
    }

    /// WP-J9: record of a `BASIC_RT` payload.
    pub fn rt_get(&self, basic: u32) -> Option<&RtRecord> {
        if basic & !0x0fff_ffff != BASIC_RT {
            return None;
        }
        self.rt.get((basic & 0x0fff_ffff) as usize)
    }

    /// WP-J10: FUN_001f0598(link) (method 0x00104bf8): a link still on 0x3e126c (FUN_001f0638) leaves it
    /// and its template's +0x68 list. The +0x30 ASEGA retract is not reached (FUN_001f13a8 never sets it
    /// here, see `methods::getters`); the vtable +0x1c message 3 and +0x14 free are not run (record-only
    /// links). Returns false when `basic` is not a link record.
    pub fn ee_sub_1f0598(&mut self, basic: u32) -> bool {
        // 0x001f0598
        if self.rt_get(basic).map(|r| r.class) != Some(136) {
            return false;
        }
        self.rt_unlisted.insert(basic & 0x0fff_ffff);
        true
    }

    /// FUN_001f6788(alo, oid) (list 0x3e1218 = ASEGAs) / FUN_001f67e0(alo, oid) (list 0x3e126c = links):
    /// first record of `class` whose ALO is `alo` and whose ASEG / template has oid `oid` (-1 = any).
    pub fn rt_find(&self, class: u16, alo: u32, oid: i32, oid_of: impl Fn(u32) -> i32) -> u32 {
        // 0x001f6788 / 0x001f67e0
        for (i, r) in self.rt.iter().enumerate() {
            if !self.rt_unlisted.contains(&(i as u32)) && r.class == class && r.alo == alo && (oid == -1 || oid_of(r.owner) == oid) {
                return BASIC_RT | i as u32;
            }
        }
        0
    }
}

/// WP-J8: global 0x3da0c0 (class 154, ctor FUN_001de008): a request state machine. The +0x28 actor
/// (FUN_001da2e0: first object of oid 0x371, set at FUN_001d9520) is not modelled and stays 0, so the
/// actor branches of FUN_001da5c8 (FUN_00212880 / FUN_00213830 / vtable +0x194) and the per-frame
/// FUN_001da350 (returns at once when +0x28 == 0) do nothing.
#[derive(Clone, Debug)]
pub struct G3da0c0 {
    /// +0x10
    pub state: i32,
    /// +0x14: clock (DAT_002dded0) of the last state change
    pub t: f32,
    /// +0x18 / +0x1c: current request id / value
    pub id: i32,
    pub value: f32,
    /// +0x20 / +0x24: queued request id / value
    pub next: i32,
    pub next_value: f32,
    /// +0x28: actor handle (0 = none)
    pub actor: u32,
}

impl Default for G3da0c0 {
    /// FUN_001da320 (level reset): +0x20 = +0x18 = -1, +0x28 = 0, state 0.
    fn default() -> Self {
        G3da0c0 { state: 0, t: 0.0, id: -1, value: 0.0, next: -1, next_value: 0.0, actor: 0 }
    }
}

impl G3da0c0 {
    /// FUN_001da5c8(this, state) with actor +0x28 branches; `t` = DAT_002dded0.
    /// `h` is None when called from VM dispatch (actor calls are skipped).
    pub fn ee_sub_1da5c8(&mut self, mut p: i32, t: f32, h: Option<&mut dyn crate::engine::classes::ClassHost>) {
        // 0x001da5c8
        if p == self.state {
            return;
        }
        if self.state == 3 {
            if p == 1 && self.t == t {
                p = 2;
            }
        } else if self.state == 4 {
            self.id = self.next;
            self.value = self.next_value;
            self.next = -1;
            self.next_value = -1.0;
        }
        if p == 0 {
            self.id = -1;
        }
        // +0x28 actor branches
        if self.actor != 0 {
            if let Some(h) = h {
            if self.state == 4 && p == 2 {
                // FUN_00213830(actor, 0x310)
                h.ee_call(0x213830, &[self.actor, 0x310]);
            }
            if p == 3 {
                // vtable +0x194 on actor (via offset +0x190)
                h.vcall(self.actor, 0x194);
                self.state = 3;
                self.t = t;
                return;
            }
            if p == 4 {
                // FUN_00213830(actor, 0x312)
                h.ee_call(0x213830, &[self.actor, 0x312]);
            } else if p == 1 {
                // FUN_00212880(actor, 1, &id)
                h.ee_call(0x212880, &[self.actor, 1, self.id as u32]);
            }
            } // if let Some(h)
        }
        self.state = p;
        self.t = t;
    }

    /// FUN_001da578(this, id): cancel a request.
    pub fn ee_sub_1da578(&mut self, id: i32) {
        // 0x001da578
        if id == -1 {
            self.next = -1;
            self.value = 0.0;
        } else if id == self.id {
            self.value = 0.0;
        } else if id == self.next {
            self.next = -1;
        }
    }

    /// FUN_001da4b8(value, this, id): request `id`.
    pub fn ee_sub_1da4b8(&mut self, value: f32, id: i32, t: f32, h: Option<&mut dyn crate::engine::classes::ClassHost>) {
        // 0x001da4b8
        if id == -1 {
            return self.ee_sub_1da578(-1);
        }
        match self.state {
            0 | 1 | 3 => {
                if self.state == 1 {
                    self.ee_sub_1da5c8(-1, t, None);
                }
                self.id = id;
                self.value = value;
                return self.ee_sub_1da5c8(1, t, h);
            }
            2 => {
                if self.id == id {
                    self.value = value;
                    return;
                }
                self.next_value = value;
            }
            4 => self.next_value = value,
            _ => return,
        }
        self.next = id;
    }
}

/// One world+0x3ca8 node (pool world+0x3c90): {+0 object, +4 PMF (s16 adj, s16 vtable index,
/// fn / s16 vtable offset), +0xc arg, +0x10 next}.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeferredCall {
    pub object: u32,
    /// PMF: +4 this-adjust, +6 vtable index (-1 = direct fn), +8 fn (index < 0) or vtable offset
    pub adj: i16,
    pub vindex: i16,
    pub fnw: u32,
    pub arg: u32,
}

impl ObjectFields {
    /// WP-J11: FUN_00259ae8 (field 27 getter): LO +0x28 bit 60.
    pub fn in_world(&self, w: Option<&World>, obj: u32) -> bool {
        match (&self.out_of_world, w) {
            (Some(s), _) => !s.contains(&obj),
            (None, Some(w)) => !out_of_world_at_load(w, obj as usize),
            (None, None) => true,
        }
    }

    /// WP-J11: the bit side of LO Add (vtable +0x44 FUN_001a5478 -> FUN_001a5800 sets it) / Remove
    /// (vtable +0x4c FUN_001a5630 -> FUN_001a5848 clears it), both recursive over the subtree.
    pub fn set_in_world(&mut self, w: &World, obj: usize, on: bool) {
        // REPLACED (script path): the whole subtree, as before FEEL R36 -- BOSS's Dimitri (122332) relies on the
        // level script's Add of the in-world boss re-adding his field-26-removed parts (FUN_001a5478 would return
        // at once: bit 56 clear); REQUESTS.md FEEL R36 -> BOSS
        let s = self.out_of_world.get_or_insert_with(|| (0..w.objects.len()).filter(|&i| out_of_world_at_load(w, i)).map(|i| i as u32).collect());
        // POOL1 R99: LO +0x28 bit 56 (removed on its own): the Add clears it on `obj` and FUN_001a5800 walks only
        // the in-world child lists, so a child removed on its own stays out with its subtree (rec11 005: the
        // dazed-birds 13144 copies under the in-world hub guards 11539 / 44949 all out, b56 1). Opt-in SLY2_R99_B56=1 (default off: with it feel_inworld_live_saved_matches_original_ep1_hub and wired_guards hear_search / sly_knocks_out fail, POOL1.md R99).
        self.lo_b56.insert(obj as u32, !on);
        let sub = if on && b56_skip_on() {
            let mut out = vec![];
            let mut st = vec![obj];
            while let Some(i) = st.pop() {
                out.push(i);
                st.extend(w.objects[i].children.iter().rev().copied().filter(|&c| !b56_skips_script(w, obj, c, self.lo_b56.get(&(c as u32)).copied())));
            }
            out
        } else {
            crate::engine::playerdraw::subtree(w, obj)
        };
        for i in sub {
            let changed = if on { s.remove(&(i as u32)) } else { s.insert(i as u32) };
            if changed && std::env::var("SLY2_R120_WATCH").is_ok_and(|v| v.split(',').any(|x| x == i.to_string())) {
                eprintln!("R120 watch obj {i} (script, via {obj}) in_world {on}
{}", std::backtrace::Backtrace::force_capture());
            }
        }
    }

    pub fn set_in_world_lo(&mut self, w: &World, obj: usize, on: bool) {
        let s = self.out_of_world.get_or_insert_with(|| (0..w.objects.len()).filter(|&i| out_of_world_at_load(w, i)).map(|i| i as u32).collect());
        // FEEL R36: FUN_001a5478 (Add) returns at once unless LO +0x28 bit 56 (removed) is set, FUN_001a5630
        // (Remove) unless it is clear; each recursed child (vtable +0x44 / +0x4c) checks its own bit. So an Add
        // of an object already in the world leaves its removed children (e.g. unplaced copies) out.
        // POOL1 R99: and the walk skips a child removed on its own (bit 56, see `set_in_world`)
        let b56_on = on && b56_skip_on();
        self.lo_b56.insert(obj as u32, !on);
        let mut st = vec![obj];
        while let Some(i) = st.pop() {
            let changed = if on { s.remove(&(i as u32)) } else { s.insert(i as u32) };
            if changed && std::env::var("SLY2_R120_WATCH").is_ok_and(|v| v.split(',').any(|x| x == i.to_string())) {
                eprintln!("R120 watch obj {i} (via {obj}) in_world {on}
{}", std::backtrace::Backtrace::force_capture());
            }
            if changed {
                let b = &self.lo_b56;
                st.extend(w.objects[i].children.iter().rev().copied().filter(|&c| !b56_on || !b56_skips(w, c, b.get(&(c as u32)).copied())));
            }
        }
    }

    /// get-o (FUN_00200ba0) for non-method fields and user properties. `None` = symid not bound to a
    /// field of that class (FUN_001ffcf8 returned 0: FUN_001ffd98 is not called, result Nil).
    pub fn ee_sub_200ba0(&self, w: Option<&World>, obj: u32, class: Option<u16>, symid: u32) -> Ref {
        // 0x00200ba0
        if symid >= SYMID_USER {
            // FUN_0012ab28
            return self.user.get(&obj).and_then(|l| l.iter().find(|(s, _)| *s == symid)).map(|(_, r)| r.clone()).unwrap_or(Ref::Nil);
        }
        let Some(field) = class.and_then(|c| ee_sub_1ffcf8_field(c, symid as u16)) else { return Ref::Nil };
        if let Some(r) = self.members.get(&(obj, field)) {
            return r.clone();
        }
        let Some(kind) = crate::engine::world::props::field(field as i16).map(|d| d.kind) else { return Ref::Nil };
        // WP-J13: field 926 (SW, flags 4, kind 0x2032) getter FUN_001c4840 = SW +0x3efc current player,
        // set by FUN_001c4940 from the JT's setup -> the level's JT object (`spawn::player_object`)
        if field == FIELD_SW_PLAYER {
            if let Some(p) = w.and_then(crate::engine::game::spawn::player_object) {
                return Ref::Basic(p as u32);
            }
        }
        let Some(o) = w.and_then(|w| w.objects.get(obj as usize)) else { return Ref::Nil };
        // WP-J8: field 19 (kind 0x2030, flags 2, offset 0x10) is the LO +0x10 owner member, read
        // directly (FUN_00129a10(*(obj + 0x10)), 0 at entry level), not a stream property
        if field == FIELD_LO_OWNER {
            return Ref::Basic(o.owner.map(|i| i as u32).unwrap_or(0));
        }
        // WP-J12: field 15 is the LO +0x18 oid member (FUN_00129518)
        if field == FIELD_LO_OID {
            return Ref::S32(o.oid);
        }
        // WP-J11: flags-4 getter rows (descriptor +0xc fn) of the LO / ALO base
        match field {
            FIELD_IN_WORLD => return Ref::Bool(self.in_world(w, obj)),
            FIELD_ALO_POS => {
                if let Some(w) = w {
                    return Ref::Vector(ee_sub_1a62e8(w, obj as usize));
                }
            }
            // WP-J12: the kind-4 conversion of the matrix pointer (FUN_001ffd98: 3 rows, w = 1)
            FIELD_ALO_MAT => {
                if let Some(w) = w {
                    let m = ee_sub_1a6318(w, obj as usize);
                    let mut o = [0.0f32; 16];
                    for (i, row) in m.iter().take(3).enumerate() {
                        o[i * 4..i * 4 + 3].copy_from_slice(row);
                    }
                    o[15] = 1.0;
                    return Ref::Matrix(o);
                }
            }
            _ => {}
        }
        // WIRE-EP7: FUN_001c9b08 override rows were applied after the copy's own props
        let ov = w.and_then(|w| w.override_props.get(&(obj as usize))).and_then(|p| p.iter().rev().find(|p| p.field == field as i16));
        match ov.or_else(|| object_props(o).and_then(|p| p.iter().rev().find(|p| p.field == field as i16))) {
            Some(Prop { value: PropValue::Scalar(b), .. }) => member_to_ref(kind, b, w),
            _ => Ref::Nil,
        }
    }

    /// WP-J8: FUN_001fee90 / FUN_001feea0 / FUN_001feeb0 / FUN_001feec0 (0x3dd040 +0x5c += 1 / -= 1,
    /// +0x60 += 1 / -= 1), methods 0x00102250 / 0x001022c8 / 0x00102340 / 0x001023b8.
    pub fn ee_sub_1fee90(&mut self, callee: u32) {
        // 0x001fee90 / 0x001feea0 / 0x001feeb0 / 0x001feec0
        match callee {
            0x001f_ee90 => self.music_5c += 1,
            0x001f_eea0 => self.music_5c -= 1,
            0x001f_eeb0 => self.music_60 += 1,
            _ => self.music_60 -= 1,
        }
    }

    /// WP-J8: FUN_001fe310(0x3dd040, id) (method 0x001024b0): FUN_001fe478 removes the first stack
    /// entry whose definition is FUN_001fcff8(id) (the +0xf0 record keyed `id`), shifting the rest down.
    /// Replaced: the found instance's fade-out (FUN_001fb098(0, inst) -> FUN_001fb658(inst, 0x1fff, 8)),
    /// the resume of the new top (vtable +0x3c) and the release (vtable +0x14) are audio, not modelled;
    /// entries are matched by key (the first record of that key is the definition).
    pub fn ee_sub_1fe310(&mut self, id: i32) {
        // 0x001fe310
        if let Some(i) = self.music_stack.iter().position(|&k| k == id) {
            self.music_stack.remove(i);
        }
    }

    /// WP-J8: FUN_001f7918(alo, mask, b): +0x110[i] = b for every bit i < 0x1a of mask.
    pub fn ee_sub_1f7918(&mut self, alo: u32, mask: u32, b: u8) {
        // 0x001f7918
        let a = self.alo_110.entry(alo).or_default();
        for (i, v) in a.iter_mut().enumerate() {
            if mask & (1 << i) != 0 {
                *v = b;
            }
        }
    }

    /// WP-J8: add-o (BIF 108, FUN_00123eb0 kind 0x6c -> FUN_00200ca8): FUN_00132b78 (create the list at
    /// object +4 when missing) + FUN_0012a980 = a new {symid, value} node at the list head, for any
    /// symid (no 0x4d1 split). Returns Void (FUN_0012a0b0 tag 0x24).
    pub fn ee_sub_200ca8(&mut self, obj: u32, symid: u32, v: Ref) -> Ref {
        // 0x00200ca8
        self.user.entry(obj).or_default().insert(0, (symid, v));
        Ref::Void
    }

    /// WP-J8: ensure-o (BIF 109, kind 0x6d -> FUN_00200d10): FUN_0012ab28 finds the symid -> FUN_0012aa70
    /// (overwrite) else FUN_0012a980 (add at the head); both return Void (FUN_0012a0b0 tag 0x24).
    pub fn ee_sub_200d10(&mut self, obj: u32, symid: u32, v: Ref) -> Ref {
        // 0x00200d10
        let l = self.user.entry(obj).or_default();
        match l.iter_mut().find(|(s, _)| *s == symid) {
            Some(e) => {
                e.1 = v;
                Ref::Void
            }
            None => self.ee_sub_200ca8(obj, symid, v),
        }
    }

    /// set-o (FUN_00200c28). Returns the CRef FUN_00123eb0 hands back (the value written).
    pub fn ee_sub_200c28(&mut self, obj: u32, class: Option<u16>, symid: u32, v: Ref) -> Ref {
        // 0x00200c28
        // WP-J8: both paths return Void (FUN_0012a0b0 tag 0x24)
        if symid >= SYMID_USER {
            // FUN_0012aa70 replaces an existing property only (a missing one is not added; that is
            // add-o / ensure-o). WP-J9: 1:1 (the J5 append is gone).
            if let Some(e) = self.user.get_mut(&obj).and_then(|l| l.iter_mut().find(|(s, _)| *s == symid)) {
                e.1 = v;
            }
            return Ref::Void;
        }
        // FUN_001ffcf8 + FUN_00200598 (member store; no field = nothing written)
        if let Some(field) = class.and_then(|c| ee_sub_1ffcf8_field(c, symid as u16)) {
            if class == Some(4) {
                if self.light_writes.len() >= 256 {
                    self.light_writes.remove(0);
                }
                self.light_writes.push((obj, field, v.clone()));
            }
            self.members.insert((obj, field), v);
        }
        Ref::Void
    }

    /// BIF 107 RefOpDeferObjectUpdate 0x00123e18: object(argv[0]+4)+0x30 = DAT_002dded0 + (float)argv[1];
    /// returns Void (FUN_0012a0b0 tag 0x24).
    pub fn ee_sub_123e18(&mut self, t: f32, obj: u32, delay: f32) {
        // 0x00123e18
        self.next_update.insert(obj, t + delay);
    }

    /// FUN_00149a50: append a node (FUN_0011a070 = list append).
    pub fn ee_sub_149a50(&mut self, c: DeferredCall) {
        // 0x00149a50
        self.deferred.push(c);
    }

    /// FUN_00149ab8: append unless an identical {object, PMF, arg} node exists (PMF compared as in the
    /// original: vindex equal, then for vindex == -1 the fn word, else the low s16 of +8).
    pub fn ee_sub_149ab8(&mut self, c: DeferredCall) {
        // 0x00149ab8
        let same = |d: &DeferredCall| d.object == c.object && d.vindex == c.vindex && if c.vindex == -1 { d.fnw == c.fnw } else { d.fnw as i16 == c.fnw as i16 } && d.arg == c.arg;
        if !self.deferred.iter().any(same) {
            self.ee_sub_149a50(c);
        }
    }

    /// FUN_00149b58(mask, object, pmf, arg): remove every node matching the masked keys
    /// (1 object, 2 PMF, 4 arg).
    pub fn ee_sub_149b58(&mut self, mask: u32, object: u32, pmf: (i16, u32), arg: u32) {
        // 0x00149b58
        self.deferred.retain(|d| {
            let m = (mask & 1 == 0 || d.object == object)
                && (mask & 2 == 0 || (d.vindex == pmf.0 && if pmf.0 != -1 { d.fnw as i16 == pmf.1 as i16 } else { d.fnw == pmf.1 }))
                && (mask & 4 == 0 || d.arg == arg);
            !m
        });
    }

    /// FUN_00149c80 (deferred part): detach the whole list and call every node, repeated while new
    /// nodes were added by the calls. `call` = the member call (object, PMF, arg).
    pub fn ee_sub_149c80_deferred(&mut self, mut call: impl FnMut(&mut Self, DeferredCall)) {
        // 0x00149c80: while (list not empty) { head = list; clear; for node: call; free }
        while !self.deferred.is_empty() {
            let batch = std::mem::take(&mut self.deferred);
            for c in batch {
                call(self, c);
            }
        }
    }
}

/// HEIST: all FUN_0013f810 user-property rows (field table rows 3..10, method thunks 0x001002a8..
/// 0x00100760 -> q_131990 setters, each ensure-o FUN_00200d10 {symid (kind 0x15), value}) of `obj`:
/// own stream props, then the FUN_001c9b08 override rows, in order. Row 3 FUN_00132bc8 Bool, 4
/// FUN_00132c20 / 8 FUN_00132d88 S32, 5 FUN_00132c78 F32, 10 FUN_00132e40 Basic(null); rows 6/7/9
/// (Clq / Lm / Vector) are passed through as the raw item (unk: not seen on load rows).
/// A load-time copy (`clone_of`) starts from its template's user list (the clone copies the LO body,
/// +8 list included; UNV), its own / override rows then overwrite.
pub fn load_user_props_all(w: &World, obj: usize) -> Vec<(u32, Ref)> {
    use crate::engine::world::props::ArrayItem;
    let own = w.objects.get(obj).and_then(object_props).unwrap_or(&[]);
    let ov = w.override_props.get(&obj).map(|v| v.as_slice()).unwrap_or(&[]);
    let mut out = w.objects.get(obj).and_then(|o| o.clone_of).filter(|&c| c != obj).map(|c| load_user_props_all(w, c)).unwrap_or_default();
    for p in own.iter().chain(ov).filter(|p| (3..=10).contains(&p.field)) {
        let PropValue::Array(items) = &p.value else { continue };
        let Some(ArrayItem::One(0x15, s)) = items.first() else { continue };
        let sym = s.get(..2).map(|b| u16::from_le_bytes([b[0], b[1]]) as u32).unwrap_or(0);
        let raw = match items.get(1) { Some(ArrayItem::One(_, v)) => v.as_slice(), _ => &[][..] };
        let word = || raw.get(..4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])).unwrap_or(0);
        let v = match p.field {
            3 => Ref::Bool(word() != 0),
            4 | 8 => Ref::S32(word() as i32),
            5 => Ref::F32(f32::from_bits(word())),
            10 => Ref::Basic(0),
            _ => continue,
        };
        out.push((sym, v));
    }
    out
}
