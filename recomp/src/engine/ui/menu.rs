#![allow(non_snake_case)]
//! WP-I3: the menu manager, global object 0x3d9ed0 (pause / options / yes-no / controls / save-slot menus).
//!
//! Layout: +0x10 pad, +0x20 pause screen object, +0x24 depth, +0x28 + k*0x1c frames {+0 count, +4 items,
//! +8 screen object, +0xc mode (1 = pushed in place), +0x14 selection, +0x18 screen animation ids},
//! +0x1e8 state, +0x1ec pending depth (-1 none). Items are 0x20 bytes {+0 string id, +4 on-select,
//! +8 n, +0xc list, +0x10 per-frame fn, +0x14 on-push fn, +0x18 animation ids, +0x1c flags}; the static
//! item tables live in SCUS_973.16 (pause 0x2c00e8, options 0x2bfec8, ...) and are read from the ELF.
//!
//! REPLACED: every wait on a screen-object animation (FUN_002121e8(screen, anim) busy) completes at once,
//! and the state-entry animations FUN_00213b00 / vslot 0x18c are not played (the menu is drawn by
//! `ui::plugin` from the frame's rows instead of the class-126 screen objects). Item callbacks whose
//! effect is outside the UI (quit / exit / abandon / job help / save slots / settings apply) are reported
//! as `MenuEvent`s.
use super::bind::{elf_read, MenuView};
use super::options::{AudioOpts, ModsOpts, VideoOpts, AUD_ROWS, VID_ROWS};
use super::settings::{self, GfxRow, GraphicsOpts, OptState, SavedOpts};
use crate::engine::input::pad::{self, Clock, PadState};
use std::sync::Arc;

pub const MGR: u32 = 0x003d_9ed0;
/// FUN_001b0290 pause items (7) / screen animations
pub const PAUSE_ITEMS: u32 = 0x002c_00e8;
pub const PAUSE_ANIMS: u32 = 0x002b_fd38;
/// FUN_001b04e0 options items (11)
pub const OPTIONS_ITEMS: u32 = 0x002b_fec8;
/// FUN_001b0078 save-slot menus: item table -> menu id
pub const SAVE_MENUS: [(u32, i32); 3] = [(0x002b_fda8, 0x1e), (0x002b_fe08, 0x1f), (0x002b_fe68, 0x19)];
/// frames that fit between +0x28 and +0x1e8
pub const MAX_FRAMES: usize = 16;

/// WP-I4 port additions: pseudo callback addresses / item tables (not in the ELF).
/// "Graphics" row of the options menu (pushes the graphics page).
pub const PORT_GFX_OPEN: u32 = 0xffff_0001;
/// graphics page row (item n = row index into `GraphicsOpts::rows`)
pub const PORT_GFX_TOGGLE: u32 = 0xffff_0002;
/// "Yes" of the delete-slot confirmation (item n = slot)
pub const PORT_DELETE_SLOT: u32 = 0xffff_0003;
/// frame source of the graphics page
pub const PORT_GFX_SRC: u32 = 0xffff_1000;
/// string id of the port's "Graphics" row (drawn from `MenuView::text`)
pub const PORT_GFX_ID: i32 = -0x6766;
/// level string ids "No" / "Yes" of the confirm lists
pub const ID_NO: i32 = 0xb;

