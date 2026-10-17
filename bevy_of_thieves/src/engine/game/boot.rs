//! WP-J15: the boot transition queue 0x2e1e88 (main FUN_001613a8 -> FUN_001a4e08; also the "quit to title"
//! FUN_001660f8) 1:1: +0 count, +4 entries (0x1c bytes: kind, movie desc, level desc, spawn, flags, two
//! FUN_001651d8 args; max 0x10, overflow -> dummy DAT_002e1e68), +0x1c4 next entry.
//! The original boot is NOT "load the first level of the static table": FUN_001a4e08 queues
//!   0: the intro movie 0x2a3868 (static row 24) + a transition to the zero descriptor 0x2bfca0 (no level),
//!   5: the memory-card check (FUN_001acc38(0x2e6550, 1) when the card manager is idle),
//!   2: level 0x2a38a8 (static row 25 = level-table slot 0, disc level 0) at spawn 0x189, flags 1, a 0x30a,
//! and pops the first one. FUN_0020e890 pops the next one after every transition whose descriptor has
//! size 0 (`TransitionOutcome::Loaded { reset_only: true }`): movie -> card check + level 0. Level 0 holds
//! the title screen (sequence oid 1672 with the field-694 action 0x44 = FUN_001dd820 0x3da530).
#![allow(non_snake_case)]

use std::sync::atomic::{AtomicBool, Ordering};

use super::flow::{self, Dest, Post, ZERO_DESC};

/// 0x2a3868: static row 24, the intro movie.
pub const BOOT_MOVIE: u32 = 0x002a_3868;
/// 0x2a38a8: static row 25, level 0 (level-table slot 0).
pub const BOOT_LEVEL: u32 = 0x002a_38a8;
/// FUN_001a4fb0 capacity.
pub const QUEUE_CAP: usize = 0x10;

/// One 0x1c-byte entry: [kind, movie desc, level desc, spawn, flags, 0x1651d8 a, 0x1651d8 b].
pub type BootEntry = [u32; 7];

/// 0x2e1e88.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BootQueue {
    /// +0 count / +4 entries
    pub entries: Vec<BootEntry>,
    /// +0x1c4
    pub next: usize,
}

/// Engine services of FUN_001a4e80.
pub trait BootHost {
    /// FUN_001b3a48(0x2f66b0, desc): queue a movie.
    fn ee_sub_1b3a48(&mut self, desc: u32);
    /// DAT_002f67b8 = 1 (kind 1 movies)
    fn set_2f67b8(&mut self) {}
    /// FUN_0022b490(0x3dd490, ..)
    fn post(&mut self, p: Post) {
        flow::FUN_0022b490(p);
    }
    /// FUN_001651d8(0x2deb30, a, b)
    fn ee_sub_1651d8(&mut self, _a: i32, _b: i32) {}
    /// DAT_002e6594 (card manager 0x2e6550 +0x44 state)
    fn card_state(&self) -> i32 {
        0
    }
    /// FUN_001acc38(0x2e6550, s): card manager state change
    fn ee_sub_1acc38(&mut self, _s: i32) {}
}

/// FUN_001a5020 / FUN_001a5078 descriptor test: non-null and w1 != w15 (size != 0).
fn desc_ok(addr: u32) -> bool {
    addr != 0 && flow::static_desc(addr).is_some_and(|d| d.size() != 0)
}

impl BootQueue {
    /// FUN_001a4fb0(q, kind): append a zeroed entry; None = full (the dummy DAT_002e1e68 absorbs the writes).
    pub fn FUN_001a4fb0(&mut self, kind: u32) -> Option<&mut BootEntry> {
        // 0x001a4fb0
        if self.entries.len() < QUEUE_CAP {
            self.entries.push([kind, 0, 0, 0, 0, 0, 0]);
            self.entries.last_mut()
        } else {
            None
        }
    }

    /// FUN_001a5020(q, desc): movie entry.
    pub fn FUN_001a5020(&mut self, desc: u32) {
        // 0x001a5020
        if desc_ok(desc) {
            if let Some(e) = self.FUN_001a4fb0(0) {
                e[1] = desc;
            }
        }
    }

    /// FUN_001a50f8(q): memory-card check entry.
    pub fn FUN_001a50f8(&mut self) {
        // 0x001a50f8
        self.FUN_001a4fb0(5);
    }

    /// FUN_001a5078(q, desc, spawn, flags): level entry.
    pub fn FUN_001a5078(&mut self, desc: u32, spawn: u32, flags: u32) {
        // 0x001a5078
        if desc_ok(desc) {
            if let Some(e) = self.FUN_001a4fb0(2) {
                e[4] = flags;
                e[2] = desc;
                e[3] = spawn;
            }
        }
    }

    /// FUN_001a4e70: entries left.
    pub fn FUN_001a4e70(&self) -> bool {
        // 0x001a4e70
        self.next < self.entries.len()
    }

    /// FUN_001a4e08: reset, queue the boot sequence, run its first entry.
    pub fn FUN_001a4e08(&mut self, h: &mut dyn BootHost) {
        // 0x001a4e08
        self.entries.clear();
        self.next = 0;
        self.FUN_001a5020(BOOT_MOVIE);
        self.FUN_001a50f8();
        self.FUN_001a5078(BOOT_LEVEL, 0x189, 1);
        self.FUN_001a4e80(h);
    }

    /// FUN_001a4e80: pop and run one entry.
    pub fn FUN_001a4e80(&mut self, h: &mut dyn BootHost) {
        // 0x001a4e80
        if !self.FUN_001a4e70() {
            return;
        }
        let e = self.entries[self.next];
        self.next += 1;
        match e[0] {
            0 | 1 => {
                h.ee_sub_1b3a48(e[1]);
                if e[0] == 1 {
                    h.set_2f67b8();
                }
                h.post(Post { dest: Dest::Static(ZERO_DESC), spawn: 0x189, spawn_parent: -1, flags: 0, a: -1, slots: None });
            }
            2 => h.post(Post { dest: Dest::Static(e[2]), spawn: e[3] as i32, spawn_parent: -1, flags: e[4], a: 0x30a, slots: None }),
            3 => h.ee_sub_1651d8(e[5] as i32, e[6] as i32),
            5 => {
                if h.card_state() == 0 {
                    h.ee_sub_1acc38(1);
                }
                self.FUN_001a4e80(h);
            }
            _ => {}
        }
    }
}

static RESTART: AtomicBool = AtomicBool::new(false);

/// FUN_001660f8 (pause menu "quit" with no card slot in use, card dialog 0x1accc8): FUN_001a4e08(0x2e1e88).
/// Posted: the binary owns the queue and runs FUN_001a4e08 on its next level transition (`take_restart`).
pub fn FUN_001660f8() {
    RESTART.store(true, Ordering::SeqCst);
}

/// FUN_001660f8 requests since the last call.
pub fn take_restart() -> bool {
    RESTART.swap(false, Ordering::SeqCst)
}
