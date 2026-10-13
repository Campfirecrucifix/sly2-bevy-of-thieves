//! WP-J16: music layer volumes of the music manager 0x3dd040 (world update step FUN_001fd830). 1:1 kernels:
//! FUN_001fdc08 (layer select +0x64 and the equal-power crossfade of the three weights +0x108) and
//! FUN_001fc2e0 (instance layer volume +0x70[layer], clamped to 0..1 by FUN_0011d948(0x2bbb60) and
//! applied only when it changes at 1/1024 resolution: dirty bit +0x3bc, FUN_001fb658(inst, 0x80, 0x80)).
//! The crossfade speed +0x114 + layer * 0xc ({vmax, v0, accel} of FUN_0011d018) is never written by the
//! ELF (0x3dd040 is BSS, FUN_001fd700 does not set it): with vmax 0 FUN_0011d018 holds the weight, so a
//! layer change takes effect when the instance (re)starts (+0x52 / +0x50 & 1: weight forced to 1, the
//! others scaled to 0). Kept 1:1 (`speed`).
//! REPLACED: the 989snd voice volume write behind FUN_001fb658 -> the Bevy sink volume of each layer
//! (`game/app.rs`); layer n = KPV channel pair n (`layer_channels`).

use crate::engine::physics::dynamics::FUN_0011d018;

/// Player (FUN_001c4a48(0x3dd490), class 0x34 by FUN_00132b10) fields FUN_001fdc08 reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerMusic {
    /// jt+0x10ec focus mode
    pub f_10ec: i32,
    /// jt+0x10c8 (-1 = none)
    pub f_10c8: i32,
    /// jt+0x4c8 attach kind
    pub f_4c8: i32,
}

/// Music instance (+0x204 stack top) state FUN_001fdc08 tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstState {
    /// +0x52 != 0 || +0x50 & 1: not yet steady (starting / loading)
    pub starting: bool,
}

/// 0x3dd040 layer part.
#[derive(Clone, Debug, PartialEq)]
pub struct MusicLayers {
    /// +0x5c / +0x60 overrides (FUN_001fd700 clears them)
    pub f_5c: i32,
    pub f_60: i32,
    /// +0x64 active layer
    pub layer: usize,
    /// +0x108 weights
    pub w: [f32; 3],
    /// +0x114 crossfade speed per layer
    pub speed: [[f32; 3]; 3],
    /// instance +0x70 layer volumes (6) and +0x3bc dirty bits
    pub vol: [f32; 6],
    pub dirty: u32,
}

impl Default for MusicLayers {
    fn default() -> Self {
        Self { f_5c: 0, f_60: 0, layer: 1, w: [0.0; 3], speed: [[0.0; 3]; 3], vol: [0.0; 6], dirty: 0 }
    }
}

impl MusicLayers {
    /// FUN_001fdc08(0x3dd040). `player`: Some when the player exists and is class 0x34. `inst`: the +0x204
    /// stack top. `dt` = DAT_002ddf10.
    pub fn ee_sub_1fdc08(&mut self, player: Option<PlayerMusic>, inst: Option<InstState>, dt: f32) {
        // 0x001fdc08
        self.layer = 1;
        if self.f_5c != 0 {
            self.layer = 2;
        } else if self.f_60 != 0 {
            self.layer = 0;
        } else if let Some(p) = player {
            if p.f_10ec == 3 {
                self.layer = 0;
            } else if p.f_10ec == 4 {
                self.layer = 2;
            } else if p.f_10c8 != -1 || (0..=2).contains(&p.f_4c8) || p.f_4c8 == 0xd {
                self.layer = 0;
            }
        }
        let a = self.layer;
        match inst {
            Some(i) if !i.starting => self.w[a] = FUN_0011d018(self.w[a], 1.0, dt, self.speed[a], None),
            _ => self.w[a] = 1.0,
        }
        let wa = self.w[a];
        let others: f32 = (0..3).filter(|&i| i != a).map(|i| self.w[i] * self.w[i]).sum();
        if others < 0.0001 {
            for i in 0..3 {
                self.w[i] = if i == a { 1.0 } else { 0.0 };
            }
        } else {
            let k = ((1.0 - wa * wa) / others).sqrt();
            for i in (0..3).filter(|&i| i != a) {
                self.w[i] *= k;
            }
        }
        if inst.is_some() {
            for i in 0..3 {
                self.ee_sub_1fc2e0(self.w[i], i);
            }
        }
    }

    /// FUN_001fc2e0(vol, inst, layer): true when the volume changed (FUN_001fb658 applies it).
    pub fn ee_sub_1fc2e0(&mut self, vol: f32, layer: usize) -> bool {
        // 0x001fc2e0
        if layer >= 6 {
            return false;
        }
        let v = vol.clamp(0.0, 1.0);
        if (v * 1024.0) as i32 == (self.vol[layer] * 1024.0) as i32 {
            return false;
        }
        self.dirty |= 1 << layer;
        self.vol[layer] = v;
        true
    }
}

/// REPLACED mapping: KPV channels of layer `n` (pairs = stereo, a trailing odd channel = mono layer).
pub fn layer_channels(n: usize, channels: usize) -> Vec<usize> {
    (2 * n..(2 * n + 2).min(channels)).collect()
}

/// Number of layers of a `channels`-channel stream.
pub fn layer_count(channels: usize) -> usize {
    channels.div_ceil(2)
}
