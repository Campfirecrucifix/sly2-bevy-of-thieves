//! WP-Q2: run 0x001c0320..0x001c3370 - the navmesh object wrappers (mesh at obj +0xe0: +0 node count,
//! +4 nodes (stride 0x20: pos, +0x10 edge count, +0x14 edges {node, cost}), +8 tri count, +0xc tris,
//! +0x10 BSP node count, +0x14 BSP nodes), the default 2-triangle mesh, the zone sweep-and-prune
//! (obj +0xf8 zones, +0xfc capacity, +0x100 endpoint count, +0x104 / +0x108 sorted x / y endpoint lists
//! {f32 v, u16 is_min @4, u16 slot @6}, +0x10c slot table, +0x114 stamp from DAT_003e1200; query cache
//! 0x40 bytes: x bits, y bits, +0x20 stamp, +0x24.. cursors), the room / portal graph (+0x10 rooms,
//! +0x18 room objects, +0x1c per-room portal lists, +0x20 / +0x24 portals) with its A* search, the
//! scratchpad buffer layout switch and the thrown-object helpers.
//! 1:1 against `QMem`; out-of-run callees go through `QMem::call` by address.
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, vslot, QMem, V4};
use crate::engine::game::q_001bd040 as nav;

/// DAT_003e1200: global sweep-and-prune stamp
pub const STAMP: u32 = 0x003e_1200;
/// DAT_002a76dc / DAT_002a76e4 (FLT_MAX), DAT_002a76e0 (-FLT_MAX)
pub const BIG: f32 = f32::MAX;
/// DAT_002c0c10: the default mesh nodes {x, y, edge count, edges[3]} (z = w = 0)
pub const DEFAULT_NODES: [(f32, f32, u32, [i32; 3]); 4] = [
    (-100000.0, -100000.0, 3, [1, 2, 3]),
    (100000.0, -100000.0, 2, [0, 2, -1]),
    (100000.0, 100000.0, 3, [0, 1, 3]),
    (-100000.0, 100000.0, 2, [0, 2, -1]),
];
/// DAT_002c0c90: the default mesh triangles
pub const DEFAULT_TRIS: [[u32; 3]; 2] = [[0, 1, 2], [0, 2, 3]];

fn sub(a: V4, b: V4) -> V4 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
}
fn len3(a: V4) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}
fn dist_xy(a: V4, b: V4) -> f32 {
    let d = sub(a, b);
    (d[0] * d[0] + d[1] * d[1]).sqrt()
}
fn f(x: f32) -> V4 {
    [x, 0.0, 0.0, 0.0]
}
fn stamp_bump<M: QMem + ?Sized>(m: &mut M, obj: u32) {
    let s = m.r32(STAMP).wrapping_add(1);
    m.w32(STAMP, s);
    m.w32(obj + 0x114, s);
}
/// allocator `addr` with an emulated-stack {size, align} request
fn alloc<M: QMem + ?Sized>(m: &mut M, addr: u32, size: u32, extra: &[u32]) -> u32 {
    let req = stack_push(m, 8);
    m.w32(req, size);
    m.w32(req + 4, 0x10);
    let mut a = vec![req];
    a.extend_from_slice(extra);
    let p = m.call(addr, &a, &[]).0;
    stack_pop(m, 8);
    p
}

/// FUN_001c0320: snap `p` onto the mesh: BSP locate (with slab) then closest triangle point -> `out`
pub fn ee_sub_1c0320<M: QMem + ?Sized>(m: &mut M, mesh: u32, p: V4, out: u32, flags: u32) -> bool {
    let root = m.r32(mesh + 0x14);
    let t = nav::ee_sub_1bf1f8(m, mesh, root, p, flags);
    if t != 0 {
        let q = nav::ee_sub_1bfdb0(m, t, p);
        m.wv(out, q);
    }
    t != 0
}

/// FUN_001c0648: build the default 4-node / 2-triangle mesh (one BSP plane along the diagonal) at
/// obj +0xe0, edge costs by FUN_001c0880, new stamp.
pub fn ee_sub_1c0648<M: QMem + ?Sized>(m: &mut M, obj: u32) {
    m.w32(obj + 0xe0, 4);
    let nodes = alloc(m, 0x001a_e2a0, 0x80, &[]);
    m.w32(obj + 0xe4, nodes);
    for (i, &(x, y, n, e)) in DEFAULT_NODES.iter().enumerate() {
        let nd = m.r32(obj + 0xe4) + i as u32 * 0x20;
        m.wv(nd, [x, y, 0.0, 0.0]);
        m.w32(nd + 0x10, n);
        let edges = alloc(m, 0x001a_e1d0, n * 8 + 8, &[]);
        m.w32(nd + 0x14, edges);
        for k in 0..n {
            let to = m.r32(obj + 0xe4).wrapping_add((e[k as usize] as u32).wrapping_mul(0x20));
            let _t0 = m.r32(nd + 0x14);
            m.w32(_t0 + k * 8, to);
        }
    }
    m.w32(obj + 0xe8, 2);
    let tris = alloc(m, 0x001a_e2a0, 0x18, &[]);
    m.w32(obj + 0xec, tris);
    for (i, t) in DEFAULT_TRIS.iter().enumerate() {
        for (k, &v) in t.iter().enumerate() {
            let a = m.r32(obj + 0xe4) + v * 0x20;
            let _t0 = m.r32(obj + 0xec);
            m.w32(_t0 + i as u32 * 0xc + k as u32 * 4, a);
        }
    }
    m.w32(obj + 0xf0, 1);
    let bsp = alloc(m, 0x001a_e2a0, 0x10, &[]);
    m.w32(obj + 0xf4, bsp);
    m.w16(bsp + 0xe, 0xc000);
    m.w16(bsp + 0xc, 0xc001);
    m.w32(bsp + 8, 0);
    m.w32(bsp, 0x3f35_04f3);
    m.w32(bsp + 4, 0xbf35_04f3);
    ee_sub_1c0880(m, obj + 0xe0);
    stamp_bump(m, obj);
}

