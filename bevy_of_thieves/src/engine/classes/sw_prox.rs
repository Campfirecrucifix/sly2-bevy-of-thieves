//! ANIM-ALL R64: the world proximity set (0x3dd490 +0x20 list / +0x2c active set, FUN_00148300 / FUN_00147f40)
//! and the camera render mask DAT_002ddf54 (FUN_0014e078 / FUN_0014e148), wired to `game::world_update::ProxList`.
//! FUN_001a61a0(o) = +0x28 bit 60 && o's SW root (+0x14) is in the active set; the class-29 emitters' slot 15
//! (FUN_00159c80) only runs off the update list, which they join through vt+0xbc only while that holds.
//! Opt-out: SLY2_R64OFF (every not-removed emitter updates, as before R64).
use super::plugin::EngineHost;
use super::ClassHost;
use crate::engine::game::world_update::{ProxEntry, ProxHost, ProxList};
use crate::engine::world::objects::{LoadData, Payload};
use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct SwProx {
    /// world the state belongs to (Arc pointer)
    pub wkey: usize,
    pub list: ProxList,
    /// root +0x28 bits 52..55 set at run time (FUN_001a6ca8), by object index
    pub mode: HashMap<usize, i8>,
    /// DAT_002ddf54 (cm +0x14): 0xff from FUN_0014d6d8, then the containing VOL's +0xe0 (sticky)
    pub mask: u8,
    /// class-5 VOLs (object, compiled volume, +0xe0)
    pub vols: Option<Vec<(super::guard::sensor::Vol, u32)>>,
    /// R65: per VOL +0x110 (camera inside; -1 after init FUN_00227350) / +0x114 (time of the last +0x110 change,
    /// FUN_00227440; a frame counter keeps the order DAT_002dded0 gives) and the frame counter
    pub vst: Vec<(i32, u64)>,
    pub tick: u64,
    pub built: bool,
    pub dumped: bool,
    /// R66: run-time +0x2d (FUN_001a6d28 from the FUN_001caec8 claim) by object index, and the changes not yet
    /// applied to the ClassSet parts (`sw_take_masks`)
    pub rt: HashMap<usize, u8>,
    pub rt_new: Vec<(usize, u8)>,
}

/// R67: OPT-IN (SLY2_R64=1). COORD 18:58 / CRAWL R100: with R64 on, feel_inworld_live_saved_matches_original_ep1_hub's
/// hub guard spawner pick (FUN_00174490 FUN_0011d3f8 shuffle) gives 11539 copy 0 instead of 44949 copy 1. Measured R67:
/// that pick is a pure RNG-draw-order effect, not an R64 error: SLY2_RNG_SHARED=1 -> R64 on fails (125317 copy 1), off
/// passes; SLY2_RNG_LOADSEED=9f20088d (rec5 052 hub clk 0 rand) -> R64 on PASSES, off fails (44949 copy 1). Kept opt-in
/// until the test seeds the rand from its own savestate (or allows the pick as the EP3 test does). SLY2_R64OFF still wins.
pub fn off() -> bool {
    std::env::var_os("SLY2_R64OFF").is_some() || std::env::var_os("SLY2_R64").is_none()
}

struct Pos(HashMap<u32, [f32; 3]>);
impl ProxHost for Pos {
    fn pos(&self, obj: u32) -> [f32; 3] {
        self.0.get(&obj).copied().unwrap_or([0.0; 3])
    }
    fn enter(&mut self, _obj: u32) {}
    fn leave(&mut self, _obj: u32) {}
}

/// topmost owner (the SW root, `**(o + 0x14)`)
pub fn root(w: &crate::engine::world::World, mut i: usize) -> usize {
    while let Some(p) = w.objects.get(i).and_then(|o| o.owner) {
        i = p;
    }
    i
}

/// the load xform (FUN_00231768) of `i`, following a load-time copy (FUN_0013e510 copies the source's xf / +0x2d)
fn load_xf(w: &crate::engine::world::World, mut i: usize) -> Option<crate::engine::world::loads::Xform> {
    use crate::engine::world::loads_b4::B4Load as B;
    loop {
        let o = w.objects.get(i)?;
        match &o.payload {
            Payload::Clone => i = o.clone_of?,
            Payload::Loaded(LoadData::L231170 { s, xform, word, .. }) => return Some(crate::engine::world::loads::Xform { s: *s, xform: *xform, word: *word }),
            Payload::Loaded(LoadData::L227380 { xform, .. }) | Payload::Loaded(LoadData::L231170Blob { xform, .. }) => return Some(xform.clone()),
            Payload::Loaded(LoadData::B4(b)) => {
                return match &**b {
                    B::L185150 { xform, .. } | B::L1e5df8 { xform, .. } | B::L220fd0 { xform, .. } | B::L1c03c0 { xform, .. }
                    | B::L258598 { xform, .. } | B::L15c1e8 { xform, .. } | B::L231170 { xform, .. } => Some(xform.clone()),
                    _ => None,
                }
            }
            _ => return None,
        }
    }
}

