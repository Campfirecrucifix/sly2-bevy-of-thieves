//! WP-J14: game flow glue: FUN_0022b490 world requests posted by code without the game loop (title,
//! death, job restarts), the playtime clock hook FUN_00161478, the time-scale gadgets' clock slot and the
//! save slot files (memory card -> one file per slot, `save::MemoryCard`).
//!
//! FUN_0022b490(0x3dd490, desc, spawn, spawn parent, flags, a) only records the request in the world;
//! the level transition FUN_0020e890 acts on it next frame. The port posts `Post`s into a process-wide
//! mailbox (REPLACED: the world global 0x3dd490) that the game binary drains into
//! `TransitionState::request` (`resolve`: a null descriptor = the current level, flags | 2 when the
//! level is the current one, as 0x22b490 does).
//! 1:1: 0x161478 ("dt" part; episode timers DAT_003e0fe8 UNPORTED: FUN_00210ae0 episode index and its
//! save handles are not bound), 0x1ad320 load (slot file -> FUN_001d8d80 -> FUN_00165e38).
//! Time-scale gadgets 0x1d / 0x2a: 0x18dxxx call FUN_0014d1d0(scale, g_clock, 1) on use / release; the
//! port's JT keeps the value (`Gadgets::time_scale`) and `apply_time_scale` writes clock slot 1.
//! REPLACED: FUN_001abd98(0x2e65e0, 0) autosave (card manager dialog) -> `save_slot` (slot file write).
#![allow(non_snake_case)]

use std::sync::Mutex;

use crate::engine::disc::descriptor::{Descriptor, LevelTable};
use crate::engine::game::clock::{self, Clock};
use crate::engine::game::job_flow::{FUN_00210b98, JobSlots};
use crate::engine::game::save::{GameData, McError, MemoryCard, SaveValue, SaveVars};
use crate::engine::game::transition::LevelRequest;
use crate::engine::jobs::{JobMgr, JobReload};

/// FUN_0022b490 descriptor argument.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Dest {
    /// null descriptor: the current level
    Current,
    /// FUN_00210b98(i): level table slot i (DAT_003e1c50)
    Slot(i32),
    /// FUN_00210990(row): level table row (DAT_003e1c40)
    Row(i32),
    /// a static descriptor of the executable by address (0x2a3828 hideout, boot queue rows 0x2a3868 /
    /// 0x2a38a8, the zero descriptor 0x2bfca0); WP-J15: resolved through `static_desc`
    Static(u32),
    /// WP-J17: a descriptor stored in an object (class-26 warp volume +0x144, FUN_00228538)
    Desc(Descriptor),
}

/// One FUN_0022b490 request; `slots` = the job manager slots 0x2deb40..4c written with it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Post {
    pub dest: Dest,
    pub spawn: i32,
    pub spawn_parent: i32,
    pub flags: u32,
    pub a: i32,
    pub slots: Option<JobSlots>,
}

static POSTED: Mutex<Vec<Post>> = Mutex::new(Vec::new());

/// WP-J15: the ELF static descriptor table 0x2a3268 (30 rows of 64 bytes), set once by the binary / tests
/// from SCUS_973.16 (`descriptor::static_table`).
static STATIC_TABLE: Mutex<Vec<Descriptor>> = Mutex::new(Vec::new());

/// 0x2bfca0: a zero (.bss) descriptor: size 0 = "no level" (FUN_0020e890 resets only, then pops the boot queue).
pub const ZERO_DESC: u32 = 0x002b_fca0;

pub fn set_static_table(t: Vec<Descriptor>) {
    *STATIC_TABLE.lock().unwrap_or_else(|e| e.into_inner()) = t;
}

/// WP-J15: the descriptor at a static address: a row of the table 0x2a3268 (0x2a3828 = row 23 hideout,
/// 0x2a3868 = row 24 intro movie, 0x2a38a8 = row 25 level 0), the zero descriptor 0x2bfca0, else None.
pub fn static_desc(addr: u32) -> Option<Descriptor> {
    use crate::engine::disc::descriptor::STATIC_TABLE_ADDR;
    if addr == ZERO_DESC {
        return Some(Descriptor([0; 64]));
    }
    let off = addr.checked_sub(STATIC_TABLE_ADDR)?;
    if off % 64 != 0 {
        return None;
    }
    STATIC_TABLE.lock().unwrap_or_else(|e| e.into_inner()).get((off / 64) as usize).copied()
}

