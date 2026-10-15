//! WP-K2: Bevy wiring of the class overrides. `ClassesLevel` (set by the game layer on world activation) ->
//! build `ClassSet` (construct + init slot 6 + post-load slot 14) -> per frame (Update) the per-frame slots.
//! REPLACED: the engine walks the world's object lists; here the set owns per-class state by object index.
//! Engine calls without a port land in `EngineHost` counters (explicit hooks, see K2: Findings).
use super::{ClassHost, ClassSet};
use crate::engine::physics::driven::GameRng;
use crate::engine::world::World;
use bevy_app::{App, Plugin, Update};
use bevy_ecs::prelude::*;
use std::sync::Arc;

/// Active world (None = no level).
#[derive(Resource, Default, Clone)]
pub struct ClassesLevel(pub Option<Arc<World>>);

/// WP-SCR2: one branch outcome of FUN_00143ee0 for Sly, enqueued by `EngineHost::k22_pose_143ee0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerPoseReq {
    /// slot-87 index (FUN_00143ee0 param_4; `Clear` always restores idx 0 through FUN_00143ff8)
    pub idx: u32,
    /// the oid that resolved (0 for `Clear`, oid -1 branch), the resolved object (anim or sm), and the
    /// FUN_001305b8 / FUN_001f1448 flags (rec[6] | 1)
    pub kind: PlayerPoseKind,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayerPoseKind {
    /// oid == -1: FUN_00143ff8 (restore slot-87 idx 0 when the current pose is still 5 / 6 state 10 / 11)
    Clear,
    /// (vt+0x2c4)(subj, idx) then FUN_00130658(subj + 0x1030) + FUN_001f1448(sm, subj, -1, flags, &subj + 0x1034)
    Sm { obj: u32, flags: u32 },
    /// (vt+0x2c4)(subj, idx) then FUN_001f15b8(subj + 0x1034) + FUN_001305b8(0, 1.0, anim, subj, flags, &subj + 0x1030)
    Anim { obj: u32, flags: u32 },
}

