//! WP-K6: guards = the class 56 family (concrete 57-67). `c56` = the shared class-56 overrides, `sensor` = the
//! +0xf64 sensor object, `brain` = the root goal 0x9c, `patrol` = the routine / rail / goto / walk goals.
//! Instances live in `ClassSet::k6`. The concrete classes 57-67 override only presentation / special-move
//! slots (see K6: Findings); they all run the class-56 core here.
pub mod attack;
pub mod region;
pub mod brain;
pub mod c56;
pub mod los;
pub mod slots;
pub mod patrol;
pub mod sensor;
pub mod search;
pub mod lift;
pub mod hit;
pub mod hearing;
pub mod perch;
pub mod holder;

use super::*;
use crate::engine::game::fields::object_props;
use crate::engine::gfx::attach::BoneWorlds;
use crate::engine::gfx::attach::affine_from_bytes;
use crate::engine::gfx::scene::{invert, mul};
use crate::engine::physics::geom::{build_bsp, build_geom};
use crate::engine::player::p5_attach::Path;
use crate::engine::player::p6::{path_of, FUN_001e6180};
use crate::engine::world::loads_b4::B4Load;
use patrol::Cmd;

/// K6 world class id -> vtable (kept in `ClassSet::k6`).
pub const PORTED_K6: &[(ClassId, u32)] = &[
    (57, 0x2cf520),
    (58, 0x2c9dc0),
    (59, 0x2cde40),
    (60, 0x2d4820),
    (61, 0x2d7188),
    (62, 0x2c91c8),
    (63, 0x2ccbb8),
    (64, 0x2cd0a0),
    (65, 0x2c8ef0),
    (66, 0x2c8c18),
    (67, 0x2c8940),
];

/// Level data the guard goals resolve against (class-113 routines with their rails / points resolved).
#[derive(Clone, Debug, Default)]
pub struct K6Level {
    /// (class-113 object index, resolved commands)
    pub routines: Vec<(usize, Vec<Cmd>)>,
    /// commands only (indexed like `routines`)
    pub cmds: Vec<Vec<Cmd>>,
    /// BOSS: DAT_003e1a5c region tree (world +0x45cc, None = no tree)
    pub regions: Option<region::Tree>,
    /// ANIM R26: the kind-1 spawn point routines of FUN_001733f8 (FUN_0013e3c8(0x71, 0, 0x2cb) + FUN_00170c50(r,
    /// point oid, 1, -1) = one kind-6 rail command), (spawner object, point index) -> index in `routines`
    pub pt_routines: std::collections::HashMap<(usize, usize), usize>,
}

/// FUN_001715b8 for a kind-6 entry: the rail (is-a 10) object with this oid
pub fn rail_by_oid(w: &World, oid: i32) -> Option<Path> {
    w.find_oid(oid).iter().copied().find(|&k| crate::engine::player::p4::FUN_00132b10(w.objects[k].class as i32, 10)).and_then(|k| rail_of(w, k))
}

/// Rail of a class-10 object (spline kinds walked as polylines, as `p6::path_of` for kind 0).
/// R79: the rail is walked in its vtable +0xfc WORLD matrix (FUN_001e6180 copies it to +0x100, path10 `world_xf`:
/// owner bone + P15 placement), not the bare load xform (opt-out SLY2_NO_R79_RAILXF).
pub fn rail_of(w: &World, i: usize) -> Option<Path> {
    let r79 = std::env::var_os("SLY2_NO_R79_RAILXF").is_none();
    if !r79 {
        if let Some(p) = path_of(w, i) {
            return Some(p);
        }
    }
    let Payload::Loaded(LoadData::B4(bx)) = &w.objects[i].payload else { return None };
    let B4Load::L1e5df8 { xform, a, b, p0, .. } = &**bx else { return None };
    if p0.len() < 2 {
        return None;
    }
    let local = crate::engine::gfx::attach::affine_from_bytes(&xform.xform);
    let m = if r79 { crate::engine::classes::shared::path10::world_xf(w, i, None).unwrap_or(local) } else { local };
    let mut p = Path::polyline(p0.clone(), *b != 0);
    p.kind = *a as i32;
    p.rot = [m[0], m[1], m[2]];
    p.pos = m[3];
    FUN_001e6180(&mut p);
    p.s = p.l.clone();
    Some(p)
}

fn is_under(w: &World, mut k: usize, root: usize) -> bool {
    for _ in 0..64 {
        if k == root {
            return true;
        }
        match w.objects[k].owner {
            Some(o) => k = o,
            None => return false,
        }
    }
    false
}

/// first object with `oid` under `root` (else anywhere)
fn find_under(w: &World, oid: i32, root: usize) -> Option<usize> {
    let all = w.find_oid(oid);
    all.iter().copied().find(|&k| is_under(w, k, root)).or_else(|| all.first().copied())
}

