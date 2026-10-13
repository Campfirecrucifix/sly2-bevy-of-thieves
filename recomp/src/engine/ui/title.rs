//! WP-A11: title / episode screen 0x3da530 input (FUN_001dd920) and the new-game / continue start
//! (FUN_001ddb08, FUN_00165e38, FUN_001d54b8).
//!
//! FUN_001dd920(screen) per frame while the screen is active (+0x24) and has a pad (+0x10):
//! - START (FUN_00187d80 0x800): DAT_002f6330 = DAT_002f6334 (card slot chosen on the load menu, 0 = none);
//!   0 -> FUN_001ddb08 (new game), else FUN_001acc38(0x2e6550, 1) + FUN_00165e38(slot) (continue).
//! - else SELECT pressed (FUN_00187dd0 0x100) with buttons +0x10b4 & 0x108 == 0x100 (START not held):
//!   consume SELECT, FUN_001b04e0(0) = options menu pushed from the title.
//! - else triangle (0x10) when FUN_001a2c98() > 1 (language count of the region DAT_002e1e58 table
//!   0x2bfbd0) and FUN_0015e938(0x3dd490, 5, 0x5d6, 0): language menu (items DAT_003da578, on-select
//!   FUN_001dd8e0 = set language + reload the level). The US image has one language -> never taken.
//!
//! REPLACED: FUN_001da808 (screen object timing / FUN_001da950), the screen hide vslots of FUN_001dd888
//! (the port's title is `UiGame::title`), FUN_001acc38 card refresh and the load path (no memory card
//! slots are ported: `DAT_002f6334` is always 0 in the port), FUN_001440d0 hp refill of the three
//! characters (P7 `Dmg` starts full), FUN_0022b490 transitions (returned as `TitleStart::Transition`;
//! WP-J14: also posted with the job manager slots 0x2deb40..4c to `game::flow::FUN_0022b490`, which the
//! game binary turns into the level transition).
#![allow(non_snake_case)]
use super::settings::{word, SavePaths, F_NO_VIBRATION};
use crate::engine::game::save::{GameData, SaveValue, SaveVars};
use crate::engine::input::pad::{self, PadState};

/// Result of FUN_001dd920 for the UI side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleInput {
    None,
    /// START with no slot -> FUN_001ddb08
    NewGame,
    /// START with slot DAT_002f6334 != 0 -> FUN_00165e38(slot)
    Continue(i32),
    /// SELECT -> FUN_001b04e0(0)
    Options,
    /// triangle with > 1 languages (FUN_001a2c48 list)
    Language,
}

/// FUN_001dd920 input part. `slot` = DAT_002f6334, `languages` = FUN_001a2c98().
pub fn FUN_001dd920(active: bool, pad: &mut PadState, slot: i32, languages: i32) -> TitleInput {
    if !active {
        return TitleInput::None;
    }
    if pad::FUN_00187d80(pad, 0x800) != 0 {
        return if slot == 0 { TitleInput::NewGame } else { TitleInput::Continue(slot) };
    }
    if pad::FUN_00187dd0(pad, 0x100) != 0 && pad.buttons & 0x108 == 0x100 {
        pad::FUN_00187de0(pad, 0x100);
        return TitleInput::Options;
    }
    // FUN_0015e938 (the language dialog opener) is assumed to succeed
    if 1 < languages && pad::FUN_00187d80(pad, 0x10) != 0 {
        return TitleInput::Language;
    }
    TitleInput::None
}

/// FUN_00165e38 outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleStart {
    /// title active and `dt` == 0: FUN_001d54b8 + FUN_001dd888 (hide the title, play on in the level
    /// already loaded behind it)
    HideTitle,
    /// FUN_0022b490(0x3dd490, level row, spawn 0x189, -1, flags, -1); DAT_002deb48 = `movie`
    /// (0x62f for the new-game intro, else -1)
    Transition { row: i32, flags: u32, movie: i32 },
}

/// FUN_001d54b8: store the vibration mode DAT_002c2d8c (2 = on, 3 = off) into "flags fNoVibration"
/// (tag 0x17 Bool, payload 0 / 1).
pub fn FUN_001d54b8(vibration: i32, vars: &SaveVars, paths: &SavePaths, data: &mut GameData) {
    let v = match vibration {
        2 => false,
        3 => true,
        _ => return,
    };
    if let Some(p) = &paths.flags[F_NO_VIBRATION] {
        vars.ee_sub_1d80c8(p, SaveValue::Bool(v), data);
    }
}

