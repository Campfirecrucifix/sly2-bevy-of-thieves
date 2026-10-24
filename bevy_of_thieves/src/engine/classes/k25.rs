//! WP-K25: run-time anim objects behind the `ClassHost` anim hooks of `EngineHost` (before K25 every one was a
//! no-op default): ASEGAs (FUN_0012f9b8, the live set 0x3e1218) and SMAs (FUN_001f01b8, the live set 0x3e126c)
//! started by the class code on world ASEG (class 109/110) / SM (class 111) objects, advanced once per class
//! frame (`ClassHost::k25_tick` at the top of `ClassSet::frame`) with the ported playback (anim::play
//! FUN_00130918) and SM walk (anim::sm FUN_001f0d88 / FUN_001f1078).
//!
//! Handles: ASEGA = `ASEGA_TAG | n`, SMA = `SMA_TAG | n` (never reused; a freed / unknown handle reads as "not in
//! the live set", as FUN_0011a380 on a freed object). REPLACED: the vtable +0xc / +0x14 reference counts are
//! one counter per handle (`refs`); the list memberships (0x3e1218 / ASEG +0x50, 0x3e126c / SM +0x68) are
//! `alive`; an entry is dropped once it is dead and unreferenced.
//! UNPORTED: the bindings (channel curves of the target ALO - the draw layer poses from its own players), the
//! FUN_00131990 +0x58 sound hold. K26: the ASEG event dispatch FUN_00131290 (`fired`) and the SMA state-change
//! messages FUN_001f13a8 (`msgs`) are queued here and delivered by the host / ClassSet (classes/k26.rs).
use crate::engine::anim::aseg::{Aseg, Event};
use crate::engine::anim::play::{ee_sub_130918, ee_sub_130eb0, ee_sub_131100, Asega, AsegaHost};
use crate::engine::anim::sm::{ee_sub_1f0400, Sm, Sma, SmaPlay};
use crate::engine::physics::driven::GameRng;
use crate::engine::playerdraw::aseg_of;
use crate::engine::world::World;
use std::collections::HashMap;
use std::sync::Arc;

pub const ASEGA_TAG: u32 = 0x4100_0000;
pub const SMA_TAG: u32 = 0x4200_0000;
const TAG_MASK: u32 = 0xff00_0000;
/// FUN_001f0380 passes t0 = 4 as the SMA mask (+0x48), see npcdraw::NPC_SMA_MASK
pub const SMA_MASK_1F0380: u32 = 4;
/// FUN_0012f248 (ASEG init): +0x78 rate 1, +0x80 / +0x84 / +0x88 rate polynomial {1, 0, 0}, clamp +0x90 = 0 /
/// +0x94 = 1 (+0x78 is overridden by property field 109: Aseg::f78)
pub const ASEG_78: f32 = 1.0;
const ASEG_80: [f32; 3] = [1.0, 0.0, 0.0];
const ASEG_90: f32 = 0.0;
const ASEG_94: f32 = 1.0;

/// One ASEGA (0x60 bytes in the original).
#[derive(Clone, Debug)]
pub struct AsegaRt {
    /// +0x14 ASEG world object index, +0x18 target ALO handle
    pub aseg: usize,
    pub obj: u32,
    pub st: Asega,
    /// +0x48 / +0x4c rate factors (FUN_0012f9b8)
    pub f48: f32,
    pub f4c: f32,
    pub refs: i32,
    /// SMA whose transition this is (FUN_001333c0 end notification)
    pub sma: Option<u32>,
}

/// One SMA (0x50 bytes).
#[derive(Clone, Debug)]
pub struct SmaRt {
    /// +0x24 SM world object index, +0x28 actor handle
    pub sm: usize,
    pub obj: u32,
    pub sma: Sma,
    pub alive: bool,
    pub refs: i32,
    /// ASEGA handle of `sma.asega` (+0x30)
    pub asega: Option<u32>,
}

#[derive(Default)]
pub struct AnimRt {
    /// ANIM-ALL: every FUN_0012f9b8 request (t, rate, anim handle, obj handle, flags), replayed into the live image
    /// (`LiveQ::guest_anim_starts`) so the original ASEGA set world+0x3d88 plays them
    pub guest_reqs: Vec<(f32, f32, u32, u32, u32)>,
    pub asegas: HashMap<u32, AsegaRt>,
    pub smas: HashMap<u32, SmaRt>,
    next: u32,
    sms: HashMap<usize, Arc<Sm>>,
    pub rng: GameRng,
    world: usize,
    /// FUN_00131290 timed events reached (dispatch UNPORTED), FUN_001f13a8 state changes (delivery UNPORTED)
    pub events: usize,
    pub sma_msgs: usize,
    /// K26: FUN_00131290 events to dispatch (ASEG object, event index, reverse, ASEGA handle (K27)) and
    /// FUN_001f13a8 messages to deliver (SMA +0x28 owner handle, msg 10, {old, new} labels); drained by the host
    pub fired: Vec<(usize, usize, bool, u32)>,
    pub msgs: Vec<(u32, i32, crate::engine::anim::sm::SmaMsg)>,
    /// POOL1: FUN_001f13a8 tail: SMA vt+0x1c = FUN_001331f8(sma, 8, 0, 2, {old, new}) = script message 8 in the
    /// SM object's frame (FUN_00256ee0(sma, sm+0)); (SM object index, old label, new label)
    pub script8: Vec<(usize, i32, i32)>,
    /// POOL1 r36: (SMA handle, +0x28 owner handle) of each `script8` entry (same order)
    pub script8_sma: Vec<(u32, u32)>,
    /// POOL1 r30: FUN_0012f9b8 tail (ASEGA vt+0x1c msg 2, after FUN_00256ee0(asega, *aseg) bound the ASEG's frame)
    /// and FUN_00130b18 (msg 3): (ASEG object index, ASEGA handle, ASEGA +0x1c owner handle, msg)
    pub script_aseg: Vec<(usize, u32, u32, u32)>,
    /// ASEGAs / SMAs started (statistics for the tests)
    pub started: usize,
    pub sma_started: usize,
    /// CRAWL R77: FUN_001333c0(sma, node) listeners (FUN_0020fff0 with an owner node): SMA handles whose
    /// FUN_001f13a8 message 10 the sequence node receives (C2d8c08_v4 = FUN_001e1bc8), and the {sma, new label}
    /// messages delivered to them since (consumed by the node's 0xd wait)
    pub k22_listen: std::collections::HashSet<u32>,
    pub k22_msg10: Vec<(u32, i32)>,
    /// CRAWL R50: (target, ASEG) of ASEGAs dropped since the last K26 dispatch: an instant start (flags & 2,
    /// FUN_00131100 from FUN_001f0948) fires its events and is retracted in the same call, as in the original where
    /// FUN_00131290 runs inside FUN_00131100 while the ASEGA is still live
    pub gone: HashMap<u32, (u32, usize)>,
}

