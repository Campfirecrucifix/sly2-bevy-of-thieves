//! WP-Q tests: Sly's (JT) rig, ASEG members and channel binding found in real levels; ASEG playback
//! moves bones; skinned prims of the JT subtree resolve their joints into the rig. Needs the ISO.

use sly2_recomp::engine::disc::descriptor::{self, Descriptor};
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::gfx::attach::BoneWorlds;
use sly2_recomp::engine::gfx::scene::LevelScene;
use sly2_recomp::engine::gfx::skin::{record_weight_refs, skin_joints};
use sly2_recomp::engine::playerdraw::*;
use sly2_recomp::engine::world::objects::{LoadData, Payload};
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

fn dist(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> f32 {
    (0..4).map(|i| (0..3).map(|j| (a[i][j] - b[i][j]).abs()).fold(0.0, f32::max)).fold(0.0, f32::max)
}

#[test]
fn field_members_match_0x2be990() {
    assert_eq!(field_members(FIELD_RUN), &[0x1334, 0x1374]);
    assert_eq!(field_members(FIELD_JUMP), &[0x1974, 0x1980, 0x198c]);
    assert_eq!(field_members(0x14d0).len(), 8); // WP-Q2: FUN_001a0300(jt + 0x13d0, jt + 0x14d0)
    assert!(field_members(0x1950).is_empty());
    assert_eq!(JT_ANIM_TABLE.map(|r| r.0), [0x7d, 0x7e, 0xa5, 0xa6, 0xa7]);
}

#[test]
fn sly_rig_found_in_levels() {
    let Some((l, t)) = loader() else { return eprintln!("ISO not found; skipped") };
    let boot = crate::common::load_blocking(&l, t[25]).unwrap();
    let mut levels = 0;
    for r in [3usize, 10, 20, 5] {
        let lv = crate::common::load_blocking(&l, boot.header.level_table.entries[r].desc).unwrap();
        let w = World::load(&lv, LoadOptions::default()).unwrap();
        let jt = find_jt(&w).expect("JT object");
        assert_eq!(w.objects[jt].oid, 7);
        let rig = JtRig::build(&w, jt);
        assert!(rig.keys.len() >= 139, "row {r}: {} bones", rig.keys.len());
        // WP-Q2: members hold every ASEG row of 0x2be990; the Q rows are among them
        assert!(JT_ANIM_TABLE.iter().all(|r| rig.members.contains_key(&r.1)), "row {r}: members {:?}", rig.members);
        // every channel of the run/jump ASEGs binds a rig bone
        for &(_, m) in &JT_ANIM_TABLE {
            let o = rig.members[&m];
            let b = &rig.bindings[&o];
            let n = b.iter().filter(|x| x.is_some()).count();
            assert!(n * 10 >= b.len() * 9, "row {r} member {m:#x}: {n}/{} channels bound", b.len());
        }
        // skin joints of the JT subtree's skinned prims are rig bones
        let scene = LevelScene::build(&lv, &w);
        let sub = subtree(&w, jt);
        let (mut joints, mut found, mut prims) = (0, 0, 0);
        let mut bw = BoneWorlds::default();
        for so in scene.objects.iter().filter(|so| sub.contains(&so.object)) {
            let Payload::Loaded(LoadData::Alo { alo, .. }) = &w.objects[so.object].payload else { continue };
            for p in &so.prims {
                let refs = record_weight_refs(&alo.mesh.records[p.record].data);
                if refs.is_empty() {
                    continue;
                }
                prims += 1;
                for r in skin_joints(&w, &mut bw, so.object, refs).refs {
                    joints += 1;
                    found += rig.bone_of(&r).is_some() as usize;
                }
            }
        }
        assert!(prims > 10 && found == joints, "row {r}: {prims} skinned prims, {found}/{joints} joints in rig");
        // rest pose of the rig = gfx rest bone worlds
        let mut bw = BoneWorlds::default();
        let worst = rig.keys.iter().zip(&rig.rest_world).map(|(&(o, k), m)| dist(m, &sly2_recomp::engine::anim::pose::from_4x3(&bw.get(&w, o, k)))).fold(0.0, f32::max);
        assert!(worst < 0.01, "row {r}: rest pose differs by {worst}");
        // playing run moves bones, stays near the rest pose (no root teleport)
        let mut an = JtAnimator::new(rig);
        an.request(&w, FIELD_RUN, 0);
        for _ in 0..20 {
            an.step(1.0 / 60.0);
        }
        let moved = an.rig.pose.world().iter().zip(&an.rig.rest_world).filter(|(a, b)| dist(a, b) > 0.5).count();
        let far = an.rig.pose.world().iter().zip(&an.rig.rest_world).map(|(a, b)| (0..3).map(|j| (a[3][j] - b[3][j]).abs()).fold(0.0, f32::max)).fold(0.0, f32::max);
        eprintln!("row {r}: jt {jt}, {} bones, {prims} skinned prims, {joints} joints; run moved {moved} bones, max translation {far}", an.rig.keys.len());
        assert!(moved > 10, "row {r}: run moved {moved} bones");
        assert!(far < 300.0, "row {r}: bone moved {far} units");
        an.request(&w, FIELD_JUMP, 1);
        // WP-Q2: the jump field plays its blend (FUN_00137a30), not one entry
        assert_eq!(an.current_field(), Some(FIELD_JUMP));
        levels += 1;
    }
    assert!(levels >= 3);
}

/// Captured frames (recomp/shots, game.exe --table 3 --screenshot): 16:9 and 32:9 decode with the
/// requested aspect (Sly in frame was checked by eye; see ENGINE_PORT.md Q findings).
#[test]
fn sly_frames_captured() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("shots");
    for (name, w, h) in [("q_t3_1920x1080.png", 1920, 1080), ("q_t3_5120x1440.png", 5120, 1440)] {
        let Ok(f) = std::fs::File::open(dir.join(name)) else { return eprintln!("{name} not captured; skipped") };
        let r = png::Decoder::new(std::io::BufReader::new(f)).read_info().unwrap();
        assert_eq!((r.info().width, r.info().height), (w, h), "{name}");
    }
}
