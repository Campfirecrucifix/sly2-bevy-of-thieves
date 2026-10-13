#![allow(non_snake_case)]
//! WP-X3: binocucom recon (photo) mode = the binocucom HUD's +0x14 object (`BinocHud` param_1[5]): the
//! player looks at recon targets (class 46 C2d9f70 objects) through the binocucom, centres and zooms one and
//! "takes the photo" (state 8 freezes the clock), which completes the target's job.
//!
//! 1:1: FUN_00202d38 (per frame, called by FUN_0019da78 while +0x14 != 0), FUN_00202768 (state change,
//! called by FUN_0019d660 with 0 / 10 / -1), FUN_00202540 (pick the nearest visible target), FUN_00202378
//! (4 occlusion rays -> target +0x108: 1 clear, 3 partial, 2 hidden), FUN_00201e88 / FUN_00201f00
//! (link object vtable +0x4c / +0x44 for every target), FUN_00201f90 (any 0x505 sensor contains the eye),
//! FUN_00202030 (list up to 5 targets with an id), FUN_002020c8 (link fade by camera distance),
//! FUN_002021f8 (list elements 0x2f5..0x2f9: 0x33a done / 0x33b open), FUN_002022a0, FUN_002022d8,
//! FUN_002036e8 (complete), FUN_00203758 (done?), FUN_00203790 (pickable?).
//! States: -1 off, 0 opening (screen anim 0x314), 1 idle, 2 searching, 3 partly hidden, 4 hidden / not
//! centred, 5 too small, 6 too big, 7 framed, 8 photo, 9 photo done, 10 closing.
//! Jump table 0x2a85c0 (index state + 1) and every FUN_00202768 argument were read in the raw disassembly
//! (the decompile drops the a1 delay-slot values).
//! HOST (`ReconHost`): screen objects (FUN_00213928 hide / FUN_00213b00 show / FUN_00213bd8 /
//! FUN_00212e40 scrub / FUN_00214300 / vtable +0x18c / +0x194 / FUN_002121e8 playing), the clock
//! (DAT_002dded0, FUN_0014d1d0), world (FUN_0020efc0 / FUN_0020f030, ClearWorld + FUN_0014cc58 rays),
//! sounds, the binocucom object (+0xa8), pads (Revoke 0x2de2e0, FUN_001c46c0).
//! NOT wired: the port binocucom has no recon object (no ctor for C2d9f70 recon lists is ported), so
//! `BinocHud::recon` stays None in the game.
use crate::engine::camera::cm::{Cm, FUN_0014f308, FUN_0014f850, FUN_0014f898};

pub type V3 = [f32; 3];

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn scale(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn len(a: V3) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}
fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

/// A recon target (class 46 C2d9f70 object), the fields the recon code reads.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReconTarget {
    /// FUN_001a62e8 position
    pub pos: V3,
    /// +0x28 bit 60
    pub active: bool,
    /// +0x38 render-distance scale
    pub mrd: f32,
    /// +0xe0 frustum radius
    pub r_e0: f32,
    /// +0xe4 size radius (screen size and occlusion rays)
    pub r_e4: f32,
    /// +0xe8 zoom tolerance
    pub tol_e8: f32,
    /// +0xec job (Some(job +0x54 state)), None = no job
    pub job: Option<i32>,
    /// +0xf0 done flag when there is no job
    pub done_f0: i32,
    /// +0xf4
    pub f_f4: i32,
    /// +0xf8 list id (-1 = not listed)
    pub id_f8: i32,
    /// +0xfc sound / getter id (-1 = none)
    pub snd_fc: i32,
    /// +0x100 link object (Some(its +0x28 bit 60)), None = no link
    pub link: Option<bool>,
    /// +0x104 link fade distance
    pub fade_104: f32,
    /// +0x108 visibility: 0 unknown, 1 clear, 2 hidden, 3 partial
    pub vis_108: i32,
    /// +0x10c / +0x110
    pub f_10c: i32,
    pub f_110: i32,
}