/// Host recording the hook calls that have no engine port yet.
#[derive(Default, Clone)]
pub struct EngineHost {
    pub emits: usize,
    pub particles: i64,
    pub sounds: usize,
    pub draws: usize,
    /// K3: world the oid lookups (FUN_0015e938) resolve against
    pub world: Option<Arc<World>>,
    /// K3: oid lookups that found an object / deferred calls queued / template pushes
    pub found: usize,
    pub deferred: usize,
    pub tpl_pushes: usize,
    /// WP-SCR10: warp-volume task entries (FUN_002284e0) keyed by world object index,
    /// fed from SpawnLists::warp_tasks after level scripts run.
    pub warp_tasks: std::collections::HashMap<usize, Vec<[i32; 2]>>,
    /// K3: virtual calls on objects outside the set
    pub vcalls: usize,
    /// K7: world +0x3d84 no-detection counter (FUN_0020f1b8), vision blockers (class 23 list DAT_003e1480),
    /// SO collision entries for the LOS raycast, built for the world `k7_for`
    pub world_3d84: i32,
    pub blockers: Option<Arc<super::guard::los::Blockers>>,
    pub so: Option<Arc<crate::engine::physics::query::CollisionWorld>>,
    pub k7_for: usize,
    /// K7: hits delivered to Sly (FUN_001f7598 with the player as target), drained by the player layer
    pub player_hits: Vec<super::guard::slots::GuardHit>,
    /// K7: sensor-state script events (FUN_001a61e0 msg 0x1c) raised
    pub sensor_events: usize,
    /// K7: contact pairs per guard object for slot 73 (fed by the contact solver; the SO dynamics contact
    /// list +0xec -> +0x6c is not built for guards yet), drained each frame
    pub k7_contacts: Vec<(usize, super::guard::slots::Contact)>,
    /// K8: Sly's pickup record (player +0x15c: alive, radius; None = no record until the player layer feeds
    /// it), Sly's position (FUN_001a62e8 of the player handle), hp / mp gifts of class-93 pickups (player slot
    /// 80, drained by the player layer)
    pub player_rec: Option<super::k8::PickRec>,
    pub player_pos: [f32; 3],
    pub player_gifts: Vec<(i32, i32)>,
    /// K11: FUN_0020fff0 per entry of FUN_0020ff70 (flags 1): (ALO object index, FUN_001f67e0 state-machine
    /// id, FUN_001f0660 state); the oid lookup FUN_0015e650 + is-a 0x30 are done here, the SM set is drained
    /// by the anim layer
    pub sm_states: Vec<(usize, u32, i32)>,
    /// K11: FUN_00161f48 entries resolved (FUN_0015e938 kind 0x105): (object index, FUN_00164640 signal arg)
    pub signals: Vec<(usize, u32)>,
    /// K11: FUN_001dfd58(seq, player) sequence starts, drained by game/ (J7 sequence runner)
    pub seq_starts: Vec<(u32, u32)>,
    /// K22: sequence action 0x44 (FUN_001dd820(0x3da530)) requests and the title-active flag the ui/ layer
    /// (owner of 0x3da530) should mirror; UNPORTED sequence actions (owner, subject, record)
    pub k22_title_shows: u32,
    /// A14: FUN_001b3a48(0x2f66b0, desc) movie requests (class 26 +0x14c[k]), drained by ui/ into J16's FMV queue
    pub movies: Vec<[u8; 64]>,
    /// A14: FUN_00161868 (coin HUD) / FUN_001618b0 (clue-bottle HUD) calls, read by ui/ (world+0x3bac / +0x3bb8 labels)
    pub hud_coins: u32,
    pub hud_bottles: u32,
    /// WP-K33: last FUN_001618f8 / FUN_00161958 countdown request of the loot system (request count, seconds;
    /// < 0 = stop), read by ui/ (world+0x3bc4 label)
    pub hud_timer: (u32, f32),
    /// WP-UI4: FUN_001a3078 (on) / FUN_001a30d0 (off) of the HUD object 0x2e1e60 (screen element 0x373, the
    /// vault / safe indicator shown while world +0x3d80 > 0; class-78 Vault state 2 edges), read by ui/
    /// (request count, shown)
    pub hud_vault: (u32, bool),
    pub k22_title_active: bool,
    pub k22_seq_actions: Vec<(u32, u32, [u32; 20])>,
    pub k22_po_switches: Vec<u32>,
    /// WP-SCR2: FUN_00143ee0 (slot-87 pose) requests for Sly, written by `k22_pose_143ee0` for subjects
    /// that resolve to PLAYER_HANDLE and drained by player::plugin::link_classes. Each entry is the ELF
    /// decomp branch outcome (oid resolved to object, is-a 0x6d anim / 0x6f sm, else `Clear` for oid -1).
    pub player_pose: Vec<PlayerPoseReq>,
    /// WP-SCR2: FUN_00144038(Sly) (slot-88 index is 5 / 6 = states 10 / 11, the pose is active). Written
    /// by the player layer each frame, read by the k22 wait condition case 0xa to keep the sequence node
    /// blocked while the pose plays.
    pub player_posing: bool,
    /// K22: the DAT_003dd4bc object set (FUN_0011a220 add / FUN_0011a2e8 remove / FUN_0011a380 member), handles
    pub set_3dd4bc: Vec<u32>,
    /// K11: engine calls reached with no ported callee (explicit Unported errors)
    pub unported: Vec<String>,
    /// K13: Sly exists (PlayerFocus valid): FUN_001c4840 returns PLAYER_HANDLE (the class-53 JT), else 0
    pub player_valid: bool,
    /// K13: rest/placed world matrices of world objects (FUN_001a62e8 row 3; K24: FUN_001a6318 / vtable +0xfc),
    /// cached for the world `pos_for`
    pub pos_cache: std::collections::HashMap<usize, crate::engine::gfx::scene::Affine>,
    pub pos_for: usize,
    /// WP-J17: collision volumes (+0xec GEOM / BSP) of class-26 warp objects, per (world, object index)
    pub warp_vols: std::collections::HashMap<(usize, usize), Option<Arc<(crate::engine::physics::geom::CGeom, crate::engine::physics::geom::BspC)>>>,
    /// K16: class-10 path data (FUN_001e5df8 / FUN_001e6180, world frame) per object index, for the world `path_for`
    pub paths: std::collections::HashMap<usize, Option<Arc<crate::engine::player::p5_target::Path>>>,
    pub path_for: usize,
    /// P20: shadow JT of the controlled Bentley / Murray (`ClassHost::po_call`), set by the player layer
    pub po: Option<crate::engine::player::p20::PoHost>,
    /// P22: this frame's FUN_0020bb58(u, blk) calls of the controlled Bentley / Murray (obj, blk, u), read by
    /// the player layer (player::p22)
    pub po_bb58: Vec<(usize, u32, f32)>,
    /// P23: this frame's slot-78 / FUN_002206c8 switch requests of the class code, applied by the player layer
    /// (player::p23::apply_switches)
    pub p23_switch: crate::engine::player::p23::SwitchQueue,
    /// P22: a K22 switch to Sly (class 53, not a k18 object) this frame: the player layer releases the
    /// controlled Bentley / Murray (player::p22::set_active_character(None))
    pub po_release: bool,
    /// K21: the camera FUN_0014f308 projects with (None = no camera: the class-119 probes all fail)
    pub cam: Option<super::k21_119::CamView>,
    /// K25: run-time ASEGAs / SMAs of the class anim hooks (classes/k25.rs)
    pub k25: super::k25::AnimRt,
    /// K25: sound emitters of the class sound hooks (classes/k25.rs)
    pub k25_snd: super::k25::SoundRt,
    /// K25: class-owned body snapshots + run-time constraint records of the class physics hooks (classes/k25.rs)
    pub k25_phys: super::k25::PhysRt,
    /// K25: field overlay of world objects not owned by the class set (classes/k25.rs FieldRt)
    pub k25_fld: super::k25::FieldRt,
    /// K26: class-pool emitters handed to the host audio (classes/k26.rs SoundOut)
    pub k26_out: super::k26::SoundOut,
    /// K26: anim event dispatch / SMA message statistics and queues (classes/k26.rs)
    pub k26: super::k26::K26Rt,
    /// K26: world matrices of the objects posed by the class-run ASEGAs this frame (classes/k26.rs k26_bind)
    pub k26_posed: std::collections::HashMap<usize, crate::engine::gfx::scene::Affine>,
    /// WP-EP1d: run-time SO collision part +0x10 bit 50 state (FUN_001f6ac8 / 001f6b48: (object index, part)
    /// -> linked), part +0x18 words (FUN_001f6bc8), bumped `so_gen` on every change (the player layer
    /// rebuilds its collision set when it moves), for the world `so_for`
    pub so_parts: std::collections::HashMap<(usize, usize), bool>,
    pub so_part_18: std::collections::HashMap<(usize, usize), u32>,
    pub so_gen: u32,
    /// WP-EP1d: SetParent (FUN_001a58d0 -> vtable +0x54, ALO FUN_001f21b8) run-time hierarchy: object index ->
    /// (new parent handle, 0 = none; matrix relative to it), for the world `pos_for`
    pub rt_parent: std::collections::HashMap<usize, (u32, crate::engine::gfx::scene::Affine)>,
    pub rt_depth: u32,
    /// K29: the ASEG -> bone binding (anim::bind, D6) of the class-run ASEGAs (k26_bind); shared on clone
    pub k29_binder: Arc<std::sync::Mutex<crate::engine::anim::bind::AnimBinder>>,
    /// K29: world matrices of the bones (object, bone) the class-run ASEGAs drive this frame (and their descendants)
    pub k29_bones: std::collections::HashMap<(usize, usize), crate::engine::gfx::scene::Affine>,
    /// K26: music manager 0x3dd040 layer keys / DAT_003dd268 instance sources (classes/k26.rs MusicRt); set
    /// from the level block before the class set is built (`k26_music_block`)
    pub k26_music: super::k26::MusicRt,
    /// K27: non-JT character PO bodies (classes/k27.rs) by object index
    pub k27_po: std::collections::HashMap<usize, super::k27::CharPo>,
    /// K27: anim event kind 2 (FUN_0014f750 StartCmShake kinds) of class-run ASEGAs, applied to g_cm
    /// by apply_class_shakes (each FUN_0014f750 overwrites kind / start: the last of the frame is kept)
    pub k27_shake: Option<i32>,
    /// K29: camera requests of the class code (FUN_0014e328 / FUN_0014e4c8 / sequence camera moves), run on
    /// g_cm in order by apply_class_camera (classes/k29_cam.rs)
    pub k29_cam: Vec<super::k29_cam::CamReq>,
    /// K30: camera state the class objects publish for the CM presets (classes/k30_cam.rs), handed to
    /// apply_class_camera
    pub k30_cam: std::collections::HashMap<u32, super::k30_cam::CamState>,
    /// K30: the zoom look's getters after the last camera apply (None before: the waits pass)
    pub k30_zoom_waits: Option<[bool; 3]>,
    /// K27: world +0x3d78 / +0x3d7c counters (action 0x12), pad lock bits 2 / 4 (FUN_00187078 / 001870a0),
    /// fades applied (FUN_001f7c68 / 001f7bf8)
    /// K27: FUN_001331f8 script messages (object + 1, msg, argument words) for the game layer (K28: delivered
    /// with FUN_00200da0 argument CRefs by game::vm_state::send_messages, classes/k28.rs)
    pub k27_script_msgs: Vec<(u32, u32, Vec<u32>)>,
    pub k27_cine: [i32; 4],
    pub k27_pad_lock: u32,
    /// PAD1: live pad state for ee_call(0x187d80/0x187de0) — written by game app or test harness each frame
    pub pad: Option<crate::engine::input::pad::PadState>,
    pub k27_fades: usize,
    /// K27: anim event kind 5 seeks applied (FUN_00130d30 on class-run ASEGAs)
    pub k27_seeks: usize,
    /// K28: ASEGA +0x58 (anim event kinds 9 / 10 world state) by ASEGA handle; event calls applied (kinds 7..0x10)
    pub k28_asega_58: std::collections::HashMap<u32, bool>,
    pub k28_events: usize,
    /// K30: anim event 0x11 slot-51 emit requests (target handle, request) for `ClassSet::k28_spawn`
    pub k30_reqs: Vec<(u32, [u32; 15])>,
    /// K30: world +0x476c (anim events 0x12 / 0x13, FUN_001618f8 / FUN_00161958) and the dispatch time
    pub k30_476c: f32,
    pub k30_t: f32,
    /// K28: job-manager calls of the class code (callee 0x164640 / 0x164df0 / 0x1619a0, object handle, k,
    /// ctx handle) for `jobs::JobMgr` (applied by game::vm_state::apply_job_calls)
    pub k28_task_calls: Vec<(u32, u32, i32, u32)>,
    /// K28: FUN_0015bd20 requests for `ClassSet::k28_spawn` (classes/k28_emit.rs)
    pub k28_emits: Vec<super::k28_emit::EmitReq>,
    /// J17: (emitter, template, link) of the FUN_0015b428 requests taken this frame, for the look handoff
    pub k28_looks: Vec<(usize, [u32; 64], bool)>,
}
impl EngineHost {
    /// K26: listener = camera position (g_cm), else the player
    pub fn k26_listener(&self) -> [f32; 3] {
        self.cam.as_ref().map_or(self.player_pos, |c| [c.mat[3][0], c.mat[3][1], c.mat[3][2]])
    }
    /// K26: class emitters that start now: (sound, volume) (classes/k26.rs SoundOut::due)
    pub fn k26_due_sounds(&mut self) -> Vec<(i32, f32)> {
        let mut out = std::mem::take(&mut self.k26_out);
        let mut snd = std::mem::take(&mut self.k25_snd);
        let l = self.k26_listener();
        let due = out.due(&mut snd, l, &mut |v| self.ee_sub_1a62e8(v as u32));
        self.k25_snd = snd;
        self.k26_out = out;
        due
    }
    /// K16: class-10 path of object handle `obj` (None = not a path object / no knots)
    pub fn path10(&mut self, obj: u32) -> Option<Arc<crate::engine::player::p5_target::Path>> {
        let w = self.world.clone()?;
        let i = (obj as usize).checked_sub(1).filter(|&i| i < w.objects.len())?;
        let id = Arc::as_ptr(&w) as usize;
        if self.path_for != id {
            self.paths.clear();
            self.path_for = id;
        }
        self.paths.entry(i).or_insert_with(|| super::shared::path10::world_path(&w, i, None).map(Arc::new)).clone()
    }
    /// K25: FUN_0012dc18(owner, owner + xf, prio, f_14, f_1c, v) on the run-time xf list of `owner`
    pub fn k25_con(&mut self, owner: u32, xf: u32, prio: i32, f_14: u32, f_1c: u32, v: [f32; 3]) -> u32 {
        let Some(w) = self.world.clone() else { return 0 };
        self.k25_phys.set_world(&w);
        let mut x = self.k25_phys.take_xf(owner, xf);
        let mut pool = std::mem::take(&mut self.k25_phys.pool);
        let r = super::k25::ee_sub_12dc18(&mut pool, owner, &mut x, prio, f_14, f_1c, v, self);
        self.k25_phys.pool = pool;
        if x.list.first().is_some_and(|a| a.id == r) {
            // the new head drives the xf (its +0x20 value is the target the act's +0xc activation sets)
            self.k25_phys.targets.insert(x.xf, v);
        }
        self.k25_phys.put_xf(x);
        r
    }
}
impl EngineHost {
    fn k7_cache(&mut self) -> Option<Arc<World>> {
        let w = self.world.clone()?;
        let id = Arc::as_ptr(&w) as usize;
        if self.k7_for != id {
            self.blockers = Some(Arc::new(super::guard::los::Blockers::build(&w)));
            self.so_parts.clear();
            self.so_part_18.clear();
            self.so = Some(Arc::new(crate::engine::physics::query::CollisionWorld::build_so(&w)));
            self.k7_for = id;
        }
        Some(w)
    }
}
impl std::fmt::Debug for EngineHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EngineHost {{ emits: {}, particles: {}, sounds: {}, draws: {}, found: {}, deferred: {} }}", self.emits, self.particles, self.sounds, self.draws, self.found, self.deferred)
    }
}
impl ClassHost for EngineHost {
    /// WP-J17: FUN_001a6f00(obj, pos, 0): world point -> obj-local (inverse of the FUN_001a62e8 world matrix)
    fn ee_sub_1a6f00(&mut self, obj: usize, pos: [f32; 3]) -> [f32; 3] {
        let Some(m) = self.k24_world_xf(obj as u32 + 1) else { return pos };
        let m = crate::engine::gfx::scene::invert(&m);
        [0, 1, 2].map(|k| pos[0] * m[0][k] + pos[1] * m[1][k] + pos[2] * m[2][k] + m[3][k])
    }
    /// WP-J17: FUN_00118fb0(+0xec, 0x7fffffff, local pos) != 0 of the class-26 warp volume (LoadData::L2282b0 solid)
    fn ee_sub_118fb0(&mut self, obj: usize, pos: [f32; 3]) -> bool {
        use crate::engine::physics::geom::{build_bsp, build_geom};
        use crate::engine::world::objects::{LoadData, Payload};
        let Some(w) = self.world.clone() else { return false };
        let v = self.warp_vols.entry((Arc::as_ptr(&w) as usize, obj)).or_insert_with(|| match w.objects.get(obj).map(|o| &o.payload) {
            Some(Payload::Loaded(LoadData::L2282b0 { solid, .. })) => Some(Arc::new((build_geom(&solid.geom), build_bsp(&solid.bsp)))),
            _ => None,
        });
        v.as_ref().is_some_and(|v| !v.1.nodes.is_empty() && crate::engine::physics::point::FUN_00118fb0(&v.0, &v.1, 0, 0x7fffffff, pos).is_some())
    }
    /// WP-J17: FUN_0022b490(0x3dd490, +0x144, spawn, -1, flags, a) of the class-26 warp object: the request
    /// goes to the `game::flow` mailbox with the warp's own descriptor (`flow::Dest::Desc`)
    fn ee_sub_22b490(&mut self, obj: usize, level: i32, flags: i32, a: i32) {
        use crate::engine::game::flow::{Dest, Post, FUN_0022b490};
        use crate::engine::world::objects::{LoadData, Payload};
        let Some(w) = &self.world else { return };
        let Some(Payload::Loaded(LoadData::L2282b0 { first, .. })) = w.objects.get(obj).map(|o| &o.payload) else { return };
        let dest = first.map_or(Dest::Current, |d| Dest::Desc(crate::engine::disc::descriptor::Descriptor(d)));
        FUN_0022b490(Post { dest, spawn: level, spawn_parent: -1, flags: flags as u32, a, slots: None });
    }
    /// A14: FUN_00161868(world) -> ui::hud_slots
    fn ee_sub_161868(&mut self) {
        self.hud_coins = self.hud_coins.wrapping_add(1);
    }
    /// A14: FUN_001618b0(world) -> ui::hud_slots
    fn ee_sub_1618b0(&mut self) {
        self.hud_bottles = self.hud_bottles.wrapping_add(1);
    }
    /// WP-UI4: FUN_001a3078(0x2e1e60, on) / FUN_001a30d0: show / hide the vault HUD object (screen element
    /// 0x373), driven by the class-78 Vault +0x3d80 counter edges -> ui::hud::vault
    fn ee_sub_1a3078(&mut self, on: bool) {
        self.hud_vault = (self.hud_vault.0.wrapping_add(1), on);
    }
    /// A14: FUN_001b3a48(0x2f66b0, +0x14c[k]) of the class-26 warp object (list of LoadData::L2282b0)
    fn ee_sub_1b3a48(&mut self, obj: usize, k: i32) {
        use crate::engine::world::objects::{LoadData, Payload};
        let Some(w) = &self.world else { return };
        if let Some(Payload::Loaded(LoadData::L2282b0 { list, .. })) = w.objects.get(obj).map(|o| &o.payload) {
            if let Some(Some(d)) = usize::try_from(k).ok().and_then(|k| list.get(k)) {
                self.movies.push(*d);
            }
        }
    }
    /// K22: the only `base(0x11a220, o)` caller is class 7 FUN_001beb28's FUN_0011a220(0x3dd4bc, o)
    fn base(&mut self, addr: u32, obj: usize) {
        if addr == 0x11a220 && !self.set_3dd4bc.contains(&(obj as u32 + 1)) {
            self.set_3dd4bc.push(obj as u32 + 1);
        }
    }
    /// K22: FUN_0011a380(0x3dd4bc, obj)
    fn ee_sub_11a380(&mut self, obj: u32) -> bool {
        self.set_3dd4bc.contains(&obj)
    }
    /// K22: FUN_0011a2e8(0x3dd4bc, obj)
    fn ee_sub_11a2e8(&mut self, obj: u32) {
        self.set_3dd4bc.retain(|&x| x != obj);
    }
    /// K22: FUN_001a61a0 - +0x28 bit 60 (in the world) is implied: members are added right after vt+0x44
    fn ee_sub_1a61a0(&mut self, obj: u32) -> bool {
        self.set_3dd4bc.contains(&obj)
    }
    fn k22_title_show_1dd820(&mut self) {
        self.k22_title_shows += 1;
        self.k22_title_active = true;
    }
    fn k22_title_active_1d9880(&mut self) -> bool {
        self.k22_title_active
    }
    fn k22_po_switch(&mut self, po: u32) {
        self.k22_po_switches.push(po);
    }
    fn k22_take_po_switches(&mut self) -> Vec<u32> {
        let mut v = std::mem::take(&mut self.k22_po_switches);
        // P22: Sly has no k18 slot 78 here (his JT is the player layer's): released through the player API
        if let Some(w) = self.world.clone() {
            v.retain(|&po| {
                let sly = (po as usize).checked_sub(1).and_then(|o| w.objects.get(o)).is_some_and(|x| x.class == crate::engine::player::p20::CLASS_SLY);
                self.po_release |= sly;
                !sly
            });
        }
        v
    }
    fn k22_seq_action(&mut self, owner: u32, subj: u32, rec: &[u32; 20]) {
        // WIRE-A: FUN_001e1e30 case 0xb: s = FUN_001e1db8(node, rec[4]); s is-a 0x72 (sequence) ->
        // FUN_001dfd58(s, subject). The node +0x50 = s wait (rec[1] == 0) is not modelled (outer timing only).
        if rec[0] == 0xb {
            let s = self.ee_sub_15e938(1, rec[4] as i32, subj);
            if s != 0 && self.ee_sub_132b10(s, 0x72) != 0 {
                self.ee_sub_1dfd58(s, subj);
                return;
            }
        }
        if self.k22_seq_actions.len() < 4096 {
            self.k22_seq_actions.push((owner, subj, *rec));
        }
    }
    /// WP-SCR2: FUN_00143ee0 for Sly (class 53 / PLAYER_HANDLE); other 0x34-derived subjects (Bentley 55,
    /// Murray 54) are UNPORTED here and recorded through `k22_seq_action` via the fallback below. The oid
    /// -> object resolve and is-a 0x6d / 0x6f test are the FUN_0015e938 / FUN_00132b10 engine calls.
    fn k22_pose_143ee0(&mut self, subj: u32, oid: i32, flags: u32, idx: u32) {
        if subj != super::guard::PLAYER_HANDLE {
            // record for debugging; actual pose on Bentley / Murray is UNPORTED
            if self.k22_seq_actions.len() < 4096 {
                let mut rec = [0u32; 20];
                rec[0] = 0xa;
                rec[4] = oid as u32;
                rec[5] = subj;
                rec[6] = flags;
                self.k22_seq_actions.push((0, subj, rec));
            }
            return;
        }
        if oid == -1 {
            self.player_pose.push(PlayerPoseReq { idx: 0, kind: PlayerPoseKind::Clear });
            return;
        }
        let o = self.ee_sub_15e938(1, oid, subj);
        if o == 0 {
            return;
        }
        if self.ee_sub_132b10(o, 0x6d) == 0 {
            if self.ee_sub_132b10(o, 0x6f) != 0 {
                self.player_pose.push(PlayerPoseReq { idx, kind: PlayerPoseKind::Sm { obj: o, flags } });
            }
        } else {
            self.player_pose.push(PlayerPoseReq { idx, kind: PlayerPoseKind::Anim { obj: o, flags } });
        }
    }
    fn k22_posing_144038(&mut self, subj: u32) -> bool {
        subj == super::guard::PLAYER_HANDLE && self.player_posing
    }
    /// P20: embedded JT-layout component calls of the controlled Bentley / Murray (player::p20::PoHost)
    /// P23: slot 78 (+0x27c) calls -> player::p23::apply_switches
    fn vcall_arg(&mut self, obj: u32, off: u32, a: i32) {
        if off == 0x27c {
            self.p23_switch.slot78.push((obj, a));
        }
    }
    /// P23: FUN_002206c8 (carry machine SetState) -> player::p23::apply_switches
    fn po_part_s(&mut self, addr: u32, obj: usize, off: u32, s: i32) {
        if addr == 0x002206c8 {
            self.p23_switch.carry.push((obj, off, s));
        }
    }
    fn po_call(&mut self, addr: u32, obj: usize, off: u32, args: &[u32]) -> i64 {
        if addr == 0x002206c8 {
            self.p23_switch.carry.push((obj, off, args.first().copied().unwrap_or(0) as i32));
        }
        // K27: FUN_00130d30(f1, f2, asega, mode) on a class-run ASEGA (classes/k27.rs)
        if addr == 0x00130d30 && args.len() >= 4 && super::k25::AnimRt::is_asega(args[2]) {
            if let Some(w) = self.world.clone() {
                self.k25.ee_sub_130d30(&w, f32::from_bits(args[0]), f32::from_bits(args[1]), args[2], args[3] as i32 as i64);
            }
            return 0;
        }
        if addr == 0x0020bb58 && self.po.as_ref().is_some_and(|p| p.obj == obj) {
            self.po_bb58.push((obj, off, f32::from_bits(args.first().copied().unwrap_or(0))));
        }
        self.po.as_ref().map_or(0, |p| p.po_call(addr, obj, off, args))
    }
    /// K18: LO +0x28 bit 60 (in the world): every loaded world object is (no run-time removal yet)
    fn lo_28_bit60(&mut self, obj: u32) -> bool {
        self.world.as_ref().is_some_and(|w| (obj as usize).checked_sub(1).is_some_and(|i| i < w.objects.len()))
    }
    fn ee_sub_20ff70(&mut self, _obj: usize, entries: &[[u32; 3]]) {
        let Some(w) = self.world.clone() else { return };
        for e in entries {
            // FUN_0020fff0(.., e, obj, flags 1, 0, 0): FUN_0015e650 oid -> is-a 0x30 -> state e[2] (-1 = none)
            let Some(&k) = w.find_oid(e[0] as i32).first() else { continue };
            if !crate::engine::world::class::ee_sub_15e988(w.objects[k].class, 0x30) || e[2] as i32 == -1 {
                continue;
            }
            self.sm_states.push((k, e[1], e[2] as i32));
        }
    }
    fn k22_sma_20fff0(&mut self, _ctx: u32, entry: &[u32; 4], _mode: u32, _owner: u32) -> u32 {
        // Same as ee_sub_20ff70 but for a single [name, key, val, -1] entry: resolve the SMA
        // object and queue a state change. Returns 0 (no live handle in the native recomp).
        let Some(w) = self.world.clone() else { return 0 };
        let Some(&k) = w.find_oid(entry[0] as i32).first() else { return 0 };
        if !crate::engine::world::class::ee_sub_15e988(w.objects[k].class, 0x30) || entry[2] as i32 == -1 {
            return 0;
        }
        self.sm_states.push((k, entry[1], entry[2] as i32));
        0
    }
    fn k22_release_1e5410(&mut self, _owner: u32) {
        // In the native recomp the SMA handle is not a live QMem resource; release is a no-op.
    }
    fn ee_sub_161f48_list(&mut self, _obj: usize, entries: &[[u32; 2]]) {
        let Some(w) = self.world.clone() else { return };
        for e in entries {
            if let Some(&k) = w.find_oid(e[0] as i32).first() {
                self.signals.push((k, e[1]));
                // K28: FUN_00164640(task, e[1], ctx)
                self.k28_task_calls.push((0x164640, k as u32 + 1, e[1] as i32, _obj as u32 + 1));
            }
        }
    }
    /// WP-SCR10: FUN_00161f48(0x2deb30, +0x164, +0x168, obj) for warp volumes.
    /// Same as ee_sub_161f48_list but takes i32 pairs from C2dc6f0::task_entries. // 0x002283d0
    fn ee_sub_161f48_warp(&mut self, obj: usize, entries: &[[i32; 2]]) {
        let Some(w) = self.world.clone() else { return };
        let ctx = obj as u32 + 1;
        for e in entries {
            if let Some(&k) = w.find_oid(e[0]).first() {
                self.signals.push((k, e[1] as u32));
                // K28: FUN_00164640(task, e[1], ctx)
                self.k28_task_calls.push((0x164640, k as u32 + 1, e[1], ctx));
            }
        }
    }
    /// WP-SCR10: warp-volume task entries.
    fn warp_task_entries(&self, obj: usize) -> Vec<[i32; 2]> {
        self.warp_tasks.get(&obj).cloned().unwrap_or_default()
    }
    /// K28: FUN_00164640 / FUN_00164df0 / FUN_001619a0 -> the job manager (`k28_task_calls`)
    fn ee_sub_164640(&mut self, obj: u32, k: i32) {
        self.k28_task_calls.push((0x164640, obj, k, 0));
    }
    fn ee_sub_164640_src(&mut self, obj: u32, k: i32, src: u32) {
        self.k28_task_calls.push((0x164640, obj, k, src));
    }
    fn ee_sub_164df0(&mut self, obj: u32, k: i32) {
        self.k28_task_calls.push((0x164df0, obj, k, 0));
    }
    fn ee_sub_1619a0(&mut self, k: i32, obj: u32) {
        self.k28_task_calls.push((0x1619a0, obj, k, 0));
    }
    fn ee_sub_1dffd8(&mut self, seq: u32) -> bool {
        // 0x001dffd8: return *(seq + 0x74) — sequence running field
        self.obj_i32(seq, 0x74) != 0
    }
    fn ee_sub_1dfd58(&mut self, seq: u32, player: u32) {
        self.seq_starts.push((seq, player));
    }
    fn ee_sub_1b7000(&mut self, user: u32, point: u32) {
        self.unported.push(format!("Unported: FUN_001b7000(bentley {user:#x}, point {point:#x}) - Bentley is not a ported class"));
    }
    fn ee_sub_15e938(&mut self, kind: i32, oid: i32, ctx: u32) -> u32 {
        // FUN_0015e938 = FUN_0015e958 (handle = index + 1). K16: modes 1-4 with a context object run the 1:1
        // relative search (game::methods::find; mode 4 = nearest relative, e.g. a class-76 swarm's own rails
        // when several swarms reuse the rail oids); REPLACED: nothing found there (port owner tree differs:
        // nested loads are siblings), kind 0 / high modes (0x1e, 0x30, ...) and mode 5 -> first object by oid.
        let Some(w) = &self.world else { return 0 };
        if (1..=4).contains(&(kind & 0xff)) && ctx != 0 {
            let r = crate::engine::game::methods::find::ee_sub_15e958(w, kind as u32, oid, ctx - 1);
            if r != 0 {
                self.found += 1;
                return r + 1;
            }
        }
        match w.find_oid(oid).first() {
            Some(&i) => {
                self.found += 1;
                i as u32 + 1
            }
            None => 0,
        }
    }
    fn vcall(&mut self, obj: u32, off: u32) {
        // K25: reference counts of the run-time ASEGA / SMA handles
        if !self.k25.vcall(obj, off) {
            self.vcalls += 1;
        }
    }
    // ---- K25 anim (classes/k25.rs)
    fn k25_tick(&mut self, dt: f32) {
        if let Some(w) = self.world.clone() {
            self.k25.tick(&w, dt);
            // K26: FUN_00131290 for the events reached in the tick (classes/k26.rs)
            self.k26_dispatch(0.0, dt);
            self.k26_bind();
        }
    }
    fn k26_take_msgs(&mut self) -> Vec<(u32, i32, crate::engine::anim::sm::SmaMsg)> {
        self.k26_msgs()
    }
    fn k26_handled(&mut self, n: usize) {
        self.k26.handled += n;
    }
    /// K27: FUN_001f7bf8(fade, obj) fade-out: REPLACED by alpha +0x12c = 0 at once
    fn ee_sub_1f7bf8(&mut self, obj: usize, _rate: f32) {
        self.k27_fades += 1;
        self.obj_set_u32(obj as u32 + 1, 0x12c, 0);
    }
    fn k27_script_msg(&mut self, obj: u32, msg: u32, args: &[u32]) {
        if self.k27_script_msgs.len() < 256 {
            self.k27_script_msgs.push((obj, msg, args.to_vec()));
        }
    }
    /// WP-EP1f: (vtable +0x1c)(obj, msg, ..) = FUN_001a61e0 (slot 2 of the LO-derived classes, C2c94e0): the
    /// object's level-script message (C2c94a0_v2 -> FUN_001331f8, posted) and, for messages 5 / 6, the job
    /// manager event 0x1a / 0x1b with the object (FUN_001619a0). REPLACED: classes overriding slot 2 are not
    /// distinguished; only messages 5 / 6 are routed (the other vt+0x1c messages of the class code stay
    /// unsent as before).
    fn vcall_1c(&mut self, obj: usize, a: i32, _b: i32) {
        // 0x001a61e0
        let ev = match a {
            5 => 0x1a,
            6 => 0x1b,
            _ => return,
        };
        self.k27_script_msg(obj as u32 + 1, a as u32, &[]);
        self.ee_sub_1619a0(ev, obj as u32 + 1);
    }
    /// WP-EP1f: FUN_00228170(warp): FUN_001c4840 player != 0 -> (player vt+0x134)(player, warp) (slot 37
    /// C2cd748_v37 = place the player at the warp; REPLACED: not modelled, the job's reload follows) and
    /// (warp vt+0x1c)(warp, 5, 0, 0, 0, 0) -> event 0x1a.
    fn ee_sub_228170(&mut self, warp: u32) {
        // 0x00228170
        if let Some(o) = (warp as usize).checked_sub(1) {
            self.vcall_1c(o, 5, 0);
        }
    }
    fn k27_shake_14f750(&mut self, kind: i32) {
        self.k27_shake = Some(kind);
    }
    fn ee_sub_14e328(&mut self, k: i32, desc: u32, target: u32, obj: u32) {
        self.k29_req(super::k29_cam::CamReq::Set { cpp: k, desc, pso: target, pv: obj });
    }
    fn ee_sub_14e4c8_k(&mut self, k: i32, a: i32, desc: u32, b: i32, obj: u32) {
        self.k29_req(super::k29_cam::CamReq::Revoke { grf: k as u32, cpp: a, desc, pso: b, pv: obj, if_active: false });
    }
    fn ee_sub_14e4c8(&mut self, desc: u32) {
        self.k29_req(super::k29_cam::CamReq::Revoke { grf: 2, cpp: -1, desc, pso: 0, pv: 0, if_active: false });
    }
    fn k30_cam_state(&mut self, key: u32, st: super::k30_cam::CamState) {
        self.k30_cam.insert(key, st);
    }
    fn k30_zoom(&mut self, op: super::k30_cam::ZoomOp) {
        self.k29_req(super::k29_cam::CamReq::Zoom(op));
    }
    fn k30_zoom_wait(&mut self, kind: u32) -> bool {
        let i = match kind {
            0x3d => 0,
            0x3e => 1,
            _ => 2,
        };
        self.k30_zoom_waits.map_or(true, |w| w[i])
    }
    fn k29_cm_move(&mut self, pv: u32, mv: super::k29_cam::CmMove) {
        self.k29_req(super::k29_cam::CamReq::Move { pv, mv });
    }
    fn k29_cm_move_cut(&mut self, pv: u32, cut: u32) {
        self.k29_req(super::k29_cam::CamReq::MoveCut { pv, cut });
    }
    fn k29_cm_revoke_if_active(&mut self, grf: u32, cpp: i32, desc: u32, pso: i32, pv: u32) {
        self.k29_req(super::k29_cam::CamReq::Revoke { grf, cpp, desc, pso, pv, if_active: true });
    }
    fn k27_rumble_187e28(&mut self, duration: f32, id: i32) {
        self.k26.rumble.push(crate::engine::game::splice_host::RumbleRequest { pattern: id as usize, duration });
    }
    /// FUN_0020efc0: world +0x3d78 up; the first sets pad lock bit 2 (FUN_00187078); flags bit 0 / 1 raise the
    /// +0x3d80 / +0x3d84 counters. WP-UI4: +0x3d80 == 1 makes the FUN_001a3078 vault HUD object call (FUN_0020f128).
    fn k27_world_20efc0(&mut self, flags: u32) {
        self.k27_cine[0] += 1;
        if self.k27_cine[0] == 1 {
            self.k27_pad_lock |= 2;
        }
        if flags & 1 != 0 {
            self.k27_cine[2] += 1;
            if self.k27_cine[2] == 1 {
                self.ee_sub_1a3078(true);
            }
        }
        if flags & 2 != 0 {
            self.k27_cine[3] += 1;
        }
    }
    /// FUN_0020f030: +0x3d78 down; at 0 the lock bit 2 clears (FUN_001870a0); flags bit 0 / 1 lower +0x3d80 / +0x3d84.
    /// WP-UI4: +0x3d80 == 0 makes the FUN_001a30d0 vault HUD hide call (FUN_0020f160).
    fn k27_world_20f030(&mut self, flags: u32) {
        self.k27_cine[0] -= 1;
        if self.k27_cine[0] == 0 {
            self.k27_pad_lock &= !2;
        }
        if flags & 1 != 0 {
            self.k27_cine[2] -= 1;
            if self.k27_cine[2] == 0 {
                self.ee_sub_1a3078(false);
            }
        }
        if flags & 2 != 0 {
            self.k27_cine[3] -= 1;
        }
    }
    /// FUN_0020f0c0: +0x3d7c up; the first sets pad lock bit 4 (FUN_00187078)
    fn k27_world_20f0c0(&mut self) {
        self.k27_cine[1] += 1;
        if self.k27_cine[1] == 1 {
            self.k27_pad_lock |= 4;
        }
    }
    /// FUN_0020f0f0: +0x3d7c down; at 0 the lock bit 4 clears (FUN_001870a0)
    fn k27_world_20f0f0(&mut self) {
        self.k27_cine[1] -= 1;
        if self.k27_cine[1] == 0 {
            self.k27_pad_lock &= !4;
        }
    }
    fn k27_slot4_base(&mut self, obj: usize) -> bool {
        let Some(w) = &self.world else { return false };
        let Some(o) = w.objects.get(obj) else { return false };
        let vt = crate::engine::world::loads_b4::vtable_load(o.class).map(|v| v.0).or_else(|| crate::engine::world::class::class(o.class).map(|c| c.vtable));
        vt.is_some_and(super::k27::slot4_is_base)
    }
    fn emitter_attach(&mut self, obj: usize, e: [i32; 4], _g_3dd268: i32) {
        // K26: FUN_001fe990(0x3dd040, e[0]) + FUN_001fc408(e[3], DAT_003dd268, that, obj, &e[1]) (classes/k26.rs)
        let l = self.k26_music.ee_sub_1fe990(e[0]);
        self.k26_music.ee_sub_1fc408(e[3], l, obj as u32 + 1, [e[1], e[2]]);
    }
    fn ee_sub_1fe5f0(&mut self, id: i32, k: i32) -> u32 {
        self.k26_music.ee_sub_1fe5f0(id, k)
    }
    fn ee_sub_1fe738(&mut self) {
        self.k26_music.ee_sub_1fe738()
    }
    fn ee_sub_12f9b8(&mut self, a: u32, rate: f32, anim: u32, obj: u32, flags: u32) -> u32 {
        let Some(w) = self.world.clone() else { return 0 };
        self.k25.ee_sub_12f9b8(&w, f32::from_bits(a), rate, anim, obj, flags)
    }
    fn ee_sub_131968(&mut self, a: u32) -> bool {
        self.k25.ee_sub_131968(a)
    }
    /// FUN_001306a8(&slot): vtable +0x14 of the slot's handle (the caller clears the slot)
    fn ee_sub_1306a8(&mut self, a: u32) {
        self.k25.vcall(a, 0x14);
    }
    fn ee_sub_130b18(&mut self, handle: u32) {
        self.k25.ee_sub_130b18(handle);
    }
    /// ASEG +0x74 (length)
    fn anim_74(&mut self, anim: u32) -> f32 {
        let Some(w) = self.world.clone() else { return 0.0 };
        super::k25::AnimRt::aseg(&w, anim).map_or(0.0, |a| a.length)
    }
    /// ASEG +0x78 (rate, FUN_0012f248 default)
    fn k22_anim_f(&mut self, anim: u32, off: u32) -> f32 {
        let Some(w) = self.world.clone() else { return 0.0 };
        match (super::k25::AnimRt::aseg(&w, anim), off) {
            (Some(a), 0x74) => a.length,
            (Some(_), 0x78) => super::k25::ASEG_78,
            _ => 0.0,
        }
    }
    fn anim_slot_f(&mut self, slot: u32, off: u32) -> f32 {
        self.k25.slot_f(slot, off)
    }
    fn ee_sub_131a00(&mut self, rate: f32, slot: u32) {
        self.k25.ee_sub_131a00(rate, slot);
    }
    fn root_vel_204e38(&mut self, slot: u32) -> [f32; 3] {
        let Some(w) = self.world.clone() else { return [0.0; 3] };
        self.k25.ee_sub_204e38(&w, slot)
    }
    /// FUN_001f0380(sm, obj, label, flags): FUN_001f01b8 with mask 4
    fn ee_sub_1f0380_link(&mut self, sm: u32, obj: u32, a: i32, b: u32) -> u32 {
        let Some(w) = self.world.clone() else { return 0 };
        self.k25.ee_sub_1f01b8(&w, sm, obj, a, b, super::k25::SMA_MASK_1F0380)
    }
    fn ee_sub_1f03a8_link(&mut self, sm: u32, obj: u32, label: i32, flags: u32, mask: i32) -> u32 {
        let Some(w) = self.world.clone() else { return 0 };
        self.k25.ee_sub_1f01b8(&w, sm, obj, label, flags, mask as u32)
    }
    fn ee_sub_1f0400(&mut self, sm: u32, label: i32) -> i32 {
        let Some(w) = self.world.clone() else { return -1 };
        self.k25.ee_sub_1f0400(&w, sm, label)
    }
    fn ee_sub_1f0638(&mut self, sm: u32) -> bool {
        self.k25.ee_sub_1f0638(sm)
    }
    /// FUN_001f0660(sma, label) on a run-time SMA handle (other callers pass object indices: no-op as before)
    fn ee_sub_1f0660(&mut self, obj: usize, anim: i32) {
        let Some(w) = self.world.clone() else { return };
        if super::k25::AnimRt::is_sma(obj as u32) {
            self.k25.ee_sub_1f0660(&w, obj as u32, anim);
        }
    }
    fn ee_sub_1f09d8(&mut self, sm: u32, label: i32) {
        let Some(w) = self.world.clone() else { return };
        self.k25.ee_sub_1f09d8(&w, sm, label);
    }
    fn ee_sub_1f1350(&mut self, sm: u32, mask: i32) {
        let Some(w) = self.world.clone() else { return };
        self.k25.ee_sub_1f1350(&w, sm, mask as u32);
    }
    /// FUN_001f1448(sm, obj, -1, flags, &slot) (the caller released the old slot through FUN_001f15b8): new SMA
    /// + vtable +0xc
    fn ee_sub_1f1448(&mut self, snd: u32, obj: u32, flags: u32) -> u32 {
        let Some(w) = self.world.clone() else { return 0 };
        let s = self.k25.ee_sub_1f01b8(&w, snd, obj, -1, flags, super::k25::SMA_MASK_1F0380);
        self.k25.vcall(s, 0xc);
        s
    }
    fn ee_sub_1f15b8(&mut self, slot: &mut u32) {
        self.k25.ee_sub_1f15b8(slot);
    }
    /// FUN_001f07e8() == FUN_001f07c8(sma): the SMA reached its goal state
    fn snd_done_1f07c8(&mut self, handle: u32) -> bool {
        self.k25.ee_sub_1f07c8_done(handle)
    }
    // ---- K25 sound (classes/k25.rs SoundRt)
    fn ee_sub_1fbb90(&mut self, obj: usize, id: i32) {
        self.k25_snd.ee_sub_1fbb90(obj, id);
    }
    fn emitter_create(&mut self, obj: usize, k: usize, e: [i32; 5]) -> i32 {
        self.k25_snd.emitter_create(obj, k, e)
    }
    fn emitter_release(&mut self, obj: usize, slot: usize) {
        self.k25_snd.emitter_release(obj, slot);
    }
    fn ee_sub_149a50(&mut self, _obj: usize, _pmf: u32, _arg: i32) {
        self.deferred += 1;
    }
    fn ee_sub_1394c8(&mut self, _r: i32, _t: &super::c2ce4d0::Template) {
        self.tpl_pushes += 1;
    }
    fn ee_sub_1cde70(&mut self, _r: i32, _t: &super::c2ce4d0::Template) {
        self.tpl_pushes += 1;
    }
    fn ee_sub_15bd20(&mut self, obj: usize, count: i32, t: &[u32; 64], blk: &[u32; 5], link: bool) {
        self.emits += 1;
        self.particles += count as i64;
        // K28: FUN_0015ad48 frame of the block's source object (FUN_001a6318)
        let frame = self.world.as_ref().and_then(|w| {
            let i = (blk[0] as usize).checked_sub(1).filter(|&i| i < w.objects.len())?;
            Some(crate::engine::game::fields::ee_sub_1a6318(w, i))
        });
        self.k28_emits.push(super::k28_emit::EmitReq { obj, count, t: *t, blk: *blk, link, frame });
    }
    fn k30_take_reqs(&mut self) -> Vec<(u32, [u32; 15])> {
        std::mem::take(&mut self.k30_reqs)
    }
    fn k28_take_emits(&mut self) -> Vec<super::k28_emit::EmitReq> {
        let v = std::mem::take(&mut self.k28_emits);
        // J17: the requests ClassSet::k28_spawn hands to FUN_0015b428 (count >= 1, template +4 != 1)
        self.k28_looks.extend(v.iter().filter(|r| r.count >= 1 && r.t[1] != 1).map(|r| (r.obj, r.t, r.link)));
        v
    }
    fn sound_15c690(&mut self, src: u32, pos: [f32; 4], sd: [u32; 5]) {
        self.sounds += 1;
        self.k25_snd.sound_15c690(src, pos, sd);
    }
    // ---- K25 object fields (classes/k25.rs FieldRt)
    fn obj_i32(&mut self, obj: u32, off: u32) -> i32 {
        let Some(w) = self.world.clone() else { return 0 };
        self.k25_fld.set_world(&w);
        self.k25_fld.get(obj, off).unwrap_or_else(|| super::k25::FieldRt::base(&w, &self.k25, obj, off)) as i32
    }
    fn obj_f32(&mut self, obj: u32, off: u32) -> f32 {
        f32::from_bits(self.obj_i32(obj, off) as u32)
    }
    fn obj_set_u32(&mut self, obj: u32, off: u32, v: u32) {
        let Some(w) = self.world.clone() else { return };
        self.k25_fld.set_world(&w);
        self.k25_fld.set(obj, off, v);
    }
    fn obj_or_40(&mut self, obj: u32, bits: u32) {
        let v = self.obj_i32(obj, 0x40) as u32 | bits;
        self.obj_set_u32(obj, 0x40, v);
    }
    fn obj_and_40(&mut self, obj: u32, mask: u32) {
        let v = self.obj_i32(obj, 0x40) as u32 & mask;
        self.obj_set_u32(obj, 0x40, v);
    }
    fn obj_flags_28(&mut self, obj: u32) -> u64 {
        let Some(w) = self.world.clone() else { return 0 };
        self.k25_fld.set_world(&w);
        let lo = self.k25_fld.get(obj, 0x28);
        let hi = self.k25_fld.get(obj, 0x2c);
        u64::from(lo.unwrap_or(0)) | u64::from(hi.unwrap_or(0)) << 32
    }
    fn ee_sub_1f4c58(&mut self, alo: u32, oid: i32, k: i32) -> u32 {
        self.world.as_ref().map_or(0, |w| super::k25::ee_sub_1f4c58(w, alo, oid, k))
    }
    // ---- K25 physics (classes/k25.rs PhysRt)
    fn k25_bodies(&mut self, b: Vec<(usize, super::k25::BodySnap)>) {
        if let Some(w) = self.world.clone() {
            self.k25_phys.set_bodies(&w, b);
        }
    }
    fn vcall_10c(&mut self, obj: usize) -> [f32; 3] {
        if let Some(v) = self.k27_vel(obj) {
            return v;
        }
        self.k25_phys.vcall_10c(obj)
    }
    // ---- K27 character PO (classes/k27.rs)
    fn ee_sub_1f9b20(&mut self, obj: usize) {
        self.k27_1f9b20(obj)
    }
    fn ee_sub_204108(&mut self, obj: usize) {
        self.k27_204108(obj)
    }
    fn ee_sub_1fac30(&mut self, obj: usize, a: [f32; 3]) {
        self.k27_1fac30(obj, a);
    }
    fn k27_po_step(&mut self, obj: usize, m: crate::engine::physics::motor::PoMotor, dt: f32) {
        self.k27_step(obj, m, dt)
    }
    fn ee_sub_22d348(&mut self, alo: u32, p: [f32; 3]) -> [f32; 3] {
        (alo as usize).checked_sub(1).map_or([0.0; 3], |i| self.k25_phys.ee_sub_22d348(i, p))
    }
    fn phys_e8(&mut self, obj: usize) -> Option<super::actor::alo::PhysE8> {
        // REPLACED: +0xdc / +0xa0 of a class-owned body (no angular constraint kind in `AloPhys`: free)
        self.k25_phys.body(obj).map(|b| super::actor::alo::PhysE8 { dc: false, s_a0: 0, s_c0: b.ckind as i32 })
    }
    fn phys_e8_80(&mut self, obj: usize) -> [f32; 3] {
        self.k25_phys.body(obj).map_or([0.0; 3], |b| b.gravity)
    }
    fn ee_sub_1f5860(&mut self, obj: u32) -> [f32; 3] {
        (obj as usize).checked_sub(1).and_then(|i| self.k25_phys.body(i)).map_or([0.0; 3], |b| b.gravity)
    }
    fn ee_sub_12dc18(&mut self, obj: u32, xf: u32, v: [f32; 3]) -> u32 {
        // FUN_0012dc18(obj, xf, 0, 7, 2, v)
        self.k25_con(obj, xf, 0, 7, 2, v)
    }
    fn k22_pin_12dc18(&mut self, obj: u32, v: [f32; 3]) -> u32 {
        // FUN_0012dc18(obj, +0x5c, 1, 5, 3, v)
        self.k25_con(obj, 0x5c, 1, 5, 3, v)
    }
    fn ee_sub_12e460(&mut self, slot: &mut u32) {
        let Some(mut xf) = self.k25_phys.xf_of(*slot) else {
            *slot = 0;
            return;
        };
        let mut pool = std::mem::take(&mut self.k25_phys.pool);
        let rec = *slot;
        super::k25::ee_sub_12e460(&mut pool, &mut xf, slot, self);
        self.k25_phys.pool = pool;
        self.k25_phys.recs.remove(&rec);
        self.k25_phys.put_xf(xf);
    }
    fn k22_pin_12d108(&mut self, handle: u32, v: [f32; 3]) {
        let Some(mut xf) = self.k25_phys.xf_of(handle) else { return };
        if let Some(t) = super::k25::ee_sub_12d108(&mut xf, handle, v) {
            self.k25_phys.targets.insert(xf.xf, t);
        }
        self.k25_phys.put_xf(xf);
    }
    fn k23_guard_fc4(&mut self, guard: u32) -> u32 {
        // REPLACED: the guard's +0xfc4 scale record (created by the unported guard init) is made on first
        // query on a REPLACED xf (guard, +0xfc4) with the unit scale
        if let Some(&r) = self.k25_phys.fc4.get(&guard) {
            return r;
        }
        if self.world.is_none() || guard == 0 {
            return 0;
        }
        let r = self.k25_con(guard, 0xfc4, 0, 0, 0, [1.0; 3]);
        self.k25_phys.fc4.insert(guard, r);
        r
    }
    fn ee_call(&mut self, addr: u32, args: &[u32]) -> i64 {
        match (addr, args) {
            // FUN_0012dc18(obj, xf, prio, f_14, f_1c) with the vector argument lost in the decompile (0)
            (0x12dc18, &[obj, xf, prio, f14, f1c, ..]) => self.k25_con(obj, xf, prio as i32, f14, f1c, [0.0; 3]) as i64,
            // K27: FUN_001f7c68(fade, obj) fade-in: REPLACED by alpha +0x12c = 1 at once (classes/k27.rs)
            // WP-EP1d: FUN_001a58d0(obj, parent, flags) SetParent / FUN_001f6bc8(obj, id, v) part +0x18
            (0x1a58d0, &[o, p, f, ..]) => {
                if let Some(i) = (o as usize).checked_sub(1) {
                    self.ee_sub_1a58d0_set_parent(i, p, f);
                }
                0
            }
            (0x1f6bc8, &[o, id, v, ..]) => {
                if let Some(i) = (o as usize).checked_sub(1) {
                    self.ee_sub_1f6bc8(i, id as i32, v);
                }
                0
            }
            (0x1f7c68, &[_, o, ..]) => {
                self.k27_fades += 1;
                self.obj_set_u32(o, 0x12c, 1f32.to_bits());
                0
            }
            // PAD1: FUN_00187d80(pad_addr, mask) — test-and-consume pressed buttons
            (0x187d80, &[_, mask, ..]) => {
                if let Some(pad) = self.pad.as_mut() {
                    crate::engine::input::pad::FUN_00187d80(pad, mask) as i64
                } else { 0 }
            }
            // PAD1: FUN_00187de0(pad_addr, mask) — consume pressed, latch held
            (0x187de0, &[_, mask, ..]) => {
                if let Some(pad) = self.pad.as_mut() {
                    crate::engine::input::pad::FUN_00187de0(pad, mask);
                }
                0
            }
            _ => 0,
        }
    }
    fn ee_sub_168638(&mut self, _obj: usize, _f: f32, _a: i32) {
        self.draws += 1;
    }
    // ---- K5: hooks whose callee is ported elsewhere / answerable from the world
    fn ee_sub_132b10(&mut self, obj: u32, class: i32) -> u32 {
        // FUN_00132b10 (player::p4): class of obj derives from `class`; PLAYER_HANDLE = Sly, the class-53 JT
        if obj == super::guard::PLAYER_HANDLE {
            return crate::engine::player::p4::FUN_00132b10(53, class) as u32;
        }
        let Some(w) = &self.world else { return 0 };
        let Some(o) = (obj as usize).checked_sub(1).and_then(|i| w.objects.get(i)) else { return 0 };
        crate::engine::player::p4::FUN_00132b10(o.class as i32, class) as u32
    }
    fn ee_sub_1a6290(&mut self, obj: u32, oid: i32) -> bool {
        // FUN_001a6290: +0x18 oid match (the +0x3c attachment chain is not in the world model)
        let Some(w) = &self.world else { return false };
        (obj as usize).checked_sub(1).and_then(|i| w.objects.get(i)).is_some_and(|o| o.oid == oid)
    }
    // ---- K6
    /// K32: guard anim length (the table anim's ASEG, guard::los::anim_length)
    fn anim_len(&mut self, guard: usize, anim: i32) -> f32 {
        let Some(w) = &self.world else { return 0.0 };
        super::guard::los::anim_length(w, guard, anim)
    }
    fn ee_sub_209618(&mut self, guard: usize, anim: i32, t: f32, rate: f32) -> [f32; 3] {
        // FUN_00209618 -> FUN_00204e38: the anim's root channel velocity; +0x190 = -v (returned as v)
        let Some(w) = &self.world else { return [0.0; 3] };
        let v = super::guard::los::root_velocity(w, guard, anim, t, rate);
        // K25: the +0xfc4 record's scale (1 until a class-102 bomb drives it, FUN_0012d108)
        let k = self.k25_phys.fc4_scale(guard as u32 + 1);
        [v[0] * k, v[1] * k, v[2] * k]
    }
    // ---- K8
    fn ee_sub_20faa0_1c5570(&mut self, lo: [f32; 3], hi: [f32; 3]) -> Vec<super::k8::K8Part> {
        // FUN_0020faa0 box query over the SO collision parts, filter FUN_001c5570 (owner not is-a 0x32 / 0x4f;
        // the +0x10 flag bits 48 / 49 are not in the world model: all parts are geometry parts)
        let Some(_) = self.k7_cache() else { return Vec::new() };
        let Some(so) = self.so.clone() else { return Vec::new() };
        let mut out = Vec::new();
        for (k, c) in so.colliders.iter().enumerate() {
            let hit = !c.geom.verts.is_empty()
                && (0..3).all(|a| {
                    let (mn, mx) = c.geom.verts.iter().fold((f32::MAX, f32::MIN), |(m, x), v| (m.min(v[a]), x.max(v[a])));
                    mn <= hi[a] && lo[a] <= mx
                });
            if !hit {
                continue;
            }
            let owner = c.object as u32 + 1;
            if self.ee_sub_132b10(owner, 0x32) != 0 || self.ee_sub_132b10(owner, 0x4f) != 0 {
                continue;
            }
            out.push(super::k8::K8Part { id: k as u32 + 1, f_74: 1, ..Default::default() });
        }
        out
    }
    fn alo_parts_e4(&mut self, alo: u32) -> Vec<super::k8::K8Part> {
        let Some(_) = self.k7_cache() else { return Vec::new() };
        let Some(so) = self.so.clone() else { return Vec::new() };
        so.colliders
            .iter()
            .enumerate()
            .filter(|(_, c)| c.object as u32 + 1 == alo)
            .map(|(k, _)| super::k8::K8Part { id: k as u32 + 1, f_74: 1, ..Default::default() })
            .collect()
    }
    fn ee_sub_1a93c8(&mut self, radius: f32, pos: [f32; 3], part: u32, max: usize) -> Vec<[[f32; 4]; 2]> {
        let Some(so) = self.so.clone() else { return Vec::new() };
        let Some(c) = (part as usize).checked_sub(1).and_then(|k| so.colliders.get(k)) else { return Vec::new() };
        let mut out = Vec::new();
        crate::engine::physics::sphere::FUN_001a93c8(radius, pos, &c.geom, &c.bsp, max, &mut out);
        out.iter().map(|s| [[s.pos[0], s.pos[1], s.pos[2], 0.0], s.plane]).collect()
    }
    fn rec_15c(&mut self, player: u32) -> super::k8::PickRec {
        match self.player_rec {
            Some(r) if player == super::guard::PLAYER_HANDLE => super::k8::PickRec { obj: player, ..r },
            _ => Default::default(),
        }
    }
    fn ee_sub_1c4840(&mut self) -> u32 {
        // K13: FUN_001c4840(0x3dd490): the current player (Sly, PlayerFocus) or 0 while there is none
        if self.player_valid { super::guard::PLAYER_HANDLE } else { 0 }
    }
    fn k22_path(&mut self, obj: u32) -> Option<Arc<crate::engine::player::p5_target::Path>> {
        // K22: the class-10 path of the object (same table as `path_len_1e6648`)
        self.path10(obj)
    }
    fn k22_targets_3eb8(&mut self) -> Vec<u32> {
        // K22: the world +0x3ebc list = the player character (when valid)
        if self.player_valid { vec![super::guard::PLAYER_HANDLE] } else { Vec::new() }
    }
    fn ee_sub_1a62e8(&mut self, obj: u32) -> [f32; 3] {
        if obj == super::guard::PLAYER_HANDLE {
            return self.player_pos;
        }
        if let Some(p) = (obj as usize).checked_sub(1).and_then(|i| self.k27_pos(i)) {
            return p;
        }
        self.k24_world_xf(obj).map_or([0.0; 3], |m| m[3])
    }
    /// K24: FUN_001a6318(obj): the rotation rows of the same world matrix as FUN_001a62e8 (identity = none)
    fn ee_sub_1a6318(&mut self, obj: u32) -> [[f32; 4]; 3] {
        let m = self.k24_world_xf(obj).unwrap_or(crate::engine::gfx::scene::IDENTITY);
        [0, 1, 2].map(|r| [m[r][0], m[r][1], m[r][2], 0.0])
    }
    /// K24: (vtable +0xfc)(obj): the world matrix (row 3 = FUN_001a62e8)
    fn vcall_fc(&mut self, obj: u32) -> crate::engine::anim::pose::Mat4 {
        let Some(m) = self.k24_world_xf(obj) else { return crate::engine::anim::pose::IDENTITY };
        [0, 1, 2, 3].map(|r| [m[r][0], m[r][1], m[r][2], (r == 3) as u32 as f32])
    }
    /// K24: FUN_00227bb8 +0x110 of a class 22-27 volume: the bounding-sphere centre of its (object-local)
    /// solid geometry (physics::geom `bounding_sphere_227bb8`, as the collision world builds it)
    fn vol_centre_110(&mut self, obj: usize) -> [f32; 3] {
        use crate::engine::physics::geom::{bounding_sphere_227bb8, build_geom};
        use crate::engine::world::objects::{LoadData, Payload};
        let Some(w) = self.world.as_ref() else { return [0.0; 3] };
        match w.objects.get(obj).map(|o| &o.payload) {
            Some(Payload::Loaded(LoadData::Solid(s) | LoadData::L2282b0 { solid: s, .. })) => bounding_sphere_227bb8(&build_geom(&s.geom)).0,
            _ => [0.0; 3],
        }
    }
    /// K24: FUN_001a6f00(obj, 0, p): obj-local point -> world, with the FUN_001a62e8 world matrix
    fn ee_sub_1a6f00_to_world(&mut self, obj: usize, p: [f32; 3]) -> [f32; 3] {
        let Some(m) = self.k24_world_xf(obj as u32 + 1) else { return p };
        [0, 1, 2].map(|k| p[0] * m[0][k] + p[1] * m[1][k] + p[2] * m[2][k] + m[3][k])
    }
    /// WP-PC2: FUN_001a6f00(a, b, 0): origin of a's local frame expressed in b's local frame.
    /// Composition: a != 0 lifts p = (0,0,0) through world_xf(a) (which equals the translation column
    /// of a's world matrix), then b != 0 brings the result into b's local frame via inverse(world_xf(b)).
    /// Either operand may be 0, matching the two branches of the Ghidra body.
    fn ee_sub_1a6f00_of(&mut self, a: u32, b: u32) -> [f32; 3] {
        use crate::engine::gfx::scene::invert;
        let mut p = [0.0f32; 3];
        if a != 0 {
            if let Some(m) = self.k24_world_xf(a) {
                p = [m[3][0], m[3][1], m[3][2]];
            }
        }
        if b != 0 {
            if let Some(mb) = self.k24_world_xf(b) {
                let mi = invert(&mb);
                p = [0, 1, 2].map(|k| p[0] * mi[0][k] + p[1] * mi[1][k] + p[2] * mi[2][k] + mi[3][k]);
            }
        }
        p
    }
    /// K24: obj +0x10 = the owning LO (FUN_0013fb40 param_1; world `Lo::owner`)
    fn parent_10(&mut self, obj: u32) -> u32 {
        let Some(w) = self.world.as_ref() else { return 0 };
        // WP-EP1d: run-time SetParent first
        if self.pos_for == Arc::as_ptr(w) as usize {
            if let Some(&(p, _)) = (obj as usize).checked_sub(1).and_then(|i| self.rt_parent.get(&i)) {
                return p;
            }
        }
        (obj as usize).checked_sub(1).and_then(|i| w.objects.get(i)).and_then(|o| o.owner).map_or(0, |o| o as u32 + 1)
    }
    fn path_len_1e6648(&mut self, path: u32) -> f32 {
        // K16: FUN_001e6648 on the object's +0xf0 table
        self.path10(path).map_or(0.0, |p| p.FUN_001e6648())
    }
    fn path_point_1e6f10(&mut self, path: u32, d: f32) -> [f32; 3] {
        // K16: FUN_001e6f10 (FUN_001e64c8 + FUN_001e67b8, world point)
        self.path10(path).map_or([0.0; 3], |p| p.FUN_001e6f10(d).0)
    }
    fn player_vcall_28c(&mut self, _player: u32, hp: i32, mp: i32) {
        self.player_gifts.push((hp, mp));
    }
    // ---- K7
    fn ee_sub_20f1b8(&mut self) -> bool {
        self.world_3d84 > 0
    }
    fn ee_sub_118668_blockers(&mut self, eye: [f32; 3], tp: [f32; 3]) -> bool {
        self.k7_cache();
        self.blockers.as_ref().is_some_and(|b| b.blocked(eye, tp))
    }
    fn ee_sub_1545f0(&mut self, guard: usize, eye: [f32; 3], tp: [f32; 3], f8: u32) -> bool {
        let Some(w) = self.k7_cache() else { return false };
        let so = self.so.clone().unwrap();
        super::guard::los::ee_sub_1545f0(&w, &so, guard, eye, tp, f8)
    }
    fn sensor_event_1c(&mut self, _guard: usize, _t: u32, _old: i32, new: i32) -> i32 {
        // FUN_001a61e0(sensor, 0x1c, ..): no level-script handler is ported -> the state stands
        self.sensor_events += 1;
        new
    }
    fn contacts(&mut self, guard: usize) -> Vec<super::guard::slots::Contact> {
        let (mine, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut self.k7_contacts).into_iter().partition(|c| c.0 == guard);
        self.k7_contacts = rest;
        mine.into_iter().map(|c| c.1).collect()
    }
    fn ee_sub_1f7598(&mut self, hit: &super::guard::slots::GuardHit) {
        self.player_hits.push(hit.clone());
    }
    fn alo_children(&mut self, obj: u32) -> Vec<u32> {
        let Some(w) = &self.world else { return Vec::new() };
        let mut v: Vec<u32> = (obj as usize).checked_sub(1).and_then(|i| w.objects.get(i)).map(|o| o.children.iter().map(|&c| c as u32 + 1).collect()).unwrap_or_default();
        // WP-EP1d: run-time SetParent moves children between lists
        if self.pos_for == Arc::as_ptr(w) as usize && !self.rt_parent.is_empty() {
            v.retain(|&c| self.rt_parent.get(&(c as usize - 1)).is_none_or(|&(p, _)| p == obj));
            let mut add: Vec<u32> = self.rt_parent.iter().filter(|&(&c, &(p, _))| p == obj && !v.contains(&(c as u32 + 1))).map(|(&c, _)| c as u32 + 1).collect();
            add.sort_unstable();
            v.extend(add);
        }
        v
    }
    // ---- WP-EP1d: SetParent / SO collision parts
    fn ee_sub_1a58d0(&mut self, obj: usize, a: i32, b: i32) {
        self.ee_sub_1a58d0_set_parent(obj, a as u32, b as u32);
    }
    fn ee_sub_1a58d0_to(&mut self, obj: usize, other: u32, k: i32) {
        self.ee_sub_1a58d0_set_parent(obj, other, k as u32);
    }
    fn ee_sub_1f6ac8(&mut self, obj: usize, id: i32) {
        self.so_part_link(obj, id, true);
    }
    fn ee_sub_1f6b48(&mut self, obj: usize, id: i32) {
        self.so_part_link(obj, id, false);
    }
    // ---- K21
    fn k21_world(&mut self) -> Option<Arc<World>> {
        self.world.clone()
    }
    fn k21_cam_pos(&mut self) -> Option<[f32; 3]> {
        self.cam.as_ref().map(|c| [c.mat[3][0], c.mat[3][1], c.mat[3][2]])
    }
    fn k30_cam_rows(&mut self) -> Option<[[f32; 4]; 3]> {
        self.cam.as_ref().map(|c| [c.mat[0], c.mat[1], c.mat[2]])
    }
    fn cam_2ddfa0(&mut self) -> [[f32; 4]; 2] {
        self.cam.as_ref().map_or([[0.0; 4]; 2], |c| [c.mat[0], c.mat[1]])
    }
    fn k21_cam_14f308(&mut self, q: [f32; 3]) -> Option<[f32; 3]> {
        self.cam.as_ref().map(|c| c.ee_sub_14f308(q))
    }
    // REPLACED: FUN_0020faa0 world collision list = the SO collision entries (`CollisionWorld::build_so`)
    fn k21_blocked_14cc58(&mut self, a: [f32; 3], b: [f32; 3]) -> bool {
        self.k7_cache();
        self.so.as_ref().is_some_and(|so| !so.clip_segment(a, b).is_empty())
    }
    fn k21_ground_173610(&mut self, d0: f32, d1: f32, pos: [f32; 3]) -> Option<[f32; 3]> {
        self.k7_cache();
        let so = self.so.clone()?;
        let mut p = pos;
        crate::engine::player::Jt::FUN_00173610(&so, d0, d1, &mut p, true).then_some(p)
    }
}


