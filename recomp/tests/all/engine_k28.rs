//! WP-K28 tests: guard routine starters (census), live-boot gaps, sequence / anim-event / message gaps.
use sly2_recomp::engine::classes::ClassSet;
use std::collections::HashSet;

use crate::common as k20;

mod scr {
    use sly2_recomp::engine::audio::emitter::EmitterSystem;
    use sly2_recomp::engine::camera::Cm;
    use sly2_recomp::engine::classes::plugin::EngineHost;
    use sly2_recomp::engine::classes::ClassSet;
    use sly2_recomp::engine::game::methods::cls::ClassLink;
    use sly2_recomp::engine::game::save::{GameData, SaveVars};
    use sly2_recomp::engine::game::scripts::run_level_scripts_framed;
    use sly2_recomp::engine::game::spawn::SpawnAliases;
    use sly2_recomp::engine::game::splice_host::*;
    use sly2_recomp::engine::game::vm_state::*;
    use sly2_recomp::engine::jobs::JobMgr;
    use sly2_recomp::engine::splice::frame::lookup;
    use sly2_recomp::engine::world::{LoadOptions, World};
    use std::sync::Arc;

    /// J10 driver: level scripts, then every task with a 0x13 handler entered. Returns the state.
    pub fn run(lv: &sly2_recomp::engine::disc::LoadedLevel, enter: bool) -> (Arc<World>, SpliceState) {
        let w = Arc::new(World::load(lv, LoadOptions::default()).unwrap());
        let vars = SaveVars::from_block(&lv.header.block_3a18);
        let mut st = SpliceState::default();
        st.enter_level(1);
        let mut host = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut set = ClassSet::build(&w, &mut host);
        set.post_load(&mut host);
        st.sched.cls = ClassLink { set: Some(set), host, ..Default::default() };
        let mut jobs = JobMgr::load(&w);
        let (mut em, mut data, mut al, mut rum) = (EmitterSystem::default(), GameData::default(), SpawnAliases::default(), Vec::new());
        let mut cm = Cm::new();
        {
            let base = GameSpliceHost { t: 0.0, cm: None, emitters: &mut em, vars: Some(&vars), data: &mut data };
            let level = LevelSpliceHost { base, mission: -1, aliases: &mut al, rumble: &mut rum };
            let mut h = PersistentHost { level, cm: Some(&mut cm), clk: Default::default(), world: Some(&w), table: Some(&lv.header.level_table), sched: &mut st.sched };
            let (_, frames) = run_level_scripts_framed(&mut st.vm, &lv.data, &w, &mut h);
            st.frames = frames;
        }
        if enter {
            let tasks: Vec<usize> = (0..jobs.tasks.len())
                .filter(|&t| object_frame(&st.frames, &w, jobs.tasks[t].obj as u32).is_some_and(|f| lookup(&f, 0x13, true).is_some()))
                .collect();
            for &t in &tasks {
                jobs.FUN_00163c28(t);
                job_messages(&mut st.sched, &jobs);
                let SpliceState { vm, sched, errors, frames, .. } = &mut st;
                let base = GameSpliceHost { t: 0.0, cm: None, emitters: &mut em, vars: Some(&vars), data: &mut data };
                let level = LevelSpliceHost { base, mission: -1, aliases: &mut al, rumble: &mut rum };
                let mut h = PersistentHost { level, cm: Some(&mut cm), clk: Default::default(), world: Some(&w), table: Some(&lv.header.level_table), sched };
                send_messages(vm, errors, frames, &mut h);
            }
        }
        (w, st)
    }

