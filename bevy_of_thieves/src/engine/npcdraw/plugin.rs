//! WP-A4 (feature `game`): draw the guards and pickups posed by [`NpcSet`] (see the module doc of
//! `npcdraw`).
//!
//! Every NPC gets its own copy of its template's level entities (spawn_level_geometry spawns each ALO
//! once, at the template's rest place; most NPCs are clones without entities of their own): the copy
//! shares the mesh / material / inverse-bindpose assets (Solari: same meshes, skinned ones stay out of
//! the BLAS as before) and gets fresh skin-joint entities. The template's own entities are hidden. Posing
//! is playerdraw's: object entity = posed root (bone (object, 0)) relative to its owner, joint = posed bone
//! world * inverse(object frame), rigid prim = instance xform * posed bone * inverse(object frame); the copy
//! root carries the class transform. Ink outlines are copied for rigid prims (own LineList asset each,
//! posed as rigid prims, GfxSettings::ink_outlines toggle); ink edges of skinned records are not (as
//! playerdraw: an ink edge cannot follow the skin). A6: the stun FX subtree (`stun_fx`) is copied hidden and shown
//! while the guard is knocked out (K32 `Guard::knocked_out`, brain substate 0xb); K8 collect / sparkle transitions push `gfx::fx` effects.
//!
//! Debug aids for captures: SLY_NPC_PATROL=1 (debug builds only, A8 [`super::patrol_override`]) starts the rail
//! routines of guards the level script has not started (J10 runs the script's FUN_00171cf8 calls,
//! [`patrol_routines`]); SLY_NPC_CAM=1 puts the camera behind the first
//! walking guard.

use super::{template_of, NpcAnim, NpcSet, GUARD_ANIM_WALK};
use crate::engine::anim::pose::Mat4;
use crate::engine::camera::bevy_sync::{sync_cm_camera, CmCamera};
use crate::engine::classes::plugin::{tick_classes, Classes, ClassesLevel};
use crate::engine::classes::Inst;
use crate::engine::game::app::lend_classes;
use crate::engine::gfx::attach::{affine_from_bytes, BoneWorlds};
use crate::engine::gfx::game::{affine_to_transform, AloObject, GfxLevelData, InkOutline, LevelGeometryRoot};
use crate::engine::gfx::fx::{collect_desc, sparkle_life, FxModels, FxQueue, FxRequest, FxTargets, DESC_COIN, DESC_HEALTH};
use crate::engine::gfx::mesh::MeshRole;
use crate::engine::gfx::scene::{invert, mul, Affine, LevelScene, IDENTITY};
use crate::engine::gfx::skin::{record_weight_refs, skin_joints};
use crate::engine::gfx::attach::alo_of;
use crate::engine::playerdraw::subtree;
use crate::engine::playerdraw::vis::FIELD_REMOVE;
use crate::engine::world::objects::{LoadData, Payload};
use bevy::mesh::skinning::SkinnedMesh;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// K8 pickup classes with a FUN_001ce888(9) collect effect: coin (FUN_001c65f8), health (FUN_001c68f0).
pub const CLASS_K8_COIN: u16 = 91;
pub const CLASS_K8_HEALTH: u16 = 93;

/// Root entity of an NPC's entity copy (object index).
#[derive(Component)]
pub struct NpcCopy(pub usize);

/// One drawn NPC.
pub struct NpcDrawn {
    pub obj: usize,
    pub root: Entity,
    /// rest level world of the template root (the rig's space)
    pub tpl_rest: Affine,
    /// class transform at load (bone 0 world * placement, as the K6/K8 builds) and the body yaw then
    pub m0: Affine,
    pub yaw0: Option<f32>,
    /// (entity, object, owner object (None = template root), rest level world)
    pub objects: Vec<(Entity, usize, Option<usize>, Affine)>,
    /// (joint entity, rig bone, object)
    pub joints: Vec<(Entity, usize, usize)>,
    /// (prim entity, instance xform, rig bone, object)
    pub rigid: Vec<(Entity, Affine, usize, usize)>,
    /// (copy entity, level entity) of the sub-ALOs: the copy follows the level entity's visibility (script
    /// removal, apply_script_vis)
    pub mirror: Vec<(Entity, Entity)>,
    /// guard body position and yaw last frame (camera aid)
    pub body: Option<([f32; 3], f32)>,
    /// A6: copies of the stun FX subtree (dizzy birds), shown while the guard is knocked out (K32)
    pub stun: Vec<Entity>,
    /// A6: K8 state (+0x150) and last sparkle time (+0x1c0) seen last frame (effect hooks)
    pub k8_prev: Option<(i32, f32)>,
    /// A9: last seen K32 `hit_fx` (a larger value = FUN_00176be8 played the hit effect -> hit star)
    pub hit_fx_prev: Option<u32>,
    /// A7: K8 position last frame (FUN_001c65f8 reads FUN_001a62e8 before the state change resets it)
    pub k8_pos: [f32; 3],
}

#[derive(Resource, Default)]
pub struct NpcDraw {
    pub key: usize,
    pub set: NpcSet,
    pub drawn: Vec<NpcDrawn>,
    /// WP-D6: class-run ASEGAs (K25) bound to bones (anim::bind), overlaid on the NPC anims
    pub binder: crate::engine::anim::bind::AnimBinder,
}

pub struct NpcDrawPlugin;
impl Plugin for NpcDrawPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NpcDraw>()
            .add_systems(Update, (setup_npc_draw, drive_npcs).chain().after(tick_classes).before(lend_classes))
            .add_systems(Update, npc_camera.after(sync_cm_camera).after(drive_npcs))
            .init_resource::<FxDebugAt>()
            .add_systems(Update, fx_debug.after(drive_npcs).before(lend_classes))
            .add_systems(Update, fx_debug_camera.after(sync_cm_camera).after(fx_debug))
            .add_systems(Update, hit_debug.after(drive_npcs).before(lend_classes).before(fx_debug_camera));
        if !app.is_plugin_added::<crate::engine::gfx::fx::FxPlugin>() {
            app.add_plugins(crate::engine::gfx::fx::FxPlugin);
        }
        if !app.is_plugin_added::<super::objects::ObjDrawPlugin>() {
            app.add_plugins(super::objects::ObjDrawPlugin);
        }
    }
}

