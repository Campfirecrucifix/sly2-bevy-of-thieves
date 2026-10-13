//! WP-EP1d tests: Episode 1 mission completion (SetParent / SO collision parts, prologue -> Episode 1 job
//! flow, Paris hub mission flow).
use crate::engine_ep1a as ep1a;

use ep1a::Run;

fn dump_jobs(r: &Run) {
    for (k, j) in r.m.jobs.iter().enumerate() {
        eprintln!("job#{k} oid {} start {:?} end {:?} tasks {:?} state {} f50 {}", j.oid, j.start.map(|t| r.m.tasks[t].oid), j.end.map(|t| r.m.tasks[t].oid), j.tasks.iter().map(|&t| r.m.tasks[t].oid).collect::<Vec<_>>(), j.state, j.f_50);
    }
    for t in &r.m.tasks {
        eprintln!("task oid {} job {:?} state {} level {} spawn {} c0 {} prereq {:?} deps {:?}", t.oid, t.job.map(|j| r.m.jobs[j].oid), t.state, t.level, t.spawn, t.c0, t.prereq.iter().map(|&p| r.m.tasks[p].oid).collect::<Vec<_>>(), t.deps.iter().map(|&p| r.m.tasks[p].oid).collect::<Vec<_>>());
    }
}

#[test]
#[ignore]
fn ep1d_probe_jobs() {
    for row in [0usize, 2] {
        eprintln!("==== row {row}");
        let r = Run::new(row);
        dump_jobs(&r);
    }
}

/// Probe: prologue after START, log task states / level requests.
#[test]
#[ignore]
fn ep1d_probe_prologue() {
    let _flow = crate::common::flow_lock(); // INT: shares flow POSTED with ep1d_prologue_warp_to_episode1
    let mut r = Run::new(0);
    r.log = true;
    while r.host().k22_title_shows == 0 && r.f < 600 {
        r.step(1);
    }
    r.step(60);
    r.host().k22_title_active = false;
    eprintln!("--- START at f{}", r.f);
    r.step(60 * 180);
    dump_jobs(&r);
}

/// Probe: dump every level script expression (SLY2_EP1D_ROW, default 0) to C:/Temp/ep1d_scripts_row<row>.txt.
#[test]
#[ignore]
fn ep1d_probe_dump_scripts() {
    use sly2_recomp::engine::splice::serialize::{print_ref, read_script_block};
    let row: usize = std::env::var("SLY2_EP1D_ROW").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    let r = Run::new(row);
    let mut out = String::new();
    for (ei, e) in r.w.entries.iter().enumerate() {
        let off = e.start_offset as usize + 12;
        let Some(Ok((blk, _))) = r.lv.data.get(off..).map(|d| read_script_block(d, e.sections.ext)) else { continue };
        if blk.is_empty() {
            continue;
        }
        let objs: Vec<(i32, u16)> = e.objects.iter().take(4).map(|&o| (r.w.objects[o].oid, r.w.objects[o].class as u16)).collect();
        out += &format!("=== entry {ei} objs {objs:?}\n");
        for (xi, x) in blk.iter().enumerate() {
            if let Some(p) = x {
                let s = print_ref(&p.borrow().car, &r.st.vm.symbols);
                out += &format!("[{xi}] {s}\n");
            }
        }
    }
    std::fs::write(format!("{}/ep1d_scripts_row{row}.txt", std::env::temp_dir().display()), out).unwrap();
}