#[derive(Resource, Default)]
pub struct Classes {
    pub set: ClassSet,
    pub host: EngineHost,
    pub rng: GameRng,
    pub frames: u64,
    built_for: usize,
}

/// Build the set when the world changes, then run one frame.
pub fn tick_classes(level: Res<ClassesLevel>, mut c: ResMut<Classes>, focus: Option<Res<crate::engine::player::plugin::PlayerFocus>>, cm: Option<Res<crate::engine::camera::cm::Cm>>) {
    let c = &mut *c;
    let Some(w) = &level.0 else { return };
    // K6: Sly (DAT_002e1e40) for the guard sensors: PlayerFocus position (far away while there is no player)
    let valid = focus.as_ref().is_some_and(|f| f.valid);
    let sly = focus.filter(|f| f.valid).map(|f| [f.pos[0], f.pos[1], f.pos[2]]).unwrap_or([1.0e9; 3]);
    c.set.g.k6_players = vec![(super::guard::PLAYER_HANDLE, sly)];
    c.host.player_pos = sly;
    c.host.player_valid = valid;
    // K21: g_cm (0x2ddf40) matrix / scales for the class-119 spawn probes
    c.host.cam = cm.map(|m| super::k21_119::CamView { mat: m.mat, x_scale: m.x_scale, y_scale: m.y_scale });
    let id = Arc::as_ptr(w) as usize;
    if c.built_for != id {
        c.host.world = Some(w.clone());
        let players = std::mem::take(&mut c.set.g.k6_players);
        // K26: a new level: the music instance sources start empty (keys from feed_music_block)
        c.host.k26_music.layers = Default::default();
        c.set = ClassSet::build(w, &mut c.host);
        c.set.g.k6_players = players;
        c.set.g.g_3dd268 = c.host.k26_music.inst;
        c.set.post_load(&mut c.host);
        c.built_for = id;
    }
    // clock (J: dt clamped to [1/60, 1/30]; fixed 1/60 here)
    c.set.g.dt = 1.0 / 60.0;
    c.set.g.t += c.set.g.dt;
    let before: std::collections::HashSet<usize> = c.set.k21c.iter().map(|x| x.0).collect();
    c.set.frame(&mut c.host);
    k28_apply_looks(&mut c.set, &mut c.host, &before);
    c.frames += 1;
}