fn to_affine(m: &Mat4) -> Affine {
    [[m[0][0], m[0][1], m[0][2]], [m[1][0], m[1][1], m[1][2]], [m[2][0], m[2][1], m[2][2]], [m[3][0], m[3][1], m[3][2]]]
}

/// Once per world: copy each NPC's template entities and map joints / rigid prims to its rig bones.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn setup_npc_draw(
    mut commands: Commands,
    level: Res<ClassesLevel>,
    data: Option<Res<GfxLevelData>>,
    mut classes: ResMut<Classes>,
    mut nd: ResMut<NpcDraw>,
    roots: Query<Entity, With<LevelGeometryRoot>>,
    objs: Query<(Entity, &AloObject, Option<&Children>)>,
    meshes: Query<(Option<&InkOutline>, Option<&SkinnedMesh>, &Mesh3d, Option<&MeshMaterial3d<StandardMaterial>>, &Transform, Option<&crate::engine::gfx::twolayer::TwoLayerSwapped>)>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    tfs: Query<&Transform>,
    // LOOK (K2): the WIRE-C3 colour pass child of a two-layer prim (gfx::game ColourPass)
    colour: Query<(&Mesh3d, &MeshMaterial3d<StandardMaterial>, &Transform), With<crate::engine::gfx::game::ColourPass>>,
    kids: Query<&Children>,
    // ANIM R74: per-frame VU1 light of skinned prims (gfx::game::DynLit), per copy
    dyn_lit: Query<&crate::engine::gfx::game::DynLit>,
) {
    let (Some(w), Some(data)) = (level.0.clone(), data) else { return };
    let key = Arc::as_ptr(&w) as usize;
    if nd.key == key {
        return;
    }
    // wait for spawn_level_geometry's entities
    let Some(level_root) = roots.iter().next() else { return };
    let ent: HashMap<usize, (Entity, Vec<Entity>)> =
        objs.iter().map(|(e, o, c)| (o.0, (e, c.map(|c| c.iter().collect()).unwrap_or_default()))).collect();
    if ent.is_empty() {
        return;
    }
    let c = &mut *classes;
    if super::patrol_override() {
        for (gi, ri) in super::patrol_routines(&c.set, 300.0) {
            // A8: never override a routine the level script already started (J10)
            if c.set.k6_guard(gi).is_some_and(|g| g.f_2e8.is_none()) {
                c.set.k6_run_routine(gi, ri, &mut c.host);
            }
        }
    }
    let scene = LevelScene::build_from(data.lsn, &data.data, &w);
    let scene_of: HashMap<usize, usize> = scene.objects.iter().enumerate().map(|(i, so)| (so.object, i)).collect();
    let npcs: Vec<usize> = c.set.k6.iter().chain(&c.set.k8).filter(|(_, x)| matches!(x, Inst::Guard(_) | Inst::K8(_))).map(|x| x.0).collect();
    let mut bw = BoneWorlds::default();
    let mut hidden = HashSet::new();
    let mut drawn = vec![];
    let (mut njoints, mut nrigid) = (0, 0);
    for o in npcs {
        let tpl = template_of(&w, o);
        let Some(&tsi) = scene_of.get(&tpl) else { continue };
        let rig = crate::engine::playerdraw::JtRig::build(&w, tpl);
        let m0 = match w.life.placed.get(&o).map(affine_from_bytes) {
            Some(pm) => mul(&bw.get(&w, o, 0), &pm),
            None => bw.get(&w, o, 0),
        };
        let root = commands.spawn((NpcCopy(o), affine_to_transform(&m0), Visibility::default(), ChildOf(level_root))).id();
        let sub: HashSet<usize> = subtree(&w, tpl).into_iter().collect();
        // playerdraw::vis generic parts: property field 26 = removed at post-load (recursive), mesh record
        // / instance u1 & 1 = hidden (+0xc 0x800)
        let mut removed: HashSet<usize> = HashSet::new();
        for &k in &sub {
            if alo_of(&w, k).is_some_and(|a| a.props.iter().any(|p| p.field == FIELD_REMOVE)) {
                removed.extend(subtree(&w, k));
            }
        }
        // A6: the stun FX (dizzy birds) are copied hidden and shown while knocked out (drive_npcs)
        let stun_objs: HashSet<usize> = super::stun_fx(&w, tpl).into_iter().collect();
        // A6: guard post-load message 0xe (FUN_00206170) hides the unused weapon meshes and shows the
        // instance's weapon (property 1256); without it every record keeps its load bit (u1 & 1)
        let post = super::script::POST_LOAD_CLASSES.contains(&w.objects[o].class).then(|| super::script::ee_sub_206170(&w, &data.data, o, tpl, &removed, -1)).flatten();
        if let Some(h) = &post {
            removed = h.vis.removed.clone();
        }
        // load bits + property fields 44 / 45 of every ALO in the subtree (the template also gets the
        // instance's own rows)
        let mut base = crate::engine::playerdraw::vis::JtVis::default();
        for &k in &sub {
            let Some(a) = alo_of(&w, k) else { continue };
            for (ri, r) in a.mesh.records.iter().enumerate() {
                if r.u1 & 1 != 0 {
                    base.hidden.insert((k, ri, None));
                }
                for (ii, _) in r.instances.iter().enumerate().filter(|x| x.1.u1 & 1 != 0) {
                    base.hidden.insert((k, ri, Some(ii)));
                }
            }
            base.ee_stream_mesh(&w, k, k);
        }
        if o != tpl {
            base.ee_stream_mesh(&w, o, tpl);
        }
        let shown = |obj: usize, _rec: &crate::engine::world::alo::MeshRecord, r: usize, i: Option<usize>| match &post {
            Some(h) if obj == h.mesh_obj => !h.vis.hidden.contains(&(obj, r, i)),
            _ => !base.hidden.contains(&(obj, r, i)),
        };
        let mut ent_of: HashMap<usize, Entity> = HashMap::new();
        let (mut objects, mut joints, mut rigid, mut mirror) = (vec![], vec![], vec![], vec![]);
        // scene order: owners come before their children
        for (si, so) in scene.objects.iter().enumerate().filter(|(_, so)| sub.contains(&so.object)) {
            let Some((oe, children)) = ent.get(&so.object) else { continue };
            if removed.contains(&so.object) {
                continue;
            }
            let Payload::Loaded(LoadData::Alo { alo, .. }) = &w.objects[so.object].payload else { continue };
            let owner = so.owner.map(|k| scene.objects[k].object).filter(|k| sub.contains(k) && *k != tpl);
            let e = if so.object == tpl {
                if hidden.insert(*oe) {
                    commands.entity(*oe).insert(Visibility::Hidden);
                }
                root
            } else {
                let parent = owner.and_then(|k| ent_of.get(&k).copied()).unwrap_or(root);
                let e = commands.spawn((affine_to_transform(&so.local), Visibility::default(), ChildOf(parent))).id();
                objects.push((e, so.object, owner, scene.world_of(si)));
                mirror.push((e, *oe));
                e
            };
            ent_of.insert(so.object, e);
            // PERF2: merged two-layer prims report their WIRE-C3 originals (gfx::twolayer)
            let ents: Vec<_> = children
                .iter()
                .filter_map(|&c| meshes.get(c).ok().map(|m| (c, m)))
                .filter_map(|(c, (ink, sm, mesh, mat, t, sw))| match sw {
                    Some(s) => Some((ink, sm, &s.mesh, &s.mat, t, None)),
                    None => Some((ink, sm, mesh, mat?, t, Some(c))),
                })
                .collect();
            let prims = so.prims.iter().filter(|p| matches!(p.prim.role, MeshRole::Surface | MeshRole::InkEdges));
            let all: Vec<_> = prims.zip(ents).collect();
            // records drawn skinned: their ink edges cannot follow the skin (as playerdraw)
            let skinned: HashSet<(usize, Option<usize>)> =
                all.iter().filter(|(_, (ink, sm, ..))| ink.is_none() && sm.is_some()).map(|(p, _)| (p.record, p.instance)).collect();
            for (p, (ink, sm, mesh, mat, t, src)) in all {
                let rec = &alo.mesh.records[p.record];
                if !shown(so.object, rec, p.record, p.instance) {
                    continue;
                }
                if let Some(ink) = ink {
                    // A5: ink outline copy (own LineList asset: update_ink_outlines rewrites it per entity),
                    // posed as a rigid prim; same GfxSettings::ink_outlines toggle as the level / Sly
                    if skinned.contains(&(p.record, p.instance)) || !record_weight_refs(&rec.data).is_empty() {
                        continue;
                    }
                    let Some(m) = mesh_assets.get(&mesh.0).cloned() else { continue };
                    let pe = commands.spawn((Mesh3d(mesh_assets.add(m)), mat.clone(), *t, ink.clone(), ChildOf(e))).id();
                    let (pre, r) = match p.instance {
                        Some(ii) => (rec.instances[ii].xform, rec.instances[ii].r#ref),
                        None => (IDENTITY, rec.r#ref),
                    };
                    if let Some(&b) = (r >= 0).then(|| rig.index.get(&(so.object, r as usize))).flatten() {
                        rigid.push((pe, pre, b, so.object));
                    }
                    continue;
                }
                match sm {
                    Some(sm) => {
                        let refs = skin_joints(&w, &mut bw, so.object, record_weight_refs(&rec.data)).refs;
                        let mut js = vec![];
                        for (k, &old) in sm.joints.iter().enumerate() {
                            let jt = tfs.get(old).copied().unwrap_or_default();
                            let j = commands.spawn((jt, ChildOf(e))).id();
                            if let Some(b) = refs.get(k).and_then(|r| rig.bone_of(r)) {
                                joints.push((j, b, so.object));
                            }
                            js.push(j);
                        }
                        // ANIM R74: DynLit copies own their mesh assets (lit per copy), two draws (no merge)
                        let orig = children.iter().copied().find(|&c| meshes.get(c).is_ok_and(|m| m.2.0 == mesh.0 || m.5.is_some_and(|s| s.mesh.0 == mesh.0)));
                        if let Some(d) = orig.and_then(|c| dyn_lit.get(c).ok()) {
                            let Some(m) = mesh_assets.get(&mesh.0).cloned() else { continue };
                            let main = commands
                                .spawn((
                                    Mesh3d(mesh_assets.add(m)),
                                    mat.clone(),
                                    SkinnedMesh { inverse_bindposes: sm.inverse_bindposes.clone(), joints: js.clone() },
                                    bevy::camera::visibility::DynamicSkinnedMeshBounds,
                                    *t,
                                    ChildOf(e),
                                ))
                                .id();
                            let mut d = d.clone();
                            d.skin.joints = js.clone();
                            d.root = root;
                            commands.entity(main).insert(d);
                            for k in orig.and_then(|c| kids.get(c).ok()).into_iter().flat_map(|k| k.iter()) {
                                if let Ok((cm, cmat, ct)) = colour.get(k) {
                                    let Some(m) = mesh_assets.get(&cm.0).cloned() else { continue };
                                    commands.spawn((
                                        crate::engine::gfx::game::ColourPass,
                                        Mesh3d(mesh_assets.add(m)),
                                        cmat.clone(),
                                        SkinnedMesh { inverse_bindposes: sm.inverse_bindposes.clone(), joints: js.clone() },
                                        bevy::camera::visibility::DynamicSkinnedMeshBounds,
                                        *ct,
                                        ChildOf(main),
                                    ));
                                }
                            }
                            continue;
                        }
                        let main = commands.spawn((
                            mesh.clone(),
                            mat.clone(),
                            SkinnedMesh { inverse_bindposes: sm.inverse_bindposes.clone(), joints: js.clone() },
                            // A10: posed culling bounds (mesh asset carries the per-joint bounds, gfx::game)
                            bevy::camera::visibility::DynamicSkinnedMeshBounds,
                            *t,
                            ChildOf(e),
                        )).id();
                        // LOOK (K2): copy the unlit colour pass too (FUN_001eb960 draws both passes); without
                        // it the guards showed only the dark lit grey layer (solid black in the cellar)
                        for k in src.and_then(|c| kids.get(c).ok()).into_iter().flat_map(|k| k.iter()) {
                            if let Ok((cm, cmat, ct)) = colour.get(k) {
                                commands.spawn((
                                    crate::engine::gfx::game::ColourPass,
                                    cm.clone(),
                                    cmat.clone(),
                                    SkinnedMesh { inverse_bindposes: sm.inverse_bindposes.clone(), joints: js.clone() },
                                    bevy::camera::visibility::DynamicSkinnedMeshBounds,
                                    *ct,
                                    ChildOf(main),
                                ));
                            }
                        }
                    }
                    None => {
                        let pe = commands.spawn((mesh.clone(), mat.clone(), *t, ChildOf(e))).id();
                        let (mut pre, r) = match p.instance {
                            Some(ii) => (rec.instances[ii].xform, rec.instances[ii].r#ref),
                            None => (IDENTITY, rec.r#ref),
                        };
                        // ANIM-ALL R76: translucent prims carry their vertices shifted by the record sort centre c
                        // (gfx::game WP-C8: mesh - c, Transform = local x T(c)); the rigid pose below replaces the
                        // Transform, so put T(c) first or the mesh collapses onto the bone origin (the guard
                        // flashlight cone, centred ~640 ahead of bone 2, sat on the guard). Opt-out SLY2_R76NOSORTC.
                        let c = affine_to_transform(&p.local).compute_affine().inverse().transform_point3(t.translation);
                        if c.length() > 0.01 && std::env::var_os("SLY2_R76NOSORTC").is_none() {
                            pre = mul(&[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], c.to_array()], &pre);
                        }
                        if let Some(&b) = (r >= 0).then(|| rig.index.get(&(so.object, r as usize))).flatten() {
                            rigid.push((pe, pre, b, so.object));
                        }
                    }
                }
            }
        }
        njoints += joints.len();
        nrigid += rigid.len();
        let stun: Vec<Entity> = objects.iter().filter(|x| stun_objs.contains(&x.1)).map(|x| x.0).collect();
        for &e in &stun {
            commands.entity(e).insert(Visibility::Hidden);
        }
        drawn.push(NpcDrawn { obj: o, root, tpl_rest: scene.world_of(tsi), m0, yaw0: None, objects, joints, rigid, mirror, body: None, stun, k8_prev: None, k8_pos: [0.0; 3], hit_fx_prev: None });
    }
    info!("npcs: {} drawn ({} templates hidden), {njoints} skin joints, {nrigid} rigid prims posed", drawn.len(), hidden.len());
    *nd = NpcDraw { key, set: NpcSet::default(), drawn, binder: Default::default() };
}

/// A9: height of the hit star above the guard's body position (chest of the table-3 guards, ~ 1.4 m)
pub const HIT_STAR_Z: f32 = 140.0;

/// Every frame after the class update: anims from the class state, pose, transforms.
#[allow(clippy::too_many_arguments)]
/// ANIM-ALL: env `SLY2_NPCDRAW=0` disables [`drive_npcs`].
pub fn npcdraw_off() -> bool {
    static OFF: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *OFF.get_or_init(|| std::env::var("SLY2_NPCDRAW").is_ok_and(|v| v == "0"))
}

/// ANIM-ALL: env `SLY2_GUEST_POSE=1` draws the bone postures the guest pose step (FUN_0022de88 / FUN_0022faa0,
/// `LiveQ::pose_pass`) wrote into the ALO bone arrays (+0x54, stride 0x100, world +0x40) instead of the native pose.
pub fn guest_pose_on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("SLY2_GUEST_POSE").map_or(true, |v| v != "0"))
}

/// ANIM-ALL R15/R16: the JT rig (Sly/Bentley/Murray) and its po props take the guest locals
/// ([`guest_locals_overlay`]); on by default with SLY2_GUEST_POSE, `SLY2_GUEST_JT=0` keeps the native JT pose.
/// R16: the JT got its class-53 vtable (FUN_001116d8 installs 0x2d2e98 through its last callee, exe_scan), so its
/// load / postload set the record modes (FUN_002303a0); DrivenXf::FUN_0022c8c0 keeps the xf rows like the
/// original memcpy (wired_anim_all_jt_guest_pose_matches_native: native vs guest within 0.003).
pub fn guest_jt_on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| guest_pose_on() && std::env::var("SLY2_GUEST_JT").map_or(true, |v| v != "0"))
}

