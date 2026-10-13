//! WP-D6: binding of ASEG channels to the bones they drive (FUN_0012f9b8 channel part,
//! FUN_0012fe58, FUN_00230018 / FUN_00230100 act lists) and the pose of every bound object.
//!
//! 1:1 parts:
//! - FUN_0012fe58(aseg, target, ch, &obj, &bone): obj = FUN_0015e938(world, 1, oid, target) when the
//!   ASEGA has a target, else FUN_0015e938(world, 4, oid, aseg) (nearest relative of the ASEG); oid =
//!   the channel word sign-extended from 20 bits. bone = obj vtable +0x19c(kind) where kind = the
//!   channel word bits 20..29 sign-extended (`(w << 2) >> 22`); +0x19c of every ALO class is
//!   FUN_0022d778 = FUN_0022d6b0(kind, obj +0x10 owner, obj) (gfx::attach::ee_sub_22d6b0). The other
//!   +0x19c implementations belong to non-ALO classes and return null (no bone). The channel `kind`
//!   is therefore a bone REFERENCE, not a matrix part: the matrix part is the bound bone's kind
//!   (+0x94), written by anim::actor::set_channel.
//! - FUN_0012f9b8 per channel without bit 30: if a bone resolves, an ACTSEG record (0x2c, vtable
//!   0x2caf08: +4 obj, +8 bone, +0xc priority = ASEG +0x6c, +0x14 motion mode 5 (7 when the start
//!   flags have bit 0), +0x1c vector curve or +0x20 scalar curve when bit 31, +0x24 ASEGA) is
//!   linked into the ASEGA (+0x34) and inserted into the bone's act list by FUN_00230018.
//! - FUN_00230018: the bone list (+0x90) is sorted by descending priority; the new act goes in
//!   front of the first act whose priority is <= its own (equal priority: newest wins). Only the top
//!   act drives the bone. FUN_00230100 unlinks an act; the next one takes over at once (ACTSEG has
//!   no fade: slot 0x131c40 activate / 0x131c90 deactivate only switch the motion mode).
//! - FUN_00130b18 (ASEGA retract) removes all acts of the ASEGA (FUN_00230100 each).
//! - FUN_0022dd00: the top act refetches its segment when the ASEGA time leaves the cached range;
//!   here the curve is evaluated at the ASEGA time directly (VecCurve::eval = segment + Hermite).
//! - Bone world = local * parent bone world (FUN_0022faa0), parents resolved like gfx::attach
//!   BoneWorlds (FUN_0022d6b0 + A6 template sockets); object world = bone 0 world.
//!
//! Not ported: the scalar-curve acts (bit 31) take their list slot but write no value (the scalar
//! bone motion modes are not decoded); the ASEG second list (FUN_00130010 -> target vtable +0x15c)
//! binds nothing in practice (FUN_001a6ee8 returns 0 for the 133 vtables that use it); FUN_001f8668
//! (obj, 0) wakes the object (no sleep state in the port); the motion mode velocities of mode 5/7.
//! The ASEG +0x6c priority is 0 unless a property sets it (FUN_0012f248 leaves it zeroed); callers
//! pass it.

use super::actor::set_channel;
use super::aseg::{Aseg, Channel};
use super::pose::Mat4;
use crate::engine::game::methods::find::ee_sub_15e958;
use crate::engine::gfx::attach::{affine_from_bytes, alo_bone_refs, alo_of, template_socket, BoneRef};
use crate::engine::world::alo::Alo;
use crate::engine::gfx::scene::{mul, Affine, IDENTITY};
use crate::engine::world::World;
use std::collections::{BTreeMap, HashMap, HashSet};

/// (world object index, bone index)
pub type BoneKey = (usize, usize);

/// Channel oid (20 bits, sign-extended as FUN_0012fe58 does).
pub fn channel_oid(ch: &Channel) -> i32 {
    ((ch.oid << 12) as i32) >> 12
}

/// Channel bone reference (10 bits at 20..29, sign-extended: `(w << 2) >> 22`).
pub fn channel_bone_ref(ch: &Channel) -> i16 {
    ((ch.kind << 6) as i16) >> 6
}

