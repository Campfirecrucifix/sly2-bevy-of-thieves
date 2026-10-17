//! C2ce828 (world class 31): emitter group — forwards slot-50 emit requests to its attached class-30
//! objects (+0xe0 count / +0xe4 list, filled by the load FUN_0015c1e8 = `World::life.attached`).
//! Object handles in hooks are index + 1 (0 = null).
use super::ClassHost;
use crate::engine::anim::pose::{ee_sub_11c168, ee_sub_11c318, Mat4};

#[derive(Clone, Debug, Default)]
pub struct C2ce828 {
    /// +0xe0 / +0xe4: attached objects (handles)
    pub f_e4: Vec<u32>,
}

/// slot 5: FUN_00113c00
pub fn ee_sub_113c00() -> u32 {
    0x2bad54
}

/// Request matrix block (r[13] when r[0] & 0x40): [0] translation, [1..3] basis, [4] / [5] extra vectors.
pub type ReqBlock = [[f32; 4]; 6];

impl C2ce828 {
    /// Load list (FUN_0015c1e8 tail): attached object indices.
    pub fn feed(&mut self, attached: &[usize]) {
        self.f_e4 = attached.iter().map(|&n| n as u32 + 1).collect();
    }

    /// slot 27: FUN_0015c310 -> (+0xe0, +0xe4)
    pub fn ee_sub_15c310(&self) -> (i32, &[u32]) {
        (self.f_e4.len() as i32, &self.f_e4)
    }

    /// slot 40: FUN_0015c320 (clone fixup): base FUN_002311b0, fresh list, each entry cloned into the
    /// clone's owner (+0x10) by FUN_0013e510.
    pub fn ee_sub_15c320(&mut self, obj: usize, owner: u32, h: &mut dyn ClassHost) {
        h.base(0x2311b0, obj);
        for k in 0..self.f_e4.len() {
            self.f_e4[k] = h.ee_sub_13e510(self.f_e4[k], owner);
        }
    }

    /// slot 50: FUN_0015c3c0 — forward the request `r` (with request block `m` = *r[13]) to every attached
    /// object's slot 50 (vtable +0x19c), re-expressed in the child's frame: rel = inv(this +0x60) * child +0x60,
    /// composed with `m` when r[0] & 0x40. Source = this object unless r[0] & 1.
    pub fn ee_sub_15c3c0(&self, obj: usize, r: &[u32; 15], m: &ReqBlock, h: &mut dyn ClassHost) {
        let mut q = *r;
        q[0] = r[0] | 0x40;
        if r[0] & 1 == 0 {
            q[0] = r[0] | 0x41;
            q[1] = obj as u32 + 1;
        }
        for &c in &self.f_e4 {
            let inv = ee_sub_11c318(&h.obj_mat(obj as u32 + 1, 0x60));
            let rel: Mat4 = ee_sub_11c168(&inv, &h.obj_mat(c, 0x60));
            let mut b: ReqBlock = [[0.0; 4]; 6];
            if r[0] & 0x40 == 0 {
                b[0] = rel[3];
                b[1] = rel[0];
                b[2] = rel[1];
                b[3] = rel[2];
            } else {
                // basis' = M1 * x + M2 * y + M3 * z (vmula/vmadda/vmadd bc), translation' = M0 + that
                let xf = |v: [f32; 4]| -> [f32; 4] { std::array::from_fn(|k| m[1][k] * v[0] + m[2][k] * v[1] + m[3][k] * v[2]) };
                b[1] = xf(rel[0]);
                b[2] = xf(rel[1]);
                b[3] = xf(rel[2]);
                let t = xf(rel[3]);
                b[0] = std::array::from_fn(|k| m[0][k] + t[k]);
                b[4] = m[4];
                b[5] = m[5];
            }
            h.vcall_19c(c, &q, &b);
        }
    }
}