/// ANIM-ALL R15: the guest bone locals (+0 of the 0x100-byte records at ALO +0x54, written by the act evaluators
/// FUN_00131cd8 / FUN_00131d68 in the FUN_0022de88 pose step) replace the native locals of every bone that carries
/// a guest act (+0x90); other bones keep the native pose (fallback). The guest +0x40 world is not used: the JT's
/// records never get one (the world chain of FUN_0022faa0 does not run for them), the native parent walk
/// (`ActorPose::recompose`) builds the worlds. Returns the number of replaced locals.
/// `only` (non-empty): the top act must belong to one of these ASEGAs (act +0x24), so ASEGAs another driver
/// started on the same bones (and the native animator does not play) do not take the bones over.
pub fn guest_locals_overlay(lq: &mut crate::engine::classes::q::live::LiveMem, keys: &[(usize, usize)], pose: &mut crate::engine::anim::actor::ActorPose, only: &[u32]) -> usize {
    use crate::engine::classes::q::q2mem::QMem;
    let mut n = 0;
    for (i, &(o, k)) in keys.iter().enumerate() {
        let Some(&a) = lq.obj_addr.get(o) else { continue };
        if a == 0 {
            continue;
        }
        let (nb, bp) = (lq.r32(a + 0x50) as i32, lq.r32(a + 0x54));
        if nb <= 0 || nb > 512 || k as i32 >= nb || bp == 0 {
            continue;
        }
        let r = bp + ((k as u32) << 8);
        let act = lq.r32(r + 0x90);
        // ANIM-ALL R78: a FUN_0021cb18 spring bone (xf +0x98 == 0x10, Sly's mask tail) is posed by the guest pose
        // pass itself (its act is the kind-3 holder, not an anim slot): take its local too
        let spring = lq.r16(r + 0x98) == 0x10 && std::env::var_os("SLY2_R78TAILOFF").is_none();
        if act == 0 || (!spring && !only.is_empty() && !only.contains(&lq.r32(act + 0x24))) {
            continue;
        }
        let m = lq.rm(r);
        if !m.iter().flatten().all(|v| v.is_finite()) || (m[3][3] - 1.0).abs() > 1e-3 {
            continue;
        }
        if let Some(b) = pose.bones.get_mut(i) {
            b.local = m;
            n += 1;
        }
    }
    n
}