/// FUN_001f2b20 links only the parts without bit 50 (b2); FUN_001f6b48 / 001f6ac8 unlink / relink by part
/// id (+0xc) and bump `so_gen`.
#[test]
fn ep1d_so_part_links() {
    use sly2_recomp::engine::classes::ClassHost;
    use sly2_recomp::engine::physics::query::CollisionWorld;
    use sly2_recomp::engine::world::objects::{LoadData, Payload};
    let mut r = Run::new(2);
    let w = r.w.clone();
    let (mut all, mut off) = (0, 0);
    let mut pick = None;
    for (i, o) in w.objects.iter().enumerate() {
        let Payload::Loaded(LoadData::Alo { alo, .. }) = &o.payload else { continue };
        if o.class == 89 {
            continue;
        }
        all += alo.parts.len();
        off += alo.parts.iter().filter(|p| p.b2 & 1 != 0).count();
        if pick.is_none() {
            if let Some(p) = alo.parts.iter().find(|p| p.b2 & 1 == 0) {
                pick = Some((i, p.w0, alo.parts.iter().filter(|q| q.w0 == p.w0).count()));
            }
        }
    }
    let so = CollisionWorld::build_so(&w);
    assert_eq!(so.colliders.len(), all - off, "bit-50 parts are not linked");
    let (i, id, n) = pick.expect("a linked part");
    let h = r.host();
    let count = |h: &mut sly2_recomp::engine::classes::plugin::EngineHost| h.so.as_ref().unwrap().colliders.iter().filter(|c| c.object == i).count();
    h.ee_sub_1f6ac8(i, id); // already linked: no change
    let before = count(h);
    let g0 = h.so_gen;
    h.ee_sub_1f6b48(i, id);
    assert_eq!(count(h), before - n);
    assert_eq!(h.so_gen, g0 + 1);
    h.ee_sub_1f6b48(i, id);
    assert_eq!(h.so_gen, g0 + 1, "already unlinked");
    h.ee_sub_1f6ac8(i, id);
    assert_eq!(count(h), before);
    assert_eq!(h.so_gen, g0 + 2);
    eprintln!("parts {all} bit50 {off}; obj {i} id {id} parts {n}");
}

/// FUN_001a58d0 SetParent: flags & 3 keep the world matrix; the object then follows its new parent;
/// parent_10 / alo_children see the run-time hierarchy; detaching (0, 0xf) keeps the world matrix.
#[test]
fn ep1d_set_parent() {
    use sly2_recomp::engine::classes::guard::PLAYER_HANDLE;
    use sly2_recomp::engine::classes::ClassHost;
    let mut r = Run::new(2);
    let w = r.w.clone();
    let h = r.host();
    h.player_pos = [100.0, 200.0, 300.0];
    // an object with a static parent
    let i = w.objects.iter().position(|o| o.owner.is_some() && o.class == 48).expect("child ALO");
    let o = i as u32 + 1;
    let p0 = h.ee_sub_1a62e8(o);
    let old = h.parent_10(o);
    assert_ne!(old, 0);
    h.ee_sub_1a58d0_to(i, PLAYER_HANDLE, 3);
    assert_eq!(h.parent_10(o), PLAYER_HANDLE);
    assert!(!h.alo_children(old).contains(&o));
    let p1 = h.ee_sub_1a62e8(o);
    assert!((0..3).all(|k| (p1[k] - p0[k]).abs() < 1e-2), "world kept {p0:?} {p1:?}");
    h.player_pos[0] += 50.0;
    let p2 = h.ee_sub_1a62e8(o);
    assert!((p2[0] - p0[0] - 50.0).abs() < 1e-2 && (p2[1] - p0[1]).abs() < 1e-2, "follows the parent {p2:?}");
    h.ee_sub_1a58d0(i, 0, 0xf);
    assert_eq!(h.parent_10(o), 0);
    h.player_pos[0] += 50.0;
    let p3 = h.ee_sub_1a62e8(o);
    assert!((0..3).all(|k| (p3[k] - p2[k]).abs() < 1e-2), "detached, world kept {p2:?} {p3:?}");
    // re-attach to the old parent with flags 0: the kept local matrix is relative to the new parent
    h.ee_sub_1a58d0_to(i, old, 3);
    assert!(h.alo_children(old).contains(&o));
}

