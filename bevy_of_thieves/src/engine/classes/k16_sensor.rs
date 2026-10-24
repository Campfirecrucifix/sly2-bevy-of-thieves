//! HEIST: standalone sensor objects (world class 16, vtable 0x2cdba0, size 0x180) - the same class the guards
//! embed at +0xf64 (`guard::sensor::Sensor`), placed on its own by level entries such as the EP3
//! `spotlight_shooter_thick` (entry 129: sensor 27033, volume 8566, SM 27037, launcher 9070).
//! Fields (table 0x2ac880): 345 +0xe8 inner volume oid, 346 +0xec outer volume oid, 348 +0xe0 eye owner,
//! 349 method FUN_00153f18 (add a static entry, e.g. oids 7 / 9 / 8 = the player characters).
//! Slot 15 FUN_00153db0 = FUN_001a5e28 + FUN_00153de0: every entry's vision state (FUN_001541d8), a change
//! goes through FUN_00154080 -> script message 0x1c {entry object, old, new} to the sensor itself (the
//! spotlight's handler forwards it to its owner's 1379 / 1380 callbacks).
use super::guard::sensor::{Sensor, Tgt, Vol, See};
use super::guard::PLAYER_HANDLE;
use super::{ClassHost, ClassSet};
use crate::engine::game::fields::object_props;
use crate::engine::gfx::attach::BoneWorlds;
use crate::engine::gfx::scene::invert;
use crate::engine::physics::geom::{build_bsp, build_geom};
use crate::engine::world::objects::{LoadData, Payload};
use crate::engine::world::World;

pub const CLASS_SENSOR: u16 = 16;

#[derive(Clone, Debug, Default)]
pub struct Sensor16 {
    pub sensor: Sensor,
    /// static entry oids of method 349 (resolved at post-load)
    pub add_oids: Vec<i32>,
    /// FUN_001a62e8(s) at load (REPLACED: the sensor's world position is not re-read per frame)
    pub pos: [f32; 3],
}

fn w0(v: &crate::engine::world::props::PropValue) -> Option<i32> {
    use crate::engine::world::props::PropValue;
    match v {
        PropValue::Scalar(b) if b.len() >= 4 => Some(i32::from_le_bytes([b[0], b[1], b[2], b[3]])),
        _ => None,
    }
}

fn is_under(w: &World, mut k: usize, root: usize) -> bool {
    for _ in 0..64 {
        if k == root {
            return true;
        }
        match w.objects[k].owner {
            Some(o) => k = o,
            None => return false,
        }
    }
    false
}

/// FUN_00227bb8: the class-22 VOL `oid` (nearest: under the sensor's owner) compiled in world space
fn vol(w: &World, bw: &mut BoneWorlds, i: usize, oid: i32) -> Option<Vol> {
    if oid == 0 {
        return None;
    }
    let root = w.objects[i].owner.unwrap_or(i);
    let all = w.find_oid(oid);
    let mut k = all.iter().copied().find(|&k| is_under(w, k, root)).or_else(|| all.first().copied())?;
    // cloned children carry their template's solid
    while let (Payload::Clone, Some(c)) = (&w.objects[k].payload, w.objects[k].clone_of) {
        k = c;
    }
    let solid = match &w.objects[k].payload {
        Payload::Loaded(LoadData::Solid(s)) => s,
        Payload::Loaded(LoadData::L2282b0 { solid, .. }) => solid,
        _ => return None,
    };
    let mw = bw.xfm_world(w, k)?;
    Some(Vol::new(k, invert(&mw), build_geom(&solid.geom), build_bsp(&solid.bsp)))
}

impl ClassSet {
    /// construct (FUN_00153d58 init) + load fields 345 / 346 / 349 of every class-16 object
    pub(super) fn build_sensor16(&mut self, w: &World) {
        self.launcher_owner = w.objects.iter().enumerate().filter(|(_, o)| o.class == 112).filter_map(|(i, o)| Some((i, crate::engine::game::fields::ee_sub_1a62e8(w, o.owner?)))).collect();
        if std::env::var("SLY2_K16").is_ok_and(|v| v == "0") {
            return;
        }
        let mut bw = BoneWorlds::default();
        let player = crate::engine::game::spawn::player_object(w);
        for (i, o) in w.objects.iter().enumerate() {
            if o.class as u16 != CLASS_SENSOR || o.clone_of.is_some() {
                continue;
            }
            let props = object_props(o).unwrap_or(&[]);
            let mut s = Sensor16::default();
            for p in props {
                match (p.field, w0(&p.value)) {
                    (345, Some(v)) => s.sensor.f_e8 = vol(w, &mut bw, i, v),
                    (346, Some(v)) => s.sensor.f_ec = vol(w, &mut bw, i, v),
                    (349, Some(v)) => s.add_oids.push(v),
                    _ => {}
                }
            }
            // a sensor without volumes never sees anything (FUN_00227c90 of a null volume)
            if s.sensor.f_e8.is_none() {
                continue;
            }
            // +0xe0 eye owner stays 0 (field 348 is set by the level script) -> FUN_00153e60 = FUN_001a62e8(s)
            s.pos = bw.xfm_world(w, i).map_or([0.0; 3], |m| [m[3][0], m[3][1], m[3][2]]);
            // FUN_00153f18 per method-349 oid: a player character is the player handle (the controlled JT)
            for oid in std::mem::take(&mut s.add_oids) {
                let Some(&k) = w.find_oid(oid).first() else { continue };
                let hdl = if Some(k) == player { PLAYER_HANDLE } else { k as u32 + 1 };
                s.sensor.ee_sub_153f18(hdl, self.g.t);
            }
            self.sensor16.push((i, s));
        }
    }

    /// slot 15 FUN_00153db0: FUN_00153de0 over the entries; state changes -> script message 0x1c
    pub(super) fn frame_sensor16(&mut self, h: &mut dyn ClassHost) {
        if self.sensor16.is_empty() {
            return;
        }
        let players = self.g.k6_players.clone();
        let ptgt = self.g.k6_player_tgt;
        let tgt = |o: u32| -> Option<Tgt> {
            let p = players.iter().find(|p| p.0 == o)?;
            if let Some(t) = ptgt.filter(|_| o == PLAYER_HANDLE) {
                return Some(Tgt { pos: p.1, ..t });
            }
            Some(Tgt { pos: p.1, c33: true, c34: true, active: true, ..Default::default() })
        };
        let now = self.g.t;
        let id = |p: [f32; 3]| p;
        for (i, s) in &mut self.sensor16 {
            let hdl = *i as u32 + 1;
            let pos = s.pos;
            // FUN_00153e60: the +0xe0 owner's eye (the sensor itself: its position)
            let see = See { now, eye: pos, pos, to_rest: &id, tgt: &tgt, player_c46: false };
            for k in 0..s.sensor.entries.len() {
                let e = s.sensor.entries[k];
                let n = s.sensor.ee_sub_1541d8(*i, e.obj, e.state, &see, h);
                // FUN_00154080 (the FUN_001619a0 alarm tail needs an is-a 0x33 owner: never a class-16 object)
                if n == e.state {
                    continue;
                }
                // (*(vt+0x1c))(s, 0x1c, 0, 3, {&entry, &entry.state, &new}) -> FUN_001331f8
                h.k27_script_msg(hdl, 0x1c, &[e.obj, e.state as u32, n as u32]);
                let e = &mut s.sensor.entries[k];
                if e.state == 0 {
                    e.t_first = now;
                }
                e.state = n;
                e.t = now;
            }
        }
    }
}
