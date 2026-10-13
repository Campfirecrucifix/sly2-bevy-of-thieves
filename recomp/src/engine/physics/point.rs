//! WP-E2: point / point-cloud / sphere queries against a BSPC (0x00118fb0, 0x00119090, 0x00118d08, 0x00118a88).
//! Node references are pointers in the original; here node indices (>= `limit` count as null, like
//! the `node < param_2` tests). Plane distance = the VU0 idiom n.p - w (see bsp.rs `dist`).
#![allow(non_snake_case)]

use super::geom::{dot, BspC, CGeom, Plane, V3, LEAF_PLANE};

fn plane_of(g: &CGeom, s: Option<usize>) -> Plane {
    match s {
        None => LEAF_PLANE,
        Some(i) => g.surfs.get(i).map(|s| s.plane).unwrap_or([0.0; 4]),
    }
}
fn dist(p: Plane, v: V3) -> f32 {
    dot([p[0], p[1], p[2]], v) - p[3]
}
fn lim(b: &BspC, limit: usize, i: Option<usize>) -> Option<usize> {
    i.filter(|&i| i < limit && i < b.nodes.len())
}

/// 0x00118fb0 (PointInBsp-like): walk down; d >= 1e-4 -> positive child (null = empty, returns None),
/// d < 1e-4 -> negative child; returns the last node taken on the negative side whose child is null
/// (= solid). Some(node) = the point is inside.
pub fn FUN_00118fb0(g: &CGeom, b: &BspC, root: usize, limit: usize, p: V3) -> Option<usize> {
    let mut cur = lim(b, limit, Some(root));
    while let Some(n) = cur {
        let node = b.nodes[n];
        if dist(plane_of(g, node.surf), p) < 0.0001 {
            match lim(b, limit, node.child[0]) {
                None => return Some(n),
                c => cur = c,
            }
        } else {
            cur = lim(b, limit, node.child[1]);
        }
    }
    None
}

/// 0x00119090: inclusive variant: within +-1e-4 of a plane both sides are tried (positive first);
/// returns the solid node found or None.
pub fn FUN_00119090(g: &CGeom, b: &BspC, root: usize, limit: usize, p: V3) -> Option<usize> {
    let mut n = lim(b, limit, Some(root))?;
    loop {
        let node = b.nodes[n];
        let d = dist(plane_of(g, node.surf), p);
        let next = if d < -0.0001 {
            match lim(b, limit, node.child[0]) {
                None => return Some(n),
                c => c,
            }
        } else if d <= 0.0001 {
            if let Some(c) = lim(b, limit, node.child[1]) {
                if let Some(r) = FUN_00119090(g, b, c, limit, p) {
                    return Some(r);
                }
            }
            match lim(b, limit, node.child[0]) {
                None => return Some(n),
                c => c,
            }
        } else {
            lim(b, limit, node.child[1])
        };
        n = next?;
    }
}

/// Result classes of 0x00118a88 / 0x00118d08: 0 = solid, 1 = empty, 2 = mixed.
pub const SOLID: u32 = 0;
pub const EMPTY: u32 = 1;
pub const MIXED: u32 = 2;

