//! WP-J15: class C2dc018 (the world SW 0x3dd490, vtable 0x2dc018, parent C2c94a0, ctor 0x20d9d0) slots and
//! the world update FUN_00226550 (slot 6) with its own callees.
//!
//! Slots: 0 0x132b00 / 1 0x132b08 (empty, inherited), 2 0x1331f8 / 3 0x133508 (inherited, anim::sm), 4
//! 0x133590 (inherited, classes::k13), 5 0x113a20 (class info 0x2baafc), 6 0x226550 (here), 7 0x1cda80 /
//! 8 0x157fa0 (render, main_loop `RenderWorld` / `RenderPost`, gfx/).
//!
//! FUN_00226550 is 1:1 as an ordered step list (`WorldStep`, `ee_sub_226550`) over `WorldUpdateHost`; the
//! steps owned by other modules are host calls (the Bevy app runs those systems in this order, see
//! main_loop `WorldUpdate`). 1:1 kernels here: 0x2267c0 (paused pass flag masks), 0x147f40 / 0x147ec8 /
//! 0x147ee8 (camera / focus proximity list world+0x20, `ProxList`), 0x19f120 / 0x19f170 (world+0x4748
//! alarm timeout, 6 s), 0x156728 (timed objects world+0x3ffc), 0x20f7e0 (the four world+0x3ba0 slots:
//! slide-in offsets 0 / 65 / 130 / 195 and the tick-tock sounds 0x30 / 0x31 of the world+0x476c
//! countdown), 0x1be1e8 / 0x1ec968 (queue drains as host loops).
//! REPLACED: the DAT_002dd2fc safe-iterator frames -> index iteration over `Vec`s.
#![allow(non_snake_case)]

use crate::engine::classes::k14::FUN_0011d1c0;

/// FUN_00113a20 (slot 5): class info.
pub const CLASS_INFO: u32 = 0x002b_aafc;
/// DAT_002bd450: proximity epsilon (camera movement / boundary distance).
pub const PROX_EPS: f32 = 500.0;
/// DAT_002bf938: world+0x4748 alarm duration.
pub const ALARM_TIME: f32 = 6.0;
/// DAT_002c3e80: 0x3ba0 slot spacing.
pub const SLOT_STEP: f32 = 65.0;
/// 0x2c3e88: FUN_0011d1c0 table of the 0x3ba0 slots.
pub const SLOT_SMOOTH: [f32; 4] = [500.0, 0.0, 0.25, 2500.0];

/// FUN_00132b00 / FUN_00132b08 (slots 0 / 1).
pub fn FUN_00132b00() {}

/// FUN_00113a20 (slot 5).
pub fn FUN_00113a20() -> u32 {
    // 0x00113a20
    CLASS_INFO
}

/// The FUN_00226550 calls in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldStep {
    /// FUN_00149c80: due Splice callbacks (vm_state::ee_sub_149c80)
    Scripts,
    /// FUN_00147f40(sw): proximity list (`ProxList::FUN_00147f40`)
    Proximity,
    /// FUN_002267c0(sw): paused update (dt <= 0)
    Paused,
    /// FUN_001c4468 for each world+0x3eac (+0x84 next) object: pad-driven objects
    PadObjects,
    /// FUN_001f77c0(sw): hazard records (player::p8_hazard)
    Hazards,
    /// world+0x3f20 entries (count +0x3f14) with flag 2: vt+0x84 update
    ObjectUpdate,
    /// FUN_0019f120(sw): alarm timeout
    Alarm,
    /// FUN_00130918 per world+0x3d88 ASEGA (anim::play)
    Asegas,
    /// FUN_00156728 per world+0x3ffc object
    Timed,
    /// FUN_001f8dd8(sw): physics (physics::driver)
    Physics,
    /// FUN_001be1e8: 0x3e1a4c queue -> FUN_001bd040
    Queue1be1e8,
    /// FUN_001ec968: DAT_003e159c list (+0x40 next, stops at +0x30 != 0) -> vt+0x44 when FUN_001eda20
    List1ec968,
    /// FUN_001ab1a0(0x2e65e0): card manager dialog
    Card,
    /// FUN_001fd830(0x3dd040): music / sound
    Music,
    /// FUN_0014d478(g_cm) when dt_real > 0 (camera::cm)
    Camera,
    /// FUN_0020f7e0(sw): the 0x3ba0 slots
    Slots,
    /// FUN_00128148(0x2dd598): REPLACED (pool statistics)
    PoolStats,
}

