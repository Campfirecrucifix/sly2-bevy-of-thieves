//! UPSCALE: render-scale + upscale passes (port addition, no PS2 counterpart).
//! Native scale = the old path (CM camera draws straight to the window). Below native the CM camera draws
//! into `lo` (render size, `RenderTarget::Image`), a Camera2d (order 1, `LAYER_EASU`) runs the own EASU pass
//! (`easu.wesl`) into `hi` (window size), a second Camera2d (order 2, `LAYER_RCAS`) runs RCAS (`rcas.wesl`)
//! into the window. HUD / wipe / FMV cameras (orders 9, 10, 1000) still draw to the window -> native res.
//! Ray traced GI: Solari runs on the CM camera's main texture, so `gi_scale` (render scale while RT is on)
//! lowers the GI/lighting resolution together with the main pass (Solari's own buffers follow the view size).
//! Backends: `UpscaleBackend` (spatial, temporal = TAA at render res + spatial); DLSS/XeSS would implement it
//! behind an optional feature and replace the EASU/RCAS cameras with Bevy's `MainPassResolutionOverride`.
pub mod opts;
pub use opts::UpscaleOpts;

#[cfg(feature = "game")]
pub use imp::*;

#[cfg(feature = "game")]
mod imp {
    use super::UpscaleOpts;
    use crate::engine::camera::bevy_sync::CmCamera;
    use bevy::anti_alias::taa::TemporalAntiAliasing;
    use bevy::camera::visibility::RenderLayers;
    use bevy::camera::{ImageRenderTarget, RenderTarget};
    use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
    use bevy::prelude::*;
    use bevy::render::render_resource::{AsBindGroup, TextureFormat};
    use bevy::shader::ShaderRef;
    use bevy::sprite_render::{Material2d, Material2dPlugin, MeshMaterial2d};
    use bevy::window::PrimaryWindow;

    pub const LAYER_EASU: usize = 28;
    pub const LAYER_RCAS: usize = 29;