/// FUN_001c0880: edge cost = 2D distance between the node and the edge's target
pub fn ee_sub_1c0880<M: QMem + ?Sized>(m: &mut M, mesh: u32) {
    for i in 0..(m.r32(mesh) as i32).max(0) as u32 {
        let nd = m.r32(mesh + 4) + i * 0x20;
        for k in 0..(m.r32(nd + 0x10) as i32).max(0) as u32 {
            let e = m.r32(nd + 0x14) + k * 8;
            let _t0 = m.r32(e);
            let (a, b) = (m.rv(nd), m.rv(_t0));
            m.wf(e + 4, dist_xy(a, b));
        }
    }
}

/// FUN_001c0920: is `p` on the object's mesh?
pub fn ee_sub_1c0920<M: QMem + ?Sized>(m: &mut M, obj: u32, p: V4, flags: u32) -> bool {
    nav::ee_sub_1bf110(m, obj + 0xe0, p, flags) != 0
}

/// FUN_001c0940
pub fn ee_sub_1c0940<M: QMem + ?Sized>(m: &mut M, obj: u32, p: V4, out: u32, flags: u32) -> bool {
    ee_sub_1c0320(m, obj + 0xe0, p, out, flags)
}

/// FUN_001c0960
pub fn ee_sub_1c0960<M: QMem + ?Sized>(m: &mut M, obj: u32, p0: V4, p1: V4, max: i32, out: u32, flags: u32) -> i32 {
    nav::ee_sub_1bf3d8(m, obj + 0xe0, p0, p1, max, out, flags)
}

/// FUN_001c0980
pub fn ee_sub_1c0980<M: QMem + ?Sized>(m: &mut M, obj: u32, p0: V4, p1: V4, flags: u32) -> u32 {
    nav::ee_sub_1bf900(m, obj + 0xe0, p0, p1, flags)
}

/// FUN_001c0b50: nearest walkable point to `p`: off-mesh -> 0. On-mesh point q; if `p` is inside a zone
/// (FUN_001bc9f0 over the zones around q), the closest mesh-clear point of that zone's outline (edges
/// at q.z, FUN_001c0960 1 span, FUN_0011ee08 closest point / distance) is snapped instead. Returns 1.
pub fn ee_sub_1c0b50<M: QMem + ?Sized>(m: &mut M, obj: u32, p: V4, out: u32) -> u32 {
    let t = nav::ee_sub_1bf110(m, obj + 0xe0, p, 1);
    if t == 0 {
        return 0;
    }
    let q = nav::ee_sub_1bfdb0(m, t, p);
    let buf = stack_push(m, 0x210);
    let n = ee_sub_1c1700(m, obj, 0, q, q, 0x80, buf);
    let mut zone = 0;
    for i in 0..n.max(0) as u32 {
        let z = m.r32(buf + i * 4);
        if m.call(0x001b_c9f0, &[z], &[p]).0 != 0 {
            zone = z;
            break;
        }
    }
    stack_pop(m, 0x210);
    if zone == 0 {
        m.wv(out, q);
        return 1;
    }
    let mut best = BIG;
    let mut bestp = [0.0f32; 4];
    let world = |m: &mut M, i: u32| {
        let _t0 = m.r32(zone + 0xf4);
        let l = m.rv(_t0 + i * 0x10);
        let w = m.call(0x001a_6f00, &[zone, 0], &[l]).1;
        [w[0], w[1], q[2], w[3]]
    };
    let mut s = world(m, 0);
    let spans = stack_push(m, 0x60);
    let tmp = stack_push(m, 0x20);
    let mut i = 1;
    while i < m.r32(zone + 0xe8) as i32 {
        let e = world(m, i as u32);
        let k = ee_sub_1c0960(m, obj, s, e, 1, spans, 2);
        for j in 0..k.max(0) as u32 {
            let sp = spans + j * 0x60;
            let (a, b) = (m.rv(sp), m.rv(sp + 0x10));
            m.call(0x0011_ee08, &[tmp, 0, tmp + 0x10], &[q, a, b]);
            let d = m.rf(tmp + 0x10);
            if d < best {
                bestp = m.rv(tmp);
                best = d;
            }
        }
        s = e;
        i += 1;
    }
    stack_pop(m, 0x20);
    stack_pop(m, 0x60);
    ee_sub_1c0940(m, obj, bestp, out, 0);
    1
}

/// FUN_001c10c8: lazily create the world's navmesh object (class 0x2ca, +0x45c8) with the default mesh
/// and its room graph (+0x45cc, 0x30 bytes), then (re)collect rooms / portals (FUN_001c1138).
pub fn ee_sub_1c10c8<M: QMem + ?Sized>(m: &mut M, world: u32) {
    if m.r32(world + 0x45cc) == 0 {
        let o = m.call(0x0013_e3c8, &[0x11, 0, 0x2ca, u32::MAX], &[]).0;
        m.w32(world + 0x45c8, o);
        ee_sub_1c0648(m, o);
        let g = alloc(m, 0x001a_e2a0, 0x30, &[]);
        m.w32(world + 0x45cc, g);
    }
    let g = m.r32(world + 0x45cc);
    ee_sub_1c1138(m, g);
}