/// words of a method-232 / 233 prop (attack oids)
fn attack_oids(v: &crate::engine::world::props::PropValue) -> Vec<i32> {
    use crate::engine::world::props::{ArrayItem, PropValue};
    let w = |b: &[u8]| b.chunks_exact(4).map(|x| i32::from_le_bytes([x[0], x[1], x[2], x[3]])).collect::<Vec<_>>();
    match v {
        PropValue::Scalar(b) => w(b),
        PropValue::Array(it) => it
            .iter()
            .flat_map(|i| match i {
                ArrayItem::One(_, b) => w(b),
                ArrayItem::List(_, l) => l.iter().flat_map(|b| w(b)).collect(),
            })
            .collect(),
    }
}

impl K6Level {
    /// Resolve every class-113 routine (C2cf880 entries: kind 6 rail oid, kind 7 object, 5 wait, 9 / 10 labels)
    pub fn build(w: &World, bw: &mut BoneWorlds) -> K6Level {
        let mut l = K6Level::default();
        for (i, o) in w.objects.iter().enumerate() {
            if o.class != 113 {
                continue;
            }
            let mut r = c2cf880::C2cf880::default();
            r.feed(w.objects.get(o.clone_of.unwrap_or(i)).and_then(object_props).unwrap_or(&[]));
            let cmds = r
                .f_50
                .iter()
                .map(|e| match e.kind {
                    5 => Cmd::Wait(f32::from_bits(e.f_8[0])),
                    // ANIM R27: the 0xc0 task's start u +0x38 is -1 (FUN_0017eb48: closest point); entry +0xc is an
                    // object oid (+0x50, the goto's +0xa8), not a u
                    6 => Cmd::Rail(rail_by_oid(w, e.id), e.f_8[0] as i32, -1.0),
                    // ANIM R27: kinds 1 / 2 (FUN_001715f0): guard +0x2cc / +0x2d4 = entry +8 (8 bytes)
                    1 => Cmd::Set2cc([f32::from_bits(e.f_8[0]), f32::from_bits(e.f_8[1])]),
                    2 => Cmd::Set2d4([f32::from_bits(e.f_8[0]), f32::from_bits(e.f_8[1])]),
                    7 => match find_under(w, e.id, i) {
                        // ANIM R27: case 7, FUN_00132b10(obj, 10): the target is the path itself (FUN_001c1ec0)
                        Some(k) if crate::engine::player::p4::FUN_00132b10(w.objects[k].class as i32, 10) => match rail_of(w, k) {
                            Some(p) => Cmd::GotoPath(p),
                            None => Cmd::Other(7),
                        },
                        Some(k) => Cmd::Goto(bw.xfm_world(w, k).unwrap_or_else(|| bw.get(w, k, 0))[3], None),
                        None => Cmd::Other(7),
                    },
                    // BOSS: kind 3 (FUN_001715f0 case 3): FUN_001715b8 object -> FUN_00171af8 / FUN_00171b60 =
                    // the guard is placed at the object's world position + orientation
                    3 => match find_under(w, e.id, i) {
                        Some(k) => {
                            let m = bw.xfm_world(w, k).unwrap_or_else(|| bw.get(w, k, 0));
                            Cmd::Warp(m[3], m[0][1].atan2(m[0][0]))
                        }
                        None => Cmd::Other(3),
                    },
                    // ANIM R25: kind 0xe (FUN_001715f0 case 0xe): anim record {oid +4, flags +8}
                    0xe => Cmd::Anim(e.id, e.f_8[0]),
                    9 => Cmd::Label(e.id),
                    10 => Cmd::Jump(e.id),
                    // BOSS: kind 0xf (FUN_001715f0 case 0xf): FUN_001715b8 object (the sequence) ->
                    // FUN_001e0020(seq, +8): post signal +8 to it
                    0xf => match find_under(w, e.id, i) {
                        Some(k) => Cmd::Signal(k as u32 + 1, e.f_8[0] as i32),
                        None => Cmd::Other(0xf),
                    },
                    k => Cmd::Other(k),
                })
                .collect::<Vec<_>>();
            l.cmds.push(cmds.clone());
            l.routines.push((i, cmds));
        }
        // FUN_001c0ec0 per directory entry: a non-empty block replaces world +0x45cc (the last one stays)
        l.regions = w.entries.iter().filter_map(|e| e.sections.block_45cc.as_ref()).last().map(region::Tree::from_block);
        l
    }
}

