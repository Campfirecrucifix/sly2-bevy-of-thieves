//! Evaluator: 0x00127838 (eval dispatch), special forms 0x00126438..0x00127228, apply 0x001272e0.

use super::bif::{BIF_TABLE, call_bif};
use super::frame::{self, FrameRef};
use super::host::SpliceHost;
use super::serialize::Symbols;
use super::value::*;
use super::SpliceError;
use std::rc::Rc;

/// Maximum argument slots of the apply stack buffer (0x001272e0: 32 CRefs).
pub const MAX_ARGS: usize = 32;

/// Splice VM state (no globals; wrap in a Bevy Resource at the call site).
pub struct Vm {
    pub global: FrameRef,
    pub symbols: Symbols,
    /// RNG state for RefOpRandom / list shuffle (replaces the game RNG FUN_0011d3f8/FUN_0011d448).
    pub rng: u32,
    /// Output of RefOpPrint / assert messages (replaces the printf to the debug console).
    pub log: Vec<String>,
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}

impl Vm {
    /// Creates a VM whose global frame binds every BIF name (table name, e.g. `RefOpAdd`, and the
    /// Scheme-style alias where one exists) to its Bifk.
    pub fn new() -> Self {
        let mut vm = Vm { global: frame::new_frame(None), symbols: Symbols::default(), rng: 0x1234_5678, log: Vec::new() };
        for (i, b) in BIF_TABLE.iter().enumerate() {
            let id = vm.symbols.intern(b.name);
            frame::add_binding(&vm.global, id, Ref::Bifk(i as u32));
            if let Some(a) = b.alias {
                let id = vm.symbols.intern(a);
                frame::add_binding(&vm.global, id, Ref::Bifk(i as u32));
            }
        }
        vm
    }

    /// Breaks frame<->proc cycles (replacement for the mark/sweep collector 0x001281b0).
    pub fn reset(&mut self) {
        self.global.borrow_mut().bindings.clear();
        *self = Vm::new();
    }

    /// Parses and evaluates text in the global frame, returning the last value.
    pub fn eval_text(&mut self, host: &mut dyn SpliceHost, src: &str) -> Result<Ref, SpliceError> {
        let exprs = super::serialize::read_text(src, &mut self.symbols).map_err(|e| SpliceError::Read(e.to_string()))?;
        let g = self.global.clone();
        let mut last = Ref::Void;
        for e in exprs {
            last = self.eval(host, &e, &g)?;
        }
        Ok(last)
    }

    /// 0x001ffa70: evaluates each expression of a loaded script block in the global frame.
    pub fn run_script_block(&mut self, host: &mut dyn SpliceHost, exprs: &[Option<PairRef>]) -> Result<(), SpliceError> {
        let g = self.global.clone();
        for e in exprs.iter().flatten() {
            let car = e.borrow().car.clone();
            self.eval(host, &car, &g)?;
        }
        Ok(())
    }