/// WP-A11 port pages (pseudo callbacks, not in the ELF): "Display" / "Sound Levels" / "Controls (Keys)" /
/// "Mods" rows of the options menu and their page rows (item n = row index).
pub const PORT_VID_OPEN: u32 = 0xffff_0004;
pub const PORT_VID_ROW: u32 = 0xffff_0005;
pub const PORT_AUD_OPEN: u32 = 0xffff_0006;
pub const PORT_AUD_ROW: u32 = 0xffff_0007;
pub const PORT_CTL_OPEN: u32 = 0xffff_0008;
pub const PORT_CTL_ROW: u32 = 0xffff_0009;
pub const PORT_MOD_OPEN: u32 = 0xffff_000a;
pub const PORT_MOD_ROW: u32 = 0xffff_000b;
/// info row of the Graphics page: the GI choice differs from the running one (applies on restart; A11
/// builds without the live switch) or, n = 1, A12: why ray-traced GI is greyed out
pub const PORT_GI_NOTE: u32 = 0xffff_000c;
pub const PORT_VID_SRC: u32 = 0xffff_1001;
pub const PORT_AUD_SRC: u32 = 0xffff_1002;
pub const PORT_CTL_SRC: u32 = 0xffff_1003;
pub const PORT_MOD_SRC: u32 = 0xffff_1004;
/// string ids of the A11 options rows (text from `MenuView::text`)
pub const PORT_VID_ID: i32 = -0x6800;
pub const PORT_AUD_ID: i32 = -0x6801;
pub const PORT_CTL_ID: i32 = -0x6802;
pub const PORT_MOD_ID: i32 = -0x6803;
/// base string id of A11 page rows (id = base - row)
pub const PORT_ROW_ID: i32 = -0x6900;
/// text of the Graphics page note row
pub const GI_NOTE: &str = "GI: restart to apply";
pub const ID_YES: i32 = 0xa;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MenuItem {
    pub id: i32,
    pub on_select: u32,
    pub n: i32,
    pub list: u32,
    pub per_frame: u32,
    pub on_push: u32,
    pub anims: u32,
    pub flags: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Frame {
    pub items: Vec<MenuItem>,
    /// table the items were copied from (FUN_001b0078 compares it)
    pub src: u32,
    pub screen: u32,
    pub mode: i32,
    pub sel: i32,
    pub anims: u32,
}

/// Effects of item callbacks that leave the UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuEvent {
    /// FUN_001b0878 "Yes, Quit"
    Quit,
    /// FUN_001b09a8 "Yes, Exit" (exit to episode menu)
    ExitToEpisode,
    /// FUN_001b0970 "Yes, Abandon"
    AbandonJob,
    /// FUN_001b0808 job help (0x3da3c0)
    JobHelp,
    /// FUN_001ad250 save slot n
    Slot(i32),
    /// FUN_001afb88 settings apply for the item's (new) string id
    Setting(i32),
    /// FUN_001aff88: slider cancelled -> FUN_001d8ff8 re-applies the stored options
    Revert,
    /// circle + confirm in a save-slot menu: delete slot n (the original's 0x2e6550 state 0x20 dialog)
    DeleteSlot(i32),
    /// port addition: a graphics option changed (`MenuMgr::gfx` holds the new values)
    Graphics,
    /// A11: a Display page value changed (`MenuMgr::video`)
    Video,
    /// A11: a Sound Levels page value changed (`MenuMgr::audio`)
    Audio,
    /// A11: the Mods page changed (`MenuMgr::mods`)
    Mods,
    /// unported callback address
    Other(u32),
}

/// Engine state FUN_001b0290 / FUN_001b04e0 test.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MenuCtx {
    /// FUN_0020e040 level transition pending
    pub transition: bool,
    /// FUN_001d9880(0x2e6550) save UI active
    pub save_ui: bool,
    /// FUN_002127c8
    pub screens_ready: bool,
    /// FUN_00132ea0(player) character id (0x35..0x37 = Sly / Bentley / Murray)
    pub player_char: Option<i32>,
    /// FUN_00210ae0 != 0 (level table entry of the current level)
    pub level_flag: bool,
    /// DAT_003da3e4 / DAT_003e0f9c
    pub job_help: bool,
    pub job_active: bool,
    /// FUN_001d9880(0x3da530) title / episode screen active
    pub title: bool,
    /// port addition: append the "Graphics" page to the options menu
    pub graphics: bool,
}

