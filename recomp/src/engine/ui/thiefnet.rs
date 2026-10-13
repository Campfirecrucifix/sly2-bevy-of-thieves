#![allow(non_snake_case)]
//! WP-X3: the safehouse ThiefNet shop screen (global 0x3da150, class vtable 0x2d84e8): mode 0 buys gadgets
//! ("powerUps" / "mppukfAcquired[%d]") with coins, mode 1 sells loot ("loot" / "mplootkc[%d]"); square swaps
//! the two pages, X buys (first press opens the confirm dialog 0x34, or 0x35 "not enough coins"), triangle
//! closes the dialog / the screen, the stick (8 directions) scrolls and the left / right directions pick the page.
//!
//! 1:1: C2d84e8_v6 FUN_001db870 (per frame), FUN_001dc260 (page change / open / close), C2d84e8_v13
//! FUN_001dc4b0 (stick direction; jump table 0x2a79f0 read in the raw disassembly: 0 -> page 1, 1..3 ->
//! up, 4 -> page 0, 5..7 -> down), FUN_001dbd10 (build the list), FUN_001dbbe0 (select + scroll),
//! FUN_001dbba0 (last visible row), FUN_001dbef0 (rows / cursor / arrows), FUN_001da950 (dialog),
//! FUN_001da9e8, FUN_001da858 (stick -> direction), FUN_001dae58 (gadget on sale).
//! Tables: gadgets DAT_002bcd00 = `player::p18_gadget::GADGETS` (+4 owner -> row icon 0x456 / 0x457 / 0x458,
//! +8 price, +0x10 shop episode); loot DAT_002c3288 (0xc stride, +0 sale price, `LOOT_PRICE`); rows
//! DAT_002c3588 = element ids 0x2f5.. ; DAT_002c35b4 = 6 rows, DAT_002c35b0 = 40.0 row height,
//! DAT_002c3254 = 5.0 dialog time.
//! HOST (`ShopHost`): pad (+0x10, FUN_00187d80 / stick +0x103c..+0x1044), coins (FUN_00161798 /
//! FUN_001617e0 on 0x2deb30), save variables (bif 138 acquired, FUN_001daf38 acquire, mplootkc via
//! FUN_001d8a38 / FUN_001d8b20), screen +0x24 (FUN_00212a70 / FUN_00212a50 / FUN_00213a08 /
//! FUN_00213bd8 / FUN_00213db0 / FUN_00213cd8 / FUN_00213b00 / FUN_00213928, vtable +0x194, +0x58,
//! +0x70/+0x74/+0x78 ids), base update C2d83e8_v6, own vtable +0x54 / +0x5c, clock / world pause, sounds.
//! NOT wired: the port has no 0x3da150 screen object (ui::hud is REPLACED and FUN_001d9648 is partial).
use crate::engine::player::p18_gadget::GADGETS;

/// DAT_002c3288 +0 (loot sale price, 0x40 entries x 0xc; read from SCUS_973.16 by the census test).
pub const LOOT_PRICE_ADDR: u32 = 0x2c3288;
/// DAT_002c3588
pub const ROW_ELEMS: [i32; 8] = [0x2f5, 0x2f6, 0x2f7, 0x2f8, 0x2f9, 0x2fa, 0x2fb, 0x2fc];
/// DAT_002c3288 +0 for the 0x40 loot kinds, read from the ELF.
pub fn loot_prices(elf: &[u8]) -> Option<Vec<i32>> {
    let b = crate::engine::disc::descriptor::elf_bytes(elf, LOOT_PRICE_ADDR, 0x40 * 0xc)?;
    Some((0..0x40).map(|i| i32::from_le_bytes(b[i * 0xc..i * 0xc + 4].try_into().unwrap())).collect())
}
/// DAT_002c35b4 / DAT_002c35b0 / DAT_002c3254
pub const ROWS: i32 = 6;
pub const ROW_H: f32 = 40.0;
pub const DIALOG_T: f32 = 5.0;