/// K6 construct + init (slot 6) + feed + volumes of guard object `i`; `place` = the vtable +0x12c placement
/// (template instance matrix) applied on top of the root bone. K21: also used for the FUN_001cb250 pool copies.
pub(super) fn k6_make_guard(w: &World, bw: &mut BoneWorlds, i: usize, vt: u32, place: Option<crate::engine::gfx::scene::Affine>, h: &mut dyn ClassHost) -> Box<c56::Guard> {
    let o = &w.objects[i];
    let src = o.clone_of.unwrap_or(i);
    let props = w.objects.get(src).and_then(object_props).unwrap_or(&[]);
    let mut g = Box::<guard::c56::Guard>::default();
    g.class = o.class;
    g.vt = vt;
    let m = match &place {
        Some(pm) => mul(&bw.get(w, i, 0), pm),
        None => bw.get(w, i, 0),
    };
    g.body.pos = m[3];
    g.body.yaw = m[0][1].atan2(m[0][0]);
    g.body.f_1a8 = g.body.yaw;
    g.rest0 = (g.body.pos, g.body.yaw);
    g.body.c3a = o.class == 0x3a;
    g.init(i, h);
    g.feed(props);
    // CRAWL R25: class-6 override records (FUN_001c9b08 -> FUN_0013f810) onto this guard, e.g. method 309 keys
    if let Some(ov) = w.override_props.get(&i) {
        g.feed(ov);
    }
    // ANIM R25: field 273 setter FUN_0020a540 (+0x2e8 = the class-113 routine object): index in K6Level::routines
    // (built over every class-113 object in world order)
    if g.f_2e8_oid != 0 {
        g.f_2e8 = find_under(w, g.f_2e8_oid, i).filter(|&k| w.objects[k].class == 113).map(|k| w.objects[..k].iter().filter(|o| o.class == 113).count());
    }
    // K11: methods 232 / 233 attack oids (+0xeb8), resolved like slot 14 FUN_0020aaa8
    let oids: Vec<i32> = props
        .iter()
        .filter(|p| p.field == 232 || p.field == 233)
        .flat_map(|p| attack_oids(&p.value))
        .filter(|&o| o != 0)
        .collect();
    g.atk = attack::Attacks::ee_sub_20aaa8(w, src, &oids);
    // BOSS: class 59 post-load FUN_00155450: perch oids -> objects (FUN_0015e938(0x3dd490, 4, oid, g))
    let mut at = |oid: i32| {
        let k = find_under(w, oid, i).or_else(|| w.find_oid(oid).first().copied())?;
        let p = bw.xfm_world(w, k).unwrap_or_else(|| bw.get(w, k, 0))[3];
        Some([p[0], p[1], p[2]])
    };
    let perch_at: Vec<_> = g.perches.iter().map(|r| (at(r[0]), at(r[2]))).collect();
    let mut vol_of = |oid: i32| -> Option<sensor::Vol> {
        if oid == 0 {
            return None;
        }
        // WIRE-D: a C119 copy (clone_of = template) has no child objects: its volume is the template's,
        // carried by the copy's placement (template rest -> copy world)
        let k = w.find_oid(oid).iter().copied().find(|&k| is_under(w, k, i)).or_else(|| find_under(w, oid, src))?;
        // CRAWL R28: a HEIST FUN_001f66f0 child copy (Payload::Clone) carries its template's solid
        let mut kp = k;
        while let (Payload::Clone, Some(c)) = (&w.objects[kp].payload, w.objects[kp].clone_of) {
            kp = c;
        }
        // and its world transform is the template's carried by the copy placement (the WIRE-D path below)
        let k = kp;
        let solid = match &w.objects[k].payload {
            Payload::Loaded(LoadData::Solid(s)) => s,
            Payload::Loaded(LoadData::L2282b0 { solid, .. }) => solid,
            _ => return None,
        };
        let mut mw = bw.xfm_world(w, k)?;
        if let (Some(pm), true) = (&place, is_under(w, k, i)) {
            mw = mul(&mw, pm);
        } else if src != i && !is_under(w, k, i) && is_under(w, k, src) {
            mw = mul(&mul(&mw, &invert(&bw.get(w, src, 0))), &m);
        }
        Some(sensor::Vol::new(k, invert(&mw), build_geom(&solid.geom), build_bsp(&solid.bsp)))
    };
    for which in 0..2 {
        if let Some(v) = vol_of(g.vol_oids[which]) {
            c56::set_volume(&mut g, which, v);
        }
    }
    for (r, (a, b)) in g.perches.clone().into_iter().zip(perch_at) {
        let (va, vb) = (vol_of(r[1]), vol_of(r[3]));
        g.perch_recs.push(perch::Rec { a, va, b, vb, used: false });
    }
    g
}

/// handle of a player-character global (DAT_002e1e40 ...) that is not a world object
pub const PLAYER_HANDLE: u32 = 0x8000_0000;