/// +0x2d after load: FUN_00231768's xform word / the ALO word (FUN_001a6d28 -> vt+0x174), else 0xff (FUN_001a5270)
pub fn load_mask(w: &crate::engine::world::World, i: usize) -> u8 {
    if std::env::var_os("SLY2_R65COFF").is_some() {
        return place_mask(w, i).or_else(|| own_mask(w, i)).unwrap_or(0xff);
    }
    // R65: vt+0x174 FUN_001a6dc8 stores the word and runs FUN_001a6d28(child, word) down every child (+100 / +0x24,
    // children without +0x28 bit 58), so the load of an owner (which ends after its children's) overwrites the
    // subtree, and FUN_001c9b08's placement word (after all loads) overwrites again: the topmost word wins.
    let mut chain = vec![i];
    while let Some(p) = w.objects.get(*chain.last().unwrap()).and_then(|o| o.owner) {
        chain.push(p);
    }
    chain.iter().rev().find_map(|&k| place_mask(w, k)).or_else(|| chain.iter().rev().find_map(|&k| own_mask(w, k))).unwrap_or(0xff)
}

/// FUN_001c9b08: the in-place object / every FUN_001ca370 collector object (not +0x28 bit 58) gets
/// FUN_001a6d28(record word) after its load (text_001c0000.c 7424 / 7492). Opt-in SLY2_R64B=1 (shifts the class-29
/// update set and the GameRng order the guard tests depend on, like R60)
fn place_mask(w: &crate::engine::world::World, i: usize) -> Option<u8> {
    use crate::engine::world::loads_b4::B4Load as B;
    if std::env::var_os("SLY2_R64B").is_none() {
        return None;
    }
    let c6 = |r: usize| match &w.objects.get(r)?.payload {
        Payload::Loaded(LoadData::B4(b)) => match &**b {
            B::Class6 { word, .. } => Some(*word as u8),
            _ => None,
        },
        _ => None,
    };
    c6(i).or_else(|| w.template_instances.iter().rev().find(|t| t.objects.contains(&i)).and_then(|t| c6(t.record)))
}

/// the object's own load word (ALO / load xform), following load-time copies
fn own_mask(w: &crate::engine::world::World, i: usize) -> Option<u8> {
    let mut k = i;
    while let Some(o) = w.objects.get(k) {
        match (&o.payload, o.clone_of) {
            (Payload::Clone, Some(s)) => k = s,
            (Payload::Loaded(LoadData::Alo { alo, .. }), _) => return Some(alo.word as u8),
            _ => break,
        }
    }
    load_xf(w, i).map(|x| x.word as u8)
}

impl EngineHost {
    fn sw_reset_if_new(&mut self) -> Option<std::sync::Arc<crate::engine::world::World>> {
        let w = self.world.clone()?;
        let k = std::sync::Arc::as_ptr(&w) as usize;
        if self.sw.wkey != k {
            self.sw = SwProx { wkey: k, mask: 0xff, ..Default::default() };
        }
        Some(w)
    }

    /// root mode: run-time FUN_001a6ca8 value, else the init default (ALO init FUN_001f2108 -> 0, LO 2)
    fn sw_mode(&self, w: &crate::engine::world::World, r: usize) -> i8 {
        if let Some(&m) = self.sw.mode.get(&r) {
            return m;
        }
        let alo = w.objects.get(r).is_some_and(|o| crate::engine::world::class::ee_sub_15e988(o.class, 0x30));
        if alo { 0 } else { 2 }
    }

    fn sw_entry(&mut self, w: &crate::engine::world::World, r: usize) -> ProxEntry {
        let kind = self.sw_mode(w, r);
        // a load-time copy carries its source's +0x38 (FUN_0013e510 vt+0x144 -> FUN_001a6730 copies the LO block)
        let mut s = r;
        while let (crate::engine::world::objects::Payload::Clone, Some(c)) = (&w.objects[s].payload, w.objects[s].clone_of) {
            s = c;
        }
        let radius = self.obj_f32(s as u32 + 1, 0x38);
        ProxEntry { obj: r as u32 + 1, kind, mask: load_mask(w, r), radius }
    }

