//! WP-Q2: run 0x001bd040..0x001c0320 - the convex "zone" polygons (pool 0x3e1a40, class 0x2cd objects:
//! +0xe8 point count, +0xf4 local points, +0x1a0/+0x1b0 world bounds, +0x1c4 child DList, +0x1d8 parent,
//! +0x1ec edge planes), the zone-merge (convex hull) builder, two projected-texture wrappers, and the
//! 2D navmesh: BSP point location / segment traversal (bsp +0xc leaves (3 node ptrs, stride 0xc), +0x10 node
//! count, +0x14 nodes {a,b,c, u16 back @0xc, u16 front @0xe}), A* over the vertex graph (nodes stride 0x20:
//! pos, +0x10 edge count, +0x14 edges {node, cost}) and string pulling.
//! 1:1 against `QMem`; callees outside the run go through `QMem::call` by address. Address-taken locals
//! live on the emulated stack (`stack_push`). Distances in the pathfinder are 2D (VU `mul.xy`).
//! SKIPPED: FUN_001be398 (GS packet: render-target clear/upload for the projected texture) and
//! FUN_001be6f0 (projector camera matrix + VU1 microprogram kick) - GS/VU rendering; called via `QMem::call`.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};

pub const ZONE_POOL: u32 = 0x003e_1a40;
pub const ZONE_LIVE: u32 = 0x003e_1a4c;
/// DAT_002c0c00 (BSP plane epsilon)
pub const EPS: f32 = 0.01;
/// DAT_002c0c04 / DAT_002c0ac0
pub const STEP: f32 = 50.0;
/// DAT_002dd2fc: head of the safe-iteration chain (DList iterators registered while the body may unlink)
pub const ITER_CHAIN: u32 = 0x002d_d2fc;

/// local point `i` of a zone (+0xf4 array)
fn zpt<M: QMem + ?Sized>(m: &mut M, z: u32, i: u32) -> V4 {
    let b = m.r32(z + 0xf4);
    m.rv(b + i * 0x10)
}
/// *(V4 *)*(u32 *)p
fn deref_v<M: QMem + ?Sized>(m: &mut M, p: u32) -> V4 {
    let a = m.r32(p);
    m.rv(a)
}
fn copy32<M: QMem + ?Sized>(m: &mut M, dst: u32, src: u32) {
    let v = m.r32(src);
    m.w32(dst, v)
}

fn sub(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}
/// VU opmula/opmsub pair: a x b (xyz; w of `a` kept as the destination register's)
fn cross(a: V4, b: V4) -> V4 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0], a[3]]
}
fn dot3(a: V4, b: V4) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn dist_xy(a: V4, b: V4) -> f32 {
    let d = sub(a, b);
    (d[0] * d[0] + d[1] * d[1]).sqrt()
}
/// p0 * (1 - f) + p1 * f (mulax / maddx, xyzw)
fn lerp(p0: V4, p1: V4, f: f32) -> V4 {
    let g = 1.0 - f;
    [p0[0] * g + p1[0] * f, p0[1] * g + p1[1] * f, p0[2] * g + p1[2] * f, p0[3] * g + p1[3] * f]
}
/// mulax/madday/maddaz/maddw.xyz: rows 0..2 * p.xyz + row 3 (w of the row-0 register kept)
fn xform(m: &[V4; 4], p: V4) -> V4 {
    let mut r = [0.0; 4];
    for i in 0..3 {
        r[i] = m[0][i] * p[0] + m[1][i] * p[1] + m[2][i] * p[2] + m[3][i];
    }
    r[3] = m[0][3];
    r
}
/// (x, y, -1) query vector (prot3w / pextlw / pcpyld with 0xffffffff_bf800000)
fn q2(p: V4) -> V4 {
    [p[0], p[1], -1.0, f32::from_bits(0xffff_ffff)]
}

/// virtual slot 30 (+0xfc): world matrix
fn world_matrix<M: QMem + ?Sized>(m: &mut M, obj: u32) -> [V4; 4] {
    let p = vslot(m, obj, 30, &[], &[]).0;
    m.rm(p)
}

/// FUN_001ae318(&{size, align}) through an emulated-stack request block
fn frame_alloc<M: QMem + ?Sized>(m: &mut M, addr: u32, size: u32, align: u32) -> u32 {
    let req = stack_push(m, 8);
    m.w32(req, size);
    m.w32(req + 4, align);
    let p = m.call(addr, &[req], &[]).0;
    stack_pop(m, 8);
    p
}

