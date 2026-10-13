//! WP-F tests: Splice VM + BIFs.
use sly2_recomp::engine::splice::bif::{bif_index, bif_kind, call_bif, BifKind, BIF_TABLE};
use sly2_recomp::engine::splice::serialize::{
    ppair_serialize_in, ppair_serialize_out, print_ref, read_script_block, read_text,
};
use sly2_recomp::engine::splice::value::*;
use sly2_recomp::engine::splice::{NullHost, SpliceError, SpliceHost, Vm};

fn run(src: &str) -> String {
    let mut vm = Vm::new();
    let r = vm.eval_text(&mut NullHost, src).unwrap();
    print_ref(&r, &vm.symbols)
}

#[test]
fn arithmetic() {
    assert_eq!(run("(+ 1 2 3)"), "6");
    assert_eq!(run("(+ 1 2.5)"), "3.5");
    assert_eq!(run("(- 10 3 2)"), "5");
    assert_eq!(run("(- 5)"), "5"); // quirk: no negation
    assert_eq!(run("(* 2 3 4)"), "24");
    assert_eq!(run("(/ 7 2)"), "3"); // int / int truncates
    assert_eq!(run("(/ 7.0 2)"), "3.5");
    assert_eq!(run("(+ #v(1 2 3) #v(1 1 1))"), "#v(2.0 3.0 4.0)");
    assert_eq!(run("(* #v(1 2 3) 2)"), "#v(2.0 4.0 6.0)");
    assert_eq!(run("(modulo 7 3)"), "1");
    assert_eq!(run("(modulo -7 3)"), "-1");
    assert_eq!(run("(abs -4)"), "4");
    assert_eq!(run("(abs -4.5)"), "4.5");
    assert_eq!(run("(max 1 5 3)"), "5");
    assert_eq!(run("(min 4 2.5 3)"), "2.5");
    assert_eq!(run("(floor -1.5)"), "-1"); // quirk: truncation
    assert_eq!(run("(round 1.5)"), "2");
    assert_eq!(run("(sqrt 16)"), "4.0");
    assert_eq!(run("(RefOpVectorDotProduct #v(1 2 3) #v(4 5 6))"), "32.0");
    assert_eq!(run("(RefOpVectorCrossProduct #v(1 0 0) #v(0 1 0))"), "#v(0.0 0.0 1.0)");
    assert_eq!(run("(RefOpClqEvaluate #clq(1 2 3) 2)"), "17.0");
    assert_eq!(run("(RefOpLmLimit #lm(0 10) 12)"), "10.0");
    let mut vm = Vm::new();
    assert_eq!(vm.eval_text(&mut NullHost, "(modulo 1 0)").unwrap_err(), SpliceError::DivideByZero);
}

#[test]
fn predicates_and_compare() {
    assert_eq!(run("(< 1 2)"), "#t");
    assert_eq!(run("(>= 1 2.0)"), "#f");
    assert_eq!(run("(eqv? 1 1)"), "#t");
    assert_eq!(run("(eqv? 1.0 1.0)"), "#f"); // quirk: floats never eqv
    assert_eq!(run("(equal? 1.0 1.0)"), "#t");
    assert_eq!(run("(equal? '(1 (2 3)) '(1 (2 3)))"), "#t");
    assert_eq!(run("(eqv? '() '())"), "#t");
    assert_eq!(run("(number? 1.5)"), "#t");
    assert_eq!(run("(integer? 1.5)"), "#f");
    assert_eq!(run("(list? '())"), "#t");
    assert_eq!(run("(null? '(1))"), "#f");
    assert_eq!(run("(symbol? 'a)"), "#t");
    assert_eq!(run("(procedure? car)"), "#t");
    assert_eq!(run("(procedure? (lambda (x) x))"), "#t");
    assert_eq!(run("(not #f)"), "#t");
    assert_eq!(run("(not 0)"), "#f");
    assert_eq!(run("(RefOpIsNullObj #obj(0))"), "#t");
    assert_eq!(run("(= #t 1)"), "#t"); // IntEqual compares payloads
}

