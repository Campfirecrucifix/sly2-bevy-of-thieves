//! WP-P14: jts 8 pipe climb (class 0xb C2d56f8 path bodies, air target category F / jtbs 0xf).
//!
//! 1:1: FUN_0019e110 (u limits), FUN_0019e1e0 (jts 8 input: X jump-off, R3 0x19cc50 recorded, enter / turn
//! waits, turn test from the path tangent, stick mode machine through FUN_001c4768, speed blend of the two
//! climb ASEG entries, smoothed stick s+0x1c, u clamp by FUN_0011d018 toward the limits, blend weights
//! s+0x50 / s+0x60, sub-state through the existing p5_states::FUN_0019eab0), FUN_001c4768 (camera-projected
//! path direction), FUN_0019de58 speeds (entry +0xc = (root(0).z - root(T).z) / T of the ASEGs jt+0xfa4 /
//! jt+0xfb4).
//! REPLACED: the 0x19e9d8 / 0x19ea00 constraint (jt position = path(u) - jt+0x1010, velocity = tangent *
//! s+0x14) is applied directly to the body instead of through the constraint solver; FUN_001c3500 path frame
//! (tangent search + 0x11ba08 rotation) by rows built from the tangent and the jt+0x1010 direction;
//! FUN_0011f088 decay of jt+0x1010 uses prm {|max speed|, 0, 0.2} (the original register args were not
//! recovered); FUN_00131968 anim-done of the state 0 / 4 ASEGs = elapsed time >= ASEG length (the port's
//! anim driver does not report back); the path object's bit-60 "active" test is assumed set.

#![allow(non_snake_case)]

use super::p5_target::Path;
use super::{Jt, JtAnim, BTN_CROSS};
use crate::engine::physics::geom::V3;
use crate::engine::world::World;

/// R3 (joy bit 0x400): FUN_0019cc50 binocucom look.
pub const BTN_R3: u32 = 0x0400;

/// Pipe-climb data loaded from the JT (FUN_0019de58) plus the per-ride path / offset.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct JtP14 {
    /// s+0x5c / s+0x6c: speeds of the climb blend entries (ASEG jt+0xfa4 / jt+0xfb4)
    pub sp: [f32; 2],
    /// s+0x50 / s+0x60 blend weights
    pub wt: [f32; 2],
    /// lengths of the state ASEGs jt+0xfc4 (enter) / jt+0xfcc (turn), for the REPLACED anim-done test
    pub len_fc4: Option<f32>,
    pub len_fcc: Option<f32>,
    /// s+0xc path of the ridden object (jt+0xf5c)
    pub path: Option<Path>,
    /// jt+0x1010 offset path point - jt position
    pub off_1010: V3,
    /// s+0x14 path speed (du/dt)
    pub v_14: f32,
    /// s+0x1c smoothed stick
    pub f_1c: f32,
    /// s+0x20 stick mode (-1 / 0 / 1)
    pub mode: i32,
    /// FUN_001910f8 -> FUN_0020b868(a+0x240) rope blend (entries jt+0x714 / jt+0x754) and a+0x180 hide blend
    /// (jt+0x654 / jt+0x694); rope_v0 / rope_vmax = a+0x260 / a+0x2a0 (entry velocities .x)
    pub rope_blend: super::p5_attach::Blend2,
    pub hide_blend: super::p5_attach::Blend2,
    pub rope_v0: f32,
    pub rope_vmax: f32,
    /// FUN_001910f8 -> FUN_001a0300(jt, a+0x70, ..) 8-way blend: a+0xc0 / a+0x100 = entry 2 / 4 velocities
    /// (ASEGs jt+0x574 / jt+0x5b4)
    pub b8_vel_c0: V3,
    pub b8_vel_100: V3,
    /// class 0x13 swing points: (object, +0x124 mode (prop 1011), +0x134 length reduction (prop 1015),
    /// WP-P16: +0x130 sound (prop 1014, -1 when absent))
    pub swing: Vec<(usize, i32, f32, i32)>,
    /// class 0x13 +0x138 (prop 1016, object ref) resolved: (point object, landing ref)
    pub ref_138: Vec<(usize, Ref138)>,
    /// jt+0x18cc time the current jts was entered (0x18c250)
    pub t_18cc: f32,
    /// jt+0x360 horizontal launch velocity written by the jts 0x16 X jump
    pub v_360: V3,
}

