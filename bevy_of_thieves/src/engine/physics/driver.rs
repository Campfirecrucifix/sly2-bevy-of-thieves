//! WP-E5: the real Sly 2 per-frame physics chain (call tree: tools/engineport/so_vtable_map.md "E5").
//! world update 0x226550 -> 0x1f8dd8 driver: [ALO slot16 0x1f3980 per object: slot69 0x1f9b20 forces
//! (0x1fa810 constraint, 0x1fac30 accel) + 0x22de88 xf integrator] -> 0x134040 broadphase -> 0x1f8f80(mode 1)
//! islands -> 0x1f9410 solve -> slot64 0x1f9e70 apply impulse -> 0x1f9c48 sleep.
//! 1:1: 0x1f3590 defaults, 0x1f9b20, 0x1fa810, 0x1fac30, 0x22de88 kinds 1/2 (dof 1 linear, dof 6 angular),
//! 0x1f9e70, 0x1f9c48 + its sleep test in 0x1f9410, 0x11b570 exp map, 0x11b3d0 orthonormalise, 0x11d4d0 approx-eq.
//! Replaced: 0x1f9410's island LCP (0x1585d8..0x158e20) by the E3 contact LCP (0x14eda0 / 0x1a93c8 / 0x11ad30)
//! producing the island's linear impulse; broadphase 0x134040 + narrowphase 0x1aa1d8 (level BSP only, no body/body).
//! Parent frames (0x22fcc8 with xf+0xa0) are identity: bodies are top-level.
#![allow(non_snake_case)]
use super::geom::{dot, V3};
use super::sphere::{FUN_0011ad30, FUN_001a93c8, Sbi};
use super::CollisionWorld;

pub type M3 = [V3; 3];
pub const IDENT: M3 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// 0x1f3590 (ALO slot63): default gravity e8+0x80 = (0,0,-1750) (0xc4dac000).
pub const GRAVITY_Z: f32 = -1750.0;
/// 0x1f9410 sleep thresholds: |vLin|^2 <= 25, |vAng|^2 <= 0.0025 (this and previous frame).
pub const SLEEP_LIN2: f32 = 25.0;
pub const SLEEP_ANG2: f32 = 0.0025;

/// Physics state of one ALO: xfLin (+0x58) / xfAng (+0x5c) and the +0xe8 physics block.
#[derive(Clone, Debug, PartialEq)]
pub struct AloPhys {
    /// xfAng rows +0x00..+0x2f
    pub rot: M3,
    /// xfLin +0x30
    pub pos: V3,
    /// xfLin +0xb0 / xfAng +0xb0
    pub v: V3,
    pub w: V3,
    /// xf +0xf0 accelerations
    pub a: V3,
    pub alpha: V3,
    /// xf +0x98 kind: 1/2 dynamic, 3 = asleep (0x230630 sets 1->3, 0x230598 wakes 3->1)
    pub kind: u16,
    /// e8+0x80 gravity
    pub gravity: V3,
    /// e8+0xb0 constraint normal, e8+0xc0 kind (0 free, 1 plane, 2 line, 3 locked)
    pub cnorm: V3,
    pub ckind: u32,
    /// ALO +0xf0 / +0x100 pending impulses (velocity deltas, consumed by 0x1f9e70)
    pub imp_lin: V3,
    pub imp_ang: V3,
    /// e8+0x60 / e8+0x70 previous-frame velocities (0x1f3980)
    pub prev_v: V3,
    pub prev_w: V3,
    /// DAT_003e11e8 generation stamp (xf +0xa4)
    pub xf_gen: u32,
    /// WP-E8: xf +0xa0 parent rows (None = top level): 0x1fa810 re-expresses the constraint normal by them,
    /// 0x1fac30 converts the acceleration with 0x22fcc8(0, parent, a).
    pub parent: Option<M3>,
}

impl Default for AloPhys {
    /// 0x1f3590 (gravity, constraint kind) — the game then sets e8+0xc0 per object; dynamic bodies use 0 (free).
    fn default() -> Self {
        Self {
            rot: IDENT,
            pos: [0.0; 3],
            v: [0.0; 3],
            w: [0.0; 3],
            a: [0.0; 3],
            alpha: [0.0; 3],
            kind: 1,
            gravity: [0.0, 0.0, GRAVITY_Z],
            cnorm: [0.0; 3],
            ckind: 0,
            imp_lin: [0.0; 3],
            imp_ang: [0.0; 3],
            prev_v: [0.0; 3],
            prev_w: [0.0; 3],
            xf_gen: 0,
            parent: None,
        }
    }
}

fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn scale(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn norm(a: V3) -> V3 {
    scale(a, 1.0 / dot(a, a).sqrt())
}
fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

/// 0x0011d4d0(a, b, eps): RELATIVE compare |a - b| / max(1, |a|) < eps (WP-E8: was absolute; equal for
/// |a| < 1, e.g. the zero-vector tests against DAT_002a2dd0).
pub fn FUN_0011d4d0(a: f32, b: f32, eps: f32) -> bool {
    let d = if a.abs() < 1.0 { 1.0 } else { a.abs() };
    (a - b).abs() / d < eps
}
/// The x/y/z 0x11d4d0 chain used everywhere against the zero vector DAT_002a2dd0.
pub fn near_zero(v: V3) -> bool {
    v.iter().all(|&x| FUN_0011d4d0(0.0, x, 1e-4))
}

/// 0x0011b500: rotation matrix (rows) of `angle` about unit `axis` (Rodrigues; row-vector convention).
pub fn FUN_0011b500(axis: V3, angle: f32) -> M3 {
    let (s, c) = angle.sin_cos();
    let t = 1.0 - c;
    let [x, y, z] = axis;
    [
        [t * x * x + c, t * x * y + s * z, t * x * z - s * y],
        [t * x * y - s * z, t * y * y + c, t * y * z + s * x],
        [t * x * z + s * y, t * y * z - s * x, t * z * z + c],
    ]
}

/// 0x0011b570: exp map of a rotation vector; |v| < 1e-4 -> identity (0x2a2d00).
pub fn FUN_0011b570(v: V3) -> M3 {
    let l = dot(v, v).sqrt();
    if l < 0.0001 { IDENT } else { FUN_0011b500(scale(v, 1.0 / l), l) }
}

/// 0x0011b3d0: Gram-Schmidt keeping row 0's direction: r0 = n(r0); r2 = n(r0 x r1); r1 = n(r2 x r0).
pub fn FUN_0011b3d0(m: &mut M3) {
    let r0 = norm(m[0]);
    let r2 = norm(cross(r0, m[1]));
    let r1 = norm(cross(r2, r0));
    *m = [r0, r1, r2];
}

/// rows' = rows * M (the vmula/vmadda sequence in 0x22de88 / 0x1f9e70 / 0x1f9c48), then 0x11b3d0.
fn rotate(rot: &mut M3, m: &M3) {
    let mul = |r: V3| add(add(scale(m[0], r[0]), scale(m[1], r[1])), scale(m[2], r[2]));
    *rot = [mul(rot[0]), mul(rot[1]), mul(rot[2])];
    FUN_0011b3d0(rot);
}

/// 0x001fa810(constraint e8+0xb0, xf, g): project g by the constraint kind (+0x10): 1 plane g-(g.n)n,
/// 2 line (g.n)n, 3 locked 0, else g. (Parent-frame rotation of n when xf+0xa0 != 0 is identity here.)
pub fn FUN_001fa810(ckind: u32, n: V3, g: V3) -> V3 {
    match ckind {
        1 => add(g, scale(n, -dot(g, n))),
        2 => scale(n, dot(g, n)),
        3 => [0.0; 3],
        _ => g,
    }
}

/// 0x001fa810 with the xf+0xa0 parent (WP-E8 1:1): kinds 1 / 2 use d' = n(d R_parent) (rows +0x40..+0x6f,
/// row-vector product, then normalised); no parent -> d as stored.
pub fn FUN_001fa810_xf(ckind: u32, n: V3, parent: Option<&M3>, g: V3) -> V3 {
    let d = match (ckind, parent) {
        (1 | 2, Some(r)) => norm(add(add(scale(r[0], n[0]), scale(r[1], n[1])), scale(r[2], n[2]))),
        _ => n,
    };
    FUN_001fa810(ckind, d, g)
}

/// 0x001fac30(alo, a): if 0x230380(xf kind) xf+0xf0 += 0x22fcc8(0, xf+0xa0, a) (world -> parent frame).
pub fn FUN_001fac30(b: &mut AloPhys, a: V3) {
    if FUN_00230380(b.kind) {
        let a = super::driven::FUN_0022fcc8(None, b.parent.as_ref(), a);
        b.a = add(b.a, a);
    }
}

/// WP-E8 constraint following (REPLACED: the original keeps a constrained body on its constraint through
/// the response tables of the island rows (+0x54), not ported): velocity deltas consumed by 0x1f9e70 and the
/// integrated velocity are projected with the same 0x1fa810 rule as gravity (1 plane, 2 line, 3 locked).
pub fn constrain(b: &AloPhys, v: V3) -> V3 {
    if b.ckind == 0 { v } else { FUN_001fa810_xf(b.ckind, b.cnorm, b.parent.as_ref(), v) }
}

/// 0x00230380: xf kind in 1..=3 (has dynamics).
pub fn FUN_00230380(kind: u16) -> bool {
    (1..=3).contains(&kind)
}

/// 0x001f9b20 (ALO slot69): gravity e8+0x80 unless ~0 (0x11d4d0 x/y/z vs DAT_002a2dd0) or constraint kind 3
/// -> 0x1fa810 (constraint, parent frame) -> 0x1fac30. The +0x138 attachment walk 0x229798 is
/// `FUN_001f9b20_att` (AloPhys holds no attachment list).
pub fn FUN_001f9b20(b: &mut AloPhys) {
    if !near_zero(b.gravity) && b.ckind != 3 {
        let g = FUN_001fa810_xf(b.ckind, b.cnorm, b.parent.as_ref(), b.gravity);
        FUN_001fac30(b, g);
    }
}

/// 0x001f9b20 with its tail: +0x138 non-empty -> 0x229798 (attachments placed relative to this body).
pub fn FUN_001f9b20_att(b: &mut AloPhys, att: &[super::driven::Attachment], place: impl FnMut(usize, M3, V3)) {
    FUN_001f9b20(b);
    if !att.is_empty() {
        super::driven::FUN_00229798((&b.rot, b.pos), att, place);
    }
}

/// 0x0022de88 kinds 1/2: dof 1 (xfLin): v' = v + a dt; both ~0 -> v = 0 (kind 1: 0x2303a0 re-evaluates
/// the kind; modelled as staying 1); else pos += (v + v') dt/2. dof 6 (xfAng) likewise with exp map.
pub fn FUN_0022de88(b: &mut AloPhys, dt: f32) {
    if !matches!(b.kind, 1 | 2) {
        return;
    }
    let v1 = add(b.v, scale(b.a, dt));
    if near_zero(v1) && near_zero(b.v) {
        b.v = [0.0; 3];
    } else {
        b.pos = add(b.pos, scale(add(v1, b.v), dt * 0.5));
        b.v = v1;
        b.xf_gen += 1;
    }
    let w1 = add(b.w, scale(b.alpha, dt));
    if near_zero(w1) && near_zero(b.w) {
        b.w = [0.0; 3];
    } else {
        let m = FUN_0011b570(scale(add(b.w, w1), dt * 0.5));
        rotate(&mut b.rot, &m);
        b.w = w1;
        b.xf_gen += 1;
    }
}

/// PART-3: full 0x0022de88 switch on xf +0x98: kinds 1/2 integrate the body (`FUN_0022de88`), kinds 4..0xb
/// the driven xf (`driven::FUN_0022de88_driven`, 1:1), kind 3 (asleep) / others no-op.
pub fn FUN_0022de88_all(b: &mut AloPhys, x: Option<&mut super::driven::DrivenXf>, now: f32, dt: f32, mirror: Option<(V3, u32)>) {
    match (b.kind, x) {
        (1 | 2, _) => FUN_0022de88(b, dt),
        (_, Some(x)) if (4..=0xb).contains(&x.kind) => super::driven::FUN_0022de88_driven(x, now, dt, mirror),
        _ => {}
    }
}

/// 0x001f3980 (ALO slot16) physics part: remember previous velocities, zero accel (kinds 1..3), slot69, 0x22de88.
pub fn FUN_001f3980(b: &mut AloPhys, dt: f32) {
    b.prev_v = b.v;
    b.prev_w = b.w;
    if matches!(b.kind, 1 | 2) {
        b.a = [0.0; 3];
        b.alpha = [0.0; 3];
        FUN_001f9b20(b);
    }
    FUN_0022de88(b, dt);
}

/// 0x001f9e70 (ALO slot64): consume +0xf0/+0x100 impulses (0x230598 wakes kind 3 -> 1); v += J; pos += J dt/2;
/// w += Ja; R = expmap(Ja dt/2) R; generation bump.
pub fn FUN_001f9e70(b: &mut AloPhys, dt: f32) {
    let (jl, ja) = (constrain(b, b.imp_lin), b.imp_ang);
    b.imp_lin = jl;
    let (zl, za) = (near_zero(jl), near_zero(ja));
    if !zl {
        if b.kind == 3 {
            b.kind = 1;
        }
        b.v = add(b.v, jl);
        b.imp_lin = [0.0; 3];
    }
    if !za {
        if b.kind == 3 {
            b.kind = 1;
        }
        b.w = add(b.w, ja);
        b.imp_ang = [0.0; 3];
    }
    if zl && za {
        return;
    }
    if !zl {
        b.pos = add(b.pos, scale(jl, dt * 0.5));
        b.xf_gen += 1;
    }
    if !za {
        rotate(&mut b.rot, &FUN_0011b570(scale(ja, dt * 0.5)));
        b.xf_gen += 1;
    }
}

/// 0x001fa328 (ALO vt +0x214, the 0x1f9410 bit-0-clear apply: mode 2 / restitution pass): consume +0xf0 / +0x100
/// impulses (0x230598 wakes kind 3 -> 1, 0x22fcc8 constraint) into v / w (+0xb0) only, 0x1f4bf8 notify per part.
/// Unlike 0x1f9e70 (vt +0x20c, mode 1) there is NO pos / rotation half-step. FEEL R115: vt 0x2d2e98 words +0x20c =
/// 0x1f9e70, +0x214 = 0x1fa328, +0x21c = 0x1fa528 (read from SCUS_973.16; the earlier "0x214 = 0x1f9e70" was off by one).
pub fn FUN_001fa328(b: &mut AloPhys) {
    let (jl, ja) = (constrain(b, b.imp_lin), b.imp_ang);
    if !near_zero(b.imp_lin) {
        if b.kind == 3 {
            b.kind = 1;
        }
        b.v = add(b.v, jl);
        b.imp_lin = [0.0; 3];
    }
    if !near_zero(ja) {
        if b.kind == 3 {
            b.kind = 1;
        }
        b.w = add(b.w, ja);
        b.imp_ang = [0.0; 3];
    }
}

/// 0x001f9c48: put to sleep — undo half a step (pos -= v dt/2, R = expmap(-w dt/2) R), v = w = 0, kind 1 -> 3.
pub fn FUN_001f9c48(b: &mut AloPhys, dt: f32) {
    if b.kind == 1 {
        b.kind = 3;
    }
    b.pos = add(b.pos, scale(b.v, -dt * 0.5));
    rotate(&mut b.rot, &FUN_0011b570(scale(b.w, -dt * 0.5)));
    b.v = [0.0; 3];
    b.w = [0.0; 3];
    b.xf_gen += 1;
}

/// 0x1f9410 sleep test: current and previous velocities under the thresholds.
pub fn sleep_test(b: &AloPhys) -> bool {
    dot(b.v, b.v) <= SLEEP_LIN2 && dot(b.w, b.w) <= SLEEP_ANG2 && dot(b.prev_v, b.prev_v) <= SLEEP_LIN2 && dot(b.prev_w, b.prev_w) <= SLEEP_ANG2
}

/// Off-centre impulse helper (REPLACED: what 0x1f9410's contact rows produce): linear J, angular (p - c) x J * 5/(2 r^2)
/// (solid sphere, unit mass).
pub fn impulse_at(b: &mut AloPhys, j: V3, p: V3, radius: f32) {
    b.imp_lin = add(b.imp_lin, j);
    let r = [p[0] - b.pos[0], p[1] - b.pos[1], p[2] - b.pos[2]];
    b.imp_ang = add(b.imp_ang, scale(cross(r, j), 2.5 / (radius * radius)));
}

/// One frame of 0x1f8dd8 for a single sphere body against the level: slot16, contacts (0x1f9410 mode 1 velocity pass,
/// slot64, re-gather, mode 4 positional pass gated as 0x1f9240; PHYS1: no velocity penetration bias), sleep. Returns the mode-1 contacts' normals.
pub fn FUN_001f8dd8(cw: &CollisionWorld, b: &mut AloPhys, radius: f32, dt: f32) -> Vec<V3> {
    if b.kind == 3 && near_zero(b.imp_lin) && near_zero(b.imp_ang) {
        return Vec::new(); // asleep: skipped by 0x1f8f80 mode 1 (all xfs static per 0x230348)
    }
    FUN_001f3980(b, dt);
    // contact gather (0x1f9240 -> level_contacts, max 0x40; narrowphase = 0x1a93c8 per collider) for SolveIslands(1)
    let mut sbi = Vec::new();
    for col in &cw.colliders {
        let left = 0x40 - sbi.len();
        if left == 0 {
            break;
        }
        if !col.near(b.pos, radius) {
            continue;
        }
        FUN_001a93c8(radius, b.pos, &col.geom, &col.bsp, left, &mut sbi);
    }
    let n = sbi.len();
    let nrm = |s: &Sbi| [s.plane[0], s.plane[1], s.plane[2]];
    let normals: Vec<V3> = sbi.iter().map(nrm).collect();
    let mut a = vec![0.0f32; n * n];
    for i in 0..n {
        for k in 0..n {
            a[i * n + k] = dot(normals[i], normals[k]);
        }
    }
    // SolveIslands(1) (0x1f8f80: runs for a moving body vs the static level): 0x1f9410 mode 1 velocity LCP,
    // b = n.v (contact +0xb0 relative velocity), bound 0x158e20(2 sqrt(max |v|^2)); slot 0x20c -> J (0x1f9e70).
    let vn: Vec<f32> = normals.iter().map(|nn| dot(*nn, b.v)).collect();
    if n > 0 {
        let speed = dot(b.v, b.v).sqrt();
        let x = FUN_0011ad30(speed + speed, &a, &vn);
        let mut j = [0.0f32; 3];
        for i in 0..n {
            j = add(j, scale(normals[i], x[i]));
        }
        b.imp_lin = add(b.imp_lin, j);
    }
    // mode 2 (only when some contact n.v < -5 after mode 1): b = n.v (1 + e rand(0.9, 1.1)) with e = +0x98 = 0
    // for these level contacts = the mode-1 system again, already satisfied: not repeated.
    FUN_001f9e70(b, dt);
    // SolveIslands(6) re-gathers (0x1f9240 -> level_contacts) after the mode-1 apply and UpdateBroadphase
    let mut sbi = Vec::new();
    for col in &cw.colliders {
        let left = 0x40 - sbi.len();
        if left == 0 {
            break;
        }
        if !col.near(b.pos, radius) {
            continue;
        }
        FUN_001a93c8(radius, b.pos, &col.geom, &col.bsp, left, &mut sbi);
    }
    let n = sbi.len();
    let n6: Vec<V3> = sbi.iter().map(nrm).collect();
    let mut a = vec![0.0f32; n * n];
    for i in 0..n {
        for k in 0..n {
            a[i * n + k] = dot(n6[i], n6[k]);
        }
    }
    // contact +0x90 = signed gap (plane w = n.q, 0x119390): d = n.c - w - r (< 0 = penetration)
    let pos = b.pos;
    let gap: Vec<f32> = sbi.iter().map(|s| dot(nrm(s), pos) - s.plane[3] - radius).collect();
    // SolveIslands(6), mode 4 (0x1f9240: only when some contact has +0x90 < -2): positional LCP, b = +0x90 + 1,
    // bound 0x158e20(2 max|+0x90|); slot 0x21c moves the body by sum x n. Velocity is untouched.
    if gap.iter().any(|&d| d < -2.0) {
        let bb: Vec<f32> = gap.iter().map(|d| d + 1.0).collect();
        let m = gap.iter().fold(0.0f32, |m, d| m.max(d.abs()));
        let x = FUN_0011ad30(m + m, &a, &bb);
        for i in 0..n {
            b.pos = add(b.pos, scale(n6[i], x[i]));
        }
        b.xf_gen += 1;
    }
    if sleep_test(b) {
        FUN_001f9c48(b, dt);
    }
    normals
}