/// FUN_001c1138: rooms (FUN_0015e938(0x3dd490, 5, id, 0), +0x124 = index) and portals (class 0x7c
/// found by FUN_0015e9c8, at most 0x400, copied by FUN_001ae238; +0x150 = index); +0x2c = 1.
pub fn ee_sub_1c1138<M: QMem + ?Sized>(m: &mut M, g: u32) {
    let mut i = 0;
    while i < m.r32(g + 0x10) as i32 {
        let _t0 = m.r32(g + 0x14);
        let id = m.r32(_t0 + i as u32 * 4);
        let r = m.call(0x0015_e938, &[0x003d_d490, 5, id, 0], &[]).0;
        m.w32(r + 0x124, i as u32);
        let _t0 = m.r32(g + 0x18);
        m.w32(_t0 + i as u32 * 4, r);
        i += 1;
    }
    let buf = stack_push(m, 0x1000);
    let found = m.call(0x0015_e9c8, &[0x003d_d490, 0x205, 0x7c, 0, 0x400, buf], &[]).0 as i32;
    let n = found.min(0x400);
    m.w32(g + 0x20, n as u32);
    let p = alloc(m, 0x001a_e238, (n as u32) << 2, &[buf]);
    stack_pop(m, 0x1000);
    m.w32(g + 0x24, p);
    for i in 0..n.max(0) as u32 {
        let _t0 = m.r32(g + 0x24);
        let portal = m.r32(_t0 + i * 4);
        m.w32(portal + 0x150, i);
    }
    m.w32(g + 0x2c, 1);
}

/// FUN_001c1260: per-room outgoing portal lists (+0x1c: {count, indices}) from portals with both rooms.
pub fn ee_sub_1c1260<M: QMem + ?Sized>(m: &mut M, g: u32) {
    let mark = stack_push(m, 0x10);
    m.call(0x001a_e478, &[mark], &[]);
    let nr = m.r32(g + 0x10);
    let counts = alloc(m, 0x001a_e378, nr << 2, &[]);
    let rooms_of = |m: &mut M, i: u32| {
        let _t0 = m.r32(g + 0x24);
        let p = m.r32(_t0 + i * 4);
        let a = m.r32(p + 0x124);
        if a != 0 && m.r32(p + 0x128) != 0 {
            Some(m.r32(a + 0x124))
        } else {
            None
        }
    };
    for i in 0..(m.r32(g + 0x20) as i32).max(0) as u32 {
        if let Some(r) = rooms_of(m, i) {
            let c = m.r32(counts + r * 4);
            m.w32(counts + r * 4, c + 1);
        }
    }
    let nr = m.r32(g + 0x10);
    let lists = alloc(m, 0x001a_e1d0, nr << 3, &[]);
    m.w32(g + 0x1c, lists);
    for i in 0..(m.r32(g + 0x10) as i32).max(0) as u32 {
        let l = m.r32(g + 0x1c) + i * 8;
        m.w32(l, 0);
        let c = m.r32(counts + i * 4);
        let a = alloc(m, 0x001a_e1d0, c << 2, &[]);
        m.w32(l + 4, a);
    }
    for i in 0..(m.r32(g + 0x20) as i32).max(0) as u32 {
        if let Some(r) = rooms_of(m, i) {
            let l = m.r32(g + 0x1c) + r * 8;
            let c = m.r32(l);
            let _t0 = m.r32(l + 4);
            m.w32(_t0 + c * 4, i);
            m.w32(l, c + 1);
        }
    }
    m.call(0x001a_e4a0, &[mark, 2], &[]);
    stack_pop(m, 0x10);
}

/// FUN_001c1440: reset a query cache (bits clear, stamp -1, all cursors at endpoint count - 2)
pub fn ee_sub_1c1440<M: QMem + ?Sized>(m: &mut M, obj: u32, cache: u32) {
    m.call(0x0023_33b4, &[cache, 0, 0x40], &[]);
    let c = m.r32(obj + 0x100).wrapping_sub(2);
    m.w32(cache + 0x24, c);
    m.w32(cache + 0x28, c);
    m.w32(cache + 0x20, u32::MAX);
    m.w32(cache + 0x2c, c);
    m.w32(cache + 0x30, c);
}

fn toggle<M: QMem + ?Sized>(m: &mut M, bits: u32, slot: u16, set: bool) {
    let w = bits + (slot >> 5) as u32 * 4;
    let b = 1u32 << (slot & 0x1f);
    let v = m.r32(w);
    m.w32(w, if set { v | b } else { v.wrapping_sub(b) });
}

/// FUN_001c14a8: move cursor `cur` of the sorted endpoint list to `v`, toggling the slot bits of the
/// crossed endpoints whose is_min == `kind` (down: kind 0 sets / 1 clears; up: kind 0 clears / 1 sets).
pub fn ee_sub_1c14a8<M: QMem + ?Sized>(m: &mut M, v: f32, list: u32, mut cur: i32, bits: u32, kind: u16) -> i32 {
    let at = |i: i32| list.wrapping_add((i * 8) as u32);
    if v <= m.rf(at(cur)) {
        loop {
            let e = at(cur);
            if m.r16(e + 4) == kind {
                let s = m.r16(e + 6);
                toggle(m, bits, s, kind == 0);
            }
            cur -= 1;
            if !(v <= m.rf(at(cur))) {
                return cur;
            }
        }
    }
    let mut i = cur + 1;
    while m.rf(at(i)) <= v {
        let e = at(i);
        if m.r16(e + 4) == kind {
            let s = m.r16(e + 6);
            toggle(m, bits, s, kind != 0);
        }
        cur = i;
        i += 1;
    }
    cur
}

