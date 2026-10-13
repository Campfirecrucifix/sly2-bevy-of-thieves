//! The game clock g_clock at 0x2dded0 (CLOCK). 1:1 except the time source: the EE `Count` register
//! (294.912 MHz, FUN_0014d240 with its own wrap counter) is replaced by host elapsed seconds.

/// EE cycles -> seconds factor used by FUN_0014d0b8 (1 / 294912000).
pub const SECONDS_PER_TICK: f32 = 3.390842e-09;
/// dt clamps of FUN_0014d0b8: never below one 60 Hz frame, never above two.
pub const DT_MIN: f32 = 0.016666668;
pub const DT_MAX: f32 = 0.033333335;

/// Scale slot used by pause (FUN_001ae9e8 sets slot 0 to 0.0, FUN_001aea40 back to 1.0).
pub const SCALE_PAUSE: usize = 0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clock {
    /// +0x00 DAT_002dded0: game time (scaled)
    pub t: f32,
    /// +0x04 DAT_002dded4: game dt (scaled, 0 while paused)
    pub dt: f32,
    /// +0x08: product of the 9 scales (FUN_0014d1f8)
    pub r_dt: f32,
    /// +0x0c: = scales[8] (+0x38)
    pub r_dt_real: f32,
    /// +0x10 DAT_002ddee0: real dt (scaled only by scales[8])
    pub dt_real: f32,
    /// +0x14: accumulated real time
    pub t_real: f32,
    /// +0x18..+0x38: 9 multiplicative time scales
    pub scales: [f32; 9],
    /// +0x3c: accumulated "scale 3" time
    pub t_3: f32,
    /// +0x40: dt * scales[3]
    pub dt_3: f32,
    /// +0x48: last tick (replaced: host seconds)
    pub tick: f64,
}

impl Default for Clock {
    fn default() -> Self {
        let mut c = Self { t: 0.0, dt: 0.0, r_dt: 0.0, r_dt_real: 0.0, dt_real: 0.0, t_real: 0.0, scales: [0.0; 9], t_3: 0.0, dt_3: 0.0, tick: 0.0 };
        ee_sub_14cfe0(&mut c, 0.0);
        c
    }
}

/// FUN_0014cfe0: clock startup — all 9 scales = 1, +0x08/+0x0c = 1, then FUN_0014d068.
pub fn ee_sub_14cfe0(c: &mut Clock, now: f64) {
    // 0x0014cfe0
    c.tick = now;
    c.scales = [1.0; 9];
    c.r_dt_real = 1.0;
    c.r_dt = 1.0;
    ee_sub_14d068(c);
}

/// FUN_0014d068: reset at level load. t = 0 and every scale except slot 3 (+0x24) back to 1.0 (quirk kept).
pub fn ee_sub_14d068(c: &mut Clock) {
    // 0x0014d068
    c.t = 0.0;
    for i in [0, 1, 2, 4, 5, 6, 7, 8] {
        c.scales[i] = 1.0;
    }
    ee_sub_14d1f8(c);
}

/// FUN_0014d1f8: recompute r_dt (product of scales) and r_dt_real (= scales[8]).
pub fn ee_sub_14d1f8(c: &mut Clock) {
    // 0x0014d1f8
    c.r_dt = 1.0;
    for s in c.scales {
        c.r_dt *= s;
    }
    c.r_dt_real = c.scales[8];
}

/// FUN_0014d1d0(value, clock, index): set one scale (pause uses slot 0).
pub fn ee_sub_14d1d0(value: f32, c: &mut Clock, index: usize) {
    // 0x0014d1d0
    c.scales[index] = value;
    ee_sub_14d1f8(c);
}

/// FUN_0014d0b8(clock, fRealOnly): per-frame advance. `now` = host seconds (replaces Count).
/// Returns true when the game time advanced (the original then calls FUN_00161478(0x2deb30)).
pub fn ee_sub_14d0b8(c: &mut Clock, now: f64, real_only: bool) -> bool {
    // 0x0014d0b8
    let raw = (now - c.tick) as f32;
    let mut f = DT_MIN;
    if DT_MIN <= raw {
        f = raw;
        if DT_MAX < raw {
            f = DT_MAX;
        }
    }
    c.dt_real = f * c.r_dt_real;
    c.dt_3 = f * c.scales[3];
    c.t_real += f * c.r_dt_real;
    if !real_only {
        let d = f * c.scales[3] * c.r_dt;
        c.dt = d;
        c.t += d;
    }
    c.tick = now;
    c.t_3 += c.dt_3;
    !real_only
}

/// FUN_001ae9e8 (clock part): pause.
pub fn pause(c: &mut Clock) {
    ee_sub_14d1d0(0.0, c, SCALE_PAUSE);
}

/// FUN_001aea40 (clock part): unpause.
pub fn unpause(c: &mut Clock) {
    ee_sub_14d1d0(1.0, c, SCALE_PAUSE);
}

impl Clock {
    pub fn paused(&self) -> bool {
        self.scales[SCALE_PAUSE] == 0.0
    }
}