/// FUN_001bd040: world bounds of the polygon (z unbounded); the transformed ring goes to a frame
/// temporary; then FUN_001c0d70(position, 0) -> FUN_001c1ba0(room, z) (add to the room's zone set).
pub fn ee_sub_1bd040<M: QMem + ?Sized>(m: &mut M, z: u32) {
    let mark = stack_push(m, 0x10);
    m.call(0x001a_e478, &[mark], &[]);
    let n = m.r32(z + 0xe8) as i32;
    let ring = frame_alloc(m, 0x001a_e318, (n * 0x10 - 1) as u32, 0x10);
    let mx = world_matrix(m, z);
    let pts = m.r32(z + 0xf4);
    let p = m.rv(pts);
    let w = xform(&mx, p);
    m.wv(z + 0x1a0, w);
    m.wv(z + 0x1b0, w);
    m.wv(ring, w);
    let first = m.rv(z + 0x1a0);
    m.wv(ring + ((n - 1) as u32) * 0x10, first);
    let mut i = 1;
    while i < n - 1 {
        let p = zpt(m, z, i as u32);
        let w = xform(&mx, p);
        let (lo, hi) = (m.rv(z + 0x1a0), m.rv(z + 0x1b0));
        let mut a = lo;
        let mut b = hi;
        for k in 0..3 {
            a[k] = lo[k].min(w[k]);
            b[k] = hi[k].max(w[k]);
        }
        m.wv(z + 0x1a0, a);
        m.wv(z + 0x1b0, b);
        m.wv(ring + i as u32 * 0x10, w);
        i += 1;
    }
    m.wf(z + 0x1a8, f32::MIN);
    m.wf(z + 0x1b8, f32::MAX);
    let pos = m.call(0x001a_62e8, &[z], &[]).1;
    let found = m.call(0x001c_0d70, &[0], &[pos]).0;
    if found != 0 {
        crate::engine::game::q_001c0320::ee_sub_1c1ba0(m, found, z);
    }
    m.call(0x001a_e4a0, &[mark, 2], &[]);
    stack_pop(m, 0x10);
}

/// FUN_001bd1e0: release the +0x1dc link through FUN_001c1968(link, z)
pub fn ee_sub_1bd1e0<M: QMem + ?Sized>(m: &mut M, z: u32) {
    let l = m.r32(z + 0x1dc);
    if l != 0 {
        crate::engine::game::q_001c0320::ee_sub_1c1968(m, l, z);
        m.w32(z + 0x1dc, 0);
    }
}

/// Safe DList walk (iterator registered on DAT_002dd2fc): unlink each child of `list`, clear its +0x1d8
/// parent, then `each(child)`.
fn drain_children<M: QMem + ?Sized>(m: &mut M, list: u32, mut each: impl FnMut(&mut M, u32)) {
    let it = stack_push(m, 0x10);
    let off = m.r32(list + 8);
    let mut c = m.r32(list);
    m.w32(it, list);
    m.w32(it + 4, c.wrapping_add(off));
    m.w32(it + 8, off);
    copy32(m, it + 0xc, ITER_CHAIN);
    m.w32(ITER_CHAIN, it);
    while c != 0 {
        m.call(0x0011_a2e8, &[list, c], &[]);
        m.w32(c + 0x1d8, 0);
        each(m, c);
        let nx = m.r32(it + 4);
        c = m.r32(nx);
        let off = m.r32(it + 8);
        m.w32(it + 4, c.wrapping_add(off));
    }
    let prev = m.r32(it + 0xc);
    m.w32(ITER_CHAIN, prev);
    stack_pop(m, 0x10);
}

/// FUN_001bd218: free a zone: children back to the live set (unless already in it), slot 8 (+0x4c),
/// then onto the free pool.
pub fn ee_sub_1bd218<M: QMem + ?Sized>(m: &mut M, z: u32) {
    drain_children(m, z + 0x1c4, |m, c| {
        if m.call(0x0011_a380, &[ZONE_LIVE, c], &[]).0 == 0 {
            m.call(0x0011_a220, &[ZONE_LIVE, c], &[]);
        }
    });
    vslot(m, z, 8, &[], &[]);
    m.call(0x0011_a220, &[ZONE_POOL, z], &[]);
}

/// FUN_001bd400: rebuild the edge planes (+0x1ec, one per edge) when the parent's +0xa4 stamp changed.
/// Normal = normalize((p[i] - p[i-1]) x (0,0,1)) through p[i-1] (InitPlaneFromNormalPoint).
pub fn ee_sub_1bd400<M: QMem + ?Sized>(m: &mut M, z: u32) {
    let parent = m.r32(z + 0x50);
    let stamp = if parent != 0 { m.r32(parent + 0xa4) } else { 0 };
    if m.r32(z + 0x1e8) == stamp {
        return;
    }
    let mx = world_matrix(m, z);
    let p = zpt(m, z, 0);
    let mut prev = xform(&mx, p);
    let mut i = 1;
    let mut off = 0;
    while i < m.r32(z + 0xe8) as i32 {
        let dst = m.r32(z + 0x1ec) + off;
        let cur = xform(&mx, zpt(m, z, i as u32));
        let c = cross(sub(cur, prev), [0.0, 0.0, 1.0, 0.0]);
        let q = 1.0 / dot3(c, c).sqrt();
        let n = [c[0] * q, c[1] * q, c[2] * q, c[3] * q];
        m.call(0x0011_9390, &[dst], &[n, prev]);
        prev = cur;
        i += 1;
        off += 0x10;
    }
    m.w32(z + 0x1e8, stamp);
}

