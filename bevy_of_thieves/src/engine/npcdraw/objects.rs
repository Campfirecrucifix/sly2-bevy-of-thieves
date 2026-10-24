//! WP-A7 (feature `game`): draw the K13 / K14 gameplay objects from their real level entities.
//!
//! - class 81 projectiles (K13 `C2d3d20`): every instance (template or pool clone) gets a copy of its
//!   template's entity subtree, shown while it is in flight and placed at its physics body (xfLin +0x30 /
//!   xfAng rows: `Proj::phys`). In flight = launched: FUN_0013a9e8 sets +0x200 = self, FUN_001d6b00's
//!   FUN_0013bc60 sets +0x200 = the launcher (`ClassSet::k14_fire`). The template's own level entities stay
//!   at their rest place until the template itself is launched (then hidden; the copy flies).
//! - class 89 water volumes (K13 `C2dca30`): templates are level geometry already; clones (no entities of
//!   their own) get a copy of the template subtree at the clone's placement (bone 0 world * placed, as
//!   npcdraw).
//! - class 76 swarm riders (K14 `C2d0b30`): slot 18 FUN_001821c0 gives one matrix per alive rider and calls
//!   FUN_001a5ee8(1, 1, model, m), which draws the model ALO (+0x1c4) with inverse(model world) * m on top of
//!   its world: the model subtree moved rigidly so its root sits at m. Each rider is a copy of the model's
//!   entity subtree with root transform m; copies beyond the alive count are hidden.
//!
//! Copies share mesh / material / inverse-bindpose assets (Solari: same meshes); ink outlines get their own
//! LineList asset (update_ink_outlines rewrites it per entity); skinned prims get fresh joint entities in
//! their rest pose (these objects are not animated by the ported classes).
//!
//! Debug aid for captures: SLY_A7_FIRE=1 places every class-81 template at its rest place and fires it from
//! the first class-112 launcher record of the level (stand-in for the Splice method 468 / class-69 / class-55
//! callers of FUN_001d6b00, none of which is driven yet).

use crate::engine::camera::bevy_sync::{sync_cm_camera, CmCamera};
use crate::engine::classes::k7::K7;
use crate::engine::classes::plugin::{tick_classes, Classes, ClassesLevel};
use crate::engine::classes::Inst;
use crate::engine::game::app::lend_classes;
use crate::engine::gfx::attach::BoneWorlds;
use crate::engine::gfx::game::{affine_to_transform, AloObject, GfxLevelData, InkOutline, LevelGeometryRoot};
use crate::engine::gfx::scene::{Affine, LevelScene};
use bevy::mesh::skinning::SkinnedMesh;
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

pub const CLASS_PROJ: u16 = 81;
pub const CLASS_WATER: u16 = 89;

/// One class-81 projectile copy: (object, copy root, template level entity when object == template).
pub struct ProjDrawn {
    pub obj: usize,
    pub root: Entity,
    pub level: Option<Entity>,
}

/// One class-76 swarm: (object, rider copy roots).
pub struct SwarmDrawn {
    pub obj: usize,
    pub copies: Vec<Entity>,
}

#[derive(Resource, Default)]
pub struct ObjDraw {
    pub key: usize,
    pub proj: Vec<ProjDrawn>,
    pub water: Vec<(usize, Entity)>,
    pub swarm: Vec<SwarmDrawn>,
    pub fired: bool,
    /// rest level world of the class-81 templates (SLY_A7_FIRE)
    pub rest: HashMap<usize, Affine>,
    /// SLY_A7_CAM aid: (position, forward) of the object the camera looks at this frame
    pub focus: Option<([f32; 3], [f32; 3])>,
    /// LOOK R34: pool copies standing on an owner bone (entity, template rest world, socket matrix, bone):
    /// world = rest x socket x bone world each frame, so a copy follows its animated owner bone
    /// (EP4 searchlight beams on the turning lamp heads)
    pub follow: Vec<(Entity, Affine, Affine, crate::engine::anim::bind::BoneKey)>,
    /// LOOK R34: K12 (class 107 hub job marker) slot 18 FUN_0022a908 draws: (marker object, glow slot 0 = +0x16c
    /// sign / 2 = +0x174 base, copy entity, glow object rest world, marker world position)
    pub markers: Vec<(usize, usize, Entity, Affine, [f32; 3])>,
    /// LOOK R36: per marker copy root: the fade alpha now on its materials and (prim entity, authored
    /// material, faded clone) of every prim under it (built on the first fade)
    pub marker_mats: HashMap<Entity, (f32, Vec<(Entity, Handle<StandardMaterial>, Handle<StandardMaterial>)>)>,
    /// LOOK R39: per marker: the +0x16c sign glow object's +0xa0 (its MeshSet bound radius)
    pub marker_r: HashMap<usize, f32>,
    /// LOOK R40: K12 marker copy prims on a sign bone: (copy prim, bone keys (object, template), rest bone 0
    /// world, bone 0 key): local = posed bone world x inverse(bone 0 world) each frame
    pub marker_bones: Vec<(Entity, [crate::engine::anim::bind::BoneKey; 2], Affine, crate::engine::anim::bind::BoneKey)>,
}

