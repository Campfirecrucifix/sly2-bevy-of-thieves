//! WP-K22 class-set plumbing (classes/k22_*.rs instances live in `ClassSet::k22c`).
use super::{ClassHost, ClassSet, Inst};
use crate::engine::world::World;

impl ClassSet {
    pub(super) fn build_k22(&mut self, w: &World, h: &mut dyn ClassHost) {
        self.build_k22_96(w, h);
        self.build_k22_115(w, h);
        self.build_k22_69(w, h);
        self.build_k22_121(w, h);
    }
    pub(super) fn post_load_k22(&mut self, h: &mut dyn ClassHost) {
        self.post_load_k22_96(h);
        self.post_load_k22_69(h);
        self.post_load_k22_121(h);
    }
    pub(super) fn frame_k22(&mut self, h: &mut dyn ClassHost) {
        self.frame_k22_96(h);
        self.frame_k22_115(h);
        self.frame_k22_69(h);
        self.frame_k22_121(h);
        // sequence action 7 (classes/k22_seq.rs): slot 78 through the player API
        for po in h.k22_take_po_switches() {
            if let Some(o) = (po as usize).checked_sub(1) {
                self.k18_set_active(Some(o), h);
            }
        }
    }
    /// the K22 instance of object `obj`
    pub fn k22c(&mut self, obj: usize) -> Option<&mut Inst> {
        self.k22c.iter_mut().find(|x| x.0 == obj).map(|x| &mut x.1)
    }
}
