//! WP-P8: Sly falls into a real water volume and takes damage (0x1a2460 -> 0x1f7598 -> 0x1a19a0), the
//! 0x3e15c0 hit list, and job restart on death (0x1a2388 -> 0x1619a0 -> 0x162bb8 -> 0x162df8 -> 0x165538)
//! with the jobs of a real level.
use sly2_recomp::engine::disc::descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::jobs::{JobMgr, JobReload, EV_DIED, SPAWN_DEFAULT};
use sly2_recomp::engine::physics::CollisionWorld;
use sly2_recomp::engine::player::p7_damage::{self as dmg, DmgEv, HP_MAX};
use sly2_recomp::engine::player::p8_hazard::{self as hz, Hazards, Water, HIT_TTL, JT_10A0, WATER_AMOUNT, WATER_KIND};
use sly2_recomp::engine::player::{Jt, JtInput};
use sly2_recomp::engine::world::{LoadOptions, World};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

const ISO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso");
const DT: f32 = 1.0 / 60.0;

type Lv = (Arc<World>, Vec<descriptor::Descriptor>);

fn level(row: usize) -> Option<Lv> {
    static C: OnceLock<Mutex<HashMap<usize, Option<Lv>>>> = OnceLock::new();
    let c = C.get_or_init(Default::default);
    if let Some(v) = c.lock().unwrap().get(&row) {
        return v.clone();
    }
    let v = (|| {
        let iso = Arc::new(IsoImage::open(ISO).ok()?);
        let t = descriptor::static_table(&iso.read_file("SCUS_973.16").ok()?)?;
        let l = LevelLoader::new(iso, ModOverlay::default());
        let boot = crate::common::load_blocking(&l, t[25]).ok()?;
        let table: Vec<_> = boot.header.level_table.entries.iter().map(|e| e.desc).collect();
        let lv = crate::common::load_blocking(&l, table[row]).ok()?;
        Some((Arc::new(World::load(&lv, LoadOptions::default()).ok()?), table))
    })();
    c.lock().unwrap().insert(row, v.clone());
    v
}

/// Points inside the water: the topmost inside sample of every column of a grid over the transformed BSP
/// part's bounding box.
fn inside_points(w: &Water) -> Vec<[f32; 3]> {
    let mut out = Vec::new();
    for p in &w.parts {
        let Some((g, _)) = &p.bsp else { continue };
        let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
        for v in &g.verts {
            for k in 0..3 {
                lo[k] = lo[k].min(v[k]);
                hi[k] = hi[k].max(v[k]);
            }
        }
        let n = 12;
        for i in 1..n {
            for j in 1..n {
                let f = |a: usize, d: usize| lo[d] + (hi[d] - lo[d]) * a as f32 / n as f32;
                if let Some(q) = (1..=4 * n)
                    .rev()
                    .map(|k| {
                        [
                            f(i, 0),
                            f(j, 1),
                            lo[2] + (hi[2] - lo[2]) * k as f32 / (4 * n) as f32 - 1.0,
                        ]
                    })
                    .find(|q| w.FUN_001f72f8(*q))
                {
                    out.push(q);
                }
            }
        }
    }
    out
}

#[test]
fn pure_parts() {
    // 0x1f78d8: OR of the +0x110 bytes of the set kind bits (< 26)
    let mut t = [0u8; 26];
    t[13] = 0x10;
    t[3] = 0x01;
    assert_eq!(hz::FUN_001f78d8(&t, 0x2000), 0x10);
    assert_eq!(hz::FUN_001f78d8(&t, 0x2008), 0x11);
    assert_eq!(hz::FUN_001f78d8(&t, 0x4000000), 0);
    // 0x1f77c0: records expire after 0.25 s
    let mut h = Hazards::default();
    h.hits.push(hz::HitRec {
        src: 0,
        w4: 0,
        kind: 0x2000,
        obj: 0,
        t: 1.0,
    });
    h.FUN_001f77c0(1.0 + HIT_TTL - 0.01);
    assert_eq!(h.hits.len(), 1);
    h.FUN_001f77c0(1.0 + HIT_TTL);
    assert!(h.hits.is_empty());
    let jt = Jt::new([0.0; 3]);
    assert_eq!((jt.p8.water, jt.p8.off_10a0), (None, JT_10A0));
    // 0x210990
    assert_eq!(JobMgr::FUN_00210990(-1), None);
    assert_eq!(JobMgr::FUN_00210990(27), Some(27));
}

