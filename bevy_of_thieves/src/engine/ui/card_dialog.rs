//! WP-X1: memory-card message dialog 0x2e6550 (vtable 0x2d3750, ctor FUN_001aca98, screen string 0x52c)
//! = the "card manager" UI on top of the card manager 0x2e65e0 (`game::world_update::ee_sub_1ab1a0`).
//!
//! FUN_001acb58(msg) shows message `msg` (+0x40; vtable +0x54 show, screen object +0x24 slot 0x18c),
//! FUN_001acbb8 hides it (+0x40 = -1). FUN_001accc8 (per frame from FUN_001d9648) waits until the screen
//! object is fully shown (+0x58 == 2, +0x64 == 2, > 0.1 s since +0x5c) and the world is not loading, then:
//! X (0x80) on a 1-button message -> FUN_001acfe0 (acknowledge); on a 2-button message X -> FUN_001ad168
//! (yes) and triangle (0x10) -> FUN_001acfe0 (no); otherwise the message's own progress step (card thread
//! 0x2f633c finished -> next message / close). The card thread jobs (DAT_002a6d88 format, 0x2a6e88,
//! 0x2a6e90, 0x2a6c90 save) and the save blob writes are host calls (`CardDialogHost`, game side; the
//! memory card itself is `game::save::MemoryCard`).
#![allow(non_snake_case)]

/// Card thread jobs FUN_001aaf90(thread, job) (job descriptor addresses).
pub const JOB_2A6D88: u32 = 0x2a6d88;
pub const JOB_2A6E88: u32 = 0x2a6e88;
pub const JOB_2A6E90: u32 = 0x2a6e90;
/// FUN_001abd40 (save): FUN_001aaf90(0x2e65e0 + 0xfd5c, DAT_002a6c90)
pub const JOB_2A6C90: u32 = 0x2a6c90;

/// Dialog object 0x2e6550 fields the logic uses.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardDialog {
    /// +0x40 shown message (-1 = none; FUN_001aca98 init)
    pub msg: i32,
    /// +0x24 != 0: the screen object exists
    pub screen: bool,
    /// +0x10 != 0: a pad is bound
    pub pad: bool,
    /// +0x28 / +0x2c (FUN_001acc18)
    pub sel: i32,
    pub sel_time: f32,
    /// +0x44 mode / +0x48 time of change (FUN_001acc38)
    pub mode: i32,
    pub mode_time: f32,
    /// +0x4c purpose (1 = quit to title after save, 2 = close menus after save, 3 = autosave warning path)
    pub purpose: i32,
}

impl Default for CardDialog {
    fn default() -> Self {
        // FUN_001aca98: +0x40 = -1
        CardDialog { msg: -1, screen: false, pad: false, sel: 0, sel_time: 0.0, mode: 0, mode_time: 0.0, purpose: 0 }
    }
}

/// Engine state / calls of the dialog functions.
pub trait CardDialogHost {
    /// DAT_002ddf0c
    fn now(&self) -> f32;
    /// screen object +0x58 == 2 && +0x64 == 2, and its +0x5c show time
    fn screen_ready(&self) -> Option<f32>;
    /// FUN_0020e040(0x3dd490)
    fn world_busy(&self) -> bool;
    /// FUN_00187d80(pad, mask)
    fn pressed(&mut self, mask: u32) -> bool;
    /// FUN_001ab088(0x2f633c): the dialog's card thread is running
    fn thread_busy(&self) -> bool;
    /// DAT_002f634c: result of the last thread job (!= 0 = error)
    fn thread_error(&self) -> bool;
    /// FUN_001aaf90(0x2f633c, job)
    fn start_job(&mut self, job: u32);
    /// DAT_002e65e8 card manager state
    fn card_state(&self) -> i32;
    /// vtable +0x54 show / +0x5c hide (screen object slots 0x18c / 0x194)
    fn screen_show(&mut self, msg: i32);
    fn screen_hide(&mut self);
    /// DAT_002f6330 / DAT_002f6338 (selected save slot pointer, 0 = none)
    fn set_f6330_from_f6338(&mut self);
    fn clear_f6330(&mut self);
    fn clear_f6338(&mut self);
    /// DAT_002f6338 = save slot copy `slot` (0x2e6604 + slot * 0x5464)
    fn set_f6338(&mut self, slot: i32);
    /// DAT_002e659c
    fn set_e659c(&mut self, v: i32);
    /// FUN_002581c0(DAT_003e0fe4 "fAutosaveWarned", 0x3d4a60) payload
    fn autosave_warned(&self) -> i32;
    /// FUN_00258200(DAT_003e0fe4, Bool 1, 0x3d4a60)
    fn set_autosave_warned(&mut self);
    /// FUN_002581c0(DAT_003e0fac "dt", slot copy) as f32
    fn slot_dt(&self, slot: i32) -> f32;
    /// FUN_001abd40(0x2e65e0): save (FUN_001d54b8, FUN_001d84b0, job DAT_002a6c90)
    fn save(&mut self);
    /// FUN_001d9880(0x3da530): title screen active
    fn title_active(&self) -> bool;
    /// FUN_001b0160(0x3d9ed0) / FUN_001b0108(0x3d9ed0) (IsSaveMenu)
    fn menu_b0160(&self) -> bool;
    fn is_save_menu(&self) -> bool;
    /// FUN_001af660 MenuCloseAll / FUN_001af608 MenuPop
    fn menu_close_all(&mut self);
    fn menu_pop(&mut self);
    /// FUN_001ddb08(0x3da530) new game
    fn new_game(&mut self);
    /// FUN_001660f8 quit to title
    fn quit_to_title(&mut self);
}

