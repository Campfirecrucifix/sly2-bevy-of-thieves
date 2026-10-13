//! WP-P26: pickpocket loot (class 103 C2cbba0, size 0x190) and the guard side of the pickpocket
//! (class 37 C2cb9f8 slot 50).
//!
//! Loot object fields: +0x150 / +0x154 loot def rows (0x2c bytes: [0] id, [1] mplootkc key, [3] fx flag,
//! [4] allowed in a vault, [5] kind, [6] mesh), +0x158 current def, +0x15c guard (carrier), +0x160 thief (JT),
//! +0x180 spin, +0x184 state, +0x188 state time, +0x18c flight fx. Guard: +0x184 default loot id, +0x188 kind,
//! +0x190 alert, +0x194 carried loot. JT: +0x11a4 loot anchor object, +0x11a8 held loot.
//! States: 0 free, 1 on the guard, 2 flying guard -> thief, 3 flying thief -> guard, 4 held by the thief,
//! 5 dropped (physics, 5 s back to the guard), 6 given (no flight).
//!
//! REPLACED: the loot pool DAT_003e1ac8 (FUN_001cb250) = the free (state 0) class-103 objects of the world;
//! the flight fx (FxQueue_push(6), +0x18 duration = max(dist, 250) * 0.002) = a timer that sends message 8 to
//! the loot when it runs out (as the fx does on arrival); engine calls (attach FUN_001a58d0, meshes, sounds,
//! vt messages, physics FUN_001f5720 / 0x1f57c0 / 0x1f5880, activity FollowWorld) are recorded in `calls`.
//! The save variables ("oidBooty" per JT, the per-guard loot path FUN_0013d1a8, "mplootkc") are maps.
use std::collections::BTreeMap;

pub const CLASS_GUARD: u16 = 37;
pub const CLASS_LOOT: u16 = 103;
/// FUN_0013c560 state 2 / 3: flight time per unit (min distance 250).
pub const FX_PER_UNIT: f32 = 0.002;
/// C2cbba0_v15: dropped loot returns to its guard after 5 s.
pub const DROP_RETURN: f32 = 5.0;

/// One 0x2c-byte loot def row (+0x154).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LootDef {
    pub id: i32,
    /// [1] +4: mplootkc key (-1 = not counted)
    pub key: i32,
    /// [3] +0xc: timed loot (fx / drop at DAT_003e1bfc)
    pub timed: i32,
    /// [4] +0x10: may be stolen inside a vault (DAT_003e0f9c)
    pub allowed: i32,
    /// [5] +0x14: kind (must match guard +0x188 to swap)
    pub kind: i32,
    /// [6] +0x18: mesh (-1 = none)
    pub mesh: i32,
}