/// Target +0x138 of a ride point: the jts 0x16 X jump lands on it.
#[derive(Clone, Debug, PartialEq)]
pub enum Ref138 {
    /// is-a 10 object: its path (aim = path(L * (0.5 - lean * 0.5)))
    Path(Path),
    /// other object: its FUN_001a62e8 world position
    Pos(V3),
}

/// FUN_0020b868 entry velocity: (root(T) - root(0)) / T of the JT member ASEG `field` (note the sign is the
/// opposite of FUN_0019de58 / FUN_001a0300).
fn aseg_vel(w: &World, jt: usize, m: &std::collections::HashMap<u32, usize>, field: u32) -> Option<V3> {
    let a = crate::engine::playerdraw::aseg_of(w, *m.get(&field)?)?;
    let c = super::p6::FUN_0012ff10(w, a, jt)?;
    let v = c.vec.as_ref()?;
    let (p0, p1) = (super::p6::slot2(v, 0.0), super::p6::slot2(v, a.length));
    (a.length > 0.0).then(|| [0, 1, 2].map(|k| (p1[k] - p0[k]) / a.length))
}

/// 0x0020b868(b): 2-entry blend init. b+4 blend ASEG, entries b+0x10 + i*0x40 (ASEG at +4, velocity at +0x10).
/// REPLACED: the 0x20bad8 factor k (entry +0x24 over the blend ASEG duration; fields not recovered) = 1.
fn blend_init(w: &World, jt: usize, m: &std::collections::HashMap<u32, usize>, b: u32) -> super::p5_attach::Blend2 {
    let e0 = aseg_vel(w, jt, m, b + 0x14);
    let e1 = aseg_vel(w, jt, m, b + 0x54);
    super::p5_attach::Blend2 {
        on: m.contains_key(&(b + 0x54)),
        w: [1.0, 0.0],
        k: [1.0, 1.0],
        vel: [e0.unwrap_or([0.0; 3]), e1.unwrap_or([0.0; 3])],
        aseg: m.contains_key(&(b + 4)),
    }
}

/// FUN_0019de58 entry speed: (root(0).z - root(T).z) / T of the JT member ASEG `field`.
fn aseg_speed(w: &World, jt: usize, m: &std::collections::HashMap<u32, usize>, field: u32) -> Option<(f32, f32)> {
    let a = crate::engine::playerdraw::aseg_of(w, *m.get(&field)?)?;
    let c = super::p6::FUN_0012ff10(w, a, jt)?;
    let v = c.vec.as_ref()?;
    let (p0, p1) = (super::p6::slot2(v, 0.0), super::p6::slot2(v, a.length));
    Some(((p0[2] - p1[2]) / a.length, a.length))
}

impl JtP14 {
    /// FUN_0019de58 (jts 8 init): speeds of the entries s+0x50 (ASEG jt+0xfa4) / s+0x60 (jt+0xfb4).
    pub fn load(w: &World, jt: usize) -> Self {
        let m = crate::engine::playerdraw::jt_anim_members(w, jt);
        let s0 = aseg_speed(w, jt, &m, 0xfa4);
        let s1 = aseg_speed(w, jt, &m, 0xfb4);
        let len = |f: u32| m.get(&f).and_then(|&o| crate::engine::playerdraw::aseg_of(w, o)).map(|a| a.length);
        // a = jt+0x4c0: rope blend a+0x240 = jt+0x700, hide blend a+0x180 = jt+0x640
        let rope_blend = blend_init(w, jt, &m, 0x700);
        let hide_blend = blend_init(w, jt, &m, 0x640);
        // FUN_001a0300 entry velocity = (root(0) - root(T)) / T (sign of 0x19de58)
        let v8 = |f: u32| aseg_vel(w, jt, &m, f).map(|v| v.map(|x| -x)).unwrap_or([0.0; 3]);
        Self {
            b8_vel_c0: v8(0x574),
            b8_vel_100: v8(0x5b4),
            rope_v0: rope_blend.vel[0][0],
            rope_vmax: rope_blend.vel[1][0],
            rope_blend,
            hide_blend,
            sp: [s0.map_or(0.0, |s| s.0), s1.map_or(0.0, |s| s.0)],
            wt: [1.0, 0.0],
            len_fc4: len(0xfc4),
            len_fcc: len(0xfcc),
            mode: -1,
            swing: swing_props(w),
            ref_138: ref_138(w),
            ..Default::default()
        }
    }
}