#[derive(Clone, Debug)]
pub struct MenuMgr {
    pub elf: Option<Arc<Vec<u8>>>,
    /// +0x20
    pub screen: u32,
    /// +0x24
    pub depth: i32,
    pub frames: Vec<Frame>,
    /// +0x1e8
    pub state: i32,
    /// +0x1ec
    pub pending: i32,
    pub events: Vec<MenuEvent>,
    /// WP-I4: live option values (mixer levels, screen centre, vibration mode)
    pub opts: OptState,
    /// WP-I4: stored option values (FUN_001af9c8 reads them; refreshed by the game side)
    pub saved: SavedOpts,
    /// WP-I4: `dt` save variable of the 3 card slots 0x2e6604 + k * 0x5464 (0 = empty)
    pub slots: [f32; 3],
    /// DAT_002f6330 slot in use (index)
    pub cur_slot: Option<i32>,
    /// port addition: graphics page values
    pub gfx: GraphicsOpts,
    /// A11: Display / Sound Levels / Mods page values, Controls page rows
    pub video: VideoOpts,
    pub audio: AudioOpts,
    pub mods: ModsOpts,
    pub controls: Vec<String>,
    /// A11 mouse input for the next FUN_001aeee0: extra pressed buttons (0x40 cross / 0x10 triangle) and
    /// a row step (wheel: -1 up / 1 down); cleared by FUN_001aeee0
    pub port_btn: u32,
    pub port_dir: i32,
    /// A12: GI switches live (`gfx::rt::RtState`, set by `ui::plugin`) -> no "restart to apply" row
    pub gi_live: bool,
    /// A12: why ray-traced GI is not offered (greyed info row on the Graphics page; set by `ui::plugin`)
    pub gi_reason: Option<String>,
}

impl MenuMgr {
    pub fn new(elf: Option<Arc<Vec<u8>>>) -> Self {
        MenuMgr {
            elf,
            screen: 1,
            depth: 0,
            frames: vec![Frame::default(); MAX_FRAMES],
            state: 0,
            pending: -1,
            events: Vec::new(),
            opts: OptState::default(),
            saved: SavedOpts::default(),
            slots: [0.0; 3],
            cur_slot: None,
            gfx: GraphicsOpts { raytraced: false, rt_available: false, hor_plus: true, ink_outlines: true },
            video: VideoOpts::default(),
            audio: AudioOpts::default(),
            mods: ModsOpts::default(),
            controls: Vec::new(),
            port_btn: 0,
            port_dir: 0,
            gi_live: false,
            gi_reason: None,
        }
    }

    /// Read `n` 0x20-byte items at `va` from SCUS_973.16.
    pub fn items(&self, va: u32, n: i32) -> Vec<MenuItem> {
        let Some(elf) = &self.elf else { return Vec::new() };
        let Some(b) = elf_read(elf, va, n.max(0) as usize * 0x20) else { return Vec::new() };
        b.chunks(0x20)
            .map(|c| {
                let w = |o: usize| u32::from_le_bytes(c[o..o + 4].try_into().unwrap());
                MenuItem { id: w(0) as i32, on_select: w(4), n: w(8) as i32, list: w(0xc), per_frame: w(0x10), on_push: w(0x14), anims: w(0x18), flags: w(0x1c) }
            })
            .collect()
    }

    pub fn top(&self) -> Option<&Frame> {
        (self.depth > 0).then(|| &self.frames[self.depth as usize - 1])
    }
    fn top_mut(&mut self) -> Option<&mut Frame> {
        (self.depth > 0).then(|| &mut self.frames[self.depth as usize - 1])
    }

    /// Rows for FUN_00211298 (None when no frame is open).
    pub fn view(&self) -> Option<MenuView> {
        self.top().map(|f| {
            let rows = self.gfx.rows();
            let text = f
                .items
                .iter()
                .map(|i| match (i.on_select, i.id) {
                    (PORT_GFX_TOGGLE, _) => rows.get(i.n as usize).map(|&r| self.gfx.label(r).into_bytes()),
                    (_, PORT_GFX_ID) => Some(b"Graphics".to_vec()),
                    _ => self.port_text(i).map(String::into_bytes),
                })
                .collect();
            MenuView { ids: f.items.iter().map(|i| i.id).collect(), sel: f.sel, text }
        })
    }

    /// FUN_001b0078: menu id of the top frame (save-slot menus), else -1.
    pub fn FUN_001b0078(&self) -> i32 {
        self.top().and_then(|f| SAVE_MENUS.iter().find(|m| m.0 == f.src)).map_or(-1, |m| m.1)
    }
    /// FUN_001b0108: top frame is a save-slot menu (0x19, 0x1e, 0x1f).
    pub fn FUN_001b0108(&self) -> bool {
        matches!(self.FUN_001b0078(), 0x19 | 0x1e | 0x1f)
    }

