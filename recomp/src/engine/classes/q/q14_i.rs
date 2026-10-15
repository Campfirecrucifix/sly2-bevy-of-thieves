//! WP-Q14 part I: FUN_00196aa8 (obstacle-plane constrained move: collect up to 0x40 projected contact planes of
//! the object's contact lists, solve the inequalities with SolveInequalities FUN_0011ad30, move the point).
//! The scratch pool FUN_001ae478 / 001ae318 / 001ae4a0 is replaced by the QMem stack.
#![allow(unused_variables, unused_mut, dead_code)]
use crate::engine::classes::q::q2mem::{stack_pop, stack_push, QMem, V4};

fn dot3(a: V4, b: V4) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// FUN_00196aa8(obj, q): `q` points at a quadword that is moved to the constrained position
pub fn ee_sub_196aa8<M: QMem + ?Sized>(m: &mut M, obj: u32, q: u32) {
    let up = m.rv(0x002a_2dc0);
    let mut planes: Vec<V4> = Vec::new();
    let o0 = m.r32(obj);
    let o1 = m.r32(o0 + 0xec);
    let head = m.r32(o1 + 100);
    let mut list = head;
    'outer: loop {
        let mut item = if list == 0 { m.r32(o1 + 0x68) } else { m.r32(list) };
        while item != 0 {
            if planes.len() > 0x3f {
                break 'outer;
            }
            let mut v = m.rv(item + 0x80);
            if m.r32(obj) != m.r32(item + 8) {
                v = [v[0] * -1.0, v[1] * -1.0, v[2] * -1.0, v[3] * -1.0];
            }
            if !(0.7f32 < v[2]) {
                let d = dot3(v, up);
                planes.push([v[0] - up[0] * d, v[1] - up[1] * d, v[2] - up[2] * d, v[3] - up[3] * d]);
            }
            item = m.r32(item + 0xac);
        }
        if list == 0 {
            break;
        }
        list = m.r32(list + 4);
    }
    let n = planes.len() as u32;
    if n == 0 {
        return;
    }
    let sz = n * n * 4 + 2 * n * 4 + n * 4 + 0x30;
    let sp = stack_push(m, (sz + 15) & !15);
    let (a_m, b_v, x_v) = (sp, sp + ((n * n * 4 + 15) & !15), sp + ((n * n * 4 + 15) & !15) + ((n * 4 + 15) & !15));
    for i in 0..n {
        for j in 0..n {
            let d = dot3(planes[j as usize], planes[i as usize]);
            m.wf(a_m + (i * n + j) * 4, d);
        }
    }
    let q0 = m.rv(q);
    for j in 0..n {
        let d = dot3(q0, planes[j as usize]);
        m.wf(b_v + j * 4, d);
    }
    let eps = m.rf(0x002a_63b0);
    m.call(0x0011_ad30, &[n, a_m, b_v, x_v], &[[eps, 0.0, 0.0, 0.0]]);
    let mut c = q0;
    for i in 0..n {
        let s = m.rf(x_v + i * 4);
        let p = planes[i as usize];
        c = [c[0] + p[0] * s, c[1] + p[1] * s, c[2] + p[2] * s, c[3] + 1.0 + p[3] * s];
    }
    let d = [q0[0] - c[0], q0[1] - c[1], q0[2] - c[2], q0[3] - c[3]];
    if 1.0f32 < dot3(d, d) {
        let st = m.r32(obj + 0x1c);
        if st == 0 {
            m.call(0x0019_5fb8, &[obj, 1, 0], &[]);
        } else if st == 2 {
            m.call(0x0019_5fb8, &[obj, 3, 0], &[]);
        }
    }
    m.wv(q, c);
    stack_pop(m, (sz + 15) & !15);
}
