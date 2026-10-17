


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

/// The 43 level descriptors (db disc_files + level_table, as in engine_a).
fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

fn load(loader: &LevelLoader, d: Descriptor) -> (LoadedLevel, World) {
    let lv = crate::common::load_blocking(&loader, d).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap_or_else(|e| panic!("lsn {}: {e}", d.lsn()));
    (lv, w)
}



use sly2_recomp::engine::anim::actor::{set_channel, ActorPose};
use sly2_recomp::engine::anim::events::{ee_sub_131290, EffectCall};
use sly2_recomp::engine::anim::aseg::{Aseg, Event, EventData};
use sly2_recomp::engine::anim::play::{drive, AnimClock, Asega, AsegaPlayer, SEGA_LOOP};
use sly2_recomp::engine::anim::pose::{ee_sub_11c168, from_4x3, Mat4, IDENTITY};
use sly2_recomp::engine::anim::skin_mesh::skin_attributes;
use sly2_recomp::engine::gfx::mesh::convert;
use sly2_recomp::engine::world::alo::Alo;
use sly2_recomp::engine::world::objects::{LoadData, Payload};

fn bones_of(alo: &Alo) -> Vec<(i32, Mat4, Option<usize>)> {
    let b = &alo.base.bones;
    b.iter()
        .enumerate()
        .map(|(i, x)| {
            let p = if x.r#ref & 0x3000 == 0 && (x.r#ref as usize) < i { Some(x.r#ref as usize) } else { None };
            (x.kind as i32, x.xform.map(|m| from_4x3(&m)).unwrap_or(IDENTITY), p)
        })
        .collect()
}

#[test]
fn actor_pose_after_n_frames() {
    let (loader, descs) = levels();
    let mut found = None;
    for d in descs.iter().copied() {
    let (_lv, w) = load(&loader, d);
    let asegs: Vec<Aseg> = w.objects.iter().filter_map(|o| match &o.payload { Payload::Loaded(LoadData::Aseg(a)) => Some((**a).clone()), _ => None }).collect();
    let aseg = asegs.iter().filter(|a| a.length > 0.5 && a.channels.iter().filter(|c| c.vec.is_some()).count() >= 2).next().cloned();
    if let Some(a) = aseg { found = Some((a, w)); break; }
    }
    let (aseg, w) = found.expect("aseg");
    let alo = w.objects.iter().filter_map(|o| match &o.payload { Payload::Loaded(LoadData::Alo { alo, .. }) => Some(alo), _ => None }).max_by_key(|a| a.base.bones.len()).unwrap();
    let spec = bones_of(alo);
    let nb = spec.len();
    assert!(nb > 4);
    // channel -> bone binding (oid resolution vtable +0x19c unported): channel c -> bone c % nb
    let binding: Vec<Option<usize>> = (0..aseg.channels.len()).map(|c| Some(c % nb)).collect();
    let mut player = AsegaPlayer::new(std::sync::Arc::new(aseg.clone()), SEGA_LOOP, 1.0);
    let mut pose = ActorPose::new(&spec);
    let clock = AnimClock { dt: 1.0 / 60.0, real_dt: 1.0 / 60.0 };
    for _ in 0..45 {
        drive(&mut player, clock);
        pose.bind(&player.segments, &binding, player.state.t, 1.0);
        pose.compose(&IDENTITY);
    }
    let t = player.state.t;
    assert!((t - (45.0f32 / 60.0) % aseg.length).abs() < 1e-3, "t {t}");
    // independent: curve eval (not the bound segment), last binding wins, world chain
    let mut locals: Vec<Mat4> = spec.iter().map(|s| s.1).collect();
    let mut val: Vec<Option<[f32; 3]>> = vec![None; nb];
    for (c, ch) in aseg.channels.iter().enumerate() {
        if let Some(v) = ch.vec.as_ref().and_then(|v| v.eval(t, 1.0)) {
            val[c % nb] = Some(v);
        }
    }
    for i in 0..nb {
        if let Some(v) = val[i] {
            set_channel(spec[i].0, &mut locals[i], v);
        }
    }
    let mut world: Vec<Mat4> = Vec::new();
    for i in 0..nb {
        let p = spec[i].2.map(|p| world[p]).unwrap_or(IDENTITY);
        world.push(ee_sub_11c168(&p, &locals[i]));
    }
    let got = pose.world();
    let mut driven = 0;
    for i in 0..nb {
        if val[i].is_some() { driven += 1; }
        for r in 0..4 { for c in 0..4 {
            assert!((got[i][r][c] - world[i][r][c]).abs() <= 1e-3 * (1.0 + world[i][r][c].abs()), "bone {i} [{r}][{c}] {} vs {}", got[i][r][c], world[i][r][c]);
        }}
    }
    assert!(driven > 0);
    let _ = pose.world_velocity(nb - 1);
}

#[test]
#[ignore = "heavy"]
fn skin_weights_sum_to_one_all_levels() {
    let (loader, descs) = levels();
    let mut skinned = 0usize;
    for d in descs {
        let (_lv, w) = load(&loader, d);
        for o in &w.objects {
            if let Payload::Loaded(LoadData::Alo { alo, .. }) = &o.payload {
                for r in &alo.mesh.records {
                    for p in convert(&r.data) {
                        if p.weights.is_empty() { continue; }
                        let s = skin_attributes(&p).expect("skin attrs");
                        assert_eq!(s.weights.len(), p.positions.len());
                        for wv in &s.weights {
                            let sum: f32 = wv.iter().sum();
                            assert!((sum - 1.0).abs() < 1e-5, "lsn {} sum {sum}", d.lsn());
                        }
                        skinned += 1;
                    }
                }
            }
        }
    }
    println!("skinned prims {skinned}");
    assert!(skinned > 0);
}

#[test]
fn event_dispatch() {
    let mut a = Asega::default();
    let e = |k, data| Event { time: Some(0.0), kind: k, data };
    assert_eq!(ee_sub_131290(&mut a, &e(7, EventData::S32F(3, 1.0)), false), Some(EffectCall::E7 { on: true, a: 3, f: 1.0 }));
    assert_eq!(ee_sub_131290(&mut a, &e(8, EventData::S32F(3, 1.0)), true), Some(EffectCall::E7 { on: true, a: 3, f: 1.0 }));
    assert_eq!(ee_sub_131290(&mut a, &e(0x17, EventData::None), false), Some(EffectCall::E16(-1)));
    assert!(a.alive);
    assert_eq!(ee_sub_131290(&mut a, &e(6, EventData::None), false), None);
    assert!(!a.alive);
}
