//! WP-EP1e: the game side of the memory-card dialog 0x2e6550 (`card_dialog::CardDialog`) for the in-game
//! "Save Game..." / "Load Game..." slot menus and "Quit Game": FUN_001ad480 / FUN_001ad3e0 / FUN_001ad320 /
//! FUN_001b0878 / FUN_001abd40, with the card manager replaced by the slot files of `game::save::MemoryCard`.
//!
//! REPLACED: the card thread (0x2f633c) runs its job synchronously (never busy, card state 6 = ready); the
//! dialog screen (string 0x52c) is drawn as a caption with port text (`msg_text`); DAT_002f6330 (slot in use)
//! = `SaveSlots::slot` + `CardApp::in_use` (a new game / continue / --table boot always has a slot).
#![allow(non_snake_case)]

use super::card_dialog::{CardDialog, CardDialogHost};
use super::settings::{SavePaths, F_AUTOSAVE_WARNED};
use crate::engine::game::save::{GameData, MemoryCard, SaveValue, SaveVars, SLOT_COUNT};
use crate::engine::input::pad::{self, PadState};

/// Dialog state kept across frames.
#[derive(Clone, Debug)]
pub struct CardApp {
    pub dlg: CardDialog,
    /// screen +0x5c: time the current message was shown
    pub shown: f32,
    /// DAT_002f6330 != 0
    pub in_use: bool,
    /// DAT_002f6338 (slot index)
    pub f6338: Option<usize>,
    /// DAT_002f634c: the last card job failed
    pub error: bool,
    /// verification log: last logged message
    pub logged: i32,
}

impl Default for CardApp {
    fn default() -> Self {
        // the dialog's screen object exists and pad 0 is bound; +0x44 mode 1 = card present (the slot
        // directory is always there)
        CardApp { dlg: CardDialog { screen: true, pad: true, mode: 1, ..Default::default() }, shown: 0.0, in_use: true, f6338: None, error: false, logged: -1 }
    }
}

/// Port text of the dialog messages (REPLACED: screen string 0x52c).
pub fn msg_text(msg: i32) -> Option<&'static str> {
    Some(match msg {
        0x1a => "Saving... Please do not quit the game.",
        0x1c => "Overwrite this saved game?  Square: Yes   Triangle: No",
        0x1d | 0x23 => "The save file could not be written.  Square: OK",
        0x22 => "Sly 2 auto-saves your progress to this slot.  Square: OK",
        0x20 => "Delete this saved game?  Square: Yes   Triangle: No",
        _ => return None,
    })
}

/// What the dialog asked the menu / game to do this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CardOut {
    pub close_all: bool,
    pub pop: bool,
    pub quit: bool,
    pub saved: Option<usize>,
}

impl CardOut {
    pub fn merge(&mut self, o: CardOut) {
        self.close_all |= o.close_all;
        self.pop |= o.pop;
        self.quit |= o.quit;
        self.saved = o.saved.or(self.saved);
    }
}

/// Verification log of a blob: `dt`, coins, byte sum of the variable data and the signature.
pub fn blob_summary(vars: &SaveVars, paths: &SavePaths, d: &GameData) -> String {
    let get = |p: Option<&[u16]>| p.map(|p| vars.ee_sub_1d7ed8(p, d));
    let sum: u64 = d.data.iter().map(|&b| b as u64).sum();
    format!("dt {:?} coins {:?} data sum {sum:#x} version {:#x} checksum {:#x}", get(paths.dt.as_deref()), get(paths.coins.as_deref()), d.version, d.checksum)
}

/// `dt` of a slot file (0 = empty / unreadable): the FUN_001acc38 card refresh of the slot copies.
pub fn slot_dt(card: &MemoryCard, slot: usize, vars: &SaveVars, paths: &SavePaths) -> f32 {
    let Some(p) = paths.dt.as_deref() else { return 0.0 };
    match card.ee_sub_1ac4c8(slot) {
        Ok(d) if vars.ee_sub_1d7d30(&d) => match vars.ee_sub_1d7ed8(p, &d) {
            SaveValue::Float(f) => f,
            _ => 0.0,
        },
        _ => 0.0,
    }
}

/// All slots' `dt` (MenuMgr::slots).
pub fn slots_dt(card: &MemoryCard, vars: &SaveVars, paths: &SavePaths) -> [f32; 3] {
    std::array::from_fn(|i| if i < SLOT_COUNT { slot_dt(card, i, vars, paths) } else { 0.0 })
}

/// FUN_001abd40(0x2e65e0): FUN_001d54b8 (vibration option into the live blob), FUN_001d84b0 (slot copy with
/// `from_memcard` set) and the card job DAT_002a6c90 (slot file write).
pub fn FUN_001abd40(card: &MemoryCard, slot: usize, vars: &SaveVars, paths: &SavePaths, vibration: i32, live: &mut GameData) -> std::io::Result<()> {
    super::title::FUN_001d54b8(vibration, vars, paths, live);
    let copy = vars.ee_str_from_memcard_1d84b0(live);
    card.ee_sub_1ac420(slot, &copy)
}