/// pad masks of FUN_00187d80
pub const PAD_TRIANGLE: u32 = 0x10;
pub const PAD_X: u32 = 0x40;
pub const PAD_SQUARE: u32 = 0x80;

/// Screen slot holders of FUN_00213db0 (object +0x25c / +0x260 / +0x264).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Dialog,
    Up,
    Down,
}

/// Calls out of the shop screen.
pub trait ShopHost {
    /// C2d83e8_v6 (base update)
    fn base_update(&mut self);
    /// FUN_00187d80(+0x10, mask)
    fn pressed(&mut self, mask: u32) -> bool;
    /// pad +0x103c / +0x1040 / +0x1044 (stick x, y, magnitude)
    fn stick(&self) -> (f32, f32, f32);
    /// FUN_00161798(0x2deb30) / FUN_001617e0(0x2deb30, v)
    fn coins(&self) -> i32;
    fn set_coins(&mut self, v: i32);
    /// splice bif 138: gadget acquired
    fn acquired(&mut self, g: i32) -> bool;
    /// FUN_001daf38(g, 1)
    fn acquire(&mut self, g: i32, on: bool);
    /// FUN_00210b28
    fn episode(&self) -> i32;
    /// mplootkc[i] (FUN_001d8a38 get / FUN_001d8b20 set)
    fn loot(&mut self, i: i32) -> i32;
    fn set_loot(&mut self, i: i32, v: i32);
    /// FUN_0019f0a8(0x3dd490)
    fn world_19f0a8(&mut self);
    /// emitter_create(id) + FUN_001fb2f8(e, 0)
    fn sound(&mut self, id: i32);
    /// DAT_002ddf0c
    fn time(&self) -> f32;
    /// FUN_0014d1d0(scale, 0x2dded0, 0)
    fn clock_scale(&mut self, s: f32);
    /// FUN_0020efc0(0x3dd490, 0) / FUN_0020f030(0x3dd490, 0)
    fn world_pause(&mut self, on: bool);
    /// own vtable +0x54 (open) / +0x5c (close)
    fn vt_open(&mut self);
    fn vt_close(&mut self);
    /// screen +0x58
    fn scr_state(&self) -> i32;
    /// screen vtable +0x194
    fn scr_close(&mut self);
    /// screen +0x74 / +0x70 / +0x78 = ids
    fn scr_ids(&mut self, a74: i32, a70: i32, a78: i32);
    /// FUN_00212a70(scr, k) / FUN_00212a50(scr, k)
    fn scr_212a70(&mut self, k: i32);
    fn scr_212a50(&mut self, k: i32);
    /// FUN_00213a08 / FUN_00213bd8 (element, row)
    fn scr_213a08(&mut self, e: i32, row: i32);
    fn scr_213bd8(&mut self, e: i32, row: i32);
    /// FUN_00213db0(scr, element, slot)
    fn scr_place(&mut self, e: i32, slot: Slot);
    /// FUN_00213cd8(scr, element, {1,0,0,1, 0, ty})
    fn scr_xform(&mut self, e: i32, ty: f32);
    /// FUN_00213b00 / FUN_00213928
    fn scr_show(&mut self, e: i32);
    fn scr_hide(&mut self, e: i32);
}

