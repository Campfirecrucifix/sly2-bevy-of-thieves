//! WP-K23: class 102 C2db8a0 (parent class 101 C2c9810, the thrown bomb, classes/k15_101.rs; 42 instances, one
//! template per level): the bomb variant that lifts a guard. Row 1113 = +0x1b0 anchor object (the bomb itself
//! when 0). Entering state 5 (slot 75 FUN_00225660) picks the nearest class-0x38 object within 400 that
//! FUN_00206eb0(o, 0xb) accepts, links the record +0x1b8 {self, guard} into the guard's +0x138 list
//! (FUN_00229710), halves its horizontal velocity and gives it z speed 400, and sends brain message 6
//! (FUN_00209e28). While held, slot 70 (FUN_002255c8, run by the guard's physics per record) cancels the guard's
//! gravity and springs it to the anchor; slot 15 (FUN_00225300) drives the guard's hold component (+0x26c kind
//! 0xb0: FUN_0017a8c8 state 4 once within 100), shrinks its +0xfc4 constraint scale over 3 s and then knocks
//! it out (FUN_001c6fd8 kind 0x5d, guard vtable +0x4c, FUN_001619a0(0x18)) and goes to state 3 (fuse).
//! Leaving state 5 unlinks the record (FUN_00229738), sets the component to 2, restores the scale {1,1,1}.
//! Own slots 5 FUN_00114070, 6 FUN_002252e0, 15 FUN_00225300, 70 FUN_002255c8, 75 FUN_00225660 1:1; the
//! parent's virtual slot-75 calls (FUN_00224068 / 224190 / 2241d8 / 224258) dispatch here (`C2c9810::k23`).
//! REPLACED: the guard's +0x138 record walk calling slot 70 -> `ClassSet::frame_k15_101` calls it after slot
//! 15 while a guard is held; FUN_001f5e10's FUN_0022fcc8 parent frame -> world frame (as player::p5_states).
//! Guard internals behind `ClassHost::k23_*` hooks (component 0xb0, FUN_00209840, +0xfc4, FUN_00206eb0,
//! FUN_00209e28, FUN_00229710, FUN_001c6fd8 (1644 B, unported)).
use super::k15_101::{C2c9810, Cx};
use super::ClassHost;
use crate::engine::player::p5_states::FUN_001f5e10;

/// slot 5 FUN_00114070
pub fn ee_sub_114070() -> u32 {
    0x2bb2e0
}

/// row 1113 -> +0x1b0
pub const ROW_1113: i16 = 1113;
/// DAT_002c43b0: hold time before the knock-out
pub const D_2C43B0: f32 = 3.0;
/// DAT_002c43c0..c8: constraint scale polynomial
pub const P_2C43C0: [f32; 3] = [1.2, 0.0, -1.0];
/// DAT_002bbb60 / 64: [0, 1]
pub const R_2BBB60: [f32; 2] = [0.0, 1.0];
/// DAT_002c43f0 / f4: grab velocity scale and z speed
pub const D_2C43F0: f32 = 0.5;
pub const D_2C43F4: f32 = 400.0;
/// grab radius (400.0 literal)
pub const GRAB_R: f32 = 400.0;
/// 0x2c43d0 position spring {0, 15, 0}, 0x2c43e0 velocity damping {0, 2.5, 0}, DAT_002a2dd0 target velocity 0
pub const P_2C43D0: [f32; 3] = [0.0, 15.0, 0.0];
pub const P_2C43E0: [f32; 3] = [0.0, 2.5, 0.0];
/// _DAT_002a2de0 {1, 1, 1}
pub const V_2A2DE0: [f32; 3] = [1.0, 1.0, 1.0];

/// the class-102 members past the class-101 block
#[derive(Clone, Debug, Default)]
pub struct C2db8a0 {
    /// +0x1b0 anchor object (row 1113)
    pub f_1b0: u32,
    /// +0x1b4 held guard
    pub f_1b4: u32,
    /// +0x1b8 record {+0x1b8 owner, +0x1bc guard, +0x1c0 next}
    pub f_1b8: u32,
    pub f_1bc: u32,
    /// guards grabbed / knocked out (tests)
    pub grabs: u32,
    pub knockouts: u32,
}

fn me(obj: usize) -> u32 {
    obj as u32 + 1
}
fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

impl C2db8a0 {
    /// row 1113 (object reference)
    pub fn feed(&mut self, props: &[crate::engine::world::props::Prop], obj: usize, h: &mut dyn ClassHost) {
        use crate::engine::world::props::PropValue;
        for p in props {
            let PropValue::Scalar(b) = &p.value else { continue };
            if p.field == ROW_1113 {
                let w = super::k4::word(b);
                self.f_1b0 = if w != 0 { h.ee_sub_15e938(0x2002, w as i32, me(obj)) } else { 0 };
            }
        }
    }
    /// the anchor: +0x1b0, else the bomb
    pub fn anchor(&self, obj: usize) -> u32 {
        if self.f_1b0 == 0 { me(obj) } else { self.f_1b0 }
    }
}

/// slot 6 FUN_002252e0: FUN_00223f70
pub fn ee_sub_2252e0(b: &mut C2c9810, obj: usize, h: &mut dyn ClassHost) {
    b.ee_sub_223f70(obj, h);
}

