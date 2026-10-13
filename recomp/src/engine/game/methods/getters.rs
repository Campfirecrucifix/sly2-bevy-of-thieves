//! WP-J9: thunks of the get-o getter rows (0x002ac880 rows with kind != 0x12 and flags & 1, see
//! `methods::GetterDesc`). Each thunk is `fn(out, obj, ?, argv)`: FUN_00201360 unpacks the arguments,
//! one engine function computes a value, a CRef setter (`GetterRet`) wraps it, FUN_001291a8 copies it out.
//!
//! Ported callees:
//!  - 0x00100228 -> FUN_00132b10(obj, class): is-a (DAT_002baaf0 subtree ends, `player::p4`). 1:1.
//!  - 0x00106910 -> FUN_001e00e8(obj, oid): oid in the +0x84 list (`ObjectFields::oid_list`). 1:1.
//!  - 0x00101910 -> FUN_00130488 -> FUN_0012f9b8: new ASEGA (class 134) {+0x14 ASEG, +0x18 ALO,
//!    +0x1c svt, +0x3c flags} appended to 0x3e1218. Replaced: the record goes to `ObjectFields::rt` and
//!    the playback itself is a request (`aseg_play`) `anim_play::LevelAnims::play` executes; the
//!    0x60-byte state, the per-part ASEGA chain (FUN_0012fe58 ..) and FUN_00131a00 are not built here.
//!  - 0x001013e8 -> FUN_001f6788(alo, oid): first ASEGA of `alo` whose ASEG has oid `oid`. 1:1 on `rt`.
//!  - 0x00104840 -> FUN_001f0380 -> FUN_001f01b8: new link record (class 136) {+0x24 template = obj,
//!    +0x28 ALO, +0x2c third argument} appended to 0x3e126c. Replaced: record only (FUN_001f0438 /
//!    FUN_001f13a8 / FUN_001f0d88 / the vtable +0x1c activation are not run).
//!  - 0x00101468 -> FUN_001f67e0(alo, oid): first link of `alo` whose template has oid `oid`. 1:1 on `rt`.
//!  - 0x0010d840 -> FUN_00156698 -> FUN_00156548: new class-147 record {+0x14 obj, +0x40 arg} appended
//!    to 0x3e148c. Replaced: record only (vector, DAT_003e11f8 serial, vtable +0x1c / +0x14 not run).
//!  - WP-J11: 0x00110738 -> FUN_001d3fa8(obj, key): entry of method 0x001105b8 (FUN_001d4b90) by key.
//!  - WP-J11: 0x001008d8 -> FUN_001a6290(lo, oid): lo or a direct child has oid `oid`. 1:1.

use super::{GetterDesc, GetterRet, MArg};
use crate::engine::game::fields::RtRecord;
use crate::engine::game::vm_state::PersistentHost;
use crate::engine::splice::SpliceError;
use crate::engine::splice::value::Ref;

/// Wrap a raw result word the way the thunk's CRef setter does.
/// WP-S1: getter thunks served by `call` (census, tests/engine_s1.rs).
pub const IMPLEMENTED: &[u32] = &[
    0x0010_0228, 0x0010_08d8, 0x0010_1368, 0x0010_13e8, 0x0010_1468, 0x0010_1588, 0x0010_1910, 0x0010_1a88, 0x0010_2038, 0x0010_2150,
    0x0010_2430, 0x0010_3ab8, 0x0010_4840, 0x0010_6910, 0x0010_d840, 0x0010_e2b8, 0x0010_e328, 0x0010_f7b0, 0x0011_0738,
];

pub fn wrap(ret: GetterRet, v: u32) -> Ref {
    match ret {
        GetterRet::Bool => Ref::Bool(v != 0),
        GetterRet::S32 => Ref::S32(v as i32),
        GetterRet::F32 => Ref::F32(f32::from_bits(v)),
        GetterRet::Basic => Ref::Basic(v),
        _ => Ref::Nil,
    }
}

fn int(a: &[MArg], i: usize) -> i32 {
    match a.get(i) {
        Some(MArg::I32(n)) => *n,
        Some(MArg::Word(w)) => *w as i32,
        Some(MArg::F32(f)) => f.to_bits() as i32,
        _ => 0,
    }
}

fn flt(a: &[MArg], i: usize) -> f32 {
    match a.get(i) {
        Some(MArg::F32(f)) => *f,
        _ => 0.0,
    }
}

