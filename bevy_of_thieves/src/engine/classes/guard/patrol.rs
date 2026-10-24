//! Guard patrol task chain (goal stack at guard +0x26c, R2): 0xaa routine -> 0xa9 script -> 0xc0 rail patrol ->
//! 0xba goto point -> 0xa7 walk path (carrot), plus the yaw / root-motion / turn-slow parts of the guard update.
//! Each task keeps the fields its functions touch (named by offset). Tasks are owned by their parent
//! (the original's +0x18 child pointer); "kill child" = drop.
use super::super::ClassHost;
use crate::engine::camera::math::FUN_0011d2d0;
use crate::engine::player::p5_attach::{FUN_001e6620, FUN_001e6648, FUN_001e66c8, FUN_001e6ed8, FUN_001e6f10, FUN_001e6f60, FUN_0011cf60, Path};
use crate::engine::player::p6::FUN_001e6180;

type V3 = [f32; 3];
fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sc(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn len(a: V3) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}
/// remove the DAT_002a2dc0 (0,0,1) component
fn flat(a: V3) -> V3 {
    [a[0], a[1], 0.0]
}
/// normalize, zero below 1e-4 (DAT_002a2da0)
fn nrm(a: V3) -> V3 {
    let l = len(a);
    if l < 1e-4 { [0.0; 3] } else { sc(a, 1.0 / l) }
}

// DAT_002be6xx / 7xx (values read from the ELF, see K6: Findings)
const D_624: f32 = 25.0;
const D_628: f32 = 60.0;
const D_62C: f32 = 100.0;
const D_630: f32 = 500.0;
const D_634: f32 = 100.0;
const D_640: f32 = 200.0;
const D_644: f32 = -0.5;
const D_648: f32 = 0.0;
const D_650: f32 = 0.0;
const D_654: f32 = 150.0;
const D_658: f32 = 7.5;
const D_65C: f32 = 30.0;
const D_694: f32 = 1.5;
const D_6A0: f32 = 0.5;
const D_6A4: f32 = 1.0 / 300.0;
const D_6A8: f32 = 0.0;
const D_6B0: f32 = 25.0;
const D_758: f32 = 5.0;
const D_7B8: f32 = 10.0;
const D_7BC: f32 = 100.0;
const D_7C0: f32 = 250.0;
const D_7C4: f32 = 5.0;
/// DAT_002c3c80/84/88, clamp DAT_002bbb60/64
const D_C80: f32 = 1.5;
const D_C84: f32 = -1.0;
const D_C88: f32 = 0.0;
/// guard +0x1f4 yaw SMP (FUN_00204d48)
pub const YAW_SMP: [f32; 4] = [25.0, 0.0, 0.25, 1.0];

/// The guard state the tasks read / write (guard object fields).
#[derive(Clone, Debug, Default)]
pub struct Body {
    /// CRAWL R97: guard +0xfc8 (alarm), copied by the guard layer each frame for FUN_00209098
    pub f_fc8: i32,
    /// FUN_001a62e8 world position
    pub pos: V3,
    /// BOSS: FUN_001c0d70(target, 1) of the brain target and whether it equals the guard's own region
    /// (+0xe8 -> +0xdc -> +0x10; REPLACED: FUN_001c0d70(guard pos, 1)); set by the guard layer each frame
    pub tgt_region: i32,
    pub region_diff: bool,
    /// BOSS: guard +0x31c / +0x324 (0xb3 reposition: goal distance range / keep range), set by the guard layer
    pub f_31c: [f32; 2],
    pub f_324: [f32; 2],
    /// current body yaw (atan2 of the vslot 0xfc matrix x row)
    pub yaw: f32,
    /// +0x1a8 target yaw (FUN_00204660)
    pub f_1a8: f32,
    /// +0x180 turn speed factor (FUN_00203fc0)
    pub f_180: f32,
    /// +0x190 local root-motion velocity (FUN_00209618 stores -v; kept as +v here)
    pub f_190: V3,
    /// +0xfe8 anim capability bits (slot 25)
    pub f_fe8: u32,
    /// is-a 0x3a
    pub c3a: bool,
    /// K24: the REPLACED lift integrator is flying the body (`Guard::po`; +0x208 ground contact = !k24_air)
    pub k24_air: bool,
    /// ANIM R27: guard +0x2cc / +0x2d0 routine run range (FUN_00205eb8: DAT_002a8600 = FLT_MAX) and +0x2d4 / +0x2d8
    /// routine idle range (FUN_00179e80 / FUN_00179ef8; written by field 274 / 275 and routine kinds 1 / 2)
    pub f_2cc: [f32; 2],
    pub f_2d4: [f32; 2],
    /// ANIM R27: FUN_0011d448 state (ClassSet::rng, lent to the guard for its tick)
    pub rng: crate::engine::physics::driven::GameRng,
}