/// FUN_001bd560: march along the path `path` from parameter `t` in steps of +-50 (dir != 0 forward)
/// until segment (prev, cur) hits `zone` (FUN_001bc688, up to 2 hits): writes the last hit point
/// (`out_pos`) and parameter (`out_t`) and returns -1; returns 3 when the path end is passed twice.
/// Path ends: FUN_001e6620 min, FUN_001e6648 max, FUN_001e66c8 wrap (when path +0xe4 loops).
pub fn ee_sub_1bd560<M: QMem + ?Sized>(m: &mut M, mut t: f32, zone: u32, path: u32, dir: u32, out_pos: u32, out_t: u32) -> u32 {
    let pos = stack_push(m, 0x10);
    let hits = stack_push(m, 0xc0);
    let ret;
    m.call(0x001e_6f10, &[path, pos, 0], &[[t, 0.0, 0.0, 0.0]]);
    let step = if dir == 0 { -STEP } else { STEP };
    let mut wrapped = false;
    let f = |m: &mut M, a: u32, t: Option<f32>| -> f32 {
        let v = t.map(|t| vec![[t, 0.0, 0.0, 0.0]]).unwrap_or_default();
        m.call(a, &[path], &v).1[0]
    };
    loop {
        t += step;
        let prev = m.rv(pos);
        if t <= f(m, 0x001e_6620, None) {
            if wrapped {
                ret = 3;
                break;
            }
            t = if m.r32(path + 0xe4) == 0 { f(m, 0x001e_6620, None) } else { f(m, 0x001e_66c8, Some(t)) };
            wrapped = true;
        } else if f(m, 0x001e_6648, None) <= t {
            if wrapped {
                ret = 3;
                break;
            }
            t = if m.r32(path + 0xe4) == 0 { f(m, 0x001e_6648, None) } else { f(m, 0x001e_66c8, Some(t)) };
            wrapped = true;
        }
        m.call(0x001e_6f10, &[path, pos, 0], &[[t, 0.0, 0.0, 0.0]]);
        let cur = m.rv(pos);
        let n = m.call(0x001b_c688, &[zone, 2, hits], &[prev, cur]).0 as i32;
        if n > 0 {
            if out_t != 0 {
                m.wf(out_t, t);
            }
            if out_pos != 0 {
                let h = m.rv(hits + (n as u32 - 1) * 0x60);
                m.wv(out_pos, h);
            }
            ret = u32::MAX;
            break;
        }
    }
    stack_pop(m, 0xc0);
    stack_pop(m, 0x10);
    ret
}

/// FUN_001bd8a8: does zone `z` overlap zone `other`? Any edge of `z` crossing `other` (FUN_001bc688,
/// 1 hit), else `other`'s first world point inside `z` (FUN_001bc9f0).
pub fn ee_sub_1bd8a8<M: QMem + ?Sized>(m: &mut M, z: u32, other: u32) -> bool {
    let mx = world_matrix(m, z);
    let mut prev = xform(&mx, zpt(m, z, 0));
    if m.r32(z + 0xe8) as i32 >= 2 {
        let hits = stack_push(m, 0x60);
        let mut i = 1;
        loop {
            let cur = xform(&mx, zpt(m, z, i as u32));
            let h = m.call(0x001b_c688, &[other, 1, hits], &[prev, cur]).0;
            prev = cur;
            if h != 0 {
                break;
            }
            i += 1;
            if m.r32(z + 0xe8) as i32 <= i {
                break;
            }
        }
        stack_pop(m, 0x60);
        if i < m.r32(z + 0xe8) as i32 {
            return true;
        }
    }
    let om = world_matrix(m, other);
    let p = xform(&om, zpt(m, other, 0));
    m.call(0x001b_c9f0, &[z], &[p]).0 != 0
}

/// FUN_001bda00: take a zone from the free pool (first call fills it with 16 class-0x2cd objects:
/// identity +0x60 matrix, +0xec 32 floats 0..31, +0xf0 / +0xf4 / +0x1ec frame buffers).
pub fn ee_sub_1bda00<M: QMem + ?Sized>(m: &mut M) -> u32 {
    if m.r32(ZONE_POOL) == 0 {
        for _ in 0..0x10 {
            let z = m.call(0x0013_e3c8, &[0xe, 0, 0x2cd, u32::MAX], &[]).0;
            m.w32(z + 0xe0, 0);
            m.w32(z + 0xe4, 1);
            m.w32(z + 0x1c0, 1);
            m.call(0x0023_33b4, &[z + 0x60, 0, 0x40], &[]);
            for o in [0x74, 0x9c, 0x88, 0x60] {
                m.wf(z + o, 1.0);
            }
            let p = frame_alloc(m, 0x001a_e1d0, 0x80, 0x10);
            m.w32(z + 0xec, p);
            for i in 0..0x20 {
                let b = m.r32(z + 0xec);
                m.wf(b + i * 4, i as f32);
            }
            let p = frame_alloc(m, 0x001a_e1d0, 0x80, 0x10);
            m.w32(z + 0xf0, p);
            let p = frame_alloc(m, 0x001a_e1d0, 0x200, 0x10);
            m.w32(z + 0xf4, p);
            let p = frame_alloc(m, 0x001a_e1d0, 0x1f0, 0x10);
            m.w32(z + 0x1ec, p);
            vslot(m, z, 8, &[], &[]);
            m.call(0x0011_a220, &[ZONE_POOL, z], &[]);
        }
    }
    let z = m.r32(ZONE_POOL);
    m.call(0x0011_a2e8, &[ZONE_POOL, z], &[]);
    m.w32(z + 0x1e8, u32::MAX);
    vslot(m, z, 7, &[], &[]);
    m.w32(z + 0x1d8, 0);
    z
}

/// z of (b - a) x (c - b) for hull[k-1], hull[k], hull[k+1]
fn turn<M: QMem + ?Sized>(m: &mut M, at: u32) -> f32 {
    let (a, b, c) = (m.rv(at - 0x10), m.rv(at), m.rv(at + 0x10));
    cross(sub(b, a), sub(c, b))[2]
}