/// Class 0x13 props read by the swing code: 1011 -> +0x124 (attach mode; default 1 as SwingTarget),
/// 1015 -> +0x134.
/// WP-P15: plus one row per placed copy (p15::instance_frames target ids) with the prototype's props.
pub fn swing_props(w: &World) -> Vec<(usize, i32, f32, i32)> {
    let mut bw = crate::engine::gfx::attach::BoneWorlds::default();
    let mut v = vec![];
    for i in (0..w.objects.len()).filter(|&i| super::p4::FUN_00132b10(w.objects[i].class as i32, 0x13)) {
        let m = super::p6::prop_u32(w, i, 1011).map_or(1, |v| v as i32);
        let f = super::p6::prop_f32(w, i, 1015).unwrap_or(0.0);
        let snd = super::p6::prop_u32(w, i, 1014).map_or(-1, |v| v as i32);
        v.push((i, m, f, snd));
        if w.objects[i].clone_of.is_none() {
            for (id, _) in super::p15::instance_frames(w, &mut bw, i) {
                if id != i {
                    v.push((id, m, f, snd));
                }
            }
        }
    }
    v
}

/// Prop 1016 (+0x138 object ref by oid) of the class 0x13 points, resolved to a path or a world position.
pub fn ref_138(w: &World) -> Vec<(usize, Ref138)> {
    let mut bw = crate::engine::gfx::attach::BoneWorlds::default();
    let mut v = vec![];
    for i in 0..w.objects.len() {
        if !super::p4::FUN_00132b10(w.objects[i].class as i32, 0x13) {
            continue;
        }
        let Some(oid) = super::p6::prop_u32(w, i, 1016).filter(|&o| o != 0) else { continue };
        let Some(j) = w.objects.iter().position(|o| o.oid as u32 == oid) else { continue };
        if super::p4::FUN_00132b10(w.objects[j].class as i32, 10) {
            if let Some(p) = super::p6::path_of(w, j) {
                v.push((i, Ref138::Path(super::target_path(&p))));
                continue;
            }
        }
        if let Some(m) = bw.xfm_world(w, j) {
            v.push((i, Ref138::Pos(m[3])));
        }
    }
    v
}

/// 0x001d6848(blend, from, to, g): launch velocity from `from` to `to` under gravity g: elevation
/// a = max(atan2(dz, h), 0) + (pi/2 - a) * sinf(blend) (WP-P17: FUN_00245000 = sinf of the blend argument,
/// as K15's classes/shared/launch.rs copy; P14 used the raw blend); k = g.z / (2 (dz - h tan a)); k > 0 -> speed h sqrt(k) / cos a
/// along (dx, dy, h tan a); else 0.
pub fn FUN_001d6848(blend: f32, from: V3, to: V3, g: V3) -> V3 {
    let d = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
    let h = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let a0 = d[2].atan2(h).max(0.0);
    let a = a0 + (std::f32::consts::FRAC_PI_2 - a0) * blend.sin();
    let den = d[2] - h * a.tan();
    let k = g[2] / (den + den);
    if !(0.0 < k) {
        return [0.0; 3];
    }
    let sp = h * k.sqrt() / a.cos();
    let u = [d[0], d[1], h * a.tan()];
    let l = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
    if l == 0.0 {
        return [0.0; 3];
    }
    [u[0] / l * sp, u[1] / l * sp, u[2] / l * sp]
}

/// 0x0019e110(s, out): u limits of the ride (looped path: -FLT_MAX..FLT_MAX).
pub fn FUN_0019e110(path: &Path, dir: bool) -> (f32, f32) {
    if path.looped {
        return (-f32::MAX, f32::MAX);
    }
    let (l0, l1) = (path.l[0], path.FUN_001e6648());
    if !dir {
        (l0 + 100.0, l1 - 35.0)
    } else {
        (l0 + 35.0, l1 - 100.0)
    }
}

