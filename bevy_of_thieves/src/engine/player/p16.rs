//! WP-P16: jts 0xd / 0xe / 0x11 and the cane swing sounds.
//!
//! 1:1: 0x18a8c8 cases 0xd (ASEG done -> FUN_00195a00(0xb); on ground with normal z > 0.7 and 0.4 s in the
//! state -> SetJtJts(0, -1)), 0xe (ASEG done -> SetJtJts(0, -1); on steep-free ground after 0.4 s the ASEG is
//! seeked forward to its event 0x16d, FUN_0012f990 / FUN_00130d30 mode 0) and 0x11 (stay while +0x1d30 is set
//! and t - +0x18cc <= DAT_002bf360 = 5.0, else SetJtJts(0, -1)); 0x18c250 enter 0xd (FUN_0018e540(-1),
//! +0x190..+0x198 = 0, FUN_0018f740, ASEG jt+0x1a80, +0x19b0 = t + DAT_002bf3e4 = 0.25), enter 0xe
//! (FUN_0018e540(-1), +0x190..+0x198 = 0, ASEG jt+0x1a84), enter 0x11 (ASEG jt+0x1d34 mode 9, FUN_001619a0(
//! 0x2deb30, 0x37, jt)), exit 0x11 (+0x1d30 = 0, FUN_001619a0(0x2deb30, 0x38, jt)); the FUN_0018e540 sounds
//! (leaving jtcs 1: jt+0x18c4 target +0x130 = class 0x13 prop 1014; entering jtcs 0: 0xf; jtcs 3..7: 0xb;
//! not when chained) as FUN_001fbb90 + FUN_001fb2f8(e, 0) + FUN_001fb3f0(e, FUN_001a62e8(jt)), fire-and-forget.
//! REPLACED: FUN_00131968(jt+0x1030) "state ASEG done" = state time >= ASEG length (time = t - +0x18cc plus
//! the 0x16d seek; a missing ASEG counts as done); the seek moves this clock, not the drawn pose. Sounds and
//! job events are queued on the JT and drained by the player plugin into `EmitterSystem` / `JobMgr`.
//! UNPORTED: FUN_0018f740 (aim blend, recorded), exit 0xd +0x18c0 = 0 (field not modelled), the jts 0 exit
//! FUN_0018e540 chain (jtcs 6/7/0xc/0xd -> 4/5/0xe/0xf), who enters jts 0xd / 0xe (0x1e1e30 msg 0x1c of
//! class 0x35, 0x195a00 paths not traced).
#![allow(non_snake_case)]
use super::{Jt, JtAnim, JTS_STAND, JTBS_NIL};
use crate::engine::audio::emitter::{fuzzy_eq, EmitterId, EmitterSystem, EPS};
use crate::engine::physics::geom::V3;

/// DAT_002bf360: jts 0x11 maximum time while +0x1d30 holds.
pub const DAT_002BF360: f32 = 5.0;
/// DAT_002bf3e4: jts 0xd +0x19b0 delay.
pub const DAT_002BF3E4: f32 = 0.25;
/// jts 0xd / 0xe / 0x11 state ASEG members.
pub const ASEG_D: u32 = 0x1a80;
pub const ASEG_E: u32 = 0x1a84;
pub const ASEG_11: u32 = 0x1d34;
/// jts 0xe ASEG event (kind 0xb) the landing seeks to.
pub const EV_16D: i32 = 0x16d;

#[derive(Clone, Debug, PartialEq)]
pub struct JtP16 {
    /// jt+0x18c4 target +0x130 sound (set with the swing target, -1 none)
    pub snd_18c4: i32,
    /// state ASEG seek offset (0x130d30) on top of t - +0x18cc
    pub t_off: f32,
    /// FUN_001fbb90 sounds (id, FUN_001a62e8 position) for the plugin
    pub sfx: Vec<(i32, V3)>,
    /// FUN_001619a0(0x2deb30, k, jt) job events for the plugin
    pub jobs: Vec<i32>,
    /// WIRE-B3: (state ASEG, its time) up to which FUN_00131290 has dispatched events (None = state entered)
    pub t_ev: Option<(u32, f32)>,
}
impl Default for JtP16 {
    fn default() -> Self {
        Self { snd_18c4: -1, t_off: 0.0, sfx: vec![], jobs: vec![], t_ev: None }
    }
}

impl Jt {
    /// FUN_0018e540 sound tail: FUN_001fbb90(id) + FUN_001fb2f8(e, 0) + FUN_001fb3f0(e, FUN_001a62e8(jt)).
    pub fn FUN_0018e540_sound(&mut self, snd: i32) {
        if snd != -1 {
            self.p16.sfx.push((snd, self.phys.pos));
        }
    }

    /// State ASEG time (REPLACED jt+0x1030 +0x1c).
    pub fn p16_aseg_t(&self) -> f32 {
        self.t - self.p14.t_18cc + self.p16.t_off
    }

    /// REPLACED FUN_00131968(jt+0x1030): state ASEG `field` played to its end (or not loaded).
    pub fn p16_aseg_done(&self, field: u32) -> bool {
        self.cane.data.asegs.get(&field).map_or(true, |a| a.len <= self.p16_aseg_t())
    }

    /// 0x0018c250 enter actions of jts 0xd / 0xe / 0x11.
    pub fn FUN_0018c250_enter_p16(&mut self, new: i32, anim: &mut dyn JtAnim) {
        self.p16.t_off = 0.0;
        self.p16.t_ev = None;
        match new {
            0xd | 0xe => {
                self.FUN_0018e540(-1, anim);
                self.v_target = [0.0; 3]; // +0x190..+0x198
                if new == 0xd {
                    self.FUN_0018f740(); // WP-P18
                    self.cane.t_19b0 = self.t + DAT_002BF3E4;
                    anim.play_aseg(ASEG_D, 0xd, -1);
                } else {
                    anim.play_aseg(ASEG_E, 0xe, -1);
                }
            }
            0x11 => {
                anim.play_aseg(ASEG_11, 0x11, -1); // mode 9
                self.p16.jobs.push(0x37);
            }
            _ => {}
        }
    }

