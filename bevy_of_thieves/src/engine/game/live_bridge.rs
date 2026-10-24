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
pub fn args_to_qmem(args: &[MArg]) -> (Vec<u32>, Vec<V4>) {
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

/// ANIM-ALL R22: the guest address of a script object handle. Object Basics are world object indices
/// (0-based, as `PersistentHost::class_of` / FUN_0015e958 use them; 0 = none); payloads past the object table
/// are global addresses (SW, world globals). It used to take the handle as index + 1, so a bridged method ran
/// on the object before its own (CRAWL R51: the cellar seq's FUN_00142540 on class-54 object 1443 ran on the
/// class-48 object 1442, whose non-brain +0x26c words gave the 0xc2 behaviour block address 8 -> C2cc3c0
/// slot 7 FUN_00145080(8) -> FUN_001a62e8(0) -> vt+0xfc of address 0 = the ALO loader -> budget stop).
pub fn basic_addr(mem: &LiveMem, h: u32) -> u32 {
    match h {
        0 => 0,
        h if (h as usize) < mem.obj_addr.len() => mem.obj_addr[h as usize],
        h => h,
    }
}

/// ANIM-ALL R22: FUN_00201110 passes an object argument's Basic payload (the object pointer on the PS2) as a
/// word; the port's payload is the object index, so the bridged callee gets its guest address. `raw` = the
/// script arguments `args` were converted from (same order).
pub fn basic_words(live_q: &LiveQ, raw: &[Ref], args: &[MArg]) -> Vec<MArg> {
    let Some(mem) = live_q.mem.as_ref().filter(|_| raw.len() == args.len()) else { return args.to_vec() };
    raw.iter()
        .zip(args)
        .map(|(r, a)| match (r, a) {
            (Ref::Basic(b), MArg::Word(w)) if b == w => MArg::Word(basic_addr(mem, *b)),
            _ => a.clone(),
        })
        .collect()
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

    let obj_addr = basic_addr(mem, obj_handle);

    // WIRE-UI: FUN_001d7780 (hub C2d80d0 character list, thunk 0x10f648): the list is a host splice list, not
    // a guest CRef, so FUN_002013d8(kind 6 = int) is done here on the host list (count +0x220, ints +0x224,
    // max 6), then the rest of FUN_001d7780 (FUN_001d77d0(this, 0)) runs on the guest.
    if callee == 0x001d_7780 && obj_addr != 0 {
        let mut vals = Vec::new();
        if let Some(MArg::Ref(Ref::Pair(p)) | MArg::Value(Ref::Pair(p))) = args.first() {
            let mut n = Some(p.clone());
            while let Some(c) = n {
                let b = c.borrow();
                match b.car {
                    Ref::S32(v) => vals.push(v),
                    Ref::F32(f) => vals.push(f as i32),
                    _ => {}
                }
                n = b.next.clone();
            }
        }
        use crate::engine::classes::q::q2mem::QMem;
        let k = vals.len().min(6);
        mem.w32(obj_addr + 0x220, k as u32);
        for (i, v) in vals[..k].iter().enumerate() {
            mem.w32(obj_addr + 0x224 + 4 * i as u32, *v as u32);
        }
        let _ = mem.guarded(|m| crate::engine::classes::q::q8_runs_b::ee_sub_1d77d0(m, obj_addr, 0));
        return Some(Ok(Ref::Void));
    }
    let (mut a_int, a_vec) = args_to_qmem(args);
    // The original thunk passes param_2 (object pointer) as the first argument to the callee.
    // The QMem port functions typically take (m, obj_addr, ...args...).
    // Prepend the object address.
    a_int.insert(0, obj_addr);

    // WIRE-W1: same budget / panic guard as LiveMem::enter (a budget overrun used to unwind out of the host)
    let (r_int, _r_vec) = mem.guarded(|m| qfn(m, &a_int, &a_vec)).unwrap_or((0, [0.0; 4]));

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

    let obj_addr = basic_addr(mem, obj_handle);

    let (mut a_int, a_vec) = args_to_qmem(args);
    a_int.insert(0, obj_addr);

    let (r_int, r_vec) = mem.guarded(|m| qfn(m, &a_int, &a_vec))?;

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