/// p(u + 50) - p(u - 50) (0x19e1e0 / 0x1c3500 tangent sample; 0x1e6728 wraps / clamps u).
fn tangent(path: &Path, u: f32) -> V3 {
    let wrap = |x: f32| if path.looped { path.FUN_001e66c8(x) } else { x.clamp(path.l[0], path.FUN_001e6648()) };
    let a = path.FUN_001e6f10(wrap(u + 50.0)).0;
    let b = path.FUN_001e6f10(wrap(u - 50.0)).0;
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if l < 0.0001 {
        [0.0, 0.0, 1.0]
    } else {
        [d[0] / l, d[1] / l, d[2] / l]
    }
}

/// 0x001c4768(joy, v): camera rows (joy+0x40..) times v, returned as the normalised screen vector
/// (-c.y, c.z) (fallback (0, 1)). REPLACED when no camera rows are fed: rows from the JT facing.
pub fn FUN_001c4768(rows: &[V3; 3], v: V3) -> [f32; 2] {
    let c: [f32; 3] = [0, 1, 2].map(|i| rows[i][0] * v[0] + rows[i][1] * v[1] + rows[i][2] * v[2]);
    let (x, y) = (-c[1], c[2]);
    let l = (x * x + y * y).sqrt();
    if l < 0.0001 {
        [0.0, 1.0]
    } else {
        [x / l, y / l]
    }
}

/// REPLACED FUN_001c3500 frame: row2 = tangent (negated when s+0x18 is set), row0 = horizontal direction
/// toward the path (jt+0x1010, else the current facing) made orthogonal, row1 = row2 x row0.
pub fn path_frame(path: &Path, u: f32, dir: bool, toward: V3, facing: V3) -> [V3; 3] {
    let mut t = tangent(path, u);
    if dir {
        t = [-t[0], -t[1], -t[2]];
    }
    let pick = |f: V3| {
        let d = f[0] * t[0] + f[1] * t[1] + f[2] * t[2];
        let r = [f[0] - t[0] * d, f[1] - t[1] * d, f[2] - t[2] * d];
        let l = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
        (l > 0.05).then(|| [r[0] / l, r[1] / l, r[2] / l])
    };
    let r0 = pick([toward[0], toward[1], 0.0]).or_else(|| pick(facing)).unwrap_or([1.0, 0.0, 0.0]);
    let r1 = [t[1] * r0[2] - t[2] * r0[1], t[2] * r0[0] - t[0] * r0[2], t[0] * r0[1] - t[1] * r0[0]];
    [r0, r1, t]
}

/// State ASEG played by the 0x19eab0 setter (0x1305b8 ids 0x74 / 0x78 / 0x70 / 0x7c -> JT members).
pub fn jts8_aseg(state: i32, p: &JtP14) -> Option<u32> {
    match state {
        0 => Some(if p.len_fc4.is_some() { 0xfc4 } else { 0xfcc }),
        1 => Some(0xfc8),
        2 | 3 => Some(0xfc0),
        4 => Some(0xfcc),
        _ => None,
    }
}

impl Jt {
    /// jtbs 0xe arrival: jt+0x18c4 = the target (0x13 swing point) -> P5World.swing (pivot = FUN_001a62e8,
    /// rows FUN_001a6318, static pivot velocity 0).
    pub fn p14_swing_target(&mut self) {
        let Some(id) = self.flight.target else { return };
        let Some(tg) = self.target_attach(Some(id)) else { return };
        let (mode, f134, snd) = self.p14.swing.iter().find(|s| s.0 == id).map_or((1, 0.0, -1), |s| (s.1, s.2, s.3));
        self.p5w.swing = Some(super::p5_states::SwingTarget { pos: tg.pos, rot: tg.rot, v_point: [0.0; 3], mode_124: mode, f_134: f134 });
        self.p16.snd_18c4 = snd; // WP-P16: jt+0x18c4 target +0x130
    }

