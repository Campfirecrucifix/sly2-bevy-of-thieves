//! Startup FUN_001a7970: loading-bar object (FUN_001c8cc0 colours 0x807f0000/0x803f3f3f/0x80003f3f/0x8000003f),
//! FUN_0021ec18 (heap), FUN_001c2a78(1), then the 0x13 init functions of the table at 0x2bfce0, each preceded
//! by a real-time clock tick FUN_0014d0b8(0x2dded0, 1). From the entry *after* FUN_001700d0 (GS/DMA init) on,
//! the bar is drawn with FUN_001c8d40(bar, 0x13 - i) before the call; finally FUN_001c8d40(bar, 0),
//! FUN_001c8db8 and FUN_001c2ba8(1).
//!
//! Game-specific entries are ported (clock 0x14cfe0, pad 0x187030, sound globals 0x1fcea0); PS2 SDK / GS / VU /
//! thread / IOP setup is replaced by Bevy (window, renderer, task pools, bevy_audio) and only recorded.

use super::clock::{self, Clock};

/// How the Rust port handles one table entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InitKind {
    /// game-specific, ported 1:1 (host performs it)
    Ported,
    /// PS2 SDK/hardware setup replaced by Bevy / std (nothing to run)
    Replaced,
}

/// One entry of the table 0x2bfce0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InitFn {
    pub addr: u32,
    pub kind: InitKind,
    pub what: &'static str,
}

use InitKind::*;

/// PTR_FUN_002bfce0[0..0x13] (read from SCUS_973.16).
pub const STARTUP_TABLE: [InitFn; super::main_loop::STARTUP_FUNCS as usize] = [
    InitFn { addr: 0x00157200, kind: Replaced, what: "DMA channels FUN_00241468/FUN_00241440 (0,1,2,8,9) -> Bevy renderer" },
    InitFn { addr: 0x0021f100, kind: Replaced, what: "main thread prio 5 + render thread FUN_00160b70 -> Bevy task pools" },
    InitFn { addr: 0x001a7630, kind: Replaced, what: "(not decompiled) SDK/IOP setup" },
    InitFn { addr: 0x001700d0, kind: Replaced, what: "GS: video mode, frame buffers, VSync/GS interrupt handlers -> Bevy window" },
    InitFn { addr: 0x001ae168, kind: Replaced, what: "(not decompiled) memory/heap setup" },
    InitFn { addr: 0x0014ba30, kind: Replaced, what: "(not decompiled) disc state machine init (DAT_002bd4c0) -> ISO reader" },
    InitFn { addr: 0x0014b660, kind: Replaced, what: "(not decompiled) CD/IOP file system init -> ISO reader" },
    InitFn { addr: 0x0015ef00, kind: Replaced, what: "FUN_0015ef30 + DAT_002de760 = DAT_002de9f0 (GS packet heap)" },
    InitFn { addr: 0x00187030, kind: Ported, what: "pad: FUN_001870d0 input lock, FUN_00242d08(0) padman, FUN_00187150(0x2dfc00,0,0), FUN_0014c2d8" },
    InitFn { addr: 0x001607a0, kind: Replaced, what: "VBlank handler 0x167f78, semaphores, DMA tag buffers 0x454/0x1090" },
    InitFn { addr: 0x001c90d0, kind: Replaced, what: "loader thread FUN_001c9190 + 3 semaphores -> AsyncComputeTaskPool" },
    InitFn { addr: 0x0014cfe0, kind: Ported, what: "clock init: tick base, 9 scales = 1.0, FUN_0014d068" },
    InitFn { addr: 0x001a7890, kind: Replaced, what: "VU0 microcode upload 0x259df0 (VU0 math is ported as Rust)" },
    InitFn { addr: 0x001a7900, kind: Replaced, what: "VU1 microcode upload 0x25e050 (renderer -> Bevy)" },
    InitFn { addr: 0x001fcea0, kind: Ported, what: "989snd init (FUN_002319e8(2), 'SUKR' RPC) + sound volumes = 1.0, tick stamps" },
    InitFn { addr: 0x00138fc0, kind: Replaced, what: "FUN_00138f08(1,0x2dd5c0)/(2,0x2dd600) memory pools -> Rust allocator" },
    InitFn { addr: 0x00211528, kind: Replaced, what: "GS register/packet templates (alpha, test, frame, zbuf) -> Bevy materials" },
    InitFn { addr: 0x001aa978, kind: Replaced, what: "FUN_001613f8(0x2deb30) activity timer, DAT_004425d8=-1, FUN_001ab128(0x2e65e0) memcard -> files" },
    InitFn { addr: 0x001b2940, kind: Replaced, what: "(not decompiled) FMV/IPU init -> fmv_app::FmvPlayback (PSS via fmv_pss)" },
];

