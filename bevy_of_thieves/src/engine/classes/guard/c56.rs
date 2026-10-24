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
    /// +0x2dc saved script cursor
    pub f_2dc: Option<usize>,
    /// +0x2e8 routine (index into the level routines), +0x2ec home routine (FUN_002061d8)
    pub f_2e8: Option<usize>,
    pub f_2ec: Vec<Cmd>,
    /// ANIM R25: field 273 (row 0x2af328: kind 0x2071, +0x2e8, setter FUN_0020a540) routine oid, resolved to
    /// `f_2e8` by `k6_make_guard` (fields 274 / 275 = body.f_2cc / f_2d4)
    pub f_2e8_oid: i32,
    /// +0x300 home position, +0x310 home yaw
    pub f_300: [f32; 3],
    pub f_310: f32,
    /// BOSS R30: load pose (body pos / yaw when the sensor volumes were compiled); the vision volumes are child
    /// objects that move with the guard, so the rest frame is this pose, not the +0x300 home (method 454 moves it)
    pub rest0: ([f32; 3], f32),
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
    /// BOSS: class-59 perch records +0x1014[+0x1010] (FUN_00155530): oids {perch A, vol A, perch B, vol B}
    pub perches: Vec<[i32; 4]>,
    /// BOSS: the resolved records (FUN_00155450)
    pub perch_recs: Vec<super::perch::Rec>,
    /// BOSS: +0x1018 health at the last perch pick (FUN_001553f0: 0x7fffffff)
    pub f_1018: i32,
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
    /// WIRE-OBJ2 loot: +0x3d8 coin count (field 276, default -1 = from the +0x404 list), +0x3f0 (field 339,
    /// default -200, FUN_00154e10 item delta), +0x404 loot list {kind, 0, item, count} x n (+0x484 = len),
    /// +0x488 next un-pickpocketed entry, +0x490 (+0x404 list frozen), +0x494 loot object oid (field 306;
    /// kept apart from `f_494`, the resolved slot-86 object)
    pub f_3d8: i32,
    pub f_3f0: i32,
    pub f_404: Vec<[i32; 4]>,
    pub f_488: i32,
    pub f_490: i32,
    /// CRAWL R26: +0x48c FUN_001f1cf8 blocked / retry flag, +0x494 holder state (class 99, guard/holder.rs)
    pub f_48c: i32,
    pub holder: super::holder::Holder99,
    pub f_494_oid: i32,
    /// +0x3e0 local drop point (field 277; FUN_001a6f00 -> world at the knock-out)
    pub f_3e0: [f32; 3],
    pub f_e84: i32,
    pub f_e88: i32,
    /// +0x8d0 / +0xa70 / +0xc10 hit-anim tables (empty = every record -1, `hit_anim`): record kind +0
    /// (FUN_00209130: -1 none, 0 ASEG, 1 class 111; REPLACED: kind 2 blend (FUN_0012f938(a, 0x22)) reads as 0)
    pub hit_anims: Vec<[i32; 26]>,
    /// HEIST r31: the records' +4 anim oid (fields 247 / 246 / 248; 0 = none) and +0xc word (4107)
    pub hit_oids: Vec<[(i32, i32); 26]>,
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
        self.body.f_2cc = [f32::MAX; 2];
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
        self.f_3f0 = -200;
        self.f_3d8 = -1;
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
            // BOSS: class 59 slot 1 FUN_001553f0: +0x1018 = 0x7fffffff
            59 => self.f_1018 = i32::MAX,
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
                    // BOSS: FUN_00209ad0 matches an entry only when its id mask (FUN_00207a40) is a subset of the
                    // single queried bit, so multi-id entries never answer a one-anim query
                    if let Some([id]) = a.first().map(|x| x.as_slice()) {
                        self.anims.push(*id as i32);
                    }
                }
                // HEIST r31: method fields 247 / 246 / 248 (load setters 0x1031b8 / 0x103048 / 0x103328 ->
                // FUN_00207eb8(g, mask, oid, w, +0x8d0 / +0xa70 / +0xc10)): per set bit of the mask (< 26) the
                // record = {-1, oid, _, w} (FUN_002090d0 reset; resolved at slot 14 FUN_002080f0)
                246..=248 => {
                    let t = match p.field { 247 => 0, 246 => 1, _ => 2 };
                    let x = |i: usize| a.get(i).and_then(|v| v.first().copied()).unwrap_or(0);
                    if self.hit_oids.is_empty() {
                        self.hit_oids = vec![[(0, 0); 26]; 3];
                        self.hit_anims = vec![[-1; 26]; 3];
                    }
                    for k in (0..26).filter(|k| x(0) & (1 << k) != 0) {
                        self.hit_oids[t][k] = (x(1) as i32, x(2) as i32);
                        self.hit_anims[t][k] = -1;
                    }
                }
                // ANIM R25: rows 273 / 274 / 275 (+0x2e8 routine, +0x2cc / +0x2d4 kind 0xa ranges)
                273 => self.f_2e8_oid = w0 as i32,
                274 => self.body.f_2cc = pair(),
                275 => self.body.f_2d4 = pair(),
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
                276 => self.f_3d8 = w0 as i32,
                306 => self.f_494_oid = w0 as i32,
                // CRAWL R25: method fields 309 / 310 (FUN_00103ca0 / FUN_00103d28 -> FUN_001f17b0) in the property list
                309 | 310 => self.ee_sub_1f17b0(w0 as i32),
                339 => self.f_3f0 = w0 as i32,
                // BOSS: method field 1340 (thunk 0x10f6e8 -> FUN_00155530): class-59 perch record
                1340 => {
                    let x: Vec<u32> = a.iter().flatten().copied().collect();
                    self.perches.push(std::array::from_fn(|k| x.get(k).copied().unwrap_or(0) as i32));
                }
                277 => {
                    let x: Vec<u32> = a.iter().flatten().copied().collect();
                    self.f_3e0 = std::array::from_fn(|k| f(x.get(k).copied().unwrap_or(0)));
                }
                338 => self.f_4a4 = w0 as i32,
                _ => {}
            }
        }
    }

    /// FUN_00209ad0(g, id): anim `id` in the table
    fn ee_sub_209ad0(&self, id: i32) -> bool {
        self.anims.contains(&id)
    }

    /// HEIST r31: FUN_00209130 record resolution: FUN_0015e938(4, +4 oid, g); is-a 0x6d (ASEG) -> kind 0,
    /// is-a 0x6f -> kind 1, else none. Returns (object handle, kind).
    pub fn hit_anim_obj(&self, obj: usize, t: usize, k: usize, h: &mut dyn ClassHost) -> Option<(u32, i32)> {
        let oid = self.hit_oids.get(t)?[k].0;
        if oid == 0 {
            return None;
        }
        let o = h.ee_sub_15e938(4, oid, obj as u32 + 1);
        if o == 0 {
            None
        } else if h.ee_sub_132b10(o, 0x6d) != 0 {
            Some((o, 0))
        } else if h.ee_sub_132b10(o, 0x6f) != 0 {
            Some((o, 1))
        } else {
            None
        }
    }

    /// slot 14 FUN_002061d8: home routine +0x2ec = one kind-7 entry (FUN_00170c90(40.0, obj, -1, -1))
    pub fn ee_sub_2061d8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x203a38, obj);
        // HEIST r31: FUN_002080f0 -> FUN_002080a0(+0xc10 / +0x8d0 / +0xa70) -> FUN_00209130 per record
        for t in 0..self.hit_oids.len() {
            for k in 0..26 {
                if self.hit_oids[t][k].0 != 0 {
                    self.hit_anims[t][k] = self.hit_anim_obj(obj, t, k, h).map_or(-1, |(_, kind)| kind);
                }
            }
        }
        self.f_2ec = vec![Cmd::Goto(self.body.pos, Some(self.body.yaw))];
    }

    /// slot 25 FUN_00206518 (add to world): +0xfe8 anim bits, home pose, static sensor entries (the player
    /// globals DAT_002e1e40 / 2f7900 / 2dd5bc / 2dd648 -> `players`), brain start (substate 0)
    pub fn ee_sub_206518(&mut self, obj: usize, players: &[u32], now: f32, h: &mut dyn ClassHost) {
        h.base(0x1f3ea8, obj);
        let mut fe8 = 0;
        if self.ee_sub_209ad0(9) {
            fe8 |= 2;
        } else {
            self.f_e78 = 2; // BOSS: 1:1 FUN_00206518 (e78 = 2 when anim 9 is missing)
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

    /// FUN_001f17b0 (Splice methods 309 / 310): queue item `oid` {oid, 0, -1, 1} at +0x404 while the
    /// queue is empty or +0x490 is set (CRAWL R25)
    pub fn ee_sub_1f17b0(&mut self, oid: i32) {
        if self.f_404.is_empty() || self.f_490 != 0 {
            self.f_404.push([oid, 0, -1, 1]);
        }
    }

    /// FUN_001f1a40 (slot 25 FUN_00206518 tail, when +0x494 is set; FUN_001f1cf8 follows): the guard's loot
    /// list +0x404. FUN_001c2db8(+0x494) clears the loot object's +0x150..+0x158 (object not modelled). A
    /// pre-set first entry is kept as the item (the FUN_0015e938 / is-a 0x5c / +0x230 / FUN_00164030
    /// cancel check of that entry is not modelled: list entries only come from this function in the port);
    /// else with probability table+8 one weighted item (6 {weight, item} pairs at table +0xc) -> {0x2b5, 0,
    /// item, 1}. Table = 0x2c3748 (0x2c3968 for is-a 0x3a) + FUN_00210ae0() * 0x3c (-1 -> 1). Coins: total
    /// FUN_0011d3f8(t+0, t+4) split into FUN_0011d3f8(2, 3) piles {0x2b3, 0, -, pile}; the item last.
    pub fn ee_sub_1f1a40(&mut self, obj: usize, rng: &mut crate::engine::physics::driven::GameRng, h: &mut dyn ClassHost) {
        if self.f_490 != 0 {
            return;
        }
        let m = h.ee_sub_210ae0();
        let m = if m != -1 { m } else { 1 };
        let base = if h.ee_sub_132b10(obj as u32 + 1, 0x3a) == 0 { 0x002c_3748u32 } else { 0x002c_3968 };
        let t = (base as i32 + m * 0x3c) as u32;
        let tw = |k: u32| crate::engine::disc::exe::exe().w(t + 4 * k) as i32;
        let mut item = [0i32; 4];
        if !self.f_404.is_empty() {
            item = self.f_404[0];
        } else {
            let r = rng.FUN_0011d448(0.0, 1.0);
            if f32::from_bits(tw(2) as u32) < r {
                item[0] = -1;
                item[2] = -1;
            } else {
                let sum: i32 = (0..6).map(|u| tw(3 + 2 * u)).sum();
                let mut pick = rng.FUN_0011d3f8(0, sum - 1);
                let mut u = 0u32;
                let mut found = true;
                while tw(3 + 2 * u) <= pick {
                    pick -= tw(3 + 2 * u);
                    u += 1;
                    if 5 < u {
                        found = false;
                        break;
                    }
                }
                if found {
                    item[2] = tw(3 + 2 * u + 1);
                }
                item[3] = 1;
                item[0] = 0x2b5;
            }
        }
        self.f_404.clear();
        let mut total = rng.FUN_0011d3f8(tw(0), tw(1)); 
        let piles = rng.FUN_0011d3f8(2, 3);
        let mut k = 0;
        let mut d9 = 0;
        while k < piles {
            let q = total / (piles + d9);
            let c = if 0 < q { q } else { 1 };
            total -= c;
            self.f_404.push([0x2b3, 0, 0, c]);
            if total == 0 {
                break;
            }
            k += 1;
            d9 = -k;
        }
        if 0 < item[3] {
            self.f_404.push(item);
        }
    }

    /// FUN_001a6f00(guard, 0, +0x3e0): the local drop point in the world (root yaw frame)
    pub fn drop_pos_3e0(&self) -> [f32; 3] {
        let (s, c) = self.body.yaw.sin_cos();
        let v = self.f_3e0;
        [self.body.pos[0] + c * v[0] - s * v[1], self.body.pos[1] + s * v[0] + c * v[1], self.body.pos[2] + v[2]]
    }

    /// C2d08b0_v7 first knock-out: coin count n = +0x3d8, or (< 0) (sum of the 0x2b3 counts of the
    /// un-pickpocketed list entries +0x488.. + 1) / 2
    pub fn ko_coins(&self) -> i32 {
        if 0 <= self.f_3d8 {
            return self.f_3d8;
        }
        let s: i32 = self.f_404.iter().skip(self.f_488.max(0) as usize).filter(|e| e[0] == 0x2b3).map(|e| e[3]).sum();
        (s + 1) / 2
    }

    /// slot 26 FUN_00206748 reset: sensor +0xf0 / +0xf4, target, mode -1
    pub fn ee_sub_206748(&mut self, obj: usize, now: f32, h: &mut dyn ClassHost) {
        self.sensor.f_f0 = 0;
        self.sensor.f_f4 = 0;
        self.f_38c = 0;
        self.f_3cc = None;
        // HEIST r33: FUN_00206748 also clears the first-knock-out flag +0x330 (C2d08b0_v7): a pool copy handed out
        // again counts its next knock-out (FollowWorld 0x17 / 0x18). The other plain fields it clears are not
        // ported here yet.
        self.f_330 = 0;
        self.ee_sub_207030(obj, -1, now, h);
    }

    /// POOL1 R71: FUN_00208540(g, 0) after FUN_00176750 set its bVar3 (Brain::clear_target)
    pub fn take_clear_target(&mut self) {
        if std::mem::take(&mut self.brain.clear_target) {
            self.f_38c = 0;
            self.f_3cc = None;
        }
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

    /// FUN_00209970 (cleared bits) / FUN_00209890 (set bits): +0x2c0 = new bits and per changed bit the
    /// guard's level-script message 7 (vtable +0x1c (g, 7, 0, 2, {set, cleared}, 0) -> FUN_001331f8) with
    /// {bit, -1} / {-1, bit} (CRAWL: R7 cellar script entry 128 turns the vent lasers 136591 / blocker 136587
    /// off on message 7 of its guards). REPLACED: FUN_00209890's job event table DAT_002c3d98 not posted.
    fn set_child_bits(&mut self, obj: usize, h: &mut dyn ClassHost) {
        let mask = (1u64 << 0x1e) | (1 << 0x21) | (1 << 0xb) | (1 << 0x1d) | (1 << 0x1f) | (1 << 0x16) | (1 << 0x22) | (1 << 0x23) | super::hit::CHILD_HIT_BITS;
        let old = self.f_2c0;
        self.f_2c0 = (old & !mask) | self.child_bits();
        self.post_bit_msgs(obj, old, h);
    }
    /// FUN_00209890 / FUN_00209970 script message 7 for each +0x2c0 bit changed since `old`
    pub(super) fn post_bit_msgs(&mut self, obj: usize, old: u64, h: &mut dyn ClassHost) {
        let ch = old ^ self.f_2c0;
        for b in 0..64u32 {
            if ch >> b & 1 != 0 && old >> b & 1 != 0 {
                h.k27_script_msg(obj as u32 + 1, 7, &[u32::MAX, b]);
            }
        }
        for b in 0..64u32 {
            if ch >> b & 1 != 0 && old >> b & 1 == 0 {
                h.k27_script_msg(obj as u32 + 1, 7, &[b, u32::MAX]);
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
            Child::Chase(..) => 1 << 0x1d,
            Child::Search(_) => 1 << 0x1f,
            // K24: 0xb0 lifted goal
            Child::Lifted => 1 << 0x16,
            // K32: hit goals (bits 5 / 6 / 7)
            Child::Hit(ref g) if g.bit >= 0 => 1 << g.bit,
            // BOSS R33: class-59 0xb4 perch goal (FUN_00155560 / 155598: bit 0x22) and its 0xb5 perch-side child
            // (FUN_001559b0 / 1559f8: bit 0x23; Dimitri's msg-0x18 script asks (get self 405) 35 = this bit)
            Child::Perch(ref p) => (1 << 0x22) | if p.b5.is_some() { 1 << 0x23 } else { 0 },
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
        let (pos0, yaw0) = self.rest0;
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
        // (sound: FUN_00156918 radius/z test; sight: no blocker test)
        let gate = self.f_e64 != 0 && self.f_2c0 & GATE_208D70 == 0;
        let heard: Vec<super::slots::Stimulus> =
            self.stimuli.iter().filter(|s| self.ee_sub_208be0(obj, s, gate, to_rest(s.pos), || s.kind.as_ref().is_none_or(|k| k.ee_sub_156918(s, pos, now)), |_, _| false)).cloned().collect();
        let m = Mind {
            chase_r: [self.f_314, self.f_318],
            atk_v: self.atk.root_v(now),
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
            routines,
            heard: &heard,
            sg: self.ee_sub_search_g(),
            f_3cc_t: self.f_3cc.map(|r| r.0),
            perches: &self.perch_recs,
            hp: self.f_33c,
            f_1018: self.f_1018,
        };
        let mut cx = Cx { guard: obj, now, dt, body: &mut self.body, h };
        let (t, saved) = self.brain.tick(&m, &mut cx);
        // K24: guard fields the 0xb0 component wrote
        drop(cx);
        self.k24_sync();
        self.take_clear_target();
        if let Some(t) = t {
            self.f_38c = t;
            self.f_3cc = Some((now, 0));
        }
        // WIRE-G: chase state 4 entry: FUN_00154038(sensor, t, 0) + FUN_00208540(g, 0)
        if let super::brain::Child::Chase(c) = &mut self.brain.child {
            if std::mem::take(&mut c.drop_target) {
                if let Some(k) = self.sensor.ee_sub_154180(self.brain.sub.obj) {
                    self.sensor.ee_sub_154080(obj, k, 0, now, h);
                }
                self.f_38c = 0;
                self.f_3cc = None;
            }
        }
        // BOSS: 0xb4 setstate writes (FUN_00155788): +0x1018 = +0x33c, record +0x20 = 1
        if let super::brain::Child::Perch(p) = &mut self.brain.child {
            if std::mem::take(&mut p.set_1018) {
                self.f_1018 = self.f_33c;
            }
            if let Some(r) = p.mark_used.take().and_then(|k| self.perch_recs.get_mut(k)) {
                r.used = true;
            }
        }
        if saved.is_some() {
            self.f_2dc = saved;
        }
        self.set_child_bits(obj, h);
        ee_sub_204ca8_move(&mut self.body, dt);
    }

    /// FUN_00171cf8(g, routine): brain substate {5, routine} (level-script method field 317)
    pub fn ee_sub_171cf8(&mut self, obj: usize, routine: usize, now: f32, dt: f32, routines: &[Vec<Cmd>], h: &mut dyn ClassHost) {
        let pos_of = |_: u32| None;
        let no = |_: u32| false;
        let heard: Vec<super::slots::Stimulus> = Vec::new();
        let m = Mind {
            chase_r: [self.f_314, self.f_318],
            atk_v: self.atk.root_v(now),
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
            routines,
            heard: &heard,
            sg: self.ee_sub_search_g(),
            f_3cc_t: self.f_3cc.map(|r| r.0),
            perches: &self.perch_recs,
            hp: self.f_33c,
            f_1018: self.f_1018,
        };
        let mut cx = Cx { guard: obj, now, dt, body: &mut self.body, h };
        let s = super::brain::Sub { code: 5, obj: 0, routine: Some(routine) };
        if let Some(c) = self.brain.ee_sub_176750(s, &m, &mut cx) {
            self.f_2dc = Some(c);
        }
        drop(cx);
        self.take_clear_target();
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
            chase_r: [self.f_314, self.f_318],
            atk_v: self.atk.root_v(now),
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
            routines,
            heard: &heard,
            sg: self.ee_sub_search_g(),
            f_3cc_t: self.f_3cc.map(|r| r.0),
            perches: &self.perch_recs,
            hp: self.f_33c,
            f_1018: self.f_1018,
        };
        let mut cx = Cx { guard: obj, now, dt, body: &mut self.body, h };
        let s = super::brain::Sub { code, obj: 0, routine: None };
        if let Some(c) = self.brain.ee_sub_176750(s, &m, &mut cx) {
            self.f_2dc = Some(c);
        }
        drop(cx);
        self.take_clear_target();
        self.set_child_bits(obj, h);
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