#[derive(Clone, Debug, Default)]
pub struct Loot {
    pub obj: u32,
    pub defs: Vec<LootDef>,
    pub def: Option<usize>,
    pub guard: u32,
    pub thief: u32,
    pub spin: f32,
    pub state: i32,
    pub t: f32,
    /// +0x18c: fx handle (REPLACED timer id) and its arrival time
    pub fx: u32,
    pub fx_end: f32,
    /// pool record +8 (RECORDED_1cb488 / 1cb4b8)
    pub active: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Guard {
    pub obj: u32,
    /// +0x180: index of the guard's save path
    pub idx: i32,
    pub booty: i32,
    pub kind: i32,
    pub alert: i32,
    pub loot: Option<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct LootSys {
    pub loots: Vec<Loot>,
    pub guards: Vec<Guard>,
    /// JT +0x11a8 held loot / +0x11a4 anchor (default the JT itself)
    pub held: BTreeMap<u32, usize>,
    pub anchor: BTreeMap<u32, u32>,
    /// save "oidBooty" per JT oid
    pub oid_booty: BTreeMap<u32, i32>,
    /// per-guard save path (FUN_0013d1a8: guard +0x180); 0 = unset
    pub guard_save: BTreeMap<i32, i32>,
    /// save "mplootkc" counts
    pub mplootkc: BTreeMap<i32, i32>,
    /// DAT_002dded0, DAT_003e1bfc (timed loot deadline), DAT_003e0f9c (vault) set
    pub now: f32,
    pub t_3e1bfc: f32,
    pub vault: bool,
    /// recorded engine calls (address / vt offset, object, argument)
    pub calls: Vec<(u32, u32, i32)>,
    fx_seq: u32,
}

impl LootSys {
    /// Builds the system from a world: class-103 objects form the pool, class-37 objects are the guards.
    pub fn from_objects(objs: impl IntoIterator<Item = (u32, u16)>, defs: &[LootDef]) -> Self {
        let mut s = Self::default();
        for (h, c) in objs {
            if c == CLASS_LOOT {
                s.loots.push(Loot { obj: h, defs: defs.to_vec(), ..Default::default() });
            } else if c == CLASS_GUARD {
                let idx = s.guards.len() as i32;
                s.guards.push(Guard { obj: h, idx, ..Default::default() });
            }
        }
        s
    }

    fn guard_of(&self, obj: u32) -> Option<usize> {
        self.guards.iter().position(|g| g.obj == obj)
    }

    fn def(&self, l: usize) -> Option<LootDef> {
        self.loots[l].def.map(|d| self.loots[l].defs[d])
    }

    /// 0x0013c478 FUN_0013c478: id of the current def (-1 = none).
    pub fn ee_sub_13c478(&self, l: usize) -> i32 {
        self.def(l).map_or(-1, |d| d.id)
    }

    /// 0x0013c490 FUN_0013c490: select the def with `id`, else the first row.
    pub fn ee_sub_13c490(&mut self, l: usize, id: i32) {
        let lo = &mut self.loots[l];
        lo.def = lo.defs.iter().position(|d| d.id == id).or(if lo.defs.is_empty() { None } else { Some(0) });
    }

    /// 0x001cb250 FUN_001cb250 (REPLACED): a free loot of the pool.
    pub fn ee_sub_1cb250(&self) -> Option<usize> {
        self.loots.iter().position(|l| l.state == 0 && l.def.is_none() && !l.active)
    }

    fn fx(&mut self, l: usize, from: [f32; 3], to: [f32; 3]) {
        let d = {
            let v = [from[0] - to[0], from[1] - to[1], from[2] - to[2]];
            (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
        };
        self.fx_seq += 1;
        let lo = &mut self.loots[l];
        lo.fx = self.fx_seq;
        lo.fx_end = self.now + if d < 250.0 { 250.0 } else { d } * FX_PER_UNIT;
    }

    fn anchor_of(&self, jt: u32) -> u32 {
        self.anchor.get(&jt).copied().unwrap_or(jt)
    }

    /// 0x0013c560 FUN_0013c560: loot SetState. `pos` = FUN_001a62e8 world position of an object.
    pub fn ee_sub_13c560(&mut self, l: usize, s: i32, pos: &dyn Fn(u32) -> [f32; 3]) {
        let prev = self.loots[l].state;
        if prev == s {
            return;
        }
        let obj = self.loots[l].obj;
        if prev < 2 && 1 < s {
            self.calls.push((0x001cb488, obj, 1));
            self.loots[l].active = true;
        } else if prev >= 2 && s <= 1 {
            self.calls.push((0x001cb4b8, obj, 0));
            self.loots[l].active = false;
        }
        let thief = self.loots[l].thief;
        let def = self.def(l).unwrap_or_default();
        match prev {
            0 => {
                self.calls.push((0x001f6a68, obj, 0));
                if def.mesh != -1 {
                    self.calls.push((0x001f6ac8, obj, def.mesh));
                }
            }
            1 => {
                let g = self.loots[l].guard;
                if def.timed != 0 {
                    self.calls.push((0x001618f8, g, 0));
                }
                self.calls.push((0x00156c80, g, 1));
                if s == 2 {
                    self.calls.push((0x2f, obj, 0)); // FollowWorld activity 0x2f
                }
                if let Some(gi) = self.guard_of(g) {
                    self.guards[gi].loot = None;
                }
                self.loots[l].guard = 0;
                self.calls.push((0x1c, obj, 5));
            }
            2 => {
                self.loots[l].fx = 0;
                if s == 4 {
                    self.loots[l].spin = 0.3;
                }
            }
            3 => self.loots[l].fx = 0,
            4 => {
                self.loots[l].thief = 0;
                self.calls.push((0x4c, self.anchor_of(thief), 0));
                self.held.remove(&thief);
                if def.mesh != -1 {
                    self.calls.push((0x001f6ac8, obj, def.mesh));
                }
                if def.timed != 0 {
                    self.calls.push((0x00161958, obj, 0));
                    }
            }
            5 => {
                let g = self.loots[l].guard;
                if g != 0 {
                    self.calls.push((0x19c, g, 1));
                }
                self.calls.push((0x001f5720, obj, 1));
                self.calls.push((0x001f57c0, obj, 1));
                self.calls.push((0x001f5880, obj, 0));
            }
            6 => self.loots[l].spin = 0.0,
            _ => {}
        }
        self.loots[l].state = s;
        self.loots[l].t = self.now;
        match s {
            0 => {
                self.loots[l].def = None;
                self.calls.push((0x001a58d0, obj, 0));
                self.calls.push((0x4c, obj, 0));
            }
            1 => {
                let g = self.loots[l].guard;
                self.calls.push((0x00156c80, g, -1));
                self.calls.push((0x001a58d0, obj, g as i32));
                if let Some(gi) = self.guard_of(g) {
                    self.guards[gi].loot = Some(l);
                }
                if prev != 3 {
                    self.calls.push((0x4c, obj, 0));
                }
                self.calls.push((0x1c, obj, 6));
            }
            2 | 6 => {
                let thief = self.loots[l].thief;
                let anc = self.anchor_of(thief);
                self.calls.push((0x44, anc, 0));
                if s == 2 {
                    self.fx(l, pos(obj), pos(anc));
                    self.calls.push((0x44, obj, 0));
                } else {
                    self.loots[l].fx = 0;
                    self.calls.push((0x1c, obj, 6));
                    self.calls.push((0x1c, obj, 5));
                }
                self.calls.push((0x001a58d0, obj, anc as i32));
                if self.loots[l].fx == 0 {
                    self.ee_sub_13c560(l, 4, pos);
                }
            }
            3 => {
                let g = self.loots[l].guard;
                self.calls.push((0x001a58d0, obj, g as i32));
                let anc = self.anchor_of(thief);
                self.fx(l, pos(anc), pos(obj));
                self.calls.push((0x44, obj, 0));
                // the fx always exists in the port (FxQueue_push failing -> SetState(1))
            }
            4 => {
                self.held.insert(self.loots[l].thief, l);
                if def.mesh != -1 {
                    self.calls.push((0x001f6b48, obj, def.mesh));
                }
            }
            5 => {
                // detach, physics on, velocity = vt+0xfc dir * 1000, random spin in (-2pi, 2pi), activity 0x30
                self.calls.push((0x001a58d0, obj, 0));
                self.calls.push((0x001f5720, obj, 0));
                self.calls.push((0x001f57c0, obj, 0));
                self.calls.push((0x11c, obj, 1000));
                self.calls.push((0x124, obj, 0));
                self.calls.push((0x001f5880, obj, 0));
                self.calls.push((0x30, obj, 0));
            }
            _ => {}
        }
    }

    /// 0x0013c1b8 C2cbba0_v4: message `msg` (4 = collected while dropped, 8 = fx `arg` arrived).
    pub fn ee_sub_13c1b8(&mut self, l: usize, msg: i32, arg: u32, pos: &dyn Fn(u32) -> [f32; 3]) {
        let st = self.loots[l].state;
        if msg == 4 {
            if st == 5 {
                self.ee_sub_13c560(l, 0, pos);
            }
        } else if msg == 8 && arg == self.loots[l].fx {
            if st == 2 {
                self.ee_sub_13c560(l, 4, pos);
            } else if st == 3 {
                self.ee_sub_13c560(l, 1, pos);
            }
        }
    }

    /// 0x0013bf50 C2cbba0_v15 (per-frame update) for every loot, plus the REPLACED fx timers (message 8).
    pub fn update(&mut self, now: f32, pos: &dyn Fn(u32) -> [f32; 3], alive: &dyn Fn(u32) -> bool) {
        self.now = now;
        for l in 0..self.loots.len() {
            let lo = &self.loots[l];
            if lo.fx != 0 && (lo.state == 2 || lo.state == 3) && now >= lo.fx_end {
                let fx = lo.fx;
                self.ee_sub_13c1b8(l, 8, fx, pos);
            }
            let lo = &self.loots[l];
            if lo.state == 4 {
                if self.def(l).is_some_and(|d| d.timed != 0) && self.t_3e1bfc <= now {
                    let th = lo.thief;
                    self.ee_sub_143380(th, pos);
                }
            } else if lo.state == 5 && DROP_RETURN <= now - lo.t {
                self.ee_sub_13c4e0(l, pos, alive);
            }
        }
    }

    /// 0x0013c4e0 FUN_0013c4e0: dropped loot goes back to its (live, FUN_001a61a0) guard or is freed.
    pub fn ee_sub_13c4e0(&mut self, l: usize, pos: &dyn Fn(u32) -> [f32; 3], alive: &dyn Fn(u32) -> bool) {
        if self.loots[l].state != 5 {
            return;
        }
        let g = std::mem::take(&mut self.loots[l].guard);
        let d = self.loots[l].def;
        self.ee_sub_13c560(l, 0, pos);
        if g != 0 && alive(g) {
            self.loots[l].guard = g;
            self.loots[l].def = d;
            self.ee_sub_13c560(l, 1, pos);
        }
    }

    /// 0x0013d248 FUN_0013d248: the guard's loot id from its save path (0 -> +0x184; 7..9 -> -1; remaps).
    pub fn ee_sub_13d248(&self, gi: usize) -> i32 {
        let mut v = self.guard_save.get(&self.guards[gi].idx).copied().unwrap_or(0);
        if v == 0 {
            v = self.guards[gi].booty;
        } else if (7..10).contains(&v) {
            v = -1;
        }
        match v {
            0x2c0 => 0x2bd,
            0x2bf => 700,
            0x2c1 => 0x2be,
            v => v,
        }
    }

    /// 0x0013cf38 FUN_0013cf38: give the guard its loot from the pool.
    pub fn ee_sub_13cf38(&mut self, gi: usize, pos: &dyn Fn(u32) -> [f32; 3]) {
        if self.guards[gi].loot.is_some() {
            return;
        }
        let id = self.ee_sub_13d248(gi);
        if id == -1 {
            return;
        }
        if let Some(l) = self.ee_sub_1cb250() {
            self.guards[gi].loot = Some(l);
            self.ee_sub_13c490(l, id);
            self.loots[l].guard = self.guards[gi].obj;
            self.ee_sub_13c560(l, 1, pos);
        }
    }

    /// 0x0013d328 FUN_0013d328: set the guard's saved loot id (`v`), refill when != -1.
    pub fn ee_sub_13d328(&mut self, gi: usize, v: i32, pos: &dyn Fn(u32) -> [f32; 3]) {
        if let Some(l) = self.guards[gi].loot {
            self.ee_sub_13c560(l, 0, pos);
        }
        self.guard_save.insert(self.guards[gi].idx, v);
        if v != -1 {
            self.ee_sub_13cf38(gi, pos);
        }
    }

    /// 0x0013d020 C2cb9f8_v50: the JT `jt` (FUN_00132b10 class 0x34 subtree, `is_jt`) pickpockets guard `g`.
    /// Returns true when loot changed hands.
    pub fn ee_sub_13d020(&mut self, g: u32, jt: u32, is_jt: bool, pos: &dyn Fn(u32) -> [f32; 3]) -> bool {
        let Some(gi) = self.guard_of(g) else { return false };
        if !is_jt {
            return false;
        }
        let held = self.held.get(&jt).copied();
        let gl = self.guards[gi].loot;
        if held.is_none() && gl.is_none() {
            return false;
        }
        if let Some(gl) = gl {
            if self.vault && self.def(gl).is_some_and(|d| d.allowed == 0) {
                // FUN_001da4b8(15.0, hint 0x124) + sound emitter 0x36
                self.calls.push((0x001da4b8, g, 0x124));
                self.calls.push((0x36, g, 0));
                return false;
            }
        }
        let kind_ok = held.is_none_or(|h| self.def(h).map_or(0, |d| d.kind) == self.guards[gi].kind);
        if self.guards[gi].alert != 0 || !kind_ok {
            self.calls.push((0x32, g, 0)); // FollowWorld activity 0x32
            return false;
        }
        if gl.is_some() {
            self.calls.push((0x27, g, 0)); // sound emitter 0x27
        }
        self.ee_sub_143678(jt, 0, Some(gi), pos);
        match (held, gl) {
            (None, Some(gl)) => {
                self.loots[gl].thief = jt;
                self.ee_sub_13c560(gl, 2, pos);
            }
            (Some(h), None) => {
                self.loots[h].guard = g;
                self.ee_sub_13c560(h, 3, pos);
            }
            (Some(h), Some(gl)) => {
                self.loots[h].guard = g;
                self.ee_sub_13c560(h, 3, pos);
                self.loots[gl].thief = jt;
                self.ee_sub_13c560(gl, 2, pos);
            }
            (None, None) => {}
        }
        self.calls.push((0x50, g, jt as i32)); // parent C2d1ee8_v50
        true
    }

    /// 0x00143410 FUN_00143410: the guard save path whose value is `oid` (index), if any.
    fn ee_sub_143410(&self, oid: i32) -> Option<i32> {
        self.guard_save.iter().find(|(_, v)| **v == oid).map(|(k, _)| *k)
    }

    /// 0x00143678 FUN_00143678(jt, mode, guard): the loot save variables. Mode 0 = pickpocket of `guard`,
    /// 1 = loot delivered (counted in "mplootkc"), 2 = loot dropped (guard path back to the loot id; the loot
    /// returns to the class-37 guard with that path index).
    pub fn ee_sub_143678(&mut self, jt: u32, mode: i32, guard: Option<usize>, _pos: &dyn Fn(u32) -> [f32; 3]) {
        let oid = jt as i32;
        let held = self.held.get(&jt).copied();
        match mode {
            1 => {
                if let Some(p) = self.ee_sub_143410(oid) {
                    self.guard_save.insert(p, -1);
                }
                self.oid_booty.insert(jt, -1);
                if let Some(d) = held.and_then(|h| self.def(h)) {
                    if d.key != -1 {
                        *self.mplootkc.entry(d.key).or_insert(0) += 1;
                    }
                }
            }
            0 => {
                let Some(gi) = guard else { return };
                let v = self.guards[gi].loot.map_or(-1, |l| self.ee_sub_13c478(l));
                self.oid_booty.insert(jt, v);
                let idx = self.guards[gi].idx;
                match held {
                    None => {
                        self.guard_save.insert(idx, oid);
                    }
                    Some(h) => {
                        if self.guards[gi].loot.is_none() {
                            if let Some(p) = self.ee_sub_143410(oid) {
                                self.guard_save.insert(p, -1);
                            }
                        }
                        let id = self.ee_sub_13c478(h);
                        self.guard_save.insert(idx, id);
                    }
                }
            }
            2 => {
                let Some(h) = held else { return };
                let id = self.ee_sub_13c478(h);
                // FUN_00143af0: path of the JT -> id, oidBooty -1
                let p = self.ee_sub_143410(oid);
                if let Some(p) = p {
                    self.guard_save.insert(p, id);
                }
                self.oid_booty.insert(jt, -1);
                if let Some(p) = p {
                    if let Some(g) = self.guards.iter().find(|g| g.idx == p) {
                        self.loots[h].guard = g.obj;
                    }
                }
            }
            _ => {}
        }
    }

    /// 0x00143380 FUN_00143380: the JT drops its loot (hit, timed loot run out, splice 0x10b140).
    pub fn ee_sub_143380(&mut self, jt: u32, pos: &dyn Fn(u32) -> [f32; 3]) {
        if let Some(h) = self.held.get(&jt).copied() {
            self.ee_sub_143678(jt, 2, None, pos);
            self.ee_sub_13c560(h, 5, pos);
        }
    }

    /// 0x001433c8 FUN_001433c8: the JT delivers its loot (FUN_001d7480 level / vault end).
    pub fn ee_sub_1433c8(&mut self, jt: u32, pos: &dyn Fn(u32) -> [f32; 3]) {
        if let Some(h) = self.held.get(&jt).copied() {
            self.ee_sub_143678(jt, 1, None, pos);
            self.ee_sub_13c560(h, 0, pos);
        }
    }

    /// 0x001432e0 FUN_001432e0: give the JT loot `id` (-1 = just drop the current one).
    pub fn ee_sub_1432e0(&mut self, jt: u32, id: i32, pos: &dyn Fn(u32) -> [f32; 3]) {
        if self.held.get(&jt).is_some_and(|&h| self.ee_sub_13c478(h) == id) {
            return;
        }
        self.ee_sub_143380(jt, pos);
        if id == -1 {
            return;
        }
        if let Some(l) = self.ee_sub_1cb250() {
            self.ee_sub_13c490(l, id);
            self.loots[l].thief = jt;
            self.ee_sub_13c560(l, 6, pos);
            self.ee_sub_13c560(l, 4, pos);
        }
    }

    /// The loot held by `jt` (JT +0x11a8) and its def id.
    pub fn held_by(&self, jt: u32) -> Option<(usize, i32)> {
        self.held.get(&jt).map(|&l| (l, self.ee_sub_13c478(l)))
    }
}

impl LootSys {
    /// WP-EP1a: the level-load state of the loot system from a class set: class-103 pool and class-37 holders
    /// (handles = object index + 1), the holders' K31 fields (+0x180 index, +0x184 default id, +0x188 kind,
    /// +0x190 alert), their field-1168 FUN_0013d328 calls (fed at load), then the slot-22 deferred call
    /// FUN_00149ab8(DAT_002a4bb0) = PMF {0, -1, 0x13cf38}: FUN_0013cf38 hands every holder its loot.
    pub fn from_class_set(classes: impl IntoIterator<Item = u16>, k31: &[(usize, crate::engine::classes::k31::C2cb9f8)], defs: &[LootDef], pos: &dyn Fn(u32) -> [f32; 3]) -> Self {
        let mut s = Self::from_objects(classes.into_iter().enumerate().map(|(i, c)| (i as u32 + 1, c)), defs);
        for g in &mut s.guards {
            if let Some((_, c)) = k31.iter().find(|x| x.0 as u32 + 1 == g.obj) {
                (g.idx, g.booty, g.kind, g.alert) = (c.f_180, c.f_184, c.f_188, c.f_190);
            }
        }
        for gi in 0..s.guards.len() {
            let calls = k31.iter().find(|x| x.0 as u32 + 1 == s.guards[gi].obj).map(|x| x.1.m_1168.clone()).unwrap_or_default();
            for v in calls {
                s.ee_sub_13d328(gi, v, pos);
            }
        }
        for gi in 0..s.guards.len() {
            s.ee_sub_13cf38(gi, pos);
        }
        s
    }
}