    /// Sub-state setter with the state ASEG start (p5_states::FUN_0019eab0 records the 0x1305b8 call).
    fn jts8_set(&mut self, new: i32, anim: &mut dyn JtAnim) {
        let old = self.p5.s8.state;
        self.with_p5(|jt, st, w| super::p5_states::FUN_0019eab0(jt, st, w, new));
        if new != old {
            if new == 0 {
                self.p14.f_1c = 0.0;
            }
            if let Some(f) = jts8_aseg(new, &self.p14) {
                anim.play_aseg(f, 8, new);
            }
        }
    }

    /// 0x0018a8c8 case 0x15: X within 0.25 s -> FUN_00195a00(0x12).
    pub fn FUN_0018a8c8_15(&mut self, anim: &mut dyn JtAnim) {
        if self.joy.FUN_001c46c0(self.t, 0.25, BTN_CROSS) != 0 {
            self.FUN_00195a00(0x12, 0, anim);
        }
    }

    /// 0x0018a8c8 case 0x16 (ride point X jump): aim = T+0x138 path at L * (0.5 - jt+0x1f94 * 0.5), else
    /// T-xf * (R(jt+0x1f94 * 0.5) * local) with local = (-50, 0, 300) (DAT_002bf370) or the +0x138 object in
    /// T space (FUN_001a6f00); v = FUN_001d6848(0.5, jt pos, aim - jt+0x240, g) * jt+0x1f98 -> vt+0x11c,
    /// FUN_00195a00(0xc), jt+0x360 = v - z (v.z). REPLACED: the 0x11b500 axis is passed in a register
    /// Ghidra did not recover; the T z axis (lean = yaw about the point) is used.
    pub fn FUN_0018a8c8_16(&mut self, anim: &mut dyn JtAnim) {
        if self.joy.FUN_001c46c0(self.t, 0.25, BTN_CROSS) == 0 {
            return;
        }
        let Some(tg) = self.p5w.swing else { return };
        let id = self.flight.target.or(self.attach.target);
        let r = id.and_then(|i| self.p14.ref_138.iter().find(|x| x.0 == i).map(|x| x.1.clone()));
        let lean = self.p5.f_1f94;
        let xf = |l: V3| -> V3 {
            let (c, s) = ((lean * 0.5).cos(), (lean * 0.5).sin());
            let q = [c * l[0] - s * l[1], s * l[0] + c * l[1], l[2]];
            [0, 1, 2].map(|k| q[0] * tg.rot[0][k] + q[1] * tg.rot[1][k] + q[2] * tg.rot[2][k] + tg.pos[k])
        };
        let aim = match r {
            Some(Ref138::Path(p)) => p.FUN_001e6f10(p.FUN_001e6648() * (0.5 - lean * 0.5)).0,
            Some(Ref138::Pos(q)) => {
                let d = [q[0] - tg.pos[0], q[1] - tg.pos[1], q[2] - tg.pos[2]];
                xf([0, 1, 2].map(|k| d[0] * tg.rot[k][0] + d[1] * tg.rot[k][1] + d[2] * tg.rot[k][2]))
            }
            None => xf([-50.0, 0.0, 300.0]),
        };
        let g = self.att.grab;
        let to = [aim[0] - g[0], aim[1] - g[1], aim[2] - g[2]];
        let v = FUN_001d6848(0.5, self.phys.pos, to, self.phys.gravity);
        let m = self.p5.f_1f98;
        self.set_velocity([v[0] * m, v[1] * m, v[2] * m]);
        self.FUN_00195a00(0xc, 0, anim);
        self.p14.v_360 = [v[0], v[1], 0.0];
    }