/// Shop screen object 0x3da150 (the fields the shop code owns).
#[derive(Clone, Debug, PartialEq)]
pub struct Shop {
    /// +0x10 != 0 (pad) / +0x24 != 0 (screen)
    pub pad: bool,
    pub scr: bool,
    /// +0x28 stick direction, +0x2c its time
    pub dir: i32,
    pub dir_t: f32,
    /// +0x30 dialog (-1 none, 0 confirm, 1 error), +0x34 dialog time, +0x38 message
    pub dialog: i32,
    pub dialog_t: f32,
    pub dialog_msg: i32,
    /// +0x40 page (-1 closed, 0 gadgets, 1 loot), +0x44 time
    pub page: i32,
    pub page_t: f32,
    /// +0x48 count / +0x4c entries {id, row icon} (0x40 slots; rows past the count keep stale icons)
    pub count: i32,
    pub ent: [[i32; 2]; 0x48], // 0x40 + room for the 6 rows drawn past the end (the original reads +0x24c.. there)
    /// +0x24c selected row, +0x250 first visible row, +0x254 gadget, +0x258 loot
    pub sel: i32,
    pub first: i32,
    pub gadget: i32,
    pub loot: i32,
    /// DAT_003da570 (scroll arrows take a row)
    pub da570: bool,
    /// DAT_002c3288 +0 per loot kind
    pub loot_price: Vec<i32>,
}

impl Default for Shop {
    fn default() -> Self {
        Self {
            pad: true,
            scr: true,
            dir: -1,
            dir_t: 0.0,
            dialog: -1,
            dialog_t: 0.0,
            dialog_msg: -1,
            page: -1,
            page_t: 0.0,
            count: 0,
            ent: [[-1; 2]; 0x48], // inf: ctor value not traced
            sel: -1,
            first: 0,
            gadget: -1,
            loot: -1,
            da570: false,
            loot_price: Vec::new(),
        }
    }
}

/// 0x001dae58: gadget g is on sale (DAT_002bcd10 = shop episode)
pub fn FUN_001dae58(h: &dyn ShopHost, g: i32) -> bool {
    let e = GADGETS[g as usize].order;
    e != -1 && e <= h.episode()
}

impl Shop {
    /// 0x001da950
    pub fn FUN_001da950(&mut self, h: &mut dyn ShopHost, s: i32, msg: i32) {
        self.dialog_msg = msg;
        self.dialog = s;
        if s == 0 {
            self.dialog_t = f32::MAX; // DAT_002a7984
            h.sound(0x29);
        } else if s == -1 {
            self.dialog_t = -10.0;
        } else {
            self.dialog_t = h.time() + DIALOG_T;
            h.sound(0x36);
        }
    }

    /// 0x001da9e8
    pub fn FUN_001da9e8(&mut self, h: &mut dyn ShopHost) {
        self.FUN_001da950(h, -1, -1);
    }

    /// 0x001dbba0: one past the last visible row
    pub fn FUN_001dbba0(&self) -> i32 {
        let mut n = self.first + ROWS;
        if self.da570 {
            n -= (0 < self.first) as i32;
            n -= (n < self.count) as i32;
        }
        n
    }

    /// 0x001dbbe0
    pub fn FUN_001dbbe0(&mut self, h: &mut dyn ShopHost, sel: i32) {
        if self.count == 0 {
            self.first = 0;
            self.loot = -1;
            self.sel = -1;
            self.gadget = -1;
        } else {
            let sel = sel.clamp(0, self.count - 1);
            self.sel = sel;
            if !self.da570 {
                let lo = (sel - ROWS + 1).max(0);
                self.first = if self.first < lo { lo } else { self.first.min(sel) };
            } else {
                if sel < self.first {
                    self.first = sel;
                } else {
                    while self.FUN_001dbba0() - 1 < self.sel {
                        self.first += 1;
                    }
                }
                if self.first == 1 {
                    self.first = 0;
                }
            }
            let id = self.ent[self.sel as usize][0];
            self.gadget = id;
            self.loot = id;
        }
        self.FUN_001da9e8(h);
    }

