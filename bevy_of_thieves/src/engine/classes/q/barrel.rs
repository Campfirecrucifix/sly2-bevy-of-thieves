//! BOSS R21: Episode 3 (India hub, level-table row 12) TNT barrel = class 88 C2d20d0 (oid 95845, 1 + 3
//! clones; rec11 C:/Temp/emu_rec11_ep3 snapshots 238-261). The class methods are queue ports
//! (classes/q/c2d20d0.rs: slot 74 FUN_00183e08, slot 69 FUN_00183b00; game/q_00182b18.rs FUN_00183b80
//! state change); this module is the engine side of the JT calls into them:
//! - FUN_00192bd8 attach kind 1 (`AttachEv::Call(0x00183b80, 2)`): `*(barrel + 400) = jt`, FUN_00183b80(barrel, 2)
//!   (worn: rec11 jt+0x4c8 = 1, jt+0x4d4 = barrel, barrel +0x17c = 2, +0x190 = jt); leaving (case -1 / kind 1,
//!   `AttachEv::Call(0x00183b80, 0)`): FUN_00183b80(barrel, 0) (rec11 +0x17c 2 -> 0);
//! - FUN_001a19a0 with jt+0x4c8 == 1 (`DmgEv::Delegated`): the hit message goes to the barrel's slot 74
//!   (vt+0x25c); FUN_001f7598 filled +0x44 = FUN_001f78d8(jt, kind) from the JT's +0x110 table before, so
//!   a laser hit (kind bit in 0x10ff00) carries flag 8 and FUN_00183e08 explodes the barrel: stimulus
//!   FUN_00156b98, state 3 (rec11 snapshot 240: the worn barrel 0xfc9800 in state 3 after the laser death),
//!   vt+0x1c(0xb, 0, 2, {msg, msg + 0xc}).
//! REPLACED: the 0x60-byte message is built in the guest image with src 0 (the laser object has no guest
//! address here); `deliver` returns the slot-74 result the original hands back to FUN_001f7598.
use crate::engine::classes::q::live::LiveMem;
use crate::engine::classes::q::q2mem::QMem;

/// FUN_00183b80 caller side of FUN_00192bd8 (kind 1): `st == 2` stores the wearer in +0x190 first.
pub fn attach_state(m: &mut LiveMem, barrel: u32, jt: u32, st: u32) {
    if st == 2 {
        m.w32(barrel + 400, jt);
    }
    m.guarded(|m| crate::engine::game::q_00182b18::ee_sub_183b80(m, barrel, st));
}

/// FUN_001a19a0 jt+0x4c8 == 1 branch: (vt+0x25c)(jt+0x4d4, msg) with the FUN_001f7598 message
/// {+0 src, +8 target, +0xc kind, +0x10 amount, +0x44 = FUN_001f78d8(jt, kind)}.
pub fn deliver(m: &mut LiveMem, barrel: u32, jt: u32, kind: i32, amount: i32, zap: u8) -> u32 {
    deliver_msg(m, barrel, 0, jt, kind, amount, zap)
}

/// R26: slot 74 of `barrel` with the FUN_001f7598 message {+0 src, +8 target, +0xc kind, +0x10 amount, +0x44 zap}.
pub fn deliver_msg(m: &mut LiveMem, barrel: u32, src: u32, target: u32, kind: i32, amount: i32, zap: u8) -> u32 {
    let msg = m.alloc(0x60);
    for k in 0..0x18 {
        m.w32(msg + 4 * k, 0);
    }
    m.w32(msg, src);
    m.w32(msg + 8, target);
    m.w32(msg + 0xc, kind as u32);
    m.w32(msg + 0x10, amount as u32);
    m.w32(msg + 0x44, zap as u32);
    m.call_slot(barrel, 74, &[msg], &[]).1
}