/// Calls out of the recon object.
pub trait ReconHost {
    /// DAT_002dded0 clock time
    fn clock(&self) -> f32;
    /// FUN_0014d1d0(scale, 0x2dded0, 0)
    fn clock_scale(&mut self, s: f32);
    /// FUN_00213928(screen, element) hide
    fn elem_hide(&mut self, scr: u32, e: i32);
    /// FUN_00213b00(screen, element) show / play
    fn elem_show(&mut self, scr: u32, e: i32);
    /// FUN_00213bd8(screen, element, arg)
    fn elem_set(&mut self, scr: u32, e: i32, arg: i32);
    /// FUN_00212e40(t, screen, element) scrub
    fn elem_scrub(&mut self, scr: u32, e: i32, t: f32);
    /// FUN_002121e8(screen, element): element animation playing
    fn elem_playing(&mut self, scr: u32, e: i32) -> bool;
    /// FUN_00214300(screen, arg)
    fn screen_214300(&mut self, scr: u32, arg: i32);
    /// screen vtable +0x18c (message)
    fn screen_msg(&mut self, scr: u32, msg: i32);
    /// screen vtable +0x194 (close)
    fn screen_close(&mut self, scr: u32);
    /// screen +0x58 (0 = closed)
    fn screen_state(&self, scr: u32) -> i32;
    /// FUN_00212a70(screen, 1) + FUN_00212a50(screen, 1)
    fn screen_off(&mut self, scr: u32);
    /// screen +0xf4 = target
    fn screen_set_target(&mut self, scr: u32, t: Option<usize>);
    /// screen word at +off (+0x110..+0x120 result values)
    fn screen_word(&self, scr: u32, off: u32) -> i32;
    /// FUN_001619a0(0x2deb30, msg, 0)
    fn game_msg(&mut self, msg: i32);
    /// FUN_0020efc0(0x3dd490) / FUN_0020f030(0x3dd490, 0)
    fn world_pause(&mut self, on: bool);
    /// FUN_001fe5b8(0x3dd040, 1, on)
    fn music_1fe5b8(&mut self, on: i32);
    /// FUN_0014fe28(0x2de390)
    fn cam_preset_off(&mut self);
    /// getters_call FUN_001fe5f0(0x3dd040, id, 0)
    fn getter_call(&mut self, id: i32);
    /// FUN_00147c20(obj, 0x608, id)
    fn obj_msg(&mut self, obj: u32, msg: i32, arg: i32);
    /// emitter_create(0x52) + FUN_001fb2f8(e, 0)
    fn sound(&mut self, id: i32);
    /// binocucom (+0xa8) object +0x40 flag word (tagged pointer resolved)
    fn bn_flags(&self) -> u32;
    fn set_bn_flags(&mut self, v: u32);
    /// binocucom vtable +0x30c: eye position
    fn bn_eye(&self) -> V3;
    /// binocucom +0x20 (1 = through an object)
    fn bn_mode(&self) -> i32;
    /// FUN_0019cce8(binocucom)
    fn bn_19cce8(&mut self);
    /// Revoke(0x2de2e0)
    fn revoke(&mut self) -> bool;
    /// FUN_001c46c0(0, DAT_002e1e40 +0x150, mask)
    fn joy_take_pressed(&mut self, mask: u32) -> bool;
    /// FUN_001ae478 + ClearWorld(0x3dd490, pos, eye, 0x202350, ..) + FUN_0014cc58(0, eye, p, ..) != 0
    fn ray_blocked(&mut self, t: usize, eye: V3, p: V3) -> bool;
    /// FUN_0015e9c8(0x3dd490, 0x505, 0x19, 0, 0x10) + FUN_00227c90(sensor, eye) for any
    fn sensor_contains(&mut self, eye: V3) -> bool;
    /// link object vtable +0x4c / +0x44 / FUN_001f7bf8(0.25) / FUN_001f7c68(0.25)
    fn link_4c(&mut self, t: usize);
    fn link_44(&mut self, t: usize);
    fn link_1f7bf8(&mut self, t: usize, f: f32);
    fn link_1f7c68(&mut self, t: usize, f: f32);
    /// FUN_00164030(job)
    fn job_done(&mut self, t: usize) -> bool;
    /// FUN_00164640(job, 2) + FUN_001c4840 / FUN_00132b10(0x34) / FUN_0019d948(jt + 0xf24)
    fn job_complete(&mut self, t: usize);
}