    /// 0x00127838 CRef eval. Symbols are looked up, lists are special forms or applications,
    /// everything else evaluates to itself.
    pub fn eval(&mut self, host: &mut dyn SpliceHost, r: &Ref, fr: &FrameRef) -> Result<Ref, SpliceError> {
        match r {
            // 0x00126438: unbound symbols evaluate to Nil (lookup leaves the -1 ref).
            Ref::Symid(s) => Ok(frame::lookup(fr, *s, true).unwrap_or(Ref::Nil)),
            Ref::Pair(head) => {
                let head = head.clone();
                let tag = head.borrow().car.tagk();
                let rest = head.borrow().next.clone();
                match tag {
                    TAGK_SET => self.form_set(host, rest, fr, true),
                    TAGK_DEFINE => self.form_set(host, rest, fr, false),
                    TAGK_ASSERT => self.form_assert(host, rest, fr),
                    TAGK_FORM6 => match rest {
                        // 0x001266c8
                        None => Ok(Ref::Void),
                        Some(p) => {
                            let e = p.borrow().car.clone();
                            self.eval(host, &e, fr)
                        }
                    },
                    TAGK_IF => self.form_if(host, rest, fr),
                    TAGK_OR => {
                        // 0x00126840: first value that is not #f, else #f.
                        for e in list_items(rest) {
                            let v = self.eval(host, &e, fr)?;
                            if !v.is_false() {
                                return Ok(v);
                            }
                        }
                        Ok(Ref::Bool(false))
                    }
                    TAGK_AND => {
                        // 0x00126928: #t, then each value; stops at the first #f.
                        let mut v = Ref::Bool(true);
                        for e in list_items(rest) {
                            v = self.eval(host, &e, fr)?;
                            if v.is_false() {
                                break;
                            }
                        }
                        Ok(v)
                    }
                    TAGK_COND => self.form_cond(host, rest, fr),
                    TAGK_CASE => self.form_case(host, rest, fr),
                    TAGK_LET => self.form_let(host, rest, fr),
                    TAGK_WHILE => {
                        // 0x00126f78: quirk — only the FIRST body expression is evaluated.
                        let p = rest.ok_or(SpliceError::Malformed("while"))?;
                        let test = p.borrow().car.clone();
                        let body = p.borrow().next.as_ref().map(|b| b.borrow().car.clone());
                        loop {
                            if self.eval(host, &test, fr)?.is_false() {
                                break;
                            }
                            if let Some(b) = &body {
                                self.eval(host, b, fr)?;
                            }
                        }
                        Ok(Ref::Void)
                    }
                    TAGK_LAMBDA => self.form_lambda(rest, fr),
                    TAGK_BEGIN => {
                        // 0x00127228
                        let mut v = Ref::Void;
                        for e in list_items(rest) {
                            v = self.eval(host, &e, fr)?;
                        }
                        Ok(v)
                    }
                    // 0x00127838 case 0x11: import evaluates to Nil.
                    TAGK_IMPORT => Ok(Ref::Nil),
                    // case 0x12: quote returns the next element unevaluated.
                    TAGK_QUOTE => Ok(rest.map(|p| p.borrow().car.clone()).unwrap_or(Ref::Nil)),
                    _ => self.apply(host, &head, fr),
                }
            }
            other => Ok(other.clone()),
        }
    }

    /// 0x00126490 (set!) / 0x00126540 (define): `(set! sym expr)`; both return void.
    fn form_set(&mut self, host: &mut dyn SpliceHost, rest: Option<PairRef>, fr: &FrameRef, set: bool) -> Result<Ref, SpliceError> {
        let p = rest.ok_or(SpliceError::Malformed("define/set!"))?;
        let sym = p.borrow().car.payload();
        let ve = p.borrow().next.as_ref().map(|n| n.borrow().car.clone()).ok_or(SpliceError::Malformed("define/set!"))?;
        let v = self.eval(host, &ve, fr)?;
        Ok(if set { frame::set_binding(fr, sym, v) } else { frame::add_binding(fr, sym, v) })
    }

    /// 0x001265f0: `(assert expr msg...)`; on a zero payload prints "Splice assert: " + message.
    fn form_assert(&mut self, host: &mut dyn SpliceHost, rest: Option<PairRef>, fr: &FrameRef) -> Result<Ref, SpliceError> {
        let p = rest.ok_or(SpliceError::Malformed("assert"))?;
        let e = p.borrow().car.clone();
        let v = self.eval(host, &e, fr)?;
        if v.payload() == 0 {
            let msg: Vec<String> = list_items(p.borrow().next.clone()).iter().map(|r| super::serialize::print_ref(r, &self.symbols)).collect();
            self.log.push(format!("Splice assert: {}", msg.join(" ")));
        }
        Ok(Ref::Void)
    }