/// J17: the look part of FUN_001394c8 / FUN_00139370 for one emitter template (`gfx::particles::ee_sub_1394c8_look`
/// arguments): template +0xdc colour keys + +0xcc flag = the FUN_00258f40 list of the template's load (B4
/// `colors`, FUN_001ed278 is the identity without the grey mode); +0xd0 shader id = field 526 (class 29) /
/// 581 (classes 32 / 33) else the template word; FUN_001ec8d0: the FUN_001eb678 node of that id in the
/// DAT_003e1590 list (shaders with flags & 0x40, entry order), else the list head. None = empty list.
/// Node: +8 flags (shader +5), +0xc RGBA (shader +8), +0x10 frames (first frame +6 bitmap count).
pub struct K28Look {
    pub keys: Option<Vec<[u8; 4]>>,
    pub flag_cc: u32,
    pub shader_flags: u32,
    pub shader_rgba: [u8; 4],
    pub frames: i32,
    pub life: f32,
}
pub fn k28_look(w: &World, obj: usize, t: &[u32; 64]) -> Option<K28Look> {
    use crate::engine::world::loads_b4::B4Load;
    use crate::engine::world::props::PropValue;
    let b4 = w.objects.get(obj).and_then(|o| match &w.objects.get(o.clone_of.unwrap_or(obj))?.payload {
        super::Payload::Loaded(super::LoadData::B4(b)) => match &**b {
            B4Load::L258598 { props, colors, .. } => Some((props, colors)),
            _ => None,
        },
        _ => None,
    });
    let field = if w.objects.get(obj).is_some_and(|o| o.class == 29) { 526 } else { 581 };
    let id = b4
        .and_then(|(p, _)| p.iter().rev().find(|p| p.field == field))
        .and_then(|p| match &p.value {
            PropValue::Scalar(b) if b.len() >= 4 => Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]])),
            _ => None,
        })
        .unwrap_or(t[0xd0 / 4]);
    let nodes: Vec<_> = w.entries.iter().flat_map(|e| e.sections.eb2f0.iter()).filter(|s| s.head[5] & 0x40 != 0).collect();
    let id_of = |s: &&crate::engine::world::entry::Rec1eb2f0| u32::from_le_bytes([s.head[0], s.head[1], s.head[2], s.head[3]]);
    let s = (id != u32::MAX).then(|| nodes.iter().find(|s| id_of(s) == id)).flatten().or(nodes.first())?;
    let keys = b4.and_then(|(_, c)| c.as_ref()).map(|c| c.colors.iter().take(0x20).map(|k| k.to_le_bytes()).collect::<Vec<_>>());
    Some(K28Look {
        flag_cc: b4.and_then(|(_, c)| c.as_ref()).map_or(0, |c| c.flag as u32),
        keys,
        shader_flags: s.head[5] as u32,
        shader_rgba: [s.head[8], s.head[9], s.head[10], s.head[11]],
        frames: s.subs.first().map_or(0, |f| f.head[6] as i32),
        life: f32::from_bits(t[0x8c / 4]),
    })
}

