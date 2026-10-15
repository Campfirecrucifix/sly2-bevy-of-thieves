//! WP-J4 tests: persistent Splice state over 300 frames on 3 levels (world frame + scheduled callbacks
//! survive frames, rebuilt only on level change), method binding table coverage, BIFs 98/99.
//! Needs the ISO (skipped otherwise).

use sly2_recomp::engine::audio::emitter::EmitterSystem;
use sly2_recomp::engine::disc::descriptor::{self, Descriptor, LevelTable};
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::game::methods::{self, coverage, ee_sub_201110, get_o_method, MArg};
use sly2_recomp::engine::game::save::{GameData, SaveVars};
use sly2_recomp::engine::game::scripts::run_level_scripts_with;
use sly2_recomp::engine::game::spawn::SpawnAliases;
use sly2_recomp::engine::game::splice_host::*;
use sly2_recomp::engine::game::vm_state::*;
use sly2_recomp::engine::splice::value::Ref;
use sly2_recomp::engine::splice::SpliceHost;
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

struct Ctx {
    em: EmitterSystem,
    data: GameData,
    al: SpawnAliases,
    rum: Vec<RumbleRequest>,
}

fn host<'a>(c: &'a mut Ctx, t: f32, mission: i32, vars: Option<&'a SaveVars>, w: Option<&'a World>, table: Option<&'a LevelTable>, sched: &'a mut Schedule) -> PersistentHost<'a> {
    let base = GameSpliceHost { t, cm: None, emitters: &mut c.em, vars, data: &mut c.data };
    let level = LevelSpliceHost { base, mission, aliases: &mut c.al, rumble: &mut c.rum , spawn_lists: Box::leak(Box::new(sly2_recomp::engine::player::p25::SpawnLists::default()))};
    PersistentHost { level, live_q: None, cm: None, clk: Default::default(), world: w, table, sched }
}

#[test]
fn script_state_persists_300_frames_on_3_levels() {
    let Some((l, t)) = loader() else { return eprintln!("ISO not found; skipped") };
    let boot = crate::common::load_blocking(&l, t[25]).unwrap();
    let mut st = SpliceState::default();
    let mut last_id = 0;
    for (n, r) in [3usize, 10, 20].into_iter().enumerate() {
        let lv = crate::common::load_blocking(&l, boot.header.level_table.entries[r].desc).unwrap();
        let w = World::load(&lv, LoadOptions::default()).unwrap();
        let vars = SaveVars::from_block(&lv.header.block_3a18);
        let mut c = Ctx { em: EmitterSystem::default(), data: GameData::default(), al: SpawnAliases::default(), rum: Vec::new() };
        assert!(st.enter_level(n as u32), "new level rebuilds the state");
        assert!(st.sched.list.is_empty());
        {
            let mut h = host(&mut c, 0.0, -1, Some(&vars), Some(&w), Some(&lv.header.level_table), &mut st.sched);
            let rep = run_level_scripts_with(&mut st.vm, &lv.data, &w, &mut h);
            assert!(rep.exprs > 0 && rep.ok == rep.exprs, "row {r}: {:?}", rep.errors.iter().take(3).collect::<Vec<_>>());
        }
        let frame_len = st.vm.global.borrow().bindings.len();
        assert!(frame_len > 0, "row {r}: empty world frame");
        assert!(!st.enter_level(n as u32), "same level keeps the state");
        // BIF 105 from a script: 10 callbacks (rumble id k after 0.5*k s), one cancelled by BIF 106
        let mut ids = Vec::new();
        {
            let mut h = host(&mut c, 0.0, -1, Some(&vars), Some(&w), None, &mut st.sched);
            for k in 1..=10 {
                let rest = sly2_recomp::engine::splice::value::list_from(vec![Ref::S32(k), Ref::F32(0.1)]);
                let id = h.ref_op_schedule_callback(&[Ref::F32(0.5 * k as f32), Ref::Bifk(130), rest]).unwrap();
                let Ref::S32(id) = id else { panic!("{id:?}") };
                ids.push(id);
            }
            h.bif106_splice_bif_106(&[Ref::S32(ids[4])]).unwrap();
        }
        // DAT_003e11fc is global: ids keep counting across levels
        assert!(ids[0] > last_id && ids.windows(2).all(|p| p[1] == p[0] + 1));
        last_id = ids[9];
        assert_eq!(st.sched.list.len(), 9);
        // 300 frames at 1/60 s
        for f in 1..=300 {
            let tt = f as f32 / 60.0;
            let SpliceState { vm, sched, errors, ticks, .. } = &mut st;
            let mut h = host(&mut c, tt, -1, Some(&vars), Some(&w), None, sched);
            tick_world_update(vm, errors, &mut h, tt, 1.0 / 60.0);
            *ticks += 1;
            assert_eq!(st.vm.global.borrow().bindings.len(), frame_len, "row {r} frame {f}: world frame changed");
        }
        assert!(st.errors.is_empty(), "{:?}", st.errors);
        assert_eq!(st.ticks, 300);
        assert!(st.sched.list.is_empty());
        let fired: Vec<usize> = c.rum.iter().map(|q| q.pattern).collect();
        assert_eq!(fired, vec![1, 2, 3, 4, 6, 7, 8, 9, 10], "row {r}");
    }
}

