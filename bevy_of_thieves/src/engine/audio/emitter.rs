//! WP-G2: game-side code behind Splice BIFs 129 "StartSound" (RefOpStartSound 0x001251e8) and
//! 131 "StopSound" (RefOpStopSound 0x00125398). Names come from Sly 1; the Sly 2 code shows:
//!  - BIF 129 allocates a 0x70-byte sound-emitter object (class vtable 0x2d9c70, base ctor C2d9d00
//!    FUN_001fafd0) from the pool 0x3dd05c (FUN_001fbb90), sets its 2D/3D flag (FUN_001fb2f8) and returns it
//!    as a Basic ref (tag 0x21, FUN_001294c0 adds a reference).
//!  - BIF 131 turns its list argument into a u16 path (FUN_001252d0) and reads a value from the option tree
//!    0x3e0ea8 / struct 0x3d4a60 (FUN_001d7ed8): it is a path lookup, not a sound stop.
//! 1:1 ports (`// 0x...` = original). Pool / intrusive list (FUN_0011ccc0, FUN_0011a220/2e8/380) are replaced
//! by a Vec slab + Vec active list with the same observable order and reference counting.

use bevy_ecs::prelude::Resource;

/// DAT_002a2dd0: the zero vector the "unpositioned" test compares against.
pub const ZERO_VEC: [f32; 3] = [0.0; 3];

/// FUN_0011d4d0: relative float compare |a-b| / (|a| >= 1 ? |a| : 1) < eps.
pub fn fuzzy_eq(a: f32, b: f32, eps: f32) -> bool {
    let fa = a.abs();
    (a - b).abs() / if fa < 1.0 { 1.0 } else { fa } < eps
}

/// 0x38d1b717
pub const EPS: f32 = 1.0e-4;

/// Sound emitter (class vtable 0x2d9c70, 0x70 bytes). Field offsets in comments.
#[derive(Clone, Debug, PartialEq)]
pub struct SoundEmitter {
    pub f10: f32,           // +0x10 = 1.0 (volume)
    /// +0x14 pan in degrees 0..359 (FUN_001fb268); WP-G3
    pub pan: i32,
    /// +0x18 pitch (FUN_001fb2c0); WP-G3
    pub pitch: f32,
    /// +0x1c BIF arg 2. FUN_001fb528 reads it as the attach object (nonzero = position relative to that object)
    pub unpositioned: i32,
    pub pos: [f32; 3],      // +0x20
    pub near: f32,          // +0x30 = 300.0
    pub far: f32,           // +0x34 = 3000.0
    /// +0x40 989snd voice handle (FUN_001fbaf8 stores it, FUN_001fbe80 stops it; 0 = none)
    pub obj40: u32,
    pub f44: f32,           // +0x44 = 1.0 effective volume (+0x10 x distance falloff)
    /// +0x48 stop fade in seconds (FUN_001fb098); WP-G3
    pub fade: f32,
    pub refs: i16,          // +0x4c reference count (vtable slot 0 FUN_001fc090 ++, slot 1 FUN_001fc0a0 --)
    pub s4e: i16,           // +0x4e pause count (FUN_001fb018 ++ / FUN_001fb060 --)
    pub flags: u16,         // +0x50 activity bits (1 = created, 0x200 = audible/positioned, 0x1f1/0x1f3 = FUN_001fb0c0)
    pub s52: i16,           // +0x52
    pub sound: i32,         // +0x60 bank sound index
    /// +0x68 play time in EE bus cycles (music / dialogue instances; read by FUN_001fe3f8); WP-X2
    pub cycles: u64,
}

impl SoundEmitter {
    /// FUN_001fafd0 (C2d9d00 base ctor) + FUN_001fbbc8 (C2d9c70 ctor) field init on zeroed pool memory.
    fn new(sound: i32) -> Self {
        Self { f10: 1.0, pan: 0, pitch: 0.0, unpositioned: 0, pos: ZERO_VEC, near: 300.0, far: 3000.0, obj40: 0, f44: 1.0, fade: 0.0, refs: 0, s4e: 0, flags: 0, s52: 0, sound, cycles: 0 }
    }

    /// The "is the position the zero vector" test shared by FUN_001fb2f8 and FUN_001fb0c0 (x, then y, then z).
    pub fn at_origin(&self) -> bool {
        fuzzy_eq(ZERO_VEC[0], self.pos[0], EPS) && fuzzy_eq(ZERO_VEC[1], self.pos[1], EPS) && fuzzy_eq(ZERO_VEC[2], self.pos[2], EPS)
    }
}

pub type EmitterId = usize;

/// Emitter pool 0x3dd05c + active list 0x3dd08c.
#[derive(Resource, Default, Debug)]
pub struct EmitterSystem {
    pub slots: Vec<Option<SoundEmitter>>,
    /// active list 0x3dd08c in insertion order (FUN_0011a220 appends at the tail)
    pub active: Vec<EmitterId>,
    /// emitters whose last reference was dropped: FUN_00258348(this, 0) (stop) then FUN_001fc110 (free)
    pub freed: Vec<(EmitterId, i32)>,
}

impl EmitterSystem {
    pub fn get(&self, id: EmitterId) -> Option<&SoundEmitter> {
        self.slots.get(id)?.as_ref()
    }
    pub fn get_mut(&mut self, id: EmitterId) -> Option<&mut SoundEmitter> {
        self.slots.get_mut(id)?.as_mut()
    }

