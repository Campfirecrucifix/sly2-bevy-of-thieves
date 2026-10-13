//! Class 56 (guard base, abstract, 0x1010-byte objects; concrete 57-67): shared overrides
//! slot 6 FUN_00205eb8 (init), slot 25 FUN_00206518, slot 26 FUN_00206748, slot 15 FUN_00207350 (update),
//! mode FUN_00207030 / FUN_00207288, sensor pass FUN_00206900.
use super::brain::{Brain, Mind};
use super::patrol::{ee_sub_203fc0, ee_sub_204ca8_move, Body, Cmd, Cx};
use super::sensor::{See, Sensor, Tgt, Vol};
use super::super::ClassHost;
use crate::engine::world::props::{ArrayItem, Prop, PropValue};

/// behaviour bits that suppress sensing (FUN_00206900 block, FUN_00207a20 per bit)
/// FUN_00208d70 behaviour bits that close the stimulus gate (FUN_00207a20 per bit)
const GATE_208D70: u64 = (1 << 0xb) | (1 << 0x1d) | (1 << 7) | (1 << 5) | (1 << 6) | (1 << 0x10) | (1 << 1) | (1 << 0x16) | (1 << 0x2a);

const SENSE_BLOCK: u64 = (1 << 5) | (1 << 6) | (1 << 0x17) | (1 << 7) | (1 << 0x10) | (1 << 1) | (1 << 0x16) | (1 << 0x2a);

#[derive(Clone, Debug, Default)]
pub struct Guard {
    /// world class (57-67) / vtable
    pub class: u16,
    pub vt: u32,
    pub body: Body,
    /// +0x2c0 behaviour bits (FUN_00209890 / FUN_00209970 by the child goals)
    pub f_2c0: u64,
    /// +0x2cc / +0x2d0 routine timer range (DAT_002a8600)
    pub f_2cc: [f32; 2],
    /// +0x2dc saved script cursor
    pub f_2dc: Option<usize>,
    /// +0x2e8 routine (index into the level routines), +0x2ec home routine (FUN_002061d8)
    pub f_2e8: Option<usize>,
    pub f_2ec: Vec<Cmd>,
    /// +0x300 home position, +0x310 home yaw
    pub f_300: [f32; 3],
    pub f_310: f32,
    /// +0x314 / +0x318, +0x31c / +0x324 pairs, +0xf48 pair, +0xf50 / +0xf58 attack windows
    pub f_314: f32,
    pub f_318: f32,
    pub f_31c: [f32; 2],
    pub f_324: [f32; 2],
    pub f_f48: [f32; 2],
    pub f_f50: [f32; 2],
    pub f_f58: [f32; 2],
    /// +0x38c target, +0x390 forced target, +0x3a4 dynamic sensor entries, +0x394 sensed classes
    pub f_38c: u32,
    pub f_390: u32,
    pub f_3a4: usize,
    pub f_394: Vec<i32>,
    /// +0x3cc target knowledge record {+4 time, +8 level}
    pub f_3cc: Option<(f32, i32)>,
    /// +0xe64 hearing, +0xe68 min stimulus level, +0xe6c / +0xe70 dwell, +0xe78
    pub f_e64: i32,
    pub f_e68: i32,
    pub f_e6c: f32,
    pub f_e70: f32,
    pub f_e78: i32,
    /// +0xf64 sensor
    pub sensor: Sensor,
    /// +0xf80 (-1: script rail mode), +0xf94 / +0xfa8
    pub f_f80: i32,
    pub f_f94: i32,
    pub f_fa8: i32,
    /// +0xfbc mode, +0xfc0 mode start
    pub f_fbc: i32,
    pub f_fc0: f32,
    /// +0x26c goal stack: root brain (0x9c)
    pub brain: Brain,
    /// anim ids of the field-243 table (+0x4c0)
    pub anims: Vec<i32>,
    /// vision / outer volume oids (fields 315 / 316)
    pub vol_oids: [i32; 2],
    /// concrete-class init fields: +0x4a4 (59-61/63/64 slot 6), +0x364 / +0x1070..+0x1088 (60), DAT_002dd648 (63)
    pub f_4a4: i32,
    pub f_364: i32,
    pub f_1070: [f32; 3],
    pub f_1080: [f32; 3],
    pub dat_2dd648: bool,
    /// K7: +0x298 / +0x32c (immunity, slot 75), +0x338 (max health, field 254: lethal threshold of slot 85), +0x494 (slot 86 object),
    /// +0x49c +0xc tracker weapon (slot 4), +0xff8 (slots 21 / 22)
    pub f_298: i32,
    pub f_32c: i32,
    pub f_338: i32,
    pub f_494: u32,
    pub f_4a8: u32,
    pub f_ff8: f32,
    /// hits taken (slot 74 -> FUN_001eed78(+0x26c, 0xc, hit) goal message, consumed by the goal stack)
    pub f_damage: Vec<super::slots::GuardHit>,
    /// K22: the DAT_003e148c stimulus list this guard polls (REPLACED global pool view, fed by the caller)
    pub stimuli: Vec<super::slots::Stimulus>,
    /// K22: +0xfc8 alarm, +0xe74 leash object position
    pub f_fc8: i32,
    pub f_e74: Option<[f32; 3]>,
    /// K32 (replaces the K12 stun): hit reaction fields of FUN_00176be8 - +0x29c (field 232, no damage),
    /// +0x330 knocked-out once, +0x33c health (field 255; = +0x338 at slot 25), +0x340 / +0x344 armour
    /// (fields 256-258), +0x348 flinch anim (259), +0x350 / +0x354 / +0x358 kind masks of subs 9 / 10 / 0xb
    /// (249-251), +0x360 last flinch, +0x3d0 (333), +0xe84 / +0xe88 (294) kind-0x80000 meter
    pub f_29c: i32,
    pub f_330: i32,
    pub f_33c: i32,
    pub f_340: i32,
    pub f_344: i32,
    pub f_348: i32,
    pub f_350: u32,
    pub f_354: u32,
    pub f_358: u32,
    pub f_360: f32,
    pub f_3d0: i32,
    pub f_e84: i32,
    pub f_e88: i32,
    /// +0x8d0 / +0xa70 / +0xc10 hit-anim tables (empty = every record -1, `hit_anim`)
    pub hit_anims: Vec<[i32; 26]>,
    /// K32: hit effects FUN_00156698 played (REPLACED counter: npcdraw shows the hit star when it grows)
    pub hit_fx: u32,
    /// K11: +0xeb8..+0xf0c attack records / running attack
    pub atk: super::attack::Attacks,
    /// K21: +0x1000 (set to 1 by the class-119 spawner FUN_00174490)
    pub f_1000: i32,
    /// K24: +0x138 attachment records (owner handles; FUN_00229710 link / FUN_00229738 unlink)
    pub f_138: Vec<u32>,
    /// K24: REPLACED PO of a lifted guard (guard/lift.rs `GuardPo`; None = on its patrol body)
    pub po: Option<super::lift::GuardPo>,
    /// K24: removed from the world (vtable +0x4c FUN_001a5630): +0x28 bit 60 clear, no more updates
    pub removed: bool,
}

