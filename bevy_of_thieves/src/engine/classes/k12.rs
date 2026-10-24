//! WP-K12: class 107 C2dcc98 (SO, parent C2d4260): player-proximity glow. Slot 6 FUN_00229ef0 init, slot 14
//! FUN_00229f88 load-post (clears +0x40 bit 3 of the +0x16c / +0x170 / +0x174 objects), slot 15 FUN_0022a060
//! per-frame proximity state +0x164 (vtable +0x1c 5 / 6), 1:1. Props: 1313 -> +0x154 (is-a class), 1316 ->
//! +0x150 mode, 1317 -> +0x168, 1318-1320 (oid refs) -> +0x16c / +0x170 / +0x174. Handles = object index + 1.
//! REPLACED: slot 18 FUN_0022a908 (VU glow / flare draw of the +0x16c..+0x174 objects: FUN_0022a220,
//! FUN_0011ee08 ray, FUN_001a5ee8 / FUN_001a5fb8 matrix draws, FUN_001cd780 sprite) is renderer-side, a
//! `ClassHost::base(0x22a908)` hook; the oid refs are resolved at load with FUN_0015e938 (first object by oid).
use super::k4::word;
use super::*;
use crate::engine::gfx::attach::alo_of;
use crate::engine::world::props::{Prop, PropValue};

/// K12 world class id -> vtable (kept in `ClassSet::k12`).
pub const PORTED_K12: &[(ClassId, u32)] = &[(107, 0x2dcc98)];

/// slot 5 FUN_001140c0 (class descriptor getter)
pub fn slot5(class: ClassId) -> Option<u32> {
    (class == 107).then_some(0x2bb344)
}

fn handle(obj: usize) -> u32 {
    obj as u32 + 1
}

/// LOOK R36: DAT_003e1be0 = DAT_002dded0 writers (FUN_0014ff38 / FUN_00150160 look, FUN_00150938 / FUN_00150a28
/// zoom) run in the camera systems on another clock -> they raise this flag and `frame_k12`'s caller stamps
/// `ClassGlobals::g_3e1be0` with the class clock t of that frame.
pub static LOOK_T_STAMP: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub fn stamp_look_t() {
    LOOK_T_STAMP.store(true, std::sync::atomic::Ordering::Relaxed);
}
/// LOOK R39: DAT_003e1be4 = DAT_002dded0 writer FUN_00140338 (q10_g, world +0x4754) runs on QMem -> same
/// flag scheme as `LOOK_T_STAMP`; the caller stamps `ClassGlobals::g_3e1be4` with the class clock t.
pub static JOB_T_STAMP: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub fn stamp_job_t() {
    JOB_T_STAMP.store(true, std::sync::atomic::Ordering::Relaxed);
}

#[derive(Clone, Debug, PartialEq)]
pub struct C2dcc98 {
    /// +0x40 flags (slot 6 |= 10), +0x148 bit 35 (slot 6 clears)
    pub f_40: u32,
    pub bit35: bool,
    /// +0x150 mode (field 1316; 0 / 2 = off while DAT_003e0f9c), +0x154 is-a class of the player (field 1313)
    pub f_150: i32,
    pub f_154: i32,
    /// +0x158 / +0x15c on / off radius
    pub f_158: f32,
    pub f_15c: f32,
    /// +0x164 proximity state
    pub f_164: i32,
    /// +0x168 (field 1317)
    pub f_168: u32,
    /// +0x16c / +0x170 / +0x174 glow objects (fields 1318-1320, handles)
    pub f_16c: [u32; 3],
    /// +0x178 / +0x184
    pub f_178: i32,
    pub f_184: i32,
    /// +0x188 (FUN_0022a908 mode 2: FUN_00163400 object; no prop / init writes it -> 0 as allocated)
    pub f_188: u32,
    /// LOOK R34: slot 18 FUN_0022a908 alphas of this frame (fVar10 = glow / sign alpha, fVar12 = label alpha);
    /// the gfx side (npcdraw::objects) draws the +0x16c / +0x174 objects with them
    pub draw_a: f32,
    pub draw_la: f32,
}

impl Default for C2dcc98 {
    fn default() -> Self {
        C2dcc98 { f_40: 0, bit35: false, f_150: 0, f_154: 0, f_158: 0.0, f_15c: 0.0, f_164: 0, f_168: 0, f_16c: [0; 3], f_178: 0, f_184: 0, f_188: 0, draw_a: 0.0, draw_la: 0.0 }
    }
}

