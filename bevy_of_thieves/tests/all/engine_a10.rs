//! WP-A10 (render): table-14 magenta water, two-layer shadow model, FX precache, ultrawide sweep.
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::gfx::scene::LevelScene;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

/// LDR blend source alpha (unorm target clamps the fragment output) vs the A8 split used by the Hdr camera.
fn table(t: usize) -> (sly2_recomp::engine::disc::LoadedLevel, Arc<World>) {
    let (loader, descs) = levels();
    let l0 = crate::common::load_blocking(&loader, descs[0]).unwrap();
    let lv = crate::common::load_blocking(&loader, l0.header.level_table.entries[t].desc).unwrap();
    let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
    (lv, w)
}

fn save_png(path: &str, w: u32, h: u32, rgba: &[u8]) {
    let f = std::fs::File::create(path).unwrap();
    let mut e = png::Encoder::new(std::io::BufWriter::new(f), w, h);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().unwrap().write_image_data(rgba).unwrap();
}

/// Probe: dump the CLUT sets of SLY_A10_PROBE="table,entry,shader".
#[test]
#[ignore]
fn a10_probe_dump_shader() {
    let spec = std::env::var("SLY_A10_PROBE").unwrap_or("14,283,643".into());
    let v: Vec<usize> = spec.split(',').map(|x| x.parse().unwrap()).collect();
    let (lv, w) = table(v[0]);
    let scene = LevelScene::build(&lv, &w);
    let tex = &scene.textures[v[1]];
    let sh = &tex.shaders[v[2]];
    println!("shader {:?}", sh);
    for so in &scene.objects {
        for p in &so.prims {
            if p.prim.shader == Some(v[2] as u32) && so.entry == v[1] {
                let n = p.prim.colors.len().max(1) as f32;
                let mut s = [0f32; 4];
                let (mut lo, mut hi) = ([f32::MAX; 4], [f32::MIN; 4]);
                for c in &p.prim.colors { for k in 0..4 { s[k] += c[k]; lo[k] = lo[k].min(c[k]); hi[k] = hi[k].max(c[k]); } }
                println!("obj {} oid {} class {} rec {} verts {} vcol avg {:?} min {:?} max {:?} cull {}", so.object, so.oid, w.objects[so.object].class, p.record, p.prim.positions.len(), s.map(|x| x / n), lo, hi, p.prim.cull_back);
            }
        }
    }
    for set in 0..3 {
        if let Some(Ok(t)) = tex.decode_shader_pass(v[2], 0, set) {
            let n = (t.rgba.len() / 4) as u64;
            let mut s = [0u64; 4];
            for c in t.rgba.chunks_exact(4) { for k in 0..4 { s[k] += c[k] as u64; } }
            println!("set {set}: {}x{} avg {:?}", t.width, t.height, s.map(|x| x / n));
            save_png(&format!("{}/a10_probe_{}_{}_{}_set{set}.png", std::env::temp_dir().display(), v[0], v[1], v[2]), t.width, t.height, &t.rgba);
        }
    }
}

use sly2_recomp::engine::gfx::material::{two_layer_grey_weight, two_layer_means, two_layer_vertex_color, two_layer_vu, VF11_ROW10_FILL};

/// A10 (2): VU1 kinds-1/7 terms: no vf11 fill, colour factor linear in the light (k = Lw) in range.
#[test]
fn a10_two_layer_vu_linear() {
    assert_eq!(VF11_ROW10_FILL, [0.0; 3]);
    for &x in &[0.0f32, 0.1, 0.25, 0.5, 0.75, 1.0] {
        let (g, k) = two_layer_vu([x; 3], x);
        assert_eq!(g, [x; 3]);
        assert!((k - x).abs() < 1e-6, "k {k} for L {x}");
    }
    // overbright: grey saturates at 2, colour fades with h = 2 - max(L)
    let (g, k) = two_layer_vu([2.5, 1.0, 1.0], 2.5);
    assert_eq!(g, [2.0, 1.0, 1.0]);
    assert_eq!(k, 0.0);
}