/// FUN_0022b490 (record): post a world request.
pub fn FUN_0022b490(p: Post) {
    POSTED.lock().unwrap_or_else(|e| e.into_inner()).push(p);
}

/// Requests posted since the last call.
pub fn take_posted() -> Vec<Post> {
    std::mem::take(&mut *POSTED.lock().unwrap_or_else(|e| e.into_inner()))
}

/// FUN_0022b490 request -> transition request. None = unresolved descriptor (no level table / static).
pub fn resolve(p: &Post, table: Option<&LevelTable>, current: Option<Descriptor>) -> Option<LevelRequest> {
    let d = match p.dest {
        Dest::Current => current?,
        Dest::Slot(i) => FUN_00210b98(table?, i).or(current)?,
        Dest::Row(r) => table?.entries.get(usize::try_from(r).ok()?)?.desc,
        Dest::Static(a) => static_desc(a)?,
        Dest::Desc(d) => d,
    };
    let mut r = LevelRequest::new(d);
    // 0x22b490: same level = (+8 ^ +0x38) of the request equals the current descriptor's (WP-J17: was
    // byte equality; the warp descriptors are obfuscated differently from the level-table rows)
    r.flags = p.flags | if current.is_some_and(|c| c.same_file(&d)) { 2 } else { 0 };
    r.spawn = p.spawn;
    r.spawn_parent = p.spawn_parent;
    Some(r)
}

/// Death reload (0x161a58 case 0x10 -> FUN_0022b490(world, current, ...)) as a post.
pub fn respawn_post(r: &crate::engine::player::p7_damage::Respawn) -> Post {
    Post { dest: Dest::Current, spawn: r.spawn, spawn_parent: r.spawn_parent, flags: r.flags, a: r.a as i32, slots: None }
}

/// Job restart / hub reload (0x165538 / 0x162e38 / 0x162a60) as a post; the slots carry the job
/// manager's current job / task / restart task over the load (0x165dc0 / 0x165de8 / 0x165e10).
pub fn job_post(m: &JobMgr, r: &JobReload, slots: &mut JobSlots) -> Post {
    crate::engine::game::job_flow::sync_slots(m, slots);
    let dest = match r.row {
        Some(row) => Dest::Row(row),
        None => Dest::Current,
    };
    Post { dest, spawn: r.spawn, spawn_parent: r.spawn_parent, flags: r.flags, a: r.a, slots: Some(*slots) }
}

/// FUN_00161478 (0x14d0b8 hook when the game time advanced): "dt" save variable += dt, then the current
/// job's timer. `dt_path` = DAT_003e0fac (None = unbound: nothing happens). `level_kind` = DAT_002e0c20.
pub fn FUN_00161478(dt: f32, vars: &SaveVars, dt_path: Option<&[u16]>, data: &mut GameData, jobs: Option<&mut JobMgr>, level_kind: f32) {
    FUN_00161478_full(dt, vars, dt_path, None, data, jobs, level_kind);
}

/// WP-J15: the 9 episodes of FUN_0020e3c0 (PTR_s_intro_002a39e8).
pub const EPISODES: [&str; 9] = ["intro", "nightclub", "palace", "temple", "prison", "castle", "train", "sawmill", "blimp"];

/// WP-J15: FUN_0020e3c0 episode part: world +0x3b58 (DAT_003e0fe8)[i] = path `episode <name> dt`, +0x3b7c
/// (DAT_003e100c)[i] = `episode <name> percent_complete` (FUN_001d88d0; None = a name not in the table).
pub fn episode_paths(vars: &SaveVars) -> [[Option<Vec<u16>>; 2]; 9] {
    let path = |names: &[&str]| -> Option<Vec<u16>> { names.iter().map(|n| u16::try_from(vars.ee_sub_1d8720(n)).ok()).collect() };
    std::array::from_fn(|i| [path(&["episode", EPISODES[i], "dt"]), path(&["episode", EPISODES[i], "percent_complete"])])
}

/// WP-J15: FUN_00210ae0: episode of the current level row (DAT_003e1c44[row], -1 without a row).
pub fn FUN_00210ae0(row: i32, table: Option<&LevelTable>) -> i32 {
    match (row, table) {
        (r, Some(t)) if r >= 0 => t.entries.get(r as usize).map_or(-1, |e| e.idx as i32),
        _ => -1,
    }
}

