//! WP-Q (feature `game`): draw Sly where the player is, pose his skin joints from the JT ASEGs that
//! 0x18c250 requests (player::plugin::JtAnimEvents), and run the follow camera on the player.
//!
//! Drawing reuses the level's JT entities (gfx::game::spawn_level_geometry already spawns every ALO of
//! the JT subtree, skinned prims with joint entities): the JT object entity gets the player's
//! transform (AloPhys rot/pos = the JT's xform, as the original draws it) and each joint entity the
//! posed bone world * inverse(rest object world) (the same space spawn_level_geometry uses).
//! Replaced, not 1:1: the VU1 palette upload (Bevy SkinnedMesh, see gfx::skin). WP-Q2: child ALOs follow
//! their parent bone (rig bone (object, 0) world, parent from the ALO hierarchy as gfx::attach does for
//! level objects) and are shown / hidden by vis::JtVis.
//! Camera: CplcyFollow (camera::follow, 0x002219c8 ctor) over player::plugin::PlayerFollowWorld, pushed
//! with SetCmPolicy FUN_0014e328 and updated by UpdateCm FUN_0014d478 every frame.

use super::vis::JtVis;
use super::{find_jt, glide_track, subtree, JtAnimator, JtRig, FIELD_JUMP, FIELD_RUN, SM_LABEL_STAND};
use crate::engine::player::JTS_RUN;
use crate::engine::anim::pose::Mat4;
use crate::engine::camera::bevy_sync::sync_cm_camera;
use crate::engine::camera::cm::{add_policy, Cpdefi, CmWorld, FUN_0014d478, FUN_0014e328};
use crate::engine::camera::collide::CmSegmentQuery;
use crate::engine::camera::follow::{CplcyFollow, FollowJt, FollowParams, FollowTrack, FollowWorld};
use crate::engine::player::p15::{CamCollide, CwQuery};
use crate::engine::camera::policy::CplcyCtx;
use crate::engine::camera::{Cm, CmClock};
use crate::engine::gfx::attach::BoneWorlds;
use crate::engine::gfx::game::{affine_to_transform, AloObject, GfxLevelData, InkOutline};
use crate::engine::gfx::scene::{invert, mul, Affine, LevelScene};
use crate::engine::gfx::skin::{record_weight_refs, skin_joints};
use crate::engine::player::plugin::{JtAnimEvents, Player, PlayerFocus, PlayerFollowWorld, PlayerPad};
use crate::engine::world::objects::{LoadData, Payload};
use crate::engine::world::World;
use bevy::mesh::skinning::SkinnedMesh;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// Sly's drawing state for the current world.
#[derive(Resource, Default)]
pub struct SlyDraw {
    /// world identity the state was built for
    pub world_key: usize,
    pub animator: Option<JtAnimator>,
    /// JT object entity
    pub jt_entity: Option<Entity>,
    /// JT-subtree ALO entities: (entity, object, owner object (None = the JT), rest level world)
    pub objects: Vec<(Entity, usize, Option<usize>, Affine)>,
    /// (skin joint entity, rig bone, object)
    pub joints: Vec<(Entity, usize, usize)>,
    /// rigid prim entities: (entity, instance xform (identity for the record itself), rig bone, object)
    pub rigid: Vec<(Entity, Affine, usize, usize)>,
    /// rest level world of the JT object (its entity carries the player transform)
    pub jt_rest: Affine,
    /// WP-Q2: removed objects / hidden records of the JT subtree
    pub vis: Option<JtVis>,
    /// every drawn prim entity (surfaces and ink edges): (entity, object, record, instance)
    pub prims: Vec<(Entity, usize, usize, Option<usize>)>,
    /// Jt::calls already scanned for FUN_0018e540
    pub calls_seen: usize,
    /// ink edge entities of skinned records (port limitation: never drawn)
    pub ink_skinned: HashSet<Entity>,
    /// InkOutline components taken off hidden ink entities (update_ink_outlines re-shows every
    /// InkOutline each frame)
    pub ink_parked: HashMap<Entity, InkOutline>,
    /// visibility written since the last change
    pub applied: bool,
}

/// Follow camera policy slot (camera policy table index) and the focus it reads.
#[derive(Resource, Default)]
pub struct SlyFollow {
    pub focus: Arc<Mutex<PlayerFocus>>,
    /// WP-P15: camera collision input of the boxed policy (set every frame by follow_camera)
    pub coll: Arc<Mutex<CamCollide>>,
    pub policy: Option<usize>,
    pub world_key: usize,
    /// WP-P24: the JT's glider camera track (jmp[0x6f] = 0x5f4, class 15) as (pv, track) of this world
    pub track: Arc<Mutex<Option<(u32, FollowTrack)>>>,
    /// WP-P24: jt+0x1024 set (glide track policy pushed)
    pub glide: bool,
    /// WP-P24: CM 0x2de390 C2cd4e0 look (binocucom mode 0/2 policy) table slot, and whether it is pushed
    pub look: Option<usize>,
    pub binoc: bool,
    /// WP-I5: the world's class-15 region tracks (camera/track.rs, FUN_00221298 each frame)
    pub regions: Arc<Mutex<crate::engine::camera::track::TrackSet>>,
    /// FEEL R24: LiveQ memory lent to the follow policy while `follow_camera` runs
    live: LiveMemPtr,
    /// FEEL R25: jt +0x104c / +0x1050 (FUN_00141640 camera height; 0 for a fresh JT = new world)
    jt_cam_z: [f32; 2],
}

/// WP-P24: LookWorld of the binocucom look: the viewer is Sly (FOLLOW_PSO, a JT: eye 150 over the origin).
struct SlyLook(Arc<Mutex<PlayerFocus>>);
impl crate::engine::camera::looks::LookWorld for SlyLook {
    fn is_jt(&mut self, pso: u32) -> bool {
        pso == FOLLOW_PSO
    }
    fn so_pos(&mut self, _pso: u32) -> [f32; 3] {
        self.0.lock().unwrap().pos
    }
    /// DAT_003e1be0 = t (LOOK R36: read by the K12 job marker fade on the class clock)
    fn set_t_look(&mut self, _t: f32) {
        crate::engine::classes::k12::stamp_look_t();
    }
}

/// FollowWorld of the boxed CplcyFollow: the player's focus of this frame.
struct SharedFollow(Arc<Mutex<PlayerFocus>>, Arc<Mutex<CamCollide>>, Option<CwBoxQuery>, Arc<Mutex<Option<(u32, FollowTrack)>>>, Arc<Mutex<crate::engine::camera::track::TrackSet>>, Arc<Mutex<CwBounds>>, LiveMemPtr);

/// FEEL R24: the LiveQ object memory lent to the boxed follow policy for one `follow_camera` run (null
/// outside it): `FollowWorld::mem` / `track_addr` (track pv = object index + 1 -> `LiveMem::obj_addr`).
type LiveMemPtr = Arc<std::sync::atomic::AtomicPtr<crate::engine::classes::q::live::LiveMem>>;
struct LendMem(LiveMemPtr);
impl Drop for LendMem {
    fn drop(&mut self) {
        self.0.store(std::ptr::null_mut(), std::sync::atomic::Ordering::Release);
    }
}

/// PERF2: per-collider vertex AABBs of the camera collision world (computed once per world instead of on
/// every clip / sweep / push-out query). Pre-filters the colliders with the SAME box tests the p15 / physics
/// queries apply (identical bounds, order kept), so the queries see exactly the colliders they would keep.
#[derive(Default)]
struct CwBounds {
    key: usize,
    boxes: Vec<Option<([f32; 3], [f32; 3])>>,
    recent: Vec<(Vec<usize>, Arc<crate::engine::physics::CollisionWorld>)>,
}
impl CwBounds {
    fn subset(&mut self, cw: &Arc<crate::engine::physics::CollisionWorld>, keep: impl Fn(&[f32; 3], &[f32; 3]) -> bool) -> Arc<crate::engine::physics::CollisionWorld> {
        let key = Arc::as_ptr(cw) as usize;
        if self.key != key || self.boxes.len() != cw.colliders.len() {
            self.boxes = cw
                .colliders
                .iter()
                .map(|c| {
                    let v = c.geom.verts.first()?;
                    let (mut bl, mut bh) = (*v, *v);
                    for v in &c.geom.verts {
                        for k in 0..3 {
                            bl[k] = bl[k].min(v[k]);
                            bh[k] = bh[k].max(v[k]);
                        }
                    }
                    Some((bl, bh))
                })
                .collect();
            self.key = key;
            self.recent.clear();
        }
        let idx: Vec<usize> = self.boxes.iter().enumerate().filter(|(_, b)| b.as_ref().is_some_and(|(l, h)| keep(l, h))).map(|(i, _)| i).collect();
        if idx.len() == cw.colliders.len() {
            return cw.clone();
        }
        // the kept sets repeat frame to frame: reuse their worlds (no per-query collider clones)
        if let Some(k) = self.recent.iter().position(|(i, _)| *i == idx) {
            let hit = self.recent.remove(k);
            let w = hit.1.clone();
            self.recent.insert(0, hit);
            return w;
        }
        let w = Arc::new(crate::engine::physics::CollisionWorld { colliders: idx.iter().map(|&i| cw.colliders[i].clone()).collect() });
        self.recent.insert(0, (idx, w.clone()));
        self.recent.truncate(8);
        w
    }
}

