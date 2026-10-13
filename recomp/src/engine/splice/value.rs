//! CRef value model (Sly 2 tag numbering, recovered from SCUS_973.16).
//!
//! Sly 2 TAGK values (differ from Sly 1!): derived from the type predicates 0x00120820..0x00120a20,
//! the serializer 0x0012a3d8 and CRef::SetTag 0x0012a0b0.
//! -1 Nil, 0 None (empty list), 1..=0x14 special-form keywords, 0x15 S32, 0x16 F32, 0x17 Bool,
//! 0x18 Symid, 0x19 Bifk, 0x1a Vector, 0x1b Matrix, 0x1c Clq, 0x1d Lm, 0x1e Smp, 0x1f Pair,
//! 0x20 Proc, 0x21 Basic (object), 0x22 Method, 0x23 Err, 0x24 Void.
//!
//! The original refcounts heap cells (pairs, procs, frames, vec/mat blocks) and runs a mark/sweep
//! pass (0x001281b0) for cycles. Replaced by Rc (cycles through frames leak only on VM drop of
//! self-referential closures, which `Vm::reset` breaks explicitly).

use std::cell::RefCell;
use std::rc::Rc;

use super::frame::FrameRef;

pub const TAGK_NIL: i32 = -1;
pub const TAGK_NONE: i32 = 0;
pub const TAGK_SET: i32 = 1;
pub const TAGK_DEFINE: i32 = 2;
pub const TAGK_DEFINE_MACRO: i32 = 3;
pub const TAGK_DEFINE_MACRO_LAMBDA: i32 = 4;
pub const TAGK_ASSERT: i32 = 5;
/// 0x001266c8: Sly 2-only form (evaluates its single argument, void when absent).
pub const TAGK_FORM6: i32 = 6;
pub const TAGK_IF: i32 = 7;
pub const TAGK_OR: i32 = 8;
pub const TAGK_AND: i32 = 9;
pub const TAGK_COND: i32 = 10;
pub const TAGK_ELSE: i32 = 11;
pub const TAGK_CASE: i32 = 12;
pub const TAGK_LET: i32 = 13;
pub const TAGK_WHILE: i32 = 14;
pub const TAGK_LAMBDA: i32 = 15;
pub const TAGK_BEGIN: i32 = 16;
pub const TAGK_IMPORT: i32 = 17;
pub const TAGK_QUOTE: i32 = 18;
pub const TAGK_PIPE: i32 = 19;
pub const TAGK_DOT: i32 = 20;
pub const TAGK_S32: i32 = 0x15;
pub const TAGK_F32: i32 = 0x16;
pub const TAGK_BOOL: i32 = 0x17;
pub const TAGK_SYMID: i32 = 0x18;
pub const TAGK_BIFK: i32 = 0x19;
pub const TAGK_VECTOR: i32 = 0x1a;
pub const TAGK_MATRIX: i32 = 0x1b;
pub const TAGK_CLQ: i32 = 0x1c;
pub const TAGK_LM: i32 = 0x1d;
pub const TAGK_SMP: i32 = 0x1e;
pub const TAGK_PAIR: i32 = 0x1f;
pub const TAGK_PROC: i32 = 0x20;
pub const TAGK_BASIC: i32 = 0x21;
pub const TAGK_METHOD: i32 = 0x22;
pub const TAGK_ERR: i32 = 0x23;
pub const TAGK_VOID: i32 = 0x24;

/// Keyword names used by the text reader/printer (the binary format only carries the tag).
pub const FORM_NAMES: [(&str, i32); 20] = [
    ("set!", TAGK_SET),
    ("define", TAGK_DEFINE),
    ("define-macro", TAGK_DEFINE_MACRO),
    ("define-macro-lambda", TAGK_DEFINE_MACRO_LAMBDA),
    ("assert", TAGK_ASSERT),
    ("form6", TAGK_FORM6),
    ("if", TAGK_IF),
    ("or", TAGK_OR),
    ("and", TAGK_AND),
    ("cond", TAGK_COND),
    ("else", TAGK_ELSE),
    ("case", TAGK_CASE),
    ("let", TAGK_LET),
    ("while", TAGK_WHILE),
    ("lambda", TAGK_LAMBDA),
    ("begin", TAGK_BEGIN),
    ("import", TAGK_IMPORT),
    ("quote", TAGK_QUOTE),
    ("pipe", TAGK_PIPE),
    (".", TAGK_DOT),
];

pub type Vector = [f32; 3];
pub type Matrix = [f32; 16];
/// CLQ: g0 + g1*x + g2*x^2.
pub type Clq = [f32; 3];
/// LM: {gMin, gMax}.
pub type Lm = [f32; 2];
/// SMP: {svFast, svSlow, dtFast}.
pub type Smp = [f32; 3];

/// CPair: `{refcount, pad, CRef m_ref (+8), CPair* m_ppairNext (+0x10)}`.
#[derive(Debug)]
pub struct Pair {
    pub car: Ref,
    pub next: Option<PairRef>,
}
pub type PairRef = Rc<RefCell<Pair>>;

