//! WP-K21 group 3: the ClassSet wiring of the classes 28, 68 and 49 (171, 96, 115, 131, 69, 121, 87 unported)
//! (classes/k21_<class>.rs).
use super::*;
use crate::engine::world::World;

impl ClassSet {
    pub(super) fn build_k21(&mut self, w: &World, h: &mut dyn ClassHost) {
        self.build_k21_28(w, h);
        self.build_k21_68(w, h);
        self.build_k21_49(w, h);
    }
    pub(super) fn post_load_k21(&mut self, h: &mut dyn ClassHost) {
        self.post_load_k21_68(h);
        self.post_load_k21_49(h);
    }
    pub(super) fn frame_k21(&mut self, h: &mut dyn ClassHost) {
        self.frame_k21_28(h);
        self.frame_k21_68(h);
        self.frame_k21_49(h);
    }
    /// the K21 instance of object `obj`
    pub fn k21c(&mut self, obj: usize) -> Option<&mut Inst> {
        self.k21c.iter_mut().find(|x| x.0 == obj).map(|x| &mut x.1)
    }
}