    /// The live order per frame: class tick (`tick_classes`), lend (`lend_classes`: sequence starts), script
    /// tick (`tick_scripts`: job / class messages, then the world update callbacks). Calls `each` after
    /// every frame with the class set.
    pub fn frames(lv: &sly2_recomp::engine::disc::LoadedLevel, w: &Arc<World>, st: &mut SpliceState, n: usize, log: bool, mut each: impl FnMut(&sly2_recomp::engine::classes::ClassSet)) -> usize {
        let mut sent = 0;
        let vars = SaveVars::from_block(&lv.header.block_3a18);
        let (mut em, mut data, mut al, mut rum) = (EmitterSystem::default(), GameData::default(), SpawnAliases::default(), Vec::new());
        let mut cm = Cm::new();
        for f in 0..n {
            let t = (f + 1) as f32 / 60.0;
            {
                let l = &mut st.sched.cls;
                let set = l.set.as_mut().unwrap();
                set.g.t = t;
                set.g.dt = 1.0 / 60.0;
                set.frame(&mut l.host);
            }
            st.sched.cls.drain_seq_starts();
            sent += st.sched.cls.host.k27_script_msgs.len();
            if log {
                for (o, m, a) in &st.sched.cls.host.k27_script_msgs {
                    let fr = object_frame(&st.frames, w, o - 1);
                    let hd = fr.and_then(|f| lookup(&f, *m, true)).map(|r| match r {
                        sly2_recomp::engine::splice::value::Ref::Proc(p) => format!("nargs {} params {} body {}", p.nargs, p.params.clone().map(|b| super::pp(&sly2_recomp::engine::splice::value::Ref::Pair(b), 0)).unwrap_or_default(), p.body.clone().map(|b| super::pp(&sly2_recomp::engine::splice::value::Ref::Pair(b), 0)).unwrap_or_default()),
                        r => format!("{r:?}"),
                    });
                    eprintln!("msg obj {} class {} msg {m} args {a:?}: {}", o - 1, w.objects[*o as usize - 1].class, hd.unwrap_or_default().chars().take(700).collect::<String>());
                }
            }
            let SpliceState { vm, sched, errors, frames, .. } = st;
            let base = GameSpliceHost { t, cm: None, emitters: &mut em, vars: Some(&vars), data: &mut data };
            let level = LevelSpliceHost { base, mission: -1, aliases: &mut al, rumble: &mut rum };
            let mut h = PersistentHost { level, cm: Some(&mut cm), clk: Default::default(), world: Some(&**w), table: Some(&lv.header.level_table), sched };
            send_messages(vm, errors, frames, &mut h);
            tick_world_update(vm, errors, &mut h, t, 1.0 / 60.0);
            each(st.sched.cls.set.as_ref().unwrap());
        }
        sent
    }
}


fn patrolling(set: &ClassSet) -> impl Iterator<Item = usize> + '_ {
    set.k6.iter().map(|x| x.0).filter(|&i| set.k6_guard(i).is_some_and(|g| g.brain.sub.code == 5 && g.brain.sub.routine.is_some()))
}

/// method-594 / 0x171cf8 calls seen by the class link (recorded = failed)
#[test]
#[ignore]
fn k28_probe_routine_calls() {
    let (loader, descs) = k20::levels();
    let mut hist = std::collections::BTreeMap::new();
    for (li, d) in descs.into_iter().enumerate() {
        let Ok(lv) = crate::common::load_blocking(&loader, d) else { continue };
        let (_w, st) = scr::run(&lv, true);
        let set = st.sched.cls.set.as_ref().unwrap();
        let rec: Vec<_> = st.sched.cls.recorded.iter().filter(|r| matches!(r.callee, 0x170e68 | 0x171cf8)).map(|r| (r.callee, r.obj)).collect();
        let errs = st.errors.len();
        eprintln!("level {li}: guards {} code5 {} failed {rec:x?} errors {errs}", set.k6.len(), patrolling(set).count());
        for e in &st.errors {
            *hist.entry(format!("{e:?}").chars().take(120).collect::<String>()).or_insert(0) += 1;
        }
    }
    for (k, v) in &hist {
        eprintln!("{v:5} {k}");
    }
}

/// Item 1 census: guards running a routine (brain substate 5 with a routine, the K27 measure) over the 43
/// levels x 900 frames, with the class tick, the script tick and the class -> script messages interleaved as
/// in the live app (K27 measured 8 with the class tick alone after the task enters).
#[test]
#[ignore = "heavy"]
fn k28_guard_patrol_census() {
    let (loader, descs) = k20::levels();
    let (mut pat, mut guards, mut levels, mut evs) = (0, 0, 0, 0);
    for (li, d) in descs.into_iter().enumerate() {
        let Ok(lv) = crate::common::load_blocking(&loader, d) else { continue };
        levels += 1;
        let (w, mut st) = scr::run(&lv, true);
        let mut seen = HashSet::new();
        seen.extend(patrolling(st.sched.cls.set.as_ref().unwrap()));
        let e0 = st.errors.len();
        let sent = scr::frames(&lv, &w, &mut st, 900, false, |s| seen.extend(patrolling(s)));
        let g = st.sched.cls.set.as_ref().unwrap().k6.len();
        if !seen.is_empty() || sent > 0 {
            eprintln!("level {li}: guards {g} patrolling {} class msgs {sent} errors {e0} -> {}", seen.len(), st.errors.len());
            for e in st.errors[e0..].iter().take(4) {
                eprintln!("    {}", format!("{e:?}").chars().take(160).collect::<String>());
            }
        }
        pat += seen.len();
        guards += g;
        evs += st.sched.cls.host.k28_events;
    }
    eprintln!("K28 census: {levels} levels, guards {guards}, patrolling {pat}, anim event calls 7..0x10 {evs}");
    assert!(levels >= 40);
    assert!(pat > 8, "patrolling guards {pat} (K27: 8)");
}