/// J17: class-28 systems FUN_0015b428 built this frame get the look of their request's template
/// (`gfx::particles::ee_sub_1394c8_look`; k28_emit::ee_sub_1394c8 leaves it out). New systems in creation
/// order: linked ones through `ClassSet::k28_sys`, the others pair with the non-link requests in order.
pub fn k28_apply_looks(set: &mut ClassSet, host: &mut EngineHost, before: &std::collections::HashSet<usize>) {
    let reqs = std::mem::take(&mut host.k28_looks);
    let Some(w) = host.world.clone() else { return };
    let mut free = reqs.iter().filter(|r| !r.2);
    for (g, inst) in set.k21c.iter_mut() {
        if before.contains(g) {
            continue;
        }
        let super::Inst::K21C28(c) = inst else { continue };
        let r = match set.k28_sys.iter().find(|x| x.1 == *g) {
            Some(&(obj, _)) => reqs.iter().rev().find(|r| r.2 && r.0 == obj),
            None => free.next(),
        };
        let Some(l) = r.and_then(|r| k28_look(&w, r.0, &r.1)) else { continue };
        // a new system's +0x3b0 is 0 before FUN_001394c8 (k28_emit's one-frame default undone)
        c.m.set_i(0x3b0, 0);
        crate::engine::gfx::particles::ee_sub_1394c8_look(&mut **c, l.keys.as_deref(), l.flag_cc, l.shader_flags, l.shader_rgba, l.frames, l.life);
    }
}

