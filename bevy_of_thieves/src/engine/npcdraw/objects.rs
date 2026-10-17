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
}

pub struct ObjDrawPlugin;
impl Plugin for ObjDrawPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ObjDraw>()
            .add_systems(Update, (setup_obj_draw, drive_obj_draw).chain().after(tick_classes).before(lend_classes))
            .add_systems(Update, obj_camera.after(sync_cm_camera).after(drive_obj_draw).after(super::plugin::npc_camera));
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
    parts: &Query<(&Transform, Option<&Mesh3d>, Option<&MeshMaterial3d<StandardMaterial>>, Option<&InkOutline>, Option<&SkinnedMesh>)>,
    mesh_assets: &mut Assets<Mesh>,
) -> Entity {
    let root = commands.spawn((affine_to_transform(m), Visibility::Hidden, ChildOf(parent))).id();
    let mut map: HashMap<Entity, Entity> = HashMap::new();
    let mut skinned: Vec<(Entity, Entity)> = vec![];
    let mut stack: Vec<(Entity, Entity)> = kids.get(src).map(|c| c.iter().map(|k| (k, root)).collect()).unwrap_or_default();
    while let Some((e, p)) = stack.pop() {
        let Ok((t, mesh, mat, ink, sm)) = parts.get(e) else { continue };
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
        let Ok((_, _, _, _, Some(sm))) = parts.get(e) else { continue };
        let joints = sm.joints.iter().map(|j| map.get(j).copied().unwrap_or(root)).collect();
        // A10: posed culling bounds (the shared mesh asset carries the per-joint bounds, gfx::game)
        commands.entity(n).insert((SkinnedMesh { inverse_bindposes: sm.inverse_bindposes.clone(), joints }, bevy::camera::visibility::DynamicSkinnedMeshBounds));
    }
    root
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
    parts: Query<(&Transform, Option<&Mesh3d>, Option<&MeshMaterial3d<StandardMaterial>>, Option<&InkOutline>, Option<&SkinnedMesh>)>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
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
pub fn drive_obj_draw(level: Res<ClassesLevel>, mut classes: ResMut<Classes>, mut od: ResMut<ObjDraw>, mut tf: Query<&mut Transform>, mut vq: Query<&mut Visibility>) {
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