/// CProc (lambda, 0x00127068): `+8 frame, +0xc nargs, +0x10 fRest, +0x14 params, +0x18 body`.
#[derive(Debug)]
pub struct Proc {
    pub frame: FrameRef,
    pub nargs: usize,
    pub rest: bool,
    pub params: Option<PairRef>,
    pub body: Option<PairRef>,
}

/// CMethod: a native method bound to an object (`+8 context, +0xc fn, +0x10 nargs`).
/// The function pointer is replaced by a host-side method id.
#[derive(Debug, Clone, PartialEq)]
pub struct Method {
    pub object: u32,
    pub method_id: u32,
    pub nargs: usize,
}

#[derive(Debug, Clone)]
pub enum Ref {
    Nil,
    /// Tags 0..=0x14 carry no payload (0 = empty list, others = keywords).
    Tag(i32),
    S32(i32),
    F32(f32),
    Bool(bool),
    Symid(u32),
    Bifk(u32),
    Vector(Vector),
    Matrix(Matrix),
    Clq(Clq),
    Lm(Lm),
    Smp(Smp),
    Pair(PairRef),
    Proc(Rc<Proc>),
    /// Game object handle (BASIC*); 0 = null object.
    Basic(u32),
    Method(Rc<Method>),
    Err,
    Void,
}

impl Ref {
    pub fn empty() -> Ref {
        Ref::Tag(TAGK_NONE)
    }

    pub fn tagk(&self) -> i32 {
        match self {
            Ref::Nil => TAGK_NIL,
            Ref::Tag(t) => *t,
            Ref::S32(_) => TAGK_S32,
            Ref::F32(_) => TAGK_F32,
            Ref::Bool(_) => TAGK_BOOL,
            Ref::Symid(_) => TAGK_SYMID,
            Ref::Bifk(_) => TAGK_BIFK,
            Ref::Vector(_) => TAGK_VECTOR,
            Ref::Matrix(_) => TAGK_MATRIX,
            Ref::Clq(_) => TAGK_CLQ,
            Ref::Lm(_) => TAGK_LM,
            Ref::Smp(_) => TAGK_SMP,
            Ref::Pair(_) => TAGK_PAIR,
            Ref::Proc(_) => TAGK_PROC,
            Ref::Basic(_) => TAGK_BASIC,
            Ref::Method(_) => TAGK_METHOD,
            Ref::Err => TAGK_ERR,
            Ref::Void => TAGK_VOID,
        }
    }

    /// The 32-bit payload word (`CRef+4`). Heap kinds return a non-null, non-1 stand-in for
    /// their pointer, which is all the original code ever observes about it.
    pub fn payload(&self) -> u32 {
        match self {
            Ref::S32(n) => *n as u32,
            Ref::F32(g) => g.to_bits(),
            Ref::Bool(b) => *b as u32,
            Ref::Symid(s) | Ref::Bifk(s) | Ref::Basic(s) => *s,
            Ref::Nil | Ref::Tag(_) | Ref::Err | Ref::Void => 0,
            _ => 0x8000_0000,
        }
    }

    /// The "false" test used by or/and/cond/while/filter: `tag == Bool && value == 0`.
    pub fn is_false(&self) -> bool {
        matches!(self, Ref::Bool(false))
    }

    /// CRef::GetFloat (0x00129da8): S32 converted, F32 as is, anything else 0.
    pub fn as_f32(&self) -> f32 {
        match self {
            Ref::S32(n) => *n as f32,
            Ref::F32(g) => *g,
            _ => 0.0,
        }
    }

    /// Reads the payload as a float regardless of tag (what floor/round/truncate do).
    pub fn raw_f32(&self) -> f32 {
        f32::from_bits(self.payload())
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Ref::Pair(_)) || matches!(self, Ref::Tag(TAGK_NONE))
    }

    pub fn pair(&self) -> Option<PairRef> {
        match self {
            Ref::Pair(p) => Some(p.clone()),
            _ => None,
        }
    }
}

pub fn new_pair(car: Ref, next: Option<PairRef>) -> PairRef {
    Rc::new(RefCell::new(Pair { car, next }))
}

/// CRef::SetPair (0x00129970 / 0x00129480): a null list becomes the empty list.
pub fn list_ref(head: Option<PairRef>) -> Ref {
    match head {
        Some(p) => Ref::Pair(p),
        None => Ref::empty(),
    }
}

/// Builds a proper list from refs.
pub fn list_from(items: Vec<Ref>) -> Ref {
    let mut head: Option<PairRef> = None;
    for r in items.into_iter().rev() {
        head = Some(new_pair(r, head));
    }
    list_ref(head)
}

/// Collects the cars of a pair chain.
pub fn list_items(head: Option<PairRef>) -> Vec<Ref> {
    let mut out = Vec::new();
    let mut cur = head;
    while let Some(p) = cur {
        out.push(p.borrow().car.clone());
        cur = p.borrow().next.clone();
    }
    out
}

/// Iterates pair cells.
pub fn pairs(head: Option<PairRef>) -> Vec<PairRef> {
    let mut out = Vec::new();
    let mut cur = head;
    while let Some(p) = cur {
        cur = p.borrow().next.clone();
        out.push(p);
    }
    out
}
