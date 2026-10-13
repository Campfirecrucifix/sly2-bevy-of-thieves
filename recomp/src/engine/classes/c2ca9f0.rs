//! C2ca9f0 (world class 44, parent C2c9a80 ALO): registers itself as a kind-10 camera request
//! (FUN_0014e328 / FUN_0014e4c8 on the camera 0x2ddf40, descriptor 0x2de3e8) while in the world, after load.
//! Handles in hooks = index + 1.
use super::ClassHost;

#[derive(Clone, Debug, Default)]
pub struct C2ca9f0 {
    /// +0x28 bit 60: in the world
    pub in_world: bool,
    /// +0x100 / +0x110 (DAT_002a2da0 = (1,0,0,0), DAT_002a2dc0 = (0,0,1,0))
    pub f_100: [f32; 4],
    pub f_110: [f32; 4],
    /// +0x124 load-post done
    pub f_124: i32,
    /// +0x12c counter (slots 43 / 44)
    pub f_12c: i32,
    /// K30: +0xe0 look-at object (field 1055, oid resolved; handle, 0 = none), +0x120 cut on deactivate
    /// (field 1059), +0x128 focal length (slot 42 address, no loader field), +0x130 fov (field 1054)
    pub f_e0: u32,
    pub f_120: i32,
    pub f_128: f32,
    pub f_130: f32,
}

/// slot 5: FUN_00113cd0
pub fn ee_sub_113cd0() -> u32 {
    0x2bae58
}

pub const DESC_2DE3E8: u32 = 0x2de3e8;

impl C2ca9f0 {
    /// K30: loader fields 1054 / 1055 / 1059 (FIELD_MEMBER +0x130 / +0xe0 / +0x120)
    pub fn feed(&mut self, obj: usize, props: &[crate::engine::world::props::Prop], h: &mut dyn ClassHost) {
        use crate::engine::world::props::PropValue;
        for p in props {
            let PropValue::Scalar(b) = &p.value else { continue };
            match p.field {
                1054 => self.f_130 = super::k4::f32_of(b),
                1055 => self.f_e0 = h.ee_sub_15e938(1, super::k4::word(b) as i32, obj as u32 + 1),
                1059 => self.f_120 = super::k4::word(b) as i32,
                _ => {}
            }
        }
    }

    /// K30: the state the 0x2de3e8 policy (C2cab90, classes/k30_cam.rs) reads through its +0x10 object,
    /// published while the camera request can be registered (in the world, after load-post).
    pub fn k30_publish(&self, obj: usize, h: &mut dyn ClassHost) {
        if !(self.in_world && self.f_124 != 0) {
            return;
        }
        let me = obj as u32 + 1;
        let st = super::k30_cam::ObjCam {
            rows: h.ee_sub_1a6318(me),
            pos: h.ee_sub_1a62e8(me),
            target: (self.f_e0 != 0).then(|| h.ee_sub_1a62e8(self.f_e0)),
            f_120: self.f_120,
            f_128: self.f_128,
            f_12c: self.f_12c,
            f_130: self.f_130,
        };
        h.k30_cam_state(me, super::k30_cam::CamState::Obj(st));
    }

    /// slot 6: FUN_0012e600
    pub fn ee_sub_12e600(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x231100, obj);
        self.f_100 = [1.0, 0.0, 0.0, 0.0];
        self.f_110 = [0.0, 0.0, 1.0, 0.0];
    }

    /// slot 10: FUN_0012e6a0
    pub fn ee_sub_12e6a0(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5800, obj);
        self.in_world = true;
        if self.f_124 != 0 {
            h.ee_sub_14e328(10, DESC_2DE3E8, 0, obj as u32 + 1);
        }
    }

    /// slot 11: FUN_0012e6e8
    pub fn ee_sub_12e6e8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5848, obj);
        self.in_world = false;
        if self.f_124 != 0 {
            h.ee_sub_14e4c8_k(0xf, 10, DESC_2DE3E8, 0, obj as u32 + 1);
        }
    }

    /// slot 14: FUN_0012e648
    pub fn ee_sub_12e648(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a5c48, obj);
        self.f_124 = 1;
        if self.in_world {
            h.ee_sub_14e328(10, DESC_2DE3E8, 0, obj as u32 + 1);
        }
    }

    /// slot 42: FUN_0012e738 (address of +0x128)
    pub fn ee_sub_12e738(obj_addr: u32) -> u32 {
        obj_addr + 0x128
    }

    /// slot 43: FUN_0012e740
    pub fn ee_sub_12e740(&mut self) {
        self.f_12c += 1;
    }

    /// slot 44: FUN_0012e750
    pub fn ee_sub_12e750(&mut self) {
        self.f_12c -= 1;
    }
}