pub struct ObjDrawPlugin;
impl Plugin for ObjDrawPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ObjDraw>()
            .init_resource::<crate::engine::ui::world_label::WorldLabels>()
            .add_systems(Update, (setup_obj_draw, drive_obj_draw, fade_k12_markers, crate::engine::ui::world_label::draw_world_labels).chain().after(tick_classes).before(lend_classes))
            .add_systems(Update, obj_camera.after(sync_cm_camera).after(drive_obj_draw).after(super::plugin::npc_camera));
    }
}

/// LOOK R36: FUN_0022a908 objects_drive_obj_draw(a, a, glow, m) / FUN_001a5fb8(.., a, ..): the marker draws its
/// sign / beam / base with the fade alpha a (vt +0xa4 draw alpha). a < 1 -> the copy's prims get clones of
/// their materials with alpha x a (opaque ones blended); a = 1 puts the authored materials back.
#[allow(clippy::type_complexity)]
pub fn fade_k12_markers(
    mut commands: Commands,
    classes: Res<Classes>,
    lent: Option<NonSend<crate::engine::game::vm_state::SpliceState>>,
    mut od: ResMut<ObjDraw>,
    kids: Query<&Children>,
    mq: Query<&MeshMaterial3d<StandardMaterial>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let od = &mut *od;
    for &(i, _, root, _, _) in &od.markers {
        // ANIM-ALL R79: the class set lent to the scripts between frames (WIRE-EP8; `Classes::set` is empty then)
        let Some(k) = lent.as_ref().and_then(|s| s.sched.cls.set.as_ref()).unwrap_or(&classes.set).k12_get_ref(i) else { continue };
        let a = if std::env::var("SLY2_LOOK_K12FORCE").is_ok_and(|v| v == "2" || k.f_150 == 1) { 1.0 } else { k.draw_a };
        // LOOK R36 diag: SLY2_LOOK_K12A=x overrides the fade alpha of the drawn markers (check the faded look)
        let a = match std::env::var("SLY2_LOOK_K12A").ok().and_then(|v| v.parse::<f32>().ok()) {
            Some(x) if a > 0.0 => x,
            _ => a,
        };
        let a = if a > 0.0 { a.min(1.0) } else { continue };
        let st = od.marker_mats.entry(root).or_insert((1.0, Vec::new()));
        if st.0 == a {
            continue;
        }
        if st.1.is_empty() && a < 1.0 {
            let mut stack = vec![root];
            while let Some(e) = stack.pop() {
                if let Ok(m) = mq.get(e) {
                    if let Some(src) = mats.get(&m.0).cloned().filter(|x| x.base_color.alpha() > 0.0) {
                        let h = mats.add(src);
                        st.1.push((e, m.0.clone(), h));
                    }
                }
                if let Ok(c) = kids.get(e) {
                    stack.extend(c.iter());
                }
            }
        }
        for (e, orig, h) in &st.1 {
            if a >= 1.0 {
                commands.entity(*e).insert(MeshMaterial3d(orig.clone()));
                continue;
            }
            let Some(o) = mats.get(orig).cloned() else { continue };
            if let Some(mut m) = mats.get_mut(h) {
                let mut c = o.base_color;
                c.set_alpha(o.base_color.alpha() * a);
                m.base_color = c;
                if m.alpha_mode == AlphaMode::Opaque || matches!(m.alpha_mode, AlphaMode::Mask(_)) {
                    m.alpha_mode = AlphaMode::Blend;
                }
            }
            commands.entity(*e).insert(MeshMaterial3d(h.clone()));
        }
        st.0 = a;
    }
}

/// Copy the children of a level object entity (prims, joints, sub-object entities, recursively) under a
/// new root with transform `m` (level space). Returns the root.
#[allow(clippy::type_complexity)]
pub fn copy_subtree(
    commands: &mut Commands,
    parent: Entity,
    m: &Affine,
    src: Entity,
    kids: &Query<&Children>,
    parts: &Query<(&Transform, Option<&Mesh3d>, Option<&MeshMaterial3d<StandardMaterial>>, Option<&InkOutline>, Option<&SkinnedMesh>, Option<&crate::engine::gfx::twolayer::TwoLayerSwapped>)>,
    mesh_assets: &mut Assets<Mesh>,
) -> Entity {
    copy_subtree_map(commands, parent, m, src, kids, parts, mesh_assets).0
}

