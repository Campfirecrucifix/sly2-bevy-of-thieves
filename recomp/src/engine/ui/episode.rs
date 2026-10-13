//! WP-A13: episode select screen, class 131 (vtable 0x2ceeb8, size 0x140, parent 126 = the ASEGA label
//! screen). One instance in the game (level 1 of the disc order, oid 144117); DAT_003e1c04 = the instance.
//!
//! 1:1: FUN_0015cd68 (init slot 7), FUN_0015cda8 (post-load slot 15: unlocked episode count +0x118 from the
//! episode save variables, start mode), FUN_0015cf30 (per frame slot 16: the mode machine below),
//! FUN_0015d648 (set mode / sub-row), FUN_0015d5c8 (label of the mode), FUN_0015d880 (hint label 0x5be /
//! 0x5bd), FUN_0015d798 (last replay row), FUN_0015da88 (odd / even part string 0x77 / 0x78). The tables
//! 0x2be080 / 0x2be1b8 / 0x2be248 / 0x2be278 / 0x2be2a8 / 0x2be2d0 are copied from the ELF (checked by
//! tests/engine_a13.rs).
//!
//! Modes (+0x11c): -1 off, 0 open, 1 / 3 / 5 / 9 transition labels, 2 browse episodes (+0x110, up / down;
//! cross -> 7, triangle -> 6 back to episode 0, square on the hint -> episode cutscene), 4 / 6 scroll to
//! +0x114, 7 -> 8, 8 episode page (+0x120 = -1: cross 10 start / triangle 9 back; +0x120 = 0: replay row
//! +0x128 left / right, cross = replay the level 0x2be200[row], 1 / 2 = row scroll labels), 10 start the
//! episode (intro movie object 0x2be2a8[ep] when never played, else FUN_0022b490 to FUN_00210b98(ep)).
//! After a label (FUN_002121e8 == 0) the mode advances by 0x2be278.
//!
//! REPLACED: the class-126 label base (FUN_002123c8 / FUN_00212c50 / FUN_002128e8 / FUN_00212a50 / a70 /
//! FUN_00213b00 / FUN_00213300 draw): label requests are `EpEvent::Label` / `Anim`, "label playing"
//! (FUN_002121e8) is the caller's `busy`; sounds (FUN_001fbb90 + FUN_001fb2f8), FUN_001b3a48(0x2f66b0, desc),
//! FUN_0022b490 transitions, FUN_001abd98(0x2e65e0, 1), FUN_0015e938 + FUN_00228358 / FUN_002283d0 movie
//! objects are returned as `EpEvent`s. FUN_002581c0 save reads = `EpIn::dt` / `pct` (None = unbound path).
#![allow(non_snake_case)]
use crate::engine::input::pad::{self, Clock, PadState};