pub trait WorldUpdateHost {
    fn step(&mut self, s: WorldStep);
}

/// FUN_00226550 (slot 6): `dt` = DAT_002dded4, `dt_real` = DAT_002ddee0.
pub fn ee_sub_226550(h: &mut impl WorldUpdateHost, dt: f32, dt_real: f32) {
    // 0x00226550
    use WorldStep::*;
    h.step(Scripts);
    h.step(Proximity);
    if dt <= 0.0 {
        h.step(Paused);
    } else {
        for s in [PadObjects, Hazards, ObjectUpdate, Alarm, Asegas, Timed, Scripts, Physics, Queue1be1e8, List1ec968, Scripts] {
            h.step(s);
        }
    }
    h.step(Card);
    h.step(Music);
    if 0.0 < dt_real {
        h.step(Camera);
    }
    h.step(Slots);
    h.step(PoolStats);
}

/// FUN_00226550 world+0x3f20 entry flag test (vt+0x84 update).
pub fn update_flag(f: u32) -> bool {
    f & 2 != 0
}

/// FUN_002267c0 passes over the 0x3f20 entries / ASEGAs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PausedPass {
    /// flags & 0x102 == 0x102: vt+0x84 update
    Update,
    /// ASEGA (+0x14 +0x40 flags, low bit set = immediate else pointer) & 0x100: FUN_00130918
    Asega,
    /// flags & 0x124 == 0x104: vt+0x8c
    Paused,
}

/// FUN_002267c0 entry test of a pass (`Asega`: the ASEGA's flag word).
pub fn paused_flag(p: PausedPass, f: u32) -> bool {
    match p {
        PausedPass::Update => f & 0x102 == 0x102,
        PausedPass::Asega => f & 0x100 != 0,
        PausedPass::Paused => f & 0x124 == 0x104,
    }
}

// ---------------------------------------------------------------- 0x147f40

/// One world+0x20 node: object, (+0x28 << 8) >> 0x3c kind (signed 4 bits), +0x2d mask, +0x38 radius.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProxEntry {
    pub obj: u32,
    pub kind: i8,
    pub mask: u8,
    pub radius: f32,
}

/// World proximity state: +0x20 list, +0x2c active set, +0x39a4 stop node, +0x39a8 mask, +0x39b0 camera /
/// +0x39c0 focus snapshot.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProxList {
    pub list: Vec<ProxEntry>,
    pub active: Vec<u32>,
    pub stop: Option<u32>,
    pub mask: u32,
    pub cam: [f32; 3],
    pub focus: [f32; 3],
}