pub struct Host<'a> {
    pub app: &'a mut CardApp,
    pub now: f32,
    pub pad: &'a mut PadState,
    pub card: &'a MemoryCard,
    pub slot: &'a mut usize,
    pub vars: &'a SaveVars,
    pub paths: &'a SavePaths,
    pub data: &'a mut GameData,
    pub vibration: i32,
    pub title: bool,
    /// FUN_001b0108 / FUN_001b0160: the top menu is a save-slot menu
    pub save_menu: bool,
    pub purpose: Option<i32>,
    pub out: CardOut,
}

impl CardDialogHost for Host<'_> {
    fn now(&self) -> f32 {
        self.now
    }
    fn screen_ready(&self) -> Option<f32> {
        Some(self.app.shown)
    }
    fn world_busy(&self) -> bool {
        false
    }
    fn pressed(&mut self, mask: u32) -> bool {
        pad::FUN_00187d80(self.pad, mask) != 0
    }
    fn thread_busy(&self) -> bool {
        false
    }
    fn thread_error(&self) -> bool {
        self.app.error
    }
    fn start_job(&mut self, _job: u32) {
        // format / create / delete jobs: the slot directory is created on write
        self.app.error = false;
    }
    fn card_state(&self) -> i32 {
        6
    }
    fn screen_show(&mut self, _msg: i32) {
        self.app.shown = self.now;
    }
    fn screen_hide(&mut self) {}
    fn set_f6330_from_f6338(&mut self) {
        if let Some(s) = self.app.f6338 {
            *self.slot = s;
            self.app.in_use = true;
        }
    }
    fn clear_f6330(&mut self) {
        self.app.in_use = false;
    }
    fn clear_f6338(&mut self) {
        self.app.f6338 = None;
    }
    fn set_f6338(&mut self, slot: i32) {
        self.app.f6338 = usize::try_from(slot).ok().filter(|&s| s < SLOT_COUNT);
    }
    fn set_e659c(&mut self, v: i32) {
        self.purpose = Some(v);
    }
    fn autosave_warned(&self) -> i32 {
        match self.paths.flags[F_AUTOSAVE_WARNED].as_deref().map(|p| self.vars.ee_sub_1d7ed8(p, self.data)) {
            Some(SaveValue::Bool(b)) => b as i32,
            Some(SaveValue::Int(i)) => i,
            _ => 0,
        }
    }
    fn set_autosave_warned(&mut self) {
        if let Some(p) = self.paths.flags[F_AUTOSAVE_WARNED].as_deref() {
            self.vars.ee_sub_1d80c8(p, SaveValue::Bool(true), self.data);
        }
    }
    fn slot_dt(&self, slot: i32) -> f32 {
        usize::try_from(slot).map_or(0.0, |s| slot_dt(self.card, s, self.vars, self.paths))
    }
    fn save(&mut self) {
        let r = FUN_001abd40(self.card, *self.slot, self.vars, self.paths, self.vibration, self.data);
        self.app.error = r.is_err();
        match r {
            Ok(()) => {
                // verification: the slot file as written (the FUN_001d84b0 copy) for comparison with the load log
                let file = self.card.ee_sub_1ac4c8(*self.slot).map(|d| blob_summary(self.vars, self.paths, &d));
                log::info!("card: saved slot {} ({}): {:?}", *self.slot + 1, self.card.slot_path(*self.slot).display(), file);
                self.out.saved = Some(*self.slot);
            }
            Err(e) => log::warn!("card: save slot {} failed: {e}", *self.slot + 1),
        }
    }
    fn title_active(&self) -> bool {
        self.title
    }
    fn menu_b0160(&self) -> bool {
        self.save_menu
    }
    fn is_save_menu(&self) -> bool {
        self.save_menu
    }
    fn menu_close_all(&mut self) {
        self.out.close_all = true;
    }
    fn menu_pop(&mut self) {
        self.out.pop = true;
    }
    fn new_game(&mut self) {}
    fn quit_to_title(&mut self) {
        self.out.quit = true;
    }
}

impl Host<'_> {
    /// Run `f` on the dialog with this host; the dialog's +0x4c (DAT_002e659c) writes are applied after.
    pub fn run(&mut self, f: impl FnOnce(&mut CardDialog, &mut Self)) {
        let mut d = self.app.dlg;
        f(&mut d, self);
        if let Some(p) = self.purpose.take() {
            d.purpose = p;
        }
        self.app.dlg = d;
    }

    /// FUN_001b0878 "Yes, Quit" with a slot in use: save (purpose 1 = quit to title afterwards), or the
    /// autosave notice 0x22 first. Returns false when no slot is in use (FUN_001660f8 directly).
    pub fn FUN_001b0878(&mut self) -> bool {
        if !self.app.in_use {
            return false;
        }
        self.run(|d, h| {
            d.purpose = 1;
            if h.autosave_warned() == 0 {
                d.FUN_001acb58(h, 0x22);
            } else {
                d.FUN_001acb58(h, 0x1a);
                h.save();
            }
        });
        true
    }
}