impl std::fmt::Debug for AnimRt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AnimRt {{ asegas: {}, smas: {}, started: {}, sma_started: {} }}", self.asegas.len(), self.smas.len(), self.started, self.sma_started)
    }
}

impl Clone for AnimRt {
    fn clone(&self) -> Self {
        AnimRt { k22_listen: self.k22_listen.clone(), k22_msg10: self.k22_msg10.clone(), guest_reqs: self.guest_reqs.clone(), asegas: self.asegas.clone(), smas: self.smas.clone(), next: self.next, sms: self.sms.clone(), rng: self.rng.clone(), world: self.world, events: self.events, sma_msgs: self.sma_msgs, fired: self.fired.clone(), msgs: self.msgs.clone(), script8: self.script8.clone(), script8_sma: self.script8_sma.clone(), script_aseg: self.script_aseg.clone(), started: self.started, sma_started: self.sma_started, gone: self.gone.clone() }
    }
}

/// world object index of handle `h` (index + 1)
fn obj_index(w: &World, h: u32) -> Option<usize> {
    (h as usize).checked_sub(1).filter(|&i| i < w.objects.len() && h & TAG_MASK == 0)
}

struct Ends {
    ended: bool,
    events: usize,
    /// K26: (event index, reverse) reached, for FUN_00131290
    fired: Vec<(usize, bool)>,
}
impl AsegaHost for Ends {
    fn event(&mut self, _a: &mut Asega, i: usize, _ev: &Event, rev: bool) {
        self.events += 1;
        self.fired.push((i, rev));
    }
    fn ended(&mut self, _a: &mut Asega) {
        self.ended = true;
    }
    /// R53: FUN_00130b18 from FUN_00130918 (finished, no loop): out of the live set, then FUN_00131290 on
    /// each +0xa8 (retract) event (the bomb SM's 7 -> 8 ASEG 507: kind 8 (3931) removes the bomb copy)
    fn retract(&mut self, a: &mut Asega) {
        if a.alive {
            self.fired.push((crate::engine::player::p23::RETRACT_EVENTS, false));
        }
        a.alive = false;
    }
}

impl AnimRt {
    /// drop everything when the world changes (handles of an old world are dead)
    fn for_world(&mut self, w: &Arc<World>) {
        let id = Arc::as_ptr(w) as usize;
        if self.world != id {
            *self = AnimRt { world: id, next: self.next, guest_reqs: std::mem::take(&mut self.guest_reqs), ..Default::default() };
        }
    }
    fn id(&mut self, tag: u32) -> u32 {
        self.next = (self.next + 1) & !TAG_MASK;
        tag | self.next
    }
    pub fn is_asega(h: u32) -> bool {
        h & TAG_MASK == ASEGA_TAG
    }
    pub fn is_sma(h: u32) -> bool {
        h & TAG_MASK == SMA_TAG
    }