/// FUN_001bdbf0: convex hull (monotone chain over `pts` sorted by FUN_002338c0 qsort with comparator
/// 0x1bdbc8) into the zone's points (max 32): right turns are popped.
pub fn ee_sub_1bdbf0<M: QMem + ?Sized>(m: &mut M, z: u32, n: i32, pts: u32) {
    m.call(0x0023_38c0, &[pts, n as u32, 0x10, 0x001b_dbc8], &[]);
    let mark = stack_push(m, 0x10);
    m.call(0x001a_e478, &[mark], &[]);
    let h = frame_alloc(m, 0x001a_e318, 0x800, 0x10);
    m.call(0x001a_e5a0, &[h, pts, 0x20], &[]);
    let at = |i: i32| (i * 0x10) as u32;
    let mut k = 2;
    let mut i = 2;
    while i < n {
        let p = m.rv(pts + at(i));
        m.wv(h + at(k), p);
        k += 1;
        i += 1;
        while k > 2 {
            let c = h + at(k - 2);
            if turn(m, c) >= 0.0 {
                break;
            }
            let nx = m.rv(c + 0x10);
            m.wv(c, nx);
            k -= 1;
        }
    }
    let p = m.rv(pts + at(n - 2));
    m.wv(h + at(k), p);
    let lower = k;
    let mut j = k + 1;
    let mut r = n - 3;
    loop {
        if r < 0 {
            let cnt = j.min(0x20);
            m.w32(z + 0xe8, cnt as u32);
            let dst = m.r32(z + 0xf4);
            m.call(0x001a_e5a0, &[dst, h, (cnt << 4) as u32], &[]);
            m.call(0x001a_e4a0, &[mark, 2], &[]);
            stack_pop(m, 0x10);
            return;
        }
        let p = m.rv(pts + at(r));
        m.wv(h + at(j), p);
        j += 1;
        r -= 1;
        if j <= lower {
            continue;
        }
        while lower < j {
            let c = h + at(j - 2);
            if turn(m, c) >= 0.0 {
                break;
            }
            let nx = m.rv(c + 0x10);
            m.wv(c, nx);
            j -= 1;
        }
    }
}

/// FUN_001bde18: attach `z` to `parent` (+0x1d8 / parent's +0x1c4 list), drop its link and leave the
/// live set.
pub fn ee_sub_1bde18<M: QMem + ?Sized>(m: &mut M, z: u32, parent: u32) {
    m.w32(z + 0x1d8, parent);
    m.call(0x0011_a220, &[parent + 0x1c4, z], &[]);
    ee_sub_1bd1e0(m, z);
    if m.call(0x0011_a380, &[ZONE_LIVE, z], &[]).0 != 0 {
        m.call(0x0011_a2e8, &[ZONE_LIVE, z], &[]);
    }
}

/// FUN_001bde80: merge `n` zones (`zones` = u32 array) into a new one: hull of all world points
/// (closing point of each ring dropped), recentred on the average (slot 28 (+0xec) gets the centre),
/// sources re-parented (merged zones hand over their children and are freed), +0xe4 = 1,
/// FUN_001e6180, +0x190 = FUN_001bc2f8.
pub fn ee_sub_1bde80<M: QMem + ?Sized>(m: &mut M, n: i32, zones: u32) {
    let mut total = 0i32;
    for i in 0..n.max(0) {
        let z = m.r32(zones + i as u32 * 4);
        total += m.r32(z + 0xe8) as i32 - 1;
    }
    let mark = stack_push(m, 0x10);
    m.call(0x001a_e478, &[mark], &[]);
    let buf = frame_alloc(m, 0x001a_e318, (total << 4) as u32, 0x10);
    let mut out = buf;
    for i in 0..n.max(0) {
        let z = m.r32(zones + i as u32 * 4);
        let mx = world_matrix(m, z);
        let mut k = 0;
        while k < m.r32(z + 0xe8) as i32 - 1 {
            let p = zpt(m, z, k as u32);
            m.wv(out, xform(&mx, p));
            out += 0x10;
            k += 1;
        }
    }
    let nz = ee_sub_1bda00(m);
    ee_sub_1bdbf0(m, nz, total, buf);
    let cnt = m.r32(nz + 0xe8) as i32;
    let mut c = [0.0f32; 4];
    let mut i = cnt - 2;
    while i >= 0 {
        let p = zpt(m, nz, i as u32);
        for k in 0..4 {
            c[k] += p[k];
        }
        i -= 1;
    }
    let q = 1.0 / (cnt - 1) as f32;
    let c = [c[0] * q, c[1] * q, c[2] * q, c[3] * q];
    vslot(m, nz, 28, &[], &[c]);
    let mut i = m.r32(nz + 0xe8) as i32 - 1;
    while i >= 0 {
        let a = m.r32(nz + 0xf4) + i as u32 * 0x10;
        let p = m.rv(a);
        m.wv(a, sub(p, c));
        i -= 1;
    }
    for i in 0..n.max(0) {
        let z = m.r32(zones + i as u32 * 4);
        if m.r32(z + 0x1c0) == 0 {
            ee_sub_1bde18(m, z, nz);
        } else {
            drain_children(m, z + 0x1c4, |m, c| ee_sub_1bde18(m, c, nz));
            vslot(m, z, 8, &[], &[]);
            m.call(0x0011_a220, &[ZONE_POOL, z], &[]);
        }
    }
    m.w32(nz + 0xe4, 1);
    m.call(0x001e_6180, &[nz], &[]);
    let r = m.call(0x001b_c2f8, &[nz], &[]).0;
    m.w32(nz + 0x190, r);
    m.call(0x001a_e4a0, &[mark, 2], &[]);
    stack_pop(m, 0x10);
}