    /// FUN_001a6ca8(o, k) host side: a root in the world with a new mode is queued again (FUN_00147e88)
    pub fn sw_set_mode(&mut self, obj: usize, k: i32) {
        let Some(w) = self.sw_reset_if_new() else { return };
        let old = self.sw_mode(&w, obj);
        self.sw.mode.insert(obj, k as i8);
        if !self.sw.built || old == k as i8 || w.objects.get(obj).map_or(true, |o| o.owner.is_some()) || self.lo_removed(obj) {
            return;
        }
        let e = self.sw_entry(&w, obj);
        let l = &mut self.sw.list;
        l.list.retain(|x| x.obj != e.obj);
        // FUN_0011a280(+0x20, stop, node): before the stop node (at the tail when none)
        match l.stop.and_then(|s| l.list.iter().position(|x| x.obj == s)) {
            Some(p) => l.list.insert(p, e),
            None => l.list.push(e),
        }
    }

    /// once per frame before the class updates: FUN_0014e078's mask (class-5 VOLs containing the camera) and
    /// FUN_00147f40. Returns DAT_002ddf54.
    pub fn sw_frame(&mut self, cam: [f32; 3], focus: Option<[f32; 3]>) -> u8 {
        let Some(w) = self.sw_reset_if_new() else { return 0xff };
        if self.sw.vols.is_none() {
            use crate::engine::physics::geom::{build_bsp, build_geom};
            let mut bw = crate::engine::gfx::attach::BoneWorlds::default();
            let mut v = Vec::new();
            for (k, o) in w.objects.iter().enumerate() {
                if let (5, Payload::Loaded(LoadData::L227380 { word, geom, bsp, .. })) = (o.class, &o.payload) {
                    if let Some(mw) = bw.xfm_world(&w, k) {
                        let inv = crate::engine::gfx::scene::invert(&mw);
                        v.push((super::guard::sensor::Vol::new(k, inv, build_geom(geom), build_bsp(bsp)), *word));
                    }
                }
            }
            self.sw.vols = Some(v);
        }
        // FUN_0014e078: FUN_002273f0 on every VOL (FUN_0015e9c8: at most 0x20), sort FUN_0014e000 (+0x110 desc,
        // +0xe4 desc (0: no VOL sets it at load), +0x114 asc, address asc), FUN_0014e148 when the first has +0x110 == 1.
        // SLY2_R65OFF: the first containing VOL in load order (R64)
        let vols = self.sw.vols.as_ref().unwrap();
        if std::env::var_os("SLY2_R65OFF").is_some() {
            if let Some(m) = vols.iter().find(|(v, _)| super::guard::sensor::ee_sub_227c90(v, cam)).map(|x| x.1) {
                self.sw.mask = m as u8;
            }
        } else {
            let n = vols.len().min(0x20);
            self.sw.vst.resize(n, (-1, 0));
            self.sw.tick += 1;
            for k in 0..n {
                let inside = super::guard::sensor::ee_sub_227c90(&vols[k].0, cam) as i32;
                if self.sw.vst[k].0 != inside {
                    self.sw.vst[k] = (inside, self.sw.tick);
                }
            }
            let st = &self.sw.vst;
            if let Some(k) = (0..n).min_by(|&a, &b| st[b].0.cmp(&st[a].0).then(st[a].1.cmp(&st[b].1)).then(a.cmp(&b))) {
                if st[k].0 == 1 {
                    self.sw.mask = vols[k].1 as u8;
                }
            }
        }
        if !self.sw.built {
            // FUN_001a5478 at load: every root in the world with mode != 2 is queued (FUN_00147e88); the
            // immediate FUN_00148300 test is the first FUN_00147f40 pass
            self.sw.built = true;
            for r in 0..w.objects.len() {
                if w.objects[r].owner.is_none() && !self.lo_removed(r) && self.sw_mode(&w, r) != 2 {
                    let e = self.sw_entry(&w, r);
                    self.sw.list.list.push(e);
                }
            }
        }
        // ANIM-ALL R90: roots that enter the world after the load (FUN_001a5478 Add of a removed root / pool claim
        // -> FUN_00147e88 queue) join the list; removed ones leave it (FUN_001a5630 -> FUN_0011a2e8(0x3dd4bc) +
        // FUN_00147e20). PCSX2 EP2 002: guard 11539 (out at the port's load) is an in-world mode-0 root whose
        // child 32598 feeds group 34066 (balcony sparkles); the port's list never had it. Opt-out SLY2_R90_NO_SWSYNC.
        if std::env::var_os("SLY2_R90_NO_SWSYNC").is_none() {
            let listed: std::collections::HashSet<u32> = self.sw.list.list.iter().map(|e| e.obj).collect();
            let mut add = Vec::new();
            let mut drop = Vec::new();
            for r in 0..w.objects.len() {
                if w.objects[r].owner.is_some() {
                    continue;
                }
                let inw = !self.lo_removed(r) && self.sw_mode(&w, r) != 2;
                let l = listed.contains(&(r as u32 + 1));
                if inw && !l {
                    add.push(r);
                } else if !inw && l {
                    drop.push(r as u32 + 1);
                }
            }
            if !drop.is_empty() {
                let l = &mut self.sw.list;
                l.list.retain(|e| !drop.contains(&e.obj));
                l.active.retain(|o| !drop.contains(o));
                if l.stop.is_some_and(|s| drop.contains(&s)) {
                    l.stop = None;
                }
            }
            for r in add {
                let e = self.sw_entry(&w, r);
                let l = &mut self.sw.list;
                match l.stop.and_then(|s| l.list.iter().position(|x| x.obj == s)) {
                    Some(p) => l.list.insert(p, e),
                    None => l.list.push(e),
                }
            }
        }
        let objs: Vec<u32> = self.sw.list.list.iter().map(|e| e.obj).collect();
        let pos = Pos(objs.into_iter().map(|o| {
            let p = self.ee_sub_1a62e8(o);
            // roots the port's world-xf getter doesn't cover (B4 / copied payloads): the root's load xf translation
            let p = if p == [0.0; 3] { load_xf(&w, o as usize - 1).map_or(p, |x| crate::engine::gfx::attach::affine_from_bytes(&x.xform)[3]) } else { p };
            (o, p)
        }).collect());
        if std::env::var_os("SLY2_R64DBG2").is_some() && !self.sw.list.active.is_empty() && !self.sw.dumped {
            self.sw.dumped = true;
            for e in &self.sw.list.list {
                eprintln!("R64E oid {} kind {} mask {:#x} r {} pos {:?} act {}", w.objects[e.obj as usize - 1].oid, e.kind, e.mask, e.radius, pos.0.get(&e.obj), self.sw.list.active.contains(&e.obj));
            }
        }
        let mut ph = pos;
        let mask = self.sw.mask;
        self.sw.list.FUN_00147f40(&mut ph, mask as u32, cam, focus);
        if std::env::var_os("SLY2_R64DBG").is_some() {
            eprintln!("R64 mask {mask:#x} cam {cam:?} list {} active {}", self.sw.list.list.len(), self.sw.list.active.len());
        }
        mask
    }