    /// ASEG of world handle `anim` (an ASEG object)
    pub fn aseg<'w>(w: &'w World, anim: u32) -> Option<&'w Aseg> {
        aseg_of(w, obj_index(w, anim)?)
    }

    /// FUN_001efac0 tables of SM world handle `sm`
    pub fn sm(&mut self, w: &World, sm: u32) -> Option<(usize, Arc<Sm>)> {
        let i = obj_index(w, sm)?;
        if let Some(m) = self.sms.get(&i) {
            return Some((i, m.clone()));
        }
        let m = Arc::new(crate::engine::npcdraw::sm_tables(w, i)?);
        self.sms.insert(i, m.clone());
        Some((i, m))
    }

    /// FUN_0012f9b8(t, rate, aseg, obj, flags): new live ASEGA at time `t` -> handle (0: `anim` is no ASEG)
    pub fn ee_sub_12f9b8(&mut self, w: &Arc<World>, t: f32, rate: f32, anim: u32, obj: u32, flags: u32) -> u32 {
        self.for_world(w);
        let Some(ai) = obj_index(w, anim).filter(|&i| aseg_of(w, i).is_some()) else { return 0 };
        self.start(w, ai, t, rate, obj, flags, None)
    }

    /// The queued npcdraw starts of the level `world` (`anim_play::world_key`).
    pub fn take_npc_reqs(world: usize) -> Vec<(f32, f32, u32, u32, u32)> {
        let mut q = NPC_GUEST_REQS.lock().unwrap();
        let (mine, rest): (Vec<_>, Vec<_>) = q.drain(..).partition(|e| e.0 == world);
        *q = rest;
        mine.into_iter().map(|e| e.1).collect()
    }

    fn start(&mut self, w: &World, ai: usize, t: f32, rate: f32, obj: u32, flags: u32, sma: Option<u32>) -> u32 {
        // ANIM-ALL: every start is a FUN_0012f9b8 call of the original (direct or from the SMA FUN_001f0d88)
        if self.guest_reqs.len() < 4096 {
            self.guest_reqs.push((t, rate, ai as u32 + 1, obj, flags));
        }
        let aseg = aseg_of(w, ai).expect("ASEG object");
        // 0x0012f9b8: +0x48 = ASEG +0x78, +0x4c = clamp(+0x80 + v (+0x84 + v +0x88), +0x90, +0x94), v = FUN_00154c90
        let v = 0.0f32;
        let p = ASEG_80[0] + v * (ASEG_80[1] + v * ASEG_80[2]);
        let f4c = if ASEG_90 <= p { p.min(ASEG_94) } else { ASEG_90 };
        let mut st = Asega { t, t_event: t, flags, ..Default::default() };
        // FUN_00131a00(rate)
        st.svt2 = rate;
        st.svt = rate * aseg.f78() * f4c;
        // FUN_00230cf0 / flags & 2: FUN_00131100 (instant, events up to the end) else FUN_00130eb0 (event cursor)
        let mut ends = Ends { ended: false, events: 0, fired: vec![] };
        if flags & 2 == 0 {
            ee_sub_130eb0(&mut st, aseg);
        } else {
            ee_sub_131100(&mut st, aseg, &mut ends);
        }
        self.events += ends.events;
        let h = self.id(ASEGA_TAG);
        // HEIST r16: 0x0012f9b8 fires FUN_00131290 on each ASEG +0x98 (start, no-time) event at creation, before
        // FUN_00230cf0 / the event cursor (e.g. the EP2 chopper turret's barrel ASEG 119809: kind 0xc 30809 = fire)
        if !aseg.events0.is_empty() {
            self.fired.push((ai, crate::engine::player::p23::START_EVENTS, false, h));
        }
        self.fired.extend(ends.fired.into_iter().map(|(i, rev)| (ai, i, rev, h)));
        self.asegas.insert(h, AsegaRt { aseg: ai, obj, st, f48: aseg.f78(), f4c, refs: 0, sma });
        // FUN_0012f9b8 tail: (vt+0x1c)(asega, 2, 0, 0, 0, 0) = script message 2 in the ASEG's frame
        self.script_aseg.push((ai, h, obj, 2));
        self.started += 1;
        h
    }

    /// FUN_00131968: not in the live set 0x3e1218
    pub fn ee_sub_131968(&self, a: u32) -> bool {
        !self.asegas.get(&a).is_some_and(|x| x.st.alive)
    }
    /// FUN_00130b18: retract (out of 0x3e1218 / ASEG +0x50; the remaining no-time events are UNPORTED)
    pub fn ee_sub_130b18(&mut self, a: u32) {
        if let Some(x) = self.asegas.get_mut(&a) {
            // 0x00130b18: (vt+0x1c)(asega, 3, 0, 0, 0, 0) = script message 3 when it leaves the live set
            if x.st.alive {
                self.script_aseg.push((x.aseg, a, x.obj, 3));
                // R53: then FUN_00131290 on each +0xa8 (retract) event from +0x54 (e.g. the bomb SM's 7 -> 8
                // ASEG 507: kind 8 (3931, 0) = FUN_00210338 remove the bomb copy from the world)
                self.fired.push((x.aseg, crate::engine::player::p23::RETRACT_EVENTS, false, a));
            }
            x.st.alive = false;
        }
        self.gc_asega(a);
    }
    /// vtable +0xc (add reference) / +0x14 (release) of a run-time handle; false = not a run-time handle
    pub fn vcall(&mut self, h: u32, off: u32) -> bool {
        let d = match off {
            0xc => 1,
            0x14 => -1,
            _ => 0,
        };
        if let Some(x) = self.asegas.get_mut(&h) {
            x.refs += d;
            self.gc_asega(h);
            true
        } else if let Some(x) = self.smas.get_mut(&h) {
            x.refs += d;
            self.gc_sma(h);
            true
        } else {
            Self::is_asega(h) || Self::is_sma(h)
        }
    }
    fn gc_asega(&mut self, a: u32) {
        if self.asegas.get(&a).is_some_and(|x| !x.st.alive && x.refs <= 0) {
            if let Some(x) = self.asegas.remove(&a) {
                self.gone.insert(a, (x.obj, x.aseg));
            }
        }
    }
    fn gc_sma(&mut self, s: u32) {
        if self.smas.get(&s).is_some_and(|x| !x.alive && x.refs <= 0) {
            self.smas.remove(&s);
        }
    }
    /// ASEGA float at `off`: +0x1c time, +0x20 event time, +0x40 effective rate, +0x44 rate, +0x48 / +0x4c
    pub fn slot_f(&self, a: u32, off: u32) -> f32 {
        let Some(x) = self.asegas.get(&a) else { return 0.0 };
        match off {
            0x1c => x.st.t,
            0x20 => x.st.t_event,
            0x40 => x.st.svt,
            0x44 => x.st.svt2,
            0x48 => x.f48,
            0x4c => x.f4c,
            _ => 0.0,
        }
    }
    /// FUN_00131a00(rate, asega)
    pub fn ee_sub_131a00(&mut self, rate: f32, a: u32) {
        if let Some(x) = self.asegas.get_mut(&a) {
            x.st.svt2 = rate;
            x.st.svt = rate * x.f48 * x.f4c;
        }
    }
    /// FUN_00131a20(v, asega): +0x48 = v, +0x40 = +0x44 * v * +0x4c (WP-Q3)
    pub fn ee_sub_131a20(&mut self, v: f32, a: u32) {
        if let Some(x) = self.asegas.get_mut(&a) {
            x.f48 = v;
            x.st.svt = x.st.svt2 * v * x.f4c;
        }
    }
    /// FEEL R48: the live ASEGA's effective rate +0x40 (svt) (None: not live)
    pub fn svt(&self, a: u32) -> Option<f32> {
        self.asegas.get(&a).filter(|x| x.st.alive).map(|x| x.st.svt)
    }
    /// FUN_00204e38(t = +0x1c, obj, asega): root-channel velocity at the effective rate (+0x40); zero when not live
    pub fn ee_sub_204e38(&self, w: &World, a: u32) -> [f32; 3] {
        let Some(x) = self.asegas.get(&a).filter(|x| x.st.alive) else { return [0.0; 3] };
        aseg_of(w, x.aseg).map_or([0.0; 3], |g| super::guard::los::ee_sub_204e38(g, x.st.t, x.st.svt))
    }

    // ---- SMA
    /// FUN_001f01b8(sm, obj, label, flags, mask) -> SMA handle (0: `sm` is no SM object)
    pub fn ee_sub_1f01b8(&mut self, w: &Arc<World>, sm: u32, obj: u32, label: i32, flags: u32, mask: u32) -> u32 {
        self.for_world(w);
        let Some((si, m)) = self.sm(w, sm) else { return 0 };
        let len = |o: usize| aseg_of(w, o).map_or(0.0, |a| a.length);
        let sma = Sma::ee_sub_1f01b8(&m, &mut self.rng, label, flags, mask, &len);
        let h = self.id(SMA_TAG);
        self.smas.insert(h, SmaRt { sm: si, obj, sma, alive: true, refs: 0, asega: None });
        self.sma_started += 1;
        self.sync(w, h);
        h
    }
    /// FUN_001f03c8(sm, obj): the live SMA of SM handle `sm` on actor `obj` (the SM +0x68 list, +0x28 == obj; 0 none)
    pub fn ee_sub_1f03c8(&self, w: &World, sm: u32, obj: u32) -> u32 {
        let Some(i) = obj_index(w, sm) else { return 0 };
        self.smas.iter().find(|(_, x)| x.alive && x.sm == i && x.obj == obj).map_or(0, |(h, _)| *h)
    }
    /// FUN_001f0638: not in the live set 0x3e126c
    pub fn ee_sub_1f0638(&self, s: u32) -> bool {
        !self.smas.get(&s).is_some_and(|x| x.alive)
    }
    /// FUN_001f0598: stop (transition ASEGA retracted, out of 0x3e126c / SM +0x68)
    pub fn ee_sub_1f0598(&mut self, s: u32) {
        let Some(x) = self.smas.get_mut(&s).filter(|x| x.alive) else { return };
        x.alive = false;
        x.sma.asega = None;
        if let Some(a) = x.asega.take() {
            self.ee_sub_130b18(a);
        }
    }
    /// FUN_001f15b8(&slot) / the release part of FUN_001f1448: stop + vtable +0x14
    pub fn ee_sub_1f15b8(&mut self, slot: &mut u32) {
        if *slot != 0 {
            self.ee_sub_1f0598(*slot);
            self.vcall(*slot, 0x14);
            *slot = 0;
        }
    }
    /// FUN_001f0400 on SM handle `sm`
    pub fn ee_sub_1f0400(&mut self, w: &World, sm: u32, label: i32) -> i32 {
        self.sm(w, sm).map_or(-1, |(_, m)| ee_sub_1f0400(&m.states, label))
    }
    /// FUN_001f0660(sma, label)
    pub fn ee_sub_1f0660(&mut self, w: &Arc<World>, s: u32, label: i32) {
        let Some((m, mut sma)) = self.take_sma(s) else { return };
        let len = |o: usize| aseg_of(w, o).map_or(0.0, |a| a.length);
        sma.ee_sub_1f0660(&m, &mut self.rng, label, &len);
        self.put_sma(w, s, sma);
    }
    /// FUN_001f09d8(sma, label): jump to state `label` (the playing transition is retracted)
    /// POOL1: FUN_001f0730(sma, label): unless FUN_001f0638(sma) or label == -1: idx = FUN_001f0400(+0x24, label);
    /// if idx != -1 and FUN_001f0b80(+0x24, +0x48, cur, idx, 0) (cur = +0x38 if +0x30 else +0x34) then
    /// FUN_001f0660(sma, label)
    pub fn ee_sub_1f0730(&mut self, w: &Arc<World>, s: u32, label: i32) {
        if self.ee_sub_1f0638(s) || label == -1 {
            return;
        }
        let Some((m, sma)) = self.take_sma(s) else { return };
        let idx = ee_sub_1f0400(&m.states, label);
        let cur = if sma.asega.is_some() { sma.next } else { sma.cur };
        let ok = idx != -1 && m.ee_sub_1f0b80(&mut self.rng, sma.mask, cur, idx).is_some();
        self.put_sma(w, s, sma);
        if ok {
            self.ee_sub_1f0660(w, s, label);
        }
    }
    /// CRAWL R50: FUN_001f67e0(obj, sm_oid): first live SMA (0x3e126c) whose actor (+0x28) is `obj` and whose SM
    /// (+0x24) oid (+0x18) is `sm_oid` (-1: any) -> handle, 0 = none. REPLACED order: the live list is the handle
    /// order (handles are never reused).
    pub fn ee_sub_1f67e0(&self, w: &World, obj: u32, sm_oid: i32) -> u32 {
        let mut hs: Vec<u32> = self.smas.iter().filter(|(_, x)| x.alive && x.obj == obj).map(|(&h, _)| h).collect();
        hs.sort_unstable();
        hs.into_iter().find(|h| sm_oid == -1 || self.smas[h].sm < w.objects.len() && w.objects[self.smas[h].sm].oid == sm_oid).unwrap_or(0)
    }
    /// CRAWL R50: FUN_001f0948(sma, label): unless FUN_001f0638: +0x3c goal = FUN_001f0438(SM, label), FUN_001f08d0
    /// (a playing transition is cut: FUN_00130d30 + FUN_00130b18 + FUN_001f0fa0), FUN_001f0d88(sma, 1) (instant)
    pub fn ee_sub_1f0948(&mut self, w: &Arc<World>, s: u32, label: i32) {
        let Some((m, mut sma)) = self.take_sma(s) else { return };
        sma.goal = ee_sub_1f0400(&m.states, label);
        if sma.asega.is_some() {
            sma.ee_sub_1f0fa0();
        }
        let len = |o: usize| aseg_of(w, o).map_or(0.0, |a| a.length);
        sma.ee_sub_1f0d88(&m, &mut self.rng, true, &len);
        self.put_sma(w, s, sma);
    }
    /// CRAWL R120 (debug): (+0x34 cur, +0x38 next, +0x3c goal) of a live SMA
    pub fn sma_state(&self, s: u32) -> Option<(i32, i32, i32)> {
        self.smas.get(&s).filter(|x| x.alive).map(|x| (x.sma.cur, x.sma.next, x.sma.goal))
    }
    /// CRAWL R50: FUN_001f07e8(sma) = FUN_001f04f8(SM, +0x34): label of the current state (-1: no live SMA)
    pub fn ee_sub_1f07e8(&self, s: u32) -> i32 {
        let Some(x) = self.smas.get(&s).filter(|x| x.alive) else { return -1 };
        self.sms.get(&x.sm).map_or(-1, |m| crate::engine::anim::sm::ee_sub_1f04f8(m, x.sma.cur))
    }
    /// FUN_001f09d8(sma, label): jump to state `label` (the playing transition is retracted)
    pub fn ee_sub_1f09d8(&mut self, w: &Arc<World>, s: u32, label: i32) {
        let Some((m, mut sma)) = self.take_sma(s) else { return };
        let i = ee_sub_1f0400(&m.states, label);
        if sma.cur == i && (sma.goal == i || sma.goal == -1) {
            return self.put_sma(w, s, sma);
        }
        sma.asega = None; // +0x30 = 0, FUN_00130b18 (sync retracts the handle)
        let old = sma.cur;
        sma.goal = i;
        sma.cur = i;
        sma.next = -1;
        sma.changes.push((old, i));
        let len = |o: usize| aseg_of(w, o).map_or(0.0, |a| a.length);
        sma.ee_sub_1f0d88(&m, &mut self.rng, false, &len);
        self.put_sma(w, s, sma);
    }
    /// FUN_001f1350(sma, mask): +0x48 = mask; a playing transition whose +0x10 has a bit of `mask` is cut to its
    /// end (FUN_001f08d0: FUN_00130d30 + FUN_00130b18 + FUN_001f0fa0) and the walk goes on (FUN_001f0d88)
    pub fn ee_sub_1f1350(&mut self, w: &Arc<World>, s: u32, mask: u32) {
        let Some((m, mut sma)) = self.take_sma(s) else { return };
        sma.mask = mask;
        if sma.asega.is_some() && sma.trans.is_some_and(|t| m.trans[t].f10 as u32 & mask != 0) {
            sma.ee_sub_1f0fa0();
            let len = |o: usize| aseg_of(w, o).map_or(0.0, |a| a.length);
            sma.ee_sub_1f0d88(&m, &mut self.rng, false, &len);
        }
        self.put_sma(w, s, sma);
    }
    fn take_sma(&mut self, s: u32) -> Option<(Arc<Sm>, Sma)> {
        let x = self.smas.get(&s).filter(|x| x.alive)?;
        let m = self.sms.get(&x.sm)?.clone();
        Some((m, x.sma.clone()))
    }
    fn put_sma(&mut self, w: &World, s: u32, sma: Sma) {
        if let Some(x) = self.smas.get_mut(&s) {
            x.sma = sma;
        }
        self.sync(w, s);
    }
    /// bind the SMA's +0x30 transition to a run-time ASEGA (FUN_0012f9b8 from FUN_001f0d88) / retract it
    fn sync(&mut self, w: &World, s: u32) {
        let Some(x) = self.smas.get_mut(&s) else { return };
        self.sma_msgs += x.sma.changes.len();
        let sma_h = s;
        // K26: FUN_001f13a8 data {sma, FUN_001f04f8(old), FUN_001f04f8(new)} for the owner's slot 4 (msg 10)
        if let Some(m) = self.sms.get(&x.sm) {
            for (old, new) in x.sma.changes.drain(..) {
                let msg = crate::engine::anim::sm::SmaMsg { old: crate::engine::anim::sm::ee_sub_1f04f8(m, old), new: crate::engine::anim::sm::ee_sub_1f04f8(m, new) };
                self.msgs.push((x.obj, crate::engine::anim::sm::MSGID_SMA_STATE, msg));
                self.script8.push((x.sm, msg.old, msg.new));
                self.script8_sma.push((sma_h, x.obj));
                if self.k22_listen.contains(&sma_h) && self.k22_msg10.len() < 256 {
                    self.k22_msg10.push((sma_h, msg.new));
                }
            }
        }
        x.sma.changes.clear();
        // CRAWL R50: instant transitions: start (fires the FUN_00131100 events) and retract at once
        let inst = std::mem::take(&mut x.sma.instant);
        let obj = x.obj;
        for p in inst.into_iter().filter(|p| aseg_of(w, p.aseg).is_some()) {
            let a = self.start(w, p.aseg, p.start, p.svt, obj, p.flags, Some(s));
            self.ee_sub_130b18(a);
        }
        let Some(x) = self.smas.get_mut(&s) else { return };
        let want: Option<SmaPlay> = x.sma.asega.clone();
        let cur = x.asega;
        let obj = x.obj;
        let same = match (cur, &want) {
            (Some(a), Some(p)) => self.asegas.get(&a).is_some_and(|r| r.st.alive && r.aseg == p.aseg),
            _ => false,
        };
        if same {
            return;
        }
        if let Some(a) = cur {
            self.ee_sub_130b18(a);
        }
        let a = want.filter(|p| aseg_of(w, p.aseg).is_some()).map(|p| self.start(w, p.aseg, p.start, p.svt, obj, p.flags, Some(s)));
        if let Some(x) = self.smas.get_mut(&s) {
            x.asega = a;
        }
    }

    /// One class frame: FUN_00130918 on every live ASEGA (dt = DAT_002dded4; the unscaled clock is the same in
    /// the class port); an SMA transition that reached its end goes on (FUN_001f1078).
    pub fn tick(&mut self, w: &Arc<World>, dt: f32) {
        self.for_world(w);
        let mut keys: Vec<u32> = self.asegas.iter().filter(|x| x.1.st.alive).map(|x| *x.0).collect();
        keys.sort_unstable();
        let mut done_smas = vec![];
        for a in keys {
            let x = self.asegas.get_mut(&a).expect("live ASEGA");
            let Some(g) = aseg_of(w, x.aseg) else { continue };
            let mut ends = Ends { ended: false, events: 0, fired: vec![] };
            ee_sub_130918(&mut x.st, g, dt, dt, &mut ends);
            self.events += ends.events;
            let ai = x.aseg;
            self.fired.extend(ends.fired.into_iter().map(|(i, rev)| (ai, i, rev, a)));
            if ends.ended {
                if let Some(s) = x.sma {
                    done_smas.push((s, a));
                }
            }
            self.gc_asega(a);
        }
        for (s, a) in done_smas {
            let Some(r) = self.smas.get_mut(&s).filter(|r| r.alive && r.asega == Some(a)) else { continue };
            r.asega = None;
            let Some(m) = self.sms.get(&r.sm).cloned() else { continue };
            let len = |o: usize| aseg_of(w, o).map_or(0.0, |a| a.length);
            r.sma.ee_sub_1f1078(&m, &mut self.rng, &len);
            self.ee_sub_130b18(a);
            self.sync(w, s);
        }
    }
}

