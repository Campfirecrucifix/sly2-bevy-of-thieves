//! WP-K4: construction / feed / per-phase dispatch of the K4 classes (`ClassSet::k4`), plus prop decoders.
use super::*;
use crate::engine::game::fields::object_props;

/// K4 world class id -> vtable (kept in `ClassSet::k4`).
pub const PORTED_K4: &[(ClassId, u32)] = &[(105, 0x2db0d0), (15, 0x2db6b8), (132, 0x2cf378), (7, 0x2d5dc8), (13, 0x2d6fe8), (11, 0x2d56f8), (113, 0x2cf880), (129, 0x2dad90), (116, 0x2cf058), (17, 0x2d52b0), (3, 0x2dd090), (117, 0x2ce2f8), (39, 0x2d9dd0), (44, 0x2ca9f0)];

/// Scalar prop word (kind 0 = 1 byte bool, others little-endian u32).
pub fn word(b: &[u8]) -> u32 {
    match b.len() {
        0 => 0,
        1..=3 => b[0] as u32,
        _ => u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
    }
}
pub fn f32_of(b: &[u8]) -> f32 {
    f32::from_bits(word(b))
}
/// kind 2 vec3 -> vec4 (w = 0)
pub fn vec3_of(b: &[u8]) -> [f32; 4] {
    let g = |i: usize| b.get(i * 4..i * 4 + 4).map(f32_of).unwrap_or(0.0);
    [g(0), g(1), g(2), 0.0]
}

