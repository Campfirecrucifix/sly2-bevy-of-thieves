//! WP-J15 tests.

use sly2_recomp::engine::audio::emitter::EmitterSystem;
use sly2_recomp::engine::camera::Cm;
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::{ClassSet, Inst};
use sly2_recomp::engine::disc::descriptor::static_table;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::game::methods::cls::ClassLink;
use sly2_recomp::engine::game::save::{GameData, SaveVars};
use sly2_recomp::engine::game::spawn::SpawnAliases;
use sly2_recomp::engine::game::splice_host::*;
use sly2_recomp::engine::game::vm_state::*;
use sly2_recomp::engine::world::{LoadOptions, World};
use sly2_recomp::engine::game::boot::{BootHost, BootQueue, BOOT_LEVEL, BOOT_MOVIE};
use sly2_recomp::engine::game::clock::Clock;
use sly2_recomp::engine::game::flow::{self, Dest, Post, ZERO_DESC};
use sly2_recomp::engine::game::job_flow::{self, JobSlots, SavedTask};
use sly2_recomp::engine::game::transition::{ee_str_oidPoCurrent_20e890, DiscLevelHost, TransitionOutcome, TransitionState};
use sly2_recomp::engine::jobs::{JobMgr, VarOwner};
use sly2_recomp::engine::game::methods::MArg;

/// the flow mailbox / static table are process-wide
use crate::common::FLOW as SERIAL;
use std::path::PathBuf;
use std::sync::Arc;