/// The JT side per frame: `att_ev` (FUN_00192bd8 `AttachEv::Call(0x00183b80, st)`) and `dmg_ev`
/// (`DmgEv::Delegated`) of this frame into the live image. `jt_obj` = the JT's world object index.
/// Leaving (st != 2) finds the barrel worn by the JT (+0x17c == 2, +0x190 == jt) because the attach target
/// is already cleared when the events are read. Returns the slot-74 results of the delegated hits.
pub fn jt_link(
    m: &mut LiveMem,
    w: &crate::engine::world::World,
    jt_obj: Option<usize>,
    jt: &crate::engine::player::Jt,
    att_ev: &[crate::engine::player::p5_attach::AttachEv],
    dmg_ev: &[crate::engine::player::p7_damage::DmgEv],
) -> Vec<u32> {
    use crate::engine::player::{p5_attach::AttachEv, p7_damage::DmgEv};
    let jt_a = jt_obj.and_then(|i| m.obj_addr.get(i).copied()).unwrap_or(0);
    for e in att_ev {
        let AttachEv::Call(0x0018_3b80, st) = *e else { continue };
        let b = if st == 2 {
            jt.attach.target.and_then(|i| m.obj_addr.get(i).copied())
        } else if st == 1 {
            // R22: FUN_00195fb8 entry 6: FUN_00183b80(jmp+0xc = the picked barrel, 1)
            jt.flight.target.and_then(|i| m.obj_addr.get(i).copied())
        } else {
            let ws: Vec<u32> = w.objects.iter().enumerate().filter(|(_, o)| o.class == 88).filter_map(|(i, _)| m.obj_addr.get(i).copied()).collect();
            // R22: also the barrel of an aborted jtbs 6 flight (state 1, FUN_00195fb8 exit 6)
            ws.into_iter().find(|&a| (m.r32(a + 0x17c) == 2 && m.r32(a + 400) == jt_a) || m.r32(a + 0x17c) == 1)
        };
        if let Some(b) = b {
            attach_state(m, b, jt_a, st as u32);
        }
    }
    let mut out = Vec::new();
    for e in dmg_ev {
        let DmgEv::Delegated { partner: Some(p), kind, amount } = *e else { continue };
        let Some(b) = m.obj_addr.get(p).copied() else { continue };
        // FUN_001f7598: +0x44 = FUN_001f78d8(0x1a5bf0(target) = the JT, kind)
        let zap = crate::engine::player::p8_hazard::FUN_001f78d8(&jt.p8.zap_110, kind as u32);
        out.push(deliver(m, b, jt_a, kind, amount, zap));
    }
    out
}