/// slot 75 FUN_00225660(self, state)
pub fn ee_sub_225660(b: &mut C2c9810, obj: usize, s: i32, cx: &mut Cx) {
    if s == b.f_150 {
        return;
    }
    let mut e = b.k23.take().unwrap_or_default();
    if b.f_150 == 5 && e.f_1b4 != 0 {
        cx.h.ee_sub_229738(e.f_1b4, me(obj));
        if cx.h.k23_comp_b0(e.f_1b4).is_some() {
            cx.h.k23_comp_b0_set(e.f_1b4, 2);
        }
        let c = cx.h.k23_guard_fc4(e.f_1b4);
        if c != 0 {
            cx.h.k22_pin_12d108(c, V_2A2DE0);
        }
        e.f_1b4 = 0;
    }
    b.ee_sub_224ec8(obj, s, cx);
    if s == 5 {
        let p = cx.h.ee_sub_1a62e8(e.anchor(obj));
        // FUN_0015e9c8(0x3dd490, 0x105, 0x38, 0, 0x20, out)
        let mut best = GRAB_R;
        let mut g = 0;
        for o in cx.h.query_15e9c8(0x38).into_iter().take(0x20) {
            if cx.h.k23_guard_206eb0(o, 0xb) {
                let d = dist(cx.h.ee_sub_1a62e8(o), p);
                if d < best {
                    best = d;
                    g = o;
                }
            }
        }
        if g != 0 {
            e.f_1b4 = g;
            e.f_1b8 = me(obj);
            e.f_1bc = g;
            cx.h.k23_link_229710(g, me(obj));
            let v = cx.h.k22_speed_10c(g);
            cx.h.vcall_11c_v(g, [v[0] * D_2C43F0, v[1] * D_2C43F0, D_2C43F4]);
            cx.h.k23_brain_209e28(g, 0);
            e.grabs += 1;
        }
    }
    b.k23 = Some(e);
}

/// slot 15 FUN_00225300
pub fn ee_sub_225300(b: &mut C2c9810, obj: usize, cx: &mut Cx) {
    b.ee_sub_2240b0(obj, cx);
    let mut s0 = b.f_150;
    loop {
        if s0 == 5 {
            let e = b.k23.clone().unwrap_or_default();
            let mut next = 3;
            let mut held = false;
            if e.f_1b4 != 0 {
                if let Some(cs) = cx.h.k23_comp_b0(e.f_1b4) {
                    // FUN_0017a9c0: component state -1
                    if cs != -1 {
                        held = true;
                        let a = e.anchor(obj);
                        let m = cx.h.ee_sub_1a6318(a);
                        cx.h.k23_guard_209840(e.f_1b4, m);
                        if cs != 4 && dist(cx.h.ee_sub_1a62e8(a), cx.h.ee_sub_1a62e8(e.f_1b4)) < 100.0 {
                            cx.h.k23_comp_b0_set(e.f_1b4, 4);
                        }
                        let c = cx.h.k23_guard_fc4(e.f_1b4);
                        if c != 0 {
                            let f = (cx.t - b.f_154) / D_2C43B0;
                            let g = P_2C43C0[0] + f * (P_2C43C0[1] + f * P_2C43C0[2]);
                            let k = if R_2BBB60[0] <= g { if R_2BBB60[1] < g { R_2BBB60[1] } else { g } } else { R_2BBB60[0] };
                            cx.h.k22_pin_12d108(c, [k, k, k]);
                        }
                        if D_2C43B0 < cx.t - b.f_154 {
                            let p = cx.h.ee_sub_1a62e8(e.f_1b0);
                            cx.h.k23_world_1c6fd8(0x5d, p, me(obj), 1);
                            cx.h.vcall(e.f_1b4, 0x4c);
                            cx.h.ee_sub_1619a0(0x18, e.f_1b4);
                            if let Some(x) = b.k23.as_mut() {
                                x.knockouts += 1;
                            }
                            b.slot75(obj, 3, cx);
                        }
                    }
                }
                if !held {
                    next = 2;
                }
            }
            if !held {
                b.slot75(obj, next, cx);
            }
        }
        if b.f_150 == s0 {
            return;
        }
        s0 = b.f_150;
    }
}

/// slot 70 FUN_002255c8(self, rec): the held guard (rec +4): gravity cancelled (FUN_001fac30(-g)), sprung to the
/// anchor (FUN_001f5e10(guard, anchor, {0,15,0}, 0, {0,2.5,0}))
pub fn ee_sub_2255c8(b: &C2c9810, obj: usize, dt: f32, h: &mut dyn ClassHost) {
    let Some(e) = b.k23.as_ref() else { return };
    let g = e.f_1bc;
    let Some(gi) = (g as usize).checked_sub(1) else { return };
    let gr = h.phys_e8_80(gi);
    h.ee_sub_1fac30(gi, [-gr[0], -gr[1], -gr[2]]);
    let tp = h.ee_sub_1a62e8(e.anchor(obj));
    let pos = h.ee_sub_1a62e8(g);
    let v = h.k22_speed_10c(g);
    let a = FUN_001f5e10(pos, v, tp, Some(P_2C43D0), [0.0; 3], Some(P_2C43E0), dt);
    h.ee_sub_1fac30(gi, a);
}