/// FUN_001c15f8: move the (lo, hi) cursors (order chosen so the interval never inverts)
pub fn ee_sub_1c15f8<M: QMem + ?Sized>(m: &mut M, lo: f32, hi: f32, list: u32, plo: u32, phi: u32, bits: u32) {
    let (mut a, mut b) = (m.r32(plo) as i32, m.r32(phi) as i32);
    if m.rf(list.wrapping_add((a * 8) as u32)) < lo {
        b = ee_sub_1c14a8(m, hi, list, b, bits, 1);
        a = ee_sub_1c14a8(m, lo, list, a, bits, 0);
    } else {
        a = ee_sub_1c14a8(m, lo, list, a, bits, 0);
        b = ee_sub_1c14a8(m, hi, list, b, bits, 1);
    }
    m.w32(plo, a as u32);
    m.w32(phi, b as u32);
}

/// FUN_001c1700: zones whose x / y extents overlap the box (p0, p1) -> `out` (stops once more than
/// `max` were written); `cache` (0 = a temporary) keeps the cursors between calls with the same stamp.
pub fn ee_sub_1c1700<M: QMem + ?Sized>(m: &mut M, obj: u32, cache: u32, p0: V4, p1: V4, max: i32, out: u32) -> i32 {
    if m.r32(obj + 0x100) == 0 {
        return 0;
    }
    let tmp = stack_push(m, 0x40);
    let (c, st) = if cache == 0 {
        m.w32(tmp + 0x20, u32::MAX);
        (tmp, u32::MAX)
    } else {
        (cache, m.r32(cache + 0x20))
    };
    if st != m.r32(obj + 0x114) {
        ee_sub_1c1440(m, obj, c);
    }
    let hi: V4 = std::array::from_fn(|i| p0[i].max(p1[i]));
    let lo: V4 = std::array::from_fn(|i| p0[i].min(p1[i]));
    let xl = m.r32(obj + 0x104);
    ee_sub_1c15f8(m, lo[0], hi[0], xl, c + 0x24, c + 0x28, c);
    let yl = m.r32(obj + 0x108);
    ee_sub_1c15f8(m, lo[1], hi[1], yl, c + 0x2c, c + 0x30, c + 0x10);
    let bits: [u32; 4] = std::array::from_fn(|i| m.r32(c + 4 * i as u32) & m.r32(c + 0x10 + 4 * i as u32));
    let mut n = 0;
    for i in 0..(m.r32(obj + 0xfc) as i32).max(0) as u32 {
        if bits[(i >> 5) as usize] & (1 << (i & 0x1f)) == 0 {
            continue;
        }
        if max < n {
            break;
        }
        let _t0 = m.r32(obj + 0x10c);
        let z = m.r32(_t0 + i * 4);
        m.w32(out + n as u32 * 4, z);
        n += 1;
    }
    let s = m.r32(obj + 0x114);
    m.w32(c + 0x20, s);
    stack_pop(m, 0x40);
    n
}

/// FUN_001c18b8: drop slot `slot`'s two endpoints (min first, then max) from an `n`-entry list.
/// (A missing max endpoint leaves the original's s0 undefined; 0 is used.)
pub fn ee_sub_1c18b8<M: QMem + ?Sized>(m: &mut M, n: i32, list: u32, slot: u32) {
    let (mut lo, mut hi) = (0i32, 0i32);
    for i in 0..n.max(0) {
        let e = list + i as u32 * 8;
        if m.r16(e + 6) as u32 != slot {
            continue;
        }
        if m.r16(e + 4) == 0 {
            hi = i;
            break;
        }
        lo = i;
    }
    let a = list.wrapping_add((lo * 8) as u32);
    m.call(0x001a_e5a0, &[a, a + 8, ((hi - lo - 1) * 8) as u32], &[]);
    let b = list.wrapping_add((hi * 8) as u32);
    m.call(0x001a_e5a0, &[b - 8, b + 8, ((n - hi - 1) * 8) as u32], &[]);
}

/// FUN_001c1968: remove zone `z` from the sweep-and-prune set
pub fn ee_sub_1c1968<M: QMem + ?Sized>(m: &mut M, obj: u32, z: u32) {
    let cap = m.r32(obj + 0xfc) as i32;
    let slots = m.r32(obj + 0x10c);
    let mut i = 0;
    while i < cap && m.r32(slots + i as u32 * 4) != z {
        i += 1;
    }
    m.w32(slots + i as u32 * 4, 0);
    let n = m.r32(obj + 0x100) as i32;
    let xl = m.r32(obj + 0x104);
    ee_sub_1c18b8(m, n, xl, i as u32);
    let yl = m.r32(obj + 0x108);
    ee_sub_1c18b8(m, n, yl, i as u32);
    let _t0 = m.r32(obj + 0xf8);
    m.w32(obj + 0xf8, _t0.wrapping_sub(1));
    let _t0 = m.r32(obj + 0x100);
    m.w32(obj + 0x100, _t0.wrapping_sub(2));
    stamp_bump(m, obj);
}

