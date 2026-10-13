//! WP-X1: language / region selection (DAT_002e1e58 region, DAT_002e1e54 language).
//!
//! Boot FUN_001613a8: FUN_001a2c20(0) (region 0 = the US image), then FUN_001a2da0(sceScfGetLanguage())
//! picks the system language if the region offers it. The language menu (title triangle, only with > 1
//! languages) on-select FUN_001dd8e0 sets the language and reloads the level. FUN_001d8ff8 applies the stored
//! `flags lang` only with > 1 languages, so on the US image the language is always 0 (English).
#![allow(non_snake_case)]

/// Region table 0x2bfbd0: 7 x {s32 count, s32* list} (lists 0x2bfb60..0x2bfbc8).
pub const REGIONS_2BFBD0: [&[i32]; 7] = [
    &[0],
    &[0],
    &[0, 1, 3, 4, 2],
    &[0, 1, 3, 4, 2, 5, 6, 7, 8, 9, 10],
    &[11],
    &[12],
    &[13],
];

/// 0x2a69e8: 14 x s32, game language index -> sceScf language code (-1 = none).
/// (1 English, 2 French, 5 Italian, 4 German, 3 Spanish, 6 Dutch, 7 Portuguese, 0 Japanese.)
pub const SCF_LANGUAGE_2A69E8: [i32; 14] = [1, 2, 5, 4, 3, 6, 7, -1, -1, -1, -1, 0, -1, -1];

/// sceScfGetLanguage (FUN_0024a860, SDK, replaced): the PS2 system configuration language.
/// Port: always SCE_ENGLISH_LANGUAGE (1).
pub fn FUN_0024a860() -> i32 {
    1
}

/// DAT_002e1e58 / DAT_002e1e54
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Language {
    /// DAT_002e1e58
    pub region: i32,
    /// DAT_002e1e54
    pub lang: i32,
}

impl Language {
    /// FUN_001a2c20(region): -1 -> 0.
    pub fn FUN_001a2c20(&mut self, region: i32) {
        self.region = if region == -1 { 0 } else { region };
    }

    /// FUN_001a2c48: the region's language list (region > 6 is reset to 0 first, quirk kept: written back).
    pub fn FUN_001a2c48(&mut self) -> &'static [i32] {
        if 6 < self.region {
            self.region = 0;
        }
        // negative regions would index before the table in the original (never set)
        REGIONS_2BFBD0[self.region.clamp(0, 6) as usize]
    }

    /// FUN_001a2c98: number of languages of the region.
    pub fn FUN_001a2c98(&mut self) -> i32 {
        self.FUN_001a2c48().len() as i32
    }

    /// FUN_001a2bd0(lang): -1 = the region's first language.
    pub fn FUN_001a2bd0(&mut self, lang: i32) {
        self.lang = if lang == -1 { self.FUN_001a2c48()[0] } else { lang };
    }

    /// FUN_001a2da0(scf): first game language whose sceScf code is `scf` and that the region offers, else
    /// FUN_001a2bd0(-1).
    pub fn FUN_001a2da0(&mut self, scf: i32) {
        for (i, &code) in SCF_LANGUAGE_2A69E8.iter().enumerate() {
            if code == scf && self.FUN_001a2c48().contains(&(i as i32)) {
                self.FUN_001a2bd0(i as i32);
                return;
            }
        }
        self.FUN_001a2bd0(-1);
    }

    /// FUN_001613a8 language part (boot): region 0, system language.
    pub fn boot() -> Self {
        let mut l = Language::default();
        l.FUN_001a2c20(0);
        l.FUN_001a2da0(FUN_0024a860());
        l
    }

    /// FUN_001dd8e0(item): language menu on-select = FUN_001a2bd0(item +8), then
    /// FUN_0022b490(0x3dd490, 0, 0x189, -1, 1, -1) (level reload request; returned for the caller to send).
    pub fn FUN_001dd8e0(&mut self, item_lang: i32) -> LanguageReload {
        self.FUN_001a2bd0(item_lang);
        LanguageReload { msg: 0x189 }
    }
}

/// FUN_0022b490(0x3dd490, 0, msg, -1, 1, -1) request emitted by FUN_001dd8e0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LanguageReload {
    pub msg: i32,
}
