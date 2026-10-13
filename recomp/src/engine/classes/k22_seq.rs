//! WP-K22 item 2: the class-114 (`C2d8c48`) sequence records and their node interpreter.
//! Records (0x50 bytes = 20 words): +0 kind, +4 "don't wait" (0 = the node waits on the action's condition
//! FUN_001e3890 before the next record), +0x10.. arguments. Fields 625..708 append one record each through
//! their thunks 0x106a80..0x10a1b8 -> FUN_001e0290..FUN_001e1940 (kind = field - 625 up to 0x24, fields 661 /
//! 662 both kind 0x24, then field - 626). Kind 0 opens a block (+0x10 context oid), kind 1 closes it.
//! A node (0xf0 bytes, pool 0x3e1560) runs the records [first, last] of one block:
//! FUN_001e1c40 update, FUN_001e1e30 next action, FUN_001e3890 wait condition, FUN_001e1d48, FUN_001e5778
//! (node unlinked), FUN_001e5628 (stop), FUN_001e1db8 (oid -> object), FUN_001e0180 (find a node by
//! block oid), FUN_001e0020 / 1e0070 / 1e00e8 (+0x84 signal list).
//! 1:1: the record layout of every field, the node loop, the subject selection, actions 0 (sub-block),
//! 1, 2 (stop a block), 5 / 6 on the owner, 7 (slot-78 character switch, applied after the frame's class
//! updates through `ClassSet::k18_set_active`: REPLACED timing), 8 (+0x20 subject), 0x44 (title screen FUN_001dd820(0x3da530)),
//! wait conditions 3, 4, 0x44, 0x4a / 0x4e (music time), 0x53.
//! REPLACED: nodes live in the owner (`C2d8c48::k22_nodes`); a node created by the script start
//! (`game::methods::cls::ee_sub_1e5ae0`, handle `NODE_HANDLE | n`, always the root block at record 0)
//! gets its state lazily on its first update, which is therefore the owner's next frame instead of
//! inside FUN_001dfd58. The node refcount / vtable +0xc / +0x14 lock calls are not modelled. The +0x84
//! signal list is the owner's `k22_sig` (the script rows 0x106800 / 0x106888 keep theirs in
//! `game::fields::ObjectFields::oid_list`). Action start time = DAT_002dded0 for both clocks.
//! K23: actions 0xc (context), 0x10 (clock scale 6 + owner bit 0x100), 0x25 / 0x26 (+0xe0), 0x45 (stream
//! FUN_001fe5f0, wait FUN_001fe848) through `ClassHost::k23_clock_14d1d0` / `ee_sub_1fe5f0` / `k23_stream_1fe848`.
//! UNPORTED: the other ~75 action kinds (actors, anims, camera, sounds, dialog, music, HUD, ...) go to
//! `ClassHost::k22_seq_action` (recorded by `EngineHost`), and their wait conditions count as done; the
//! per-action resources at node +0x40..+0xdc and their release in FUN_001e5628 / FUN_001e1d48.
use super::k30_cam::ZoomOp;
use super::c2d8c48::C2d8c48;
use super::ClassHost;
use crate::engine::world::props::{ArrayItem, Prop, PropValue};

pub type Rec = [u32; 20];

/// handles of nodes made by the interpreter (kind 0): this bit | counter
pub const SUB_HANDLE: u32 = 0x0800_0000;
/// `game::methods::cls::NODE_HANDLE`
const ROOT_HANDLE: u32 = 0x4000_0000;

/// one node (the parts the ported actions use)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Node {
    pub handle: u32,
    /// +0x18 context object, +0x1c block target, +0x20 subject override (action 8), +0x24 current subject
    pub ctx: u32,
    pub target: u32,
    pub f_20: u32,
    pub f_24: u32,
    /// +0x28 / +0x2c first / last record, +0x30 current (None = 0)
    pub first: usize,
    pub last: usize,
    pub cur: Option<usize>,
    /// +0x34 current action start time
    pub f_34: f32,
    /// K23: +0x68 the action-0x45 record, +0xe0 actions 0x25 / 0x26 flag
    pub f_68: Option<usize>,
    pub f_e0: i32,
    /// K27: +0xd8 / +0xdc guard / routine of a waiting action 0x55
    pub f_d8: u32,
    pub f_dc: u32,
    /// K27: +0x38 / +0x3c action 0x12 world state (on, flags); +0x60 the object of a waiting fade 0x2c / 0x2d
    pub f_38: u32,
    pub f_3c: u32,
    pub f_60: u32,
    /// K28: +0x40 / +0x44 the action-0x23 record and its subject (FUN_001e4eb8)
    pub f_40: Option<usize>,
    pub f_44: u32,
    /// K28: action 0x38 camera move (FUN_001e57a0): +0x80 end time, +0x84 / +0x88 the 0x38 / 0x39 records,
    /// +0x8c duration, +0x90 / +0x94 / +0x9c state words
    pub f_80: f32,
    pub f_84: usize,
    pub f_88: usize,
    pub f_8c: f32,
    pub f_90: u32,
    pub f_94: u32,
    pub f_9c: u32,
}