/// FUN_001be240 / FUN_001befb8: this TU's copy of the shared pool static initialiser
pub fn ee_sub_1be240<M: QMem + ?Sized>(m: &mut M, init: u32, prio: u32) {
    crate::engine::game::q_0021c6c8::ee_sub_21c6c8(m, init, prio)
}

/// FUN_001be378 / FUN_001bf0f0
pub fn ee_sub_1be378<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1be240(m, 1, 0xffff)
}

/// FUN_001be990 / FUN_001be9e8: refresh projector 0x3e1a78 / 0x3e1aa0 (FUN_001be6f0) then FUN_0016e1a8
pub fn ee_sub_1be990<M: QMem + ?Sized>(m: &mut M, obj: u32, a: u32, b: u32) {
    // VFIX3 1:1 per asm: FUN_001be6f0(&DAT_003e1a78, obj); FUN_0016e1a8(obj, a, b)
    m.call(0x001b_e6f0, &[0x003e_1a78, obj], &[]);
    m.call(0x0016_e1a8, &[obj, a, b], &[]);
}

/// FUN_001be9e8: same as FUN_001be990 with projector DAT_003e1aa0
pub fn ee_sub_1be9e8<M: QMem + ?Sized>(m: &mut M, obj: u32, a: u32, b: u32) {
    m.call(0x001b_e6f0, &[0x003e_1aa0, obj], &[]);
    m.call(0x0016_e1a8, &[obj, a, b], &[]);
}

/// FUN_001bee10: projector `obj +0x128` (0x3e1a78 + 0x28k) -> table PTR_DAT_002c0bf8[k][i], else -1
pub fn ee_sub_1bee10<M: QMem + ?Sized>(m: &mut M, obj: u32, i: u32) -> u32 {
    let p = m.r32(obj + 0x128);
    if p == 0 {
        return u32::MAX;
    }
    // asm: mult by 0xcccccccd (low word) then sra 3 = exact /0x28 for element pointers
    let k = ((p.wrapping_sub(0x003e_1a78).wrapping_mul(0xcccc_cccd) as i32) >> 3) as u32;
    let t = m.r32(0x002c_0bf8u32.wrapping_add(k.wrapping_mul(4)));
    m.r32(t.wrapping_add(i.wrapping_mul(4)))
}

/// FUN_001beed8: movie-end event: 0x2ae / 0x2af go to the player (FUN_001c4840(0x3dd490); for 0x2ae
/// its +0xf40 when FUN_00132b10(player, 0x34)) as message 1, anything else as message 0x405.
pub fn ee_sub_1beed8<M: QMem + ?Sized>(m: &mut M, id: u32) {
    const MGR: u32 = 0x003d_d490;
    if id == 0x2ae {
        let mut p = m.call(0x001c_4840, &[MGR], &[]).0;
        if p != 0 && m.call(0x0013_2b10, &[p, 0x34], &[]).0 != 0 {
            p = m.r32(p + 0xf40);
        }
        m.call(0x0015_e938, &[MGR, 1, 0x2ae, p], &[]);
    } else if id == 0x2af {
        let p = m.call(0x001c_4840, &[MGR], &[]).0;
        m.call(0x0015_e938, &[MGR, 1, 0x2af, p], &[]);
    } else {
        m.call(0x0015_e938, &[MGR, 0x405, id, 0], &[]);
    }
}

/// leaf child -> triangle pointer when `mask` (0x4000, or 0x8000 with flag bit 0) is set
fn leaf<M: QMem + ?Sized>(m: &mut M, bsp: u32, c: u16, mask: u16) -> u32 {
    if c & mask != 0 {
        m.r32(bsp + 0xc) + (c & 0x3fff) as u32 * 0xc
    } else {
        0
    }
}

/// FUN_001bf110: BSP point location (dot > 0 -> front child +0xe)
pub fn ee_sub_1bf110<M: QMem + ?Sized>(m: &mut M, bsp: u32, p: V4, flags: u32) -> u32 {
    let q = q2(p);
    let nodes = m.r32(bsp + 0x14);
    let end = nodes + m.r32(bsp + 0x10) * 0x10;
    let mask = if flags & 1 != 0 { 0x8000 } else { 0x4000 };
    if nodes == end {
        return 0;
    }
    let mut nd = nodes;
    loop {
        let d = dot3(m.rv(nd), q);
        let c = m.r16(nd + 0xc + if d > 0.0 { 2 } else { 0 });
        if c & 0x8000 != 0 {
            return leaf(m, bsp, c, mask);
        }
        nd = nodes + c as u32 * 0x10;
        if nd == end {
            return 0;
        }
    }
}

/// FUN_001bf1f8: BSP point location with an EPS slab: points on a plane try the front subtree first.
pub fn ee_sub_1bf1f8<M: QMem + ?Sized>(m: &mut M, bsp: u32, mut nd: u32, p: V4, flags: u32) -> u32 {
    let q = q2(p);
    let nodes = m.r32(bsp + 0x14);
    let end = nodes + m.r32(bsp + 0x10) * 0x10;
    let mask = if flags & 1 != 0 { 0x8000 } else { 0x4000 };
    loop {
        if nd == end {
            return 0;
        }
        let d = dot3(m.rv(nd), q);
        let c;
        if -EPS <= d {
            let f = m.r16(nd + 0xe);
            if d <= EPS {
                if f & 0x8000 == 0 {
                    let sub = nodes + f as u32 * 0x10;
                    if sub != end {
                        let r = ee_sub_1bf1f8(m, bsp, sub, p, flags);
                        if r != 0 {
                            return r;
                        }
                    }
                } else if f & mask != 0 {
                    return leaf(m, bsp, f, mask);
                }
                c = m.r16(nd + 0xc);
            } else {
                c = f;
            }
        } else {
            c = m.r16(nd + 0xc);
        }
        if c & 0x8000 != 0 {
            return leaf(m, bsp, c, mask);
        }
        nd = nodes + c as u32 * 0x10;
    }
}

