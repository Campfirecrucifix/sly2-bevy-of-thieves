//! WP-A3: state machine (SM, class 111, load FUN_001efac0) and its runtime instance (SMA,
//! FUN_001f01b8). An SM is a graph of labelled states joined by transitions that each play an
//! ASEG; an SMA walks it towards a goal state choosing transitions by mask and weight.
//!
//! SM layout: +0x50/+0x54 states {label, next} (next: -1 stop, 0x20 -> -2 = any transition,
//! else the state index of a label), +0x58/+0x5c transitions (0x18 bytes: +0 ASEG, +4 from,
//! +8 to, +0xc mask (bit 0 = usable backwards), +0x10, +0x14 weight), +0x60/+0x64 distance
//! tables {group mask, n x n u8 hop counts}.
//! SMA (0x50 bytes): +0x24 SM, +0x28 actor, +0x2c ASEGA flags, +0x30 ASEGA, +0x34 state,
//! +0x38 next state, +0x3c goal, +0x40 transition, +0x44 speed (1.0), +0x48 mask.

use crate::engine::physics::driven::GameRng;

/// One SM transition (+0x5c entry).
#[derive(Clone, Debug, PartialEq)]
pub struct Transition {
    /// +0x00: ASEG object (None: oid -1, no animation)
    pub aseg: Option<usize>,
    /// +0x04 / +0x08: state indices (-1: label not a state)
    pub from: i32,
    pub to: i32,
    /// +0x0c
    pub mask: u32,
    /// +0x10
    pub f10: i32,
    /// +0x14
    pub weight: f32,
}

/// Post-load SM (FUN_001efac0 after the stream reads).
#[derive(Clone, Debug, PartialEq)]
pub struct Sm {
    /// +0x54: (label, next state index / -1 / -2)
    pub states: Vec<(i32, i32)>,
    pub trans: Vec<Transition>,
    /// +0x64: (group mask, n x n hop counts, 0xff = unreachable)
    pub groups: Vec<(u32, Vec<u8>)>,
}

/// FUN_001f0400(sm, label): state index of a label, -1 if none.
pub fn ee_sub_1f0400(states: &[(i32, i32)], label: i32) -> i32 {
    // 0x001f0400
    states.iter().position(|s| s.0 == label).map_or(-1, |i| i as i32)
}

impl Sm {
    /// FUN_001efac0 from the stream fields (`pairs`, `links` = {from, to, mask, +0x10},
    /// `link_f` = {weight, ASEG oid}) and the SM's nested objects as (object, oid). Transitions
    /// whose ASEG oid is not a nested object are dropped (oid -1 = no ASEG is kept).
    pub fn ee_sub_1efac0(pairs: &[(i32, i32)], links: &[[i32; 4]], link_f: &[(f32, i32)], nested: &[(usize, i32)]) -> Sm {
        // 0x001efac0
        let mut states: Vec<(i32, i32)> = pairs.to_vec();
        let labels = states.clone();
        for s in &mut states {
            s.1 = match s.1 {
                -1 => -1,
                0x20 => -2,
                l => ee_sub_1f0400(&labels, l),
            };
        }
        let mut trans = vec![];
        let mut all = 0u32;
        for (l, f) in links.iter().zip(link_f) {
            let aseg = if f.1 == -1 {
                None
            } else {
                match nested.iter().find(|n| n.1 == f.1) {
                    Some(n) => Some(n.0),
                    None => continue,
                }
            };
            let t = Transition { aseg, from: ee_sub_1f0400(&labels, l[0]), to: ee_sub_1f0400(&labels, l[1]), mask: l[2] as u32, f10: l[3], weight: f.0 };
            all |= t.mask & 0xfffc;
            trans.push(t);
        }
        // mask groups: for each bit 4..0x1000 not yet grouped, the bits always set together with it
        let mut gm: Vec<u32> = vec![];
        let mut bit = 4u32;
        while bit <= 0x1000 {
            if bit & all != 0 && !gm.iter().any(|g| g & bit != 0) {
                let mut g = all;
                for t in &trans {
                    if t.mask & bit != 0 {
                        g &= t.mask;
                    }
                }
                for t in &trans {
                    if t.mask & g != 0 && t.mask & bit == 0 {
                        g &= !t.mask;
                    }
                }
                gm.push(g);
            }
            bit <<= 1;
        }
        // hop counts per group (breadth-first; bit 0 transitions also count backwards)
        let n = states.len();
        let groups = gm
            .into_iter()
            .map(|g| {
                let mut tab: Vec<i8> = (0..n * n).map(|i| if i / n == i % n { 0 } else { -1 }).collect();
                let mut d = 1i32;
                loop {
                    let mut changed = false;
                    for t in trans.iter().filter(|t| t.mask & g != 0) {
                        let (f, to) = (t.from as usize, t.to as usize);
                        if t.from < 0 || t.to < 0 {
                            continue;
                        }
                        for r in 0..n {
                            let row = r * n;
                            if tab[row + f] as i32 == d - 1 && tab[row + to] < 0 {
                                tab[row + to] = d as i8;
                                changed = true;
                            } else if t.mask & 1 != 0 && tab[row + to] as i32 == d - 1 && tab[row + f] < 0 {
                                tab[row + f] = d as i8;
                                changed = true;
                            }
                        }
                    }
                    d += 1;
                    if !changed {
                        break;
                    }
                }
                (g, tab.into_iter().map(|x| x as u8).collect())
            })
            .collect();
        Sm { states, trans, groups }
    }

