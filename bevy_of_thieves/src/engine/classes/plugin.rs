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

/// TAKEN BY MURRAY: a thrown SO object in flight (pos, velocity, gravity, FUN_001f7a68 part kind, age)
#[derive(Clone, Debug, Default)]
pub struct SoFly {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub g: [f32; 3],
    pub kind: u32,
    pub age: f32,
}

/// Host recording the hook calls that have no engine port yet.
#[derive(Default, Clone)]
pub struct EngineHost {
    /// TAKEN BY MURRAY: SO bodies of carried / thrown objects (REPLACED physics, see `so_carry` / `so_fly_step`)
    pub so_carry: std::collections::HashMap<u32, u32>,
    pub so_fly: std::collections::HashMap<u32, SoFly>,
    pub so_radius: std::collections::HashMap<usize, f32>,
    /// POOL1: FUN_001cb250 template pools of non-guard spawners (Bentley dart / bomb hosts bound by
    /// FUN_001a64e8 rows of DAT_002bc390): spawner -> current copy (entry, copy) (spawner +0xe8 head)
    pub pool_cur: std::collections::HashMap<u32, (usize, i32)>,
    /// POOL1: copy record owner (FUN_001ca978 score 4 = unowned)
    pub pool_owner: std::collections::HashMap<(usize, i32), u32>,
    /// POOL1 R80: FUN_001caec8 vt +0x54(copy, sp +0x10, sp +0x50): a claimed copy's LO parent = the claiming
    /// record's owner (copy object -> owner object). REPLACED: only its in-world side is modelled (FUN_001a5478
    /// / FUN_001a5630 recurse over it in `lo_rt`); the copy's frame stays the spawner's (`pool_place`)
    pub pool_parent: std::collections::HashMap<usize, usize>,
    /// CRAWL R84: FUN_001caec8 key-6 field-243 rows written into a handed-out copy's anim table (copy -> (anim id,
    /// ASEG oid)); they stay with the copy (MREC9: kept after later hand-outs), `guard::los::spawner_anim_ovr`
    pub anim_ovr: std::collections::HashMap<usize, Vec<(i32, i32)>>,
    /// POOL1: ALO +0x110 per-kind bytes (FUN_001f7918, script method 0x00100fe0), copied from the script's
    /// `ObjectFields::alo_110` each frame by player::plugin::feed_alo_110 (FUN_001f78d8 of FUN_001f7598)
    pub alo_110: std::collections::HashMap<u32, [u8; 26]>,
    /// POOL1: copy objects added to the world by FUN_001caec8 (vtable +0x44) (score 2 = active)
    pub pool_live: std::collections::HashSet<u32>,
    /// POOL1: FUN_001cb488 / FUN_001cb4b8 attachment flags (+8) per copy object
    pub pool_attach: std::collections::HashMap<u32, u32>,
    /// POOL1: copies claimed this frame (FUN_001caec8 places them at the spawner frame)
    pub pool_place: std::collections::HashSet<u32>,
    /// FEEL R35: FUN_00149ab8 nodes {spawner, PMF 0x2a77c0 = FUN_001ca8c0, 0} (C2d5dc8 slot 22), run by
    /// the next FUN_00149c80 (after the load-post here)
    pub spawn_q: Vec<usize>,
    /// POOL1 R99: with the proximity set on (SLY2_R64=1): the spawners whose slot 23 (FUN_001ca8e0) runs on
    /// entering it, with their last FUN_001a61a0 state (`sp_watch_frame`)
    pub sp_watch: Vec<(usize, bool)>,
    /// POOL1 R99: run-time LO +0x28 bit 56 (removed on its own) by object index (`lo_rt`); else `removed_self_at_load`
    pub lo_b56: std::collections::HashMap<usize, bool>,
    /// R53: copies added to the world this frame (FUN_001a5478 -> vt +0x5c slot 10, run by player/plugin.rs)
    pub pool_add: Vec<u32>,
    /// R53: **(obj +0x14) roots the world model lacks: Bentley's riding emitter +0x16e4 -> Bentley (player/plugin.rs)
    pub ride_root: std::collections::HashMap<u32, u32>,
    /// R53: live copies taken out of the world (vt +0x4c) this frame, removed by player/plugin.rs
    pub pool_off: std::collections::HashSet<u32>,
    /// POOL1: vt +0x124 velocities set on pool copies (applied by player::plugin's pool pass)
    /// POOL1 R56: `pool_off` copies that were in the world (+0x28 bit 60) when vt +0x4c reached them:
    /// FUN_001a5630 calls vt +0x64 (slot 11, the SMA freed) at the Remove, before the bit is cleared
    pub pool_off_in: std::collections::HashSet<u32>,
    pub pool_v: Vec<(u32, [f32; 3])>,
    /// POOL1: (vtable +0x264)(copy, s) = slot 75 SetState of a pool copy (K15 class 101 / K23 102), applied
    /// by player::plugin::po_link on the class instance
    pub pool_264: Vec<(u32, i32)>,
    /// POOL1: FUN_001357c0(bentley, tgt) requests (anim event 0x243), fired by player::plugin::po_link
    pub pool_fire: Vec<(usize, u32)>,
    /// POOL1: FUN_001b7000(murray, carry point) requests (class-10 point slot 50, guard carry handles)
    pub murray_1b7000: Vec<(u32, u32)>,
    /// POOL1: run-time +0x18 hit kind of collision part 0 (+0xe4 record) per object, e.g. FUN_001357c0's
    /// `*(dart +0xe4 + 0x18) = kind` (0x80000 sleep dart)
    pub part0_kind: std::collections::HashMap<usize, i32>,
    /// POOL1: DAT_002dfbc0 pad routing mode (input backend `InputLock::mode`, FUN_001870d0), fed each frame
    pub g_2dfbc0: u32,
    /// POOL1: slot-57 contacts of pool copies found by the player-layer ground probe (drained by so_contacts_57)
    pub pool_contacts: std::collections::HashMap<usize, Vec<super::k15_101::Contact57>>,
    /// POOL1 R51: last attach point of each held copy (its hand velocity = attach-point motion per frame)
    pub pool_att_prev: std::collections::HashMap<u32, [f32; 3]>,
    /// CRAWL R76: live K6 guard positions (FUN_001a62e8 reads the ALO world xf the guard update writes; the
    /// guard body lives in the class set) - handle -> body pos, republished each frame_k6
    pub k6_live: std::collections::HashMap<u32, [f32; 3]>,
    /// CRAWL R85: live K6 guard body yaws (handle -> yaw), republished with k6_live
    pub k6_live_yaw: std::collections::HashMap<u32, f32>,
    /// R53: FUN_001f7bf8 fade-outs of pool copies (alpha +0x12c, rate +0x130 = -1 / t): seconds left until the
    /// FUN_001f3980-tail vtable +0x4c world removal (the spawner's FUN_001ca978 then sees the copy out: 3)
    pub pool_fade: std::collections::HashMap<u32, f32>,
    /// WIRE-D: FUN_00156b98(world, 0, pos, obj) noise records this frame (obj handle, pos), drained by
    /// guard::hearing::sly_noise into the DAT_003e148c pool
    pub noises: Vec<(u32, [f32; 3])>,
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
    /// BOSS R26: in-world class-88 TNT barrels (handle, FUN_001a62e8 position), fed by classes::q::barrel::beams;
    /// FUN_001f7598 hits on them (class-85 beams) wait in `barrel_hits` for the live image's slot 74
    pub barrels_85: Vec<(u32, [f32; 3])>,
    pub barrel_hits: Vec<super::guard::slots::GuardHit>,
    /// BOSS: FUN_001e0020(seq, signal) posts, drained by ClassSet before the sequence update
    pub seq_signals: Vec<(u32, i32)>,
    /// K7: sensor-state script events (FUN_001a61e0 msg 0x1c) raised
    pub sensor_events: usize,
    /// K7: contact pairs per guard object for slot 73 (fed by the contact solver; the SO dynamics contact
    /// list +0xec -> +0x6c is not built for guards yet), drained each frame
    pub k7_contacts: Vec<(usize, super::guard::slots::Contact)>,
    /// K8: Sly's pickup record (player +0x15c: alive, radius; None = no record until the player layer feeds
    /// it), Sly's position (FUN_001a62e8 of the player handle), hp / mp gifts of class-93 pickups (player slot
    /// 80, drained by the player layer)
    pub player_rec: Option<super::k8::PickRec>,
    /// WIRE-OBJ2: Sly's collision part (GEOM 199 hull) with his orientation, for FUN_0014c6a0 segment tests
    pub player_hull: Option<(crate::engine::player::p5_hull::JtHull, crate::engine::physics::driver::M3)>,
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
    /// WIRE-OBJ: FUN_001c6fd8 requests from class hooks, drained by `ClassSet::k8_drops`
    pub k8_drop_req: Vec<(i32, [f32; 3], u32, i32)>,
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
    /// EMU5: FUN_001e1e30 case 0x16 teleports (subject, locator, rec[5]); Sly's (PLAYER_HANDLE) are
    /// drained by the player layer (`player::plugin::drain_k22_teleports`)
    pub k22_teleports: Vec<(u32, u32, u32)>,
    /// FEEL R81: FUN_001e1e30 case 0x2a hops (subject, target, rec[5]) drained by player::plugin::loot_link
    pub k22_hops: Vec<(u32, u32, u32)>,
    /// FEEL R86: FUN_001e1e30 case 0x2b launches (subject, target, f12, f13) drained by player::plugin::loot_link
    pub k22_launches: Vec<(u32, u32, f32, f32)>,
    /// FEEL R81: the live player layer drains `k22_hops` (set by loot_link) and mirrors Sly's JMP jtbs
    pub hop_drainer: bool,
    pub player_jtbs: i32,
    /// FEEL R92: Sly's jts (8 = riding a path, +0xf54 != -1), k22 0x1a climb goals of this frame (subject,
    /// locator pos) and the player's last reported |u - target u|
    pub player_jts: i32,
    pub k22_climbs: Vec<(u32, [f32; 3])>,
    pub k22_climb_left: Option<f32>,
    /// CRAWL R43: FUN_001e1e30 case 0x24 FUN_001938c0 auto-walks (subject, mask, target); UNPORTED walker
    /// (REQUEST POOL1); nothing drains them yet
    pub k22_autowalks: Vec<(u32, u32, u32)>,
    /// CRAWL R46: FUN_001448f0 walk-to goals (subject, world point) from the 0x24 wait; drained by the player layer
    pub k22_walktos: Vec<(u32, [f32; 3], f32)>,
    /// CRAWL R122: live positions of NPCs (Murray / Bentley handles) moved by k22 0x24 walks (FUN_001a62e8 reads them)
    pub npc_pos: std::collections::HashMap<u32, [f32; 3]>,
    /// FEEL R93: subjects of this frame's 0x24 walks with rec[9] != 0 (joy +0x30 |= 8, run)
    pub k22_walk_r1: Vec<u32>,
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
    /// ANIM-ALL R92: vt+0xf4 world transforms of run-time objects past the load world (FUN_00159500's class-29
    /// groups: `(vt+0xf4)(group, FUN_001a6318(member))`), index -> world matrix
    pub rt_xf: std::collections::HashMap<usize, crate::engine::gfx::scene::Affine>,
    /// ANIM-ALL R64: world proximity set / camera mask (classes/sw_prox.rs)
    pub sw: super::sw_prox::SwProx,
    /// K11: engine calls reached with no ported callee (explicit Unported errors)
    pub unported: Vec<String>,
    /// K13: Sly exists (PlayerFocus valid): FUN_001c4840 returns PLAYER_HANDLE (the class-53 JT), else 0
    pub player_valid: bool,
    /// CRAWL R52: `player_pos` is fed from a live JT this frame (player::plugin / the test harness): Sly's own
    /// class-53 JT object then reads the JT position in FUN_001a62e8 (the original's JT object IS the player)
    pub jt_live: bool,
    /// WIRE-EP6: FUN_00144068(jt) of the player (JT +0x270 > 0 || +0x1084 != 0), for the field-759 getter
    pub jt_busy: bool,
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
    /// CRAWL R103: a sequence action 7 (slot 78(1)) on Sly: the live JT's FUN_001c3ed8(jt, 1) (takes the world
    /// focus back from e.g. the safehouse hub, whose +0x27c(0) leaves its select state); done by ui::live_menu
    pub sly_focus_req: bool,
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
    /// CRAWL: LOs removed by a script LO remove (FUN_001a5630; laser_gate.rs lo_world_link)
    pub laser_off: Vec<usize>,
    /// FEEL R32: the out-of-world objects (LO +0x28 bit 60 clear) last synced from the scripts' object state
    /// (`sync_in_world`); None = not synced yet (the load-time state, `out_of_world_at_load`)
    pub oow: Option<std::collections::HashSet<u32>>,
    /// FEEL R34: class-code LO Add / Remove (`lo_rt`) not yet applied to the scripts' object state (drained by
    /// game::app::lend_classes before `sync_in_world`), and their subtree state until then
    pub rt_queue: Vec<(usize, bool)>,
    /// BOSS R28: class-code calls on objects whose class runs in the live image (EP3 class-88 TNT barrel
    /// copies): (callee, object handle, argument); callee < 0x1000 = a vtable offset (+0x44 Add / +0x4c
    /// Remove), else FUN_001f6978 / 69f8 / 6ac8 / 6b48(obj, oid) of an ASEG kind 0xd..0x10 event. Applied to
    /// the live object by `SpliceState::flush_live_deferred` (vm_state.rs) at the next world update
    pub live_calls: Vec<(u32, u32, u32)>,
    pub rt_overlay: std::collections::HashMap<usize, bool>,
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
    /// CRAWL R123: last posed bone worlds / object worlds per world (FUN_00130b18 retract leaves the bone locals and the
    /// object xf as the ASEGA last wrote them), world id
    pub k29_held: (usize, std::collections::HashMap<(usize, usize), crate::engine::gfx::scene::Affine>, std::collections::HashMap<usize, crate::engine::gfx::scene::Affine>),
    /// K26: music manager 0x3dd040 layer keys / DAT_003dd268 instance sources (classes/k26.rs MusicRt); set
    /// from the level block before the class set is built (`k26_music_block`)
    pub k26_music: super::k26::MusicRt,
    /// K27: non-JT character PO bodies (classes/k27.rs) by object index
    pub k27_po: std::collections::HashMap<usize, super::k27::CharPo>,
    /// TAKEN BY HEIST: queued FUN_00136530(obj, s) calls (ClassHost::ee_sub_136530)
    pub q_136530: Vec<(usize, i32)>,
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
    /// BOSS R30: FUN_001758a0 message-0x18 queries (guard object, candidate (record index, oid) list), run by
    /// vm_state's flush; answers (guard -> (candidates, allowed record)) are used by the next poll with the same list
    pub atk_msg18_q: Vec<(usize, Vec<(usize, i32)>)>,
    pub atk_msg18_ans: std::collections::HashMap<usize, (Vec<(usize, i32)>, Option<usize>)>,
    /// BOSS R32: guards whose script frame has a message 0x18 handler (found by the script flush); any other guard
    /// gets FUN_001331f8's no-op (the reply {0x17, 1} stays: the first allowed record) in the same poll
    pub atk_msg18_hdl: std::collections::HashSet<usize>,
    /// POOL1 r30: ASEGA script messages (ASEG object index, ASEGA handle, owner handle, msg), drained by vm_state
    pub aseg_script_msgs: Vec<(usize, u32, u32, u32)>,
    /// ANIM-ALL R84: live-image SMA script messages in send order (SM object index, SMA, owner handle, msg, words),
    /// delivered by game::vm_state::send_messages before the split queues (SLY2_NO_R84_SMAORDER = split as before)
    pub live_sma_seq: Vec<(usize, u32, u32, u32, Vec<u32>)>,
    /// POOL1 r30: the ASEGA whose script message is running (handle, owner handle): get-o field 251 (+0x1c owner)
    pub aseg_script_self: Option<(u32, u32)>,
    /// POOL1 r36: SMA script message 8 selves keyed by (SM object handle, old, new) -> (SMA handle, owner handle)
    pub sma_script_self_q: std::collections::HashMap<(u32, u32, u32), (u32, u32)>,
    /// HEIST: ASEG event kind 0xc (FUN_00131290: asega vt+0x1c(4, 0, 1, &payload)) = script message 4 of the
    /// ASEGA with the event's s32 (a symid) (ASEG object, handle, owner handle, payload)
    pub heist_aseg_obj: u32,
    pub heist_aseg_msg4: Vec<(usize, u32, u32, i32)>,
    pub k27_cine: [i32; 4],
    pub k27_pad_lock: u32,
    /// PAD1: live pad state for ee_call(0x187d80/0x187de0) — written by game app or test harness each frame
    pub pad: Option<crate::engine::input::pad::PadState>,
    /// WIRE-MG: DAT_003e138c focus when it is a non-Bentley/Murray usable object (class 69 turret / scope):
    /// its +0x150 tracker (C2ca240 on joystick 0x3dd490) reads `pad` (player::p23::apply_switches sets it)
    pub po_focus: Option<u32>,
    pub k27_fades: usize,
    /// K27: anim event kind 5 seeks applied (FUN_00130d30 on class-run ASEGAs)
    pub k27_seeks: usize,
    /// WIRE-BM2: FUN_00131290 case 0xb owner messages 9 (k26_dispatch -> ClassSet slot 4)
    pub k26_msg9: Vec<(u32, [u32; 3])>,
    /// WIRE-W1: the same owner messages for the live queue objects (q::live::live_k26_msg9 -> QMem slot 4)
    pub k26_msg9_live: Vec<(u32, [u32; 3])>,
    /// K28: ASEGA +0x58 (anim event kinds 9 / 10 world state) by ASEGA handle; event calls applied (kinds 7..0x10)
    pub k28_asega_58: std::collections::HashMap<u32, bool>,
    pub k28_events: usize,
    /// K30: anim event 0x11 slot-51 emit requests (target handle, request) for `ClassSet::k28_spawn`
    pub k30_reqs: Vec<(u32, [u32; 15])>,
    /// POOL1 R54: the vt+0x19c emit requests queued by `vcall_19c` (target, r[0], r[1]); last 64
    pub k30_19c: Vec<(u32, u32, u32)>,
    /// K30: world +0x476c (anim events 0x12 / 0x13, FUN_001618f8 / FUN_00161958) and the dispatch time
    pub k30_476c: f32,
    pub k30_t: f32,
    /// K28: job-manager calls of the class code (callee 0x164640 / 0x164df0 / 0x1619a0, object handle, k,
    /// ctx handle) for `jobs::JobMgr` (applied by game::vm_state::apply_job_calls)
    pub k28_task_calls: Vec<(u32, u32, i32, u32)>,
    /// CRAWL R26: task states by task object handle (fed by game::vm_state after the K28 drain)
    pub task_states: std::collections::HashMap<u32, i32>,
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
    /// FEEL R35 / CRAWL R89: the load's deferred FUN_001ca8c0(spawner) nodes (FUN_00149ab8 PMF 0x2a77c0 from
    /// class-6/7 slot 22): FUN_001caec8(spawner, 2) Adds a free copy of the spawner's template (FUN_001cb250(sp, 0)
    /// = the same call for the single-object templates; UNPORTED: m > 1). Run once after `ClassSet::post_load`.
    pub fn drain_spawn_q(&mut self, set: &mut ClassSet) {
        let _ = self.k7_cache();
        for sp in std::mem::take(&mut self.spawn_q) {
            let c = ClassHost::ee_sub_1cb250(self, sp as u32 + 1, 0);
            if std::env::var_os("SLY2_R89_SQ").is_some() { eprintln!("R89SQ sp {sp} -> {c} cur {:?}", self.pool_cur.get(&(sp as u32 + 1))); }
            // CRAWL R89: FUN_001caec8's copy rebuild (vt +0x12c / +0x44) for a guard-class copy at the spawner frame,
            // as the method-1019 hand-out (getters.rs 0x0010e2b8), e.g. the EP1 pump-yard guard of spawner 123920
            if c == 0 || std::env::var_os("SLY2_NO_R89_SQG").is_some() {
                continue;
            }
            let Some(w) = self.world.clone() else { continue };
            let pl = self.k24_world_xf(sp as u32 + 1).unwrap_or(crate::engine::gfx::scene::IDENTITY);
            if set.k6_guard(c as usize - 1).map_or(true, |g| g.f_330 != 0) {
                let ok = set.k6_pool_guard(&w, c as usize - 1, pl, Some(sp), self);
                if std::env::var_os("SLY2_R89_SQ").is_some() { eprintln!("R90SQG sp {sp} oid {} copy {c} class {} ok {ok} pos {:?}", w.objects[sp].oid, w.objects[c as usize - 1].class, [pl[3][0].round(), pl[3][1].round(), pl[3][2].round()]); }
            }
        }
    }
    /// POOL1 R99: after the frame's FUN_00147f40 (in `ClassSet::frame`): a watched spawner that entered the
    /// proximity set runs slot 23 (FUN_001ca8e0 -> FUN_00149ab8 node -> FUN_001ca8c0 claim, drained this frame)
    pub fn sp_watch_frame(&mut self, set: &mut ClassSet) {
        if self.sp_watch.is_empty() {
            return;
        }
        let mut w = std::mem::take(&mut self.sp_watch);
        for (sp, was) in w.iter_mut() {
            let now = !crate::engine::classes::ClassHost::lo_removed(self, *sp) && self.sw.built && EngineHost::sw_active(self, *sp);
            if now && !*was && !self.spawn_q.contains(sp) {
                self.spawn_q.push(*sp);
            }
            *was = now;
        }
        self.sp_watch = w;
        if !self.spawn_q.is_empty() {
            self.drain_spawn_q(set);
        }
    }
    pub(crate) fn k7_cache(&mut self) -> Option<Arc<World>> {
        let w = self.world.clone()?;
        let id = Arc::as_ptr(&w) as usize;
        if self.k7_for != id {
            self.blockers = Some(Arc::new(super::guard::los::Blockers::build(&w)));
            self.so_parts.clear();
            self.so_part_18.clear();
            self.oow = None;
            self.rt_queue.clear();
            self.rt_overlay.clear();
            self.npc_pos.clear();
            self.lo_b56.clear();
            self.pool_parent.clear();
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
impl EngineHost {
    /// POOL1: world position of the spawner owning pool copy `copy` while FUN_001cb488 holds it attached
    /// (+8 = 1): the class-6 spawner rides its owner's bone (load `bone`, e.g. Bentley 4132) + xform, the copy
    /// sits at the spawner (REPLACED: the attachment record's matrix -> the spawner's K24 world matrix)
    pub fn pool_attach_pos(&mut self, copy: u32, carrier: u32, carrier_pos: [f32; 3]) -> Option<[f32; 3]> {
        let place = self.pool_place.remove(&copy);
        if self.pool_attach.get(&copy) != Some(&1) && !place {
            return None;
        }
        let w = self.world.clone()?;
        let sp = self.pool_cur.iter().find(|(_, ec)| w.entries[ec.0].instances.get(ec.1 as usize).map(|&o| o as u32 + 1) == Some(copy)).map(|(&s, _)| s)?;
        // the loaded spawner offset from its carrier's loaded placement, moved with the carrier's live body
        // (REPLACED: bone 4132's animated matrix / carrier yaw not applied)
        let (sm, cm) = (self.k24_world_xf(sp)?, self.k24_world_xf(carrier)?);
        let (a, b) = (sm[3], cm[3]);
        let mut d = [0, 1, 2].map(|k| a[k] - b[k]);
        let mut on_bone = false;
        // POOL1 R89: the spawner's XFM hangs on a carrier bone (Bentley's bomb spawner: bone 36 of the carrier); when the
        // class-run ASEGAs pose that bone this frame (K29), the spawner world is XFM local x the POSED bone world (both in
        // the carrier's load frame) instead of the rest pose (rest = hand out to the side: gate_blast bomb 56 u to the
        // right of Bentley). SLY2_R89_NO_ATTBONE=1 = the rest-pose offset.
        if std::env::var_os("SLY2_R89_NO_ATTBONE").is_none() {
            use crate::engine::gfx::attach::{xfm_bone_ref, xfm_of, BoneRef};
            let i = sp as usize - 1;
            if let (Some((_, xl)), Some(BoneRef::Bone(o, k))) = (xfm_of(&w, i), xfm_bone_ref(&w, i)) {
                if o as u32 + 1 == carrier {
                    on_bone = true;
                    if let Some(pb) = self.k29_bones.get(&(o, k)) {
                        let m = crate::engine::gfx::scene::mul(&xl, pb);
                        d = [0, 1, 2].map(|k| m[3][k] - b[k]);
                    }
                }
            }
        }
        // POOL1 R89: the offset is a carrier-local one (the spawner rides the carrier's bone): take it in the carrier's
        // loaded frame and turn it with the live body's rows (vt +0xfc world matrix = the JT's rot for the po), so a
        // carrier facing away from its load yaw holds the copy in front, not at the load-yaw side (gate_blast: bomb
        // 56 u to Bentley's right). Only for spawners on a carrier bone; OPT-IN SLY2_R89_ATTROT=1 (R90: breaks pool1_disco_all_pillars, not needed for gate_blast).
        if on_bone && std::env::var_os("SLY2_R89_ATTROT").is_some() {
            if let Some(rot) = self.po.as_ref().filter(|p| p.obj as u32 + 1 == carrier).map(|p| p.jt.phys.rot) {
                let l = [0, 1, 2].map(|i| d[0] * cm[i][0] + d[1] * cm[i][1] + d[2] * cm[i][2]);
                return Some([0, 1, 2].map(|k| carrier_pos[k] + l[0] * rot[0][k] + l[1] * rot[1][k] + l[2] * rot[2][k]));
            }
        }
        Some([0, 1, 2].map(|k| carrier_pos[k] + d[k]))
    }
    /// POOL1: FUN_001ca978(spawner, copy) (text_001c0000.c:7925): record +8 != 0 (FUN_001cb488 attached) -> 0;
    /// unowned -> 4; else min over the copy objects: in the world (+0x28 bit 60, here `pool_live`) and
    /// FUN_001a61a0 (in the 0x3dd4bc set) -> 1 (returned at once), in the world -> 2, out -> 3.
    /// POOL1 R63: ranks 0 and 1 (the attach record is per copy slot; the port keys `pool_attach` by object)
    fn pool_1ca978(&self, w: &crate::engine::world::World, e: usize, copy: i32) -> i32 {
        let en = &w.entries[e];
        let m = en.objects.len();
        let objs = || (0..m).filter_map(|j| en.instances.get(copy as usize * m + j).map(|&o| o as u32 + 1));
        if objs().any(|c| self.pool_attach.get(&c) == Some(&1)) {
            return 0;
        }
        if !self.pool_owner.contains_key(&(e, copy)) {
            return 4;
        }
        let mut r = 5;
        for c in objs() {
            if self.pool_live.contains(&c) {
                if self.set_3dd4bc.contains(&c) {
                    return 1;
                }
                r = r.min(2);
            } else {
                r = r.min(3);
            }
        }
        r
    }
    /// Spawner +0xe0 template entry. FEEL R37: a load-time copy (FUN_0013e510, `clone_of`) carries its
    /// source's +0xe0, so e.g. each guard's copy of coin spawner 691 draws from the same entry pool.
    pub fn sp_entry(w: &crate::engine::world::World, mut k: usize) -> Option<usize> {
        loop {
            if let Some(e) = super::k21_119::Host119::entry_of(w, k) {
                return Some(e);
            }
            k = w.objects.get(k)?.clone_of.filter(|&c| c != k)?;
        }
    }
    /// FEEL R38: DAT_003e1ac8 = the loot pool spawner: the class-6 spawner whose template entry holds loot `obj`
    pub fn loot_spawner(w: &crate::engine::world::World, obj: u32) -> Option<u32> {
        let o = (obj as usize).checked_sub(1)?;
        let e = w.entries.iter().position(|e| e.instances.iter().any(|&i| i as usize == o))?;
        (0..w.objects.len()).find(|&i| w.objects[i].class == 6 && Self::sp_entry(w, i) == Some(e)).map(|i| i as u32 + 1)
    }
    /// FEEL R39: spawner +0xe4 (field 1178, `classes::class6_e4`)
    pub fn sp_e4(w: &crate::engine::world::World, k: usize) -> bool {
        k < w.objects.len() && super::class6_e4(w, k)
    }
    /// POOL1: FUN_001cade0(spawner, min) -> copy index (None = 0)
    fn pool_1cade0(&self, w: &crate::engine::world::World, sp: u32, min: i32) -> Option<(usize, i32)> {
        let e = Self::sp_entry(w, (sp as usize).checked_sub(1)?)?;
        // +0xe4 == 0 with a +0xe8 head: only the head (FEEL R39: +0xe4 != 0 spawners always scan)
        if let Some(&(ce, c)) = self.pool_cur.get(&sp).filter(|_| !Self::sp_e4(w, sp as usize - 1)) {
            return (self.pool_1ca978(w, ce, c) >= 3).then_some((ce, c));
        }
        let (mut min, mut best) = (min, None);
        let en = &w.entries[e];
        for c in en.load_count..en.dir.n {
            let s = self.pool_1ca978(w, e, c);
            if min <= s {
                min = s;
                best = Some((e, c));
            }
        }
        best
    }
}

/// TAKEN BY MURRAY: REPLACED hand height of a carried object above its carrier (bone frames not modelled)
/// REPLACED: flight life (s) without a hit
pub const SO_FLY_MAX: f32 = 3.0;
pub const SO_HAND_Z: f32 = 150.0;
/// REPLACED: smallest bounding radius of a thrown object
pub const SO_R_MIN: f32 = 20.0;

impl ClassHost for EngineHost {
    fn sw_focus_po(&mut self) -> u32 {
        self.po.as_ref().map_or(0, |p| p.obj as u32 + 1)
    }
    fn atk_msg18(&mut self, guard: usize, cands: &[(usize, i32)]) -> Option<usize> {
        // BOSS R32: no 0x18 handler -> FUN_001331f8 is a no-op and the default reply picks the first record in the
        // same poll (R31 deferred every guard by a flush, which broke the hub guards' attacks)
        if !self.atk_msg18_hdl.contains(&guard) {
            self.atk_msg18_q.push((guard, cands.to_vec()));
            return cands.first().map(|c| c.0);
        }
        // REPLACED (no synchronous VM path): the handler runs in the frame's script flush; this poll uses the
        // answer of the last flush for the same candidate list, and asks again
        let ans = self.atk_msg18_ans.remove(&guard).filter(|a| a.0 == cands).map(|a| a.1);
        self.atk_msg18_q.push((guard, cands.to_vec()));
        ans.flatten()
    }
    fn so_carry(&mut self, x: u32, carrier: u32) {
        if carrier == 0 {
            self.so_carry.remove(&x);
        } else {
            self.so_fly.remove(&x);
            self.so_carry.insert(x, carrier);
        }
    }
    fn vcall_11c_v(&mut self, obj: u32, v: [f32; 3]) {
        // TAKEN BY MURRAY: (vt+0x11c)(x, v) on a released object: start its SO flight from where it is
        let pos = self.ee_sub_1a62e8(obj);
        let g = self.ee_sub_1f5860(obj);
        self.so_carry.remove(&obj);
        self.so_fly.insert(obj, SoFly { pos, vel: v, g, kind: 0, age: 0.0 });
    }
    fn so_fly_step(&mut self, dt: f32) -> Vec<(u32, [f32; 3], u32)> {
        let mut out = Vec::new();
        self.so_fly.retain(|&x, b| {
            for i in 0..3 {
                b.vel[i] += b.g[i] * dt;
                b.pos[i] += b.vel[i] * dt;
            }
            b.age += dt;
            out.push((x, b.pos, b.kind));
            b.age < SO_FLY_MAX
        });
        out
    }
    /// CRAWL R95: the hand position of a carried SO object (`so_carry`), None when not carried
    fn so_carried_pos(&mut self, x: u32) -> Option<[f32; 3]> {
        let c = *self.so_carry.get(&x)?;
        let p = self.ee_sub_1a62e8(c);
        Some([p[0], p[1], p[2] + SO_HAND_Z])
    }
    fn so_land(&mut self, x: u32) {
        self.so_fly.remove(&x);
    }
    fn so_bsphere(&mut self, obj: u32) -> Option<([f32; 3], f32)> {
        // REPLACED: the sphere around the AABB of the object's world-space collision parts
        self.k7_cache()?;
        let so = self.so.clone()?;
        let oi = (obj as usize).checked_sub(1)?;
        let mut vs = so.colliders.iter().filter(|c| c.object == oi).flat_map(|c| c.geom.verts.iter().copied()).peekable();
        let first = *vs.peek()?;
        let (lo, hi) = vs.fold((first, first), |(lo, hi), v| ([0, 1, 2].map(|k| lo[k].min(v[k])), [0, 1, 2].map(|k| hi[k].max(v[k]))));
        let c = [0, 1, 2].map(|k| (lo[k] + hi[k]) * 0.5);
        Some((c, ((hi[0] - c[0]).powi(2) + (hi[1] - c[1]).powi(2) + (hi[2] - c[2]).powi(2)).sqrt()))
    }
    fn so_touch(&mut self, x: u32, p: [f32; 3], other: u32) -> Option<bool> {
        // TAKEN BY MURRAY: FUN_00133a38 (AABB overlap of the two SOs) + FUN_001a7c18 narrowphase on the SO
        // collision parts (`CollisionWorld::build_so`, world space). REPLACED: the moving object is its
        // collision parts' bounding sphere (FUN_001a93c8 sphere vs mesh) at its flight position.
        self.k7_cache()?;
        let so = self.so.clone()?;
        let (xi, oi) = ((x as usize).checked_sub(1)?, (other as usize).checked_sub(1)?);
        let r = *self.so_radius.entry(xi).or_insert_with(|| {
            let vs: Vec<[f32; 3]> = so.colliders.iter().filter(|c| c.object == xi).flat_map(|c| c.geom.verts.iter().copied()).collect();
            if vs.is_empty() {
                return SO_R_MIN;
            }
            let n = vs.len() as f32;
            let c = [0, 1, 2].map(|k| vs.iter().map(|v| v[k]).sum::<f32>() / n);
            vs.iter().map(|v| ((v[0] - c[0]).powi(2) + (v[1] - c[1]).powi(2) + (v[2] - c[2]).powi(2)).sqrt()).fold(SO_R_MIN, f32::max)
        });
        let mut any = false;
        for c in so.colliders.iter().filter(|c| c.object == oi) {
            if c.geom.verts.is_empty() || c.bsp.nodes.is_empty() {
                continue;
            }
            any = true;
            let (mut lo, mut hi) = (c.geom.verts[0], c.geom.verts[0]);
            for v in &c.geom.verts {
                for k in 0..3 {
                    lo[k] = lo[k].min(v[k]);
                    hi[k] = hi[k].max(v[k]);
                }
            }
            if (0..3).any(|k| p[k] + r < lo[k] || p[k] - r > hi[k]) {
                continue;
            }
            let mut out = Vec::new();
            if crate::engine::physics::sphere::FUN_001a93c8(r, p, &c.geom, &c.bsp, 0x10, &mut out) != 0 {
                return Some(true);
            }
        }
        any.then_some(false)
    }
    fn g_2dfbc0(&mut self) -> u32 {
        self.g_2dfbc0
    }
    fn ee_sub_1357c0(&mut self, obj: usize, k: u32) {
        self.pool_fire.push((obj, k));
    }
    fn so_contacts_57(&mut self, obj: usize) -> Vec<super::k15_101::Contact57> {
        self.pool_contacts.remove(&obj).unwrap_or_default()
    }
    fn ee_sub_149ab8(&mut self, obj: usize, pmf: u32, arg: i32) {
        // FEEL R35: only the C2d5dc8 slot-22 node is ported (FUN_00149ab8 skips an identical node)
        if pmf == super::c2d5dc8::PMF_2A77C0 && arg == 0 && !self.spawn_q.contains(&obj) {
            // POOL1 R99: FUN_001a5478 (vt +0x44) calls slot 23 only when FUN_001a61a0(obj) (root in the proximity
            // set), else FUN_00148300 queues it and FUN_00147f40 calls slot 23 on entering the set (vt +0xbc).
            // rec5 001 (row 2, clk 31.6): 1 of the 8 class-7 spawners of pool 68179 owns a record (138403, the last
            // copy), the port's load claim of all 8 put all 6 copies in the world. With the set on, the load claim
            // waits for the set (`sp_watch_frame`). Opt-out SLY2_R99_NO_SPWATCH.
            if !super::sw_prox::off() && std::env::var_os("SLY2_R99_NO_SPWATCH").is_none() {
                if !self.sp_watch.iter().any(|x| x.0 == obj) {
                    self.sp_watch.push((obj, false));
                }
                return;
            }
            self.spawn_q.push(obj);
        }
    }
    fn ee_sub_1cade0(&mut self, host: u32, k: i32) -> bool {
        let Some(w) = self.world.clone() else { return false };
        self.pool_1cade0(&w, host, k).is_some()
    }
    /// FUN_001cb250(spawner, alt): single-template spawners: FUN_001caec8 claims a copy (owner, +0xe8 head,
    /// vtable +0x44 add) -> its object (FUN_001ca930 -> entry +4 [copy])
    fn ee_sub_1cb250(&mut self, host: u32, k: i32) -> u32 {
        let Some(w) = self.world.clone() else { return 0 };
        let Some(e) = (host as usize).checked_sub(1).and_then(|i| Self::sp_entry(&w, i)) else { return 0 };
        if w.entries[e].objects.len() != 1 {
            return 0;
        }
        let Some((e, copy)) = self.pool_1cade0(&w, host, if k != 0 { 1 } else { 2 }) else { return 0 };
        let owned = self.pool_owner.insert((e, copy), host);
        if let Some(old) = owned {
            if old != host {
                self.pool_cur.remove(&old);
            }
        }
        self.pool_cur.insert(host, (e, copy));
        let Some(&o) = w.entries[e].instances.get(copy as usize) else { return 0 };
        let c = o as u32 + 1;
        // POOL1 R59: FUN_001caec8 slot already owned (*slot != 0, any owner) -> FUN_001caa60(old owner, slot):
        // unlink + identity matrix, then per template copy vt +0x4c (remove from the world) before the re-claim
        // re-adds it (REPLACED: its vt +0xdc / msg 0xf / vt +0x12c / FUN_001a58d0 are not modelled here)
        // (the vt +0x44 below in the same call cancels the deferred pool removal, see `vcall`)
        if owned.is_some() {
            self.vcall(c, 0x4c);
        }
        self.pool_live.insert(c);
        // POOL1 R80: vt +0x54(copy, sp +0x10, sp +0x50, 0): the copy becomes a child of the record's owner
        crate::engine::classes::ClassHost::k21_pool_parent(self, o, host);
        // HEIST r36: a re-claimed copy is not fading (+0x130 back to 0; REPLACED: the vt +0x12c fade-in)
        self.obj_set_u32(c, 0x130, 0);
        // vtable +0x44 (add to world): +0x28 bit 60 -> the copy's class frame (K15 class 101 / K23 102) runs
        let hi = self.obj_i32(c, 0x2c) as u32 | 1 << 28;
        self.obj_set_u32(c, 0x2c, hi);
        self.vcall(c, 0x44);
        self.pool_add.push(c);
        // FUN_001caec8: the copy starts at the spawner's frame (+0x60 transform copied)
        self.pool_place.insert(c);
        c
    }
    fn pool_take_place(&mut self, copy: u32) -> bool {
        self.pool_place.remove(&copy)
    }
    fn ee_sub_1cb488(&mut self, _host: u32, obj: u32) {
        self.pool_attach.insert(obj, 1);
    }
    /// FUN_001333c0(carried, carrier) on a pool copy: held by the carrier (its hand frame = the spawner's, as
    /// the FUN_001cb488 attachment) until FUN_00133448 / FUN_001cb4b8 / FUN_002205a8 releases it
    fn ee_sub_1333c0_carry(&mut self, carried: u32, _carrier: u32) {
        if self.pool_live.contains(&carried) {
            self.pool_attach.insert(carried, 1);
        }
    }
    fn ee_sub_133448_carry(&mut self, carried: u32, _carrier: u32) {
        if self.pool_live.contains(&carried) {
            self.pool_attach.insert(carried, 0);
        }
    }
    fn ee_sub_1cb4b8(&mut self, _host: u32, obj: u32) {
        self.pool_attach.insert(obj, 0);
    }
    fn so_set_v_124(&mut self, obj: u32, v: [f32; 3]) {
        if self.pool_live.contains(&obj) {
            self.pool_v.push((obj, v));
        }
        // TAKEN BY HEIST: vt +0x124 (PO velocity) of a Bentley / Murray: its K27 body
        if let Some(p) = (obj as usize).checked_sub(1).and_then(|o| self.k27_po.get_mut(&o)) {
            p.phys.v = v;
        }
    }
    fn ee_sub_136530(&mut self, obj: usize, s: i32) {
        // REPLACED: the mounted Bentley is parented to Murray's +0x191c point (oid 481) in the original and
        // FUN_001a58d0(b, 0, 0xf) keeps his world xf; the port does not parent him, so his K27 body is put at
        // that point (its load offset from Murray, carried by Murray's body) before he flies
        if s == 1 {
            if let Some(w) = self.world.clone() {
                if let Some(o2) = w.objects.iter().position(|o| o.class == 54) {
                    let x = w.objects[o2].children.iter().copied().chain(w.objects.iter().enumerate().filter(|(_, o)| o.owner == Some(o2)).map(|(i, _)| i)).find(|&i| w.objects[i].oid == 481);
                    if let Some(x) = x {
                        let (xl, ml) = (self.k24_world_xf(x as u32 + 1).map_or([0.0; 3], |m| m[3]), self.k24_world_xf(o2 as u32 + 1).map_or([0.0; 3], |m| m[3]));
                        let mp = ClassHost::ee_sub_1a62e8(self, o2 as u32 + 1);
                        let pos = [0, 1, 2].map(|k| mp[k] + xl[k] - ml[k]);
                        let e = self.k27_po.entry(obj).or_insert_with(|| super::k27::CharPo { phys: crate::engine::physics::driver::AloPhys { kind: 2, ..Default::default() }, ..Default::default() });
                        e.phys.pos = pos;
                        e.phys.gravity = crate::engine::player::p22::CHAR_GRAVITY;
                    }
                }
            }
        }
        self.q_136530.push((obj, s));
    }
    fn take_136530(&mut self) -> Vec<(usize, i32)> {
        std::mem::take(&mut self.q_136530)
    }
    /// FUN_00224590: 0 < +0x150 < 3
    fn ee_sub_224590(&mut self, obj: u32) -> bool {
        obj != 0 && (1..3).contains(&self.obj_i32(obj, 0x150))
    }
    /// FUN_00224610: +0x15c is 7 or 0xb
    fn ee_sub_224610(&mut self, obj: u32) -> bool {
        obj != 0 && matches!(self.obj_i32(obj, 0x15c), 7 | 0xb)
    }
    fn ee_sub_156b98(&mut self, obj: usize, pos: [f32; 3]) {
        self.noises.push((obj as u32 + 1, pos));
    }
    /// WP-J17: FUN_001a6f00(obj, pos, 0): world point -> obj-local (inverse of the FUN_001a62e8 world matrix)
    fn ee_sub_1a6f00(&mut self, obj: usize, pos: [f32; 3]) -> [f32; 3] {
        let Some(m) = self.k24_world_xf(obj as u32 + 1) else { return pos };
        let m = crate::engine::gfx::scene::invert(&m);
        [0, 1, 2].map(|k| pos[0] * m[0][k] + pos[1] * m[1][k] + pos[2] * m[2][k] + m[3][k])
    }
    /// WP-J17: FUN_00118fb0(+0xec, 0x7fffffff, local pos) != 0 of the class-26 warp volume (LoadData::L2282b0 solid)
    fn vol_contains(&mut self, obj: usize, pos: [f32; 3]) -> bool {
        crate::engine::game::methods::port2::ee_sub_104718(self.world.as_deref(), obj as u32, pos)
    }
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
    /// K23 / WIRE-OBJ: FUN_001c6fd8(0x3dd490, kind, pos, src, k) -> ClassSet::k8_drop
    fn k23_world_1c6fd8(&mut self, kind: i32, pos: [f32; 3], src: u32, k: i32) {
        self.k8_drop_req.push((kind, pos, src, k));
    }
    fn k8_take_drops(&mut self) -> Vec<(i32, [f32; 3], u32, i32)> {
        std::mem::take(&mut self.k8_drop_req)
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
    fn ee_sub_1a6ca8(&mut self, obj: usize, a: i32) {
        self.sw_set_mode(obj, a);
    }
    fn sw_active(&mut self, obj: usize) -> bool {
        EngineHost::sw_active(self, obj)
    }
    fn sw_take_masks(&mut self) -> Vec<(usize, u8)> {
        EngineHost::sw_take_masks(self)
    }
    fn sw_frame(&mut self, cam: [f32; 3]) -> Option<u8> {
        self.world.is_some().then(|| {
            let foc = self.player_valid.then_some(self.player_pos);
            EngineHost::sw_frame(self, cam, foc)
        })
    }
    /// K22: FUN_001a61a0 - +0x28 bit 60 (in the world) is implied: members are added right after vt+0x44
    /// POOL1 R96: 0x3dd4bc is 0x3dd490 +0x2c, the R64 proximity active set (FUN_00147f40 adds the near SW roots,
    /// FUN_0011a220 at text_00140000.c 5524 / text_001b0000.c 11154 the forced ones): with R64 on, a root in
    /// `sw.list.active` is a member too. FUN_001ca978 scores such an in-world spawner copy 1 (never recycled);
    /// rec_ep1d hub guard 44949 copies in the world are in it. Opt-out SLY2_R96_NO_PROX61A0.
    fn ee_sub_1a61a0(&mut self, obj: u32) -> bool {
        if self.set_3dd4bc.contains(&obj) {
            return true;
        }
        // ANIM-ALL R92: FUN_001a61a0 tests the object's own +0x28 bit 60 (in the world) before the root lookup
        // (FUN_0011a380(0x3dd4bc, +0x14)): an out-of-world copy under an active root is not a member.
        // Opt-out SLY2_R92_NO_61A0WORLD.
        obj != 0
            && !super::sw_prox::off()
            && self.sw.built
            && std::env::var_os("SLY2_R96_NO_PROX61A0").is_none()
            && (std::env::var_os("SLY2_R92_NO_61A0WORLD").is_some() || !self.lo_out_of_world(obj as usize - 1))
            && EngineHost::sw_active(self, obj as usize - 1)
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
    fn live_vcall(&mut self, off: u32, obj: u32, arg: u32) {
        self.live_calls.push((off, obj, arg));
    }
    fn k22_take_po_switches(&mut self) -> Vec<u32> {
        let mut v = std::mem::take(&mut self.k22_po_switches);
        // P22: Sly has no k18 slot 78 here (his JT is the player layer's): released through the player API
        if let Some(w) = self.world.clone() {
            v.retain(|&po| {
                let sly = (po as usize).checked_sub(1).and_then(|o| w.objects.get(o)).is_some_and(|x| x.class == crate::engine::player::p20::CLASS_SLY);
                self.po_release |= sly;
                self.sly_focus_req |= sly;
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
    fn k22_jt_live(&mut self) -> bool {
        self.jt_live
    }
    /// CRAWL R122: the NPC's joy from FUN_001e46a0 case 0x24 (walkto::ee_sub_1e46a0_24, magnitude m <= 1) drives
    /// its PO walk: Murray's walk sub-state (K20 FUN_001b5380 state 1 -> FUN_0020bb58 blend by m) moves at the walk
    /// anim root speed. REPLACED: the NPC PO update / anim root motion = a straight step of m * NPC_WALK_V per frame
    /// (800 = Murray's anim root speed, FEEL R50), z interpolated to the goal; no collision.
    fn k22_npc_walk_1e46a0(&mut self, subj: u32, goal: [f32; 3], speed: f32) {
        const NPC_WALK_V: f32 = 800.0;
        // CRAWL R123: the player-controlled PO (e.g. Bentley under po_set 8 in heist_tb_1896_pad) is walked by its
        // live JT (k22_walkto_1448f0 drain), not as an NPC: the R122 step overrode its FUN_001a62e8 position.
        if self.po.as_ref().is_some_and(|p| p.obj as u32 + 1 == subj) {
            return;
        }
        let p = self.ee_sub_1a62e8(subj);
        let m = crate::engine::player::walkto::ee_sub_1e46a0_24(p, goal, speed);
        let m = (m[0] * m[0] + m[1] * m[1]).sqrt();
        let d = ((goal[0] - p[0]).powi(2) + (goal[1] - p[1]).powi(2)).sqrt();
        let step = (m * NPC_WALK_V / 60.0).min(d);
        let f = if d > 1e-4 { step / d } else { 1.0 };
        let q = [p[0] + (goal[0] - p[0]) * f, p[1] + (goal[1] - p[1]) * f, p[2] + (goal[2] - p[2]) * f];
        self.npc_pos.insert(subj, q);
    }
    fn k22_walkto_1448f0(&mut self, subj: u32, pos: [f32; 3], speed: f32) {
        self.k22_walktos.retain(|w| w.0 != subj);
        self.k22_walktos.push((subj, pos, speed));
    }
    fn k22_walk_r1_1e46a0(&mut self, subj: u32) {
        if !self.k22_walk_r1.contains(&subj) {
            self.k22_walk_r1.push(subj);
        }
    }
    fn k22_autowalk_1938c0(&mut self, subj: u32, mask: u32, target: u32) {
        if self.k22_autowalks.len() < 64 {
            self.k22_autowalks.push((subj, mask, target));
        }
    }
    fn k22_hop_1e1e30_2a(&mut self, subj: u32, target: u32, rec5: u32) {
        if self.k22_hops.len() < 64 {
            self.k22_hops.push((subj, target, rec5));
        }
    }
    fn k22_launch_1e1e30_2b(&mut self, subj: u32, target: u32, f12: f32, f13: f32) {
        if std::env::var_os("SLY2_K22DBG").is_some() {
            eprintln!("K22 0x2b launch subj {subj} target {target} f12 {f12} f13 {f13}");
        }
        if self.k22_launches.len() < 64 {
            self.k22_launches.push((subj, target, f12, f13));
        }
    }
    fn k22_on_path_1a(&mut self, subj: u32) -> Option<bool> {
        let sly = subj == super::guard::PLAYER_HANDLE || self.world.as_deref().and_then(|w| w.objects.iter().position(|o| o.class == 53)).is_some_and(|k| k as u32 + 1 == subj);
        (self.hop_drainer && sly).then(|| self.player_jts == 8)
    }
    fn k22_climb_1a(&mut self, subj: u32, pos: [f32; 3]) -> Option<f32> {
        if !self.hop_drainer || self.player_jts != 8 {
            return None;
        }
        self.k22_climbs.retain(|w| w.0 != subj);
        self.k22_climbs.push((subj, pos));
        Some(self.k22_climb_left.unwrap_or(f32::MAX))
    }
    fn k22_hop_done_2a(&mut self, subj: u32) -> Option<bool> {
        let sly = subj == super::guard::PLAYER_HANDLE || self.world.as_deref().and_then(|w| w.objects.iter().position(|o| o.class == 53)).is_some_and(|k| k as u32 + 1 == subj);
        (self.hop_drainer && sly).then(|| !self.k22_hops.iter().any(|h| h.0 == subj) && self.player_jtbs == -1)
    }
    fn k22_teleport_1e1e30_16(&mut self, subj: u32, loc: u32, rec5: u32) {
        // CRAWL R122: an NPC subject (Murray 0x36 / Bentley 0x37) takes the locator position (FUN_001e4d40 pos ->
        // SetPosition); the player drain below handles Sly. Opt-out SLY2_NO_R122_NPCWALK=1.
        if std::env::var_os("SLY2_NO_R122_NPCWALK").is_none() && subj != super::guard::PLAYER_HANDLE && !self.po.as_ref().is_some_and(|p| p.obj as u32 + 1 == subj) && (self.ee_sub_132b10(subj, 0x36) != 0 || self.ee_sub_132b10(subj, 0x37) != 0) {
            let p = self.ee_sub_1a62e8(loc);
            self.npc_pos.insert(subj, p);
        }
        if self.k22_teleports.len() < 64 {
            self.k22_teleports.push((subj, loc, rec5));
        }
    }
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
        // ANIM-ALL R42: FUN_0015e938(world, 1, oid, jt) searches under the JT object itself; PLAYER_HANDLE is not an
        // object index (mode 1 found nothing from it), so the context is the class-53 object
        let ctx = self.world.as_deref().and_then(crate::engine::game::spawn::player_object).map_or(subj, |i| i as u32 + 1);
        let o = self.ee_sub_15e938(1, oid, ctx);
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
    /// WIRE-MG: obj +0x150 tracker +0x10 / +0x14 / +0x20 / +0x24 = joystick 0x3dd490 left / right sticks
    fn k22_stick(&mut self, obj: u32) -> Option<[f32; 4]> {
        if self.po_focus != Some(obj) {
            return None;
        }
        let p = self.pad.as_ref()?;
        Some([p.left.x, p.left.y, p.right.x, p.right.y])
    }
    fn k9_stick(&mut self, obj: u32) -> [f32; 2] {
        self.k22_stick(obj).map_or([0.0; 2], |s| [s[0], s[1]])
    }
    /// WIRE-MG: FUN_001c46c0(0.0, +0x150, mask): pressed bits of `mask` on the tracker (consumed)
    fn k9_pressed(&mut self, obj: u32, mask: u32) -> u32 {
        if self.po_focus != Some(obj) {
            return 0;
        }
        let Some(p) = self.pad.as_mut() else { return 0 };
        let r = p.pressed & mask;
        p.pressed &= !r;
        r
    }
    fn vcall_arg(&mut self, obj: u32, off: u32, a: i32) {
        if off == 0x27c {
            self.p23_switch.slot78.push((obj, a));
        }
        if off == 0x264 && self.pool_live.contains(&obj) {
            self.pool_264.push((obj, a));
        }
    }
    /// P23: FUN_002206c8 (carry machine SetState) -> player::p23::apply_switches
    fn po_part_s(&mut self, addr: u32, obj: usize, off: u32, s: i32) {
        if addr == 0x002206c8 {
            self.p23_switch.carry.push((obj, off, s));
        }
    }
    fn po_jtbs(&self, obj: usize) -> Option<i32> {
        self.po.as_ref().filter(|p| p.obj == obj).map(|p| p.jt.jtbs)
    }
    fn po_call(&mut self, addr: u32, obj: usize, off: u32, args: &[u32]) -> i64 {
        // TAKEN BY MURRAY: FUN_001f7a68(x, 2 / 3): the thrown object's hit-part words (+0x18 = 0x10000 / 0)
        if addr == 0x1f7a68 && args.len() >= 2 {
            if let Some(b) = self.so_fly.get_mut(&args[0]) {
                b.kind = if args[1] == 2 { 0x10000 } else { 0 };
            }
        }
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
        self.po.as_mut().map_or(0, |p| p.po_call(addr, obj, off, args))
    }
    /// K18: LO +0x28 bit 60 (in the world): every loaded world object is (no run-time removal yet)
    fn lo_28_bit60(&mut self, obj: u32) -> bool {
        // FEEL R33: one in-world check with the update-list gate (`lo_removed`: scripts / job toggles / load-time Remove)
        let Some(i) = (obj as usize).checked_sub(1).filter(|&i| self.world.as_ref().is_some_and(|w| i < w.objects.len())) else { return false };
        !self.lo_removed(i)
    }
    fn ee_sub_20ff70(&mut self, _obj: usize, entries: &[[u32; 3]]) {
        let Some(w) = self.world.clone() else { return };
        if std::env::var_os("SLY2_R83IDLE").is_some() {
            eprintln!("R83 20ff70 call obj {_obj} entries {:?}", entries.iter().map(|e| [e[0] as i32, e[1] as i32, e[2] as i32]).collect::<Vec<_>>());
        }
        for e in entries {
            // FUN_0020fff0(.., e, obj, flags 1, 0, 0): FUN_0015e650 oid -> is-a 0x30 -> state e[2] (-1 = none)
            let Some(&k) = w.find_oid(e[0] as i32).first() else { continue };
            if !crate::engine::world::class::ee_sub_15e988(w.objects[k].class, 0x30) || e[2] as i32 == -1 {
                continue;
            }
            self.sm_states.push((k, e[1], e[2] as i32));
            // ANIM-ALL R83: the drain was missing (sm_states had no reader): FUN_0020fff0(.., e, obj, 1, 0, 0) sets the
            // ALO's live SMA of SM e[1] to state e[2] (FUN_001f0660). Safehouse table: Bentley / Murray SM 419 seated
            // idle goal (PCSX2 cap_table_seated SMA +0x3c goal 2). Opt-out SLY2_NO_R83_20FF70=1.
            if std::env::var_os("SLY2_NO_R83_20FF70").is_none() {
                let r = self.k22_sma_20fff0(0, &[e[0], e[1], e[2], e[2]], 1, 0);
                if std::env::var_os("SLY2_R83IDLE").is_some() {
                    eprintln!("R83 20ff70 obj {k} oid {} sm {} label {} -> {r:#x}", w.objects[k].oid, e[1] as i32, e[2] as i32);
                }
            }
        }
    }
    /// CRAWL R50: FUN_0020fff0(world, entry {oid, sm oid, label (mode & 1), label (else)}, ctx, mode, owner, &out)
    /// 1:1 (text_00200000.c:12063): FUN_0015e650 oid lookup (first match), is-a 0x30, FUN_001f67e0 the ALO's live
    /// SMA of that SM; label != -1: owner != 0 -> out = sma (vtable +0xc and the FUN_001333c0 owner listener are
    /// REPLACED: the k22 0xd wait polls FUN_001f07e8 instead of message 10), then mode & 2 ? FUN_001f0948 (instant)
    /// : FUN_001f0660. Returns out (0 = no SMA).
    fn k22_sma_20fff0(&mut self, _ctx: u32, entry: &[u32; 4], mode: u32, owner: u32) -> u32 {
        let Some(w) = self.world.clone() else { return 0 };
        let Some(&k) = w.find_oid(entry[0] as i32).first() else { return 0 };
        if !crate::engine::world::class::ee_sub_15e988(w.objects[k].class, 0x30) {
            return 0;
        }
        let s = self.k25.ee_sub_1f67e0(&w, k as u32 + 1, entry[1] as i32);
        if std::env::var_os("SLY2_K22DBG").is_some() {
            eprintln!("[R50] 20fff0 obj {k} sm {:#x} -> sma {s:#x} mode {mode} entry {entry:x?}", entry[1]);
        }
        if s == 0 {
            return 0;
        }
        let label = if mode & 1 == 0 { entry[3] } else { entry[2] } as i32;
        if label == -1 {
            return 0;
        }
        let before = self.k25.ee_sub_1f07e8(s);
        let out = if owner != 0 { s } else { 0 };
        // CRAWL R77: FUN_001333c0(sma, owner node) before the goal is set: the node hears the SMA's message 10
        if owner != 0 && std::env::var_os("SLY2_R77_NO_MSG10").is_none() {
            self.k25.k22_msg10.retain(|x| x.0 != s);
            self.k25.k22_listen.insert(s);
        }
        if mode & 2 == 0 {
            self.k25.ee_sub_1f0660(&w, s, label);
        } else {
            self.k25.ee_sub_1f0948(&w, s, label);
        }
        if std::env::var_os("SLY2_K22DBG").is_some() {
            eprintln!("[R50] 20fff0 sma {s:#x} label {before:#x} -> {:#x} (goal {label:#x})", self.k25.ee_sub_1f07e8(s));
        }
        out
    }
    fn k22_sma_msg10(&mut self, sma: u32, label: i32) -> bool {
        if !self.k25.k22_msg10.contains(&(sma, label)) {
            return false;
        }
        self.k25.k22_msg10.retain(|x| x.0 != sma);
        self.k25.k22_listen.remove(&sma);
        true
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
    fn task_state_54(&mut self, obj: u32) -> i32 {
        self.task_states.get(&obj).copied().unwrap_or(-1)
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
    /// FUN_001b7000(murray, point): queued for Murray's class frame (classes::k18_55 frame_k18_55)
    fn ee_sub_1b7000(&mut self, user: u32, point: u32) {
        self.murray_1b7000.push((user, point));
    }
    fn take_1b7000(&mut self) -> Vec<(u32, u32)> {
        std::mem::take(&mut self.murray_1b7000)
    }
    fn ee_sub_15e9c8_505(&mut self, class: i32, max: usize) -> Vec<u32> {
        let Some(w) = &self.world else { return Vec::new() };
        crate::engine::game::methods::find::ee_sub_15e9c8(w, 0x505, class, 0, max).into_iter().map(|i| i + 1).collect()
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
            // ANIM-ALL R34: no fallback when FUN_0015e3e8's scope check rejects every candidate (1:1 0; e.g. Sly's
            // SM 1550 for Bentley's FUN_00205478 kind 2); R36: both buckets (own +0x3c / alias +4) as FUN_0015e678
            let chain = crate::engine::game::methods::find::scope_chain(w, ctx - 1);
            if (kind as u32) & 0x400 == 0 && !crate::engine::game::methods::find::any_in_scope(w, &chain, oid) {
                return 0;
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
        // R53: vt +0x4c on a live pool copy (e.g. the placed bomb's SM 7 -> 8 retract event kind 8 =
        // FUN_00210338(0, off)): it leaves the world now (the faded-out path, player/plugin.rs, at t 0)
        // (a later vt +0x44 in the same frame puts it back: e.g. a dart copy re-added at its launch)
        if off == 0x4c && self.pool_live.contains(&obj) {
            if crate::engine::classes::ClassHost::obj_i32(self, obj, 0x2c) as u32 & 1 << 28 != 0 {
                self.pool_off_in.insert(obj);
            }
            self.pool_off.insert(obj);
        } else if off == 0x44 {
            self.pool_off.remove(&obj);
            self.pool_off_in.remove(&obj);
        }
        // FEEL R34: vtable +0x44 FUN_001a5478 / +0x4c FUN_001a5630 = LO +0x28 bit 60 of the object's subtree
        if off == 0x44 || off == 0x4c {
            self.live_calls.push((off, obj, 0));
            if let Some(i) = (obj as usize).checked_sub(1) {
                self.lo_rt(i, off == 0x44);
            }
        }
    }
    fn lo_rt(&mut self, obj: usize, on: bool) {
        let Some(w) = self.world.clone().filter(|w| obj < w.objects.len()) else { return };
        // POOL1 R80: FUN_001a5478 (Add): under a parent that is out of the world (+0x28 bit 60 clear) the
        // vt +0x5c world add is skipped (the parent's later Add brings the subtree in)
        if on {
            if let Some(&p) = self.pool_parent.get(&obj) {
                if crate::engine::classes::ClassHost::lo_removed(self, p) {
                    return;
                }
            }
        }
        // POOL1 R82: a class-code Add (vt +0x44) after a script Remove of the same object (vm_state method
        // 0x100a90 path -> `lo_world_link`, which `sync_in_world` never sees as a diff): the Add relinks it too
        // (the object itself: FUN_001a5478 re-adds a child only when its own bit 56 is set, as set_in_world_lo)
        if on && self.laser_off.contains(&obj) {
            self.lo_world_link(obj, true);
        }
        // POOL1 R99: LO +0x28 bit 56 = removed on its own (on the parent's removed-child list +0x70): FUN_001a5478
        // (Add) clears it on `obj` and vt +0x5c (FUN_001a5800 -> vt +0x24) walks only the in-world child lists
        // (+0x64), so an Add skips a child that is removed on its own, with its subtree (rec11 005: hub guard
        // 11539 in the world, its dazed-birds child 13144 b56 1 out). Opt-in SLY2_R99_B56=1 (default off: with it feel_inworld_live_saved_matches_original_ep1_hub and wired_guards hear_search / sly_knocks_out fail, POOL1.md R99).
        // CRAWL R109: default on with the class-44 own-field-26 load bit (fields::b56_skips; opt-out SLY2_NO_R109_B56OWN=1)
        let b56_on = std::env::var_os("SLY2_R99_B56").is_some() || std::env::var_os("SLY2_NO_R109_B56OWN").is_none();
        self.lo_b56.insert(obj, !on);
        let mut st = vec![obj];
        while let Some(r) = st.pop() {
            let sub = if on && b56_on {
                let mut out = vec![];
                let mut s2 = vec![r];
                while let Some(i) = s2.pop() {
                    out.push(i);
                    s2.extend(w.objects[i].children.iter().rev().copied().filter(|&c| !crate::engine::game::fields::b56_skips(&w, c, self.lo_b56.get(&c).copied())));
                }
                out
            } else {
                crate::engine::playerdraw::subtree(&w, r)
            };
            for i in sub {
                self.rt_overlay.insert(i, on);
                // POOL1 R80: and the claimed pool copies parented to it (vt +0x54 of FUN_001caec8)
                let mut kids: Vec<usize> = self.pool_parent.iter().filter(|&(&c, &p)| p == i && c != r).map(|(&c, _)| c).collect();
                kids.sort_unstable();
                for c in kids {
                    if self.rt_overlay.get(&c) != Some(&on) {
                        self.rt_queue.push((c, on));
                        st.push(c);
                    }
                }
            }
        }
        self.rt_queue.push((obj, on));
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
    fn k26_take_msg9(&mut self) -> Vec<(u32, [u32; 3])> {
        std::mem::take(&mut self.k26_msg9)
    }
    fn k26_handled(&mut self, n: usize) {
        self.k26.handled += n;
    }
    /// K27: FUN_001f7bf8(fade, obj) fade-out: REPLACED by alpha +0x12c = 0 at once
    fn ee_sub_1f7bf8(&mut self, obj: usize, rate: f32) {
        self.k27_fades += 1;
        self.obj_set_u32(obj as u32 + 1, 0x12c, 0);
        // HEIST r36: FUN_001f7d90 rate +0x130 = -1 / fade (0 for fade <= 0): FUN_00144640 skips a fading guard
        self.obj_set_u32(obj as u32 + 1, 0x130, if rate > 0.0 { (-1.0 / rate).to_bits() } else { 0 });
        if self.pool_live.contains(&(obj as u32 + 1)) {
            self.pool_fade.insert(obj as u32 + 1, rate.max(0.0));
        }
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
    /// HEIST r37: FUN_0020f0b0 world +0x3d78 = the FUN_0020efc0 / FUN_0020f030 count (k27_cine[0])
    fn world_3d78(&mut self) -> i32 {
        self.k27_cine[0]
    }
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
            (Some(a), 0x78) => a.f78(),
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
    /// CRAWL R50: class-111 slot 14 FUN_001f00c8 -> FUN_001f0380(sm, obj, label, flags) = FUN_001f01b8(sm, obj,
    /// label, flags, mask 4): the SM's +0x74 (prop 370) SMA on its target ALO (e.g. the row-6 balcony door 128381's
    /// SM 0x441a). Before R50 this hook was the trait no-op, so no prop-370 SMA existed and FUN_0020fff0 found none.
    fn ee_sub_1f0380(&mut self, obj: usize, target: u32, a: u32, b: u32) {
        let Some(w) = self.world.clone() else { return };
        if target == 0 {
            return;
        }
        let h = self.k25.ee_sub_1f01b8(&w, obj as u32 + 1, target, a as i32, b, super::k25::SMA_MASK_1F0380);
        if std::env::var_os("SLY2_K22DBG").is_some() {
            eprintln!("[R50] 1f0380 sm obj {obj} target {target} label {a:#x} flags {b:#x} -> sma {h:#x} state {:#x}", self.k25.ee_sub_1f07e8(h));
        }
    }
    fn ee_sub_1f0380_link(&mut self, sm: u32, obj: u32, a: i32, b: u32) -> u32 {
        let Some(w) = self.world.clone() else { return 0 };
        self.k25.ee_sub_1f01b8(&w, sm, obj, a, b, super::k25::SMA_MASK_1F0380)
    }
    /// ANIM-ALL R33: FUN_001f0598(rec): stop the SMA (FUN_00205588 last disable; was the trait no-op)
    fn ee_sub_1f0598_link(&mut self, rec: u32) {
        self.k25.ee_sub_1f0598(rec);
    }
    /// ANIM-ALL R33: FUN_001f03c8(sm, obj) (was the trait 0: FUN_00205478 / FUN_00205588 found no record)
    fn ee_sub_1f03c8(&mut self, sm: u32, obj: u32) -> u32 {
        let Some(w) = self.world.clone() else { return 0 };
        self.k25.ee_sub_1f03c8(&w, sm, obj)
    }
    /// ANIM-ALL R33: FUN_001f0198(sm, obj, label, flags) = FUN_001f01b8 with mask 4 (EP4 Bentley layer SMs
    /// 1548..1552 via FUN_00205478; was the trait no-op)
    fn ee_sub_1f01b8(&mut self, sm: u32, obj: u32, a: i32, b: u32) -> u32 {
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
    fn ee_sub_1f07e8(&mut self, sma: u32) -> i32 {
        self.k25.ee_sub_1f07e8(sma)
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
    /// POOL1: FUN_001f0730(sma, label) (seek a goal reachable by the SMA's mask)
    fn ee_sub_1f0730(&mut self, snd: u32, id: i32) {
        let Some(w) = self.world.clone() else { return };
        self.k25.ee_sub_1f0730(&w, snd, id);
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
    /// ANIM-ALL R92: (vtable +0xf4)(obj, FUN_001a6318(src)) = set the world transform of a run-time object
    /// (class-29 group from FUN_00159500) to the member's frame (FUN_00159c80 makes the group its own block
    /// source, so its particles start there). Opt-out SLY2_R92_NO_GRPXF (group particles at the origin, R91).
    fn vcall_f4_1a6318(&mut self, obj: usize, src: u32) {
        if src == 0 || std::env::var_os("SLY2_R92_NO_GRPXF").is_some() {
            return;
        }
        let m = if crate::engine::gfx::attach::r81_frame_on() { self.k24_world_xf(src) } else { None };
        let m = m.or_else(|| {
            let w = self.world.clone()?;
            let i = (src as usize).checked_sub(1).filter(|&i| i < w.objects.len())?;
            Some(crate::engine::game::fields::ee_sub_1a6318(&w, i))
        });
        if let Some(m) = m {
            self.rt_xf.insert(obj, m);
        }
    }
    fn ee_sub_15bd20(&mut self, obj: usize, count: i32, t: &[u32; 64], blk: &[u32; 5], link: bool) {
        self.emits += 1;
        self.particles += count as i64;
        // K28: FUN_0015ad48 frame of the block's source object (FUN_001a6318)
        let frame = self.world.clone().and_then(|w| {
            // ANIM-ALL R92: a run-time group is its own source (FUN_00159c80) and sits where vt+0xf4 put it
            if let Some(m) = (blk[0] as usize).checked_sub(1).and_then(|i| self.rt_xf.get(&i)) {
                return Some(*m);
            }
            let i = (blk[0] as usize).checked_sub(1).filter(|&i| i < w.objects.len())?;
            // ANIM-ALL R81: the same world (vt+0xfc) as the emitter's FUN_001a62e8 gate (k24_world_xf: placement,
            // posed owners) instead of the static rest XFM (SLY2_R81FRAMEOFF = old)
            if crate::engine::gfx::attach::r81_frame_on() {
                if let Some(m) = self.k24_world_xf(blk[0]) {
                    return Some(m);
                }
            }
            Some(crate::engine::game::fields::ee_sub_1a6318(&w, i))
        });
        self.k28_emits.push(super::k28_emit::EmitReq { obj, count, t: *t, blk: *blk, link, frame });
        if std::env::var_os("SLY2_R81RNG").is_some() {
            let (oid, cls) = self.world.as_ref().and_then(|w| w.objects.get(obj)).map_or((0, 0), |o| (o.oid, o.class));
            let p = frame.map_or([0.0; 3], |m| m[3]);
            eprintln!("R81EMIT obj {obj} oid {oid} cls {cls} n {count} t1 {} src {} at {:.0} {:.0} {:.0}", t[1], blk[0], p[0], p[1], p[2]);
        }
    }
    /// POOL1 R54: vt+0x19c emit request (receiver stage kind 7 / the slot-50 child forward of FUN_0015c3c0) ->
    /// the emit request queue `k30_reqs`, drained by ClassSet::k28_spawn (class 31 FUN_0015c3c0, class 33
    /// FUN_0015c690; other targets dropped there). The request block (r[13]) is not carried (k28_spawn ignores it).
    fn vcall_19c(&mut self, obj: u32, req: &[u32; 15], _m: &super::c2ce828::ReqBlock) {
        if self.k30_19c.len() >= 64 {
            self.k30_19c.remove(0);
        }
        self.k30_19c.push((obj, req[0], req[1]));
        self.k30_reqs.push((obj, *req));
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
        // MURRAY: an unwritten +0x2c keeps bit 60 (in the world) of every loaded object, as lo_28_bit60
        let hi = self.k25_fld.get(obj, 0x2c).unwrap_or_else(|| if self.lo_28_bit60(obj) { 1 << 28 } else { 0 });
        u64::from(lo.unwrap_or(0)) | u64::from(hi) << 32
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
    fn k27_grounded(&mut self, obj: usize) -> bool {
        // ANIM-ALL R21: FUN_00204768 probes the po's own body; for the controlled Bentley / Murray that is the JT the
        // pad drives (as k27_pos, REPLACED p20): its +0x208 ground contact, not the parked k27 body (which hangs in
        // the air on the EP2/EP3 hubs, so the po stayed in the fall state 2 with no body anim)
        if let Some(po) = self.po.as_ref().filter(|p| p.obj == obj) {
            return po.jt.ground_nz.is_some();
        }
        self.k27_po.get(&obj).is_some_and(|p| p.ground)
    }
    fn ee_sub_22d348(&mut self, alo: u32, p: [f32; 3]) -> [f32; 3] {
        (alo as usize).checked_sub(1).map_or([0.0; 3], |i| self.k25_phys.ee_sub_22d348(i, p))
    }
    fn phys_e8(&mut self, obj: usize) -> Option<super::actor::alo::PhysE8> {
        // REPLACED: +0xdc / +0xa0 of a class-owned body (no angular constraint kind in `AloPhys`: free)
        self.k25_phys.body(obj).map(|b| super::actor::alo::PhysE8 { dc: false, s_a0: 0, s_c0: b.ckind as i32 })
    }
    fn phys_e8_80(&mut self, obj: usize) -> [f32; 3] {
        // TAKEN BY HEIST: a character's K27 body carries its gravity
        if let Some(b) = self.k27_po.get(&obj) {
            return b.phys.gravity;
        }
        // TAKEN BY HEIST: a character's K27 body carries its gravity
        if let Some(b) = self.k27_po.get(&obj) {
            return b.phys.gravity;
        }
        self.k25_phys.body(obj).map_or([0.0; 3], |b| b.gravity)
    }
    fn ee_sub_1f5860(&mut self, obj: u32) -> [f32; 3] {
        // TAKEN BY MURRAY: a carried / thrown SO has its dynamic body (+0xe8 +0x80 = the world gravity)
        // (REPLACED: the static world snapshot has no dynamic body for it)
        // CRAWL R85: a live K6 guard (a STEPGUARD SO) has its dynamic body too (Murray's FUN_001b7970 throw of a carried guard)
        if self.so_carry.contains_key(&obj) || self.so_fly.contains_key(&obj) || (self.k6_live.contains_key(&obj) && std::env::var_os("SLY2_NO_R85_K6G").is_none()) {
            return super::guard::lift::GRAVITY;
        }
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
    fn root_14(&mut self, obj: u32) -> u32 {
        if let Some(&r) = self.ride_root.get(&obj) {
            return r;
        }
        // HEIST r17: **(obj + 0x14) = the top of the owner chain (e.g. the EP2 turret's launcher 9070 and its
        // class-98 SO 30820 share the entry root 27891). REPLACED: the loaded owner chain.
        let Some(w) = &self.world else { return obj };
        let mut i = obj as usize;
        let mut n = 0;
        while let Some(o) = i.checked_sub(1).and_then(|k| w.objects.get(k)).and_then(|x| x.owner) {
            i = o as usize + 1;
            n += 1;
            if n > 64 {
                break;
            }
        }
        i as u32
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
        let ovr = self.anim_ovr.get(&guard).and_then(|v| v.iter().rev().find(|x| x.0 == anim)).map(|x| x.1);
        super::guard::los::anim_length_ovr(w, guard, anim, ovr)
    }
    fn aseg_oid_len(&mut self, guard: usize, oid: i32) -> Option<(f32, f32)> {
        let w = self.world.as_ref()?;
        let o = crate::engine::npcdraw::script_aseg(w, guard, oid)?;
        crate::engine::playerdraw::aseg_of(w, o).map(|a| (a.length, a.f78()))
    }
    fn ee_sub_209618(&mut self, guard: usize, anim: i32, t: f32, rate: f32) -> [f32; 3] {
        // FUN_00209618 -> FUN_00204e38: the anim's root channel velocity; +0x190 = -v (returned as v)
        let Some(w) = &self.world else { return [0.0; 3] };
        let ovr = self.anim_ovr.get(&guard).and_then(|v| v.iter().rev().find(|x| x.0 == anim)).map(|x| x.1);
        let v = super::guard::los::root_velocity_ovr(w, guard, anim, ovr, t, rate);
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
    /// FUN_0014cc58(0, a, b, parts): FUN_0014c6a0 per part (the collider BSP clip FUN_00118668, first
    /// record) -> the record with the smallest entry t +0x40
    fn ee_sub_14cc58(&mut self, parts: &[super::k8::K8Part], a: [f32; 3], b: [f32; 3]) -> Option<crate::engine::physics::bsp::Lsg> {
        let so = self.so.clone()?;
        let mut best: Option<crate::engine::physics::bsp::Lsg> = None;
        for p in parts {
            let Some(c) = (p.id as usize).checked_sub(1).and_then(|k| so.colliders.get(k)) else { continue };
            if c.bsp.nodes.is_empty() {
                continue;
            }
            let mut out = Vec::new();
            if crate::engine::physics::bsp::FUN_00118668(&c.geom, &c.bsp, 0, c.bsp.nodes.len(), a, b, 1, &mut out) > 0 {
                if best.as_ref().is_none_or(|x| out[0].t[0] < x.t[0]) {
                    best = Some(out[0].clone());
                }
            }
        }
        best
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
        if let Some(&p) = self.npc_pos.get(&obj) {
            return p;
        }
        // BOSS R26: a TNT barrel lives in the live image (barrel::beams feeds its position)
        if let Some(&(_, p)) = self.barrels_85.iter().find(|b| b.0 == obj) {
            return p;
        }
        // MURRAY: the controlled character's own object (class 54 Murray / class 52 ...) is the player: its
        // world position is the JT's (FUN_001a62e8 reads the same xf the player update writes)
        if self.player_valid && self.po.as_ref().is_some_and(|p| p.obj as u32 + 1 == obj) {
            return self.player_pos;
        }
        // CRAWL R52: Sly's class-53 JT object (the first class-53 handle, as the k22 teleport / walk drains): its
        // vtable +0xfc world matrix is the one the JT update writes -> the live JT position, not the load xf
        if self.jt_live && self.world.as_ref().is_some_and(|w| w.objects.iter().position(|o| o.class == 53).is_some_and(|k| k as u32 + 1 == obj)) {
            return self.player_pos;
        }
        // TAKEN BY MURRAY: a thrown / carried SO object (REPLACED: the SO body integration / attach acts)
        if let Some(b) = self.so_fly.get(&obj) {
            return b.pos;
        }
        if let Some(&c) = self.so_carry.get(&obj) {
            let p = self.ee_sub_1a62e8(c);
            return [p[0], p[1], p[2] + SO_HAND_Z];
        }
        if let Some(p) = (obj as usize).checked_sub(1).and_then(|i| self.k27_pos(i)) {
            return p;
        }
        if std::env::var_os("SLY2_R76_NO_K6POS").is_none() {
            if let Some(&p) = self.k6_live.get(&obj) {
                return p;
            }
            // CRAWL R85: an ALO under a live guard (its class-36 carry point, ...) rides the guard's world xf
            // (FUN_001a62e8 = the child's world matrix = local xf * the guard body matrix the guard update writes)
            if std::env::var_os("SLY2_NO_R85_K6CHILD").is_none() && !self.k6_live.is_empty() {
                if let Some(p) = self.k6_child_pos(obj) {
                    return p;
                }
            }
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
        // TAKEN BY HEIST: a Bentley / Murray with a K27 body: its xf is the body (rows = body rotation), not the
        // load placement (the character objects sit off-map in the level data)
        if let Some(b) = self.k27_po.get(&obj) {
            let (r, o) = (b.phys.rot, b.phys.pos);
            return [0, 1, 2].map(|k| p[0] * r[0][k] + p[1] * r[1][k] + p[2] * r[2][k] + o[k]);
        }
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
        // CRAWL R97: ONE world +0x3d84 counter = k27_cine[3] (FUN_0020efc0 / 0020f030 flags bit 1, FUN_0020f198 /
        // 0020f1a8); world_3d84 kept for the old reading (SLY2_NO_R97_3D84)
        if std::env::var_os("SLY2_NO_R97_3D84").is_some() {
            return self.world_3d84 > 0;
        }
        self.k27_cine[3] > 0
    }
    fn world_3d84_add(&mut self, d: i32) {
        self.k27_cine[3] += d;
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
    /// FUN_001f31e0 parts of the object (a copy reads its template's): +0x18 = `AloPart::w1`, +0x14 = `s0`
    fn so_hit_parts(&mut self, obj: usize) -> Vec<(i32, i16)> {
        use crate::engine::world::objects::{LoadData, Payload};
        let Some(w) = self.k7_cache() else { return Vec::new() };
        let src = w.objects.get(obj).and_then(|o| o.clone_of).unwrap_or(obj);
        let Some(Payload::Loaded(LoadData::Alo { alo, .. })) = w.objects.get(src).map(|o| &o.payload) else { return Vec::new() };
        let k0 = self.part0_kind.get(&obj).copied();
        alo.parts.iter().enumerate().map(|(k, p)| (if k == 0 { k0.unwrap_or(p.w1 as i32) } else { p.w1 as i32 }, p.s0)).filter(|x| x.0 != 0).collect()
    }
    fn alo_110(&mut self, obj: u32) -> [u8; 26] {
        if let Some(t) = self.alo_110.get(&obj) {
            return *t;
        }
        // load-time property rows 53 = method 0x00100fe0 {kind 0x1011 mask, kind 0x1012 byte} -> FUN_001f7918, in
        // stream order (a clone reads its source's stream)
        use crate::engine::world::props::{ArrayItem, PropValue};
        let mut t = [0u8; 26];
        let Some(w) = self.world.clone() else { return t };
        let Some(i) = (obj as usize).checked_sub(1).filter(|&i| i < w.objects.len()) else { return t };
        let src = w.objects[i].clone_of.unwrap_or(i);
        // TAKEN BY HEIST: class 98 (C2d4b78) init slot 6 FUN_001bab50: FUN_001f7918(self, 0x3ffffff, 8) then
        // FUN_001f7918(self, 0x80001, 0), before the load-time rows below (EMU5 242: switch 123934 +0x110 =
        // 4 x26 except kind bit 8 = 8)
        if w.objects[i].class == 98 {
            t = [8; 26];
            t[0] = 0;
            t[19] = 0;
        }
        let word = |b: &[u8]| b.get(..4).map_or(0, |x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]));
        for p in crate::engine::game::fields::object_props(&w.objects[src]).into_iter().flatten() {
            let PropValue::Array(a) = &p.value else { continue };
            if p.field != 53 {
                continue;
            }
            let get = |k: u16| a.iter().find_map(|x| match x { ArrayItem::One(kk, b) if *kk == k => Some(word(b)), _ => None });
            let (Some(mask), Some(b)) = (get(0x1011), get(0x1012)) else { continue };
            for (k, v) in t.iter_mut().enumerate() {
                if mask & (1 << k) != 0 {
                    *v = b as u8;
                }
            }
        }
        t
    }
    fn ee_sub_1e0020(&mut self, seq: u32, ev: i32) {
        self.seq_signals.push((seq, ev));
    }
    fn take_seq_signals(&mut self) -> Vec<(u32, i32)> {
        std::mem::take(&mut self.seq_signals)
    }
    fn ee_sub_1f7598(&mut self, hit: &super::guard::slots::GuardHit) {
        if self.barrels_85.iter().any(|b| b.0 == hit.target) {
            self.barrel_hits.push(hit.clone());
            return;
        }
        self.player_hits.push(hit.clone());
    }
    /// BOSS R20: FUN_0013aeb0 contacts (REPLACED collision pairs, as k24 r_proj_contacts_13aeb0 for guards):
    /// Sly (PLAYER_HANDLE) while valid, same reach box around his feet
    fn proj_contacts_13aeb0(&mut self, _obj: usize, pos: [f32; 3]) -> Vec<super::k15_79::ProjContact> {
        use crate::engine::player::plugin::GUARD_CONTACT_REACH as R;
        let s = self.player_pos;
        let d = [s[0] - pos[0], s[1] - pos[1], pos[2] - s[2]];
        if self.player_valid && d[0] * d[0] + d[1] * d[1] < R * R && d[2] > -R && d[2] < 2.0 * R {
            let o = super::guard::PLAYER_HANDLE;
            return vec![super::k15_79::ProjContact { obj: o, so: o, point: pos }];
        }
        Vec::new()
    }
    /// HEIST r15: swept projectile contact (REPLACED SO solver): the entry point of p0 -> p1 into Sly's hull
    /// (FUN_0014c6a0, as the class-85 beams) or else the nearest entry into the static SO world (clip records
    /// whose segment start is inside a solid, zero entry plane, are skipped: the muzzle inside its launcher)
    fn proj_sweep_13aeb0(&mut self, _obj: usize, p0: [f32; 3], p1: [f32; 3]) -> Option<super::k15_79::ProjContact> {
        let o = super::guard::PLAYER_HANDLE;
        if let Some(pt) = self.beam_hit_14c6a0(o, p0, p1) {
            return Some(super::k15_79::ProjContact { obj: o, so: o, point: pt });
        }
        self.k7_cache();
        let so = self.so.clone()?;
        let l = so.clip_segment(p0, p1).into_iter().filter(|l| l.plane[0] != [0.0f32; 4]).min_by(|a, b| a.t[0].total_cmp(&b.t[0]))?;
        Some(super::k15_79::ProjContact { obj: 0, so: 0, point: l.pos[0] })
    }
    /// BOSS R20: FUN_0013b5e8 victims (REPLACED FUN_0020fc60 sphere query): Sly's sphere overlapping the `r` sphere at `pos`
    fn proj_victims_13b5e8(&mut self, _obj: u32, pos: [f32; 3], r: f32, _r_obj: f32, _r_34: f32) -> Vec<u32> {
        let s = self.player_pos;
        // Sly's bound sphere (REPLACED): the contact reach sphere above his feet
        use crate::engine::player::plugin::GUARD_CONTACT_REACH as R;
        let d = [s[0] - pos[0], s[1] - pos[1], s[2] + R - pos[2]];
        if self.player_valid && d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= (r + R) * (r + R) { vec![super::guard::PLAYER_HANDLE] } else { Vec::new() }
    }
    /// WIRE-OBJ: K15 class 85 SO overlap (REPLACED): Sly (PlayerFocus) while live; the beam set gates it by
    /// segment distance (k15_85::frame_k15_85)
    fn lo_removed(&mut self, obj: usize) -> bool {
        self.lo_out_of_world(obj)
    }
    fn so_overlaps_85(&mut self, _obj: usize) -> Vec<u32> {
        // BOSS R26: the TNT barrels' SOs overlap the beams too (slot 60 FUN_001de2d8 filters them per set: EP3 laser
        // defense 94890 lists classes [51, 88] = ignored)
        let mut v = if self.player_valid { vec![super::guard::PLAYER_HANDLE] } else { Vec::new() };
        v.extend(self.barrels_85.iter().map(|b| b.0));
        v
    }
    /// FUN_001df0a0 per beam: FUN_0014c6a0(Sly's collision part, p0, p1, 1, out) = FUN_00118668 against his
    /// GEOM-199 hull BSP (p5_hull::JtHull::FUN_0014c6a0); hit -> the entry point. (The FUN_00205670 fallback
    /// is for is-a 0x33 owners only.)
    fn beam_hit_14c6a0(&mut self, other: u32, p0: [f32; 3], p1: [f32; 3]) -> Option<[f32; 3]> {
        // BOSS R26: a barrel's SO (REPLACED geometry: the sphere r 20 of barrel::phys) -> segment entry point
        if let Some(&(_, c)) = self.barrels_85.iter().find(|b| b.0 == other) {
            return crate::engine::classes::q::barrel::seg_sphere(p0, p1, c, 20.0);
        }
        if other != super::guard::PLAYER_HANDLE || !self.player_valid {
            return None;
        }
        let (hull, rot) = self.player_hull.as_ref()?;
        hull.FUN_0014c6a0(rot, self.player_pos, p0, p1)
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
    fn k6_live_pos(&mut self, live: &[(u32, [f32; 3])]) {
        self.k6_live = live.iter().copied().collect();
    }
    fn k6_live_yaw(&mut self, live: &[(u32, f32)]) {
        self.k6_live_yaw = live.iter().copied().collect();
    }
    fn k21_pool_parent(&mut self, copy: usize, sp: u32) {
        // CRAWL R84: the FUN_001caec8 +0xf0 record loop of the claim (key-6 field-243 rows -> the copy's anim table)
        if std::env::var_os("SLY2_NO_R84_SPOVR").is_none() {
            if let Some(w) = self.world.clone() {
                let rows = (sp as usize).checked_sub(1).filter(|&i| i < w.objects.len()).map(|i| super::guard::los::spawner_anim_ovr(&w, i)).unwrap_or_default();
                for (id, oid) in rows {
                    let t = self.anim_ovr.entry(copy).or_default();
                    t.retain(|x| x.0 != id);
                    t.push((id, oid));
                }
            }
        }
        match self.parent_10(sp) {
            0 => self.pool_parent.remove(&copy),
            p => self.pool_parent.insert(copy, p as usize - 1),
        };
        // ANIM-ALL R66: the claim's vt +0xd4 (FUN_001cb518 -> FUN_001cb4e0): the copy subtree takes sp +0x2d
        self.sw_claim_mask(copy, sp);
    }
    fn k21_pool(&mut self) -> Option<(&mut std::collections::HashMap<u32, (usize, i32)>, &mut std::collections::HashMap<(usize, i32), u32>)> {
        Some((&mut self.pool_cur, &mut self.pool_owner))
    }
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
    fn k21_cam_sphere(&mut self, c: [f32; 3], r: f32) -> Option<bool> {
        self.cam.as_ref().map(|v| v.sphere_visible(c, r))
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
pub fn tick_classes(level: Res<ClassesLevel>, mut c: ResMut<Classes>, focus: Option<Res<crate::engine::player::plugin::PlayerFocus>>, cm: Option<Res<crate::engine::camera::cm::Cm>>, jobs: Option<Res<crate::engine::player::plugin::PlayerJobs>>, js: Option<ResMut<crate::engine::game::app::GameJobSlots>>) {
    let _crawl_prof = super::ProfGuard::new("tick_classes");
    let c = &mut *c;
    let Some(w) = &level.0 else { return };
    // K6: Sly (DAT_002e1e40) for the guard sensors: PlayerFocus position (far away while there is no player)
    let valid = focus.as_ref().is_some_and(|f| f.valid);
    let sly = focus.as_ref().filter(|f| f.valid).map(|f| [f.pos[0], f.pos[1], f.pos[2]]).unwrap_or([1.0e9; 3]);
    c.set.g.k6_player_tgt = focus.filter(|f| f.valid).map(|f| f.tgt);
    c.set.g.k6_players = vec![(super::guard::PLAYER_HANDLE, sly)];
    c.host.player_pos = sly;
    c.host.player_valid = valid;
    // K21: g_cm (0x2ddf40) matrix / scales for the class-119 spawn probes
    // POOL1 R55: DAT_002ddfd0 = g_cm +0x90 (camera position), DAT_002ddf5c = g_cm +0x1c (LOD scale)
    let lod = cm.as_ref().map(|m| ([m.mat[3][0], m.mat[3][1], m.mat[3][2]], m.r_mrd));
    c.host.cam = cm.map(|m| super::k21_119::CamView { mat: m.mat, x_scale: m.x_scale, y_scale: m.y_scale });
    let id = Arc::as_ptr(w) as usize;
    if c.built_for != id {
        c.host.world = Some(w.clone());
        let players = std::mem::take(&mut c.set.g.k6_players);
        let tgt = c.set.g.k6_player_tgt.take();
        // K26: a new level: the music instance sources start empty (keys from feed_music_block)
        c.host.k26_music.layers = Default::default();
        let tb = std::time::Instant::now();
        c.host.sp_watch.clear();
        c.set = ClassSet::build(w, &mut c.host);
        if std::env::var_os("SLY2_CLS_PROF").is_some() { eprintln!("CLSPROF build {:.0} ms", tb.elapsed().as_secs_f64() * 1000.0); }
        c.set.g.k6_players = players;
        c.set.g.k6_player_tgt = tgt;
        c.set.g.g_3dd268 = c.host.k26_music.inst;
        c.set.post_load(&mut c.host);
        // CRAWL R121: the load-time 0x162250 task / job cue records (FUN_00161a58 case 5 -> FUN_00164270 ->
        // FUN_001657d0 -> FUN_001623d0) run inside the original's world load, before the level clock starts: their
        // SMA transitions (FUN_001f0948 / 0x1f0660 -> FUN_001f0b80 -> FUN_001f0ad0 rand) draw from the one rand
        // before any class update and before the clk-0 rand state (the test load seeds below). Applied here, the
        // batch `job_level_loaded` collected this frame; `lend_classes` only applies later (run-time) records.
        // Opt-out SLY2_NO_R121_LOADCUE=1 (= R118 order: after the first class frame).
        if std::env::var_os("SLY2_NO_R121_LOADCUE").is_none() {
            if let (Some(j), Some(mut js)) = (jobs.as_ref(), js) {
                crate::engine::game::job_flow::apply_anims(&j.0, &mut c.host.k25, w, &mut js.anim_seen);
            }
        }
        // FEEL R35: the deferred FUN_001ca8c0(spawner) nodes: FUN_001caec8(spawner, 2) Adds a free copy of the
        // spawner's template (FUN_001cb250(sp, 0) = the same call for the single-object templates; UNPORTED: m > 1)
        c.host.drain_spawn_q(&mut c.set);
        // POOL1 R92: SLY2_RNG_LOADSEED=<hex>: the shared rand (physics::driven) = the original's state after this level
        // load (a savestate's 0x2c4740 at level clock 0, e.g. rec11 001 0xd943b486 for the EP3 hub)
        if let Some(s) = std::env::var("SLY2_RNG_LOADSEED").ok().and_then(|v| u32::from_str_radix(v.trim_start_matches("0x"), 16).ok()) {
            crate::engine::physics::driven::rng_shared(Some(s));
        }
        // POOL1 R96: the per-thread form (tests on single-threaded schedules)
        if let Some(s) = crate::engine::physics::driven::rng_thread_loadseed_take() {
            crate::engine::physics::driven::rng_shared_thread(Some(s));
        }
        c.built_for = id;
    }
    // clock (J: dt clamped to [1/60, 1/30]; fixed 1/60 here)
    c.set.g.dt = 1.0 / 60.0;
    c.set.g.t += c.set.g.dt;
    if crate::engine::classes::k12::LOOK_T_STAMP.swap(false, std::sync::atomic::Ordering::Relaxed) {
        c.set.g.g_3e1be0 = c.set.g.t;
    }
    if crate::engine::classes::k12::JOB_T_STAMP.swap(false, std::sync::atomic::Ordering::Relaxed) {
        c.set.g.g_3e1be4 = c.set.g.t;
    }
    let before: std::collections::HashSet<usize> = c.set.k21c.iter().map(|x| x.0).collect();
    if let Some((p, s)) = lod {
        c.set.g.g_2ddfd0 = p;
        c.set.g.g_2ddf5c = s;
    }
    c.set.frame(&mut c.host);
    c.host.sp_watch_frame(&mut c.set);
    let tl = std::time::Instant::now();
    k28_apply_looks(&mut c.set, &mut c.host, &before);
    if std::env::var_os("SLY2_CLS_PROF").is_some() { super::prof_add("k28_apply_looks", tl.elapsed()); }
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
    let clk = crate::engine::camera::cm::CmClock { t: time.elapsed_secs(), dt: time.delta_secs(), dt_real: time.delta_secs() };
    let look = sf.and_then(|f| f.look);
    apply_class_camera_on(&mut c.host, cm.map(|c| c.into_inner()), &mut st, look, clk);
}

/// WIRE-AN: the apply_class_camera body on one host (the app's `Classes`, or a `ClassLink`'s host).
pub fn apply_class_camera_on(host: &mut EngineHost, cm: Option<&mut crate::engine::camera::cm::Cm>, st: &mut super::k29_cam::K29Cam, look: Option<usize>, clk: crate::engine::camera::cm::CmClock) {
    let reqs = std::mem::take(&mut host.k29_cam);
    // K30: published object states and playerdraw's look policy (classes/k30_cam.rs)
    st.objs.lock().unwrap().extend(host.k30_cam.drain());
    st.look = look;
    if let Some(cm) = cm {
        // K30: positions the zoom look reads (its objects, those of the queued setters, the player)
        {
            use super::k30_cam::ZoomOp;
            let h: &mut dyn ClassHost = host;
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
        st.apply(cm, &reqs, clk);
        host.k30_zoom_waits = Some(st.zoom_waits);
    }
}

/// K28: the class code's pad lock bits 2 / 4 (`EngineHost::k27_pad_lock`, action 0x12 world state) on the
/// input backend: FUN_00187078 for bits newly set, FUN_001870a0 for bits cleared (DAT_002dfbc4 / 002dfbc0).
#[cfg(feature = "game")]
pub fn apply_class_pad_lock(mut c: ResMut<Classes>, pad: Option<ResMut<crate::engine::input::backend::PadInput>>, live: Option<Res<super::q::live::LiveQ>>, mut prev: Local<u32>) {
    let Some(mut pad) = pad else { return };
    let p = &mut *pad;
    // POOL1: DAT_002dfbc0 for the class code (FUN_00136870 kind 3, ...)
    // (FUN_001870d0 of the current DAT_002dfbc4 flags: the backend's lock.mode is only refreshed on a change)
    let (mut l, mut scratch) = (p.lock, p.pad.clone());
    crate::engine::input::pad::FUN_001870d0(&mut l, &mut scratch);
    c.host.g_2dfbc0 = l.mode;
    let mut bits = c.host.k27_pad_lock & 6;
    // CRAWL R103: + the live image's DAT_002dfbc4 lock bits (FUN_00187078 from live code: the ThiefNet open
    // FUN_001dc260 -> FUN_0020efc0(world, 0) -> lock 2 until FUN_0020f030 on close): the original has one
    // DAT_002dfbc4, so while the shop is up the gameplay pad (Sly) gets nothing
    if std::env::var_os("SLY2_NO_R103_LIVELOCK").is_none() {
        if let Some(m) = live.as_ref().and_then(|l| l.mem.as_ref()) {
            bits |= m.peek32(0x002d_fbc4) & 6;
        }
    }
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
        app.init_resource::<super::guard::hearing::StimPool>().add_systems(Update, super::guard::hearing::sly_noise.after(crate::engine::game::app::return_classes).before(tick_classes));
        // ANIM R80: sly_noise reads the class clock / host noises / guards: only valid between return_classes and
        // lend_classes (unordered it ran on the lent empty stand-in: now = 0 and no guard got the stimulus pool)
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
    /// FEEL R88: `ClassHost::lo_removed` for shared readers (the player layer's water list)
    pub fn lo_out_of_world(&self, obj: usize) -> bool {
        // FEEL R32: out of the world's update list = LO +0x28 bit 60 clear (script / job-toggle Remove, or the
        // load-time field-26 Remove)
        if self.laser_off.contains(&obj) {
            return true;
        }
        if let Some(&on) = self.rt_overlay.get(&obj) {
            return !on;
        }
        match (&self.oow, &self.world) {
            (Some(s), _) => s.contains(&(obj as u32)),
            (None, Some(w)) => crate::engine::game::fields::out_of_world_at_load(w, obj),
            (None, None) => false,
        }
    }
    /// K29: queue a camera request (bounded: a host without a CM drops the oldest)
    fn k29_req(&mut self, r: super::k29_cam::CamReq) {
        if self.k29_cam.len() >= 256 {
            self.k29_cam.remove(0);
        }
        self.k29_cam.push(r);
    }

    /// K13 / K24: world object matrix = root bone world (rest pose) under its +0x12c placement (clones); K16: no
    /// own frame -> the owner-chain frame (P15 instance_frames). None = not a world object.
    /// CRAWL R85: world position of an ALO whose owner chain reaches a live K6 guard: its static transform relative
    /// to the guard, carried by the live guard body (pos, yaw: x row (cos, sin, 0), y row (-sin, cos, 0))
    fn k6_child_pos(&mut self, obj: u32) -> Option<[f32; 3]> {
        let w = self.world.clone()?;
        let mut a = (obj as usize).checked_sub(1)?;
        let g = loop {
            a = w.objects.get(a)?.owner? as usize;
            if self.k6_live.contains_key(&(a as u32 + 1)) {
                break a as u32 + 1;
            }
        };
        use crate::engine::gfx::scene::{invert, mul};
        let rel = mul(&self.k24_world_xf_static(obj)?, &invert(&self.k24_world_xf_static(g)?));
        let (p, y) = (self.k6_live[&g], self.k6_live_yaw.get(&g).copied().unwrap_or(0.0));
        let (s, c) = y.sin_cos();
        let live = [[c, s, 0.0], [-s, c, 0.0], [0.0, 0.0, 1.0], p];
        Some(mul(&rel, &live)[3])
    }
    pub fn k24_world_xf(&mut self, obj: u32) -> Option<crate::engine::gfx::scene::Affine> {
        let m = self.k24_world_xf_static(obj)?;
        if self.k25_phys.targets.is_empty() {
            return Some(m);
        }
        // HEIST r17: an ALO whose xfAng record +0x5c (FUN_0012dc18(alo, +0x5c, 1, 5, 3, v), e.g. the class-69 turret
        // pin) holds a target euler: its local rotation is that euler (FUN_0012d108 -> xf +0xc0; the constraint
        // drives the body there), and its subtree follows. REPLACED: the xf integration toward the target is
        // instant; the target replaces the local rotation of the static hierarchy.
        let w = self.world.clone()?;
        let mut a = (obj as usize).checked_sub(1)?;
        let v = loop {
            if let Some(v) = self.k25_phys.targets.get(&((a as u32 + 1).wrapping_mul(0x100) + 0x5c)) {
                break *v;
            }
            match w.objects.get(a).and_then(|o| o.owner) {
                Some(o) => a = o as usize,
                None => return Some(m),
            }
        };
        use crate::engine::gfx::scene::{invert, mul};
        let am = self.k24_world_xf_static(a as u32 + 1)?;
        let pr = match w.objects[a].owner {
            Some(o) => self.k24_world_xf(o as u32 + 1)?,
            None => crate::engine::gfx::scene::IDENTITY,
        };
        let e = crate::engine::anim::actor::euler_rows(v);
        let mut pa = am;
        for r in 0..3 {
            pa[r] = std::array::from_fn(|k| e[r][0] * pr[0][k] + e[r][1] * pr[1][k] + e[r][2] * pr[2][k]);
        }
        Some(mul(&mul(&m, &invert(&am)), &pa))
    }
    pub(crate) fn k24_world_xf_static(&mut self, obj: u32) -> Option<crate::engine::gfx::scene::Affine> {
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
        // HEIST: objects under a placed owner that is itself under a placed root (TB water-tower interior 141875
        // at z 20000 -> valves 142138 + copies -> use points 80398). EMU5 252 RAM: valve +0x180 (local) = its
        // placement, world +0x1c0 = placement x 141875 world (z 20008); use point world = its XFM x the valve
        // bone x the valve world ((779, 686, 20048) for the template). Template copies (Payload::Clone) take
        // the local data of their clone_of source.
        if self.rt_depth < 64 {
            if let Some(m) = self.heist_nested_world(&w, i) {
                self.pos_cache.insert(i, m);
                return Some(m);
            }
        }
        // HEIST r17: an XFM on a bone of its unplaced parent ALO that holds a pin target (e.g. the EP2 turret's shell
        // frame 28585 on bone 1 of class 69 28548): world = local x the parent bone (model space) x the parent's world
        // (GetXfmPos = ConvertAloPos(paloParent, bone, local)). The static rest pose of the others is unchanged.
        if !w.life.placed.contains_key(&i) && self.rt_depth < 64 {
            use crate::engine::gfx::attach::{xfm_bone_ref, xfm_of, BoneRef, BoneWorlds};
            if let (Some((_, local)), Some(BoneRef::Bone(o, k))) = (xfm_of(&w, i), xfm_bone_ref(&w, i)) {
                if w.objects[i].owner == Some(o)
                    && !w.life.placed.contains_key(&o)
                    && self.k25_phys.targets.contains_key(&((o as u32 + 1).wrapping_mul(0x100) + 0x5c))
                {
                    self.rt_depth += 1;
                    let pw = self.k24_world_xf_static(o as u32 + 1);
                    self.rt_depth -= 1;
                    if let Some(pw) = pw {
                        let bone = BoneWorlds::default().get(&w, o, k);
                        return Some(crate::engine::gfx::scene::mul(&local, &crate::engine::gfx::scene::mul(&bone, &pw)));
                    }
                }
            }
        }
        // ANIM-ALL R82: an unplaced XFM on a bone of an unplaced owner that a class-run ASEGA poses this frame (K29) rides
        // the POSED bone (GetXfmPos = local x bone world): the fresh Paris hub intro camera 141488 (class 44, XFM on bone 2
        // of holder 123962, ASEG 123963 drives bones 1/2 for 1330 frames) flies instead of staring from its rest pose.
        // Opt-out SLY2_NO_R82_XFMBONE=1 (= rest bone).
        if !w.life.placed.contains_key(&i) && !self.k29_bones.is_empty() && std::env::var_os("SLY2_NO_R82_XFMBONE").is_none() {
            use crate::engine::gfx::attach::{xfm_bone_ref, xfm_of, BoneRef};
            if let (Some((_, local)), Some(BoneRef::Bone(o, k))) = (xfm_of(&w, i), xfm_bone_ref(&w, i)) {
                if !w.life.placed.contains_key(&o) {
                    if let Some(pb) = self.k29_bones.get(&(o, k)) {
                        return Some(crate::engine::gfx::scene::mul(&local, pb));
                    }
                }
            }
        }
        // HEIST r18: an unplaced child (no bone XFM on a placed object) of a K26-posed owner follows it: its local xf x
        // the posed owner (the EP2 chopper 118349's launcher 9070 / missile spawner: FUN_001a62e8 of the launcher =
        // the moving chopper's frame, rec10 198 missiles leave the chopper)
        let posed_owner = match (w.life.placed.contains_key(&i), w.objects[i].owner) {
            (false, Some(o)) if !matches!(crate::engine::gfx::attach::xfm_bone_ref(&w, i), Some(crate::engine::gfx::attach::BoneRef::Bone(..))) => self.k26_posed.get(&o).copied(),
            _ => None,
        };
        let r = *self.pos_cache.entry(i).or_insert_with(|| {
            use crate::engine::gfx::attach::{affine_from_bytes, BoneWorlds};
            let mut bw = BoneWorlds::default();
            match w.life.placed.get(&i).map(affine_from_bytes) {
                // CRAWL R75: a template copy root hung off its owner's socket already carries the +0x12c placement in
                // its bone-0 chain (BoneWorlds A6: local x pm x owner bone) -> that IS the world (rec9 012: row-7
                // pump oid 125253 +0x190 = (-5454.99, 2982.11, 52.81) = bone0; x pm again gave (5465, 7105))
                Some(_) if std::env::var_os("SLY2_R75_PM2").is_none()
                    && matches!(crate::engine::gfx::attach::alo_bone_refs(&w, i).first(), Some((crate::engine::gfx::attach::BoneRef::None, _)))
                    && crate::engine::gfx::attach::template_socket(&w, i).is_some() => bw.get(&w, i, 0),
                Some(pm) => crate::engine::gfx::scene::mul(&bw.get(&w, i, 0), &pm),
                // WP-K33: an XFM on a bone of a placed (P15) object, e.g. a class-37 guard loot holder under its
                // class-48 owner: local x the owner bone x the owner's placement (the case above for bone k)
                None if let (Some((_, local)), Some(crate::engine::gfx::attach::BoneRef::Bone(o, k))) =
                    (crate::engine::gfx::attach::xfm_of(&w, i), crate::engine::gfx::attach::xfm_bone_ref(&w, i))
                    && w.life.placed.contains_key(&o) =>
                {
                    let pm = affine_from_bytes(&w.life.placed[&o]);
                    // CRAWL R75: the owner's bone chain already holds its socket placement (as above): no 2nd pm
                    // (rec9 012: pump hit volume 125273 +0xd0 = (-5455.0, 2982.1, 293.2) = local x pump world)
                    if std::env::var_os("SLY2_R75_PM2").is_none()
                        && matches!(crate::engine::gfx::attach::alo_bone_refs(&w, o).first(), Some((crate::engine::gfx::attach::BoneRef::None, _)))
                        && crate::engine::gfx::attach::template_socket(&w, o).is_some()
                    {
                        crate::engine::gfx::scene::mul(&local, &bw.get(&w, o, k))
                    } else {
                        crate::engine::gfx::scene::mul(&local, &crate::engine::gfx::scene::mul(&bw.get(&w, o, k), &pm))
                    }
                }
                None => bw.xfm_world(&w, i).or_else(|| super::shared::path10::world_xf(&w, i, None)).unwrap_or_else(|| bw.get(&w, i, 0)),
            }
        });
        Some(posed_owner.map_or(r, |pw| crate::engine::gfx::scene::mul(&r, &pw)))
    }

    /// HEIST: see `k24_world_xf` (nested placed hierarchy). None = not that case.
    fn heist_nested_world(&mut self, w: &Arc<crate::engine::world::World>, i: usize) -> Option<crate::engine::gfx::scene::Affine> {
        use crate::engine::gfx::attach::{affine_from_bytes, xfm_bone_ref, xfm_of, BoneRef, BoneWorlds};
        use crate::engine::gfx::scene::{invert, mul};
        let src = |mut k: usize| { while let Some(c) = w.objects[k].clone_of { k = c; } k };
        let placed_root = |k: usize| w.life.placed.contains_key(&k) && w.objects[k].owner.is_none();
        let o = w.objects[i].owner?;
        if let Some(pm) = w.life.placed.get(&i) {
            if !placed_root(o) { return None; }
            self.rt_depth += 1;
            let ow = self.k24_world_xf(o as u32 + 1);
            self.rt_depth -= 1;
            return Some(mul(&affine_from_bytes(pm), &ow?));
        }
        let oo = w.objects[o].owner?;
        if !w.life.placed.contains_key(&o) || !placed_root(oo) { return None; }
        let si = src(i);
        let (_, local) = xfm_of(w, si)?;
        let BoneRef::Bone(ob, k) = xfm_bone_ref(w, si)? else { return None };
        let mut bw = BoneWorlds::default();
        let rel = mul(&bw.get(w, ob, k), &invert(&bw.get(w, ob, 0)));
        self.rt_depth += 1;
        let ow = self.k24_world_xf(o as u32 + 1);
        self.rt_depth -= 1;
        Some(mul(&local, &mul(&rel, &ow?)))
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
        if std::env::var("SLY2_K22DBG").is_ok() {
            eprintln!("[R50] so_part_link obj {obj} id {id:#x} on {on}");
        }
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
