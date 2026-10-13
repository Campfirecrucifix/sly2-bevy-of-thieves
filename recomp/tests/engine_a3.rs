//! WP-A3: Sly's JT state machine (class 111 SM, FUN_001efac0 / FUN_001f01b8 / FUN_001f0d88) and
//! bone-world sanity of the posed rig on the real level.

use sly2_recomp::engine::anim::pose::Mat4;
use sly2_recomp::engine::anim::sm::{ee_sub_1f0400, Sm, Sma};
use sly2_recomp::engine::disc::descriptor::{self, Descriptor};
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::physics::driven::GameRng;
use sly2_recomp::engine::playerdraw::*;
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
fn level() -> Option<World> {
    let (l, t) = loader()?;
    let boot = l.load_blocking(t[25]).ok()?;
    let lv = l.load_blocking(boot.header.level_table.entries[3].desc).ok()?;
    World::load(&lv, LoadOptions::default()).ok()
}

/// Synthetic SM: A <-> B (bit 0 = backwards too), B -> C, C self loop; one mask group.
#[test]
fn sm_tables_and_walk() {
    let pairs = [(10, 30), (20, -1), (30, 0x20)];
    let links = [[10, 20, 8 | 1, 0], [20, 30, 8, 0], [30, 30, 8, 0], [30, 10, 0x10, 0], [20, 10, 8, 0]];
    let link_f = [(1.0, 100), (1.0, 101), (1.0, 102), (1.0, 103), (1.0, 999)];
    let nested = [(5, 100), (6, 101), (7, 102), (8, 103)];
    let sm = Sm::ee_sub_1efac0(&pairs, &links, &link_f, &nested);
    // oid 999 is not nested -> dropped; next labels -> indices, 0x20 -> -2
    assert_eq!(sm.trans.len(), 4);
    assert_eq!(sm.states, vec![(10, 2), (20, -1), (30, -2)]);
    // groups: bit 8 -> {8}, bit 0x10 -> {0x10}
    assert_eq!(sm.groups.iter().map(|g| g.0).collect::<Vec<_>>(), vec![8, 0x10]);
    assert_eq!(sm.ee_sub_1f0498(8, 0, 2), 2);
    assert_eq!(sm.ee_sub_1f0498(8, 1, 0), 1); // backwards over the bit-0 transition
    assert_eq!(sm.ee_sub_1f0498(8, 2, 0), 0xff);
    assert_eq!(sm.ee_sub_1f0498(0x10, 2, 0), 1);
    assert_eq!(sm.ee_sub_1f0498(0x40, 2, 0), 0);
    let len = |o: usize| o as f32;
    let mut rng = GameRng::default();
    // label 10: goal = next (C), first hop A -> B plays ASEG 5 forwards
    let mut s = Sma::ee_sub_1f01b8(&sm, &mut rng, 10, 1, 8, &len);
    let p = s.asega.clone().unwrap();
    assert_eq!((p.aseg, p.start, p.svt, s.cur, s.next, s.goal), (5, 0.0, 1.0, 0, 1, 2));
    s.ee_sub_1f1078(&sm, &mut rng, &len);
    assert_eq!((s.asega.as_ref().unwrap().aseg, s.cur, s.next), (6, 1, 2));
    s.ee_sub_1f1078(&sm, &mut rng, &len);
    // at the goal C: next -2 -> any transition out of C under mask 8 = the self loop
    assert_eq!((s.asega.as_ref().unwrap().aseg, s.cur, s.goal), (7, 2, 2));
    // backwards: from B to A over the bit-0 transition plays ASEG 5 from its end at -svt
    let mut s = Sma::ee_sub_1f01b8(&sm, &mut rng, 20, 1, 8, &len);
    assert!(s.asega.is_none()); // B: next -1 -> stays
    s.goal = 0;
    s.ee_sub_1f0d88(&sm, &mut rng, false, &len);
    let p = s.asega.clone().unwrap();
    assert_eq!((p.aseg, p.start, p.svt, s.next), (5, 5.0, -1.0, 0));
    // finish: jumped to the end at once (flags & ~1 | 2), state entered without a notification
    let mut s = Sma::ee_sub_1f01b8(&sm, &mut rng, 20, 1, 8, &len);
    s.goal = 0;
    s.ee_sub_1f0d88(&sm, &mut rng, true, &len);
    assert!(s.changes.contains(&(1, 0)), "{:?}", s.changes);
    // ... then on from A (its next = C) through A -> B
    assert_eq!((s.cur, s.next, s.asega.as_ref().unwrap().start), (0, 1, 0.0));
}

