//! WP-C8: screen-space visuals of a playable episode.
//!
//! Screen wipe (level transitions, death reloads, mission starts): the global at 0x443740 is a
//! small state machine around a level object (`oid`, found with FUN_0015e938(0x3dd490, 0x405, oid))
//! whose own animation draws the wipe:
//!   +0 state (-1 none, 0 idle, 1 closing, 2 opening, 3 closed), +4 request held, +8 the 0x4c-byte
//!   FUN_0022b490 request record, +0x54 wipe object, +0x58 wipe oid.
//!   FUN_0022b050 init, FUN_0022b090 set state, FUN_0022b220 begin, FUN_0022b198 per-frame poll
//!   (from plugin_ui_update FUN_001d9648), FUN_0022b420 after the level load (FUN_0020e890).
//!   Object messages: vtable +0x18c (0x311 close, 0x30f open), +0x194 stop, job_flow_feed_playing
//!   (obj, 0x314) = the close / open animation is still running.
//! FUN_0022b490(world, desc, spawn, parent, flags, a): a == -1 posts the request straight away
//! (FUN_0020df80), otherwise it goes through the wipe with oid `a` (0x30a in most scripts).
//!
//! WP-C9: FUN_001618f8(secs) / FUN_00161958 are NOT a screen fade: world +0x3bc4 is HUD slot 3 (the
//! countdown label, oid 0x36f, `ui::hud_slots`) and +0x476c its end time; the load carry-over goes to
//! `UiGame::countdown_carry`. `ScreenWipe::fade_in` is no longer called by the game.
//!
//! Sky: class 97 (C2d9500, vtable 0x2d9500) update slot 15 C2d9500_v15 (0x1ef8c8) moves the object to
//! the camera position (g_cm +0x90) every frame, keeping its own z when field +0x150 is 0.
//!
//! REPLACED: the wipe / fade objects' own animations (screen-space ALO flipbooks driven by their
//! job flows) are drawn as one full-screen black overlay (a 2D sprite on its own camera, larger than
//! any window: aspect independent, 32:9 safe, a 2D pass that Solari does not see) whose opacity
//! follows the state machine; job_flow_feed_playing is a fixed duration (`WIPE_SECS`).
#![allow(non_snake_case)]

/// Wipe object messages (vtable +0x18c) and the running query (job_flow_feed_playing).
pub const MSG_CLOSE: u32 = 0x311;
pub const MSG_OPEN: u32 = 0x30f;
pub const MSG_PLAYING: u32 = 0x314;

/// Services FUN_0022b090 / 0x22b198 / 0x22b220 / 0x22b420 call.
pub trait WipeHost<R> {
    /// FUN_0015e938(0x3dd490, 0x405, oid)
    fn find(&mut self, oid: i32) -> Option<u32>;
    /// vtable +0x18c(obj, msg)
    fn send(&mut self, obj: u32, msg: u32);
    /// vtable +0x194(obj)
    fn stop(&mut self, obj: u32);
    /// job_flow_feed_playing(obj, 0x314)
    fn playing(&mut self, obj: u32) -> bool;
    /// FUN_0020df80(0x3dd490, req): post the level request
    fn post(&mut self, req: R);
}

/// The 0x443740 wipe.
#[derive(Clone, Debug, PartialEq)]
pub struct Wipe<R> {
    /// +0
    pub state: i32,
    /// +4 / +8: the request held while the wipe closes
    pub req: Option<R>,
    /// +0x54
    pub obj: Option<u32>,
    /// +0x58
    pub oid: i32,
}

impl<R> Default for Wipe<R> {
    fn default() -> Self {
        // FUN_0022b050 before the FUN_0022b090(0) (that one only clears the already empty request)
        Self { state: 0, req: None, obj: None, oid: -1 }
    }
}

impl<R: Clone> Wipe<R> {
    /// FUN_0022b050 (0x0022b050)
    pub fn FUN_0022b050(&mut self, h: &mut impl WipeHost<R>) {
        self.obj = None;
        self.oid = -1;
        self.state = -1;
        self.FUN_0022b090(h, 0);
    }

    /// FUN_0022b090 (0x0022b090): enter `s`.
    pub fn FUN_0022b090(&mut self, h: &mut impl WipeHost<R>, s: i32) {
        if self.state == s {
            return;
        }
        self.state = s;
        match s {
            0 => {
                self.req = None;
                if let Some(o) = self.obj {
                    h.stop(o);
                }
            }
            1 => self.send(h, MSG_CLOSE),
            2 => self.send(h, MSG_OPEN),
            3 => {
                if let Some(r) = self.req.take() {
                    h.post(r);
                }
            }
            _ => {}
        }
    }