/// BOSS R22: the per-frame engine entry (player::plugin `classes_link` after link_classes, and the test
/// harness): this frame's `att_ev` plus the `dmg.ev` pushed since the last call into [`jt_link`].
/// R22: a delegated hit that blew the worn barrel up (FUN_00183e08 flag 8) also ran, on the wearer +0x190,
/// FUN_0019cd38(jt+0xf00, -1) and vt+0x2c4(jt, 0) (= FUN_0018d568 m 0: Sly drops out); the guest did that
/// on its JT shadow, so it is applied to the Rust JT here.
pub fn jt_frame(m: &mut LiveMem, w: &crate::engine::world::World, jt: &mut crate::engine::player::Jt, anim: &mut dyn crate::engine::player::JtAnim) -> Vec<u32> {
    use crate::engine::player::p5_attach::AttachEv;
    let jo = w.objects.iter().position(|o| o.class == 53);
    // R25: the JT's transform into its live root part first (the worn barrel hangs off it)
    if let Some(a) = jo.and_then(|i| m.obj_addr.get(i).copied()) {
        super::live::jt_xf_sync(m, a, jt.phys.rot, jt.phys.pos);
    }
    // R26: a worn barrel's parts hang off the JT (FUN_00183b80 vt+0x54 reparent): their world frames follow it
    for b in barrels_list(m) {
        if m.r32(b + 0x17c) == 2 {
            for o in [0x58u32, 0x5c] {
                let p = m.r32(b + o);
                if p != 0 {
                    part_world(m, p, 0);
                }
            }
        }
    }
    let n = jt.dmg.ev.len();
    let ev0 = jt.barrel_ev0.min(n);
    jt.barrel_ev0 = n;
    let r = jt_link(m, w, jo, jt, &jt.att_ev, &jt.dmg.ev[ev0..]);
    // the delegated hit's partner (= jt+0x4d4, the worn barrel) now in state 3
    let ps: Vec<u32> = jt.dmg.ev[ev0..n].iter().filter_map(|e| match *e { crate::engine::player::p7_damage::DmgEv::Delegated { partner: Some(p), .. } => m.obj_addr.get(p).copied(), _ => None }).collect();
    let blown = ps.into_iter().any(|b| m.r32(b + 0x17c) == 3);
    if jt.attach.kind == 1 && blown {
        jt.FUN_0019cd38(-1, anim);
        jt.apply_attach_ev(vec![AttachEv::SetJtJts { m: 0 }], anim);
    }
    // R26: the worn barrel as the attach target of 0x191a10 case 1 (kind 1 riding): its +0x18c gates the stick
    // walk (sub 4, hide blend a+0x180 -> jt+0x190; rec11 238 / 241: +0x18c 1, sub 4, jt+0x190 (-416.7, 0, 0))
    if jt.jts == 7 && jt.attach.kind == 1 {
        if let Some(a) = jt.attach.target.and_then(|i| m.obj_addr.get(i).copied()) {
            let p = m.guarded(|m| m.call(0x001a_62e8, &[a], &[]).1).map_or(jt.phys.pos, |p| [p[0], p[1], p[2]]);
            let mut tg = crate::engine::player::p5_attach::AttachTarget::new(88, p);
            tg.f_18c = m.r32(a + 0x18c) != 0;
            jt.att_tg = Some(tg);
        }
    }
    feed(m, jt);
    r
}

/// R22: DAT_003e13e4 (FUN_00183998 list, link +0x174) -> `Jt::barrels` for the FUN_00197f98 air pick:
/// FUN_001a62e8 position, +0x158 range, +0x17c state. REPLACED: vt slot 32 velocity = 0 (barrels at rest).
fn rd(m: &mut LiveMem, a: u32) -> [f32; 3] {
    [f32::from_bits(m.r32(a)), f32::from_bits(m.r32(a + 4)), f32::from_bits(m.r32(a + 8))]
}

/// DAT_003e13e4 (the class-88 list, linked by +0x174).
fn barrels_list(m: &mut LiveMem) -> Vec<u32> {
    let (mut a, mut v) = (m.r32(0x003e_13e4), Vec::new());
    while a != 0 && v.len() < 64 {
        v.push(a);
        a = m.r32(a + 0x174);
    }
    v
}

/// R26 REPLACED (the ALO world-frame propagation of the JT's children is unported): part world (+0x40 rows /
/// +0x70 pos) = local (+0x00 rows / +0x30 pos) in the parent part's (+0xa0) world, parents first.
/// rec11 238: worn barrel part +0x5c local (0, 0, 50.2), world +0x70 = (-4879.6, 1416.9, -1240.1) beside Sly.
fn part_world(m: &mut LiveMem, p: u32, depth: u32) {
    let par = m.r32(p + 0xa0);
    if par == 0 || depth > 8 {
        return;
    }
    part_world(m, par, depth + 1);
    let pr = [rd(m, par + 0x40), rd(m, par + 0x50), rd(m, par + 0x60)];
    let pp = rd(m, par + 0x70);
    let row = |v: [f32; 3]| [0, 1, 2].map(|j| v[0] * pr[0][j] + v[1] * pr[1][j] + v[2] * pr[2][j]);
    for i in 0..3u32 {
        let r = row(rd(m, p + 0x10 * i));
        m.wv(p + 0x40 + 0x10 * i, [r[0], r[1], r[2], 0.0]);
    }
    let l = row(rd(m, p + 0x30));
    m.wv(p + 0x70, [pp[0] + l[0], pp[1] + l[1], pp[2] + l[2], 1.0]);
}