fn pp(r: &sly2_recomp::engine::splice::value::Ref, d: usize) -> String {
    use sly2_recomp::engine::splice::value::{list_items, Ref};
    match r {
        Ref::Pair(p) if d < 12 => format!("({})", list_items(Some(p.clone())).iter().map(|x| pp(x, d + 1)).collect::<Vec<_>>().join(" ")),
        Ref::Pair(_) => "(..)".into(),
        Ref::Symid(s) => format!("s{s:x}"),
        Ref::Bifk(b) => format!("b{b}"),
        Ref::S32(n) => format!("{n}"),
        Ref::F32(f) => format!("{f}f"),
        other => format!("<{}>", other.tagk()),
    }
}

#[test]
#[ignore]
fn k28_probe_class_msgs() {
    let (loader, descs) = k20::levels();
    for (li, d) in descs.into_iter().enumerate() {
        if li != 9 && li != 28 {
            continue;
        }
        let Ok(lv) = crate::common::load_blocking(&loader, d) else { continue };
        let (w, mut st) = scr::run(&lv, true);
        let e0 = st.errors.len();
        scr::frames(&lv, &w, &mut st, 300, true, |_| {});
        for e in &st.errors[e0..] {
            eprintln!("  err {e:?}");
        }
    }
}

/// Item 3: FUN_00200da0 argument kinds (DAT_0027a1d8 / DAT_0027a10c) and FUN_001e4530 durations
#[test]
fn k28_msg_args_and_durations() {
    use sly2_recomp::engine::classes::k28::ee_sub_200da0;
    use sly2_recomp::engine::splice::value::Ref;
    // message 4: first kind DAT_0027a10c[4] = 6 (int)
    assert!(matches!(ee_sub_200da0(4, &[7]).as_deref(), Some([Ref::S32(7)])));
    assert!(ee_sub_200da0(0x40, &[1]).is_none());
    let mut r = [0u32; 20];
    r[0] = 0x23;
    r[4] = 2.5f32.to_bits();
    assert_eq!(sly2_recomp::engine::classes::k22_seq::ee_sub_1e4530(&r), 2.5);
    r[1] = 1;
    assert_eq!(sly2_recomp::engine::classes::k22_seq::ee_sub_1e4530(&r), 0.0);
}

/// Item 2 probe: level 0's task handlers that mention sequence 1672 / 1667
#[test]
#[ignore]
fn k28_probe_level0_tasks() {
    use sly2_recomp::engine::game::vm_state::object_frame;
    use sly2_recomp::engine::splice::frame::lookup;
    use sly2_recomp::engine::splice::value::Ref;
    let (loader, descs) = k20::levels();
    let lv = crate::common::load_blocking(&loader, descs.into_iter().next().unwrap()).unwrap();
    let (w, st) = scr::run(&lv, false);
    for (i, o) in w.objects.iter().enumerate().filter(|(_, o)| matches!(o.class, 132 | 133) || o.oid == 1667 || o.oid == 1672) {
        let Some(fr) = object_frame(&st.frames, &w, i as u32) else { continue };
        for m in (0..0x40u32).chain(if o.oid == 1667 { vec![0xfdb, 0xfda] } else { vec![] }) {
            if let Some(Ref::Proc(p)) = lookup(&fr, m, true) {
                let b = p.body.clone().map(|b| pp(&Ref::Pair(b), 0)).unwrap_or_default();
                eprintln!("obj {i} oid {} class {} msg {m:#x}: {}", o.oid, o.class, b.chars().take(1500).collect::<String>());
            }
        }
    }
}