fn root() -> PathBuf {
    let r = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    if r.join("Sly 2 - Band of Thieves (USA).iso").exists() { r } else { PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/..")) }
}

fn iso() -> Option<Arc<IsoImage>> {
    IsoImage::open(root().join("Sly 2 - Band of Thieves (USA).iso")).ok().map(Arc::new)
}

#[test]
#[ignore]
fn j15_explore_boot() {
    let Some(iso) = iso() else { return eprintln!("ISO not found; skipped") };
    let elf = iso.read_file("SCUS_973.16").unwrap();
    let row: usize = std::env::var("J15_ROW").ok().and_then(|v| v.parse().ok()).unwrap_or(22);
    let boot = static_table(&elf).unwrap()[row];
    let loader = LevelLoader::new(iso, ModOverlay::default());
    let lv = crate::common::load_blocking(&loader, boot).unwrap();
    let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
    let mut counts = std::collections::BTreeMap::new();
    for o in &w.objects {
        *counts.entry(o.class).or_insert(0) += 1;
    }
    eprintln!("classes {counts:?}");
    let vars = SaveVars::from_block(&lv.header.block_3a18);
    let mut st = SpliceState::default();
    st.enter_level(1);
    let mut host = EngineHost { world: Some(w.clone()), ..Default::default() };
    let mut set = ClassSet::build(&w, &mut host);
    set.post_load(&mut host);
    st.sched.cls = ClassLink { set: Some(set), host, ..Default::default() };
    let (mut em, mut data, mut al, mut rum) = (EmitterSystem::default(), GameData::default(), SpawnAliases::default(), Vec::new());
    let mut cm = Cm::new();
    {
        let base = GameSpliceHost { t: 0.0, cm: None, emitters: &mut em, vars: Some(&vars), data: &mut data };
        let level = LevelSpliceHost { base, mission: -1, aliases: &mut al, rumble: &mut rum };
        let mut h = PersistentHost { level, cm: Some(&mut cm), clk: Default::default(), world: Some(&w), table: Some(&lv.header.level_table), sched: &mut st.sched };
        let (rep, frames) = sly2_recomp::engine::game::scripts::run_level_scripts_framed(&mut st.vm, &lv.data, &w, &mut h);
        st.frames = frames;
        eprintln!("scripts: {} entries {}/{} ok", rep.entries, rep.ok, rep.exprs);
        for e in rep.errors.iter().take(30) {
            eprintln!("  err {e:?}");
        }
    }
    for f in 1..=120 {
        let t = f as f32 / 60.0;
        let SpliceState { vm, sched, errors, frames, .. } = &mut st;
        let base = GameSpliceHost { t, cm: None, emitters: &mut em, vars: Some(&vars), data: &mut data };
        let level = LevelSpliceHost { base, mission: -1, aliases: &mut al, rumble: &mut rum };
        let mut h = PersistentHost { level, cm: Some(&mut cm), clk: Default::default(), world: Some(&w), table: Some(&lv.header.level_table), sched };
        send_messages(vm, errors, frames, &mut h);
        tick_world_update(vm, errors, &mut h, t, 1.0 / 60.0);
    }
    for e in st.errors.iter().take(30) {
        eprintln!("tick err {e:?}");
    }
    let l = &st.sched.cls;
    let s0 = l.set.as_ref().unwrap();
    for (i, inst) in s0.insts.iter().chain(s0.more.iter()).chain(s0.k4.iter()).chain(s0.k5.iter()).chain(s0.k6.iter()) {
        if let Inst::C2d8c48(c) = inst {
            eprintln!("seq obj {i} oid {} running {} recs {}", w.objects[*i].oid, c.f_74, c.f_54.len());
            for r in &c.f_54 {
                eprintln!("    {:x?}", r);
            }
        }
    }
    eprintln!("nodes {:?}", l.nodes);
    for r in l.recorded.iter().take(40) {
        eprintln!("recorded {r:?}");
    }
}

fn sx(r: &sly2_recomp::engine::splice::value::Ref, out: &mut String, depth: usize) {
    use sly2_recomp::engine::splice::value::Ref;
    if depth > 60 {
        out.push_str("...");
        return;
    }
    match r {
        Ref::Pair(p) => {
            out.push('(');
            let mut cur = Some(p.clone());
            let mut first = true;
            while let Some(c) = cur {
                if !first {
                    out.push(' ');
                }
                first = false;
                let b = c.borrow();
                sx(&b.car, out, depth + 1);
                cur = b.next.clone();
            }
            out.push(')');
        }
        Ref::Symid(s) => out.push_str(&format!("s{s:x}")),
        Ref::S32(n) => out.push_str(&format!("{n}")),
        Ref::F32(f) => out.push_str(&format!("{f}f")),
        Ref::Bifk(b) => out.push_str(&format!("bif{b}")),
        Ref::Tag(t) => out.push_str(&format!("t{t}")),
        Ref::Bool(b) => out.push_str(if *b { "#t" } else { "#f" }),
        other => out.push_str(&format!("{other:?}")),
    }
}

#[test]
#[ignore]
fn j15_dump_scripts() {
    let Some(iso) = iso() else { return eprintln!("ISO not found; skipped") };
    let elf = iso.read_file("SCUS_973.16").unwrap();
    let row: usize = std::env::var("J15_ROW").ok().and_then(|v| v.parse().ok()).unwrap_or(22);
    let boot = static_table(&elf).unwrap()[row];
    let loader = LevelLoader::new(iso, ModOverlay::default());
    let lv = crate::common::load_blocking(&loader, boot).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    let mut out = String::new();
    for (ei, e) in w.entries.iter().enumerate() {
        let off = e.start_offset as usize + 12;
        let objs: Vec<(i32, u16)> = e.objects.iter().take(8).map(|&o| (w.objects[o].oid, w.objects[o].class)).collect();
        out.push_str(&format!("== entry {ei} objs {objs:?}\n"));
        if let Some(Ok((blk, _))) = lv.data.get(off..).map(|d| sly2_recomp::engine::splice::serialize::read_script_block(d, e.sections.ext)) {
            for (xi, x) in blk.iter().enumerate() {
                if let Some(p) = x {
                    let mut s = String::new();
                    sx(&p.borrow().car, &mut s, 0);
                    out.push_str(&format!("[{xi}] {s}\n"));
                }
            }
        }
    }
    std::fs::write(format!("{}/j15_script_row{row}.txt", std::env::temp_dir().display()), out).unwrap();
}

#[test]
#[ignore]
fn j15_dump_seq_props() {
    use sly2_recomp::engine::world::objects::Payload;
    use sly2_recomp::engine::world::objects::LoadData;
    let Some(iso) = iso() else { return eprintln!("ISO not found; skipped") };
    let elf = iso.read_file("SCUS_973.16").unwrap();
    let row: usize = std::env::var("J15_ROW").ok().and_then(|v| v.parse().ok()).unwrap_or(25);
    let boot = static_table(&elf).unwrap()[row];
    let loader = LevelLoader::new(iso, ModOverlay::default());
    let lv = crate::common::load_blocking(&loader, boot).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    let mut out = String::new();
    for (i, o) in w.objects.iter().enumerate() {
        if o.class != 114 {
            continue;
        }
        out.push_str(&format!("== obj {i} oid {} entry {}\n", o.oid, o.entry));
        if let Payload::Loaded(LoadData::Lo(props)) = &o.payload {
            for p in props {
                out.push_str(&format!("  f{} {:?}\n", p.field, p.value));
            }
        }
    }
    std::fs::write(format!("{}/j15_seq_row{row}.txt", std::env::temp_dir().display()), out).unwrap();
}

#[derive(Default)]
struct TestBoot {
    movies: Vec<u32>,
    posts: Vec<Post>,
    card: i32,
    flag: bool,
}

impl BootHost for TestBoot {
    fn ee_sub_1b3a48(&mut self, desc: u32) {
        self.movies.push(desc);
    }
    fn set_2f67b8(&mut self) {
        self.flag = true;
    }
    fn post(&mut self, p: Post) {
        self.posts.push(p);
    }
    fn card_state(&self) -> i32 {
        self.card
    }
    fn ee_sub_1acc38(&mut self, s: i32) {
        self.card = s;
    }
}

/// FUN_001a4e08 / FUN_001a4e80 order (static table 0x2a3268 from the ELF).
#[test]
fn j15_boot_queue_entries() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let Some(iso) = iso() else { return eprintln!("ISO not found; skipped") };
    flow::set_static_table(static_table(&iso.read_file("SCUS_973.16").unwrap()).unwrap());
    let mut q = BootQueue::default();
    let mut h = TestBoot::default();
    q.FUN_001a4e08(&mut h);
    assert_eq!(q.entries, vec![[0, BOOT_MOVIE, 0, 0, 0, 0, 0], [5, 0, 0, 0, 0, 0, 0], [2, 0, BOOT_LEVEL, 0x189, 1, 0, 0]]);
    assert_eq!(q.next, 1);
    assert_eq!(h.movies, vec![BOOT_MOVIE]);
    assert!(!h.flag);
    assert_eq!(h.posts, vec![Post { dest: Dest::Static(ZERO_DESC), spawn: 0x189, spawn_parent: -1, flags: 0, a: -1, slots: None }]);
    // after the size-0 transition: card check (idle -> 1) and level 0
    q.FUN_001a4e80(&mut h);
    assert_eq!(h.card, 1);
    assert_eq!(h.posts[1], Post { dest: Dest::Static(BOOT_LEVEL), spawn: 0x189, spawn_parent: -1, flags: 1, a: 0x30a, slots: None });
    assert!(!q.FUN_001a4e70());
    q.FUN_001a4e80(&mut h);
    assert_eq!(h.posts.len(), 2);
    // card manager busy: no state change
    let mut h2 = TestBoot { card: 3, ..Default::default() };
    q.FUN_001a4e08(&mut h2);
    q.FUN_001a4e80(&mut h2);
    assert_eq!(h2.card, 3);
    // capacity 0x10
    let mut q = BootQueue::default();
    for _ in 0..0x12 {
        q.FUN_001a50f8();
    }
    assert_eq!(q.entries.len(), 0x10);
    assert!(flow::static_desc(0x2a3828).is_some_and(|d| d.size() != 0), "hideout descriptor 0x2a3828 resolves");
    assert!(flow::static_desc(0x2a3829).is_none());
    assert_eq!(flow::static_desc(ZERO_DESC).map(|d| d.size()), Some(0));
}