    /// FUN_001aeaa8: state change (leaving 5 / 6 moves the selection; the screen animations are REPLACED).
    pub fn FUN_001aeaa8(&mut self, s: i32) {
        if self.state == s {
            return;
        }
        let old = self.state;
        if let Some(f) = self.top_mut() {
            let n = f.items.len() as i32;
            match old {
                5 => {
                    f.sel += 1;
                    if n <= f.sel {
                        f.sel = 0;
                    }
                }
                6 => {
                    f.sel -= 1;
                    if f.sel < 0 {
                        f.sel = n - 1;
                    }
                }
                _ => {}
            }
        }
        self.state = s;
    }

    fn run_on_push(&mut self, idx: usize) {
        for i in 0..self.frames[idx].items.len() {
            let it = self.frames[idx].items[i];
            match it.on_push {
                0x001a_f9c8 => self.frames[idx].items[i].id = settings::FUN_001af9c8(it.id, &self.saved, &self.opts),
                0x001a_d538 => self.FUN_001ad538(),
                _ => {}
            }
        }
    }

    /// FUN_001ad538 (save-slot menu on-push): initial selection from the card slots' `dt` variables
    /// (FUN_001acc38 card refresh = `slots`, filled by the game side). 0x1e save: slot 0 if empty, else the
    /// first empty one; 0x1f load: slot 0 if used, else the first used one; 0x19: the slot in use.
    pub fn FUN_001ad538(&mut self) {
        if self.depth < 1 {
            return;
        }
        let s = self.slots;
        let sel = match self.FUN_001b0078() {
            0x1e => {
                if s[0] == 0.0 {
                    Some(0)
                } else {
                    (1..3).find(|&k| s[k] == 0.0).map(|k| k as i32)
                }
            }
            0x1f => {
                if s[0] != 0.0 {
                    Some(0)
                } else {
                    (1..3).find(|&k| s[k] != 0.0).map(|k| k as i32)
                }
            }
            0x19 => self.cur_slot,
            _ => None,
        };
        if let (Some(v), Some(f)) = (sel, self.top_mut()) {
            f.sel = v;
        }
    }

    /// FUN_001af4d8(mgr, count, items, screen, anims): push a frame.
    pub fn FUN_001af4d8(&mut self, items: Vec<MenuItem>, src: u32, mut screen: u32, anims: u32) {
        if screen == 0 && self.depth > 0 {
            screen = self.frames[self.depth as usize - 1].screen;
        }
        let d = self.depth as usize;
        if d >= MAX_FRAMES {
            return;
        }
        self.depth += 1;
        self.frames[d] = Frame { items, src, screen, mode: 0, sel: 0, anims };
        self.FUN_001aeaa8(1);
        self.run_on_push(d);
        if self.FUN_001b0108() {
            self.pending = self.depth;
            self.depth -= 1;
        }
    }

    /// FUN_001af390(mgr, count, items): push in place (mode 1, same screen / animations), state 3.
    pub fn FUN_001af390(&mut self, items: Vec<MenuItem>, src: u32) {
        self.FUN_001aeaa8(3);
        let d = self.depth as usize;
        if d >= MAX_FRAMES {
            return;
        }
        let (screen, anims) = if d > 0 { (self.frames[d - 1].screen, self.frames[d - 1].anims) } else { (0, 0) };
        self.pending = self.depth + 1;
        self.frames[d] = Frame { items, src, screen, mode: 1, sel: 0, anims };
        self.run_on_push(d);
    }

    /// FUN_001af490: pop in place (state 3 to depth - 1).
    pub fn FUN_001af490(&mut self) {
        if self.state == 7 {
            self.FUN_001aeaa8(3);
            self.pending = self.depth - 1;
        }
    }

    /// FUN_001af608: pop.
    pub fn FUN_001af608(&mut self) {
        if self.state == 7 {
            self.FUN_001aeaa8(2);
            if self.FUN_001b0108() {
                self.depth -= 1;
                self.pending = self.depth;
            }
        }
    }

    /// FUN_001af660: close all.
    pub fn FUN_001af660(&mut self) {
        if self.state == 7 {
            if 1 < self.depth {
                self.depth = 1;
            }
            self.FUN_001aeaa8(8);
        }
    }

