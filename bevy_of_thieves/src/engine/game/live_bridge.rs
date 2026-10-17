//! WP-TR1: bridge from script method dispatch (`PersistentHost::call_method`) to the QMem-based
//! callee implementations in `classes::q::live_dispatch::TABLE`.
//!
//! Method thunks unpack args via `FUN_00201360` then call an engine function on the object pointer;
//! those engine functions are ported as `fn(&mut dyn QMem, ...)` in the `q` modules. This bridge
//! resolves the object's guest address in the LiveQ store and dispatches through the generated table.

use crate::engine::classes::q::live::{LiveMem, LiveQ};
use crate::engine::classes::q::live_dispatch;
use crate::engine::classes::q::q2mem::V4;
use crate::engine::splice::value::Ref;
use super::methods::{GetterRet, MArg};

/// Convert `MArg` arguments into the `(u32_args, v4_args)` layout that `live_dispatch::QFn` expects.
/// Integer / word / payload kinds -> u32; float -> u32 (bits); vector -> V4; in source order.
fn args_to_qmem(args: &[MArg]) -> (Vec<u32>, Vec<V4>) {
    let mut a = Vec::new();
    let mut v = Vec::new();
    for arg in args {
        match arg {
            MArg::I32(n) => a.push(*n as u32),
            MArg::Word(w) => a.push(*w),
            MArg::F32(f) => a.push(f.to_bits()),
            MArg::Ref(r) | MArg::Value(r) => match r {
                Ref::Vector(vec) => v.push([vec[0], vec[1], vec[2], 0.0]),
                _ => a.push(r.payload()),
            },
            MArg::None => a.push(0),
        }
    }
    (a, v)
}

/// Try to dispatch `callee` through the live_dispatch TABLE on object `obj_handle` (Basic payload).
/// Returns `Some(Ok(Ref))` on successful dispatch, `Some(Err)` on address resolution failure,
/// `None` when the callee is not in the TABLE.
pub fn try_method(
    live_q: &mut LiveQ,
    callee: u32,
    obj_handle: u32,
    args: &[MArg],
) -> Option<Result<Ref, ()>> {
    // Look up the callee in the generated TABLE (binary search by address)
    let idx = live_dispatch::TABLE.binary_search_by_key(&callee, |e| e.0).ok()?;
    let qfn = live_dispatch::TABLE[idx].1;

    let mem = live_q.mem.as_mut()?;

    // Resolve object address: obj_handle is a Basic payload (object index for most objects,
    // or a global address for SW/world globals). Handle = 0 means no object.
    let obj_addr = if obj_handle == 0 {
        0
    } else {
        // Try as object index first (handle - 1 for class instances, or raw for globals)
        let i = (obj_handle as usize).wrapping_sub(1);
        if i < mem.obj_addr.len() {
            mem.obj_addr[i]
        } else {
            // Might be a global address (SW, world global, etc.)
            obj_handle
        }
    };

    let (mut a_int, a_vec) = args_to_qmem(args);
    // The original thunk passes param_2 (object pointer) as the first argument to the callee.
    // The QMem port functions typically take (m, obj_addr, ...args...).
    // Prepend the object address.
    a_int.insert(0, obj_addr);

    let (r_int, _r_vec) = qfn(mem, &a_int, &a_vec);

    // Method thunks return Void (FUN_001291a8 copies the frame's return CRef, which is usually Void
    // for methods). If the callee produces a meaningful u32, the getter path handles it separately.
    let _ = r_int;
    Some(Ok(Ref::Void))
}

/// Try to dispatch a getter callee through the live_dispatch TABLE.
/// Returns the getter's return value wrapped according to `ret_kind`.
pub fn try_getter(
    live_q: &mut LiveQ,
    callee: u32,
    obj_handle: u32,
    args: &[MArg],
    ret: GetterRet,
) -> Option<Result<Ref, ()>> {
    let idx = live_dispatch::TABLE.binary_search_by_key(&callee, |e| e.0).ok()?;
    let qfn = live_dispatch::TABLE[idx].1;

    let mem = live_q.mem.as_mut()?;

    let obj_addr = if obj_handle == 0 {
        0
    } else {
        let i = (obj_handle as usize).wrapping_sub(1);
        if i < mem.obj_addr.len() {
            mem.obj_addr[i]
        } else {
            obj_handle
        }
    };

    let (mut a_int, a_vec) = args_to_qmem(args);
    a_int.insert(0, obj_addr);

    let (r_int, r_vec) = qfn(mem, &a_int, &a_vec);

    let ref_val = match ret {
        GetterRet::Bool => Ref::Bool(r_int != 0),
        GetterRet::S32 => Ref::S32(r_int as i32),
        GetterRet::F32 => Ref::F32(f32::from_bits(r_int)),
        GetterRet::Basic => Ref::Basic(r_int),
        GetterRet::Vector => Ref::Vector(r_vec[..3].try_into().unwrap_or([0.0; 3])),
        GetterRet::Matrix | GetterRet::Clq | GetterRet::Lm => Ref::Void, // rare types — not bridged
        GetterRet::F32x2 => Ref::Vector([f32::from_bits(r_int), r_vec[0], r_vec[1]]),
        GetterRet::Ref | GetterRet::Unknown => Ref::Void,
    };
    Some(Ok(ref_val))
}