/// LOOK R18: a guest bone world (ALO +0x54 record +0x40, 3x4 affine stored as 4 qw) as a Mat4 with the
/// .w lanes forced to (0, 0, 0, 1).
pub fn guest_affine(mut m: [[f32; 4]; 4]) -> bevy::math::Mat4 {
    (0..3).for_each(|r| m[r][3] = 0.0);
    m[3][3] = 1.0;
    bevy::math::Mat4::from_cols_array_2d(&m)
}

/// Per object: keep the native (object, 0) world as the frame, put the guest posture relative to the guest
/// bone 0 on top (row-vector: W[k] = G[k] * inv(G[0]) * W[0]); objects without guest acts are left native.
pub fn guest_pose_overlay(lq: &mut crate::engine::classes::q::live::LiveMem, keys: &[(usize, usize)], worlds: &mut [Mat4]) {
    use crate::engine::classes::q::q2mem::QMem;
    use bevy::math::Mat4 as G;
    // LOOK R18: the guest bone world (+0x40) is a 3x4 affine; the .w lanes are not part of it (row 3 .w held
    // 5.0 for the cellar guards -> inverse(G0) became projective and scaled the posed bones 5..15x: the giant
    // guard over the same-camera cellar view). Read it as affine (w lanes 0, 0, 0, 1).
    let gm = |lq: &mut crate::engine::classes::q::live::LiveMem, a: u32| -> G { guest_affine(lq.rm(a)) };
    let mut root: HashMap<usize, usize> = HashMap::new();
    for (i, &(o, k)) in keys.iter().enumerate() {
        if k == 0 {
            root.insert(o, i);
        }
    }
    for (i, &(o, k)) in keys.iter().enumerate() {
        let Some(&r) = root.get(&o) else { continue };
        let Some(&a) = lq.obj_addr.get(o) else { continue };
        if a == 0 {
            continue;
        }
        let (nb, bp) = (lq.r32(a + 0x50) as i32, lq.r32(a + 0x54));
        if nb <= 0 || nb > 512 || k as i32 >= nb || bp == 0 {
            continue;
        }
        if !(0..nb as u32).any(|j| lq.r32(bp + (j << 8) + 0x90) != 0) {
            continue;
        }
        if k == 0 || i == r {
            continue;
        }
        let (g0, gk) = (gm(lq, bp + 0x40), gm(lq, bp + ((k as u32) << 8) + 0x40));
        // bones whose guest world was never built (zero matrix) keep the native pose (fallback)
        if g0.determinant().abs() < 1e-8 || gk.determinant().abs() < 1e-8 {
            continue;
        }
        let w0 = G::from_cols_array_2d(&worlds[r]);
        worlds[i] = (w0 * g0.inverse() * gk).to_cols_array_2d();
    }
}

