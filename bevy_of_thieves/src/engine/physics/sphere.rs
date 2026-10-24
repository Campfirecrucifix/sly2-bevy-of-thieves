//! WP-E3: sphere contacts (0x001a93c8, Sly 1 CsbiIntersectSphereBsp family), bounded LCP (0x0011ad30, Sly 1
//! SolveInequalities/DZG) and the sphere sweep 0x0014eda0 the follow camera uses (camera::FollowWorld::sweep_sphere).
#![allow(non_snake_case)]

use super::bsp::FUN_00118668;
use super::geom::{dot, sub, CGeom, Plane, V3};
use super::point::{FUN_00118a88, FUN_00119090, EMPTY, SOLID};

/// 0x001a93c8 tail: no contacts and the centre inside solid (0x119090) -> one contact from 0x1a9108.
fn deep(r: f32, c: V3, g: &CGeom, bsp: &super::geom::BspC, limit: usize, out: &mut Vec<Sbi>) -> usize {
    if FUN_00119090(g, bsp, 0, limit, c).is_none() {
        return 0;
    }
    match FUN_001a9108_lim(r, c, g, bsp, limit) {
        Some(s) => {
            out.push(s);
            1
        }
        None => 0,
    }
}

/// 0x001a9108 deep penetration (WP-E5): R = max(r, 100), doubling while R < 52000: prune at R (budget 0x1555);
/// if mixed, per pruned surface (deduped by surface flag 0x1000) clip c + n (R + 1) -> c (max 0x10) and keep the
/// first interval with the largest t_in > 0: pos = its entry point, plane = 0x119390(normalize(p1 - c), pos),
/// kind 0xc. First R with a hit wins. (Clip root = full BSP instead of the pruned tree; same intervals near c.)
pub fn FUN_001a9108(r: f32, c: V3, g: &CGeom, bsp: &super::geom::BspC) -> Option<Sbi> {
    FUN_001a9108_lim(r, c, g, bsp, bsp.nodes.len())
}

/// FUN_001a9108 with the node limit `limit` (a1 of 0x1a93c8, see `FUN_001a93c8_lim`).
pub fn FUN_001a9108_lim(r: f32, c: V3, g: &CGeom, bsp: &super::geom::BspC, limit: usize) -> Option<Sbi> {
    let mut rr = if 100.0 < r { r } else { 100.0 };
    while rr < 52000.0 {
        let mut left = 0x1555i32;
        let mut pruned = Vec::new();
        if FUN_00118a88(g, bsp, 0, limit, c, rr, &mut left, &mut pruned) == 2 {
            let mut best = 0.0f32;
            let mut hit = None;
            let mut seen = vec![false; g.surfs.len()];
            for p in &pruned {
                let Some(s) = p.surf else { continue };
                if seen[s] {
                    continue;
                }
                seen[s] = true;
                let pl = g.surfs[s].plane;
                let p1 = madd(c, [pl[0], pl[1], pl[2]], rr + 1.0);
                let mut lsg = Vec::new();
                if FUN_00118668(g, bsp, 0, limit, p1, c, 0x10, &mut lsg) > 0 && best < lsg[0].t[0] {
                    best = lsg[0].t[0];
                    let d = sub(p1, c);
                    let n = { let l = dot(d, d).sqrt(); [d[0] / l, d[1] / l, d[2] / l] };
                    hit = Some(Sbi { pos: lsg[0].pos[0], plane: FUN_00119390(n, lsg[0].pos[0]), kind: 0xc, feature: s });
                }
            }
            if best > 0.0 {
                return hit;
            }
        }
        rr += rr;
    }
    None
}
use super::query::CollisionWorld;

/// SBI record (0x30 B): +0 contact position, +0x10 plane (normal, w = n.point), +0x20 kind, +0x24 feature.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sbi {
    pub pos: V3,
    pub plane: Plane,
    /// 6 face, 5 edge, 4 vertex
    pub kind: u32,
    /// surface / edge / vertex index (by kind)
    pub feature: usize,
}

fn madd(a: V3, b: V3, s: f32) -> V3 {
    [a[0] + b[0] * s, a[1] + b[1] * s, a[2] + b[2] * s]
}
fn len2(a: V3) -> f32 {
    dot(a, a)
}

/// 0x00119390: plane through `p` with normal `n` (single implementation in `geom`).
pub use super::geom::FUN_00119390;

