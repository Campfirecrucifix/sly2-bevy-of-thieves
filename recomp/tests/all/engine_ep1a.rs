//! WP-EP1a tests: Episode 1 boot path (headless subset): boot queue -> level 0 title sequence -> New Game
//! (title hidden, sequence 1672 runs on) -> prologue job flow; Paris hub (level-table row 2) loads with guards.
use std::sync::Arc;

use crate::common as k20;

use sly2_recomp::engine::audio::emitter::EmitterSystem;
use sly2_recomp::engine::camera::Cm;
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::ClassSet;
use sly2_recomp::engine::game::job_flow::{self, JobSlots, SavedTask};
use sly2_recomp::engine::game::methods::cls::ClassLink;
use sly2_recomp::engine::game::save::{GameData, SaveVars};
use sly2_recomp::engine::game::scripts::run_level_scripts_framed;
use sly2_recomp::engine::game::spawn::SpawnAliases;
use sly2_recomp::engine::game::splice_host::*;
use sly2_recomp::engine::game::vm_state::*;
use sly2_recomp::engine::jobs::JobMgr;
use sly2_recomp::engine::world::{LoadOptions, World};

/// Live per-frame order of the app (tick_classes -> lend -> job calls / messages -> world update).
pub struct Run {
    pub lv: sly2_recomp::engine::disc::LoadedLevel,
    pub w: Arc<World>,
    pub st: SpliceState,
    pub m: JobMgr,
    pub f: usize,
    pub started: Vec<i32>,
    pub log: bool,
    vars: SaveVars,
    em: EmitterSystem,
    data: GameData,
    al: SpawnAliases,
    rum: Vec<RumbleRequest>,
    cm: Cm,
}