/// FUN_001c1a38: insert slot `slot`'s endpoints {lo, is_min 1} / {hi, 0} into an `n`-entry sorted list
pub fn ee_sub_1c1a38<M: QMem + ?Sized>(m: &mut M, lo: f32, hi: f32, n: i32, list: u32, slot: u16) {
    let at = |i: i32| list + (i * 8) as u32;
    let mut i = 0;
    while i < n && m.rf(at(i)) < lo {
        i += 1;
    }
    let mut j = i;
    while j < n && m.rf(at(j)) <= hi {
        j += 1;
    }
    let pj = at(j);
    m.call(0x0023_32b8, &[pj + 0x10, pj, ((n - j) * 8) as u32], &[]);
    m.wf(pj + 8, hi);
    m.w16(pj + 0xe, slot);
    m.w16(pj + 0xc, 0);
    let pi = at(i);
    m.call(0x0023_32b8, &[pi + 8, pi, ((j - i) * 8) as u32], &[]);
    m.w16(pi + 6, slot);
    m.wf(pi, lo);
    m.w16(pi + 4, 1);
}

/// FUN_001c1ba0: add zone `z`: merged (FUN_001bde80) with the first overlapping zone, else inserted
/// into a free slot (growing via FUN_001c1d20) with its x / y endpoints; z +0x1dc = obj.
pub fn ee_sub_1c1ba0<M: QMem + ?Sized>(m: &mut M, obj: u32, z: u32) {
    let buf = stack_push(m, 0x90);
    let (lo, hi) = (m.rv(z + 0x1a0), m.rv(z + 0x1b0));
    let n = ee_sub_1c1700(m, obj, 0, lo, hi, 0x20, buf);
    for i in 0..n.max(0) as u32 {
        let o = m.r32(buf + i * 4);
        if nav::ee_sub_1bd8a8(m, z, o) {
            let pair = stack_push(m, 8);
            m.w32(pair, z);
            m.w32(pair + 4, o);
            nav::ee_sub_1bde80(m, 2, pair);
            stack_pop(m, 8);
            stack_pop(m, 0x90);
            return;
        }
    }
    stack_pop(m, 0x90);
    if m.r32(obj + 0xf8) as i32 >= m.r32(obj + 0xfc) as i32 {
        ee_sub_1c1d20(m, obj);
    }
    let cnt = m.r32(obj + 0xf8) as i32 + 1;
    m.w32(obj + 0xf8, cnt as u32);
    let slots = m.r32(obj + 0x10c);
    let mut k = 0;
    if cnt > 0 && m.r32(slots) != 0 {
        k = 1;
        while k < cnt && m.r32(slots + k as u32 * 4) != 0 {
            k += 1;
        }
    }
    let _t0 = m.r32(obj + 0x10c);
    m.w32(_t0 + k as u32 * 4, z);
    let ne = m.r32(obj + 0x100) as i32;
    let (xl, yl) = (m.r32(obj + 0x104), m.r32(obj + 0x108));
    let (x0, x1, y0, y1) = (m.rf(z + 0x1a0), m.rf(z + 0x1b0), m.rf(z + 0x1a4), m.rf(z + 0x1b4));
    ee_sub_1c1a38(m, x0, x1, ne, xl, k as u16);
    ee_sub_1c1a38(m, y0, y1, ne, yl, k as u16);
    m.w32(z + 0x1dc, obj);
    m.w32(obj + 0x100, (ne + 2) as u32);
    stamp_bump(m, obj);
}

/// FUN_001c1d20: grow the slot capacity 0 -> 16 -> 128 (other capacities leave the original's s3
/// undefined; 128 is used), copying the lists; an empty set gets the -FLT_MAX / FLT_MAX sentinels
/// (slot 0xff).
pub fn ee_sub_1c1d20<M: QMem + ?Sized>(m: &mut M, obj: u32) {
    let cap = if m.r32(obj + 0xfc) == 0 { 0x10 } else { 0x80 };
    let sz = cap * 0x10 + 0x10;
    let xl = alloc(m, 0x001a_e1d0, sz, &[]);
    let yl = alloc(m, 0x001a_e1d0, sz, &[]);
    let sl = alloc(m, 0x001a_e1d0, cap << 2, &[]);
    let n = m.r32(obj + 0x100);
    let (ox, oy, os) = (m.r32(obj + 0x104), m.r32(obj + 0x108), m.r32(obj + 0x10c));
    m.call(0x001a_e5a0, &[xl, ox, n << 3], &[]);
    m.call(0x001a_e5a0, &[yl, oy, n << 3], &[]);
    let old = m.r32(obj + 0xfc);
    m.call(0x001a_e5a0, &[sl, os, old << 2], &[]);
    m.call(0x0023_33b4, &[sl + old * 4, 0, (cap - old) * 4], &[]);
    m.w32(obj + 0x108, yl);
    m.w32(obj + 0x10c, sl);
    m.w32(obj + 0xfc, cap);
    m.w32(obj + 0x104, xl);
    if n == 0 {
        for l in [xl, yl] {
            m.wf(l, -BIG);
            m.w16(l + 4, 1);
            m.w16(l + 6, 0xff);
            m.wf(l + 8, BIG);
            m.w16(l + 0xc, 0);
            m.w16(l + 0xe, 0xff);
        }
        m.w32(obj + 0x100, 2);
    }
}

/// FUN_001c1ec0: position of a target {track, object, pos}: the track point nearest `p`
/// (FUN_001e6f60 against 0x3dd020), else the object's position (FUN_001a62e8), else the stored one.
pub fn ee_sub_1c1ec0<M: QMem + ?Sized>(m: &mut M, t: u32, p: V4) -> V4 {
    let track = m.r32(t);
    if track != 0 {
        let o = stack_push(m, 0x10);
        m.call(0x001e_6f60, &[track, 0x003d_d020, o, 0, 0, 0], &[p]);
        let r = m.rv(o);
        stack_pop(m, 0x10);
        return r;
    }
    let obj = m.r32(t + 4);
    if obj != 0 {
        m.call(0x001a_62e8, &[obj], &[]).1
    } else {
        m.rv(t + 0x10)
    }
}