#[test]
#[ignore]
fn k28_probe_seq_records() {
    use sly2_recomp::engine::classes::Inst;
    let (loader, descs) = k20::levels();
    let lv = crate::common::load_blocking(&loader, descs.into_iter().next().unwrap()).unwrap();
    let (w, st) = scr::run(&lv, false);
    let set = st.sched.cls.set.as_ref().unwrap();
    for oid in [1667, 1672] {
        let i = w.objects.iter().position(|o| o.class == 114 && o.oid == oid).unwrap();
        for (j, x) in &set.more {
            if let Inst::C2d8c48(c) = x {
                if *j == i {
                    for r in &c.f_54 {
                        eprintln!("{oid}: {:x?}", &r[..]);
                    }
                }
            }
        }
    }
}

/// Item 2: the live level-0 boot path headless (J15 job post-load + task enter, then per frame: class tick,
/// job calls, job / class messages, sequence starts, script messages, world update): which sequences start
/// and whether the title sequence 1672 is reached.
fn level0_boot(frames: usize) -> (Vec<i32>, u32, Vec<(i32, i32)>) {
    use sly2_recomp::engine::audio::emitter::EmitterSystem;
    use sly2_recomp::engine::camera::Cm;
    use sly2_recomp::engine::game::job_flow::{self, JobSlots, SavedTask};
    use sly2_recomp::engine::game::save::{GameData, SaveVars};
    use sly2_recomp::engine::game::spawn::SpawnAliases;
    use sly2_recomp::engine::game::splice_host::*;
    use sly2_recomp::engine::game::vm_state::*;
    use sly2_recomp::engine::jobs::JobMgr;
    let (loader, descs) = k20::levels();
    let lv = crate::common::load_blocking(&loader, descs.into_iter().next().unwrap()).unwrap();
    let (w, mut st) = scr::run(&lv, false);
    let vars = SaveVars::from_block(&lv.header.block_3a18);
    let mut m = JobMgr::load(&w);
    job_flow::FUN_001638f8(&mut m, &w);
    let mut slots = JobSlots::default();
    job_flow::FUN_00161a58_case5(&mut m, &w, &mut slots, 0, &|_| SavedTask::default());
    let (mut em, mut data, mut al, mut rum) = (EmitterSystem::default(), GameData::default(), SpawnAliases::default(), Vec::new());
    let mut cm = Cm::new();
    let mut started = Vec::new();
    let tasks = |m: &JobMgr| m.tasks.iter().filter(|t| matches!(t.oid, 1664 | 1582)).map(|t| (t.oid, t.state)).collect::<Vec<_>>();
    for f in 0..frames {
        let t = (f + 1) as f32 / 60.0;
        {
            let l = &mut st.sched.cls;
            let set = l.set.as_mut().unwrap();
            set.g.t = t;
            set.g.dt = 1.0 / 60.0;
            set.frame(&mut l.host);
        }
        apply_job_calls(&mut st.sched, &mut m);
        st.sched.jobs = JobView::of(&m);
        job_messages(&mut st.sched, &m);
        st.sched.cls.drain_seq_starts();
        for n in &st.sched.cls.nodes {
            let oid = w.objects[n.owner as usize - 1].oid;
            if !started.contains(&oid) {
                eprintln!("frame {f}: sequence {oid} started; tasks {:?}", tasks(&m));
                started.push(oid);
            }
        }
        if f == 0 {
            for oid in [1665, 1666, 1667, 1672, 1582] {
                eprintln!("oid {oid}: {:?}", w.objects.iter().enumerate().filter(|(_, o)| o.oid == oid).map(|(i, o)| (i, o.class)).collect::<Vec<_>>());
            }
        }
        for (o, msg, a) in &st.sched.cls.host.k27_script_msgs {
            eprintln!("frame {f}: class msg obj {} oid {} msg {msg} args {a:x?}", o - 1, w.objects[*o as usize - 1].oid);
        }
        let e0 = st.errors.len();
        let r0 = st.sched.cls.recorded.len();
        let SpliceState { vm, sched, errors, frames, .. } = &mut st;
        let base = GameSpliceHost { t, cm: None, emitters: &mut em, vars: Some(&vars), data: &mut data };
        let level = LevelSpliceHost { base, mission: -1, aliases: &mut al, rumble: &mut rum };
        let mut h = PersistentHost { level, cm: Some(&mut cm), clk: Default::default(), world: Some(&*w), table: Some(&lv.header.level_table), sched };
        send_messages(vm, errors, frames, &mut h);
        tick_world_update(vm, errors, &mut h, t, 1.0 / 60.0);
        for r in &st.sched.cls.recorded[r0..] {
            eprintln!("frame {f}: recorded {:x?} obj {}", r.callee, r.obj);
        }
        for e in &st.errors[e0..] {
            eprintln!("frame {f}: error {}", format!("{e:?}").chars().take(200).collect::<String>());
        }
    }
    (started, st.sched.cls.host.k22_title_shows, tasks(&m))
}