/// Per-call context.
pub struct Cx<'a> {
    pub guard: usize,
    pub now: f32,
    pub dt: f32,
    pub body: &'a mut Body,
    pub h: &'a mut dyn ClassHost,
}

/// FUN_00209818(yaw, guard + 0xfd0) -> FUN_00204660 (fd0[1] == 0): +0x1a8 = yaw (rows rebuilt by FUN_0011b500)
pub fn ee_sub_209818(b: &mut Body, yaw: f32) {
    b.f_1a8 = yaw;
}

/// FUN_00203fc0 (+0x1a4 == 0 part): +0x180 = clamp(1.5 - |a|, 0, 1), a = wrap(+0x1a8 - body yaw)
pub fn ee_sub_203fc0(b: &mut Body) {
    let a = FUN_0011cf60(b.f_1a8 - b.yaw).abs();
    b.f_180 = (D_C80 + a * (D_C84 + a * D_C88)).clamp(0.0, 1.0);
}

/// FUN_00204ca8: body yaw approaches +0x1a8 (FUN_0011d2d0 with the +0x1f4 SMP); then the slot-56 contact motor
/// (FUN_002044b0) / SO solver motion, reduced to pos += R(+0x1a8) * v190 * k180 * dt (R2 §3).
pub fn ee_sub_204ca8_move(b: &mut Body, dt: f32) {
    b.yaw = FUN_0011d2d0(b.yaw, b.f_1a8, dt, &YAW_SMP);
    let (s, c) = b.f_1a8.sin_cos();
    let v = b.f_190;
    let w = [c * v[0] - s * v[1], s * v[0] + c * v[1], v[2]];
    b.pos = add(b.pos, sc(w, b.f_180 * dt));
}

/// 0xa7 WALK-PATH task (vt 0x2d06a0, size 0xf0).
#[derive(Clone, Debug)]
pub struct Walk {
    /// +0x30 path (the guard +0x294 nav path)
    pub path: Path,
    /// +0x34 u, +0x38 dir, +0x3c finite, +0x40 no-avoid, +0x44 end u, +0x4c state
    pub u: f32,
    pub dir: i32,
    pub finite: bool,
    pub no_avoid: bool,
    pub end_u: f32,
    pub st: i32,
    /// +0x64 arrive radius, +0x68 / +0x6c stop radii, +0x70 error
    pub r64: f32,
    pub r68: f32,
    pub r6c: f32,
    pub e70: f32,
    /// +0x80 carrot, +0x90 carrot velocity, +0xa0 reset flag, +0xb0 end point
    pub c80: V3,
    pub v90: V3,
    pub reset: bool,
    pub b0: V3,
    /// +0xc0 anim id, +0xc4 rate scale
    pub anim: i32,
    pub c4: f32,
    /// +0xd0 last position, +0xe0 stuck timer
    pub d0: V3,
    pub e0: f32,
    /// anim task 0x9d playback time (*(task +0x38) +0x1c), advanced at the FUN_00131a00 rate
    pub t_anim: f32,
}