    /// 0x00126758: quirk — the test is true iff its payload word == 1 (so `#t` and `1`).
    fn form_if(&mut self, host: &mut dyn SpliceHost, rest: Option<PairRef>, fr: &FrameRef) -> Result<Ref, SpliceError> {
        let p = rest.ok_or(SpliceError::Malformed("if"))?;
        let test = p.borrow().car.clone();
        let then = p.borrow().next.clone().ok_or(SpliceError::Malformed("if"))?;
        let els = then.borrow().next.clone();
        let t = self.eval(host, &test, fr)?;
        if t.payload() == 1 {
            let e = then.borrow().car.clone();
            self.eval(host, &e, fr)
        } else if let Some(e) = els {
            let e = e.borrow().car.clone();
            self.eval(host, &e, fr)
        } else {
            Ok(Ref::Void)
        }
    }

    /// 0x00126a18: clauses `(test body...)` / `(else body...)`; a clause without body yields the test value.
    fn form_cond(&mut self, host: &mut dyn SpliceHost, rest: Option<PairRef>, fr: &FrameRef) -> Result<Ref, SpliceError> {
        let mut out = Ref::Void;
        for clause in list_items(rest) {
            let Some(cp) = clause.pair() else { continue };
            let test = cp.borrow().car.clone();
            let body = cp.borrow().next.clone();
            if test.tagk() != TAGK_ELSE {
                let v = self.eval(host, &test, fr)?;
                if v.is_false() {
                    continue;
                }
                if body.is_none() {
                    out = v;
                }
            }
            for e in list_items(body) {
                out = self.eval(host, &e, fr)?;
            }
            break;
        }
        Ok(out)
    }

    /// 0x00126ba0: `(case key ((d1 d2) body...) (else body...))`, data compared with eqv (unevaluated).
    fn form_case(&mut self, host: &mut dyn SpliceHost, rest: Option<PairRef>, fr: &FrameRef) -> Result<Ref, SpliceError> {
        let p = rest.ok_or(SpliceError::Malformed("case"))?;
        let ke = p.borrow().car.clone();
        let key = self.eval(host, &ke, fr)?;
        let mut out = Ref::Void;
        for clause in list_items(p.borrow().next.clone()) {
            let Some(cp) = clause.pair() else { continue };
            let data = cp.borrow().car.clone();
            let hit = data.tagk() == TAGK_ELSE
                || list_items(data.pair()).iter().any(|d| super::bif::ref_op_eqv(&key, d));
            if hit {
                for e in list_items(cp.borrow().next.clone()) {
                    out = self.eval(host, &e, fr)?;
                }
                break;
            }
        }
        Ok(out)
    }

    /// 0x00126db8: `(let ((sym expr)...) body...)`. Inits are evaluated in the OUTER frame, bound in
    /// a new frame whose parent is set afterwards. A non-list second element (e.g. `()`) means no
    /// new frame: the body starts right after it and runs in the current frame.
    fn form_let(&mut self, host: &mut dyn SpliceHost, rest: Option<PairRef>, fr: &FrameRef) -> Result<Ref, SpliceError> {
        let p = rest.ok_or(SpliceError::Malformed("let"))?;
        let binds = p.borrow().car.clone();
        let body = p.borrow().next.clone();
        let f = if let Ref::Pair(bl) = binds {
            let nf = frame::new_frame(None);
            for b in list_items(Some(bl)) {
                let bp = b.pair().ok_or(SpliceError::Malformed("let binding"))?;
                let sym = bp.borrow().car.payload();
                let e = bp.borrow().next.as_ref().map(|n| n.borrow().car.clone()).unwrap_or(Ref::Nil);
                let v = self.eval(host, &e, fr)?;
                frame::add_binding(&nf, sym, v);
            }
            frame::set_parent(&nf, Some(fr.clone()));
            nf
        } else {
            fr.clone()
        };
        let mut out = Ref::Nil;
        for e in list_items(body) {
            out = self.eval(host, &e, &f)?;
        }
        Ok(out)
    }