/// K27: guard brain calls of the sequence actions (object handles)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GuardOp {
    /// FUN_00171cf8(guard, routine): substate {5, routine}
    Run(u32, u32),
    /// FUN_00171d58(guard): substate {1}
    Stop(u32),
}

/// K27: FUN_00171cf8 (queued for `ClassSet`; the running view is updated at once for the wait)
fn ee_sub_171cf8(c: &mut C2d8c48, g: u32, r: u32) {
    if g == 0 {
        return;
    }
    c.k27_running.retain(|x| x.0 != g);
    c.k27_running.push((g, r));
    c.k27_ops.push(GuardOp::Run(g, r));
}

/// K27: FUN_00171d58
fn ee_sub_171d58(c: &mut C2d8c48, g: u32) {
    if g == 0 {
        return;
    }
    c.k27_running.retain(|x| x.0 != g);
    c.k27_ops.push(GuardOp::Stop(g));
}

/// K27: FUN_00171da8(guard, routine): guard in substate 5 running `routine`
fn ee_sub_171da8(c: &C2d8c48, g: u32, r: u32) -> bool {
    c.k27_running.iter().any(|x| x.0 == g && x.1 == r)
}

/// K27: FUN_001e50e0(node, on, flags): node +0x38 / +0x3c; on: FUN_0020efc0(world, flags) +
/// FUN_0020f0c0 (after releasing a different previous one); off: FUN_0020f030(world, +0x3c) + FUN_0020f0f0
fn ee_sub_1e50e0(c: &mut C2d8c48, n: u32, on: u32, flags: u32, h: &mut dyn ClassHost) {
    let Some(k) = ix(c, n) else { return };
    let nd = &mut c.k22_nodes[k];
    if on == 0 {
        if nd.f_38 == 0 {
            return;
        }
        h.k27_world_20f030(nd.f_3c);
        nd.f_3c = flags;
    } else {
        if nd.f_38 != 0 {
            if flags == nd.f_3c {
                return;
            }
            // FUN_0020f030(0x3dd490) with the flags argument dropped by the decompiler: the old +0x3c
            h.k27_world_20f030(nd.f_3c);
        }
        h.k27_world_20efc0(flags);
        nd.f_3c = flags;
    }
    nd.f_38 = on;
}

/// K27: FUN_00210338(fade, world, on, oid, ctx, owner): FUN_0015e938(4, oid, ctx); fade 0 or no ALO (0x30):
/// vtable +0x44 (on) / +0x4c (off); else FUN_001f7c68(fade, o) (on) / FUN_001f7bf8(fade, o) (off) -> the object
pub(super) fn ee_sub_210338(fade: f32, on: bool, oid: u32, ctx: u32, h: &mut dyn ClassHost) -> u32 {
    let o = h.ee_sub_15e938(4, oid as i32, ctx);
    if o == 0 {
        return 0;
    }
    if fade == 0.0 || h.ee_sub_132b10(o, 0x30) == 0 {
        h.vcall(o, if on { 0x44 } else { 0x4c });
    } else if on {
        h.ee_call(0x1f7c68, &[fade.to_bits(), o]);
    } else {
        h.ee_sub_1f7bf8(o as usize - 1, fade);
    }
    o
}

/// how a field's arguments land in its record
#[derive(Clone, Copy)]
enum B {
    /// +4 = the constant; arguments go to +0x10.. in order
    C(u32),
    /// +4 = argument i, the others to +0x10.. in order
    A(usize),
}