impl AnimRt {
    /// FUN_001f07e8(sma) == FUN_001f07c8(sma): FUN_001f04f8(SM +0x24, +0x34 current) == FUN_001f04f8(+0x3c goal),
    /// the SMA reached its goal state (labels compared; -1 out of range). Unknown handle -> true (done).
    pub fn ee_sub_1f07c8_done(&self, s: u32) -> bool {
        let Some(x) = self.smas.get(&s) else { return true };
        let Some(m) = self.sms.get(&x.sm) else { return true };
        crate::engine::anim::sm::ee_sub_1f04f8(m, x.sma.cur) == crate::engine::anim::sm::ee_sub_1f04f8(m, x.sma.goal)
    }
}

/// K25 sound: the class code's sound emitters (C2d9c70, 0x70 bytes, pool 0x3dd05c / active list 0x3dd08c) on
/// the ported `audio::emitter::EmitterSystem` with the camera's 1:1 setter ports (camera::sound) and
/// player::p16::FUN_001fb3f0. REPLACED: the class host owns its own pool (the game binary's script / camera pool
/// is a separate Bevy resource); `plays` lists every emitter created this run (obj, sound) for the host layer.
#[derive(Default, Debug)]
pub struct SoundRt {
    pub es: crate::engine::audio::emitter::EmitterSystem,
    /// C2cabd8 +0xec handle slots per (object, entry)
    pub slots: HashMap<(usize, usize), crate::engine::camera::sound::CmEmitter>,
    pub plays: Vec<(usize, i32)>,
}
impl Clone for SoundRt {
    fn clone(&self) -> Self {
        let es = crate::engine::audio::emitter::EmitterSystem { slots: self.es.slots.clone(), active: self.es.active.clone(), freed: self.es.freed.clone() };
        SoundRt { es, slots: self.slots.clone(), plays: self.plays.clone() }
    }
}
/// emitter handle = SND_TAG | (pool index + 1)
pub const SND_TAG: u32 = 0x4300_0000;