/// A10 (2): grey layer gets only the vertex colour's brightness (n.w), the colour layer gets the hue.
#[test]
fn a10_two_layer_vertex_color_hue_on_colour_only() {
    // n.w = (char)(max * /255 * 127) / 128
    assert!((two_layer_grey_weight([1.0, 0.2, 0.5, 1.0]) - 63.0 / 128.0).abs() < 1e-6);
    // grey only: hue removed (all channels the same weight)
    let m = two_layer_means(&[100, 100, 100, 255], &[0, 0, 0, 0]);
    let c = two_layer_vertex_color([1.7, 0.23, 1.23, 1.0], &m);
    assert!((c[0] - c[1]).abs() < 1e-6 && (c[1] - c[2]).abs() < 1e-6);
    assert_eq!(c[3], 1.0);
    // colour only: the vertex colour unchanged
    let m = two_layer_means(&[0, 0, 0, 255], &[50, 60, 70, 255]);
    let v = [1.7, 0.23, 1.23, 0.5];
    assert_eq!(two_layer_vertex_color(v, &m), v);
    // table-14 water (grey 36 + colour (33, 18, 53)) is less saturated than the plain vertex colour
    let m = two_layer_means(&[36, 36, 36, 255], &[33, 18, 53, 255]);
    let c = two_layer_vertex_color(v, &m);
    assert!(c[0] < v[0] && c[1] > v[1], "{c:?}");
}

/// A10 (3): effect billboards draw from pre-created materials: the pool exists at startup and a
/// spawned hit star takes a pool material (no new asset) and returns it when it expires.
#[cfg(feature = "game")]
#[test]
fn a10_fx_material_pool() {
    use bevy::prelude::*;
    use sly2_recomp::engine::gfx::fx::{FxPlugin, FxQueue, FxRequest, FX_MAT_POOL};
    use sly2_recomp::engine::gfx::game::LevelGeometryRoot;
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), TransformPlugin))
        .init_asset::<Mesh>()
        .init_asset::<Image>()
        .init_asset::<StandardMaterial>()
        .add_plugins(FxPlugin);
    app.world_mut().spawn((LevelGeometryRoot(0), Transform::default()));
    app.world_mut().spawn((sly2_recomp::engine::camera::bevy_sync::CmCamera, Transform::default()));
    app.update();
    let n0 = app.world().resource::<Assets<StandardMaterial>>().len();
    assert!(n0 >= FX_MAT_POOL, "pool {n0}");
    app.world_mut().resource_mut::<FxQueue>().push(FxRequest::hit([0.0, 0.0, 0.0]));
    app.update();
    assert_eq!(app.world().resource::<Assets<StandardMaterial>>().len(), n0, "hit star created a new material");
    assert_eq!(app.world().resource::<FxQueue>().spawned, 1);
}

/// A10 (4): every skinned level prim gets per-joint culling bounds (DynamicSkinnedMeshBounds needs
/// them; without, Bevy culls the posed model by the raw rest positions in the wrong frame).
#[cfg(feature = "game")]
#[test]
fn a10_skinned_bounds_generated() {
    for t in [3usize, 20] {
        let (lv, w) = table(t);
        let scene = LevelScene::build(&lv, &w);
        let mut n = 0;
        for so in &scene.objects {
            for p in &so.prims {
                if let Some(mut m) = sly2_recomp::engine::anim::skin_mesh::bevy_skinned_mesh(&p.prim) {
                    m.generate_skinned_mesh_bounds().unwrap_or_else(|e| panic!("t{t} obj {} rec {}: {e:?}", so.object, p.record));
                    assert!(m.skinned_mesh_bounds().is_some_and(|b| b.iter().count() > 0));
                    n += 1;
                }
            }
        }
        assert!(n > 10, "t{t}: {n} skinned prims");
    }
}