    fn send(&mut self, h: &mut impl WipeHost<R>, msg: u32) {
        // the original dereferences +0x54 unchecked; the begin / after-load paths only enter
        // states 1 / 2 with an object
        if let Some(o) = self.obj {
            h.send(o, msg);
        }
    }

    /// FUN_0022b220 (0x0022b220): begin a wiped transition to `req` with wipe object `oid`
    /// (`next` = the state to enter, 1 from FUN_0022b490). The flags bit-1 "current level"
    /// recompute is done by `flow::resolve` in the port.
    pub fn FUN_0022b220(&mut self, h: &mut impl WipeHost<R>, req: R, oid: i32, next: i32) {
        match self.state {
            1 | 3 => return,
            2 => self.FUN_0022b090(h, 0),
            _ => {}
        }
        self.obj = h.find(oid);
        if self.obj.is_none() {
            h.post(req);
            return;
        }
        self.req = Some(req);
        self.oid = oid;
        self.FUN_0022b090(h, next);
    }

    /// FUN_0022b198 (0x0022b198): per-frame poll.
    pub fn FUN_0022b198(&mut self, h: &mut impl WipeHost<R>) {
        let mut s = self.state;
        if let (1 | 2, Some(o)) = (s, self.obj) {
            if !h.playing(o) {
                s = if s == 1 { 3 } else { 0 };
            }
        }
        self.FUN_0022b090(h, s);
    }

    /// FUN_0022b420 (0x0022b420): after a level load, re-find the wipe object in the new world and
    /// open it if the wipe was closed.
    pub fn FUN_0022b420(&mut self, h: &mut impl WipeHost<R>) {
        self.obj = h.find(self.oid);
        if self.obj.is_none() || self.state != 3 {
            self.FUN_0022b090(h, 0);
        } else {
            self.FUN_0022b090(h, 2);
        }
    }
}

/// Stand-in duration of the wipe object's close / open animation (REPLACED job_flow_feed_playing).
pub const WIPE_SECS: f32 = 0.5;

/// Overlay opacity of a wipe in `state` that entered it `t` seconds ago (REPLACED drawing).
pub fn wipe_alpha(state: i32, t: f32) -> f32 {
    let k = (t / WIPE_SECS).clamp(0.0, 1.0);
    match state {
        1 => k,
        3 => 1.0,
        2 => 1.0 - k,
        _ => 0.0,
    }
}

/// Opacity of a screen fade-in of `secs` ending at `t_end`, now `t` (WP-C9: no longer fed by FUN_001618f8).
pub fn fade_alpha(t_end: f32, secs: f32, t: f32) -> f32 {
    if secs <= 0.0 {
        return 0.0;
    }
    ((t_end - t) / secs).clamp(0.0, 1.0)
}

/// C2d9500_v15 (0x001ef8c8): the sky position for camera `cam` (both in PS2 world space) given
/// its own position `own` and field +0x150 (`follow_z`).
/// The C code calls super FUN_001f36e0 first; in the port the base ALO update runs for all actors
/// in actor/mod.rs slot 15, and the sky-follow position override runs in `update_sky_follow` after.
pub fn C2d9500_v15(cam: [f32; 3], own: [f32; 3], follow_z: bool) -> [f32; 3] {
    if follow_z { cam } else { [cam[0], cam[1], own[2]] }
}

/// WP-C8 translucent order. The PS2 render list (RPL, Sly 1 render.h: +4 z, +0x10 pos) sorts its translucent
/// entries back to front by z (NCmpPrplReverseZ: larger z first, ties keep submission order); z is the
/// distance of the RPL position (the drawn record's centre). Bevy sorts transparent entities by their
/// origin, so a translucent prim is re-centred on its bounds centre (`sort_centre`) and its entity
/// moved there: the same sort key, vertices unchanged in world space.
pub fn sort_centre(pos: &[[f32; 3]]) -> [f32; 3] {
    if pos.is_empty() {
        return [0.0; 3];
    }
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    for p in pos {
        for k in 0..3 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5, (lo[2] + hi[2]) * 0.5]
}