/// FUN_001c1fa0: is `obj` within `radius` (2D) of target `t`? (always for FUN_00132b10(obj, 0x40))
pub fn ee_sub_1c1fa0<M: QMem + ?Sized>(m: &mut M, radius: f32, t: u32, obj: u32) -> u32 {
    if m.call(0x0013_2b10, &[obj, 0x40], &[]).0 != 0 {
        return 1;
    }
    let _t0 = m.r32(obj + 0x218);
    let p = m.rv(_t0);
    let q = ee_sub_1c1ec0(m, t, p);
    (dist_xy(p, q) <= radius) as u32
}

/// FUN_001c20e8: append a 0x20-byte target (max 6, count +0x50, entries +0x60): `obj` goes to
/// {track} when FUN_00132b10(obj, 10), else to {object}.
pub fn ee_sub_1c20e8<M: QMem + ?Sized>(m: &mut M, o: u32, obj: u32) {
    let n = m.r32(o + 0x50);
    if n < 6 {
        m.w32(o + 0x50, n + 1);
        let e = o + n * 0x20 + 0x60;
        if m.call(0x0013_2b10, &[obj, 10], &[]).0 == 0 {
            m.w32(e + 4, obj);
        } else {
            m.w32(e, obj);
        }
    }
}

/// FUN_001c21d8: A* over rooms (portals = directed edges from +0x124 to +0x128 room; cost = distance
/// to the portal entry +0x130 + its length to +0x140; heuristic = distance to `goal`; all 3D) from the
/// room of `actor` to the room at `goal` (mode != 0) or the first room whose mesh contains `goal`
/// (mode == 0). Portals pass FUN_001c26e0 (cached per portal). Records 0x20 per room {room, portal,
/// parent, g, f, state (1 closed / 2 open), link}. Writes the portals in order to `out` (at most `max`);
/// returns their count (1 with out[0] = 0 when already there in mode 0; 0 when unreachable).
pub fn ee_sub_1c21d8<M: QMem + ?Sized>(m: &mut M, g: u32, actor: u32, goal: V4, mode: u32, max: i32, out: u32) -> i32 {
    let mark = stack_push(m, 0x10);
    m.call(0x001a_e478, &[mark], &[]);
    let (nr, np) = (m.r32(g + 0x10), m.r32(g + 0x20));
    let recs = alloc(m, 0x001a_e378, nr << 5, &[]);
    let allowed = alloc(m, 0x001a_e318, np << 2, &[]);
    let _t0 = m.r32(g + 0x20);
    m.call(0x0023_33b4, &[allowed, 0xff, _t0 << 2], &[]);
    let start = m.call(0x001a_62e8, &[actor], &[]).1;
    let r0 = m.call(0x001c_0d70, &[0], &[start]).0;
    let rg = m.call(0x001c_0d70, &[0], &[goal]).0;
    let mut ret = 0;
    if r0 != 0 && rg != 0 {
        let lists = stack_push(m, 0x20);
        let (open, closed) = (lists, lists + 0x10);
        for l in [open, closed] {
            m.call(0x0011_a200, &[l], &[]);
            m.call(0x0011_a1f8, &[l, 0x18], &[]);
        }
        let rec = |m: &mut M, room: u32| recs + m.r32(room + 0x124) * 0x20;
        let s = rec(m, r0);
        m.w32(s, r0);
        m.wf(s + 0x10, len3(sub(start, goal)));
        m.w32(s + 0x14, 2);
        m.call(0x0011_a220, &[open, s], &[]);
        let mut done = 0;
        while m.call(0x0011_a3b0, &[open], &[]).0 == 0 {
            let cur = m.r32(open);
            let room = m.r32(cur);
            let reached = if mode == 0 { ee_sub_1c0920(m, room, goal, 1) } else { room == rg };
            if reached {
                done = cur;
                break;
            }
            m.call(0x0011_a2e8, &[open, cur], &[]);
            m.w32(cur + 0x14, 1);
            m.call(0x0011_a220, &[closed, cur], &[]);
            let via = m.r32(cur + 4);
            let here = if via == 0 { start } else { m.rv(via + 0x140) };
            let adj = m.r32(g + 0x1c) + m.r32(room + 0x124) * 8;
            let mut i = 0;
            while i < m.r32(adj) as i32 {
                let _t0 = m.r32(adj + 4);
                let pi = m.r32(_t0 + i as u32 * 4);
                let ok = allowed + pi * 4;
                let _t0 = m.r32(g + 0x24);
                let portal = m.r32(_t0 + pi * 4);
                if m.r32(ok) == u32::MAX {
                    let v = ee_sub_1c26e0(m, g, portal, actor);
                    m.w32(ok, v);
                }
                i += 1;
                if m.r32(ok) == 0 {
                    continue;
                }
                let (pa, pb) = (m.rv(portal + 0x130), m.rv(portal + 0x140));
                let gn = m.rf(cur + 0xc) + len3(sub(here, pa)) + len3(sub(pa, pb));
                let to = m.r32(portal + 0x128);
                let r = rec(m, to);
                let fnew = gn + len3(sub(pb, goal));
                let st = m.r32(r + 0x14) as i32;
                if st != 0 && !(fnew < m.rf(r + 0x10)) {
                    continue;
                }
                if st == 1 {
                    m.call(0x0011_a2e8, &[closed, r], &[]);
                } else if st == 2 {
                    m.call(0x0011_a2e8, &[open, r], &[]);
                }
                m.w32(r, to);
                m.w32(r + 4, portal);
                m.wf(r + 0xc, gn);
                m.w32(r + 8, cur);
                m.wf(r + 0x10, fnew);
                m.w32(r + 0x14, 2);
                let head = m.r32(open);
                let mut before = head;
                if head != 0 && m.rf(head + 0x10) <= fnew {
                    before = m.r32(head + 0x18);
                    while before != 0 && !(fnew < m.rf(before + 0x10)) {
                        before = m.r32(before + 0x18);
                    }
                }
                m.call(0x0011_a280, &[open, before, r], &[]);
            }
        }
        if done != 0 {
            let parent = m.r32(done + 8);
            if mode == 0 && parent == 0 {
                ret = 1;
                m.w32(out, 0);
            } else {
                let mut depth = 0;
                let mut e = done;
                while m.r32(e + 8) != 0 {
                    e = m.r32(e + 8);
                    depth += 1;
                }
                ret = depth.min(max);
                let mut e = done;
                let mut slot = out.wrapping_add(((depth - 1) * 4) as u32);
                while m.r32(e + 8) != 0 {
                    if depth <= max {
                        let _t0 = m.r32(e + 4);
                        m.w32(slot, _t0);
                    }
                    e = m.r32(e + 8);
                    slot = slot.wrapping_sub(4);
                    depth -= 1;
                }
            }
        }
        stack_pop(m, 0x20);
    }
    m.call(0x001a_e4a0, &[mark, 2], &[]);
    stack_pop(m, 0x10);
    ret
}