    /// FUN_001af6a0: enter the slider (state 0xb).
    pub fn FUN_001af6a0(&mut self) {
        if self.state == 7 {
            self.FUN_001aeaa8(0xb);
        }
    }

    /// FUN_001b0290(item): open the pause menu. Returns whether it opened.
    pub fn FUN_001b0290(&mut self, ctx: &MenuCtx) -> bool {
        // port guard (see Findings I3): not while a frame is open
        if self.screen == 0 || self.depth != 0 || ctx.transition || ctx.save_ui || !ctx.screens_ready {
            return false;
        }
        let all = self.items(PAUSE_ITEMS, 7);
        let items = all
            .into_iter()
            .filter(|it| match it.id {
                0x43 => ctx.player_char.is_some_and(|c| c > 0x34 && c < 0x38),
                0x42 => ctx.level_flag,
                0x6a => ctx.job_help && ctx.level_flag,
                0x6b => ctx.job_active && ctx.level_flag,
                _ => true,
            })
            .collect();
        self.FUN_001af4d8(items, PAUSE_ITEMS, self.screen, PAUSE_ANIMS);
        true
    }

    /// FUN_001b04e0(item): options menu (0x3c "New Game..." only on the title screen, 0x3e "Save Game..."
    /// only in game). From an item: pushed in place; from the title (item 0): pushed.
    pub fn FUN_001b04e0(&mut self, ctx: &MenuCtx, from_item: bool) {
        let items: Vec<MenuItem> = self
            .items(OPTIONS_ITEMS, 0xb)
            .into_iter()
            .filter(|it| match it.id {
                0x3c => ctx.title,
                0x3e => !ctx.title,
                _ => true,
            })
            .chain(ctx.graphics.then_some(MenuItem { id: PORT_GFX_ID, on_select: PORT_GFX_OPEN, flags: 2, ..Default::default() }))
            .chain(
                [(PORT_VID_ID, PORT_VID_OPEN), (PORT_AUD_ID, PORT_AUD_OPEN), (PORT_CTL_ID, PORT_CTL_OPEN), (PORT_MOD_ID, PORT_MOD_OPEN)]
                    .into_iter()
                    .filter(|_| ctx.graphics)
                    .map(|(id, on_select)| MenuItem { id, on_select, flags: 2, ..Default::default() }),
            )
            .collect();
        if from_item {
            self.FUN_001af390(items, OPTIONS_ITEMS);
        } else {
            self.FUN_001af4d8(items, OPTIONS_ITEMS, self.screen, PAUSE_ANIMS);
        }
    }

    /// Item +4 callback; returns the item flags after the call (+0x1c, bit 2 = handled, 1 = back).
    fn select(&mut self, ctx: &MenuCtx, it: MenuItem) -> u32 {
        let handled = it.flags | 2;
        match it.on_select {
            0 => it.flags,
            0x001b_0270 => {
                self.FUN_001af660();
                it.flags
            }
            0x001b_06d8 => {
                let items = self.items(it.list, it.n);
                self.FUN_001af390(items, it.list);
                handled
            }
            0x001b_0688 => {
                let items = self.items(it.list, it.n);
                self.FUN_001af4d8(items, it.list, 0, it.anims);
                handled
            }
            0x001b_0668 => {
                self.FUN_001af490();
                it.flags
            }
            0x001b_04e0 => {
                self.FUN_001b04e0(ctx, true);
                handled
            }
            0x001b_0720 => {
                self.FUN_001af6a0();
                it.flags
            }
            0x001b_0740 => {
                // controls screen of the current character (no rows)
                let anims = match ctx.player_char {
                    Some(0x35) => 0x002c_01d0,
                    Some(0x37) => 0x002c_01f0,
                    Some(0x36) => 0x002c_0210,
                    _ => return it.flags,
                };
                self.FUN_001af4d8(Vec::new(), 0, 0, anims);
                handled
            }
            0x001a_ffc0 => self.FUN_001affc0(it),
            PORT_GFX_OPEN => {
                self.port_open_graphics();
                handled
            }
            PORT_GFX_TOGGLE => {
                if let Some(&r) = self.gfx.rows().get(it.n as usize) {
                    self.gfx.toggle(r);
                    self.events.push(MenuEvent::Graphics);
                    if r == GfxRow::Lighting {
                        // A11: the restart note row comes and goes with the GI choice
                        let items = self.graphics_items();
                        if let Some(f) = self.top_mut() {
                            f.items = items;
                        }
                    }
                }
                handled
            }
            PORT_VID_OPEN | PORT_AUD_OPEN | PORT_CTL_OPEN | PORT_MOD_OPEN => {
                self.port_open_page(it.on_select);
                handled
            }
            PORT_VID_ROW | PORT_AUD_ROW | PORT_MOD_ROW => {
                self.port_adjust(it, 0);
                handled
            }
            PORT_CTL_ROW | PORT_GI_NOTE => handled,
            PORT_DELETE_SLOT => {
                self.events.push(MenuEvent::DeleteSlot(it.n));
                self.slots[it.n.clamp(0, 2) as usize] = 0.0;
                // back to the slot list
                it.flags | 1
            }
            0x001b_0878 => {
                self.events.push(MenuEvent::Quit);
                handled
            }
            0x001b_09a8 => {
                self.events.push(MenuEvent::ExitToEpisode);
                it.flags
            }
            0x001b_0970 => {
                self.events.push(MenuEvent::AbandonJob);
                it.flags
            }
            0x001b_0808 => {
                self.events.push(MenuEvent::JobHelp);
                it.flags
            }
            0x001a_d250 => {
                self.events.push(MenuEvent::Slot(it.n));
                it.flags
            }
            a => {
                self.events.push(MenuEvent::Other(a));
                it.flags
            }
        }
    }