/// 0x001a93c8(r, centre, collider, limit, ..., max, out): sphere vs one collider's BSP. Prunes the BSP to the
/// sphere (0x00118a88, budget min(+0x6c, 0x1555)); solid/empty -> no candidates. Candidate surfaces (flag 0x10)
/// get, in reverse pruned order: face contacts (kind 6) when the diameter c+r*n -> c-r*n clipped through the
/// BSP (0x00118668, max 0x10) enters/leaves on that same surface (t_in+t_out > 1 picks the entry surface; flag 0x80);
/// then edge contacts (kind 5) on each surface's own edges whose other surface is a candidate without a face
/// contact, closest point strictly inside (0.0001 < t < 0.9999) within r; edge ends become vertex candidates
/// (max 0x100, deduplicated) emitted last (kind 4) unless an edge contact already uses that vertex.
/// Contact pos = c - r*n for every kind. Returns the number written (<= max).
/// No contacts and the centre inside solid (0x00119090): 0x001a9108 deep-penetration fallback (deep()).
pub fn FUN_001a93c8(r: f32, c: V3, g: &CGeom, bsp: &super::geom::BspC, max: usize, out: &mut Vec<Sbi>) -> usize {
    FUN_001a93c8_lim(r, c, g, bsp, bsp.nodes.len(), max, out)
}

/// R29: 0x001a93c8 with its a2 node limit (saved at sp+0x364c, passed as a1 to 0x00118a88 / 0x00119090 /
/// 0x001a9108): BSP nodes at index >= `limit` count as null. The other callers pass 0x7fffffff (whole tree,
/// `FUN_001a93c8`); 0x0014eda0 passes part +0x74 + part +0x70 * 0xc = the BSPC third word (`BspC::b`, the
/// camera node limit, ee_sub_118220).
pub fn FUN_001a93c8_lim(r: f32, c: V3, g: &CGeom, bsp: &super::geom::BspC, limit: usize, max: usize, out: &mut Vec<Sbi>) -> usize {
    if max == 0 || bsp.nodes.is_empty() {
        return 0;
    }
    let mut left = 0x1555i32.min(bsp.nodes.len() as i32);
    let mut pruned = Vec::new();
    let res = FUN_00118a88(g, bsp, 0, limit, c, r, &mut left, &mut pruned);
    if res == EMPTY {
        return 0;
    }
    if res == SOLID {
        return deep(r, c, g, bsp, limit, out);
    }
    let start = out.len();
    let ns = g.surfs.len();
    let mut flag = vec![0u32; ns];
    for p in &pruned {
        if let Some(s) = p.surf {
            flag[s] |= 0x10;
        }
    }
    // faces
    for p in pruned.iter().rev() {
        let Some(s) = p.surf else { continue };
        if flag[s] & 0x10 == 0 || flag[s] & 0x40 != 0 {
            continue;
        }
        flag[s] |= 0x40;
        let pl = g.surfs[s].plane;
        let n = [pl[0], pl[1], pl[2]];
        let (p1, p2) = (madd(c, n, r), madd(c, n, -r));
        let mut lsg = Vec::new();
        let k = FUN_00118668(g, bsp, 0, limit, p1, p2, 0x10, &mut lsg);
        for l in &lsg[..k.min(lsg.len())] {
            let hit = if 1.0 < l.t[0] + l.t[1] { l.surf[0] } else { l.surf[1] };
            if hit == Some(s) {
                if out.len() - start < max {
                    out.push(Sbi { pos: p2, plane: pl, kind: 6, feature: s });
                    if out.len() - start >= max {
                        return max;
                    }
                }
                flag[s] |= 0x80;
                break;
            }
        }
    }
    let first_edge = out.len();
    let mut verts: Vec<(usize, V3, Plane)> = Vec::new();
    for p in pruned.iter().rev() {
        let Some(s) = p.surf else { continue };
        if flag[s] & 0x10 == 0 || flag[s] & 0xa0 != 0 {
            continue;
        }
        flag[s] |= 0x20;
        for &ei in &g.surfs[s].own_edges {
            let e = g.edges[ei];
            let o = e.other as usize;
            if o >= ns || flag[o] & 0x10 == 0 || flag[o] & 0x80 != 0 {
                continue;
            }
            let (a, b) = (g.verts[e.a as usize], g.verts[e.b as usize]);
            let d = sub(b, a);
            let t = dot(sub(c, a), d) / len2(d);
            let (vid, q) = if t < 0.0001 {
                (Some(e.a as usize), a)
            } else if 0.9999 < t {
                (Some(e.b as usize), b)
            } else {
                (None, madd(a, d, t))
            };
            let v = sub(c, q);
            if len2(v) > r * r {
                continue;
            }
            if let Some(vi) = vid {
                if verts.iter().any(|x| x.0 == vi) {
                    continue;
                }
            }
            let l = len2(v).sqrt();
            let n = [v[0] / l, v[1] / l, v[2] / l];
            let pl = FUN_00119390(n, q);
            let pos = madd(c, n, -r);
            match vid {
                None => {
                    if out.len() - start >= max {
                        continue;
                    }
                    out.push(Sbi { pos, plane: pl, kind: 5, feature: ei });
                    if out.len() - start >= max {
                        return max;
                    }
                }
                Some(vi) => {
                    if verts.len() < 0x100 {
                        verts.push((vi, pos, pl));
                    }
                }
            }
        }
    }
    for &(vi, pos, pl) in verts.iter().rev() {
        let used = out[first_edge..].iter().any(|x| x.kind == 5 && {
            let e = g.edges[x.feature];
            e.a as usize == vi || e.b as usize == vi
        });
        if used {
            continue;
        }
        if out.len() - start < max {
            out.push(Sbi { pos, plane: pl, kind: 4, feature: vi });
            if out.len() - start >= max {
                break;
            }
        }
    }
    if out.len() == start {
        return deep(r, c, g, bsp, limit, out);
    }
    out.len() - start
}