impl SoundRt {
    /// FUN_001fb4f0(near, far, e): +0x34 = near < far ? far : near + 1, +0x30 = near
    pub fn ee_sub_1fb4f0(&mut self, near: f32, far: f32, id: usize) {
        if let Some(e) = self.es.get_mut(id) {
            e.far = if far <= near { near + 1.0 } else { far };
            e.near = near;
        }
    }
    /// FUN_001fbb90(sound) + FUN_001fb2f8(e, 0) (k8 / k15_83 / k22_96 one-shots: no reference kept)
    pub fn ee_sub_1fbb90(&mut self, obj: usize, sound: i32) -> usize {
        let id = self.es.new_emitter_1fbb90(sound);
        self.es.set_unpositioned_1fb2f8(id, 0);
        self.plays.push((obj, sound));
        id
    }
    /// FUN_0012ed20 loop body: FUN_001fbb90(e[0]), FUN_001fb2f8(e, obj), FUN_001fb4f0(e[1], e[2]), FUN_001fb1f8(e[3]),
    /// FUN_001fb2c0(e[4]), FUN_00258040(&slot, e) -> handle
    pub fn emitter_create(&mut self, obj: usize, k: usize, e: [i32; 5]) -> i32 {
        use crate::engine::camera::sound as cs;
        let f = |i: usize| f32::from_bits(e[i] as u32);
        let id = self.es.new_emitter_1fbb90(e[0]);
        self.plays.push((obj, e[0]));
        self.es.set_unpositioned_1fb2f8(id, obj as i32 + 1);
        self.ee_sub_1fb4f0(f(1), f(2), id);
        let mut slot = self.slots.remove(&(obj, k)).unwrap_or_default();
        cs::FUN_00258040(&mut self.es, &mut slot, Some(id));
        cs::FUN_001fb1f8(&mut self.es, f(3), &slot);
        cs::FUN_001fb2c0(&mut self.es, f(4), &mut slot);
        self.slots.insert((obj, k), slot);
        (SND_TAG | (id as u32 + 1)) as i32
    }
    /// FUN_0012ede8 loop body: FUN_001fb098(0, e) + FUN_00258040(&slot, 0)
    pub fn emitter_release(&mut self, obj: usize, k: usize) {
        use crate::engine::camera::sound as cs;
        let Some(mut slot) = self.slots.remove(&(obj, k)) else { return };
        cs::FUN_001fb098(&mut self.es, 0.0, &mut slot);
        cs::FUN_00258040(&mut self.es, &mut slot, None);
    }
    /// FUN_0015c690 sound part: FUN_001fbb90(sd[0]) + FUN_001fb2f8(0) + FUN_001fb3f0(pos) + FUN_001fb4f0(sd[1], sd[2])
    /// + FUN_001fb1f8(sd[3]) + FUN_001fb2c0(sd[4]) (no reference kept)
    pub fn sound_15c690(&mut self, src: u32, pos: [f32; 4], sd: [u32; 5]) {
        use crate::engine::camera::sound as cs;
        let f = |i: usize| f32::from_bits(sd[i]);
        let id = self.es.new_emitter_1fbb90(sd[0] as i32);
        self.plays.push((src as usize, sd[0] as i32));
        self.es.set_unpositioned_1fb2f8(id, 0);
        crate::engine::player::p16::FUN_001fb3f0(&mut self.es, id, [pos[0], pos[1], pos[2]]);
        self.ee_sub_1fb4f0(f(1), f(2), id);
        let mut slot = cs::CmEmitter { id: Some(id), pitch: 0.0, fade: 0.0 };
        cs::FUN_001fb1f8(&mut self.es, f(3), &slot);
        cs::FUN_001fb2c0(&mut self.es, f(4), &mut slot);
    }
}