impl Run {
    pub fn new(row: usize) -> Run {
        let (loader, descs) = k20::levels();
        let lv = crate::common::load_blocking(&loader, descs[row]).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let vars = SaveVars::from_block(&lv.header.block_3a18);
        let mut st = SpliceState::default();
        st.enter_level(1);
        let mut host = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut set = ClassSet::build(&w, &mut host);
        set.post_load(&mut host);
        st.sched.cls = ClassLink { set: Some(set), host, ..Default::default() };
        let mut r = Run { m: JobMgr::load(&w), lv, w, st, f: 0, started: Vec::new(), log: false, vars, em: Default::default(), data: Default::default(), al: Default::default(), rum: Vec::new(), cm: Cm::new() };
        {
            let base = GameSpliceHost { t: 0.0, cm: None, emitters: &mut r.em, vars: Some(&r.vars), data: &mut r.data };
            let level = LevelSpliceHost { base, mission: -1, aliases: &mut r.al, rumble: &mut r.rum };
            let mut h = PersistentHost { level, cm: Some(&mut r.cm), clk: Default::default(), world: Some(&r.w), table: Some(&r.lv.header.level_table), sched: &mut r.st.sched };
            let (_, frames) = run_level_scripts_framed(&mut r.st.vm, &r.lv.data, &r.w, &mut h);
            r.st.frames = frames;
        }
        job_flow::FUN_001638f8(&mut r.m, &r.w);
        let mut slots = JobSlots::default();
        job_flow::FUN_00161a58_case5(&mut r.m, &r.w, &mut slots, 0, &|_| SavedTask::default());
        r
    }
    pub fn host(&mut self) -> &mut EngineHost {
        &mut self.st.sched.cls.host
    }
    pub fn task(&self, oid: i32) -> Option<i32> {
        self.m.tasks.iter().find(|t| t.oid == oid).map(|t| t.state)
    }
    pub fn step(&mut self, n: usize) {
        for _ in 0..n {
            let f = self.f;
            self.f += 1;
            let t = (f + 1) as f32 / 60.0;
            {
                let l = &mut self.st.sched.cls;
                let set = l.set.as_mut().unwrap();
                set.g.t = t;
                set.g.dt = 1.0 / 60.0;
                set.frame(&mut l.host);
            }
            let ts0: Vec<(i32, i32)> = self.m.tasks.iter().map(|t| (t.oid, t.state)).collect();
            apply_job_calls(&mut self.st.sched, &mut self.m);
            self.st.sched.jobs = JobView::of(&self.m);
            job_messages(&mut self.st.sched, &self.m);
            self.st.sched.cls.drain_seq_starts();
            for n in &self.st.sched.cls.nodes {
                let oid = self.w.objects[n.owner as usize - 1].oid;
                if !self.started.contains(&oid) {
                    if self.log {
                        eprintln!("f{f}: sequence {oid} started");
                    }
                    self.started.push(oid);
                }
            }
            if self.log {
                for (a, b) in ts0.iter().zip(self.m.tasks.iter()) {
                    if a.1 != b.state {
                        eprintln!("f{f}: task {} state {} -> {}", a.0, a.1, b.state);
                    }
                }
                for x in std::mem::take(&mut self.st.sched.cls.host.k22_seq_actions) {
                    eprintln!("f{f}: UNPORTED seq action owner {} subject {} rec {:x?}", x.0, x.1, &x.2[..6]);
                }
                for u in std::mem::take(&mut self.st.sched.cls.host.unported) {
                    eprintln!("f{f}: unported {u}");
                }
            }
            let e0 = self.st.errors.len();
            let r0 = self.st.sched.cls.recorded.len();
            let SpliceState { vm, sched, errors, frames, .. } = &mut self.st;
            let base = GameSpliceHost { t, cm: None, emitters: &mut self.em, vars: Some(&self.vars), data: &mut self.data };
            let level = LevelSpliceHost { base, mission: -1, aliases: &mut self.al, rumble: &mut self.rum };
            let mut h = PersistentHost { level, cm: Some(&mut self.cm), clk: Default::default(), world: Some(&*self.w), table: Some(&self.lv.header.level_table), sched };
            send_messages(vm, errors, frames, &mut h);
            tick_world_update(vm, errors, &mut h, t, 1.0 / 60.0);
            if self.log {
                for r in &self.st.sched.cls.recorded[r0..] {
                    eprintln!("f{f}: recorded {:x?} obj {}", r.callee, r.obj);
                }
                for e in &self.st.errors[e0..] {
                    eprintln!("f{f}: error {}", format!("{e:?}").chars().take(200).collect::<String>());
                }
                for p in sly2_recomp::engine::game::flow::take_posted() {
                    eprintln!("f{f}: level request {p:?}");
                }
            }
        }
    }
}

/// Probe: level 0 boot, title shown, START (title released) and the prologue afterwards.
#[test]
#[ignore]
fn ep1a_probe_newgame() {
    let _flow = crate::common::flow_lock(); // INT: log mode drains flow::take_posted (ep1d warp test raced it)
    let mut r = Run::new(0);
    r.log = true;
    while r.host().k22_title_shows == 0 && r.f < 600 {
        r.step(1);
    }
    eprintln!("title shown at f{}; tasks 1664 {:?} 1582 {:?} 1583 {:?}", r.f, r.task(1664), r.task(1582), r.task(1583));
    r.step(60);
    r.host().k22_title_active = false; // START -> FUN_001dd888
    eprintln!("--- START at f{}", r.f);
    r.step(60 * 60);
    eprintln!("started {:?}", r.started);
    for t in &r.m.tasks {
        eprintln!("task {} state {}", t.oid, t.state);
    }
}