/// `copy_subtree` that also returns the (source entity -> copy entity) map of the copied nodes.
#[allow(clippy::type_complexity)]
pub fn copy_subtree_map(
    commands: &mut Commands,
    parent: Entity,
    m: &Affine,
    src: Entity,
    kids: &Query<&Children>,
    parts: &Query<(&Transform, Option<&Mesh3d>, Option<&MeshMaterial3d<StandardMaterial>>, Option<&InkOutline>, Option<&SkinnedMesh>, Option<&crate::engine::gfx::twolayer::TwoLayerSwapped>)>,
    mesh_assets: &mut Assets<Mesh>,
) -> (Entity, HashMap<Entity, Entity>) {
    let root = commands.spawn((affine_to_transform(m), Visibility::Hidden, ChildOf(parent))).id();
    let mut map: HashMap<Entity, Entity> = HashMap::new();
    let mut skinned: Vec<(Entity, Entity)> = vec![];
    let mut stack: Vec<(Entity, Entity)> = kids.get(src).map(|c| c.iter().map(|k| (k, root)).collect()).unwrap_or_default();
    while let Some((e, p)) = stack.pop() {
        let Ok((t, mesh, mat, ink, sm, sw)) = parts.get(e) else { continue };
        // PERF2: a merged two-layer prim copies its WIRE-C3 originals (swapped again by gfx::twolayer)
        let (mesh, mat) = match sw {
            Some(s) => (Some(&s.mesh), Some(&s.mat)),
            None => (mesh, mat),
        };
        let n = commands.spawn((*t, Visibility::Inherited, ChildOf(p))).id();
        map.insert(e, n);
        match (mesh, mat, ink) {
            (Some(mesh), Some(mat), Some(ink)) => {
                let h = mesh_assets.get(&mesh.0).cloned().map(|m| mesh_assets.add(m)).unwrap_or_else(|| mesh.0.clone());
                commands.entity(n).insert((Mesh3d(h), mat.clone(), ink.clone()));
            }
            (Some(mesh), Some(mat), None) => {
                commands.entity(n).insert((mesh.clone(), mat.clone()));
                if sm.is_some() {
                    skinned.push((e, n));
                }
            }
            _ => {}
        }
        if let Ok(c) = kids.get(e) {
            stack.extend(c.iter().map(|k| (k, n)));
        }
    }
    for (e, n) in skinned {
        let Ok((_, _, _, _, Some(sm), _)) = parts.get(e) else { continue };
        let joints = sm.joints.iter().map(|j| map.get(j).copied().unwrap_or(root)).collect();
        // A10: posed culling bounds (the shared mesh asset carries the per-joint bounds, gfx::game)
        commands.entity(n).insert((SkinnedMesh { inverse_bindposes: sm.inverse_bindposes.clone(), joints }, bevy::camera::visibility::DynamicSkinnedMeshBounds));
    }
    (root, map)
}