// ================================================================ physics (WP-K25 group 3)
use super::shared::act::{self, Act, ActPool, XfActs};
use super::{ClassHost, ClassSet, Inst};
use crate::engine::physics::driver::AloPhys;

type V3 = [f32; 3];
/// act-record handles of the run-time constraint pool (`ActPool::next` starts here, below the ASEGA tag)
pub const CON_TAG: u32 = 0x4400_0000;

/// Velocity state of an object whose physics a class instance (or a ported guard) owns: xfLin / xfAng of its
/// `AloPhys` and the +0xe8 block fields the hooks read. Taken at the top of `ClassSet::frame` (the previous
/// frame's end state: REPLACED, the original reads the live xf mid-frame).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BodySnap {
    pub pos: V3,
    pub v: V3,
    pub w: V3,
    /// e8 +0x80
    pub gravity: V3,
    /// e8 +0xc0 (constraint kind of the linear xf; 0 free .. 3 locked)
    pub ckind: u32,
}

impl BodySnap {
    pub fn of(p: &AloPhys) -> Self {
        BodySnap { pos: p.pos, v: p.v, w: p.w, gravity: p.gravity, ckind: p.ckind }
    }
}

fn snap_inst(x: &Inst) -> Option<BodySnap> {
    match x {
        Inst::K13(c) => Some(BodySnap::of(&c.p.phys)),
        Inst::K15(c) => Some(BodySnap::of(&c.phys)),
        Inst::K15b(c) | Inst::K23C102(c) => Some(BodySnap::of(&c.phys)),
        Inst::K15c(c) => Some(BodySnap::of(&c.p.phys)),
        Inst::Guard(g) => {
            // kinematic patrol body: FUN_00204ca8 motion R(+0x1a8) * v190 * k180 (patrol::ee_sub_204ca8_move);
            // lifted by a class-102 bomb: the K24 PO velocity
            let v = match &g.po {
                Some(p) => p.vel,
                None => {
                    let b = &g.body;
                    let (s, c) = b.f_1a8.sin_cos();
                    let r = b.f_190;
                    [(c * r[0] - s * r[1]) * b.f_180, (s * r[0] + c * r[1]) * b.f_180, r[2] * b.f_180]
                }
            };
            Some(BodySnap { pos: g.body.pos, v, w: [0.0; 3], gravity: super::guard::lift::GRAVITY, ckind: 0 })
        }
        _ => None,
    }
}

