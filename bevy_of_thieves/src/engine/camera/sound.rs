//! Camera-side sound calls (the look/zoom policies' motor loops 0x25/0x26 and the follow camera).
//! Ported 1:1 as plain calls onto `audio::emitter::EmitterSystem` (audio/ is not edited): the emitter
//! fields the camera writes that `SoundEmitter` does not model (+0x18 pitch, +0x48 stop fade) are kept in
//! the `CmEmitter` handle next to the emitter id.
#![allow(non_snake_case)]

use crate::engine::audio::emitter::{EmitterId, EmitterSystem};

/// A policy's emitter handle slot (the CRef at e.g. policy +0x34; FUN_00258040 assigns it).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CmEmitter {
    pub id: Option<EmitterId>,
    /// emitter +0x18 (pitch, FUN_001fb2c0); pool memory is zero-filled.
    pub pitch: f32,
    /// emitter +0x48 (stop fade, FUN_001fb098).
    pub fade: f32,
}

/// FUN_001fbb90(sound) + FUN_001fb2f8(e, flag) + FUN_00258040(&slot, e).
pub fn new_emitter(es: &mut EmitterSystem, slot: &mut CmEmitter, sound: i32, flag: i32) {
    let id = es.new_emitter_1fbb90(sound);
    es.set_unpositioned_1fb2f8(id, flag);
    FUN_00258040(es, slot, Some(id));
}

/// 0x00258040: assign a CRef (add-ref the new one (vtable +0xc), release the old one (vtable +0x14)).
pub fn FUN_00258040(es: &mut EmitterSystem, slot: &mut CmEmitter, id: Option<EmitterId>) {
    if let Some(n) = id {
        es.add_ref(n);
    }
    if let Some(o) = slot.id {
        es.release(o);
    }
    *slot = CmEmitter { id, pitch: 0.0, fade: 0.0 };
}

/// emitter +0x4e (play count).
pub fn playing(es: &EmitterSystem, slot: &CmEmitter) -> i16 {
    slot.id.and_then(|i| es.get(i)).map(|e| e.s4e).unwrap_or(0)
}

/// 0x001fb018: ++play count; on 0 -> 1 set flags (6, 2).
pub fn FUN_001fb018(es: &mut EmitterSystem, slot: &CmEmitter) {
    let Some(id) = slot.id else { return };
    let Some(e) = es.get_mut(id) else { return };
    e.s4e += 1;
    if e.s4e == 1 {
        es.set_flags_1fb658(id, 6, 2);
    }
}

/// 0x001fb060: --play count; on 1 -> 0 set flags (6, 4).
pub fn FUN_001fb060(es: &mut EmitterSystem, slot: &CmEmitter) {
    let Some(id) = slot.id else { return };
    let Some(e) = es.get_mut(id) else { return };
    let old = e.s4e;
    e.s4e = old - 1;
    if old == 1 {
        es.set_flags_1fb658(id, 6, 4);
    }
}

/// 0x001fb098: +0x48 = fade, set flags (0x1fff, 8) (stop).
pub fn FUN_001fb098(es: &mut EmitterSystem, fade: f32, slot: &mut CmEmitter) {
    let Some(id) = slot.id else { return };
    slot.fade = fade;
    es.set_flags_1fb658(id, 0x1fff, 8);
}

/// 0x001fb1f8: volume (+0x10) = clamp(v, [0, 1]); flags 0x10 when the 1/1024 step changed.
pub fn FUN_001fb1f8(es: &mut EmitterSystem, v: f32, slot: &CmEmitter) {
    let Some(id) = slot.id else { return };
    let Some(e) = es.get_mut(id) else { return };
    let v = super::math::FUN_0011d948(v, [0.0, 1.0]);
    if (v * 1024.0) as i32 != (e.f10 * 1024.0) as i32 {
        e.f10 = v;
        es.set_flags_1fb658(id, 0x10, 0x10);
    }
}

/// 0x001fb2c0: pitch (+0x18); flags 0x40 when changed.
pub fn FUN_001fb2c0(es: &mut EmitterSystem, p: f32, slot: &mut CmEmitter) {
    let Some(id) = slot.id else { return };
    if p != slot.pitch {
        slot.pitch = p;
        es.set_flags_1fb658(id, 0x40, 0x40);
    }
}