/// Recon object (binocucom HUD +0x14).
#[derive(Clone, Debug, PartialEq)]
pub struct BinocRecon {
    /// +0x00
    pub state: i32,
    /// +0x10 screen centre (pixels; inf: set by the unported ctor, default = 640 x 492.8 centre)
    pub center: [f32; 2],
    /// +0x20 eye (+0x28 z + 20)
    pub eye: V3,
    /// +0x30 count / +0x34 targets (max 16)
    pub targets: Vec<ReconTarget>,
    /// +0x74 / +0x78: listed targets (max 5)
    pub list: Vec<usize>,
    /// +0x8c current target
    pub cur: Option<usize>,
    /// +0x90 centred (0..1), +0x94 zoom (0..1), +0x98 too small
    pub centered: f32,
    pub zoom: f32,
    pub small: i32,
    /// +0x9c photo time
    pub photo_t: f32,
    /// +0xa0 saved binocucom flags
    pub saved_flags: u32,
    /// +0xa4 message object (0 = none)
    pub msg_obj: u32,
    /// +0xac / +0xb0 screens (0 = none)
    pub scr_a: u32,
    pub scr_b: u32,
    /// +0xb4 / +0xb8
    pub refresh: i32,
    pub in_sensor: i32,
    /// +0xbc / +0xc0
    pub f_bc: i32,
    pub f_c0: i32,
}

impl Default for BinocRecon {
    fn default() -> Self {
        Self {
            state: -1,
            center: [320.0, 246.4],
            eye: [0.0; 3],
            targets: Vec::new(),
            list: Vec::new(),
            cur: None,
            centered: 0.0,
            zoom: 0.0,
            small: 0,
            photo_t: 0.0,
            saved_flags: 0,
            msg_obj: 0,
            scr_a: 0,
            scr_b: 0,
            refresh: 0,
            in_sensor: 0,
            f_bc: -1,
            f_c0: -1,
        }
    }
}

impl BinocRecon {
    /// 0x00201e88
    pub fn FUN_00201e88(&mut self, h: &mut dyn ReconHost) {
        for i in 0..self.targets.len() {
            if self.targets[i].link.is_some() {
                h.link_4c(i);
            }
        }
    }

    /// 0x00201f00
    pub fn FUN_00201f00(&mut self, h: &mut dyn ReconHost) {
        for i in 0..self.targets.len() {
            if self.targets[i].active && self.targets[i].link.is_some() {
                h.link_44(i);
            }
        }
    }

    /// 0x00201f90
    pub fn FUN_00201f90(&mut self, h: &mut dyn ReconHost) {
        self.in_sensor = 0;
        if h.sensor_contains(self.eye) {
            self.in_sensor = 1;
        }
    }

    /// 0x00202030
    pub fn FUN_00202030(&mut self) {
        self.list.clear();
        for (i, t) in self.targets.iter().enumerate() {
            if t.id_f8 == -1 {
                continue;
            }
            if let Some(js) = t.job {
                if js != 1 && js != 2 {
                    continue;
                }
            }
            if self.list.len() < 5 {
                self.list.push(i);
            }
        }
    }

    /// 0x002020c8: link fade by camera distance (0.25 s)
    pub fn FUN_002020c8(&mut self, h: &mut dyn ReconHost, cm: &Cm) {
        let cp = cm.pos();
        for i in 0..self.targets.len() {
            let t = &self.targets[i];
            let Some(link) = t.link else { continue };
            let d = len(sub(t.pos, cp));
            if t.fade_104 < d && !link {
                h.link_1f7c68(i, 0.25);
            } else if d <= t.fade_104 && link {
                h.link_1f7bf8(i, 0.25);
            }
        }
    }

    /// 0x002021f8
    pub fn FUN_002021f8(&mut self, h: &mut dyn ReconHost) {
        const DAT_002C3588: [i32; 5] = [0x2f5, 0x2f6, 0x2f7, 0x2f8, 0x2f9];
        for k in 0..self.list.len() {
            let e = if self.FUN_00203758(h, self.list[k]) { 0x33a } else { 0x33b };
            h.elem_set(self.scr_a, e, DAT_002C3588[k]);
        }
    }

    /// 0x002022a0
    pub fn FUN_002022a0(&mut self) {
        for t in &mut self.targets {
            t.vis_108 = 0;
        }
    }

    /// 0x002022d8: anything left to photograph
    pub fn FUN_002022d8(&self) -> bool {
        if self.in_sensor != 0 {
            return true;
        }
        self.targets.iter().any(|t| t.active && t.link != Some(true))
    }

    /// 0x00203758
    pub fn FUN_00203758(&self, h: &mut dyn ReconHost, t: usize) -> bool {
        if self.targets[t].job.is_none() {
            self.targets[t].done_f0 != 0
        } else {
            h.job_done(t)
        }
    }

    /// 0x00203790
    pub fn FUN_00203790(&self, t: usize) -> bool {
        let t = &self.targets[t];
        if !t.active || t.link == Some(true) {
            return false;
        }
        matches!(t.job, None | Some(1))
    }