#[test]
fn lists() {
    assert_eq!(run("(cons 1 '(2 3))"), "(1 2 3)");
    assert_eq!(run("(cons 1 2)"), "(1)"); // quirk: non-pair cdr dropped
    assert_eq!(run("(car '(1 2))"), "1");
    assert_eq!(run("(cdr '(1))"), "()");
    assert_eq!(run("(length '(1 2 3))"), "3");
    assert_eq!(run("(length 5)"), "0");
    assert_eq!(run("(nth '(a b c) 1)"), "b");
    assert_eq!(run("(nth '(a b c) 9)"), "()");
    assert_eq!(run("(member? 2 '(1 2 3))"), "#t");
    assert_eq!(run("(list 1 2 (+ 1 2))"), "(1 2 3)");
    assert_eq!(run("(append '(1 2) '() '(3))"), "(1 2 3)");
    assert_eq!(run("(map (lambda (x) (* x x)) '(1 2 3))"), "(1 4 9)");
    assert_eq!(run("(filter (lambda (x) (> x 1)) '(1 2 3))"), "(2 3)");
    assert_eq!(run("(define l '(1 2)) (set-car! l 9) l"), "(9 2)");
    assert_eq!(run("(define n 0) (for-each (lambda (x) (set! n (+ n x))) '(1 2 3)) n"), "6");
    assert_eq!(run("(eval '(+ 1 2))"), "3");
    assert_eq!(run("(length (shuffle '(1 2 3 4)))"), "5"); // F2: 0x00120fb0 appends a '() marker after a reshuffle
}

#[test]
fn special_forms() {
    assert_eq!(run("(define x 5) x"), "5");
    assert_eq!(run("(define x 5)"), "#void");
    assert_eq!(run("(if (< 1 2) 'y 'n)"), "y");
    assert_eq!(run("(if 2 'y 'n)"), "n"); // quirk: only payload 1 is true
    assert_eq!(run("(if #f 'y)"), "#void");
    assert_eq!(run("(or #f 3)"), "3");
    assert_eq!(run("(and 1 #f 3)"), "#f");
    assert_eq!(run("(and 1 2)"), "2");
    assert_eq!(run("(cond (#f 1) ((< 1 2) 2) (else 3))"), "2");
    assert_eq!(run("(cond (#f 1) (else 3))"), "3");
    assert_eq!(run("(cond (7))"), "7");
    assert_eq!(run("(case 2 ((1) 'a) ((2 3) 'b) (else 'c))"), "b");
    assert_eq!(run("(case 9 ((1) 'a) (else 'c))"), "c");
    assert_eq!(run("(let ((a 1) (b 2)) (+ a b))"), "3");
    assert_eq!(run("(define i 0) (while (< i 5) (set! i (+ i 1))) i"), "5");
    assert_eq!(run("(begin 1 2 3)"), "3");
    assert_eq!(run("(define (f) 1)"), "#void"); // not Scheme sugar: binds a garbage symbol, harmless
    assert_eq!(run("(define sq (lambda (x) (* x x))) (sq 7)"), "49");
    assert_eq!(run("(define f (lambda (a . r) r)) (f 1 2 3)"), "(2 3)");
    assert_eq!(run("(define f (lambda (a . r) r)) (f 1)"), "()");
    assert_eq!(run("(define mk (lambda (n) (lambda (x) (+ x n)))) ((mk 10) 5)"), "15");
    assert_eq!(run("(quote (a b))"), "(a b)");
    assert_eq!(run("(import foo)"), "#nil");
    let mut vm = Vm::new();
    vm.eval_text(&mut NullHost, "(assert #f oops)").unwrap();
    assert_eq!(vm.log, vec!["Splice assert: oops".to_string()]);
}