pub fn drive_npcs(
    time: Res<Time>,
    level: Res<ClassesLevel>,
    mut classes: ResMut<Classes>,
    mut nd: ResMut<NpcDraw>,
    mut tf: Query<&mut Transform>,
    mut vq: Query<&mut Visibility>,
    mut fxq: ResMut<FxQueue>,
    mut fxt: ResMut<FxTargets>,
    mut fxm: ResMut<FxModels>,
    mut liveq: Option<ResMut<crate::engine::classes::q::live::LiveQ>>,
    clock: Option<Res<crate::engine::player::plugin::PlayerClock>>,
    od: Option<Res<super::objects::ObjDraw>>,
) {
    // ANIM-ALL: SLY2_NPCDRAW=0 turns this native NPC animation stand-in off (default on: the guest ASEGA set
    // world+0x3d88 plays the class-code starts, but guard anim tasks and the bone readback are not wired yet)
    if npcdraw_off() {
        return;
    }
    let Some(w) = level.0.clone() else { return };
    if nd.key != Arc::as_ptr(&w) as usize {
        return;
    }
    let c = &mut *classes;
    let nd = &mut *nd;
    // ANIM-ALL R15: DAT_002dded4 (g_clock, clamped [1/60, 1/30]) when the game clock runs, as the guard's own
    // FUN_00130918 updates; else the Bevy delta (capped)
    let dt = clock.as_deref().map_or(time.delta_secs().min(1.0 / 15.0), |c| c.dt);
    nd.set.step(&w, &mut c.set, &mut c.host, dt);
    // WP-D6: the class-run ASEGAs pose the bones they bind (FUN_0012f9b8 acts) over the NPC anims
    let live: Vec<(u64, usize, Option<usize>)> = c.host.k25.asegas.iter().filter(|x| x.1.st.alive).map(|(k, x)| (*k as u64, x.aseg, (x.obj as usize).checked_sub(1))).collect();
    let mut live = live;
    let mut times: HashMap<u64, (f32, f32)> = c.host.k25.asegas.iter().map(|(k, x)| (*k as u64, (x.st.t, x.st.svt))).collect();
    // LOOK R34 diag: SLY2_LOOK_ASEGSTART=oid,.. plays those ASEGs (looping on the frame clock) as if the level
    // script had started them (splice getters_call_101910 -> FUN_00130488, not run by the port here)
    if let Ok(v) = std::env::var("SLY2_LOOK_ASEGSTART") {
        for (n, oid) in v.split(',').filter_map(|x| x.trim().parse::<i32>().ok()).enumerate() {
            let Some(i) = (0..w.objects.len()).find(|&i| w.objects[i].oid == oid && w.objects[i].class == 109) else { continue };
            let Some(a) = crate::engine::playerdraw::aseg_of(&w, i) else { continue };
            let h = 0x7f00_0000u64 | n as u64;
            live.push((h, i, None));
            times.insert(h, (time.elapsed_secs() % a.length.max(1.0e-3), 1.0));
        }
    }
    nd.binder.sync(&w, &live);
    nd.set.overlay_class_anims(&w, &nd.binder, &|k| times.get(&k).copied());
    // LOOK R34: pool copies on an animated owner bone follow it (FUN_0022faa0: the copy is a child of the
    // spawner's parent bone, so its world = rest x socket x posed bone world)
    if let Some(od) = od.as_deref().filter(|o| !(o.follow.is_empty() && o.marker_bones.is_empty()) && o.key == Arc::as_ptr(&w) as usize) {
        let locals = nd.binder.locals(&w, &|k| times.get(&k).copied());
        if !locals.is_empty() {
            let pose = nd.binder.compose(&w, locals);
            if std::env::var_os("SLY2_LOOK_ASEGDBG").is_some() {
                let owners: HashSet<i32> = od.follow.iter().map(|f| w.objects[f.3 .0].oid as i32).collect();
                for i in 0..w.objects.len() {
                    let Some(a) = crate::engine::playerdraw::aseg_of(&w, i) else { continue };
                    let hit: Vec<i32> = a.channels.iter().map(crate::engine::anim::bind::channel_oid).filter(|o| owners.contains(o)).collect();
                    if !hit.is_empty() {
                        info!("ASEGDBG aseg obj {i} oid {} cls {} live {} hits {hit:?}", w.objects[i].oid, w.objects[i].class, live.iter().any(|l| l.1 == i));
                        for j in i.saturating_sub(6)..(i + 3).min(w.objects.len()) {
                            let x = &w.objects[j];
                            info!("ASEGDBG   obj {j} oid {} cls {} owner {:?} sma {}", x.oid, x.class, x.owner.map(|o| w.objects[o].oid), c.host.k25.smas.values().any(|m| m.sm == j && m.alive));
                        }
                    }
                }
            }
            if std::env::var_os("SLY2_LOOK_ORIGINDBG").is_some() {
                info!("FOLLOW {} of {} pool copies on posed bones", od.follow.iter().filter(|f| pose.bones.contains_key(&f.3)).count(), od.follow.len());
            }
            // LOOK R40: K12 sign copies (objects::ObjDraw::marker_bones): prim local = posed bone x inverse(bone 0)
            for (e, keys, root0, k0) in &od.marker_bones {
                let Some(bw) = keys.iter().find_map(|k| pose.bones.get(k)) else { continue };
                let r0 = pose.bones.get(k0).unwrap_or(root0);
                if let Ok(mut t) = tf.get_mut(*e) {
                    *t = affine_to_transform(&mul(bw, &invert(r0)));
                }
            }
            for (e, rest, pm, key) in &od.follow {
                let Some(bw) = pose.bones.get(key) else { continue };
                let m = mul(rest, &mul(pm, bw));
                if let Ok(mut t) = tf.get_mut(*e) {
                    *t = affine_to_transform(&m);
                }
            }
        }
    }
    for d in &mut nd.drawn {
        let Some(an) = nd.set.npcs.get_mut(&d.obj) else { continue };
        // class transform: placement rotation turned by the body yaw change, class position
        let mut stunned = false;
        let (m, show) = match c.set.k6.iter().chain(&c.set.k8).find(|x| x.0 == d.obj).map(|x| &x.1) {
            Some(Inst::Guard(g)) => {
                stunned = g.knocked_out();
                // A9: the hit effect FUN_00156698 of the guard's damage reaction (K32 FUN_00176be8 `hit_fx`)
                // shows the hit star (REPLACED visual, gfx::fx additive burst) at the guard's chest; the hit
                // record's own +0x20 position is cleared before this runs
                if d.hit_fx_prev.is_some_and(|n| g.hit_fx > n) {
                    fxq.push(FxRequest::hit([g.body.pos[0], g.body.pos[1], g.body.pos[2] + HIT_STAR_Z]));
                }
                d.hit_fx_prev = Some(g.hit_fx);
                let y0 = *d.yaw0.get_or_insert(g.body.yaw);
                let (s, co) = (g.body.yaw - y0).sin_cos();
                let rz = [[co, s, 0.0], [-s, co, 0.0], [0.0, 0.0, 1.0], [0.0; 3]];
                let mut m = mul(&d.m0, &rz);
                m[3] = g.body.pos;
                d.body = Some((g.body.pos, g.body.yaw));
                (m, true)
            }
            Some(Inst::K8(p)) => {
                let mut m = d.m0;
                m[3] = [p.xf_pos[0], p.xf_pos[1], p.xf_pos[2]];
                let pos = m[3];
                fxt.0.insert(d.obj, pos);
                // A6 effect hooks (the ClassHost calls are no-ops in classes::plugin::EngineHost): the
                // collect transition 1 -> 2 is where FUN_001c65f8 / FUN_001c68f0 call FUN_001ce888(9) and
                // FUN_001c7ee8 calls FUN_00156b98; a new +0x1c0 is a sparkle FUN_001c6308 started
                if let Some((s0, sp0)) = d.k8_prev {
                    if p.f_150 == 2 && s0 != 2 {
                        // A7: FUN_001ce888(9) +0x1c = the pickup (drawn flying to the HUD); FUN_001d04e8 draws
                        // its start velocity from the game rng (FUN_0011d448 x, y, z in order)
                        let desc = match w.objects[d.obj].class {
                            CLASS_K8_COIN => Some(DESC_COIN),
                            CLASS_K8_HEALTH => Some(DESC_HEALTH),
                            _ => None,
                        };
                        fxq.push(match desc.and_then(|k| collect_desc(k).map(|cd| (k, cd))) {
                            Some((k, cd)) => {
                                let vel = std::array::from_fn(|i| c.set.rng.FUN_0011d448(cd.vel_min[i], cd.vel_max[i]));
                                fxm.0.insert(d.obj, d.root);
                                FxRequest::collect_obj(k, d.k8_pos, d.obj, vel)
                            }
                            None => FxRequest::hit(d.k8_pos),
                        });
                    }
                    if p.f_1c0 != sp0 {
                        fxq.push(FxRequest::sparkle(pos, Some(d.obj), sparkle_life(p.xf_vel[2])));
                    }
                }
                d.k8_prev = Some((p.f_150, p.f_1c0));
                d.k8_pos = pos;
                (m, p.in_world)
            }
            _ => continue,
        };
        if let Ok(mut t) = tf.get_mut(d.root) {
            *t = affine_to_transform(&m);
        }
        if let Ok(mut v) = vq.get_mut(d.root) {
            let want = if show { Visibility::Inherited } else { Visibility::Hidden };
            if *v != want {
                *v = want;
            }
        }
        for &(e, oe) in &d.mirror {
            let Ok(v) = vq.get(oe).copied() else { continue };
            let want = if v == Visibility::Hidden { Visibility::Hidden } else { Visibility::Inherited };
            if let Ok(mut c) = vq.get_mut(e) {
                if *c != want {
                    *c = want;
                }
            }
        }
        for &e in &d.stun {
            let want = if stunned { Visibility::Inherited } else { Visibility::Hidden };
            if let Ok(mut v) = vq.get_mut(e) {
                if *v != want {
                    *v = want;
                }
            }
        }
        let an = &*an;
        let mut worlds = an.rig.pose.world();
        if guest_pose_on() {
            if let Some(m) = liveq.as_deref_mut().and_then(|lq| lq.mem.as_mut()) {
                guest_pose_overlay(m, &an.rig.keys, &mut worlds);
            }
        }
        let rest_of = |o: usize| d.objects.iter().find(|x| x.1 == o).map(|x| x.3);
        let posed = |o: usize| an.rig.index.get(&(o, 0)).map(|&b| to_affine(&worlds[b])).or_else(|| rest_of(o));
        let frame = |o: Option<usize>| match o {
            None => Some(d.tpl_rest),
            Some(o) => posed(o),
        };
        for &(e, o, owner, _) in &d.objects {
            let (Some(p), Some(q)) = (posed(o), frame(owner)) else { continue };
            if let Ok(mut t) = tf.get_mut(e) {
                *t = affine_to_transform(&mul(&p, &invert(&q)));
            }
        }
        let tpl = template_of(&w, d.obj);
        let fr = |o: usize| frame((o != tpl).then_some(o));
        for &(e, b, o) in &d.joints {
            let Some(f) = fr(o) else { continue };
            if let Ok(mut t) = tf.get_mut(e) {
                *t = affine_to_transform(&mul(&to_affine(&worlds[b]), &invert(&f)));
            }
        }
        for &(e, pre, b, o) in &d.rigid {
            let Some(f) = fr(o) else { continue };
            if let Ok(mut t) = tf.get_mut(e) {
                *t = affine_to_transform(&mul(&mul(&pre, &to_affine(&worlds[b])), &invert(&f)));
            }
        }
    }
}