    /// FUN_001affc0(item): cycle the item's string id through its list (item +0xc, +8 entries), apply
    /// (FUN_001afb88 -> `MenuEvent::Setting`; that also sets state 9 for the volume / centre items).
    fn FUN_001affc0(&mut self, it: MenuItem) -> u32 {
        let list: Vec<i32> = self.elf.as_ref().and_then(|e| elf_read(e, it.list, it.n.max(0) as usize * 4)).map_or_else(Vec::new, |b| {
            b.chunks(4).map(|c| i32::from_le_bytes(c.try_into().unwrap())).collect()
        });
        let mut id = it.id;
        let n = it.n;
        if 0 < n && list.len() == n as usize {
            if id == list[0] {
                id = list[(1 % n) as usize];
            } else if let Some(k) = (1..n as usize).find(|&k| list[k] == id) {
                id = list[(k + 1) % n as usize];
            }
        }
        if let Some(f) = self.top_mut() {
            let s = f.sel as usize;
            if let Some(x) = f.items.get_mut(s) {
                x.id = id;
            }
        }
        self.events.push(MenuEvent::Setting(id));
        it.flags | 2
    }

    /// FUN_001aeee0: per-frame update with the pad (mgr +0x10).
    pub fn FUN_001aeee0(&mut self, pad: &mut PadState, clock: Clock, ctx: &MenuCtx) {
        self.FUN_001aeee0_inner(pad, clock, ctx);
        self.port_btn = 0;
        self.port_dir = 0;
    }