    /// 0x001dbd10
    pub fn FUN_001dbd10(&mut self, h: &mut dyn ShopHost) {
        for i in 0..self.count as usize {
            self.ent[i][0] = -1;
        }
        self.count = 0;
        if self.page == 0 {
            for g in 7..0x2b {
                if FUN_001dae58(h, g) && !h.acquired(g) {
                    // 0x1dbdc0: the bound test is `0x40 < count` (36 gadgets / 0x40 loot kinds never reach it)
                    if 0x40 <= self.count {
                        break;
                    }
                    self.ent[self.count as usize][0] = g;
                    self.count += 1;
                }
            }
        } else if self.page == 1 {
            for i in 0..0x40 {
                if h.loot(i) != 0 {
                    if 0x40 <= self.count {
                        break;
                    }
                    self.ent[self.count as usize][0] = i;
                    self.count += 1;
                }
            }
        }
        self.FUN_001dbbe0(h, self.sel);
    }

    /// 0x001dc4b0 (C2d84e8_v13): stick direction
    pub fn FUN_001dc4b0(&mut self, h: &mut dyn ShopHost, d: i32) {
        if d == self.dir {
            return;
        }
        self.dir = d;
        self.dir_t = h.time();
        match d {
            0 => self.FUN_001dc260(h, 1),
            4 => self.FUN_001dc260(h, 0),
            1..=3 => {
                if 0 < self.sel {
                    self.FUN_001dbbe0(h, self.sel - 1);
                    h.sound(0x35);
                } else {
                    h.sound(0x36);
                }
            }
            5..=7 => {
                if self.sel < self.count - 1 {
                    self.FUN_001dbbe0(h, self.sel + 1);
                    h.sound(0x35);
                } else {
                    h.sound(0x36);
                }
            }
            _ => {}
        }
    }

    /// 0x001da858: stick -> one of 8 directions (held past 0.8, released below 0.5)
    pub fn FUN_001da858(&mut self, h: &mut dyn ShopHost) {
        if !self.pad {
            return;
        }
        let (x, y, m) = h.stick();
        if self.dir == -1 {
            if 0.8 < m {
                let d = (y.atan2(x) * 1.2732395 + 8.5) as i32;
                self.FUN_001dc4b0(h, d % 8);
            }
        } else if m < 0.5 {
            self.FUN_001dc4b0(h, -1);
        }
    }

    /// 0x001dc260: page change (-1 = close)
    pub fn FUN_001dc260(&mut self, h: &mut dyn ShopHost, s: i32) {
        let s = if self.scr { s } else { -1 };
        if s == self.page {
            return;
        }
        if self.page == -1 {
            h.clock_scale(0.0);
            h.world_pause(true);
            h.vt_open();
            h.scr_212a70(1);
        }
        let prev = self.page;
        self.page = s;
        self.page_t = h.time();
        match s {
            0 => {
                h.scr_ids(0x4d4, 0x4d5, 0x4d6);
                if prev == 1 {
                    h.scr_212a70(1);
                    h.scr_212a70(2);
                } else {
                    h.scr_212a50(2);
                }
                h.sound(0x35);
            }
            1 => {
                h.scr_ids(0x49b, 0x49c, 0x49d);
                if prev == 0 {
                    h.scr_212a70(1);
                    h.scr_212a70(2);
                } else {
                    h.scr_212a50(2);
                }
                h.sound(0x35);
            }
            -1 => {
                h.scr_close();
                h.clock_scale(1.0);
                h.world_pause(false);
                h.vt_close();
                if 0 < h.scr_state() {
                    h.scr_212a50(1);
                }
            }
            _ => {}
        }
        self.FUN_001dbd10(h);
        self.FUN_001dbbe0(h, 0);
        self.FUN_001dc4b0(h, -1);
    }

