//! main() = FUN_001a7518 (called from entry 0x00100008 with argc/argv) and its per-frame order.
//! The steps are the original callees in the original order; the Bevy app (src/bin/game.rs) maps each
//! step to a system set chained in this order, and tests drive `run_frame` with a recording host.

/// One call of the main loop body, in FUN_001a7518 order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MainStep {
    /// FUN_0020e890(0x3dd490): pending level transition (no-op unless world+0x3bf0 is set)
    LevelTransition,
    /// FUN_001871c0(0x2dfc00): pad read (WP-H)
    PadRead,
    /// FUN_0014ba38: disc/boot-file state machine (DAT_002bd4c0; CRC32 over 0x800 bytes at 0x2dd680)
    DiscCheck,
    /// FUN_0014d470(0x2ddf40): camera (g_cm) pre-update (WP-I)
    CameraPre,
    /// FUN_0014d0b8(0x2dded0, 0): clock advance
    Clock,
    /// FUN_00226550(0x3dd490) "C2dc018_v6": world update (objects, scripts (FUN_00128148 = Splice), sound
    /// (FUN_001fd830(0x3dd040)), camera FUN_0014d478 when dt_real > 0). When dt <= 0 (paused) it runs
    /// FUN_002267c0 instead of the object update.
    WorldUpdate,
    /// FUN_001d9648(0x3d9e70): UI/screen update
    UiUpdate,
    /// FUN_00160828: begin frame (GS double buffer, WaitSema)
    FrameBegin,
    /// FUN_001cda80(0x3dd490) "C2dc018_v7": render world
    RenderWorld,
    /// FUN_00157fa0(0x3dd490) "C2dc018_v8": render pass 2
    RenderPost,
    /// FUN_001d9820(0x3d9e70): UI draw
    UiDraw,
    /// FUN_001477d8(DAT_002dd644): overlay (load/memory-card screen) draw
    Overlay,
    /// FUN_00160a98: end frame (SignalSema, DMA kick)
    FrameEnd,
}

/// Engine globals the main loop reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MainState {
    /// DAT_002be538: frame counter
    pub frame: u32,
    /// DAT_002dd644 -> +0x150: overlay state (None = no overlay object)
    pub overlay_state: Option<i32>,
}

/// FUN_001477b0: overlay lets the world render unless its state (+0x150) is 2 or 3.
pub fn ee_sub_1477b0(state: i32) -> bool {
    // 0x001477b0
    3 < state || state < 2
}

/// Startup FUN_001a7970: 0x13 init functions from the table at 0x2bfce0; once FUN_001700d0 has run, a
/// loading bar FUN_001c8d40(bar, 0x13 - i) is shown. Between each, FUN_0014d0b8(clock, 1) (real time only).
pub const STARTUP_FUNCS: u32 = 0x13;

pub trait MainLoopHost {
    fn step(&mut self, s: MainStep);
}

/// One iteration of the FUN_001a7518 do/while(true) loop.
pub fn ee_sub_1a7518_frame(host: &mut impl MainLoopHost, st: &mut MainState) {
    // 0x001a7518
    use MainStep::*;
    for s in [LevelTransition, PadRead, DiscCheck, CameraPre, Clock, WorldUpdate, UiUpdate, FrameBegin] {
        host.step(s);
    }
    if st.overlay_state.is_none_or(ee_sub_1477b0) {
        host.step(RenderWorld);
        host.step(RenderPost);
        host.step(UiDraw);
    }
    if st.overlay_state.is_some() {
        host.step(Overlay);
    }
    host.step(FrameEnd);
    st.frame = st.frame.wrapping_add(1);
}

/// Recording host (tests / debugging).
#[derive(Default, Debug)]
pub struct Recorder(pub Vec<MainStep>);
impl MainLoopHost for Recorder {
    fn step(&mut self, s: MainStep) {
        self.0.push(s);
    }
}