pub trait ProxHost {
    /// FUN_001a62e8(obj): position (vt+0xfc +0x30)
    fn pos(&self, obj: u32) -> [f32; 3];
    /// vt+0xbc: enter (active)
    fn enter(&mut self, obj: u32);
    /// vt+0xb4: leave
    fn leave(&mut self, obj: u32);
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

impl ProxList {
    /// FUN_00147f40(sw): `mask` = DAT_002ddf54, `cam` = DAT_002ddfd0 (g_cm position), `focus` = FUN_001a62e8
    /// of world+0x3efc (FUN_001c4840; None -> the camera).
    pub fn FUN_00147f40(&mut self, h: &mut impl ProxHost, mask: u32, cam: [f32; 3], focus: Option<[f32; 3]>) {
        // 0x00147f40
        let focus = focus.unwrap_or(cam);
        let dc = dist(cam, self.cam);
        let df = dist(focus, self.focus);
        if dc > PROX_EPS || df > PROX_EPS || mask != self.mask {
            self.stop = None;
            self.mask = mask;
            self.cam = cam;
            self.focus = focus;
        }
        let mut i = 0;
        let mut inside = false; // unaff_s4 (kept across nodes, as the original)
        while i < self.list.len() && Some(self.list[i].obj) != self.stop {
            let e = self.list[i];
            let mut advance = true;
            match e.kind {
                1 | 2 => {
                    inside = e.kind == 1;
                    // FUN_00147ec8: off the list
                    self.list.remove(i);
                    advance = false;
                }
                0 | 3 => {
                    if mask & e.mask as u32 == 0 {
                        inside = false;
                        if e.kind == 0 {
                            self.FUN_00147ee8(i);
                            advance = false;
                        }
                    } else {
                        let (c, m) = if e.kind == 0 { (cam, dc) } else { (focus, df) };
                        let d = dist(h.pos(e.obj), c) - e.radius;
                        inside = d < 0.0;
                        if PROX_EPS < d.abs() - m {
                            self.FUN_00147ee8(i);
                            advance = false;
                        }
                    }
                }
                _ => {}
            }
            // FUN_0011a380(+0x2c, node)
            let was = self.active.contains(&e.obj);
            if inside != was {
                if inside {
                    self.active.push(e.obj);
                    h.enter(e.obj);
                } else {
                    h.leave(e.obj);
                    self.active.retain(|&o| o != e.obj);
                }
            }
            if advance {
                i += 1;
            }
        }
    }

    /// FUN_00147ee8(sw, node): to the tail; the first moved node becomes the stop node.
    fn FUN_00147ee8(&mut self, i: usize) {
        // 0x00147ee8
        let e = self.list.remove(i);
        if self.stop.is_none() {
            self.stop = Some(e.obj);
        }
        self.list.push(e);
    }
}

// ---------------------------------------------------------------- 0x19f120 / 0x156728 / 0x20f7e0

/// FUN_0019f120(sw): world+0x4748 alarm on and older than 6 s -> FUN_0019f170(sw, 0). `t` = DAT_002dded0.
pub fn FUN_0019f120(alarm: &mut (i32, f32), t: f32, h: &mut impl FnMut(i32)) {
    // 0x0019f120
    if alarm.0 != 0 && ALARM_TIME < t - alarm.1 {
        FUN_0019f170(alarm, t, 0, h);
    }
}

/// FUN_0019f170(sw, on): +0x4748 = on, +0x474c = now; the class-0x105 objects (FUN_0015e9c8, max 0x20) get
/// FUN_00207030(o, 0xe) (on) / FUN_00207030(o, -1) when their +0xfbc state is 0xe (off): `h(on)`.
pub fn FUN_0019f170(alarm: &mut (i32, f32), t: f32, on: i32, h: &mut impl FnMut(i32)) {
    // 0x0019f170
    *alarm = (on, t);
    h(on);
}

/// FUN_00156728(o): o+0x14 +0x60 duration >= 0 and o+0x4c start + duration < now -> FUN_00156780(o).
pub fn FUN_00156728(duration: f32, start: f32, t: f32) -> bool {
    // 0x00156728
    0.0 <= duration && start + duration < t
}

/// One world+0x3ba0 slot (0xc bytes): +0 object, +4 speed, +8 reset.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Slot {
    pub obj: u32,
    pub v: f32,
    pub reset: bool,
}

pub trait SlotHost {
    /// FUN_002132d0(o): o+0x7c != -1 and FUN_002137f0() == 0
    fn shown(&self, obj: u32) -> bool;
    /// o+0xf8 -> +0x14
    fn offset(&mut self, obj: u32) -> &mut f32;
    /// FUN_001fb2f8(FUN_001fbb90(id), 0)
    fn sound(&mut self, id: u32);
}