/// The real JT +0x1028 SM: idle label 0x175 -> stand ASEGs -> state 0x159c, then a weighted
/// random pick among its mask-8 transitions (not always the first link).
#[test]
fn jt_idle_sm() {
    let Some(w) = level() else { return eprintln!("ISO not found; skipped") };
    let jt = find_jt(&w).unwrap();
    let mut an = JtAnimator::new(JtRig::build(&w, jt));
    an.request_sm(&w, SM_LABEL_STAND);
    let (smo, sma) = an.sm.clone().unwrap();
    let sm = an.sms[&smo].clone();
    assert_eq!(sm.states.len(), 12);
    let s175 = ee_sub_1f0400(&sm.states, 0x175);
    let s159c = ee_sub_1f0400(&sm.states, 0x159c);
    assert_eq!(sm.states[s175 as usize].1, s159c);
    assert_eq!(sm.states[s159c as usize].1, -2);
    let oid = |o: usize| w.objects[o].oid;
    let first = oid(sma.asega.as_ref().unwrap().aseg);
    assert!([4576, 4578, 4580].contains(&first), "{first}");
    assert_eq!((sma.cur, sma.next, sma.goal), (s175, s159c, s159c));
    // walk many transitions: every played ASEG must be a link of the current state under mask 8
    let mut seen = std::collections::BTreeSet::new();
    let len = |o: usize| aseg_of(&w, o).map_or(0.0, |a| a.length);
    let mut rng = GameRng::default();
    let mut s = Sma::ee_sub_1f01b8(&sm, &mut rng, 0x175, 1, 8, &len);
    for _ in 0..200 {
        let Some(p) = s.asega.clone() else { break };
        let t = &sm.trans[s.trans.unwrap()];
        assert!(t.mask & 8 != 0 && t.aseg == Some(p.aseg));
        seen.insert(oid(p.aseg));
        s.ee_sub_1f1078(&sm, &mut rng, &len);
    }
    eprintln!("idle SM ASEGs played: {seen:?}");
    // crouch fidget, stand-up and the upright loops are reached
    for o in [5539, 5542] {
        assert!(seen.contains(&o), "{o} never played: {seen:?}");
    }
    assert!(seen.len() >= 8, "{seen:?}");
    // state 0x159c picks among its three weight-20 crouch fidgets at random
    let mut picks = std::collections::BTreeSet::new();
    for _ in 0..40 {
        let s = Sma::ee_sub_1f01b8(&sm, &mut rng, 0x159c, 1, 8, &len);
        picks.insert(oid(s.asega.unwrap().aseg));
    }
    assert_eq!(picks.into_iter().collect::<Vec<_>>(), vec![5539, 5540, 5541]);
}

fn tr(m: &Mat4) -> [f32; 3] {
    [m[3][0], m[3][1], m[3][2]]
}
fn det3(m: &Mat4) -> f32 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0]) + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

/// Bone-world sanity over idle (SM), run and jump: bones not driven by a translation channel keep
/// their rest distance to the parent (up to the rig's scale channels), and no bone basis is
/// mirrored vs the rest pose.
#[test]
fn bone_world_sanity() {
    let Some(w) = level() else { return eprintln!("ISO not found; skipped") };
    let jt = find_jt(&w).unwrap();
    let rig = JtRig::build(&w, jt);
    let rest = rig.rest_world.clone();
    let mut an = JtAnimator::new(rig);
    let n = rest.len();
    let kinds: Vec<i32> = an.rig.pose.bones.iter().map(|b| b.kind).collect();
    let parents: Vec<Option<usize>> = an.rig.pose.bones.iter().map(|b| b.parent).collect();
    let d = |a: [f32; 3], b: [f32; 3]| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
    let check = |an: &mut JtAnimator, frames: usize, what: &str| {
        let mut worst = (0.0f32, 0);
        for _ in 0..frames {
            an.sm_tick(&w);
            an.step(1.0 / 60.0);
            let wm = an.rig.pose.world();
            let mut squashed = vec![false; n];
            for i in 0..n {
                let dt = det3(&wm[i]);
                // a scale channel's curve may undershoot through zero (eyelid blink scale z): such a
                // bone and its children mirror with it; every other basis keeps det > 0
                let own = det3(&an.rig.pose.bones[i].local) < 0.0;
                squashed[i] = own || parents[i].is_some_and(|p| squashed[p]);
                if own {
                    assert_eq!(kinds[i], 0xc, "{what}: bone {i} local mirrored");
                }
                assert!(squashed[i] || (dt > 0.0 && dt < 4.0), "{what}: bone {i} basis det {dt}");
                let Some(p) = parents[i] else { continue };
                if matches!(kinds[i], 1 | 4 | 5 | 9 | 10) {
                    continue;
                }
                let (d0, d1) = (d(tr(&rest[i]), tr(&rest[p])), d(tr(&wm[i]), tr(&wm[p])));
                let e = (d1 - d0).abs() / (d0 + 1.0);
                if e > worst.0 {
                    worst = (e, i);
                }
            }
        }
        eprintln!("{what}: worst relative limb change {worst:?}");
        assert!(worst.0 < 0.08, "{what}: limb length changed {worst:?}");
    };
    an.request_sm(&w, SM_LABEL_STAND);
    check(&mut an, 1200, "idle");
    an.request(&w, FIELD_RUN, -1);
    check(&mut an, 300, "run");
    an.request(&w, FIELD_JUMP, 0);
    check(&mut an, 120, "jump");
}