#[test]
#[ignore]
fn k28_probe_level0_boot() {
    let (s, title, tasks) = level0_boot(60 * 90);
    eprintln!("started {s:?} title shows {title} tasks {tasks:?}");
}

/// Item 2: on the live boot path level 0's sequence 1667 sets task 1664 to 3 / task 1582 to 1 (action 0x11
/// FUN_00161f48 -> FUN_00164640), task 1582's message 0x15 starts the title sequence 1672 (action 0x44).
#[test]
fn k28_level0_boot_starts_title_sequence() {
    let (s, title, tasks) = level0_boot(60 * 5);
    assert_eq!(&s[..2], &[1667, 1672], "sequences started {s:?}");
    assert!(title >= 1, "title shows {title}");
    assert!(tasks.contains(&(1582, 1)), "tasks {tasks:?}");
}

/// Item 2: the emitter spawn path (FUN_0015bd20 -> FUN_0015b7e8 / FUN_0015b428): default template
/// (FUN_00159118) at speed 750, life 1, cone over the full grid, through a source frame.
#[test]
fn k28_emit_particles_pure() {
    use sly2_recomp::engine::classes::c2ceb78::ee_sub_159118;
    use sly2_recomp::engine::classes::k28_emit::*;
    use sly2_recomp::engine::physics::driven::GameRng;
    let mut t = ee_sub_159118();
    t[0x30 / 4] = 10f32.to_bits(); // shape kind 0 point (10, 0, 0)
    let mut rng = GameRng::default();
    let frame = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [100.0, 200.0, 300.0]];
    let ps = ee_sub_15b7e8(50, &t, 5.0, Some(&frame), &mut rng);
    assert_eq!(ps.len(), 50);
    for (p, v, b, d) in &ps {
        assert_eq!(*p, [110.0, 200.0, 300.0]);
        let s = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        assert!((s - 750.0).abs() < 1.0, "speed {s}");
        assert_eq!((*b, *d), (5.0, 6.0));
    }
    // FUN_0015ae38: 50 particles, grid of 200 -> one row of 50 columns
    let (cols, g) = ee_sub_15ae38(200, [0.0, 0.0], 50, &mut rng);
    assert_eq!(cols, 50);
    assert!((g[1] - std::f32::consts::TAU / 50.0).abs() < 1e-6);
}

/// Item 2: in normal play the level emitters (class 29) spawn class-28 systems with live particles.
#[test]
#[ignore = "heavy"]
fn k28_emitters_spawn_class28() {
    use sly2_recomp::engine::classes::Inst;
    let (loader, descs) = k20::levels();
    let (mut levels, mut sys_total, mut peak_total) = (0, 0, 0);
    for (li, d) in descs.into_iter().enumerate() {
        let Ok(lv) = crate::common::load_blocking(&loader, d) else { continue };
        let (w, mut st) = scr::run(&lv, false);
        let n29 = w.objects.iter().filter(|o| o.class == 29).count();
        if n29 == 0 {
            continue;
        }
        levels += 1;
        let mut peak = 0usize;
        scr::frames(&lv, &w, &mut st, 240, false, |s| {
            let live: usize = s.k21c.iter().map(|(_, x)| if let Inst::K21C28(c) = x { if c.removed { 0 } else { c.groups.iter().map(|g| g.parts.len()).sum() } } else { 0 }).sum();
            peak = peak.max(live);
        });
        let set = st.sched.cls.set.as_ref().unwrap();
        let sys = set.k21c.iter().filter(|(i, x)| *i >= w.objects.len() && matches!(x, Inst::K21C28(_))).count();
        eprintln!("level {li}: emitters {n29} emits {} systems {sys} peak particles {peak}", st.sched.cls.host.emits);
        sys_total += sys;
        peak_total += peak;
    }
    eprintln!("K28 emit census: {levels} levels with emitters, systems {sys_total}, peak particles {peak_total}");
    assert!(sys_total > 0 && peak_total > 0);
}