/// FUN_001bf398: plane side: 0 front (> EPS), 1 back (< -EPS), 2 on
pub fn ee_sub_1bf398(d: f32) -> u32 {
    if d <= EPS {
        if -EPS <= d {
            2
        } else {
            1
        }
    } else {
        0
    }
}

/// FUN_001bf3d8: segment p0 -> p1 through the BSP; writes up to `max` 0x60-byte spans to `out`
/// ({start, end, plane in, plane out, t0 @0x40, t1 @0x44, 3 @0x48, leaf @0x4c}); adjacent spans of the
/// same leaf (or any leaf with flag bit 1) are merged. Returns the span count (`max` when full).
pub fn ee_sub_1bf3d8<M: QMem + ?Sized>(m: &mut M, bsp: u32, p0: V4, p1: V4, max: i32, mut out: u32, flags: u32) -> i32 {
    let leaves = m.r32(bsp + 0xc);
    let nodes = m.r32(bsp + 0x14);
    let end = nodes + m.r32(bsp + 0x10) * 0x10;
    let mask: u16 = if flags & 1 != 0 { 0x8000 } else { 0x4000 };
    let (q0, q1) = (q2(p0), q2(p1));
    // traversal stack: entry 0 = {t = 1.0}; each entry {node, leaf, t, -, plane}
    let base = stack_push(m, 0x400);
    let mut top = base;
    for i in 0..8 {
        m.w32(base + i * 4, 0);
    }
    m.wf(base + 8, 1.0);
    let (mut node, mut lf, mut t0, mut plane) = (nodes, 0u32, 0.0f32, [0.0f32; 4]);
    let mut last = 0u32;
    let mut count = 0;
    let child = |c: u16, node: &mut u32, lf: &mut u32| {
        if c & 0x8000 == 0 {
            *node = nodes + c as u32 * 0x10;
        } else {
            *lf = if c & mask != 0 { leaves + (c & 0x3fff) as u32 * 0xc } else { 0 };
            *node = end;
        }
    };
    let ret = 'run: loop {
        while node == end {
            if lf != 0 {
                let t1 = m.rf(top + 8);
                let merge = last != 0 && t0 == m.rf(last + 0x44) && (flags & 2 != 0 || lf == m.r32(last + 0x4c));
                if merge {
                    m.wv(last + 0x10, lerp(p0, p1, t1));
                    m.wf(last + 0x44, t1);
                    let pl = m.rv(top + 0x10);
                    m.wv(last + 0x30, pl);
                } else {
                    if max <= count {
                        break 'run max;
                    }
                    m.call(0x0023_33b4, &[out, 0, 0x60], &[]);
                    count += 1;
                    m.wv(out, lerp(p0, p1, t0));
                    m.wv(out + 0x10, lerp(p0, p1, t1));
                    m.w32(out + 0x48, 3);
                    m.wf(out + 0x40, t0);
                    m.wf(out + 0x44, t1);
                    m.wv(out + 0x20, plane);
                    let pl = m.rv(top + 0x10);
                    m.wv(out + 0x30, pl);
                    m.w32(out + 0x4c, lf);
                    last = out;
                    out += 0x60;
                }
            }
            if top <= base {
                break 'run count;
            }
            node = m.r32(top);
            lf = m.r32(top + 4);
            t0 = m.rf(top + 8);
            plane = m.rv(top + 0x10);
            top -= 0x20;
        }
        let n = m.rv(node);
        let d0 = dot3(n, q0);
        let d1 = dot3(n, q1);
        let (s0, s1) = (ee_sub_1bf398(d0), ee_sub_1bf398(d1));
        let back = s0 == 1 || s1 == 1;
        let front = s0 == 0 || s1 == 0;
        if back && front {
            let t_end = m.rf(top + 8);
            let tc = d0 / (d0 - d1);
            let dm = t0 * d1 + (1.0 - t0) * d0;
            if t0 < tc && tc < t_end {
                let e = top + 0x20;
                m.wf(e + 8, tc);
                let pl = [n[0], n[1], 0.0, n[2]];
                m.wv(e + 0x10, pl);
                let c = m.r16(node + 0xc + if dm <= pl[2] { 2 } else { 0 });
                if c & 0x8000 == 0 {
                    m.w32(e + 4, 0);
                    m.w32(e, nodes + c as u32 * 0x10);
                } else {
                    m.w32(e, end);
                    let l = if c & mask != 0 { (c & 0x3fff) as u32 * 0xc + leaves } else { 0 };
                    m.w32(e + 4, l);
                }
                top = e;
            }
            let near_back = dm < 0.0 || (dm == 0.0 && t_end * d1 + (1.0 - t_end) * d0 <= 0.0);
            let c = m.r16(node + 0xc + if near_back { 0 } else { 2 });
            child(c, &mut node, &mut lf);
            continue;
        }
        let c = m.r16(node + 0xc + if front { 2 } else { 0 });
        child(c, &mut node, &mut lf);
    };
    stack_pop(m, 0x400);
    ret
}

