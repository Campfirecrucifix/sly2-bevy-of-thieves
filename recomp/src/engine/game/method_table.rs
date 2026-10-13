//! WP-PUB2 (was the WP-J4/J9 generated `methods_gen.rs`): the Splice method binding table, read from
//! SCUS_973.16 on first use. Field table 0x002ac880 kind-0x12 rows (METHODS) and other rows with
//! flags & 1 (GETTERS) + per-class symid tables 0x00279ad0 (BINDINGS / GETTER_BINDINGS, FUN_001ffcf8
//! order). The thunk's FUN_00201360 kind array, engine callee and result setter come from its code
//! (`exe_scan::scan_thunk`).

use super::fieldbind::class_symid_rows;
use super::methods::{GetterDesc, GetterRet, MethodBinding, MethodDesc};
use crate::engine::disc::exe::exe;
use crate::engine::disc::exe_scan::scan_thunk;
use crate::engine::world::field_table::{field_index, field_word, FIELD_COUNT};
use std::sync::LazyLock;

pub const METHOD_COUNT: usize = 383;
pub const BINDING_COUNT: usize = 4079;
pub const GETTER_COUNT: usize = 47;
pub const GETTER_BINDING_COUNT: usize = 891;

/// Thunk helpers (CRef construction / argument unpacking) that are not the thunk's engine callee.
const HELPERS: [u32; 17] = [
    0x201360, 0x129ec8, 0x12a0b0, 0x1291a8, 0x129658, 0x129598, 0x129518, 0x129558, 0x1296a8, 0x129718, 0x129a10, 0x129808, 0x129878,
    0x1298f0, 0x129da8, 0x129d58, 0x129a70,
];
/// CRef setters a getter thunk wraps its result with (first match in this order).
const RET_SETTERS: [(u32, GetterRet); 10] = [
    (0x129598, GetterRet::Bool),
    (0x129558, GetterRet::F32),
    (0x129518, GetterRet::S32),
    (0x129a10, GetterRet::Basic),
    (0x1296a8, GetterRet::Vector),
    (0x129808, GetterRet::Matrix),
    (0x129878, GetterRet::Clq),
    (0x1298f0, GetterRet::Lm),
    (0x129718, GetterRet::F32x2),
    (0x129658, GetterRet::Ref),
];

fn desc(i: usize) -> (MethodDesc, GetterRet) {
    let flags = field_word(i, 1);
    let thunk = if flags & 1 != 0 { field_word(i, 4) } else { field_word(i, 2) };
    let n = field_word(i, 6);
    let mut m = MethodDesc { field: i as u16, flags, thunk, nargs: if n < 0x100 { n as u8 } else { 0 }, kinds: None, callee: 0 };
    let mut ret = GetterRet::Unknown;
    if thunk != 0 {
        let s = scan_thunk(exe(), thunk);
        m.kinds = s.kinds.map(|k| &*Box::leak(k.into_boxed_slice()));
        m.callee = s.calls.iter().copied().find(|c| !HELPERS.contains(c)).unwrap_or(0);
        if !s.void_ret {
            ret = RET_SETTERS.iter().find(|(a, _)| s.calls.contains(a)).map_or(GetterRet::Unknown, |r| r.1);
        }
    }
    (m, ret)
}

struct Tables {
    methods: Vec<MethodDesc>,
    bindings: Vec<MethodBinding>,
    getters: Vec<GetterDesc>,
    getter_bindings: Vec<MethodBinding>,
}

static TABLES: LazyLock<Tables> = LazyLock::new(|| {
    let mut t = Tables { methods: Vec::new(), bindings: Vec::new(), getters: Vec::new(), getter_bindings: Vec::new() };
    // field index -> position in methods / getters
    let mut pos = vec![None; FIELD_COUNT];
    for i in 0..FIELD_COUNT {
        let kind = field_word(i, 0);
        if kind == 0x12 {
            pos[i] = Some(t.methods.len() as u16);
            t.methods.push(desc(i).0);
        } else if field_word(i, 1) & 1 != 0 {
            pos[i] = Some(t.getters.len() as u16);
            let (m, ret) = desc(i);
            t.getters.push(GetterDesc { kind, ret, m });
        }
    }
    let x = exe();
    for (class, symid, f) in class_symid_rows() {
        if f == 0 {
            continue;
        }
        let b = |i: usize| MethodBinding { class, symid: symid as u16, method: pos[i].expect("bound row is a method/getter") };
        if x.w(f) == 0x12 {
            t.bindings.push(b(field_index(f).expect("method field")));
        } else if x.w(f + 4) & 1 != 0 {
            t.getter_bindings.push(b(field_index(f).expect("getter field")));
        }
    }
    assert_eq!(
        (t.methods.len(), t.bindings.len(), t.getters.len(), t.getter_bindings.len()),
        (METHOD_COUNT, BINDING_COUNT, GETTER_COUNT, GETTER_BINDING_COUNT),
        "SCUS_973.16 method tables"
    );
    t
});

pub static METHODS: LazyLock<&'static [MethodDesc]> = LazyLock::new(|| &TABLES.methods);
/// (class id, symid, index into METHODS), sorted by class then symid (FUN_001ffcf8 order).
pub static BINDINGS: LazyLock<&'static [MethodBinding]> = LazyLock::new(|| &TABLES.bindings);
/// WP-J9: rows of other kinds with flags & 1 (get-o CMethod getters) + the CRef setter of the result.
pub static GETTERS: LazyLock<&'static [GetterDesc]> = LazyLock::new(|| &TABLES.getters);
/// (class id, symid, index into GETTERS), FUN_001ffcf8 order.
pub static GETTER_BINDINGS: LazyLock<&'static [MethodBinding]> = LazyLock::new(|| &TABLES.getter_bindings);