    /// +0x2d now: the run-time value (R66) else the load value
    fn sw_mask_of(&self, w: &crate::engine::world::World, i: usize) -> u8 {
        self.sw.rt.get(&i).copied().unwrap_or_else(|| load_mask(w, i))
    }

    /// R66: FUN_001caec8's vt +0xd4 (slot 25, ALO FUN_001cb518) of the claimed copy with the spawner `sp`:
    /// FUN_001cb4e0 = FUN_001a6d28(o, sp +0x2d) when o's +0x28 bit 58 is clear, then slot 25 of every child with
    /// the same spawner -> the whole subtree takes the spawner's word; FUN_001a6d28 ORs it into every ancestor
    /// (the copy's parent after vt +0x54 = the record's owner) and re-queues a root in the world (FUN_00147e88).
    /// rec5 053 -> 054: claimed C2d4b78 129465 / C2d9978 13105 copies 255 -> 2 (spawner records all 2).
    /// Only with SLY2_R64B=1 (the spawner records' own +0x2d is their placement word, R64B; without it they read
    /// 0xff and the claim would spread 0xff: diverge rec5 053->054 sum |orig-port| of 2-counts per oid 208 -> 295,
    /// with R64B 226 -> 173). Opt-out SLY2_R66OFF.
    pub fn sw_claim_mask(&mut self, copy: usize, sp: u32) {
        if std::env::var_os("SLY2_R66OFF").is_some() || std::env::var_os("SLY2_R64B").is_none() {
            return;
        }
        let Some(w) = self.sw_reset_if_new() else { return };
        let Some(s) = (sp as usize).checked_sub(1).filter(|&i| i < w.objects.len()) else { return };
        if copy >= w.objects.len() {
            return;
        }
        let m = self.sw_mask_of(&w, s);
        let b58 = |h: &mut Self, o: usize| h.obj_i32(o as u32 + 1, 0x2c) as u32 >> 26 & 1 != 0;
        let mut set = Vec::new();
        for o in crate::engine::playerdraw::subtree(&w, copy) {
            if !b58(self, o) {
                set.push((o, m));
            }
        }
        // ancestors |= m: the run-time parent (vt +0x54) then the load tree
        let mut a = self.pool_parent.get(&copy).copied().or(w.objects[copy].owner);
        while let Some(p) = a {
            set.push((p, self.sw_mask_of(&w, p) | m));
            a = self.pool_parent.get(&p).copied().or(w.objects[p].owner);
        }
        for (o, v) in set {
            self.sw.rt.insert(o, v);
            self.sw.rt_new.push((o, v));
            if let Some(e) = self.sw.list.list.iter_mut().find(|e| e.obj == o as u32 + 1) {
                e.mask = v;
            }
        }
    }