/// FUN_00165e38(save): start from the save blob (the port's live `GameData`). `title` = FUN_001d9880(0x3da530).
/// `episode` = save variable DAT_003e0fb8 (the episode / world row; None = not resolved -> 0).
pub fn FUN_00165e38(title: bool, vibration: i32, vars: &SaveVars, paths: &SavePaths, data: &mut GameData, episode: Option<i32>) -> TitleStart {
    let dt = f32::from_bits(word(vars, data, &paths.dt));
    if dt == 0.0 {
        if title {
            FUN_001d54b8(vibration, vars, paths, data);
            return TitleStart::HideTitle;
        }
        // FUN_00210b98(0): level table row 0 (the intro)
        return post(TitleStart::Transition { row: 0, flags: 0x48, movie: 0x62f });
    }
    // FUN_00210b98(episode) / completion > 99 -> 0x2a3828 (hideout row): the completion lookup
    // DAT_003e100c[episode] is not ported -> the episode row
    post(TitleStart::Transition { row: episode.unwrap_or(0), flags: 0x40, movie: -1 })
}

/// WP-J14: DAT_002deb48 = `movie` (restart task), DAT_002deb40 / 44 = -1, DAT_002deb4c = the live blob,
/// FUN_0022b490(0x3dd490, FUN_00210b98(row), 0x189, -1, flags, -1).
fn post(t: TitleStart) -> TitleStart {
    use crate::engine::game::flow::{self, Dest, Post};
    use crate::engine::game::job_flow::JobSlots;
    if let TitleStart::Transition { row, flags, movie } = t {
        let slots = JobSlots { job: -1, task: -1, restart: movie, slot: -1 };
        flow::FUN_0022b490(Post { dest: Dest::Slot(row), spawn: 0x189, spawn_parent: -1, flags, a: -1, slots: Some(slots) });
    }
    t
}

/// FUN_001ddb08: new game = "dt" := 0 (tag 0x16 Float) in the save blob, FUN_001440d0(hp refill) on the
/// three characters (REPLACED, see module doc), then FUN_00165e38(0x3d4a60).
pub fn FUN_001ddb08(title: bool, vibration: i32, vars: &SaveVars, paths: &SavePaths, data: &mut GameData) -> TitleStart {
    if let Some(p) = &paths.dt {
        vars.ee_sub_1d80c8(p, SaveValue::Float(0.0), data);
    }
    FUN_00165e38(title, vibration, vars, paths, data, None)
}

/// 0x2a7a70: the line break FUN_001ddbf0 appends between the title rows.
pub const STR_2A7A70: &[u8] = b"\n";

/// WP-A14: FUN_001ddbf0(text): the title screen's prompt text (string ids of the level string table,
/// FUN_0013e368(0x3dd490, id)). `alt` = DAT_002e0c0c == 1 && DAT_002e0c10 == 0 -> string 9 alone (never
/// set in the port). Else string 5 (New Game) or 6 (Continue) when a card slot is chosen (DAT_002f6334 != 0)
/// and that blob's "dt" (FUN_002581c0(DAT_003e0fac = world +0x3b1c) as float, FUN_00129ec8) is not 0,
/// then "\n" + string 8, and "\n" + string 0x56 when FUN_001a2c98() > 1 languages.
/// `dt` is only read when `slot != 0` (as in the original).
pub fn FUN_001ddbf0(alt: bool, slot: i32, dt: impl FnOnce() -> f32, languages: i32, s: &super::strings::LevelStrings) -> Vec<u8> {
    // 0x001ddbf0
    if alt {
        return s.FUN_0013e368(9).to_vec();
    }
    let f = if slot != 0 { dt() } else { 0.0 };
    let mut out = s.FUN_0013e368(if f != 0.0 { 6 } else { 5 }).to_vec();
    out.extend_from_slice(STR_2A7A70);
    out.extend_from_slice(s.FUN_0013e368(8));
    if 1 < languages {
        out.extend_from_slice(STR_2A7A70);
        out.extend_from_slice(s.FUN_0013e368(0x56));
    }
    out
}

