//! WP-K27 tests: guard routines started the original's way, remaining hooks, sequence actions.
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::{ClassSet, Inst};
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

#[path = "engine_k20.rs"]
#[allow(dead_code)]
mod k20;

/// sequence record kinds 0x55 / 0x56 (guard routine start / stop) per level, guards per level
#[test]
#[ignore]
fn k27_probe_seq_routines() {
    let (loader, descs) = k20::levels();
    let mut tot = [0usize; 4];
    for (li, d) in descs.into_iter().enumerate() {
        let Ok(lv) = loader.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        let w = Arc::new(w);
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let s = ClassSet::build(&w, &mut h);
        let (mut a, mut b, mut seqs) = (0, 0, 0);
        for (_, x) in s.insts.iter().chain(s.more.iter()) {
            if let Inst::C2d8c48(c) = x {
                seqs += 1;
                for r in &c.f_54 {
                    a += (r[0] == 0x55) as usize;
                    b += (r[0] == 0x56) as usize;
                    if r[0] == 0x55 && li < 50 {
                        eprintln!("  lvl {li} 0x55 rec {:x?}", &r[..8]);
                    }
                }
            }
        }
        let g = s.k6.len();
        eprintln!("level {li}: seqs {seqs} 0x55 x{a} 0x56 x{b} guards {g}");
        tot[0] += a;
        tot[1] += b;
        tot[2] += g;
        tot[3] += seqs;
    }
    eprintln!("total 0x55 {} 0x56 {} guards {} seqs {}", tot[0], tot[1], tot[2], tot[3]);
}

/// guard property field 273 (routine, setter FUN_0020a540) per level: value, resolved class-113 object
#[test]
#[ignore]
fn k27_probe_field273() {
    let (loader, descs) = k20::levels();
    let (mut n, mut with, mut res) = (0, 0, 0);
    for (li, d) in descs.into_iter().enumerate() {
        let Ok(lv) = loader.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        for (i, o) in w.objects.iter().enumerate().filter(|o| (57..=67).contains(&o.1.class)) {
            n += 1;
            for p in sly2_recomp::engine::classes::props_of(&w, o.clone_of.unwrap_or(i)).unwrap_or(&[]) {
                if p.field == 273 {
                    with += 1;
                    let cand: Vec<(usize, u16)> = match &p.value {
                        sly2_recomp::engine::world::props::PropValue::Scalar(b) if b.len() >= 4 => {
                            let oid = i32::from_le_bytes([b[0], b[1], b[2], b[3]]);
                            w.find_oid(oid).iter().map(|&k| (k, w.objects[k].class)).collect()
                        }
                        _ => vec![],
                    };
                    res += cand.iter().any(|c| c.1 == 113) as usize;
                    if li < 12 {
                        eprintln!("lvl {li} guard {i} class {} 273 = {:?} -> {:?}", o.class, p.value, cand);
                    }
                }
            }
        }
    }
    eprintln!("guards {n}, with field 273 {with}, resolved to class 113 {res}");
}

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
}

/// what starts guard routines in the scripts: field-273 writes (set-o, setter FUN_0020a540) / method 317
#[test]
#[ignore]
fn k27_probe_script_routines() {
    let (loader, descs) = k20::levels();
    let (mut f273, mut pat, mut guards) = (0, 0, 0);
    for (li, d) in descs.into_iter().enumerate() {
        let Ok(lv) = loader.load_blocking(d) else { continue };
        let (_w, st) = scr::run(&lv, true);
        let set = st.sched.cls.set.as_ref().unwrap();
        let gi: Vec<usize> = set.k6.iter().map(|x| x.0).collect();
        let a = st.sched.objs.members.keys().filter(|k| k.1 == 273).count();
        let p = gi.iter().filter(|&&i| set.k6_guard(i).is_some_and(|g| g.brain.sub.code == 5 && g.brain.sub.routine.is_some())).count();
        let keys: Vec<_> = st.sched.objs.members.iter().filter(|k| k.0 .1 == 273).map(|(k, v)| (k.0, format!("{v:?}"))).take(4).collect();
        let rec: Vec<_> = st.sched.cls.recorded.iter().filter(|r| r.callee == 0x171cf8).map(|r| (r.obj, format!("{:?}", r.args))).collect();
        eprintln!("level {li}: guards {} field-273 writes {a} {keys:?} routines (code 5) {p} failed 171cf8 {rec:?}", gi.len());
        f273 += a;
        pat += p;
        guards += gi.len();
    }
    eprintln!("total guards {guards} field-273 writes {f273} code-5 {pat}");
}

