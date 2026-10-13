//! WP-J5 tests: method coverage count, get-o/set-o round trip on real objects (fields + user
//! properties), BIF 107 + deferred list, ASEG events of a real level firing their sound effect.
//! Needs the ISO (skipped otherwise).

use sly2_recomp::engine::anim::aseg::EventData;
use sly2_recomp::engine::audio::emitter::EmitterSystem;
use sly2_recomp::engine::camera::Cm;
use sly2_recomp::engine::disc::descriptor::{self, Descriptor};
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::game::anim_play::{LevelAnims, CLASS_ASEG};
use sly2_recomp::engine::game::fieldbind::{FIELD_BINDINGS, FIELD_BINDING_COUNT};
use sly2_recomp::engine::game::fields::{object_props, DeferredCall};
use sly2_recomp::engine::game::methods::{coverage, IMPLEMENTED};
use sly2_recomp::engine::game::save::GameData;
use sly2_recomp::engine::game::spawn::SpawnAliases;
use sly2_recomp::engine::game::splice_host::*;
use sly2_recomp::engine::game::vm_state::*;
use sly2_recomp::engine::splice::value::Ref;
use sly2_recomp::engine::splice::SpliceHost;
use sly2_recomp::engine::world::objects::{LoadData, Payload};
use sly2_recomp::engine::world::props::PropValue;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::path::PathBuf;
use std::sync::Arc;