#[test]
fn every_bif_index_dispatches() {
    assert_eq!(BIF_TABLE.len(), 169);
    let mut vm = Vm::new();
    let g = vm.global.clone();
    for i in 0..169 {
        let kind = bif_kind(i).expect("handler");
        let b = BIF_TABLE[i];
        let mut args: Vec<Ref> = (0..b.nargs).map(|_| Ref::S32(1)).collect();
        if b.rest {
            args.push(Ref::empty());
        }
        let r = call_bif(&mut vm, &mut NullHost, i, &args, &g);
        match kind {
            BifKind::Host => assert!(matches!(r, Err(SpliceError::HostUnimplemented(_))), "bif {i} {:?}", r),
            BifKind::Pure => assert!(!matches!(r, Err(SpliceError::HostUnimplemented(_)) | Err(SpliceError::BadBif(_))), "bif {i}"),
        }
    }
    assert!(bif_kind(169).is_none());
    assert_eq!(bif_index("RefOpAdd"), Some(0));
    assert_eq!(bif_index("RefOpSetMusicRegister"), Some(163));
}

struct TestHost;
impl SpliceHost for TestHost {
    fn ref_op_find_player_object(&mut self, _: &[Ref]) -> Result<Ref, SpliceError> {
        Ok(Ref::Basic(42))
    }
}

#[test]
fn host_bifs_route_to_host() {
    let mut vm = Vm::new();
    let r = vm.eval_text(&mut TestHost, "(RefOpFindPlayerObject)").unwrap();
    assert!(matches!(r, Ref::Basic(42)));
    assert!(matches!(vm.eval_text(&mut TestHost, "(RefOpStartSound 1 2)"), Err(SpliceError::HostUnimplemented(_))));
}

#[test]
fn binary_round_trip() {
    let src = "(define x (lambda (a . r) (if (< a 2) #v(1 2 3) '(#m(1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1) #clq(1 2 3) #lm(0 1) #smp(1 2 3) #bif(4) #obj(7) 1.5 #t))))";
    let mut vm = Vm::new();
    let exprs = read_text(src, &mut vm.symbols).unwrap();
    // A level script block: u32 count + one chain per expression (car = the expression).
    let mut bin = (exprs.len() as u32).to_le_bytes().to_vec();
    for e in &exprs {
        ppair_serialize_out(Some(&new_pair(e.clone(), None)), &mut bin);
    }
    let (block, used) = read_script_block(&bin, false).unwrap();
    assert_eq!(used, bin.len());
    let back = block[0].as_ref().unwrap().borrow().car.clone();
    assert_eq!(print_ref(&back, &vm.symbols), print_ref(&exprs[0], &vm.symbols));
    let mut bin2 = Vec::new();
    ppair_serialize_out(block[0].as_ref(), &mut bin2);
    assert_eq!(&bin[4..], &bin2[..]);
    vm.run_script_block(&mut NullHost, &block).unwrap();
    assert_eq!(print_ref(&vm.eval_text(&mut NullHost, "(x 1)").unwrap(), &vm.symbols), "#v(1.0 2.0 3.0)");
    assert_eq!(vm.eval_text(&mut NullHost, "(length (x 5))").unwrap().payload(), 8);
    // null list marker and debug-info skipping
    assert!(ppair_serialize_in(&[0xff], false).unwrap().0.is_none());
    let dbg = [0x15u8, 0, 0, 0, 0, 1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 6, 0, 9, 0, 0, 0];
    let (p, n) = ppair_serialize_in(&dbg, true).unwrap();
    assert_eq!(n, dbg.len());
    assert!(matches!(p.unwrap().borrow().car, Ref::S32(9)));
}

#[test]
fn text_round_trip() {
    for s in ["(a (b c) 1 2.5 #t #f ())", "(lambda (x . y) (quote z))", "#v(1.0 2.0 3.0)"] {
        let mut vm = Vm::new();
        let e = read_text(s, &mut vm.symbols).unwrap();
        assert_eq!(print_ref(&e[0], &vm.symbols), s);
    }
}
