//! WP-Q2: object-memory view for the breadth-first queue ports (classes/q/*, engine/*/q_*.rs).
//!
//! The queue functions are engine plumbing that walks raw object layouts (ALO child lists +0x64 / +0x24,
//! vtables +0xc, DLists, navmesh tables). They are ported 1:1 against this trait: field reads / writes by
//! address, virtual slots through the vtable (`vslot`), and every callee that is not part of the same
//! package through `QMem::call` (by original address), so the host routes it to the owning Rust port
//! (or records it). `FlatMem` is the reference host used by the tests (byte memory + call log).
//! Calling convention notes: integer / pointer arguments go in `a` (a0..), float / quadword (VU0 u128)
//! arguments in `v` in source order; the result is (v0, quadword / f0 result).
use std::collections::HashMap;

pub type V4 = [f32; 4];

pub trait QMem {
    fn r8(&mut self, a: u32) -> u8;
    fn w8(&mut self, a: u32, v: u8);
    /// engine function `addr` (not in the calling package / not ported here)
    fn call(&mut self, addr: u32, a: &[u32], v: &[V4]) -> (u32, V4);
    /// call with full 64-bit integer args (GS packet helpers 0x166e90/0x166ef0/0x166fe8/0x167048,
    /// whose register/GIFtag args are u64). Default records through `call` with the low 32 bits
    /// (unit-test / verifier hosts stub callees); the live host overrides it to keep all 64 bits.
    fn call64(&mut self, addr: u32, a: &[u64]) {
        let lo: Vec<u32> = a.iter().map(|&x| x as u32).collect();
        self.call(addr, &lo, &[]);
    }
    /// PAD1: record a FUN_001619a0(addr, event, arg) call for host drain (default: no-op)
    fn record_1619a0(&mut self, _addr: u32, _ev: i32, _arg: i32) {}
    /// PAD1: record a FUN_00164640(handle, state) call for host drain (default: no-op)
    fn record_164640(&mut self, _handle: u32, _state: i32) {}
    /// PAD1: record a gadget-purchase save-variable write for host drain (default: no-op).
    /// entry_id = the gadget entry, purchased = true/false.
    fn record_gadget_purchase(&mut self, _entry_id: u32, _purchased: bool) {}

    /// VFIX3: stack_pop zeroes released bytes (WIRE-Q2) unless false; FlatMem (unit-test mock) keeps them so
    /// tests can inspect the arg buffers a port passed to a recorded callee
    fn scrub_popped_stack(&self) -> bool { true }
    fn r16(&mut self, a: u32) -> u16 {
        u16::from_le_bytes([self.r8(a), self.r8(a + 1)])
    }
    fn r32(&mut self, a: u32) -> u32 {
        u32::from_le_bytes([self.r8(a), self.r8(a + 1), self.r8(a + 2), self.r8(a + 3)])
    }
    fn w32(&mut self, a: u32, v: u32) {
        for (i, b) in v.to_le_bytes().into_iter().enumerate() {
            self.w8(a + i as u32, b);
        }
    }
    fn w16(&mut self, a: u32, v: u16) {
        for (i, b) in v.to_le_bytes().into_iter().enumerate() {
            self.w8(a + i as u32, b);
        }
    }
    fn r64(&mut self, a: u32) -> u64 {
        self.r32(a) as u64 | (self.r32(a.wrapping_add(4)) as u64) << 32
    }
    fn rf(&mut self, a: u32) -> f32 {
        f32::from_bits(self.r32(a))
    }
    fn wf(&mut self, a: u32, v: f32) {
        self.w32(a, v.to_bits())
    }
    fn rv(&mut self, a: u32) -> V4 {
        [self.rf(a), self.rf(a.wrapping_add(4)), self.rf(a.wrapping_add(8)), self.rf(a.wrapping_add(12))]
    }
    fn wv(&mut self, a: u32, v: V4) {
        for (i, x) in v.into_iter().enumerate() {
            self.wf(a.wrapping_add(4 * i as u32), x);
        }
    }
    /// 4 quadwords at `a` (VU0 matrix: rows = basis vectors, row 3 = translation)
    fn rm(&mut self, a: u32) -> [V4; 4] {
        [self.rv(a), self.rv(a.wrapping_add(0x10)), self.rv(a.wrapping_add(0x20)), self.rv(a.wrapping_add(0x30))]
    }
    fn wm(&mut self, a: u32, m: [V4; 4]) {
        for (i, r) in m.into_iter().enumerate() {
            self.wv(a + 0x10 * i as u32, r);
        }
    }
}

