//! WP-K25 tests: the `EngineHost` anim hooks backed by run-time ASEGAs / SMAs (classes/k25.rs).
use sly2_recomp::engine::classes::k25::AnimRt;
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::{ClassHost, ClassSet};
use sly2_recomp::engine::playerdraw::aseg_of;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

const DT: f32 = 1.0 / 60.0;

use crate::common as k20;

/// first level with an ASEG of length >= 0.5 s and an SM with at least two states
fn anim_level() -> (Arc<World>, usize, usize) {
    let (loader, descs) = k20::levels();
    for d in descs {
        let Ok(lv) = crate::common::load_blocking(&loader, d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        let a = (0..w.objects.len()).find(|&i| aseg_of(&w, i).is_some_and(|a| a.length >= 0.5));
        let sm = (0..w.objects.len()).find(|&i| sly2_recomp::engine::npcdraw::sm_tables(&w, i).is_some_and(|m| m.states.len() >= 2 && m.trans.iter().any(|t| t.aseg.is_some())));
        if let (Some(a), Some(sm)) = (a, sm) {
            return (Arc::new(w), a, sm);
        }
    }
    panic!("no level with ASEG + SM");
}

fn tick(h: &mut EngineHost, n: usize) {
    for _ in 0..n {
        h.k25_tick(DT);
    }
}

/// FUN_0012f9b8 / FUN_00130918 / FUN_00131968 / FUN_00131a00 / FUN_00130b18 / references through the hooks
#[test]
fn k25_asega_hooks() {
    let (w, a, _) = anim_level();
    let len = aseg_of(&w, a).unwrap().length;
    let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
    let anim = a as u32 + 1;
    assert_eq!(h.anim_74(anim), len);
    assert_eq!(h.k22_anim_f(anim, 0x78), 1.0);
    // one-shot: live until its end, then out of 0x3e1218
    let s = h.ee_sub_12f9b8(0f32.to_bits(), 1.0, anim, 1, 0);
    assert!(AnimRt::is_asega(s));
    h.vcall(s, 0xc);
    assert!(!h.ee_sub_131968(s));
    assert_eq!(h.anim_slot_f(s, 0x40), 1.0);
    tick(&mut h, 10);
    assert!((h.anim_slot_f(s, 0x1c) - 10.0 * DT).abs() < 1e-4);
    // FUN_00131a00: +0x44 = rate, +0x40 = rate * +0x48 * +0x4c
    h.ee_sub_131a00(2.0, s);
    assert_eq!((h.anim_slot_f(s, 0x44), h.anim_slot_f(s, 0x40)), (2.0, 2.0));
    tick(&mut h, (len * 30.0) as usize + 5);
    assert!(h.ee_sub_131968(s), "one-shot finished");
    assert!(h.k25.asegas.contains_key(&s), "still referenced by the slot");
    h.ee_sub_1306a8(s);
    assert!(!h.k25.asegas.contains_key(&s), "released");
    // loop (flag 8): still live after 2 lengths; retract ends it
    let l = h.ee_sub_12f9b8(0f32.to_bits(), 1.0, anim, 1, 8);
    tick(&mut h, (len * 120.0) as usize + 2);
    assert!(!h.ee_sub_131968(l));
    assert!(h.anim_slot_f(l, 0x1c) <= len + 1e-4);
    h.ee_sub_130b18(l);
    assert!(h.ee_sub_131968(l));
    // not an ASEG / unknown handles
    assert_eq!(h.ee_sub_12f9b8(0, 1.0, 0, 1, 0), 0);
    assert!(h.ee_sub_131968(0x4100_7777));
}

/// FUN_001f03a8 / FUN_001f0400 / FUN_001f09d8 / FUN_001f0638 / FUN_001f15b8 on a real SM
#[test]
fn k25_sma_hooks() {
    let (w, _, sm) = anim_level();
    let m = sly2_recomp::engine::npcdraw::sm_tables(&w, sm).unwrap();
    let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
    let smh = sm as u32 + 1;
    let (l0, l1) = (m.states[0].0, m.states[1].0);
    assert_eq!(h.ee_sub_1f0400(smh, l1), 1);
    assert_eq!(h.ee_sub_1f0400(smh, 0x7fff_0000), -1);
    let mut slot = h.ee_sub_1f03a8_link(smh, 1, l0, 0, -1);
    assert!(AnimRt::is_sma(slot));
    assert!(!h.ee_sub_1f0638(slot));
    assert_eq!(h.k25.smas[&slot].sma.cur, 0);
    tick(&mut h, 300);
    assert!(!h.ee_sub_1f0638(slot));
    // FUN_001f09d8: jump to state 1
    h.ee_sub_1f09d8(slot, l1);
    let r = &h.k25.smas[&slot];
    assert!(r.sma.cur == 1 || r.sma.asega.is_some() || r.sma.cur != 0, "jumped");
    // every SMA transition with an ASEG runs as a live run-time ASEGA
    if let Some(a) = r.asega {
        assert!(!h.ee_sub_131968(a));
    }
    tick(&mut h, 600);
    h.ee_sub_1f15b8(&mut slot);
    assert_eq!(slot, 0);
    assert!(h.k25.smas.is_empty() && h.k25.asegas.is_empty(), "{:?}", h.k25);
    // FUN_001f1448 = FUN_001f0380(sm, obj, -1, flags) + reference
    let s2 = h.ee_sub_1f1448(smh, 1, 0);
    assert!(!h.ee_sub_1f0638(s2));
    assert_eq!(h.k25.smas[&s2].refs, 1);
}

/// every level: build + post-load + 300 class frames with the run-time anim hooks
#[test]
#[ignore = "heavy"]
fn k25_levels_anim() {
    let (loader, descs) = k20::levels();
    let (mut levels, mut asegas, mut smas, mut sounds) = (0, 0, 0, 0);
    let (mut bodies, mut moving, mut cons, mut fields) = (0, 0, 0, 0);
    let (mut c127, mut c127_lv) = (0, 0);
    let mut vmax = (0.0f32, 0, 0.0f32);
    for d in descs {
        let Ok(lv) = crate::common::load_blocking(&loader, d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        levels += 1;
        let w = Arc::new(w);
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        for _ in 0..300 {
            s.g.t += DT;
            s.g.dt = DT;
            s.frame(&mut h);
        }
        let n127 = s.k25c.len();
        assert_eq!(n127, w.objects.iter().filter(|o| o.class == 127).count());
        for (i, _) in &s.k25c {
            assert_eq!(s.k25_127_get(*i).map(|c| c.f_b0), Some(1));
        }
        c127 += n127;
        c127_lv += (n127 > 0) as usize;
        bodies += h.k25_phys.bodies.len();
        moving += h.k25_phys.bodies.values().filter(|b| b.v.iter().any(|c| *c != 0.0)).count();
        cons += (h.k25_phys.pool.next - sly2_recomp::engine::classes::k25::CON_TAG) as usize;
        for (i, b) in h.k25_phys.bodies.iter() {
            let sp = (b.v[0] * b.v[0] + b.v[1] * b.v[1] + b.v[2] * b.v[2]).sqrt();
            if sp > vmax.0 {
                vmax = (sp, w.objects[*i].class, b.pos[2]);
            }
            assert!(b.pos.iter().chain(b.v.iter()).all(|c| c.is_finite()));
        }
        asegas += h.k25.started;
        sounds += h.k25_snd.plays.len();
        smas += h.k25.sma_started;
        fields += h.k25_fld.fields.len();
        for x in h.k25.asegas.values() {
            assert!(x.st.t.is_finite() && x.st.svt.is_finite());
        }
    }
    eprintln!("K25: {levels} levels, {asegas} ASEGAs and {smas} SMAs started, {sounds} sound emitters created by the class code in 300 frames");
    eprintln!("K25: {bodies} class-owned bodies ({moving} moving) at frame 300, {cons} constraint records, {fields} overlay fields written, fastest {vmax:?} (speed, class, z)");
    eprintln!("K25: class 127: {c127} instances on {c127_lv} levels");
    assert!(c127 > 0);
    assert!(levels >= 40);
    assert!(asegas + smas > 0);
    assert!(bodies > 0);
}

/// sound hooks: FUN_001fbb90 one-shot, FUN_0012ed20 / FUN_0012ede8 emitter slots, FUN_0015c690 positioned sound
#[test]
fn k25_sound_hooks() {
    let mut h = EngineHost::default();
    h.ee_sub_1fbb90(3, 0x29);
    let e = &h.k25_snd.es;
    assert_eq!(e.active.len(), 1);
    let one = e.get(e.active[0]).unwrap();
    assert_eq!((one.sound, one.flags, one.refs), (0x29, 1, 1));
    // C2cabd8 slot 22 / 21: near 10, far 5 -> far = near + 1; volume 0.5; pitch 2
    let ent = [7, 10.0f32.to_bits() as i32, 5.0f32.to_bits() as i32, 0.5f32.to_bits() as i32, 2.0f32.to_bits() as i32];
    let hd = h.emitter_create(4, 0, ent);
    assert_ne!(hd, 0);
    let id = h.k25_snd.slots[&(4, 0)].id.unwrap();
    let em = h.k25_snd.es.get(id).unwrap().clone();
    assert_eq!((em.sound, em.near, em.far, em.f10), (7, 10.0, 11.0, 0.5));
    assert_eq!(em.flags & 0x251, 0x251, "{em:?}");
    assert_eq!(em.refs, 2); // active list + slot
    h.emitter_release(4, 0);
    // FUN_001fb098: flags = 8 (stop request, still in the active list), the slot reference is gone
    let st = h.k25_snd.es.get(id).unwrap();
    assert_eq!((st.flags, st.refs), (8, 1));
    assert!(!h.k25_snd.slots.contains_key(&(4, 0)));
    h.sound_15c690(5, [1.0, 2.0, 3.0, 0.0], [9, 1.0f32.to_bits(), 50.0f32.to_bits(), 1.0f32.to_bits(), 0]);
    let last = *h.k25_snd.es.active.last().unwrap();
    let p = h.k25_snd.es.get(last).unwrap();
    assert_eq!((p.sound, p.pos, p.far), (9, [1.0, 2.0, 3.0], 50.0));
    assert_eq!(p.flags & 0x200, 0x200);
    assert_eq!(h.k25_snd.plays, vec![(3, 0x29), (4, 7), (5, 9)]);
    // FUN_001f07c8 on an unknown handle: done
    assert!(h.snd_done_1f07c8(0x4200_0099));
}

/// physics hooks: class-owned body snapshots (FUN_001f4558 velocity, FUN_0022d348 point velocity, +0xe8 reads)
#[test]
fn k25_physics_hooks() {
    use sly2_recomp::engine::classes::k25::BodySnap;
    let (w, _, _) = anim_level();
    let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
    let b = BodySnap { pos: [1.0, 2.0, 3.0], v: [4.0, 0.0, 0.0], w: [0.0, 0.0, 1.0], gravity: [0.0, 0.0, -10.0], ckind: 2 };
    h.k25_bodies(vec![(5, b)]);
    assert_eq!(h.vcall_10c(5), [4.0, 0.0, 0.0]);
    assert_eq!(h.vcall_10c(6), [0.0; 3]);
    // v + w x (p - pos): w = z, r = x -> w x r = y
    assert_eq!(h.ee_sub_22d348(6, [2.0, 2.0, 3.0]), [4.0, 1.0, 0.0]);
    assert_eq!(h.ee_sub_22d348(7, [2.0, 2.0, 3.0]), [0.0; 3]);
    assert_eq!(h.phys_e8(5).map(|p| p.s_c0), Some(2));
    assert!(h.phys_e8(6).is_none());
    assert_eq!(h.phys_e8_80(5), [0.0, 0.0, -10.0]);
    assert_eq!(h.ee_sub_1f5860(6), [0.0, 0.0, -10.0]);
    // a real level: the guards and the class bodies are in the snapshot after one class frame
    let mut s = ClassSet::build(&w, &mut h);
    s.post_load(&mut h);
    s.g.dt = DT;
    s.frame(&mut h);
    assert_eq!(h.k25_phys.bodies.len(), s.k25_bodies().len());
}

/// constraint hooks: FUN_0012dc18 (kind-1 act record into xf +0x90), FUN_0012d108 retarget, FUN_0012e460 release,
/// the guard +0xfc4 scale record and its effect on FUN_00209618
#[test]
fn k25_constraint_hooks() {
    let (w, _, _) = anim_level();
    let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
    let mut a = h.ee_sub_12dc18(3, 0x58, [1.0, 2.0, 3.0]);
    assert!(a != 0);
    // priority 1 goes before priority 0 (FUN_00230018: before the first record with priority <= its own)
    let mut b = h.k22_pin_12dc18(3, [0.0; 3]);
    let mut c = h.k22_pin_12dc18(3, [0.0; 3]);
    let xf = h.k25_phys.xfs.get(&(3 * 0x100 + 0x5c)).unwrap();
    assert_eq!(xf.list.iter().map(|r| r.id).collect::<Vec<_>>(), vec![c, b]);
    assert_eq!(xf.list[0].w[0], 1);
    assert_eq!(h.k25_phys.xfs[&(3 * 0x100 + 0x58)].list.len(), 1);
    assert_eq!(h.k25_phys.pool.live, 3);
    // only the head record (the later of equal priorities) drives the xf
    h.k22_pin_12d108(b, [9.0; 3]);
    assert_eq!(h.k25_phys.targets.get(&(3 * 0x100 + 0x5c)), Some(&[0.0; 3]));
    h.k22_pin_12d108(c, [0.5, 0.0, 0.0]);
    assert_eq!(h.k25_phys.targets.get(&(3 * 0x100 + 0x5c)), Some(&[0.5, 0.0, 0.0]));
    h.ee_sub_12e460(&mut b);
    assert_eq!(b, 0);
    assert_eq!(h.k25_phys.xfs[&(3 * 0x100 + 0x5c)].list.len(), 1);
    h.ee_sub_12e460(&mut c);
    h.ee_sub_12e460(&mut a);
    assert_eq!(h.k25_phys.pool.live, 0);
    assert!(h.k25_phys.xfs.values().all(|x| x.list.is_empty()));
    let mut z = 1234;
    h.ee_sub_12e460(&mut z);
    assert_eq!(z, 0);
    // guard +0xfc4: one record per guard; FUN_0012d108 scales the guard's root-motion speed
    let g = (0..w.objects.len()).find(|&i| (57..=67).contains(&w.objects[i].class)).map(|i| i as u32 + 1);
    let r = h.k23_guard_fc4(0x3000_0000);
    assert!(r != 0 && h.k23_guard_fc4(0x3000_0000) == r);
    assert_eq!(h.k25_phys.fc4_scale(0x3000_0000), 1.0);
    h.k22_pin_12d108(r, [0.25; 3]);
    assert_eq!(h.k25_phys.fc4_scale(0x3000_0000), 0.25);
    if let Some(g) = g {
        let v1 = h.ee_sub_209618(g as usize - 1, 1, 0.3, 1.0);
        let r = h.k23_guard_fc4(g);
        h.k22_pin_12d108(r, [0.5; 3]);
        let v2 = h.ee_sub_209618(g as usize - 1, 1, 0.3, 1.0);
        for k in 0..3 {
            assert!((v2[k] - v1[k] * 0.5).abs() < 1e-5);
        }
    }
    // FUN_0012dc18 through ee_call (k21_54)
    assert!(h.ee_call(0x12dc18, &[3, 0x5c, 5, 7, 2]) != 0);
}

/// object field overlay (obj_i32 / obj_f32 / obj_set_u32 / obj_or_40 / obj_and_40 / obj_flags_28) and the
/// FUN_001f4c58 child lookup
#[test]
fn k25_field_hooks() {
    use sly2_recomp::engine::classes::k25::BONE_TAG;
    let (w, a, sm) = anim_level();
    let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
    let o = 1u32;
    assert_eq!(h.obj_i32(o, 0x18), w.objects[0].oid);
    assert_eq!(h.obj_flags_28(o), 0);
    h.obj_set_u32(o, 0x2c, 1 << 28);
    assert_eq!(h.obj_flags_28(o) >> 60 & 1, 1);
    assert_eq!(h.obj_i32(o, 0x40), 0);
    h.obj_or_40(o, 0x102);
    h.obj_and_40(o, !2);
    assert_eq!(h.obj_i32(o, 0x40), 0x100);
    h.obj_set_u32(o, 0x38, 2.5f32.to_bits());
    assert_eq!(h.obj_f32(o, 0x38), 2.5);
    // run-time ASEGA / SMA fields
    let x = h.ee_sub_12f9b8(0, 1.0, a as u32 + 1, o, 0);
    assert_eq!(h.obj_i32(x, 0x14) as u32, a as u32 + 1);
    assert_eq!(h.obj_f32(x, 0x40), h.anim_slot_f(x, 0x40));
    tick(&mut h, 3);
    assert!((h.obj_f32(x, 0x1c) - 3.0 * DT).abs() < 1e-4);
    let s = h.ee_sub_1f03a8_link(sm as u32 + 1, o, -1, 0, 0);
    if s != 0 {
        assert_eq!(h.obj_i32(s, 0x24) as u32, sm as u32 + 1);
    }
    // FUN_001f4c58: first ALO with a bone list; lookup of its first bone's (oid, kind)
    let mut found = false;
    for i in 0..w.objects.len() {
        let Some(al) = sly2_recomp::engine::gfx::attach::alo_of(&w, i) else { continue };
        let Some((k, b)) = al.base.bones.iter().enumerate().find(|(_, b)| b.word != 0) else { continue };
        let first = al.base.bones.iter().position(|c| c.word == b.word && c.kind == b.kind).unwrap();
        let c = h.ee_sub_1f4c58(i as u32 + 1, b.word, b.kind as i32);
        assert_eq!(c, BONE_TAG | (i as u32 + 1) << 10 | (first as u32 + 1), "bone {k}");
        assert_eq!(h.ee_sub_1f4c58(i as u32 + 1, b.word, b.kind as i32 + 0x1000), 0);
        found = true;
        break;
    }
    assert!(found);
    assert_eq!(h.ee_sub_1f4c58(0x3000_0000, 0, 0), 0);
}

/// class 54 slot 54 helpers: FUN_0011f198 cosine, FUN_001aa020 + FUN_001a9fe0 prepend to the +0x13e4 list
#[test]
fn k25_c54_contact_list() {
    use sly2_recomp::engine::classes::k18_54::{ee_sub_11f198, ee_sub_1a9fe0, ee_sub_1aa020};
    assert_eq!(ee_sub_11f198([0.0; 3], [0.0, 0.0, 1.0]), 0.0);
    assert!((ee_sub_11f198([2.0, 0.0, 0.0], [-3.0, 0.0, 0.0]) + 1.0).abs() < 1e-6);
    assert!((ee_sub_11f198([1.0, 1.0, 0.0], [0.0, 1.0, 0.0]) - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
    use sly2_recomp::engine::player::p5_hull::{ContactRec, CtSide, GeomRef};
    let side = CtSide { geom: GeomRef::Jt, point: [0.0; 3], local: [0.0; 3], vel: [0.0; 3] };
    let a = ContactRec { side: [side, side], n: [0.0, 0.0, 1.0], gap: 1.0, f94: 0.5, f98: 0.5, kind: 0, feat_a: 0, feat_b: 0, relvel: [0.0; 3] };
    let mut b = a;
    b.gap = 2.0;
    let mut list = vec![];
    ee_sub_1a9fe0(&mut list, ee_sub_1aa020(&a));
    ee_sub_1a9fe0(&mut list, ee_sub_1aa020(&b));
    assert_eq!(list.iter().map(|r| r.gap).collect::<Vec<_>>(), vec![2.0, 1.0]);
}

/// class 127 slots 15 / 48 against a host that models the sequencer base fields, pad and screens
#[test]
fn k25_c127_prompt() {
    use sly2_recomp::engine::classes::k25_127::{ee_sub_2121c8, C2dabf0, PAD_MASK};
    use sly2_recomp::engine::classes::ClassGlobals;
    #[derive(Default)]
    struct M {
        f58: i32,
        f5c: f32,
        f7c: i32,
        pad: bool,
        screen: bool,
        calls: Vec<(u32, Vec<u32>)>,
        finished: usize,
        unpaused: usize,
        paused: usize,
    }
    impl ClassHost for M {
        fn obj_i32(&mut self, _o: u32, off: u32) -> i32 {
            match off {
                0x58 => self.f58,
                0x7c => self.f7c,
                _ => 0,
            }
        }
        fn obj_f32(&mut self, _o: u32, off: u32) -> f32 {
            if off == 0x5c { self.f5c } else { 0.0 }
        }
        fn ee_call(&mut self, addr: u32, args: &[u32]) -> i64 {
            self.calls.push((addr, args.to_vec()));
            match addr {
                0x187d80 => (self.pad && args[1] == PAD_MASK) as i64,
                0x1d9880 => self.screen as i64,
                _ => 0,
            }
        }
        fn vcall(&mut self, _o: u32, off: u32) {
            if off == 0x194 {
                self.finished += 1;
            }
        }
        fn ee_sub_20f030(&mut self, _k: i32) {
            self.unpaused += 1;
        }
        fn ee_sub_20efc0(&mut self, _k: i32) {
            self.paused += 1;
        }
    }
    assert!(ee_sub_2121c8(1, 1) && !ee_sub_2121c8(2, 0) && !ee_sub_2121c8(0, 2));
    let mut g = ClassGlobals { t: 10.0, g_3e0f9c: 1, ..Default::default() };
    let mut h = M { f58: 1, ..Default::default() };
    let mut c = C2dabf0::default();
    c.ee_sub_211d08(4, &mut h);
    assert_eq!(c.f_b0, 1);
    // not running: nothing
    c.ee_sub_211d38(4, &mut g, &mut h);
    assert_eq!((c.f_110, h.finished), (0, 0));
    // running, state 2 for 0.6 s, pad pressed
    h.f58 = 2;
    h.f5c = 9.4;
    h.pad = true;
    h.screen = true;
    c.ee_sub_211d38(4, &mut g, &mut h);
    assert_eq!((c.f_110, h.finished), (1, 0), "a screen holds the prompt");
    h.screen = false;
    c.ee_sub_211d38(4, &mut g, &mut h);
    assert_eq!((h.finished, h.unpaused, g.g_2dd318), (1, 1, 1));
    assert!(h.calls.iter().any(|x| x.0 == 0x162df8));
    // +0x114: no count; too early: no finish
    c.f_114 = 1;
    c.ee_sub_211d38(4, &mut g, &mut h);
    assert_eq!((h.finished, g.g_2dd318), (2, 1));
    h.f5c = 9.6;
    c.ee_sub_211d38(4, &mut g, &mut h);
    assert_eq!(h.finished, 2);
    // slot 48 only while +0x7c == -1
    let mut c = C2dabf0::default();
    c.ee_sub_211ca8(4, 7, &mut h);
    assert_eq!((h.paused, c.f_110), (0, 0));
    h.f7c = -1;
    h.calls.clear();
    c.ee_sub_211ca8(4, 7, &mut h);
    assert_eq!((h.paused, c.f_110), (1, 1));
    assert_eq!(h.calls, vec![(0x2128e8, vec![5, 7]), (0x213b00, vec![5, 0x33a])]);
}