/// K26: start the class-pool emitters that became audible this frame (sound bank PCM, as
/// game::app::play_sound_events does for the script pool; volume x the SFX volume).
#[cfg(feature = "game")]
pub fn play_class_sounds(
    mut c: ResMut<Classes>,
    sys: Option<Res<crate::engine::audio::AudioSystem>>,
    ga: Option<Res<crate::engine::game::app::GameAudio>>,
    mut sources: ResMut<bevy::asset::Assets<bevy::audio::AudioSource>>,
    mut commands: Commands,
) {
    use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings, Volume};
    let due = c.host.k26_due_sounds();
    let (Some(sys), Some(ga)) = (sys, ga) else { return };
    for (sound, v) in due {
        if let Some(p) = sys.sound_pcm(sound) {
            let h = sources.add(AudioSource { bytes: Arc::from(p.to_wav()) });
            commands.spawn((AudioPlayer(h), PlaybackSettings { volume: Volume::Linear(v * ga.sfx_volume), ..PlaybackSettings::DESPAWN }));
        }
    }
}

/// K27: anim event kind 2 of class-run ASEGAs: FUN_0014f750(g_cm, kind) at the camera clock (Time, as
/// playerdraw's CmClock); the last kind of the frame wins (each call overwrites kind / start time).
#[cfg(feature = "game")]
pub fn apply_class_shakes(mut c: ResMut<Classes>, cm: Option<ResMut<crate::engine::camera::cm::Cm>>, time: Res<bevy::time::Time>) {
    if let (Some(mut cm), Some(k)) = (cm, c.host.k27_shake.take()) {
        crate::engine::camera::cm::FUN_0014f750(&mut cm, k, crate::engine::camera::cm::CmClock { t: time.elapsed_secs(), dt: 0.0, dt_real: 0.0 });
    }
}

