//! WIRE-D: guards hearing Sly. Sly's noise (FUN_00188f20 msg 9 event 0x16d -> FUN_00156b98(world, 0, jt pos,
//! jt) -> FUN_00156698 / FUN_00156548: a stimulus record in DAT_003e148c of the world's kind 0, class 117
//! id 0x2c2) becomes a `Stimulus` every guard polls (FUN_0020a408 through slot 88 FUN_00208be0, whose sound
//! branch is FUN_00156918 here).
//!
//! REPLACED: the pool lifetime. The record is ended by its own slot 0x1c timer (not ported); here it lives
//! `LIFE` seconds (the FUN_00156898 fade cap DAT_002bdce0).
use super::slots::Stimulus;
use bevy_ecs::prelude::*;

/// DAT_002bdce0 (fade cap), DAT_002bdcf0 / f4 / f8 (fade polynomial), read from SCUS_973.16.
const FADE_CAP: f32 = 1.0;
const FADE_C: [f32; 3] = [0.15, 1.1, -0.25];
const LIFE: f32 = FADE_CAP;
/// FUN_00188f20 msg 9 id 0x16d (cane swing noise)
pub const EV_NOISE: i32 = 0x16d;

/// The stimulus kind (class 117, C2ce2f8): +0x50 mode, +0x54 radius, +0x58 z range, +0x60 fade, +0x64 level.
#[derive(Clone, Debug, PartialEq)]
pub struct StimKind {
    pub mode: i32,
    pub radius: f32,
    pub z: [f32; 2],
    pub fade: f32,
    pub level: i32,
}

impl StimKind {
    /// world +0x4020 (FUN_001569b8 id 0x2c2: +0x50 = 0 over the slot-6 defaults FUN_001564e8:
    /// 1500, (-200, 500), 0.25)
    pub fn world_sound() -> Self {
        StimKind { mode: 0, radius: 1500.0, z: [-200.0, 500.0], fade: 0.25, level: 0 }
    }

    /// FUN_00156898(rec): the radius grown over the fade (`born` = rec +0x4c)
    pub fn ee_sub_156898(&self, born: f32, now: f32) -> f32 {
        if 0.0 < self.fade {
            let t = ((now - born) / self.fade.min(FADE_CAP)).min(1.0);
            return self.radius * (FADE_C[0] + t * (FADE_C[1] + t * FADE_C[2]));
        }
        self.radius
    }

    /// FUN_00156918(rec, pos): d = pos - rec +0x20; d.z in the kind's range (FUN_0011d910) and
    /// hypot(d.x, d.y) <= FUN_00156898(rec)
    pub fn ee_sub_156918(&self, s: &Stimulus, pos: [f32; 3], now: f32) -> bool {
        let d = [pos[0] - s.pos[0], pos[1] - s.pos[1], pos[2] - s.pos[2]];
        if !(self.z[0] <= d[2] && d[2] <= self.z[1]) {
            return false;
        }
        (d[0] * d[0] + d[1] * d[1]).sqrt() <= self.ee_sub_156898(s.f_4c, now)
    }

    /// FUN_00156548(kind, pos, src): the record (+0x20 / +0x30 pos, +0x40 src, +0x4c born, +0x54 level)
    pub fn ee_sub_156548(&self, pos: [f32; 3], src: u32, now: f32) -> Stimulus {
        Stimulus { src, subject: 0, level: self.level, pos, pos30: pos, f_4c: now, mode: self.mode, kind: Some(self.clone()) }
    }
}

/// The DAT_003e148c records made by Sly (live pool).
#[derive(Resource, Default, Debug)]
pub struct StimPool {
    pub recs: Vec<Stimulus>,
    /// cursor into `Jt::calls` (msg-9 events already turned into records)
    seen: usize,
}

/// Bevy system (before `tick_classes`): Sly's new msg-9 0x16d events -> kind-0 records at his position;
/// expired records dropped; every guard's stimulus list = the pool.
pub fn sly_noise(
    q: Query<&crate::engine::player::plugin::Player>,
    classes: Option<ResMut<crate::engine::classes::plugin::Classes>>,
    mut pool: ResMut<StimPool>,
) {
    let Some(mut c) = classes else { return };
    let now = c.set.g.t;
    if let Some(p) = q.iter().next() {
        let calls = &p.jt.calls;
        if calls.len() < pool.seen {
            pool.seen = 0;
        }
        let pos = p.jt.phys.pos;
        let n = calls[pool.seen..].iter().filter(|c| **c == (0x0018_8f20, EV_NOISE)).count();
        pool.seen = calls.len();
        for _ in 0..n {
            // FUN_00156b98(0x3dd490, 0, FUN_001a62e8(jt), jt): src = Sly (no class object -> 0)
            pool.recs.push(StimKind::world_sound().ee_sub_156548(pos, 0, now));
        }
    }
    // class code's FUN_00156b98 calls (clue bottles, guard alarms, ...): src = the object
    for (o, pos) in std::mem::take(&mut c.host.noises) {
        pool.recs.push(StimKind::world_sound().ee_sub_156548(pos, o, now));
    }
    pool.recs.retain(|s| now - s.f_4c <= LIFE);
    for (_, x) in c.set.k6.iter_mut() {
        if let crate::engine::classes::Inst::Guard(g) = x {
            g.stimuli.clone_from(&pool.recs);
        }
    }
}