    /// 0x001dbef0: rows, cursor, scroll arrows, dialog
    pub fn FUN_001dbef0(&mut self, h: &mut dyn ShopHost) {
        if h.scr_state() != 2 {
            return;
        }
        let mut ty = 0.0f32;
        let end = self.FUN_001dbba0();
        if self.first < 1 {
            h.scr_place(-1, Slot::Up);
        } else {
            let e = if self.page == 0 { 0x4da } else { 0x4a1 };
            if self.da570 {
                h.scr_xform(e, ty);
                ty += ROW_H;
            }
            h.scr_place(e, Slot::Up);
        }
        let mut i = self.first;
        while i < end {
            let row = ROW_ELEMS[(i - self.first) as usize];
            // rows past +0x48 read the cleared (-1) entries
            let id = self.ent[i as usize][0];
            if self.page == 0 {
                if id == -1 {
                    h.scr_213a08(0x455, row);
                    h.scr_213a08(self.ent[i as usize][1], row);
                    self.ent[i as usize][1] = -1;
                } else {
                    let x = &mut self.ent[i as usize];
                    match GADGETS[id as usize].owner {
                        8 => x[1] = 0x457,
                        7 => x[1] = 0x456,
                        9 => x[1] = 0x458,
                        _ => {}
                    }
                    let e = x[1];
                    h.scr_213bd8(0x455, row);
                    h.scr_213bd8(e, row);
                }
            } else if self.page == 1 {
                if id == -1 {
                    h.scr_213a08(0x455, row);
                } else {
                    h.scr_213bd8(0x455, row);
                }
            }
            i += 1;
        }
        let n;
        if self.sel < 0 {
            h.scr_hide(0x437);
            n = self.count;
        } else {
            h.scr_show(0x437);
            ty = ROW_H * (self.sel - self.first) as f32;
            h.scr_xform(0x437, ty);
            n = self.count;
        }
        if end < n {
            let e = if self.page == 0 { 0x4dd } else { 0x4a4 };
            if self.da570 {
                h.scr_xform(e, ty);
            }
            h.scr_place(e, Slot::Down);
        } else {
            h.scr_place(-1, Slot::Down);
        }
        h.scr_place(if self.dialog == -1 { 0x439 } else { 0x43b }, Slot::Dialog);
    }

    /// 0x001db870 (C2d84e8_v6, per frame)
    pub fn FUN_001db870(&mut self, h: &mut dyn ShopHost) {
        h.base_update();
        if self.page == -1 || !self.pad {
            return;
        }
        if self.dialog != 0 && h.pressed(PAD_TRIANGLE) {
            self.FUN_001dc260(h, -1);
            return;
        }
        if self.page == 0 {
            if h.pressed(PAD_X) {
                if self.gadget != -1 {
                    let price = GADGETS[self.gadget as usize].price;
                    let coins = h.coins();
                    if self.dialog == 0 {
                        self.FUN_001da950(h, -1, -1);
                        h.set_coins(coins - price);
                        h.acquire(self.gadget, true);
                        self.FUN_001dbd10(h);
                        h.sound(0x29);
                        h.world_19f0a8();
                    } else if coins < price {
                        self.FUN_001da950(h, 1, 0x35);
                    } else {
                        self.FUN_001da950(h, 0, 0x34);
                    }
                } else {
                    h.sound(0x36);
                }
            }
        } else if self.page == 1 && h.pressed(PAD_X) {
            if self.loot == -1 {
                h.sound(0x36);
            } else {
                let kc = h.loot(self.loot);
                if 0 < kc {
                    h.set_loot(self.loot, kc - 1);
                    let c = h.coins();
                    h.set_coins(c + self.loot_price.get(self.loot as usize).copied().unwrap_or(0));
                    self.FUN_001dbd10(h);
                    h.sound(0x29);
                }
            }
        }
        if self.dialog == 0 && h.pressed(PAD_TRIANGLE) {
            self.FUN_001da950(h, -1, -1);
        }
        if h.pressed(PAD_SQUARE) {
            self.FUN_001da950(h, -1, -1);
            let to = (self.page == 0) as i32;
            self.FUN_001dc260(h, to);
        }
        if self.dialog != 0 {
            self.FUN_001da858(h);
        }
        self.FUN_001dbef0(h);
    }
}