/// FUN_0012fe58 (0x0012fe58): (object, bone) driven by channel `ch` of the ASEG object `aseg_obj`
/// played on `target` (None = no target). obj None = no object with the oid; bone None = no bone.
pub fn ee_sub_12fe58(w: &World, aseg_obj: usize, target: Option<usize>, ch: &Channel) -> (Option<usize>, Option<BoneKey>) {
    let oid = channel_oid(ch);
    let o = match target {
        Some(t) => ee_sub_15e958(w, 1, oid, t as u32),
        None => ee_sub_15e958(w, 4, oid, aseg_obj as u32),
    };
    if o == 0 || o as usize >= w.objects.len() {
        return (None, None);
    }
    let o = o as usize;
    // vtable +0x19c: FUN_0022d778 for ALOs, null otherwise
    if alo_data(w, o).is_none() {
        return (Some(o), None);
    }
    (Some(o), ee_sub_22d778(w, o, channel_bone_ref(ch)))
}

/// ALO data of object `i`; FUN_0013e510 clones (`Payload::Clone`, vtable +0x144 copies the state)
/// share their source's bones.
pub fn alo_data(w: &World, i: usize) -> Option<&Alo> {
    let mut i = i;
    for _ in 0..16 {
        let o = w.objects.get(i)?;
        match o.clone_of {
            Some(src) if alo_of(w, i).is_none() => i = src,
            _ => return alo_of(w, i),
        }
    }
    None
}

/// FUN_0022d778 (ALO vtable +0x19c) = FUN_0022d6b0(r, alo +0x10 owner, alo) at run time (empty
/// load-context stack: 0x2000 refs are null). Clone-aware (bones counted from [`alo_data`]).
pub fn ee_sub_22d778(w: &World, alo: usize, r: i16) -> Option<BoneKey> {
    bone_ref(w, r, w.objects.get(alo)?.owner, Some(alo))
}

fn bone_ref(w: &World, r: i16, owner: Option<usize>, this: Option<usize>) -> Option<BoneKey> {
    let p = r as i32 as u32;
    let (o, k) = match p & 0x3000 {
        0x1000 => (owner?, p & 0xffff0fff),
        0 => (this?, p),
        _ => return None,
    };
    ((k as usize) < alo_data(w, o)?.base.bones.len()).then_some((o, k as usize))
}

/// One ACTSEG record (FUN_0012f9b8).
#[derive(Clone, Debug, PartialEq)]
pub struct Act {
    /// +0x24: the ASEGA (caller's handle)
    pub asega: u64,
    /// channel index in the ASEG (+0x1c / +0x20 curve)
    pub channel: usize,
    /// +0xc: ASEG +0x6c
    pub prio: i32,
    /// +0x14: 5, or 7 when the start flags have bit 0
    pub mode: u16,
    /// bit 31: scalar curve (+0x20)
    pub scalar: bool,
}

/// Bone act lists (+0x90 of every bone), highest priority first.
#[derive(Clone, Debug, Default)]
pub struct BoneActs {
    pub lists: HashMap<BoneKey, Vec<Act>>,
}

impl BoneActs {
    /// FUN_00230018 (0x00230018): insert; true when `act` became the top (driving) act.
    pub fn ee_sub_230018(&mut self, bone: BoneKey, act: Act) -> bool {
        let l = self.lists.entry(bone).or_default();
        let pos = l.iter().position(|a| a.prio <= act.prio).unwrap_or(l.len());
        l.insert(pos, act);
        pos == 0
    }

    /// FUN_00230100 (0x00230100): unlink the act of (`asega`, `channel`); true when the top changed.
    pub fn ee_sub_230100(&mut self, bone: BoneKey, asega: u64, channel: usize) -> bool {
        let Some(l) = self.lists.get_mut(&bone) else { return false };
        let Some(pos) = l.iter().position(|a| a.asega == asega && a.channel == channel) else { return false };
        l.remove(pos);
        if l.is_empty() {
            self.lists.remove(&bone);
        }
        pos == 0
    }