/// Boot: level 0 (jb_intro) raises the title from its sequence; START (FUN_001dd888 hides it) lets the
/// prologue job flow on (tasks 1582 -> 2, 1583 -> 1).
#[test]
fn ep1a_title_then_new_game_starts_prologue() {
    let mut r = Run::new(0);
    while r.host().k22_title_shows == 0 && r.f < 600 {
        r.step(1);
    }
    assert!(r.host().k22_title_shows > 0, "title never shown");
    assert_eq!(r.task(1582), Some(1), "prologue waits on the title");
    r.step(60);
    assert_eq!(r.task(1582), Some(1), "still on the title");
    r.host().k22_title_active = false;
    r.step(120);
    assert_eq!(r.task(1582), Some(2));
    assert_eq!(r.task(1583), Some(1));
}

/// Episode 1 hub: level-table row 2 (f_nightclub_exterior_vz, Paris) loads; its guards come from the 4 C119 spawners once it runs.
#[test]
fn ep1a_paris_hub_loads_with_guards() {
    let mut r = Run::new(2);
    let k6 = |r: &Run| r.st.sched.cls.set.as_ref().unwrap().k6.len();
    assert_eq!(k6(&r), 0, "hub guards are all template copies (no load placement)");
    assert_eq!(r.w.objects.iter().filter(|o| o.class == 119).count(), 4, "C119 spawners");
    r.step(300);
    assert!(!r.m.tasks.is_empty(), "no job tasks");
    assert!(k6(&r) > 0, "no guards");
}

/// P26 loot wired per world (`player::plugin::loot_link` -> `LootSys::from_class_set`): Paris has 2 class-37
/// holders and 8 class-103 loot; FUN_0013cf38 (slot-22 deferred PMF 0x2a4bb0) hands the holders their loot
/// (default id 700); a pickpocket flies it to Sly and he holds it on arrival.
#[test]
fn ep1a_paris_loot_system() {
    use sly2_recomp::engine::classes::ClassHost;
    use sly2_recomp::engine::player::loot::LootSys;
    let mut r = Run::new(2);
    let n = r.w.objects.len() as u32;
    let ps: Vec<[f32; 3]> = (1..=n).map(|o| r.host().ee_sub_1a62e8(o)).collect();
    let pos = move |h: u32| ps.get(h as usize - 1).copied().unwrap_or([0.0; 3]);
    let k31 = &r.st.sched.cls.set.as_ref().unwrap().k31;
    let mut s = LootSys::from_class_set(r.w.objects.iter().map(|o| o.class as u16), k31, &[], &pos);
    assert_eq!((s.guards.len(), s.loots.len()), (2, 8));
    let gi = s.guards.iter().position(|g| g.loot.is_some()).expect("a holder got its loot");
    assert_eq!(s.guards[gi].booty, 700);
    let sly = r.w.objects.iter().position(|o| o.class == sly2_recomp::engine::player::p20::CLASS_SLY).unwrap() as u32 + 1;
    let l = s.guards[gi].loot.unwrap();
    assert!(s.ee_sub_13d020(s.guards[gi].obj, sly, true, &pos));
    assert_eq!(s.loots[l].state, 2);
    let end = s.loots[l].fx_end;
    s.update(end, &pos, &|_| true);
    assert_eq!(s.held_by(sly).map(|x| x.0), Some(l));
}

#[test]
#[ignore]
fn ep1a_probe_loot_classes() {
    let (loader, descs) = k20::levels();
    for row in [0usize, 2, 3] {
        let lv = crate::common::load_blocking(&loader, descs[row]).unwrap();
        let w = World::load(&lv, LoadOptions::default()).unwrap();
        let tpl: std::collections::HashSet<usize> = w.entries.iter().flat_map(|e| e.instances.iter().copied()).collect();
        for c in [37u16, 103] {
            let v: Vec<(usize, Option<usize>, bool)> = w.objects.iter().enumerate().filter(|(_, o)| o.class as u16 == c).map(|(i, o)| (i, o.owner, tpl.contains(&i))).collect();
            eprintln!("row {row} class {c}: {} {:?}", v.len(), v.iter().take(12).map(|x| (x.0, x.1.map(|p| w.objects[p].class), x.2)).collect::<Vec<_>>());
        }
    }
}