impl ClassSet {
    /// K4: construct + init (slot 6) + feed (load props) the `PORTED_K4` classes.
    pub(super) fn build_k4(&mut self, w: &World, h: &mut dyn ClassHost) {
        for (i, o) in w.objects.iter().enumerate() {
            let src = o.clone_of.unwrap_or(i);
            let props = w.objects.get(src).and_then(object_props).unwrap_or(&[]);
            let inst = match o.class {
                105 => {
                    let mut c = Box::<c2db0d0::C2db0d0>::default();
                    c.ee_sub_21bb88(i, h);
                    c.feed(i, props, h);
                    Inst::C2db0d0(c)
                }
                15 => {
                    let mut c = c2db6b8::C2db6b8::default();
                    c.ee_sub_220ed0(i, h);
                    c.feed(props);
                    if o.clone_of.is_some() {
                        // slot 40 (FUN_0013e510 -> vtable +0x144) of a cloned instance
                        c.ee_sub_221140(i, o.owner.map(|x| x as u32 + 1).unwrap_or(0), h);
                    }
                    Inst::C2db6b8(c)
                }
                132 => {
                    let mut c = Box::<c2cf378::C2cf378>::default();
                    c.ee_sub_162448(i, h);
                    c.f_50 = o.owner.map(|x| x as u32 + 1).unwrap_or(0);
                    c.feed(i, props, h);
                    Inst::C2cf378(c)
                }
                7 => {
                    let mut c = c2d5dc8::C2d5dc8::default();
                    c.ee_sub_1ca788(i, h);
                    c.feed(props);
                    if o.clone_of.is_some() {
                        let id = c.f_f0_id;
                        c.ee_sub_1ca7b8(i, id, h);
                    }
                    Inst::C2d5dc8(c)
                }
                13 => {
                    let mut c = c2d6fe8::C2d6fe8::default();
                    c.ee_sub_1cb908(i, h);
                    c.feed(props);
                    if o.clone_of.is_some() {
                        c.ee_sub_1cba20(i, h);
                    }
                    Inst::C2d6fe8(c)
                }
                11 => {
                    let mut c = c2d56f8::C2d56f8::default();
                    c.ee_sub_1c3390(i, h);
                    c.feed(props);
                    if o.clone_of.is_some() {
                        c.ee_sub_1c34b8(i, h);
                    }
                    Inst::C2d56f8(c)
                }
                113 => {
                    let mut c = c2cf880::C2cf880::default();
                    c.ee_sub_1709f8(i, h);
                    c.feed(props);
                    Inst::C2cf880(c)
                }
                129 => {
                    let mut c = c2dad90::C2dad90::default();
                    h.base(0x1a5270, i); // slot 6 inherited (FUN_001a5270)
                    c.feed(props);
                    Inst::C2dad90(c)
                }
                116 => {
                    let mut c = c2cf058::C2cf058::default();
                    c.ee_sub_15dc80(i, h);
                    c.feed(i, props, &self.g, h);
                    Inst::C2cf058(c)
                }
                17 => {
                    let mut c = c2d52b0::C2d52b0::default();
                    c.ee_sub_1c0378(i, h);
                    c.feed(props);
                    Inst::C2d52b0(c)
                }
                3 => {
                    h.base(0x231100, i); // slot 6 inherited (ALO init)
                    Inst::C2dd090(c2dd090::C2dd090)
                }
                117 => {
                    let mut c = c2ce2f8::C2ce2f8::default();
                    c.ee_sub_1564e8(i, h);
                    c.feed(props);
                    Inst::C2ce2f8(c)
                }
                39 => {
                    let mut c = c2d9dd0::C2d9dd0::default();
                    c.ee_sub_1ff3b8(i, h);
                    c.feed(props);
                    if o.clone_of.is_some() {
                        c.ee_sub_1ff4a8(i, h);
                    }
                    Inst::C2d9dd0(c)
                }
                44 => {
                    let mut c = c2ca9f0::C2ca9f0::default();
                    c.ee_sub_12e600(i, h);
                    c.r106_top = o.owner.is_none_or(|k| k == i);
                    c.feed(i, props, h);
                    Inst::C2ca9f0(c)
                }
                _ => continue,
            };
            self.k4.push((i, inst));
        }
    }
    /// K4 load-post (slot 14).
    pub(super) fn post_load_k4(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &mut self.k4 {
            match x {
                Inst::C2db0d0(c) => c.ee_sub_21bd18(*i, &self.g, h),
                Inst::C2cf378(c) => c.ee_sub_162780(*i, &self.g, h),
                Inst::C2d5dc8(c) => {
                    c.ee_sub_1c8030(*i, &mut self.g, h);
                    // slot 22 (add to world) right after slot 14, as K3's light registration
                    // FEEL R35: only for a spawner in the world (8 out-of-world class-6/7 spawners of rec11 012 hold none)
                    if !h.lo_removed(*i) {
                        c.ee_sub_1ca8e0(*i, h);
                    }
                }
                Inst::C2d6fe8(c) => {
                    c.ee_sub_1cb950(*i, h);
                    c.ee_sub_1cb9b0(*i, &mut self.g, h);
                }
                Inst::C2d56f8(c) => {
                    c.ee_sub_1c3458(*i, h);
                    c.ee_sub_1c33e8(*i, &mut self.g, h);
                }
                Inst::C2cf880(c) => c.ee_sub_170a28(*i, h),
                Inst::C2dad90(c) => c.ee_sub_211ec0(*i, h),
                Inst::C2ca9f0(c) => c.ee_sub_12e648(*i, h),
                Inst::C2d9dd0(c) => {
                    c.ee_sub_1ff3e8(*i, h);
                    c.ee_sub_1ff438(*i, &mut self.g, h);
                }
                _ => {}
            }
        }
    }
    /// K4 per-frame slots: 15 (update), 16 (SO forces / integrate), 18 (draw).
    pub(super) fn frame_k4(&mut self, h: &mut dyn ClassHost) {
        let more = &self.more;
        let c133 = |o: u32| c133_view(more, o);
        // CRAWL R109: every class-44 camera, owned ones too, follows Add / Remove (an Add skips a child removed on
        // its own, fields::b56_at_load; old owner-less-only = SLY2_R109_CAMTOP_ONLY=1 or SLY2_NO_R109_B56OWN=1).
        // CRAWL R110: DEFAULT (opt-out SLY2_NO_R110_CAMWORLD=1): the exit now retracts the hub select SMA (vm_state
        // get-o class-114 field 620 + set-o Sly 806 -> FUN_001c3ed8 -> FUN_001f0598 msg 3 -> 249.1 s3 Removes 27337 / 27335).
        let camworld = self.r106_camworld || std::env::var_os("SLY2_R106_CAMWORLD").is_some() || std::env::var_os("SLY2_NO_R110_CAMWORLD").is_none();
        let camall = std::env::var_os("SLY2_R109_CAMTOP_ONLY").is_none() && std::env::var_os("SLY2_NO_R109_B56OWN").is_none();
        for (i, x) in &mut self.k4 {
            // FEEL R33: slot 15 runs from the world update list only (LO +0x28 bit 60)
            if !matches!(x, Inst::C2ca9f0(_)) && h.lo_removed(*i) {
                continue;
            }
            match x {
                Inst::C2db0d0(c) => {
                    c.ee_sub_21c0d8(*i, &self.g, h);
                    c.ee_sub_21c1c8(*i, &self.g, h);
                    c.ee_sub_21c340(*i, &self.g, h);
                }
                Inst::C2db6b8(c) => c.ee_sub_221388(*i, &mut self.k4_deferred, h),
                // K30: no update slot; the camera state the 0x2de3e8 policy reads (classes/k30_cam.rs)
                Inst::C2ca9f0(c) => {
                    // CRAWL R106: script Add / Remove of a class-44 camera (LO +0x28 bit 60, `lo_removed`) runs
                    // its slot 10 FUN_0012e6a0 / slot 11 FUN_0012e6e8 (camera request 10 on 0x2de3e8 / revoke):
                    // the safehouse table camera 27335 (obj 3920; 27337 is class 4) of script 249.1 takes the view
                    // Only Add / Remove transitions after load: registering every in-world class-44 at load put
                    // the character-attached cameras (7563 under Bentley 8, 4332 under Sly 7) on top of the CM,
                    // which the original does not show (rec_ep1b: follow camera); why they stay off is open. Those
                    // flip with their character's subtree (script Add of the gang), so only owner-less cameras
                    // (`c.r106_top`, set at build) follow the transitions here.
                    // CRAWL R108 (R109: the live image's LO bits mirror the port's state, not a ground truth): after load every class-44 camera has LO +0x28 bit 60 clear
                    // (7563 / 4332 too, while their Bentley / Sly owners are in the world), i.e. they are removed on
                    // their own and an owner's script Add (249.1 s2 Remove+Add of the gang) does not re-add them
                    // (FUN_001a5800 walks only the in-world child list); the port's subtree Add flipped them in.
                    // OPT-IN SLY2_R106_CAMWORLD: after X (exit) 27335 stays the top request for 600+ frames (no
                    // script Remove reached while the port's exit walk stalls) -> the view would stay locked.
                    if (c.r106_top || camall) && camworld {
                        let inw = !h.lo_removed(*i);
                        if !c.r106_seen {
                            c.r106_seen = true;
                            c.r106_inw = inw;
                        } else if inw && !c.r106_inw {
                            c.r106_inw = true;
                            if std::env::var_os("SLY2_R82CAM").is_some() {
                                let me = *i as u32 + 1;
                                let ow = h.obj_i32(me, 0x10) as u32;
                                let ow2 = if ow != 0 { h.obj_i32(ow, 0x10) as u32 } else { 0 };
                                eprintln!("R82ADD obj {i} oid {} top {} own {ow} oid {} own2 {ow2} oid {}", h.obj_i32(me, 0x18), c.r106_top, if ow != 0 { h.obj_i32(ow, 0x18) } else { 0 }, if ow2 != 0 { h.obj_i32(ow2, 0x18) } else { 0 });
                            }
                            c.ee_sub_12e6a0(*i, h);
                        } else if !inw && c.r106_inw {
                            c.r106_inw = false;
                            c.ee_sub_12e6e8(*i, h);
                        }
                    }
                    c.k30_publish(*i, h)
                }
                Inst::C2cf378(c) => c.ee_sub_163078(*i, &mut self.g, &c133, h),
                // slot 16 inherited from class 10 (FUN_001e6340)
                Inst::C2d6fe8(c) => c.base.ee_sub_1e6340(*i, h),
                Inst::C2d56f8(c) => c.sub.base.ee_sub_1e6340(*i, h),
                Inst::C2dad90(c) => c.ee_sub_211f90(*i, &c133, h),
                // only while added (FUN_0015e1d8): slot 15, then slot 18
                Inst::C2cf058(c) if c.active => {
                    c.ee_sub_15dcd8(*i, &self.g, h);
                    if c.active {
                        c.ee_sub_15dd48(*i, &self.g, h);
                    }
                }
                _ => {}
            }
        }
        // FUN_00149c80: run the queued member calls until the queue stays empty
        while !self.k4_deferred.is_empty() {
            for (o, f) in std::mem::take(&mut self.k4_deferred) {
                let Some((i, x)) = self.k4.iter_mut().find(|x| x.0 == o) else { continue };
                match (x, f) {
                    (Inst::C2db6b8(c), c2db6b8::PMF_221298) => c.ee_sub_221298(*i, &self.g, h),
                    _ => {}
                }
            }
        }
    }
}

/// Class-133 fields read by class 132 (handle -> `ClassSet::more` instance).
pub fn c133_view(more: &[(usize, Inst)], o: u32) -> Option<c2cf378::C133View> {
    let i = (o as usize).checked_sub(1)?;
    more.iter().find(|x| x.0 == i).and_then(|(_, x)| match x {
        Inst::C2cf1e8(c) => Some(c2cf378::C133View { f_54: c.f_54, f_c4: c.f_c4, f_98: c.f_98.clone(), f_a4: c.f_a4.clone(), f_d8: c.f_d8.clone() }),
        _ => None,
    })
}