    /// The act driving `bone` (list head).
    pub fn top(&self, bone: BoneKey) -> Option<&Act> {
        self.lists.get(&bone).and_then(|l| l.first())
    }
}

/// Channel targets of one started ASEGA.
#[derive(Clone, Debug)]
pub struct BoundAsega {
    pub aseg_obj: usize,
    pub target: Option<usize>,
    /// per channel: object found / bone bound (None for bit-30 channels)
    pub objs: Vec<Option<usize>>,
    pub bones: Vec<Option<BoneKey>>,
}

/// Parent of a bone: parent bone and the matrix between (A6 template socket +0x12c, or the class
/// placement of a root bone).
#[derive(Clone, Copy, Debug)]
struct Parent {
    bone: Option<BoneKey>,
    socket: Option<Affine>,
}

/// Bone topology of a world (parents + children + rest worlds), built once per world.
#[derive(Default)]
struct Topo {
    world: (usize, usize),
    parent: HashMap<BoneKey, Parent>,
    children: HashMap<BoneKey, Vec<BoneKey>>,
    rest: HashMap<BoneKey, Affine>,
}

impl Topo {
    /// Parents as gfx::attach BoneWorlds resolves them (FUN_0022d6b0 refs + A6 sockets; clones via
    /// [`alo_data`]); a root bone of an object placed by its class (Lifecycle::placed, FUN_001ca370
    /// +0x12c) hangs off that placement (as classes' k24_world_xf composes bone 0 with it).
    fn build(w: &World) -> Topo {
        let mut t = Topo { world: (w as *const World as usize, w.objects.len()), ..Default::default() };
        for o in 0..w.objects.len() {
            let Some(a) = alo_data(w, o) else { continue };
            let clone = alo_of(w, o).is_none();
            let refs = if clone { vec![] } else { alo_bone_refs(w, o) };
            let mut socket = None;
            for (k, b) in a.base.bones.iter().enumerate() {
                let r = if clone {
                    match bone_ref(w, b.r#ref, w.objects[o].owner, Some(o)) {
                        Some(x) => BoneRef::Bone(x.0, x.1),
                        None => BoneRef::None,
                    }
                } else {
                    refs.get(k).map(|r| r.0).unwrap_or(BoneRef::None)
                };
                let p = match r {
                    BoneRef::Bone(po, pk) if (po, pk) != (o, k) => Parent { bone: Some((po, pk)), socket: None },
                    _ => {
                        let s = *socket.get_or_insert_with(|| template_socket(w, o));
                        match s {
                            Some((BoneRef::Bone(po, pk), pm)) => Parent { bone: Some((po, pk)), socket: Some(pm) },
                            _ => Parent { bone: None, socket: w.life.placed.get(&o).map(affine_from_bytes) },
                        }
                    }
                };
                if let Some(pb) = p.bone {
                    t.children.entry(pb).or_default().push((o, k));
                }
                t.parent.insert((o, k), p);
            }
        }
        t
    }

    /// Rest world of a bone (FUN_0022faa0 with the rest locals).
    fn rest_world(&mut self, w: &World, k: BoneKey, depth: usize) -> Affine {
        if let Some(m) = self.rest.get(&k) {
            return *m;
        }
        let local = alo_data(w, k.0).and_then(|a| a.base.bones.get(k.1)).and_then(|b| b.xform).unwrap_or(IDENTITY);
        let m = self.place(w, k, local, depth, &mut |t, w, pb, d| t.rest_world(w, pb, d));
        self.rest.insert(k, m);
        m
    }

    fn place(&mut self, w: &World, k: BoneKey, local: Affine, depth: usize, parent_world: &mut dyn FnMut(&mut Topo, &World, BoneKey, usize) -> Affine) -> Affine {
        let p = self.parent.get(&k).copied().unwrap_or(Parent { bone: None, socket: None });
        let local = match p.socket {
            Some(pm) => mul(&local, &pm),
            None => local,
        };
        match p.bone {
            Some(pb) if depth < 256 => mul(&local, &parent_world(self, w, pb, depth + 1)),
            _ => local,
        }
    }
}