pub fn feed(m: &mut LiveMem, jt: &mut crate::engine::player::Jt) {
    jt.barrels.clear();
    let mut a = m.r32(0x003e_13e4);
    let mut n = 0;
    while a != 0 && n < 64 {
        if let Some(id) = m.obj_addr.iter().position(|&o| o == a) {
            let Some(p) = m.guarded(|m| m.call(0x001a_62e8, &[a], &[]).1) else { break };
            jt.barrels.push(crate::engine::player::p5_barrel::Barrel { id, pos: [p[0], p[1], p[2]], vel: [0.0; 3], range: f32::from_bits(m.r32(a + 0x158)), state: m.r32(a + 0x17c) });
        }
        a = m.r32(a + 0x174);
        n += 1;
    }
}

/// BOSS R25: the live image's FUN_001a61e0 script messages (`LiveMem::recorded_1a61e0`) of the class-88
/// barrels -> the class host's K27 queue (handle = object index + 1), delivered to the level script by
/// `game::vm_state::send_messages` (FUN_001331f8; FUN_00200da0 argument kinds, message 0xb: {src, kind}).
/// FUN_00183e08's explosion message 0xb reaches the row-13 script handlers this way.
/// REPLACED: messages of the other live-image senders are dropped as before (their Rust-side classes
/// send their own).
pub fn script_msgs(m: &mut LiveMem, w: &crate::engine::world::World, host: &mut crate::engine::classes::plugin::EngineHost) {
    beams(m, w, host);
    // CRAWL R103: live SMA script messages -> the SM object's frame, self = the SMA (field 251 = its +0x28 owner,
    // `aseg_script_self`): msg 8 {old, new} through k27_script_msgs + sma_script_self_q, others as aseg_script_msgs
    for (sma, msg, words, sm0, ow0) in std::mem::take(&mut m.recorded_sma_msgs) {
        if std::env::var_os("R109SMA").is_some() && msg != 2 { eprintln!("R109SMA sma {sma:#x} msg {msg} words {words:?} sm {sm0:#x}"); }
        // CRAWL R110: SM / owner as of the send (opt-out SLY2_NO_R110_SMACTX = read them now)
        let (sm, owner) = if std::env::var_os("SLY2_NO_R110_SMACTX").is_none() { (sm0, ow0) } else { (m.r32(sma + 0x24), m.r32(sma + 0x28)) };
        let Some(si) = m.obj_addr.iter().position(|&o| o == sm) else { continue };
        let oh = m.obj_addr.iter().position(|&o| o == owner).map_or(0, |i| i as u32 + 1);
        if std::env::var_os("SLY2_NO_R84_SMAORDER").is_none() {
            if (msg == 8 && words.len() == 2) || words.is_empty() {
                if host.live_sma_seq.len() < 8192 {
                    host.live_sma_seq.push((si, sma, oh, msg, words));
                }
            }
            continue;
        }
        if msg == 8 && words.len() == 2 {
            if host.k27_script_msgs.len() < 256 {
                host.k27_script_msgs.push((si as u32 + 1, 8, words.clone()));
                host.sma_script_self_q.insert((si as u32 + 1, words[0], words[1]), (sma, oh));
            }
        } else if words.is_empty() && host.aseg_script_msgs.len() < 8192 {
            host.aseg_script_msgs.push((si, sma, oh, msg));
        }
    }
    for (a, msg, words) in std::mem::take(&mut m.recorded_1a61e0) {
        if std::env::var_os("EP3_MSG").is_some() { eprintln!("EP3M 1a61e0 {a:#x} msg {msg} {words:x?} obj {:?}", m.obj_addr.iter().position(|&o| o == a)); }
        let Some(i) = m.obj_addr.iter().position(|&o| o == a) else { continue };
        if w.objects.get(i).is_some_and(|o| o.class == 88) && host.k27_script_msgs.len() < 256 {
            host.k27_script_msgs.push((i as u32 + 1, msg, words));
        }
    }
}