/// WP-A14 (port REPLACEMENT of the card manager's load screen, FUN_001ab1a0 / card dialogs, which pick
/// DAT_002f6334): slot list shown over the title when save slot files exist. Row 0 = "New Game"
/// (DAT_002f6334 = 0 -> START runs FUN_001ddb08), rows 1..=3 = slot files (DAT_002f6334 = row, the blob is
/// loaded by FUN_001ad320 -> FUN_001ddbf0 then reads "Continue"). Up / down move, cross picks, triangle
/// closes (keeps the current choice); on the title, square reopens it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LoadMenu {
    pub open: bool,
    pub row: i32,
    /// completion (FUN_001d82b8) of each used slot file, None = empty slot
    pub slots: [Option<i32>; crate::engine::game::save::SLOT_COUNT],
    /// the menu was offered for this title show already
    pub offered: bool,
    /// SLY2_UI_SLOT debug pick done (port)
    pub auto: bool,
}

/// `LoadMenu::input` result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadPick {
    None,
    /// row 0: DAT_002f6334 = 0
    NewGame,
    /// slot index (DAT_002f6334 = index + 1)
    Slot(usize),
    /// triangle: closed without a change
    Close,
}

impl LoadMenu {
    pub fn any(&self) -> bool {
        self.slots.iter().any(Option::is_some)
    }
    /// One frame of input (consumes START / cross / triangle / square so the title underneath does not
    /// see them).
    pub fn input(&mut self, p: &mut PadState, clock: pad::Clock) -> LoadPick {
        let n = self.slots.len() as i32;
        let r = pad::FUN_001880d8(p, clock);
        if r != 0 {
            self.row = (self.row + r).clamp(0, n);
        }
        if pad::FUN_00187d80(p, 0x40) != 0 || pad::FUN_00187d80(p, 0x800) != 0 {
            pad::FUN_00187de0(p, 0x40 | 0x800);
            if self.row == 0 {
                self.open = false;
                return LoadPick::NewGame;
            }
            let i = self.row as usize - 1;
            if self.slots[i].is_some() {
                self.open = false;
                return LoadPick::Slot(i);
            }
            return LoadPick::None;
        }
        if pad::FUN_00187d80(p, 0x10) != 0 {
            self.open = false;
            return LoadPick::Close;
        }
        LoadPick::None
    }
    /// Row texts (port strings: the card manager's own texts are not ported).
    pub fn rows(&self) -> Vec<String> {
        let mut v = vec!["New Game".to_string()];
        for (i, s) in self.slots.iter().enumerate() {
            v.push(match s {
                Some(p) => format!("Slot {}   {p}%", i + 1),
                None => format!("Slot {}   Empty", i + 1),
            });
        }
        v
    }
}

/// WP-A14: FUN_00165e38 as the app runs it: `game::flow::FUN_00165e38_full` (J16, continue branch with
/// episode level / hideout) + FUN_001d54b8 when it only hides the title. `slot` = DAT_002f6334
/// (DAT_002deb4c blob slot). The A11 `FUN_00165e38` above stays for its callers / tests (episode row only).
pub fn FUN_00165e38_app(
    vibration: i32,
    vars: &SaveVars,
    paths: &SavePaths,
    data: &mut GameData,
    table: Option<&crate::engine::disc::descriptor::LevelTable>,
    slot: i32,
) -> crate::engine::game::flow::Start {
    use crate::engine::game::flow;
    let st = flow::FUN_00165e38_full(true, vars, &flow::StartPaths::new(vars), data, table, slot);
    if st == flow::Start::HideTitle {
        FUN_001d54b8(vibration, vars, paths, data);
    }
    st
}

/// WP-A14: FUN_001ddb08 for the app: "dt" := 0.0, FUN_001440d0 hp refill (REPLACED, module doc), then
/// `FUN_00165e38_app(slot 0)`.
pub fn FUN_001ddb08_app(
    vibration: i32,
    vars: &SaveVars,
    paths: &SavePaths,
    data: &mut GameData,
    table: Option<&crate::engine::disc::descriptor::LevelTable>,
) -> crate::engine::game::flow::Start {
    // 0x001ddb08
    if let Some(p) = &paths.dt {
        vars.ee_sub_1d80c8(p, SaveValue::Float(0.0), data);
    }
    FUN_00165e38_app(vibration, vars, paths, data, table, 0)
}