/// Headless 1:1 boot: FUN_001a4e08 -> intro movie + size-0 transition -> FUN_001a4e80 -> card check + level 0
/// (static row 25 = level-table slot 0) at spawn 393, through flow::resolve / FUN_0020e890.
#[test]
fn j15_boot_to_level0() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let Some(iso) = iso() else { return eprintln!("ISO not found; skipped") };
    let st_table = static_table(&iso.read_file("SCUS_973.16").unwrap()).unwrap();
    flow::set_static_table(st_table.clone());
    let loader = LevelLoader::new(iso, ModOverlay::default());
    let mut st = TransitionState::default();
    let mut c = Clock::default();
    let mut host = DiscLevelHost { loader: &loader, options: LoadOptions::default(), world: None };
    let mut q = BootQueue::default();
    let mut bh = TestBoot::default();
    q.FUN_001a4e08(&mut bh);
    let mut table = None;
    let mut loads = Vec::new();
    for _ in 0..4 {
        for p in std::mem::take(&mut bh.posts) {
            st.request(flow::resolve(&p, table.as_ref(), st.current).expect("boot request resolves"));
        }
        let o = ee_str_oidPoCurrent_20e890(&mut st, &mut c, &mut host);
        if let TransitionOutcome::Loaded { reset_only, spawn_found, .. } = o {
            loads.push((st.current.unwrap(), reset_only, spawn_found));
            if reset_only {
                q.FUN_001a4e80(&mut bh);
            } else {
                table = Some(crate::common::load_blocking(&loader, st.current.unwrap()).unwrap().header.level_table);
            }
        }
    }
    eprintln!("boot loads {:?}", loads.iter().map(|l| (l.0.lsn(), l.1, l.2)).collect::<Vec<_>>());
    assert_eq!(bh.movies, vec![BOOT_MOVIE]);
    assert_eq!(bh.card, 1);
    assert_eq!(loads.len(), 2);
    assert!(loads[0].1, "first transition: zero descriptor (movie only)");
    assert_eq!(loads[1].0, st_table[25]);
    assert!(!loads[1].1 && loads[1].2, "level 0 loaded, spawn 0x189 found");
    let table = table.unwrap();
    assert!(table.slots[0].same_file(&st_table[25]), "static row 25 = level-table slot 0 (the new-game level)");
    assert_eq!(st.last_spawn, Some((393, -1)));
    assert_eq!(st.request.unwrap().flags, 1);
}

