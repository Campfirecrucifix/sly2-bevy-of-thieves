//! WP-I4: option values behind the options menu (0x2bfec8) and the save-variable paths the UI uses.
//!
//! FUN_0020e3c0 (level start) resolves fixed save-variable paths into world +0x3b1c.. (= DAT_003e0fac..):
//! `dt` (+0x3b1c, 0x3e0fac, slot in use when != 0), `loot ccoin` (+0x3b20, 0x3e0fb0 = the HUD coin counter;
//! I3 looked up a "coins" variable, which does not exist), `widCurrent`, `episodeCurrent`, then
//! `flags {lang nMixerMode uMusicVolume uEffectsVolume fUseHeadset dxScreenCenter dyScreenCenter fNoVibration
//! fReverseLookAround fReverseAim fAutosaveWarned}` = 0x3e0fbc..0x3e0fe4.
//! FUN_001afb88 writes an option into the save blob 0x3d4a60, FUN_001d8ff8 re-applies every option from it,
//! FUN_001af9c8 (item on-push) shows the stored value, FUN_001af6d0 is the left/right slider (live values).
//! The engine values they drive (989snd mixer 0x3dd040 levels, GS DISPLAY DX/DY, DAT_002c2d8c vibration,
//! DAT_002e1e54 language) are kept in `OptState`; the Bevy side maps them in `ui::plugin`.
#![allow(non_snake_case)]
use crate::engine::game::save::{GameData, SaveValue, SaveVars};

/// Save-variable paths of FUN_0020e3c0 (symbol ids; None = a name is not in the level's table).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SavePaths {
    /// +0x3b1c / DAT_003e0fac ["dt"]
    pub dt: Option<Vec<u16>>,
    /// +0x3b20 / DAT_003e0fb0 ["loot" "ccoin"]
    pub coins: Option<Vec<u16>>,
    /// DAT_003e0fbc..DAT_003e0fe4 ["flags" FLAG_NAMES[k]]
    pub flags: [Option<Vec<u16>>; 11],
}

/// Children of "flags" in FUN_0020e3c0 order (0x2a89a8..0x2a8a48).
pub const FLAG_NAMES: [&str; 11] = [
    "lang",
    "nMixerMode",
    "uMusicVolume",
    "uEffectsVolume",
    "fUseHeadset",
    "dxScreenCenter",
    "dyScreenCenter",
    "fNoVibration",
    "fReverseLookAround",
    "fReverseAim",
    "fAutosaveWarned",
];
pub const F_LANG: usize = 0;
pub const F_MIXER: usize = 1;
pub const F_MUSIC: usize = 2;
pub const F_EFFECTS: usize = 3;
pub const F_HEADSET: usize = 4;
pub const F_DX: usize = 5;
pub const F_DY: usize = 6;
pub const F_NO_VIBRATION: usize = 7;
pub const F_REVERSE_LOOK: usize = 8;
pub const F_REVERSE_AIM: usize = 9;
pub const F_AUTOSAVE_WARNED: usize = 10;

impl SavePaths {
    /// FUN_0020e3c0 path part: FUN_001d88d0(names...) = FUN_001d87c0 -> FUN_001d8720 per name. The original
    /// builds the path even when a name is missing (FUN_00233800 interns it); a missing symbol can never
    /// match a record, so the port keeps None.
    pub fn FUN_0020e3c0(vars: &SaveVars) -> Self {
        let path = |names: &[&str]| -> Option<Vec<u16>> {
            names.iter().map(|n| u16::try_from(vars.ee_sub_1d8720(n)).ok()).collect()
        };
        SavePaths {
            dt: path(&["dt"]),
            coins: path(&["loot", "ccoin"]),
            flags: std::array::from_fn(|k| path(&["flags", FLAG_NAMES[k]])),
        }
    }
}

/// Engine values the options drive.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OptState {
    /// DAT_002e1e54 language
    pub lang: i32,
    /// DAT_002e1e58 region (X1: `ui::language`; 0 = US image, set at boot by FUN_001613a8)
    pub region: i32,
    /// 0x3dd040 +0x104 mixer mode (FUN_001fe9c8)
    pub mixer_mode: i32,
    /// DAT_003dd190 = mixer level channel 1 slot 1 (music), 0..1
    pub music: f32,
    /// DAT_003dd18c = mixer level channel 0 (and 3) slot 1 (effects), 0..1
    pub effects: f32,
    /// DAT_003dd050
    pub headset: f32,
    /// DAT_002f6468 / DAT_002f646c: GS DISPLAY DX / DY (default 636 / 50)
    pub screen_x: f32,
    pub screen_y: f32,
    /// DAT_002c2d8c: 0/3 off, 2 on, 1 = read "fNoVibration" (initial value 0 in the ELF data)
    pub vibration: i32,
}

