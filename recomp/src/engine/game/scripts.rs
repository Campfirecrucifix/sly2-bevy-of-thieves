//! WP-J3: running the level's Splice script blocks inside the game. Each directory entry's FUN_001ff858
//! block (right after the s32 blob size / blob key / ext flag of FUN_0020f5d0) is evaluated
//! expression by expression in the world frame (FUN_001ffa70) with the game host.

use super::splice_host::LevelSpliceHost;
use crate::engine::splice::eval::Vm;
use crate::engine::splice::frame::{new_frame, FrameRef};
use crate::engine::splice::serialize::read_script_block;
use crate::engine::splice::SpliceError;
use crate::engine::world::World;

/// Outcome of one level's script run (errors are reported, never swallowed).
#[derive(Debug, Default, Clone)]
pub struct ScriptRunReport {
    pub entries: usize,
    pub exprs: usize,
    pub ok: usize,
    /// (entry index, expression index, error)
    pub errors: Vec<(usize, usize, SpliceError)>,
}

impl ScriptRunReport {
    pub fn host_unimplemented(&self) -> impl Iterator<Item = &str> {
        self.errors.iter().filter_map(|(_, _, e)| match e {
            SpliceError::HostUnimplemented(s) => Some(s.as_str()),
            _ => None,
        })
    }
}

/// FUN_001ffa70 over every entry of a loaded level. An expression that errors is reported and the
/// next one still runs (the original VM has no error unwinding: a failing BIF yields Nil and continues).
pub fn run_level_scripts(vm: &mut Vm, level: &[u8], w: &World, host: &mut LevelSpliceHost) -> ScriptRunReport {
    run_level_scripts_with(vm, level, w, host)
}

/// `run_level_scripts` with any host (WP-J4 `vm_state::PersistentHost`). WP-J10: each expression runs
/// in its own slot frame (see [`run_level_scripts_framed`]); the frames are dropped here.
pub fn run_level_scripts_with(vm: &mut Vm, level: &[u8], w: &World, host: &mut dyn crate::engine::splice::SpliceHost) -> ScriptRunReport {
    run_level_scripts_framed(vm, level, w, host).0
}

/// FUN_001ff858 per directory entry: the block's n expressions are the entry's anchor list
/// (world+0x39e8: {+0 entry, +4 n, +8 slots {expr, frame}}); FUN_001ffa70 evaluates slot i in a NEW frame
/// (FUN_00256dd0: no parent, so a slot's `define`s stay in that slot) and keeps the frame in the slot.
/// Objects get the frame of their slot as script frame +0 (FUN_0013e3c8 -> FUN_001ffa40(parent slot))
/// and their messages look handlers up there (FUN_001331f8). Returns the frames per entry, per slot.
pub fn run_level_scripts_framed(vm: &mut Vm, level: &[u8], w: &World, host: &mut dyn crate::engine::splice::SpliceHost) -> (ScriptRunReport, Vec<Vec<FrameRef>>) {
    // 0x001ff858 / 0x001ffa70
    let mut r = ScriptRunReport::default();
    let mut frames = Vec::with_capacity(w.entries.len());
    for (ei, e) in w.entries.iter().enumerate() {
        r.entries += 1;
        let mut slots = Vec::new();
        let off = e.start_offset as usize + 12;
        let blk = match level.get(off..).map(|d| read_script_block(d, e.sections.ext)) {
            Some(Ok((b, _))) => b,
            Some(Err(err)) => {
                r.errors.push((ei, 0, SpliceError::Read(err.to_string())));
                Vec::new()
            }
            None => Vec::new(),
        };
        for (xi, x) in blk.iter().enumerate() {
            r.exprs += 1;
            let fr = new_frame(None);
            let res = match x {
                Some(p) => {
                    let car = p.borrow().car.clone();
                    vm.eval(host, &car, &fr).map(|_| ())
                }
                None => Ok(()),
            };
            match res {
                Ok(()) => r.ok += 1,
                Err(err) => r.errors.push((ei, xi, err)),
            }
            slots.push(fr);
        }
        frames.push(slots);
    }
    (r, frames)
}