impl ClassSet {
    /// K25: bodies of the class-owned physics (`AloPhys` instances, ported guards) by object index
    pub fn k25_bodies(&self) -> Vec<(usize, BodySnap)> {
        let lists = [
            &self.insts, &self.more, &self.k4, &self.k5, &self.k6, &self.k7, &self.k8, &self.k9, &self.k10, &self.k11, &self.k12, &self.k13, &self.k14, &self.k15, &self.k16,
            &self.k18, &self.k20, &self.k21c, &self.k22c, &self.k23c,
        ];
        lists.iter().flat_map(|l| l.iter()).filter_map(|(i, x)| snap_inst(x).map(|b| (*i, b))).collect()
    }
}

/// Run-time physics state behind the physics hooks of `EngineHost`: class-owned body snapshots and the xf
/// actuator lists (+0x90) of world objects that class code constrains (FUN_0012dc18 / 12d108 / 12e460).
#[derive(Clone, Debug)]
pub struct PhysRt {
    pub bodies: HashMap<usize, BodySnap>,
    pub pool: ActPool,
    /// xf handle (owner handle * 0x100 + xf offset, the shared::act convention) -> its +0x90 list
    pub xfs: HashMap<u32, XfActs>,
    /// xf +0xc0 driven target (FUN_0012d108 on the head record)
    pub targets: HashMap<u32, V3>,
    /// record id -> xf handle
    pub recs: HashMap<u32, u32>,
    /// guard handle -> its +0xfc4 scale record
    pub fc4: HashMap<u32, u32>,
    world: usize,
}

impl Default for PhysRt {
    fn default() -> Self {
        PhysRt { bodies: HashMap::new(), pool: ActPool { next: CON_TAG, live: 0 }, xfs: HashMap::new(), targets: HashMap::new(), recs: HashMap::new(), fc4: HashMap::new(), world: 0 }
    }
}

fn v3_bits(v: V3) -> [u32; 3] {
    [v[0].to_bits(), v[1].to_bits(), v[2].to_bits()]
}
fn rec_v(a: &Act) -> V3 {
    [f32::from_bits(a.w[8]), f32::from_bits(a.w[9]), f32::from_bits(a.w[10])]
}

/// FUN_0012dc18(owner, xf, prio, f_14, f_1c, v): kind-1 record (FUN_0012db50), +0x1c = f_1c, +0x20 = v,
/// inserted into xf +0x90 (FUN_00230018); returns the record
pub fn ee_sub_12dc18(pool: &mut ActPool, owner: u32, xf: &mut XfActs, prio: i32, f_14: u32, f_1c: u32, v: V3, h: &mut dyn ClassHost) -> u32 {
    let Some(mut a) = act::ee_sub_12db50(pool, 1, owner, xf.xf, prio, f_14) else { return 0 };
    a.w[7] = f_1c;
    a.w[8..11].copy_from_slice(&v3_bits(v));
    let id = a.id;
    act::ee_sub_230018(owner, xf, a, h);
    id
}

/// FUN_0012d058(rec): the record is in its xf's +0x90 list
pub fn ee_sub_12d058(xf: &XfActs, rec: u32) -> bool {
    xf.list.iter().any(|a| a.id == rec)
}

/// FUN_00230100(owner, xf, rec): unlink; a new head -> FUN_0022c840, old head +0x14, new head +0xc; the same
/// head -> head +0x1c
pub fn ee_sub_230100(owner: u32, xf: &mut XfActs, rec: u32, h: &mut dyn ClassHost) {
    let Some(k) = xf.list.iter().position(|a| a.id == rec) else { return };
    let old = (xf.list[0].id, xf.list[0].vt());
    xf.list.remove(k);
    if k == 0 {
        act::ee_sub_22c840(owner, xf, h);
        h.act_vcall(old.1, 0x14, xf.xf, old.0);
        if let Some(n) = xf.list.first() {
            h.act_vcall(n.vt(), 0xc, xf.xf, n.id);
        }
    } else {
        h.act_vcall(old.1, 0x1c, xf.xf, old.0);
    }
}

/// FUN_0012e460(&slot): FUN_0012d020 (FUN_0012d058 + FUN_00230100) + FUN_0012e210 (FUN_0011cd98 back to the
/// 0x3e123c pool: live count --), slot = 0
pub fn ee_sub_12e460(pool: &mut ActPool, xf: &mut XfActs, slot: &mut u32, h: &mut dyn ClassHost) {
    let rec = *slot;
    if rec == 0 {
        return;
    }
    let owner = xf.list.iter().find(|a| a.id == rec).map_or(0, |a| a.w[1]);
    if ee_sub_12d058(xf, rec) {
        ee_sub_230100(owner, xf, rec, h);
    }
    pool.live -= 1;
    *slot = 0;
}

/// FUN_0012d108(rec, v): +0x20 = v; the head record of its xf with v not approx-equal (FUN_0011d4d0, 1e-4 per
/// component) to the previous +0x20 -> xf +0xc0 = v and FUN_00230598 (wake). Returns the new xf target.
pub fn ee_sub_12d108(xf: &mut XfActs, rec: u32, v: V3) -> Option<V3> {
    let head = xf.list.first().map(|a| a.id);
    let a = xf.list.iter_mut().find(|a| a.id == rec)?;
    let old = rec_v(a);
    a.w[8..11].copy_from_slice(&v3_bits(v));
    let same = (0..3).all(|k| crate::engine::physics::driver::FUN_0011d4d0(old[k], v[k], 1e-4));
    (head == Some(rec) && !same).then_some(v)
}