/// field -> (kind, +4)
fn layout(f: i16) -> Option<(u32, B)> {
    use B::*;
    let k = match f {
        625..=661 => (f - 625) as u32,
        662..=708 => (f - 626) as u32,
        // K27: FUN_001e19b0..FUN_001e1b10 (fields 710..716 -> kinds 0x54..0x5a). Field 709 (FUN_001e1970,
        // kind 0x53) is left out: its float / int argument order is ambiguous in the decompile.
        710..=716 => (f - 626) as u32,
        _ => return None,
    };
    let b = match f {
        628 | 629 | 652 | 654..=660 | 665 | 666 | 692 | 693 | 694 | 700 | 704 | 706..=708 => C(0),
        711 | 714 | 715 | 716 => A(2),
        635 | 636 | 639 | 651 | 653 | 670 | 671 | 677 | 695 => A(2),
        638 | 661 | 662 | 667 | 668 | 669 | 685 => A(3),
        676 | 687 | 688 | 690 => A(1),
        686 => A(0),
        705 => A(5),
        _ => C(1),
    };
    Some((k, b))
}

/// one argument value -> words (bool 0 = byte, vector 2 = 3 floats, else the word)
fn words(it: &ArrayItem) -> Vec<u32> {
    let w = |b: &[u8]| b.chunks_exact(4).map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]])).collect::<Vec<_>>();
    match it {
        ArrayItem::One(0, b) => vec![b.first().copied().unwrap_or(0) as u32],
        ArrayItem::One(_, b) => w(b),
        ArrayItem::List(_, l) => l.iter().map(|b| w(b).first().copied().unwrap_or(0)).collect(),
    }
}

/// the record adders FUN_001e0290..FUN_001e1940 (through their field thunks)
pub fn add_record(c: &mut C2d8c48, p: &Prop) {
    let Some((kind, b)) = layout(p.field) else { return };
    let PropValue::Array(items) = &p.value else { return };
    let args: Vec<Vec<u32>> = items.iter().map(words).collect();
    let a = |i: usize| args.get(i).and_then(|x| x.first().copied()).unwrap_or(0);
    let bv = match b {
        B::C(v) => v,
        B::A(i) => a(i),
    };
    let Some(k) = c.ee_sub_1e01f8(kind, bv) else { return };
    let r = &mut c.f_54[k];
    match p.field {
        // FUN_001e0310: count (at most 4) + the oid list
        628 => {
            let l = args.first().cloned().unwrap_or_default();
            let n = l.len().min(4);
            r[4] = n as u32;
            r[5..5 + n].copy_from_slice(&l[..n]);
        }
        // FUN_001e1238 / 12e0 / 1488: position, rotation in degrees -> radians (+0x20), +0x30, +0x40, +0x44
        682 | 683 | 689 => {
            let v = |i: usize| args.get(i).cloned().unwrap_or_default();
            for (j, x) in v(0).iter().take(3).enumerate() {
                r[4 + j] = *x;
            }
            for (j, x) in v(1).iter().take(3).enumerate() {
                r[8 + j] = (f32::from_bits(*x) * 0.017453294).to_bits();
            }
            r[12] = a(2);
            if p.field != 689 {
                r[16] = a(3);
                r[17] = a(4);
            }
        }
        // FUN_001e1870: +0x10 a4, +0x14 a0, +0x18 a2, +0x1c a1, +0x20 a3
        705 => {
            r[4] = a(4);
            r[5] = a(0);
            r[6] = a(2);
            r[7] = a(1);
            r[8] = a(3);
        }
        _ => {
            let skip = if let B::A(i) = b { Some(i) } else { None };
            let mut w = 4;
            for (i, x) in args.iter().enumerate() {
                if Some(i) == skip {
                    continue;
                }
                for v in x {
                    if w < 20 {
                        r[w] = *v;
                        w += 1;
                    }
                }
            }
            match p.field {
                // FUN_001e05a0: a4 != 0 -> +4 = 1
                638 if a(4) != 0 => r[1] = 1,
                // FUN_001e0b90 / 0c08: +0x24 = 1 / 0
                661 => r[9] = 1,
                662 => r[9] = 0,
                // FUN_001e1560: -1 -> 0x40
                692 if a(0) as i32 == -1 => r[4] = 0x40,
                _ => {}
            }
        }
    }
}

/// the matching close record of the block opened at `rec` (FUN_001e5ae0's scan)
fn block_end(c: &C2d8c48, rec: usize) -> usize {
    let mut depth = 1;
    for (k, r) in c.f_54.iter().enumerate().skip(rec + 1) {
        match r[0] {
            0 => depth += 1,
            1 => {
                depth -= 1;
                if depth == 0 {
                    return k;
                }
            }
            _ => {}
        }
    }
    rec
}

fn ix(c: &C2d8c48, n: u32) -> Option<usize> {
    c.k22_nodes.iter().position(|x| x.handle == n)
}