fn words(v: &PropValue) -> Vec<Vec<u32>> {
    let w = |b: &Vec<u8>| b.chunks_exact(4).map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]])).collect::<Vec<_>>();
    match v {
        PropValue::Scalar(b) => vec![w(b)],
        PropValue::Array(items) => items
            .iter()
            .map(|i| match i {
                ArrayItem::One(_, b) => w(b),
                ArrayItem::List(_, l) => l.iter().flat_map(w).collect(),
            })
            .collect(),
    }
}

impl Guard {
    /// slot 6 FUN_00205eb8 (with the class-51 FUN_00203948 part it chains: yaw 0, +0x260, task pool)
    pub fn ee_sub_205eb8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x203948, obj);
        self.f_fbc = -1;
        self.f_f80 = -1;
        self.f_2cc = [f32::MAX; 2];
        self.f_e6c = 5.5;
        self.f_e70 = 6.5;
        self.f_e78 = 0;
        self.f_314 = 800.0;
        self.f_318 = 1200.0;
        self.f_31c = [300.0, 450.0];
        self.f_324 = [300.0, 600.0];
        self.f_f48 = [300.0, 1200.0];
        self.f_f50 = [-0.785_398_2, 0.785_398_2];
        self.f_f58 = [-1.308_997, 1.308_997];
        self.f_e64 = 1;
        self.f_e68 = -1;
        self.body.f_180 = 1.0;
        // K32: hit reaction defaults
        self.f_350 = 0x20f800;
        self.f_354 = 0x29ff7e;
        self.f_358 = 0xf800;
        self.f_360 = -10.0;
        self.f_e84 = 50;
        self.f_364 = 1;
        // FUN_0013e3c8(0x10, 0, 0x288, -1): the sensor
        self.sensor = Sensor::default();
    }

    /// Slot 6 of the concrete class: FUN_00205eb8 then the class's own part (57 FUN_0016f148, 60 FUN_001b9f80,
    /// 61 FUN_001cbeb0, 63 FUN_0014b1a0, 64 FUN_0014fac8; 58/59/62/65/66 inherit slot 6)
    pub fn init(&mut self, obj: usize, h: &mut dyn ClassHost) {
        self.ee_sub_205eb8(obj, h);
        match self.class {
            57 => self.f_e78 = 1,
            60 => {
                self.f_1070 = [1.35, f32::from_bits(0xb99d_4952), 0.0];
                self.f_1080 = [0.9, 1.2, 7.0];
                self.f_364 = 0;
                self.f_e64 = 0;
                self.sensor.f_f8 = 3;
                self.f_4a4 = 1;
            }
            63 => {
                self.dat_2dd648 = true;
                self.f_4a4 = 1;
            }
            61 | 64 => self.f_4a4 = 1,
            _ => {}
        }
    }

    /// Load-time fields (FUN_00201360 property list)
    pub fn feed(&mut self, props: &[Prop]) {
        let f = |w: u32| f32::from_bits(w);
        for p in props {
            let a = words(&p.value);
            let w0 = a.first().and_then(|x| x.first().copied()).unwrap_or(0);
            let pair = || {
                let x = a.iter().flatten().copied().collect::<Vec<_>>();
                [f(x.first().copied().unwrap_or(0)), f(x.get(1).copied().unwrap_or(0))]
            };
            match p.field {
                243 => {
                    if let Some(id) = a.first().and_then(|x| x.first()) {
                        self.anims.push(*id as i32);
                    }
                }
                281 => self.f_e64 = w0 as i32,
                292 => self.f_314 = f(w0),
                293 => self.f_318 = f(w0),
                296 => self.f_31c = pair(),
                297 => self.f_324 = pair(),
                298 => self.f_f48[0] = f(w0),
                312 => self.sensor.f_f8 = w0,
                313 => self.sensor.f_f0 = w0 as i32,
                314 => self.sensor.f_f4 = w0 as i32,
                315 => self.vol_oids[0] = w0 as i32,
                316 => self.vol_oids[1] = w0 as i32,
                323 => self.f_e68 = w0 as i32,
                // K32: hit reaction fields (FIELDS offsets)
                232 => self.f_29c = w0 as i32,
                249 => self.f_350 = w0,
                250 => self.f_354 = w0,
                251 => self.f_358 = w0,
                254 => self.f_338 = w0 as i32,
                255 => self.f_33c = w0 as i32,
                256 | 257 => self.f_340 = w0 as i32,
                258 => self.f_344 = w0 as i32,
                259 => self.f_348 = w0 as i32,
                294 => self.f_e84 = w0 as i32,
                333 => self.f_3d0 = w0 as i32,
                338 => self.f_4a4 = w0 as i32,
                _ => {}
            }
        }
    }

    /// FUN_00209ad0(g, id): anim `id` in the table
    fn ee_sub_209ad0(&self, id: i32) -> bool {
        self.anims.contains(&id)
    }

    /// slot 14 FUN_002061d8: home routine +0x2ec = one kind-7 entry (FUN_00170c90(40.0, obj, -1, -1))
    pub fn ee_sub_2061d8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x203a38, obj);
        self.f_2ec = vec![Cmd::Goto(self.body.pos, Some(self.body.yaw))];
    }

    /// slot 25 FUN_00206518 (add to world): +0xfe8 anim bits, home pose, static sensor entries (the player
    /// globals DAT_002e1e40 / 2f7900 / 2dd5bc / 2dd648 -> `players`), brain start (substate 0)
    pub fn ee_sub_206518(&mut self, obj: usize, players: &[u32], now: f32, h: &mut dyn ClassHost) {
        h.base(0x1f3ea8, obj);
        let mut fe8 = 0;
        if self.ee_sub_209ad0(9) {
            fe8 |= 2;
        }
        if self.ee_sub_209ad0(8) {
            fe8 |= 4;
        }
        if self.ee_sub_209ad0(0xf) {
            fe8 |= 1;
        }
        if self.ee_sub_209ad0(0x11) {
            fe8 |= 8;
        }
        if fe8 == 0 {
            self.f_e78 = 2;
        }
        self.body.f_fe8 = fe8;
        self.f_300 = self.body.pos;
        self.f_310 = self.body.yaw;
        self.f_2ec = vec![Cmd::Goto(self.f_300, Some(self.f_310))];
        // K32: health / armour / meter start values
        self.f_33c = self.f_338;
        self.f_344 = self.f_340;
        self.f_e88 = self.f_e84;
        self.sensor.f_e0 = obj as u32 + 1;
        for &p in players {
            self.sensor.ee_sub_153f18(p, now);
        }
    }

    /// slot 26 FUN_00206748 reset: sensor +0xf0 / +0xf4, target, mode -1
    pub fn ee_sub_206748(&mut self, obj: usize, now: f32, h: &mut dyn ClassHost) {
        self.sensor.f_f0 = 0;
        self.sensor.f_f4 = 0;
        self.f_38c = 0;
        self.f_3cc = None;
        self.ee_sub_207030(obj, -1, now, h);
    }

    /// FUN_00207030(g, m): mode +0xfbc (enter / exit actions on the speed scale +0xfc4 etc. are hooks)
    pub fn ee_sub_207030(&mut self, obj: usize, m: i32, now: f32, h: &mut dyn ClassHost) {
        if m == self.f_fbc {
            return;
        }
        self.f_fbc = m;
        self.f_fc0 = now;
        h.base(0x207030, obj);
    }

    /// FUN_00207288: mode timeouts (DAT_002c3d40 10 s, 2c3d48 5 s, 2c3d44 30 s)
    pub fn ee_sub_207288(&mut self, obj: usize, now: f32, h: &mut dyn ClassHost) {
        let lim = match self.f_fbc {
            8 | 0xd => 10.0,
            0x15 => 5.0,
            0x22 | 0x25 => 30.0,
            _ => return,
        };
        if now - self.f_fc0 > lim {
            self.ee_sub_207030(obj, -1, now, h);
        }
    }

    /// FUN_00206900 sensor pass. `cands` = objects of the +0x394 classes (FUN_0015e9c8) with their positions.
    pub fn ee_sub_206900(&mut self, obj: usize, cands: &[(u32, [f32; 3])], see: &See, h: &mut dyn ClassHost) {
        if self.f_2c0 & SENSE_BLOCK != 0 {
            return;
        }
        if !self.f_394.is_empty() {
            let me = self.body.pos;
            let mut c: Vec<(f32, u32)> = cands
                .iter()
                .filter(|(o, _)| *o != obj as u32 + 1)
                .map(|(o, p)| (((p[0] - me[0]).powi(2) + (p[1] - me[1]).powi(2) + (p[2] - me[2]).powi(2)).sqrt(), *o))
                .filter(|(d, _)| *d < 1500.0) // DAT_002c3d04
                .take(64)
                .collect();
            c.sort_by(|a, b| a.0.total_cmp(&b.0)); // FUN_002338c0, comparator 0x2068d8 (by distance, R1)
            let n = self.sensor.entries.len() - self.f_3a4.min(self.sensor.entries.len());
            let old: Vec<_> = self.sensor.entries.drain(n..).collect();
            self.f_3a4 = 0;
            for (_, o) in c {
                // FUN_002085d0 (cap 8)
                if self.sensor.entries.len() >= 8 || self.sensor.ee_sub_154180(o).is_some() {
                    continue;
                }
                let prev = old.iter().find(|e| e.obj == o).copied();
                self.sensor.entries.push(prev.unwrap_or(super::sensor::Entry { obj: o, state: 0, t: see.now, t_first: 0.0 }));
                self.f_3a4 += 1;
            }
        }
        self.sensor.ee_sub_153de0(obj, see, h);
        // target knowledge record +0x3cc (FUN_0020a9a8)
        if self.f_38c != 0 {
            if let Some(k) = self.sensor.ee_sub_154180(self.f_38c) {
                match self.sensor.entries[k].state {
                    1 => self.f_3cc = Some((see.now, 1)),
                    2 => {
                        if let Some(tg) = (see.tgt)(self.f_38c) {
                            let v = [tg.pos[0] - self.body.pos[0], tg.pos[1] - self.body.pos[1], tg.pos[2] - self.body.pos[2]];
                            if (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt() < 1000.0 && v[2].abs() < 250.0 {
                                self.f_3cc = Some((see.now, 2));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// behaviour bit of the brain child (FUN_00209890 / FUN_00209970)
    fn child_bits(&self) -> u64 {
        use super::brain::Child;
        match self.brain.child {
            Child::Routine(_) => 1 << 0x1e,
            Child::Script(_) => 1 << 0x21,
            Child::React(..) => 1 << 0xb,
            Child::Chase => 1 << 0x1d,
            Child::Search(_) => 1 << 0x1f,
            // K24: 0xb0 lifted goal
            Child::Lifted => 1 << 0x16,
            // K32: hit goals (bits 5 / 6 / 7)
            Child::Hit(ref g) if g.bit >= 0 => 1 << g.bit,
            _ => 0,
        }
    }

    /// slot 15 FUN_00207350: per-frame update
    #[allow(clippy::too_many_arguments)]
    pub fn ee_sub_207350(
        &mut self,
        obj: usize,
        now: f32,
        dt: f32,
        cands: &[(u32, [f32; 3])],
        tgt: &dyn Fn(u32) -> Option<Tgt>,
        is_guard: &dyn Fn(u32) -> bool,
        player_c46: bool,
        routines: &[Vec<Cmd>],
        h: &mut dyn ClassHost,
    ) {
        // K32: the FUN_001eed78(+0x26c, 0xc, hit) messages queued by slot 74 -> brain slot 12 FUN_00176be8
        for hit in std::mem::take(&mut self.f_damage) {
            self.ee_sub_176be8(obj, &hit, now, dt, h);
        }
        ee_sub_203fc0(&mut self.body);
        if self.f_fbc == 0xe {
            return;
        }
        self.ee_sub_207288(obj, now, h);
        let (pos0, yaw0) = (self.f_300, self.f_310);
        let (pos, yaw) = (self.body.pos, self.body.yaw);
        // undo the guard's motion since load (volumes are kept in rest space)
        let to_rest = move |p: [f32; 3]| {
            let d = [p[0] - pos[0], p[1] - pos[1], p[2] - pos[2]];
            let (s, c) = (yaw0 - yaw).sin_cos();
            [pos0[0] + c * d[0] - s * d[1], pos0[1] + s * d[0] + c * d[1], pos0[2] + d[2]]
        };
        // FUN_00153e60: eye = guard + 80 * (0,0,1)
        let see = See { now, eye: [pos[0], pos[1], pos[2] + 80.0], pos, to_rest: &to_rest, tgt, player_c46 };
        self.ee_sub_206900(obj, cands, &see, h);
        // goal stack (FUN_001eed10)
        let routine = match self.f_2e8 {
            Some(k) => routines.get(k).cloned(),
            None if self.body.f_fe8 & 2 != 0 => Some(self.f_2ec.clone()),
            None => None,
        };
        let pos_of = |o: u32| tgt(o).map(|t| t.pos);
        // K22: FUN_0020a408's list - DAT_003e148c stimuli (`stimuli`) through gate FUN_00208d70 and slot 88
        // (sound: FUN_00156918 unported -> heard; sight: no blocker test)
        let gate = self.f_e64 != 0 && self.f_2c0 & GATE_208D70 == 0;
        let heard: Vec<super::slots::Stimulus> =
            self.stimuli.iter().filter(|s| self.ee_sub_208be0(obj, s, gate, to_rest(s.pos), || true, |_, _| false)).cloned().collect();
        let m = Mind {
            sensor: &self.sensor,
            pos,
            pos_of: &pos_of,
            is_guard,
            f_38c: self.f_38c,
            f_390: self.f_390,
            f_fbc: self.f_fbc,
            fa8_f94: self.f_fa8 != 0 || self.f_f94 != 0,
            c3b: self.class == 0x3b,
            routine,
            cursor: self.f_2dc,
            t30: now + self.f_2cc[0],
            routines,
            heard: &heard,
            sg: self.ee_sub_search_g(),
        };
        let mut cx = Cx { guard: obj, now, dt, body: &mut self.body, h };
        let (t, saved) = self.brain.tick(&m, &mut cx);
        // K24: guard fields the 0xb0 component wrote
        drop(cx);
        self.k24_sync();
        if let Some(t) = t {
            self.f_38c = t;
            self.f_3cc = Some((now, 0));
        }
        if saved.is_some() {
            self.f_2dc = saved;
        }
        self.f_2c0 = (self.f_2c0 & !((1 << 0x1e) | (1 << 0x21) | (1 << 0xb) | (1 << 0x1d) | (1 << 0x1f) | (1 << 0x16) | super::hit::CHILD_HIT_BITS)) | self.child_bits();
        ee_sub_204ca8_move(&mut self.body, dt);
    }

    /// FUN_00171cf8(g, routine): brain substate {5, routine} (level-script method field 317)
    pub fn ee_sub_171cf8(&mut self, obj: usize, routine: usize, now: f32, dt: f32, routines: &[Vec<Cmd>], h: &mut dyn ClassHost) {
        let pos_of = |_: u32| None;
        let no = |_: u32| false;
        let heard: Vec<super::slots::Stimulus> = Vec::new();
        let m = Mind {
            sensor: &self.sensor,
            pos: self.body.pos,
            pos_of: &pos_of,
            is_guard: &no,
            f_38c: self.f_38c,
            f_390: self.f_390,
            f_fbc: self.f_fbc,
            fa8_f94: false,
            c3b: false,
            routine: None,
            cursor: None,
            t30: f32::MAX,
            routines,
            heard: &heard,
            sg: self.ee_sub_search_g(),
        };
        let mut cx = Cx { guard: obj, now, dt, body: &mut self.body, h };
        let s = super::brain::Sub { code: 5, obj: 0, routine: Some(routine) };
        if let Some(c) = self.brain.ee_sub_176750(s, &m, &mut cx) {
            self.f_2dc = Some(c);
        }
    }
}

impl Guard {
    /// K21: FUN_00176750(FUN_00205458(g, 0x9c), {code}) - the brain goal set by the class-119 spawner
    /// (FUN_00174490: -1 then 1 = routine +0x2e8)
    pub fn k21_goal(&mut self, obj: usize, code: i32, now: f32, dt: f32, routines: &[Vec<Cmd>], h: &mut dyn ClassHost) {
        let pos_of = |_: u32| None;
        let no = |_: u32| false;
        let heard: Vec<super::slots::Stimulus> = Vec::new();
        let routine = match self.f_2e8 {
            Some(k) => routines.get(k).cloned(),
            None if self.body.f_fe8 & 2 != 0 => Some(self.f_2ec.clone()),
            None => None,
        };
        let m = Mind {
            sensor: &self.sensor,
            pos: self.body.pos,
            pos_of: &pos_of,
            is_guard: &no,
            f_38c: self.f_38c,
            f_390: self.f_390,
            f_fbc: self.f_fbc,
            fa8_f94: false,
            c3b: false,
            routine,
            cursor: self.f_2dc,
            t30: now + self.f_2cc[0],
            routines,
            heard: &heard,
            sg: self.ee_sub_search_g(),
        };
        let mut cx = Cx { guard: obj, now, dt, body: &mut self.body, h };
        let s = super::brain::Sub { code, obj: 0, routine: None };
        if let Some(c) = self.brain.ee_sub_176750(s, &m, &mut cx) {
            self.f_2dc = Some(c);
        }
        self.f_2c0 = (self.f_2c0 & !((1 << 0x1e) | (1 << 0x21) | (1 << 0xb) | (1 << 0x1d) | (1 << 0x1f) | (1 << 0x16) | super::hit::CHILD_HIT_BITS)) | self.child_bits();
    }
}

/// Compiled sensor volume for a guard (vision `which` 0 -> +0xe8, 1 -> +0xec)
pub fn set_volume(g: &mut Guard, which: usize, v: Vol) {
    if which == 0 {
        g.sensor.f_e8 = Some(v);
    } else {
        g.sensor.f_ec = Some(v);
    }
}

impl Guard {
    /// K22: the guard fields the 0xab search goal reads (FUN_00209098 alarm: +0xfc8 != 0; the FUN_0020f1b8
    /// global alarm query is unported -> false)
    pub fn ee_sub_search_g(&self) -> super::search::SGuard {
        super::search::SGuard { alarm: self.f_fc8 != 0, f_e6c: self.f_e6c, f_e70: self.f_e70, f_e78: self.f_e78, leash: self.f_e74 }
    }
}