impl Walk {
    /// FUN_00177e80 init + FUN_00178d90(t, path, dir)
    pub fn ee_sub_177e80(path: Path, dir: i32, pos: V3) -> Walk {
        let hit = FUN_001e6f60(&path, pos);
        let mut w = Walk {
            u: -1.0,
            dir,
            finite: false,
            no_avoid: false,
            end_u: 0.0,
            st: -1,
            r64: D_62C,
            r68: D_624,
            r6c: D_628,
            e70: 0.0,
            c80: pos,
            v90: [0.0; 3],
            reset: true,
            b0: [0.0; 3],
            anim: 9,
            // +0xc4: set by the goto setstate FUN_0017d450 (not read; 1.0 assumed)
            c4: 1.0,
            d0: pos,
            e0: D_694,
            t_anim: 0.0,
            path,
        };
        // FUN_00178d90
        w.u = hit.l;
        if !w.path.looped {
            w.finite = true;
            w.end_u = if dir != 0 { FUN_001e6620(&w.path) } else { FUN_001e6648(&w.path) };
            w.b0 = FUN_001e6f10(&w.path, w.end_u).0;
        }
        w
    }
    /// FUN_00178d00: arrived (ANIM R27: one vaddbc of the squared delta = x² + y², the flat distance; a rail
    /// point above / below the ground position still counts as reached)
    pub fn ee_sub_178d00(&self, pos: V3) -> bool {
        self.finite && len(flat(sub(pos, self.b0))) <= self.r64
    }
    /// FUN_00177ef0 start
    pub fn ee_sub_177ef0(&mut self, cx: &mut Cx) {
        self.c80 = cx.body.pos;
        let r = self.r64.max(D_62C);
        self.r6c = D_628.min(r);
        self.r68 = D_624.min(r * 0.25);
        self.ee_sub_1791c8(cx, 0);
    }
    /// FUN_001791c8 setstate: 0/1 walk anim (task 0x9d, +0x3c = +0xc0), 2-4 idle (0x9e)
    fn ee_sub_1791c8(&mut self, cx: &mut Cx, s: i32) {
        if s == self.st {
            return;
        }
        self.st = s;
        if s < 2 {
            self.d0 = cx.body.pos;
            self.e0 = D_694;
        }
    }
    /// FUN_00177f78: carrot (critically damped point running along the path)
    fn ee_sub_177f78(&mut self, cx: &mut Cx) {
        let pos = cx.body.pos;
        let total = FUN_001e6648(&self.path);
        let u0 = self.u + if self.dir != 0 { -D_634 } else { D_634 };
        // FUN_001e7380(wrap(u0), path, pos, ...): closest point near u0 (FUN_001e6ed8 walk)
        let hit = FUN_001e6ed8(FUN_001e66c8(&self.path, u0).clamp(0.0, total), &self.path, pos);
        self.u = hit.l;
        let p = hit.cp;
        let th = nrm(flat(hit.dir));
        let e = len(sub(p, pos));
        self.e70 = e;
        let mut s = (D_640 + e * (D_644 + e * D_648)).clamp(D_650, D_654);
        if self.dir != 0 {
            s = -s;
        }
        let mut k = D_630;
        let (t, mut tv);
        if !self.ee_sub_178d00(pos) {
            let step = s.clamp(-self.u, total - self.u);
            if step != s {
                k = 0.0;
            }
            t = add(p, sc(th, step));
            tv = th;
        } else {
            t = pos;
            tv = [0.0; 3];
            k = 0.0;
        }
        if self.dir != 0 {
            tv = sc(tv, -1.0);
        }
        let tv = sc(tv, k);
        if self.reset {
            self.v90 = tv;
            self.c80 = t;
            self.reset = false;
        }
        // spring-damper; the navmesh avoidance / clamp (FUN_001c0b50, navmesh +0x118) is not modelled
        let a = flat(add(sc(sub(tv, self.v90), D_658), sc(sub(t, self.c80), D_65C)));
        let dt = cx.dt;
        let c = add(add(self.c80, sc(self.v90, dt)), sc(a, 0.5 * dt * dt));
        self.v90 = add(self.v90, sc(a, dt));
        self.c80 = c;
    }
    /// FUN_00178ee0 update (FUN_001789f8 state machine, heading + rate, stuck timer)
    pub fn ee_sub_178ee0(&mut self, cx: &mut Cx) {
        self.ee_sub_177f78(cx);
        let pos = cx.body.pos;
        // FUN_001789f8 (door / obstacle check FUN_001bd718 not modelled)
        if self.st == 0 || self.st == 2 {
            let r = if self.st == 0 { self.r68 } else { self.r6c };
            let n = if len(sub(pos, self.c80)) <= r { 2 } else { 0 };
            self.ee_sub_1791c8(cx, n);
        }
        if self.st < 2 {
            let dv = flat(sub(self.c80, pos));
            let d = len(dv);
            let f = (D_6A0 + d * (D_6A4 + d * D_6A8)).clamp(0.0, 1.0);
            // FUN_00131a00(+0xc4 * f): playback rate -> FUN_00209618 root motion (anim task 0x9d per frame)
            let rate = self.c4 * f;
            self.t_anim += cx.dt * rate;
            cx.body.f_190 = cx.h.ee_sub_209618(cx.guard, self.anim, self.t_anim, rate);
            ee_sub_209818(cx.body, dv[1].atan2(dv[0]));
            if len(sub(pos, self.d0)) <= D_6B0 {
                self.e0 -= cx.dt;
            } else {
                self.d0 = pos;
                self.e0 = D_694;
            }
        } else {
            // idle task 0x9e: FUN_00174f08 state != 0 -> +0x190 = 0
            cx.body.f_190 = [0.0; 3];
        }
    }
}