/// FUN_001c26e0: may `actor` use `portal`? Needs portal flag bit 60, the actor's +0xfe8 ability bits
/// (2, or 3 for portal +0x120 == 1) and, with a room filter (+0xe94 count, +0xe98 rooms), both rooms.
pub fn ee_sub_1c26e0<M: QMem + ?Sized>(m: &mut M, _g: u32, portal: u32, actor: u32) -> u32 {
    if (m.r64(portal + 0x28) >> 60) & 1 == 0 {
        return 0;
    }
    let need = if m.r32(portal + 0x120) == 1 { 3 } else { 2 };
    if need & m.r32(actor + 0xfe8) != need {
        return 0;
    }
    let n = m.r32(actor + 0xe94) as i32;
    if n > 0 {
        let (a, b) = (m.r32(portal + 0x124), m.r32(portal + 0x128));
        let first = m.r32(actor + 0xe98);
        let (mut fa, mut fb) = (a == first, b == first);
        let mut i = 0;
        while !fa || !fb {
            i += 1;
            if n <= i {
                break;
            }
            let r = m.r32(actor + 0xe98 + i as u32 * 4);
            fa |= a == r;
            fb |= b == r;
        }
        if !fa || !fb {
            return 0;
        }
    }
    1
}

/// FUN_001c27c8 / FUN_001c2920 / FUN_001c2bc8 / FUN_001c3238: this TU's shared pool static initialiser
pub fn ee_sub_1c27c8<M: QMem + ?Sized>(m: &mut M, init: u32, prio: u32) {
    crate::engine::game::q_0021c6c8::ee_sub_21c6c8(m, init, prio)
}

/// FUN_001c2900 / FUN_001c2a58 / FUN_001c2d00 / FUN_001c3370
pub fn ee_sub_1c2900<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1c27c8(m, 1, 0xffff)
}

/// DAT_003d3980 (current scratch layout) / DAT_002dea50 (2 buffer descriptors, stride 0x50)
pub const SCRATCH_MODE: u32 = 0x003d_3980;
pub const SCRATCH_DESC: u32 = 0x002d_ea50;

/// FUN_001c2a78: switch the 2 double-buffer descriptors (+8 / +0xc buffer A + qwords, +0x18 / +0x1c
/// buffer B + qwords) to layout 2 (small), 3 (large) or none; FlushCache(0) first.
pub fn ee_sub_1c2a78<M: QMem + ?Sized>(m: &mut M, mode: u32) {
    if mode == m.r32(SCRATCH_MODE) {
        return;
    }
    m.call(0x0023_4ca0, &[0], &[]);
    m.w32(SCRATCH_MODE, mode);
    for k in 0..2u32 {
        let d = SCRATCH_DESC + k * 0x50;
        let (a, an, b, bn) = match mode {
            2 => (0x003c_f980 + k * 0x1800, 0x180, 0x003d_2980 + k * 0x800, 0x80),
            3 => (0x002f_7980 + k * 0x6000, 0x600, 0x0030_3980 + k * 0x58000, 0x5800),
            _ => {
                m.w32(d + 0xc, 0);
                m.w32(d + 8, 0);
                m.w32(d + 0x1c, 0);
                m.w32(d + 0x18, 0);
                continue;
            }
        };
        m.w32(d + 8, a);
        m.w32(d + 0x18, b);
        m.w32(d + 0xc, an);
        m.w32(d + 0x1c, bn);
    }
}

/// FUN_001c2ba8
pub fn ee_sub_1c2ba8<M: QMem + ?Sized>(m: &mut M) {
    ee_sub_1c2a78(m, 0)
}

/// FUN_001c2db8: clear the thrower's +0x150 / +0x154 / +0x158
pub fn ee_sub_1c2db8<M: QMem + ?Sized>(m: &mut M, o: u32) {
    m.w32(o + 0x150, 0);
    m.w32(o + 0x158, 0);
    m.w32(o + 0x154, 0);
}

