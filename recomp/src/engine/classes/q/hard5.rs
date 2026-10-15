//! WP-HARD5: the 33 largest game-logic functions that were `port_state='skipped'`
//! (tools/engineport/hard5_list.txt), ported 1:1 from the raw disassembly (tools/eedis.py VU0 lane
//! decode + the Ghidra decompile for control flow). Same QMem convention as hard1/hard3: integer /
//! pointer args in `a`, float / quadword args in `v`; quadword result is `.1`. Callees outside this
//! package are reached through `QMem::call` (by original address) so the host routes them to their own
//! port. VU0 macro math reuses `engine::math::vu0`.
#![allow(clippy::too_many_arguments)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};
use crate::engine::classes::q::q6::vcall;
use crate::engine::math::vu0;

/// FUN_0014c2d8: cheat-table registration. Calls FUN_0011a050(this, 0x18) to size/init the owner's
/// record list, then registers the 15 cheat records in the static table at 0x2bd4e0 (stride 0x38)
/// through FUN_0014bea8(this, record). Ported 1:1; the two callees are host-routed.
pub fn ee_sub_14c2d8<M: QMem + ?Sized>(m: &mut M, this: u32) {
    m.call(0x0011_a050, &[this, 0x18], &[]);
    for i in 0..15u32 {
        m.call(0x0014_bea8, &[this, 0x002b_d4e0 + i * 0x38], &[]);
    }
}