/// 0xba GOTO-POINT task (vt 0x2cfe20, size 0xd0), point targets only (+0xb4 == 0).
#[derive(Clone, Debug)]
pub struct Goto {
    /// +0x30 state, +0x50 target point, +0xa0 last replan, +0xbc straight-line flag
    pub st: i32,
    pub f_50: V3,
    pub a0: f32,
    pub bc: bool,
    /// +0x18 child (0xa7)
    pub walk: Option<Walk>,
    /// ANIM R27: +0x40 target path (an is-a-10 object of a kind-7 command); FUN_001c1ec0 resolves it to the
    /// closest point to the guard (FUN_001e6f60) at every replan
    pub f_40: Option<Path>,
}

impl Goto {
    pub fn new(bc: bool) -> Goto {
        Goto { st: -1, f_50: [0.0; 3], a0: f32::MIN, bc, walk: None, f_40: None }
    }
    /// ANIM R27: FUN_0017dd38 with a path target (+0x40 = path, +0x50 unused)
    pub fn ee_sub_17dd38_path(&mut self, cx: &mut Cx, p: Path) {
        self.f_40 = Some(p);
        self.a0 = f32::MIN;
        self.ee_sub_17da28(cx);
    }
    /// FUN_0017dd38: retarget + force replan (+0xa0 = DAT_002a562c)
    pub fn ee_sub_17dd38(&mut self, cx: &mut Cx, p: V3) {
        self.f_50 = p;
        self.a0 = f32::MIN;
        self.ee_sub_17da28(cx);
    }
    /// FUN_0017da28 replan (straight-line branch, +0xbc != 0; the navmesh FUN_001c02d0 branch -> same
    /// 2-point path, FUN_001c1ec0 projection = identity)
    fn ee_sub_17da28(&mut self, cx: &mut Cx) {
        if cx.body.f_fe8 & 2 == 0 {
            return;
        }
        // ANIM R27: FUN_0017d8a8 -> FUN_001c1ec0(+0x40 target, guard pos): a path target is its closest point
        if let Some(p) = &self.f_40 {
            self.f_50 = FUN_001e6f60(p, cx.body.pos).cp;
        }
        // nav +0xe8 = 2, +0xf4 = {pos, goal}, FUN_001e6180 (zero-length path: already there, no walk child)
        if len(sub(self.f_50, cx.body.pos)) < 1e-3 {
            self.walk = None;
            self.st = 1;
            self.a0 = cx.now;
            return;
        }
        let mut p = Path::polyline(vec![cx.body.pos, self.f_50], false);
        FUN_001e6180(&mut p);
        p.s = p.l.clone();
        // FUN_0017d450(1): child 0xa7 with FUN_00178d90(child, guard + 0x294, 0), started
        let mut w = Walk::ee_sub_177e80(p, 0, cx.body.pos);
        w.ee_sub_177ef0(cx);
        self.walk = Some(w);
        self.st = 1;
        self.a0 = cx.now;
    }
    /// FUN_0017dcb8: state 1 and the walk arrived
    pub fn ee_sub_17dcb8(&self, pos: V3) -> bool {
        self.st == 1 && self.walk.as_ref().is_none_or(|w| w.ee_sub_178d00(pos))
    }
    /// FUN_0017d6c8 update (then the child's update, FUN_001eed10 order: leaf first is the caller's job)
    pub fn ee_sub_17d6c8(&mut self, cx: &mut Cx) {
        if cx.body.f_fe8 & 2 == 0 {
            return;
        }
        if (1..3).contains(&self.st) {
            let arrived = self.walk.as_ref().is_none_or(|w| w.ee_sub_178d00(cx.body.pos));
            if self.st == 2 && arrived {
                self.st = 3;
            } else if cx.now - self.a0 > D_758 {
                self.ee_sub_17da28(cx);
            }
        }
    }
    pub fn tick(&mut self, cx: &mut Cx) {
        if let Some(w) = &mut self.walk {
            w.ee_sub_178ee0(cx);
        }
        self.ee_sub_17d6c8(cx);
    }
}

