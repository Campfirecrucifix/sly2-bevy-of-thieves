//! WP-K27: the ALO / PO force hooks of the non-JT player characters (class 55 Bentley slot 69 FUN_00135940,
//! class 54 Murray slot 69 FUN_001b67f8) wired to the E8 physics API (physics::driver / physics::motor).
//! FUN_001f9b20 (ALO gravity, state 0xd / 0x14), FUN_00204108 (PO move forces: the contact motor of
//! FUN_001f8dd8, `motor::po_step`) and FUN_001fac30 (added accelerations) act on the character's `AloPhys`
//! (kind 2 = PO xf), kept by `EngineHost` per object; `ClassSet::frame_k18_55` / `frame_k18_54` step it once
//! after the slot 69 call (`ClassHost::k27_po_step`) with the PO motor fields of the character's own memory
//! (+0x1b0 rows, +0x190 goal, +0x180, +0x210, +0x248).
//! REPLACED: the body starts at the object's world matrix the first time a force hook runs; the foot point
//! offset (jt+0x218) is 0 and the probe / hull extents are the E8 player defaults (50 up / 60 down, hull
//! radius 50 at +60 z); the mass (e8+0xd0) is 1. The stepped position / velocity are what FUN_001a62e8 and
//! vtable +0x10c return for that object afterwards.
use super::plugin::EngineHost;
use crate::engine::physics::driver::{AloPhys, FUN_001f9b20, FUN_001fac30, FUN_0022de88};
use crate::engine::physics::motor::{po_step, PoMotor};

type V3 = [f32; 3];

/// one character's PO state between its slot 69 force hooks and the step
#[derive(Clone, Debug, Default)]
pub struct CharPo {
    pub phys: AloPhys,
    /// FUN_001fac30 accelerations of this frame
    pub acc: V3,
    /// 1 = FUN_001f9b20 (ALO gravity only), 2 = FUN_00204108 (PO motor), 0 = no force hook this frame
    pub mode: u8,
}

/// PO motor fields of a class 54 / 55 character (its +0x180.. +0x248 words)
pub fn motor_of(f: impl Fn(u32) -> f32) -> PoMotor {
    let row = |o: u32| [f(o), f(o + 4), f(o + 8)];
    PoMotor {
        rows_1b0: [row(0x1b0), row(0x1c0), row(0x1d0)],
        v190: row(0x190),
        k180: f(0x180),
        k210: f(0x210),
        k248: f(0x248),
        ..Default::default()
    }
}

impl EngineHost {
    fn k27_po(&mut self, obj: usize) -> &mut CharPo {
        if !self.k27_po.contains_key(&obj) {
            let m = self.k24_world_xf(obj as u32 + 1);
            let mut phys = AloPhys { kind: 2, ..Default::default() };
            if let Some(m) = m {
                phys.pos = m[3];
            }
            self.k27_po.insert(obj, CharPo { phys, ..Default::default() });
        }
        self.k27_po.get_mut(&obj).unwrap()
    }
    pub(super) fn k27_1f9b20(&mut self, obj: usize) {
        self.k27_po(obj).mode = 1;
    }
    pub(super) fn k27_204108(&mut self, obj: usize) {
        self.k27_po(obj).mode = 2;
    }
    /// FUN_001fac30 on a character body (None = not a stepped character)
    pub(super) fn k27_1fac30(&mut self, obj: usize, a: V3) -> bool {
        let Some(p) = self.k27_po.get_mut(&obj) else { return false };
        for k in 0..3 {
            p.acc[k] += a[k];
        }
        true
    }
    /// One FUN_001f3980 / FUN_001f8dd8 frame of the character body after its slot 69.
    pub(super) fn k27_step(&mut self, obj: usize, m: PoMotor, dt: f32) {
        let cw = self.so.clone();
        let Some(p) = self.k27_po.get_mut(&obj) else { return };
        let acc = std::mem::take(&mut p.acc);
        match (std::mem::take(&mut p.mode), cw) {
            (2, Some(cw)) => {
                po_step(&cw, &mut p.phys, &m, [0.0; 3], (50.0, 60.0), Some(([0.0, 0.0, 60.0], 50.0)), acc, dt);
            }
            (1, _) | (2, None) => {
                // FUN_001f3980: previous velocities, accel reset, slot 69 (FUN_001f9b20 + FUN_001fac30), 0x22de88
                let b = &mut p.phys;
                b.prev_v = b.v;
                b.prev_w = b.w;
                b.a = [0.0; 3];
                b.alpha = [0.0; 3];
                FUN_001f9b20(b);
                FUN_001fac30(b, acc);
                FUN_0022de88(b, dt);
            }
            _ => {}
        }
    }
    pub(super) fn k27_pos(&self, obj: usize) -> Option<V3> {
        self.k27_po.get(&obj).map(|p| p.phys.pos)
    }
    pub(super) fn k27_vel(&self, obj: usize) -> Option<V3> {
        self.k27_po.get(&obj).map(|p| p.phys.v)
    }
}

impl super::k25::AnimRt {
    /// FUN_00130d30(f1, f2, asega, mode): live ASEGA -> +0x44 = f2, +0x40 = f2 * +0x48 * +0x4c; new time =
    /// f1 (mode 0) / +0x1c + f1 (1) / ASEG +0x74 + f1 (2) / unchanged (else); a changed time runs
    /// FUN_001317d8 (end handling) and FUN_00130eb0 (event cursor). The +0x34 binding loop (FUN_0012cee0 /
    /// channel segment at the new time) is the K26 per-frame bind (`k26_bind`) from +0x1c.
    pub fn ee_sub_130d30(&mut self, w: &crate::engine::world::World, f1: f32, f2: f32, a: u32, mode: i64) {
        let Some(x) = self.asegas.get_mut(&a).filter(|x| x.st.alive) else { return };
        let Some(g) = crate::engine::playerdraw::aseg_of(w, x.aseg) else { return };
        x.st.svt2 = f2;
        x.st.svt = f2 * x.f48 * x.f4c;
        let t = match mode {
            0 => f1,
            1 => x.st.t + f1,
            2 => g.length + f1,
            _ => x.st.t,
        };
        if t != x.st.t {
            x.st.t = t;
            crate::engine::anim::play::ee_sub_1317d8(&mut x.st, g.length);
            crate::engine::anim::play::ee_sub_130eb0(&mut x.st, g);
        }
    }
}

/// vtable +0x2c (slot 4, message handler) == FUN_00133590 (WP-PUB2: read from SCUS_973.16; was a db
/// class_slots override list, identical for every db vtable with a slot 4).
pub fn slot4_is_base(vtable: u32) -> bool {
    vtable != 0 && crate::engine::disc::exe::exe().w(vtable + 0x2c) == 0x0013_3590
}