/// Probe: prologue after START with the task chain advanced (one state step every 30 frames).
#[test]
#[ignore]
fn ep1d_probe_prologue_skip() {
    let _flow = crate::common::flow_lock(); // INT: shares flow POSTED with ep1d_prologue_warp_to_episode1
    let mut r = Run::new(0);
    r.log = true;
    while r.host().k22_title_shows == 0 && r.f < 600 {
        r.step(1);
    }
    r.step(60);
    r.host().k22_title_active = false;
    eprintln!("--- START at f{}", r.f);
    r.step(60);
    for _ in 0..80 {
        let Some(t) = r.m.tasks.iter().position(|t| t.state == 1 || t.state == 2) else { break };
        let s = r.m.tasks[t].state;
        eprintln!("--- skip task {} {} -> {} reloads {:?}", r.m.tasks[t].oid, s, s + 1, r.m.reloads);
        r.m.FUN_00164640(Some(t), s + 1, 0);
        r.step(30);
    }
    eprintln!("reloads {:?} calls {:?}", r.m.reloads, &r.m.calls[r.m.calls.len().saturating_sub(20)..]);
    dump_jobs(&r);
}

/// Probe: warp volumes (class 26) of a row and the job / task warp oids (job +0xcc, task +0xc4).
#[test]
#[ignore]
fn ep1d_probe_warps() {
    use sly2_recomp::engine::classes::Inst;
    let rows: Vec<usize> = std::env::var("SLY2_EP1D_ROWS").ok().map(|v| v.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or(vec![0, 2]);
    for row in rows {
        let mut r = Run::new(row);
        eprintln!("==== row {row}");
        dump_jobs(&r);
        let ws: Vec<i32> = r.st.sched.cls.set.as_ref().unwrap().more.iter().filter(|(_, x)| matches!(x, Inst::C2dc6f0(_))).map(|(i, _)| r.w.objects[*i].oid).collect();
        for o in ws {
            let (i, p) = warp_point(&mut r, o);
            eprintln!("warp {o} obj {i} at {p:?}");
        }
        if std::env::var("SLY2_EP1D_OBJS").is_ok() {
            for (i, o) in r.w.objects.iter().enumerate() {
                eprintln!("obj {i} oid {} class {:?} owner {:?} filter {:?}", o.oid, o.class, o.owner, o.filter);
            }
        }
        for j in &r.m.jobs {
            eprintln!("job {} cc {:?} c0 {:?}", j.oid, j.cc, j.c0);
        }
        for t in r.m.tasks.iter().filter(|t| t.c4.is_some()) {
            eprintln!("task {} c4 {:?}", t.oid, t.c4);
        }
        for (i, x) in &r.st.sched.cls.set.as_ref().unwrap().more {
            if let Inst::C2dc6f0(c) = x {
                use sly2_recomp::engine::world::objects::{LoadData, Payload};
                let rows: Vec<usize> = match &r.w.objects[*i].payload {
                    Payload::Loaded(LoadData::L2282b0 { first: Some(d), .. }) => {
                        let d = sly2_recomp::engine::disc::descriptor::Descriptor(*d);
                        r.lv.header.level_table.entries.iter().enumerate().filter(|(_, e)| e.desc.same_file(&d)).map(|(k, _)| k).collect()
                    }
                    _ => vec![],
                };
                eprintln!("warp obj {i} oid {} state {} f134 {} f150 {} f154 {} f15c {} first {} rows {rows:?}", r.w.objects[*i].oid, c.f_13c, c.f_134, c.f_150, c.f_154, c.f_15c, c.f_144);
            }
        }
    }
}

/// World position inside the class-26 warp volume `oid` (centroid of its solid, local -> world).
fn warp_point(r: &mut Run, oid: i32) -> (usize, [f32; 3]) {
    use sly2_recomp::engine::classes::ClassHost;
    use sly2_recomp::engine::world::objects::{LoadData, Payload};
    let i = r.w.find_oid(oid)[0];
    let Payload::Loaded(LoadData::L2282b0 { solid, .. }) = &r.w.objects[i].payload else { panic!("not a warp") };
    let n = solid.geom.verts.len() as f32;
    let c = solid.geom.verts.iter().fold([0.0f32; 3], |a, v| [a[0] + v[0] / n, a[1] + v[1] / n, a[2] + v[2] / n]);
    (i, r.host().ee_sub_1a6f00_to_world(i, c))
}

/// R1 (headless): the prologue (row 0) ends through its class-26 warp volume 150652 (FUN_00228538:
/// player leaves -> armed -> enters -> state 4 -> FUN_0022b490 with the warp's descriptor = level row 1).
/// Shortcut: the task chain is advanced with FUN_00164640 and the player is placed (no input), the rest
/// is the game code.
#[test]
fn ep1d_prologue_warp_to_episode1() {
    let _flow = crate::common::flow_lock(); // T1: process-global flow state
    use sly2_recomp::engine::classes::Inst;
    use sly2_recomp::engine::game::flow::{take_posted, Dest};
    let mut r = Run::new(0);
    while r.host().k22_title_shows == 0 && r.f < 600 {
        r.step(1);
    }
    r.step(60);
    r.host().k22_title_active = false;
    r.step(60);
    for _ in 0..80 {
        let Some(t) = r.m.tasks.iter().position(|t| t.state == 1 || t.state == 2) else { break };
        let s = r.m.tasks[t].state;
        r.m.FUN_00164640(Some(t), s + 1, 0);
        r.step(5);
    }
    assert!(r.m.tasks.iter().all(|t| t.state == 3), "prologue tasks done");
    take_posted();
    let (i, p) = warp_point(&mut r, 150652);
    r.host().player_valid = true;
    r.host().player_pos = [p[0] + 5000.0, p[1], p[2]];
    r.step(30);
    r.host().player_pos = p;
    let mut posted = Vec::new();
    for _ in 0..600 {
        r.step(1);
        posted.extend(take_posted());
        if !posted.is_empty() {
            break;
        }
    }
    let st = r.st.sched.cls.set.as_ref().unwrap().more.iter().find(|x| x.0 == i).map(|x| if let Inst::C2dc6f0(c) = &x.1 { c.f_13c } else { -1 });
    assert_eq!(st, Some(4), "warp state");
    let Some(Dest::Desc(d)) = posted.first().map(|p| p.dest) else { panic!("no warp request {posted:?}") };
    let rows: Vec<usize> = r.lv.header.level_table.entries.iter().enumerate().filter(|(_, e)| e.desc.same_file(&d)).map(|(k, _)| k).collect();
    assert_eq!(rows, vec![1], "prologue warp -> level row 1");
    assert_eq!(posted[0].spawn, 0x189);
    // row 1 = the class-131 episode select: mode 10 of a never-played episode 1 starts the intro warp
    // 0x2be2a8[1] = 1329 (FUN_00228358(o, 3) + FUN_002283d0(o)); the warp posts its descriptor = row 2 (Paris)
    let mut r = Run::new(1);
    r.step(30);
    take_posted();
    sly2_recomp::engine::ui::episode::episode_event(&mut Default::default(), sly2_recomp::engine::ui::episode::EpEvent::IntroMovie(1329), 1, &mut sly2_recomp::engine::input::pad::PadState::new());
    let mut posted = Vec::new();
    for _ in 0..600 {
        r.step(1);
        posted.extend(take_posted());
        if !posted.is_empty() {
            break;
        }
    }
    let Some(Dest::Desc(d)) = posted.first().map(|p| p.dest) else { panic!("no intro warp request {posted:?}") };
    let rows: Vec<usize> = r.lv.header.level_table.entries.iter().enumerate().filter(|(_, e)| e.desc.same_file(&d)).map(|(k, _)| k).collect();
    assert_eq!(rows, vec![2], "intro warp 1329 -> Paris row 2");
}

/// Probe: mission rows (SLY2_EP1D_ROWS, default 3..=7): jobs, then the task chain advanced with the
/// FUN_00164640 shortcut; log reloads / completed / posted requests.
#[test]
#[ignore]
fn ep1d_probe_missions() {
    let _flow = crate::common::flow_lock(); // INT: shares flow POSTED with ep1d_prologue_warp_to_episode1
    use sly2_recomp::engine::game::flow::take_posted;
    let rows: Vec<usize> = std::env::var("SLY2_EP1D_ROWS").ok().map(|v| v.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or(vec![3, 4, 5, 6, 7]);
    for row in rows {
        let mut r = Run::new(row);
        eprintln!("==== row {row} hub_row {:?} cur_row {} cur {:?}", r.m.hub_row, r.m.cur_row, r.m.cur.map(|j| r.m.jobs[j].oid));
        dump_jobs(&r);
        r.log = true;
        r.step(120);
        take_posted();
        for _ in 0..200 {
            let Some(t) = r.m.tasks.iter().position(|t| t.state == 1 || t.state == 2) else { break };
            let s = r.m.tasks[t].state;
            let jc: Vec<(i32, i32, i32, i32)> = r.m.jobs.iter().map(|j| (j.oid, j.c4, j.state, j.f_50)).filter(|x| x.1 != 0 || x.2 != 0 || x.3 != 0).collect();
            eprintln!("-- advance task {} {} -> {} (f4 {} cur {:?} busy {} jobs {jc:?})", r.m.tasks[t].oid, s, s + 1, r.m.tasks[t].f4, r.m.cur.map(|j| r.m.jobs[j].oid), r.m.world_busy);
            r.m.FUN_00164640(Some(t), s + 1, 0);
            for _ in 0..5 {
                let c0 = r.m.calls.len();
                r.m.update(r.f as f32 / 60.0, 1.0 / 60.0);
                r.step(1);
                for c in &r.m.calls[c0..] {
                    if [0x2283d0, 0x162e38, 0x1a4e80, 0x213b00, 0x18c000].contains(&c.0) {
                        eprintln!("   call {:x} {}", c.0, c.1);
                    }
                }
            }
            for p in take_posted() {
                eprintln!("   posted {:?} spawn {}", p.dest, p.spawn);
            }
            if let Some(c) = r.m.completed {
                eprintln!("   completed job {}", r.m.jobs[c].oid);
            }
            for x in std::mem::take(&mut r.m.reloads) {
                eprintln!("   reload {x:?}");
            }
        }
        eprintln!("final states {:?}", r.m.tasks.iter().map(|t| (t.oid, t.state)).collect::<Vec<_>>());
    }
}

/// Probe: method rows / bindings of the job thunks (0x10ea20 fail, 0x10ea98 restart, 0x10eb10 complete).
#[test]
#[ignore]
fn ep1d_probe_job_methods() {
    use sly2_recomp::engine::game::methods::{BINDINGS, METHODS};
    for (k, m) in METHODS.iter().enumerate() {
        if [0x10ea20u32, 0x10ea98, 0x10eb10].contains(&m.thunk) || [0x162e38u32, 0x162dd0, 0x162df8].contains(&m.callee) {
            let b: Vec<(u16, u16)> = BINDINGS.iter().filter(|b| b.method as usize == k).map(|b| (b.class, b.symid)).collect();
            eprintln!("method {k} {m:?} bindings {b:?}");
        }
    }
}

/// Advance the task chain (FUN_00164640 shortcut, no player input) until task `oid` has state >= `s`.
fn advance_to(r: &mut Run, oid: i32, s: i32) {
    for _ in 0..400 {
        let k = r.m.tasks.iter().position(|t| t.oid == oid).expect("task");
        if r.m.tasks[k].state >= s {
            return;
        }
        let Some(t) = r.m.tasks.iter().position(|t| t.state == 1 || t.state == 2) else { break };
        let st = r.m.tasks[t].state;
        r.m.FUN_00164640(Some(t), st + 1, 0);
        for _ in 0..5 {
            r.m.update(r.f as f32 / 60.0, 1.0 / 60.0);
            r.step(1);
        }
    }
    panic!("task {oid} did not reach {s}");
}

/// Put the player outside then inside warp volume `oid`; the level rows / spawn of the first request.
fn try_warp(r: &mut Run, oid: i32) -> Option<(Vec<usize>, i32)> {
    use sly2_recomp::engine::game::flow::{take_posted, Dest};
    take_posted();
    let (_, p) = warp_point(r, oid);
    r.host().player_valid = true;
    r.host().player_pos = [p[0] + 5000.0, p[1], p[2]];
    r.step(30);
    r.host().player_pos = p;
    for _ in 0..300 {
        r.m.update(r.f as f32 / 60.0, 1.0 / 60.0);
        r.step(1);
        if let Some(q) = take_posted().into_iter().next() {
            let Dest::Desc(d) = q.dest else { return Some((vec![], q.spawn)) };
            let rows = r.lv.header.level_table.entries.iter().enumerate().filter(|(_, e)| e.desc.same_file(&d)).map(|(k, _)| k).collect();
            return Some((rows, q.spawn));
        }
    }
    use sly2_recomp::engine::classes::Inst;
    let i = r.w.find_oid(oid)[0];
    let st = r.st.sched.cls.set.as_ref().unwrap().more.iter().find(|x| x.0 == i).map(|x| if let Inst::C2dc6f0(c) = &x.1 { (c.f_13c, c.f_134, c.f_40) } else { (-9, 0, 0) });
    eprintln!("warp {oid} obj {i} at {p:?} state {st:?}");
    None
}

/// Probe: SLY2_EP1D_ROW, advance to SLY2_EP1D_TASK (oid:state), then try SLY2_EP1D_WARPS.
#[test]
#[ignore]
fn ep1d_probe_try_warp() {
    let _flow = crate::common::flow_lock(); // INT: shares flow POSTED with ep1d_prologue_warp_to_episode1
    let Ok(row) = std::env::var("SLY2_EP1D_ROW") else { return }; // INT: probe, full tier runs it without env
    let row: usize = row.parse().unwrap();
    let ts = std::env::var("SLY2_EP1D_TASK").unwrap();
    let (o, s) = ts.split_once(':').unwrap();
    let warps: Vec<i32> = std::env::var("SLY2_EP1D_WARPS").unwrap().split(',').filter_map(|x| x.parse().ok()).collect();
    for w in warps {
        let mut r = Run::new(row);
        r.step(120);
        advance_to(&mut r, o.parse().unwrap(), s.parse().unwrap());
        eprintln!("row {row} task {ts} cur {:?} warp {w} -> {:?}", r.m.cur.map(|j| r.m.jobs[j].oid), try_warp(&mut r, w));
    }
}

/// Probe: SLY2_EP1D_ROW with the player valid (far from every warp), advance the chain task by task;
/// log every level request.
#[test]
#[ignore]
fn ep1d_probe_chain_posts() {
    let _flow = crate::common::flow_lock(); // INT: shares flow POSTED with ep1d_prologue_warp_to_episode1
    use sly2_recomp::engine::game::flow::{take_posted, Dest};
    let Ok(row) = std::env::var("SLY2_EP1D_ROW") else { return }; // INT: probe, full tier runs it without env
    let row: usize = row.parse().unwrap();
    let mut r = Run::new(row);
    r.host().player_valid = true;
    r.host().player_pos = [1.0e6, 1.0e6, 1.0e6];
    r.step(120);
    take_posted();
    for _ in 0..400 {
        let Some(t) = r.m.tasks.iter().position(|t| t.state == 1 || t.state == 2) else { break };
        let st = r.m.tasks[t].state;
        r.m.FUN_00164640(Some(t), st + 1, 0);
        for _ in 0..5 {
            r.m.update(r.f as f32 / 60.0, 1.0 / 60.0);
            r.step(1);
        }
        for q in take_posted() {
            let rows: Vec<usize> = match q.dest {
                Dest::Desc(d) => r.lv.header.level_table.entries.iter().enumerate().filter(|(_, e)| e.desc.same_file(&d)).map(|(k, _)| k).collect(),
                _ => vec![],
            };
            eprintln!("task {} {} -> {}: posted {:?} rows {rows:?} spawn {}", r.m.tasks[t].oid, st, st + 1, q.dest, q.spawn);
        }
        for x in std::mem::take(&mut r.m.reloads) {
            eprintln!("task {} {} -> {}: reload {x:?}", r.m.tasks[t].oid, st, st + 1);
        }
    }
}

/// Mission row `row`: the hub job start (FUN_00165538 at the job's first task of that row, as the hub
/// marker / job restart does) requests row `row` at the task's spawn; in that row the job's chain run to its
/// end task (FUN_00164640 shortcut = player input only) completes the job (0x162f10 -> 0x162e38) and
/// requests the hub (FUN_00210b70 = row 2) at 0x189. Returns (start request, completed, hub request).
fn mission(row: usize, job: i32, task: i32) -> (Option<(Option<i32>, i32)>, bool, Option<(Option<i32>, i32)>) {
    use sly2_recomp::engine::game::job_flow::FUN_00210b70;
    let mut r = Run::new(row);
    r.m.cur_row = row as i32;
    r.m.hub_row = FUN_00210b70(&r.lv.header.level_table, row as i32);
    assert_eq!(r.m.hub_row, Some(2), "row {row} hub");
    r.host().player_valid = true;
    r.host().player_pos = [1.0e6, 1.0e6, 1.0e6];
    r.step(120);
    r.m.reloads.clear();
    sly2_recomp::engine::game::flow::take_posted();
    let t = r.m.tasks.iter().position(|t| t.oid == task).expect("task");
    r.m.FUN_00165538(Some(t));
    let start = r.m.reloads.drain(..).next().map(|x| (x.row, x.spawn));
    r.step(5);
    let j = r.m.jobs.iter().position(|x| x.oid == job).expect("job");
    let end = r.m.jobs[j].end.unwrap();
    let mut back = None;
    for _ in 0..400 {
        if r.m.completed.is_some() || r.m.tasks[end].state >= 3 {
            break;
        }
        let Some(t) = r.m.tasks.iter().position(|t| (t.state == 1 || t.state == 2) && t.job == Some(j)) else { break };
        let st = r.m.tasks[t].state;
        r.m.FUN_00164640(Some(t), st + 1, 0);
        for _ in 0..5 {
            r.m.update(r.f as f32 / 60.0, 1.0 / 60.0);
            r.step(1);
        }
        if let Some(x) = r.m.reloads.iter().find(|x| x.row == Some(2)) {
            back = Some((x.row, x.spawn));
        }
        r.m.reloads.clear();
        for q in sly2_recomp::engine::game::flow::take_posted() {
            if let sly2_recomp::engine::game::flow::Dest::Desc(d) = q.dest {
                let rows: Vec<usize> = r.lv.header.level_table.entries.iter().enumerate().filter(|(_, e)| e.desc.same_file(&d)).map(|(k, _)| k).collect();
                if rows.contains(&2) || rows.contains(&1) {
                    back = Some((rows.first().map(|&k| k as i32), q.spawn));
                }
            }
        }
    }
    // the job's end leads to the next task of the chain; entering it starts its +0xc4 warp (0x2283d0)
    for k in 0..300 {
        if back.is_some() {
            break;
        }
        if k % 30 == 0 {
            // a task with a +0xc4 warp is started (state 4 -> 0x163c28 enter) as by the player
            if let Some(t) = r.m.tasks.iter().position(|t| t.state == 1 && t.c4.is_some()) {
                r.m.FUN_00164640(Some(t), 4, 0);
            } else if let Some(t) = r.m.tasks.iter().position(|t| t.state == 1 || t.state == 2) {
                let st = r.m.tasks[t].state;
                r.m.FUN_00164640(Some(t), st + 1, 0);
            }
        }
        r.m.update(r.f as f32 / 60.0, 1.0 / 60.0);
        r.step(1);
        for q in sly2_recomp::engine::game::flow::take_posted() {
            if let sly2_recomp::engine::game::flow::Dest::Desc(d) = q.dest {
                let rows: Vec<usize> = r.lv.header.level_table.entries.iter().enumerate().filter(|(_, e)| e.desc.same_file(&d)).map(|(k, _)| k).collect();
                if rows.contains(&2) || rows.contains(&1) {
                    back = Some((rows.first().map(|&k| k as i32), q.spawn));
                }
            }
        }
    }
    if back.is_none() {
        // the level exit warp 1733 (rows [2] in every mission row): the player walks into it
        back = try_warp(&mut r, 1733).map(|(rows, sp)| (rows.first().map(|&k| k as i32), sp));
        eprintln!("  exit warp 1733 -> {back:?}");
    }
    let cc: Vec<i32> = r.m.calls.iter().filter(|c| c.0 == 0x2283d0 || c.0 == 0x1a4e80).map(|c| c.1).collect();
    eprintln!("  next {:?}", r.m.tasks.iter().filter(|t| [1799, 1835, 1888, 1934].contains(&t.oid)).map(|t| (t.oid, t.state, t.c4)).collect::<Vec<_>>());
    eprintln!("  cc/c0 {:?} {:?} 0x2283d0 calls {cc:?}", r.m.jobs[j].cc, r.m.jobs[j].c0);
    eprintln!("row {row} job {job}: start {start:?} completed {:?} back {back:?} states {:?}", r.m.completed.map(|c| r.m.jobs[c].oid), r.m.jobs[j].tasks.iter().map(|&t| (r.m.tasks[t].oid, r.m.tasks[t].state)).collect::<Vec<_>>());
    (start, r.m.completed == Some(j), back)
}

/// R7-R11 (headless, task chain shortcut): every Episode 1 mission row is entered from the hub job start
/// at its spawn and its job runs to the end; rows 3 / 7 return to the hub (row 2, spawn 393) through the
/// game's own warps (task 1799 +0xc4 1798, job 1877 +0xcc 1739), row 5 (job 1915, +0xcc 1933) ends the
/// episode (row 1 = episode select). Rows 4 / 6 (jobs 1809 / 1859 / 1836) have no warp back yet (the
/// FUN_00162e38 complete-event list is not filled, see EP1d Findings).
#[test]
fn ep1d_mission_rows() {
    let _flow = crate::common::flow_lock(); // T1: process-global flow state
    for (row, job, task, spawn, back) in [
        (3, 1771, 1773, 397, Some((Some(2), 393))),
        (4, 1809, 1811, 398, None),
        (4, 1859, 1586, 401, None),
        (5, 1915, 1918, 122798, Some((Some(1), 393))),
        (6, 1836, 1587, 400, None),
        (7, 1877, 1878, 399, Some((Some(2), 393))),
    ] {
        let (s, _, b) = mission(row, job, task);
        assert_eq!(s, Some((Some(row as i32), spawn)), "row {row} job {job} start request");
        assert_eq!(b, back, "row {row} job {job} return");
    }
}

/// Probe: class 132 (job) field bindings: symid, kind, flags, member offset, words.
#[test]
#[ignore]
fn ep1d_probe_job_fields() {
    use sly2_recomp::engine::game::fieldbind::FIELD_BINDINGS;
    use sly2_recomp::engine::world::field_table::field_word;
    for &(c, s, f) in FIELD_BINDINGS.iter().filter(|b| b.0 == 132) {
        let w: Vec<String> = (0..8).map(|k| format!("{:x}", field_word(f as usize, k))).collect();
        eprintln!("class {c} sym {s} field {f} words {w:?}");
    }
}

/// Probe: props of the job objects (field, value).
#[test]
#[ignore]
fn ep1d_probe_job_props() {
    use sly2_recomp::engine::world::loads_b4::B4Load;
    use sly2_recomp::engine::world::objects::{LoadData, Payload};
    let r = Run::new(2);
    for o in r.w.objects.iter().filter(|o| o.class == 132) {
        if let Payload::Loaded(LoadData::B4(b)) = &o.payload {
            let B4Load::L1624a8 { props, .. } = &**b else { continue };
            eprintln!("job {} props {:?}", o.oid, props.iter().map(|p| (p.field, format!("{:?}", p.value).chars().take(60).collect::<String>())).collect::<Vec<_>>());
        }
    }
}
