//! WP-J14: per-frame job / task update, game flow (title -> new game -> level 0), death / job reloads,
//! time-scale gadgets, save slot files.
#![allow(non_snake_case)]
use sly2_recomp::engine::disc::descriptor::{static_table, Descriptor};
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::game::clock::{self, Clock};
use sly2_recomp::engine::game::flow::{self, Dest, Post};
use sly2_recomp::engine::game::job_flow::{self, JobSlots, SavedTask, NEW_GAME_TASK};
use sly2_recomp::engine::game::save::{GameData, MemoryCard, SaveValue, SaveVars};
use sly2_recomp::engine::game::transition::{ee_str_oidPoCurrent_20e890, DiscLevelHost, LevelRequest, TransitionOutcome, TransitionState};
use sly2_recomp::engine::jobs::{Job, JobMgr, JobReload, Task};
use sly2_recomp::engine::ui::settings::SavePaths;
use sly2_recomp::engine::ui::title::{self, TitleStart};
use sly2_recomp::engine::world::LoadOptions;
use std::path::PathBuf;
use std::sync::Arc;

/// the flow mailbox is process-wide: serialize the tests that post
use crate::common::FLOW as SERIAL;

fn root() -> PathBuf {
    let r = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    if r.join("Sly 2 - Band of Thieves (USA).iso").exists() { r } else { PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/..")) }
}

fn iso() -> Option<Arc<IsoImage>> {
    IsoImage::open(root().join("Sly 2 - Band of Thieves (USA).iso")).ok().map(Arc::new)
}

/// job 0 with end task 1; task 2 depends on the end task and has a +0xc4 object.
fn mgr() -> JobMgr {
    let mut m = JobMgr { cur_row: -1, ..Default::default() };
    m.jobs.push(Job { oid: 100, start: Some(0), end: Some(1), tasks: vec![0, 1], f_9c: 480.0, f_a0: 720.0, ac: -1, b0: -1, b4: -1, ..Default::default() });
    let t = |oid| Task { oid, state: 0, spawn: -1, level: -1, c0: -1, f0: -1, f_5c: -1, job: Some(0), ..Default::default() };
    m.tasks = vec![t(1), t(2), Task { job: None, c4: Some(77), prereq: vec![1], ..t(3) }];
    m.tasks[0].deps = vec![1];
    m.tasks[1].prereq = vec![0];
    m.tasks[1].deps = vec![2];
    m
}

/// Item 1: job complete without an aseg -> end (event 0x20, autosave) and the next task requested.
#[test]
fn j14_job_complete_no_aseg() {
    let mut m = mgr();
    m.cur = Some(0);
    m.tasks[1].state = 2;
    m.FUN_00162f10(0);
    assert_eq!(m.autosave, 1);
    assert_eq!(m.jobs[0].c4, 0, "no aseg: the sequence state never leaves 0");
    // +0xc0 is only chosen by 0x163170 (aseg path); without it 0x162fd0 shows the +0xcc object (none)
    assert!(m.calls.iter().any(|c| c.0 == 0x00154a90 && c.1 == 0x20), "event 0x20 posted");
}

/// Item 1: aseg sequence 0 -> 1 (complete label) -> 3 (next task) -> next task state request.
#[test]
fn j14_job_complete_aseg_sequence() {
    let mut m = mgr();
    m.jobs[0].aseg_b8 = Some(500);
    m.jobs[0].aseg_bc = Some(0x379);
    m.jobs[0].b0 = 7;
    m.tasks[1].state = 3;
    m.playing = vec![(500, 7), (0x379, 0x30f)];
    m.FUN_00162f10(0);
    assert_eq!(m.jobs[0].c4, 1);
    assert_eq!(m.jobs[0].c0, Some(2), "first dependent of the end task with a +0xc4 object");
    assert_eq!(m.jobs[0].flags_40 & 0x102, 0x102);
    m.update(1.0, 0.016);
    assert_eq!(m.jobs[0].c4, 1, "label still playing");
    m.playing.retain(|p| p.0 != 500);
    m.update(1.1, 0.016);
    assert_eq!(m.jobs[0].c4, 3, "c0 set -> state 3");
    assert_eq!(m.autosave, 1);
    m.playing.clear();
    let s0 = m.tasks[2].state;
    m.update(1.2, 0.016);
    // 0x162fd0 -> FUN_00164640(task 2, 4): state 0, prerequisites done -> 0x164df0(t, 1)
    assert_ne!(m.tasks[2].state, s0, "next task requested");
}

/// Item 1: task +0x78 time limit fires once (msg 0x17), +0x80 object shown while active and no job.
#[test]
fn j14_task_update() {
    let mut m = mgr();
    m.tasks[2].state = 1;
    m.tasks[2].t_58 = 10.0;
    m.tasks[2].t_78 = 5.0;
    m.tasks[2].obj_80 = Some(42);
    let n17 = |m: &JobMgr| m.calls.iter().filter(|c| c.0 == 0x00165118).count();
    m.update(14.0, 0.5);
    assert_eq!(n17(&m), 0);
    m.update(15.2, 0.5);
    assert_eq!(n17(&m), 1);
    m.update(15.7, 0.5);
    assert_eq!(n17(&m), 1, "fires once");
    assert_eq!(m.toggles.last(), Some(&(42, -1, true)));
    m.cur = Some(0);
    m.update(16.0, 0.3);
    assert_eq!(m.toggles.last(), Some(&(42, -1, false)));
    // FUN_001645d0: an active prerequisite counts when its +0x6c is set
    m.tasks[0].state = 1;
    assert!(!m.FUN_001645d0(1));
    m.tasks[0].f_6c = 1;
    assert!(m.FUN_001645d0(1));
}

/// Item 1: FUN_00161478 job timer and the difficulty ramp past +0x9c.
#[test]
fn j14_playtime() {
    let mut m = mgr();
    m.cur = Some(0);
    m.jobs[0].a4 = 479.9;
    m.FUN_00161478(0.05, 0.0);
    assert!(m.calls.iter().all(|c| c.0 != 0x00154d88));
    m.FUN_00161478(0.1, 0.0);
    assert!(m.calls.iter().any(|c| c.0 == 0x00154d88 && f32::from_bits(c.1 as u32) == 0.1 / 240.0));
    m.FUN_00161478(0.1, 10.0);
    assert!((m.jobs[0].a4 - 480.05).abs() < 1e-3, "level kind >= 10: no timer");
}

/// Item 5: time-scale gadgets write clock slot 1; the game dt follows.
#[test]
fn j14_time_scale() {
    let mut c = Clock::default();
    clock::ee_sub_14d0b8(&mut c, 0.0, false);
    assert!(flow::apply_time_scale(&mut c, 0.25));
    assert!(!flow::apply_time_scale(&mut c, 0.25));
    assert!(!flow::apply_time_scale(&mut c, 0.0), "no player");
    clock::ee_sub_14d0b8(&mut c, 0.02, false);
    assert!((c.dt - 0.005).abs() < 1e-6, "{}", c.dt);
    assert!((c.dt_real - 0.02).abs() < 1e-6);
    assert!(flow::apply_time_scale(&mut c, 1.0));
    clock::ee_sub_14d0b8(&mut c, 0.04, false);
    assert!((c.dt - 0.02).abs() < 1e-6);
}

/// Item 4: death reload (current level, flags | 2) and job reload rows.
#[test]
fn j14_reload_requests() {
    let _flow = crate::common::flow_lock(); // T1: process-global flow state
    flow::set_static_table(Vec::new()); // T1: other modules of the binary may have set it
    let cur = Descriptor([1; 0x40]);
    let r = sly2_recomp::engine::player::p7_damage::Respawn { level: 0, spawn: 0x189, spawn_parent: -1, flags: 0x20, a: 0 };
    let q = flow::resolve(&flow::respawn_post(&r), None, Some(cur)).unwrap();
    assert_eq!((q.desc, q.flags, q.spawn, q.spawn_parent), (cur, 0x22, 0x189, -1));
    let mut m = mgr();
    m.cur = Some(0);
    m.cur_task = Some(1);
    let mut slots = JobSlots::default();
    let p = flow::job_post(&m, &JobReload { row: None, spawn: 5, spawn_parent: -1, flags: 4, a: 0x30a }, &mut slots);
    assert_eq!(p.dest, Dest::Current);
    assert_eq!((slots.job, slots.task), (100, 2));
    assert_eq!(flow::resolve(&p, None, Some(cur)).unwrap().flags, 6);
    assert!(flow::resolve(&Post { dest: Dest::Static(0x2a3828), spawn: 0, spawn_parent: -1, flags: 0, a: -1, slots: None }, None, Some(cur)).is_none());
}

/// Items 2 + 3: headless boot (title level, level-table row 1) -> title New Game -> level 0 loaded with
/// the spawn 0x189 object, restart task 0x62f -> save to a slot file -> load round trip.
#[test]
fn j14_boot_new_game_save_load() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let Some(iso) = iso() else { return eprintln!("ISO not found; skipped") };
    let elf = iso.read_file("SCUS_973.16").unwrap();
    let boot = static_table(&elf).unwrap().iter().skip(22).find(|d| d.size() != 0).copied().unwrap();
    let loader = LevelLoader::new(iso, ModOverlay::default());
    let mut st = TransitionState::default();
    let mut c = Clock::default();
    let mut host = DiscLevelHost { loader: &loader, options: LoadOptions::default(), world: None };
    st.request(LevelRequest::new(boot));
    assert!(matches!(ee_str_oidPoCurrent_20e890(&mut st, &mut c, &mut host), TransitionOutcome::Loaded { .. }));
    let lv = crate::common::load_blocking(&loader, boot).unwrap();
    let table = lv.header.level_table.clone();
    let vars = SaveVars::from_block(&lv.header.block_3a18);
    let paths = SavePaths::FUN_0020e3c0(&vars);
    let mut data = GameData::default();
    vars.ee_sub_1d7c80(&mut data);
    flow::take_posted();
    // title listed (the boot level's title screen): 1:1 New Game only hides the title
    assert_eq!(title::FUN_001ddb08(true, 2, &vars, &paths, &mut data), TitleStart::HideTitle);
    assert!(flow::take_posted().is_empty());
    // title not listed (FUN_001d9880(0x3da530) == 0): the new-game transition
    let t = title::FUN_001ddb08(false, 2, &vars, &paths, &mut data);
    assert_eq!(t, TitleStart::Transition { row: 0, flags: 0x48, movie: NEW_GAME_TASK });
    let posted = flow::take_posted();
    assert_eq!(posted.len(), 1);
    let slots = posted[0].slots.unwrap();
    assert_eq!(slots.restart, NEW_GAME_TASK);
    let r = flow::resolve(&posted[0], Some(&table), st.current).unwrap();
    assert_eq!(r.desc.0, table.slots[0].0);
    assert_eq!((r.flags, r.spawn, r.spawn_parent), (0x48, 0x189, -1));
    st.request(r);
    let o = ee_str_oidPoCurrent_20e890(&mut st, &mut c, &mut host);
    eprintln!("new game: {o:?}");
    let TransitionOutcome::Loaded { same_level, spawn_found, .. } = o else { panic!("{o:?}") };
    assert!(!same_level);
    assert!(spawn_found, "level 0 has the spawn 0x189 (393) object");
    assert_eq!(st.last_spawn, Some((393, -1)));
    let w = host.world.as_ref().unwrap();
    // level loaded: the job manager restores the restart task 0x62f from the slots
    let mut m = JobMgr::load(w);
    job_flow::FUN_001638f8(&mut m, w);
    let mut js = slots;
    job_flow::FUN_00161a58_case5(&mut m, w, &mut js, 0, &|_| SavedTask::default());
    eprintln!("level 0: {} jobs {} tasks; restart {:?}", m.jobs.len(), m.tasks.len(), m.restart_override.map(|t| m.tasks[t].oid));
    m.update(0.0, 1.0 / 60.0);
    // save -> slot file -> load
    let lv0 = crate::common::load_blocking(&loader, r.desc).unwrap();
    let vars0 = SaveVars::from_block(&lv0.header.block_3a18);
    let paths0 = SavePaths::FUN_0020e3c0(&vars0);
    let mut d0 = GameData::default();
    vars0.ee_sub_1d7c80(&mut d0);
    flow::FUN_00161478(12.5, &vars0, paths0.dt.as_deref(), &mut d0, Some(&mut m), 0.0);
    if let Some(p) = paths0.dt.as_deref() {
        assert_eq!(vars0.ee_sub_1d7ed8(p, &d0), SaveValue::Float(12.5));
    }
    let dir = std::env::temp_dir().join(format!("sly2_j14_{}", std::process::id()));
    let card = MemoryCard::new(&dir);
    flow::save_slot(&card, 1, Some(&vars0), &mut d0).unwrap();
    let back = flow::FUN_001ad320(&card, 1, &vars0, paths0.dt.as_deref()).unwrap();
    assert_eq!(back.to_bytes(), d0.to_bytes());
    // continue from the loaded blob ("dt" != 0, title not listed): episode transition
    let mut b = back.clone();
    if paths0.dt.is_some() {
        let t = title::FUN_00165e38(false, 2, &vars0, &paths0, &mut b, Some(0));
        assert_eq!(t, TitleStart::Transition { row: 0, flags: 0x40, movie: -1 });
        assert_eq!(flow::take_posted()[0].slots.unwrap().restart, -1);
    }
    let _ = std::fs::remove_dir_all(dir);
}