/// FUN_0020f7e0(sw): `left` = world+0x476c (countdown end), `shown_secs` = +0x4770.
pub fn FUN_0020f7e0(slots: &mut [Slot; 4], h: &mut impl SlotHost, t: f32, dt: f32, left: f32, shown_secs: &mut u32) {
    // 0x0020f7e0
    let mut target = 0.0;
    for (i, s) in slots.iter_mut().enumerate() {
        if s.obj == 0 || !h.shown(s.obj) {
            s.reset = true;
            continue;
        }
        let off = h.offset(s.obj);
        if !s.reset {
            *off = FUN_0011d1c0(*off, s.v, target, dt, SLOT_SMOOTH, &mut s.v);
        } else {
            *off = target;
            s.v = 0.0;
            s.reset = false;
        }
        target += SLOT_STEP;
        if i == 3 {
            let r = left - t;
            let secs = if 0.0 < r { r as i32 as u32 } else { 0 };
            if secs != *shown_secs {
                h.sound(if secs & 1 != 0 { 0x30 } else { 0x31 });
                *shown_secs = secs;
            }
        }
    }
}

// ---------------------------------------------------------------- WP-J16: 0x1c4468 (pad objects)

/// One world+0x3eac pad object record (the JT "JOY" +0x150 and the other pad readers), 1:1 layout.
#[derive(Clone, Debug, PartialEq)]
pub struct JoyRec {
    /// +0x10 / +0x14 left stick, +0x18 angle, +0x1c magnitude
    pub l: [f32; 2],
    pub l_angle: f32,
    pub l_mag: f32,
    /// +0x20 / +0x24 right stick, +0x28 angle, +0x2c magnitude
    pub r: [f32; 2],
    pub r_angle: f32,
    pub r_mag: f32,
    /// +0x30 held buttons
    pub held: u32,
    /// +0x40..+0x6c FUN_0011b720(camera yaw, pitch) rows 0..2
    pub mat: [[f32; 4]; 3],
    /// +0x70 / +0x74 stick magnitude hold (decays 0.2 per update)
    pub l_hold: f32,
    pub r_hold: f32,
    /// +0x78 previous held
    pub prev_held: u32,
    /// +0x80 mode (0 = off, 1 = read the pad)
    pub mode: i32,
    /// +0x88 per-button press time (DAT_002dded0), consumed = -FLT_MAX (FUN_001c4738)
    pub press_t: [f32; 16],
    /// +0xc8 / +0xcc last time the left stick was idle / deflected, +0xd0 / +0xd4 right
    pub l_t: [f32; 2],
    pub r_t: [f32; 2],
}

impl Default for JoyRec {
    fn default() -> Self {
        Self {
            l: [0.0; 2],
            l_angle: 0.0,
            l_mag: 0.0,
            r: [0.0; 2],
            r_angle: 0.0,
            r_mag: 0.0,
            held: 0,
            mat: [[0.0; 4]; 3],
            l_hold: 0.0,
            r_hold: 0.0,
            prev_held: 0,
            mode: 0,
            press_t: [crate::engine::player::PRESS_NONE; 16],
            l_t: [0.0; 2],
            r_t: [0.0; 2],
        }
    }
}

/// Pad record fields FUN_001c4468 copies (0x2dfc00, or 0x2e0d00 when DAT_002dfbc0 & 2).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PadSnap {
    /// +0x103c / +0x1040
    pub left: [f32; 2],
    /// +0x1070 / +0x1074
    pub right: [f32; 2],
    /// +0x10b4 held, +0x10bc masked
    pub held: u32,
    pub masked: u32,
}

