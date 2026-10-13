//! WP-F2 tests: formula BIFs re-derived from the Sly 2 decompile (expected values by hand).
use sly2_recomp::engine::splice::serialize::print_ref;
use sly2_recomp::engine::splice::value::*;
use sly2_recomp::engine::splice::{NullHost, Vm};

fn ev(src: &str) -> Ref {
    let mut vm = Vm::new();
    vm.eval_text(&mut NullHost, src).unwrap()
}
fn run(src: &str) -> String {
    let mut vm = Vm::new();
    let r = vm.eval_text(&mut NullHost, src).unwrap();
    print_ref(&r, &vm.symbols)
}
fn v(r: Ref) -> Vector {
    match r {
        Ref::Vector(v) => v,
        o => panic!("not a vector: {o:?}"),
    }
}
fn m(r: Ref) -> Matrix {
    match r {
        Ref::Matrix(m) => m,
        o => panic!("not a matrix: {o:?}"),
    }
}
fn g(r: Ref) -> f32 {
    match r {
        Ref::F32(g) => g,
        o => panic!("not a float: {o:?}"),
    }
}
fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-4
}
const ROTZ90: &str = "(matrix 0 1 0 0  -1 0 0 0  0 0 1 0  0 0 0 1)";

#[test]
fn mult_div() {
    // 0x0011f960: int flag survives only all-S32 operands; result truncated (cvt.w.s).
    assert_eq!(run("(* 2 3 4)"), "24");
    assert_eq!(run("(* 2 1.5)"), "3.0");
    // F32 acc * vector elem -> vector scaled by the acc.
    assert_eq!(run("(* 2 #v(1 2 3))"), "#v(2.0 4.0 6.0)");
    // Vector acc: non-numbers skipped.
    assert_eq!(run("(* #v(1 2 3) #v(5 5 5) 2)"), "#v(2.0 4.0 6.0)");
    // Matrix acc * vector: v.x r0 + v.y r1 + v.z r2 + r3; Rz90 rows r0=(0,1,0), r1=(-1,0,0).
    let x = v(ev(&format!("(* {ROTZ90} #v(1 2 3))")));
    assert_eq!(x, [-2.0, 1.0, 3.0]);
    // Non-number/vector first arg -> Nil.
    assert_eq!(ev("(* '(1) 2)").tagk(), -1);
    // Div: F32 acc / num; vector acc scaled by 1/x.
    assert_eq!(run("(/ 7 2)"), "3");
    assert_eq!(run("(/ #v(2 4 6) 2)"), "#v(1.0 2.0 3.0)");
}

#[test]
fn matrix_ctor_get_element() {
    // 0x00121f00: 1 number -> diag(s,s,s,1).
    let d = m(ev("(matrix 2)"));
    assert_eq!((d[0], d[5], d[10], d[15]), (2.0, 2.0, 2.0, 1.0));
    // 3 vectors -> rows.
    let r = m(ev("(matrix #v(1 2 3) #v(4 5 6) #v(7 8 9))"));
    assert_eq!(&r[4..7], &[4.0, 5.0, 6.0]);
    // 0x001222c8: matrix + one index -> row vector; two indices -> element.
    assert_eq!(run("(RefOpGetElement (matrix #v(1 2 3) #v(4 5 6) #v(7 8 9)) 1)"), "#v(4.0 5.0 6.0)");
    assert_eq!(run("(RefOpGetElement (matrix #v(1 2 3) #v(4 5 6) #v(7 8 9)) 2 1)"), "8.0");
    assert_eq!(run("(RefOpGetElement #v(1 2 3) 2)"), "3.0");
    assert_eq!(ev("(RefOpGetElement 5 0)").tagk(), -1); // other tags -> Nil
}

#[test]
fn are_near_ceiling_append() {
    // 0x0011d4d0: |a-b|/max(1,|a|) < eps. 100 vs 101: 1/100 = 0.01 < 0.02.
    assert_eq!(run("(RefOpAreNear 100 101 0.02)"), "#t");
    assert_eq!(run("(RefOpAreNear 0.5 0.6 0.05)"), "#f"); // 0.1/1 = 0.1
    // 0x00123688: (int)(x + 1.0): 1.5 -> 2, 2.0 -> 3 (quirk), -1.5 -> 0.
    assert_eq!(run("(ceiling 1.5)"), "2");
    assert_eq!(run("(ceiling 2.0)"), "3");
    assert_eq!(run("(ceiling -1.5)"), "0");
    // 0x001212f8: non-list args skipped.
    assert_eq!(run("(append '(1 2) 5 '(3))"), "(1 2 3)");
    assert_eq!(run("(append 1 2)"), "()");
}