/// Run one getter thunk on object `obj` with FUN_00201110-converted arguments.
pub fn call(h: &mut PersistentHost<'_>, g: &GetterDesc, obj: u32, a: &[MArg]) -> Result<Ref, SpliceError> {
    let world = h.world;
    let oid_of = |o: u32| world.and_then(|w| w.objects.get(o as usize)).map(|o| o.oid).unwrap_or(-1);
    let cls = h.class_of(obj).map(|c| c as i32);
    let objs = &mut h.sched.objs;
    let v = match g.m.thunk {
        // 0x00100228: FUN_00132b10(obj, w)
        0x0010_0228 => {
            cls.is_some_and(|c| crate::engine::player::p4::FUN_00132b10(c, int(a, 0))) as u32
        }
        // 0x00106910: FUN_001e00e8(obj, oid)
        0x0010_6910 => objs.oid_list.get(&obj).is_some_and(|l| l.contains(&int(a, 0))) as u32,
        // 0x00101910: FUN_00130488(svt, f2, aseg = obj, alo (kind 0x2030), flags (kind 0x100b))
        0x0010_1910 => {
            let alo = int(a, 0) as u32;
            let svt = flt(a, 1);
            objs.aseg_play.push((obj, svt));
            objs.rt_push(RtRecord { class: 134, owner: obj, alo, f: svt, arg: int(a, 3) })
        }
        // 0x001013e8: FUN_001f6788(alo = obj, oid)
        0x0010_13e8 => objs.rt_find(134, obj, int(a, 0), oid_of),
        // 0x00104840: FUN_001f0380(template = obj, alo (kind 0x2030), a (kind 6), b (kind 0x100b))
        0x0010_4840 => objs.rt_push(RtRecord { class: 136, owner: obj, alo: int(a, 0) as u32, f: 0.0, arg: int(a, 2) }),
        // 0x00101468: FUN_001f67e0(alo = obj, oid)
        0x0010_1468 => objs.rt_find(136, obj, int(a, 0), oid_of),
        // 0x0010d840: FUN_00156698(obj, vector (kind 2), arg (kind 0x2002))
        0x0010_d840 => objs.rt_push(RtRecord { class: 147, owner: obj, alo: int(a, 1) as u32, f: 0.0, arg: 0 }),
        // 0x00102038: FUN_001fe5f0(0x3dd040 = obj, key (kind 0x10), flag (kind 0x105d))
        0x0010_2038 => {
            // FUN_001fe738: stop the current instance; FUN_001fcfa0: the +0xec definition keyed `key`
            objs.music_cur = None;
            let key = int(a, 0);
            objs.music_cur = Some(key);
            objs.rt_push(RtRecord { class: 151, owner: obj, alo: 0, f: 0.0, arg: key })
        }
        // 0x00102150: FUN_001fe848(0x3dd040, key): +0x200 set and (key == -1 or its definition key == key)
        0x0010_2150 => objs.music_cur.is_some_and(|k| int(a, 0) == -1 || k == int(a, 0)) as u32,
        // 0x00102430: FUN_001fe150(0x3dd040, key (kind 0x13)): move / push the instance on the +0x204 stack
        0x0010_2430 => {
            let key = int(a, 0);
            match objs.music_stack.iter().position(|&k| k == key) {
                // FUN_001fe478 found it: resume (vt+0x3c) and push it on top again
                Some(i) => {
                    objs.music_stack.remove(i);
                    objs.music_stack.push(key);
                    objs.rt_push(RtRecord { class: 151, owner: obj, alo: 1, f: 0.0, arg: key })
                }
                // stack full (8) -> 0
                None if objs.music_stack.len() == 8 => 0,
                None => {
                    objs.music_stack.push(key);
                    objs.rt_push(RtRecord { class: 151, owner: obj, alo: 1, f: 0.0, arg: key })
                }
            }
        }
        // 0x0010e2b8 / 0x0010e328: FUN_001cb250(obj, 1) / FUN_001cb328 (FUN_001cb2e0 == 0 -> FUN_001cb250)
        0x0010_e2b8 | 0x0010_e328 => 0,
        // 0x00101588: FUN_001f85e0(alo = obj, id): FUN_001f6030 = first +0xe4 part (0xb0 bytes, count
        // +0xe0) whose +0xc word is `id`; found -> new class-150 record {+0x20 alo, +0x24 part}
        0x0010_1588 => {
            let id = int(a, 0);
            let part = world.and_then(|w| crate::engine::gfx::attach::alo_of(w, obj as usize)).and_then(|al| al.parts.iter().position(|p| p.w0 == id));
            match part {
                Some(p) => objs.rt_push(RtRecord { class: 150, owner: obj, alo: p as u32, f: 0.0, arg: id }),
                None => 0,
            }
        }
        // 0x00103ab8: FUN_00209a68(obj, bit) = FUN_002099f0(+0x2c0, bit) = (+0x2c0 & FUN_00207a20(bit)) != 0
        0x0010_3ab8 => {
            let bit = int(a, 0);
            let m = if bit == -1 { 0 } else { 1u64.checked_shl(bit as u32).unwrap_or(0) };
            (h.sched.cls.guard_2c0(obj as usize) & m != 0) as u32
        }
        // 0x00101a88: FUN_00130598 -> FUN_001304e0(f1, f2, aseg = obj, alo (kind 0x2030), mode (kind
        // 0x100c), flags): FUN_001304a8 = the ASEG's ASEGA on `alo` (+0x50 list, +0x18 alo), else a new one
        // (FUN_0012f9b8(0, 1.0, ..)); then FUN_00130d30(f1, f2, asega, mode) seeks / sets the speed.
        // Replaced: the seek is a play request at svt f1 (`anim_play::LevelAnims::play`).
        0x0010_1a88 => {
            let alo = int(a, 0) as u32;
            let f1 = flt(a, 2);
            let hit = objs.rt.iter().position(|r| r.class == 134 && r.owner == obj && r.alo == alo);
            objs.aseg_play.push((obj, f1));
            match hit {
                Some(i) => super::super::fields::BASIC_RT | i as u32,
                None => objs.rt_push(RtRecord { class: 134, owner: obj, alo, f: f1, arg: int(a, 4) }),
            }
        }
        // WP-J11: 0x001008d8 -> FUN_001a6290(lo = obj, oid): +0x18 oid == oid, else the first-level
        // children (+0x3c list while the entry's owner is `lo`) by oid. 1:1 on `World` owner / children.
        0x0010_08d8 => {
            let oid = int(a, 0);
            world.and_then(|w| w.objects.get(obj as usize)).is_some_and(|o| o.oid == oid || o.children.iter().any(|&k| oid_of(k as u32) == oid)) as u32
        }
        // WP-J11: 0x00110738 -> FUN_001d3fa8(obj, key): first +0x4fc entry (count +0x4f8) whose +0x14 is
        // `key` (method 0x001105b8 records). 1:1 on `rt`.
        0x0011_0738 => {
            let key = int(a, 0);
            objs.rt.iter().position(|r| r.class == 153 && r.owner == obj && r.arg == key).map(|i| super::super::fields::BASIC_RT | i as u32).unwrap_or(0)
        }
        // WP-J13: 0x00101368 -> FUN_001f8198(alo = obj, oid): element of the FUN_00167320 +0xa4 / +0xa8 list
        // (`MeshSet::descs`) by +0x18 oid (see `aim`)
        0x0010_1368 => {
            let oid = int(a, 0);
            let hit = world.and_then(|w| crate::engine::gfx::attach::alo_of(w, obj as usize)).and_then(|al| super::aim::ee_sub_1f8198(&al.mesh.descs, oid));
            match hit {
                Some((i, c)) => match objs.rt.iter().position(|r| r.class == c && r.owner == obj && r.alo == i as u32) {
                    Some(k) => super::super::fields::BASIC_RT | k as u32,
                    None => objs.rt_push(RtRecord { class: c, owner: obj, alo: i as u32, f: 0.0, arg: oid }),
                },
                None => 0,
            }
        }
        // WP-J13: 0x0010f7b0 -> FUN_00155fb0(this, O): aim point (see `aim`); FUN_00129718 -> Vector
        0x0010_f7b0 => {
            let o = match a.first() {
                Some(MArg::Value(Ref::Vector(v))) => *v,
                _ => [0.0; 3],
            };
            let host = &mut h.sched.cls.host;
            let pl = crate::engine::classes::ClassHost::ee_sub_1c4840(host);
            let p0 = crate::engine::classes::ClassHost::ee_sub_1a62e8(host, pl);
            let so = world.map(|w| h.so_parts(w));
            let mut cast = |p: [f32; 3], e: [f32; 3]| -> Option<[f32; 3]> {
                let l = so.as_ref()?.clip_segment(p, e);
                l.iter().min_by(|x, y| x.t[0].total_cmp(&y.t[0])).map(|x| x.pos[0])
            };
            return Ok(Ref::Vector(super::aim::ee_sub_155fb0(o, p0, [1.0, 0.0], [0.0; 3], 0.0, &mut cast)));
        }
        _ => return Err(SpliceError::HostUnimplemented(format!("getter {:#x} -> FUN_{:08x}", g.m.thunk, g.m.callee))),
    };
    Ok(wrap(g.ret, v))
}