/// K29: the class code's camera requests on g_cm (classes/k29_cam.rs), at the camera clock (Time, as
/// playerdraw's CmClock); the CM itself is updated by playerdraw's follow_camera (FUN_0014d478).
#[cfg(feature = "game")]
pub fn apply_class_camera(mut c: ResMut<Classes>, cm: Option<ResMut<crate::engine::camera::cm::Cm>>, time: Res<bevy::time::Time>, sf: Option<Res<crate::engine::playerdraw::plugin::SlyFollow>>, mut st: Local<super::k29_cam::K29Cam>) {
    let reqs = std::mem::take(&mut c.host.k29_cam);
    // K30: published object states and playerdraw's look policy (classes/k30_cam.rs)
    st.objs.lock().unwrap().extend(c.host.k30_cam.drain());
    st.look = sf.and_then(|f| f.look);
    if let Some(mut cm) = cm {
        // K30: positions the zoom look reads (its objects, those of the queued setters, the player)
        {
            use super::k30_cam::ZoomOp;
            let h: &mut dyn ClassHost = &mut c.host;
            let mut z = st.zoom.lock().unwrap();
            let mut objs: Vec<u32> = vec![z.f_10, z.f_14];
            objs.extend(reqs.iter().filter_map(|r| match r {
                super::k29_cam::CamReq::Zoom(ZoomOp::Center(o) | ZoomOp::Target(o)) => Some(*o),
                _ => None,
            }));
            z.pos.objs.clear();
            for o in objs.into_iter().filter(|&o| o != 0) {
                let p = h.ee_sub_1a62e8(o);
                z.pos.objs.insert(o, p);
            }
            let pl = h.ee_sub_1c4a48();
            z.pos.player = (pl != 0).then(|| h.ee_sub_1a62e8(pl));
        }
        let clk = crate::engine::camera::cm::CmClock { t: time.elapsed_secs(), dt: time.delta_secs(), dt_real: time.delta_secs() };
        st.apply(&mut cm, &reqs, clk);
        c.host.k30_zoom_waits = Some(st.zoom_waits);
    }
}