impl JoyRec {
    /// FUN_001c4468(joy). `forced` = joy+0x7c +0x154 (buttons forced on), `cam_fwd` = 0x2ddfa0 (g_cm
    /// forward), `t` = DAT_002dded0.
    pub fn ee_sub_1c4468(&mut self, pad: &PadSnap, forced: u32, cam_fwd: [f32; 3], t: f32) {
        // 0x001c4468
        let (mut yaw, mut pitch) = (0.0, 0.0);
        if self.mode == 0 {
            self.ee_sub_1c42e8();
        } else if self.mode == 1 {
            self.l = pad.left;
            self.r = pad.right;
            self.held = pad.held & !pad.masked | forced;
            (yaw, pitch) = crate::engine::camera::math::FUN_0011bc88(cam_fwd);
        }
        let m = crate::engine::camera::math::FUN_0011b720(yaw, pitch);
        self.mat = [m[0], m[1], m[2]];
        let (mut l, mut la, mut lm, mut lh) = (self.l, self.l_angle, self.l_mag, self.l_hold);
        FUN_001c4330(yaw, &mut l, &mut la, &mut lm, &mut lh);
        (self.l, self.l_angle, self.l_mag, self.l_hold) = (l, la, lm, lh);
        let (mut r, mut ra, mut rm, mut rh) = (self.r, self.r_angle, self.r_mag, self.r_hold);
        FUN_001c4330(yaw, &mut r, &mut ra, &mut rm, &mut rh);
        (self.r, self.r_angle, self.r_mag, self.r_hold) = (r, ra, rm, rh);
        self.ee_sub_1c45a8(t);
    }

    /// FUN_001c42e8: zero sticks / held, consume every press (FUN_001c4738(joy, 0xffff)).
    pub fn ee_sub_1c42e8(&mut self) {
        // 0x001c42e8
        self.held = 0;
        self.l = [0.0; 2];
        self.l_angle = 0.0;
        self.l_mag = 0.0;
        self.r = [0.0; 2];
        self.r_angle = 0.0;
        self.r_mag = 0.0;
        self.press_t = [crate::engine::player::PRESS_NONE; 16];
    }

    /// FUN_001c45a8: newly pressed buttons get the press time; stick idle / deflected times.
    pub fn ee_sub_1c45a8(&mut self, t: f32) {
        // 0x001c45a8
        let mut new = self.held & !self.prev_held;
        self.prev_held = self.held;
        let mut k = 0;
        while new != 0 && k < 32 {
            if new & 1 != 0 && k < 16 {
                self.press_t[k] = t;
            }
            new = ((new as i32) >> 1) as u32;
            k += 1;
        }
        self.l_t[(0.0 < self.l_mag) as usize] = t;
        self.r_t[(0.0 < self.r_mag) as usize] = t;
    }
}

/// FUN_001c4330(yaw, joy, &x, &y, &angle, &mag, &hold): stick angle in world yaw; a released stick keeps
/// its angle while the magnitude hold decays by 0.2 per update.
pub fn FUN_001c4330(yaw: f32, xy: &mut [f32; 2], angle: &mut f32, mag: &mut f32, hold: &mut f32) {
    // 0x001c4330
    *mag = (xy[0] * xy[0] + xy[1] * xy[1]).sqrt();
    let d = *hold - 0.2;
    *hold = if 0.0 < d { d } else { 0.0 };
    if *hold <= *mag && *mag != 0.0 {
        *angle = crate::engine::camera::math::FUN_0011cf60((xy[1].atan2(xy[0]) + yaw) - 1.5707964);
        *hold = *mag;
        return;
    }
    let (s, c) = crate::engine::camera::math::FUN_0011d608((*angle - yaw) + 1.5707964);
    *mag = *hold;
    xy[0] = *hold * c;
    xy[1] = *mag * s;
}

// ---------------------------------------------------------------- WP-J16: 0x1be1e8 / 0x1ec968

/// FUN_001be1e8: drain DAT_003e1a4c (FUN_0011a2e8 removes the head) and FUN_001bd040 each object.
pub fn ee_sub_1be1e8(queue: &mut std::collections::VecDeque<u32>, mut f: impl FnMut(u32)) {
    // 0x001be1e8
    while let Some(o) = queue.pop_front() {
        f(o);
    }
}

/// One DAT_003e159c node: +0x30 stop flag, +0x10 update stamp.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StampNode {
    pub obj: u32,
    pub stop: i32,
    pub stamp: f32,
}