    /// FUN_001fc0f0: pool alloc (FUN_0011ccc0 zero-fills); reuses the most recently freed slot first like the
    /// pool's free list.
    fn alloc(&mut self, e: SoundEmitter) -> EmitterId {
        if let Some(i) = self.slots.iter().rposition(|s| s.is_none()) {
            self.slots[i] = Some(e);
            i
        } else {
            self.slots.push(Some(e));
            self.slots.len() - 1
        }
    }

    /// AUD3: a music / ambience instance of 0x3dd040 (+0x204 stack, +0x228): pool alloc with one reference,
    /// not on the active list (the stream voice is the host's layer players; FUN_001fe150 REPLACED).
    pub fn alloc_instance(&mut self, sound: i32) -> EmitterId {
        let id = self.alloc(SoundEmitter::new(sound));
        self.add_ref(id);
        id
    }

    /// vtable slot 0 (FUN_001fc090): add reference
    pub fn add_ref(&mut self, id: EmitterId) {
        if let Some(e) = self.get_mut(id) {
            e.refs += 1;
        }
    }

    /// vtable slot 1 (FUN_001fc0a0): drop reference; at 0 -> FUN_00258348(this,0) + FUN_001fc110 (free).
    pub fn release(&mut self, id: EmitterId) {
        let Some(e) = self.get_mut(id) else { return };
        let old = e.refs;
        e.refs -= 1;
        if old == 1 {
            let s = e.sound;
            self.slots[id] = None;
            self.active.retain(|&a| a != id);
            self.freed.push((id, s));
        }
    }

    /// FUN_001fb658(this, mask, value): flags = flags & ~mask | value & mask; 0 -> leave the active list
    /// (FUN_001fb710: if listed, unlink + slot 1), else join it (FUN_001fb6a8: if not listed, slot 0 + append).
    pub fn set_flags_1fb658(&mut self, id: EmitterId, mask: u16, value: u16) {
        let Some(e) = self.get_mut(id) else { return };
        e.flags = e.flags & !mask | value & mask;
        let empty = e.flags == 0;
        let listed = self.active.contains(&id);
        if empty {
            if listed {
                self.active.retain(|&a| a != id);
                self.release(id);
            }
        } else if !listed {
            self.add_ref(id);
            self.active.push(id);
        }
    }

    /// FUN_001fbb90: new emitter for `sound` (0x001fbb90 -> FUN_001fc0f0 alloc 0x70 + FUN_001fbbc8 ctor,
    /// which ends with FUN_001fb658(this, 1, 1)).
    pub fn new_emitter_1fbb90(&mut self, sound: i32) -> EmitterId {
        let id = self.alloc(SoundEmitter::new(sound));
        self.set_flags_1fb658(id, 1, 1);
        id
    }

    /// FUN_001fb2f8: store the 2D flag (+0x1c). Nonzero -> set bit 0x200. Zero -> clear 0x200 only when the
    /// position is still the zero vector (quirk: a positioned emitter keeps whatever 0x200 state it had).
    pub fn set_unpositioned_1fb2f8(&mut self, id: EmitterId, v: i32) {
        let Some(e) = self.get_mut(id) else { return };
        e.unpositioned = v;
        if v == 0 {
            if e.at_origin() {
                self.set_flags_1fb658(id, 0x200, 0);
            }
        } else {
            self.set_flags_1fb658(id, 0x200, 0x200);
        }
    }

    /// FUN_001fb0c0 (vtable 0x2d9c70 slot 6): re-evaluate activity. Proceeds only when +0x40 == 0, +0x52 == 0
    /// and bit 1 is clear (i.e. an idle stand-alone emitter); sets 0x1f1 (499 when +0x4e > 0) plus 0x200
    /// unless the emitter is positional-at-origin.
    pub fn refresh_1fb0c0(&mut self, id: EmitterId) {
        let Some(e) = self.get(id) else { return };
        let run = e.obj40 == 0 && e.s52 == 0 && e.flags & 1 == 0;
        if !run {
            return;
        }
        let mut f: u16 = if e.s4e > 0 { 499 } else { 0x1f1 };
        if !(e.unpositioned == 0 && e.at_origin()) {
            f |= 0x200;
        }
        self.set_flags_1fb658(id, f, f);
    }

    /// RefOpStartSound 0x001251e8: argv = [S32 sound, S32 flag] payloads. Returns the emitter (Basic ref,
    /// FUN_001294c0 adds a reference for the returned CRef; the caller's CRef drop releases it).
    pub fn bif_start_sound_1251e8(&mut self, sound: i32, flag: i32) -> EmitterId {
        let id = self.new_emitter_1fbb90(sound);
        self.set_unpositioned_1fb2f8(id, flag);
        self.add_ref(id); // 0x001294c0
        id
    }
}

/// Splice value tags read by FUN_001252d0.
pub const TAG_S32: i32 = 0x15;
pub const TAG_SYMID: i32 = 0x18;

/// FUN_001252d0: list -> {count, u16 path[count]}. `list` = (tag, payload) of each element in order.
/// Elements that are neither S32 nor Symid keep their slot (index still advances) but are not written:
/// the original buffer comes from the frame stack FUN_001ae318 and is left uninitialised; 0 here.
pub fn list_to_path_1252d0(list: &[(i32, u32)]) -> Vec<u16> {
    let mut v = vec![0u16; list.len()];
    for (i, &(tag, payload)) in list.iter().enumerate() {
        if tag == TAG_S32 || tag == TAG_SYMID {
            v[i] = payload as u16;
        }
    }
    v
}
