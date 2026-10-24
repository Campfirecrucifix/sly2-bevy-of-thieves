//! CRAWL R49: the JT +0xfd0 walk-to controller, native 1:1 of FUN_001448f0 (set goal) and the stall check of
//! FUN_00144640 (update). The original never steers: it tracks the JT's distance to the goal every
//! DAT_002bd2d0 s and, when it shrank by less than DAT_002bd2d4 * DAT_002bd2d0, sends the JT to the goal with
//! FUN_0019f540 (jtbs 0x15 jump) and sets busy [10] = 1; busy clears when the JT's +0x2dc (jts) is -1 again.
//! Port notes: the FUN_0020f0b0 (co-op world) partner-alert walk of 0x144640 is not taken (single JT); the
//! FUN_00173610 / FUN_0017d608 ground probe at the goal is taken as a hit (goals are script markers on floor).
use super::{p5_states, Jt, JtAnim};
use crate::engine::classes::q::q6::rof;

#[derive(Clone, Copy, Debug, Default)]
pub struct WalkFd0 {
    /// [4..8]
    pub pos: [f32; 3],
    /// [8]
    pub dist: f32,
    /// [9]
    pub next: f32,
    /// [10]
    pub busy: bool,
    /// CRAWL R50: FUN_001e46a0 case 0x24 steering for the next JT frame (Jt stick convention)
    pub stick: Option<[f32; 2]>,
    /// FEEL R92: FUN_001e46a0 case 0x1a pipe-climb steering for the next JT frame (joy screen stick, magnitude 1)
    pub climb: Option<[f32; 2]>,
    /// FEEL R93: FUN_001e46a0 case 0x24 rec[9] != 0: joy +0x30 bit 8 (R1) for the next JT frame
    pub r1: bool,
}

