//! C2cabd8 (world class 43): sound-emitter list owner.
use super::{ClassGlobals, ClassHost};

#[derive(Clone, Debug, Default)]
pub struct C2cabd8 {
    /// +0xe4 count / +0xe8 entries (0x14 each), loaded by the ported FUN_0012eb78
    pub f_e8: Vec<[i32; 5]>,
    /// +0xec: e4 handles (0 = none)
    pub f_ec: Vec<i32>,
    /// +0xf0 count / +0xf4 entries (0x10 each)
    pub f_f4: Vec<[i32; 4]>,
    /// K26: in the world at load (no FIELD_REMOVE on the owner chain): slot 22 (AddLo hook) runs after post-load
    pub added: bool,
}

/// slot 5: FUN_00113cc0
pub fn ee_sub_113cc0() -> u32 {
    0x2bae44
}

/// FUN_00201360 numeric view of one array item (kind 1 = float, 5 / 6 = int, 0 = u8, 2-byte kinds = u16)
fn item_num(it: &crate::engine::world::props::ArrayItem) -> f64 {
    use crate::engine::world::props::ArrayItem;
    let ArrayItem::One(k, b) = it else { return 0.0 };
    match (k, b.len()) {
        (1, 4) => f32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f64,
        (_, 4) => i32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f64,
        (_, 2) => u16::from_le_bytes([b[0], b[1]]) as f64,
        (_, 1) => b[0] as f64,
        _ => 0.0,
    }
}

impl C2cabd8 {
    /// K26: the load-time property methods of FUN_0012eb78's FUN_00231170: field 99 (method 0x101768 ->
    /// FUN_0012eee0: +0xe8 entry {sound, (float)near, (float)far, volume, pitch}) and field 100 (method 0x101848
    /// -> FUN_0012ef40: +0xf4 entry {key, (float)near, (float)far, f}); at most 8 of each.
    pub fn feed(&mut self, props: &[crate::engine::world::props::Prop]) {
        use crate::engine::world::props::PropValue;
        for p in props {
            let PropValue::Array(a) = &p.value else { continue };
            let n = |k: usize| a.get(k).map_or(0.0, item_num);
            let f = |k: usize| (n(k) as f32).to_bits() as i32;
            match p.field {
                99 if self.f_e8.len() < 8 => self.f_e8.push([n(0) as i32, f(1), f(2), f(3), f(4)]),
                100 if self.f_f4.len() < 8 => self.f_f4.push([n(0) as i32, f(1), f(2), f(3)]),
                _ => {}
            }
        }
    }
    /// slot 6: FUN_0012eb48
    pub fn ee_sub_12eb48(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x231100, obj);
        h.ee_sub_1a6ca8(obj, 0);
    }
    /// slot 14: FUN_0012ec50
    pub fn ee_sub_12ec50(&mut self, obj: usize, g: &ClassGlobals, h: &mut dyn ClassHost) {
        h.base(0x1a5c48, obj);
        self.f_ec = vec![0; self.f_e8.len()];
        if g.g_3dd268 != 0 {
            for e in &self.f_f4 {
                h.emitter_attach(obj, *e, g.g_3dd268);
            }
        }
    }
    /// slot 21: FUN_0012ede8
    pub fn ee_sub_12ede8(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a6150, obj);
        for k in 0..self.f_ec.len() {
            if self.f_ec[k] != 0 {
                h.emitter_release(obj, k);
                self.f_ec[k] = 0; // FUN_00258040(slot, 0)
            }
        }
    }
    /// slot 22: FUN_0012ed20
    pub fn ee_sub_12ed20(&mut self, obj: usize, h: &mut dyn ClassHost) {
        h.base(0x1a6178, obj);
        self.f_ec.resize(self.f_e8.len(), 0);
        for k in 0..self.f_e8.len() {
            self.f_ec[k] = h.emitter_create(obj, k, self.f_e8[k]);
        }
    }
}