impl C2dcc98 {
    /// slot 6 FUN_00229ef0: base FUN_001f2108, +0x154 = 0x32, radii 300 / 500, +0x184 = +0x178 = -1,
    /// +0x40 |= 10, +0x148 bit 35 cleared
    pub fn ee_sub_229ef0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f2108, obj);
        self.f_154 = 0x32;
        self.f_158 = 300.0;
        self.f_15c = 500.0;
        self.f_184 = -1;
        self.f_178 = -1;
        self.f_40 |= 10;
        self.bit35 = false;
    }

    /// property rows (fields 1313 / 1316 / 1317; 1318-1320 through `oid`)
    pub fn feed(&mut self, props: &[Prop], oid: &mut dyn FnMut(i32) -> u32) {
        for p in props {
            let PropValue::Scalar(b) = &p.value else { continue };
            match p.field {
                // ANIM-ALL R62: 1314 (kind 0xa, +0x158 / +0x15c off / on distances; descriptor 0x2ac880) was not fed
                // (PCSX2 060: the EP1-hub copy of oid 17472 with override 1314 = (100, 200) has them, others 300 / 500)
                1314 if std::env::var_os("SLY2_R62OFF").is_none() && b.len() >= 8 => {
                    self.f_158 = f32::from_bits(word(b));
                    self.f_15c = f32::from_bits(u32::from_le_bytes([b[4], b[5], b[6], b[7]]));
                }
                1313 => self.f_154 = u16::from_le_bytes([b.first().copied().unwrap_or(0), b.get(1).copied().unwrap_or(0)]) as i32,
                1316 => self.f_150 = word(b) as i32,
                1317 => self.f_168 = word(b),
                1318..=1320 => self.f_16c[(p.field - 1318) as usize] = oid(word(b) as i32),
                _ => {}
            }
        }
    }

    /// slot 14 FUN_00229f88: base FUN_001f36a8, then clear +0x40 bit 3 of each set +0x16c / +0x170 / +0x174
    pub fn ee_sub_229f88(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1f36a8, obj);
        for o in self.f_16c {
            if o != 0 {
                h.obj_and_40(o, !8);
            }
        }
    }

    /// slot 15 FUN_0022a060: base FUN_001f36e0; on = player (FUN_001c4840) is-a +0x154 and closer than
    /// +0x158 (off) / +0x15c (on); modes 0 / 2 are off while DAT_003e0f9c; a JT (is-a 0x34) only turns it on
    /// when free (+0x2dc == -1, vtable +0x26c != 2), else the state is kept; a change sets +0x164 and calls
    /// vtable +0x1c with 5 (on) / 6 (off)
    pub fn ee_sub_22a060(&mut self, obj: usize, g: &ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1f36e0, obj);
        let p = h.ee_sub_1c4840();
        let mut on = false;
        if p != 0 && h.ee_sub_132b10(p, self.f_154) != 0 {
            let r = if self.f_164 == 0 { self.f_158 } else { self.f_15c };
            let (a, b) = (h.ee_sub_1a62e8(p), h.ee_sub_1a62e8(handle(obj)));
            let d = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
            on = d < r;
        }
        if (self.f_150 == 0 || self.f_150 == 2) && g.g_3e0f9c != 0 {
            on = false;
        }
        let mut s = on as i32;
        if on && h.ee_sub_132b10(p, 0x34) != 0 && !h.jt_free_2dc_26c(p) {
            s = self.f_164;
        }
        if s != self.f_164 {
            self.f_164 = s;
            h.vcall_1c(obj, if s == 0 { 6 } else { 5 }, 0);
        }
    }

    /// slot 18 FUN_0022a908: base FUN_001f2708-style C2caf88 slot, then the fade alphas (consts 0x2c4494.. read
    /// from the PCSX2 EP4 028 savestate): a = clamp(1 - 0.5 t, 0, 1), t = clk - max(DAT_003e1be0 + 10,
    /// DAT_003e1be4 + 10); label la = clamp(1 - (clk - (DAT_003e1be0 + 0.1)), 0, 1); both 0 when the +0x150 mode
    /// gate is shut (mode 1: only while DAT_003e0f9c; mode 0: only while !DAT_003e0f9c; mode 2: all clue
    /// bottles found, !DAT_003e0f9c, +0x184 / +0x188 tests). The draw itself (FUN_0022a220 height, FUN_0011ee08 scale,
    /// objects_drive_obj_draw of +0x16c / +0x174, FUN_001a5fb8 beam +0x170, label) is in npcdraw::objects.
    pub fn ee_sub_22a908(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x22a908, obj);
    }
    /// FUN_0022a908 alpha part (see [`Self::ee_sub_22a908`]), run right after it by `frame_k12`
    pub fn ee_sub_22a908_alphas(&mut self, g: &ClassGlobals, h: &mut dyn ClassHost) {
        let clk = g.t;
        let t = clk - (g.g_3e1be0 + 10.0).max(g.g_3e1be4 + 10.0);
        let mut a = (1.0 + t * (-0.5 + t * 0.0)).clamp(0.0, 1.0);
        let t2 = clk - (g.g_3e1be0 + 0.1);
        let mut la = (1.0 + t2 * (-1.0 + t2 * 0.0)).clamp(0.0, 1.0);
        let open = match self.f_150 {
            1 => g.g_3e0f9c != 0,
            0 => g.g_3e0f9c == 0,
            // mode 2 (LOOK R40, 1:1): only once every clue bottle is found (DAT_003e1bf8 <= DAT_003e1bf4, the
            // "%d/%d" counter; PS2 EP4 028: 0 / 0x1e -> shut) && !DAT_003e0f9c && (+0x184 == -1 ||
            // !FUN_001daea8(+0x184)) && (+0x188 == 0 || FUN_00163400(+0x188))
            2 => {
                g.g_3e1bf8 <= g.g_3e1bf4
                    && g.g_3e0f9c == 0
                    && (self.f_184 == -1 || h.po_call(0x1daea8, 0, 0, &[self.f_184 as u32]) == 0)
                    && (self.f_188 == 0 || h.po_call(0x163400, 0, 0, &[self.f_188]) != 0)
            }
            _ => true,
        };
        if !open {
            a = 0.0;
            la = 0.0;
        }
        self.draw_a = a;
        self.draw_la = la;
    }
}