/// the node state (a script-started root node gets it here)
fn state(c: &mut C2d8c48, obj: usize, n: u32, h: &mut dyn ClassHost) -> Option<usize> {
    if let Some(k) = ix(c, n) {
        return Some(k);
    }
    if n & ROOT_HANDLE == 0 || !c.f_5c.contains(&n) {
        return None;
    }
    let w4 = c.f_54.first().map(|r| r[4] as i32).unwrap_or(-1);
    let target = if w4 != -1 { h.ee_sub_15e938(5, w4, 0) } else { 0 };
    let last = block_end(c, 0);
    c.k22_nodes.push(Node { handle: n, ctx: obj as u32 + 1, target, first: 0, last, ..Default::default() });
    Some(c.k22_nodes.len() - 1)
}

/// FUN_001e5ae0(owner, rec, parent): a node over the block at `rec`, linked on +0x5c
pub fn ee_sub_1e5ae0(c: &mut C2d8c48, obj: usize, rec: usize, parent_ctx: Option<u32>, h: &mut dyn ClassHost) -> u32 {
    let w4 = c.f_54.get(rec).map(|r| r[4] as i32).unwrap_or(-1);
    let target = if w4 != -1 { h.ee_sub_15e938(5, w4, 0) } else { 0 };
    let last = block_end(c, rec);
    c.k22_next += 1;
    let handle = SUB_HANDLE | c.k22_next;
    let ctx = parent_ctx.unwrap_or(obj as u32 + 1);
    c.k22_nodes.push(Node { handle, ctx, target, first: rec, last, ..Default::default() });
    c.f_5c.push(handle);
    handle
}

/// FUN_001e5778: the node is no longer linked
pub fn ee_sub_1e5778(c: &C2d8c48, n: u32) -> bool {
    !c.f_5c.contains(&n)
}

/// FUN_001e5628: unlink and drop the node (the resource releases FUN_001e5370..1e50e0 are unported)
pub fn ee_sub_1e5628(c: &mut C2d8c48, n: u32) {
    if ee_sub_1e5778(c, n) {
        return;
    }
    // K27: FUN_001e50e0(node, 0, 0): release the action-0x12 world state (run by the host after the update)
    if let Some(k) = ix(c, n).filter(|&k| c.k22_nodes[k].f_38 != 0) {
        c.k27_world.push(c.k22_nodes[k].f_3c);
    }
    c.f_5c.retain(|&x| x != n);
    c.k22_nodes.retain(|x| x.handle != n);
}

/// FUN_001e0180(owner, oid, skip): the first linked node (not `skip`) whose block record +0x10 is `oid`
pub fn ee_sub_1e0180(c: &C2d8c48, oid: u32, skip: u32) -> u32 {
    for &n in &c.f_5c {
        if n == skip {
            continue;
        }
        let first = ix(c, n).map(|k| c.k22_nodes[k].first).unwrap_or(0);
        if c.f_54.get(first).map(|r| r[4]) == Some(oid) {
            return n;
        }
    }
    0
}

/// FUN_001e1db8(node, oid): FUN_0015e938(4, oid, ctx) (the class-6 FUN_001cb2e0 redirect is not modelled)
fn ee_sub_1e1db8(ctx: u32, oid: u32, h: &mut dyn ClassHost) -> u32 {
    if oid as i32 == -1 {
        0
    } else {
        h.ee_sub_15e938(4, oid as i32, ctx)
    }
}