/// K28: the class code's pad lock bits 2 / 4 (`EngineHost::k27_pad_lock`, action 0x12 world state) on the
/// input backend: FUN_00187078 for bits newly set, FUN_001870a0 for bits cleared (DAT_002dfbc4 / 002dfbc0).
#[cfg(feature = "game")]
pub fn apply_class_pad_lock(c: Res<Classes>, pad: Option<ResMut<crate::engine::input::backend::PadInput>>, mut prev: Local<u32>) {
    let Some(mut pad) = pad else { return };
    let p = &mut *pad;
    let bits = c.host.k27_pad_lock & 6;
    if bits & !*prev != 0 {
        crate::engine::input::pad::FUN_00187078(&mut p.lock, &mut p.pad, bits & !*prev);
    }
    if *prev & !bits != 0 {
        crate::engine::input::pad::FUN_001870a0(&mut p.lock, &mut p.pad, *prev & !bits);
    }
    *prev = bits;
}

pub struct ClassesPlugin;

/// K26: FUN_001fd0e8's music keys / instance (level header block 0x3dd040) for the next class-set build.
#[cfg(feature = "game")]
pub fn feed_music_block(res: Option<Res<crate::engine::disc::LevelLoadResult>>, mut c: ResMut<Classes>) {
    let Some(r) = res.filter(|r| r.is_changed()) else { return };
    if let Ok(l) = &r.0 {
        c.host.k26_music = super::k26::MusicRt::from_block(&l.header.block_3dd040);
    }
}
impl Plugin for ClassesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ClassesLevel>().init_resource::<Classes>().add_systems(Update, tick_classes);
        // WIRE1: queue ports over the live guest image (classes/q/live.rs)
        app.add_plugins(super::q::live::LiveQPlugin);
        #[cfg(feature = "game")]
        app.add_systems(Update, play_class_sounds.after(tick_classes));
        #[cfg(feature = "game")]
        app.add_systems(Update, apply_class_shakes.after(tick_classes));
        #[cfg(feature = "game")]
        app.add_systems(Update, apply_class_camera.after(tick_classes).before(crate::engine::game::app::lend_classes));
        // K28: before the class runtime is lent to the scripts (the host is then in `Classes`)
        #[cfg(feature = "game")]
        app.add_systems(Update, apply_class_pad_lock.after(tick_classes).before(crate::engine::game::app::lend_classes));
        #[cfg(feature = "game")]
        app.add_systems(Update, feed_music_block.before(tick_classes));
    }
}

impl EngineHost {
    /// K29: queue a camera request (bounded: a host without a CM drops the oldest)
    fn k29_req(&mut self, r: super::k29_cam::CamReq) {
        if self.k29_cam.len() >= 256 {
            self.k29_cam.remove(0);
        }
        self.k29_cam.push(r);
    }

    /// K13 / K24: world object matrix = root bone world (rest pose) under its +0x12c placement (clones); K16: no
    /// own frame -> the owner-chain frame (P15 instance_frames). None = not a world object.
    pub(super) fn k24_world_xf(&mut self, obj: u32) -> Option<crate::engine::gfx::scene::Affine> {
        let w = self.world.clone()?;
        let i = (obj as usize).checked_sub(1).filter(|&i| i < w.objects.len())?;
        let id = Arc::as_ptr(&w) as usize;
        if self.pos_for != id {
            self.pos_cache.clear();
            self.k26_posed.clear();
            self.rt_parent.clear();
            self.pos_for = id;
        }
        // K26: posed by a class-run ASEGA channel (classes/k26.rs)
        if let Some(m) = self.k26_posed.get(&i) {
            return Some(*m);
        }
        // WP-EP1d: re-parented at run time (SetParent): the kept matrix relative to the new parent
        if let Some(&(p, local)) = self.rt_parent.get(&i) {
            return Some(match p {
                0 => local,
                _ if self.rt_depth > 64 => local,
                _ => {
                    self.rt_depth += 1;
                    let pw = self.rt_world(p);
                    self.rt_depth -= 1;
                    crate::engine::gfx::scene::mul(&local, &pw)
                }
            });
        }
        Some(*self.pos_cache.entry(i).or_insert_with(|| {
            use crate::engine::gfx::attach::{affine_from_bytes, BoneWorlds};
            let mut bw = BoneWorlds::default();
            match w.life.placed.get(&i).map(affine_from_bytes) {
                Some(pm) => crate::engine::gfx::scene::mul(&bw.get(&w, i, 0), &pm),
                // WP-K33: an XFM on a bone of a placed (P15) object, e.g. a class-37 guard loot holder under its
                // class-48 owner: local x the owner bone x the owner's placement (the case above for bone k)
                None if let (Some((_, local)), Some(crate::engine::gfx::attach::BoneRef::Bone(o, k))) =
                    (crate::engine::gfx::attach::xfm_of(&w, i), crate::engine::gfx::attach::xfm_bone_ref(&w, i))
                    && w.life.placed.contains_key(&o) =>
                {
                    let pm = affine_from_bytes(&w.life.placed[&o]);
                    crate::engine::gfx::scene::mul(&local, &crate::engine::gfx::scene::mul(&bw.get(&w, o, k), &pm))
                }
                None => bw.xfm_world(&w, i).or_else(|| super::shared::path10::world_xf(&w, i, None)).unwrap_or_else(|| bw.get(&w, i, 0)),
            }
        }))
    }

    /// WP-EP1d: world matrix of a handle as the hierarchy sees it (the player = Sly's position, K27-driven
    /// objects their physics position, else `k24_world_xf`; unknown = identity)
    fn rt_world(&mut self, h: u32) -> crate::engine::gfx::scene::Affine {
        use crate::engine::gfx::scene::IDENTITY;
        if h == super::guard::PLAYER_HANDLE {
            let mut m = IDENTITY;
            m[3] = self.player_pos;
            return m;
        }
        let mut m = self.k24_world_xf(h).unwrap_or(IDENTITY);
        if let Some(p) = (h as usize).checked_sub(1).and_then(|i| self.k27_pos(i)) {
            m[3] = p;
        }
        m
    }

    /// WP-EP1d: FUN_001a58d0(obj, parent, flags) = (vtable +0x54)(obj, parent, 0, flags): ALO SetParent
    /// FUN_001f21b8 (classes 5 / 48 / 98 share it). flags & 3: the world matrix is kept (computed up the old
    /// chain, made relative to the new parent by vtable +0xec / +0xf4); else the local +0x40 matrix is kept
    /// (the object moves with the new parent). `parent` 0 = detach (world root). REPLACED: the +0x60 / +0xa0
    /// LO child-list relink (C2c94e0_v9) is the `rt_parent` table (`parent_10` / `alo_children` /
    /// `k24_world_xf` read it); flags 4 / 8 (velocity / angular velocity via +0x10c / +0x114 -> +0x11c /
    /// +0x124) carry no state here (the class physics owns velocities); the +0x54 0x100-record +0xa0 rebinding
    /// and FUN_001f2468 / 001f26a8 / 001f8668 (attachment refresh, SO re-link) are not modelled.
    pub fn ee_sub_1a58d0_set_parent(&mut self, obj: usize, parent: u32, flags: u32) {
        use crate::engine::gfx::scene::{invert, mul};
        let Some(w) = self.world.clone() else { return };
        if obj >= w.objects.len() {
            return;
        }
        // the old parent as the hierarchy sees it now
        let old = self.parent_10(obj as u32 + 1);
        let world_now = self.rt_world(obj as u32 + 1);
        let base = if flags & 3 != 0 { parent } else { old };
        let local = match base {
            0 => world_now,
            b => mul(&world_now, &invert(&self.rt_world(b))),
        };
        let parent = if parent == obj as u32 + 1 { 0 } else { parent };
        self.rt_parent.insert(obj, (parent, local));
        self.pos_cache.clear();
    }

    /// WP-EP1d: FUN_001f6ac8 (on) / FUN_001f6b48 (off): every +0xe4 part (0xb0 B, count +0xe0) whose +0xc
    /// word == id: FUN_001f2a60 (bit 50 set -> clear, link into the SO set FUN_001f2870 when the object is in
    /// the world, +0x28 bit 60) / FUN_001f2ad0 (bit 50 clear -> set, unlink FUN_001f2930). REPLACED: the SO set
    /// is rebuilt (`so`, `so_gen` for the player layer); the in-world test is always true.
    pub fn so_part_link(&mut self, obj: usize, id: i32, on: bool) {
        use crate::engine::world::objects::{LoadData, Payload};
        let Some(w) = self.k7_cache() else { return };
        let Some(Payload::Loaded(LoadData::Alo { alo, .. })) = w.objects.get(obj).map(|o| &o.payload) else { return };
        let mut changed = false;
        for (k, p) in alo.parts.iter().enumerate() {
            if p.w0 != id {
                continue;
            }
            let cur = self.so_parts.get(&(obj, k)).copied().unwrap_or(p.b2 & 1 == 0);
            if cur != on {
                self.so_parts.insert((obj, k), on);
                changed = true;
            }
        }
        if changed {
            self.so = Some(Arc::new(crate::engine::physics::query::CollisionWorld::build_so_masked(&w, &self.so_parts)));
            self.so_gen = self.so_gen.wrapping_add(1);
        }
    }

    /// WP-EP1d: FUN_001f6bc8(obj, id, v): +0x18 = v of every part whose +0xc word == id
    pub fn ee_sub_1f6bc8(&mut self, obj: usize, id: i32, v: u32) {
        use crate::engine::world::objects::{LoadData, Payload};
        let Some(w) = self.k7_cache() else { return };
        let Some(Payload::Loaded(LoadData::Alo { alo, .. })) = w.objects.get(obj).map(|o| &o.payload) else { return };
        for (k, _) in alo.parts.iter().enumerate().filter(|(_, p)| p.w0 == id) {
            self.so_part_18.insert((obj, k), v);
        }
    }
}
