//! BOSS: the world region tree (world +0x45cc, read by FUN_001c0ec0 / FUN_001c0f30 into
//! `world::entry::Block1c0f30`; DAT_003e1a5c points at it) and its point query FUN_001c0d70.
//! Tree layout: +0 node count, +4 nodes (8 bytes: plane pointer = +0xc + idx * 16, u16 child[2]), +8 / +0xc
//! planes (16 bytes: x y z w), +0x10 / +0x14 region oids (resolved to region objects at +0x18 by FUN_001c1138).
//! A child with bit 15 is a leaf (region index & 0x7fff); a child equal to the node count fell off the tree.
//! REPLACED: the region object pointer is its oid (+0x18 entry k = FUN_0015e938(world, 5, oid k)); the
//! default region DAT_003e1a58 (the class 0x2ca object FUN_001c0320 records) = `DEFAULT`.
use crate::engine::world::entry::Block1c0f30;

/// DAT_003e1a58 stand-in (fell off the tree, a1 != 0 or no regions)
pub const DEFAULT: i32 = -1;

#[derive(Clone, Debug, Default)]
pub struct Tree {
    pub nodes: Vec<[u16; 3]>,
    pub planes: Vec<[f32; 4]>,
    pub regions: Vec<i32>,
}

impl Tree {
    /// FUN_001c0ec0 data of the level (None = no tree, DAT_003e1a5c == 0)
    pub fn from_block(b: &Block1c0f30) -> Tree {
        let planes = b.vecs.iter().map(|v| [0, 1, 2, 3].map(|i| f32::from_le_bytes([v[i * 4], v[i * 4 + 1], v[i * 4 + 2], v[i * 4 + 3]]))).collect();
        Tree { nodes: b.items.clone(), planes, regions: b.words.clone() }
    }

    /// FUN_001c0d70(pt, a1): region oid of `pt`; fell off -> DEFAULT when a1 != 0 or no regions, else 0
    pub fn ee_sub_1c0d70(&self, pt: [f32; 3], a1: bool) -> i32 {
        let n = self.nodes.len();
        let mut i = 0usize;
        while i < n {
            let nd = self.nodes[i];
            let p = self.planes.get(nd[0] as usize).copied().unwrap_or([0.0; 4]);
            let d = p[0] * pt[0] + p[1] * pt[1] + p[2] * pt[2] - p[3];
            let c = nd[1 + (0.0 < d) as usize];
            if c & 0x8000 != 0 {
                return self.regions.get((c & 0x7fff) as usize).copied().unwrap_or(0);
            }
            i = c as usize;
        }
        if a1 || self.regions.is_empty() {
            DEFAULT
        } else {
            0
        }
    }
}