    /// 0x002036e8
    pub fn FUN_002036e8(&mut self, h: &mut dyn ReconHost, t: usize, v: i32) {
        if self.targets[t].job.is_none() {
            self.targets[t].done_f0 = v;
        } else {
            h.job_complete(t);
        }
    }

    /// 0x00202378: 4 rays from the eye to the target's near side (centre, +up, -up, +side) -> +0x108
    pub fn FUN_00202378(&mut self, h: &mut dyn ReconHost, cm: &Cm, t: usize) {
        let pos = self.targets[t].pos;
        let r = self.targets[t].r_e4;
        let up = [cm.mat[1][0], cm.mat[1][1], cm.mat[1][2]]; // DAT_002ddfb0 = cm +0x70
        let v = sub(self.eye, pos);
        let dir = scale(v, 1.0 / len(v));
        let side = cross(up, dir);
        let side = scale(side, 1.0 / len(side));
        let c = add(pos, scale(dir, r));
        let ur = scale(up, r);
        let pts = [c, add(c, ur), sub(c, ur), add(c, scale(side, r))];
        let mut n = 0;
        for p in pts {
            if h.ray_blocked(t, self.eye, p) {
                n += 1;
            }
        }
        self.targets[t].vis_108 = match n {
            4 => 2,
            0 => 1,
            _ => 3,
        };
    }

    /// 0x00202540: nearest pickable, on-screen target that is not hidden -> +0x8c (state 2 on change)
    pub fn FUN_00202540(&mut self, h: &mut dyn ReconHost, cm: &Cm) {
        self.eye = h.bn_eye();
        self.eye[2] += 20.0;
        let cp = cm.pos();
        let mut best = f32::MAX; // DAT_002a8550
        let mut pick = None;
        for i in 0..self.targets.len() {
            if !self.FUN_00203790(i) {
                continue;
            }
            let pos = self.targets[i].pos;
            if !FUN_0014f850(cm, sub(pos, cp), self.targets[i].r_e0) {
                continue;
            }
            // unk: a2 of FUN_0014f898 at 0x202628 is an undecoded EE op; the port passes pos - camera
            if FUN_0014f898(cm, self.targets[i].mrd, sub(pos, cp)).is_none() {
                continue;
            }
            let d = len(sub(pos, cp));
            if d < best {
                self.FUN_00202378(h, cm, i);
                // quirk 0x202684: a hidden (2) target is not kept as the best
                if self.targets[i].vis_108 != 2 {
                    best = d;
                    pick = Some(i);
                }
            }
        }
        if pick != self.cur {
            self.cur = pick;
            self.FUN_00202768(h, cm, 2);
        }
    }

