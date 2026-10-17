//! xf actuator records (the +0x90 priority list of an xf block, e.g. ALO +0x5c xfAng). Records are kept as
//! zero-filled 32-bit words like the original pool blocks: w[0] kind, w[1] owner, w[2] xf, w[3] (+0xc)
//! priority, w[4] (+0x10) next (REPLACED: list order of `XfActs::list`), w[5] (+0x14), w[6] (+0x18) vtable.
//! Virtual calls on records (+0xc activate, +0x14 deactivate, +0x1c re-sort notify) are `ClassHost::act_vcall`.
use crate::engine::classes::ClassHost;

/// One pool record (`id` = REPLACED pool address)
#[derive(Clone, Debug, PartialEq)]
pub struct Act {
    pub id: u32,
    pub w: Vec<u32>,
}
impl Act {
    pub fn kind(&self) -> u32 {
        self.w[0]
    }
    pub fn prio(&self) -> i32 {
        self.w[3] as i32
    }
    pub fn vt(&self) -> u32 {
        self.w[6]
    }
}

/// xf block fields touched by the act code: +0x80 / +0x84 / +0x88 / +0x8c, +0x98 kind, +0x9a, +0x90 list
#[derive(Clone, Debug, PartialEq, Default)]
pub struct XfActs {
    /// xf handle (owner handle * 0x100 + offset, for the hooks)
    pub xf: u32,
    pub f_80: f32,
    pub f_84: f32,
    pub f_88: u32,
    pub f_8c: u32,
    pub kind_98: u16,
    pub f_9a: u16,
    pub list: Vec<Act>,
}

/// REPLACED pool 0x3e123c (FUN_0011ccc0): record serial / live count (+0x10)
#[derive(Clone, Debug, Default)]
pub struct ActPool {
    pub next: u32,
    pub live: i32,
}

/// FUN_0012e238(kind): pool record of `kind` with its vtable and per-kind defaults (None = unknown kind)
pub fn ee_sub_12e238(pool: &mut ActPool, kind: u32) -> Option<Act> {
    let (size, vt) = match kind {
        0 => (0x2c, 0x2caf08),
        1 => (0x30, 0x2ca820),
        2 => (0x80, 0x2ca7e0),
        3 => (0x20, 0x2db338),
        4 => (0x28, 0x2ca7a0),
        5 => (0x30, 0x2ca760),
        6 => (0x60, 0x2ca720),
        7 => (0x20, 0x2d2e58),
        8 => (0x40, 0x2ca6e0),
        9 => (0x30, 0x2ca6a0),
        _ => return None,
    };
    // FUN_0012e1f0 -> FUN_0011ccc0(0x3e123c): zeroed block, live count ++
    pool.next += 1;
    pool.live += 1;
    let mut w = vec![0u32; size / 4];
    w[0] = kind;
    w[6] = vt;
    match kind {
        1 => w[7] = 2,
        6 => {
            // FUN_002333b4(+0x20, 0, 0x40), then the 4x4 identity at +0x20
            for k in [8, 13, 18, 23] {
                w[k] = 1.0f32.to_bits();
            }
        }
        9 => w[5] = 0x14,
        _ => {}
    }
    Some(Act { id: pool.next, w })
}

/// FUN_0012db50(kind, owner, xf, prio, f_14)
pub fn ee_sub_12db50(pool: &mut ActPool, kind: u32, owner: u32, xf: u32, prio: i32, f_14: u32) -> Option<Act> {
    // the original writes through the result without a null check (kind is always valid at the call sites)
    let mut a = ee_sub_12e238(pool, kind)?;
    a.w[1] = owner;
    a.w[2] = xf;
    a.w[3] = prio as u32;
    a.w[5] = f_14;
    Some(a)
}

/// FUN_0012dca0(f_1c, f_20, owner, xf, prio, f_14): kind-4 record, inserted into xf +0x90; returns its id
pub fn ee_sub_12dca0(pool: &mut ActPool, f_1c: u32, f_20: u32, owner: u32, xf: &mut XfActs, prio: i32, f_14: u32, h: &mut dyn ClassHost) -> u32 {
    let Some(mut a) = ee_sub_12db50(pool, 4, owner, xf.xf, prio, f_14) else { return 0 };
    a.w[7] = f_1c;
    a.w[8] = f_20;
    let id = a.id;
    ee_sub_230018(owner, xf, a, h);
    id
}

/// FUN_00230018(owner, xf, rec): insert before the first record with priority <= rec's; unchanged head ->
/// head +0x1c; new head -> FUN_0022c840, old head +0x14, new head +0xc
pub fn ee_sub_230018(owner: u32, xf: &mut XfActs, rec: Act, h: &mut dyn ClassHost) {
    let old = xf.list.first().map(|a| (a.id, a.vt()));
    let at = xf.list.iter().position(|n| n.prio() <= rec.prio()).unwrap_or(xf.list.len());
    xf.list.insert(at, rec);
    let head = (xf.list[0].id, xf.list[0].vt());
    if Some(head) == old {
        let (id, vt) = head;
        h.act_vcall(vt, 0x1c, xf.xf, id);
    } else {
        ee_sub_22c840(owner, xf, h);
        if let Some((id, vt)) = old {
            h.act_vcall(vt, 0x14, xf.xf, id);
        }
        let (id, vt) = (xf.list[0].id, xf.list[0].vt());
        h.act_vcall(vt, 0xc, xf.xf, id);
    }
}

/// FUN_0022c840(owner, xf): xf kinds other than 0-3 / 0xd-0x10 / 0x14 / 0x15 get +0x80 / +0x84 = -FLT_MAX
/// (DAT_002a98d0) and +0x88 / +0x8c = &DAT_002c44e0; +0x9a = 0; owner vtable +0x17c(4)
pub fn ee_sub_22c840(owner: u32, xf: &mut XfActs, h: &mut dyn ClassHost) {
    match xf.kind_98 {
        0..=3 | 0xd..=0x10 | 0x14 | 0x15 => {}
        _ => {
            xf.f_88 = 0x2c44e0;
            xf.f_80 = -f32::MAX;
            xf.f_84 = -f32::MAX;
            xf.f_8c = 0x2c44e0;
        }
    }
    xf.f_9a = 0;
    h.vcall_arg(owner, 0x17c, 4);
}