/// FUN_001e3890(node, rec): the wait condition of `rec`
pub fn ee_sub_1e3890(c: &mut C2d8c48, k: usize, rec: usize, now: f32, h: &mut dyn ClassHost) -> bool {
    let r = c.f_54[rec];
    let n = c.k22_nodes[k].clone();
    match r[0] {
        3 => (0..(r[4] as usize).min(4)).all(|i| ee_sub_1e0180(c, r[5 + i], n.handle) == 0),
        4 => {
            let on = c.k22_sig.contains(&(r[4] as i32));
            if on && r[5] != 0 {
                ee_sub_1e0070(c, r[4] as i32);
            }
            on
        }
        0x44 => !h.k22_title_active_1d9880(),
        // K23: FUN_001fe848(0x3dd040, id) == 0
        0x45 => !h.k23_stream_1fe848(r[4] as i32),
        0x4a | 0x4e => f32::from_bits(r[4]) <= h.k22_music_1fe3f8(),
        0x53 => f32::from_bits(r[5]) < now - n.f_34,
        // K30: the zoom look's getters (as of the last camera apply)
        0x3d | 0x3e | 0x40 => h.k30_zoom_wait(r[0]),
        // K27: +0x60 = the fading object: 0x2c done once it is in the world (+0x28 bit 60) with alpha
        // (+0x12c) 1, 0x2d once it left the world
        0x2c | 0x2d => {
            let o = n.f_60;
            if o != 0 {
                let inw = h.lo_28_bit60(o);
                if if r[0] == 0x2c { inw && h.obj_f32(o, 0x12c) == 1.0 } else { !inw } {
                    c.k22_nodes[k].f_60 = 0;
                }
            }
            c.k22_nodes[k].f_60 == 0
        }
        // K27: shake / rumble run for DAT_002bd850 (camera::cm::SHAKE_DURATION) / the record's duration
        0x32 => crate::engine::camera::cm::SHAKE_DURATION < now - n.f_34,
        0x33 => f32::from_bits(r[4]) < now - n.f_34,
        // K27: done once the guard no longer runs the routine (+0xd8 / +0xdc cleared)
        0x55 => {
            if n.f_dc != 0 && !ee_sub_171da8(c, n.f_d8, n.f_dc) {
                c.k22_nodes[k].f_d8 = 0;
                c.k22_nodes[k].f_dc = 0;
            }
            c.k22_nodes[k].f_dc == 0
        }
        // K28: owner-clock timer (LAB_001e42ac, as 0x33)
        0x23 => f32::from_bits(r[4]) < now - n.f_34,
        // K28: the camera move's end time (+0x80) passed -> FUN_001e5a98(node, rec[0x11]) / +0x9c = 0
        0x39 => {
            let done = n.f_80 <= now;
            if done {
                ee_sub_1e5a98_or_9c(c, k, &r, h);
            }
            done
        }
        _ => true,
    }
}

/// K28: FUN_001e4530(node, rec): the duration of a record (0 for a "don't wait" one). Kinds 10 (FUN_00154c90
/// curve), 0xd (FUN_001f11a8 anim length) and 0x45 (FUN_001fcfd0 stream length) are UNPORTED (0).
pub fn ee_sub_1e4530(r: &Rec) -> f32 {
    if r[1] != 0 {
        return 0.0;
    }
    match r[0] {
        0x23 | 0x33 => f32::from_bits(r[4]),
        0x2c | 0x2d | 0x53 => f32::from_bits(r[5]),
        0x32 => crate::engine::camera::cm::SHAKE_DURATION,
        _ => 0.0,
    }
}

/// K28: FUN_001e57a0(duration, node, rec 0x38, rec 0x39): the move's node state; an explicit duration makes
/// the 0x39 record wait (+4 = 0) for the end time. K29: the 0x39 key's rotation is wrapped to the nearest
/// turn of the 0x38 key's (FUN_0011cf60, in the record); the unset-key defaults (DAT_002ddfd0 /
/// FUN_0011bff0(0x2ddfa0) / DAT_002ddf60) and the policy (0x2de440 on g_cm: FUN_0014fd50 / 0014e4c8 /
/// 0014e328) are applied on the CM by classes/k29_cam.rs from the host requests.
fn ee_sub_1e57a0(c: &mut C2d8c48, k: usize, dur: f32, a: usize, b: usize, now: f32, h: &mut dyn ClassHost) {
    let mut end = f32::MAX; // DAT_002a8124
    if f32::from_bits(c.f_54[a][0x10]) != 0.0 {
        c.f_54[b][1] = 0;
        end = now + dur;
    }
    let nd = &mut c.k22_nodes[k];
    nd.f_80 = end;
    nd.f_84 = a;
    nd.f_88 = b;
    nd.f_8c = if dur == 0.0 { f32::MAX } else { dur };
    nd.f_90 = 1;
    nd.f_94 = c.f_54[a][0x11];
    nd.f_9c = 1;
    // pv = node + 0x84: the node handle
    let pv = nd.handle;
    for i in 0..3 {
        let (ra, rb) = (f32::from_bits(c.f_54[a][8 + i]), f32::from_bits(c.f_54[b][8 + i]));
        c.f_54[b][8 + i] = (ra + crate::engine::camera::math::FUN_0011cf60(rb - ra)).to_bits();
    }
    let mv = super::k29_cam::CmMove {
        a: super::k29_cam::MoveKey::from_rec(&c.f_54[a]),
        b: super::k29_cam::MoveKey::from_rec(&c.f_54[b]),
        duration: if dur == 0.0 { f32::MAX } else { dur },
        cut: 1,
        curve: c.f_54[a][0x11],
    };
    h.k29_cm_move(pv, mv);
    // FUN_0014fd50(0x2de440) -> FUN_0014e4c8(g_cm, 2, 10, 0x2de440, 0, DAT_002de450)
    h.k29_cm_revoke_if_active(2, 10, 0x2de440, 0, pv);
    h.ee_sub_14e328(10, 0x2de440, 0, pv);
}