    /// 0x00202768
    pub fn FUN_00202768(&mut self, h: &mut dyn ReconHost, cm: &Cm, s: i32) {
        if self.state == s || self.scr_a == 0 || self.scr_b == 0 {
            return;
        }
        let photo = |x: i32| (x as u32).wrapping_sub(8) < 2;
        if photo(self.state) != photo(s) {
            h.world_pause(photo(s));
        }
        match self.state {
            1 => {
                self.FUN_002020c8(h, cm);
                self.FUN_00202030();
                h.elem_show(self.scr_a, 0x35d);
                if !self.list.is_empty() {
                    h.elem_show(self.scr_a, 0x35e);
                }
                self.FUN_002021f8(h);
            }
            4 => h.elem_hide(self.scr_b, 0x359),
            5 | 6 => h.elem_hide(self.scr_b, 0x35a),
            7 => {
                h.elem_hide(self.scr_a, 0x356);
                h.elem_hide(self.scr_b, 0x356);
                h.elem_hide(self.scr_b, 0x357);
            }
            8 => {
                h.set_bn_flags(self.saved_flags | (h.bn_flags() & 1));
                h.clock_scale(1.0);
                h.music_1fe5b8(1);
                h.elem_hide(self.scr_a, 0x358);
            }
            9 => {
                h.cam_preset_off();
                h.screen_214300(self.scr_a, -1);
                h.screen_msg(self.scr_a, 0x310);
                // the original dereferences +0x8c unchecked; state 9 is only entered with a framed target
                if let Some(t) = self.cur {
                    if self.msg_obj != 0 {
                        let snd = self.targets[t].snd_fc;
                        if snd != -1 {
                            h.getter_call(snd);
                        }
                        h.obj_msg(self.msg_obj, 0x608, snd);
                    }
                    self.FUN_002036e8(h, t, 1);
                }
                h.game_msg(10);
                self.cur = None;
                if self.FUN_002022d8() {
                    h.elem_show(self.scr_a, 0x35d);
                    if !self.list.is_empty() {
                        h.elem_show(self.scr_a, 0x35e);
                    }
                }
                self.FUN_002021f8(h);
            }
            _ => {}
        }
        let prev = self.state;
        self.state = s;
        match s {
            0 => {
                self.cur = None;
                self.f_c0 = -1;
                self.f_bc = -1;
                self.FUN_002022a0();
                h.screen_msg(self.scr_a, -1);
                self.FUN_00201e88(h);
                self.FUN_00201f90(h);
                h.game_msg(9);
            }
            1 => {
                h.elem_hide(self.scr_a, 0x35d);
                h.screen_msg(self.scr_b, 0x339);
            }
            2 => h.screen_msg(self.scr_b, 0x339),
            3 | 4 => {
                h.screen_set_target(self.scr_b, self.cur);
                h.screen_msg(self.scr_b, 0x337);
                h.elem_show(self.scr_b, 0x359);
                h.elem_scrub(self.scr_b, 0x359, 0.0);
            }
            5 | 6 => {
                h.elem_show(self.scr_b, 0x35a);
                h.elem_scrub(self.scr_b, 0x35a, 0.0);
            }
            7 => {
                h.sound(0x52);
                if let Some(t) = self.cur {
                    self.f_bc = self.targets[t].f_f4;
                }
                h.elem_show(self.scr_a, 0x356);
                h.elem_show(self.scr_b, 0x356);
                h.elem_show(self.scr_b, 0x357);
            }
            8 => {
                h.elem_hide(self.scr_a, 0x35b);
                h.elem_show(self.scr_a, 0x358);
                let f = h.bn_flags();
                self.saved_flags = f & !1;
                h.set_bn_flags(f | 0x102);
                h.music_1fe5b8(0);
                h.clock_scale(0.0);
            }
            9 => {
                h.screen_msg(self.scr_b, 0x339);
                h.screen_msg(self.scr_a, 0x35c);
                h.screen_214300(self.scr_a, 0);
            }
            10 => {
                self.cur = None;
                self.f_c0 = -1;
                self.f_bc = -1;
                h.screen_close(self.scr_b);
                h.screen_close(self.scr_a);
                self.FUN_00201f00(h);
                h.game_msg(0xb);
            }
            -1 => {
                if prev != 10 {
                    h.game_msg(0xb);
                    if h.screen_state(self.scr_a) != 0 {
                        h.screen_off(self.scr_a);
                    }
                    if h.screen_state(self.scr_b) != 0 {
                        h.screen_off(self.scr_b);
                    }
                }
            }
            _ => {}
        }
    }