/// 0x0011ad30(gMax, n, A (n x n), b, x out): bounded LCP of the DZG solver (DZ node 0x28 B: +8 upper bound gMax,
/// +0xc x, +0x10 w = b + A x). Finds 0 <= x <= gMax with w >= 0 where x < gMax and x*w = 0; the original pivots
/// (0x0011abf8/0x0011a770) while some w < -0.01. REPLACED: projected Gauss-Seidel to the same tolerance gives the
/// same complementary solution for the PSD A = N N^T it is called with (not pivot-for-pivot identical).
pub fn FUN_0011ad30(g_max: f32, a: &[f32], b: &[f32]) -> Vec<f32> {
    let n = b.len();
    let mut x = vec![0.0f32; n];
    for _ in 0..200 {
        let mut worst = 0.0f32;
        for i in 0..n {
            let aii = a[i * n + i];
            if aii <= 0.0 {
                continue;
            }
            let w: f32 = b[i] + (0..n).map(|j| a[i * n + j] * x[j]).sum::<f32>();
            let nx = (x[i] - w / aii).clamp(0.0, g_max);
            if (w < -0.01 && x[i] < g_max) || (w > 0.01 && x[i] > 0.0) {
                worst = worst.max(w.abs());
            }
            x[i] = nx;
        }
        if worst == 0.0 {
            break;
        }
    }
    x
}

/// 0x0011e9f8: clamp |v| to `max`.
pub fn FUN_0011e9f8(max: f32, v: V3) -> V3 {
    let l = len2(v).sqrt();
    if max < l { [v[0] * (max / l), v[1] * (max / l), v[2] * (max / l)] } else { v }
}

/// DAT_002bd8d4: LCP bound scale (x_i <= 4 * radius).
pub const LCP_SCALE: f32 = 4.0;

/// 0x0014eda0(radius, max_len, ctx, n objects, objects, callback, centre, vel): gathers up to 0x40 contacts
/// over the objects (0x001a93c8 per object), A_ij = n_i.n_j, b_i = callback(ctx, centre, vel, sbi_i),
/// x = 0x0011ad30(4*radius), returns vel + clamp(sum x_i n_i, max_len) (vel unchanged without contacts).
pub fn FUN_0014eda0(cw: &CollisionWorld, radius: f32, max_len: f32, c: V3, vel: V3, cb: &mut dyn FnMut(V3, V3, &Sbi) -> f32) -> V3 {
    let mut sbi = Vec::new();
    for col in &cw.colliders {
        // R29: both callers (0x0014f110, 0x00223a70) gather with filter 0x0014f030 (`lw v0,0x70(a1); slt v0,
        // zero,v0`: part BSPC third word > 0) and 0x14eda0 limits each part's BSP to nodes < that word
        let lim = col.bsp.b as i32;
        if lim <= 0 {
            continue;
        }
        let left = 0x40 - sbi.len();
        if left == 0 {
            break;
        }
        FUN_001a93c8_lim(radius, c, &col.geom, &col.bsp, lim as usize, left, &mut sbi);
    }
    let n = sbi.len();
    if n == 0 {
        return vel;
    }
    let nrm = |s: &Sbi| [s.plane[0], s.plane[1], s.plane[2]];
    let mut a = vec![0.0f32; n * n];
    for i in 0..n {
        for j in 0..n {
            a[i * n + j] = dot(nrm(&sbi[i]), nrm(&sbi[j]));
        }
    }
    let b: Vec<f32> = sbi.iter().map(|s| cb(c, vel, s)).collect();
    let x = FUN_0011ad30(LCP_SCALE * radius, &a, &b);
    let mut dv = [0.0f32; 3];
    for i in 0..n {
        dv = madd(dv, nrm(&sbi[i]), x[i]);
    }
    let dv = FUN_0011e9f8(max_len, dv);
    [vel[0] + dv[0], vel[1] + dv[1], vel[2] + dv[2]]
}

/// Non-penetration callback (b_i = n_i.vel): the corrected velocity has no component into any contact.
/// The camera's own callback 0x0014f040 (b = n.vel - approach(gap - radius) via 0x0011d018) is not ported.
pub fn cb_nonpenetrate(_c: V3, vel: V3, s: &Sbi) -> f32 {
    dot([s.plane[0], s.plane[1], s.plane[2]], vel)
}

impl CollisionWorld {
    /// camera::FollowWorld::sweep_sphere-compatible entry (0x0014eda0 with the non-penetration callback).
    pub fn sweep_sphere(&self, radius: f32, max_len: f32, start: V3, vel: V3) -> V3 {
        FUN_0014eda0(self, radius, max_len, start, vel, &mut cb_nonpenetrate)
    }
}