/// WP-C8 blob shadows (Sly 1 alo.h +0x28c sFastShadowRadius / +0x290 depth, RenderFastShadow): a dark
/// disc under the character on the first surface below it. REPLACED: the fast-shadow texture pass and
/// the per-ALO radius (Sly 2 field not identified) -> class radius below, ground = SO segment down.
/// Character classes: 53 = JT (Sly), 57..=67 = guards (class 56 base).
pub fn blob_radius(class: u16) -> Option<f32> {
    match class {
        53 => Some(BLOB_JT),
        57..=67 => Some(BLOB_GUARD),
        _ => None,
    }
}
pub const BLOB_JT: f32 = 45.0;
pub const BLOB_GUARD: f32 = 60.0;
/// how far below the object origin the ground is searched (PS2 units)
pub const BLOB_REACH: f32 = 400.0;
/// opacity at the object, fading to 0 at `BLOB_REACH`
pub const BLOB_ALPHA: f32 = 0.55;

/// Blob placement for an object at `pos`: ground hit z (from a downward segment) -> (z, alpha).
pub fn blob_at(pos: [f32; 3], ground_z: Option<f32>) -> Option<(f32, f32)> {
    let z = ground_z?;
    let h = pos[2] - z;
    if !(-20.0..BLOB_REACH).contains(&h) {
        return None;
    }
    Some((z + 1.0, BLOB_ALPHA * (1.0 - h.max(0.0) / BLOB_REACH)))
}

/// WP-C8 light changes (flicker): a script member write on a LIGHT object (set-o, FUN_00200598 ->
/// LIGHT setter 0x1a35b0.. + FUN_001a3968) as the raw property bytes `light::set_field` takes.
pub fn light_write_bytes(v: &crate::engine::splice::value::Ref) -> Option<Vec<u8>> {
    use crate::engine::splice::value::Ref;
    Some(match v {
        Ref::F32(f) => f.to_le_bytes().to_vec(),
        Ref::S32(i) => i.to_le_bytes().to_vec(),
        Ref::Bool(b) => vec![*b as u8, 0, 0, 0],
        Ref::Vector(v) => v.iter().flat_map(|f| f.to_le_bytes()).collect(),
        _ => return None,
    })
}

#[cfg(feature = "game")]
pub use bevy_side::*;

#[cfg(feature = "game")]
mod bevy_side {
    use super::*;
    use crate::engine::game::flow::Post;
    use crate::engine::world::World;
    use bevy::prelude::*;

    /// The wipe as a Bevy resource; `posted` = requests the wipe released (state 3 / no object),
    /// drained by the level transition. `since` = real time of the last state change.
    #[derive(Resource, Default)]
    pub struct ScreenWipe {
        pub wipe: Wipe<Post>,
        pub posted: Vec<Post>,
        pub since: f32,
        pub now: f32,
        /// FUN_001618f8 fade: (end, length) on the real clock
        pub fade: Option<(f32, f32)>,
        /// FUN_0020e890 loaded a level: run FUN_0022b420 once the World built from it replaced
        /// the one with this object-array address (0 = none) (the port builds worlds async)
        pub pending_load: Option<usize>,
    }

    pub fn world_key(w: Option<&World>) -> usize {
        w.map_or(0, |w| w.objects.as_ptr() as usize)
    }

