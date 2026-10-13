//! CFrame: environment frame. Original layout `+8 parent, +0x1c binding list (CBinding: symid +8,
//! CRef +0xc, next +0x14)`; bindings are prepended, lookup walks newest-first then the parent chain.

use std::cell::RefCell;
use std::rc::Rc;

use super::value::Ref;

#[derive(Debug, Default)]
pub struct Frame {
    pub parent: Option<FrameRef>,
    /// Newest binding last (iterated in reverse = original newest-first list).
    pub bindings: Vec<(u32, Ref)>,
}
pub type FrameRef = Rc<RefCell<Frame>>;

pub fn new_frame(parent: Option<FrameRef>) -> FrameRef {
    Rc::new(RefCell::new(Frame { parent, bindings: Vec::new() }))
}

/// 0x00127b18: CFrame::SetParent.
pub fn set_parent(frame: &FrameRef, parent: Option<FrameRef>) {
    frame.borrow_mut().parent = parent;
}

/// 0x00127c00: CFrame::AddBinding (define). Always prepends, shadowing older bindings. Returns void.
pub fn add_binding(frame: &FrameRef, symid: u32, value: Ref) -> Ref {
    frame.borrow_mut().bindings.push((symid, value));
    Ref::Void
}

/// 0x00127db8 / 0x00127d78: binding lookup, optionally recursive through parents.
pub fn lookup(frame: &FrameRef, symid: u32, recurse: bool) -> Option<Ref> {
    let mut cur = Some(frame.clone());
    while let Some(f) = cur {
        let fb = f.borrow();
        if let Some((_, r)) = fb.bindings.iter().rev().find(|(s, _)| *s == symid) {
            return Some(r.clone());
        }
        if !recurse {
            return None;
        }
        cur = fb.parent.clone();
    }
    None
}

/// 0x00127cf0: set! — assigns the nearest existing binding; silently does nothing when unbound
/// (original quirk). Returns void.
pub fn set_binding(frame: &FrameRef, symid: u32, value: Ref) -> Ref {
    let mut cur = Some(frame.clone());
    while let Some(f) = cur {
        let mut fb = f.borrow_mut();
        if let Some(slot) = fb.bindings.iter_mut().rev().find(|(s, _)| *s == symid) {
            slot.1 = value;
            return Ref::Void;
        }
        cur = fb.parent.clone();
    }
    Ref::Void
}