/// FUN_00134040: body-pair collision broadphase driver (world `this`). Walks the active-body list at
/// `this+0x3d34` (next at +0x58); for each body (flag word at +0x70) not already resolved (bit 4) it
/// flags it (bit 4, and bit 3 when first seen -> pushed on the work stack), then walks its contact list
/// at +0x64 (each node: body at +0xec of node[3], next at node[1]). For each partner body not yet
/// flagged it calls the two collision-shape setup slots (vt+0x1b4 on each body's +0x60 sub-object with
/// the other body's +0x60), FUN_001a7e10 to retire the old contact, then the narrowphase
/// FUN_001aa1d8(this, a, b); the result replaces node[0] and *node[2]. When a pair produces no contact
/// each side's slot vt+0x1f4 is queried and FUN_001f5078 called on a 0 result. A second pass retires all
/// still-listed bodies (slot vt+0x1b4 with 0, FUN_001a7e10 of +0x68, slot vt+0x1dc, FUN_0011a2e8 unlink).
/// Finally the work stack is drained (clear bit 3, slot vt+0x1d4). FUN_001ae478/4a0 bracket a scratch
/// frame. Ported 1:1; the worklist (`aiStack_c90[768]`) is a local Vec since its address never escapes.
pub fn ee_sub_134040<M: QMem + ?Sized>(m: &mut M, this: u32) {
    let frame = stack_push(m, 0x10);
    m.call(0x0013_3e30, &[], &[]);
    m.call(0x0013_3e80, &[this], &[]);
    m.call(0x001a_e478, &[frame], &[]);
    let mut work: Vec<u32> = Vec::new();
    let mut body = m.r32(this + 0x3d34);
    if body != 0 {
        loop {
            let f = m.r32(body + 0x70);
            if f & 4 == 0 {
                m.w32(body + 0x70, f | 0x10);
                if f & 8 == 0 {
                    m.w32(body + 0x70, f | 0x18);
                    work.push(body);
                }
                let mut node = m.r32(body + 0x64);
                if node != 0 {
                    let mut p3 = m.r32(node + 0xc); // piVar8[3]
                    loop {
                        let partner = m.r32(p3 + 0xec);
                        let pf = m.r32(partner + 0x70);
                        if pf & 0x10 == 0 && pf & 4 == 0 {
                            if pf & 8 == 0 {
                                m.w32(partner + 0x70, pf | 8);
                                work.push(partner);
                            }
                            let ob = m.r32(body + 0x60);
                            let pb = m.r32(partner + 0x60);
                            vcall(m, ob, 0xc, 0x1b4, &[pb], &[]);
                            vcall(m, pb, 0xc, 0x1b4, &[ob], &[]);
                            let old = m.r32(node); // iVar5 = *piVar8
                            m.call(0x001a_7e10, &[this, old], &[]);
                            let nc = m.call(0x001a_a1d8, &[this, body, partner], &[]).0;
                            m.w32(node, nc);
                            let p2 = m.r32(node + 8);
                            m.w32(p2, nc);
                            if old == 0 && m.r32(node) == 0 {
                                node = m.r32(node + 4);
                            } else {
                                let ob = m.r32(body + 0x60);
                                let r = vcall(m, ob, 0xc, 0x1f4, &[], &[]).0;
                                if r == 0 {
                                    let bb = m.r32(body + 0x60);
                                    m.call(0x001f_5078, &[bb], &[]);
                                }
                                let pb = m.r32(partner + 0x60);
                                let r2 = vcall(m, pb, 0xc, 0x1f4, &[], &[]).0;
                                if r2 == 0 {
                                    let pp = m.r32(partner + 0x60);
                                    m.call(0x001f_5078, &[pp], &[]);
                                }
                                node = m.r32(node + 4);
                            }
                        } else {
                            node = m.r32(node + 4);
                        }
                        if node == 0 {
                            break;
                        }
                        p3 = m.r32(node + 0xc);
                    }
                }
            }
            body = m.r32(body + 0x58);
            if body == 0 {
                break;
            }
        }
        // second pass: retire all still-listed bodies
        let mut b2 = m.r32(this + 0x3d34);
        while b2 != 0 {
            let f = m.r32(b2 + 0x70);
            m.w32(b2 + 0x70, f & !0x10);
            let ob = m.r32(b2 + 0x60);
            vcall(m, ob, 0xc, 0x1b4, &[0], &[]);
            let old68 = m.r32(b2 + 0x68);
            m.call(0x001a_7e10, &[this, old68], &[]);
            m.w32(b2 + 0x68, 0);
            let ob = m.r32(b2 + 0x60);
            vcall(m, ob, 0xc, 0x1dc, &[], &[]);
            let f2 = m.r32(b2 + 0x70);
            m.w32(b2 + 0x70, f2 & !2);
            m.call(0x0011_a2e8, &[this + 0x3d34, b2], &[]);
            b2 = m.r32(this + 0x3d34);
        }
    }
    // drain work stack: clear bit 3 and fire slot vt+0x1d4
    for &cur in &work {
        let f = m.r32(cur + 0x70);
        m.w32(cur + 0x70, f & !8);
        let ob = m.r32(cur + 0x60);
        vcall(m, ob, 0xc, 0x1d4, &[], &[]);
    }
    m.call(0x001a_e4a0, &[frame, 2], &[]);
    stack_pop(m, 0x10);
}

