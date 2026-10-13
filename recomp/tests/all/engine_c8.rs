//! WP-C8 tests: Episode-1 visuals (screen wipe, sky follow, blob shadows, flicker). Needs the ISO at "<root>/Sly 2 - Band of Thieves (USA).iso"
//! (level list from the boot level table).
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::{LevelLoader, LoadedLevel};
use sly2_recomp::engine::world::{LoadOptions, World};
use std::path::PathBuf;
use std::sync::Arc;

fn real_root() -> PathBuf {
    let r = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    if r.join("Sly 2 - Band of Thieves (USA).iso").exists() { r } else { PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/..")) }
}

fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

fn load(loader: &LevelLoader, d: Descriptor) -> (LoadedLevel, World) {
    let lv = crate::common::load_blocking(&loader, d).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap_or_else(|e| panic!("lsn {}: {e}", d.lsn()));
    (lv, w)
}

#[test]
#[ignore]
fn c8_probe_classes() {
    use sly2_recomp::engine::world::objects::{LoadData, Payload};
    let (loader, descs) = levels();
    for i in [0usize, 2, 3, 4, 5, 6, 7] {
        let (_lv, w) = load(&loader, descs[i]);
        for (k, o) in w.objects.iter().enumerate() {
            let want = matches!(o.class, 97 | 75) || o.oid == 0x30a;
            if !want { continue; }
            let props = match &o.payload { Payload::Loaded(LoadData::Alo { alo, .. }) => format!("{:?}", alo.props.iter().map(|p| (p.field, format!("{:?}", p.value))).collect::<Vec<_>>()), _ => "-".into() };
            println!("t{i} #{k} class {} oid {:#x} owner {:?} props {}", o.class, o.oid, o.owner, &props[..props.len().min(600)]);
        }
    }
}

use sly2_recomp::engine::gfx::screen::{fade_alpha, wipe_alpha, C2d9500_v15, Wipe, WipeHost, MSG_CLOSE, MSG_OPEN, WIPE_SECS};

#[derive(Default)]
struct Wh {
    found: Option<u32>,
    playing: bool,
    log: Vec<String>,
}

impl WipeHost<u32> for Wh {
    fn find(&mut self, oid: i32) -> Option<u32> {
        self.log.push(format!("find {oid:#x}"));
        self.found
    }
    fn send(&mut self, obj: u32, msg: u32) {
        self.log.push(format!("send {obj} {msg:#x}"));
    }
    fn stop(&mut self, obj: u32) {
        self.log.push(format!("stop {obj}"));
    }
    fn playing(&mut self, _obj: u32) -> bool {
        self.playing
    }
    fn post(&mut self, req: u32) {
        self.log.push(format!("post {req}"));
    }
}

/// FUN_0022b220 without a wipe object posts at once (FUN_0020df80) and stays idle.
#[test]
fn c8_wipe_no_object_posts_now() {
    let mut w = Wipe::<u32>::default();
    let mut h = Wh::default();
    w.FUN_0022b050(&mut h);
    w.FUN_0022b220(&mut h, 7, 0x30a, 1);
    assert_eq!(h.log, ["find 0x30a", "post 7"]);
    assert_eq!(w.state, 0);
}

/// Full cycle: close (0x311) -> poll until the animation stops -> 3 posts the request -> after the
/// load the object is re-found and opens (0x30f) -> 0 stops it.
#[test]
fn c8_wipe_cycle() {
    let mut w = Wipe::<u32>::default();
    let mut h = Wh { found: Some(5), playing: true, ..Default::default() };
    w.FUN_0022b050(&mut h);
    w.FUN_0022b220(&mut h, 9, 0x30a, 1);
    assert_eq!(w.state, 1);
    assert_eq!(h.log.last().unwrap(), &format!("send 5 {MSG_CLOSE:#x}"));
    // a second request while closing is dropped (state 1 returns)
    w.FUN_0022b220(&mut h, 10, 0x30a, 1);
    w.FUN_0022b198(&mut h);
    assert_eq!(w.state, 1);
    h.playing = false;
    w.FUN_0022b198(&mut h);
    assert_eq!(w.state, 3);
    assert_eq!(h.log.last().unwrap(), "post 9");
    assert!(w.req.is_none());
    h.found = Some(6);
    h.playing = true;
    w.FUN_0022b420(&mut h);
    assert_eq!((w.state, w.obj), (2, Some(6)));
    assert_eq!(h.log.last().unwrap(), &format!("send 6 {MSG_OPEN:#x}"));
    h.playing = false;
    w.FUN_0022b198(&mut h);
    assert_eq!(w.state, 0);
    assert_eq!(h.log.last().unwrap(), "stop 6");
}

/// FUN_0022b420 in a level without the wipe object (or not closed) goes idle.
#[test]
fn c8_wipe_after_load_without_object() {
    let mut w = Wipe::<u32>::default();
    let mut h = Wh { found: Some(5), playing: false, ..Default::default() };
    w.FUN_0022b220(&mut h, 1, 0x30a, 1);
    w.FUN_0022b198(&mut h);
    assert_eq!(w.state, 3);
    h.found = None;
    w.FUN_0022b420(&mut h);
    assert_eq!(w.state, 0);
}

#[test]
fn c8_overlay_alpha_and_sky() {
    assert_eq!(wipe_alpha(1, 0.0), 0.0);
    assert_eq!(wipe_alpha(1, WIPE_SECS), 1.0);
    assert_eq!(wipe_alpha(3, 0.0), 1.0);
    assert_eq!(wipe_alpha(2, WIPE_SECS), 0.0);
    assert_eq!(wipe_alpha(0, 0.1), 0.0);
    assert_eq!(fade_alpha(10.0, 2.0, 9.0), 0.5);
    assert_eq!(fade_alpha(10.0, 0.0, 9.0), 0.0);
    // C2d9500_v15: +0x150 == 0 keeps the sky's own height
    assert_eq!(C2d9500_v15([1.0, 2.0, 3.0], [9.0, 9.0, 9.0], false), [1.0, 2.0, 9.0]);
    assert_eq!(C2d9500_v15([1.0, 2.0, 3.0], [9.0, 9.0, 9.0], true), [1.0, 2.0, 3.0]);
}

#[test]
fn c8_translucent_sort_centre() {
    use sly2_recomp::engine::gfx::screen::sort_centre;
    assert_eq!(sort_centre(&[[0.0, 0.0, 0.0], [10.0, -4.0, 2.0], [2.0, 4.0, 8.0]]), [5.0, 0.0, 4.0]);
    assert_eq!(sort_centre(&[]), [0.0; 3]);
}

#[test]
fn c8_blob_shadow_placement() {
    use sly2_recomp::engine::gfx::screen::*;
    assert_eq!(blob_radius(53), Some(BLOB_JT));
    assert_eq!(blob_radius(60), Some(BLOB_GUARD));
    assert_eq!(blob_radius(4), None);
    assert_eq!(blob_at([0.0, 0.0, 100.0], None), None);
    let (z, a) = blob_at([0.0, 0.0, 100.0], Some(100.0)).unwrap();
    assert_eq!(z, 101.0);
    assert!((a - BLOB_ALPHA).abs() < 1e-6);
    let (_, a2) = blob_at([0.0, 0.0, 300.0], Some(100.0)).unwrap();
    assert!(a2 < a && a2 > 0.0);
    assert_eq!(blob_at([0.0, 0.0, 600.0], Some(100.0)), None);
}

#[test]
fn c8_light_member_writes() {
    use sly2_recomp::engine::game::fields::{ee_sub_1ffcf8_field, ObjectFields};
    use sly2_recomp::engine::gfx::light::{ee_sub_1a3360, set_field};
    use sly2_recomp::engine::gfx::screen::light_write_bytes;
    use sly2_recomp::engine::splice::value::Ref;
    // the LIGHT colour member (field 152, setter 0x1a35f0) is reachable from a script symid
    let sym = (0u32..0x4d1).find(|&s| ee_sub_1ffcf8_field(4, s as u16) == Some(152)).expect("class 4 colour symid");
    let mut f = ObjectFields::default();
    f.ee_sub_200c28(7, Some(4), sym, Ref::Vector([30.0, 128.0, 200.0]));
    f.ee_sub_200c28(8, Some(53), sym, Ref::F32(1.0));
    assert_eq!(f.light_writes.len(), 1);
    let (obj, field, v) = f.light_writes[0].clone();
    assert_eq!((obj, field), (7, 152));
    let mut p = ee_sub_1a3360();
    assert!(set_field(&mut p, field as i16, &light_write_bytes(&v).unwrap()));
    assert_eq!(p.hsv, [30.0, 128.0, 200.0]);
}