impl ClassSet {
    /// K12: construct + init (slot 6) + feed the `PORTED_K12` classes (clones read their source's props)
    pub(super) fn build_k12(&mut self, w: &World, h: &mut dyn ClassHost) {
        for (i, o) in w.objects.iter().enumerate() {
            if !PORTED_K12.iter().any(|x| x.0 == o.class) {
                continue;
            }
            let src = o.clone_of.unwrap_or(i);
            let mut c = C2dcc98::default();
            c.ee_sub_229ef0(i, h);
            let props = props_of(w, src).map(|p| p.to_vec()).or_else(|| alo_of(w, src).map(|a| a.props.clone())).unwrap_or_default();
            let props = super::props_ov(w, i, &props).into_owned();
            // LOOK R35: object-ref fields resolve like FUN_0013e590 / FUN_0013e5e0 = FUN_0015e938(0x3dd490, 4, oid, self) (nearest
            // relative: each marker its own sign / beam / base; PS2 EP4 028 markers hold distinct glow objects)
            c.feed(&props, &mut |oid| h.ee_sub_15e938(4, oid, handle(i)));
            if std::env::var_os("SLY2_LOOK_K12LOG").is_some() {
                let oid_of = |hd: u32| if hd == 0 { -1 } else { w.objects.get(hd as usize - 1).map_or(-2, |o| o.oid as i64) };
                log::info!("K12 obj {i} oid {} mode {} 154 {} 168 {} glow oids {:?}", o.oid, c.f_150, c.f_154, oid_of(c.f_168), c.f_16c.map(oid_of));
            }
            self.k12.push((i, Inst::C2dcc98(c)));
        }
    }
    /// K12 state at object index `obj`
    pub fn k12_get(&mut self, obj: usize) -> Option<&mut C2dcc98> {
        self.k12.iter_mut().find(|x| x.0 == obj).and_then(|x| if let Inst::C2dcc98(c) = &mut x.1 { Some(c) } else { None })
    }
    pub fn k12_get_ref(&self, obj: usize) -> Option<&C2dcc98> {
        self.k12.iter().find(|x| x.0 == obj).and_then(|x| if let Inst::C2dcc98(c) = &x.1 { Some(c) } else { None })
    }
    /// LOOK R39: FUN_0022a908 label id `*(*(+0x160) + 0x7c) + 0x6c` of marker `obj`; None when a link is null
    /// or the id is -1. +0x160 = the job (C2cf1e8, class 133) whose +0x80 (field 1279) is this marker: on the
    /// PS2 EP4 020..029 savestates every non-null marker +0x160 is the job with job+0x80 == marker and no other
    /// job points at a marker (9 of 110 jobs), so the link is resolved through the job's +0x80; job +0x7c
    /// (field 1264) = a C2cf378 (class 132) whose +0x6c (field 1245) is the level string id.
    pub fn k12_label_id(&self, obj: usize) -> Option<i32> {
        let all = || self.insts.iter().chain(self.more.iter());
        let q = all().find_map(|(_, x)| match x {
            Inst::C2cf1e8(j) if j.f_80 as usize == obj + 1 => Some(j.f_7c),
            _ => None,
        })?;
        let q = (q as usize).checked_sub(1)?;
        let id = all().find_map(|(i, x)| match x {
            Inst::C2cf378(d) if *i == q => Some(d.f_6c),
            _ => None,
        })?;
        (id != -1).then_some(id)
    }
    /// K12 load-post: slot 14
    pub(super) fn post_load_k12(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &mut self.k12 {
            if let Inst::C2dcc98(c) = x {
                c.ee_sub_229f88(*i, h);
            }
        }
    }
    /// K12 per frame: slot 15, then slot 18 (draw hook)
    pub(super) fn frame_k12(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &mut self.k12 {
            // FEEL R33: slot 15 runs from the world update list only (LO +0x28 bit 60)
            if h.lo_removed(*i) {
                continue;
            }
            if let Inst::C2dcc98(c) = x {
                c.ee_sub_22a060(*i, &self.g, h);
                c.ee_sub_22a908(*i, h);
                c.ee_sub_22a908_alphas(&self.g, h);
            }
        }
    }
}