/// FUN_001a8410: contact-list build / edge dedup. Over `count` contact records (`base`, stride 0x60):
/// records with +0x4c==0 && +0x50==0 are chained through +0x5c into the local free chain; records with
/// +0x4c!=0 && +0x50!=0 are pushed onto the `*head` list (+0x5c = old head). The remaining records go
/// through edge dedup: when +0x4c!=0 && +0x50==0 the two contact halves ([+0..+0x1c],[+0x20..+0x3c],
/// scalars +0x40/+0x44, flags +0x4c/+0x50) are swapped (flip A<->B, +0x58:=1); then the edge id
/// `u16[*(+0x54) + (+0x58)*2]` is looked up in `edge_ids[0..n]` — new ids append (edge_ids[n], recs[n]),
/// matches set +0x5c = recs[match]. A second compaction pass removes records whose +0x5c==0 that are not
/// reachable along the `*head`-list +0x54 arrays, memmoving (FUN_002332b8) the recs/ids arrays down.
/// `*out_count` := final dedup count. Ported 1:1 from the decompile; pure pointer/int logic.
pub fn ee_sub_1a8410<M: QMem + ?Sized>(
    m: &mut M,
    count: i32,
    base: u32,
    out_count: u32,
    edge_ids: u32,
    recs: u32,
    head: u32,
) {
    let mut n: i32 = 0; // iVar19: dedup count
    let mut headptr: u32 = 0; // puVar11 across iterations / second-pass sentinel
    m.w32(head, 0);
    if count > 0 {
        let mut prev: u32 = 0; // puVar12
        let mut idx: i32 = 0; // iVar15
        loop {
            let rec = (idx as u32).wrapping_mul(0x60).wrapping_add(base);
            headptr = rec;
            let c4c = m.r32(rec + 0x4c);
            let c50 = m.r32(rec + 0x50);
            let mut dedup = false;
            if c4c == 0 {
                if c50 != 0 {
                    dedup = true; // goto LAB_8500
                } else {
                    m.w32(rec + 0x5c, prev);
                }
            } else if c50 == 0 {
                // --- flip the two contact halves ---
                let u1lo = m.r32(rec);
                let u1hi = m.r32(rec + 4);
                let u3 = m.r32(rec + 8);
                let u4a = m.r32(rec + 0xc);
                let u6 = m.r32(rec + 0x20);
                let u8 = m.r32(rec + 0x24);
                let u9 = m.r32(rec + 0x28);
                let u10 = m.r32(rec + 0x2c);
                let v = m.r32(rec + 0x10);
                m.w32(rec, v);
                let v = m.r32(rec + 0x14);
                m.w32(rec + 4, v);
                let v = m.r32(rec + 0x18);
                m.w32(rec + 8, v);
                let v = m.r32(rec + 0x1c);
                m.w32(rec + 0xc, v);
                m.w32(rec + 0x10, u1lo);
                m.w32(rec + 0x14, u1hi);
                m.w32(rec + 0x18, u3);
                m.w32(rec + 0x1c, u4a);
                let u4b = m.r32(rec + 0x40);
                let v = m.r32(rec + 0x30);
                m.w32(rec + 0x20, v);
                let v = m.r32(rec + 0x34);
                m.w32(rec + 0x24, v);
                let v = m.r32(rec + 0x38);
                m.w32(rec + 0x28, v);
                let v = m.r32(rec + 0x3c);
                m.w32(rec + 0x2c, v);
                let u3b = m.r32(rec + 0x4c);
                m.w32(rec + 0x30, u6);
                m.w32(rec + 0x34, u8);
                m.w32(rec + 0x38, u9);
                m.w32(rec + 0x3c, u10);
                let v = m.r32(rec + 0x44);
                m.w32(rec + 0x40, v);
                m.w32(rec + 0x44, u4b);
                let v = m.r32(rec + 0x50);
                m.w32(rec + 0x4c, v);
                m.w32(rec + 0x50, u3b);
                m.w32(rec + 0x58, 1);
                dedup = true;
            } else {
                // +0x4c!=0 && +0x50!=0 -> push on *head list
                let h = m.r32(head);
                m.w32(rec + 0x5c, h);
                m.w32(head, rec);
                headptr = prev;
            }
            if dedup {
                let eb = m.r32(rec + 0x54);
                let e13 = m.r32(rec + 0x58);
                let key = m.r16(eb + e13.wrapping_mul(2)) as u32;
                let mut slot: i32 = n; // iVar14; append position defaults to current count
                let mut append = false;
                if n < 1 {
                    slot = 0;
                    append = true;
                } else if key != m.r32(edge_ids) {
                    // scan edge_ids[1..n]
                    let mut found = false;
                    let mut i = 1i32;
                    while i < n {
                        if key == m.r32(edge_ids + (i as u32) * 4) {
                            slot = i;
                            found = true;
                            break;
                        }
                        i += 1;
                    }
                    if found {
                        let rs = m.r32(recs + (slot as u32) * 4);
                        m.w32(rec + 0x5c, rs);
                    } else {
                        slot = n;
                        append = true;
                    }
                } else {
                    // key == edge_ids[0]
                    slot = 0;
                    let r0 = m.r32(recs);
                    m.w32(rec + 0x5c, r0);
                }
                if append {
                    m.w32(edge_ids + (slot as u32) * 4, key);
                    n += 1;
                }
                m.w32(recs + (slot as u32) * 4, rec);
                headptr = prev;
            }
            idx += 1;
            prev = headptr;
            if idx >= count {
                break;
            }
        }
    }
    // second pass: drop dedup slots whose rec +0x5c==0 that are not present as a u16[0]/u16[1] edge id
    // of any node on the `*head` list; the recs/edge_ids arrays are compacted (FUN_002332b8 = memmove).
    // piVar16/puVar7 stay in lockstep with the live index `i`, so slots are addressed as base + i*4.
    if n > 0 {
        let mut i: i32 = 0;
        while i < n {
            let recp = recs + (i as u32) * 4;
            let rec = m.r32(recp);
            if m.r32(rec + 0x5c) == 0 {
                let mut keep = false;
                if headptr != 0 {
                    let key = m.r32(edge_ids + (i as u32) * 4);
                    let mut node = headptr;
                    loop {
                        let arr = m.r32(node + 0x54);
                        if m.r16(arr) as u32 == key || m.r16(arr + 2) as u32 == key {
                            keep = true;
                            break;
                        }
                        node = m.r32(node + 0x5c);
                        if node == 0 {
                            break;
                        }
                    }
                }
                if keep {
                    i += 1;
                } else {
                    // remove slot i
                    let span = ((n - i) * 4 - 4) as u32;
                    let h = m.r32(head);
                    m.w32(rec + 0x5c, h);
                    n -= 1;
                    m.w32(head, rec);
                    m.call(0x0023_32b8, &[recp, recp + 4, span], &[]);
                    let ep = edge_ids + (i as u32) * 4;
                    m.call(0x0023_32b8, &[ep, ep + 4, span], &[]);
                    // i unchanged: slot now holds the shifted-down next element
                }
            } else {
                i += 1;
            }
        }
    }
    m.w32(out_count, n as u32);
}