/// 0x1657d0 messages 0x16 / 0x15 reach the task object (level 0, case 5 with new-game slots).
#[test]
fn j15_task_state_message() {
    let Some(iso) = iso() else { return eprintln!("ISO not found; skipped") };
    let elf = iso.read_file("SCUS_973.16").unwrap();
    let lv = LevelLoader::new(iso, ModOverlay::default()).load_blocking(static_table(&elf).unwrap()[25]).unwrap();
    let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
    let vars = SaveVars::from_block(&lv.header.block_3a18);
    let mut st = SpliceState::default();
    st.enter_level(1);
    let mut host = EngineHost { world: Some(w.clone()), ..Default::default() };
    let mut set = ClassSet::build(&w, &mut host);
    set.post_load(&mut host);
    st.sched.cls = ClassLink { set: Some(set), host, ..Default::default() };
    let (mut em, mut data, mut al, mut rum) = (EmitterSystem::default(), GameData::default(), SpawnAliases::default(), Vec::new());
    let mut cm = Cm::new();
    {
        let base = GameSpliceHost { t: 0.0, cm: None, emitters: &mut em, vars: Some(&vars), data: &mut data };
        let level = LevelSpliceHost { base, mission: -1, aliases: &mut al, rumble: &mut rum };
        let mut h = PersistentHost { level, cm: Some(&mut cm), clk: Default::default(), world: Some(&w), table: Some(&lv.header.level_table), sched: &mut st.sched };
        let (_, frames) = sly2_recomp::engine::game::scripts::run_level_scripts_framed(&mut st.vm, &lv.data, &w, &mut h);
        st.frames = frames;
    }
    let mut m = JobMgr::load(&w);
    job_flow::FUN_001638f8(&mut m, &w);
    let mut slots = JobSlots::default();
    job_flow::FUN_00161a58_case5(&mut m, &w, &mut slots, 0, &|_| SavedTask::default());
    eprintln!("task msgs {:?}", m.task_msgs.iter().map(|&(t, k)| (m.tasks[t].oid, m.tasks[t].state, k)).collect::<Vec<_>>());
    st.sched.jobs = JobView::of(&m);
    job_messages(&mut st.sched, &m);
    let n15 = st.sched.msgs.iter().filter(|x| x.1 == 0x15).count();
    assert_eq!(n15, m.task_msgs.iter().filter(|x| x.1 == 0x15).count());
    for (o, k) in &st.sched.msgs {
        assert!(*k != 0x15 || w.objects[*o as usize].class == 133, "message 0x15 goes to a task (class 133)");
    }
    {
        let SpliceState { vm, sched, errors, frames, .. } = &mut st;
        let base = GameSpliceHost { t: 0.0, cm: None, emitters: &mut em, vars: Some(&vars), data: &mut data };
        let level = LevelSpliceHost { base, mission: -1, aliases: &mut al, rumble: &mut rum };
        let mut h = PersistentHost { level, cm: Some(&mut cm), clk: Default::default(), world: Some(&w), table: Some(&lv.header.level_table), sched };
        send_messages(vm, errors, frames, &mut h);
        eprintln!("errors {:?}", errors.iter().take(10).collect::<Vec<_>>());
    }
    // a second call delivers nothing new
    job_messages(&mut st.sched, &m);
    assert!(st.sched.msgs.iter().all(|x| x.1 != 0x15));
    let l = &mut st.sched.cls;
    let started = l.drain_seq_starts();
    let started_oids: Vec<i32> = l.nodes.iter().map(|n| &w.objects[n.owner as usize - 1]).inspect(|o| assert_eq!(o.class, 114)).map(|o| o.oid).collect();
    // task 1664 (state 1 at load) -> its message-0x15 handler starts sequence 1667 (FUN_001dfd58)
    assert_eq!(started_oids, vec![1667]);
    eprintln!("seq starts {started} nodes {:?} recorded {:?}", l.nodes, l.recorded.iter().filter(|r| r.callee == 0x1dfd58).count());
}