    /// FUN_001f0498(sm, mask, a, b): hops from state a to b in the first group matching `mask`
    /// (FUN_001f0458); 0 when no group matches.
    pub fn ee_sub_1f0498(&self, mask: u32, a: i32, b: i32) -> i32 {
        // 0x001f0498
        let n = self.states.len() as i32;
        match self.groups.iter().find(|g| g.0 & mask != 0) {
            Some(g) if a >= 0 && b >= 0 => g.1[(a * n + b) as usize] as i32,
            _ => 0,
        }
    }

    /// FUN_001f0ad0: weighted random pick (FUN_0011d448(0, total)).
    pub fn ee_sub_1f0ad0(&self, rng: &mut GameRng, cands: &[usize]) -> Option<usize> {
        // 0x001f0ad0
        let total: f32 = cands.iter().map(|&c| self.trans[c].weight).sum();
        let mut r = rng.FUN_0011d448(0.0, total);
        for &c in cands {
            r -= self.trans[c].weight;
            if r <= 0.0 {
                return Some(c);
            }
        }
        None
    }

    /// FUN_001f0b80(sm, mask, cur, goal): a transition out of `cur` that brings `goal` one hop
    /// closer (cur == goal: a self loop).
    pub fn ee_sub_1f0b80(&self, rng: &mut GameRng, mask: u32, cur: i32, goal: i32) -> Option<usize> {
        // 0x001f0b80
        let d = if cur == goal { 1 } else { self.ee_sub_1f0498(mask, cur, goal) };
        let cands: Vec<usize> = (0..self.trans.len())
            .filter(|&i| {
                let t = &self.trans[i];
                t.mask & mask != 0
                    && ((t.from == cur && self.ee_sub_1f0498(mask, t.to, goal) == d - 1)
                        || (t.mask & 1 != 0 && t.to == cur && self.ee_sub_1f0498(mask, t.from, goal) == d - 1))
            })
            .collect();
        if cands.is_empty() {
            None
        } else {
            self.ee_sub_1f0ad0(rng, &cands)
        }
    }

    /// FUN_001f0cf0(sm, mask, cur): any transition out of `cur`.
    pub fn ee_sub_1f0cf0(&self, rng: &mut GameRng, mask: u32, cur: i32) -> Option<usize> {
        // 0x001f0cf0
        let cands: Vec<usize> = (0..self.trans.len())
            .filter(|&i| {
                let t = &self.trans[i];
                t.mask & mask != 0 && (t.from == cur || (t.mask & 1 != 0 && t.to == cur))
            })
            .collect();
        if cands.is_empty() {
            None
        } else {
            self.ee_sub_1f0ad0(rng, &cands)
        }
    }
}

/// FUN_001f13a8 message id: SMA state change, sent to the owner and subscribers (vtable slot 4).
pub const MSGID_SMA_STATE: i32 = 10;
/// FUN_001f13a8 message id of the level-script callback (FUN_001331f8).
pub const MSGID_SMA_SCRIPT: i32 = 8;

/// FUN_001f13a8 data block {sma, old label, new label} (-1: no state).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SmaMsg {
    pub old: i32,
    pub new: i32,
}

/// A receiver of SMA messages (the object's vtable slot 4 message handler).
pub trait SmaReceiver {
    fn ee_vt_message(&mut self, msg: i32, m: &SmaMsg);
}
impl<F: FnMut(i32, &SmaMsg)> SmaReceiver for F {
    fn ee_vt_message(&mut self, msg: i32, m: &SmaMsg) {
        self(msg, m)
    }
}

/// FUN_001f04f8(sm, i): label of state `i`, -1 outside the table.
pub fn ee_sub_1f04f8(sm: &Sm, i: i32) -> i32 {
    // 0x001f04f8
    usize::try_from(i).ok().and_then(|i| sm.states.get(i)).map_or(-1, |s| s.0)
}

/// ASEGA started by the SMA (FUN_0012f9b8 arguments).
#[derive(Clone, Debug, PartialEq)]
pub struct SmaPlay {
    pub aseg: usize,
    pub start: f32,
    pub svt: f32,
    pub flags: u32,
}