/// FUN_0016da48: WP-C4 shadow render passes 6/7 (`this`, `pass_a`=param_2, `pass_b`=param_3). Opens the
/// pass (FUN_00160918 with the 0x25c1c0/0x25e040 shader pair, FUN_001574b8), optionally the fog/overlay
/// sub-pass (FUN_00226d10/00157598(0x2df800)/00226ca8 when pass_b!=7 && DAT_003e12c8!=0), grabs a tag
/// via FUN_00226e88(this,5,5,0x1b,0,&tag), then builds a GS packet into a 4 KiB scratch buffer: open
/// (FUN_00166920/00157330), FUN_0016d6c8 unless pass_b==7, FUN_00166bd8(0,1,0xe) + FUN_0016d698, the
/// FRAME FBMSK register (0x4c) 0xffff00ff00000000 (or 0xffff80ff.. when pass_a!=7 && pass_b==7), TEST
/// regs 0x46/0x47, PRIM 1 (0x7e00/0x3e00 by pass_a==7), 0x55 sprite loop (16 rows, two sprites each
/// stepping +0x200), an optional extra clear when pass_a==8 && pass_b==7, then flush (FUN_001669e0,
/// FUN_0016d698(-1)) and submit (FUN_00226f28 with the packet length>>4). Finally DAT_002deb70[pass_a==8?
/// 4:0] := 1. Ported 1:1; the 64-bit GS values are passed as [lo,hi]; all callees are host-routed.
pub fn ee_sub_16da48<M: QMem + ?Sized>(m: &mut M, this: u32, pass_a: i32, pass_b: i32) {
    // scratch frame: writer struct (0x30), packet buffer (0x1000), tag block (0x10)
    let buf = stack_push(m, 0x30);
    let packet = stack_push(m, 0x1000);
    let tag = stack_push(m, 0x10); // tag[0]=uStack_a0, tag[1]=auStack_9c[0]
    let s9c = tag + 4;
    let w64 = |m: &mut M, b: u32, reg: u32, val: u64| {
        m.call(0x0016_6e90, &[b, reg, val as u32, (val >> 32) as u32], &[]);
    };
    m.call(0x0016_0918, &[this, 0x0025_c1c0, 0x0025_e040], &[]);
    m.call(0x0015_74b8, &[this], &[]);
    if pass_b != 7 && m.r32(0x003e_12c8) != 0 {
        m.call(0x0022_6d10, &[this], &[]);
        m.call(0x0015_7598, &[this, 0x002d_f800], &[]);
        m.call(0x0022_6ca8, &[this, 0x32, 0x1e7], &[]);
    }
    m.call(0x0022_6e88, &[this, 5, 5, 0x1b, 0, tag], &[]);
    let t = m.r32(tag);
    m.w32(s9c, t); // auStack_9c[0] = uStack_a0
    m.call(0x0016_6920, &[buf], &[]);
    m.call(0x0015_7330, &[buf, 0x100, packet], &[]);
    if pass_b != 7 {
        m.call(0x0016_d6c8, &[buf, 0x299, s9c], &[]);
    }
    m.call(0x0016_6bd8, &[buf, 0, 1, 0xe], &[]);
    m.call(0x0016_d698, &[0, buf, s9c], &[]);
    if pass_a == 7 || pass_b != 7 {
        w64(m, buf, 0x4c, 0xffff_00ff_0000_0000);
    } else {
        w64(m, buf, 0x4c, 0xffff_80ff_0000_0000);
    }
    w64(m, buf, 0x46, 0);
    w64(m, buf, 0x47, 0x3_1001);
    if pass_a == 7 {
        w64(m, buf, 1, 0x1_0000_7e00);
    } else {
        w64(m, buf, 1, 0x1_0000_3e00);
    }
    w64(m, buf, 0, 6);
    m.call(0x0016_6ef0, &[buf, 2, 0x55], &[]);
    // sprite loop: iVar1 from 0x1e0 down by 0x20 while -1 < iVar1 (16 iterations)
    let mut lo = 0x7000u32;
    let mut hi = 0x7200u32;
    let mut c: i32 = 0x1e0;
    loop {
        m.call(0x0016_7048, &[buf, lo, 0x7900, 0], &[]);
        lo = lo.wrapping_add(0x200);
        m.call(0x0016_7048, &[buf, hi, 0x8700, 0], &[]);
        c -= 0x20;
        hi = hi.wrapping_add(0x200);
        if c < 0 {
            break;
        }
    }
    if pass_a == 8 && pass_b == 7 {
        m.call(0x0016_6bd8, &[buf, 0, 1, 0xe], &[]);
        m.call(0x0016_d698, &[0, buf, s9c], &[]);
        w64(m, buf, 0x4c, 0xffff_00ff_0000_0000);
    }
    m.call(0x0016_69e0, &[buf, 1], &[]);
    m.call(0x0016_d698, &[0xffff_ffff, buf, s9c], &[]);
    let start = m.r32(buf + 4); // iStack_10cc
    let end = m.r32(buf + 0xc); // iStack_10c4
    m.call(0x0022_6f28, &[this, 0xc, end.wrapping_sub(start) >> 4, start, 0x23], &[]);
    m.call(0x0022_6d28, &[this], &[]);
    m.call(0x0022_6ce0, &[this, 0x1aa0], &[]);
    m.call(0x0022_6d28, &[this], &[]);
    m.call(0x0015_76f0, &[this], &[]);
    let off = if pass_a != 8 { 0 } else { 4 };
    m.w32(0x002d_eb70 + off, 1);
    stack_pop(m, 0x10);
    stack_pop(m, 0x1000);
    stack_pop(m, 0x30);
}

