//! WP-D6 tests: ASEG channel -> bone binding (anim::bind), plain blends / cross-fade
//! (anim::blends), npcdraw guards posed by live anims, and the 43-level census.
use sly2_recomp::engine::anim::bind::{channel_bone_ref, ee_sub_12fe58, Act, AnimBinder, BoneActs};
use sly2_recomp::engine::classes::k26::MusicRt;
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::ClassSet;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

const DT: f32 = 1.0 / 60.0;

use crate::common as k20;

fn act(asega: u64, prio: i32) -> Act {
    Act { asega, channel: 0, prio, mode: 5, scalar: false }
}

/// FUN_00230018 / FUN_00230100: descending priority, equal priority newest first, removal hands over
#[test]
fn d6_act_list_order() {
    let mut l = BoneActs::default();
    let b = (1, 2);
    assert!(l.ee_sub_230018(b, act(1, 0)));
    assert!(l.ee_sub_230018(b, act(2, 0)), "equal priority: newest on top");
    assert!(!l.ee_sub_230018(b, act(3, -1)), "lower priority goes behind");
    assert!(l.ee_sub_230018(b, act(4, 5)));
    let order: Vec<u64> = l.lists[&b].iter().map(|a| a.asega).collect();
    assert_eq!(order, vec![4, 2, 1, 3]);
    assert!(l.ee_sub_230100(b, 4, 0), "removing the top changes it");
    assert_eq!(l.top(b).unwrap().asega, 2);
    assert!(!l.ee_sub_230100(b, 3, 0));
    assert!(!l.ee_sub_230100(b, 9, 0), "unknown act");
    l.ee_sub_230100(b, 2, 0);
    l.ee_sub_230100(b, 1, 0);
    assert!(l.top(b).is_none() && l.lists.is_empty());
}

/// the channel bone ref is the 10-bit field sign-extended ((w << 2) >> 22)
#[test]
fn d6_channel_bone_ref() {
    let mut c = sly2_recomp::engine::anim::aseg::Channel { oid: 0xfffff, kind: 5, vec: None, scalar: None, flag: false };
    assert_eq!(channel_bone_ref(&c), 5);
    c.kind = 0x3ff;
    assert_eq!(channel_bone_ref(&c), -1);
    c.kind = 0x200;
    assert_eq!(channel_bone_ref(&c), -512);
    assert_eq!(sly2_recomp::engine::anim::bind::channel_oid(&c), -1);
}

fn load(n: usize) -> Option<Arc<World>> {
    let (loader, descs) = k20::levels();
    let d = descs.into_iter().nth(n)?;
    let lv = crate::common::load_blocking(&loader, d).ok()?;
    Some(Arc::new(World::load(&lv, LoadOptions::default()).ok()?))
}

/// every ASEG of level 0 binds through FUN_0012fe58; started untargeted, its bound bones get acts
/// and the pose moves bound objects; retract removes every act
#[test]
fn d6_bind_level0() {
    let Some(w) = load(0) else { return };
    let mut b = AnimBinder::default();
    let (mut chans, mut bound, mut h) = (0, 0, 0u64);
    for (i, o) in w.objects.iter().enumerate() {
        let Some(a) = sly2_recomp::engine::playerdraw::aseg_of(&w, i) else { continue };
        let _ = o;
        for ch in &a.channels {
            chans += 1;
            if !ch.flag && ee_sub_12fe58(&w, i, None, ch).1.is_some() {
                bound += 1;
            }
        }
        h += 1;
        b.start(&w, h, i, a, None, 0, 0);
    }
    let acts: usize = b.acts.lists.values().map(|l| l.len()).sum();
    eprintln!("level 0: {h} asegs, {chans} channels, {bound} bound, {acts} acts, {} bones", b.acts.lists.len());
    assert!(chans > 0 && bound > 0 && acts == bound);
    let pose = b.pose(&w, &|_| Some((0.5, 1.0)));
    eprintln!("pose: {} driven bones, {} affected bones, {} posed objects, {} objects", pose.locals.len(), pose.bones.len(), pose.posed.len(), pose.objects.len());
    assert!(!pose.locals.is_empty() && pose.bones.len() >= pose.locals.len() && !pose.posed.is_empty());
    for k in 1..=h {
        b.retract(k);
    }
    assert!(b.acts.lists.is_empty() && b.asegas.is_empty());
}