/// SMA (FUN_001f01b8).
#[derive(Clone, Debug, PartialEq)]
pub struct Sma {
    /// +0x2c
    pub flags: u32,
    /// +0x30: the playing transition ASEGA
    pub asega: Option<SmaPlay>,
    /// +0x34 / +0x38 / +0x3c
    pub cur: i32,
    pub next: i32,
    pub goal: i32,
    /// +0x40
    pub trans: Option<usize>,
    /// +0x44
    pub svt: f32,
    /// +0x48
    pub mask: u32,
    /// FUN_001f13a8 state changes (old, new), drained by the caller
    pub changes: Vec<(i32, i32)>,
    /// CRAWL R50: instant transitions of FUN_001f0d88 (finish / zero length): FUN_0012f9b8 with flags & 2 runs
    /// FUN_00131100 (the state events, e.g. collision part links 0xf / 0x10) before the inline FUN_00130b18;
    /// drained by the caller, which starts + retracts them
    pub instant: Vec<SmaPlay>,
}

impl Sma {
    /// FUN_001f01b8(sm, actor, label, flags, mask): new SMA in state `label` (-1: none), then
    /// FUN_001f0d88(sma, 0). `length` = ASEG +0x74 of a transition's ASEG object.
    pub fn ee_sub_1f01b8(sm: &Sm, rng: &mut GameRng, label: i32, flags: u32, mask: u32, length: &dyn Fn(usize) -> f32) -> Sma {
        // 0x001f01b8
        let mut s = Sma { flags, asega: None, cur: 0, next: -1, goal: 0, trans: None, svt: 1.0, mask, changes: vec![], instant: vec![] };
        if label != -1 {
            let i = ee_sub_1f0400(&sm.states, label);
            s.cur = i;
            s.goal = i;
            s.changes.push((-1, i));
        }
        s.ee_sub_1f0d88(sm, rng, false, length);
        s
    }

    /// FUN_001f0d88(sma, finish): walk towards the goal until a transition ASEG is playing.
    /// `finish` (1): the transition is jumped to its end (flags & ~1 | 2, instant).
    pub fn ee_sub_1f0d88(&mut self, sm: &Sm, rng: &mut GameRng, finish: bool, length: &dyn Fn(usize) -> f32) {
        // 0x001f0d88
        let mut finish = finish;
        // guard: ASEG-less cycles would loop forever in the original
        for _ in 0..256 {
            let t = if self.cur == self.goal {
                let Some(&(_, nx)) = usize::try_from(self.cur).ok().and_then(|c| sm.states.get(c)) else { return };
                if nx == -1 {
                    return;
                }
                let t = if nx == -2 {
                    let t = sm.ee_sub_1f0cf0(rng, self.mask, self.cur);
                    if let Some(t) = t {
                        let tr = &sm.trans[t];
                        self.goal = if tr.from == self.cur { tr.to } else { tr.from };
                    }
                    t
                } else {
                    self.goal = nx;
                    sm.ee_sub_1f0b80(rng, self.mask, self.cur, nx)
                };
                finish = false;
                t
            } else {
                sm.ee_sub_1f0b80(rng, self.mask, self.cur, self.goal)
            };
            self.trans = t;
            let Some(t) = t else { return };
            let tr = &sm.trans[t];
            let mut instant = false;
            if let Some(a) = tr.aseg {
                let len = length(a);
                let (mut start, svt) = if tr.from == self.cur {
                    self.next = tr.to;
                    (0.0, self.svt)
                } else {
                    self.next = tr.from;
                    (len, -self.svt)
                };
                let mut flags = self.flags;
                if finish {
                    flags = (flags & !1) | 2;
                    instant = true;
                    start = len - start;
                } else if len == 0.0 {
                    instant = true;
                }
                self.asega = Some(SmaPlay { aseg: a, start, svt, flags });
            }
            let (old, new);
            match self.asega {
                None => {
                    old = self.cur;
                    new = if tr.from == self.cur { tr.to } else { tr.from };
                    self.cur = new;
                }
                Some(_) if !instant => return, // FUN_001333c0: notify on the ASEGA's end
                Some(_) => {
                    old = self.cur;
                    new = self.next;
                    self.cur = new;
                    self.next = -1;
                    // FUN_0012f9b8 (FUN_00131100) + FUN_00130b18; bounded for SMA users that never drain it
                    let p = self.asega.take();
                    if self.instant.len() < 8 {
                        self.instant.extend(p);
                    }
                }
            }
            self.changes.push((old, new));
        }
    }