/// SLY_NPC_CAM: camera behind / above the first walking guard (level space z up; the level root turns
/// it by -90 degrees about x: (x, y, z) -> (x, z, -y)).
pub fn npc_camera(nd: Res<NpcDraw>, mut cam: Query<&mut Transform, With<CmCamera>>, mut pick: Local<Option<usize>>) {
    if std::env::var("SLY_NPC_CAM").is_err() {
        return;
    }
    if pick.is_none() {
        // ANIM-ALL R76 (capture aid): SLY_NPC_CAM=<oid> picks that NPC (else the lowest walking guard)
        *pick = std::env::var("SLY_NPC_CAM").ok().and_then(|s| s.parse().ok()).filter(|o| nd.drawn.iter().any(|d| d.obj == *o)).or_else(|| nd.set.npcs.iter().filter(|(_, a)| matches!(a.anim, NpcAnim::Loop { id: GUARD_ANIM_WALK, .. })).map(|(o, _)| *o).min());
        for d in nd.drawn.iter().filter(|d| d.body.is_some()) {
            info!("npcs: drawn {} body {:?}", d.obj, d.body);
        }
        if let Some(o) = *pick {
            info!("npcs: camera on walking guard {o}");
        }
    }
    let Some(o) = *pick else { return };
    let Some((p, yaw)) = nd.drawn.iter().find(|d| d.obj == o).and_then(|d| d.body) else { return };
    let (s, c) = yaw.sin_cos();
    let eye = [p[0] - c * 420.0 + s * 260.0, p[1] - s * 420.0 - c * 260.0, p[2] + 260.0];
    let at = [p[0], p[1], p[2] + 90.0];
    let b = |v: [f32; 3]| Vec3::new(v[0], v[2], -v[1]);
    for mut t in &mut cam {
        *t = Transform::from_translation(b(eye)).looking_at(b(at), Vec3::Y);
    }
}