/// 0x2be080: label per [episode][mode] (-1 none)
pub const LABELS_2BE080: [[i32; 11]; 9] = [
    [783, 1410, 1411, 1412, 1413, 1414, 1415, 1416, -1, 1417, -1],
    [783, 1338, 1339, 1340, 1341, 1342, 1343, 1344, 1345, 1346, 1337],
    [783, 1347, 1348, 1349, 1350, 1351, 1352, 1353, 1354, 1355, 1337],
    [783, 1356, 1357, 1358, 1359, 1360, 1361, 1362, 1363, 1364, 1337],
    [783, 1365, 1366, 1367, 1368, 1369, 1370, 1371, 1372, 1373, 1337],
    [783, 1374, 1375, 1376, 1377, 1378, 1379, 1380, 1381, 1382, 1337],
    [783, 1383, 1384, 1385, 1386, 1387, 1388, 1389, 1390, 1391, 1337],
    [783, 1392, 1393, 1394, 1395, 1396, 1397, 1398, 1399, 1400, 1337],
    [783, 1401, 1402, 1403, 1404, 1405, 1406, 1407, 1408, 1409, 1337],
];
/// 0x2be1b8: mode-8 label per [replay row +0x128][sub +0x120]
pub const LABELS_2BE1B8: [[i32; 3]; 17] = [
    [1419, 1420, -1], [1421, 1422, 1423], [1424, 1425, 1426], [1427, 1428, 1429], [1430, 1431, 1432],
    [1433, 1434, 1435], [1436, 1437, 1438], [1439, 1440, 1441], [1442, 1443, 1444], [1445, 1446, 1447],
    [1448, 1449, 1450], [1451, 1452, 1453], [1454, 1455, 1456], [1457, 1458, 1459], [1460, 1461, 1462],
    [1463, 1464, 1465], [1466, -1, 1467],
];
/// 0x2be200: level descriptor (64 bytes, ELF address) of each replay row
pub const REPLAY_2BE200: [u32; 17] = [
    0x2a3268, 0x2a32a8, 0x2a32e8, 0x2a3328, 0x2a3368, 0x2a33a8, 0x2a33e8, 0x2a3428, 0x2a3468, 0x2a34a8,
    0x2a34e8, 0x2a3528, 0x2a3568, 0x2a35a8, 0x2a35e8, 0x2a3628, 0x2a3668,
];
/// 0x2be248: episode cutscene level descriptor (0 = none)
pub const CUTSCENE_2BE248: [u32; 9] = [0, 0x2a36a8, 0, 0x2a36e8, 0, 0x2a3728, 0, 0x2a3768, 0x2a37a8];
/// 0x2be278: next mode once the label of mode i ended
pub const NEXT_2BE278: [i32; 11] = [1, 2, -1, 2, 3, 2, 5, 8, -1, 2, -1];
/// 0x2be2a8: intro movie object oid (FUN_0015e938 kind 5) per episode
pub const INTRO_2BE2A8: [i32; 9] = [-1, 1329, 1330, 1331, 1332, 1333, 1334, 1335, 1336];
/// 0x2be2d0: last replay row per episode (all done)
pub const ROWS_2BE2D0: [i32; 9] = [-1, 2, 4, 6, 8, 10, 12, 14, 16];

/// Per-frame inputs (engine globals the class reads).
#[derive(Clone, Debug, Default)]
pub struct EpIn {
    /// DAT_003e0fe8[i]: `episode <name> dt` value (None = unbound path)
    pub dt: [Option<f32>; 9],
    /// DAT_003e100c[i]: `episode <name> percent_complete` value
    pub pct: [Option<f32>; 9],
    /// DAT_002be26c (all episodes open; 0 in the shipped game)
    pub all_open: bool,
    /// FUN_002121e8: the class-126 label is still playing
    pub busy: bool,
    /// FUN_001d9880(0x3d9ed0 / 0x2e6550 / 0x3da4f0): menu, card dialog or another screen active
    pub blocked: bool,
    /// oids of the level's objects (FUN_0015e938(0x3dd490, 5, oid, 0) finds the intro movie object)
    pub oids: Vec<i32>,
}

/// Engine requests of class 131 (A14: performed by `episode_event`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EpEvent {
    /// vslot 0x18c (FUN_002128e8): play label id
    Label(i32),
    /// FUN_00213b00(label) hint animation
    Anim(i32),
    /// FUN_001fbb90(id) + FUN_001fb2f8: 0x34 confirm, 0x36 refuse
    Sound(i32),
    /// FUN_001b3a48(0x2f66b0, desc) + FUN_0022b490(0x3dd490, 0, 0x189, -1, 0, -1): load that level
    LoadDesc(u32),
    /// FUN_0022b490(0x3dd490, FUN_00210b98(ep), 0x189, -1, 0, 0x30a): enter the episode
    EnterEpisode(i32),
    /// FUN_0015e938(0x3dd490, 5, oid, 0) + FUN_00228358(obj, 3) + FUN_002283d0(obj): intro movie
    IntroMovie(i32),
    /// FUN_001abd98(0x2e65e0, 1) (leaving mode 0)
    CardDialog,
    /// FUN_00187e78(0x2dfc00): pad repeat reset
    PadReset,
}

