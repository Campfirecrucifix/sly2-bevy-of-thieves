//! WP-P18: cane aim (jt+0x18c0) and the jts 0 / 0xd / 9 cane exits.
//!
//! Aim targets = DAT_003e13b4 (class 18 C2db518 subtree, registered by slot 22 0x21d358): +0xe8 aim mask
//! (0x21d278 default 0x17e, field 215; class 19 swing points 0x184f50: 1), next +0xe0. FUN_0018f2b0(yaw, t,
//! max_ang, jt, mask, off, &target, &dir) 1:1 except the sub point: aim point q = pos + vel * t + R_z(yaw) off;
//! over the list entries with +0xe8 & mask: d = FUN_0011ee08 distance of q to the segment (obj pos, sub point
//! vt+0x104 -> 0x2309e8) (- 250 DAT_002bf4b8 when FUN_0015ed78(0x3dd490, 3, 0x33, obj)), best below 300 whose
//! bearing from the JT is within max_ang of yaw; dir = sub point - (pos + vel * t).
//! REPLACED: sub point = the object's world position (ALO slot 31 0x231550 + 0x2309e8 not ported), the
//! FUN_0015ed78 class-0x33 relation = false (scene query not ported).
//! Callers: 0x18f740 (enter jts 0xd: t 0.25, max 3.15, mask 4, off (125,0,0); face dir), 0x18f560 (cane jtcs
//! 4..7 entries of 0x18e540: t 0.15, max 3.15 in jts 0 else 1.6, mask 2, off (175,0,0); blend weight w =
//! clamp((dir.z + 50) / 110, 0, 1) (1 without a target) -> blend {1 - w, .., .., w} 0x1384c0; face dir only in
//! jts 0), 0x18f688 (t 0.25, max 1.0, mask 8, off (250,0,125); yaw = stick angle when |stick| > 0.5 else facing,
//! -> atan2(dir) with a target; caller not found). JT slot 91 vt+0x2e4 = FUN_00189458: +0x18c0 != 0 ->
//! FUN_002046f0 face the target; read by 0x18a0c8 (jts 0 update) and the 0x197f98 jts 2 steering, which then
//! skip the stick facing. Cleared by 0x18e540(-1) and the jts 9 / 0xd exits.
use super::p5_attach::FUN_0011cf60;
use super::Jt;
use crate::engine::physics::geom::V3;
use crate::engine::world::World;

/// DAT_002bf4e0 / 0x2bf4c0 / 0x2bf4d0 aim offsets (facing frame)
pub const OFF_18F740: V3 = [125.0, 0.0, 0.0];
pub const OFF_18F560: V3 = [175.0, 0.0, 0.0];
pub const OFF_18F688: V3 = [250.0, 0.0, 125.0];
/// DAT_002bf4b8 class-0x33 bonus
pub const DAT_002BF4B8: f32 = -250.0;

/// DAT_003e13b4 entry.
#[derive(Clone, Debug, PartialEq)]
pub struct AimTarget {
    pub id: usize,
    pub pos: V3,
    /// +0xe8
    pub mask: u32,
    /// FUN_0015ed78(0x3dd490, 3, 0x33, obj) (REPLACED: false)
    pub rel_33: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct JtP18 {
    pub aims: Vec<AimTarget>,
    /// jt+0x18c0 aim target (index into `aims`)
    pub aim_18c0: Option<usize>,
    /// last 0x18f560 blend weights {[0], [3]} (consumer: cane ASEG blend 0x1384c0, not drawn)
    pub aim_w: [f32; 2],
}

/// Class-18 subtree objects of a level (one per placed copy, p15::instance_frames).
pub fn level_aim_targets(w: &World) -> Vec<AimTarget> {
    use super::p4::FUN_00132b10 as isa;
    let mut bw = crate::engine::gfx::attach::BoneWorlds::default();
    let mut v = vec![];
    for (i, o) in w.objects.iter().enumerate() {
        let c = o.class as i32;
        if !isa(c, 0x12) || o.disabled || o.clone_of.is_some_and(|p| w.objects[p].class == o.class) {
            continue;
        }
        let def = if isa(c, 0x13) { 1 } else { 0x17e };
        let mask = super::p6::prop_u32(w, i, 215).unwrap_or(def);
        for (id, m) in super::p15::instance_frames(w, &mut bw, i) {
            v.push(AimTarget { id, pos: m[3], mask, rel_33: false });
        }
    }
    v
}

/// 0x0018f2b0 (module doc). Returns (+0x18c0 target, dir).
pub fn FUN_0018f2b0(aims: &[AimTarget], pos: V3, vel: V3, yaw: f32, t: f32, max_ang: f32, mask: u32, off: V3) -> (Option<usize>, V3) {
    let (s, c) = yaw.sin_cos();
    let lead = [pos[0] + vel[0] * t, pos[1] + vel[1] * t, pos[2] + vel[2] * t];
    let q = [lead[0] + c * off[0] - s * off[1], lead[1] + s * off[0] + c * off[1], lead[2] + off[2]];
    let mut best = 300.0;
    let mut out = (None, [0.0; 3]);
    for (k, a) in aims.iter().enumerate() {
        if a.mask & mask == 0 {
            continue;
        }
        let p = a.pos;
        let mut d = ((q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2) + (q[2] - p[2]).powi(2)).sqrt();
        if a.rel_33 {
            d += DAT_002BF4B8;
        }
        if d < best {
            let bearing = (p[1] - pos[1]).atan2(p[0] - pos[0]);
            if max_ang < FUN_0011cf60(bearing - yaw).abs() {
                continue;
            }
            best = d;
            out = (Some(k), [p[0] - lead[0], p[1] - lead[1], p[2] - lead[2]]);
        }
    }
    out
}

impl Jt {
    fn aim(&self, yaw: f32, t: f32, max_ang: f32, mask: u32, off: V3) -> (Option<usize>, V3) {
        FUN_0018f2b0(&self.p18.aims, self.phys.pos, self.velocity(), yaw, t, max_ang, mask, off)
    }