    /// R66: the +0x2d changes since the last call (ClassSet::frame applies them to the C2d9978 parts)
    pub fn sw_take_masks(&mut self) -> Vec<(usize, u8)> {
        std::mem::take(&mut self.sw.rt_new)
    }

    /// FUN_001a61a0(o) for the update-list gate: bit 60 (not removed) and the SW root active
    pub fn sw_active(&mut self, obj: usize) -> bool {
        let Some(w) = self.sw_reset_if_new() else { return true };
        if !self.sw.built {
            return true;
        }
        // ANIM-ALL R90: run-time roots past the load world (FUN_00159500's FUN_0013e3c8 class-29 groups, made
        // with FUN_001a6ca8(o, 1)): FUN_00148300 puts a mode-1 root in the set unconditionally (PCSX2 EP2 002:
        // groups 34066/34067/17199/17893/105212/105213/119860 mode 1, in the world). Opt-out SLY2_R90_NO_RTROOT.
        if obj >= w.objects.len() && std::env::var_os("SLY2_R90_NO_RTROOT").is_none() {
            return self.sw.mode.get(&obj) == Some(&1);
        }
        let r = root(&w, obj) as u32 + 1;
        self.sw.list.active.contains(&r)
    }
}

/// R65: +0x2d of every object's C2d9978 part: FUN_001a5270 0xff, then the load word (FUN_00231768 xform word /
/// ALO FUN_001a6d28; `load_mask`). Before R65 the port left 0 (diverge rec5 001: orig 2 / port 0 for every class-48
/// instance), so slot 18 FUN_001f3c70 / class 55's FUN_001425b0 never passed the DAT_002ddf54 test. Opt-out SLY2_R65BOFF.
pub fn set_masks(s: &mut super::ClassSet, w: &crate::engine::world::World) {
    use super::Inst;
    if std::env::var_os("SLY2_R65BOFF").is_some() {
        return;
    }
    for (i, x) in s.insts.iter_mut().chain(s.k5.iter_mut()).chain(s.k16.iter_mut()).chain(s.k22c.iter_mut()) {
        let so = match x {
            Inst::C2d9978(c) => c,
            Inst::C2d4b78(c) => &mut c.alo.so,
            Inst::C2d4260(c) => &mut c.alo.so,
            Inst::C2caf88(c) => &mut c.alo.so,
            Inst::K16(c) => &mut c.alo.so,
            Inst::K22C96(c) => &mut c.alo.so,
            _ => continue,
        };
        so.f_2d = load_mask(w, *i);
    }
}

/// R66: apply run-time +0x2d changes (`EngineHost::sw_claim_mask`) to the parts `set_masks` initialised
pub fn apply_masks(s: &mut super::ClassSet, ch: &[(usize, u8)]) {
    use super::Inst;
    if ch.is_empty() {
        return;
    }
    let m: HashMap<usize, u8> = ch.iter().copied().collect();
    for (i, x) in s.insts.iter_mut().chain(s.k5.iter_mut()).chain(s.k16.iter_mut()).chain(s.k22c.iter_mut()) {
        let Some(&v) = m.get(i) else { continue };
        let so = match x {
            Inst::C2d9978(c) => c,
            Inst::C2d4b78(c) => &mut c.alo.so,
            Inst::C2d4260(c) => &mut c.alo.so,
            Inst::C2caf88(c) => &mut c.alo.so,
            Inst::K16(c) => &mut c.alo.so,
            Inst::K22C96(c) => &mut c.alo.so,
            _ => continue,
        };
        so.f_2d = v;
    }
}