/// Class 131 instance (fields by offset).
#[derive(Clone, Debug)]
pub struct Ep131 {
    /// +0x110 current episode
    pub f_110: i32,
    /// +0x114 scroll target episode
    pub f_114: i32,
    /// +0x118 last open episode
    pub f_118: i32,
    /// +0x11c mode
    pub f_11c: i32,
    /// +0x120 mode-8 sub-row
    pub f_120: i32,
    /// +0x124 mode start time
    pub f_124: f32,
    /// +0x128 replay row
    pub f_128: i32,
    /// +300 hint state (-1 / 0 / 1)
    pub f_12c: i32,
    /// +0x130 hint state time
    pub f_130: f32,
    /// DAT_002be270: replay row to come back to (-1 none)
    pub g_2be270: i32,
    pub events: Vec<EpEvent>,
}

impl Ep131 {
    /// FUN_0015cd68 (FUN_001a5270 base init REPLACED)
    pub fn FUN_0015cd68() -> Self {
        Ep131 { f_110: -1, f_114: -1, f_118: 0, f_11c: -1, f_120: -1, f_124: 0.0, f_128: 0, f_12c: 0, f_130: 0.0, g_2be270: -1, events: Vec::new() }
    }

    /// FUN_0015cda8: count the open episodes (stop below the first not played whose predecessor is not
    /// complete), then mode 0 (or mode 8 row DAT_002be270 when coming back from a replay).
    pub fn FUN_0015cda8(&mut self, inp: &EpIn, t: f32) {
        // FUN_002123c8 (class-126 post-load) REPLACED
        self.f_118 = 8;
        if !inp.all_open {
            loop {
                if inp.dt[self.f_118 as usize].unwrap_or(0.0) != 0.0 {
                    break;
                }
                let p = inp.pct[(self.f_118 - 1) as usize].unwrap_or(0.0);
                if 99 < p as i32 {
                    break;
                }
                self.f_118 -= 1;
                if self.f_118 < 2 || inp.all_open {
                    break;
                }
            }
        }
        if self.g_2be270 == -1 {
            self.f_110 = self.f_118;
            self.FUN_0015d648(0, -1, t);
        } else {
            self.f_128 = self.g_2be270;
            self.f_110 = 0;
            self.g_2be270 = -1;
            self.FUN_0015d648(8, 0, t);
        }
        self.events.push(EpEvent::PadReset);
    }

    /// FUN_0015d5c8: label of the current mode (-1 none)
    pub fn FUN_0015d5c8(&self) -> i32 {
        if self.f_11c == -1 {
            return -1;
        }
        if self.f_11c == 8 && self.f_120 != -1 {
            return LABELS_2BE1B8.get(self.f_128 as usize).and_then(|r| r.get(self.f_120 as usize)).copied().unwrap_or(-1);
        }
        LABELS_2BE080.get(self.f_110 as usize).and_then(|r| r.get(self.f_11c as usize)).copied().unwrap_or(-1)
    }

    /// FUN_0015d880: hint state (0 -> label 0x5be, 1 -> 0x5bd)
    pub fn FUN_0015d880(&mut self, k: i32, t: f32) {
        if self.f_12c != k {
            match k {
                0 => self.events.push(EpEvent::Anim(0x5be)),
                1 => self.events.push(EpEvent::Anim(0x5bd)),
                _ => {}
            }
            self.f_12c = k;
            self.f_130 = t;
        }
    }

    /// FUN_0015d648: switch to (mode, sub-row)
    pub fn FUN_0015d648(&mut self, mode: i32, sub: i32, t: f32) {
        let cur = self.f_11c;
        if cur == mode && self.f_120 == sub {
            return;
        }
        match cur {
            2 => self.FUN_0015d880(0, t),
            0 => self.events.push(EpEvent::CardDialog),
            4 | 6 => {
                self.f_110 = self.f_114;
                self.f_114 = -1;
            }
            _ => {}
        }
        if mode == 2 {
            self.FUN_0015d880(-1, t);
            self.FUN_0015d880(0, t);
        }
        // +0x70 / +0x74 / +0x78 = -1, FUN_00212a50(1) / FUN_00212a70(1): class-126 label state REPLACED
        self.f_11c = mode;
        self.f_120 = sub;
        self.f_124 = t;
        let l = self.FUN_0015d5c8();
        if l != -1 {
            self.events.push(EpEvent::Label(l));
        }
    }