#[test]
fn sly_falls_into_real_water_and_takes_damage() {
    // rows with class-89 water (Findings P8); row 3 (P7's level) has none
    let mut done = false;
    for row in [2usize, 4, 8, 10, 13, 14] {
        let Some((w, _)) = level(row) else { continue };
        let haz = Hazards::load(&w);
        if haz.waters.is_empty() {
            continue;
        }
        for wa in &haz.waters {
            assert_eq!(wa.kind, WATER_KIND);
        }
        // drop Sly onto the water from 150 above an inside point. No level collision: CollisionWorld keeps ALO
        // parts untransformed (physics/ limitation, Findings P8) and the water parts themselves would be
        // solid, so Sly free-falls through the real volume.
        let cw = CollisionWorld::default();
        let mut found = None;
        'w: for wi in (0..haz.waters.len()).filter(|&i| haz.waters[i].amount != 0) {
            for p in inside_points(&haz.waters[wi]) {
                let mut h2 = haz.clone();
                let mut jt = Jt::new([p[0], p[1], p[2] - JT_10A0 + 150.0]);
                if h2.waters[wi].FUN_001f72f8([p[0], p[1], p[2] + 150.0]) {
                    continue; // starts in the water
                }
                let mut t = 0.0;
                for f in 0..240 {
                    jt.update(&cw, JtInput::default(), DT, &mut ());
                    t += DT;
                    hz::hazard_step(&mut jt, &mut h2, t, &mut ());
                    if jt.dmg.hp < HP_MAX || jt.dmg.ev.contains(&DmgEv::Event(0x10)) {
                        if f == 0 {
                            break; // the volume reaches above the start point
                        }
                        found = Some((wi, p, jt, h2, t, f));
                        break 'w;
                    }
                }
            }
        }
        let Some((wi, p, mut jt, mut haz, t, f)) = found else {
            panic!("row {row}: Sly never took water damage")
        };
        let amount = haz.waters[wi].amount;
        let _ = p;
        assert!(f > 0, "row {row}: Sly started outside the water");
        assert_eq!(jt.p8.water, Some(wi));
        assert_eq!(haz.hits.len(), 1);
        assert_eq!((haz.hits[0].src, haz.hits[0].kind), (wi, WATER_KIND));
        assert!(jt.dmg.hit.as_ref().map(|h| h.kind) == Some(WATER_KIND));
        // dedupe: a second delivery inside 0.25 s is dropped by 0x1f7598
        let hp = jt.dmg.hp;
        let mut xp = hz::Xp {
            src: wi,
            w4: 0,
            kind: WATER_KIND,
            amount,
            pos: jt.phys.pos,
            vel: [0.0, 0.0, -1.0],
            zap: 0,
        };
        assert!(!hz::FUN_001f7598(&mut jt, &mut haz, &mut xp, t + 0.1, &mut ()));
        assert_eq!(jt.dmg.hp, hp);
        if amount == WATER_AMOUNT {
            assert!(jt.dmg.hp < HP_MAX);
        }
        eprintln!(
            "row {row}: water {wi} (oid {}) amount {amount} hit after {f} frames at {:?}",
            haz.waters[wi].oid, jt.phys.pos
        );
        done = true;
        break;
    }
    if level(2).is_some() {
        assert!(done, "no damaging water found");
    }
}

fn row27() -> Option<(Lv, JobMgr)> {
    let l = level(27)?;
    let m = JobMgr::load(&l.0);
    Some((l, m))
}

#[test]
fn real_level_jobs_load() {
    let Some((_, m)) = row27() else { return };
    assert!(!m.jobs.is_empty() && !m.tasks.is_empty());
    // job 2844: start task 2845 (prop 1243 -> +0x64), task 2845 belongs to it and restarts in row 27
    let j = m.job_by_oid(2844).expect("job 2844");
    let t = m.jobs[j].start.expect("start task");
    assert_eq!(m.tasks[t].oid, 2845);
    assert_eq!(m.tasks[t].job, Some(j));
    assert_eq!((m.tasks[t].spawn, m.tasks[t].level, m.tasks[t].c0), (0xc21a, 27, 7));
    assert_eq!(m.cur, None);
}