    /// 0x0019c688 jtbs 0xf (arrival at a category F path): jt+0xf5c = target path, jt+0xf60 = JMP +0x14
    /// (arc length of the pick), then vt+0x2c4(jt, 8) -> SetJtJts(8, -1) -> 0x19eab0(0) with 0x19e110.
    pub fn jts8_enter(&mut self, anim: &mut dyn JtAnim) {
        let tg = self.flight.target;
        let path = self.p5_targets.iter().find(|t| Some(t.obj.id) == tg).and_then(|t| match &t.obj.geom {
            super::p5_target::TargetGeom::PathBody { path } => Some(path.clone()),
            _ => None,
        });
        let Some(path) = path else {
            self.calls.push((0x2c4, 8));
            return;
        };
        let u = self.flight.f_14;
        let t = tangent(&path, u);
        self.p5w.path8_tangent_z = t[2];
        self.p5.s8.u = u;
        self.p5.s8.state = -1;
        self.p14.mode = -1;
        self.p14.v_14 = 0.0;
        self.p14.path = Some(path);
        let dt = self.dt;
        self.with_p5(|jt, st, w| super::p5_states::FUN_0019c688_p5(jt, st, w, dt, &mut *anim));
        // 0x19eab0(-1 -> 0) tail: 0x19e110 clamps u, jt+0x1010 = path(u) - jt position
        let p = self.p14.path.as_ref().unwrap();
        let (lo, hi) = FUN_0019e110(p, self.p5.s8.dir_18);
        self.p5.s8.u = self.p5.s8.u.clamp(lo, hi.max(lo));
        let q = p.FUN_001e6f10(self.p5.s8.u).0;
        self.p14.off_1010 = [q[0] - self.phys.pos[0], q[1] - self.phys.pos[1], q[2] - self.phys.pos[2]];
        if let Some(f) = jts8_aseg(self.p5.s8.state, &self.p14) {
            anim.play_aseg(f, 8, self.p5.s8.state);
        }
        self.jts8_frame();
    }

    fn jts8_frame(&mut self) {
        let Some(p) = &self.p14.path else { return };
        let face = [self.yaw.cos(), self.yaw.sin(), 0.0];
        self.p5w.path8 = Some(path_frame(p, self.p5.s8.u, self.p5.s8.dir_18, self.p14.off_1010, face));
    }