/// 0xc0 RAIL-PATROL task (vt 0x2cfda0, size 0x60).
#[derive(Clone, Debug)]
pub struct Rail {
    /// +0x30 rail, +0x34 dir (0 = backward), +0x38 u, +0x40 target, +0x58 no-node-snap flag
    pub rail: Path,
    pub dir: i32,
    pub u: f32,
    pub f_40: V3,
    pub f_58: bool,
    pub goto: Goto,
}

impl Rail {
    /// FUN_0017eb88 start (u < 0: FUN_001e6f60 closest point; else FUN_001e7380 near u)
    pub fn ee_sub_17eb88(rail: Path, dir: i32, u: f32, f_58: bool, cx: &mut Cx) -> Rail {
        let pos = cx.body.pos;
        let u = if u >= 0.0 { FUN_001e6ed8(u, &rail, pos).l } else { FUN_001e6f60(&rail, pos).l };
        // child 0xba: +0xbc = +0x58
        let mut r = Rail { rail, dir, u, f_40: [0.0; 3], f_58, goto: Goto::new(f_58) };
        r.ee_sub_17ecf0(cx, true);
        r
    }
    /// FUN_0017ecf0(this, force): advance the waypoint
    pub fn ee_sub_17ecf0(&mut self, cx: &mut Cx, force: bool) {
        let mut d = if cx.body.c3a { D_7BC } else { D_7C0 };
        if self.dir == 0 {
            d = -d;
        }
        let mut u2 = self.u + d;
        if !self.rail.looped {
            u2 = u2.clamp(FUN_001e6620(&self.rail), FUN_001e6648(&self.rail));
        } else {
            u2 = FUN_001e66c8(&self.rail, u2);
        }
        // !+0x58: rail node snap (FUN_001ea000 / FUN_001ea3b0 on rail +0x180) - no node list modelled
        if !force && (self.u - u2).abs() < D_7C4 {
            return;
        }
        self.u = u2;
        self.f_40 = FUN_001e6f10(&self.rail, u2).0;
        let p = self.f_40;
        self.goto.ee_sub_17dd38(cx, p);
    }
    /// FUN_0017eeb8: a non-looped rail reached its end
    pub fn ee_sub_17eeb8(&self) -> bool {
        !self.rail.looped && (self.u - if self.dir != 0 { FUN_001e6648(&self.rail) } else { FUN_001e6620(&self.rail) }).abs() <= D_7B8
    }
    /// FUN_0017ee78: done
    pub fn ee_sub_17ee78(&self, pos: V3) -> bool {
        self.ee_sub_17eeb8() && self.goto.ee_sub_17dcb8(pos)
    }
    /// child first (FUN_001eed10 runs the stack leaf first), then FUN_0017ecb0
    pub fn tick(&mut self, cx: &mut Cx) {
        self.goto.tick(cx);
        if self.goto.ee_sub_17dcb8(cx.body.pos) {
            self.ee_sub_17ecf0(cx, false);
        }
    }
}