impl CardDialog {
    /// FUN_001acb58(dialog, msg)
    pub fn FUN_001acb58(&mut self, h: &mut impl CardDialogHost, msg: i32) {
        // 0x001acb58
        self.msg = msg;
        if self.screen {
            h.screen_show(msg);
        }
    }

    /// FUN_001acbb8: hide
    pub fn FUN_001acbb8(&mut self, h: &mut impl CardDialogHost) {
        // 0x001acbb8
        self.msg = -1;
        if self.screen {
            h.screen_hide();
        }
    }

    /// FUN_001acc18(sel)
    pub fn FUN_001acc18(&mut self, sel: i32, now: f32) {
        if sel != self.sel {
            self.sel = sel;
            self.sel_time = now;
        }
    }

    /// FUN_001acc38(mode): modes 2/3 clear DAT_002f6330.
    pub fn FUN_001acc38(&mut self, h: &mut impl CardDialogHost, mode: i32) {
        // 0x001acc38
        if mode == self.mode {
            return;
        }
        if (2..4).contains(&mode) {
            h.clear_f6330();
        }
        self.mode = mode;
        self.mode_time = h.now();
    }

    /// FUN_001acc80: button layout of the message (1 = X only, 2 = X yes / triangle no, 0 = none).
    pub fn FUN_001acc80(&self) -> i32 {
        match self.msg {
            0x11 | 0x14 | 0x17 | 0x18 | 0x1b | 0x1d | 0x22 | 0x23 | 0x24 | 0x2e => 1,
            0x12 | 0x15 | 0x1c | 0x20 => 2,
            _ => 0,
        }
    }

    /// FUN_001accc8: per-frame update.
    pub fn FUN_001accc8(&mut self, h: &mut impl CardDialogHost) {
        // 0x001accc8
        if !self.screen {
            return;
        }
        let Some(shown) = h.screen_ready() else { return };
        if h.world_busy() || h.now() - shown <= 0.1 {
            return;
        }
        let kind = self.FUN_001acc80();
        if kind == 1 || (kind == 2 && self.pad) {
            if kind == 2 && h.pressed(0x80) {
                self.FUN_001ad168(h);
                return;
            }
            let mask = if kind == 1 { 0x80 } else { 0x10 };
            if self.pad && h.pressed(mask) {
                self.FUN_001acfe0(h);
                return;
            }
        }
        match self.msg {
            0x11 | 0x12 | 0x15 | 0x18 | 0x2e => {
                if h.card_state() != 6 {
                    return;
                }
            }
            0x13 => {
                if h.thread_busy() {
                    return;
                }
                if h.thread_error() {
                    self.FUN_001acb58(h, 0x14);
                    return;
                }
                h.start_job(JOB_2A6D88);
                self.FUN_001acb58(h, 0x16);
                return;
            }
            0x16 => {
                if h.thread_busy() {
                    return;
                }
                if h.thread_error() {
                    self.FUN_001acb58(h, 0x17);
                    return;
                }
            }
            0x1a => {
                if h.thread_busy() {
                    return;
                }
                if h.thread_error() {
                    let m = if self.purpose != 3 { 0x1d } else { 0x23 };
                    self.FUN_001acb58(h, m);
                    return;
                }
                self.FUN_001acbb8(h);
                if self.purpose == 1 {
                    h.quit_to_title();
                } else if self.purpose == 2 {
                    h.menu_close_all();
                }
                return;
            }
            0x20 | 0x22 => {
                if self.mode == 1 {
                    return;
                }
            }
            0x21 => {
                if h.thread_busy() {
                    return;
                }
                h.clear_f6338();
                if h.thread_error() {
                    self.FUN_001acb58(h, 0x1d);
                    return;
                }
            }
            _ => return,
        }
        self.FUN_001acbb8(h);
    }