#[test]
fn method_table_coverage() {
    let cv = coverage();
    eprintln!("{cv:?}");
    // stored coverage of the 0x002ac880 kind-0x12 rows / 0x00279ad0 bindings
    assert_eq!(cv, EXPECT_COVERAGE);
    // emitter/camera/alias methods resolve through get-o by class
    for (class, thunk) in [(CLASS_EMITTER, 0x0010_1e18u32), (CLASS_CM, 0x0010_26e0)] {
        let b = methods::BINDINGS.iter().find(|b| b.class == class && methods::METHODS[b.method as usize].thunk == thunk);
        let b = b.unwrap_or_else(|| panic!("class {class} thunk {thunk:#x} not bound"));
        let Some(Ref::Method(m)) = get_o_method(7, class, b.symid) else { panic!() };
        assert_eq!((m.object, m.method_id), (7, thunk));
    }
    // the spawn-alias methods are 0x400-only rows: FUN_001ffd98 falls through and get-o yields Nil
    let b = methods::BINDINGS.iter().find(|b| b.class == 42 && methods::METHODS[b.method as usize].thunk == 0x0010_b8f8).expect("alias bound");
    assert!(matches!(get_o_method(7, 42, b.symid), Some(Ref::Nil)));
    assert!(methods::BINDINGS.windows(2).all(|p| (p[0].class, p[0].symid) < (p[1].class, p[1].symid)));
    assert_eq!(format!("{:?}", ee_sub_201110(&[1, 6, 0x1004], &[Ref::S32(3), Ref::F32(2.7), Ref::S32(9)])), format!("{:?}", vec![MArg::F32(3.0), MArg::I32(2), MArg::Word(9)]));
}

// J5: +0x00101d20 (FUN_00130b18 retract)
const EXPECT_COVERAGE: methods::Coverage = methods::Coverage { methods: 383, empty_rows: 7, script_reachable: 308, bindings: 4079, kinds_recovered: 376, implemented: 9 };

#[test]
fn bifs_98_99_mission_tables() {
    let Some((l, t)) = loader() else { return eprintln!("ISO not found; skipped") };
    let boot = crate::common::load_blocking(&l, t[25]).unwrap();
    let tb = &boot.header.level_table;
    let mut sched = Schedule::default();
    let mut c = Ctx { em: EmitterSystem::default(), data: GameData::default(), al: SpawnAliases::default(), rum: Vec::new() };
    let mut seen = Vec::new();
    for m in [-1i32, 0, 3, 10, 20, tb.entries.len() as i32 - 1] {
        let mut h = host(&mut c, 0.0, m, None, None, Some(tb), &mut sched);
        let (a, b) = (h.bif98_splice_bif_98(&[]).unwrap(), h.bif99_splice_bif_99(&[]).unwrap());
        let want = if m < 0 { (-1, -1) } else { (tb.entries[m as usize].b1 as i32, tb.entries[m as usize].idx as i32) };
        assert_eq!(format!("{a:?} {b:?}"), format!("{:?} {:?}", Ref::S32(want.0), Ref::S32(want.1)), "mission {m}");
        seen.push((m, want));
    }
    eprintln!("98/99 by mission: {seen:?}");
    assert_eq!(seen, EXPECT_98_99);
}

const EXPECT_98_99: &[(i32, (i32, i32))] = &[(-1, (-1, -1)), (0, (0, 0)), (3, (2, 1)), (10, (8, 2)), (20, (12, 5)), (43, (26, 8))];