impl Default for OptState {
    fn default() -> Self {
        OptState { lang: 0, region: 0, mixer_mode: 0, music: 1.0, effects: 1.0, headset: 0.0, screen_x: 636.0, screen_y: 50.0, vibration: 0 }
    }
}

/// Snapshot of the stored option values FUN_001af9c8 reads (FUN_002581c0 on 0x3d4a60).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SavedOpts {
    pub mixer_mode: i32,
    pub no_vibration: i32,
    pub reverse_look: i32,
    pub reverse_aim: i32,
}

/// Raw payload word of a save variable (FUN_002581c0 then CRef +4), 0 when the path does not resolve.
pub fn word(vars: &SaveVars, data: &GameData, p: &Option<Vec<u16>>) -> u32 {
    let Some(p) = p else { return 0 };
    match vars.ee_sub_1d7ed8(p, data) {
        SaveValue::Int(i) => i as u32,
        SaveValue::Float(f) => f.to_bits(),
        SaveValue::Bool(b) => b as u32,
        _ => 0,
    }
}

impl SavedOpts {
    pub fn read(vars: &SaveVars, data: &GameData, paths: &SavePaths) -> Self {
        let w = |k: usize| word(vars, data, &paths.flags[k]) as i32;
        SavedOpts { mixer_mode: w(F_MIXER), no_vibration: w(F_NO_VIBRATION), reverse_look: w(F_REVERSE_LOOK), reverse_aim: w(F_REVERSE_AIM) }
    }
}

/// FUN_001af9c8(item): set a toggle item's string id from the stored value (on-push).
pub fn FUN_001af9c8(id: i32, s: &SavedOpts, st: &OptState) -> i32 {
    match id {
        0x48 | 0x49 => {
            let on = match st.vibration {
                1 => s.no_vibration,
                3 => 1,
                _ => 0,
            };
            if on != 0 {
                0x49
            } else {
                0x48
            }
        }
        // 0x4c stereo? = 1, 0x4d = 2, everything else 0x4e
        0x4c..=0x4e => match s.mixer_mode {
            1 => 0x4c,
            2 => 0x4d,
            _ => 0x4e,
        },
        // DAT_003dd050 (the live headset flag, not the save variable)
        0x4f | 0x50 => {
            if st.headset != 0.0 {
                0x4f
            } else {
                0x50
            }
        }
        0x51 | 0x52 => {
            if s.reverse_look == 0 {
                0x51
            } else {
                0x52
            }
        }
        0x53 | 0x54 => {
            if s.reverse_aim != 0 {
                0x54
            } else {
                0x53
            }
        }
        _ => id,
    }
}

/// FUN_001afb88(mgr, item): store the option of string id `id` into the save blob, then FUN_001d8ff8.
/// Returns true when the original also calls FUN_001aeaa8(mgr, 9) (slider items 0x3f / 0x67 / 0x68).
pub fn FUN_001afb88(id: i32, st: &mut OptState, vars: &SaveVars, paths: &SavePaths, data: &mut GameData) -> bool {
    let mut put = |k: usize, v: SaveValue| {
        if let Some(p) = &paths.flags[k] {
            vars.ee_sub_1d80c8(p, v, data);
        }
    };
    let state9 = match id {
        0x3f => {
            put(F_DX, SaveValue::Float(st.screen_x - 636.0));
            put(F_DY, SaveValue::Float(st.screen_y - 50.0));
            true
        }
        0x48 | 0x49 => {
            st.vibration = if id == 0x49 { 3 } else { 2 };
            // tag 0x17 Bool with a float payload of (id == 0x49) -> stored word 0 / 0x3f800000
            put(F_NO_VIBRATION, SaveValue::Float((id == 0x49) as u32 as f32));
            false
        }
        // tag 0x15 S32: 0x4c = 1, 0x4d = 2, 0x4e = 0
        0x4c => {
            put(F_MIXER, SaveValue::Int(1));
            false
        }
        0x4d => {
            put(F_MIXER, SaveValue::Int(2));
            false
        }
        0x4e => {
            put(F_MIXER, SaveValue::Int(0));
            false
        }
        0x4f | 0x50 => {
            put(F_HEADSET, SaveValue::Float((id == 0x4f) as u32 as f32));
            false
        }
        0x51 | 0x52 => {
            put(F_REVERSE_LOOK, SaveValue::Float((id == 0x52) as u32 as f32));
            false
        }
        0x53 | 0x54 => {
            put(F_REVERSE_AIM, SaveValue::Float((id == 0x54) as u32 as f32));
            false
        }
        // quirk kept: the "volume" variables store 1 - level (FUN_001d8ff8 applies 1 - stored)
        0x67 => {
            put(F_MUSIC, SaveValue::Float(1.0 - st.music));
            true
        }
        0x68 => {
            put(F_EFFECTS, SaveValue::Float(1.0 - st.effects));
            true
        }
        _ => false,
    };
    FUN_001d8ff8(st, vars, paths, data);
    state9
}