#[test]
fn death_during_job_restarts_it() {
    let Some(((_, table), mut m)) = row27() else { return };
    let j = m.job_by_oid(2844).unwrap();
    let t = m.jobs[j].start.unwrap();
    m.cur_row = 27;
    m.FUN_001628b8(j);
    assert_eq!(m.cur, Some(j));
    assert_eq!(m.jobs[j].n_7c, 1); // 0x1629a0 started it
    assert!(m.calls.contains(&(0x001629a0, 0x13)));

    // job running (+0x50 == 0): death reaches the job's scripts only (msg 0x11)
    let mut jt = Jt::new([0.0, 0.0, 0.0]);
    jt.dmg_env.job = m.cur.is_some();
    let n = jt.dmg.ev.len();
    // slot 92 0x1a2388 (the end of the death sequence, P7)
    let env = jt.dmg_env;
    dmg::FUN_001a2388(&mut jt, &env);
    assert!(jt.dmg.ev.contains(&DmgEv::Event(0x10)));
    assert!(
        jt.respawn.is_none(),
        "0x161a58 must not reload the level while a job is current"
    );
    hz::job_events(&jt, n, &mut m);
    assert!(m.calls.contains(&(0x00162bb8, EV_DIED)));
    assert!(m.reloads.is_empty());

    // job suspended by its +0x50 flag: death -> 0x162df8 -> 0x165538(start task)
    m.jobs[j].f_50 = 1;
    m.FUN_001619a0(EV_DIED, 0);
    let r = JobReload {
        row: Some(27),
        spawn: 0xc21a,
        spawn_parent: -1,
        flags: 4,
        a: 0x30a,
    };
    assert_eq!(m.reloads, vec![r]);
    assert_eq!(m.cur, Some(j));
    assert!(m.calls.contains(&(0x001d8b20, 7)) && m.calls.contains(&(0x00164df0, t as i32)));
    let req = r.level_request(&table, 27).unwrap();
    assert_eq!(
        (req.desc, req.flags, req.spawn, req.spawn_parent),
        (table[27], 6, 0xc21a, -1)
    );
    // world busy: no restart
    m.reloads.clear();
    m.world_busy = true;
    m.FUN_001619a0(EV_DIED, 0);
    assert!(m.reloads.is_empty());
    // pause-menu restart (0x211d38) and DAT_003e12c8 same-level flags 0x24
    m.world_busy = false;
    m.flag_12c8 = true;
    m.restart_current();
    assert_eq!((m.retries, m.reloads[0].flags), (1, 0x24));
    // restart with no task -> default spawn in the current level
    m.reloads.clear();
    m.FUN_00165538(None);
    assert_eq!(
        m.reloads[0],
        JobReload {
            row: None,
            spawn: SPAWN_DEFAULT,
            spawn_parent: -1,
            flags: 0x24,
            a: 0x30a
        }
    );
}

/// Ledge hang on a real ledge (0x197500 case 0x11): grab via 0x199d48, then the hang body re-checks the edge
/// with 0x1999d0 every frame after 0.03 s and faces the wall (0x2046b0(jt, -dir)); X after 0.25 s jumps off.
#[test]
fn ledge_hang_on_real_ledge() {
    use sly2_recomp::engine::player::p4::FUN_00199408;
    use sly2_recomp::engine::player::{JTBS_JUMP_FALL, JTBS_LEDGE, JTS_JUMP, JT_RADIUS};
    let mut any = false;
    for row in [0usize, 2, 4, 8] {
        let Some((w, _)) = level(row) else { continue };
        any = true;
        let cw = CollisionWorld::build(&w);
        let mut tried = 0;
        for col in &cw.colliders {
            let g = &col.geom;
            for e in 0..g.edges.len() {
                let Some(d) = FUN_00199408(g, e) else { continue };
                let ed = g.edges[e];
                let (a, b) = (g.verts[ed.a as usize], g.verts[ed.b as usize]);
                if ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt() < 80.0 {
                    continue;
                }
                let m = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0, (a[2] + b[2]) / 2.0];
                let c = [
                    m[0] + d[0] * (JT_RADIUS + 5.0),
                    m[1] + d[1] * (JT_RADIUS + 5.0),
                    m[2] - 20.0,
                ];
                if !cw.clip_segment(c, [c[0], c[1], c[2] - 300.0]).is_empty() {
                    continue;
                }
                tried += 1;
                if tried > 600 {
                    break;
                }
                let mut jt = Jt::new(c);
                jt.FUN_0018c250(JTS_JUMP, JTBS_JUMP_FALL, &mut ());
                jt.phys.v = [0.0, 0.0, -50.0];
                jt.update(&cw, JtInput::default(), DT, &mut ());
                if jt.jtbs != JTBS_LEDGE {
                    continue;
                }
                // hang 1 s: the edge stays valid, the body holds and faces the wall
                let p0 = jt.phys.pos;
                for _ in 0..60 {
                    jt.update(&cw, JtInput::default(), DT, &mut ());
                }
                assert_eq!(jt.jtbs, JTBS_LEDGE, "{:?}", jt.history);
                // WP-P18: the body eased from the grab position onto the hang point (jt+0x1010 decay)
                let h = jt.ledge_hit.unwrap();
                let hp = sly2_recomp::engine::player::p4::ledge_hang_point(h.contact.pos, h.dir);
                let _ = p0;
                assert!((0..3).all(|i| (jt.phys.pos[i] - hp[i]).abs() < 0.5), "{:?} vs {hp:?}", jt.phys.pos);
                let dir = jt.ledge_hit.unwrap().dir;
                assert!(
                    (jt.yaw - (-dir[1]).atan2(-dir[0])).abs() < 1e-5,
                    "yaw {} dir {dir:?}",
                    jt.yaw
                );
                println!(
                    "row {row}: hang at {p0:?} edge dir {dir:?} yaw {} after {tried} edges",
                    jt.yaw
                );
                // X jumps off (0x195a00(3))
                for _ in 0..5 {
                    jt.update(
                        &cw,
                        JtInput {
                            stick: [0.0; 2],
                            jump: true,
                        },
                        DT,
                        &mut (),
                    );
                }
                assert_ne!(jt.jtbs, JTBS_LEDGE);
                return;
            }
        }
        println!("row {row}: no grab in {tried} edges");
    }
    assert!(!any, "no ledge grabbed on any level");
}