/// PERF2: 0x14ea08 segment query over the box-filtered colliders (`CollisionWorld::clip_segment` rejects the
/// same colliders by the same segment AABB test).
struct CwBoxQuery(Arc<crate::engine::physics::CollisionWorld>, Arc<Mutex<CwBounds>>);
impl CmSegmentQuery for CwBoxQuery {
    fn solid_intervals(&mut self, start: [f32; 3], end: [f32; 3]) -> Vec<(f32, f32)> {
        let lo = [start[0].min(end[0]), start[1].min(end[1]), start[2].min(end[2])];
        let hi = [start[0].max(end[0]), start[1].max(end[1]), start[2].max(end[2])];
        let sub = self.1.lock().unwrap().subset(&self.0, |bl, bh| !(0..3).any(|k| bh[k] < lo[k] || bl[k] > hi[k]));
        CwQuery(sub).solid_intervals(start, end)
    }
}
impl SharedFollow {
    fn w(&self) -> PlayerFollowWorld {
        PlayerFollowWorld(*self.0.lock().unwrap())
    }
}
impl CmWorld for SharedFollow {
    fn get_cpdefi(&mut self, pso: u32) -> Cpdefi {
        self.w().get_cpdefi(pso)
    }
    fn jt_state(&mut self, pso: u32) -> Option<i32> {
        self.w().jt_state(pso)
    }
}
impl FollowWorld for SharedFollow {
    fn jt(&mut self, pso: u32) -> Option<FollowJt> {
        self.w().jt(pso)
    }
    fn so_pos(&mut self, pso: u32) -> [f32; 3] {
        self.w().so_pos(pso)
    }
    fn track(&mut self, pv: u32) -> Option<FollowTrack> {
        let g = self.3.lock().unwrap().as_ref().filter(|t| t.0 == pv).map(|t| t.1.clone());
        g.or_else(|| self.4.lock().unwrap().get(pv).map(|t| t.track.clone()))
    }
    fn mem(&mut self) -> Option<&mut dyn crate::engine::classes::q::q2mem::QMem> {
        // SAFETY: non-null only inside follow_camera, which holds the ResMut<LiveQ> it points into
        unsafe { self.6.load(std::sync::atomic::Ordering::Acquire).as_mut() }.map(|m| m as &mut dyn crate::engine::classes::q::q2mem::QMem)
    }
    fn track_addr(&mut self, pv: u32) -> Option<u32> {
        // SAFETY: as `mem`
        let m = unsafe { self.6.load(std::sync::atomic::Ordering::Acquire).as_ref() }?;
        (pv as usize).checked_sub(1).and_then(|i| m.obj_addr.get(i).copied()).filter(|&a| a != 0)
    }
    /// Sly's JT fields (+0x528 / +0x4c8 / +0x4cc) are not kept in LiveMem
    fn pso_addr(&mut self, _pso: u32) -> Option<u32> {
        None
    }
    fn set_track_params(&mut self, pv: u32, p: &FollowParams) {
        if let Some(t) = self.3.lock().unwrap().as_mut().filter(|t| t.0 == pv) {
            t.1.params = *p;
        } else if let Some(t) = self.4.lock().unwrap().get_mut(pv) {
            t.track.params = *p;
        }
    }
    fn segments(&mut self) -> Option<&mut dyn CmSegmentQuery> {
        self.2 = self.1.lock().unwrap().cw.clone().map(|cw| CwBoxQuery(cw, self.5.clone()));
        self.2.as_mut().map(|q| q as &mut dyn CmSegmentQuery)
    }
    fn sweep_sphere(&mut self, radius: f32, max_len: f32, start: [f32; 3], vel: [f32; 3]) -> [f32; 3] {
        let c = self.1.lock().unwrap().clone();
        // PERF2: same +-COLLIDE_BOX gather test as cam_sweep_14f110 on the cached bounds (same colliders, same
        // order), then its 0x14eda0 call directly (cam_sweep_14f110 re-scanned and cloned the near set per call)
        use crate::engine::camera::follow::COLLIDE_BOX;
        let Some(cw) = &c.cw else { return vel };
        let near = self.5.lock().unwrap().subset(cw, |bl, bh| (0..3).all(|k| bh[k] >= start[k] - COLLIDE_BOX && bl[k] <= start[k] + COLLIDE_BOX));
        if near.colliders.is_empty() {
            return vel;
        }
        let (nr, dt) = (c.near_radius, c.dt_real);
        crate::engine::physics::sphere::FUN_0014eda0(&near, radius, max_len, start, vel, &mut |cc, v, sb| crate::engine::physics::dynamics::FUN_0014f040(nr, dt, cc, v, sb))
    }
    fn push_out(&mut self, radius: f32, lo: [f32; 3], hi: [f32; 3], centre: [f32; 3]) -> [f32; 3] {
        let mut c = self.1.lock().unwrap().clone();
        // PERF2: same aabb_hits(lo, hi, radius) test as cam_pushout_223a70, on the cached bounds
        c.cw = c.cw.map(|cw| self.5.lock().unwrap().subset(&cw, |bl, bh| (0..3).all(|k| bh[k] >= lo[k] - radius && bl[k] <= hi[k] + radius)));
        crate::engine::player::p15::cam_pushout_223a70(&c, radius, lo, hi, centre)
    }
}

/// Focus id handed to SetCmPolicy (the original passes the JT pointer; any non-zero id selects CPDEFI).
pub const FOLLOW_PSO: u32 = 7;
/// Policy priority of the player follow.
pub const FOLLOW_CPP: i32 = 1;
/// WP-P24: 0x19cd38 binocucom look priority (C2cd4e0 at CM 0x2de390).
pub const BINOC_CPP: i32 = 7;
/// WP-P24: DAT_002bf798 / DAT_002bf79c (glide camera distances; FUN_00140c88 uses their mean).
pub const GLIDE_DIST_2BF798: f32 = -250.0;
pub const GLIDE_DIST_2BF79C: f32 = -500.0;

pub struct PlayerDrawPlugin;

impl Plugin for PlayerDrawPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SlyDraw>().init_resource::<SlyFollow>().init_resource::<PlayerAnimFired>().init_resource::<PlayerAnimDispatch>().add_systems(
            Update,
            (setup_sly_draw, drive_sly, drive_po, drive_idle_po, dispatch_player_events, follow_camera)
                .chain()
                .after(crate::engine::game::app::frame_camera)
                .after(crate::engine::player::plugin::publish_focus)
                .before(sync_cm_camera),
        )
        .add_systems(Update, hide_hidden_casters.after(drive_sly));
    }
}

/// ANIM R75: the type-2 caster of a mesh record Sly does not draw (JtVis hidden records, FUN_0018e540 /
/// the WP-Q2 set) casts nothing either: the original only walks the drawn records. The casters are
/// separate entities (gfx::game), so `apply_visibility` (surfaces / ink edges) left them on: the large
/// translucent trapezoid next to Sly (PLAYTEST 10-21 yWT6 / ds06; the r 330 48-tri record, cap_feel_hub
/// 000 door spot). SLY2_R75CASTVIS = old (all casters on).
fn hide_hidden_casters(sd: Res<SlyDraw>, mut casters: Query<(&crate::engine::gfx::shadowvol_pass::CasterRecord, &mut Visibility)>) {
    let Some(vis) = sd.vis.as_ref() else { return };
    if std::env::var_os("SLY2_R75CASTVIS").is_some() {
        return;
    }
    for (r, mut v) in &mut casters {
        let want = if vis.record_drawn(r.object, r.record, r.instance) { Visibility::Inherited } else { Visibility::Hidden };
        if *v != want {
            *v = want;
        }
    }
}

fn to_affine(m: &Mat4) -> Affine {
    [[m[0][0], m[0][1], m[0][2]], [m[1][0], m[1][1], m[1][2]], [m[2][0], m[2][1], m[2][2]], [m[3][0], m[3][1], m[3][2]]]
}

/// Once per world: build the rig and map the level's JT-subtree ALO entities, skin joints and rigid
/// prims to rig bones.
#[allow(clippy::type_complexity)]
/// Entities of the ALO subtree of `root` (spawned by spawn_level_geometry) posed from `rig`.
pub struct RigEntities {
    /// (entity, object, owner object (None = the root), rest level world), root excluded
    pub objects: Vec<(Entity, usize, Option<usize>, Affine)>,
    pub joints: Vec<(Entity, usize, usize)>,
    pub rigid: Vec<(Entity, Affine, usize, usize)>,
    pub prims: Vec<(Entity, usize, usize, Option<usize>)>,
    pub ink_skinned: HashSet<Entity>,
    pub root_rest: Affine,
}