/// BOSS R25: the free TNT barrels' fall (world update 0x226550 -> FUN_001f8dd8 physics driver over the
/// dynamic SOs; the live image has no SO physics pass). rec11 239/240: after Sly jumps out the barrel
/// (+0x2c bit 28 = in the world, parts +0x58 / +0x5c kind 1) free-falls (+0x58 +0xb0 = (0, 0, -1723) after
/// ~1 s) into the laser defense pit. Each frame, per in-world class-88 copy not worn (+0x17c != 2) and with
/// no parent (+0x10 == 0): the +0x5c part's world frame (+0x40 rows, +0x70 = the FUN_001a62e8 position),
/// +0x58 +0xb0 velocity, +0x5c +0xb0 angular velocity and +0x98 kind -> `physics::driver::FUN_001f8dd8`
/// against the level collision -> written back (both parts' world frames move by the same step).
/// REPLACED: the contact sphere (radius 20 around the +0x5c frame: rec11 236 rest 20 above the dispenser
/// floor) stands in for the barrel's SO geometry, and gravity / constraint are the 0x1f3590 defaults.
pub fn phys(m: &mut LiveMem, w: &crate::engine::world::World, cw: &crate::engine::physics::CollisionWorld, dt: f32) {
    use crate::engine::physics::driver::{AloPhys, FUN_001f8dd8};
    const R: f32 = 20.0;
    let rd = |m: &mut LiveMem, a: u32| [f32::from_bits(m.r32(a)), f32::from_bits(m.r32(a + 4)), f32::from_bits(m.r32(a + 8))];
    for (i, o) in w.objects.iter().enumerate() {
        if o.class != 88 {
            continue;
        }
        let Some(&a) = m.obj_addr.get(i) else { continue };
        if m.r32(a + 0x2c) & 0x1000_0000 == 0 || m.r32(a + 0x17c) == 2 || m.r32(a + 0x10) != 0 {
            continue;
        }
        let (pl, pa) = (m.r32(a + 0x58), m.r32(a + 0x5c));
        if pl == 0 || pa == 0 {
            continue;
        }
        let kind = (m.r32(pl + 0x98) & 0xffff) as u16;
        if kind != 1 && kind != 2 && kind != 3 {
            continue;
        }
        let mut b = AloPhys::default();
        b.rot = [rd(m, pa + 0x40), rd(m, pa + 0x50), rd(m, pa + 0x60)];
        b.pos = rd(m, pa + 0x70);
        b.v = rd(m, pl + 0xb0);
        b.w = rd(m, pa + 0xb0);
        b.kind = kind;
        if !b.pos.iter().chain(b.v.iter()).all(|x| x.is_finite()) {
            continue;
        }
        let p0 = b.pos;
        // BOSS R28: the barrel's SO shape records (+0xe0 count / +0xe4, 0xb0 bytes each; +0x10 bit 50 set =
        // inactive, FUN_001f2ad0). The fuse SM's ASEG 98049 kind 0x10 (FUN_001f6b48(barrel, 11725)) drops its
        // shape at the blast, so it has no contacts and free-falls (cap_boss_tnt_fuse: rests 1.99 s after the
        // jump-out, then dv -29.2 per frame = gravity -1750, removed at +2.97 s). REPLACED: the unregistered body
        // is woken (kind 3 -> 1) instead of through the SO list FUN_001f2930 removal
        let (n, l) = (m.r32(a + 0xe0), m.r32(a + 0xe4));
        let solid = n == 0 || l == 0 || (0..n.min(64)).any(|k| m.r32(l + 0xb0 * k + 0x14) & 1 << 18 == 0);
        let empty = crate::engine::physics::query::CollisionWorld::default();
        if !solid && b.kind == 3 {
            b.kind = 1;
        }
        let p0 = b.pos;
        FUN_001f8dd8(if solid { cw } else { &empty }, &mut b, R, dt);
        let d = [b.pos[0] - p0[0], b.pos[1] - p0[1], b.pos[2] - p0[2]];
        for p in [pl, pa] {
            let q = rd(m, p + 0x70);
            m.wv(p + 0x70, [q[0] + d[0], q[1] + d[1], q[2] + d[2], 1.0]);
            let k = m.r32(p + 0x98); m.w32(p + 0x98, (k & 0xffff_0000) | b.kind as u32);
        }
        for (k, r) in b.rot.iter().enumerate() {
            m.wv(pa + 0x40 + 0x10 * k as u32, [r[0], r[1], r[2], 0.0]);
        }
        m.wv(pl + 0xb0, [b.v[0], b.v[1], b.v[2], 0.0]);
        m.wv(pa + 0xb0, [b.w[0], b.w[1], b.w[2], 0.0]);
    }
}