/// SLY_A7_FX=N (capture aid, A7): camera on the in-world coin (class 91) nearest to the player (else a
/// pool clone placed beside the player); at frame N after the NPC
/// setup that coin is collected through its own state change (FUN_001c65f8 state 1 -> FUN_001ce888(9)
/// collect + HUD), and one FUN_00156698 hit burst (REPLACED visual) is pushed beside it as a stand-in for a
/// cane hit (no cane-hit caller is driven yet). The sparkles are the pickup's own (FUN_001c6308).
#[allow(clippy::type_complexity)]
pub fn fx_debug(
    level: Res<ClassesLevel>,
    mut classes: ResMut<Classes>,
    nd: Res<NpcDraw>,
    mut fxq: ResMut<FxQueue>,
    mut dbg: ResMut<FxDebugAt>,
    mut st: Local<(u32, Option<usize>, Option<[f32; 3]>)>,
) {
    let Ok(n) = std::env::var("SLY_A7_FX").map(|v| v.parse::<u32>().unwrap_or(60)) else { return };
    let Some(w) = level.0.clone() else { return };
    if nd.key != Arc::as_ptr(&w) as usize {
        return;
    }
    let c = &mut *classes;
    let set = &mut c.set;
    if st.1.is_none() {
        // the in-world coin nearest to the player (K6 player position), else the lowest one
        let Some(pl) = set.g.k6_players.first().map(|x| x.1).filter(|p| p.iter().all(|v| v.abs() < 1.0e8)) else { return };
        let d2 = |p: &[f32; 4]| (0..3).map(|k| (p[k] - pl[k]) * (p[k] - pl[k])).sum::<f32>();
        st.1 = set
            .k8
            .iter()
            .filter_map(|(o, x)| match x {
                Inst::K8(p) if w.objects[*o].class == CLASS_K8_COIN && p.in_world && p.f_150 == 0 => Some((d2(&p.xf_pos), *o)),
                _ => None,
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .filter(|x| x.0 < 2000.0 * 2000.0)
            .map(|x| x.1);
        // no placed coin near the player (the levels place almost none: coins are pool clones that droppers
        // put into the world): put the first out-of-world clone 150 units in front of the player, as a dropper
        if st.1.is_none() {
            if let Some((o, Inst::K8(p))) = set.k8.iter_mut().find(|(o, x)| w.objects[*o].class == CLASS_K8_COIN && matches!(x, Inst::K8(p) if !p.in_world)) {
                p.xf_pos = [pl[0] + 150.0, pl[1], pl[2] + 40.0, 1.0];
                st.1 = Some(*o);
            }
        }
        if let Some(o) = st.1 {
            info!("fx_debug: coin {o}");
        }
    }
    let Some(o) = st.1 else { return };
    st.0 += 1;
    let Some(Inst::K8(p)) = set.k8.iter_mut().find(|x| x.0 == o).map(|x| &mut x.1) else { return };
    let at = *st.2.get_or_insert([p.xf_pos[0], p.xf_pos[1], p.xf_pos[2]]);
    if st.0 < n {
        // hold the coin where it was put (the pool clone is otherwise moved by its own update)
        // (FUN_001c5148 integrates +0x170 / +0x180 into the transform, as a dropper sets them)
        p.f_170 = [at[0], at[1], at[2], 1.0];
        p.f_180 = [0.0; 4];
        p.f_190 = [0.0; 4];
        p.xf_pos = p.f_170;
        p.in_world = true;
    }
    if st.0 == n {
        let mut g = crate::engine::classes::k8::Cx { g: &mut set.g, rng: &mut set.rng };
        p.set_state(o, 1, &mut g, &mut c.host);
        fxq.push(FxRequest::hit([at[0] + 60.0, at[1], at[2] + 30.0]));
        info!("fx_debug: frame {n}: coin {o} at {at:?} state -> {}, hit pushed; player {:?}", p.f_150, set.g.k6_players.first());
    }
    dbg.0 = Some(at);
}

/// SLY_A9_HIT=N (capture aid, A9): N frames after the NPC setup the guard nearest to Sly takes one hit
/// from the player through the guard path a cane hit takes after FUN_001f7410 (`ClassSet::ee_sub_1f7598`
/// -> slot 74 FUN_00207468 -> K32 FUN_00176be8), with the cane response +0x44 = 0x20 and amount 0 (never lethal);
/// the camera looks at that guard (FxDebugAt), so `drive_npcs` shows the hit star on the next frame.
/// Scripted Circle presses do not start a ground cane attack yet (player/, see ENGINE_PORT.md A9).
pub fn hit_debug(level: Res<ClassesLevel>, mut classes: ResMut<Classes>, nd: Res<NpcDraw>, mut dbg: ResMut<FxDebugAt>, mut st: Local<(u32, Option<usize>)>) {
    use crate::engine::classes::guard::{slots::GuardHit, PLAYER_HANDLE};
    let Ok(n) = std::env::var("SLY_A9_HIT").map(|v| v.parse::<u32>().unwrap_or(60)) else { return };
    let Some(w) = level.0.clone() else { return };
    if nd.key != Arc::as_ptr(&w) as usize {
        return;
    }
    let c = &mut *classes;
    if st.1.is_none() {
        let Some(pl) = c.set.g.k6_players.first().map(|x| x.1).filter(|p| p.iter().all(|v| v.abs() < 1.0e8)) else { return };
        let d2 = |p: &[f32; 3]| (0..3).map(|k| (p[k] - pl[k]) * (p[k] - pl[k])).sum::<f32>();
        st.1 = c.set.k6.iter().filter_map(|(o, x)| if let Inst::Guard(g) = x { Some((d2(&g.body.pos), *o)) } else { None }).min_by(|a, b| a.0.total_cmp(&b.0)).map(|x| x.1);
    }
    let Some(o) = st.1 else { return };
    let Some(pos) = c.set.k6_guard(o).map(|g| g.body.pos) else { return };
    dbg.0 = Some([pos[0], pos[1], pos[2] + HIT_STAR_Z * 0.5]);
    st.0 += 1;
    if st.0 == n {
        let hit = GuardHit { src: PLAYER_HANDLE, target: o as u32 + 1, kind: 2, amount: 0, pos: [pos[0], pos[1], pos[2], 1.0], vec: [1.0, 0.0, 0.0, 0.0], f_44: 0x20 };
        let taken = c.set.ee_sub_1f7598(&hit, &mut c.host);
        info!("hit_debug: frame {n}: guard {o} at {pos:?} hit taken {taken}");
    }
}

/// SLY_A7_FX: point the camera looks at (the picked coin's start position).
#[derive(Resource, Default)]
pub struct FxDebugAt(pub Option<[f32; 3]>);

pub fn fx_debug_camera(dbg: Res<FxDebugAt>, mut cam: Query<&mut Transform, With<CmCamera>>) {
    let Some(at) = dbg.0 else { return };
    let eye = [at[0] - 260.0, at[1] - 260.0, at[2] + 140.0];
    let b = |v: [f32; 3]| Vec3::new(v[0], v[2], -v[1]);
    for mut t in &mut cam {
        *t = Transform::from_translation(b(eye)).looking_at(b([at[0], at[1], at[2] + 20.0]), Vec3::Y);
    }
}