/// 0x00118d08: convex point set `pts` vs BSP. min d > 1e-4 -> positive child, max d < -1e-4 -> negative
/// child (null = solid); straddling nodes recurse both sides; when the sides differ (or both mixed) and budget `*left > 0`,
/// the node's surface is collected once (surface flag +0x18 & 0x4000 -> `marked`). Returns 0/1/2.
/// Quirk kept: the budget is only tested, never decremented here (the caller bounds the output).
pub fn FUN_00118d08(g: &CGeom, b: &BspC, root: usize, limit: usize, pts: &[V3], left: &mut i32, marked: &mut Vec<bool>, out: &mut Vec<usize>) -> u32 {
    if pts.is_empty() {
        return EMPTY;
    }
    let mut cur = lim(b, limit, Some(root));
    while let Some(n) = cur {
        let node = b.nodes[n];
        let pl = plane_of(g, node.surf);
        let (mut mn, mut mx) = (dist(pl, pts[0]), dist(pl, pts[0]));
        for p in &pts[1..] {
            let d = dist(pl, *p);
            mn = mn.min(d);
            mx = mx.max(d);
        }
        if 0.0001 < mn {
            cur = lim(b, limit, node.child[1]);
            continue;
        }
        if mx < -0.0001 {
            cur = lim(b, limit, node.child[0]);
            if cur.is_none() {
                return SOLID;
            }
            continue;
        }
        let a = match lim(b, limit, node.child[0]) {
            None => SOLID,
            Some(c) => FUN_00118d08(g, b, c, limit, pts, left, marked, out),
        };
        let c = match lim(b, limit, node.child[1]) {
            None => EMPTY,
            Some(c) => FUN_00118d08(g, b, c, limit, pts, left, marked, out),
        };
        if a == c && a != MIXED {
            return a;
        }
        if 0 < *left {
            if let Some(s) = node.surf {
                if marked.len() <= s {
                    marked.resize(s + 1, false);
                }
                if !marked[s] {
                    marked[s] = true;
                    out.push(s);
                }
            }
            return MIXED;
        }
        return EMPTY;
    }
    EMPTY
}

/// Child of a pruned node (0x00118a88 output).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PChild {
    /// null pointer (negative side: solid, positive side: empty)
    Null,
    /// DAT_002bbb10: node with plane DAT_002bbad0 (0,0,0,-1e8) and null children = empty leaf
    Empty,
    /// DAT_002bbb20: node with plane DAT_002bbaf0 (0,0,0,1e8) = solid leaf
    Solid,
    /// index into the pruned node list
    Node(usize),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PNode {
    pub child: [PChild; 2],
    /// original surface (None = DAT_002bbaf0)
    pub surf: Option<usize>,
}

/// 0x00118a88 (PruneBsp-like): sphere (centre `c`, radius `r`) vs BSP; builds a pruned BSP of only the
/// straddled planes into `out` (budget `*left`, decremented per node; children are emitted first, so a
/// node's subtree root is the last record written). Returns 0 solid / 1 empty / 2 mixed; when the budget
/// is exhausted on a mixed node the original returns 1 (empty) — kept.
pub fn FUN_00118a88(g: &CGeom, b: &BspC, root: usize, limit: usize, c: V3, r: f32, left: &mut i32, out: &mut Vec<PNode>) -> u32 {
    let mut cur = lim(b, limit, Some(root));
    let mut d = 0.0f32;
    while let Some(n) = cur {
        let node = b.nodes[n];
        d = dist(plane_of(g, node.surf), c);
        if r < d {
            cur = lim(b, limit, node.child[1]);
            continue;
        }
        if d < -r {
            cur = lim(b, limit, node.child[0]);
            continue;
        }
        let (u4, i9) = match lim(b, limit, node.child[0]) {
            None => (SOLID, None),
            Some(k) => {
                let v = FUN_00118a88(g, b, k, limit, c, r, left, out);
                (v, out.len().checked_sub(1))
            }
        };
        let (u5, i7) = match lim(b, limit, node.child[1]) {
            None => (EMPTY, None),
            Some(k) => {
                let v = FUN_00118a88(g, b, k, limit, c, r, left, out);
                (v, out.len().checked_sub(1))
            }
        };
        if u4 == u5 && u5 != MIXED {
            return u5;
        }
        if *left <= 0 {
            return EMPTY;
        }
        *left -= 1;
        let neg = match u4 {
            EMPTY => PChild::Empty,
            SOLID => PChild::Null,
            _ => i9.map_or(PChild::Null, PChild::Node),
        };
        let pos = match u5 {
            EMPTY => PChild::Null,
            SOLID => PChild::Solid,
            _ => i7.map_or(PChild::Null, PChild::Node),
        };
        out.push(PNode { child: [neg, pos], surf: node.surf });
        return MIXED;
    }
    // loop left through a null child: class of the last tested side
    if 0.0 < d { EMPTY } else { SOLID }
}