/// LOOK R35: FUN_0022a220 (0x0022a220) +0x180 = the lift of the marker's sign above the marker position `p`
/// (the +0x168 object, else the marker) that clears the camera's line of sight. Quad Q = (p, p + Z 5000,
/// cam + Z 5000, cam) (0x2c4490, up DAT_002a2dc0), its plane N = FUN_00119308(Q0, Q1, Q2) and the four side
/// planes n_i = normalize((Q[i+1] - Q[i]) x N) through Q[i] (InitPlaneFromNormalPoint). Every collision edge
/// (a, b) whose surface the box query marked (ClearWorld 0x3dd490 / FUN_00118d08 flag 0x4000) and that straddles
/// N (d_a <= 0 or d_b <= 0, and 0 <= d_a or 0 <= d_b) gives X = a + (b - a) d_a / (d_a - d_b); when X is strictly
/// inside all four side planes its view slope (X - cam).z / |(X - cam).xy| raises the running max (start: the
/// slope of p). +0x180 = (max slope - slope(p)) * |(p - cam).xy|.
/// REPLACED: the ClearWorld / FUN_00118d08 BSP marking -> every edge of the SO collision set (`edges`, world
/// space); edges off the quad fail the side-plane test anyway. Runs once per frame (+0x178 = DAT_002de3e4).
pub fn ee_sub_22a220(edges: &mut dyn Iterator<Item = ([f32; 3], [f32; 3])>, cam: [f32; 3], p: [f32; 3]) -> f32 {
    use crate::engine::physics::geom::{dot, sub, FUN_00119308};
    let up = |v: [f32; 3]| [v[0], v[1], v[2] + 5000.0];
    let q = [p, up(p), up(cam), cam];
    let n = FUN_00119308(q[0], q[1], q[2]);
    let nn = [n[0], n[1], n[2]];
    if !(nn[0].is_finite() && nn[1].is_finite() && nn[2].is_finite()) {
        return 0.0;
    }
    let side: Vec<[f32; 4]> = (0..4)
        .map(|i| {
            let e = sub(q[(i + 1) % 4], q[i]);
            let c = [e[1] * nn[2] - nn[1] * e[2], e[2] * nn[0] - nn[2] * e[0], e[0] * nn[1] - nn[0] * e[1]];
            let l = dot(c, c).sqrt();
            let c = [c[0] / l, c[1] / l, c[2] / l];
            [c[0], c[1], c[2], dot(c, q[i])]
        })
        .collect();
    let slope = |x: [f32; 3]| {
        let d = sub(x, cam);
        d[2] / (d[0] * d[0] + d[1] * d[1]).sqrt()
    };
    let s0 = slope(p);
    let mut best = s0;
    for (a, b) in edges {
        let da = dot(nn, a) - n[3];
        let db = dot(nn, b) - n[3];
        if !((da <= 0.0 || db <= 0.0) && (0.0 <= da || 0.0 <= db)) {
            continue;
        }
        let t = da / (da - db);
        let x = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];
        if side.iter().all(|pl| dot([pl[0], pl[1], pl[2]], x) - pl[3] < 0.0) {
            let s = slope(x);
            if best < s {
                best = s;
            }
        }
    }
    let d = sub(p, cam);
    (best - s0) * (d[0] * d[0] + d[1] * d[1]).sqrt()
}