/// One routine command (class-113 +0x50 entry, resolved).
#[derive(Clone, Debug)]
pub enum Cmd {
    /// kind 5: wait f_8[0] seconds
    Wait(f32),
    /// kind 6: rail, dir (+8), start u (+0xc)
    Rail(Option<Path>, i32, f32),
    /// kind 7: go to a point (object position / guard +0x300), then face yaw (None = keep)
    Goto(V3, Option<f32>),
    /// ANIM R27: kind 7 on an is-a-10 object (FUN_00132b10(obj, 10)): target +0 = the path, goes to its closest point
    GotoPath(Path),
    /// ANIM R27: kinds 1 / 2: guard +0x2cc / +0x2d4 = entry +8 / +0xc (routine run / idle ranges)
    Set2cc([f32; 2]),
    Set2d4([f32; 2]),
    /// kind 9 label / kind 10 jump to label id
    Label(i32),
    Jump(i32),
    /// kind 3: place the guard at an object (FUN_00171b60: position vslot +0xec, FUN_00204660 orientation)
    Warp(V3, f32),
    /// kind 0xf: FUN_001e0020(sequence handle, signal)
    Signal(u32, i32),
    /// ANIM R25: kind 0xe: play the ASEG oid (+4, FUN_0015e938 under the guard) with record flags (+8)
    Anim(i32, u32),
    Other(i32),
}

#[derive(Clone, Debug)]
enum Child {
    None,
    Rail(Box<Rail>),
    Goto(Box<Goto>),
}

/// 0xa9 SCRIPT task (vt 0x2cf7f8, size 0x60).
#[derive(Clone, Debug)]
pub struct Script {
    /// +0x30 routine commands, +0x38 current entry, +0x3c command start, +0x40 busy, +0x48 yaw, +0x4c flag
    pub cmds: Vec<Cmd>,
    pub cur: Option<usize>,
    pub f_3c: f32,
    pub busy: bool,
    pub f_48: Option<f32>,
    pub f_4c: bool,
    child: Child,
    /// ANIM R25: kind 0xe anim command: (ASEG oid, record flags, start time) of the pushed 0x9d anim task
    /// (vt 0x2d0ab0, +0x40 = the +0x50 record FUN_002090d0 / FUN_00209130)
    pub anim: Option<(i32, u32, f32)>,
}

