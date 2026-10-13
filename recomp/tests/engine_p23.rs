//! WP-P23: Bentley / Murray posed from their own anims (player::p23 slot-13 bindings, playerdraw::po).

use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::ClassSet;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::player::p20::{apply_po, PoCurrent, PoHost, CLASS_BENTLEY, CLASS_MURRAY};
use sly2_recomp::engine::player::{Jt, Joy};
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;


fn tables() -> (LevelLoader, Vec<sly2_recomp::engine::disc::descriptor::Descriptor>) {
    let iso = Arc::new(IsoImage::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso")).expect("ISO"));
    let t = sly2_recomp::engine::disc::descriptor::static_table(&iso.read_file("SCUS_973.16").unwrap()).unwrap();
    let l = LevelLoader::new(iso, ModOverlay::default());
    let boot = l.load_blocking(t[25]).unwrap();
    let d = boot.header.level_table.entries.iter().map(|e| e.desc).collect();
    (l, d)
}

#[test]
fn p23_po_anims() {
    let (l, ds) = tables();
    let mut seen = [false; 2];
    for d in ds {
        let Ok(lv) = l.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        for (k, cls) in [CLASS_BENTLEY, CLASS_MURRAY].into_iter().enumerate() {
            if seen[k] { continue; }
            let Some(o) = w.objects.iter().position(|x| x.class == cls) else { continue };
            let w = Arc::new(w.clone());
            let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
            let mut set = ClassSet::build(&w, &mut h);
            set.post_load(&mut h);
            let mut cur = PoCurrent::default();
            if !apply_po(&w, w.objects[o].oid, &mut set, &mut h, &mut cur) || cur.obj != Some(o) { continue; }
            h.po = Some(PoHost { obj: o, jt: Box::new(Jt::new([0.0; 3])) });
            let mut an = sly2_recomp::engine::playerdraw::po::PoAnimator::new(&w, o);
            eprintln!("rig bones {} blends {:?}", an.rig.keys.len(), an.blends.keys().collect::<Vec<_>>());
            for f in 0..120 {
                set.g.dt = 1.0 / 60.0;
                let mut joy = Joy::default();
                joy.angle = 0.3;
                joy.mag = if f < 30 { 0.0 } else { 1.0 };
                set.k18_feed_joy(joy);
                set.frame(&mut h);
                let bb = std::mem::take(&mut h.po_bb58);
                let bb = bb.iter().rev().find(|r| r.0 == o).map(|r| (r.1, r.2));
                let plays = sly2_recomp::engine::playerdraw::po::po_plays(&h.k25, o);
                an.step(&w, &plays, bb, 1.0 / 60.0);
                if f % 10 == 0 {
                    let wm = an.rig.pose.world();
                    let r0 = an.rig.index.get(&(o, 0)).map(|&b| wm[b][3]);
                    let maxd = wm.iter().zip(&an.rig.rest_world).map(|(a, b)| (0..3).map(|i| (a[3][i] - b[3][i]).abs()).fold(0f32, f32::max)).fold(0f32, f32::max);
                    if std::env::var("P23_SCALE").is_ok() {
                        for (b, m) in wm.iter().enumerate() {
                            let sc: Vec<f32> = (0..3).map(|i| (0..3).map(|j| m[i][j] * m[i][j]).sum::<f32>().sqrt()).collect();
                            if sc.iter().any(|&x| !(0.5..2.0).contains(&x)) { eprintln!("    bone {b} {:?} oid {} scale {sc:?}", an.rig.keys[b], w.objects[an.rig.keys[b].0].oid); }
                        }
                    }
                    eprintln!("  src {:?} bound {} bb {bb:?} root {r0:?} maxd {maxd}", an.source, an.bound_bones());
                    let (st, a, s) = if cls == CLASS_BENTLEY { let b = set.k16_get(o).unwrap(); (b.state, b.m.u(0x1030), b.m.u(0x1034)) } else { let b = &set.k18_murray(o).unwrap().b; (b.state, b.m.u(0x1030), b.m.u(0x1034)) };
                    let mine: Vec<_> = h.k25.asegas.iter().filter(|x| x.1.obj == o as u32 + 1).map(|(k, x)| (*k, x.aseg, w.objects[x.aseg].oid, x.st.t, x.st.alive)).collect();
                    let sm = h.k25.smas.get(&s).map(|x| (x.sm, x.asega));
                    eprintln!("cls {cls} f{f} state {st} 1030 {a:#x} 1034 {s:#x} sm {sm:?} asegas {mine:?}");
                    // idle: the state-0 SM's ASEGA poses the po; run: the class's FUN_0020bb58 block blend
                    if f < 30 {
                        assert!(s != 0 && sm.is_some(), "state 0 SMA bound");
                        assert!(matches!(an.source, sly2_recomp::engine::playerdraw::po::PoSource::Asega(_)), "{:?}", an.source);
                    } else {
                        assert!(matches!(an.source, sly2_recomp::engine::playerdraw::po::PoSource::Blend(..)), "{:?}", an.source);
                    }
                    assert!(an.bound_bones() > 20 && maxd > 1.0, "posed");
                }
            }
            seen[k] = true;
        }
        if seen.iter().all(|&s| s) { break; }
    }
    assert!(seen.iter().all(|&s| s), "Bentley and Murray found");
}

/// WP-P23 item 2: the slot-78 callers on a real level (level-table row 4: Sly, Bentley, Murray and a vault):
/// FUN_002206c8 carry machine (Murray +0x1910 -> 3 makes Murray active, leaving 3 hands back to Bentley),
/// a Sly slot 78(1) and the vault FUN_001c8730 state 0 slot 78(+0x190 = Sly, 1) release the controlled Bentley.
#[test]
fn p23_switch_callers() {
    use sly2_recomp::engine::classes::ClassHost;
    use sly2_recomp::engine::player::p23::apply_switches;
    let (l, ds) = tables();
    let lv = l.load_blocking(ds[4]).unwrap();
    let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
    let cls = |c: u16| w.objects.iter().position(|x| x.class == c).expect("class in hub");
    let (b, m, sly) = (cls(CLASS_BENTLEY), cls(CLASS_MURRAY), cls(sly2_recomp::engine::player::p20::CLASS_SLY));
    let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
    let mut set = ClassSet::build(&w, &mut h);
    set.post_load(&mut h);
    let mut cur = PoCurrent::default();
    assert!(apply_po(&w, w.objects[b].oid, &mut set, &mut h, &mut cur));
    assert_eq!(set.k18_active, Some(b));
    let _ = std::mem::take(&mut h.p23_switch);
    let run = |set: &mut ClassSet, h: &mut EngineHost| {
        let q = std::mem::take(&mut h.p23_switch);
        apply_switches(&w, set, h, q)
    };
    // FUN_002206c8(Murray + 0x1910, 3): owner slot 78(1)
    h.po_part_s(0x2206c8, m, 0x1910, 3);
    assert_eq!(run(&mut set, &mut h), vec![Some(m)]);
    assert_eq!(set.k18_active, Some(m));
    assert_eq!(set.k18_murray(m).unwrap().b.m.i(0x1914), 3);
    // same state: return
    h.po_part_s(0x2206c8, m, 0x1910, 3);
    assert!(run(&mut set, &mut h).is_empty());
    // 3 -> 4: no hand-back; 4 -> 0: none either (only leaving 3)
    h.po_part_s(0x2206c8, m, 0x1910, 4);
    assert!(run(&mut set, &mut h).is_empty());
    assert_eq!(set.k18_active, Some(m));
    // back to 3, then 3 -> -1: DAT_002dd5bc (Bentley) slot 78(1)
    h.po_part_s(0x2206c8, m, 0x1910, 3);
    assert_eq!(run(&mut set, &mut h), vec![Some(m)]);
    h.po_part_s(0x2206c8, m, 0x1910, -1);
    assert_eq!(run(&mut set, &mut h), vec![Some(b)]);
    assert_eq!(set.k18_active, Some(b));
    // Sly slot 78(1): the controlled Bentley is released
    h.vcall_arg(sly as u32 + 1, 0x27c, 1);
    assert_eq!(run(&mut set, &mut h), vec![None]);
    assert_eq!(set.k18_active, None);
    // vault FUN_001c8730: 1 -> 0 hands control to its +0x190 player (Sly) with Bentley controlled
    set.k18_set_active(Some(b), &mut h);
    assert_eq!(set.k18_active, Some(b));
    let v = cls(78);
    {
        let (k9, g) = (&mut set.k9, &mut set.g);
        let c = k9.iter_mut().find(|x| x.0 == v).map(|x| &mut x.1).expect("vault instance");
        let sly2_recomp::engine::classes::Inst::K9(c) = c else { panic!("vault inst") };
        c.f_180 = 1;
        c.f_190 = sly as u32 + 1;
        c.ee_sub_1c8730(v, 0, g, &mut h);
    }
    assert_eq!(run(&mut set, &mut h), vec![None]);
    assert_eq!(set.k18_active, None);
    // the vault's own slot 78 (state 1) is not a character switch
    {
        let (k9, g) = (&mut set.k9, &mut set.g);
        let c = k9.iter_mut().find(|x| x.0 == v).map(|x| &mut x.1).unwrap();
        let sly2_recomp::engine::classes::Inst::K9(c) = c else { panic!() };
        c.ee_sub_1c8730(v, 1, g, &mut h);
    }
    assert!(run(&mut set, &mut h).is_empty());
}

/// WP-P23 item 3: the player's ASEG timed events reach FUN_00131290 and its receiver: Sly's run (JtAnimator
/// fired list), the po's ASEGAs (PoAnimator window replay); the level effect sink gets the sound / rumble calls.
#[test]
fn p23_player_anim_events() {
    use sly2_recomp::engine::anim::events::{ee_sub_131290, EffectCall};
    use sly2_recomp::engine::audio::emitter::EmitterSystem;
    use sly2_recomp::engine::camera::CmClock;
    use sly2_recomp::engine::game::anim_play::LevelEffectSink;
    use sly2_recomp::engine::player::p23::{dispatch_events, events_between};
    use sly2_recomp::engine::playerdraw::{aseg_of, find_jt, JtAnimator, JtRig, FIELD_RUN};
    let (l, ds) = tables();
    let lv = l.load_blocking(ds[4]).unwrap();
    let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
    // window replay order (FUN_00130f88 cursor): forward, wrap
    let jt = find_jt(&w).unwrap();
    let mut an = JtAnimator::new(JtRig::build(&w, jt));
    an.request(&w, FIELD_RUN, -1);
    let mut fired = vec![];
    for _ in 0..300 {
        an.step(1.0 / 60.0);
        fired.extend(an.fired.iter().copied());
    }
    let kinds = |f: &[(usize, usize, bool)]| -> Vec<u8> { f.iter().map(|&(o, i, _)| aseg_of(&w, o).unwrap().events1[i].kind).collect() };
    eprintln!("Sly run: {} events kinds {:?}", fired.len(), kinds(&fired));
    // dispatch through the level receiver
    let mut em = EmitterSystem::default();
    let mut rumble = vec![];
    let mut sink = LevelEffectSink { world: &w, emitters: &mut em, rumble: &mut rumble, cm: None, clk: CmClock { t: 0.0, dt: 1.0 / 60.0, dt_real: 1.0 / 60.0 }, current: 0, sounds: vec![], start: vec![], pending: vec![] };
    let mut calls = 0;
    for &(o, i, r) in &fired {
        sink.current = o;
        calls += dispatch_events(aseg_of(&w, o).unwrap(), &[(i, r)], &mut sink);
    }
    eprintln!("Sly run: {calls} calls, {} sounds, {} rumble, pending {:?}", sink.sounds.len(), rumble_len(&sink), &sink.pending.iter().take(8).collect::<Vec<_>>());
    // every fired event with an effect produced exactly one call
    let want = fired.iter().filter(|&&(o, i, r)| ee_sub_131290(&mut Default::default(), &aseg_of(&w, o).unwrap().events1[i], r).is_some()).count();
    assert_eq!(calls, want);
    let sounds = fired.iter().filter(|&&(o, i, r)| matches!(ee_sub_131290(&mut Default::default(), &aseg_of(&w, o).unwrap().events1[i], r), Some(EffectCall::K0 { s, .. }) if s != -1)).count();
    assert_eq!(sink.sounds.len(), sounds);
    // Sly's run blend (FUN_00137a30 shares its first entry's lists): the four footfall labels 0x17d..0x180 as
    // owner message 9 (FUN_00188f20 msg 9 handles only 0x164 / 0x165: no effect), once per loop
    assert!(fired.len() >= 8 && kinds(&fired).iter().all(|&k| k == 0xb), "{fired:?}");
    assert!(sink.pending.iter().all(|c| matches!(c, EffectCall::Msg9 { v: 0x17d..=0x180, .. })));
    // the po (Bentley, row 4): its run blend events through PoAnimator
    let b = w.objects.iter().position(|x| x.class == CLASS_BENTLEY).unwrap();
    let mut pa = sly2_recomp::engine::playerdraw::po::PoAnimator::new(&w, b);
    let blk = *pa.blends.keys().min().expect("po blend");
    let mut pf = vec![];
    for _ in 0..300 {
        pa.step(&w, &[], Some((blk, 0.5)), 1.0 / 60.0);
        pf.extend(pa.fired.iter().copied());
    }
    eprintln!("Bentley blend {blk:#x}: {} events kinds {:?}", pf.len(), kinds(&pf));
    assert!(!pf.is_empty(), "po blend events");
    // the po: window replay over an ASEG with timed events
    let (b, a) = w.objects.iter().enumerate().filter(|(_, x)| x.class == 0x6d).filter_map(|(i, _)| aseg_of(&w, i).map(|a| (i, a))).find(|(_, a)| a.length > 0.5 && a.events1.iter().filter(|e| e.time.is_some_and(|t| t > 0.0)).count() >= 2).expect("ASEG with timed events");
    let all = events_between(a, -1.0, a.length);
    assert_eq!(all.len(), a.events1.iter().filter(|e| e.time.is_some()).count());
    let half = a.length * 0.5;
    let mut split = events_between(a, -1.0, half);
    split.extend(events_between(a, half, a.length));
    assert_eq!(split, all, "ASEG {b}");
    // wrap: (t0, length] then [0, t1]
    let wrap = events_between(a, half, 0.0);
    let n0 = a.events1.iter().filter(|e| e.time.is_some_and(|t| t > half)).count() + a.events1.iter().filter(|e| e.time == Some(0.0)).count();
    assert_eq!(wrap.len(), n0);
    assert!(!fired.is_empty() || a.length > 0.0);
}

fn rumble_len(s: &sly2_recomp::engine::game::anim_play::LevelEffectSink) -> usize {
    s.rumble.len()
}

/// WP-P23 item 4: Sly anim audit. Every JT field the port's SetJtJts paths can request (0x18c250 over every
/// jts x jtbs from a fresh JT, the JMP body table 0x195fb8, jts 8 0x19eab0 bodies, cane jtcs, damage states,
/// binoc, jts 0xd / 0xe / 0x11) must resolve on Sly's rig (blend field or bound ASEG member); the rest falls
/// back (keeps the previous pose; rest pose if nothing played yet). Prints the unresolved set per level.
#[test]
fn p23_sly_anim_audit() {
    use sly2_recomp::engine::player::JtAnim;
    use sly2_recomp::engine::playerdraw::{blend_fields, find_jt, JtRig};
    use std::collections::{BTreeMap, BTreeSet};
    struct Rec(Vec<(u32, i32, i32)>);
    impl JtAnim for Rec {
        fn play_aseg(&mut self, f: u32, jts: i32, jtbs: i32) {
            self.0.push((f, jts, jtbs));
        }
    }
    let mut req: BTreeMap<u32, BTreeSet<(i32, i32)>> = BTreeMap::new();
    for jts in 0..0x1c {
        for jtbs in -1..0x20 {
            let r = std::panic::catch_unwind(|| {
                let mut jt = Jt::new([0.0; 3]);
                let mut rec = Rec(vec![]);
                jt.set_jts(jts, jtbs, &mut rec);
                rec.0
            });
            for (f, a, b) in r.unwrap_or_default() {
                req.entry(f).or_default().insert((a, b));
            }
        }
    }
    let mut add = |f: Option<u32>, tag: (i32, i32)| {
        if let Some(f) = f {
            req.entry(f).or_default().insert(tag);
        }
    };
    for jtbs in -1..0x20 {
        for idx in 0..5 {
            add(sly2_recomp::engine::player::jmp_body_aseg_195fb8(jtbs, idx), (-2, jtbs));
        }
    }
    let mut p14 = sly2_recomp::engine::player::p14::JtP14::default();
    for s in 0..12 {
        add(sly2_recomp::engine::player::p14::jts8_aseg(s, &p14), (8, s));
    }
    p14.len_fc4 = Some(1.0);
    add(sly2_recomp::engine::player::p14::jts8_aseg(0, &p14), (8, 0));
    for n in 0..0x18 {
        add(sly2_recomp::engine::player::p11_cane::jtcs_field(n), (-3, n));
    }
    for s in 0..13 {
        add(Some(0xe90 + 4 * s), (-4, s as i32));
    }
    for f in [sly2_recomp::engine::player::p15_binoc::FIELD_BINOC_ENTER, sly2_recomp::engine::player::p15_binoc::FIELD_BINOC_EXIT, 0x1a80, 0x1a84, 0x1d34] {
        add(Some(f), (-5, 0));
    }
    let (l, ds) = tables();
    let blends: BTreeSet<u32> = blend_fields().into_iter().map(|b| b.0).collect();
    let mut worst = 0;
    for row in [0usize, 4, 14] {
        let lv = l.load_blocking(ds[row]).unwrap();
        let w = World::load(&lv, LoadOptions::default()).unwrap();
        let rig = JtRig::build(&w, find_jt(&w).unwrap());
        let built = rig.build_blends(&w);
        let mut bad = vec![];
        for (f, who) in &req {
            let ok = if blends.contains(f) {
                built.contains_key(f)
            } else if let Some(oid) = sly2_recomp::engine::playerdraw::sm_field_oid(*f) {
                // WP-P23: SM fields start their SM (FUN_001f03a8) instead of an ASEGA
                sly2_recomp::engine::playerdraw::ee_sub_15e938_below(&w, rig.jt, oid).is_some()
            } else {
                rig.aseg_for(*f).is_some()
            };
            if !ok {
                bad.push((*f, who.iter().take(3).copied().collect::<Vec<_>>()));
            }
        }
        eprintln!("row {row}: {} fields requested, {} unresolved:", req.len(), bad.len());
        for (f, _) in &bad {
            let oid = sly2_recomp::engine::playerdraw::JT_TABLE.iter().find(|r| r.2 == *f).map(|r| r.1);
            let objs: Vec<(usize, u16)> = oid.map(|o| w.find_oid(o).iter().map(|&i| (i, w.objects[i].class)).collect()).unwrap_or_default();
            eprintln!("  probe {f:#x}: member {:?} field_members {:?} oid {oid:?} objs {objs:?}", rig.members.get(f), sly2_recomp::engine::playerdraw::field_members(*f));
        }
        for (f, who) in &bad {
            eprintln!("  {f:#x} from {who:?}");
        }
        worst = worst.max(bad.len());
        // WP-P23: jtbs 3 / 4 / 10 start their SM (0x18c250 iStack_118) and pose from its transition ASEGA
        for (f, jtbs) in [(0x1950u32, 3), (0x1bc4, 4), (0x1f14, 10)] {
            let mut an = sly2_recomp::engine::playerdraw::JtAnimator::new(JtRig::build(&w, rig.jt));
            an.request(&w, f, jtbs);
            let (smo, sma) = an.sm.clone().unwrap_or_else(|| panic!("row {row} field {f:#x}: no SM"));
            eprintln!("row {row} sm field {f:#x}: SM obj {smo} asega {:?} current {}", sma.asega.as_ref().map(|p| p.aseg), an.current.is_some());
            assert!(an.current.is_some(), "row {row} field {f:#x}: SM gave no ASEGA");
        }
        // the locomotion / jump / ledge / damage / cane fields stay resolved
        for f in [0x1324u32, 0x1994, 0x414, 0x1ce0, 0xe90, 0x1950, 0x1bc4, 0x1f14, 0x3e0, 0x408, 0x40c] {
            assert!(!bad.iter().any(|b| b.0 == f), "row {row} field {f:#x}");
        }
    }
    eprintln!("worst unresolved {worst}");
    // WP-P23 remaining: 0x478 (JMP[0x6e], unbound for Sly), 0xfc0 / 0xfc4 (pipe, no oid-170 objects in
    // these levels), 0x1c34 (run2 blend: its entries +0x1c44 / +0x1c84 have no FUN_001a64e8 rows)
    assert!(worst <= 4, "{worst}");
}