/// K28: rec[0x10] != 0 -> FUN_001e5a98(node, rec[0x11]) (+0x90, +0x9c = 0, FUN_0014e4c8(g_cm, 2, 10,
/// 0x2de440, 0, node + 0x84)), else +0x9c = 0
fn ee_sub_1e5a98_or_9c(c: &mut C2d8c48, k: usize, r: &Rec, h: &mut dyn ClassHost) {
    c.k22_nodes[k].f_9c = 0;
    if r[0x10] != 0 {
        c.k22_nodes[k].f_90 = r[0x11];
        h.k29_cm_move_cut(c.k22_nodes[k].handle, r[0x11]);
        h.ee_sub_14e4c8_k(2, 10, 0x2de440, 0, c.k22_nodes[k].handle);
    }
}

/// FUN_001e0020(owner, oid): add a signal (8 kept)
pub fn ee_sub_1e0020(c: &mut C2d8c48, s: i32) {
    if !c.k22_sig.contains(&s) && c.k22_sig.len() < 8 {
        c.k22_sig.push(s);
    }
}

/// FUN_001e0070(owner, oid): drop a signal
pub fn ee_sub_1e0070(c: &mut C2d8c48, s: i32) {
    if let Some(i) = c.k22_sig.iter().position(|&x| x == s) {
        c.k22_sig.remove(i);
    }
}

/// FUN_001e1c40(node): run records until one waits or the block ends
pub fn ee_sub_1e1c40(c: &mut C2d8c48, obj: usize, n: u32, now: f32, h: &mut dyn ClassHost) {
    let Some(k) = state(c, obj, n, h) else { return };
    if c.k22_nodes[k].cur.is_none() {
        // +0x30 = +0x28, straight into the action
        c.k22_nodes[k].cur = Some(c.k22_nodes[k].first);
        if ee_sub_1e1e30_step(c, obj, n, now, h) {
            return;
        }
    }
    ee_sub_1e1c40_loop(c, obj, n, now, h);
}

/// FUN_001e1c40's loop: the current record's wait (FUN_001e1d48 has no +0x40 resource to release here),
/// past the end -> FUN_001e5628
fn ee_sub_1e1c40_loop(c: &mut C2d8c48, obj: usize, n: u32, now: f32, h: &mut dyn ClassHost) {
    loop {
        let Some(k) = ix(c, n) else { return };
        let (cur, last) = (c.k22_nodes[k].cur.unwrap_or(0), c.k22_nodes[k].last);
        if last < cur || (c.f_54[cur][1] == 0 && !ee_sub_1e3890(c, k, cur, now, h)) {
            if last < cur {
                ee_sub_1e5628(c, n);
            }
            return;
        }
        if ee_sub_1e1e30_step(c, obj, n, now, h) {
            return;
        }
    }
}

/// FUN_001e1e30 then FUN_001e5778: true = the node was unlinked
fn ee_sub_1e1e30_step(c: &mut C2d8c48, obj: usize, n: u32, now: f32, h: &mut dyn ClassHost) -> bool {
    ee_sub_1e1e30(c, obj, n, now, h);
    ee_sub_1e5778(c, n)
}

/// the record word holding the action's subject oid (FUN_001e1e30's first switch)
fn subject_word(kind: u32) -> Option<usize> {
    Some(match kind {
        0xb | 0x18 | 0x19 | 0x1d | 0x1f | 0x20 | 0x21 | 0x43 | 0x4b | 0x58 | 0x59 | 0x5a => 5,
        0x16 | 0x1a | 0x1c | 0x1e | 0x23 => 6,
        0x1b | 0x47 | 0x48 | 0x4f | 0x50 | 0x51 | 0x57 => 4,
        0x22 | 0x24 | 0x29 | 0x2a | 0x2b | 0x4a => 7,
        0x27 | 0x28 => 8,
        _ => return None,
    })
}