#[test]
#[ignore]
fn j15_dump_task_props() {
    use sly2_recomp::engine::world::loads_b4::B4Load;
    use sly2_recomp::engine::world::objects::{LoadData, Payload};
    let Some(iso) = iso() else { return };
    let elf = iso.read_file("SCUS_973.16").unwrap();
    let lv = LevelLoader::new(iso, ModOverlay::default()).load_blocking(static_table(&elf).unwrap()[25]).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    for o in w.objects.iter().filter(|o| o.class == 132).take(2) {
        if let Payload::Loaded(LoadData::B4(b)) = &o.payload {
            eprintln!("job {} {:?}", o.oid, b); if let B4Load::L1635f8 { props, blob, .. } = &**b {
                eprintln!("task {} props {:?} blob {}", o.oid, props, blob.len());
            }
        }
    }
}

/// Task / job save-variable handles (kind-0x16 props 1295..1299 / 1260..1263) resolve in the level's save
/// table; FUN_00258200 writes (`flush_vars`) are read back by FUN_00164270 / FUN_00162780.
#[test]
fn j15_task_save_vars() {
    let Some(iso) = iso() else { return eprintln!("ISO not found; skipped") };
    let elf = iso.read_file("SCUS_973.16").unwrap();
    let lv = LevelLoader::new(iso, ModOverlay::default()).load_blocking(static_table(&elf).unwrap()[25]).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    let vars = SaveVars::from_block(&lv.header.block_3a18);
    let mut data = GameData::default();
    let mut m = JobMgr::load(&w);
    let t = m.tasks.iter().position(|t| t.oid == 1664).unwrap();
    assert_eq!(m.tasks[t].save, [vec![204, 111], vec![204, 193], vec![204, 112], vec![204, 194], vec![204, 195]]);
    for p in &m.tasks[t].save {
        assert!(vars.ee_sub_1d7da0(p).is_some(), "task path {p:?} resolves");
    }
    let j = m.jobs.iter().position(|j| j.oid == 1671).unwrap();
    assert_eq!(m.jobs[j].save[1], vec![110, 111]);
    assert_eq!(m.jobs[j].save[3], vec![110, 113]);
    assert_eq!(job_flow::saved_task(&m, t, &vars, &data), SavedTask::default());
    m.vars.push((VarOwner::Task(t), 0xe0, 0x15, 2));
    m.vars.push((VarOwner::Task(t), 0xe8, 0x17, 1));
    m.vars.push((VarOwner::Job(j), 0xd0, 0x15, 3));
    m.vars.push((VarOwner::Job(j), 0xe0, 0x16, 12.5f32.to_bits() as i32));
    job_flow::flush_vars(&mut m, &vars, &mut data);
    assert!(m.vars.is_empty());
    assert_eq!(job_flow::saved_task(&m, t, &vars, &data), SavedTask { state: 2, e8: 1 });
    let js = job_flow::saved_job(&m, j, &vars, &data);
    assert_eq!((js.n_7c, js.a4), (3, 12.5));
    // FUN_00164270 with the saved state: state 2 kept (done), +0xe8 set
    job_flow::FUN_00164270(&mut m, t, SavedTask { state: 2, e8: 0 });
    assert_eq!(m.tasks[t].state, 2);
}