    /// WIRE-B3: FUN_00131290 over the jt+0x1030 state ASEG of jts 0xd / 0xe / 0x11: kind-0xb events in
    /// (last, now] (events at time 0 included on the first frame; a forward seek FUN_00130d30 mode 0 passes the
    /// skipped events) -> FUN_00188f20 msg 9 (e.g. jts 0xe 0x16d -> `calls` (0x188f20, 0x16d), heard by guards).
    pub fn p16_events(&mut self, anim: &mut dyn JtAnim) {
        let f = match self.jts {
            0xd => ASEG_D,
            0xe => ASEG_E,
            0x11 => ASEG_11,
            _ => {
                self.p16.t_ev = None;
                return;
            }
        };
        let t1 = self.p16_aseg_t();
        let t0 = self.p16.t_ev.filter(|x| x.0 == f).map_or(-1.0, |x| x.1);
        self.p16.t_ev = Some((f, t1));
        let Some(a) = self.cane.data.asegs.get(&f).cloned() else { return };
        let jts = self.jts;
        for (t, k, x) in a.ev {
            if !(t0 < t && t <= t1 && self.jts == jts) {
                continue;
            }
            // FUN_00131290: kind 0xb -> msg 9; POOL1 R65: kinds 0xf / 0x10 (FUN_002104a0 -> FUN_001f6ac8 /
            // FUN_001f6b48) turn the SO geom with that id on / off, as for the cane ASEGs (air attack 0x1a80: part 0x14d)
            match k {
                0xb => self.FUN_00188f20_msg9(x, anim),
                0xf => self.cane.geom_set(x as u32, true),
                0x10 => self.cane.geom_set(x as u32, false),
                _ => {}
            }
        }
    }

    /// 0x0018c250 exit actions of jts 0xd / 0x11.
    pub fn FUN_0018c250_exit_p16(&mut self, old: i32) {
        if old == 0x11 {
            self.p13.f_1d30 = 0;
            self.p16.jobs.push(0x38);
        }
        if old == 0xd {
            self.p18.aim_18c0 = None; // +0x18c0 = 0 (WP-P18)
        }
    }

    /// 0x0018a8c8 case 0xd.
    pub fn FUN_0018a8c8_d(&mut self, anim: &mut dyn JtAnim) {
        if self.p16_aseg_done(ASEG_D) {
            self.FUN_00195a00(0xb, 0, anim);
        } else if self.ground_nz.is_some_and(|z| 0.7 < z) && 0.4 < self.t - self.p14.t_18cc {
            self.set_jts(JTS_STAND, JTBS_NIL, anim);
        }
    }

    /// 0x0018a8c8 case 0xe.
    pub fn FUN_0018a8c8_e(&mut self, anim: &mut dyn JtAnim) {
        if self.p16_aseg_done(ASEG_E) {
            self.set_jts(JTS_STAND, JTBS_NIL, anim);
        } else if self.ground_nz.is_some_and(|z| 0.7 < z) && 0.4 < self.t - self.p14.t_18cc {
            // FUN_0012f990(*(jt+0x1a84), 0x16d): first kind-0xb event 0x16d time, 0 when absent
            let tev = self.cane.data.asegs.get(&ASEG_E).and_then(|a| a.ev.iter().find(|e| e.1 == 0xb && e.2 == EV_16D)).map_or(0.0, |e| e.0);
            let ta = self.p16_aseg_t();
            if ta < tev {
                self.p16.t_off += tev - ta; // FUN_00130d30(tev, rate, aseg, 0)
            }
        }
    }

    /// 0x0018a8c8 case 0x11.
    pub fn FUN_0018a8c8_11(&mut self, anim: &mut dyn JtAnim) {
        if !(self.p13.f_1d30 != 0 && self.t - self.p14.t_18cc <= DAT_002BF360) {
            self.set_jts(JTS_STAND, JTBS_NIL, anim);
        }
    }
}

/// FUN_001fb3f0(e, pos): store the position; at the origin only a 3D emitter (+0x1c == 0) clears 0x200,
/// anywhere else 0x200 is set.
pub fn FUN_001fb3f0(es: &mut EmitterSystem, id: EmitterId, pos: V3) {
    let Some(e) = es.get_mut(id) else { return };
    e.pos = pos;
    let unpositioned = e.unpositioned;
    let origin = (0..3).all(|i| fuzzy_eq(0.0, pos[i], EPS));
    if !origin {
        es.set_flags_1fb658(id, 0x200, 0x200);
    } else if unpositioned == 0 {
        es.set_flags_1fb658(id, 0x200, 0);
    }
}

/// Plugin side: queued swing sounds -> emitters (no reference kept), job events -> FUN_001619a0.
pub fn drain(jt: &mut Jt, es: Option<&mut EmitterSystem>, jobs: &mut crate::engine::jobs::JobMgr) {
    let sfx = std::mem::take(&mut jt.p16.sfx);
    if let Some(es) = es {
        for (snd, pos) in sfx {
            let id = es.new_emitter_1fbb90(snd);
            es.set_unpositioned_1fb2f8(id, 0);
            FUN_001fb3f0(es, id, pos);
        }
    }
    for k in std::mem::take(&mut jt.p16.jobs) {
        jobs.FUN_001619a0(k, 0);
    }
}