fn root() -> PathBuf {
    let r = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    if r.join("Sly 2 - Band of Thieves (USA).iso").exists() { r } else { PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/..")) }
}

fn loader() -> Option<(LevelLoader, Vec<Descriptor>)> {
    let iso = Arc::new(IsoImage::open(root().join("Sly 2 - Band of Thieves (USA).iso")).ok()?);
    let table = descriptor::static_table(&iso.read_file("SCUS_973.16").ok()?)?;
    Some((LevelLoader::new(iso, ModOverlay::default()), table))
}

fn dbg(r: &Ref) -> String {
    format!("{r:?}")
}

#[test]
fn method_coverage_count() {
    let c = coverage();
    // J4: 8 implemented; J5 adds 0x00101d20 (FUN_00130b18 retract). Engine functions of the remaining
    // HostUnimplemented rows are not ported anywhere (ENGINE_PORT.md J5 findings).
    assert_eq!(c.implemented, 9, "{c:?}");
    assert_eq!(IMPLEMENTED.len(), 9);
    assert_eq!(FIELD_BINDING_COUNT, 12001);
    assert!(FIELD_BINDINGS.windows(2).all(|p| (p[0].0, p[0].1) < (p[1].0, p[1].1)));
}

#[test]
fn get_o_round_trip_on_real_objects() {
    let Some((l, t)) = loader() else { return eprintln!("ISO not found; skipped") };
    let boot = crate::common::load_blocking(&l, t[25]).unwrap();
    let lv = crate::common::load_blocking(&l, boot.header.level_table.entries[3].desc).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    let mut em = EmitterSystem::default();
    let mut data = GameData::default();
    let mut al = SpawnAliases::default();
    let mut rum = Vec::new();
    let mut sched = Schedule::default();
    let base = GameSpliceHost { t: 2.0, cm: None, emitters: &mut em, vars: None, data: &mut data };
    let level = LevelSpliceHost { base, mission: -1, aliases: &mut al, rumble: &mut rum };
    let mut cm = Cm::new();
    let mut h = PersistentHost { level, cm: Some(&mut cm), clk: Default::default(), world: Some(&w), table: None, sched: &mut sched };

    // find real (object, symid, field) with a stream-written scalar of kind 1 (f32)
    let mut checked = 0;
    for (i, o) in w.objects.iter().enumerate() {
        let Some(props) = object_props(o) else { continue };
        for p in props {
            let PropValue::Scalar(b) = &p.value else { continue };
            if sly2_recomp::engine::world::props::field(p.field).map(|d| d.kind) != Some(1) {
                continue;
            }
            let Some(&(_, sym, _)) = FIELD_BINDINGS.iter().find(|x| x.0 == o.class && x.2 == p.field as u16) else { continue };
            if p.field as usize >= 1449 || props.iter().filter(|q| q.field == p.field).count() != 1 {
                continue;
            }
            let v = f32::from_le_bytes([b[0], b[1], b[2], b[3]]);
            let obj = Ref::Basic(i as u32);
            let got = h.ref_op_get_o(&[obj.clone(), Ref::Symid(sym as u32)]).unwrap();
            assert_eq!(dbg(&got), dbg(&Ref::F32(v)), "object {i} class {} field {}", o.class, p.field);
            h.ref_op_set_o(&[obj.clone(), Ref::Symid(sym as u32), Ref::F32(v + 1.0)]).unwrap();
            assert_eq!(dbg(&h.ref_op_get_o(&[obj.clone(), Ref::Symid(sym as u32)]).unwrap()), dbg(&Ref::F32(v + 1.0)));
            checked += 1;
        }
        if checked >= 20 {
            break;
        }
    }
    assert!(checked >= 20, "only {checked} f32 fields found");
    // user property (symid >= 0x4d1): Nil until added; set-o (FUN_0012aa70) only replaces an existing
    // one (WP-J9), add-o (FUN_00200ca8) creates it
    let obj = Ref::Basic(w.objects.iter().position(|o| o.class == CLASS_ASEG).unwrap() as u32);
    assert_eq!(dbg(&h.ref_op_get_o(&[obj.clone(), Ref::Symid(0x600)]).unwrap()), dbg(&Ref::Nil));
    h.ref_op_set_o(&[obj.clone(), Ref::Symid(0x600), Ref::S32(41)]).unwrap();
    assert_eq!(dbg(&h.ref_op_get_o(&[obj.clone(), Ref::Symid(0x600)]).unwrap()), dbg(&Ref::Nil));
    h.ref_op_add_o(&[obj.clone(), Ref::Symid(0x600), Ref::S32(1)]).unwrap();
    h.ref_op_set_o(&[obj.clone(), Ref::Symid(0x600), Ref::S32(42)]).unwrap();
    assert_eq!(dbg(&h.ref_op_get_o(&[obj.clone(), Ref::Symid(0x600)]).unwrap()), dbg(&Ref::S32(42)));
    // BIF 107: +0x30 = t + delay; Void
    assert_eq!(dbg(&h.ref_op_defer_object_update(&[obj.clone(), Ref::F32(0.5)]).unwrap()), dbg(&Ref::Void));
    // camera BIF inside a callback host (cm borrowed mutably) now sees the camera
    assert!(matches!(h.ref_op_near_clip_center(&[]).unwrap(), Ref::Vector(_)));
    drop(h);
    assert_eq!(sched.objs.next_update.get(&obj.payload()), Some(&2.5));
    // deferred list: add-unique, remove by mask, drain
    let c = DeferredCall { object: 7, adj: 0, vindex: -1, fnw: 0x1234, arg: 1 };
    sched.objs.ee_sub_149ab8(c);
    sched.objs.ee_sub_149ab8(c);
    sched.objs.ee_sub_149a50(DeferredCall { arg: 2, ..c });
    assert_eq!(sched.objs.deferred.len(), 2);
    sched.objs.ee_sub_149b58(5, 7, (0, 0), 2);
    assert_eq!(sched.objs.deferred, vec![c]);
    sched.objs.ee_sub_149c80_deferred(|o, c| o.dispatched.push(c));
    assert!(sched.objs.deferred.is_empty() && sched.objs.dispatched == vec![c]);
}

#[test]
fn aseg_event_fires_sound() {
    let Some((l, t)) = loader() else { return eprintln!("ISO not found; skipped") };
    let boot = crate::common::load_blocking(&l, t[25]).unwrap();
    for row in [3usize, 10, 20] {
        let lv = crate::common::load_blocking(&l, boot.header.level_table.entries[row].desc).unwrap();
        let w = World::load(&lv, LoadOptions::default()).unwrap();
        // an ASEG with a timed sound event (case 0, sound != -1)
        let found = w.objects.iter().enumerate().find_map(|(i, o)| match &o.payload {
            Payload::Loaded(LoadData::Aseg(a)) if o.class == CLASS_ASEG => a.events1.iter().find_map(|e| match &e.data {
                EventData::K0 { s, .. } if *s != -1 && e.time.is_some_and(|t| t > 0.0 && t < a.length) => Some((i, *s as i32, e.time.unwrap())),
                _ => None,
            }),
            _ => None,
        });
        let Some((obj, sound, at)) = found else { continue };
        let mut anims = LevelAnims::default();
        assert!(anims.play(&w, obj, 1.0));
        let mut em = EmitterSystem::default();
        let mut rum = Vec::new();
        let frames = (at * 60.0).ceil() as usize + 2;
        for _ in 0..frames {
            anims.step(&w, 1.0 / 60.0, 1.0 / 60.0, &mut em, &mut rum, None, Default::default(), &mut Vec::new());
            if anims.sounds.iter().any(|s| s.sound == sound) {
                break;
            }
        }
        let s = anims.sounds.iter().find(|s| s.aseg == obj && s.sound == sound).unwrap_or_else(|| panic!("row {row}: no sound in {frames} frames"));
        assert_eq!(s.sound, sound);
        assert!(em.active.contains(&s.emitter), "emitter in the active list (played by app::play_sound_events)");
        eprintln!("row {row}: ASEG {obj} event at {at:.3}s fired sound {sound} after {} frames", anims.frames);
        // method 0x00101d20 retract
        anims.step(&w, 0.0, 0.0, &mut em, &mut rum, None, Default::default(), &mut vec![obj as u32]);
        assert!(anims.players.iter().all(|p| p.object != obj));
        return;
    }
    panic!("no ASEG with a timed sound event on rows 3/10/20");
}