/// FUN_001fcea0's EE-side sound globals (0x3dd1ec.., 0x3dd1fc, 0x3dd200.., 0x3dd230/0x3dd238).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SoundGlobals {
    /// DAT_003dd1fc master
    pub master: f32,
    /// 4 x {8 group volumes (stride 0x10 backwards from 0x3dd1ec), 1 bus volume 0x3dd200}
    pub groups: [[f32; 8]; 4],
    pub bus: [f32; 4],
    /// DAT_003dd230 / DAT_003dd238 = FUN_0014d240(clock) (tick stamp)
    pub tick: u64,
}

impl Default for SoundGlobals {
    fn default() -> Self {
        Self { master: 0.0, groups: [[0.0; 8]; 4], bus: [0.0; 4], tick: 0 }
    }
}

/// FUN_001fcea0 (EE side; the IOP init FUN_002319e8/FUN_00233048 'SUKR' is replaced by bevy_audio).
pub fn ee_sub_1fcea0(g: &mut SoundGlobals, tick: u64) {
    // 0x001fcea0
    g.master = 1.0;
    for i in 0..4 {
        g.bus[i] = 1.0;
        for v in g.groups[i].iter_mut() {
            *v = 1.0;
        }
    }
    g.tick = tick;
}

/// Engine services the startup table needs.
pub trait StartupHost {
    /// FUN_001c8d40(bar, n): loading bar shows `n` remaining steps.
    fn loading_bar(&mut self, _remaining: u32) {}
    /// FUN_00187030: pad open (port 0 slot 0) + input lock reset.
    fn init_pad(&mut self) {}
    /// FUN_001fcea0 tail: hand the sound globals to the audio backend.
    fn init_sound(&mut self, _g: &SoundGlobals) {}
    /// A replaced entry (for logging).
    fn replaced(&mut self, _f: &InitFn) {}
}

/// Results of the startup sequence.
#[derive(Clone, Debug, Default)]
pub struct Startup {
    pub sound: SoundGlobals,
}

/// FUN_001a7970. `now` = real seconds (Count register in the original).
pub fn ee_str_0813_0032_1a7970(clk: &mut Clock, now: f64, host: &mut impl StartupHost) -> Startup {
    // 0x001a7970
    let mut out = Startup::default();
    let mut bar = false;
    for (i, f) in STARTUP_TABLE.iter().enumerate() {
        clock::ee_sub_14d0b8(clk, now, true);
        if bar {
            host.loading_bar(STARTUP_TABLE.len() as u32 - i as u32);
        } else {
            // the bar starts with the entry AFTER FUN_001700d0 (flag set when the pointer matches)
            bar = f.addr == 0x001700d0;
        }
        match f.addr {
            0x0014cfe0 => clock::ee_sub_14cfe0(clk, now),
            0x00187030 => host.init_pad(),
            0x001fcea0 => {
                ee_sub_1fcea0(&mut out.sound, (now * 1e6) as u64);
                host.init_sound(&out.sound);
            }
            _ => host.replaced(f),
        }
    }
    host.loading_bar(0);
    out
}