    fn FUN_001aeee0_inner(&mut self, pad: &mut PadState, clock: Clock, ctx: &MenuCtx) {
        if self.depth < 1 {
            return;
        }
        match self.state {
            1 => {
                if self.pending != -1 {
                    self.depth = self.pending;
                    self.pending = -1;
                }
                self.FUN_001aeaa8(7);
            }
            2 => {
                if self.pending == -1 {
                    self.depth -= 1;
                } else {
                    self.pending = -1;
                }
                if self.depth == 0 {
                    // screen vslot 0x194 (hide)
                    self.FUN_001aeaa8(0);
                } else {
                    self.FUN_001aeaa8(7);
                }
            }
            3 => {
                self.depth = self.pending;
                self.pending = -1;
                self.FUN_001aeaa8(4);
            }
            4 | 9 => self.FUN_001aeaa8(7),
            5 | 6 => {
                self.FUN_001aeaa8(7);
                self.nav(pad, clock);
            }
            7 => {
                let b = pad::FUN_00187d80(pad, 0x870) | (self.port_btn & 0x70);
                if b & 0x800 != 0 {
                    self.FUN_001af660();
                    return;
                }
                let Some(f) = self.top().cloned() else { return };
                if b & 0x10 != 0 || (f.items.is_empty() && b != 0) {
                    if f.mode != 0 {
                        self.FUN_001af490();
                    } else {
                        self.FUN_001af608();
                    }
                    return;
                }
                if b & 0x40 != 0 {
                    let Some(it) = f.items.get(f.sel as usize).copied() else { return };
                    let flags = self.select(ctx, it);
                    if flags & 2 != 0 {
                        return;
                    }
                    if flags & 1 == 0 {
                        self.FUN_001af660();
                    } else if f.mode != 0 {
                        self.FUN_001af490();
                    } else {
                        self.FUN_001af608();
                    }
                    return;
                }
                if b & 0x20 != 0 {
                    // circle: delete in the save menus 0x19 / 0x1e when the slot is used (DAT_002f6338 = slot,
                    // FUN_001acb58(0x2e6550, 0x20) = the save UI's confirmation). Port: in-place No / Yes list.
                    let m = self.FUN_001b0078();
                    if m != 0x19 && m != 0x1e {
                        return;
                    }
                    let k = f.sel.clamp(0, 2);
                    if self.slots[k as usize] != 0.0 {
                        let items = vec![
                            MenuItem { id: ID_NO, on_select: 0x001b_0668, flags: 2, ..Default::default() },
                            MenuItem { id: ID_YES, on_select: PORT_DELETE_SLOT, n: k, flags: 2, ..Default::default() },
                        ];
                        self.FUN_001af390(items, 0);
                    }
                    return;
                }
                // A11: left / right on the port's value rows (FUN_00187e88 auto-repeat)
                if let Some(it) = f.items.get(f.sel as usize).copied().filter(|i| matches!(i.on_select, PORT_VID_ROW | PORT_AUD_ROW | PORT_MOD_ROW)) {
                    let dx = pad::FUN_00187e88(pad, clock);
                    if dx != 0 {
                        self.port_adjust(it, dx);
                    }
                }
                self.nav(pad, clock);
            }
            8 => {
                // screen vslot 0x194 (hide)
                self.depth = 0;
                self.FUN_001aeaa8(0);
            }
            10 => {
                let b = pad::FUN_00187d80(pad, 0x850);
                let id = self.top().and_then(|f| f.items.get(f.sel as usize)).map_or(0, |i| i.id);
                if b & 0x800 != 0 {
                    // FUN_001aff88 then FUN_001af660 (a no-op: state is 9 by then, quirk kept)
                    self.FUN_001aff88();
                    self.FUN_001af660();
                } else if b & 0x10 != 0 {
                    self.FUN_001aff88();
                } else if b & 0x40 != 0 {
                    // FUN_001afb88: the save write is done by the game side on the event
                    self.events.push(MenuEvent::Setting(id));
                    if matches!(id, 0x3f | 0x67 | 0x68) {
                        self.FUN_001aeaa8(9);
                    }
                } else {
                    // FUN_001af6d0: FUN_00187e88 always, FUN_001880d8 only for the screen-centre item
                    let dx = pad::FUN_00187e88(pad, clock);
                    let dy = if id == 0x3f { pad::FUN_001880d8(pad, clock) } else { 0 };
                    settings::FUN_001af6d0(id, &mut self.opts, dx, dy, clock.dt);
                }
            }
            0xb => self.FUN_001aeaa8(10),
            _ => {}
        }
    }

    /// Top frame (verification hooks).
    pub fn top_mut_pub(&mut self) -> Option<&mut Frame> {
        self.top_mut()
    }

    /// Port addition: push the Graphics page (one toggle row per `GraphicsOpts::rows`).
    pub fn port_open_graphics(&mut self) {
        let items = self.graphics_items();
        self.FUN_001af4d8(items, PORT_GFX_SRC, 0, 0);
    }