impl ClassSet {
    /// ANIM-ALL R34: FUN_001442d0 (`on`) / FUN_00144338 on Sly (class 53 JT, not a class instance): the
    /// FUN_00205478 / FUN_00205588 layer counters + the +0x12f4 nesting count. FUN_0019cd38(self + 0xf00, -1)
    /// goes through the binocucom host hook. ANIM-ALL R42: the nest bodies (FUN_00143ee0(self, 0x60b, 9, 5) /
    /// FUN_00143ff8 on the JT body slots +0x1030 / +0x1034) go through the WP-SCR2 player pose queue
    /// (`ClassHost::k22_pose_143ee0`, idx 5 / Clear).
    pub(crate) fn jt_slot_82_83(&mut self, o: usize, mask: u32, nest: bool, on: bool, h: &mut dyn ClassHost) {
        let (mut n, mut k) = self.jt_2a0.get(&o).copied().unwrap_or_default();
        if on {
            super::k17_55::ee_sub_205478_cnt(&mut n, o as u32 + 1, mask, h);
            if nest {
                k += 1;
                if k == 1 {
                    // ANIM-ALL R42: FUN_00143ee0(jt, 0x60b, 9, 5) (Sly's pose ASEG 0x60b under the JT, slot-87 idx 5)
                    // through the WP-SCR2 player pose queue, then FUN_0019cd38(jt + 0xf00, -1)
                    h.k22_pose_143ee0(PLAYER_HANDLE, 0x60b, 9, 5);
                    h.ee_sub_19cd38_po(o, -1);
                }
            }
        } else {
            super::k17_55::ee_sub_205588_cnt(&mut n, o as u32 + 1, mask, h);
            if nest {
                k -= 1;
                if k == 0 {
                    // ANIM-ALL R42: FUN_00143ff8(jt): posing (slot 88 in 10 / 11) -> slot-87 idx 0 (player pose Clear)
                    h.k22_pose_143ee0(PLAYER_HANDLE, -1, 0, 0);
                }
            }
        }
        self.jt_2a0.insert(o, (n, k));
    }
    /// K6: construct + init (slot 6) + feed + volumes of every class 57-67 object
    pub(super) fn build_k6(&mut self, w: &World, h: &mut dyn ClassHost) {
        let mut bw = BoneWorlds::default();
        self.k6l = K6Level::build(w, &mut bw);
        // template copies never handed out by FUN_001ca370 at load (no +0x12c placement) are not in the world
        let template: std::collections::HashSet<usize> = w.entries.iter().flat_map(|e| e.instances.iter().copied()).collect();
        for (i, o) in w.objects.iter().enumerate() {
            let Some(&(_, vt)) = PORTED_K6.iter().find(|x| x.0 == o.class) else { continue };
            if template.contains(&i) && !w.life.placed.contains_key(&i) {
                continue;
            }
            let place = w.life.placed.get(&i).map(affine_from_bytes);
            let g = k6_make_guard(w, &mut bw, i, vt, place, h);
            self.k6.push((i, Inst::Guard(g)));
        }
    }
    /// K6 load-post: slot 14 FUN_002061d8, then slot 25 FUN_00206518 (add to world)
    pub(super) fn post_load_k6(&mut self, h: &mut dyn ClassHost) {
        let players = self.g.player_globals();
        for (i, x) in &mut self.k6 {
            if let Inst::Guard(g) = x {
                g.ee_sub_2061d8(*i, h);
                g.ee_sub_206518(*i, &players, self.g.t, h);
                // FUN_00206518 tail: +0x494 set -> FUN_001f1a40 loot list (FUN_001f1cf8 below, CRAWL R26)
                if g.f_494_oid != 0 {
                    g.ee_sub_1f1a40(*i, &mut self.rng, h);
                }
                // slot 22 FUN_002079e0 (the K6 add-to-world above is slot 25's FUN_001f3ea8 part)
                g.ee_sub_2079e0(*i, &mut self.g.k7_live, h);
            }
        }
        // CRAWL R26: FUN_00206518 tail FUN_001f1cf8 (after FUN_001f1a40): arm the +0x494 holder
        for gi in 0..self.k6.len() {
            if matches!(&self.k6[gi].1, Inst::Guard(g) if g.f_494_oid != 0) {
                self.ee_sub_1f1cf8(gi, h);
            }
        }
    }
    /// the live guard positions FUN_001a62e8 reads through the host (R76 k6_live)
    fn k6_publish_live(&mut self, h: &mut dyn ClassHost) -> Vec<(u32, [f32; 3])> {
        let guards: Vec<(u32, [f32; 3])> =
            self.k6.iter().filter_map(|(i, x)| if let Inst::Guard(g) = x { (!g.removed && !h.lo_removed(*i)).then_some((*i as u32 + 1, g.body.pos)) } else { None }).collect();
        h.k6_live_pos(&guards);
        let yaws: Vec<(u32, f32)> =
            self.k6.iter().filter_map(|(i, x)| if let Inst::Guard(g) = x { (!g.removed && !h.lo_removed(*i)).then_some((*i as u32 + 1, g.body.yaw)) } else { None }).collect();
        h.k6_live_yaw(&yaws);
        guards
    }
    /// K6 per frame: slot 15 FUN_00207350 of every guard
    pub(super) fn frame_k6(&mut self, h: &mut dyn ClassHost) {
        let guards = self.k6_publish_live(h);
        let players = self.g.k6_players.clone();
        let ptgt = self.g.k6_player_tgt;
        let tgt = |o: u32| -> Option<sensor::Tgt> {
            if let Some(p) = players.iter().find(|p| p.0 == o) {
                // WIRE-B3: Sly's JT facts (dying / +0x11ac / FUN_00154770 cover) when the player layer runs
                if let Some(t) = ptgt.filter(|_| o == PLAYER_HANDLE) {
                    return Some(sensor::Tgt { pos: p.1, ..t });
                }
                return Some(sensor::Tgt { pos: p.1, c33: true, c34: true, active: true, ..Default::default() });
            }
            guards.iter().find(|g| g.0 == o).map(|g| sensor::Tgt { pos: g.1, c33: true, active: true, ..Default::default() })
        };
        let is_guard = |o: u32| guards.iter().any(|g| g.0 == o);
        let (now, dt) = (self.g.t, self.g.dt);
        let cmds = &self.k6l.cmds;
        for (i, x) in &mut self.k6 {
            if let Inst::Guard(g) = x {
                // K24: removed from the world (vtable +0x4c, the class-102 knock-out): no slot 15
                // POOL1 R69: slot 15 runs from the world update list only (LO +0x28 bit 60): a guard out of the
                // world (load-time unplaced copy / field-26 Remove, or a script / job-toggle Remove) is not updated
                if g.removed || h.lo_removed(*i) {
                    continue;
                }
                // FUN_0015e9c8 over the +0x394 classes: guards only (players are static entries)
                let cands: Vec<(u32, [f32; 3])> = if g.f_394.is_empty() { Vec::new() } else { guards.clone() };
                // BOSS: FUN_00176098 region compare (FUN_001c0d70 on DAT_003e1a5c)
                if let (Some(tr), Some(t)) = (self.k6l.regions.as_ref(), tgt(g.brain.sub.obj)) {
                    g.body.tgt_region = tr.ee_sub_1c0d70(t.pos, true);
                    g.body.region_diff = g.body.tgt_region != tr.ee_sub_1c0d70(g.body.pos, true);
                } else {
                    g.body.region_diff = false;
                }
                g.body.f_31c = g.f_31c;
                g.body.f_fc8 = g.f_fc8;
                g.body.f_324 = g.f_324;
                // ANIM R27: the routine's FUN_0011d448 draws (FUN_00179e80 / FUN_00179ef8) use the shared rand state
                std::mem::swap(&mut self.rng, &mut g.body.rng);
                let xy0 = [g.body.pos[0], g.body.pos[1]];
                g.ee_sub_207350(*i, now, dt, &cands, &tgt, &is_guard, false, cmds, h);
                std::mem::swap(&mut self.rng, &mut g.body.rng);
                // K24: the REPLACED PO of a lifted guard
                let ground = g.k24_ground(h, xy0 != [g.body.pos[0], g.body.pos[1]]);
                g.k24_po_step(dt, ground);
                // CRAWL R95: a guard Murray carries rides his hand (REPLACED attach, as the host's so_carry position
                // FUN_001a62e8 reads), so his FUN_001b7970 throw starts the guard's flight from the hand (MREC9:
                // the thrown guard reaches the pump volume 125273); before, the body stayed where it was grabbed
                // (opt-out SLY2_NO_R95_CARRY)
                if std::env::var_os("SLY2_NO_R95_CARRY").is_none() {
                    if let Some(p) = h.so_carried_pos(*i as u32 + 1) {
                        g.body.pos = p;
                    }
                }
                // K11: attack records (chase substate polls FUN_00175ff0; module doc of `attack`)
                let tp = tgt(g.brain.sub.obj).map(|t| t.pos);
                // BOSS: 0xb4 state 1 runs the 0xa5 chase (+0x40 = 1); 0xb5 state 1 -> FUN_001758a0 -> state 3
                // (0xa4 attack), state 3 ends with FUN_00175748
                let chase = match &g.brain.child {
                    brain::Child::Chase(..) => tp,
                    brain::Child::Perch(p) if p.chase.is_some() => tp,
                    _ => None,
                };
                if let (brain::Child::Perch(p), Some(t)) = (&mut g.brain.child, tp) {
                    if let Some(b) = p.b5.as_mut() {
                        let mut cx = patrol::Cx { guard: *i, now, dt, body: &mut g.body, h };
                        if b.want_attack(now) {
                            if let Some(k) = g.atk.ee_sub_1758a0(*i, cx.body.pos, cx.body.yaw, t, now, &mut self.rng, cx.h) {
                                g.atk.ee_sub_20ab68(Some(k), now, &mut self.rng);
                                g.atk.started_pending = true;
                                if let Some(r) = g.perch_recs.get(b.rec) {
                                    b.ee_sub_155ae0(3, r, t, &mut cx);
                                }
                            }
                        } else if b.st == 3 && g.atk.f_f00.is_none() {
                            if let Some(r) = g.perch_recs.get(b.rec) {
                                b.attack_done(r, t, &mut cx);
                            }
                        }
                    }
                }
                let n0 = g.atk.started;
                g.atk.step(*i, g.body.pos, g.body.yaw, chase, &players, now, &mut self.rng, h);
                // BOSS: FUN_00209278 binds the record's anim on the guard (FUN_0012f9b8 / the SMA's ASEGA): its
                // non-shape events (0x11 emits, 7 / 8 fades, sounds) run on the anim system (k25 / k28)
                if g.atk.started != n0 || g.atk.started_pending {
                    g.atk.started_pending = false;
                    if let Some(c) = g.atk.f_f00.and_then(|k| g.atk.recs.get(k)).and_then(|r| r.clips.get(r.cur)) {
                        h.ee_sub_12f9b8(0f32.to_bits(), 1.0, c.aseg as u32 + 1, *i as u32 + 1, 0);
                    }
                }
            }
        }
        // CRAWL R26: FUN_00207350: +0x48c set -> FUN_001f1cf8 retry
        for gi in 0..self.k6.len() {
            if matches!(&self.k6[gi].1, Inst::Guard(g) if g.f_48c != 0 && !g.removed) {
                self.ee_sub_1f1cf8(gi, h);
            }
        }
        self.frame_k7(h);
    }
    /// K7: slot 73 FUN_001f7410 contact damage of every guard, hits delivered by [`Self::ee_sub_1f7598`]
    pub fn frame_k7(&mut self, h: &mut dyn ClassHost) {
        let mut hits = Vec::new();
        let guards: Vec<u32> = self.k6.iter().map(|x| x.0 as u32 + 1).collect();
        for (i, x) in &self.k6 {
            let mut cs = h.contacts(*i);
            if let Inst::Guard(g) = x {
                cs.extend(g.atk.pending.iter().cloned());
            }
            if cs.is_empty() {
                continue;
            }
            // FUN_001f7530: the player / guards accept (REPLACED: obj +0x110 kind bytes are not loaded)
            let mut accept = |o: u32, _k: i32| if o == PLAYER_HANDLE || guards.contains(&o) { o } else { 0 };
            c56::Guard::ee_sub_1f7410(*i, &cs, &mut accept, &mut |x| hits.push(x));
        }
        for x in hits {
            self.ee_sub_1f7598(&x, h);
        }
    }
    /// FUN_001f7598(hit): to a guard -> its slot 74 FUN_00207468; to the player -> `ClassHost::ee_sub_1f7598`
    /// (the JT's slot 74 = player::p7_damage::FUN_001a19a0). The 0x3e15c0 recent-hit dedupe is the target's.
    pub fn ee_sub_1f7598(&mut self, hit: &slots::GuardHit, h: &mut dyn ClassHost) -> bool {
        if hit.target == PLAYER_HANDLE {
            h.ee_sub_1f7598(hit);
            return true;
        }
        let k = hit.target as usize;
        // POOL1 r37: class 101 / 102 bombs (bentley_mine etc.) slot 74 FUN_00224190 (vt+0x25c): flying / stuck -> 3
        if k > 0 && self.k15_101_get(k - 1).is_some() {
            return self.k15_101_with(k - 1, h, |b, cx| b.ee_sub_224190(k - 1, cx)).unwrap_or(false);
        }
        match self.k6.iter_mut().find(|x| x.0 + 1 == k) {
            Some((i, Inst::Guard(g))) => {
                // HEIST r35: FUN_001f7598 sets +0x44 = FUN_001f78d8(target, +0xc kind) for every target (the guard's
                // ALO +0x110 reaction bytes) before vt+0x25c; the caller's +0x44 is overwritten
                let mut x = hit.clone();
                x.f_44 = crate::engine::player::p8_hazard::FUN_001f78d8(&h.alo_110(hit.target), hit.kind as u32) as _;
                g.ee_sub_207468(*i, &x, false, h)
            }
            // WIRE-EP8: class-98 staged switch slot 74 FUN_001badf0 (hit +0xc = touch +0xc, +0x40/+0x44 words)
            _ => match self.k5.iter_mut().find(|x| x.0 + 1 == k) {
                Some((i, Inst::C2d4b78(c))) => {
                    // FUN_001f7598: +0x44 = FUN_001f78d8(target, +0xc kind) (target ALO +0x110 bytes)
                    let f44 = crate::engine::player::p8_hazard::FUN_001f78d8(&h.alo_110(hit.target), hit.kind as u32) as u32;
                    let t = crate::engine::classes::actor::alo::Touch { other: hit.src, w_c: hit.kind as u32, flags_40: (f44 as u64) << 32, w_44: f44 };
                    c.ee_sub_1badf0(*i, &t, h) != 0
                }
                _ => {
                    // WIRE-EP8: class-83 switch slot 74 FUN_00148f20, +0x44 = FUN_001f78d8(target, +0xc kind)
                    let f44 = crate::engine::player::p8_hazard::FUN_001f78d8(&h.alo_110(hit.target), hit.kind as u32);
                    let mut x = hit.clone();
                    x.f_44 = f44 as _;
                    self.k15_83_hit(k - 1, &x, h).unwrap_or(false)
                }
            },
        }
    }
    /// FUN_00171cf8(guard, routine) for the guard at object index `obj` (level-script method field 317)
    pub fn k6_run_routine(&mut self, obj: usize, routine: usize, h: &mut dyn ClassHost) {
        let (now, dt) = (self.g.t, self.g.dt);
        let cmds = &self.k6l.cmds;
        if let Some((_, Inst::Guard(g))) = self.k6.iter_mut().find(|x| x.0 == obj) {
            g.ee_sub_171cf8(obj, routine, now, dt, cmds, h);
        }
    }
    /// K27: guards (handle) in brain substate 5 with their routine (class-113 object handle): the
    /// FUN_00171da8 view of the sequence action-0x55 wait
    pub fn k27_running(&self) -> Vec<(u32, u32)> {
        self.k6
            .iter()
            .filter_map(|(i, x)| match x {
                Inst::Guard(g) if g.brain.sub.code == 5 => g.brain.sub.routine.and_then(|r| self.k6l.routines.get(r)).map(|r| (*i as u32 + 1, r.0 as u32 + 1)),
                _ => None,
            })
            .collect()
    }
    /// K27: a sequence guard action on the guard brain: FUN_00171cf8 (routine handle -> level routine; not a
    /// guard or not a routine = FUN_00205458 / the routine lookup find nothing) / FUN_00171d58 (substate {1})
    pub fn k27_guard_op(&mut self, op: crate::engine::classes::k22_seq::GuardOp, h: &mut dyn ClassHost) {
        use crate::engine::classes::k22_seq::GuardOp;
        match op {
            GuardOp::Run(g, r) => {
                let Some(ri) = self.k6l.routines.iter().position(|x| x.0 as u32 + 1 == r) else { return };
                self.k6_run_routine(g as usize - 1, ri, h);
            }
            GuardOp::Slot82(o, a, b) => {
                // vtable +0x29c = FUN_001442d0 for all players (vtables 0x2cb230 / 0x2d2e98 / 0x2d44c8)
                let o = o as usize - 1;
                if self.k18_with(o, h, |c, cx| c.ee_sub_1442d0(o, a, b != 0, cx.h)).is_none()
                    && self.k19_murray_with(o, h, |c, cx| c.b.ee_sub_1442d0(o, a, b != 0, cx.h)).is_none()
                {
                    self.jt_slot_82_83(o, a, b != 0, true, h);
                }
            }
            GuardOp::Slot83(o, a, b) => {
                // vtable +0x2a4 = FUN_00144338
                let o = o as usize - 1;
                if self.k18_with(o, h, |c, cx| c.ee_sub_144338(o, a, b != 0, cx.h)).is_none()
                    && self.k19_murray_with(o, h, |c, cx| c.b.ee_sub_144338(o, a, b != 0, cx.h)).is_none()
                {
                    self.jt_slot_82_83(o, a, b != 0, false, h);
                }
            }
            // ANIM-ALL R35: sequence action 0x3b / FUN_001e54b0 binocucom calls: Bentley (K18, own +0xf00), else
            // Sly's JT (queued for player/). ANIM-ALL R43: Murray (K19 C2d44c8) runs the same FUN_0019cc50 on his +0xf00.
            GuardOp::Binoc(o, m, a, b) => {
                let i = o as usize - 1;
                if self.k18_with(i, h, |c, cx| c.binoc_19cc50(i, m, a, b, cx)).is_none() && self.k19_murray_with(i, h, |c, cx| c.b.binoc_19cc50(i, m, a, b, cx)).is_none() {
                    self.jt_binoc_ops.push(op);
                }
            }
            GuardOp::BinocLeave(o) => {
                // FUN_0019cce8: +0x38 null (REPLACED as in player::p15_binoc) -> FUN_0019cd38(2)
                let i = o as usize - 1;
                if self.k18_with(i, h, |c, cx| c.binoc_19cd38(i, 2, cx)).is_none() && self.k19_murray_with(i, h, |c, cx| c.b.binoc_19cd38(i, 2, cx)).is_none() {
                    self.jt_binoc_ops.push(op);
                }
            }
            GuardOp::Stop(g) => {
                let (now, dt) = (self.g.t, self.g.dt);
                let cmds = &self.k6l.cmds;
                if let Some((_, Inst::Guard(gd))) = self.k6.iter_mut().find(|x| x.0 + 1 == g as usize) {
                    std::mem::swap(&mut self.rng, &mut gd.body.rng);
                    gd.k21_goal(g as usize - 1, 1, now, dt, cmds, h);
                    std::mem::swap(&mut self.rng, &mut gd.body.rng);
                }
            }
        }
    }
    /// guard state at object index `obj`
    /// mutable guard at object index `obj` (tests / placement)
    pub fn k6_guard_mut(&mut self, obj: usize) -> Option<&mut c56::Guard> {
        self.k6.iter_mut().find(|x| x.0 == obj).and_then(|x| if let Inst::Guard(g) = &mut x.1 { Some(&mut **g) } else { None })
    }
    /// HEIST r19: FUN_001cb250 copy `obj` (a script method-1019 hand-out, e.g. the EP4 Wall Bombing guards 11539
    /// from class-6 spawner 77180) of a guard class: the copy's vtable +0x12c / +0x44 rebuild at the spawner frame
    /// (FUN_001caec8: the spawner's world matrix) as the K21 path (k21_119.rs `Host119::ee_sub_1cb250`): slot 6
    /// make + FUN_002061d8 + FUN_00206518 (+ the FUN_001f1a40 loot list) + FUN_002079e0. Other classes: no-op.
    pub fn k6_pool_guard(&mut self, w: &World, obj: usize, place: crate::engine::gfx::scene::Affine, sp: Option<usize>, h: &mut dyn ClassHost) -> bool {
        let Some(&(_, vt)) = PORTED_K6.iter().find(|x| x.0 == w.objects[obj].class) else { return false };
        self.k6.retain(|x| x.0 != obj);
        self.g.k7_live.retain(|&g| g != obj as u32 + 1);
        let mut bw = BoneWorlds::default();
        let mut g = k6_make_guard(w, &mut bw, obj, vt, Some(place), h);
        // CRAWL R89: the spawner's key-6 field-273 row (FUN_0013e818 -> setter FUN_0020a540: +0x2e8 routine), as
        // k6_make_guard's own field 273 (opt-out SLY2_NO_R89_RT)
        if let Some(sp) = sp.filter(|_| std::env::var_os("SLY2_NO_R89_RT").is_none()) {
            let rows = los::spawner_key6_props(w, sp, 273);
            if !rows.is_empty() {
                g.feed(&rows);
                g.f_2e8 = find_under(w, g.f_2e8_oid, obj).filter(|&k| w.objects[k].class == 113).map(|k| w.objects[..k].iter().filter(|o| o.class == 113).count());
            }
        }
        g.ee_sub_2061d8(obj, h);
        let players = self.g.player_globals();
        g.ee_sub_206518(obj, &players, self.g.t, h);
        if g.f_494_oid != 0 {
            g.ee_sub_1f1a40(obj, &mut self.rng, h);
        }
        g.ee_sub_2079e0(obj, &mut self.g.k7_live, h);
        self.k6.push((obj, Inst::Guard(g)));
        h.lo_rt(obj, true);
        // CRAWL R83: FUN_001caec8's vt +0x54 placed the copy's world matrix at the claim, so FUN_001a62e8 (e.g. the
        // row-7 pump generator's (19960 .108 (guard .34)) right after method 1019) reads the new place at once, not
        // the copy's stale / (0,0,0) position until the next K6 frame. Opt-out SLY2_NO_R83_POOLPOS.
        if std::env::var_os("SLY2_NO_R83_POOLPOS").is_none() {
            self.k6_publish_live(h);
        }
        true
    }
    pub fn k6_guard(&self, obj: usize) -> Option<&c56::Guard> {
        self.k6.iter().find(|x| x.0 == obj).and_then(|x| if let Inst::Guard(g) = &x.1 { Some(&**g) } else { None })
    }
}