/// FUN_001d8ff8(blob): apply every stored option. `lang` only when FUN_001a2c98 (> 1 languages; X1:
/// `ui::language`, never on the US region).
pub fn FUN_001d8ff8(st: &mut OptState, vars: &SaveVars, paths: &SavePaths, data: &GameData) {
    let f = |k: usize| paths.flags[k].as_ref().map(|_| f32::from_bits(word(vars, data, &paths.flags[k])));
    let i = |k: usize| paths.flags[k].as_ref().map(|_| word(vars, data, &paths.flags[k]) as i32);
    if let Some(v) = i(F_LANG) {
        let mut l = super::language::Language { region: st.region, lang: st.lang };
        if 1 < l.FUN_001a2c98() {
            // FUN_001a2bd0: -1 = the region's first language
            l.FUN_001a2bd0(v);
        }
        st.region = l.region;
        st.lang = l.lang;
    }
    if let Some(v) = i(F_MIXER) {
        st.mixer_mode = v;
    }
    if let Some(v) = f(F_MUSIC) {
        st.music = clamp01(1.0 - v);
    }
    if let Some(v) = f(F_EFFECTS) {
        st.effects = clamp01(1.0 - v);
    }
    if let Some(v) = f(F_HEADSET) {
        st.headset = v;
    }
    // DX/DY are written back as integers (FUN_0023e1c8) into the GS DISPLAY registers
    if let Some(v) = f(F_DX) {
        st.screen_x = ((v + 636.0) as i32 & 0xfff) as f32;
    }
    if let Some(v) = f(F_DY) {
        st.screen_y = ((v + 50.0) as i32 & 0x7fe) as f32;
    }
}

/// FUN_0011d948 with the range 0x2bbb60 = [0, 1].
pub fn clamp01(v: f32) -> f32 {
    if (0.0..=1.0).contains(&v) {
        v
    } else if v < 0.0 {
        0.0
    } else {
        1.0
    }
}

/// FUN_001af6d0(mgr, frame, item): slider step for the selected item. `dx` / `dy` = FUN_00187e88 /
/// FUN_001880d8 (-1, 0, 1), `dt` = DAT_002ddf10. Returns true when a value changed.
pub fn FUN_001af6d0(id: i32, st: &mut OptState, dx: i32, dy: i32, dt: f32) -> bool {
    match id {
        0x67 => {
            if dx == 0 {
                return false;
            }
            st.music = clamp01(st.music + dx as f32 * dt * 1.5);
            true
        }
        0x68 => {
            if dx == 0 {
                return false;
            }
            st.effects = clamp01(st.effects + dx as f32 * dt * 1.5);
            true
        }
        0x3f => {
            if dx == 0 && dy == 0 {
                return false;
            }
            // DAT_002c01c8 = 5.0, DAT_002c01cc = 2.0 per step; DX in [424, 848], DY in [10, 90]
            st.screen_x = (st.screen_x + dx as f32 * 5.0).clamp(424.0, 848.0);
            st.screen_y = (st.screen_y + dy as f32 * 2.0).clamp(10.0, 90.0);
            true
        }
        _ => false,
    }
}