fn d3(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// FUN_001448f0(jt + 0xfd0, q)
pub fn FUN_001448f0(jt: &mut Jt, q: [f32; 3]) {
    let w = &mut jt.walk_fd0;
    if !w.busy && rof(0x002b_d2d8) <= d3(w.pos, q) {
        w.pos = q;
        w.dist = d3(jt.phys.pos, q);
        w.next = jt.t + rof(0x002b_d2d0);
    }
}

/// FUN_00144640(jt + 0xfd0); `seq_24`: FUN_00144068 (+0x1084 seq) with the current record 0x24.
pub fn FUN_00144640(jt: &mut Jt, seq_24: bool, jts_idle: bool, anim: &mut dyn JtAnim) {
    if jt.walk_fd0.busy {
        if jts_idle {
            jt.walk_fd0.busy = false;
        }
        return;
    }
    if !seq_24 || jt.walk_fd0.next > jt.t {
        return;
    }
    jt.walk_fd0.next = jt.t + rof(0x002b_d2d0);
    let q = jt.walk_fd0.pos;
    let d = d3(jt.phys.pos, q);
    if jt.walk_fd0.dist - d < rof(0x002b_d2d4) * rof(0x002b_d2d0) {
        if std::env::var_os("SLY2_K22DBG").is_some() {
            eprintln!("WALKTO stall hop t {} pos {:?} q {q:?} jts {} jtbs {}", jt.t, jt.phys.pos, jt.jts, jt.jtbs);
        }
        jt.with_p5(|jt, st, w| {
            p5_states::FUN_0019f540(jt, st, w, q, anim);
        });
        jt.walk_fd0.busy = true;
    }
    jt.walk_fd0.dist = d;
}

/// CRAWL R50: one JT frame of the k22 0x24 walk-to: the FUN_001e3890 case-0x24 goals of this frame
/// (`walks` = host `k22_walktos`, (subject handle, goal)) refresh FUN_001448f0(jt + 0xfd0, goal) for Sly
/// (`PLAYER_HANDLE` or the JT object handle `jt_h`), then the JT's FUN_00144640 stall check. Shared by the game
/// (player::plugin::loot_link) and the headless harness.
pub fn k22_walk_frame(jt: &mut Jt, walks: Vec<(u32, [f32; 3], f32)>, jt_h: Option<u32>) {
    let mut seq24 = false;
    for (subj, g, speed) in walks {
        if subj != crate::engine::classes::guard::PLAYER_HANDLE && Some(subj) != jt_h {
            continue;
        }
        FUN_001448f0(jt, g);
        jt.walk_fd0.stick = Some(ee_sub_1e46a0_24(jt.phys.pos, g, speed));
        seq24 = true;
    }
    // owner +0x2dc == -1 (busy clear): +0x2dc = JMP +0x1c = the jtbs (JMP at jt+0x2c0, FUN_00195a00 param_1[7]).
    // FEEL R91: busy clears only when the jump module is idle (jtbs -1, landed), not as soon as 0x15 ends (0xb/4
    // fall): the old test let the next stall check hop again mid-air (live game.exe table 2 safehouse exit: chained
    // hops to z 2600 onto the roofs). SLY2_FEEL_R91WALK=0 = old (jtbs != 0x15).
    let idle = if std::env::var("SLY2_FEEL_R91WALK").map_or(true, |v| v != "0") { jt.jtbs == -1 } else { jt.jtbs != 0x15 };
    FUN_00144640(jt, seq24, idle, &mut ());
}

/// FEEL R92: one JT frame of the k22 0x1a pipe climb (`goals` = host `k22_climbs`, (subject, locator pos)).
/// FUN_001e1e30 case 0x1a: node +0x7c = FUN_001e6f60(jt+0xf5c path, locator) (target arc length); FUN_001e46a0
/// case 0x1a: joy +0x10/+0x14 = FUN_001c4768(joy, p(u+50) - p(u-50)) (u = jt+0xf60), negated (and +0x18 + pi)
/// when the target lies behind u (looped path: the shorter way round), +0x1c = 1. Returns FUN_001e3890 case 0x1a's
/// |u - FUN_0011d948(+0x7c, FUN_0019e110(jt+0xf50))| (None: not on a path = done).
pub fn k22_climb_frame(jt: &mut Jt, goals: Vec<(u32, [f32; 3])>, jt_h: Option<u32>) -> Option<f32> {
    let g = goals.into_iter().rev().find(|(s, _)| *s == crate::engine::classes::guard::PLAYER_HANDLE || Some(*s) == jt_h)?.1;
    if jt.jts != 8 {
        return None;
    }
    let path = jt.p14.path.clone()?;
    let st = path.FUN_001e6f60(g).l;
    let u = jt.p5.s8.u;
    let (lo, hi) = super::p14::FUN_0019e110(&path, jt.p5.s8.dir_18);
    let back = if !path.looped {
        st < u
    } else {
        let len = path.FUN_001e6648();
        let (a, b) = if st < u { (u - st, len + st - u) } else { (len + u - st, st - u) };
        a < b
    };
    let rows = jt.p5w.cam_rows.unwrap_or_else(|| {
        let (c, s) = (jt.yaw.cos(), jt.yaw.sin());
        [[c, s, 0.0], [-s, c, 0.0], [0.0, 0.0, 1.0]]
    });
    let sv = super::p14::FUN_001c4768(&rows, super::p14::tangent(&path, u));
    jt.walk_fd0.climb = Some(if back { [-sv[0], -sv[1]] } else { sv });
    Some((u - st.clamp(lo, hi)).abs())
}

/// CRAWL R50: FUN_001e46a0 case 0x24 off rails (game/q_001e46a0.rs, text_001e0000.c FUN_001e46a0): heading
/// j+0x18 = atan2(goal - pos), magnitude j+0x1c = |op[5]| * clamp(DAT_002c3630 + d (DAT_002c3634 + d DAT_002c3638),
/// DAT_002bbb60, DAT_002bbb64), d = xy distance; returned as the JT stick (y = world +x, x = world -y). The rail
/// branch (+0x4c8 == 3) and the flag-8 / 0x20 bits of the record are not taken (Sly is never on a rail here).
pub fn ee_sub_1e46a0_24(pos: [f32; 3], goal: [f32; 3], speed: f32) -> [f32; 2] {
    let d = [goal[0] - pos[0], goal[1] - pos[1]];
    let dist = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let c = rof(0x002c_3630) + dist * (rof(0x002c_3634) + dist * rof(0x002c_3638));
    let (lo, hi) = (rof(0x002b_bb60), rof(0x002b_bb64));
    let c = if c < lo { lo } else if hi < c { hi } else { c };
    let m = speed.abs() * c;
    if dist < 1e-4 {
        return [0.0; 2];
    }
    [-d[1] / dist * m, d[0] / dist * m]
}