    struct Host<'a> {
        world: Option<&'a World>,
        posted: &'a mut Vec<Post>,
        playing: bool,
    }

    impl WipeHost<Post> for Host<'_> {
        fn find(&mut self, oid: i32) -> Option<u32> {
            self.world?.find_oid(oid).first().map(|&i| i as u32)
        }
        fn send(&mut self, _obj: u32, _msg: u32) {}
        fn stop(&mut self, _obj: u32) {}
        fn playing(&mut self, _obj: u32) -> bool {
            self.playing
        }
        fn post(&mut self, req: Post) {
            self.posted.push(req);
        }
    }

    impl ScreenWipe {
        fn run(&mut self, world: Option<&World>, f: impl FnOnce(&mut Wipe<Post>, &mut Host)) {
            let before = self.wipe.state;
            let playing = self.now - self.since < WIPE_SECS;
            let mut h = Host { world, posted: &mut self.posted, playing };
            f(&mut self.wipe, &mut h);
            if self.wipe.state != before {
                self.since = self.now;
            }
        }

        /// FUN_0022b490 tail: `a` == -1 posts directly, else FUN_0022b220(0x443740, req, a, 1).
        pub fn request(&mut self, world: Option<&World>, p: Post) {
            if p.a == -1 {
                self.posted.push(p);
                return;
            }
            let a = p.a;
            self.run(world, |w, h| w.FUN_0022b220(h, p, a, 1));
        }

        /// FUN_0020e890 loaded a level (`old` = the World before it): FUN_0022b420 on the new one.
        pub fn level_load_started(&mut self, old: Option<&World>) {
            self.pending_load = Some(world_key(old));
        }

        /// FUN_0022b420 after a level load.
        pub fn level_loaded(&mut self, world: Option<&World>) {
            self.run(world, |w, h| w.FUN_0022b420(h));
        }

        /// FUN_001618f8: fade in over `secs`.
        pub fn fade_in(&mut self, secs: f32) {
            self.fade = (secs > 0.0).then_some((self.now + secs, secs));
        }

        pub fn alpha(&self) -> f32 {
            let w = wipe_alpha(self.wipe.state, self.now - self.since);
            let f = self.fade.map_or(0.0, |(e, s)| fade_alpha(e, s, self.now));
            w.max(f)
        }
    }

    /// Render layer of the wipe camera / sprite.
    pub const WIPE_LAYER: usize = 29;

    /// Full-screen overlay sprite.
    #[derive(Component)]
    pub struct ScreenWipeOverlay;

    /// FUN_0022b198 per frame + overlay opacity.
    pub fn update_screen_wipe(
        time: Res<Time<Real>>,
        mut sw: ResMut<ScreenWipe>,
        world: Option<Res<World>>,
        mut commands: Commands,
        mut q: Query<&mut Sprite, With<ScreenWipeOverlay>>,
    ) {
        sw.now = time.elapsed_secs();
        if let (Some(old), Some(w)) = (sw.pending_load, world.as_deref()) {
            if world_key(Some(w)) != old {
                sw.pending_load = None;
                sw.level_loaded(Some(w));
            }
        }
        sw.run(world.as_deref(), |w, h| w.FUN_0022b198(h));
        let a = sw.alpha();
        match q.single_mut() {
            Ok(mut sp) => sp.color = Color::srgba(0.0, 0.0, 0.0, a),
            Err(_) => {
                // own 2D camera (order 9: over the 3D view, under the HUD camera of order 10) on its own
                // render layer; the sprite is far larger than any window so every aspect is covered
                let layer = bevy::camera::visibility::RenderLayers::layer(WIPE_LAYER);
                // UiCamera: ui::plugin::sync_overlay_camera copies the 3D camera's Hdr / Msaa / main texture usages
                // (Solari runs Msaa::Off; a mismatched sample count is a wgpu validation error)
                commands.spawn((Camera2d, Camera { order: 9, clear_color: ClearColorConfig::None, ..default() }, layer.clone(), crate::engine::ui::plugin::UiCamera));
                commands.spawn((ScreenWipeOverlay, Sprite { color: Color::srgba(0.0, 0.0, 0.0, a), custom_size: Some(Vec2::splat(1.0e6)), ..default() }, Transform::default(), layer));
            }
        }
    }

    /// Class-97 sky marker: own position at spawn and field +0x150.
    #[derive(Component)]
    pub struct SkyFollow {
        pub own: [f32; 3],
        pub follow_z: bool,
    }

    /// Field id of class 97 +0x150 (sym 0x37c).
    pub const SKY_FOLLOW_Z_FIELD: i16 = 978;

    pub fn sky_follow_z(world: &World, i: usize) -> bool {
        use crate::engine::world::objects::{LoadData, Payload};
        let Payload::Loaded(LoadData::Alo { alo, .. }) = &world.objects[i].payload else { return false };
        alo.props.iter().rev().find(|p| p.field == SKY_FOLLOW_Z_FIELD).is_some_and(|p| match &p.value {
            crate::engine::world::props::PropValue::Scalar(b) => b.iter().any(|&x| x != 0),
            _ => false,
        })
    }

    /// Tag the class-97 root objects of a freshly spawned level.
    pub fn tag_sky_objects(mut commands: Commands, world: Option<Res<World>>, q: Query<(Entity, &super::super::game::AloObject, &Transform), Added<super::super::game::AloObject>>) {
        let Some(world) = world else { return };
        for (e, o, t) in &q {
            let Some(lo) = world.objects.get(o.0) else { continue };
            if lo.class == 97 && lo.owner.is_none() {
                commands.entity(e).insert(SkyFollow { own: t.translation.to_array(), follow_z: sky_follow_z(&world, o.0) });
            }
        }
    }

    /// C2d9500_v15 per frame. Root children are in PS2 world space (the root carries the z-up turn).
    pub fn update_sky_follow(cm: Option<Res<crate::engine::camera::cm::Cm>>, mut q: Query<(&SkyFollow, &mut Transform)>) {
        let Some(cm) = cm else { return };
        for (s, mut t) in &mut q {
            t.translation = Vec3::from_array(C2d9500_v15(cm.pos(), s.own, s.follow_z));
        }
    }

    /// G5: script LIGHT writes -> `LightPropQueue` (applied by `apply_light_props`).
    pub fn drain_light_writes(st: Option<NonSendMut<crate::engine::game::vm_state::SpliceState>>, q: Option<ResMut<crate::engine::gfx::game::LightPropQueue>>) {
        let (Some(mut st), Some(mut q)) = (st, q) else { return };
        for (obj, field, v) in st.sched.objs.light_writes.drain(..) {
            if let Some(value) = light_write_bytes(&v) {
                q.0.push(crate::engine::gfx::game::LightPropSet { object: obj as usize, field: field as i16, value, world: None });
            }
        }
    }

    #[derive(Component)]
    pub struct BlobShadow(pub Entity);

    /// G4: one disc per character entity, placed on the SO ground below it every frame.
    #[allow(clippy::type_complexity)]
    pub fn update_blob_shadows(
        mut commands: Commands,
        world: Option<Res<World>>,
        pw: Option<Res<crate::engine::player::plugin::PlayerWorldData>>,
        roots: Query<Entity, With<crate::engine::gfx::game::LevelGeometryRoot>>,
        chars: Query<(Entity, &crate::engine::gfx::game::AloObject, &GlobalTransform, &InheritedVisibility)>,
        mut blobs: Query<(Entity, &BlobShadow, &mut Transform, &MeshMaterial3d<StandardMaterial>)>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut mats: ResMut<Assets<StandardMaterial>>,
        mut disc: Local<Option<Handle<Mesh>>>,
    ) {
        let (Some(world), Ok(root)) = (world, roots.single()) else { return };
        let cw = pw.as_ref().and_then(|p| p.cw.clone());
        let mut have = std::collections::HashSet::new();
        for (b, s, mut t, m) in &mut blobs {
            let Ok((_, o, g, vis)) = chars.get(s.0) else {
                commands.entity(b).despawn();
                continue;
            };
            have.insert(s.0);
            let r = world.objects.get(o.0).and_then(|lo| blob_radius(lo.class)).unwrap_or(0.0);
            // Bevy world -> PS2 (root = Rx(-90deg)): ps2 = (x, -z, y)
            let p = g.translation();
            let pos = [p.x, -p.z, p.y];
            let ground = cw.as_ref().and_then(|cw| cw.clip_segment(pos, [pos[0], pos[1], pos[2] - BLOB_REACH]).first().map(|h| h.pos[0][2]));
            match blob_at(pos, ground).filter(|_| vis.get()) {
                Some((z, a)) => {
                    *t = Transform::from_xyz(pos[0], pos[1], z).with_scale(Vec3::splat(r));
                    if let Some(mut mat) = mats.get_mut(&m.0) {
                        mat.base_color = Color::srgba(0.0, 0.0, 0.0, a);
                    }
                }
                None => t.scale = Vec3::ZERO,
            }
        }
        for (e, o, _, _) in &chars {
            if have.contains(&e) || world.objects.get(o.0).and_then(|lo| blob_radius(lo.class)).is_none() {
                continue;
            }
            let d = disc.get_or_insert_with(|| meshes.add(Circle::new(1.0))).clone();
            let mat = mats.add(StandardMaterial { base_color: Color::NONE, unlit: true, alpha_mode: AlphaMode::Blend, depth_bias: 1.0, fog_enabled: false, ..default() });
            commands.spawn((BlobShadow(e), Mesh3d(d), MeshMaterial3d(mat), Transform::from_scale(Vec3::ZERO), bevy::light::NotShadowCaster, ChildOf(root)));
        }
    }

    pub struct ScreenPlugin;

    impl Plugin for ScreenPlugin {
        fn build(&self, app: &mut App) {
            app.init_resource::<ScreenWipe>()
                .add_systems(Update, (update_screen_wipe, drain_light_writes.before(crate::engine::gfx::game::apply_light_props)))
                .add_systems(PostUpdate, update_blob_shadows.after(bevy::transform::TransformSystems::Propagate))
                .add_systems(PostUpdate, (tag_sky_objects, update_sky_follow).chain().before(bevy::transform::TransformSystems::Propagate));
        }
    }
}