/// FUN_001f6068: object-stream relocation / fixup pass (`this`, `src`=param_2). After a transient
/// FUN_001a6730() that it brackets by saving+restoring +0xec/+0x70/+0x64(qw)/+0x6c and clearing +0x134,
/// it reallocates the element table +0x54 (count +0x50, stride 0x100) and walks each element, relocating
/// the embedded pointer at +0xa0 (vtable slot +0x1a4) and, by the element's type code at +0x98: 0xd (sub
/// record: realloc +0xa8 to 0xe0, relocate it + its +4/+8/+0xc, back-link +0xa8), 4 (relocate +0xa8),
/// 0xf (realloc +0xa8 to 0x10, relocate unless head==&DAT_002c44f0, relocate +4), 0x10 (shared sub-table:
/// realloc the 0x50 header + its 0x70-stride rows, back-link +0x60, relocate +100). Each element's child
/// list at +0x90 is reversed into an FUN_001ae318 scratch array and re-inserted via FUN_0012e3f0 +
/// FUN_00230018 under an FUN_001ae478/4a0 frame. Then it relocates +0x58/+0x5c/+0x60, clones the child
/// FUN_0016ea80(this+0x80,..), and builds the +0xe4 instance table (count +0xe0 + slot-0x1fc extra,
/// stride 0xb0): per row relocate, back-link +4, FUN_0011a210(+0x60), FUN_00166270 pose init, bind
/// FUN_001191e8 against src's row, and realloc the +0x10 sub-arrays. Finally the +0xe8 block + its +0xdc
/// sub, the +0x28 bit 0x3c -> FUN_001f2b20, and FUN_001f2500. Ported 1:1; allocator/relocate callees
/// are host-routed. The {size,align} arg struct FUN_001ae238 expects is emulated on the scratch stack.
pub fn ee_sub_1f6068<M: QMem + ?Sized>(m: &mut M, this: u32, src: u32) {
    let ei = stack_push(m, 0x10); // aiStack_e0: {size, align}
    let frame = stack_push(m, 0x10); // auStack_d0
    let pose = stack_push(m, 0x20); // uStack_c0 block (3 qw)
    // FUN_001ae238({size,align}, old) -> new ptr
    fn ae<M: QMem + ?Sized>(m: &mut M, ei: u32, size: u32, align: u32, old: u32, has_old: bool) -> u32 {
        m.w32(ei, size);
        m.w32(ei + 4, align);
        if has_old {
            m.call(0x001a_e238, &[ei, old], &[]).0
        } else {
            m.call(0x001a_e238, &[ei], &[]).0
        }
    }
    // relocate pointer `ptr` through vtable slot +0x1a4 (this-adjust +0x1a0), arg (src, ptr)
    fn reloc<M: QMem + ?Sized>(m: &mut M, this: u32, src: u32, ptr: u32) {
        vcall(m, this, 0xc, 0x1a4, &[src, ptr], &[]);
    }

    let u_ec = m.r32(this + 0xec);
    let u_70 = m.r32(this + 0x70);
    let u_64 = m.r64(this + 0x64);
    let u_6c = m.r32(this + 0x6c);
    m.call(0x001a_6730, &[], &[]);
    m.w32(this + 0xec, u_ec);
    m.w32(this + 0x64, u_64 as u32);
    m.w32(this + 0x68, (u_64 >> 32) as u32);
    m.w32(this + 0x6c, u_6c);
    m.w32(this + 0x70, u_70);
    m.w32(this + 0x134, 0);
    let sz = m.r32(this + 0x50) << 8;
    let old = m.r32(this + 0x54);
    let nt = ae(m, ei, sz, 0x10, old, true);
    m.w32(this + 0x54, nt);
    if (m.r32(this + 0x50) as i32) > 0 {
        let mut idx: i32 = 0;
        loop {
            let el = m.r32(this + 0x54).wrapping_add(idx as u32 * 0x100);
            reloc(m, this, src, el + 0xa0);
            let ty = m.r16(el + 0x98) as i16 as i32;
            let mut chain;
            if ty == 0xd {
                let old = m.r32(el + 0xa8);
                let new = ae(m, ei, 0xe0, 0x10, old, true);
                reloc(m, this, src, new);
                reloc(m, this, src, new + 4);
                reloc(m, this, src, new + 8);
                reloc(m, this, src, new + 0xc);
                let p4 = m.r32(new + 4);
                m.w32(p4 + 0xa8, new);
                let p8 = m.r32(new + 8);
                m.w32(p8 + 0xa8, new);
                chain = m.r32(el + 0x90);
            } else if ty < 0xe {
                if ty == 4 {
                    reloc(m, this, src, el + 0xa8);
                }
                chain = m.r32(el + 0x90);
            } else if ty == 0xf {
                let old = m.r32(el + 0xa8);
                let new = ae(m, ei, 0x10, 0x10, old, true);
                m.w32(el + 0xa8, new);
                if m.r32(new) != 0x002c_44f0 {
                    reloc(m, this, src, new);
                }
                reloc(m, this, src, new + 4);
                chain = m.r32(el + 0x90);
            } else if ty == 0x10 {
                let hdr = { let a = m.r32(el + 0xa8); m.r32(a + 0x60) };
                if m.r32(el + 0xa8) == m.r32(hdr + 4) {
                    let p = ae(m, ei, 0x50, 0x10, hdr, true);
                    let rows_old = m.r32(p + 4);
                    let rc = m.r32(p).wrapping_mul(0x70);
                    let rows = ae(m, ei, rc, 0x10, rows_old, true);
                    m.w32(p + 4, rows);
                    if (m.r32(p) as i32) > 0 {
                        let mut cnt: i32 = 0;
                        let mut off: u32 = 0;
                        loop {
                            cnt += 1;
                            let row = m.r32(p + 4).wrapping_add(off);
                            m.w32(row + 0x60, p);
                            off = off.wrapping_add(0x70);
                            reloc(m, this, src, row + 100);
                            let back = m.r32(row + 100);
                            m.w32(back + 0xa8, row);
                            if m.r32(p) as i32 <= cnt {
                                break;
                            }
                        }
                    }
                    chain = m.r32(el + 0x90);
                } else {
                    chain = m.r32(el + 0x90);
                }
            } else {
                chain = m.r32(el + 0x90);
            }
            idx += 1;
            if chain != 0 {
                // count chain length (links at +0x10)
                let mut len: i32 = 0;
                let mut c = chain;
                loop {
                    c = m.r32(c + 0x10);
                    len += 1;
                    if c == 0 {
                        break;
                    }
                }
                m.call(0x001a_e478, &[frame], &[]);
                let arr = ae(m, ei, (len as u32) * 4, 0x10, 0, false);
                let mut w = arr;
                let mut c = m.r32(el + 0x90);
                while c != 0 {
                    m.w32(w, c);
                    w += 4;
                    c = m.r32(c + 0x10);
                }
                m.w32(el + 0x90, 0);
                // iterate reversed (len-1 .. 0)
                let mut i: i32 = len - 1;
                let mut rp = arr + (i as u32) * 4;
                let mut node = m.r32(rp);
                loop {
                    rp = rp.wrapping_sub(4);
                    let r = m.call(0x0012_e3f0, &[node, this, el], &[]);
                    i -= 1;
                    m.call(0x0023_0018, &[this, el, r.0], &[r.1]);
                    if i < 0 {
                        break;
                    }
                    node = m.r32(rp);
                }
                m.call(0x001a_e4a0, &[frame, 2], &[]);
            }
            if m.r32(this + 0x50) as i32 <= idx {
                break;
            }
        }
    }
    reloc(m, this, src, this + 0x58);
    reloc(m, this, src, this + 0x5c);
    reloc(m, this, src, this + 0x60);
    m.call(0x0016_ea80, &[this + 0x80, this, src], &[]);
    let extra = vcall(m, this, 0xc, 0x1fc, &[], &[]).0;
    let sz = (m.r32(this + 0xe0).wrapping_add(extra)).wrapping_mul(0xb0);
    let old = m.r32(this + 0xe4);
    let nt = ae(m, ei, sz, 0x10, old, true);
    m.w32(this + 0xe4, nt);
    if (m.r32(this + 0xe0) as i32) > 0 {
        let mut off: u32 = 0;
        let mut i: i32 = 0;
        loop {
            let row = m.r32(this + 0xe4).wrapping_add(off);
            reloc(m, this, src, row);
            m.w32(row + 4, this);
            m.call(0x0011_a210, &[row + 0x60], &[]);
            m.call(0x0016_6270, &[row + 0x90, 0, pose], &[]);
            let q0 = m.r64(pose);
            m.w32(row + 0x90, q0 as u32);
            m.w32(row + 0x94, (q0 >> 32) as u32);
            let q1 = m.r64(pose + 8);
            m.w32(row + 0x98, q1 as u32);
            m.w32(row + 0x9c, (q1 >> 32) as u32);
            let q2 = m.r64(pose + 0x10);
            m.w32(row + 0xa0, q2 as u32);
            m.w32(row + 0xa4, (q2 >> 32) as u32);
            let srow = m.r32(src + 0xe4).wrapping_add(off);
            m.call(0x0011_91e8, &[srow + 0x68, srow + 0x90, row + 0x68, row + 0x90], &[]);
            if m.r32(row + 0x10) != 0 {
                let arr = ae(m, ei, 0xc, 0x10, 0, false);
                m.w32(row + 0x10, arr);
                let old0 = m.r32(arr);
                let sz0 = m.r32(row + 0x78) << 4;
                let n0 = ae(m, ei, sz0, 0x10, old0, true);
                let a = m.r32(row + 0x10);
                m.w32(a, n0);
                let a = m.r32(row + 0x10);
                let old1 = m.r32(a + 4);
                let sz1 = m.r32(row + 0x78) << 4;
                let n1 = ae(m, ei, sz1, 0x10, old1, true);
                let a = m.r32(row + 0x10);
                m.w32(a + 4, n1);
                let a = m.r32(row + 0x10);
                let old2 = m.r32(a + 8);
                let sz2 = m.r32(row + 0x80).wrapping_mul(0xc);
                let n2 = ae(m, ei, sz2, 0x10, old2, true);
                let a = m.r32(row + 0x10);
                m.w32(a + 8, n2);
            }
            i += 1;
            off = off.wrapping_add(0xb0);
            if i >= m.r32(this + 0xe0) as i32 {
                break;
            }
        }
    }
    if m.r32(this + 0xe8) != 0 {
        let new = ae(m, ei, 0xe0, 0x10, 0, false);
        m.w32(this + 0xe8, new);
        if m.r32(new + 0xdc) != 0 {
            let n2 = ae(m, ei, 0x70, 0x10, 0, false);
            let blk = m.r32(this + 0xe8);
            m.w32(blk + 0xdc, n2);
            let p = { let b = m.r32(this + 0xe8); m.r32(b + 0xdc) };
            m.w32(p, this);
        }
    }
    if m.r64(this + 0x28) >> 0x3c & 1 != 0 {
        m.call(0x001f_2b20, &[this], &[]);
    }
    m.call(0x001f_2500, &[this], &[]);
    stack_pop(m, 0x20);
    stack_pop(m, 0x10);
    stack_pop(m, 0x10);
}
