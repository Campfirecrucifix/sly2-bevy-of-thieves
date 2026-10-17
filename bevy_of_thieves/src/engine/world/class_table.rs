//! WP-PUB2: class descriptor table 0x002baae8 (199 x 0x14: parent*, id, id_end, size, ctor), read from
//! SCUS_973.16 on first use. vtable = the last `+0xc` vtable store of the ctor (`exe_scan::ctor_vtable`),
//! load = vtable word +0x74, name = `C<vtable>` placeholder ("?" without vtable).

use super::class::ClassInfo;
use crate::engine::disc::exe::exe;
use crate::engine::disc::exe_scan::ctor_vtable;
use std::sync::LazyLock;

pub const CLASS_TABLE: u32 = 0x002b_aae8;
pub const CLASS_STRIDE: u32 = 0x14;
pub const CLASS_COUNT: usize = 199;

pub static CLASSES: LazyLock<Vec<ClassInfo>> = LazyLock::new(|| {
    let x = exe();
    (0..CLASS_COUNT as u32)
        .map(|i| {
            let a = CLASS_TABLE + i * CLASS_STRIDE;
            let (p, id, end, size, ctor) = (x.w(a), x.w(a + 4), x.w(a + 8), x.w(a + 0xc), x.w(a + 0x10));
            let vtable = if ctor != 0 { ctor_vtable(x, ctor) } else { 0 };
            ClassInfo {
                id: id as u16,
                parent: if p != 0 { ((p - CLASS_TABLE) / CLASS_STRIDE) as i16 } else { -1 },
                id_end: end as u16,
                size,
                ctor,
                vtable,
                load: if vtable != 0 { x.w(vtable + 0x74) } else { 0 },
                name: if vtable != 0 { Box::leak(format!("C{vtable:x}").into_boxed_str()) } else { "?" },
            }
        })
        .collect()
});