/// The pose of one frame: every bone moved by an act (directly or through its parents).
#[derive(Clone, Debug, Default)]
pub struct AnimPose {
    /// bones driven by a top act -> posed local
    pub locals: HashMap<BoneKey, Affine>,
    /// world matrix of every affected bone (driven bones and their descendants)
    pub bones: HashMap<BoneKey, Affine>,
    /// objects with an affected bone 0 -> object world (bone 0 world)
    pub objects: HashMap<usize, Affine>,
    /// objects with any affected bone
    pub posed: HashSet<usize>,
}

impl AnimPose {
    /// World of `key`: posed when affected, else `rest` (e.g. gfx::attach BoneWorlds::get).
    pub fn bone_world(&self, key: BoneKey, rest: &mut dyn FnMut(BoneKey) -> Affine) -> Affine {
        self.bones.get(&key).copied().unwrap_or_else(|| rest(key))
    }
}

fn to4(m: &Affine) -> Mat4 {
    [0, 1, 2, 3].map(|r| [m[r][0], m[r][1], m[r][2], (r == 3) as u32 as f32])
}

fn to3(m: &Mat4) -> Affine {
    [0, 1, 2, 3].map(|r| [m[r][0], m[r][1], m[r][2]])
}

/// Run-time ASEG -> bone binding of a world: the started ASEGAs and the bone act lists.
/// classes/ owns the ASEGAs (K25 AnimRt); it reports starts / retracts here (or calls [`AnimBinder::sync`]
/// each frame) and reads [`AnimBinder::pose`].
#[derive(Default)]
pub struct AnimBinder {
    pub acts: BoneActs,
    pub asegas: BTreeMap<u64, BoundAsega>,
    topo: Topo,
}

impl AnimBinder {
    /// FUN_0012f9b8 (0x0012f9b8) channel part: bind every channel of `aseg` (object `aseg_obj`)
    /// started as ASEGA `handle` on `target` with start `flags` and priority `prio` (ASEG +0x6c).
    /// Returns the number of acts created. A handle started twice is retracted first.
    pub fn start(&mut self, w: &World, handle: u64, aseg_obj: usize, aseg: &Aseg, target: Option<usize>, flags: u32, prio: i32) -> usize {
        self.retract(handle);
        let mode = if flags & 1 != 0 { 7 } else { 5 };
        let mut b = BoundAsega { aseg_obj, target, objs: vec![], bones: vec![] };
        let mut n = 0;
        for (c, ch) in aseg.channels.iter().enumerate() {
            // bit 30 channels are skipped
            let (o, bone) = if ch.flag { (None, None) } else { ee_sub_12fe58(w, aseg_obj, target, ch) };
            if let Some(bone) = bone {
                self.acts.ee_sub_230018(bone, Act { asega: handle, channel: c, prio, mode, scalar: ch.vec.is_none() && ch.scalar.is_some() });
                n += 1;
            }
            b.objs.push(o);
            b.bones.push(bone);
        }
        self.asegas.insert(handle, b);
        n
    }

    /// FUN_00130b18 (0x00130b18) act part: remove every act of ASEGA `handle`.
    pub fn retract(&mut self, handle: u64) {
        let Some(b) = self.asegas.remove(&handle) else { return };
        for (c, bone) in b.bones.iter().enumerate() {
            if let Some(bone) = bone {
                self.acts.ee_sub_230100(*bone, handle, c);
            }
        }
    }

    /// Start / retract to match the live ASEGA set: `live` = (handle, ASEG object, target).
    /// New handles are started in handle order (flags 0, priority 0); missing ones retracted.
    pub fn sync(&mut self, w: &World, live: &[(u64, usize, Option<usize>)]) {
        let set: HashSet<u64> = live.iter().map(|l| l.0).collect();
        let gone: Vec<u64> = self.asegas.keys().filter(|h| !set.contains(h)).copied().collect();
        for h in gone {
            self.retract(h);
        }
        let mut new: Vec<&(u64, usize, Option<usize>)> = live.iter().filter(|l| !self.asegas.contains_key(&l.0)).collect();
        new.sort_by_key(|l| l.0);
        for &(h, ai, t) in new {
            let Some(a) = crate::engine::playerdraw::aseg_of(w, ai) else { continue };
            self.start(w, h, ai, a, t, 0, 0);
        }
    }