/// FUN_001e1e30(node): advance one record and start its action
pub fn ee_sub_1e1e30(c: &mut C2d8c48, obj: usize, n: u32, now: f32, h: &mut dyn ClassHost) {
    let Some(k) = ix(c, n) else { return };
    let cur = c.k22_nodes[k].cur.map_or(0, |x| x + 1);
    c.k22_nodes[k].cur = Some(cur);
    if c.k22_nodes[k].last < cur {
        return;
    }
    let r = c.f_54[cur];
    let nd = c.k22_nodes[k].clone();
    let subj = match subject_word(r[0]) {
        Some(w) => ee_sub_1e1db8(nd.ctx, r[w], h),
        None if nd.f_20 != 0 => nd.f_20,
        None if nd.target != 0 => nd.target,
        None => c.f_6c as u32,
    };
    c.k22_nodes[k].f_24 = subj;
    let me = obj as u32 + 1;
    match r[0] {
        0 => {
            let s = ee_sub_1e5ae0(c, obj, cur, Some(nd.ctx), h);
            let last = ix(c, s).map(|j| c.k22_nodes[j].last).unwrap_or(cur);
            if let Some(k) = ix(c, n) {
                c.k22_nodes[k].cur = Some(last);
            }
            ee_sub_1e1c40(c, obj, s, now, h);
        }
        1 => {}
        2 => {
            let t = if r[4] as i32 != -1 { ee_sub_1e0180(c, r[4], 0) } else { n };
            if t != 0 {
                ee_sub_1e5628(c, t);
            }
        }
        // 5 / 6 on another sequence object (FUN_00132b10(.., 0x72)) are unported
        5 if r[5] as i32 == -1 => ee_sub_1e0020(c, r[4] as i32),
        6 if r[4] as i32 == -1 => c.ee_sub_1dfeb8(obj, h),
        // (po vtable +0x27c)(po, 1): slot 78, the controlled character switch (classes derived from 0x32)
        7 => {
            let po = ee_sub_1e1db8(nd.ctx, r[4], h);
            if po != 0 && h.ee_sub_132b10(po, 0x32) != 0 {
                h.k22_po_switch(po);
            }
        }
        8 => {
            let s = ee_sub_1e1db8(nd.ctx, r[4], h);
            if let Some(k) = ix(c, n) {
                c.k22_nodes[k].f_20 = s;
            }
        }
        0x44 => h.k22_title_show_1dd820(),
        // K27: FUN_00171cf8(guard, routine); r[1] == 0 (wait) keeps both at +0xd8 / +0xdc
        0x55 => {
            let g = ee_sub_1e1db8(nd.ctx, r[4], h);
            let ro = ee_sub_1e1db8(nd.ctx, r[5], h);
            ee_sub_171cf8(c, g, ro);
            if r[1] == 0 {
                if let Some(k) = ix(c, n) {
                    c.k22_nodes[k].f_d8 = g;
                    c.k22_nodes[k].f_dc = ro;
                }
            }
        }
        // K27: owner +0x68 = node; owner vtable +0x1c (FUN_001a61e0 for class 114) (owner, 4, 0, 1, &rec[4], 0)
        // -> FUN_001331f8: the owner's level-script message 4 with the record's argument block (queued by the
        // host: `ClassHost::k27_script_msg`); +0x68 = 0 after
        // K28: count 1 (the original's param_4): the argument block is the one word rec[4]
        9 => h.k27_script_msg(me, 4, &r[4..5]),
        // K27: FUN_00161f48(0x2deb30, 1, &rec[4], ctx)
        0x11 => h.ee_sub_161f48_list(nd.ctx.saturating_sub(1) as usize, &[[r[4], r[5]]]),
        // K27: FUN_001e50e0(node, rec[4], rec[5])
        0x12 => ee_sub_1e50e0(c, n, r[4], r[5], h),
        // K27: FUN_00210338(rec[5] fade, 0x3dd490, on, rec[4] oid, ctx, owner); a waiting fade of an ALO
        // keeps it at +0x60
        0x2c | 0x2d => {
            let o = ee_sub_210338(f32::from_bits(r[5]), r[0] == 0x2c, r[4], nd.ctx, h);
            if o != 0 && r[1] == 0 && h.ee_sub_132b10(o, 0x30) != 0 {
                if let Some(k) = ix(c, n) {
                    c.k22_nodes[k].f_60 = o;
                }
            }
        }
        // K27: FUN_0014f750(g_cm, rec[4]) / FUN_00187e28(rec[4] duration, g_pad, rec[5])
        0x32 => h.k27_shake_14f750(r[4] as i32),
        0x33 => h.k27_rumble_187e28(f32::from_bits(r[4]), r[5] as i32),
        // K27: FUN_00171d58(guard)
        0x56 => {
            let g = ee_sub_1e1db8(nd.ctx, r[4], h);
            ee_sub_171d58(c, g);
        }
        // K23: +0x18 context = the oid's object, else the owner
        0xc => {
            let o = ee_sub_1e1db8(nd.ctx, r[4], h);
            if let Some(k) = ix(c, n) {
                c.k22_nodes[k].ctx = if o == 0 { me } else { o };
            }
        }
        // K23: FUN_0014d1d0(v, 0x2dded0, 6); owner +0x40 bit 0x100 set when v == 0 (its time runs on the other clock)
        0x10 => {
            let v = f32::from_bits(r[4]);
            h.k23_clock_14d1d0(v, 6);
            if v == 0.0 {
                c.f_40 |= 0x100;
            } else {
                c.f_40 &= !0x100;
            }
        }
        0x25 | 0x26 => {
            if let Some(k) = ix(c, n) {
                c.k22_nodes[k].f_e0 = (r[0] == 0x25) as i32;
            }
        }
        // K23: FUN_001fe5f0(0x3dd040, id, k); +0x68 = the record
        0x45 => {
            h.ee_sub_1fe5f0(r[4] as i32, r[5] as i32);
            if let Some(k) = ix(c, n) {
                c.k22_nodes[k].f_68 = Some(cur);
            }
        }
        // K28: FUN_001e1db8(rec[5]) and the subject of class 0x34 -> FUN_001e4eb8(node, rec, subject)
        // (+0x40 / +0x44; FUN_00143de8 on the old / new subject is UNPORTED); the wait is the timer rec[4]
        0x23 => {
            if ee_sub_1e1db8(nd.ctx, r[5], h) != 0 && subj != 0 && h.ee_sub_132b10(subj, 0x34) != 0 {
                if let Some(k) = ix(c, n) {
                    c.k22_nodes[k].f_40 = Some(cur);
                    c.k22_nodes[k].f_44 = subj;
                }
            }
        }
        // K28: camera move to the next 0x39 of this block (depth 0); duration rec[0x10], else the sum of the
        // FUN_001e4530 durations of the depth-0 records between
        0x38 => {
            let d0 = f32::from_bits(r[0x10]);
            let mut dur = if d0 != 0.0 { d0 } else { 0.0 };
            let (mut depth, mut end) = (0i32, None);
            for j in cur + 1..nd.last.min(c.f_54.len()) {
                match c.f_54[j][0] {
                    1 => depth -= 1,
                    0 => depth += 1,
                    0x39 if depth == 0 => {
                        end = Some(j);
                        break;
                    }
                    0x39 => {}
                    _ if depth == 0 && d0 == 0.0 => dur += ee_sub_1e4530(&c.f_54[j]),
                    _ => {}
                }
            }
            if let (Some(j), Some(k)) = (end, ix(c, n)) {
                ee_sub_1e57a0(c, k, dur, cur, j, now, h);
            }
        }
        // K28: a move without end time (+0x80 == FLT_MAX, DAT_002a7ad0) ends here
        0x39 => {
            if let Some(k) = ix(c, n) {
                if c.k22_nodes[k].f_80 == f32::MAX {
                    ee_sub_1e5a98_or_9c(c, k, &r, h);
                }
            }
        }
        // K30: the zoom look 0x2de460 setters (FUN_00150e20 / e60 centre / look-at object of rec[4], FUN_00150ea0
        // key rec+0x10 pos / +0x20 Euler / rec[0xc] fov, FUN_00150ee8 zoom rec[4], FUN_00150ef0 snap)
        0x3d | 0x3e => {
            let o = ee_sub_1e1db8(nd.ctx, r[4], h);
            h.k30_zoom(if r[0] == 0x3d { ZoomOp::Center(o) } else { ZoomOp::Target(o) });
        }
        0x3f => {
            let k = super::k29_cam::MoveKey::from_rec(&r);
            h.k30_zoom(ZoomOp::Key { pos: k.pos, rot: k.rot, fov: k.fov });
        }
        0x40 => h.k30_zoom(ZoomOp::Zoom(f32::from_bits(r[4]))),
        0x41 => h.k30_zoom(ZoomOp::Snap),
        _ => h.k22_seq_action(me, subj, &r),
    }
    // LAB_001e380c: the action start time
    if let Some(k) = ix(c, n) {
        c.k22_nodes[k].f_34 = now;
    }
}
