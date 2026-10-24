//! PORT2: script method / getter thunks whose body is an inline virtual call on the object
//! (`(**(code **)(*(obj + 0xc) + off))(obj + adj, args..)`, no callee address in the method table).
//! 1:1 from the thunk disassembly: FUN_00201360 unpacks the args, the vtable slot `(off - 0xc) / 8`
//! is called through `q2mem::vslot` on the object's live guest memory (LiveMem routes the slot's
//! function to its Rust port), result Void (FUN_0012a0b0(.., 0x24)) or Bool (FUN_00129598(v0 != 0)).
use super::methods::MArg;
use crate::engine::classes::q::live::{LiveMem, LiveQ};
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem};
use crate::engine::splice::value::Ref;

/// Argument shape of the virtual call after `this`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    /// no args (0x00100a90, 0x00100b88, 0x0010db78)
    None,
    /// a1 = -1 (0x0010daf0)
    M1,
    /// a1 = argv[0] word (int / object)
    W,
    /// a1 = argv[0] word, a2 = &argv[1] (vector on the stack) (0x00104e80)
    WV,
}

/// (thunk, vtable offset of the code word, shape, returns Bool)
pub const ROWS: &[(u32, u32, Shape, bool)] = &[
    (0x0010_0a00, 0x134, Shape::W, false),
    (0x0010_0a90, 0x044, Shape::None, false),
    (0x0010_0b88, 0x04c, Shape::None, false),
    (0x0010_4e80, 0x274, Shape::WV, false),
    (0x0010_bc88, 0x264, Shape::W, true),
    (0x0010_d5d0, 0x19c, Shape::W, false),
    (0x0010_daf0, 0x18c, Shape::M1, false),
    (0x0010_db78, 0x194, Shape::None, false),
    (0x0010_fbf0, 0x294, Shape::W, false),
    (0x0011_0998, 0x2d4, Shape::W, false),
];

pub fn row(thunk: u32) -> Option<(u32, Shape, bool)> {
    ROWS.iter().find(|r| r.0 == thunk).map(|r| (r.1, r.2, r.3))
}

/// The thunk body on any QMem: `obj` = guest address of the object.
pub fn call<M: QMem + ?Sized>(m: &mut M, thunk: u32, obj: u32, args: &[MArg]) -> Option<Ref> {
    let (off, shape, boolean) = row(thunk)?;
    let slot = (off - 0xc) / 8;
    let (ai, av) = super::live_bridge::args_to_qmem(args);
    let w = ai.first().copied().unwrap_or(0);
    let r = match shape {
        Shape::None => vslot(m, obj, slot, &[], &[]).0,
        Shape::M1 => vslot(m, obj, slot, &[u32::MAX], &[]).0,
        Shape::W => vslot(m, obj, slot, &[w], &[]).0,
        Shape::WV => {
            let p = stack_push(m, 0x10);
            m.wv(p, av.first().copied().unwrap_or([0.0; 4]));
            let r = vslot(m, obj, slot, &[w, p], &[]).0;
            stack_pop(m, 0x10);
            r
        }
    };
    Some(if boolean { Ref::Bool(r != 0) } else { Ref::Void })
}

/// Resolve a script object handle to its guest address (as `live_bridge`) and run the thunk.
pub fn try_live(lq: &mut LiveQ, thunk: u32, obj_handle: u32, args: &[MArg]) -> Option<Ref> {
    row(thunk)?;
    let mem: &mut LiveMem = lq.mem.as_mut()?;
    let obj = if obj_handle == 0 { return None } else { super::live_bridge::basic_addr(mem, obj_handle) };
    call(mem, thunk, obj, args)
}