/// Once per world: copies for the class-81 / class-89 clone / class-76 rider objects.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn setup_obj_draw(
    mut commands: Commands,
    level: Res<ClassesLevel>,
    data: Option<Res<GfxLevelData>>,
    mut classes: ResMut<Classes>,
    mut od: ResMut<ObjDraw>,
    roots: Query<Entity, With<LevelGeometryRoot>>,
    objs: Query<(Entity, &AloObject)>,
    kids: Query<&Children>,
    parts: Query<(&Transform, Option<&Mesh3d>, Option<&MeshMaterial3d<StandardMaterial>>, Option<&InkOutline>, Option<&SkinnedMesh>, Option<&crate::engine::gfx::twolayer::TwoLayerSwapped>)>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    tags: Query<&crate::engine::gfx::pick::PrimTag>,
) {
    let (Some(w), Some(data)) = (level.0.clone(), data) else { return };
    let key = Arc::as_ptr(&w) as usize;
    if od.key == key {
        return;
    }
    let Some(level_root) = roots.iter().next() else { return };
    let ent: HashMap<usize, Entity> = objs.iter().map(|(e, o)| (o.0, e)).collect();
    if ent.is_empty() {
        return;
    }
    let scene = LevelScene::build_from(data.lsn, &data.data, &w);
    let world_of: HashMap<usize, Affine> = scene.objects.iter().enumerate().map(|(i, so)| (so.object, scene.world_of(i))).collect();
    let c = &mut *classes;
    let mut out = ObjDraw { key, ..Default::default() };
    let tpl = |o: usize| super::template_of(&w, o);
    // class 81
    for (o, x) in &c.set.k13 {
        if !matches!(x, Inst::K13(_)) {
            continue;
        }
        let Some(&src) = ent.get(&tpl(*o)) else { continue };
        let m = world_of.get(&tpl(*o)).copied().unwrap_or(crate::engine::gfx::scene::IDENTITY);
        let root = copy_subtree(&mut commands, level_root, &m, src, &kids, &parts, &mut mesh_assets);
        out.proj.push(ProjDrawn { obj: *o, root, level: (tpl(*o) == *o).then_some(src) });
    }
    // class 89 clones
    let mut bw = BoneWorlds::default();
    for (o, x) in &c.set.k13 {
        if !matches!(x, Inst::K13Water(_)) || tpl(*o) == *o {
            continue;
        }
        let Some(&src) = ent.get(&tpl(*o)) else { continue };
        let m = super::objdata::clone_world(&w, &mut bw, *o);
        let root = copy_subtree(&mut commands, level_root, &m, src, &kids, &parts, &mut mesh_assets);
        commands.entity(root).insert(Visibility::Inherited);
        out.water.push((*o, root));
    }
    // class 76 riders
    for (o, x) in &c.set.k14 {
        let Inst::K14(s) = x else { continue };
        let Some(&src) = super::objdata::swarm_model(&w, s).and_then(|m| ent.get(&m)) else { continue };
        let n = s.riders.len();
        let copies = (0..n).map(|_| copy_subtree(&mut commands, level_root, &crate::engine::gfx::scene::IDENTITY, src, &kids, &parts, &mut mesh_assets)).collect();
        out.swarm.push(SwarmDrawn { obj: *o, copies });
    }
    out.rest = out.proj.iter().filter(|d| d.level.is_some()).filter_map(|d| world_of.get(&d.obj).map(|m| (d.obj, *m))).collect();
    // LOOK R27: FUN_001caec8 pool claims of the static template copies (the load's deferred FUN_001ca8c0 spawner
    // nodes, `spawn_q`): per copy object vt+0x54(spawner +0x10, +0x50) and vt+0x12c(spawner +0x60), then +0x44
    // Add, so the copy stands at the spawner (hub cafe tables / plants / flower boxes). The class-driven copies
    // (K6 guards, K15 bodies, class 81 / 89) are placed by their own draw; the stream object (copy 0) has a level
    // entity that is moved, a Payload::Clone copy gets a copy of its template's entity subtree.
    // REPLACED: placement = template rest world x the spawner's world (k24) instead of the vt+0x54 parent bone.
    let mut claims: Vec<(u32, usize, i32)> = c.host.pool_cur.iter().map(|(&s, &(e, k))| (s, e, k)).collect();
    claims.sort();
    let mut pooled = 0;
    for (sp, e, k) in claims {
        let Some(en) = w.entries.get(e) else { continue };
        let m = en.objects.len();
        for j in 0..m {
            let Some(&o) = en.instances.get(k as usize * m + j) else { continue };
            let cls = w.objects[o].class;
            if crate::engine::gfx::scene::copy_drawn_elsewhere(cls) || c.set.k15_101_get(o).is_some() {
                continue;
            }
            let Some(&rest) = world_of.get(&tpl(o)) else { continue };
            // REPLACED: an owner without a bone socket -> its rest / clone world (npcdraw clone_world)
            let mut on_bone = None;
            let spw = match crate::engine::gfx::attach::spawner_socket(&w, sp as usize - 1) {
                Some((crate::engine::gfx::attach::BoneRef::Bone(ob, bk), pm)) => {
                    on_bone = Some((pm, (ob, bk)));
                    crate::engine::gfx::scene::mul(&pm, &bw.get(&w, ob, bk))
                }
                Some((_, pm)) => match w.objects[sp as usize - 1].owner {
                    Some(ow) => crate::engine::gfx::scene::mul(&pm, &world_of.get(&ow).copied().unwrap_or_else(|| super::objdata::clone_world(&w, &mut bw, ow))),
                    None => pm,
                },
                None => continue,
            };
            let mw = crate::engine::gfx::scene::mul(&rest, &spw);
            let e = if let Some(&eo) = ent.get(&o).filter(|_| w.objects[o].owner.is_none()) {
                commands.entity(eo).insert(affine_to_transform(&mw));
                eo
            } else if let Some(&src) = ent.get(&tpl(o)) {
                let root = copy_subtree(&mut commands, level_root, &mw, src, &kids, &parts, &mut mesh_assets);
                commands.entity(root).insert(Visibility::Inherited);
                root
            } else {
                continue;
            };
            if let Some((pm, key)) = on_bone {
                out.follow.push((e, rest, pm, key));
            }
            pooled += 1;
            if std::env::var_os("SLY2_LOOK_ORIGINDBG").is_some() {
                let spo = &w.objects[sp as usize - 1];
                let ow = spo.owner.map(|x| (x, w.objects[x].oid, w.objects[x].class, w.objects[x].owner.map(|y| (w.objects[y].oid, w.objects[y].class))));
                let sock = crate::engine::gfx::attach::spawner_socket(&w, sp as usize - 1).map(|(b, _)| format!("{b:?}"));
                info!("POOLDRAW sp {} oid {} cls {} owner {ow:?} sock {sock:?} -> obj {o} oid {} cls {cls} entity {} at {:?}", sp - 1, spo.oid, spo.class, w.objects[o].oid, ent.contains_key(&o), mw[3]);
            }
        }
    }
    info!("objdraw: {pooled} pool copies placed at their spawners");
    // LOOK R34: FUN_0022a908 objects_drive_obj_draw(a, a, +0x16c / +0x174, matrix): the class draws the shared
    // glow objects once per marker at its own matrix, so each marker gets a copy (hidden until drawn); the glow
    // objects are not drawn at their own (rest) place.
    let mut glow_src: Vec<Entity> = Vec::new();
    for (i, x) in &c.set.k12 {
        let crate::engine::classes::Inst::C2dcc98(k) = x else { continue };
        let mp = world_of.get(i).copied().unwrap_or_else(|| super::objdata::clone_world(&w, &mut bw, *i))[3];
        for slot in [0usize, 1, 2] {
            let Some(o) = (k.f_16c[slot] as usize).checked_sub(1) else { continue };
            let Some(&src) = ent.get(&o).or_else(|| ent.get(&tpl(o))) else { continue };
            let rest = world_of.get(&o).or_else(|| world_of.get(&tpl(o))).copied().unwrap_or(crate::engine::gfx::scene::IDENTITY);
            let (root, map) = copy_subtree_map(&mut commands, level_root, &rest, src, &kids, &parts, &mut mesh_assets);
            commands.entity(root).insert(Visibility::Hidden);
            // LOOK R40: FUN_001a5ee8 draws each record on its bone relative to the object (record +4 = bone
            // world): the sign's field-107 ASEG (ANIM R45) poses its glow bones, so each copied prim takes
            // bone world x inverse(bone 0 world) of the posed sign (npcdraw::plugin drive, as `follow`)
            let so_o = [o, tpl(o)].into_iter().find(|oo| scene.objects.iter().any(|so| so.object == *oo));
            if let Some((oo, crate::engine::world::objects::Payload::Loaded(crate::engine::world::objects::LoadData::Alo { alo, .. }))) = so_o.map(|oo| (oo, &w.objects[oo].payload)) {
                // the scene world of the sign (the posed bones are level world; BoneWorlds bone 0 of a
                // child ALO is owner-relative)
                let root0 = world_of.get(&oo).copied().unwrap_or(rest);
                for (se, ce) in &map {
                    let Ok(tag) = tags.get(*se) else { continue };
                    let Some(rec) = alo.mesh.records.get(tag.record) else { continue };
                    if rec.r#ref < 0 || !rec.instances.is_empty() {
                        continue;
                    }
                    let b = rec.r#ref as usize;
                    if std::env::var_os("SLY2_LOOK_K12LOG").is_some() {
                        info!("K12BONESET marker {i} slot {slot} obj {oo} rec {} shader {:?} bone {b} prim {ce:?}", tag.record, tag.shader);
                    }
                    out.marker_bones.push((*ce, [(o, b), (tpl(o), b)], root0, (oo, 0)));
                }
            }
            if slot == 0 {
                // +0xa0 = the ALO's MeshSet bound radius (FUN_00167320 load; EP4: port 105.2403 / 103.14649 /
                // 231.16406 for the glows of 1426 / 1537 / 4014 = PS2 028 +0xa0 of the same signs)
                let r = [o, tpl(o)].into_iter().find_map(|oo| match w.objects.get(oo).map(|x| &x.payload) {
                    Some(crate::engine::world::objects::Payload::Loaded(crate::engine::world::objects::LoadData::Alo { alo, .. })) => Some(alo.mesh.radius),
                    _ => None,
                });
                out.marker_r.insert(*i, r.unwrap_or(0.0));
            }
            out.markers.push((*i, slot, root, rest, mp));
            glow_src.push(src);
        }
    }
    for e in glow_src.into_iter().filter(|_| std::env::var_os("SLY2_LOOK_K12KEEP").is_none()) {
        commands.entity(e).insert(Visibility::Hidden);
    }
    info!("objdraw: {} K12 marker glow copies, {} prims on sign bones", out.markers.len(), out.marker_bones.len());
    info!("objdraw: {} projectiles, {} water clones, {} swarms ({} rider copies)", out.proj.len(), out.water.len(), out.swarm.len(), out.swarm.iter().map(|s| s.copies.len()).sum::<usize>());
    *od = out;
}