/// Methods 1258 (0x162660) / 1284 (0x165118) recorded by the class link reach the job manager.
#[test]
fn j15_job_methods() {
    let Some(iso) = iso() else { return eprintln!("ISO not found; skipped") };
    let elf = iso.read_file("SCUS_973.16").unwrap();
    let lv = LevelLoader::new(iso, ModOverlay::default()).load_blocking(static_table(&elf).unwrap()[25]).unwrap();
    let w = World::load(&lv, LoadOptions::default()).unwrap();
    let mut m = JobMgr::load(&w);
    let mut sched = Schedule::default();
    let (jo, to) = (m.jobs[0].obj as u32, m.tasks[0].obj as u32);
    assert!(sched.cls.call(0x10eda8, 0x162660, jo, &[], vec![MArg::I32(77), MArg::I32(1), MArg::I32(2)]));
    assert!(sched.cls.call(0x10efb0, 0x165118, to, &[], vec![MArg::Word(5)]));
    apply_job_calls(&mut sched, &mut m);
    assert_eq!((m.jobs[0].ac, m.jobs[0].b0, m.jobs[0].b4), (77, 1, 2));
    assert!(sched.cls.job_calls.is_empty());
    // 0x165118 only messages a running task (0x1637e8)
    let n = m.calls.iter().filter(|c| c.0 == 0x165118).count();
    m.tasks[0].state = 1;
    m.cur = m.tasks[0].job;
    sched.cls.call(0x10efb0, 0x165118, to, &[], vec![MArg::Word(5)]);
    apply_job_calls(&mut sched, &mut m);
    assert!(m.calls.iter().filter(|c| c.0 == 0x165118).count() > n);
}