impl PhysRt {
    pub fn set_world(&mut self, w: &Arc<World>) {
        let id = Arc::as_ptr(w) as usize;
        if self.world != id {
            *self = PhysRt { world: id, pool: ActPool { next: self.pool.next, live: 0 }, ..Default::default() };
        }
    }
    pub fn set_bodies(&mut self, w: &Arc<World>, b: Vec<(usize, BodySnap)>) {
        self.set_world(w);
        self.bodies = b.into_iter().collect();
    }
    pub fn body(&self, obj: usize) -> Option<&BodySnap> {
        self.bodies.get(&obj)
    }
    /// (vtable +0x10c)(obj) = FUN_001f4558: xfLin velocity (REPLACED as in k13: top-level bodies); a static
    /// world object (no dynamic xf) moves at 0
    pub fn vcall_10c(&self, obj: usize) -> V3 {
        self.body(obj).map_or([0.0; 3], |b| b.v)
    }
    /// FUN_0022d348(alo, 0, p): v + w x (p - pos) (player::p5_hull convention); static objects 0
    pub fn ee_sub_22d348(&self, obj: usize, p: V3) -> V3 {
        let Some(b) = self.body(obj) else { return [0.0; 3] };
        let r = [p[0] - b.pos[0], p[1] - b.pos[1], p[2] - b.pos[2]];
        let c = [b.w[1] * r[2] - b.w[2] * r[1], b.w[2] * r[0] - b.w[0] * r[2], b.w[0] * r[1] - b.w[1] * r[0]];
        [b.v[0] + c[0], b.v[1] + c[1], b.v[2] + c[2]]
    }
    /// take the xf list of (owner, off) out (created empty on first use), to run act code with the host
    pub fn take_xf(&mut self, owner: u32, off: u32) -> XfActs {
        let xh = owner.wrapping_mul(0x100).wrapping_add(off);
        self.xfs.remove(&xh).unwrap_or_else(|| XfActs { xf: xh, ..Default::default() })
    }
    pub fn put_xf(&mut self, xf: XfActs) {
        for a in &xf.list {
            self.recs.insert(a.id, xf.xf);
        }
        self.xfs.insert(xf.xf, xf);
    }
    /// xf of record `rec` (None: not a run-time constraint of this world)
    pub fn xf_of(&mut self, rec: u32) -> Option<XfActs> {
        let xh = *self.recs.get(&rec)?;
        self.xfs.remove(&xh)
    }
    /// the driven target of the guard's +0xfc4 record (its scale; 1 before any FUN_0012d108)
    pub fn fc4_scale(&self, guard: u32) -> f32 {
        self.fc4.get(&guard).and_then(|r| self.recs.get(r)).and_then(|x| self.targets.get(x)).map_or(1.0, |v| v[0])
    }
}


/// FUN_001f4c58 result handle: `BONE_TAG | (obj + 1) << 10 | bone index + 1` (the 0x100-byte record of ALO
/// `obj`'s +0x54 list; REPLACED: the record address is a tagged handle)
pub const BONE_TAG: u32 = 0x4500_0000;

/// K25: plain fields of world objects not owned by the class set (`ClassHost::obj_*`).
/// REPLACED: the original reads / writes the object's memory; the port has no LO memory image, so the fields the
/// class code writes (obj_set_u32 / obj_or_40 / obj_and_40) are kept in an overlay and read back, and the
/// unwritten fields with a world counterpart are derived: LO +0x18 = oid, run-time ASEGA / SMA fields from
/// `AnimRt`. Any other unwritten field reads 0 (as before K25). UNPORTED: LO +0x28 (bit 60 = in the world is
/// set by the unported world insert; templates such as the class-101 throwables are loaded but
/// not in the world, so the bit cannot be derived from the load) reads 0 unless written.
#[derive(Clone, Debug, Default)]
pub struct FieldRt {
    pub fields: HashMap<(u32, u32), u32>,
    world: usize,
}

impl FieldRt {
    pub fn set_world(&mut self, w: &Arc<World>) {
        let id = Arc::as_ptr(w) as usize;
        if self.world != id {
            *self = FieldRt { world: id, ..Default::default() };
        }
    }
    pub fn get(&self, obj: u32, off: u32) -> Option<u32> {
        self.fields.get(&(obj, off)).copied()
    }
    pub fn set(&mut self, obj: u32, off: u32, v: u32) {
        self.fields.insert((obj, off), v);
    }
    /// *(obj + off) of an unwritten field
    pub fn base(w: &World, a: &AnimRt, obj: u32, off: u32) -> u32 {
        if AnimRt::is_asega(obj) {
            let Some(x) = a.asegas.get(&obj) else { return 0 };
            return match off {
                0x14 => x.aseg as u32 + 1,
                0x18 => x.obj,
                0x24 => x.st.flags,
                _ => a.slot_f(obj, off).to_bits(),
            };
        }
        if AnimRt::is_sma(obj) {
            let Some(x) = a.smas.get(&obj) else { return 0 };
            return match off {
                0x24 => x.sm as u32 + 1,
                0x28 => x.obj,
                0x30 => x.asega.unwrap_or(0),
                _ => 0,
            };
        }
        match (obj_index(w, obj).and_then(|i| w.objects.get(i)), off) {
            (Some(o), 0x18) => o.oid as u32,
            // POOL1 R55: LO +0x10 parent (handle) and +0x38 LOD radius (field 13; 1e10 from the LO init), as the
            // class-29 emitters' camera-distance gate FUN_00159c80 reads them (rec11 285: 4000 / 3000 / 1e10)
            (Some(o), 0x10) => o.owner.map_or(0, |p| p as u32 + 1),
            (Some(o), 0x38) => crate::engine::game::fields::object_props(o)
                .and_then(|p| p.iter().find(|p| p.field == 13))
                .and_then(|p| crate::engine::classes::actor::c2caf88::prop_args(p).first().copied())
                .unwrap_or(1.0e10f32.to_bits()),
            // K27: ALO +0x12c alpha, 1.0 after the ALO init (FUN_001f2d48)
            (Some(o), 0x12c) if crate::engine::world::class::ee_sub_15e988(o.class, 0x30) => 1f32.to_bits(),
            _ => 0,
        }
    }
}

/// FUN_001f4c58(alo, oid, kind) on world ALO handle `alo` (k21_77::ee_sub_1f4c58 bone walk)
pub fn ee_sub_1f4c58(w: &World, alo: u32, oid: i32, kind: i32) -> u32 {
    let Some(i) = obj_index(w, alo) else { return 0 };
    match super::k21_77::ee_sub_1f4c58(w, i, oid, kind) {
        0 => 0,
        b => {
            assert!(i < 1 << 14 && b < 1 << 10, "bone handle overflow {i} {b}");
            BONE_TAG | (i as u32 + 1) << 10 | b
        }
    }
}

/// ANIM-ALL: npcdraw's FUN_0012f9b8 stand-in starts (guards / NPCs), replayed into the guest like `guest_reqs`.
/// ANIM-ALL R22: keyed by level (`anim_play::world_key`) like `GUEST_STARTS`; unkeyed, a parallel test level's
/// guard starts (object handles of another level) were replayed into this level's image (sweep start fails).
pub static NPC_GUEST_REQS: std::sync::Mutex<Vec<(usize, (f32, f32, u32, u32, u32))>> = std::sync::Mutex::new(Vec::new());