/// FUN_001c2dc8: arm the thrower with `src`: a spawner (FUN_00132b10 6) yields its first object
/// (FUN_001cb250; none -> 1), a class-0x30 object is used directly (else 2). Class 0x5e gets
/// FUN_001c7a58(obj, a2); class 0x5a slot 75 (+0x264)(2), else slot 8 (+0x4c); +0x150 =
/// FUN_001f4c58(o, a3, 0xd); returns 3 (0 when already armed).
pub fn ee_sub_1c2dc8<M: QMem + ?Sized>(m: &mut M, o: u32, src: u32, a2: u32, a3: u32) -> u32 {
    if m.r32(o + 0x158) != 0 {
        return 0;
    }
    m.w32(o + 0x154, 0);
    if m.call(0x0013_2b10, &[src, 6], &[]).0 == 0 {
        if m.call(0x0013_2b10, &[src, 0x30], &[]).0 == 0 {
            return 2;
        }
        m.w32(o + 0x158, src);
    } else {
        m.w32(o + 0x154, src);
        let x = m.call(0x001c_b250, &[src, 0], &[]).0;
        m.w32(o + 0x158, x);
        if x == 0 {
            m.w32(o + 0x154, 0);
            return 1;
        }
    }
    let obj = m.r32(o + 0x158);
    if m.call(0x0013_2b10, &[obj, 0x5e], &[]).0 != 0 {
        m.call(0x001c_7a58, &[obj, a2], &[]);
    }
    let obj = m.r32(o + 0x158);
    if m.call(0x0013_2b10, &[obj, 0x5a], &[]).0 == 0 {
        vslot(m, obj, 8, &[], &[]);
    } else {
        vslot(m, obj, 75, &[2], &[]);
    }
    let t = m.call(0x001f_4c58, &[o, a3, 0xd], &[]).0;
    m.w32(o + 0x150, t);
    3
}

/// FUN_001c2ef0: throw `n` objects (the armed one, then more from the spawner +0x154) at the target
/// +0x150: aim point = player position + a random offset (FUN_0011e1a0(rng(-1.5, 1.5), rng(100, 150),
/// -75)) rotated into the horizontal player-from-target frame (FUN_0011b830), launch velocity
/// FUN_001d69f0(rng(0.75, 1.2), target pos, aim, (0, 0, -2250)). Class 0x5a goes through
/// FUN_001c6bd0(0.05 i, 0x5b/0x5d ? 0.15 i : FLT_MAX, ..), others slot 7, slot 38(target), slot 34(v).
/// Clears the thrower and returns the first object.
pub fn ee_sub_1c2ef0<M: QMem + ?Sized>(m: &mut M, o: u32, n: i32) -> u32 {
    let mut first = 0;
    let mut i = 0;
    let rng = |m: &mut M, lo: f32, hi: f32| f32::from_bits(m.call(0x0011_d448, &[], &[f(lo), f(hi)]).0);
    while i < n {
        let obj = if i == 0 {
            m.r32(o + 0x158)
        } else {
            let _t0 = m.r32(o + 0x154);
            let x = m.call(0x001c_b250, &[_t0, 0], &[]).0;
            if x == 0 {
                break;
            }
            if m.call(0x0013_2b10, &[x, 0x5a], &[]).0 != 0 {
                vslot(m, x, 75, &[2], &[]);
            }
            x
        };
        if obj == 0 {
            break;
        }
        if first == 0 {
            first = obj;
        }
        let _t0 = m.r32(o + 0x150);
        let tpos = m.rv(_t0 + 0x70);
        let player = m.call(0x001c_4840, &[0x003d_d490], &[]).0;
        let ppos = m.call(0x001a_62e8, &[player], &[]).1;
        let ang = rng(m, -1.5, 1.5);
        let dist = rng(m, 100.0, 150.0);
        let off = m.call(0x0011_e1a0, &[], &[f(ang), f(dist), f(-75.0)]).1;
        let d = sub(ppos, tpos);
        let up = [0.0, 0.0, 1.0, 0.0];
        let k = d[0] * up[0] + d[1] * up[1] + d[2] * up[2];
        let h: V4 = std::array::from_fn(|j| d[j] - up[j] * k);
        let l = len3(h);
        let dir = if l < 0.0001 { [1.0, 0.0, 0.0, 0.0] } else { h.map(|x| x * (1.0 / l)) };
        let mat = stack_push(m, 0x40);
        m.call(0x0011_b830, &[mat], &[dir]);
        let r = m.rm(mat);
        stack_pop(m, 0x40);
        let mut aim = [0.0f32; 4];
        for j in 0..3 {
            aim[j] = ppos[j] + r[0][j] * off[0] + r[1][j] * off[1] + r[2][j] * off[2];
        }
        aim[3] = ppos[3] + dir[3];
        let t = rng(m, 0.75, 1.2);
        let v = m.call(0x001d_69f0, &[], &[f(t), tpos, aim, [0.0, 0.0, -2250.0, 0.0]]).1;
        if m.call(0x0013_2b10, &[obj, 0x5a], &[]).0 == 0 {
            vslot(m, obj, 7, &[], &[]);
            let tg = m.r32(o + 0x150);
            vslot(m, obj, 38, &[tg], &[]);
            vslot(m, obj, 34, &[], &[v]);
        } else {
            let spread = if m.call(0x0013_2b10, &[obj, 0x5b], &[]).0 != 0 || m.call(0x0013_2b10, &[obj, 0x5d], &[]).0 != 0 {
                i as f32 * 0.15
            } else {
                BIG
            };
            let _t0 = m.r32(o + 0x150);
            let tp = m.rv(_t0 + 0x70);
            m.call(0x001c_6bd0, &[obj], &[f(i as f32 * 0.05), f(spread), tp, v]);
        }
        i += 1;
    }
    ee_sub_1c2db8(m, o);
    first
}
