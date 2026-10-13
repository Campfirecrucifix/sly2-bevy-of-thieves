//! WP-X2: the screen-object movie clips (class 126/130 screens, `world::loads_b4` FUN_00214460 groups):
//! a Flash-like timeline of key frames that place, tween and remove display elements (sprites = nested
//! movie clips, or world objects), with sound cues, per clip state.
//!
//! Layout (the original structs, offsets kept on the fields):
//! sprite 0x30 (+0 u16 n defs, +2 u16 n states, +4 defs, +8 states, +0x10/+0x18/+0x1c/+0x20 last clip,
//! +0x2c matrix), clip def 0x18 (+0 u16 n frames, +4 frames, +8 length, +0xc screen (+0x54 protos),
//! +0x10 sprite, +0x14 name), clip state 0x18, frame 0x20, placement 0x40, element 0x70, proto 0x40.
//!
//! 1:1: FUN_00219fe0 (advance a clip state), FUN_00219400 (apply a frame's placements), FUN_00219190
//! (sound cues), FUN_002190f8 (frame commands, 7 = stop), FUN_00219348 (frame removals), FUN_00218790
//! (colour transform combine), FUN_00215850 / FUN_002158c0 / FUN_002156b8 / FUN_00215730 (element
//! draw / release / reset / create), FUN_00218748, FUN_00218660 (stop a state), FUN_00217f38 (release a
//! state's elements), FUN_002183e8 (tick + draw a sprite state), FUN_00217ef8 (free state), FUN_0011a058.
//! FUN_002188b0 (stopped state draw, mode 1).
//! HOST (`MovieHost`, not ported): FUN_00217ca8 (start a
//! named clip; FUN_00217d98), FUN_001beed8 / FUN_001beb28 / FUN_001bed18 (kind-6 world object attach /
//! detach), the sound calls (emitter_create + FUN_001fb2f8 + vt+0x44 state 2, FUN_001fb098 stop,
//! FUN_001fb018 play, FUN_001fe5f0 / FUN_001fe738 on the music manager 0x3dd040).
//! NOT wired: no ported caller draws screens (ui::hud / hud_slots REPLACED them); the conversion of the
//! loaded FUN_00214460 groups (3 x vec3 placement matrices, FUN_00117ed0) into `Movie` is not done.
//! Original quirk kept: FUN_00219400 tweens at clamp(dt, t0, t1) (f13 = the caller's dt, see the
//! disassembly at 0x21972c), so freshly placed elements start at the first key.
#![allow(non_snake_case)]