    /// Pluggable upscaler. `spatial` = EASU+RCAS cameras, `temporal` = jittered TAA on the CM camera first.
    /// An SDK backend (DLSS / XeSS, optional feature) returns `spatial() == false` and drives Bevy's
    /// `MainPassResolutionOverride` + its own node instead.
    pub trait UpscaleBackend: Send + Sync + 'static {
        fn name(&self) -> &'static str;
        fn available(&self) -> bool {
            true
        }
        /// TAA jitter + history on the CM camera at render resolution
        fn temporal(&self) -> bool;
        /// uses this module's EASU/RCAS passes
        fn spatial(&self) -> bool {
            true
        }
    }
    pub struct Fsr1;
    impl UpscaleBackend for Fsr1 {
        fn name(&self) -> &'static str {
            "fsr1"
        }
        fn temporal(&self) -> bool {
            false
        }
    }
    pub struct TaaFsr1;
    impl UpscaleBackend for TaaFsr1 {
        fn name(&self) -> &'static str {
            "taa+fsr1"
        }
        fn temporal(&self) -> bool {
            true
        }
    }
    pub fn backend(method: u8) -> &'static dyn UpscaleBackend {
        match method {
            1 => &TaaFsr1,
            _ => &Fsr1,
        }
    }

    #[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
    pub struct EasuMaterial {
        #[uniform(0)]
        pub p: Vec4,
        #[texture(1)]
        pub src: Handle<Image>,
    }
    impl Material2d for EasuMaterial {
        fn fragment_shader() -> ShaderRef {
            "embedded://sly2_recomp/engine/gfx/upscale/easu.wesl".into()
        }
    }
    #[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
    pub struct RcasMaterial {
        #[uniform(0)]
        pub p: Vec4,
        #[texture(1)]
        pub src: Handle<Image>,
    }
    impl Material2d for RcasMaterial {
        fn fragment_shader() -> ShaderRef {
            "embedded://sly2_recomp/engine/gfx/upscale/rcas.wesl".into()
        }
    }

    /// Upscale options in effect (`settings::LiveGraphics::upscale` when present, else this default).
    #[derive(Resource, Default)]
    pub struct UpscaleState {
        lo: Option<(Handle<Image>, UVec2)>,
        hi: Option<(Handle<Image>, UVec2)>,
        easu: Option<(Entity, Handle<EasuMaterial>)>,
        rcas: Option<(Entity, Handle<RcasMaterial>)>,
        /// the CM camera's TAA was added here (temporal backend), not by gfx::rt
        pub taa_ours: bool,
        pub logged: Option<(UVec2, UVec2, u8, bool)>,
    }

    #[derive(Component)]
    pub struct UpscaleCamera;

    fn target_image(images: &mut Assets<Image>, s: UVec2) -> Handle<Image> {
        images.add(Image::new_target_texture(s.x, s.y, TextureFormat::Rgba8UnormSrgb, None))
    }

    /// SLY2_UPSCALE=quality|balanced|performance|native (+ SLY2_SHARPNESS, SLY2_UPSCALER) override for benches.
    pub fn env_opts(mut o: UpscaleOpts) -> UpscaleOpts {
        for (k, e) in [("render_scale", "SLY2_UPSCALE"), ("gi_scale", "SLY2_GI_SCALE"), ("sharpness", "SLY2_SHARPNESS"), ("upscaler", "SLY2_UPSCALER")] {
            if let Ok(v) = std::env::var(e) {
                if o.parse_key(k, &v) != Some(Ok(())) {
                    warn!("upscale: bad {e}={v}");
                }
            }
        }
        o
    }

    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn upscale_apply(
        mut commands: Commands,
        mut st: ResMut<UpscaleState>,
        live: Option<Res<crate::settings::LiveGraphics>>,
        rt: Option<Res<super::super::rt::RtState>>,
        win: Query<&Window, With<PrimaryWindow>>,
        cams: Query<(Entity, Option<&RenderTarget>, Has<TemporalAntiAliasing>), With<CmCamera>>,
        mut images: ResMut<Assets<Image>>,
        mut easu_m: ResMut<Assets<EasuMaterial>>,
        mut rcas_m: ResMut<Assets<RcasMaterial>>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        let Ok(w) = win.single() else { return };
        let o = live.map(|l| l.upscale).unwrap_or_default();
        let raytraced = rt.is_some_and(|r| r.active);
        let out = UVec2::new(w.physical_width().max(1), w.physical_height().max(1));
        let (rw, rh) = o.render_size(out.x, out.y, raytraced);
        let lo_size = UVec2::new(rw, rh);
        let be = backend(o.method);
        let native = lo_size == out || !be.spatial();
        // temporal backend: TAA on the CM camera (gfx::rt adds its own while ray tracing)
        let want_taa = be.temporal() && !native;
        for (e, _, has_taa) in &cams {
            if want_taa && !has_taa {
                commands.entity(e).insert((TemporalAntiAliasing::default(), Msaa::Off));
                st.taa_ours = true;
            } else if !want_taa && st.taa_ours && !raytraced {
                commands.entity(e).remove::<(TemporalAntiAliasing, bevy::render::camera::TemporalJitter, bevy::render::camera::MipBias)>().insert(Msaa::default());
                st.taa_ours = false;
            }
        }
        if st.logged != Some((lo_size, out, o.method, raytraced)) {
            info!("upscale: {} render {}x{} -> {}x{} (sharpness {:.1}, rt {raytraced})", be.name(), lo_size.x, lo_size.y, out.x, out.y, o.sharpness());
            st.logged = Some((lo_size, out, o.method, raytraced));
        }
        if native {
            for (e, t, _) in &cams {
                if !matches!(t, None | Some(RenderTarget::Window(_))) {
                    commands.entity(e).insert(RenderTarget::Window(default()));
                }
            }
            for x in [st.easu.take().map(|v| v.0), st.rcas.take().map(|v| v.0)].into_iter().flatten() {
                commands.entity(x).despawn();
            }
            st.lo = None;
            st.hi = None;
            return;
        }
        if st.lo.as_ref().map(|l| l.1) != Some(lo_size) {
            st.lo = Some((target_image(&mut images, lo_size), lo_size));
        }
        if st.hi.as_ref().map(|l| l.1) != Some(out) {
            st.hi = Some((target_image(&mut images, out), out));
        }
        let lo = st.lo.clone().unwrap().0;
        let hi = st.hi.clone().unwrap().0;
        for (e, t, _) in &cams {
            if !matches!(t, Some(RenderTarget::Image(i)) if i.handle == lo) {
                commands.entity(e).insert(RenderTarget::Image(ImageRenderTarget { handle: lo.clone(), scale_factor: 1.0 }));
            }
        }
        let ep = Vec4::new(lo_size.x as f32, lo_size.y as f32, out.x as f32, out.y as f32);
        let rp = Vec4::new(out.x as f32, out.y as f32, o.sharpness(), 0.0);
        let cam2d = |order: isize| (Camera2d, Camera { order, ..default() }, Tonemapping::None, DebandDither::Disabled, UpscaleCamera);
        let hi_t = RenderTarget::Image(ImageRenderTarget { handle: hi.clone(), scale_factor: 1.0 });
        match &st.easu {
            Some((c, m)) => {
                if let Some(mut m) = easu_m.get_mut(m) {
                    if m.p != ep || m.src != lo {
                        m.p = ep;
                        m.src = lo.clone();
                    }
                }
                commands.entity(*c).insert(hi_t);
            }
            None => {
                let m = easu_m.add(EasuMaterial { p: ep, src: lo.clone() });
                let layer = RenderLayers::layer(LAYER_EASU);
                let c = commands.spawn((cam2d(1), hi_t, layer.clone())).id();
                commands.spawn((Mesh2d(meshes.add(Rectangle::new(1.0e6, 1.0e6))), MeshMaterial2d(m.clone()), Transform::default(), layer, UpscaleQuad(c)));
                st.easu = Some((c, m));
            }
        }
        match &st.rcas {
            Some((_, m)) => {
                if let Some(mut m) = rcas_m.get_mut(m) {
                    if m.p != rp || m.src != hi {
                        m.p = rp;
                        m.src = hi.clone();
                    }
                }
            }
            None => {
                let m = rcas_m.add(RcasMaterial { p: rp, src: hi.clone() });
                let layer = RenderLayers::layer(LAYER_RCAS);
                let c = commands.spawn((cam2d(2), layer.clone())).id();
                commands.spawn((Mesh2d(meshes.add(Rectangle::new(1.0e6, 1.0e6))), MeshMaterial2d(m.clone()), Transform::default(), layer, UpscaleQuad(c)));
                st.rcas = Some((c, m));
            }
        }
    }

    /// Fullscreen quad of an upscale camera (despawned with it).
    #[derive(Component)]
    pub struct UpscaleQuad(pub Entity);

    fn upscale_gc(mut commands: Commands, quads: Query<(Entity, &UpscaleQuad)>, cams: Query<(), With<UpscaleCamera>>) {
        for (e, q) in &quads {
            if cams.get(q.0).is_err() {
                commands.entity(e).despawn();
            }
        }
    }

    /// Window logical px -> `cam` viewport px (the CM camera renders into the smaller `lo` image below native).
    pub fn window_to_viewport(cam: &Camera, win: Option<&Window>, p: Vec2) -> Vec2 {
        // LOOK R15: window -> render target, then minus the viewport origin (the Hor+-off pillarbox
        // viewport of camera::bevy_sync is offset by the bar width; without a viewport origin = 0).
        match (cam.logical_target_size(), cam.logical_viewport_rect(), win.map(|w| w.size())) {
            (Some(t), Some(r), Some(w)) if w.x > 0.0 && w.y > 0.0 => p * t / w - r.min,
            _ => p,
        }
    }

    pub struct UpscalePlugin;
    impl Plugin for UpscalePlugin {
        fn build(&self, app: &mut App) {
            bevy::asset::embedded_asset!(app, "easu.wesl");
            bevy::asset::embedded_asset!(app, "rcas.wesl");
            app.add_plugins((Material2dPlugin::<EasuMaterial>::default(), Material2dPlugin::<RcasMaterial>::default()))
                .init_resource::<UpscaleState>()
                .add_systems(PostUpdate, (upscale_apply, upscale_gc).chain());
        }
    }
}