    /// 0x00202d38 (per frame)
    pub fn FUN_00202d38(&mut self, h: &mut dyn ReconHost, cm: &Cm) {
        if self.scr_a == 0 || self.scr_b == 0 {
            return;
        }
        if self.refresh != 0 {
            self.refresh = 0;
            h.bn_19cce8();
        }
        if self.photo_t + 5.0 < h.clock() {
            h.elem_hide(self.scr_a, 0x35b);
        }
        if (2..8).contains(&self.state) {
            self.FUN_00202540(h, cm);
            self.FUN_002020c8(h, cm);
            if !self.FUN_002022d8() {
                self.FUN_00202768(h, cm, 1);
            }
        }
        if let Some(t) = self.cur {
            // screen centre / size of the target: 640 x 492.8 pixels
            let px = |p: V3| {
                let s = FUN_0014f308(cm, p);
                [s[0] * 320.0 + 320.0, s[1] * -246.40001 + 246.40001]
            };
            let pos = self.targets[t].pos;
            let up = [cm.mat[1][0], cm.mat[1][1], cm.mat[1][2]];
            let a = px(pos);
            let b = px(add(pos, scale(up, self.targets[t].r_e4)));
            let d1 = ((a[0] - self.center[0]).powi(2) + (a[1] - self.center[1]).powi(2)).sqrt();
            let d2 = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt();
            let z = d2 * 0.008333334;
            let c = (1.0 - (d1 - 128.0) * 0.0078125).min(1.0);
            self.small = 1;
            self.zoom = z;
            self.centered = if 0.0 <= c { c } else { 0.0 };
            if 1.0 < z {
                self.small = 0;
                self.zoom = 2.0 - z;
            }
            if !(0.0 < self.zoom) {
                self.zoom = 0.0;
            }
        }
        let cur = self.cur.map(|t| self.targets[t].clone());
        match self.state {
            0 => {
                if h.elem_playing(self.scr_a, 0x314) {
                    self.FUN_00202768(h, cm, 1);
                }
            }
            1 => {
                self.FUN_002020c8(h, cm);
                if self.FUN_002022d8() {
                    self.FUN_00202768(h, cm, 2);
                }
            }
            2 => {
                if let Some(t) = &cur {
                    if t.vis_108 != 2 {
                        self.FUN_00202768(h, cm, 4);
                    }
                }
            }
            3 => match &cur {
                None => self.FUN_00202768(h, cm, 2),
                Some(t) if t.vis_108 != 3 => self.FUN_00202768(h, cm, 4),
                _ => {}
            },
            4 => match &cur {
                None => self.FUN_00202768(h, cm, 2),
                Some(t) if t.vis_108 == 3 => self.FUN_00202768(h, cm, 3),
                Some(_) => {
                    if 1.0 <= self.centered {
                        let s = if self.small != 0 { 5 } else { 6 };
                        self.FUN_00202768(h, cm, s);
                    }
                }
            },
            5 | 6 => match &cur {
                None => self.FUN_00202768(h, cm, 2),
                Some(t) => {
                    if self.centered < 1.0 {
                        self.FUN_00202768(h, cm, 4);
                    } else if 1.0 - t.tol_e8 <= self.zoom {
                        self.FUN_00202768(h, cm, 7);
                    } else {
                        h.elem_scrub(self.scr_b, 0x35a, self.zoom);
                    }
                }
            },
            7 => match &cur {
                None => self.FUN_00202768(h, cm, 2),
                Some(t) => {
                    if self.centered < 1.0 || self.zoom < 1.0 - t.tol_e8 {
                        let s = if self.small != 0 { 5 } else { 6 };
                        self.FUN_00202768(h, cm, s);
                    }
                }
            },
            8 => {
                if !h.elem_playing(self.scr_a, 0x358) {
                    let mut shot = false;
                    let framed = cur.as_ref().is_some_and(|t| t.vis_108 == 1 && 1.0 <= self.centered && 1.0 - t.tol_e8 <= self.zoom);
                    if framed {
                        let t = cur.as_ref().unwrap();
                        if t.f_10c == 0 {
                            self.FUN_00202768(h, cm, 9);
                        } else {
                            self.f_c0 = t.f_110;
                            shot = true;
                            self.FUN_00202768(h, cm, 7);
                        }
                    } else {
                        let (off, s) = if cur.as_ref().is_some_and(|t| t.vis_108 != 1) {
                            (0x110, 3)
                        } else if 1.0 <= self.centered && self.small != 0 {
                            (0x114, 5)
                        } else if 1.0 <= self.centered && self.small == 0 {
                            (0x118, 6)
                        } else if cur.is_some() {
                            (0x11c, 4)
                        } else {
                            (0x120, 2)
                        };
                        self.f_c0 = h.screen_word(self.scr_a, off);
                        shot = true;
                        self.FUN_00202768(h, cm, s);
                    }
                    if shot {
                        h.elem_show(self.scr_a, 0x35b);
                        self.photo_t = h.clock();
                    }
                }
            }
            9 => {
                if !h.elem_playing(self.scr_a, 0x35c) {
                    self.FUN_00202768(h, cm, 2);
                    let mut all = true;
                    for i in 0..self.targets.len() {
                        let done = self.FUN_00203758(h, i);
                        let open = matches!(self.targets[i].job, None | Some(1));
                        if !done && open {
                            all = false;
                            break;
                        }
                    }
                    if all {
                        self.refresh = 1;
                    }
                }
            }
            10 => {
                if !h.elem_playing(self.scr_a, 0x314) {
                    self.FUN_00202768(h, cm, -1);
                }
            }
            _ => {}
        }
        // 0x20348c: X / triangle takes the photo (not while looking through an object)
        if h.bn_mode() != 1 && 1 < self.state {
            let mask = if self.state < 7 {
                10
            } else if self.state == 7 {
                0xfa
            } else {
                0
            };
            if mask != 0 && !h.revoke() && h.joy_take_pressed(mask) {
                self.FUN_00202768(h, cm, 8);
            }
        }
    }
}