impl Script {
    /// FUN_00171258(t, routine, flag, cursor): start at the cursor or the first entry, execute it
    pub fn ee_sub_171258(cmds: Vec<Cmd>, flag: bool, cursor: Option<usize>, cx: &mut Cx) -> Script {
        let cur = cursor.or(if cmds.is_empty() { None } else { Some(0) });
        let mut s = Script { cmds, cur, f_3c: 0.0, busy: false, f_48: None, f_4c: flag, child: Child::None, anim: None };
        if let Some(k) = s.cur {
            s.ee_sub_1715f0(k, cx);
        }
        s
    }
    /// FUN_00171458: script ended
    pub fn ee_sub_171458(&self) -> bool {
        self.cur.is_none()
    }
    /// FUN_001715f0: execute entry k (kinds 5, 6, 7, 9, 10)
    fn ee_sub_1715f0(&mut self, k: usize, cx: &mut Cx) {
        self.child = Child::None;
        self.anim = None;
        self.f_3c = cx.now;
        match self.cmds[k].clone() {
            // ANIM R25: case 0xe: +0x50 record {kind, oid +0x54, +0x5c = entry +8} (FUN_002090d0 / FUN_00209130),
            // guard +0xf80 == -1: push 0x9d with +0x40 = record; +0x40 busy
            Cmd::Anim(oid, fl) => {
                self.anim = Some((oid, fl, cx.now));
                self.busy = true;
            }
            Cmd::Wait(_) => self.busy = true,
            Cmd::Rail(Some(p), dir, u) => {
                // guard +0xf80 == -1 (init): task 0xc0, +0x58 = +0x4c
                self.child = Child::Rail(Box::new(Rail::ee_sub_17eb88(p, dir, u, self.f_4c, cx)));
                self.busy = true;
            }
            Cmd::Goto(p, yaw) => {
                let mut g = Goto::new(self.f_4c);
                g.ee_sub_17dd38(cx, p);
                self.child = Child::Goto(Box::new(g));
                self.f_48 = yaw;
                self.busy = true;
            }
            Cmd::GotoPath(p) => {
                let mut g = Goto::new(self.f_4c);
                g.ee_sub_17dd38_path(cx, p);
                self.child = Child::Goto(Box::new(g));
                self.f_48 = None;
                self.busy = true;
            }
            // ANIM R27: cases 1 / 2 (LAB_00171acc: not busy)
            Cmd::Set2cc(r) => {
                cx.body.f_2cc = r;
                self.busy = false;
            }
            Cmd::Set2d4(r) => {
                cx.body.f_2d4 = r;
                self.busy = false;
            }
            Cmd::Warp(p, yaw) => {
                cx.body.pos = p;
                cx.body.yaw = yaw;
                cx.body.f_1a8 = yaw;
                self.busy = false;
            }
            Cmd::Signal(seq, sig) => {
                cx.h.ee_sub_1e0020(seq, sig);
                self.busy = false;
            }
            Cmd::Jump(id) => {
                if let Some(j) = self.cmds.iter().position(|c| matches!(c, Cmd::Label(l) if *l == id)) {
                    self.cur = Some(j);
                }
                self.busy = false;
            }
            _ => self.busy = false,
        }
    }
    /// FUN_00171468: current command still running (kills the child when it finishes)
    fn ee_sub_171468(&mut self, cx: &mut Cx) -> bool {
        if !self.busy {
            return false;
        }
        let Some(k) = self.cur else { return false };
        let run = match (&self.cmds[k], &self.child) {
            (Cmd::Wait(s), _) => cx.now - self.f_3c <= *s,
            // ANIM R25: case 0xd / 0xe: FUN_00174fb8 (0x9d +0x30 == 1: FUN_00209430 -> FUN_00131968, the ASEGA left
            // the live set = played to the end without a keep flag (SEGA_KEEP 0x38, e.g. 8 = loop))
            (Cmd::Anim(oid, fl), _) => match cx.h.aseg_oid_len(cx.guard, *oid) {
                Some((len, f78)) => *fl & crate::engine::anim::play::SEGA_KEEP != 0 || (cx.now - self.f_3c) * f78 < len,
                None => false,
            },
            (Cmd::Rail(..), Child::Rail(r)) => !r.ee_sub_17ee78(cx.body.pos),
            (Cmd::Goto(..) | Cmd::GotoPath(..), Child::Goto(g)) => {
                if g.ee_sub_17dcb8(cx.body.pos) {
                    if let Some(y) = self.f_48 {
                        ee_sub_209818(cx.body, y);
                    }
                    false
                } else {
                    true
                }
            }
            _ => false,
        };
        if !run {
            self.busy = false;
            self.child = Child::None;
            self.anim = None;
        }
        run
    }
    /// child (leaf first), then FUN_001713e8
    pub fn tick(&mut self, cx: &mut Cx) {
        match &mut self.child {
            Child::Rail(r) => r.tick(cx),
            Child::Goto(g) => g.tick(cx),
            Child::None => cx.body.f_190 = [0.0; 3],
        }
        let mut guard = 0;
        while self.cur.is_some() && !self.ee_sub_171468(cx) && guard < 64 {
            guard += 1;
            let n = self.cur.unwrap() + 1;
            self.cur = (n < self.cmds.len()).then_some(n);
            if let Some(k) = self.cur {
                self.ee_sub_1715f0(k, cx);
            }
        }
    }
    /// debug: running rail / goto state (tests)
    pub fn child_dbg(&self) -> String {
        match &self.child {
            Child::Rail(r) => format!("rail dir {} u {} min {} max {} looped {} f40 {:?} goto st {} f50 {:?} walk {:?}", r.dir, r.u, FUN_001e6620(&r.rail), FUN_001e6648(&r.rail), r.rail.looped, r.f_40, r.goto.st, r.goto.f_50, r.goto.walk.as_ref().map(|w| (w.st, w.b0, w.u, w.end_u))),
            Child::Goto(g) => format!("goto st {} f50 {:?}", g.st, g.f_50),
            Child::None => "none".into(),
        }
    }
    /// waypoint target of the running rail command (tests / debug)
    pub fn rail_u(&self) -> Option<f32> {
        match &self.child {
            Child::Rail(r) => Some(r.u),
            _ => None,
        }
    }
}

/// 0xaa ROUTINE task (vt 0x2d0520, size 0x50).
#[derive(Clone, Debug)]
pub struct Routine {
    /// +0x38 state (0 script, 1 idle, 2 done, 3 interrupted), +0x30 run end / +0x34 idle end (FLT_MAX defaults),
    /// +0x40 routine, +0x18 child script (state 0; state 1's child is the 0x9e idle task)
    pub st: i32,
    pub t30: f32,
    pub t34: f32,
    pub f_40: Option<Vec<Cmd>>,
    pub script: Option<Script>,
    /// guard +0x2dc (FUN_00171c18 on leaving state 0, read back by FUN_00171258)
    pub f_2dc: Option<usize>,
}