/// word holding the emulated stack pointer for locals whose address escapes (FUN_001ae478 frames, BSP
/// traversal stacks, out-buffers); outside the 32 MB EE RAM so it never aliases game data.
pub const SCRATCH_SP: u32 = 0x0400_0000;
const SCRATCH_TOP: u32 = 0x03ff_fff0;

/// reserve `n` bytes (16-aligned) of emulated stack; returns the address
pub fn stack_push<M: QMem + ?Sized>(m: &mut M, n: u32) -> u32 {
    let sp = match m.r32(SCRATCH_SP) {
        0 => SCRATCH_TOP,
        s => s,
    };
    let a = sp - ((n + 15) & !15);
    m.w32(SCRATCH_SP, a);
    a
}

/// release a `stack_push(n)` reservation (LIFO)
/// WIRE-Q2: the released bytes are zeroed, so a later frame's uninitialised local (e.g. FUN_00210b28's out
/// word when the save lookup is not modelled) reads 0 (fresh game) instead of a previous callee's pointer
pub fn stack_pop<M: QMem + ?Sized>(m: &mut M, n: u32) {
    let sp = m.r32(SCRATCH_SP);
    let n = (n + 15) & !15;
    if m.scrub_popped_stack() {
        for k in (0..n).step_by(4) {
            m.w32(sp + k, 0);
        }
    }
    m.w32(SCRATCH_SP, sp + n);
}

/// vtable word offset of virtual slot `slot` (the DB's class_slots numbering): fn at +8n+0xc,
/// this-adjust (s16) at +8n+8.
pub const fn slot_off(slot: u32) -> u32 {
    8 * slot + 0xc
}

/// `(**(code **)(*(obj + 0xc) + 8n + 0xc))(obj + *(short *)(*(obj + 0xc) + 8n + 8), ..)`
pub fn vslot<M: QMem + ?Sized>(m: &mut M, obj: u32, slot: u32, a: &[u32], v: &[V4]) -> (u32, V4) {
    let vt = m.r32(obj.wrapping_add(0xc));
    let f = m.r32(vt.wrapping_add(slot_off(slot)));
    let adj = m.r16(vt.wrapping_add(slot_off(slot)).wrapping_sub(4)) as i16 as i32;
    let mut args = vec![obj.wrapping_add(adj as u32)];
    args.extend_from_slice(a);
    m.call(f, &args, v)
}

/// Reference host: byte memory (unset = 0), a call log and canned results per callee address. Calls to
/// addresses `route` knows run the Rust port instead (so ported callers reach ported callees).
#[derive(Default)]
pub struct FlatMem {
    pub mem: HashMap<u32, u8>,
    pub calls: Vec<(u32, Vec<u32>, Vec<V4>)>,
    pub ret: HashMap<u32, (u32, V4)>,
    pub route: Option<fn(&mut FlatMem, u32, &[u32], &[V4]) -> Option<(u32, V4)>>,
}

impl QMem for FlatMem {
    fn scrub_popped_stack(&self) -> bool { false }
    fn r8(&mut self, a: u32) -> u8 {
        self.mem.get(&a).copied().unwrap_or(0)
    }
    fn w8(&mut self, a: u32, v: u8) {
        self.mem.insert(a, v);
    }
    fn call(&mut self, addr: u32, a: &[u32], v: &[V4]) -> (u32, V4) {
        self.calls.push((addr, a.to_vec(), v.to_vec()));
        if let Some(r) = self.route {
            if let Some(x) = r(self, addr, a, v) {
                return x;
            }
        }
        self.ret.get(&addr).copied().unwrap_or((0, [0.0; 4]))
    }
}

impl FlatMem {
    /// addresses called, in order
    pub fn called(&self) -> Vec<u32> {
        self.calls.iter().map(|c| c.0).collect()
    }
    /// install a vtable at `vt` for `obj` with `slots` (slot, fn) and zero this-adjusts
    pub fn vtable(&mut self, obj: u32, vt: u32, slots: &[(u32, u32)]) {
        self.w32(obj + 0xc, vt);
        for &(s, f) in slots {
            self.w32(vt + slot_off(s), f);
        }
    }
}