/// DAT_002a3a10: identity 2x3 matrix (a, b, c, d, tx, ty).
pub const IDENT: [f32; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
pub const ADD0: u32 = 0x7f7f_7f7f;
pub const MUL0: u32 = 0xffff_ffff;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Placement {
    /// +0 proto id (0 = keep / none)
    pub id: u16,
    /// +2 element index
    pub elem: u16,
    /// +4 mask depth (elements [elem+1 ..= elem+mask] are masked by this one)
    pub mask: u16,
    /// +6 colour index into the frame's colour table (< 0 none)
    pub colour: i16,
    /// +0x10 matrix
    pub m: [f32; 6],
    /// +0x30 tween to the next key of this element, +0x31, +0x34
    pub tween: u8,
    pub b31: u8,
    pub w34: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Frame {
    /// +0
    pub time: f32,
    /// +4 / +0xc
    pub places: Vec<Placement>,
    /// +0x10 (mul, add) pairs
    pub colours: Vec<(u32, u32)>,
    /// +6 / +0x14 removed elements
    pub resets: Vec<u16>,
    /// +7 / +8 commands
    pub cmds: Vec<u8>,
    /// +0x18 / +0x1c (kind, arg) sound cues
    pub cues: Vec<(i32, i32)>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClipDef {
    pub frames: Vec<Frame>,
    /// +8
    pub len: f32,
    /// +0x14
    pub name: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Elem {
    /// +0 add, +4 mul (combined)
    pub add: u32,
    pub mul: u32,
    /// +0x10 world matrix (+0x28 two words are stack garbage in the original, not kept)
    pub m: [f32; 6],
    /// +0x30 proto index
    pub obj: Option<usize>,
    /// +0x34 0 / 1 masking / 4 masked
    pub mask: i32,
    /// +0x38 / +0x3c / +0x40 / +0x44 start key
    pub t0: f32,
    pub add0: u32,
    pub mul0: u32,
    pub m0: [f32; 6],
    /// +0x48 / +0x4c / +0x50 / +0x54 end key
    pub t1: f32,
    pub add1: u32,
    pub mul1: u32,
    pub m1: Option<[f32; 6]>,
    /// +0x58 nested sprite state
    pub clip: i32,
    /// +0x5c time of the last update
    pub last: f32,
    /// +0x60 / +0x61 / +0x64 / +0x68
    pub tween: u8,
    pub b61: u8,
    pub w64: u32,
    pub f68: u32,
}

impl Default for Elem {
    fn default() -> Self {
        let mut e = Elem {
            add: 0,
            mul: 0,
            m: [0.0; 6],
            obj: None,
            mask: 0,
            t0: 0.0,
            add0: 0,
            mul0: 0,
            m0: IDENT,
            t1: 0.0,
            add1: 0,
            mul1: 0,
            m1: None,
            clip: 0,
            last: 0.0,
            tween: 0,
            b61: 0,
            w64: 0,
            f68: 0,
        };
        ee_sub_2156b8(&mut e);
        e
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClipState {
    /// +0 / +4
    pub elems: Vec<Elem>,
    /// +8 clip def (< 0 free)
    pub def: i16,
    /// +0xa next frame
    pub next: u16,
    /// +0xc
    pub time: f32,
    /// +0x10 0 playing, 1 stopped (FUN_002188b0)
    pub mode: i32,
    /// +0x14 looping sound
    pub sound: u32,
}

impl ClipState {
    pub fn new(n: usize) -> Self {
        ClipState { elems: vec![Elem::default(); n], def: -1, next: 0, time: 0.0, mode: 0, sound: 0 }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sprite {
    pub defs: Vec<ClipDef>,
    pub states: Vec<ClipState>,
    /// +0x10 (DAT_002ddf18), +0x18 time, +0x1c def, +0x20 of the last stopped state
    pub q10: u64,
    pub last_time: f32,
    pub last_def: i32,
    pub r20: i32,
    /// +0x2c
    pub m: Option<[f32; 6]>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Proto {
    /// +0
    pub id: u32,
    /// +4 4 = sprite, 6 = world object
    pub kind: i32,
    /// +0x10 sprite index (kind 4) / object id (kind 6, -1 none)
    pub sprite: i32,
    /// +0x14 attached object (kind 6)
    pub handle: u32,
}

/// One screen: its sprites (+8 -> +0x84 table) and protos (+0xc -> +0x54 table, indexed by id).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Movie {
    pub sprites: Vec<Sprite>,
    pub protos: Vec<Proto>,
}

pub trait MovieHost {
    /// FUN_00217ca8(sprite, name, state, matrix, add, mul)
    fn ee_sub_217ca8(&mut self, _mv: &mut Movie, _sprite: usize, _name: i32, _state: usize, _m: &[f32; 6], _add: u32, _mul: u32) {}
    /// FUN_001beed8 + FUN_001beb28: attach the world object of a kind-6 proto (0 = none)
    fn attach_obj(&mut self, _oid: i32) -> u32 {
        0
    }
    /// FUN_001bed18 when the handle's +0x20 is set
    fn detach_obj(&mut self, _handle: u32) {}
    /// cue 0: emitter_create(sound) + FUN_001fb2f8; returns the emitter when its vt+0x44 state is 2 (looping)
    fn sound_start(&mut self, _sound: i32) -> u32 {
        0
    }
    /// FUN_001fb098 stop + FUN_00258040 release
    fn sound_stop(&mut self, _h: u32) {}
    /// FUN_001fb018
    fn sound_play(&mut self, _h: u32) {}
    /// cue 2: FUN_001fe5f0(0x3dd040, arg, 0)
    fn music_1fe5f0(&mut self, _arg: i32) {}
    /// cue 3: FUN_001fe738(0x3dd040)
    fn music_1fe738(&mut self) {}
}

pub struct NoMovieHost;
impl MovieHost for NoMovieHost {}

/// `m * p` (2x3 rows, the inline in FUN_00219fe0 / FUN_00219400 / FUN_002183e8)
pub fn mat_mul(m: &[f32; 6], p: &[f32; 6]) -> [f32; 6] {
    [
        m[0] * p[0] + m[1] * p[2],
        m[0] * p[1] + m[1] * p[3],
        m[2] * p[0] + m[3] * p[2],
        m[2] * p[1] + m[3] * p[3],
        m[4] * p[0] + m[5] * p[2] + p[4],
        m[4] * p[1] + m[5] * p[3] + p[5],
    ]
}

/// The byte-wise colour tween (inline in FUN_00219fe0 / FUN_00219400).
pub fn lerp_colour(a: u32, b: u32, t: f32) -> u32 {
    let (a, b) = (a.to_le_bytes(), b.to_le_bytes());
    let mut o = [0u8; 4];
    for i in 0..4 {
        let (x, y) = (a[i], b[i]);
        o[i] = if x == y {
            y
        } else if x < y {
            x.wrapping_add((t * ((y as f32 + 0.999) - x as f32)) as i32 as u8)
        } else {
            y.wrapping_add(((1.0 - t) * ((x as f32 + 0.999) - y as f32)) as i32 as u8)
        };
    }
    u32::from_le_bytes(o)
}

fn lerp6(a: &[f32; 6], b: &[f32; 6], t: f32) -> [f32; 6] {
    core::array::from_fn(|i| (1.0 - t) * a[i] + t * b[i])
}

/// FUN_00218790: combine the parent (mul, add) with the element's -> (mul, add).
pub fn ee_sub_218790(pmul: u32, padd: u32, mul: u32, add: u32) -> (u32, u32) {
    let (p1, p2, p3, p4) = (pmul.to_le_bytes(), padd.to_le_bytes(), mul.to_le_bytes(), add.to_le_bytes());
    let (mut o5, mut o6) = ([0u8; 4], [0u8; 4]);
    for i in 0..4 {
        let mut a = (p1[i] as i32 - 0x7f) as f32 * 0.007874016;
        let mut c = (p3[i] as i32 - 0x7f) as f32 * 0.007874016;
        a = if a < 1.0 { a } else { 1.0 };
        c = if c < 1.0 { c } else { 1.0 };
        a = if -1.0 < a { a } else { -1.0 };
        let mut m = a * if -1.0 < c { c } else { -1.0 };
        m = if m < 1.0 { m } else { 1.0 };
        let mut s = a * (p4[i] as f32 - 127.0) + (p2[i] as f32 - 127.0);
        s = if s < 128.0 { s } else { 128.0 };
        o5[i] = ((if -1.0 < m { m } else { -1.0 }) * 127.0 + 128.0) as i32 as u8;
        o6[i] = ((if -127.0 < s { s } else { -127.0 }) + 127.0) as i32 as u8;
    }
    (u32::from_le_bytes(o5), u32::from_le_bytes(o6))
}

/// FUN_002156b8: element defaults (FUN_0011a058 clears +0x68).
pub fn ee_sub_2156b8(e: &mut Elem) {
    e.m0 = IDENT;
    e.add1 = ADD0;
    e.mul1 = MUL0;
    e.w64 = 0xffff_ffff;
    e.obj = None;
    e.m1 = None;
    e.add = ADD0;
    e.mul = MUL0;
    e.add0 = ADD0;
    e.mul0 = MUL0;
    e.clip = 0;
    e.mask = 0;
    e.last = 0.0;
    e.t0 = 0.0;
    e.t1 = 0.0;
    e.f68 = 0;
}

fn clamp(v: f32, lo: f32, hi: f32) -> f32 {
    if v < lo {
        lo
    } else if hi < v {
        hi
    } else {
        v
    }
}

/// The element evaluation inline in FUN_00219fe0 / FUN_002188b0: the key (tweened to `t` when the
/// element tweens and has an end key), times the parent matrix and colour transform -> (m, add, mul).
fn eval_elem(e: &Elem, t: f32, pm: &[f32; 6], padd: u32, pmul: u32) -> ([f32; 6], u32, u32) {
    let (m, add, mul) = match e.m1 {
        Some(m1) if e.tween != 0 => {
            let r = (t - e.t0) / (e.t1 - e.t0);
            (lerp6(&e.m0, &m1, r), lerp_colour(e.add0, e.add1, r), lerp_colour(e.mul0, e.mul1, r))
        }
        _ => (e.m0, e.add0, e.mul0),
    };
    let (mul, add) = ee_sub_218790(pmul, padd, mul, add);
    (mat_mul(&m, pm), add, mul)
}

impl Movie {
    /// FUN_002188b0(dt, def, state, matrix, add, mul): draw a stopped state (mode 1) at its time; elements
    /// without an object only take the time.
    pub fn ee_sub_2188b0(&mut self, h: &mut dyn MovieHost, dt: f32, sprite: usize, state: usize, pm: &[f32; 6], padd: u32, pmul: u32) {
        let t = self.sprites[sprite].states[state].time;
        for i in 0..self.sprites[sprite].states[state].elems.len() {
            let e = &mut self.sprites[sprite].states[state].elems[i];
            let Some(o) = e.obj else {
                e.last = t;
                e.f68 = 0;
                continue;
            };
            let (m, add, mul) = eval_elem(e, t, pm, padd, pmul);
            let clip = e.clip;
            self.ee_sub_215850(h, dt, Some(o), clip, &m, add, mul);
            let e = &mut self.sprites[sprite].states[state].elems[i];
            e.last = t;
            e.m = m;
            e.add = add;
            e.mul = mul;
            e.f68 = 0;
        }
    }

    /// FUN_00217ef8: first free state of a sprite (n states when none).
    pub fn ee_sub_217ef8(&self, sprite: usize) -> usize {
        let s = &self.sprites[sprite];
        s.states.iter().position(|st| st.def < 0).unwrap_or(s.states.len())
    }

    /// FUN_002158c0(proto, elem): release the element's object, then FUN_002156b8.
    pub fn ee_sub_2158c0(&mut self, h: &mut dyn MovieHost, sprite: usize, state: usize, elem: usize) {
        if let Some(o) = self.sprites[sprite].states[state].elems[elem].obj {
            let p = self.protos[o].clone();
            if p.kind == 4 {
                let clip = self.sprites[sprite].states[state].elems[elem].clip;
                self.ee_sub_218660(h, p.sprite as usize, clip as usize);
            } else if p.kind == 6 && p.handle != 0 {
                h.detach_obj(p.handle);
                self.protos[o].handle = 0;
            }
        }
        ee_sub_2156b8(&mut self.sprites[sprite].states[state].elems[elem]);
    }

    /// FUN_00217f38: release / reset every element of a state.
    pub fn ee_sub_217f38(&mut self, h: &mut dyn MovieHost, sprite: usize, state: usize) {
        for i in 0..self.sprites[sprite].states[state].elems.len() {
            if self.sprites[sprite].states[state].elems[i].obj.is_none() {
                ee_sub_2156b8(&mut self.sprites[sprite].states[state].elems[i]);
            } else {
                self.ee_sub_2158c0(h, sprite, state, i);
            }
        }
    }

    /// FUN_00218660: stop a sprite state (remember it in +0x10..+0x20), drop its sound, release elements.
    pub fn ee_sub_218660(&mut self, h: &mut dyn MovieHost, sprite: usize, state: usize) {
        let s = &mut self.sprites[sprite];
        let st = &mut s.states[state];
        if st.def >= 0 {
            s.last_def = st.def as i32;
            s.last_time = st.time;
            s.r20 = -1;
            s.q10 = 0; // DAT_002ddf18
            st.def = -1;
            st.mode = 0;
            if st.sound != 0 {
                h.sound_stop(st.sound);
                st.sound = 0;
            }
        }
        self.ee_sub_217f38(h, sprite, state);
    }

    /// FUN_00218748: frame command 7, stop the state (and restart its sound).
    pub fn ee_sub_218748(&mut self, h: &mut dyn MovieHost, sprite: usize, state: usize) {
        let st = &mut self.sprites[sprite].states[state];
        st.mode = 1;
        if st.sound != 0 {
            h.sound_play(st.sound);
        }
    }

    /// FUN_00215730(proto, elem, matrix, add, mul): create the element's object.
    pub fn ee_sub_215730(&mut self, h: &mut dyn MovieHost, proto: usize, sprite: usize, state: usize, elem: usize, m: &[f32; 6], add: u32, mul: u32) {
        self.sprites[sprite].states[state].elems[elem].obj = Some(proto);
        let p = self.protos[proto].clone();
        if p.kind == 4 {
            let child = self.ee_sub_217ef8(p.sprite as usize);
            self.sprites[sprite].states[state].elems[elem].clip = child as i32;
            h.ee_sub_217ca8(self, p.sprite as usize, 0x314, child, m, add, mul);
        } else if p.kind == 6 && p.sprite != -1 {
            let o = h.attach_obj(p.sprite);
            if o != 0 {
                self.protos[proto].handle = o;
            }
        }
    }

    /// FUN_00215850(dt, proto, elem, matrix, add, mul): draw a sprite element (kind 4 only).
    pub fn ee_sub_215850(&mut self, h: &mut dyn MovieHost, dt: f32, proto: Option<usize>, clip: i32, m: &[f32; 6], add: u32, mul: u32) {
        let Some(p) = proto else { return };
        if self.protos[p].kind == 4 {
            let s = self.protos[p].sprite as usize;
            self.ee_sub_2183e8(h, dt, s, clip as usize, m, add, mul);
        }
    }

    /// FUN_002183e8(dt, sprite, state, matrix, add, mul): tick (loop at the clip length) and draw a state.
    pub fn ee_sub_2183e8(&mut self, h: &mut dyn MovieHost, dt: f32, sprite: usize, state: usize, m: &[f32; 6], add: u32, mul: u32) {
        let s = &mut self.sprites[sprite];
        let def = s.states[state].def;
        if def < 0 {
            return;
        }
        let m = match s.m {
            Some(p) => mat_mul(m, &p),
            None => *m,
        };
        let def = def as usize;
        if s.states[state].mode == 0 {
            let st = &mut s.states[state];
            if s.defs[def].len <= st.time {
                st.next = 0;
                st.time = 0.0;
            } else {
                st.time += dt;
            }
            self.ee_sub_219fe0(h, dt, sprite, def, state, &m, add, mul);
        } else {
            self.ee_sub_2188b0(h, dt, sprite, state, &m, add, mul);
        }
    }

    /// FUN_00219348: frame removals.
    pub fn ee_sub_219348(&mut self, h: &mut dyn MovieHost, sprite: usize, def: usize, frame: usize, state: usize) {
        for k in 0..self.sprites[sprite].defs[def].frames[frame].resets.len() {
            let i = self.sprites[sprite].defs[def].frames[frame].resets[k] as usize;
            if self.sprites[sprite].states[state].elems[i].obj.is_none() {
                ee_sub_2156b8(&mut self.sprites[sprite].states[state].elems[i]);
            } else {
                self.ee_sub_2158c0(h, sprite, state, i);
            }
        }
    }

    /// FUN_002190f8: frame commands (7 = FUN_00218748).
    pub fn ee_sub_2190f8(&mut self, h: &mut dyn MovieHost, sprite: usize, def: usize, frame: usize, state: usize) {
        for k in 0..self.sprites[sprite].defs[def].frames[frame].cmds.len() {
            if self.sprites[sprite].defs[def].frames[frame].cmds[k] == 7 {
                self.ee_sub_218748(h, sprite, state);
            }
        }
    }

    /// FUN_00219190: sound cues (0 start, 1 stop the state's looping sound, 2 / 3 music manager).
    pub fn ee_sub_219190(&mut self, h: &mut dyn MovieHost, sprite: usize, def: usize, frame: usize, state: usize) {
        for k in 0..self.sprites[sprite].defs[def].frames[frame].cues.len() {
            let (kind, arg) = self.sprites[sprite].defs[def].frames[frame].cues[k];
            match kind {
                0 => {
                    let e = h.sound_start(arg);
                    if e != 0 {
                        self.sprites[sprite].states[state].sound = e;
                    }
                }
                1 => {
                    let st = &mut self.sprites[sprite].states[state];
                    if st.sound != 0 {
                        h.sound_stop(st.sound);
                        st.sound = 0;
                    }
                }
                2 => h.music_1fe5f0(arg),
                3 => h.music_1fe738(),
                _ => {}
            }
        }
    }

    /// FUN_00219400(frame time, dt, def, frame, state, matrix, add, mul): apply a frame's placements.
    pub fn ee_sub_219400(&mut self, h: &mut dyn MovieHost, ft: f32, dt: f32, sprite: usize, def: usize, fi: usize, state: usize, pm: &[f32; 6], padd: u32, pmul: u32) {
        let n = self.sprites[sprite].defs[def].frames[fi].places.len();
        for pi in 0..n {
            let d = &self.sprites[sprite].defs[def];
            let fr = &d.frames[fi];
            let pl = fr.places[pi].clone();
            let col = (pl.colour >= 0).then(|| fr.colours[pl.colour as usize]);
            let ei = pl.elem as usize;
            // the tween target: the element's next key in a later frame
            let mut end = None;
            if pl.tween != 0 {
                for f2 in &d.frames[fi + 1..] {
                    if let Some(q) = f2.places.iter().find(|q| q.elem == pl.elem) {
                        end = Some((f2.time, q.m, (q.colour >= 0).then(|| f2.colours[q.colour as usize])));
                        break;
                    }
                }
            }
            let ftime = fr.time;
            let e = &mut self.sprites[sprite].states[state].elems[ei];
            (e.add0, e.mul0) = match col {
                Some((mul, add)) => (add, mul),
                None => (ADD0, MUL0),
            };
            e.m0 = pl.m;
            e.m1 = None;
            e.t0 = ftime;
            e.t1 = 0.0;
            if let Some((t1, m1, c1)) = end {
                e.m1 = Some(m1);
                e.t1 = t1;
                (e.add1, e.mul1) = match c1 {
                    Some((mul, add)) => (add, mul),
                    None => (e.add0, e.mul0),
                };
            }
            let (m, add, mul) = match e.m1 {
                Some(m1) if pl.tween != 0 && e.t0 < e.t1 => {
                    let r = (clamp(dt, e.t0, e.t1) - e.t0) / (e.t1 - e.t0);
                    (lerp6(&e.m0, &m1, r), lerp_colour(e.add0, e.add1, r), lerp_colour(e.mul0, e.mul1, r))
                }
                _ => (e.m0, e.add0, e.mul0),
            };
            let m = mat_mul(&m, pm);
            let (mul, add) = ee_sub_218790(pmul, padd, mul, add);
            let id = pl.id as usize;
            let obj = e.obj;
            let clip = e.clip;
            if id < 1 {
                self.ee_sub_215850(h, dt, obj, clip, &m, add, mul);
            } else if obj.is_some_and(|o| self.protos[o].id as usize == id) {
                self.ee_sub_215850(h, dt, obj, clip, &m, add, mul);
            } else {
                if obj.is_some() {
                    self.ee_sub_2158c0(h, sprite, state, ei);
                }
                self.ee_sub_215730(h, id, sprite, state, ei, &m, add, mul);
            }
            let els = &mut self.sprites[sprite].states[state].elems;
            let e = &mut els[ei];
            e.last = ft;
            e.m = m;
            e.add = add;
            e.mul = mul;
            e.tween = pl.tween;
            e.b61 = pl.b31;
            e.w64 = pl.w34;
            e.f68 = 0;
            if pl.mask == 0 {
                if e.mask != 4 {
                    e.mask = 0;
                }
            } else {
                e.mask = 1;
                els[ei + pl.mask as usize].mask = 4;
            }
        }
    }

    /// FUN_00219fe0(dt, def, state, matrix, add, mul): run the due frames of a clip state and update
    /// every live element at the newest key time (a restart releases the elements frame 0 did not place).
    pub fn ee_sub_219fe0(&mut self, h: &mut dyn MovieHost, dt: f32, sprite: usize, def: usize, state: usize, pm: &[f32; 6], padd: u32, pmul: u32) {
        let t = self.sprites[sprite].states[state].time;
        let nf = self.sprites[sprite].defs[def].frames.len();
        let mut fi = self.sprites[sprite].states[state].next as usize;
        let mut first = fi == 0;
        let mut cur = t;
        loop {
            if fi < nf && self.sprites[sprite].defs[def].frames[fi].time <= t {
                let ft = self.sprites[sprite].defs[def].frames[fi].time;
                self.ee_sub_219190(h, sprite, def, fi, state);
                self.ee_sub_2190f8(h, sprite, def, fi, state);
                self.ee_sub_219348(h, sprite, def, fi, state);
                self.ee_sub_219400(h, ft, dt, sprite, def, fi, state, pm, padd, pmul);
                let st = &mut self.sprites[sprite].states[state];
                st.next += 1;
                if nf < st.next as usize {
                    st.next = nf as u16;
                }
                cur = ft;
                fi += 1;
            }
            for i in 0..self.sprites[sprite].states[state].elems.len() {
                let e = &self.sprites[sprite].states[state].elems[i];
                let Some(o) = e.obj else { continue };
                if e.last == cur {
                    continue;
                }
                if first {
                    self.ee_sub_2158c0(h, sprite, state, i);
                    continue;
                }
                let (m, add, mul) = eval_elem(e, cur, pm, padd, pmul);
                let clip = e.clip;
                self.ee_sub_215850(h, dt, Some(o), clip, &m, add, mul);
                let e = &mut self.sprites[sprite].states[state].elems[i];
                e.last = cur;
                e.m = m;
                e.add = add;
                e.mul = mul;
                e.f68 = 0;
            }
            first = false;
            if nf <= fi || t < self.sprites[sprite].defs[def].frames[fi].time {
                return;
            }
        }
    }
}
