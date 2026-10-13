//! WP-I5: class-15 camera tracks (C2db6b8, load FUN_00220fd0 = `B4Load::L220fd0`) as region cameras.
//!
//! Every frame slot 15 FUN_00221388 defers FUN_00221298 for a track with +0x13c != 0: the player
//! (FUN_001c4a48) is moved into the track's frame (FUN_001a6f00(0, track, FUN_001a62e8(p))) and tested
//! against the +0x108 BSP (FUN_00118fb0); FUN_00221178 then pushes the C2db858 follow policy at cpp
//! +0x128 + 2 with pv = the track (state 1) or revokes it by pv (state 0, RevokeCmPolicy grf 8). The follow
//! (camera/follow.rs) reads the track's +0x114 params entry while it is on top of the CM priority stack;
//! a higher track wins, leaving restores the next entry (cm.rs stack + FUN_0014d8c0 blend).
//! Ported 1:1: FUN_00221298, FUN_00221178 (camera half), FUN_00221388's +0x13c gate.
//! REPLACED: the focus is always Sly (isA 0x34 true); jt+0x2dc (jtbs) is taken as -1 (PlayerFocus does not
//! carry it), so the state is re-evaluated every frame; the vtable +0x1c messages 5 / 6 of FUN_00221178
//! (script listeners) are counted in `msgs`, not dispatched. classes/c2db6b8.rs holds the class-side port
//! whose ClassHost camera hooks are not wired to the CM; this module drives the CM directly.
#![allow(non_snake_case)]

use super::follow::{follow_track, FollowTrack};
use crate::engine::gfx::attach::{ee_sub_22d6b0, BoneRef, BoneWorlds};
use crate::engine::gfx::scene::{invert, mul, Affine};
use crate::engine::physics::geom::{build_bsp, build_geom, BspC, CGeom};
use crate::engine::physics::point::FUN_00118fb0;
use crate::engine::world::loads_b4::B4Load;
use crate::engine::world::objects::{LoadData, Payload};
use crate::engine::world::props::PropValue;
use crate::engine::world::World;

/// One class-15 object with its C2db858 view.
#[derive(Clone, Debug)]
pub struct RegionTrack {
    pub obj: usize,
    pub oid: u32,
    /// pv handed to SetCmPolicy (object index + 1, like `playerdraw::glide_track`)
    pub pv: u32,
    /// world -> track frame (inverse of the track's world xform)
    pub inv: Affine,
    pub geom: CGeom,
    pub bsp: BspC,
    /// +0x124 (field 201), +0x13c (field 214, -1 = from the volume test)
    pub f_124: bool,
    pub f_13c: i32,
    /// +0x134 state, +0x138 state time
    pub f_134: i32,
    pub f_138: f32,
    pub track: FollowTrack,
}

/// A FUN_00221178 camera action.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrackCm {
    /// SetCmPolicy(cm, cpp, follow, focus, pv)
    Push { cpp: i32, pv: u32 },
    /// RevokeCmPolicy(cm, 8, -1, 0, 0, pv)
    Revoke { pv: u32 },
}

#[derive(Clone, Debug, Default)]
pub struct TrackSet {
    pub tracks: Vec<RegionTrack>,
    /// +0xe0 focus (per track in the original; one focus = Sly here)
    pub focus: u32,
    /// vtable +0x1c messages sent (5 enter, 6 leave)
    pub msgs: Vec<(u32, i32)>,
}