    /// Pose of this frame. `time(handle)` = (ASEGA time +0x1c, speed +0x40) of a live ASEGA.
    pub fn pose(&mut self, w: &World, time: &dyn Fn(u64) -> Option<(f32, f32)>) -> AnimPose {
        let l = self.locals(w, time);
        self.compose(w, l)
    }

    /// Posed locals of the bones whose top act has a vector curve (FUN_0022dd00 + set_channel).
    pub fn locals(&self, w: &World, time: &dyn Fn(u64) -> Option<(f32, f32)>) -> HashMap<BoneKey, Affine> {
        let mut out = AnimPose::default();
        for (&bone, l) in &self.acts.lists {
            let Some(act) = l.first() else { continue };
            if act.scalar {
                continue;
            }
            let Some(b) = self.asegas.get(&act.asega) else { continue };
            let Some((t, rate)) = time(act.asega) else { continue };
            let Some(a) = crate::engine::playerdraw::aseg_of(w, b.aseg_obj) else { continue };
            let Some(v) = a.channels.get(act.channel).and_then(|c| c.vec.as_ref()).and_then(|c| c.eval(t, rate)) else { continue };
            let Some(ab) = alo_data(w, bone.0).and_then(|x| x.base.bones.get(bone.1)) else { continue };
            let mut l4 = to4(&ab.xform.unwrap_or(IDENTITY));
            set_channel(ab.kind as i32, &mut l4, v);
            out.locals.insert(bone, to3(&l4));
        }
        out.locals
    }

    /// World pose from bone locals (driven bones; e.g. [`AnimBinder::locals`] after a cross-fade,
    /// anim::blends): every descendant bone and object follows (FUN_0022faa0).
    pub fn compose(&mut self, w: &World, locals: HashMap<BoneKey, Affine>) -> AnimPose {
        if self.topo.world != (w as *const World as usize, w.objects.len()) {
            self.topo = Topo::build(w);
        }
        let mut out = AnimPose { locals, ..Default::default() };
        // affected = driven bones + descendants
        let mut affected: HashSet<BoneKey> = HashSet::new();
        let mut st: Vec<BoneKey> = out.locals.keys().copied().collect();
        while let Some(k) = st.pop() {
            if affected.insert(k) {
                if let Some(c) = self.topo.children.get(&k) {
                    st.extend(c.iter().copied());
                }
            }
        }
        let mut keys: Vec<BoneKey> = affected.iter().copied().collect();
        keys.sort_unstable();
        for k in keys {
            solve(w, &mut self.topo, &out.locals, &affected, &mut out.bones, k, 0);
        }
        for &(o, k) in &affected {
            out.posed.insert(o);
            if k == 0 {
                out.objects.insert(o, out.bones[&(o, 0)]);
            }
        }
        out
    }
}

/// Posed world of an affected bone (FUN_0022faa0): local * parent world, unaffected parents at rest.
fn solve(w: &World, topo: &mut Topo, locals: &HashMap<BoneKey, Affine>, affected: &HashSet<BoneKey>, done: &mut HashMap<BoneKey, Affine>, k: BoneKey, depth: usize) -> Affine {
    if let Some(m) = done.get(&k) {
        return *m;
    }
    if !affected.contains(&k) || depth > 256 {
        return topo.rest_world(w, k, 0);
    }
    let local = match locals.get(&k) {
        Some(l) => *l,
        None => alo_data(w, k.0).and_then(|a| a.base.bones.get(k.1)).and_then(|b| b.xform).unwrap_or(IDENTITY),
    };
    let m = topo.place(w, k, local, depth, &mut |t, w, pb, d| solve(w, t, locals, affected, done, pb, d));
    done.insert(k, m);
    m
}