/// FUN_00161478 in full: "dt" += dt, then (episode < 9 with a bound path) `episode <name> dt` += dt, then
/// the current job's timer. `episode_dt` = DAT_003e0fe8[FUN_00210ae0()].
pub fn FUN_00161478_full(dt: f32, vars: &SaveVars, dt_path: Option<&[u16]>, episode_dt: Option<&[u16]>, data: &mut GameData, jobs: Option<&mut JobMgr>, level_kind: f32) {
    // 0x00161478
    let Some(p) = dt_path else { return };
    let add = |p: &[u16], data: &mut GameData| {
        let cur = match vars.ee_sub_1d7ed8(p, data) {
            SaveValue::Float(f) => f,
            _ => 0.0,
        };
        vars.ee_sub_1d80c8(p, SaveValue::Float(cur + dt), data);
    };
    add(p, data);
    if let Some(e) = episode_dt {
        add(e, data);
    }
    if let Some(m) = jobs {
        m.FUN_00161478(dt, level_kind);
    }
}

/// FUN_0014d1d0(scale, g_clock, 1) of the time-scale gadgets (0x1d 0.25 / 0x2a 1.35 / release 1.0):
/// applied when the JT's stored scale differs from clock slot 1 (DAT_002ddeec). 0 = no player published
/// (`GadgetHud::default`): ignored.
pub fn apply_time_scale(c: &mut Clock, scale: f32) -> bool {
    if scale > 0.0 && c.scales[1] != scale {
        clock::ee_sub_14d1d0(scale, c, 1);
        return true;
    }
    false
}

/// Save the live blob to slot `slot` (REPLACED FUN_001abd98 card dialog / FUN_001ac420 card write).
pub fn save_slot(card: &MemoryCard, slot: usize, vars: Option<&SaveVars>, data: &mut GameData) -> std::io::Result<()> {
    if let Some(v) = vars {
        v.ee_sub_1d7c80(data);
    }
    card.ee_sub_1ac420(slot, data)
}

/// 0x001ad320 (load menu, one slot): read the slot file, FUN_001d8d80(blob, 1) when "dt" == 0, then the
/// blob becomes the live data for FUN_00165e38 (the caller runs `ui::title::FUN_00165e38`).
pub fn FUN_001ad320(card: &MemoryCard, slot: usize, vars: &SaveVars, dt_path: Option<&[u16]>) -> Result<GameData, McError> {
    let mut d = card.ee_sub_1ac4c8(slot)?;
    let dt = dt_path.map(|p| vars.ee_sub_1d7ed8(p, &d));
    if matches!(dt, None | Some(SaveValue::Float(0.0))) {
        vars.ee_sub_1d8d80(&mut d, true, &[], None);
    }
    Ok(d)
}

/// WP-J16: 0x2a3828, the hideout descriptor (static row 23) FUN_00165e38 continues to.
pub const HIDEOUT_DESC: u32 = 0x002a_3828;

/// WP-J16: the save-variable handles FUN_00165e38 / FUN_001d82b8 read (world FUN_0020e3c0): +0x3b1c
/// DAT_003e0fac `dt`, +0x3b28 DAT_003e0fb8 `episodeCurrent`, +0x3b7c DAT_003e100c[9]
/// `episode <name> percent_complete`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StartPaths {
    pub dt: Option<Vec<u16>>,
    pub episode_current: Option<Vec<u16>>,
    pub percent: [Option<Vec<u16>>; 9],
}

impl StartPaths {
    pub fn new(vars: &SaveVars) -> Self {
        let one = |n: &str| u16::try_from(vars.ee_sub_1d8720(n)).ok().map(|s| vec![s]);
        let ep = episode_paths(vars);
        StartPaths { dt: one("dt"), episode_current: one("episodeCurrent"), percent: std::array::from_fn(|i| ep[i][1].clone()) }
    }
}

/// FUN_002581c0 + the CRef payload as a number (Int and Float slots; anything else 0).
fn num(vars: &SaveVars, p: Option<&[u16]>, data: &GameData) -> f32 {
    match p.map(|p| vars.ee_sub_1d7ed8(p, data)) {
        Some(SaveValue::Float(f)) => f,
        Some(SaveValue::Int(i)) => i as f32,
        Some(SaveValue::Bool(b)) => b as i32 as f32,
        _ => 0.0,
    }
}