fn word(b: &[u8]) -> u32 {
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

fn xf(m: &Affine, p: [f32; 3]) -> [f32; 3] {
    [0, 1, 2].map(|j| p[0] * m[0][j] + p[1] * m[1][j] + p[2] * m[2][j] + m[3][j])
}

impl TrackSet {
    /// Class-15 objects of a loaded world (FUN_00220ed0 defaults: +0x13c = -1, +0x134 = 0).
    pub fn load(w: &World) -> Self {
        let mut bw = BoneWorlds::default();
        let mut tracks = Vec::new();
        for (i, o) in w.objects.iter().enumerate() {
            let Payload::Loaded(LoadData::B4(b)) = &o.payload else { continue };
            let B4Load::L220fd0 { xform, geom, bsp, props, nested } = &**b else { continue };
            let (mut f_124, mut f_13c) = (false, -1);
            for p in props {
                let PropValue::Scalar(b) = &p.value else { continue };
                match p.field {
                    201 if b.len() >= 4 => f_124 = word(b) != 0,
                    214 if b.len() >= 4 => f_13c = word(b) as i32,
                    _ => {}
                }
            }
            let local = crate::engine::gfx::attach::affine_from_bytes(&xform.xform);
            let m = match ee_sub_22d6b0(w, xform.s, o.owner, None, BoneRef::None) {
                BoneRef::Bone(ob, k) => mul(&local, &bw.get(w, ob, k)),
                _ => local,
            };
            tracks.push(RegionTrack {
                obj: i,
                oid: o.oid as u32,
                pv: i as u32 + 1,
                inv: invert(&m),
                geom: build_geom(geom),
                bsp: build_bsp(bsp),
                f_124,
                f_13c,
                f_134: 0,
                f_138: 0.0,
                track: follow_track(props, !nested.is_empty()),
            });
        }
        Self { tracks, focus: 0, msgs: Vec::new() }
    }

    /// The C2db858 view of track `pv`.
    pub fn get(&self, pv: u32) -> Option<&RegionTrack> {
        self.tracks.iter().find(|t| t.pv == pv)
    }
    pub fn get_mut(&mut self, pv: u32) -> Option<&mut RegionTrack> {
        self.tracks.iter_mut().find(|t| t.pv == pv)
    }

    /// FUN_00221178(track, s): state change -> CM action.
    pub fn FUN_00221178(&mut self, k: usize, s: i32, t: f32, out: &mut Vec<TrackCm>) {
        let tr = &mut self.tracks[k];
        if s == tr.f_134 {
            return;
        }
        match s {
            0 => {
                out.push(TrackCm::Revoke { pv: tr.pv });
                self.msgs.push((tr.pv, 6));
            }
            1 => {
                out.push(TrackCm::Push { cpp: tr.track.f_128 + 2, pv: tr.pv });
                self.msgs.push((tr.pv, 5));
            }
            _ => {}
        }
        tr.f_134 = s;
        tr.f_138 = t;
    }

    /// FUN_00221298 for track `k` (`focus` = FUN_001c4a48 handle, `pos` = FUN_001a62e8(focus), `cut` =
    /// DAT_002de240 != 0).
    pub fn FUN_00221298(&mut self, k: usize, focus: u32, pos: [f32; 3], cut: bool, t: f32, out: &mut Vec<TrackCm>) {
        if focus == 0 {
            return;
        }
        if focus != self.focus {
            self.FUN_00221178(k, 0, t, out);
        }
        let tr = &self.tracks[k];
        let mut s = tr.f_13c;
        if s == -1 {
            let l = xf(&tr.inv, pos);
            s = (!tr.bsp.nodes.is_empty() && FUN_00118fb0(&tr.geom, &tr.bsp, 0, 0x7fffffff, l).is_some()) as i32;
        }
        // !isA(focus, 0x34) == false, focus +0x2dc == -1 (REPLACED, see module doc)
        let jtbs_free = true;
        if tr.f_124 || jtbs_free || cut || tr.f_13c != -1 {
            self.FUN_00221178(k, s, t, out);
        }
    }

    /// One frame of every track's slot 15 (FUN_00221388: deferred only when +0x13c != 0).
    pub fn update(&mut self, focus: u32, pos: [f32; 3], cut: bool, t: f32) -> Vec<TrackCm> {
        let mut out = Vec::new();
        for k in 0..self.tracks.len() {
            if self.tracks[k].f_13c != 0 {
                self.FUN_00221298(k, focus, pos, cut, t, &mut out);
            }
        }
        self.focus = focus;
        out
    }
}
