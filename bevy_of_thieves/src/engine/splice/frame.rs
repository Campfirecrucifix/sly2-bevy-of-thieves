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
    /// CRAWL R116: further parents (original +8 = parent COUNT, +0xc = parent array, FUN_00127b40 appends):
    /// FUN_001c9b08 override rows append a bound object's old frame to the slot frame they bind. Searched after
    /// `parent`, depth-first in order (FindBinding 0x00127db8, +0x20 = visiting guard).
    pub extra_parents: Vec<FrameRef>,
    /// CRAWL R116: CFrame +0x24 - an object's script-slot frame (BIF 133 `super` starts at its first parent)
    pub script_slot: bool,
}
pub type FrameRef = Rc<RefCell<Frame>>;

pub fn new_frame(parent: Option<FrameRef>) -> FrameRef {
    Rc::new(RefCell::new(Frame { parent, bindings: Vec::new(), extra_parents: Vec::new(), script_slot: false }))
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
    if recurse && has_extra(frame) {
        let mut seen = Vec::new();
        return find_dfs(frame, symid, &mut seen).map(|(f, i)| f.borrow().bindings[i].1.clone());
    }
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
    if has_extra(frame) {
        if let Some((f, i)) = find_dfs(frame, symid, &mut Vec::new()) {
            f.borrow_mut().bindings[i].1 = value;
        }
        return Ref::Void;
    }
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

/// CRAWL R116: FUN_00127b40 - append `parent` to the frame's parent list (no duplicate).
pub fn add_parent(frame: &FrameRef, parent: &FrameRef) {
    if Rc::ptr_eq(frame, parent) {
        return;
    }
    let mut fb = frame.borrow_mut();
    if fb.parent.as_ref().is_some_and(|p| Rc::ptr_eq(p, parent)) || fb.extra_parents.iter().any(|p| Rc::ptr_eq(p, parent)) {
        return;
    }
    if fb.parent.is_none() {
        fb.parent = Some(parent.clone());
    } else {
        fb.extra_parents.push(parent.clone());
    }
}

/// any frame on the plain parent chain with extra parents (else the single-chain walk is exact)
fn has_extra(frame: &FrameRef) -> bool {
    let mut cur = Some(frame.clone());
    while let Some(f) = cur {
        let fb = f.borrow();
        if !fb.extra_parents.is_empty() {
            return true;
        }
        cur = fb.parent.clone();
    }
    false
}

/// FindBinding 0x00127db8 with the parent list: own bindings, then each parent depth-first (skip visiting)
fn find_dfs(frame: &FrameRef, symid: u32, seen: &mut Vec<*const RefCell<Frame>>) -> Option<(FrameRef, usize)> {
    let p = Rc::as_ptr(frame);
    if seen.contains(&p) {
        return None;
    }
    let (hit, parents) = {
        let fb = frame.borrow();
        let hit = fb.bindings.iter().rposition(|(s, _)| *s == symid);
        (hit, fb.parent.iter().chain(fb.extra_parents.iter()).cloned().collect::<Vec<_>>())
    };
    if let Some(i) = hit {
        return Some((frame.clone(), i));
    }
    seen.push(p);
    let r = parents.iter().find_map(|q| find_dfs(q, symid, seen));
    seen.pop();
    r
}