/// FUN_001bf900: is the segment p0 -> p1 NOT a single clear span (blocked / leaves the mesh)?
pub fn ee_sub_1bf900<M: QMem + ?Sized>(m: &mut M, bsp: u32, p0: V4, p1: V4, flags: u32) -> u32 {
    let buf = stack_push(m, 0xc0);
    let n = ee_sub_1bf3d8(m, bsp, p0, p1, 2, buf, flags | 2);
    let r = (n == 0 || m.rf(buf + 0x40) != 0.0 || m.rf(buf + 0x44) != 1.0) as u32;
    stack_pop(m, 0xc0);
    r
}

/// FUN_001bf970: graph node index ((node - mesh +4) >> 5)
pub fn ee_sub_1bf970<M: QMem + ?Sized>(m: &mut M, mesh: u32, node: u32) -> i32 {
    (node.wrapping_sub(m.r32(mesh + 4)) as i32) >> 5
}

/// FUN_001bf980: A* from node `start` to node `goal` over the navmesh vertex graph. Open list of
/// 0x18-byte records {node, f, g, parent, depth, next} sorted by f (heuristic = 2D distance from the
/// *expanded* node to the goal, as in the original); visited flags per node (FUN_001ae378, zeroed).
/// Writes the node pointers start..goal to `out` and their count to `*out_count` (0 when unreachable).
pub fn ee_sub_1bf980<M: QMem + ?Sized>(m: &mut M, mesh: u32, start: u32, goal: u32, _max: u32, out_count: u32, out: u32) {
    let mark = stack_push(m, 0x10);
    m.call(0x001a_e478, &[mark], &[]);
    let nn = m.r32(mesh);
    let visited = frame_alloc(m, 0x001a_e378, nn << 2, 0x10);
    m.w32(out, start);
    let first = stack_push(m, 0x18);
    m.w32(first, start);
    let gp = m.rv(goal);
    let sp = m.rv(start);
    m.wf(first + 4, dist_xy(sp, gp));
    m.wf(first + 8, 0.0);
    m.w32(first + 0xc, 0);
    m.w32(first + 0x10, 1);
    let mut open = 0u32;
    let mut cur = first;
    let mut found = start == goal;
    while !found {
        let nd = m.r32(cur);
        let ne = m.r32(nd + 0x10) as i32;
        for i in 0..ne.max(0) {
            let e = m.r32(nd + 0x14) + i as u32 * 8;
            let to = m.r32(e);
            let add = if to == goal {
                true
            } else {
                let v = visited + (ee_sub_1bf970(m, mesh, to) as u32) * 4;
                if m.r32(v) == 0 {
                    m.w32(v, 1);
                    true
                } else {
                    false
                }
            };
            if !add {
                continue;
            }
            let r = frame_alloc(m, 0x001a_e318, 0x18, 0x10);
            m.w32(r, to);
            let g = m.rf(cur + 8) + m.rf(e + 4);
            m.wf(r + 8, g);
            m.w32(r + 0xc, cur);
            let np = m.rv(nd);
            let f = g + dist_xy(np, gp);
            m.wf(r + 4, f);
            let depth = m.r32(cur + 0x10) + 1;
            m.w32(r + 0x10, depth);
            // insert before the first record with a greater f
            let mut link = 0u32; // 0 = head
            let mut it = open;
            while it != 0 && m.rf(it + 4) <= f {
                link = it + 0x14;
                it = m.r32(it + 0x14);
            }
            m.w32(r + 0x14, it);
            if link == 0 {
                open = r;
            } else {
                m.w32(link, r);
            }
        }
        if open == 0 {
            break;
        }
        cur = open;
        open = m.r32(open + 0x14);
        found = m.r32(cur) == goal;
    }
    if found {
        let n = m.r32(cur + 0x10);
        m.w32(out_count, n);
        let mut e = cur;
        while e != 0 {
            let d = m.r32(e + 0x10);
            let nd = m.r32(e);
            m.w32(out + (d - 1) * 4, nd);
            e = m.r32(e + 0xc);
        }
    } else {
        m.w32(out_count, 0);
    }
    stack_pop(m, 0x18);
    m.call(0x001a_e4a0, &[mark, 2], &[]);
    stack_pop(m, 0x10);
}

/// FUN_001bfc18: string pulling: keep a node only when the line from the last kept node to the next
/// one is blocked (FUN_001bf900); repeats while it removes nodes and more than 2 remain.
pub fn ee_sub_1bfc18<M: QMem + ?Sized>(m: &mut M, mesh: u32, count: u32, arr: u32) {
    let mark = stack_push(m, 0x10);
    m.call(0x001a_e478, &[mark], &[]);
    let n = m.r32(count);
    let keep = frame_alloc(m, 0x001a_e318, n * 4, 0x10);
    copy32(m, keep, arr);
    let k = stack_push(m, 4);
    m.w32(k, 1);
    let endp = arr + n * 4;
    let (mut kept, mut cand, mut next) = (arr, arr + 4, arr + 8);
    while next < endp {
        let a = deref_v(m, kept);
        let b = deref_v(m, next);
        if ee_sub_1bf900(m, mesh, a, b, 0) != 0 {
            let kk = m.r32(k);
            copy32(m, keep + kk * 4, cand);
            m.w32(k, kk + 1);
            kept = cand;
        }
        cand = next;
        next += 4;
    }
    let kk = m.r32(k);
    copy32(m, keep + kk * 4, arr + (n - 1) * 4);
    m.w32(k, kk + 1);
    let kk = kk + 1;
    if kk < n && kk > 2 {
        ee_sub_1bfc18(m, mesh, k, keep);
    }
    let kk = m.r32(k);
    m.call(0x001a_e5a0, &[arr, keep, kk << 2], &[]);
    m.w32(count, kk);
    stack_pop(m, 4);
    m.call(0x001a_e4a0, &[mark, 2], &[]);
    stack_pop(m, 0x10);
}