/// FUN_001c7d08: clue bottles collected in the current world = popcount of the "afClue" bit words of every
/// child of the world's save-variable node (`world` = the 3 symbols at level-info +4, FUN_002109b8).
pub fn FUN_001c7d08(vars: &SaveVars, data: &GameData, world: [u16; 3]) -> i32 {
    let Ok(clue) = u16::try_from(vars.ee_sub_1d8720("afClue")) else { return 0 };
    let Some((node, _)) = vars.ee_sub_1d7da0(&world) else { return 0 };
    let mut n = 0;
    let mut c = node.child;
    while c != 0 {
        let Some(r) = vars.recs.get(c as usize).copied() else { break };
        // FUN_001d7e80 = pointer to the first word of the variable; only that word is counted
        if let Some((v, idx)) = vars.ee_sub_1d7da0(&[world[0], world[1], world[2], r.sym as u16, clue]) {
            let o = v.offset as usize + 4 * idx as usize;
            let w = data.data.get(o..o + 4).map_or(0, |b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
            n += w.count_ones() as i32;
        }
        c = r.next;
    }
    n
}

/// Port addition: options of the "Graphics" page (settings.toml `[video]`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphicsOpts {
    /// `global_illumination = "raytraced"`
    pub raytraced: bool,
    /// whether "Ray traced" is offered (feature `rt` and a GPU with the Solari features)
    pub rt_available: bool,
    pub hor_plus: bool,
    pub ink_outlines: bool,
    /// WIRE-OPT: settings.toml `[graphics] rt_quality` (0 low, 1 medium, 2 high, 3 auto; live)
    pub rt_quality: u8,
    /// WIRE-OPT: `[graphics] hud = "stretch"` (else the centred 16:9 safe area; live)
    pub hud_stretch: bool,
    /// UPSCALE: `[graphics] render_scale / gi_scale / sharpness / upscaler` (live)
    pub upscale: crate::engine::gfx::upscale::UpscaleOpts,
}

impl Default for GraphicsOpts {
    fn default() -> Self {
        GraphicsOpts { raytraced: false, rt_available: false, hor_plus: true, ink_outlines: true, rt_quality: 3, hud_stretch: false, upscale: Default::default() }
    }
}

use crate::engine::gfx::upscale::opts::SCALES as UP_SCALES;

/// Rows of the Graphics page (row kind, label).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GfxRow {
    Lighting,
    HorPlus,
    Ink,
    RtQuality,
    Hud,
    RenderScale,
    GiScale,
    Sharpness,
    Upscaler,
}

impl GraphicsOpts {
    pub fn rows(&self) -> Vec<GfxRow> {
        let mut r = Vec::new();
        if self.rt_available {
            r.push(GfxRow::Lighting);
            r.push(GfxRow::RtQuality);
        }
        r.push(GfxRow::HorPlus);
        r.push(GfxRow::Ink);
        r.push(GfxRow::Hud);
        r.push(GfxRow::RenderScale);
        if self.rt_available {
            r.push(GfxRow::GiScale);
        }
        r.push(GfxRow::Sharpness);
        r.push(GfxRow::Upscaler);
        r
    }

    pub fn label(&self, row: GfxRow) -> String {
        let on = |b: bool| if b { "On" } else { "Off" };
        match row {
            // "Global Illumination" does not fit the 320-unit menu row at scale 1.0 -> "GI"
            GfxRow::Lighting => format!("GI: {}", if self.raytraced { "Ray traced" } else { "Off" }),
            GfxRow::HorPlus => format!("Aspect: {}", if self.hor_plus { "Hor+" } else { "Original" }),
            GfxRow::Ink => format!("Ink Outlines: {}", on(self.ink_outlines)),
            GfxRow::RtQuality => format!("RT Quality: {}", ["Low", "Medium", "High", "Auto"][self.rt_quality.min(3) as usize]),
            GfxRow::Hud => format!("HUD: {}", if self.hud_stretch { "Stretch" } else { "Centre" }),
            GfxRow::RenderScale => format!("Render Scale: {}", UP_SCALES[(self.upscale.scale as usize).min(3)].1),
            GfxRow::GiScale => format!("RT Scale: {}", if self.upscale.gi_scale == 0 { "Same" } else { UP_SCALES[(self.upscale.gi_scale as usize).min(3)].1 }),
            GfxRow::Sharpness => format!("Sharpness: {}%", self.upscale.sharp.min(10) * 10),
            GfxRow::Upscaler => format!("Upscaler: {}", crate::engine::gfx::upscale::opts::METHODS[(self.upscale.method as usize).min(1)].1),
        }
    }

    pub fn toggle(&mut self, row: GfxRow) {
        match row {
            GfxRow::Lighting => self.raytraced = self.rt_available && !self.raytraced,
            GfxRow::HorPlus => self.hor_plus = !self.hor_plus,
            GfxRow::Ink => self.ink_outlines = !self.ink_outlines,
            GfxRow::RtQuality => self.rt_quality = (self.rt_quality.min(3) + 1) % 4,
            GfxRow::Hud => self.hud_stretch = !self.hud_stretch,
            GfxRow::RenderScale => self.upscale.scale = (self.upscale.scale.min(3) + 1) % 4,
            GfxRow::GiScale => self.upscale.gi_scale = (self.upscale.gi_scale.min(3) + 1) % 4,
            GfxRow::Sharpness => self.upscale.sharp = (self.upscale.sharp.min(10) + 1) % 11,
            GfxRow::Upscaler => self.upscale.method = (self.upscale.method.min(1) + 1) % 2,
        }
    }
}