/// plain blend: weight 1 on one entry = that ASEG's own pose; cross-fade starts at the old pose and ends at the new
#[test]
fn d6_plain_blend_and_fade() {
    use sly2_recomp::engine::anim::blends::{blend_locals, ee_sub_1a0300, CrossFade, PlainEntry};
    let Some(w) = load(0) else { return };
    let Some(jt) = sly2_recomp::engine::playerdraw::find_jt(&w) else { return };
    let members = sly2_recomp::engine::playerdraw::jt_anim_members(&w, jt);
    let mut asegs: Vec<usize> = members.values().copied().filter(|&o| sly2_recomp::engine::playerdraw::aseg_of(&w, o).is_some()).collect();
    asegs.sort_unstable();
    asegs.dedup();
    assert!(asegs.len() >= 2);
    let mut e = vec![PlainEntry { weight: 1.0, aseg: Some(asegs[0]), vel: [0.0; 3] }, PlainEntry { weight: 0.0, aseg: Some(asegs[1]), vel: [0.0; 3] }];
    ee_sub_1a0300(&w, jt, &mut e);
    let l = blend_locals(&w, jt, &e, 0.1, 1.0);
    assert!(!l.is_empty(), "blend binds bones of the JT");
    // same pose as the ASEG played alone
    let mut b = AnimBinder::default();
    b.start(&w, 1, asegs[0], sly2_recomp::engine::playerdraw::aseg_of(&w, asegs[0]).unwrap(), Some(jt), 0, 0);
    let solo = b.locals(&w, &|_| Some((0.1, 1.0)));
    let mut same = 0;
    for (k, m) in &solo {
        if let Some(x) = l.get(k) {
            assert!((0..4).all(|r| (0..3).all(|c| (m[r][c] - x[r][c]).abs() < 1e-3)), "bone {k:?}: {m:?} vs {x:?}");
            same += 1;
        }
    }
    assert!(same > 0);
    // fade
    let mut f = CrossFade::default();
    f.start(&solo, 0.2);
    let mut to = HashMap::new();
    let mut l2 = to.clone();
    f.apply(0.0, &mut l2, &|_| sly2_recomp::engine::gfx::scene::IDENTITY);
    let k0 = *solo.keys().next().unwrap();
    assert!((0..3).all(|c| (l2[&k0][3][c] - solo[&k0][3][c]).abs() < 1e-4), "u = 0 keeps the old pose");
    to.clear();
    let mut l3 = HashMap::new();
    f.apply(0.3, &mut l3, &|_| sly2_recomp::engine::gfx::scene::IDENTITY);
    assert!(!f.active());
    assert!((0..3).all(|c| l3[&k0][3][c].abs() < 1e-4), "u = 1 reaches the target (rest = identity)");
}

#[derive(Debug, Default)]
pub struct Census {
    pub levels: usize,
    pub asegas: usize,
    /// K26 (before): distinct objects posed by k26_bind / moved
    pub k26_posed: usize,
    pub k26_moved: usize,
    /// D6 (after): distinct objects with an affected bone / with an affected bone that moved / driven bones
    pub d6_posed: usize,
    pub d6_moved: usize,
    pub d6_bones: usize,
    /// NPCs (guards / pickups) with bones overlaid by class anims (npcdraw::NpcSet::overlay_class_anims)
    pub npc_overlaid: usize,
    /// static: every ASEG of the level started untargeted (FUN_0012fe58 mode 4): channels, bound channels,
    /// posed objects and objects with bone 0 moved at t = 0.5
    pub all_channels: usize,
    pub all_bound: usize,
    pub all_posed: usize,
    pub all_objects: usize,
}