/// FUN_001bfdb0: closest point of triangle `tri` (3 node ptrs) to `p` in 2D: outside an edge ->
/// FUN_0011ee08 (closest point on that segment), inside -> p moved along z onto the triangle plane.
pub fn ee_sub_1bfdb0<M: QMem + ?Sized>(m: &mut M, tri: u32, p: V4) -> V4 {
    let a = deref_v(m, tri);
    let b = deref_v(m, tri + 4);
    let c = deref_v(m, tri + 8);
    let ab = sub(b, a);
    let edge = if cross(sub(p, a), ab)[2] > 0.0 {
        Some((a, b))
    } else if cross(sub(p, b), sub(c, b))[2] > 0.0 {
        Some((b, c))
    } else if cross(sub(p, c), sub(a, c))[2] > 0.0 {
        Some((c, a))
    } else {
        None
    };
    match edge {
        Some((s, e)) => {
            let o = stack_push(m, 0x10);
            m.call(0x0011_ee08, &[o, 0, 0], &[p, s, e]);
            let r = m.rv(o);
            stack_pop(m, 0x10);
            r
        }
        None => {
            let n = cross(ab, sub(c, a));
            let t = dot3(sub(a, p), n) / n[2];
            [p[0], p[1], p[2] + t, p[3]]
        }
    }
}

/// FUN_001bff28: navmesh path p0 -> p1 into `out` (V4s, at most `max` from the graph search): same
/// triangle -> {p0, q0, q1}; else A* between temporary start / goal nodes linked to the triangles'
/// corners, string-pulled; points closer than 50 (2D) to the previous kept one are dropped; a single
/// point becomes {p + (50,0,0), p}. Returns the point count (0 when an end is off the mesh).
pub fn ee_sub_1bff28<M: QMem + ?Sized>(m: &mut M, mesh: u32, p0: V4, p1: V4, max: i32, out: u32) -> i32 {
    let root = m.r32(mesh + 0x14);
    let t0 = ee_sub_1bf1f8(m, mesh, root, p0, 1);
    if t0 == 0 {
        return 0;
    }
    let q0 = ee_sub_1bfdb0(m, t0, p0);
    let root = m.r32(mesh + 0x14);
    let t1 = ee_sub_1bf1f8(m, mesh, root, p1, 1);
    if t1 == 0 {
        return 0;
    }
    let q1 = ee_sub_1bfdb0(m, t1, p1);
    let n;
    if t0 == t1 {
        m.wv(out, p0);
        m.wv(out + 0x10, q0);
        m.wv(out + 0x20, q1);
        n = 3;
    } else {
        let mark = stack_push(m, 0x10);
        m.call(0x001a_e478, &[mark], &[]);
        let arr = frame_alloc(m, 0x001a_e318, 0x200, 0x10);
        let s = stack_push(m, 0x20);
        let g = stack_push(m, 0x20);
        let cnt = stack_push(m, 4);
        m.wv(s, q0);
        m.w32(s + 0x10, 3);
        let edges = frame_alloc(m, 0x001a_e318, 0x18, 0x10);
        m.w32(s + 0x14, edges);
        for i in 0..3 {
            let v = m.r32(t0 + i * 4);
            m.w32(edges + i * 8, v);
            let vp = m.rv(v);
            m.wf(edges + i * 8 + 4, dist_xy(q0, vp));
        }
        m.wv(g, q1);
        m.w32(g + 0x10, 0);
        m.w32(g + 0x14, 0);
        for i in 0..3 {
            let v = m.r32(t1 + i * 4);
            let k = m.r32(v + 0x10);
            let e = m.r32(v + 0x14) + k * 8;
            m.w32(v + 0x10, k + 1);
            m.w32(e, g);
            let vp = m.rv(v);
            m.wf(e + 4, dist_xy(vp, q1));
        }
        ee_sub_1bf980(m, mesh, s, g, 0x80, cnt, arr);
        if m.r32(cnt) as i32 > 2 {
            ee_sub_1bfc18(m, mesh, cnt, arr);
        }
        let k = (m.r32(cnt) as i32).min(max);
        for i in 0..k.max(0) as u32 {
            let p = deref_v(m, arr + i * 4);
            m.wv(out + i * 0x10, p);
        }
        for i in 0..3 {
            let v = m.r32(t1 + i * 4);
            let c = m.r32(v + 0x10);
            m.w32(v + 0x10, c.wrapping_sub(1));
        }
        stack_pop(m, 4);
        stack_pop(m, 0x20);
        stack_pop(m, 0x20);
        m.call(0x001a_e4a0, &[mark, 2], &[]);
        stack_pop(m, 0x10);
        n = k;
    }
    let mut last = m.rv(out);
    let mut w = out + 0x10;
    let mut i = 1;
    while i < n {
        let p = m.rv(out + i as u32 * 0x10);
        if STEP < dist_xy(p, last) {
            last = p;
            m.wv(w, p);
            w += 0x10;
        }
        i += 1;
    }
    let k = ((w - out) >> 4) as i32;
    if k != 1 {
        return k;
    }
    let p = m.rv(out);
    m.wv(out + 0x10, p);
    m.wv(out, [p[0] + STEP, p[1], p[2], p[3]]);
    2
}