    /// Graphics page rows (+ A11 note row when the GI choice differs from the running GI).
    fn graphics_items(&self) -> Vec<MenuItem> {
        let mut items: Vec<MenuItem> = (0..self.gfx.rows().len() as i32)
            .map(|k| MenuItem { id: PORT_GFX_ID - 1 - k, on_select: PORT_GFX_TOGGLE, n: k, ..Default::default() })
            .collect();
        if let (false, Some(_)) = (self.gfx.rt_available, &self.gi_reason) {
            items.insert(0, MenuItem { id: PORT_ROW_ID, on_select: PORT_GI_NOTE, n: 1, flags: 2, ..Default::default() });
        }
        if !self.gi_live && self.gfx.rt_available && self.gfx.raytraced != self.video.rt_running {
            items.push(MenuItem { id: PORT_ROW_ID, on_select: PORT_GI_NOTE, flags: 2, ..Default::default() });
        }
        items
    }

    /// A11: text of the port's options rows / page rows (None = not a port row).
    fn port_text(&self, i: &MenuItem) -> Option<String> {
        let n = i.n.max(0) as usize;
        Some(match i.on_select {
            PORT_VID_OPEN => "Display".into(),
            PORT_AUD_OPEN => "Sound Levels".into(),
            PORT_CTL_OPEN => "Controls (Keys)".into(),
            PORT_MOD_OPEN => "Mods".into(),
            PORT_VID_ROW => self.video.label(*VID_ROWS.get(n)?),
            PORT_AUD_ROW => self.audio.label(*AUD_ROWS.get(n)?),
            PORT_CTL_ROW => self.controls.get(n).cloned().unwrap_or_else(|| "controls.toml not loaded".into()),
            PORT_MOD_ROW => self.mods.label(n),
            PORT_GI_NOTE if n == 1 => format!("GI: Off ({})", self.gi_reason.as_deref().unwrap_or("unavailable")),
            PORT_GI_NOTE => GI_NOTE.into(),
            _ => return None,
        })
    }

    /// A11: push a port page (Display / Sound Levels / Controls / Mods).
    pub fn port_open_page(&mut self, open: u32) {
        let (row, n, src) = match open {
            PORT_VID_OPEN => (PORT_VID_ROW, VID_ROWS.len(), PORT_VID_SRC),
            PORT_AUD_OPEN => (PORT_AUD_ROW, AUD_ROWS.len(), PORT_AUD_SRC),
            PORT_CTL_OPEN => (PORT_CTL_ROW, self.controls.len().max(1), PORT_CTL_SRC),
            PORT_MOD_OPEN => (PORT_MOD_ROW, self.mods.entries.len().max(1), PORT_MOD_SRC),
            _ => return,
        };
        let items = (0..n as i32).map(|k| MenuItem { id: PORT_ROW_ID - k, on_select: row, n: k, flags: 2, ..Default::default() }).collect();
        self.FUN_001af4d8(items, src, 0, 0);
    }

    /// A11: change a port value row (d = 0 cross, -1 / 1 left / right).
    pub fn port_adjust(&mut self, it: MenuItem, d: i32) {
        let k = it.n.max(0) as usize;
        match it.on_select {
            PORT_VID_ROW => {
                if let Some(&r) = VID_ROWS.get(k) {
                    if self.video.adjust(r, if d == 0 { 1 } else { d }) {
                        self.events.push(MenuEvent::Video);
                    }
                }
            }
            PORT_AUD_ROW => {
                if let Some(&r) = AUD_ROWS.get(k) {
                    if self.audio.adjust(r, d) {
                        self.events.push(MenuEvent::Audio);
                    }
                }
            }
            PORT_MOD_ROW => {
                if self.mods.adjust(k, d) {
                    self.events.push(MenuEvent::Mods);
                    // the selection follows a moved mod
                    if d != 0 {
                        if let Some(f) = self.top_mut() {
                            f.sel = (f.sel + d).clamp(0, f.items.len() as i32 - 1);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// FUN_001aff88: FUN_001d8ff8 (re-apply the stored options, done by the game side on the event) + state 9.
    pub fn FUN_001aff88(&mut self) {
        self.events.push(MenuEvent::Revert);
        self.FUN_001aeaa8(9);
    }

    /// FUN_001880d8 direction: up -> state 6, down -> state 5.
    fn nav(&mut self, pad: &mut PadState, clock: Clock) {
        let d = match pad::FUN_001880d8(pad, clock) {
            0 => self.port_dir,
            d => d,
        };
        if d < 0 {
            self.FUN_001aeaa8(6);
        } else if 0 < d {
            self.FUN_001aeaa8(5);
        }
    }
}