    /// FUN_002046b0(jt, dir) + FUN_00204ca8: face along dir (yaw only).
    pub(super) fn face_dir(&mut self, d: V3) {
        if d[0] != 0.0 || d[1] != 0.0 {
            self.yaw = d[1].atan2(d[0]);
            self.att.rows = crate::engine::physics::driver::FUN_0011b500([0.0, 0.0, 1.0], self.yaw);
        }
    }

    /// 0x0018f740 (jts 0xd entry).
    pub fn FUN_0018f740(&mut self) {
        let (tg, d) = self.aim(self.yaw, 0.25, 3.15, 4, OFF_18F740);
        self.p18.aim_18c0 = tg;
        if tg.is_some() {
            self.face_dir(d);
        }
    }

    /// 0x0018f560 (cane jtcs 4..7 entries): +0x18c0, blend weights, facing in jts 0.
    pub fn FUN_0018f560(&mut self) {
        let stand = self.jts == super::JTS_STAND;
        let (tg, d) = self.aim(self.yaw, 0.15, if stand { 3.15 } else { 1.6 }, 2, OFF_18F560);
        let w = if tg.is_some() { ((d[2] + 50.0) * 0.009090909).clamp(0.0, 1.0) } else { 1.0 };
        self.p18.aim_w = [1.0 - w, w];
        self.calls.push((0x001384c0, 0)); // blend weights into the cane ASEG blend
        self.p18.aim_18c0 = tg;
        if tg.is_some() && stand {
            self.face_dir(d);
        }
    }

    /// 0x0018f688 (no caller found; ported for completeness).
    pub fn FUN_0018f688(&mut self) {
        let yaw = if 0.5 < self.joy.mag { self.joy.angle } else { self.yaw };
        let (tg, d) = self.aim(yaw, 0.25, 1.0, 8, OFF_18F688);
        self.p18.aim_18c0 = tg;
        let y = if tg.is_some() { d[1].atan2(d[0]) } else { yaw };
        self.yaw = y;
        self.att.rows = crate::engine::physics::driver::FUN_0011b500([0.0, 0.0, 1.0], y);
    }

    /// JT slot 91 vt+0x2e4 = 0x00189458: aim target set -> FUN_002046f0 face it.
    pub fn FUN_00189458(&mut self) -> bool {
        let Some(p) = self.p18.aim_18c0.and_then(|k| self.p18.aims.get(k)).map(|a| a.pos) else {
            return false;
        };
        let pos = self.phys.pos;
        self.face_dir([p[0] - pos[0], p[1] - pos[1], 0.0]);
        true
    }

    /// 0x0018c250 exit of jts 0: cane state chain (jtcs 6 -> 4, 7 -> 5, 8/9 -> -1, 0xc -> 0xe unless the
    /// new jtbs is 6, 0xd -> 0xf unless 7), then FUN_00142880 (recorded).
    pub fn FUN_0018c250_exit_0(&mut self, jtbs: i32, anim: &mut dyn super::JtAnim) {
        match self.cane.jtcs {
            6 => self.FUN_0018e540(4, anim),
            7 => self.FUN_0018e540(5, anim),
            8 | 9 => self.FUN_0018e540(-1, anim),
            0xc if jtbs != 6 => self.FUN_0018e540(0xe, anim),
            0xd if jtbs != 7 => self.FUN_0018e540(0xf, anim),
            _ => {}
        }
        self.calls.push((0x00142880, 0));
    }

    /// 0x001e1e30 msg 0x1c on a JT (is-a 0x35) with kind piVar[5]: 2 -> FUN_0018e540(3), 4 -> SetJtJts(0xd, -1),
    /// 8 -> SetJtJts(9, -1). Entry for class code / scripts (the sender side is not ported).
    pub fn FUN_001e1e30_msg1c(&mut self, kind: i32, anim: &mut dyn super::JtAnim) {
        match kind {
            2 => self.FUN_0018e540(3, anim),
            4 => self.set_jts(0xd, super::JTBS_NIL, anim),
            8 => self.set_jts(9, super::JTBS_NIL, anim),
            _ => {}
        }
    }
}