    /// FUN_0015d798: last replay row of the open episodes
    pub fn FUN_0015d798(&self, inp: &EpIn) -> i32 {
        if inp.all_open {
            return 0x10;
        }
        let i = self.f_118 as usize;
        let base = ROWS_2BE2D0[i];
        if inp.dt[i].unwrap_or(0.0) == 0.0 {
            base - 2
        } else if (inp.pct[i].unwrap_or(0.0) as i32) < 100 {
            base - 1
        } else {
            base
        }
    }

    /// FUN_0015da88: level string of the replay row part (odd rows 0x77, even rows 0x78; None otherwise)
    pub fn FUN_0015da88(&self, prev: bool) -> Option<i32> {
        let i = self.f_128 - prev as i32;
        match i {
            1 | 3 | 5 | 7 | 9 | 11 | 13 | 15 => Some(0x77),
            2 | 4 | 6 | 8 | 10 | 12 | 14 | 16 => Some(0x78),
            _ => None,
        }
    }

    fn refuse(&mut self) {
        self.events.push(EpEvent::Sound(0x36));
    }

    /// FUN_0015cf30: per frame (FUN_00212c50 class-126 tick REPLACED)
    pub fn FUN_0015cf30(&mut self, pad: &mut PadState, clock: Clock, inp: &EpIn) {
        if inp.blocked {
            return;
        }
        let t = clock.t;
        let mode = match self.f_11c {
            2 => {
                let r = pad::FUN_001880d8(pad, clock) as f32;
                if (r < 0.0 && self.f_110 < 1) || (r > 0.0 && self.f_118 <= self.f_110) {
                    return self.refuse();
                }
                if r != 0.0 {
                    self.f_114 = self.f_110 + if r < 0.0 { -1 } else { 1 };
                    if r > 0.0 { 4 } else { 6 }
                } else if pad::FUN_00187d80(pad, 0x40) != 0 {
                    7
                } else if self.f_110 != 0 && pad::FUN_00187d80(pad, 0x10) != 0 {
                    self.f_114 = 0;
                    6
                } else if self.f_12c != 0 {
                    if self.f_12c != 1 {
                        return;
                    }
                    if pad::FUN_00187d80(pad, 0x80) == 0 {
                        if 2.0 < t - self.f_130 {
                            self.FUN_0015d880(0, t);
                        }
                        return;
                    }
                    self.events.push(EpEvent::LoadDesc(CUTSCENE_2BE248[self.f_110 as usize]));
                    self.events.push(EpEvent::Sound(0x34));
                    return;
                } else {
                    let ep = self.f_110 as usize;
                    let full = inp.pct[ep].is_some_and(|p| 99 < p as i32) || inp.all_open;
                    if CUTSCENE_2BE248[ep] != 0 && full && 10.0 < t - self.f_130 {
                        self.FUN_0015d880(1, t);
                    }
                    return;
                }
            }
            -1 => return,
            8 => {
                let sub = self.f_120;
                if sub != 0 {
                    if sub > 0 {
                        if sub <= 2 && !inp.busy {
                            self.FUN_0015d648(8, 0, t);
                        }
                        return;
                    }
                    if sub != -1 {
                        return;
                    }
                    if pad::FUN_00187d80(pad, 0x40) != 0 {
                        10
                    } else if pad::FUN_00187d80(pad, 0x10) != 0 {
                        9
                    } else {
                        return;
                    }
                } else if pad::FUN_00187d80(pad, 0x40) != 0 {
                    self.g_2be270 = self.f_128;
                    self.events.push(EpEvent::LoadDesc(REPLAY_2BE200[self.g_2be270 as usize]));
                    self.events.push(EpEvent::Sound(0x34));
                    return;
                } else if pad::FUN_00187d80(pad, 0x10) != 0 {
                    9
                } else {
                    let r = pad::FUN_00187e88(pad, clock) as f32;
                    if r < 0.0 && self.f_128 <= 0 {
                        return self.refuse();
                    }
                    if r > 0.0 && self.FUN_0015d798(inp) <= self.f_128 {
                        return self.refuse();
                    }
                    if r != 0.0 {
                        self.f_128 += if r < 0.0 { -1 } else { 1 };
                        self.FUN_0015d648(8, if r > 0.0 { 2 } else { 1 }, t);
                    }
                    return;
                }
            }
            10 => {
                if inp.busy {
                    return;
                }
                let ep = self.f_110 as usize;
                // never played: the episode's intro movie object (none found -> enter directly)
                if inp.dt[ep].is_some_and(|v| v == 0.0) && inp.oids.contains(&INTRO_2BE2A8[ep]) {
                    self.events.push(EpEvent::IntroMovie(INTRO_2BE2A8[ep]));
                } else {
                    self.events.push(EpEvent::EnterEpisode(self.f_110));
                }
                -1
            }
            m => {
                // LAB_0015d554: label ended -> next mode
                if inp.busy {
                    return;
                }
                let next = NEXT_2BE278.get(m as usize).copied().unwrap_or(-1);
                let mut sub = -1;
                if next == 8 && self.f_110 == 0 {
                    self.f_128 = 0;
                    sub = 0;
                }
                self.FUN_0015d648(next, sub, t);
                return;
            }
        };
        self.FUN_0015d648(mode, -1, t);
    }
}