    /// 0x00127068: `(lambda (a b . rest) body...)`; nargs counts params before the dot.
    fn form_lambda(&mut self, rest: Option<PairRef>, fr: &FrameRef) -> Result<Ref, SpliceError> {
        let p = rest.ok_or(SpliceError::Malformed("lambda"))?;
        let params = p.borrow().car.pair();
        let body = p.borrow().next.clone();
        let mut nargs = 0;
        let mut has_rest = false;
        for c in pairs(params.clone()) {
            if c.borrow().car.tagk() == TAGK_DOT {
                has_rest = true;
                break;
            }
            nargs += 1;
        }
        Ok(Ref::Proc(Rc::new(Proc { frame: fr.clone(), nargs, rest: has_rest, params, body })))
    }

    /// 0x001272e0: evaluates the operator and `nargs` arguments; extra arguments become a list when
    /// the callee takes a rest parameter (empty list if none), then calls a BIF / method / proc.
    pub fn apply(&mut self, host: &mut dyn SpliceHost, head: &PairRef, fr: &FrameRef) -> Result<Ref, SpliceError> {
        let op_e = head.borrow().car.clone();
        let op = self.eval(host, &op_e, fr)?;
        let (nargs, rest) = match &op {
            Ref::Bifk(k) => {
                let b = BIF_TABLE.get(*k as usize).ok_or(SpliceError::BadBif(*k))?;
                (b.nargs, b.rest)
            }
            Ref::Method(m) => (m.nargs, false),
            Ref::Proc(p) => (p.nargs, p.rest),
            other => return Err(SpliceError::NotCallable(other.tagk())),
        };
        let mut cur = head.borrow().next.clone();
        let mut args = Vec::with_capacity(nargs + rest as usize);
        for _ in 0..nargs {
            let c = cur.ok_or(SpliceError::Arity)?;
            let e = c.borrow().car.clone();
            args.push(self.eval(host, &e, fr)?);
            cur = c.borrow().next.clone();
        }
        if rest {
            let mut extra = Vec::new();
            for e in list_items(cur) {
                extra.push(self.eval(host, &e, fr)?);
            }
            args.push(list_from(extra));
        }
        if args.len() > MAX_ARGS {
            return Err(SpliceError::Arity);
        }
        self.call(host, &op, args, fr)
    }

    /// Calls an already-evaluated operator with already-evaluated arguments.
    pub fn call(&mut self, host: &mut dyn SpliceHost, op: &Ref, args: Vec<Ref>, fr: &FrameRef) -> Result<Ref, SpliceError> {
        match op {
            Ref::Bifk(k) => call_bif(self, host, *k as usize, &args, fr),
            Ref::Method(m) => host.call_method(m, &args),
            Ref::Proc(p) => {
                // New frame parented to the closure frame; params bound in order, the dot skipped.
                let nf = frame::new_frame(None);
                frame::set_parent(&nf, Some(p.frame.clone()));
                let mut it = args.into_iter();
                for c in pairs(p.params.clone()) {
                    let car = c.borrow().car.clone();
                    if car.tagk() == TAGK_DOT {
                        continue;
                    }
                    frame::add_binding(&nf, car.payload(), it.next().unwrap_or(Ref::empty()));
                }
                // 0x00127190: body evaluated in sequence, last value returned.
                let mut out = Ref::Nil;
                for e in list_items(p.body.clone()) {
                    out = self.eval(host, &e, &nf)?;
                }
                Ok(out)
            }
            other => Err(SpliceError::NotCallable(other.tagk())),
        }
    }

    pub(crate) fn next_rand(&mut self) -> u32 {
        // xorshift32 (replacement RNG)
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        x
    }

    /// Replacement for FUN_0011d3f8: uniform int in [lo, hi].
    pub(crate) fn rand_int(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        let span = (hi as i64 - lo as i64 + 1) as u64;
        (lo as i64 + (self.next_rand() as u64 % span) as i64) as i32
    }

    /// Replacement for FUN_0011d448: uniform float in [lo, hi).
    pub(crate) fn rand_float(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (self.next_rand() as f32 / 4_294_967_296.0) * (hi - lo)
    }
}