/// FUN_001ec968: walk DAT_003e159c (+0x40 next) until a node with +0x30 != 0; nodes whose stamp
/// FUN_001eda20 renews (+0x10 != DAT_002dded0) get vt+0x44 (`f`).
pub fn ee_sub_1ec968(list: &mut [StampNode], t: f32, mut f: impl FnMut(u32)) {
    // 0x001ec968
    for n in list.iter_mut() {
        if n.stop != 0 {
            break;
        }
        // FUN_001eda20
        if n.stamp != t {
            n.stamp = t;
            f(n.obj);
        }
    }
}

// ---------------------------------------------------------------- WP-J16: 0x1ab1a0 (card manager)

/// Card manager 0x2e65e0 state FUN_001ab1a0 reads / writes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CardMgr {
    /// +8 state, +0xc target state
    pub state: i32,
    pub target: i32,
    /// +0x10 last dialog time, +0x14 next poll time
    pub t_10: f32,
    pub t_14: f32,
    /// +0xfd5c +4 thread id (-1 = idle; FUN_001ab088)
    pub thread: i32,
}

/// Engine calls of FUN_001ab1a0.
pub trait CardHost {
    /// FUN_001aaf90(+0xfd5c, DAT_002a6ba0): start the card poll thread
    fn poll(&mut self, c: &mut CardMgr);
    /// FUN_001ab630
    fn finish(&mut self, c: &mut CardMgr);
    /// FUN_001ab418: step towards +0xc
    fn step(&mut self, c: &mut CardMgr);
    /// FUN_001b42c0(0x2f66b0): a dialog is up
    fn dialog_up(&self) -> bool;
    /// FUN_001b01e8(0x3d9ed0)
    fn card_ok(&self) -> bool;
    /// FUN_001acb58(0x2e6550, msg)
    fn show(&mut self, msg: i32);
}

/// FUN_001ab1a0(0x2e65e0). `busy` = FUN_0020e040(0x3dd490) || DAT_002deb4c, `mode` = DAT_002e6594,
/// `now` = DAT_002ddf0c.
pub fn ee_sub_1ab1a0(c: &mut CardMgr, h: &mut impl CardHost, busy: bool, mode: i32, now: f32) {
    // 0x001ab1a0
    if busy || mode == 3 || c.thread != -1 {
        return;
    }
    if c.state == c.target {
        if c.t_14 <= now {
            h.poll(c);
        } else {
            h.finish(c);
        }
    } else {
        h.step(c);
    }
    if c.thread != -1 || h.dialog_up() || mode != 1 || now - c.t_10 < 0.3 {
        return;
    }
    if !h.card_ok() {
        match c.state {
            1 => h.show(0x11),
            2 => h.show(0x12),
            3 => h.show(0x15),
            4 => h.show(0x18),
            _ => {}
        }
    } else if c.state != 6 {
        h.show(0x2e);
    }
}

// ---------------------------------------------------------------- WP-J16: 0x20f710 (slot objects)

/// DAT_002c3e70: oids of the four world+0x3ba0 slot objects.
pub const SLOT_OIDS: [i32; 4] = [0x371, 0x36d, 0x36e, 0x36f];

/// FUN_0020f710(sw): slot i = FUN_0015e938(0x3dd490, 0x405, DAT_002c3e70[i], 0) when found (then
/// FUN_00213c68(o, {1, 0, 0, 1, ..}); its screen-element setup is not modelled). Slots not found keep
/// their object. The +0x4764 / +0x4768 copies (FUN_001c7d08 / FUN_001c7d00) are not modelled.
pub fn ee_sub_20f710(slots: &mut [Slot; 4], mut find: impl FnMut(i32) -> Option<u32>) {
    // 0x0020f710
    for (s, oid) in slots.iter_mut().zip(SLOT_OIDS) {
        if let Some(o) = find(oid) {
            s.obj = o;
        }
    }
}