    /// 0x0019e1e0: jts 8 input (see module doc).
    pub fn FUN_0019e1e0(&mut self, anim: &mut dyn JtAnim) {
        let dt = self.dt;
        let Some(path) = self.p14.path.clone() else {
            self.FUN_00195a00(0, 0, anim);
            return;
        };
        // jt+0x1010 decays toward 0 (FUN_0011f088, REPLACED prm)
        let mut v = [0.0; 3];
        let prm = [self.p14.sp[1].abs().max(1.0), 0.0, 0.2];
        self.p14.off_1010 = super::p5_attach::FUN_0011f088(dt, self.p14.off_1010, [0.0; 3], prm, &mut v);
        if self.p5.s8.state != 0 && self.joy.FUN_001c46c0(self.t, 0.25, BTN_CROSS) != 0 {
            self.FUN_00195a00(2, 0, anim);
            return;
        }
        if self.joy.FUN_001c4660(self.t, 0.15, BTN_R3) != 0 && !self.binoc.active() {
            // vt+0x31c(jt+(+0x318)): false while 0x1443d8 (WP-P15 binoc 0x19cc50)
            self.binoc_r3(anim);
        }
        loop {
            let s = self.p5.s8.state;
            if s == 0 || s == 4 {
                let len = if s == 0 { self.p14.len_fc4.or(self.p14.len_fcc.map(|l| l * 0.5)) } else { self.p14.len_fcc };
                let done = self.p5w.anim.done || self.t - self.p5.s8.t_8 >= len.unwrap_or(0.0);
                if done {
                    self.jts8_set(1, anim);
                }
            }
            if self.p5.s8.state == s {
                break;
            }
        }
        let s = self.p5.s8.state;
        if s == 0 || s == 4 {
            self.jts8_frame();
            return;
        }
        let u = self.p5.s8.u;
        let t = tangent(&path, u);
        let dir = self.p5.s8.dir_18;
        let f1c = self.p14.f_1c;
        if (0.5 < t[2].abs() && ((0.0 < t[2]) == dir)) || (0.4 < f1c && dir && -0.3 < t[2]) || (f1c < -0.4 && !dir && t[2] < 0.3) {
            self.jts8_set(4, anim);
            self.jts8_frame();
            return;
        }
        let rows = self.p5w.cam_rows.unwrap_or_else(|| {
            let (c, s) = (self.yaw.cos(), self.yaw.sin());
            [[c, s, 0.0], [-s, c, 0.0], [0.0, 0.0, 1.0]]
        });
        // p(u+50) - p(u-50) (same direction as `t`)
        let sv = FUN_001c4768(&rows, t);
        let d = sv[0] * self.p5w.stick[0] + sv[1] * self.p5w.stick[1];
        let mag = self.joy.mag;
        let m = &mut self.p14.mode;
        match *m {
            0 => {
                if mag < 0.25 {
                    *m = -1;
                } else if d < -0.7 {
                    *m = 1;
                }
            }
            -1 => {
                if 0.35 < mag {
                    *m = if d < 0.0 { 1 } else { 0 };
                }
            }
            1 => {
                if mag < 0.25 {
                    *m = -1;
                } else if 0.7 < d {
                    *m = 0;
                }
            }
            _ => {}
        }
        let (mut iv, mut f) = match *m {
            0 => (if dir { 3 } else { 2 }, mag),
            1 => (if dir { 2 } else { 3 }, mag),
            -1 => (1, 0.0),
            _ => (-1, d),
        };
        let (smin, smax) = (self.p14.sp[0], self.p14.sp[1]);
        let mut sp;
        if 0.01 < f {
            sp = (1.0 - f) * smin + f * smax;
            if iv == 3 {
                sp = -sp;
            }
        } else {
            sp = 0.0;
            iv = 1;
        }
        if dir {
            sp = -sp;
        }
        let amax = smax.abs();
        self.p14.f_1c = if amax != 0.0 { (-dt * sp) / amax } else { 0.0 } + (1.0 - dt) * self.p14.f_1c;
        let mut vv = sp;
        if !path.looped {
            let (lo, hi) = FUN_0019e110(&path, dir);
            let prm = [amax, 0.0, 0.2];
            let (mut o1, mut o2) = (0.0f32, 0.0f32);
            crate::engine::physics::dynamics::FUN_0011d018(u, lo, dt, prm, Some(&mut o1));
            crate::engine::physics::dynamics::FUN_0011d018(u, hi, dt, prm, Some(&mut o2));
            vv = -o2;
            if vv <= sp {
                vv = sp;
                if -o1 < sp {
                    vv = -o1;
                }
            }
        }
        f = 0.0;
        self.p14.v_14 = -vv;
        let mut u = u;
        if vv != 0.0 {
            u += -vv * dt;
            self.calls.push((0x001f4bf8, 0));
        }
        if path.looped {
            u = path.FUN_001e66c8(u);
        }
        self.p5.s8.u = u;
        if vv.abs() < 0.01 {
            iv = 1;
        } else {
            let den = smax.abs() - smin.abs();
            let x = if den != 0.0 { (vv.abs() - smin.abs()) / den } else { 1.0 };
            if f <= x {
                f = x.min(1.0);
            }
            self.p14.wt = [1.0 - f, f];
            self.calls.push((0x001384c0, 0));
        }
        self.jts8_set(iv, anim);
        self.jts8_frame();
    }

    /// REPLACED 0x19e9d8 / 0x19ea00 constraint + 0x19df70 orientation: body on path(u) - jt+0x1010 with the
    /// path velocity, rows = the path frame. Returns false when not riding.
    pub fn jts8_step(&mut self) -> bool {
        if self.jts != 8 {
            return false;
        }
        let Some(p) = &self.p14.path else { return false };
        let u = self.p5.s8.u;
        let q = p.FUN_001e6f10(u).0;
        let t = tangent(p, u);
        let o = self.p14.off_1010;
        self.phys.pos = [q[0] - o[0], q[1] - o[1], q[2] - o[2]];
        let s = self.p14.v_14;
        self.phys.v = [t[0] * s, t[1] * s, t[2] * s];
        self.phys.imp_lin = [0.0; 3];
        self.phys.imp_ang = [0.0; 3];
        self.phys.w = [0.0; 3];
        if let Some(m) = self.p5w.path8 {
            self.phys.rot = m;
            self.yaw = m[0][1].atan2(m[0][0]);
        }
        self.ground_nz = None;
        self.grounded = false;
        true
    }
}