/// FUN_00209098(g): +0xfc8 != 0 || FUN_0020f1b8(0x3dd490) (world +0x3d84 > 0). CRAWL R97 (opt-out SLY2_NO_R97_3D84 =
/// the old host query, always false)
fn ee_sub_209098(cx: &mut Cx) -> bool {
    if std::env::var_os("SLY2_NO_R97_3D84").is_some() {
        return cx.h.ee_sub_209098(cx.guard);
    }
    cx.body.f_fc8 != 0 || cx.h.ee_sub_20f1b8()
}

impl Routine {
    /// FUN_00179aa0 start: +0x40 = +0x2e8 or (+0xfe8 & 2) +0x2ec, else state 2; FUN_00179d30(0 / 3)
    pub fn ee_sub_179aa0(cmds: Option<Vec<Cmd>>, cursor: Option<usize>, cx: &mut Cx) -> Routine {
        let mut r = Routine { st: -1, t30: f32::MAX, t34: f32::MAX, f_40: cmds, script: None, f_2dc: cursor };
        let s = if r.f_40.is_none() { 2 } else if ee_sub_209098(cx) { 3 } else { 0 };
        r.ee_sub_179d30(s, cx);
        r
    }
    /// FUN_00179d30(state): leaving 0 saves the cursor (FUN_00171c18), the child is popped; 0 = new script at
    /// the cursor + FUN_00179e80, +0x34 = FLT_MAX; 1 = 0x9e idle + FUN_00179ef8, +0x30 = FLT_MAX
    fn ee_sub_179d30(&mut self, s: i32, cx: &mut Cx) {
        if s == self.st {
            return;
        }
        if self.st == 0 {
            self.f_2dc = self.script.as_ref().and_then(|x| x.cur);
        }
        self.script = None;
        self.st = s;
        match s {
            0 => {
                let c = self.f_40.clone().unwrap_or_default();
                self.script = Some(Script::ee_sub_171258(c, false, self.f_2dc, cx));
                // FUN_00179e80
                if self.t30 >= f32::MAX {
                    let r = cx.body.f_2cc;
                    self.t30 = if r[0] == f32::MAX { f32::MAX } else { cx.now + cx.body.rng.FUN_0011d448(r[0], r[1]) };
                }
                self.t34 = f32::MAX;
            }
            1 => {
                // FUN_00179ef8
                if self.t34 >= f32::MAX {
                    let r = cx.body.f_2d4;
                    self.t34 = if r[0] == f32::MAX { f32::MAX } else { cx.now + cx.body.rng.FUN_0011d448(r[0], r[1]) };
                }
                self.t30 = f32::MAX;
            }
            _ => {}
        }
    }
    /// FUN_00179ba0 update (child first, FUN_001eed10 leaf order), then the state loop
    pub fn tick(&mut self, cx: &mut Cx) {
        if let Some(s) = &mut self.script {
            s.tick(cx);
        } else {
            cx.body.f_190 = [0.0; 3];
        }
        let busy = ee_sub_209098(cx);
        for _ in 0..8 {
            let st = self.st;
            match st {
                0 if busy => self.ee_sub_179d30(3, cx),
                0 if cx.now <= self.t30 => {
                    if self.script.as_ref().is_some_and(|s| s.ee_sub_171458()) {
                        self.f_40 = None;
                        self.ee_sub_179d30(2, cx);
                    }
                }
                0 => self.ee_sub_179d30(1, cx),
                1 if busy => self.ee_sub_179d30(3, cx),
                1 if self.t34 < cx.now => self.ee_sub_179d30(0, cx),
                2 if busy => self.ee_sub_179d30(3, cx),
                3 if !busy => self.ee_sub_179d30(if self.f_40.is_some() { 1 } else { 2 }, cx),
                _ => {}
            }
            if self.st == st {
                break;
            }
        }
    }
    /// FUN_00171c18: cursor saved into guard +0x2dc on leaving state 0
    pub fn cursor(&self) -> Option<usize> {
        if self.st == 0 { self.script.as_ref().and_then(|s| s.cur) } else { self.f_2dc }
    }
}