/// WP-A14: FUN_001b3a48(0x2f66b0, desc): at most 4 movies, descriptors of size 0 ignored.
pub fn queue_movie(q: &mut crate::engine::game::fmv::MovieQueue, d: crate::engine::disc::descriptor::Descriptor) {
    // 0x001b3a48
    if d.size() != 0 && q.movies.len() < 4 {
        q.movies.push(d);
    }
}

/// A14: class-131 requests wired to the engine (FUN_0022b490 posts are taken by J16's transition):
/// LoadDesc = FUN_001b3a48(0x2f66b0, desc) + FUN_0022b490(null desc = current level, 0x189, -1, 0, -1)
/// (theater / cutscene movie through the UI queue -> FmvPlayback); EnterEpisode =
/// FUN_0022b490(FUN_00210b98(ep), 0x189, -1, 0, 0x30a). IntroMovie = FUN_00228358(o, 3) + FUN_002283d0(o) on
/// the warp object (WP-EP1d: queued to `classes::c2dc6f0::post_warp_start`, run by the class set). Labels / anims /
/// sounds / card dialog stay logged (ui::plugin). Movies are played by the caller (FUN_001b3b08).
pub fn episode_event(
    movies: &mut crate::engine::game::fmv::MovieQueue,
    ev: EpEvent,
    _ep: i32,
    p: &mut crate::engine::input::pad::PadState,
) {
    use crate::engine::game::flow::{self, Dest, Post};
    let post = |dest, a| flow::FUN_0022b490(Post { dest, spawn: 0x189, spawn_parent: -1, flags: 0, a, slots: None });
    match ev {
        EpEvent::LoadDesc(d) => {
            if let Some(m) = flow::static_desc(d) {
                queue_movie(movies, m);
            }
            post(Dest::Current, -1);
        }
        EpEvent::EnterEpisode(e) => post(Dest::Slot(e), 0x30a),
        // WP-EP1d: 1:1 FUN_00228358(o, 3) + FUN_002283d0(o) on the intro warp object (classes::c2dc6f0)
        EpEvent::IntroMovie(oid) => crate::engine::classes::c2dc6f0::post_warp_start(oid),
        EpEvent::PadReset => crate::engine::input::pad::FUN_00187e78(p),
        _ => {}
    }
}