/// BOSS R26: the class-85 beams' barrel side. FUN_001f7598 hits the beams delivered to a barrel last frame
/// (`EngineHost::barrel_hits`) -> its slot 74 (`deliver`, zap 8: FUN_00183e08 explodes it, message 0xb), then
/// the in-world (+0x2c bit 28), unworn (+0x17c != 2), unexploded (!= 3) class-88 copies -> `barrels_85` for the
/// beam SO overlap (k15_85 slots 59 / 53). Worn barrels are hit through Sly (FUN_001a19a0 jt+0x4c8 == 1).
pub fn beams(m: &mut LiveMem, w: &crate::engine::world::World, host: &mut crate::engine::classes::plugin::EngineHost) {
    for h in std::mem::take(&mut host.barrel_hits) {
        let Some(&a) = (h.target as usize).checked_sub(1).and_then(|i| m.obj_addr.get(i)) else { continue };
        if m.r32(a + 0x17c) == 3 {
            continue;
        }
        let src = (h.src as usize).checked_sub(1).and_then(|i| m.obj_addr.get(i).copied()).unwrap_or(0);
        if std::env::var_os("EP3_MSG").is_some() { eprintln!("EP3M beam hit barrel {a:#x} src {src:#x} kind {} amt {}", h.kind, h.amount); }
        m.guarded(|m| deliver_msg(m, a, src, a, h.kind, h.amount, 8));
    }
    host.barrels_85.clear();
    for (i, o) in w.objects.iter().enumerate() {
        if o.class != 88 {
            continue;
        }
        let Some(&a) = m.obj_addr.get(i) else { continue };
        if m.r32(a + 0x2c) & 0x1000_0000 == 0 || matches!(m.r32(a + 0x17c), 2 | 3) {
            continue;
        }
        let Some(p) = m.guarded(|m| m.call(0x001a_62e8, &[a], &[]).1) else { continue };
        if p[..3].iter().all(|x| x.is_finite()) {
            host.barrels_85.push((i as u32 + 1, [p[0], p[1], p[2]]));
        }
    }
}

/// Entry point of the segment p0 -> p1 into the sphere (c, r) (p0 inside: p0).
pub fn seg_sphere(p0: [f32; 3], p1: [f32; 3], c: [f32; 3], r: f32) -> Option<[f32; 3]> {
    let d = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let f = [p0[0] - c[0], p0[1] - c[1], p0[2] - c[2]];
    let a = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
    let b = f[0] * d[0] + f[1] * d[1] + f[2] * d[2];
    let cc = f[0] * f[0] + f[1] * f[1] + f[2] * f[2] - r * r;
    if cc <= 0.0 {
        return Some(p0);
    }
    if a <= 0.0 || b > 0.0 {
        return None;
    }
    let disc = b * b - a * cc;
    if disc < 0.0 {
        return None;
    }
    let t = (-b - disc.sqrt()) / a;
    (t <= 1.0).then(|| [p0[0] + d[0] * t, p0[1] + d[1] * t, p0[2] + d[2] * t])
}