    /// FUN_001acfe0: acknowledge / "no".
    pub fn FUN_001acfe0(&mut self, h: &mut impl CardDialogHost) {
        // 0x001acfe0
        match self.msg {
            0x11 | 0x12 | 0x15 | 0x18 | 0x14 | 0x17 | 0x1d | 0x23 | 0x24 | 0x2e => {
                if matches!(self.msg, 0x11 | 0x12 | 0x15 | 0x18) && h.title_active() && h.menu_b0160() {
                    self.FUN_001acbb8(h);
                    self.FUN_001acc38(h, 2);
                    h.menu_close_all();
                    h.new_game();
                    return;
                }
                // (fall-through of the first case group)
                self.FUN_001acbb8(h);
                self.FUN_001acc38(h, 2);
                if h.is_save_menu() {
                    h.menu_pop();
                }
            }
            0x1b | 0x1c | 0x20 => self.FUN_001acbb8(h),
            0x22 => {
                h.set_autosave_warned();
                if (1..=3).contains(&self.purpose) {
                    self.FUN_001acb58(h, 0x1a);
                    h.save();
                }
            }
            _ => {}
        }
    }

    /// FUN_001ad168: "yes".
    pub fn FUN_001ad168(&mut self, h: &mut impl CardDialogHost) {
        // 0x001ad168
        match self.msg {
            0x15 => {
                h.start_job(JOB_2A6D88);
                self.FUN_001acb58(h, 0x16);
            }
            0x12 => {
                h.start_job(JOB_2A6E88);
                self.FUN_001acb58(h, 0x13);
            }
            0x1c => self.FUN_001ad3e0(h),
            0x20 => {
                h.start_job(JOB_2A6E90);
                self.FUN_001acb58(h, 0x21);
            }
            _ => {}
        }
    }

    /// FUN_001ad3e0: commit the selected slot; warn once about autosave (0x22), else save (0x1a).
    pub fn FUN_001ad3e0(&mut self, h: &mut impl CardDialogHost) {
        // 0x001ad3e0
        h.set_f6330_from_f6338();
        h.set_e659c(2);
        if h.autosave_warned() == 0 {
            self.FUN_001acb58(h, 0x22);
        } else {
            self.FUN_001acb58(h, 0x1a);
            h.save();
        }
    }

    /// FUN_001ad480(item): save-slot item pushed (`slot` = item +8); an empty slot (dt == 0) is taken
    /// directly, otherwise the overwrite question 0x1c. Returns the item flag bit (+0x1c |= 2).
    pub fn FUN_001ad480(&mut self, h: &mut impl CardDialogHost, slot: i32) -> u32 {
        // 0x001ad480
        let dt = h.slot_dt(slot);
        h.set_f6338(slot);
        if dt == 0.0 {
            self.FUN_001ad3e0(h);
        } else {
            self.FUN_001acb58(h, 0x1c);
        }
        2
    }
}

/// Screen object (+0x24 of 0x2e6550 / the busy screen): +0x58 and +0x64 show states (2 = fully shown),
/// +0x5c time of the last state change.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScreenElem {
    pub s58: i32,
    pub s64: i32,
    pub t5c: f32,
}

/// Engine calls of the "card access" screen (vtable 0x2d36d0, ctor FUN_001addf0, screen string 0x5d4).
pub trait CardBusyHost {
    /// screen object +0x24 (None = not created)
    fn screen(&self) -> Option<ScreenElem>;
    /// screen slot 0x18c(-1) show / slot 0x194 hide
    fn show(&mut self);
    fn hide(&mut self);
    /// FUN_001ab088(0x2f633c)
    fn thread_busy(&self) -> bool;
}

/// "Accessing memory card" screen of the card manager (FUN_001abd98 shows it, FUN_001ab418 / FUN_001abbe0 hide).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CardBusy {
    /// +0x28 / +0x2c (FUN_001adf78)
    pub sel: i32,
    pub sel_time: f32,
}

impl CardBusy {
    /// FUN_001ade38: show
    pub fn FUN_001ade38(&mut self, h: &mut impl CardBusyHost) {
        if h.screen().is_some() {
            h.show();
        }
    }

    /// FUN_001ade70: hide
    pub fn FUN_001ade70(&mut self, h: &mut impl CardBusyHost) {
        if h.screen().is_some() {
            h.hide();
        }
    }

    /// FUN_001adea8: still visible (+0x58 >= 2 or +0x64 >= 2)
    pub fn FUN_001adea8(&self, h: &impl CardBusyHost) -> bool {
        match h.screen() {
            Some(s) => 2 <= s.s58 || 1 < s.s64,
            None => false,
        }
    }

    /// FUN_001adf78(sel)
    pub fn FUN_001adf78(&mut self, sel: i32, now: f32) {
        if sel != self.sel {
            self.sel = sel;
            self.sel_time = now;
        }
    }

    /// FUN_001adee8 (per frame, after the FUN_001da808 base update): hide once fully shown for > 1 s and the
    /// card thread is idle (the screen stays up at least 1 s).
    pub fn FUN_001adee8(&mut self, h: &mut impl CardBusyHost, now: f32) {
        // 0x001adee8
        if let Some(s) = h.screen() {
            if s.s58 == 2 && s.s64 == 2 && 1.0 < now - s.t5c && !h.thread_busy() {
                self.FUN_001ade70(h);
            }
        }
    }
}