#[test]
fn vector_bifs() {
    assert!(near(g(ev("(RefOpVectorDistance #v(3 4 0))")), 5.0)); // default 2nd = zero
    assert_eq!(g(ev("(RefOpVectorDistanceSquared #v(1 1 1) #v(0 0 0))")), 3.0);
    // 0x001229b8: no length parameter; tiny -> fallback vector.
    assert_eq!(v(ev("(RefOpVectorNormalize #v(0 3 4))")), [0.0, 0.6, 0.8]);
    assert_eq!(v(ev("(RefOpVectorNormalize #v(0 0 0) #v(1 0 0))")), [1.0, 0.0, 0.0]);
    // 0x00122ce0 -> (angle axis): x onto y = PI/2 about +z.
    let l = list_items(ev("(RefOpVectorRadianNormal #v(1 0 0) #v(0 1 0))").pair());
    assert!(near(g(l[0].clone()), std::f32::consts::FRAC_PI_2));
    assert_eq!(v(l[1].clone()), [0.0, 0.0, 1.0]);
    // 0x001d6848 ballistic, from 0 to (100,0,0), s=0: th=0, tan=0, k = -1750/(2*(0-0)) = -inf <= 0 -> zero.
    assert_eq!(v(ev("(RefOpVectorBallisticVelocity #v(0 0 0) #v(100 0 0) 0.0)")), [0.0; 3]);
    // s = PI/2 -> th2 = PI/2 (vertical) -> tan huge, k>0 small; just require upward non-zero z.
    // s = asin(0.5) -> th2 = PI/4: tan=1, k = -1750/(2*(0-100)) = 8.75; spd = 100*sqrt(8.75)/cos(PI/4)
    // = 418.33; v = (100,0,100)/|.| * spd = (295.80, 0, 295.80).
    let b = v(ev("(RefOpVectorBallisticVelocity #v(0 0 0) #v(100 0 0) 0.5235988)"));
    assert!((b[0] - 295.80).abs() < 0.05 && b[1] == 0.0 && (b[2] - 295.80).abs() < 0.05, "{b:?}");
}

#[test]
fn matrix_bifs() {
    // 0x00123080: unchanged copy (translation kept).
    let t = m(ev("(RefOpMatrixDecomposeToRotate (matrix 1 0 0 0 0 1 0 0 0 0 1 0 5 6 7 1))"));
    assert_eq!(&t[12..15], &[5.0, 6.0, 7.0]);
    // Euler of Rz90: m2=0 -> y=0; x=atan2(m6=0,m10=1)=0; z=atan2(m1=1,m0=0)=PI/2.
    let e = v(ev(&format!("(RefOpMatrixDecomposeToEuler {ROTZ90})")));
    assert!(near(e[0], 0.0) && near(e[1], 0.0) && near(e[2], std::f32::consts::FRAC_PI_2));
    // 0x0011bcf8 on Rz90: c=(0+0+1-1)/2=0, ang=acos 0=PI/2, axis=(0,0,1) (m1=1 > m4=-1: z stays +).
    let l = list_items(ev(&format!("(RefOpMatrixDecomposeToRadianNormal {ROTZ90})")).pair());
    assert!(near(g(l[0].clone()), std::f32::consts::FRAC_PI_2));
    assert_eq!(v(l[1].clone()), [0.0, 0.0, 1.0]);
    // Interpolate identity -> Rz90 at t=0.5: Rz45 rows (c,s,0), (-s,c,0).
    let i = m(ev(&format!("(RefOpMatrixInterpolateRotate (matrix) {ROTZ90} 0.5)")));
    let h = std::f32::consts::FRAC_1_SQRT_2;
    assert!(near(i[0], h) && near(i[1], h) && near(i[4], -h) && near(i[5], h) && i[15] == 1.0, "{i:?}");
    // Dmat(a, b) = inv(a)*b: a = translate(5,0,0), b = identity -> translate(-5,0,0).
    let d = m(ev("(RefOpMatrixCalculateDmat (matrix 1 0 0 0 0 1 0 0 0 0 1 0 5 0 0 1) (matrix))"));
    assert_eq!(d[12], -5.0);
    // Invert diag(2) -> diag(0.5).
    assert_eq!(m(ev("(RefOpMatrixInvert (matrix 2))"))[0], 0.5);
    // LookAt from 0 to (0,1,0), up z: r0=(0,1,0), r1=z x r0=(-1,0,0), r2=r0 x r1=(0,0,1).
    let la = m(ev("(RefOpMatrixLookAt #v(0 0 0) #v(0 5 0))"));
    assert_eq!((&la[0..3], &la[4..7], &la[8..11]), (&[0.0, 1.0, 0.0][..], &[-1.0, 0.0, 0.0][..], &[0.0, 0.0, 1.0][..]));
    // Tilt of an already-upright matrix keeps it (rot_between(z,z) = identity via axis perp, sin 0).
    let tu = m(ev(&format!("(RefOpMatrixTiltUpright {ROTZ90})")));
    assert!(near(tu[1], 1.0) && near(tu[10], 1.0) && tu[15] == 1.0);
}

#[test]
fn clq_lm() {
    // GLimitLm 0x0011d948: lo<=x ? min(x,hi) : lo.
    assert_eq!(run("(RefOpLmLimit #lm(0 10) -3)"), "0.0");
    assert_eq!(run("(RefOpClqEvaluateLm #clq(1 2 3) #lm(0 1) 5)"), "6.0"); // x clamped to 1: 1+2+3
    // FitClq(0, 1, 2, 4): g2 = ((4-0)/2 - 1)/(2-1) = 1; g1 = 1 - 1 = 0 -> clq(0 0 1).
    assert_eq!(run("(RefOpClqFit 0 1 2 4)"), "#clq(0.0 0.0 1.0)");
}

#[test]
fn bif41_deck() {
    // No '() marker -> shuffled, then '() appended at the end; first card always moved.
    let mut vm = Vm::new();
    let r = vm.eval_text(&mut NullHost, "(shuffle '(1 2 3))").unwrap();
    let it = list_items(r.pair());
    assert_eq!(it.len(), 4);
    assert_eq!(it[3].tagk(), 0);
    assert_ne!(it[0].payload(), 1);
    // Marker after 3 cards (k=3): head moves after card 2, marker before it: (2 3 () 1 4).
    assert_eq!(run("(shuffle '(1 2 3 () 4))"), "(2 3 () 1 4)");
    // Non-list -> '().
    assert_eq!(run("(shuffle 5)"), "()");
}