/// SLY_A7_FIRE: every class-81 template placed at its rest place and fired from the class-112 launcher
/// whose target (+0x50 in the frame of +0x60, as FUN_001d6b00 aims) is nearest beyond 300 units (else the
/// first launcher).
fn fire_all(c: &mut Classes, w: &crate::engine::world::World, world_of: &HashMap<usize, Affine>) {
    use crate::engine::classes::ClassHost;
    let ls: Vec<(usize, [f32; 3])> = c
        .set
        .k7
        .iter()
        .filter_map(|x| match &x.1 {
            Inst::K7(K7::C2d7f40(l)) => Some((x.0, l.f_60, l.f_50)),
            _ => None,
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|(li, f60, f50)| {
            let f = if f60 == 0 { 0 } else { c.host.ee_sub_15e938(0x2002, f60, li as u32 + 1) };
            let o = if f == 0 { [0.0; 3] } else { c.host.ee_sub_1a62e8(f) };
            (li, [o[0] + f50[0], o[1] + f50[1], o[2] + f50[2]])
        })
        .collect();
    let projs: Vec<usize> = c.set.k13.iter().filter(|x| matches!(x.1, Inst::K13(_)) && w.objects[x.0].clone_of.is_none()).map(|x| x.0).collect();
    info!("objdraw: SLY_A7_FIRE launcher targets {ls:?}");
    // the pool templates rest at the origin: start from the player (k6) instead when there is one
    let start = c.set.g.k6_players.first().map(|x| x.1).filter(|p| p[0].abs() < 1.0e8);
    for p in projs {
        let Some(&m0) = world_of.get(&p) else { continue };
        let mut m = m0;
        if let Some(s) = start {
            m[3] = [s[0], s[1], s[2] + 100.0];
        }
        let d = |t: &[f32; 3]| ((t[0] - m[3][0]).powi(2) + (t[1] - m[3][1]).powi(2) + (t[2] - m[3][2]).powi(2)).sqrt();
        let Some(l) = ls.iter().filter(|x| d(&x.1) > 300.0).min_by(|a, b| d(&a.1).total_cmp(&d(&b.1))).or(ls.first()).map(|x| x.0) else { return };
        if let Some(Inst::K13(k)) = c.set.k13.iter_mut().find(|x| x.0 == p).map(|x| &mut x.1) {
            k.p.phys.pos = m[3];
            k.p.phys.rot = [m[0], m[1], m[2]];
        }
        let r = c.set.k14_fire(l, p, &mut c.host);
        info!("objdraw: SLY_A7_FIRE projectile {p} from launcher {l}: {:?}", r.map(|r| (r.aim, r.v)));
    }
}

/// Every frame after the class update: projectile copies follow their bodies, rider copies their matrices.
pub fn drive_obj_draw(level: Res<ClassesLevel>, mut classes: ResMut<Classes>, mut od: ResMut<ObjDraw>, mut tf: Query<&mut Transform>, mut vq: Query<&mut Visibility>, mut labels: ResMut<crate::engine::ui::world_label::WorldLabels>) {
    labels.0.clear();
    let Some(w) = level.0.clone() else { return };
    if od.key != Arc::as_ptr(&w) as usize {
        return;
    }
    let mut set_vis = |e: Entity, show: bool| {
        let want = if show { Visibility::Inherited } else { Visibility::Hidden };
        if let Ok(mut v) = vq.get_mut(e) {
            if *v != want {
                *v = want;
            }
        }
    };
    let c = &mut *classes;
    let mode = std::env::var("SLY_A7_CAM").unwrap_or_default();
    let od = &mut *od;
    // SLY_A7_FIRE once the player position is known
    if !od.fired && std::env::var("SLY_A7_FIRE").is_ok() && c.set.g.k6_players.first().is_some_and(|x| x.1[0].abs() < 1.0e8) {
        fire_all(c, &w, &od.rest);
        od.fired = true;
    }
    // LOOK R34: K12 FUN_0022a908 marker draw (alphas from classes::k12 ee_sub_22a908)
    let mut k12_h: HashMap<usize, f32> = HashMap::new();
    for &(i, slot, e, rest, p) in &od.markers {
        let e_ent = e;
        let Some(k) = c.set.k12_get(i).map(|k| (k.draw_a, k.f_150, k.draw_la)) else { continue };
        let (a, mode, la) = k;
        // LOOK R39: FUN_0022a908 tail (once per marker, with the +0x16c sign draw): la > 0 and a label id ->
        // FUN_001cd780(s, sp + DAT_002a2dc0 (0,0,1) x s x sign +0xa0, FUN_0013e368(world, id), .., 0xc0c0c0 | la 255)
        // (a2 = 0x603030 | la 128 is not read by the callee); s / sp computed below
        // LOOK R39 diag: SLY2_LOOK_K12LA=x forces the label alpha (the PS2 shows it only ~1 s after a look-camera frame)
        let la = std::env::var("SLY2_LOOK_K12LA").ok().and_then(|v| v.parse::<f32>().ok()).unwrap_or(la);
        let label = (slot == 0 && la > 0.0).then(|| c.set.k12_label_id(i)).flatten();
        if std::env::var_os("SLY2_LOOK_K12LOG").is_some() && slot == 0 {
            info!("K12LABEL obj {i} la {la} id {:?} r {:?}", c.set.k12_label_id(i), od.marker_r.get(&i));
        }
        // LOOK R34 diag: SLY2_LOOK_K12FORCE=1 opens every mode-1 marker (a = 1) as with a job active (PS2 028 state); =2 every marker (LOOK R39)
        let a = if std::env::var("SLY2_LOOK_K12FORCE").is_ok_and(|v| v == "2" || mode == 1) { 1.0 } else { a };
        if std::env::var_os("SLY2_LOOK_K12LOG").is_some() {
            info!("K12DRAW obj {i} slot {slot} mode {mode} a {a} clk {} f9c {} at {p:?} rest {:?} cam {:?}", c.set.g.t, c.set.g.g_3e0f9c, rest[3], c.set.g.g_2ddfd0);
        }
        set_vis(e, a != 0.0);
        if a == 0.0 {
            continue;
        }
        // FUN_0022a220 +0x180 (once per marker per frame): the sign lift that clears the camera's view
        let mut cam = c.set.g.g_2ddfd0;
        // LOOK R35 diag: SLY2_LOOK_K12CAM=x,y,z replaces the camera position of the K12 draw (verify at a PS2 view)
        if let Some(v) = std::env::var("SLY2_LOOK_K12CAM").ok().map(|v| v.split(',').filter_map(|x| x.trim().parse::<f32>().ok()).collect::<Vec<_>>()).filter(|v| v.len() >= 3) {
            cam = [v[0], v[1], v[2]];
        }
        let h180 = *k12_h.entry(i).or_insert_with(|| match c.host.so.as_deref() {
            Some(cw) if std::env::var_os("SLY2_LOOK_K12NOH").is_none() => {
                let mut it = cw.colliders.iter().flat_map(|cl| cl.geom.edges.iter().filter_map(move |e| Some((*cl.geom.verts.get(e.a as usize)?, *cl.geom.verts.get(e.b as usize)?))));
                crate::engine::classes::k12::ee_sub_22a220(&mut it, cam, p)
            }
            _ => 0.0,
        });
        let b = [p[0], p[1], p[2] + h180];
        // FUN_0011ee08(cam, p, b, 0, 0, &d): distance of the camera to the segment p..b (u clamped to [0, 1])
        let seg = [b[0] - p[0], b[1] - p[1], b[2] - p[2]];
        let ll = seg[0] * seg[0] + seg[1] * seg[1] + seg[2] * seg[2];
        let u = (((cam[0] - p[0]) * seg[0] + (cam[1] - p[1]) * seg[1] + (cam[2] - p[2]) * seg[2]) / ll).clamp(0.0, 1.0);
        let q = [p[0] + seg[0] * u, p[1] + seg[1] * u, p[2] + seg[2] * u];
        let d = ((cam[0] - q[0]).powi(2) + (cam[1] - q[1]).powi(2) + (cam[2] - q[2]).powi(2)).sqrt();
        // s = clamp(0.5 + d (0.0003 + d 0), 1, 5) (0x2c44c0.. / 0x2c44d0, 0x2c44d4); NaN (p == b) -> 1 as the VU
        let s = 0.5 + d * 0.0003;
        let s = if s >= 1.0 && s <= 5.0 { s } else if s > 5.0 { 5.0 } else { 1.0 };
        // sign position auStack_a0 = b + Z s 300 (0x2c44d8)
        let sp = [b[0], b[1], b[2] + s * 300.0];
        if let Some(id) = label {
            let r = od.marker_r.get(&i).copied().unwrap_or(0.0);
            let color = 0x00c0_c0c0 | (((la * 255.0) as i32 as u32 & 0xff) << 24);
            labels.0.push(crate::engine::ui::world_label::WorldLabel { id, pos: [sp[0], sp[1], sp[2] + s * r], f: s, color });
        }
        let m: Affine = if slot == 1 {
            // +0x170 beam FUN_001a5fb8(s, a, obj, p, sp): e = sp - p, n = (p - cam) x e; |n| > 0.01: rows
            // unit(n x e), n s / |n|, e 0.01 (0x3c23d70a), translation p
            let e = [sp[0] - p[0], sp[1] - p[1], sp[2] - p[2]];
            let cv = [p[0] - cam[0], p[1] - cam[1], p[2] - cam[2]];
            let cr = |x: [f32; 3], y: [f32; 3]| [x[1] * y[2] - y[1] * x[2], x[2] * y[0] - y[2] * x[0], x[0] * y[1] - y[0] * x[1]];
            let n = cr(cv, e);
            let ln = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if !(0.01 < ln) {
                set_vis(e_ent, false);
                continue;
            }
            let r0 = cr(n, e);
            let l0 = (r0[0] * r0[0] + r0[1] * r0[1] + r0[2] * r0[2]).sqrt();
            let k = s / ln;
            [[r0[0] / l0, r0[1] / l0, r0[2] / l0], [n[0] * k, n[1] * k, n[2] * k], [e[0] * 0.01, e[1] * 0.01, e[2] * 0.01], p]
        } else {
            // +0x16c at the sign position, +0x174 at the marker; rotation = FUN_001a6318 world rows x s
            // (REPLACED: the glow object's rest rotation)
            let t = if slot == 0 { sp } else { p };
            [
                [rest[0][0] * s, rest[0][1] * s, rest[0][2] * s],
                [rest[1][0] * s, rest[1][1] * s, rest[1][2] * s],
                [rest[2][0] * s, rest[2][1] * s, rest[2][2] * s],
                t,
            ]
        };
        let t = m[3];
        // TODO the alpha a (fVar10) multiplies the glow material alpha; drawn opaque-as-authored here
        let ok = if let Ok(mut tr) = tf.get_mut(e) {
            *tr = affine_to_transform(&m);
            true
        } else {
            false
        };
        if std::env::var_os("SLY2_LOOK_K12LOG").is_some() {
            info!("K12PUT obj {i} slot {slot} tf {ok} s {s} d {d} h180 {h180} at {t:?}");
        }
    }
    let mut focus = None;
    for d in &od.water {
        if mode == "water" && focus.is_none() {
            focus = tf.get(d.1).ok().map(|t| ([t.translation.x, t.translation.y, t.translation.z], [1.0, 0.0, 0.0]));
        }
    }
    for d in &od.proj {
        let Some(Inst::K13(k)) = c.set.k13.iter().find(|x| x.0 == d.obj).map(|x| &x.1) else { continue };
        let flying = super::objdata::proj_in_flight(d.obj, &k.p);
        if flying {
            if let Ok(mut t) = tf.get_mut(d.root) {
                *t = affine_to_transform(&super::objdata::proj_world(&k.p));
            }
            if mode == "proj" && focus.is_none() {
                focus = Some((k.p.phys.pos, k.p.phys.v));
            }
        }
        set_vis(d.root, flying);
        if let (Some(e), true) = (d.level, flying) {
            set_vis(e, false);
        }
    }
    for d in &od.swarm {
        let Some(Inst::K14(s)) = c.set.k14.iter().find(|x| x.0 == d.obj).map(|x| &x.1) else { continue };
        let ms = super::objdata::rider_worlds(s, d.obj, &mut c.host);
        for (i, &e) in d.copies.iter().enumerate() {
            if let Some(m) = ms.get(i) {
                if mode == "rider" && focus.is_none() {
                    focus = Some((m[3], m[0]));
                }
                if let Ok(mut t) = tf.get_mut(e) {
                    *t = affine_to_transform(m);
                }
            }
            set_vis(e, i < ms.len());
        }
    }
    od.focus = focus;
}

/// SLY_A7_CAM=proj|rider|water: camera behind / above the first flying projectile, the first rider or the
/// first water clone (level space z up; the level root maps (x, y, z) -> (x, z, -y)).
pub fn obj_camera(od: Res<ObjDraw>, mut cam: Query<&mut Transform, With<CmCamera>>) {
    let Some((p, f)) = od.focus else { return };
    let n = (f[0] * f[0] + f[1] * f[1]).sqrt().max(1e-4);
    let (c, s) = (f[0] / n, f[1] / n);
    let d: f32 = std::env::var("SLY_A7_CAM_DIST").ok().and_then(|v| v.parse().ok()).unwrap_or(600.0);
    let eye = [p[0] - c * d + s * d * 0.5, p[1] - s * d - c * d * 0.5, p[2] + d * 0.5];
    let b = |v: [f32; 3]| Vec3::new(v[0], v[2], -v[1]);
    for mut t in &mut cam {
        *t = Transform::from_translation(b(eye)).looking_at(b(p), Vec3::Y);
    }
}