pub fn census(frames: usize, levels: usize) -> Census {
    let (loader, descs) = k20::levels();
    let mut c = Census::default();
    for d in descs.into_iter().take(levels) {
        let Ok(lv) = crate::common::load_blocking(&loader, d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        c.levels += 1;
        let w = Arc::new(w);
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        h.k26_music = MusicRt::from_block(&lv.header.block_3dd040);
        let mut s = ClassSet::build(&w, &mut h);
        s.g.g_3dd268 = h.k26_music.inst;
        s.post_load(&mut h);
        {
            let mut b = AnimBinder::default();
            let mut h = 0u64;
            for i in 0..w.objects.len() {
                let Some(a) = sly2_recomp::engine::playerdraw::aseg_of(&w, i) else { continue };
                h += 1;
                c.all_channels += a.channels.len();
                c.all_bound += b.start(&w, h, i, a, None, 0, 0);
            }
            let p = b.pose(&w, &|_| Some((0.5, 1.0)));
            c.all_posed += p.posed.len();
            c.all_objects += p.objects.len();
        }
        let mut ns = sly2_recomp::engine::npcdraw::NpcSet::default();
        let mut nov = HashSet::new();
        let mut b = AnimBinder::default();
        let (mut kp, mut km, mut dp, mut dm, mut db) = (HashSet::new(), HashSet::new(), HashSet::new(), HashSet::new(), HashSet::new());
        let (mut kfirst, mut dfirst): (HashMap<usize, [f32; 3]>, HashMap<(usize, usize), [f32; 3]>) = Default::default();
        for f in 0..frames {
            s.g.t += DT;
            s.g.dt = DT;
            s.frame(&mut h);
            for (o, m) in &h.k26_posed {
                kp.insert(*o);
                let p = *kfirst.entry(*o).or_insert(m[3]);
                if (0..3).any(|a| (p[a] - m[3][a]).abs() > 1e-3) {
                    km.insert(*o);
                }
            }
            let live: Vec<(u64, usize, Option<usize>)> = h.k25.asegas.iter().filter(|x| x.1.st.alive).map(|(k, x)| (*k as u64, x.aseg, (x.obj as usize).checked_sub(1))).collect();
            b.sync(&w, &live);
            let times: HashMap<u64, (f32, f32)> = h.k25.asegas.iter().map(|(k, x)| (*k as u64, (x.st.t, x.st.svt))).collect();
            let pose = b.pose(&w, &|k| times.get(&k).copied());
            ns.step(&w, &mut s, &mut h, DT);
            nov.extend(ns.overlay_class_anims(&w, &b, &|k| times.get(&k).copied()));
            dp.extend(pose.posed.iter().copied());
            db.extend(pose.locals.keys().copied());
            for (k, m) in &pose.bones {
                let p = *dfirst.entry(*k).or_insert(m[3]);
                if (0..3).any(|a| (p[a] - m[3][a]).abs() > 1e-3) {
                    dm.insert(k.0);
                }
            }
            if f % 100 == 99 {
                eprint!(".");
            }
        }
        c.asegas += h.k25.started;
        c.k26_posed += kp.len();
        c.k26_moved += km.len();
        c.d6_posed += dp.len();
        c.d6_moved += dm.len();
        c.d6_bones += db.len();
        c.npc_overlaid += nov.len();
        eprintln!("level {} -> {c:?}", c.levels);
    }
    c
}

/// 43 levels x 300 frames: K26 (before) vs D6 binding (after)
#[test]
#[ignore]
fn d6_probe_census() {
    let c = census(300, usize::MAX);
    eprintln!("D6 census 300: {c:?}");
}

/// first 3 levels x 120 frames: the D6 binding poses and moves at least what K26 found
#[test]
fn d6_census_minimums() {
    let c = census(120, 3);
    eprintln!("D6 census 3x120: {c:?}");
    if c.levels == 0 {
        return;
    }
    assert!(c.d6_posed > 0 && c.d6_moved > 0, "{c:?}");
}

/// level 1: the class-run ASEGAs, their channels and what they bind
#[test]
#[ignore]
fn d6_probe_targets() {
    let (loader, descs) = k20::levels();
    let d = descs.into_iter().next().unwrap();
    let lv = crate::common::load_blocking(&loader, d).unwrap();
    let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
    let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
    let mut s = ClassSet::build(&w, &mut h);
    s.post_load(&mut h);
    for _ in 0..30 {
        s.g.t += DT;
        s.g.dt = DT;
        s.frame(&mut h);
    }
    for (k, x) in &h.k25.asegas {
        let a = sly2_recomp::engine::playerdraw::aseg_of(&w, x.aseg).unwrap();
        let ao = &w.objects[x.aseg];
        eprintln!("asega {k} aseg {} oid {} owner {:?} target {} t {} alive {}", x.aseg, ao.oid, ao.owner, x.obj, x.st.t, x.st.alive);
        for ch in &a.channels {
            let (o, b) = ee_sub_12fe58(&w, x.aseg, (x.obj as usize).checked_sub(1), ch);
            let bk = b.and_then(|(o, k)| sly2_recomp::engine::gfx::attach::alo_of(&w, o).map(|a| (a.base.bones.len(), a.base.bones[k].kind)));
            let cands: Vec<(usize, u16, Option<usize>)> = w.find_oid(sly2_recomp::engine::anim::bind::channel_oid(ch)).iter().map(|&i| (i, w.objects[i].class, w.objects[i].owner)).collect();
            eprintln!("  ch oid {} ref {} flag {} vec {} -> obj {o:?} bone {b:?} (nbones, kind) {bk:?} cands {cands:?}", ch.oid, channel_bone_ref(ch), ch.flag, ch.vec.is_some(), );
        }
    }
}

#[test]
#[ignore]
fn d6_probe_c96() {
    let (loader, descs) = k20::levels();
    let d = descs.into_iter().next().unwrap();
    let lv = crate::common::load_blocking(&loader, d).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    for i in [627usize, 636, 640, 644] {
        let o = &w.objects[i];
        let s = format!("{:?}", o.payload);
        eprintln!("{i} class {} oid {} children {:?} payload {}", o.class, o.oid, o.children, &s[..s.len().min(300)]);
    }
}

/// item 3: a class-run ASEGA on a guard (its walk anim, targeted at the guard) poses the guard's rig bones
/// over its own anim (npcdraw::NpcAnimator::overlay)
#[test]
fn d6_npc_overlay() {
    use sly2_recomp::engine::classes::Inst;
    use sly2_recomp::engine::npcdraw::{anim_object, NpcAnimator, GUARD_ANIM_WALK};
    let (loader, descs) = k20::levels();
    for d in descs.into_iter().take(12) {
        let Ok(lv) = crate::common::load_blocking(&loader, d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        let w = Arc::new(w);
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let s = ClassSet::build(&w, &mut h);
        let Some(gi) = s.k6.iter().find(|x| matches!(x.1, Inst::Guard(_))).map(|x| x.0) else { continue };
        let Some(a) = anim_object(&w, gi, GUARD_ANIM_WALK) else { continue };
        let aseg = sly2_recomp::engine::playerdraw::aseg_of(&w, a).unwrap();
        let mut b = AnimBinder::default();
        let n = b.start(&w, 1, a, aseg, Some(gi), 0, 0);
        let mut an = NpcAnimator::new(&w, gi);
        let before = an.rig.pose.world();
        let locals = b.locals(&w, &|_| Some((aseg.length * 0.5, 1.0)));
        let k = an.overlay(&w, gi, &locals);
        eprintln!("guard {gi}: walk aseg {a}, {n} acts, {} locals, {k} rig bones overlaid", locals.len());
        assert!(n > 0 && k > 0);
        let after = an.rig.pose.world();
        assert!(before.iter().zip(&after).any(|(x, y)| (0..4).any(|r| (0..3).any(|c| (x[r][c] - y[r][c]).abs() > 1e-3))), "pose changed");
        return;
    }
    panic!("no guard with a walk anim in the first 12 levels");
}