/// The setup_sly_draw entity scan for any ALO root (WP-P23: shared with the Bentley / Murray draw).
pub fn rig_entities(
    w: &World,
    scene: &LevelScene,
    rig: &JtRig,
    jt: usize,
    objs: &Query<(Entity, &AloObject, Option<&Children>)>,
    meshes: &Query<(Has<InkOutline>, Option<&SkinnedMesh>), With<Mesh3d>>,
) -> RigEntities {
    let sub = subtree(w, jt);
    let (mut objects, mut joints, mut rigid, mut prims) = (vec![], vec![], vec![], vec![]);
    let mut ink_skinned = HashSet::new();
    let mut jt_rest = crate::engine::gfx::scene::IDENTITY;
    let mut bw = BoneWorlds::default();
    for (si, so) in scene.objects.iter().enumerate().filter(|(_, so)| sub.contains(&so.object)) {
        let Some((oe, _, Some(children))) = objs.iter().find(|(_, o, _)| o.0 == so.object) else { continue };
        let Payload::Loaded(LoadData::Alo { alo, .. }) = &w.objects[so.object].payload else { continue };
        if so.object == jt {
            jt_rest = scene.world_of(si);
        } else {
            objects.push((oe, so.object, so.owner.map(|o| scene.objects[o].object).filter(|&o| o != jt), scene.world_of(si)));
        }
        // spawn order: one Mesh3d per Surface prim (joints are plain entities, ink outlines tagged)
        // (surfaces and ink edges are spawned in so.prims order)
        let ents: Vec<(Entity, bool, Option<&SkinnedMesh>)> = children.iter().filter_map(|c| meshes.get(c).ok().map(|(ink, s)| (c, ink, s))).collect();
        use crate::engine::gfx::mesh::MeshRole;
        let all: Vec<_> = so.prims.iter().filter(|p| matches!(p.prim.role, MeshRole::Surface | MeshRole::InkEdges)).zip(ents).collect();
        // records drawn skinned (their ink edges cannot follow the skin: update_ink_outlines works
        // on the rest prim)
        let skinned: HashSet<(usize, Option<usize>)> = all.iter().filter(|(_, (_, ink, sm))| !ink && sm.is_some()).map(|(p, _)| (p.record, p.instance)).collect();
        for (p, (pe, ink, sm)) in all {
            prims.push((pe, so.object, p.record, p.instance));
            let rec = &alo.mesh.records[p.record];
            if ink && (skinned.contains(&(p.record, p.instance)) || !record_weight_refs(&rec.data).is_empty()) {
                // ANIM R72: skinned ink edges are posed (gfx::game::InkSkin) and drawn like the original's
                // VU-skinned type-1 records; SLY2_R72INKOFF: hidden (old)
                if std::env::var_os("SLY2_R72INKOFF").is_some() {
                    ink_skinned.insert(pe);
                }
                continue;
            }
            let Some(sm) = sm.filter(|_| !ink) else {
                // rigid prim: follows its record (or instance) bone (FUN_0022d6b0 ref)
                let (pre, r) = match p.instance {
                    Some(ii) => (rec.instances[ii].xform, rec.instances[ii].r#ref),
                    None => (crate::engine::gfx::scene::IDENTITY, rec.r#ref),
                };
                if let Some(&b) = (r >= 0).then(|| rig.index.get(&(so.object, r as usize))).flatten() {
                    rigid.push((pe, pre, b, so.object));
                }
                continue;
            };
            let refs = skin_joints(w, &mut bw, so.object, record_weight_refs(&rec.data)).refs;
            for (&e, r) in sm.joints.iter().zip(&refs) {
                if let Some(b) = rig.bone_of(r) {
                    joints.push((e, b, so.object));
                }
            }
        }
    }
    RigEntities { objects, joints, rigid, prims, ink_skinned, root_rest: jt_rest }
}

pub fn setup_sly_draw(
    world: Option<Res<World>>,
    data: Option<Res<GfxLevelData>>,
    mut sd: ResMut<SlyDraw>,
    objs: Query<(Entity, &AloObject, Option<&Children>)>,
    meshes: Query<(Has<InkOutline>, Option<&SkinnedMesh>), With<Mesh3d>>,
) {
    let Some(w) = world else { return };
    // WP-EP1f: a re-inserted World can reuse the old one's address (hub -> job-start mission load): drop the
    // previous world's draw state on the insert, rebuild once the level data is there
    if w.is_changed() {
        sd.world_key = 0;
    }
    let Some(data) = data else { return };
    let key = &*w as *const World as usize;
    if sd.world_key == key {
        return;
    }
    let Some(jt) = find_jt(&w) else {
        *sd = SlyDraw { world_key: key, ..default() };
        return;
    };
    // wait until spawn_level_geometry's entities (with their children) exist
    let Some(jt_entity) = objs.iter().find(|(_, o, c)| o.0 == jt && c.is_some()).map(|(e, ..)| e) else { return };
    let scene = LevelScene::build_from(data.lsn, &data.data, &w);
    let rig = JtRig::build(&w, jt);
    let RigEntities { objects, joints, rigid, prims, ink_skinned, root_rest: jt_rest } = rig_entities(&w, &scene, &rig, jt, &objs, &meshes);
    // WP-Q2: drawn state = JT bind tables + record bits (vis::JtVis::load); applied every frame
    let vis = JtVis::load(&w, jt);
    info!(
        "sly: JT object {jt}, {} rig bones, {} attached ALOs ({} removed), {} skin joints, {} rigid prims posed, {} hidden mesh records, {} ASEG members",
        rig.keys.len(),
        objects.len(),
        vis.removed.len(),
        joints.len(),
        rigid.len(),
        vis.hidden.len(),
        rig.members.len()
    );
    *sd = SlyDraw { world_key: key, animator: Some(JtAnimator::new(rig)), jt_entity: Some(jt_entity), objects, joints, rigid, jt_rest, vis: Some(vis), prims, calls_seen: 0, ink_skinned, ink_parked: HashMap::new(), applied: false };
}

/// Debug aid for captures (SLY_Q2_POSE=run|jump|stand): the field and jtbs requested every frame
/// instead of the player's 0x18c250 events.
fn forced_pose() -> Option<(u32, i32)> {
    match std::env::var("SLY_Q2_POSE").ok()?.as_str() {
        "run" => Some((FIELD_RUN, 0)),
        "jump" => Some((FIELD_JUMP, 1)),
        "stand" => Some((0, -1)),
        // WP-P23: any JT field "0x1950:3" (field:jtbs), e.g. the SM fields of 0x18c250 jtbs 3 / 4 / 10
        s => {
            let (f, b) = s.split_once(':')?;
            Some((u32::from_str_radix(f.trim_start_matches("0x"), 16).ok()?, b.parse().ok()?))
        }
    }
}

/// Every frame: 0x18c250 requests -> ASEG playback, pose -> attached ALOs, joints and rigid prims,
/// player -> JT entity transform.

/// FEEL: the JT animator part of `drive_sly` for one frame (state requests from the JT's
/// `JtAnimEvents`, blends, SM), before `JtAnimator::step`. Shared with the pad-driven tests.
pub fn sly_anim_frame(w: &World, an: &mut JtAnimator, jt: &crate::engine::player::Jt, evs: &[(u32, i32, i32)]) {
    let pl_jt = jt;
    if let Some((field, jtbs)) = forced_pose() {
        if field == 0 {
            an.request_sm(w, SM_LABEL_STAND);
        } else {
            an.request(w, field, jtbs);
        }
        an.jtbs = jtbs;
    } else {
        // the JT enters its first state from jts/jtbs -1 (FUN_00188488), so 0x18c250 requests the
        // state's field once at spawn: run for jts 1, the jump blend for jtbs 0..2
        if an.current.is_none() {
            if pl_jt.jts == JTS_RUN {
                an.request(w, FIELD_RUN, pl_jt.jtbs);
            }
            // ANIM-ALL R20: the animator can start after the JT entered its state (setup_sly_draw waits for the
            // level entities): the jump blend also covers the fall (jtbs 4, FUN_00190520), and the JMP body ASEG of
            // 0x195fb8's tail (ledge hang 0x11, double jump, glide) is requested as the entry did (else bind pose)
            if (0..=2).contains(&pl_jt.jtbs) || pl_jt.jtbs == crate::engine::player::JTBS_JUMP_FALL {
                an.request(w, FIELD_JUMP, pl_jt.jtbs);
            }
            if let Some(f) = crate::engine::player::jmp_body_aseg_195fb8(pl_jt.jtbs, pl_jt.jmp_idx) {
                an.request(w, f, pl_jt.jtbs);
            }
            // standing (jts 0 after FUN_00188488): 0x18c250 jts 0 -> FUN_001f03a8(JT +0x1028, 0x175)
            if an.current.is_none() && pl_jt.jts == 0 {
                an.request_sm(w, SM_LABEL_STAND);
            }
        }
        crate::engine::player::p15_anim::jt_blends_build(w, an);
        for &(field, _jts, jtbs) in evs {
            if crate::engine::player::p15_anim::anim_log() {
                eprintln!("P17 anim: request field {field:#x} jts {_jts} jtbs {jtbs} blend {}", an.blends.contains_key(&field));
            }
            an.request(w, field, jtbs);
            if !(0x1ce0..=0x1d18).contains(&field) {
                an.state_req = Some((field, jtbs));
            }
        }
        // ANIM-ALL R84: script goals of the JT SMA (pseudo handle super::JT_SMA_HANDLE)
        if super::jt_sma_on() {
            an.jt_sma_goals(w);
        }
        // ANIM-ALL R77: a jtbs entered through a sink-less 0x195fb8 caller (FEEL R81 hop: FUN_0019f540(.., &mut ()),
        // jmp_set_jtbs_195fb8) still plays its JMP body ASEG: the original's 0x195fb8 tail always starts it in
        // jt+0x1030 (FUN_001305b8), whoever calls. Opt-out SLY2_R77HOPOFF.
        if pl_jt.jtbs != an.jtbs && std::env::var_os("SLY2_R77HOPOFF").is_none() {
            if let Some(f) = crate::engine::player::jmp_body_aseg_195fb8(pl_jt.jtbs, pl_jt.jmp_idx).filter(|f| !evs.iter().any(|e| e.0 == *f)) {
                if crate::engine::player::p15_anim::anim_log() {
                    eprintln!("P17 anim: R77 body ASEG {f:#x} on jtbs {} (no 0x18c250 event)", pl_jt.jtbs);
                }
                an.request(w, f, pl_jt.jtbs);
                an.state_req = Some((f, pl_jt.jtbs));
            }
        }
        crate::engine::player::p15_anim::jt_blends_drive(an, &pl_jt);
        // jts 0 entry (the P4 port sends no request for it): 0x18c250 -> FUN_001f03a8(JT +0x1028, 0x175)
        // WP-P19: also after landing from a jump/pull-up (the jump blend 0x1994 stayed current)
        // ANIM-ALL R68: 0x18c250 case 0 starts the stand SM on every jts-0 entry. Original: all 125 jts-0 savestates of
        // rec_ep1d/rec_ep1b play the SM (no *(jt+0x1030) field). The release game held the double-jump ASEG 0x3f4, the
        // dive attack 0x1a80 (120 frames after landing) and the sneak/walk blend 0x1c34 in jts 0 -> any non-SM play in
        // jts 0 except the cane ASEGs (jt+0x18e0 in the original, merged here) goes back to the stand SM.
        // Opt-out SLY2_R68STANDOFF.
        let cane = an.current_field().is_some_and(|f| (0x1ce0..=0x1d18).contains(&f));
        if pl_jt.jts == 0 && (matches!(an.current_field(), Some(FIELD_RUN) | Some(FIELD_JUMP) | Some(0x3e0..=0x3f0)) || (std::env::var_os("SLY2_R68STANDOFF").is_none() && an.sm.is_none() && an.current.is_some() && !cane)) {
            an.request_sm(w, SM_LABEL_STAND);
        }
        // FEEL C1/C3: FUN_0018e540(-1) restarts the cane ASEGA jt+0x18e0 with no ASEG (FUN_001305b8), so the
        // state's own animation shows again; the port has one animator, so the state field is re-requested
        // (else the ended cane ASEG loops = Sly keeps attacking / stays in the charge stance)
        if pl_jt.cane.jtcs == -1 && an.current_field().is_some_and(|f| (0x1ce0..=0x1d18).contains(&f)) {
            match pl_jt.jts {
                0 => an.request_sm(w, SM_LABEL_STAND),
                JTS_RUN => an.request(w, FIELD_RUN, pl_jt.jtbs),
                // FEEL R84 (USER PLAYTEST 20:30 'attack never stops, swing sound loops'): any other state (walk
                // jts 0xc 0x1c34, jts 5 / 7 ...) shows its own jt+0x1030 animation again: re-request the state's
                // last field (else the walking swing ASEG 3051 looped and fired its sound 11 every 0.5 s).
                // SLY2_FEEL_R84CANE=0 = old (only jts 0 / 1).
                _ if std::env::var("SLY2_FEEL_R84CANE").map_or(true, |v| v != "0") => {
                    if let Some((f, b)) = an.state_req {
                        an.request(w, f, b);
                    }
                }
                _ => {}
            }
        }
        an.jtbs = pl_jt.jtbs;
    }
    an.sm_tick(w);
    an.glider_init(w);
    an.glider_tick(w);
}

/// ANIM-ALL R18: [`crate::engine::npcdraw::plugin::guest_locals_overlay`] for the JT rig, limited to the ASEGAs
/// held in the animator's guest slots (body / glider); 0 when none is held (native-only blend fields).
pub fn jt_guest_locals(m: &mut crate::engine::classes::q::live::LiveMem, an: &mut JtAnimator) -> usize {
    let slots: Vec<u32> = [super::GUEST_SLOT_BODY, super::GUEST_SLOT_GLIDER].iter().filter_map(|&k| m.anim_slots.get(&((an.rig.jt as u32) << 2 | k)).copied()).collect();
    let mine: Vec<u32> = slots.into_iter().map(|s| crate::engine::classes::q::q2mem::QMem::r32(m, s)).filter(|&h| h != 0).collect();
    if mine.is_empty() {
        // ANIM-ALL R78: native-only blend fields still take the FUN_0021cb18 spring bones (mask tail) from
        // the guest pose pass (no anim slot matches u32::MAX; spring bones bypass the slot filter)
        if std::env::var_os("SLY2_R78TAILOFF").is_some() {
            return 0;
        }
        return crate::engine::npcdraw::plugin::guest_locals_overlay(m, &an.rig.keys, &mut an.rig.pose, &[u32::MAX]);
    }
    crate::engine::npcdraw::plugin::guest_locals_overlay(m, &an.rig.keys, &mut an.rig.pose, &mine)
}

/// ANIM-ALL R15/R20: the animator's FUN_001305b8 / FUN_00130658 calls (`an.guest`) on the guest slots, then the
/// started ASEGAs get this frame's update + pose (`LiveQ::guest_slot_settle`), as in the original frame order.
pub fn jt_guest_starts(lq: &mut crate::engine::classes::q::live::LiveQ, an: &JtAnimator, guest: Vec<(u32, Option<(f32, f32, usize, u32)>)>) {
    let jt = an.rig.jt as u32;
    let mut started = vec![];
    for (slot, r) in guest {
        let start = r.is_some();
        if lq.guest_slot_play(jt << 2 | slot, r.map(|(t, svt, o, fl)| (t, svt, o as u32 + 1, jt + 1, fl))) && start {
            started.push(jt << 2 | slot);
        }
    }
    if !started.is_empty() {
        let mut objs: Vec<usize> = an.rig.keys.iter().map(|k| k.0).collect();
        objs.dedup();
        lq.guest_slot_settle(&started, &objs);
    }
}

pub fn drive_sly(
    time: Res<Time>,
    world: Option<Res<World>>,
    mut sd: ResMut<SlyDraw>,
    mut players: Query<(&mut Player, &JtAnimEvents)>,
    mut tf: Query<&mut Transform>,
    mut vq: Query<&mut Visibility>,
    inks: Query<&InkOutline>,
    mut commands: Commands,
    fired: Option<ResMut<PlayerAnimFired>>,
    mut liveq: Option<ResMut<crate::engine::classes::q::live::LiveQ>>,
    clock: Option<Res<crate::engine::player::plugin::PlayerClock>>,
) {
    let Some(w) = world else { return };
    // WP-EP1f: a new World whose draw is not set up yet (setup_sly_draw waits for its GfxLevelData): the
    // animator / vis still index the previous world (job-start load right after the hub load)
    if sd.world_key != &*w as *const World as usize {
        return;
    }
    let Some((mut pl, ev)) = players.iter_mut().next() else { return };
    // FEEL F1: FUN_0020b868 entry root velocities of the run blend (0x1334 / 0x1374) for the JT motor
    if pl.jt.run_root[0].is_none() {
        if let Some(an) = sd.animator.as_ref() {
            let e = |m: u32| an.rig.members.get(&m).and_then(|&o| crate::engine::player::p22::loco_entry_of(&w, o));
            pl.jt.run_root = [e(0x1334), e(0x1374)];
            pl.jt.walk_root = [e(0x1c44), e(0x1c84)];
        }
    }
    let sd = &mut *sd;
    // WP-P24: JT vt+0x2bc FUN_00189530 (binocucom: Sly not drawn while looking)
    if let Some(vis) = sd.vis.as_mut() {
        if vis.hide_189530 != pl.jt.binoc.hidden {
            vis.ee_sub_189530(&w, pl.jt.binoc.hidden);
            sd.applied = false;
        }
    }
    apply_visibility(&w, sd, &pl.jt.calls, &mut vq, &inks, &mut commands);
    let Some(an) = sd.animator.as_mut() else { return };
    // WIRE-AN: FUN_00142af8 look target -> FUN_001430c8 idle mask (FUN_0018a0c8 per jts-0 frame)
    an.look_update(&w, pl.jt.phys.pos, &pl.jt.phys.rot, pl.jt.joy.angle, pl.jt.joy.mag, pl.jt.jts == 0);
    sly_anim_frame(&w, an, &pl.jt, &ev.0);
    // ANIM-ALL R15: DAT_002dded4 (g_clock FUN_0014d0b8, real frame time clamped to [1/60, 1/30]) as the JT
    // update's FUN_00130918 uses, not the unclamped Bevy delta
    an.step(clock.as_deref().map_or(time.delta_secs(), |c| c.dt));
    if let Some(mut f) = fired {
        f.0.extend(an.fired.iter().copied());
    }
    // WP-P24: event kinds 7 / 8 (FUN_00210338, f == 0 path) on the JT subtree: vtable +0x44 / +0x4c
    if let Some(vis) = sd.vis.as_mut() {
        if super::glider::apply_e7(&w, vis, &an.fired) {
            sd.applied = false;
        }
    }
    // ANIM-ALL R15: the JT's ASEGA starts / releases (FUN_001305b8 / FUN_00130658 on its ASEGA fields) go to the
    // guest image too, so the original FUN_00130918 / FUN_0022de88 pose the JT bones (drawn below)
    let guest = std::mem::take(&mut an.guest);
    if let Some(lq) = liveq.as_deref_mut() {
        jt_guest_starts(lq, an, guest);
    }
    // ANIM-ALL R15/R16 (on by default, SLY2_GUEST_JT=0 off, see `guest_jt_on`): bones with guest acts take the guest locals (FUN_0022de88), recomposed natively, then the JT IK
    // (FUN_00186a50) again; the rest keep the native pose
    if crate::engine::npcdraw::plugin::guest_jt_on() {
        if let Some(m) = liveq.as_deref_mut().and_then(|lq| lq.mem.as_mut()) {
            // ANIM-ALL R18: only the acts of the ASEGAs this animator mirrors (body / glider slots): other JT ASEGAs
            // in the image (class-code starts the native animator does not play) kept the bones while a native-only
            // blend (walk 0x1c34) ran -> Sly in the old pose / T-pose while walking (cellar bug)
            let n = jt_guest_locals(m, an);
            if n > 0 {
                an.rig.pose.recompose(&crate::engine::anim::pose::IDENTITY);
                super::ik::apply(an);
            }
        }
    }
    let worlds = an.rig.pose.world();
    let jt = an.rig.jt;
    // posed object root (bone 0) in rest level space; the JT keeps its rest root (its entity
    // carries the player transform)
    let rest_of = |o: usize| sd.objects.iter().find(|x| x.1 == o).map(|x| x.3);
    let posed = |o: usize| -> Option<Affine> {
        if o == jt {
            return None;
        }
        an.rig.index.get(&(o, 0)).map(|&b| to_affine(&worlds[b])).or_else(|| rest_of(o))
    };
    let jt_rest = Some(sd.jt_rest);
    let frame = |o: usize| -> Option<Affine> { if o == jt { jt_rest } else { posed(o) } };
    for &(e, o, owner, _) in &sd.objects {
        let (Some(p), Some(q)) = (posed(o), frame(owner.unwrap_or(jt))) else { continue };
        if let Ok(mut t) = tf.get_mut(e) {
            *t = affine_to_transform(&mul(&p, &invert(&q)));
        }
    }
    for &(e, b, o) in &sd.joints {
        let Some(f) = frame(o) else { continue };
        if let Ok(mut t) = tf.get_mut(e) {
            *t = affine_to_transform(&mul(&to_affine(&worlds[b]), &invert(&f)));
        }
    }
    for &(e, pre, b, o) in &sd.rigid {
        let Some(f) = frame(o) else { continue };
        if let Ok(mut t) = tf.get_mut(e) {
            *t = affine_to_transform(&mul(&mul(&pre, &to_affine(&worlds[b])), &invert(&f)));
        }
    }
    if let Some(mut t) = sd.jt_entity.and_then(|e| tf.get_mut(e).ok()) {
        let r = pl.jt.phys.rot;
        let mut p = pl.jt.phys.pos;
        // ANIM-ALL R67 (USER PLAYTEST 4 'Sly floats'): the original draws the JT at its foot point *(jt+0x218) (rec_ep1b
        // 076 standing: *(+0x218).z -5.48, contact +0x238 -5.08, bones [+0x58]+0x30 69.52 = +75 local, the rig's 75.0),
        // the port's pos is the collision sphere centre (contact + radius; hub spawn: port 491.97 vs orig 536.88 - 75 =
        // 461.88) -> the skin was drawn one radius (30) too high. Not while hanging / attached (their offsets are placed
        // from the posed rig). Opt-out SLY2_R67FOOTOFF.
        if pl.jt.jtbs != crate::engine::player::JTBS_LEDGE && pl.jt.jts != 7 && std::env::var_os("SLY2_R67FOOTOFF").is_none() {
            p[2] -= pl.jt.body_radius();
        }
        // WP-P22: ledge hang: wrists on the ledge contact (REPLACED, super::hang_grip_offset)
        if pl.jt.jtbs == crate::engine::player::JTBS_LEDGE {
            if let Some(o) = super::hang_grip_offset(&an.rig, &worlds) {
                for i in 0..3 {
                    p[i] += o[0] * r[0][i] + o[1] * r[1][i] + o[2] * r[2][i];
                }
            }
        }
        *t = affine_to_transform(&[r[0], r[1], r[2], p]);
    }
}

/// WP-P20: the controlled Bentley / Murray (`player::p20::PoCurrent`): its ALO tree is drawn at the shadow JT
/// body and Sly's JT tree is hidden (REPLACED, see player/p20.rs). WP-P23: posed from its own anims
/// (super::po::PoAnimator over the class's ASEGAs on the K25 run time and its FUN_0020bb58 run blend).
#[allow(clippy::too_many_arguments)]
pub fn drive_po(
    cur: Option<Res<crate::engine::player::p20::PoCurrent>>,
    w: Option<Res<World>>,
    sd: Res<SlyDraw>,
    players: Query<&Player>,
    objs: Query<(Entity, &AloObject, Option<&Children>)>,
    data: Option<Res<GfxLevelData>>,
    meshes: Query<(Has<InkOutline>, Option<&SkinnedMesh>), With<Mesh3d>>,
    time: Res<Time>,
    mut tf: Query<&mut Transform>,
    mut vq: Query<&mut Visibility>,
    mut shown: Local<PoVisApplied>,
    inks: Query<&InkOutline>,
    mut commands: Commands,
    fired: Option<ResMut<PlayerAnimFired>>,
    mut liveq: Option<ResMut<crate::engine::classes::q::live::LiveQ>>,
    clock: Option<Res<crate::engine::player::plugin::PlayerClock>>,
) {
    // WP-EP1f: only a selection made for the current World (po_link redoes it after a load)
    let wk = w.as_deref().map_or(0, |w| w as *const World as usize);
    let fresh = w.as_ref().is_some_and(|w| w.is_changed());
    let cur_o = cur.as_ref().filter(|c| c.world_key == wk && !fresh).and_then(|c| c.obj);
    // WP-P22: the po's drawn state (vis::JtVis::load_po) applied once per (world, po)
    let key = (w.as_deref().map_or(0, |w| w as *const World as usize), cur_o);
    if shown.key != key {
        for &e in &shown.hidden {
            if let Ok(mut v) = vq.get_mut(e) {
                *v = Visibility::Inherited;
            }
        }
        shown.hidden.clear();
        shown.draw = None;
        for (e, ink) in shown.inks.drain(..) {
            commands.entity(e).insert(ink);
        }
        // switched back to Sly (p22::follow_active): his JT tree is drawn again
        if cur_o.is_none() && shown.key.1.is_some() {
            if let Some(mut v) = sd.jt_entity.and_then(|e| vq.get_mut(e).ok()) {
                *v = Visibility::Inherited;
            }
        }
        if let (Some(w), Some(o)) = (w.as_deref(), cur_o) {
            let table: &[(u32, i32, u32)] = match w.objects[o].class {
                crate::engine::player::p20::CLASS_BENTLEY => &super::vis::PO_BIND_2BC390,
                crate::engine::player::p20::CLASS_MURRAY => &super::vis::PO_BIND_2C0270,
                _ => &[],
            };
            let vis = JtVis::load_po(w, o, table);
            let mut hide = vec![];
            for (e, a, _) in &objs {
                if vis.removed.contains(&a.0) {
                    hide.push(e);
                }
            }
            // hidden mesh records: the object's Mesh3d children in prim order (as setup_sly_draw)
            if let Some(data) = data.as_deref().filter(|_| !vis.hidden.is_empty()) {
                use crate::engine::gfx::mesh::MeshRole;
                let scene = LevelScene::build_from(data.lsn, &data.data, w);
                let sub: HashSet<usize> = subtree(w, o).into_iter().collect();
                for so in scene.objects.iter().filter(|so| sub.contains(&so.object)) {
                    let Some((_, _, Some(children))) = objs.iter().find(|(_, a, _)| a.0 == so.object) else { continue };
                    let ents = children.iter().filter(|c| meshes.contains(*c));
                    for (p, e) in so.prims.iter().filter(|p| matches!(p.prim.role, MeshRole::Surface | MeshRole::InkEdges)).zip(ents) {
                        if !vis.record_drawn(so.object, p.record, p.instance) {
                            hide.push(e);
                        }
                    }
                }
            }
            for e in hide {
                if let Ok(mut v) = vq.get_mut(e) {
                    *v = Visibility::Hidden;
                    shown.hidden.push(e);
                }
                // ink edges: parked while hidden (update_ink_outlines re-shows every outline)
                if let Ok(ink) = inks.get(e) {
                    shown.inks.push((e, ink.clone()));
                    commands.entity(e).remove::<InkOutline>();
                }
            }
            // entities spawn after the selection: retry until the po's own entity exists
            if objs.iter().any(|(_, a, c)| a.0 == o && c.is_some()) {
                shown.key = key;
                // WP-P23: the po's rig and posed entities; skinned ink edges are not drawn (as Sly's)
                if let Some(data) = data.as_deref() {
                    let scene = LevelScene::build_from(data.lsn, &data.data, w);
                    let an = super::po::PoAnimator::new(w, o);
                    let ents = rig_entities(w, &scene, &an.rig, o, &objs, &meshes);
                    for &e in &ents.ink_skinned {
                        if let Ok(mut v) = vq.get_mut(e) {
                            *v = Visibility::Hidden;
                            shown.hidden.push(e);
                        }
                        if let Ok(ink) = inks.get(e) {
                            shown.inks.push((e, ink.clone()));
                            commands.entity(e).remove::<InkOutline>();
                        }
                    }
                    info!("P23 po draw: object {o}, {} rig bones, {} blends, {} skin joints, {} rigid prims, {} attached ALOs", an.rig.keys.len(), an.blends.len(), ents.joints.len(), ents.rigid.len(), ents.objects.len());
                    shown.draw = Some((an, ents));
                }
            }
        } else {
            shown.key = key;
        }
    }
    let Some(o) = cur_o else { return };
    let Some(pl) = players.iter().next() else { return };
    if let Some(mut v) = sd.jt_entity.and_then(|e| vq.get_mut(e).ok()) {
        *v = Visibility::Hidden;
    }
    let Some(e) = objs.iter().find(|(_, a, _)| a.0 == o).map(|(e, ..)| e) else { return };
    if let Ok(mut t) = tf.get_mut(e) {
        let r = pl.jt.phys.rot;
        *t = affine_to_transform(&[r[0], r[1], r[2], pl.jt.phys.pos]);
    }
    // WP-P23: pose from the class's own anims
    let (Some(w), Some((an, ents))) = (w.as_deref(), shown.draw.as_mut()) else { return };
    let Some(cur) = cur.as_ref() else { return };
    an.plain = cur.plain;
    an.step(w, &cur.plays, cur.bb58, clock.as_deref().map_or(time.delta_secs(), |c| c.dt));
    if let Some(mut f) = fired {
        f.0.extend(an.fired.iter().copied());
    }
    pose_entities(&mut an.rig, ents, o, &mut tf, liveq.as_deref_mut().and_then(|lq| lq.mem.as_mut()));
}

/// CRAWL R111: per-world draw state of the uncontrolled Bentley / Murray (`PoCurrent::idle`).
#[derive(Default)]
pub struct IdlePoDraw {
    world: usize,
    hidden: Vec<Entity>,
    draw: HashMap<usize, (super::po::PoAnimator, RigEntities)>,
}

/// CRAWL R111: the uncontrolled in-world Bentley / Murray are drawn at their K27 body (the original draws
/// every in-world character ALO; script 249.1 s2 seats them at the safehouse table, rec_ep1b 064), posed from
/// their own run-time ASEGAs / plain blend like the controlled po (`drive_po`), same JtVis part table.
#[allow(clippy::too_many_arguments)]
pub fn drive_idle_po(
    cur: Option<Res<crate::engine::player::p20::PoCurrent>>,
    w: Option<Res<World>>,
    objs: Query<(Entity, &AloObject, Option<&Children>)>,
    data: Option<Res<GfxLevelData>>,
    meshes: Query<(Has<InkOutline>, Option<&SkinnedMesh>), With<Mesh3d>>,
    time: Res<Time>,
    mut tf: Query<&mut Transform>,
    mut vq: Query<&mut Visibility>,
    mut st: Local<IdlePoDraw>,
    inks: Query<&InkOutline>,
    mut commands: Commands,
    mut liveq: Option<ResMut<crate::engine::classes::q::live::LiveQ>>,
    clock: Option<Res<crate::engine::player::plugin::PlayerClock>>,
) {
    let (Some(w), Some(cur)) = (w.as_deref(), cur.as_deref()) else { return };
    let wk = w as *const World as usize;
    if st.world != wk {
        *st = IdlePoDraw { world: wk, ..Default::default() };
    }
    if cur.world_key != wk {
        return;
    }
    let dt = clock.as_deref().map_or(time.delta_secs(), |c| c.dt);
    for ip in &cur.idle {
        let o = ip.obj;
        let Some(e) = objs.iter().find(|(_, a, c)| a.0 == o && c.is_some()).map(|(e, ..)| e) else { continue };
        if !st.draw.contains_key(&o) {
            let Some(data) = data.as_deref() else { continue };
            let table: &[(u32, i32, u32)] = crate::engine::player::p23::bind_table(w.objects[o].class);
            let vis = JtVis::load_po(w, o, table);
            let mut hide: Vec<Entity> = objs.iter().filter(|(_, a, _)| vis.removed.contains(&a.0)).map(|(e, ..)| e).collect();
            let scene = LevelScene::build_from(data.lsn, &data.data, w);
            if !vis.hidden.is_empty() {
                use crate::engine::gfx::mesh::MeshRole;
                let sub: HashSet<usize> = subtree(w, o).into_iter().collect();
                for so in scene.objects.iter().filter(|so| sub.contains(&so.object)) {
                    let Some((_, _, Some(children))) = objs.iter().find(|(_, a, _)| a.0 == so.object) else { continue };
                    let ents = children.iter().filter(|c| meshes.contains(*c));
                    for (p, me) in so.prims.iter().filter(|p| matches!(p.prim.role, MeshRole::Surface | MeshRole::InkEdges)).zip(ents) {
                        if !vis.record_drawn(so.object, p.record, p.instance) {
                            hide.push(me);
                        }
                    }
                }
            }
            let an = super::po::PoAnimator::new(w, o);
            let ents = rig_entities(w, &scene, &an.rig, o, &objs, &meshes);
            hide.extend(ents.ink_skinned.iter().copied());
            for h in hide {
                if let Ok(mut v) = vq.get_mut(h) {
                    *v = Visibility::Hidden;
                    st.hidden.push(h);
                }
                if inks.get(h).is_ok() {
                    commands.entity(h).remove::<InkOutline>();
                }
            }
            info!("CRAWL R111 idle po draw: object {o} at {:?}, {} rig bones", ip.pos, an.rig.keys.len());
            st.draw.insert(o, (an, ents));
        }
        if let Ok(mut t) = tf.get_mut(e) {
            *t = affine_to_transform(&[ip.rot[0], ip.rot[1], ip.rot[2], ip.pos]);
        }
        let Some((an, ents)) = st.draw.get_mut(&o) else { continue };
        an.plain = ip.plain;
        an.step(w, &ip.plays, None, dt);
        pose_entities(&mut an.rig, ents, o, &mut tf, liveq.as_deref_mut().and_then(|lq| lq.mem.as_mut()));
    }
}

/// WP-P23: timed ASEG events of the player's anims (Sly's JtAnimator, the po's PoAnimator) this frame:
/// (ASEG object, event index, reverse).
#[derive(Resource, Default, Debug)]
pub struct PlayerAnimFired(pub Vec<(usize, usize, bool)>);

/// WP-P23: FUN_00131290 for the player's anim events (the level ASEGAs go through game::anim_play): the
/// same receiver (`game::anim_play::LevelEffectSink`: case 0 FUN_00210740 sound, 2 FUN_0014f750 shake,
/// 3 FUN_00187e28 rumble, 4 ASEG start); calls it does not execute are counted in `PlayerAnimDispatch`.
#[derive(Resource, Default, Debug)]
pub struct PlayerAnimDispatch {
    pub calls: u64,
    pub sounds: u64,
    pub pending: Vec<crate::engine::anim::events::EffectCall>,
}

#[allow(clippy::too_many_arguments)]
pub fn dispatch_player_events(
    time: Res<Time>,
    world: Option<Res<World>>,
    mut fired: ResMut<PlayerAnimFired>,
    mut out: ResMut<PlayerAnimDispatch>,
    emitters: Option<ResMut<crate::engine::audio::emitter::EmitterSystem>>,
    rumble: Option<ResMut<crate::engine::game::app::RumbleQueue>>,
    game: Option<ResMut<crate::engine::game::app::LevelGame>>,
    cm: Option<ResMut<Cm>>,
) {
    let ev = std::mem::take(&mut fired.0);
    let (Some(w), Some(mut emitters), Some(mut rumble)) = (world, emitters, rumble) else { return };
    if ev.is_empty() {
        return;
    }
    let dt = time.delta_secs();
    let clk = CmClock { t: time.elapsed_secs(), dt, dt_real: dt };
    let mut sink = crate::engine::game::anim_play::LevelEffectSink { world: &w, emitters: &mut emitters, rumble: &mut rumble.0, cm: cm.map(|c| c.into_inner()), clk, current: 0, sounds: vec![], start: vec![], pending: vec![] };
    for (o, i, rev) in ev {
        let Some(a) = super::aseg_of(&w, o) else { continue };
        sink.current = o;
        out.calls += crate::engine::player::p23::dispatch_events(a, &[(i, rev)], &mut sink) as u64;
    }
    let crate::engine::game::anim_play::LevelEffectSink { sounds, start, pending, .. } = sink;
    out.sounds += sounds.len() as u64;
    if crate::engine::player::p15_anim::anim_log() {
        for s in &sounds {
            eprintln!("P23 anim event: sound {} from ASEG {}", s.sound, s.aseg);
        }
    }
    if let Some(mut g) = game {
        for i in start {
            g.anims.play(&w, i, 1.0);
        }
    }
    out.pending.extend(pending);
    let n = out.pending.len();
    if n > 256 {
        out.pending.drain(..n - 256);
    }
}

/// WP-P23: posed rig -> entity transforms of a subtree drawn from its root's entity (the drive_sly scheme:
/// attached ALOs follow their bone (object, 0), skin joints / rigid prims = bone world * inverse(owner frame)).
/// ANIM-ALL R15: with `guest` (SLY2_GUEST_POSE on) the bones of objects with guest acts come from the guest pose
/// (the po's class-code FUN_0012f9b8 starts reach the image through K25 `guest_reqs`), the rest stay native.
pub fn pose_entities(rig: &mut JtRig, ents: &RigEntities, root: usize, tf: &mut Query<&mut Transform>, guest: Option<&mut crate::engine::classes::q::live::LiveMem>) {
    if let Some(m) = guest.filter(|_| crate::engine::npcdraw::plugin::guest_jt_on()) {
        if crate::engine::npcdraw::plugin::guest_locals_overlay(m, &rig.keys, &mut rig.pose, &[]) > 0 {
            rig.pose.recompose(&crate::engine::anim::pose::IDENTITY);
        }
    }
    let rig = &*rig;
    let worlds = rig.pose.world();
    let rest_of = |o: usize| ents.objects.iter().find(|x| x.1 == o).map(|x| x.3);
    let posed = |o: usize| -> Option<Affine> {
        if o == root {
            return None;
        }
        rig.index.get(&(o, 0)).map(|&b| to_affine(&worlds[b])).or_else(|| rest_of(o))
    };
    let frame = |o: usize| -> Option<Affine> { if o == root { Some(ents.root_rest) } else { posed(o) } };
    for &(e, o, owner, _) in &ents.objects {
        let (Some(p), Some(q)) = (posed(o), frame(owner.unwrap_or(root))) else { continue };
        if let Ok(mut t) = tf.get_mut(e) {
            *t = affine_to_transform(&mul(&p, &invert(&q)));
        }
    }
    for &(e, b, o) in &ents.joints {
        let Some(f) = frame(o) else { continue };
        if let Ok(mut t) = tf.get_mut(e) {
            *t = affine_to_transform(&mul(&to_affine(&worlds[b]), &invert(&f)));
        }
    }
    for &(e, pre, b, o) in &ents.rigid {
        let Some(f) = frame(o) else { continue };
        if let Ok(mut t) = tf.get_mut(e) {
            *t = affine_to_transform(&mul(&mul(&pre, &to_affine(&worlds[b])), &invert(&f)));
        }
    }
}

/// WP-P22: entities `drive_po` hid for the po of `key` (world, po object).
#[derive(Default)]
pub struct PoVisApplied {
    key: (usize, Option<usize>),
    hidden: Vec<Entity>,
    inks: Vec<(Entity, InkOutline)>,
    /// WP-P23: the po's animator and posed entities
    draw: Option<(super::po::PoAnimator, RigEntities)>,
}

/// JT +0x18d8 changes (FUN_0018e540 calls the player records in Jt::calls) and the drawn state of the
/// subtree's objects and prims. Debug: SLY_Q2_CARRY=n forces FUN_0018e540(n); SLY_SHOW_ALL draws all.
fn apply_visibility(w: &World, sd: &mut SlyDraw, calls: &[(u32, i32)], vq: &mut Query<&mut Visibility>, inks: &Query<&InkOutline>, commands: &mut Commands) {
    let Some(vis) = sd.vis.as_mut() else { return };
    if calls.len() < sd.calls_seen {
        sd.calls_seen = 0;
    }
    for &(a, n) in &calls[sd.calls_seen..] {
        if a == 0x0018e540 {
            vis.ee_sub_18e540(w, n);
            sd.applied = false;
        }
        // WP-P24: 0x195fb8 FUN_001f0660(JT +0x1d28 glider SMA, 0x5f7 / 0x5f8)
        if a == 0x001f0660 {
            if let Some(an) = sd.animator.as_mut() {
                an.glider_goal(w, n);
            }
        }
    }
    sd.calls_seen = calls.len();
    if let Some(n) = std::env::var("SLY_Q2_CARRY").ok().and_then(|v| v.parse().ok()) {
        if vis.carry != n {
            vis.ee_sub_18e540(w, n);
            sd.applied = false;
        }
    }
    if sd.applied {
        return;
    }
    sd.applied = true;
    let all = std::env::var("SLY_SHOW_ALL").is_ok();
    let set = |vq: &mut Query<&mut Visibility>, e: Entity, on: bool| {
        if let Ok(mut v) = vq.get_mut(e) {
            *v = if on || all { Visibility::Inherited } else { Visibility::Hidden };
        }
    };
    for &(e, o, ..) in &sd.objects {
        set(vq, e, vis.drawn(o));
    }
    for &(e, o, r, i) in &sd.prims {
        let on = (vis.record_drawn(o, r, i) || all) && !sd.ink_skinned.contains(&e);
        set(vq, e, on);
        // ink edges: park the InkOutline while hidden (update_ink_outlines re-shows every outline)
        if on {
            if let Some(ink) = sd.ink_parked.remove(&e) {
                commands.entity(e).insert(ink);
            }
        } else if let Ok(ink) = inks.get(e) {
            sd.ink_parked.insert(e, ink.clone());
            commands.entity(e).remove::<InkOutline>();
        }
    }
}

/// Follow camera on the player (pushed once per world when the player exists).
/// FEEL R25: jt +0x1048 camera height over the ground contact (125.0 in every rec5 state; writer not traced)
const JT_CAM_1048: f32 = 125.0;
pub fn follow_camera(
    time: Res<Time>,
    world: Option<Res<World>>,
    focus: Res<PlayerFocus>,
    pad: Res<PlayerPad>,
    pwd: Option<Res<crate::engine::player::plugin::PlayerWorldData>>,
    mut sf: ResMut<SlyFollow>,
    mut cm: ResMut<Cm>,
    mut liveq: Option<ResMut<crate::engine::classes::q::live::LiveQ>>,
    st: Option<NonSend<crate::engine::game::vm_state::SpliceState>>,
) {
    let Some(w) = world else { return };
    let _lend = (std::env::var_os("SLY2_FEEL_NOCAMMEM").is_none())
        .then(|| liveq.as_deref_mut().and_then(|lq| lq.mem.as_mut()))
        .flatten()
        .map(|m| {
            sf.live.store(m as *mut _, std::sync::atomic::Ordering::Release);
            LendMem(sf.live.clone())
        });
    if !focus.valid {
        return;
    }
    let mut focus = *focus;
    let key = &*w as *const World as usize;
    if sf.world_key != key {
        sf.jt_cam_z = [0.0; 2];
    }
    if time.delta_secs() > 0.0 {
        // FEEL R25: 0x00141640 (+0x1044 == 0, +0x434 == 0, +0x2dc not 0x14/0x16; the JT is the cm focus)
        let (pz, cut, dt) = (focus.pos[2], cm.cut != 0, time.delta_secs());
        let s58 = if focus.ground {
            focus.z_contact + JT_CAM_1048
        } else if cut || std::env::var_os("SLY2_FEEL_NOJTCAMZ").is_some() {
            // REPLACED pos.z + jt +0x248 by the sphere bottom
            pz - crate::engine::player::JT_RADIUS + JT_CAM_1048
        } else {
            sf.jt_cam_z[0]
        };
        let z4c = if pz - 275.0 <= s58 { if pz + 150.0 < s58 { pz + 150.0 } else { s58 } } else { pz - 275.0 };
        let z18 = if cut {
            z4c
        } else {
            let z = crate::engine::physics::dynamics::FUN_0011d018(sf.jt_cam_z[1], z4c, dt, [4000.0, 0.0, 4.0], None);
            if pz - 300.0 <= z { if z <= pz + 200.0 { z } else { pz + 200.0 } } else { pz - 300.0 }
        };
        sf.jt_cam_z = [z4c, z18];
        if std::env::var_os("SLY2_FEEL_NOJTCAMZ").is_none() {
            focus.z_cpdefi = z18;
        }
    }
    focus.t = time.elapsed_secs();
    let focus = &focus;
    *sf.focus.lock().unwrap() = *focus;
    let clk = CmClock { t: time.elapsed_secs(), dt: time.delta_secs(), dt_real: time.delta_secs() };
    // WP-P15: camera collision = the player's SO collision set
    *sf.coll.lock().unwrap() = CamCollide { cw: pwd.and_then(|d| d.cw.clone()), near_radius: cm.s_near_radius, dt_real: clk.dt_real };
    if clk.dt <= 0.0 {
        return;
    }
    if sf.world_key != key || sf.policy.is_none() {
        let id = match sf.policy {
            Some(id) => id,
            None => add_policy(&mut cm, Box::new(CplcyFollow::new(Box::new(SharedFollow(sf.focus.clone(), sf.coll.clone(), None, sf.track.clone(), sf.regions.clone(), Default::default(), sf.live.clone()))))),
        };
        if sf.world_key != key {
            // FEEL R25: FUN_0013f810(0x2ddf40) level CM props through the descriptor setters (0x2ac880 fields
            // 161 0x14f5a0 fov, 164 0x14f630 near clip, 165 0x14f670 far clip; fog 166-169 = gfx::fog). The
            // hub reads 164 = 52 / 165 = 52000, as cm +0x4c / +0x50 in every rec5 EE state. A/B SLY2_FEEL_NOCMPROPS=1.
            if std::env::var_os("SLY2_FEEL_NOCMPROPS").is_none() {
                for p in &w.props_2ddf40 {
                    let crate::engine::world::props::PropValue::Scalar(b) = &p.value else { continue };
                    let Ok(b4) = <[u8; 4]>::try_from(&b[..b.len().min(4)]) else { continue };
                    let v = f32::from_le_bytes(b4);
                    match p.field {
                        161 => crate::engine::camera::cm::FUN_0014f5a0(&mut cm, v, clk),
                        164 => crate::engine::camera::cm::FUN_0014f630(&mut cm, v, clk),
                        165 => crate::engine::camera::cm::FUN_0014f670(&mut cm, v, clk),
                        _ => {}
                    }
                }
            }
            if sf.policy.is_some() {
                // RevokeCmPolicy by policy (grf 2) before pushing it for the new world
                crate::engine::camera::cm::FUN_0014e4c8(&mut cm, 2, 0, id, 0, 0, clk);
            }
            *sf.track.lock().unwrap() = glide_track(&w);
            *sf.regions.lock().unwrap() = crate::engine::camera::track::TrackSet::load(&w);
            sf.glide = false;
            if let (true, Some(look)) = (sf.binoc, sf.look) {
                crate::engine::camera::cm::FUN_0014e4c8(&mut cm, 2, 0, look, 0, 0, clk);
                sf.binoc = false;
            }
            FUN_0014e328(&mut cm, FOLLOW_CPP, id, FOLLOW_PSO, 0, clk);
            info!("sly: follow camera policy {id} pushed at {:?}", focus.pos);
        }
        sf.policy = Some(id);
        sf.world_key = key;
    }
    // WP-P24: 0x195fb8 jtbs 0x13 entry / 0x13|0x14 exit -> FUN_00140c88(jt, track | 0): FUN_00221268 pushes the
    // track's follow (FUN_00221178(1): SetCmPolicy(cm, track +0x128 + 2, follow, jt, track)) then sets the pushed
    // params' +4 to (DAT_2bf798 + DAT_2bf79c) / 2; leaving revokes by pv (FUN_00221178(0) -> RevokeCmPolicy grf 8).
    let pv = sf.track.lock().unwrap().as_ref().map(|t| (t.0, t.1.f_128));
    if let (Some((pv, f_128)), Some(id)) = (pv, sf.policy) {
        if focus.cam_glide && !sf.glide {
            FUN_0014e328(&mut cm, f_128 + 2, id, FOLLOW_PSO, pv, clk);
            info!("sly: glide camera track {pv:#x} pushed at cpp {}", f_128 + 2);
            if let Some(t) = sf.track.lock().unwrap().as_mut() {
                t.1.params.s_dist = (GLIDE_DIST_2BF798 + GLIDE_DIST_2BF79C) * 0.5;
            }
            sf.glide = true;
        } else if !focus.cam_glide && sf.glide {
            crate::engine::camera::cm::FUN_0014e4c8(&mut cm, 8, -1, 0, 0, pv, clk);
            sf.glide = false;
        }
    }
    // WP-I5: class-15 region tracks (slot 15 FUN_00221388 -> FUN_00221298 -> FUN_00221178)
    if let Some(id) = sf.policy {
        // LO +0x28 bit 60 (the scripts' object state: job toggles / get-o removes; load-time field 26 otherwise)
        let in_world = |o: usize| crate::engine::game::fields::lo_in_world(st.as_ref().map(|s| &s.sched.objs), &w, o);
        // FUN_00221298: focus +0x2dc (jtbs) != -1 holds the track state
        sf.regions.lock().unwrap().focus_jtbs = focus.valid.then_some(focus.jtbs);
        let acts = sf.regions.lock().unwrap().update(FOLLOW_PSO, focus.pos, cm.cut != 0, clk.t, &in_world);
        for a in acts {
            match a {
                crate::engine::camera::track::TrackCm::Push { cpp, pv } => {
                    FUN_0014e328(&mut cm, cpp, id, FOLLOW_PSO, pv, clk);
                    info!("sly: camera track {pv:#x} pushed at cpp {cpp}");
                }
                crate::engine::camera::track::TrackCm::Revoke { pv } => crate::engine::camera::cm::FUN_0014e4c8(&mut cm, 8, -1, 0, 0, pv, clk),
            }
        }
    }
    // WP-P24: 0x19cd38 state 1 entry / exit, mode 0: SetCmPolicy(cm, 7, 0x2de390 look, jt, 0) /
    // RevokeCmPolicy(cm, 0xf, 7, 0x2de390, jt, 0), then FUN_0014d8c0(cm). Mode 1 (0x2de460 C2cd498 zoom, cpp 8)
    // and mode 2 (pso = the binocucom object +0x40) are not ported.
    let want = focus.binoc_cam == Some(0);
    if want != sf.binoc {
        let fc = sf.focus.clone();
        let look = *sf.look.get_or_insert_with(|| add_policy(&mut cm, Box::new(crate::engine::camera::looks::CplcyLook::new(Box::new(SlyLook(fc))))));
        if want {
            FUN_0014e328(&mut cm, BINOC_CPP, look, FOLLOW_PSO, 0, clk);
        } else {
            crate::engine::camera::cm::FUN_0014e4c8(&mut cm, 0xf, BINOC_CPP, look, FOLLOW_PSO, 0, clk);
        }
        crate::engine::camera::cm::FUN_0014d8c0(&mut cm);
        sf.binoc = want;
    }
    let ctx = match &pad.0 {
        Some(p) => CplcyCtx { left: [p.left.x, p.left.y], right: [p.right.x, p.right.y, p.right.mag], ..default() },
        None => CplcyCtx::default(),
    };
    FUN_0014d478(&mut cm, &mut PlayerFollowWorld(*focus), &ctx, clk);
    if crate::engine::player::p15_anim::anim_log() {
        let c = sf.coll.lock().unwrap();
        eprintln!(
            "P17 cam: pos {:?} focus {:?} solid {:?} cut {} active {:?} cw {} near {} clip {}",
            cm.mat[3], focus.pos, c.cw.as_ref().map(|w| w.clip_segment([cm.mat[3][0], cm.mat[3][1], cm.mat[3][2]], focus.pos).iter().map(|l| (l.t[0], l.t[1])).collect::<Vec<_>>()), cm.cut, cm.acpr.iter().map(|r| (r.cpp, r.pcplcy)).collect::<Vec<_>>(), c.cw.as_ref().map_or(0, |w| w.colliders.len()), cm.s_near_radius, cm.s_near_clip
        );
    }
}