/// FUN_00226550 (C2dc018 slot 6) step order: paused vs running, camera only with dt_real > 0.
#[test]
fn j15_world_update_order() {
    use sly2_recomp::engine::game::world_update::*;
    struct Rec(Vec<WorldStep>);
    impl WorldUpdateHost for Rec {
        fn step(&mut self, s: WorldStep) {
            self.0.push(s);
        }
    }
    use WorldStep::*;
    let mut r = Rec(vec![]);
    ee_sub_226550(&mut r, 0.0, 0.0);
    assert_eq!(r.0, vec![Scripts, Proximity, Paused, Card, Music, Slots, PoolStats]);
    let mut r = Rec(vec![]);
    ee_sub_226550(&mut r, 1.0 / 60.0, 1.0 / 60.0);
    assert_eq!(r.0.iter().filter(|s| **s == Scripts).count(), 3);
    assert_eq!(&r.0[..3], &[Scripts, Proximity, PadObjects]);
    assert!(r.0.contains(&Camera) && !r.0.contains(&Paused));
    assert!(paused_flag(PausedPass::Update, 0x102) && !paused_flag(PausedPass::Update, 0x100));
    assert!(paused_flag(PausedPass::Paused, 0x104) && !paused_flag(PausedPass::Paused, 0x124));
    assert_eq!(FUN_00113a20(), 0x2baafc);
}

/// FUN_00147f40 proximity enter / leave and FUN_0020f7e0 slot offsets + tick sounds.
#[test]
fn j15_world_update_kernels() {
    use sly2_recomp::engine::game::world_update::*;
    #[derive(Default)]
    struct H {
        ev: Vec<(u32, bool)>,
    }
    impl ProxHost for H {
        fn pos(&self, obj: u32) -> [f32; 3] {
            [obj as f32 * 100.0, 0.0, 0.0]
        }
        fn enter(&mut self, obj: u32) {
            self.ev.push((obj, true));
        }
        fn leave(&mut self, obj: u32) {
            self.ev.push((obj, false));
        }
    }
    let mut p = ProxList { list: vec![ProxEntry { obj: 1, kind: 0, mask: 1, radius: 150.0 }, ProxEntry { obj: 5, kind: 0, mask: 1, radius: 150.0 }], ..Default::default() };
    let mut h = H::default();
    p.FUN_00147f40(&mut h, 1, [0.0; 3], None);
    assert_eq!(h.ev, vec![(1, true)]);
    assert_eq!(p.active, vec![1]);
    // camera moves far: 1 leaves, 5 enters
    p.FUN_00147f40(&mut h, 1, [500.0, 0.0, 0.0], None);
    p.FUN_00147f40(&mut h, 1, [500.0, 0.0, 0.0], None);
    assert!(h.ev.contains(&(1, false)) && h.ev.contains(&(5, true)));
    // timers
    let mut alarm = (1, 0.0);
    let mut calls = vec![];
    FUN_0019f120(&mut alarm, 5.0, &mut |on| calls.push(on));
    assert!(calls.is_empty());
    FUN_0019f120(&mut alarm, 6.5, &mut |on| calls.push(on));
    assert_eq!((calls, alarm), (vec![0], (0, 6.5)));
    assert!(FUN_00156728(1.0, 0.0, 1.5) && !FUN_00156728(-1.0, 0.0, 9.0));
    struct S {
        off: [f32; 5],
        snd: Vec<u32>,
    }
    impl SlotHost for S {
        fn shown(&self, obj: u32) -> bool {
            obj != 0
        }
        fn offset(&mut self, obj: u32) -> &mut f32 {
            &mut self.off[obj as usize]
        }
        fn sound(&mut self, id: u32) {
            self.snd.push(id);
        }
    }
    let mut slots = [Slot { obj: 1, v: 0.0, reset: true }, Slot { obj: 2, v: 0.0, reset: true }, Slot::default(), Slot { obj: 4, v: 0.0, reset: true }];
    let mut s = S { off: [9.0; 5], snd: vec![] };
    let mut shown = 0;
    FUN_0020f7e0(&mut slots, &mut s, 10.0, 1.0 / 60.0, 13.5, &mut shown);
    assert_eq!((s.off[1], s.off[2], s.off[4]), (0.0, 65.0, 130.0));
    assert!(slots[2].reset);
    assert_eq!((shown, s.snd.clone()), (3, vec![0x30]));
    FUN_0020f7e0(&mut slots, &mut s, 10.6, 1.0 / 60.0, 13.5, &mut shown);
    assert_eq!((shown, s.snd), (2, vec![0x30, 0x31]));
}
