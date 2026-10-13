//! WP-K6: guards = the class 56 family (concrete 57-67). `c56` = the shared class-56 overrides, `sensor` = the
//! +0xf64 sensor object, `brain` = the root goal 0x9c, `patrol` = the routine / rail / goto / walk goals.
//! Instances live in `ClassSet::k6`. The concrete classes 57-67 override only presentation / special-move
//! slots (see K6: Findings); they all run the class-56 core here.
pub mod attack;
pub mod brain;
pub mod c56;
pub mod los;
pub mod slots;
pub mod patrol;
pub mod sensor;
pub mod search;
pub mod lift;
pub mod hit;

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
}

/// Rail of a class-10 object (spline kinds walked as polylines, as `p6::path_of` for kind 0).
pub fn rail_of(w: &World, i: usize) -> Option<Path> {
    if let Some(p) = path_of(w, i) {
        return Some(p);
    }
    let Payload::Loaded(LoadData::B4(bx)) = &w.objects[i].payload else { return None };
    let B4Load::L1e5df8 { xform, a, b, p0, .. } = &**bx else { return None };
    if p0.len() < 2 {
        return None;
    }
    let m = crate::engine::gfx::attach::affine_from_bytes(&xform.xform);
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
                    6 => {
                        let rail = w
                            .find_oid(e.id)
                            .iter().copied().find(|&k| crate::engine::player::p4::FUN_00132b10(w.objects[k].class as i32, 10))
                            .and_then(|k| rail_of(w, k));
                        Cmd::Rail(rail, e.f_8[0] as i32, f32::from_bits(e.f_8[1]))
                    }
                    7 => match find_under(w, e.id, i) {
                        Some(k) => Cmd::Goto(bw.xfm_world(w, k).unwrap_or_else(|| bw.get(w, k, 0))[3], None),
                        None => Cmd::Other(7),
                    },
                    9 => Cmd::Label(e.id),
                    10 => Cmd::Jump(e.id),
                    k => Cmd::Other(k),
                })
                .collect::<Vec<_>>();
            l.cmds.push(cmds.clone());
            l.routines.push((i, cmds));
        }
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
    g.body.c3a = o.class == 0x3a;
    g.init(i, h);
    g.feed(props);
    // K11: methods 232 / 233 attack oids (+0xeb8), resolved like slot 14 FUN_0020aaa8
    let oids: Vec<i32> = props
        .iter()
        .filter(|p| p.field == 232 || p.field == 233)
        .flat_map(|p| attack_oids(&p.value))
        .filter(|&o| o != 0)
        .collect();
    g.atk = attack::Attacks::ee_sub_20aaa8(w, src, &oids);
    for which in 0..2 {
        let oid = g.vol_oids[which];
        if oid == 0 {
            continue;
        }
        let Some(k) = find_under(w, oid, i) else { continue };
        let solid = match &w.objects[k].payload {
            Payload::Loaded(LoadData::Solid(s)) => s,
            Payload::Loaded(LoadData::L2282b0 { solid, .. }) => solid,
            _ => continue,
        };
        let Some(mut mw) = bw.xfm_world(w, k) else { continue };
        if let (Some(pm), true) = (&place, is_under(w, k, i)) {
            mw = mul(&mw, pm);
        }
        let v = sensor::Vol::new(k, invert(&mw), build_geom(&solid.geom), build_bsp(&solid.bsp));
        c56::set_volume(&mut g, which, v);
    }
    g
}

/// handle of a player-character global (DAT_002e1e40 ...) that is not a world object
pub const PLAYER_HANDLE: u32 = 0x8000_0000;

impl ClassSet {
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
        let players: Vec<u32> = self.g.k6_players.iter().map(|p| p.0).collect();
        for (i, x) in &mut self.k6 {
            if let Inst::Guard(g) = x {
                g.ee_sub_2061d8(*i, h);
                g.ee_sub_206518(*i, &players, self.g.t, h);
                // slot 22 FUN_002079e0 (the K6 add-to-world above is slot 25's FUN_001f3ea8 part)
                g.ee_sub_2079e0(*i, &mut self.g.k7_live, h);
            }
        }
    }
    /// K6 per frame: slot 15 FUN_00207350 of every guard
    pub(super) fn frame_k6(&mut self, h: &mut dyn ClassHost) {
        let guards: Vec<(u32, [f32; 3])> =
            self.k6.iter().filter_map(|(i, x)| if let Inst::Guard(g) = x { (!g.removed).then_some((*i as u32 + 1, g.body.pos)) } else { None }).collect();
        let players = self.g.k6_players.clone();
        let tgt = |o: u32| -> Option<sensor::Tgt> {
            if let Some(p) = players.iter().find(|p| p.0 == o) {
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
                if g.removed {
                    continue;
                }
                // FUN_0015e9c8 over the +0x394 classes: guards only (players are static entries)
                let cands: Vec<(u32, [f32; 3])> = if g.f_394.is_empty() { Vec::new() } else { guards.clone() };
                g.ee_sub_207350(*i, now, dt, &cands, &tgt, &is_guard, false, cmds, h);
                // K24: the REPLACED PO of a lifted guard
                g.k24_po_step(dt);
                // K11: attack records (chase substate polls FUN_00175ff0; module doc of `attack`)
                let chase = matches!(g.brain.child, brain::Child::Chase).then(|| tgt(g.brain.sub.obj)).flatten().map(|t| t.pos);
                g.atk.step(g.body.pos, g.body.yaw, chase, &players, now, &mut self.rng);
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
        match self.k6.iter_mut().find(|x| x.0 + 1 == k) {
            Some((i, Inst::Guard(g))) => g.ee_sub_207468(*i, hit, false, h),
            _ => false,
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
            GuardOp::Stop(g) => {
                let (now, dt) = (self.g.t, self.g.dt);
                let cmds = &self.k6l.cmds;
                if let Some((_, Inst::Guard(gd))) = self.k6.iter_mut().find(|x| x.0 + 1 == g as usize) {
                    gd.k21_goal(g as usize - 1, 1, now, dt, cmds, h);
                }
            }
        }
    }
    /// guard state at object index `obj`
    pub fn k6_guard(&self, obj: usize) -> Option<&c56::Guard> {
        self.k6.iter().find(|x| x.0 == obj).and_then(|x| if let Inst::Guard(g) = &x.1 { Some(&**g) } else { None })
    }
}