    /// FUN_001f1350(sma, mask): +0x48 = mask; a playing transition whose +0x10 bits meet the new mask is
    /// cut (FUN_001f08d0: ASEGA retract + FUN_001f0fa0) and the walk restarts (FUN_001f0d88(sma, 0)).
    /// Returns whether the transition was cut.
    pub fn ee_sub_1f1350(&mut self, sm: &Sm, rng: &mut GameRng, mask: u32, length: &dyn Fn(usize) -> f32) -> bool {
        // 0x001f1350
        self.mask = mask;
        let hit = self.asega.is_some() && self.trans.is_some_and(|t| sm.trans[t].f10 as u32 & mask != 0);
        if hit {
            self.ee_sub_1f0fa0();
            self.ee_sub_1f0d88(sm, rng, false, length);
        }
        hit
    }

    /// FUN_001f0fa0: the transition ASEGA is done; enter its target state.
    pub fn ee_sub_1f0fa0(&mut self) {
        // 0x001f0fa0
        self.asega = None;
        let (old, new) = (self.cur, self.next);
        self.next = -1;
        self.cur = new;
        self.changes.push((old, new));
    }

    /// FUN_001f13a8(sma, old, new): state-change signal. Data block {sma, FUN_001f04f8(old),
    /// FUN_001f04f8(new)} (labels); SMA slot 3 FUN_001f0fd8 sends message 10 to the owner (+0x28,
    /// vtable slot 4) and then FUN_00133508 to every subscriber (+0x8 list, slot 4); SMA slot 2
    /// FUN_001331f8(8, 0, 2 args {old, new}, 0) calls the level-script binding of message 8
    /// (`script`, FUN_00127db8 lookup in the SMA's script frame). The FUN_001f0fd8 lock/unlock
    /// (slots 0/1 refcount) is not needed here.
    pub fn ee_sub_1f13a8(
        sm: &Sm,
        old: i32,
        new: i32,
        owner: Option<&mut dyn SmaReceiver>,
        subs: &mut [&mut dyn SmaReceiver],
        script: Option<&mut dyn SmaReceiver>,
    ) -> SmaMsg {
        // 0x001f13a8
        let m = SmaMsg { old: ee_sub_1f04f8(sm, old), new: ee_sub_1f04f8(sm, new) };
        // 0x001f0fd8
        if let Some(o) = owner {
            o.ee_vt_message(MSGID_SMA_STATE, &m);
        }
        // 0x00133508
        for s in subs.iter_mut() {
            s.ee_vt_message(MSGID_SMA_STATE, &m);
        }
        // 0x001331f8
        if let Some(s) = script {
            s.ee_vt_message(MSGID_SMA_SCRIPT, &m);
        }
        m
    }

    /// Deliver the queued state changes (FUN_001f13a8 for each, in order). The port records the
    /// changes while the SMA steps and sends them afterwards (REPLACED: the original sends from inside
    /// FUN_001f01b8 / FUN_001f0d88 / FUN_001f0fa0); receivers that re-enter the SMA do so after it
    /// settled.
    pub fn dispatch(&mut self, sm: &Sm, mut owner: Option<&mut dyn SmaReceiver>, subs: &mut [&mut dyn SmaReceiver]) -> Vec<SmaMsg> {
        let mut out = vec![];
        for (old, new) in std::mem::take(&mut self.changes) {
            let o: Option<&mut dyn SmaReceiver> = match owner {
                Some(ref mut r) => Some(&mut **r),
                None => None,
            };
            out.push(Sma::ee_sub_1f13a8(sm, old, new, o, subs, None));
        }
        out
    }

    /// FUN_001f0660(sma, label): new goal state (-1: none). Idle SMA (no transition ASEGA) walks at
    /// once (FUN_001f0d88); a playing transition whose mask has bit 1 is cut (FUN_00130b18 -> ASEGA
    /// end -> FUN_001f1078). The FUN_001f0638 gate (non-zero: SMA busy) is not modelled.
    pub fn ee_sub_1f0660(&mut self, sm: &Sm, rng: &mut GameRng, label: i32, length: &dyn Fn(usize) -> f32) {
        // 0x001f0660
        let i = if label == -1 { -1 } else { ee_sub_1f0400(&sm.states, label) };
        if i == self.goal {
            return;
        }
        self.goal = i;
        if self.asega.is_none() {
            self.ee_sub_1f0d88(sm, rng, false, length);
        } else if self.trans.is_some_and(|t| sm.trans[t].mask & 2 != 0) {
            self.ee_sub_1f1078(sm, rng, length);
        }
    }

    /// FUN_001f1078(sma, 2, asega): ASEGA end notification -> FUN_001f0fa0 + FUN_001f0d88(sma, 0).
    pub fn ee_sub_1f1078(&mut self, sm: &Sm, rng: &mut GameRng, length: &dyn Fn(usize) -> f32) {
        // 0x001f1078
        if self.asega.is_some() {
            self.ee_sub_1f0fa0();
            self.ee_sub_1f0d88(sm, rng, false, length);
        }
    }
}
