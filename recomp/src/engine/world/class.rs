//! Class registry: descriptor table 0x002baae8 (199 entries x 0x14 bytes:
//! +0 parent descriptor*, +4 class id, +8 end of the id subtree, +0xc object size, +0x10 ctor).
//! Class ids are assigned in DFS order, so `[id, id_end)` is the subtree of `id`.
//! The vtable (set by the ctor) and its +0x74 slot (the stream `load` virtual) are read from the ctor
//! code / vtable in SCUS_973.16 (`class_table.rs`); names are placeholders `C<vtable>`.

pub use super::class_table::{CLASSES, CLASS_COUNT};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClassInfo {
    pub id: u16,
    /// -1 = root
    pub parent: i16,
    pub id_end: u16,
    pub size: u32,
    /// 0 = abstract (no ctor)
    pub ctor: u32,
    pub vtable: u32,
    /// vtable +0x74 (called as `(*(vt+0x74))(obj + *(s16*)(vt+0x70), stream)`)
    pub load: u32,
    pub name: &'static str,
}

/// Class id as stored in the level stream (s16).
pub type ClassId = u16;

/// LO load virtual (FUN_001a5c28 = FUN_0013f810 property list only). Class 2 is the root of
/// all streamable objects (every class with a ctor in 3..134 derives from it).
pub const LOAD_LO: u32 = 0x001a5c28;
/// FUN_00231170: FUN_00231768 (s16, 4x3 matrix, u32) + property list.
pub const LOAD_231170: u32 = 0x00231170;
/// Class 2: the LO base class (load = [`LOAD_LO`]).
pub const CID_LO: ClassId = 2;
/// Class 6: objects of this subtree are not instantiated by FUN_0013fb40 but handed to
/// FUN_001c9b08 (see objects.rs).
pub const CID_6: ClassId = 6;

pub fn class(id: ClassId) -> Option<&'static ClassInfo> {
    CLASSES.get(id as usize)
}

/// FUN_0015e988: walk the parent chain of `id` looking for `base`.
pub fn ee_sub_15e988(id: ClassId, base: ClassId) -> bool {
    // 0x0015e988
    let mut c = class(id);
    while let Some(ci) = c {
        if ci.id == base {
            return true;
        }
        c = if ci.parent < 0 { None } else { class(ci.parent as u16) };
    }
    false
}