/// class-113 routine objects with load-time method 594 (FUN_00170e68 -> FUN_00171cf8 on the guard)
#[test]
#[ignore]
fn k27_probe_field594() {
    let (loader, descs) = k20::levels();
    let (mut n, mut with, mut g) = (0, 0, 0);
    for (li, d) in descs.into_iter().enumerate() {
        let Ok(lv) = loader.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        for (i, o) in w.objects.iter().enumerate().filter(|o| o.1.class == 113) {
            n += 1;
            for p in sly2_recomp::engine::classes::props_of(&w, o.clone_of.unwrap_or(i)).unwrap_or(&[]) {
                if p.field == 594 {
                    with += 1;
                    let cand: Vec<(usize, u16)> = match &p.value {
                        sly2_recomp::engine::world::props::PropValue::Scalar(b) if b.len() >= 4 => {
                            let oid = i32::from_le_bytes([b[0], b[1], b[2], b[3]]);
                            w.find_oid(oid).iter().map(|&k| (k, w.objects[k].class)).collect()
                        }
                        _ => vec![],
                    };
                    g += cand.iter().any(|c| (56..=67).contains(&c.1)) as usize;
                    if li < 10 {
                        eprintln!("lvl {li} routine {i} 594 = {:?} -> {:?}", p.value, cand);
                    }
                }
            }
        }
    }
    eprintln!("routines {n}, with 594 {with}, resolved to guard {g}");
}