/// WP-J16: DAT_003e100c[ep] read (`episode <name> percent_complete`; 0 when unbound / out of range).
pub fn episode_completion(vars: &SaveVars, paths: &StartPaths, data: &GameData, ep: i32) -> i32 {
    let p = usize::try_from(ep).ok().and_then(|i| paths.percent.get(i)).and_then(|p| p.as_deref());
    num(vars, p, data) as i32
}

/// WP-J16: DAT_002c2dc0, the weight of each episode in the game completion (FUN_001d82b8).
pub const EPISODE_WEIGHTS: [i32; 9] = [13, 86, 97, 147, 141, 160, 106, 182, 166];

/// WP-J16: 0x001d82b8: game completion percent of a save blob: sum(episode percent * weight) + 0x1a per
/// `fVault` flag set, over sum(weight * 100) + 0xd0; 0 = nothing, 100 = all, else clamped to 1..99.
pub fn FUN_001d82b8(vars: &SaveVars, paths: &StartPaths, data: &GameData) -> i32 {
    // 0x001d82b8
    let mut got = 0i32;
    let mut total = 0i32;
    for (i, &w) in EPISODE_WEIGHTS.iter().enumerate() {
        total += w * 100;
        if let Some(p) = paths.percent[i].as_deref() {
            got += num(vars, Some(p), data) as i32 * w;
        }
    }
    total += 0xd0;
    let vault = vars.ee_sub_1d8720("fVault") as i16;
    for r in vars.recs.iter().filter(|r| r.kind == 4 && r.sym == vault) {
        let b = |i: usize| data.data.get(r.offset as usize + i).copied().unwrap_or(0);
        if r.array == 0 {
            if b(0) != 0 {
                got += 0x1a;
            }
        } else {
            got += (0..r.array as usize).filter(|&k| b(k >> 3) & (1 << (k & 7)) != 0).count() as i32 * 0x1a;
        }
    }
    if got == 0 {
        0
    } else if got == total {
        100
    } else {
        (got * 100 / total).clamp(1, 99)
    }
}

/// WP-J16: FUN_00165e38 outcome.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Start {
    /// title listed and "dt" == 0: FUN_001d54b8 + FUN_001dd888 (hide the title)
    HideTitle,
    /// FUN_0022b490 request (already posted)
    Posted(Post),
}

/// WP-J16: 0x00165e38 in full (the continue branch with the hideout): `title` = FUN_001d9880(0x3da530).
/// Not "dt" == 0: level FUN_00210b98(episodeCurrent), or the hideout 0x2a3828 when that slot is invalid
/// or DAT_003e100c[episode] > 99; spawn 0x189, flags 0x40, slots -1 (DAT_002deb4c = the blob: `slot`).
pub fn FUN_00165e38_full(title: bool, vars: &SaveVars, paths: &StartPaths, data: &GameData, table: Option<&LevelTable>, slot: i32) -> Start {
    // 0x00165e38
    let dt = num(vars, paths.dt.as_deref(), data);
    if dt == 0.0 {
        if title {
            return Start::HideTitle;
        }
        let slots = JobSlots { job: -1, task: -1, restart: 0x62f, slot };
        let p = Post { dest: Dest::Slot(0), spawn: 0x189, spawn_parent: -1, flags: 0x48, a: -1, slots: Some(slots) };
        FUN_0022b490(p);
        return Start::Posted(p);
    }
    let ep = num(vars, paths.episode_current.as_deref(), data) as i32;
    let valid = table.is_some_and(|t| FUN_00210b98(t, ep).is_some());
    let pct = if (0..9).contains(&ep) { episode_completion(vars, paths, data, ep) } else { 0 };
    let dest = if 99 < pct || !valid { Dest::Static(HIDEOUT_DESC) } else { Dest::Slot(ep) };
    let slots = JobSlots { job: -1, task: -1, restart: -1, slot };
    let p = Post { dest, spawn: 0x189, spawn_parent: -1, flags: 0x40, a: -1, slots: Some(slots) };
    FUN_0022b490(p);
    Start::Posted(p)
}