/// J10 driver (scripts + task enters), then 300 class frames: guards running routines (code 5), and
/// sequence guard actions (0x55) seen
#[test]
#[ignore]
fn k27_probe_seq_frames() {
    let (loader, descs) = k20::levels();
    let (mut g0, mut g1, mut guards) = (0, 0, 0);
    for (li, d) in descs.into_iter().enumerate() {
        let Ok(lv) = loader.load_blocking(d) else { continue };
        let (_w, mut st) = scr::run(&lv, true);
        let mut set = st.sched.cls.set.take().unwrap();
        let mut h = std::mem::take(&mut st.sched.cls.host);
        let code5 = |s: &ClassSet| s.k6.iter().filter(|x| s.k6_guard(x.0).is_some_and(|g| g.brain.sub.code == 5)).count();
        let a = code5(&set);
        let running = |s: &ClassSet| s.more.iter().filter(|x| matches!(&x.1, Inst::C2d8c48(c) if c.f_74 != 0)).count();
        let r0 = running(&set);
        let mut mx = 0;
        for _ in 0..300 {
            set.g.t += 1.0 / 60.0;
            set.g.dt = 1.0 / 60.0;
            set.frame(&mut h);
            mx = mx.max(code5(&set));
        }
        eprintln!("level {li}: guards {} code5 {a} -> max {mx}, seqs running {r0} -> {}", set.k6.len(), running(&set));
        g0 += a;
        g1 += mx;
        guards += set.k6.len();
    }
    eprintln!("total guards {guards} code5 {g0} -> {g1}");
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

/// script procs (per-slot frames) mentioning symids 441 (field 317), 399 (field 273), 214 (field 594),
/// 442 (field 318): the message they are bound to
#[test]
#[ignore]
fn k27_probe_script_syms() {
    use sly2_recomp::engine::splice::value::Ref;
    let (loader, descs) = k20::levels();
    let mut tot = std::collections::BTreeMap::new();
    for (li, d) in descs.into_iter().enumerate() {
        let Ok(lv) = loader.load_blocking(d) else { continue };
        let (_w, st) = scr::run(&lv, false);
        let mut seen = std::collections::HashSet::new();
        for slots in &st.frames {
            for f in slots {
                if !seen.insert(std::rc::Rc::as_ptr(f) as usize) {
                    continue;
                }
                for (sym, v) in &f.borrow().bindings {
                    let Ref::Proc(p) = v else { continue };
                    let body = p.body.clone().map(|b| pp(&Ref::Pair(b), 0)).unwrap_or_default();
                    for (tag, n) in [(" 441)", 317), (" 399 ", 273), (" 399)", 273), (" 214)", 594), (" 442)", 318)] {
                        let c = body.matches(tag).count();
                        if c > 0 {
                            *tot.entry((n, *sym)).or_insert(0) += c;
                            if li == 9 {
                                eprintln!("lvl {li} msg {sym:#x} field {n}: {}", body.chars().take(300).collect::<String>());
                            }
                        }
                    }
                }
            }
        }
    }
    eprintln!("(field, bound symid) -> count: {tot:?}");
}

/// Item 1: guard routines started the original's way, no test-side routine pick: level scripts + task-enter
/// handlers (method 317 -> FUN_00171cf8, J10) and the class-114 sequence actions 0x55 / 0x56 (FUN_001e1e30 ->
/// FUN_00171cf8 / FUN_00171d58, K27), then 300 class frames. Guards patrol and their bodies move.
#[test]
fn k27_guards_patrol_from_scripts_and_sequences() {
    let (loader, descs) = k20::levels();
    let (mut patrolling, mut moving, mut levels) = (0, 0, 0);
    for d in descs {
        let Ok(lv) = loader.load_blocking(d) else { continue };
        levels += 1;
        let (_w, mut st) = scr::run(&lv, true);
        let mut set = st.sched.cls.set.take().unwrap();
        let mut h = std::mem::take(&mut st.sched.cls.host);
        let mut pat = std::collections::HashSet::new();
        let mut moved = std::collections::HashSet::new();
        for _ in 0..300 {
            set.g.t += 1.0 / 60.0;
            set.g.dt = 1.0 / 60.0;
            set.frame(&mut h);
            pat.extend(set.k6.iter().map(|x| x.0).filter(|&i| set.k6_guard(i).is_some_and(|g| g.brain.sub.code == 5 && g.brain.sub.routine.is_some())));
            for (i, b) in set.k25_bodies() {
                if b.v.iter().any(|c| *c != 0.0) {
                    moved.insert(i);
                }
            }
        }
        patrolling += pat.len();
        moving += moved.len();
    }
    eprintln!("K27: {levels} levels, guards patrolling {patrolling}, moving bodies {moving}");
    assert!(levels >= 40);
    assert!(patrolling > 0, "no guard routine started");
    assert!(moving > 0, "no moving bodies");
}

/// Item 2: the controlled Bentley / Murray slot-69 force hooks (FUN_001f9b20 / FUN_00204108 / FUN_001fac30)
/// drive an E8 PO body (classes/k27.rs): finite, moving, FUN_001a62e8 / vtable +0x10c report it.
#[test]
fn k27_character_po_bodies() {
    use sly2_recomp::engine::classes::ClassHost;
    use sly2_recomp::engine::physics::query::CollisionWorld;
    use sly2_recomp::engine::player::Joy;
    let (loader, descs) = k20::levels();
    let (mut chars, mut moved, mut rest) = (0, 0, 0);
    for d in descs {
        let Ok(lv) = loader.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        let w = Arc::new(w);
        let cs: Vec<usize> = w.objects.iter().enumerate().filter(|x| matches!(x.1.class, 54 | 55)).map(|x| x.0).take(2).collect();
        if cs.is_empty() {
            continue;
        }
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        h.so = Some(Arc::new(CollisionWorld::build_so(&w)));
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        for &o in &cs {
            s.k18_set_active(Some(o), &mut h);
            let mut start = None;
            for f in 0..120 {
                s.g.dt = 1.0 / 60.0;
                s.g.t += 1.0 / 60.0;
                s.k18_feed_joy(Joy { angle: f as f32 * 0.01, mag: 1.0, held: 8, ..Default::default() });
                s.frame(&mut h);
                start.get_or_insert(h.k27_po.get(&o).map(|p| p.phys.pos));
            }
            s.k18_set_active(None, &mut h);
            let Some(p) = h.k27_po.get(&o).map(|p| p.phys.clone()) else { continue };
            chars += 1;
            assert!(p.pos.iter().chain(p.v.iter()).all(|x| x.is_finite()), "{p:?}");
            assert_eq!(h.ee_sub_1a62e8(o as u32 + 1), p.pos);
            assert_eq!(h.vcall_10c(o), p.v);
            eprintln!("  obj {o}: {:?} -> {:?} v {:?}", start.flatten(), p.pos, p.v);
            // resting on the level (the motor / hull contacts hold it) rather than falling
            rest += p.v.iter().all(|x| x.abs() < 1.0) as usize;
            if start.flatten().is_some_and(|s0| s0 != p.pos) {
                moved += 1;
            }
        }
        if chars >= 6 {
            break;
        }
    }
    eprintln!("K27: character PO bodies {chars}, moved {moved}, resting {rest}");
    assert!(chars > 0 && moved > 0 && rest > 0);
}

/// Item 2: slot-4 check (vtable +0x2c read from SCUS_973.16, WP-PUB2); the K8 pickups'
/// FUN_001c5098 vtables are overrides.
#[test]
fn k27_slot4_table() {
    use sly2_recomp::engine::classes::k27::slot4_is_base;
    assert!(!slot4_is_base(0x002d_5f68) && !slot4_is_base(0x002d_44c8));
    assert!(!slot4_is_base(0));
    let base = sly2_recomp::engine::world::class::CLASSES.iter().filter(|c| slot4_is_base(c.vtable)).count();
    eprintln!("K27: classes with the base slot 4: {base}");
    assert!(base > 50);
}

/// Item 2: FUN_00130d30 on a class-run ASEGA: speed +0x44 / +0x40, time per mode, dead handle untouched.
#[test]
fn k27_asega_seek_130d30() {
    use sly2_recomp::engine::classes::k25::AnimRt;
    let (loader, descs) = k20::levels();
    let mut done = 0;
    for d in descs {
        let Ok(lv) = loader.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        let w = Arc::new(w);
        let Some(ai) = (0..w.objects.len()).find(|&i| sly2_recomp::engine::playerdraw::aseg_of(&w, i).is_some_and(|a| a.length > 1.0)) else { continue };
        let len = sly2_recomp::engine::playerdraw::aseg_of(&w, ai).unwrap().length;
        let mut a = AnimRt::default();
        let h = a.ee_sub_12f9b8(&w, 0.0, 1.0, ai as u32 + 1, 0, 0);
        assert_ne!(h, 0);
        a.ee_sub_130d30(&w, 0.5, 2.0, h, 0);
        let x = &a.asegas[&h];
        assert_eq!((x.st.t, x.st.svt2), (0.5, 2.0));
        assert_eq!(x.st.svt, 2.0 * x.f48 * x.f4c);
        a.ee_sub_130d30(&w, 0.25, 2.0, h, 1);
        assert_eq!(a.asegas[&h].st.t, 0.75);
        a.ee_sub_130d30(&w, -0.5, 1.0, h, 2);
        assert_eq!(a.asegas[&h].st.t, len - 0.5);
        a.ee_sub_130b18(h);
        a.ee_sub_130d30(&w, 0.0, 3.0, h, 0);
        assert!(a.asegas.get(&h).is_none_or(|x| x.st.svt2 != 3.0));
        done += 1;
        if done >= 3 {
            break;
        }
    }
    assert!(done > 0);
}

/// Items 2 + 3 census: level scripts + 600 class frames per level; the sequence actions ported here
/// (0x11, 0x12, 0x2c / 0x2d, 0x32, 0x33) no longer reach the unported `k22_seq_action` hook; world
/// counters balanced >= 0; shakes / seeks / slot-4 base deliveries counted.
#[test]
fn k27_seq_actions_and_hooks_census() {
    let (loader, descs) = k20::levels();
    let (mut levels, mut fades, mut cine, mut shakes, mut seeks, mut handled, mut rumble) = (0, 0, 0, 0, 0, 0, 0);
    let mut kinds = std::collections::BTreeMap::new();
    for d in descs {
        let Ok(lv) = loader.load_blocking(d) else { continue };
        levels += 1;
        let (_w, mut st) = scr::run(&lv, true);
        let mut set = st.sched.cls.set.take().unwrap();
        let mut h = std::mem::take(&mut st.sched.cls.host);
        for _ in 0..600 {
            set.g.t += 1.0 / 60.0;
            set.g.dt = 1.0 / 60.0;
            set.frame(&mut h);
            shakes += h.k27_shake.take().is_some() as usize;
            rumble += std::mem::take(&mut h.k26.rumble).len();
            assert!(h.k27_cine.iter().all(|&c| c >= 0), "{:?}", h.k27_cine);
            cine += (h.k27_pad_lock != 0) as usize;
        }
        for a in &h.k22_seq_actions {
            *kinds.entry(a.2[0]).or_insert(0usize) += 1;
        }
        fades += h.k27_fades;
        seeks += h.k27_seeks;
        handled += h.k26.handled;
    }
    eprintln!("K27: {levels} levels, fades {fades}, cinematic frames {cine}, shakes {shakes}, rumble {rumble}, seeks {seeks}, slot-4 handled {handled}");
    eprintln!("K27: unported sequence actions by kind {kinds:x?}");
    assert!(levels >= 40);
    for k in [9, 0x11, 0x12, 0x2c, 0x2d, 0x32, 0x33, 0x55, 0x56] {
        assert!(!kinds.contains_key(&k), "kind {k:#x} still unported");
    }
}
