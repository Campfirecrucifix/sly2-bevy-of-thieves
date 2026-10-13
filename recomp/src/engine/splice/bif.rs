//! The 169 built-in functions (BIF table at .data 0x002bbb90: `{fn, nargs, fRest}` x 169).
//! Dispatch is by index (stable mod ABI). Pure BIFs are implemented here; world-touching ones go
//! through `SpliceHost` (host.rs). BIF signature in the original: `CRef fn(int argc, CRef *argv, CFrame*)`
//! where a rest parameter arrives as one extra list argument.

use super::eval::Vm;
use super::frame::FrameRef;
use super::host::{SpliceHost, dispatch_host};
use super::value::*;
use super::SpliceError;
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy)]
pub struct Bif {
    pub name: &'static str,
    pub addr: u32,
    pub nargs: usize,
    pub rest: bool,
    pub alias: Option<&'static str>,
}

/// Name only; `fn`, `nargs`, `fRest` are read from the table in SCUS_973.16 (WP-PUB2).
const fn b(name: &'static str) -> Bif {
    Bif { name, addr: 0, nargs: 0, rest: false, alias: None }
}
const fn a(mut x: Bif, alias: &'static str) -> Bif {
    x.alias = Some(alias);
    x
}

/// BIF table address in SCUS_973.16 (169 x {fn, nargs, fRest}).
pub const BIF_TABLE_ADDR: u32 = 0x002b_bb90;

/// BIF table, index = BIFK (Sly 2 numbering): names (from the `splice_bifs` table) + the original's
/// `{fn, nargs, fRest}` rows read from SCUS_973.16 on first use.
pub static BIF_TABLE: std::sync::LazyLock<Vec<Bif>> = std::sync::LazyLock::new(|| {
    let x = crate::engine::disc::exe::exe();
    BIF_NAMES
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let a = BIF_TABLE_ADDR + 12 * i as u32;
            Bif { addr: x.w(a), nargs: x.w(a + 4) as usize, rest: x.w(a + 8) != 0, ..*b }
        })
        .collect()
});

const BIF_NAMES: [Bif; 169] = [
    a(b("RefOpAdd"), "+"),
    a(b("RefOpSub"), "-"),
    a(b("RefOpMult"), "*"),
    a(b("RefOpDiv"), "/"),
    a(b("RefOpPrint"), "print"),
    b("RefOpPrintFrame"),
    b("RefOpPrintSidebag"),
    a(b("RefOpIntEqual"), "="),
    a(b("RefOpL"), "<"),
    a(b("RefOpLE"), "<="),
    a(b("RefOpG"), ">"),
    a(b("RefOpGE"), ">="),
    a(b("RefOpEqv"), "eqv?"),
    a(b("RefOpEqual"), "equal?"),
    a(b("RefOpIsBoolean"), "boolean?"),
    a(b("RefOpIsNum"), "number?"),
    a(b("RefOpIsInteger"), "integer?"),
    a(b("RefOpIsFloat"), "float?"),
    a(b("RefOpIsSymbol"), "symbol?"),
    a(b("RefOpIsVector"), "vector?"),
    a(b("RefOpIsMatrix"), "matrix?"),
    b("RefOpIsClq"),
    b("RefOpIsLm"),
    b("RefOpIsSmp"),
    a(b("RefOpIsList"), "list?"),
    a(b("RefOpIsNull"), "null?"),
    a(b("RefOpIsObject"), "object?"),
    b("RefOpIsNullObj"),
    b("RefOpIsMethod"),
    a(b("RefOpIsProcedure"), "procedure?"),
    b("RefOpAreNear"),
    a(b("RefOpNot"), "not"),
    a(b("RefOpCons"), "cons"),
    a(b("RefOpCar"), "car"),
    a(b("RefOpCdr"), "cdr"),
    a(b("RefOpSetCar"), "set-car!"),
    a(b("RefOpSetCdr"), "set-cdr!"),
    a(b("RefOpLength"), "length"),
    a(b("RefOpNth"), "nth"),
    a(b("RefOpIsMember"), "member?"),
    a(b("RefOpList"), "list"),
    a(b("Bif41_SpliceBif_41"), "shuffle"),
    a(b("RefOpAppend"), "append"),
    a(b("RefOpMap"), "map"),
    a(b("RefOpFilter"), "filter"),
    a(b("RefOpForEach"), "for-each"),
    a(b("RefOpEval"), "eval"),
    a(b("Bif47_SpliceBif_47"), "vector"),
    a(b("RefOpMatrix"), "matrix"),
    b("RefOpClq"),
    b("RefOpLm"),
    b("RefOpSmp"),
    b("RefOpGetElement"),
    a(b("RefOpRandom"), "random"),
    a(b("RefOpSqrt"), "sqrt"),
    a(b("RefOpSin"), "sin"),
    a(b("RefOpCos"), "cos"),
    a(b("RefOpTan"), "tan"),
    a(b("RefOpAsin"), "asin"),
    a(b("RefOpAcos"), "acos"),
    a(b("RefOpAtan"), "atan"),
    b("RefOpRadNormalize"),
    a(b("RefOpAtan2"), "atan2"),
    b("RefOpVectorDotProduct"),
    b("RefOpVectorCrossProduct"),
    b("RefOpVectorLth"),
    b("RefOpVectorDistance"),
    b("RefOpVectorDistanceSquared"),
    b("RefOpVectorNormalize"),
    b("RefOpVectorProjectNormal"),
    b("RefOpVectorProjectTangent"),
    b("RefOpVectorBallisticVelocity"),
    b("RefOpVectorRadianNormal"),
    b("RefOpMatrixTranspose"),
    b("RefOpMatrixInvert"),
    b("RefOpMatrixCalculateDmat"),
    b("RefOpMatrixInterpolateRotate"),
    b("RefOpMatrixDecomposeToTranslate"),
    b("RefOpMatrixDecomposeToRotate"),
    b("RefOpMatrixDecomposeToEuler"),
    b("RefOpMatrixDecomposeToRadianNormal"),
    b("RefOpMatrixLookAt"),
    b("RefOpMatrixTiltUpright"),
    b("RefOpClqEvaluate"),
    b("RefOpClqEvaluateLm"),
    b("RefOpClqFit"),
    b("RefOpLmLimit"),
    b("RefOpLmCheck"),
    a(b("RefOpFloor"), "floor"),
    a(b("RefOpCeiling"), "ceiling"),
    a(b("RefOpRound"), "round"),
    a(b("RefOpTruncate"), "truncate"),
    a(b("RefOpAbs"), "abs"),
    a(b("RefOpMaximum"), "max"),
    a(b("RefOpMinimum"), "min"),
    a(b("RefOpModulo"), "modulo"),
    b("RefOpCurrentTime"),
    b("Bif97_SpliceBif_97"),
    b("Bif98_SpliceBif_98"),
    b("Bif99_SpliceBif_99"),
    b("Bif100_SpliceBif_100"),
    b("Bif101_SpliceBif_101"),
    b("Bif102_SpliceBif_102"),
    b("Bif103_SpliceBif_103"),
    b("Bif104_SpliceBif_104"),
    b("RefOpScheduleCallback"),
    b("Bif106_SpliceBif_106"),
    b("RefOpDeferObjectUpdate"),
    b("RefOpAddO"),
    b("RefOpEnsureO"),
    b("RefOpSetO"),
    b("RefOpGetO"),
    b("Bif112_SpliceBif_112"),
    b("Bif113_SpliceBif_113"),
    b("Bif114_SpliceBif_114"),
    b("Bif115_SpliceBif_115"),
    b("Bif116_SpliceBif_116"),
    b("RefOpFindPlayerObject"),
    b("RefOpFindWorldObject"),
    b("RefOpFindCameraObject"),
    b("RefOpFindClassObjects"),
    b("RefOpFindObjectsInBoundingBox"),
    b("RefOpFindObjectsInBoundingSphere"),
    b("RefOpHitTestObjects"),
    b("RefOpHitTestObjectsFirst"),
    b("RefOpConvertObjectPosition"),
    b("RefOpConvertObjectVector"),
    b("RefOpConvertObjectMatrix"),
    b("RefOpNearClipCenter"),
    b("RefOpStartSound"),
    b("Bif130_SpliceBif_130"),
    b("RefOpStopSound"),
    b("RefOpStartRumble"),
    b("Bif133_SpliceBif_133"),
    b("Bif134_SpliceBif_134"),
    b("Bif135_SpliceBif_135"),
    b("Bif136_SpliceBif_136"),
    b("Bif137_SpliceBif_137"),
    b("Bif138_SpliceBif_138"),
    b("RefOpEmitSmokeCloud"),
    b("Bif140_SpliceBif_140"),
    b("Bif141_SpliceBif_141"),
    b("Bif142_SpliceBif_142"),
    b("Bif143_SpliceBif_143"),
    b("Bif144_SpliceBif_144"),
    b("Bif145_SpliceBif_145"),
    b("Bif146_SpliceBif_146"),
    b("Bif147_SpliceBif_147"),
    b("Bif148_SpliceBif_148"),
    b("Bif149_SpliceBif_149"),
    b("Bif150_SpliceBif_150"),
    b("Bif151_SpliceBif_151"),
    b("Bif152_SpliceBif_152"),
    b("Bif153_SpliceBif_153"),
    b("Bif154_SpliceBif_154"),
    b("Bif155_SpliceBif_155"),
    b("Bif156_SpliceBif_156"),
    b("Bif157_SpliceBif_157"),
    b("Bif158_SpliceBif_158"),
    b("Bif159_SpliceBif_159"),
    b("Bif160_SpliceBif_160"),
    b("Bif161_SpliceBif_161"),
    b("Bif162_SpliceBif_162"),
    b("RefOpSetMusicRegister"),
    b("Bif164_SpliceBif_164"),
    b("Bif165_SpliceBif_165"),
    b("Bif166_SpliceBif_166"),
    b("Bif167_SpliceBif_167"),
    b("Bif168_SpliceBif_168"),
];

/// What kind of handler an index has (for tests / tooling).
#[derive(Debug, PartialEq, Eq)]
pub enum BifKind {
    Pure,
    Host,
}

pub fn bif_kind(idx: usize) -> Option<BifKind> {
    if idx >= BIF_TABLE.len() {
        None
    } else if super::host::HOST_BIFS.contains(&idx) {
        Some(BifKind::Host)
    } else {
        Some(BifKind::Pure)
    }
}

/// Looks a BIF index up by its table name or alias (mods' scripts call BIFs by name).
pub fn bif_index(name: &str) -> Option<usize> {
    BIF_TABLE.iter().position(|b| b.name == name || b.alias == Some(name))
}

fn arg(args: &[Ref], i: usize) -> Result<&Ref, SpliceError> {
    args.get(i).ok_or(SpliceError::Arity)
}

fn rest_items(args: &[Ref], i: usize) -> Vec<Ref> {
    args.get(i).and_then(|r| r.pair()).map(|p| list_items(Some(p))).unwrap_or_default()
}

fn vec_of(r: &Ref) -> Result<Vector, SpliceError> {
    match r {
        Ref::Vector(v) => Ok(*v),
        o => Err(SpliceError::Type("vector", o.tagk())),
    }
}
fn mat_of(r: &Ref) -> Result<Matrix, SpliceError> {
    match r {
        Ref::Matrix(m) => Ok(*m),
        o => Err(SpliceError::Type("matrix", o.tagk())),
    }
}

fn num(g: f32, int: bool) -> Ref {
    if int { Ref::S32(g as i32) } else { Ref::F32(g) }
}

/// Calls BIF `idx`.
pub fn call_bif(vm: &mut Vm, host: &mut dyn SpliceHost, idx: usize, args: &[Ref], fr: &FrameRef) -> Result<Ref, SpliceError> {
    if let Some(r) = dispatch_host(host, idx, args) {
        return r;
    }
    let f = |i| arg(args, i);
    Ok(match idx {
        0 => ref_op_arith(args, Arith::Add)?,
        1 => ref_op_arith(args, Arith::Sub)?,
        2 => ref_op_arith(args, Arith::Mult)?,
        3 => ref_op_arith(args, Arith::Div)?,
        4 => {
            // 0x001202d0 RefOpPrint
            let s = super::serialize::print_ref(f(0)?, &vm.symbols);
            vm.log.push(s);
            Ref::Void
        }
        5 => {
            // 0x00120300 RefOpPrintFrame
            let s: Vec<String> = fr
                .borrow()
                .bindings
                .iter()
                .rev()
                .map(|(s, r)| format!("{} = {}", vm.symbols.name(*s).unwrap_or("?"), super::serialize::print_ref(r, &vm.symbols)))
                .collect();
            vm.log.push(s.join("\n"));
            Ref::Void
        }
        6 => {
            // 0x00120330 RefOpPrintSidebag (sidebag = object property bag; printed as the ref)
            let s = super::serialize::print_ref(f(0)?, &vm.symbols);
            vm.log.push(format!("sidebag {s}"));
            Ref::Void
        }
        7 => Ref::Bool(f(0)?.payload() == f(1)?.payload()), // 0x00120370: compares payload words only
        8 => ref_cmp(f(0)?, f(1)?, |o| o.is_lt())?,
        9 => ref_cmp(f(0)?, f(1)?, |o| o.is_le())?,
        10 => ref_cmp(f(0)?, f(1)?, |o| o.is_gt())?,
        11 => ref_cmp(f(0)?, f(1)?, |o| o.is_ge())?,
        12 => Ref::Bool(ref_op_eqv(f(0)?, f(1)?)),
        13 => Ref::Bool(ref_equal(f(0)?, f(1)?)),
        14 => Ref::Bool(f(0)?.tagk() == TAGK_BOOL),
        15 => Ref::Bool(matches!(f(0)?.tagk(), TAGK_S32 | TAGK_F32)),
        16 => Ref::Bool(f(0)?.tagk() == TAGK_S32),
        17 => Ref::Bool(f(0)?.tagk() == TAGK_F32),
        18 => Ref::Bool(f(0)?.tagk() == TAGK_SYMID),
        19 => Ref::Bool(f(0)?.tagk() == TAGK_VECTOR),
        20 => Ref::Bool(f(0)?.tagk() == TAGK_MATRIX),
        21 => Ref::Bool(f(0)?.tagk() == TAGK_CLQ),
        22 => Ref::Bool(f(0)?.tagk() == TAGK_LM),
        23 => Ref::Bool(f(0)?.tagk() == TAGK_SMP),
        24 => Ref::Bool(f(0)?.is_list()),
        25 => Ref::Bool(f(0)?.tagk() == TAGK_NONE),
        26 => Ref::Bool(f(0)?.tagk() == TAGK_BASIC),
        27 => Ref::Bool(matches!(f(0)?, Ref::Basic(0))),
        28 => Ref::Bool(f(0)?.tagk() == TAGK_METHOD),
        29 => Ref::Bool(matches!(f(0)?.tagk(), TAGK_PROC | TAGK_BIFK)),
        30 => ref_op_are_near(f(0)?, f(1)?, f(2)?),
        31 => Ref::Bool(f(0)?.is_false()),
        32 => {
            // 0x00120b08 RefOpCons: quirk — a non-pair cdr is dropped.
            let next = f(1)?.pair();
            Ref::Pair(new_pair(f(0)?.clone(), next))
        }
        33 => {
            let p = f(0)?.pair().ok_or(SpliceError::Type("pair", f(0)?.tagk()))?;
            let c = p.borrow().car.clone();
            c
        }
        34 => {
            let p = f(0)?.pair().ok_or(SpliceError::Type("pair", f(0)?.tagk()))?;
            let n = p.borrow().next.clone();
            list_ref(n)
        }
        35 => {
            let p = f(0)?.pair().ok_or(SpliceError::Type("pair", f(0)?.tagk()))?;
            p.borrow_mut().car = f(1)?.clone();
            Ref::Void
        }
        36 => {
            let p = f(0)?.pair().ok_or(SpliceError::Type("pair", f(0)?.tagk()))?;
            p.borrow_mut().next = f(1)?.pair();
            Ref::Void
        }
        37 => Ref::S32(list_items(f(0)?.pair()).len() as i32),
        38 => {
            // 0x00120d78: out of range -> '()
            let n = f(1)?.payload() as i32;
            if n < 0 { Ref::empty() } else { list_items(f(0)?.pair()).into_iter().nth(n as usize).unwrap_or(Ref::empty()) }
        }
        39 => {
            let k = f(0)?;
            Ref::Bool(list_items(f(1)?.pair()).iter().any(|x| ref_equal(k, x)))
        }
        40 => f(0)?.clone(), // RefOpList: the rest list itself
        41 => bif41_shuffle(vm, f(0)?),
        42 => ref_op_append(&rest_items(args, 0)),
        43 | 44 | 45 => ref_op_map_like(vm, host, idx, f(0)?, f(1)?, fr)?,
        46 => {
            // 0x00121d20 RefOpEval
            let e = f(0)?.clone();
            vm.eval(host, &e, fr)?
        }
        47 => {
            // 0x00121dd8 RefOpVector: 0 args = zero, 1 = splat, 3 = xyz.
            let it = rest_items(args, 0);
            match it.len() {
                0 => Ref::Vector([0.0; 3]),
                1 => Ref::Vector([it[0].as_f32(); 3]),
                3 => Ref::Vector([it[0].as_f32(), it[1].as_f32(), it[2].as_f32()]),
                _ => Ref::Vector([0.0; 3]),
            }
        }
        48 => ref_op_matrix(&rest_items(args, 0)),
        49 => Ref::Clq([f(0)?.as_f32(), f(1)?.as_f32(), f(2)?.as_f32()]),
        50 => Ref::Lm([f(0)?.as_f32(), f(1)?.as_f32()]),
        51 => Ref::Smp([f(0)?.as_f32(), f(1)?.as_f32(), f(2)?.as_f32()]),
        52 => ref_op_get_element(f(0)?, &rest_items(args, 1)),
        53 => {
            // 0x001223c0 RefOpRandom: int range if both ints, else float.
            match (f(0)?, f(1)?) {
                (Ref::S32(lo), Ref::S32(hi)) => Ref::S32(vm.rand_int(*lo, *hi)),
                (lo, hi) => Ref::F32(vm.rand_float(lo.as_f32(), hi.as_f32())),
            }
        }
        54..=61 => {
            // 0x001224a8 RefUfo
            let g = f(0)?.as_f32();
            Ref::F32(match idx {
                54 => g.sqrt(),
                55 => g.sin(),
                56 => g.cos(),
                57 => g.tan(),
                58 => g.asin(),
                59 => g.acos(),
                60 => g.atan(),
                _ => rad_normalize(g),
            })
        }
        62 => Ref::F32(f(0)?.as_f32().atan2(f(1)?.as_f32())),
        63 => Ref::F32(dot(vec_of(f(0)?)?, vec_of(f(1)?)?)),
        64 => Ref::Vector(cross(vec_of(f(0)?)?, vec_of(f(1)?)?)),
        65 => Ref::F32(len(vec_of(f(0)?)?)),
        66 | 67 => {
            // 0x00122810 / 0x001228f0: optional 2nd vector defaults to DAT_002a2dd0 (zero).
            let a0 = vec_of(f(0)?)?;
            let b0 = rest_items(args, 1).first().map(vec_of).transpose()?.unwrap_or([0.0; 3]);
            let d = sub(a0, b0);
            Ref::F32(if idx == 66 { len(d) } else { dot(d, d) })
        }
        68 => {
            // 0x001229b8 (v [fallback]): |v| < 1e-4 -> fallback vector (default zero), else v/|v|.
            let v = vec_of(f(0)?)?;
            let l = len(v);
            if l < 1e-4 {
                Ref::Vector(rest_items(args, 1).first().map(vec_of).transpose()?.unwrap_or([0.0; 3]))
            } else {
                Ref::Vector(scale(v, 1.0 / l))
            }
        }
        69 => {
            let (v, n) = (vec_of(f(0)?)?, vec_of(f(1)?)?);
            Ref::Vector(scale(n, dot(v, n)))
        }
        70 => {
            let (v, n) = (vec_of(f(0)?)?, vec_of(f(1)?)?);
            Ref::Vector(sub(v, scale(n, dot(v, n))))
        }
        71 => {
            // 0x00122be0 (from to s [object]): gravity (0,0,-1750) unless an object is given (its
            // +0xe8->+0x80 vector; F2: object gravity not modelled -> default). s read as raw f32.
            let (a0, b0) = (vec_of(f(0)?)?, vec_of(f(1)?)?);
            Ref::Vector(ballistic_velocity(a0, b0, [0.0, 0.0, -1750.0], f(2)?.raw_f32()))
        }
        72 => {
            // 0x00122ce0: list (angle axis) from RadBetweenVectors 0x0011e4b0.
            let (ang, axis) = rad_between(vec_of(f(0)?)?, vec_of(f(1)?)?);
            list_from(vec![Ref::F32(ang), Ref::Vector(axis)])
        }
        73 => Ref::Matrix(transpose(mat_of(f(0)?)?)),
        74 => Ref::Matrix(invert(mat_of(f(0)?)?)),
        // 0x00122ea0 CalculateDmat: affine_inverse(a) * b (0x0011c318 then 0x0011c168).
        75 => Ref::Matrix(mul(affine_inverse(&mat_of(f(0)?)?), mat_of(f(1)?)?)),
        // 0x00122f00: t read as raw payload bits (an S32 is NOT converted).
        76 => Ref::Matrix(interpolate(mat_of(f(0)?)?, mat_of(f(1)?)?, f(2)?.raw_f32())),
        77 => {
            let m = mat_of(f(0)?)?;
            Ref::Vector([m[12], m[13], m[14]])
        }
        // 0x00123080: quirk — returns an unchanged copy (translation kept).
        78 => Ref::Matrix(mat_of(f(0)?)?),
        79 => Ref::Vector(to_euler(mat_of(f(0)?)?)),
        80 => {
            let (ang, axis) = decompose_rot(&mat_of(f(0)?)?);
            list_from(vec![Ref::F32(ang), Ref::Vector(axis)])
        }
        81 => {
            // 0x00123208 (from to [up]): dir = to - from, up defaults to DAT_002a2dc0 (0,0,1).
            let d = sub(vec_of(f(1)?)?, vec_of(f(0)?)?);
            let up = rest_items(args, 2).first().map(vec_of).transpose()?.unwrap_or([0.0, 0.0, 1.0]);
            Ref::Matrix(look_at(d, up))
        }
        82 => Ref::Matrix(tilt_upright(mat_of(f(0)?)?)),
        83 => {
            let (c, x) = (clq_of(f(0)?)?, f(1)?.as_f32());
            Ref::F32(eval_clq(c, x))
        }
        84 => {
            let (c, l, x) = (clq_of(f(0)?)?, lm_of(f(1)?)?, f(2)?.as_f32());
            Ref::F32(eval_clq(c, limit_lm(l, x)))
        }
        85 => {
            // 0x0011d8d8 FitClq(g0, g1, u, gU)
            let (g0, g1, u, gu) = (f(0)?.as_f32(), f(1)?.as_f32(), f(2)?.as_f32(), f(3)?.as_f32());
            let g2 = ((gu - g0) / u - (g1 - g0)) / (u - 1.0);
            Ref::Clq([g0, (g1 - g0) - g2, g2])
        }
        86 => Ref::F32(limit_lm(lm_of(f(0)?)?, f(1)?.as_f32())),
        87 => {
            let (l, g) = (lm_of(f(0)?)?, f(1)?.as_f32());
            Ref::Bool(l[0] < g && g < l[1])
        }
        // 0x00123668 / 0x001236d8: quirk — payload reinterpreted as float, truncated toward zero.
        88 | 91 => Ref::S32(f(0)?.raw_f32() as i32),
        89 => Ref::S32((f(0)?.raw_f32() + 1.0) as i32), // 0x00123688: (int)(x + 1.0)
        90 => Ref::S32((f(0)?.raw_f32() + 0.5) as i32), // 0x001236b0
        92 => match f(0)? {
            Ref::S32(n) => Ref::S32(n.wrapping_abs()),
            o => Ref::F32(o.raw_f32().abs()),
        },
        93 => ref_op_extreme(f(0)?, &rest_items(args, 1), true),
        94 => ref_op_extreme(f(0)?, &rest_items(args, 1), false),
        95 => {
            // 0x001239d8: integer remainder (C semantics); divide by zero traps on the PS2.
            let d = f(1)?.payload() as i32;
            if d == 0 {
                return Err(SpliceError::DivideByZero);
            }
            Ref::S32((f(0)?.payload() as i32).wrapping_rem(d))
        }
        // 0x00125690.. retail no-ops that return void (BIF 134) and Nil (140..=168, debug BIFs
        // compiled out in SCUS_973.16 — including RefOpSetMusicRegister).
        134 => Ref::Void,
        140..=168 => Ref::Nil,
        _ => return Err(SpliceError::BadBif(idx as u32)),
    })
}

/// 0x00120548 RefOpEqv. Quirks: same-tag keyword/empty/void refs are eqv; floats, vectors and
/// other value blocks are never eqv; ints, bools, symbols, bifks, pairs/procs/objects/methods by identity.
pub fn ref_op_eqv(x: &Ref, y: &Ref) -> bool {
    if x.tagk() != y.tagk() {
        return false;
    }
    match (x, y) {
        (Ref::Tag(_), _) | (Ref::Void, _) => true,
        (Ref::S32(a), Ref::S32(b)) => a == b,
        (Ref::Bool(a), Ref::Bool(b)) => a == b,
        (Ref::Symid(a), Ref::Symid(b)) | (Ref::Bifk(a), Ref::Bifk(b)) | (Ref::Basic(a), Ref::Basic(b)) => a == b,
        (Ref::Pair(a), Ref::Pair(b)) => std::rc::Rc::ptr_eq(a, b),
        (Ref::Proc(a), Ref::Proc(b)) => std::rc::Rc::ptr_eq(a, b),
        (Ref::Method(a), Ref::Method(b)) => std::rc::Rc::ptr_eq(a, b),
        _ => false,
    }
}

/// 0x001205b8 RefEqualHelper: structural equality (lists recursively, value blocks componentwise).
pub fn ref_equal(x: &Ref, y: &Ref) -> bool {
    match (x, y) {
        (Ref::F32(a), Ref::F32(b)) => a == b,
        (Ref::Vector(a), Ref::Vector(b)) | (Ref::Clq(a), Ref::Clq(b)) | (Ref::Smp(a), Ref::Smp(b)) => a == b,
        (Ref::Lm(a), Ref::Lm(b)) => a == b,
        (Ref::Matrix(a), Ref::Matrix(b)) => a == b,
        (Ref::Pair(_), Ref::Pair(_)) => {
            let (l1, l2) = (list_items(x.pair()), list_items(y.pair()));
            l1.len() == l2.len() && l1.iter().zip(&l2).all(|(a, b)| ref_equal(a, b))
        }
        _ => ref_op_eqv(x, y),
    }
}

/// 0x00120398 RefCmp: ints compare as ints, otherwise as floats.
fn ref_cmp(x: &Ref, y: &Ref, ok: fn(std::cmp::Ordering) -> bool) -> Result<Ref, SpliceError> {
    let o = match (x, y) {
        (Ref::S32(a), Ref::S32(b)) => a.cmp(b),
        _ => x.as_f32().partial_cmp(&y.as_f32()).unwrap_or(std::cmp::Ordering::Equal),
    };
    Ok(Ref::Bool(ok(o)))
}

#[derive(Clone, Copy, PartialEq)]
enum Arith {
    Add,
    Sub,
    Mult,
    Div,
}

/// 0x0011f440 / 0x0011f6d0 / 0x0011f960 / 0x0011fec8. The first argument's tag picks the domain.
/// Numbers are accumulated in float and converted back to int only if every operand was an int
/// (quirk: integer math goes through f32; `(- x)` returns x unchanged; int division truncates).
fn ref_op_arith(args: &[Ref], op: Arith) -> Result<Ref, SpliceError> {
    let first = arg(args, 0)?;
    let rest = rest_items(args, 1);
    if matches!(op, Arith::Add | Arith::Sub) {
        return ref_op_add_sub(first, &rest, op);
    }
    // 0x0011f960 / 0x0011fec8: the accumulator takes the first arg (S32 -> F32 + int flag);
    // any other tag gives Nil. Each element is combined according to the accumulator's tag;
    // unsupported combinations are silently skipped.
    let mut int = false;
    let mut acc = match first {
        Ref::S32(n) => {
            int = true;
            Ref::F32(*n as f32)
        }
        Ref::F32(_) | Ref::Vector(_) | Ref::Matrix(_) | Ref::Clq(_) | Ref::Lm(_) => first.clone(),
        _ => return Ok(Ref::Nil),
    };
    let div = op == Arith::Div;
    for r in &rest {
        let g = match r {
            Ref::S32(n) => Some(*n as f32),
            Ref::F32(g) => Some(*g),
            _ => None,
        };
        let k = |x: f32| if div { 1.0 / x } else { x };
        acc = match (&acc, r, g) {
            (Ref::F32(a), _, Some(g)) => {
                if r.tagk() == TAGK_F32 {
                    int = false;
                }
                Ref::F32(if div { a / g } else { a * g })
            }
            (Ref::F32(a), Ref::Vector(v), None) if !div => Ref::Vector(scale(*v, *a)),
            (Ref::F32(a), Ref::Matrix(m), None) if !div => Ref::Matrix(m.map(|x| x * a)),
            (Ref::F32(a), Ref::Clq(c), None) if !div => Ref::Clq(c.map(|x| x * a)),
            (Ref::F32(a), Ref::Lm(c), None) if !div => Ref::Lm(c.map(|x| x * a)),
            (Ref::Vector(v), _, Some(g)) => Ref::Vector(scale(*v, k(g))),
            (Ref::Matrix(m), _, Some(g)) => Ref::Matrix(m.map(|x| x * k(g))),
            (Ref::Clq(c), _, Some(g)) => Ref::Clq(c.map(|x| x * k(g))),
            (Ref::Lm(c), _, Some(g)) => Ref::Lm(c.map(|x| x * k(g))),
            // matrix acc * vector elem: row-vector transform v.x r0 + v.y r1 + v.z r2 + r3
            (Ref::Matrix(m), Ref::Vector(v), None) if !div => Ref::Vector(xform_vec(*v, m)),
            // matrix acc * matrix elem: 0x0011c168(out, acc, elem) = elem * acc
            (Ref::Matrix(m), Ref::Matrix(n), None) if !div => Ref::Matrix(mul(*n, *m)),
            _ => acc,
        };
    }
    Ok(match acc {
        Ref::F32(g) if int => Ref::S32(g as i32),
        o => o,
    })
}

/// 0x0011f440 / 0x0011f6d0 (WP-F, not re-verified by F2).
fn ref_op_add_sub(first: &Ref, rest: &[Ref], op: Arith) -> Result<Ref, SpliceError> {
    match first {
        Ref::S32(_) | Ref::F32(_) => {
            let mut all_int = first.tagk() == TAGK_S32;
            let mut acc = first.as_f32();
            for r in rest {
                if r.tagk() != TAGK_S32 {
                    all_int = false;
                }
                let g = r.as_f32();
                acc = if op == Arith::Add { acc + g } else { acc - g };
            }
            Ok(num(acc, all_int))
        }
        Ref::Vector(v) => {
            let mut acc = *v;
            for r in rest {
                acc = match (op, r) {
                    (Arith::Add, Ref::Vector(w)) => add(acc, *w),
                    (_, Ref::Vector(w)) => sub(acc, *w),
                    (_, o) => return Err(SpliceError::Type("vector", o.tagk())),
                };
            }
            Ok(Ref::Vector(acc))
        }
        Ref::Matrix(m) => {
            let mut acc = *m;
            for r in rest {
                match (op, r) {
                    (Arith::Add, Ref::Matrix(n)) => acc.iter_mut().zip(n).for_each(|(a, b)| *a += b),
                    (_, Ref::Matrix(n)) => acc.iter_mut().zip(n).for_each(|(a, b)| *a -= b),
                    (_, o) => return Err(SpliceError::Type("matrix", o.tagk())),
                }
            }
            Ok(Ref::Matrix(acc))
        }
        o => Err(SpliceError::Type("number/vector/matrix", o.tagk())),
    }
}

/// 0x00123788 / 0x001238b0 RefOpMaximum/Minimum: keeps the winning operand (with its tag).
/// Quirk: a float champion against an int challenger compares `(int)champion`.
fn ref_op_extreme(first: &Ref, rest: &[Ref], max: bool) -> Ref {
    let mut cur = first.clone();
    for r in rest {
        let better = match (&cur, r) {
            (Ref::S32(c), Ref::S32(n)) => if max { n > c } else { n < c },
            (Ref::F32(c), Ref::S32(n)) => {
                let c = *c as i32;
                if max { *n > c } else { *n < c }
            }
            (c, n) => {
                let (c, n) = (c.as_f32(), n.as_f32());
                if max { c < n } else { n < c }
            }
        };
        if better {
            cur = r.clone();
        }
    }
    cur
}

/// 0x00120a58 RefOpAreNear (a b eps): S32 -> (float)int, any other tag -> raw payload bits as f32;
/// 0x0011d4d0: |a-b| / max(1, |a|) < eps.
fn ref_op_are_near(x: &Ref, y: &Ref, eps: &Ref) -> Ref {
    let g = |r: &Ref| if let Ref::S32(n) = r { *n as f32 } else { r.raw_f32() };
    let (a, b, e) = (g(x), g(y), g(eps));
    Ref::Bool((a - b).abs() / a.abs().max(1.0) < e)
}

/// 0x001222c8 RefOpGetElement (x i [j]): vector/clq/lm/smp -> element i; matrix with one index ->
/// row i as a Vector, with two -> m[i*4+j]; other tags -> Nil. (No bounds check in the original;
/// out of range -> Nil here.)
fn ref_op_get_element(x: &Ref, idx: &[Ref]) -> Ref {
    let i = idx.first().map(|r| r.payload() as usize).unwrap_or(0);
    let get = |s: &[f32], i: usize| s.get(i).copied().map(Ref::F32).unwrap_or(Ref::Nil);
    match x {
        Ref::Vector(v) | Ref::Clq(v) | Ref::Smp(v) => get(v, i),
        Ref::Lm(v) => get(v, i),
        Ref::Matrix(m) => match idx.get(1) {
            Some(j) => get(m, i * 4 + j.payload() as usize),
            None if i < 4 => Ref::Vector(row(m, i)),
            None => Ref::Nil,
        },
        _ => Ref::Nil,
    }
}

/// 0x00121f00 RefOpMatrix: 1 number -> diag(s,s,s,1); 3 or 4 vectors -> rows; 9 numbers -> rows
/// 0-2 xyz (w 0); 16 numbers -> full; anything else -> identity.
fn ref_op_matrix(it: &[Ref]) -> Ref {
    let mut m = IDENTITY;
    let all_vec = it.iter().all(|r| matches!(r, Ref::Vector(_)));
    match it.len() {
        1 => {
            let s = it[0].as_f32();
            m[0] = s;
            m[5] = s;
            m[10] = s;
        }
        3 | 4 if all_vec => {
            for (i, r) in it.iter().enumerate() {
                let w = m[i * 4 + 3];
                set_row(&mut m, i, vec_of(r).unwrap(), w);
            }
        }
        9 => {
            for i in 0..3 {
                set_row(&mut m, i, [it[i * 3].as_f32(), it[i * 3 + 1].as_f32(), it[i * 3 + 2].as_f32()], 0.0);
            }
        }
        16 => m.iter_mut().zip(it).for_each(|(d, s)| *d = s.as_f32()),
        _ => {}
    }
    Ref::Matrix(m)
}

/// 0x001212f8 RefOpAppend: only pair arguments are used (others skipped); every list but the
/// last argument is copied, a final pair argument is shared; no lists -> '().
fn ref_op_append(it: &[Ref]) -> Ref {
    let lists: Vec<(usize, PairRef)> = it.iter().enumerate().filter_map(|(i, r)| r.pair().map(|p| (i, p))).collect();
    let mut tail: Option<PairRef> = None;
    let mut items = Vec::new();
    for (i, p) in lists {
        if i + 1 == it.len() {
            tail = Some(p);
        } else {
            items.extend(list_items(Some(p)));
        }
    }
    let mut head = tail;
    for r in items.into_iter().rev() {
        head = Some(new_pair(r, head));
    }
    list_ref(head)
}

/// 0x00121710 map / 0x00121938 filter / 0x00121c20 for-each. Quirk kept: each element is spliced
/// into a `(proc elem)` form and EVALUATED, so elements are re-evaluated (symbols looked up).
fn ref_op_map_like(vm: &mut Vm, host: &mut dyn SpliceHost, idx: usize, proc_: &Ref, list: &Ref, fr: &FrameRef) -> Result<Ref, SpliceError> {
    if list.tagk() == TAGK_NONE {
        return Ok(if idx == 45 { Ref::Void } else { list.clone() });
    }
    let items = list_items(list.pair());
    let mut out = Vec::new();
    // 0x00121578 builds `((quote proc) (quote elem))` (two tag-0x12 cells): neither the procedure
    // nor the element is evaluated again (WP-J12; was `(proc elem)`, re-evaluating each element).
    let q = |r: &Ref| list_from(vec![Ref::Tag(TAGK_QUOTE), r.clone()]);
    for it in items {
        let form = list_from(vec![q(proc_), q(&it)]);
        let v = vm.eval(host, &form, fr)?;
        match idx {
            43 => out.push(v),
            44 if !v.is_false() => out.push(it),
            _ => {}
        }
    }
    Ok(if idx == 45 { Ref::Void } else { list_from(out) })
}

/// 0x00120fb0 (unnamed BIF 41) "deck": the list's non-'() cars form a deck; a '() marker splits
/// dealt / remaining cards. k = cards before the last '() (-1 if none). If n > 1 and k <= 0 the
/// deck is reshuffled (swap a[i] with a[rand(0,n-1)]; if a[0] is still the old first card swap it
/// with a[n-1]) and k = n. Then the head card moves after card k-1 and a '() is inserted after
/// card k-1 unless it became the new head.
fn bif41_shuffle(vm: &mut Vm, list: &Ref) -> Ref {
    let Some(p) = list.pair() else { return Ref::empty() };
    let all = list_items(Some(p));
    let mut a: Vec<Ref> = Vec::new();
    let mut k: i32 = -1;
    for r in all {
        if r.tagk() == TAGK_NONE {
            k = a.len() as i32;
        } else {
            a.push(r);
        }
    }
    let n = a.len();
    let mut shuffled = false;
    if n > 1 && k <= 0 {
        let first = a[0].clone();
        for i in 0..n {
            let j = vm.rand_int(0, n as i32 - 1) as usize;
            a.swap(i, j);
        }
        if same(&a[0], &first) {
            a.swap(0, n - 1);
        }
        shuffled = true;
        k = n as i32;
    }
    if k >= 2 {
        let k = k as usize;
        if shuffled {
            a.insert(k, Ref::empty());
        } else {
            let h = a.remove(0);
            a.insert(k - 1, h);
            if k > 2 {
                a.insert(k - 1, Ref::empty());
            }
        }
    }
    list_from(a)
}

/// Identity of the original CRef cell (pointer compare) approximated by eqv.
fn same(a: &Ref, b: &Ref) -> bool {
    ref_op_eqv(a, b)
}

fn clq_of(r: &Ref) -> Result<Clq, SpliceError> {
    match r {
        Ref::Clq(c) => Ok(*c),
        o => Err(SpliceError::Type("clq", o.tagk())),
    }
}
fn lm_of(r: &Ref) -> Result<Lm, SpliceError> {
    match r {
        Ref::Lm(c) => Ok(*c),
        o => Err(SpliceError::Type("lm", o.tagk())),
    }
}

fn eval_clq(c: Clq, x: f32) -> f32 {
    c[0] + x * (c[1] + x * c[2])
}

/// GLimitLm.
fn limit_lm(l: Lm, g: f32) -> f32 {
    if g < l[0] {
        l[0]
    } else if g > l[1] {
        l[1]
    } else {
        g
    }
}

/// RadNormalize (Sly 1 util.c): wraps into [-PI, PI].
pub fn rad_normalize(rad: f32) -> f32 {
    if !(-PI..=PI).contains(&rad) {
        let m = (rad + PI) % (2.0 * PI);
        let m = if m < 0.0 { m + 2.0 * PI } else { m };
        m - PI
    } else {
        rad
    }
}

// ---- vector / matrix helpers (row-major MATRIX4, rows = basis, translation in row 3) ----
pub const IDENTITY: Matrix = [1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.];

fn add(a: Vector, b: Vector) -> Vector {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: Vector, b: Vector) -> Vector {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale(a: Vector, s: f32) -> Vector {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn dot(a: Vector, b: Vector) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: Vector, b: Vector) -> Vector {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn len(a: Vector) -> f32 {
    dot(a, a).sqrt()
}
fn xform_vec(v: Vector, m: &Matrix) -> Vector {
    let mut o = [0.0; 3];
    for (c, oc) in o.iter_mut().enumerate() {
        *oc = v[0] * m[c] + v[1] * m[4 + c] + v[2] * m[8 + c] + m[12 + c];
    }
    o
}
// ---- Sly 2 VU0 math helpers (F2: transcribed from the decompile, f32 throughout) ----

/// DAT_002a2da0: default unit axis (1,0,0).
const AXIS_X: Vector = [1.0, 0.0, 0.0];

fn row(m: &Matrix, i: usize) -> Vector {
    [m[i * 4], m[i * 4 + 1], m[i * 4 + 2]]
}
fn set_row(m: &mut Matrix, i: usize, v: Vector, w: f32) {
    m[i * 4] = v[0];
    m[i * 4 + 1] = v[1];
    m[i * 4 + 2] = v[2];
    m[i * 4 + 3] = w;
}

/// 0x0011e390: unit vector perpendicular to v (crossed with an axis-swapped helper; no zero check).
fn perp(v: Vector) -> Vector {
    let (x, y, z) = (v[0].abs(), v[1].abs(), v[2].abs());
    let w = if x <= y {
        if x <= z { [0.0, v[2], -v[1]] } else { [v[1], -v[0], 0.0] }
    } else if y <= z {
        [v[2], 0.0, -v[0]]
    } else {
        [v[1], -v[0], 0.0]
    };
    let c = cross(v, w);
    scale(c, 1.0 / len(c))
}

/// 0x0011b480 LoadRotateMatrixRad(sin, cos, axis): rows 0-2 (w = 0), row 3 untouched.
fn rot_sc(s: f32, c: f32, n: Vector) -> [Vector; 3] {
    let t = 1.0 - c;
    [
        [t * n[0] * n[0] + c, t * n[1] * n[0] + s * n[2], t * n[2] * n[0] - s * n[1]],
        [t * n[0] * n[1] - s * n[2], t * n[1] * n[1] + c, t * n[2] * n[1] + s * n[0]],
        [t * n[0] * n[2] + s * n[1], t * n[1] * n[2] - s * n[0], t * n[2] * n[2] + c],
    ]
}
/// 0x0011b500: angle -> sincos (0x0011d608 polynomial; f32 sin/cos used here) -> 0x0011b480.
fn rot_ang(a: f32, n: Vector) -> [Vector; 3] {
    rot_sc(a.sin(), a.cos(), n)
}

/// 0x0011e4b0 RadBetweenVectors(v1, v2) -> (angle, axis). acos argument is not clamped.
fn rad_between(v1: Vector, v2: Vector) -> (f32, Vector) {
    let (l1, l2) = (len(v1), len(v2));
    if l1 < 1e-4 || l2 < 1e-4 {
        return (0.0, AXIS_X);
    }
    let c = cross(v1, v2);
    let lc = len(c);
    let axis = if lc > 1e-8 { scale(c, 1.0 / lc) } else { perp(v1) };
    ((dot(v1, v2) / (l1 * l2)).acos(), axis)
}

/// 0x0011bcf8: rotation part of m -> (angle, axis).
fn decompose_rot(m: &Matrix) -> (f32, Vector) {
    let c = (m[0] + m[5] + m[10] - 1.0) * 0.5;
    if c > 0.999999 {
        return (0.0, AXIS_X);
    }
    let mut a = [0.0f32; 3];
    let mut rem = 1.0f32;
    for i in 0..2 {
        let q = ((m[5 * i] - c) / (1.0 - c)).abs().clamp(0.0, rem.max(0.0));
        a[i] = q.sqrt();
        rem -= q;
    }
    a[2] = rem.max(0.0).sqrt();
    if c >= -0.9999 {
        if m[1] < m[4] {
            a[2] = -a[2];
        }
        if m[8] < m[2] {
            a[1] = -a[1];
        }
        if m[6] < m[9] {
            a[0] = -a[0];
        }
        (c.acos(), a)
    } else {
        // Half-turn: sign of the smaller components from the symmetric off-diagonals relative to
        // the largest one (F2: reconstructed; the original branch was not fully decoded).
        let k = (0..3).max_by(|&i, &j| a[i].partial_cmp(&a[j]).unwrap()).unwrap();
        for i in 0..3 {
            if i != k && m[k * 4 + i] + m[i * 4 + k] < 0.0 {
                a[i] = -a[i];
            }
        }
        (PI, a)
    }
}

/// 0x0011c168: out = B*A (row-vector convention: out row i = B_i . A).
fn mul(a: Matrix, b: Matrix) -> Matrix {
    let mut o = [0.0; 16];
    for r in 0..4 {
        for c in 0..4 {
            o[r * 4 + c] = a[r * 4] * b[c] + a[r * 4 + 1] * b[4 + c] + a[r * 4 + 2] * b[8 + c] + a[r * 4 + 3] * b[12 + c];
        }
    }
    o
}

/// 0x0011c8e8 plain 4x4 transpose.
fn transpose(m: Matrix) -> Matrix {
    let mut o = [0.0; 16];
    for r in 0..4 {
        for c in 0..4 {
            o[c * 4 + r] = m[r * 4 + c];
        }
    }
    o
}

/// 0x0011c500 (called as (4,4,src,dst)): Gauss-Jordan with partial pivoting in f32.
/// Singular (|pivot| < 1e-4) leaves the partially reduced result (F2: approximated as the
/// working inverse at the point of failure).
fn invert(m: Matrix) -> Matrix {
    let mut a = [[0f32; 8]; 4];
    for r in 0..4 {
        for c in 0..4 {
            a[r][c] = m[r * 4 + c];
        }
        a[r][4 + r] = 1.0;
    }
    for col in 0..4 {
        let piv = (col..4).max_by(|&x, &y| a[x][col].abs().partial_cmp(&a[y][col].abs()).unwrap()).unwrap();
        if a[piv][col].abs() < 1e-4 {
            break;
        }
        a.swap(col, piv);
        let p = a[col][col];
        a[col].iter_mut().for_each(|x| *x /= p);
        for r in 0..4 {
            if r != col {
                let f = a[r][col];
                for c in 0..8 {
                    a[r][c] -= f * a[col][c];
                }
            }
        }
    }
    let mut o = [0.0; 16];
    for r in 0..4 {
        for c in 0..4 {
            o[r * 4 + c] = a[r][4 + c];
        }
    }
    o
}

/// 0x0011c318: affine inverse (cofactor rows, 1/det, translation = -t * R^-1).
fn affine_inverse(m: &Matrix) -> Matrix {
    let (r0, r1, r2, t) = (row(m, 0), row(m, 1), row(m, 2), row(m, 3));
    let (c0, c1, c2) = (cross(r1, r2), cross(r2, r0), cross(r0, r1));
    let det = (r0[0] * c0[0] + r0[1] * c0[1]) + r0[2] * c0[2];
    let id = 1.0 / det;
    let inv = [scale([c0[0], c1[0], c2[0]], id), scale([c0[1], c1[1], c2[1]], id), scale([c0[2], c1[2], c2[2]], id)];
    let mut o = [0.0; 16];
    for (i, r) in inv.iter().enumerate() {
        set_row(&mut o, i, *r, 0.0);
    }
    let tr = add(add(scale(inv[0], t[0]), scale(inv[1], t[1])), scale(inv[2], t[2]));
    set_row(&mut o, 3, scale(tr, -1.0), 1.0);
    o
}

/// 0x00122f00 RefOpMatrixInterpolateRotate: tmp = A^T*B (0x0011bc00, 3x3), decompose, rotate by
/// angle*t; out rows 0-2 = A_i.xyz * R (w = A_i.w), row 3 = (0,0,0,1).
fn interpolate(a: Matrix, b: Matrix, t: f32) -> Matrix {
    let mut tmp = IDENTITY;
    for i in 0..3 {
        for c in 0..4 {
            tmp[i * 4 + c] = (0..3).map(|k| a[k * 4 + i] * b[k * 4 + c]).sum();
        }
    }
    let (ang, axis) = decompose_rot(&tmp);
    let r = rot_ang(ang * t, axis);
    let mut o = IDENTITY;
    for i in 0..3 {
        let v = add(add(scale(r[0], a[i * 4]), scale(r[1], a[i * 4 + 1])), scale(r[2], a[i * 4 + 2]));
        set_row(&mut o, i, v, a[i * 4 + 3]);
    }
    o
}

/// 0x0011bff0 DecomposeMatrixToEuler.
fn to_euler(m: Matrix) -> Vector {
    let y = (-m[2]).clamp(-1.0, 1.0).asin();
    const LIM: f32 = 1.5706964;
    if y < LIM && y > -LIM {
        [m[6].atan2(m[10]), y, m[1].atan2(m[0])]
    } else if y <= -LIM {
        [(-m[4]).atan2(-m[8]), y, 0.0]
    } else {
        [(-m[4]).atan2(m[8]), y, 0.0]
    }
}

/// 0x0011b830 LookAt(dir, up): r0 = dir, r1 = up x r0 (perp if degenerate), r2 = r0 x r1.
/// Row 3 is left uninitialised in the original (pool memory); identity row here.
fn look_at(dir: Vector, up: Vector) -> Matrix {
    let r0 = if len(dir) < 1e-4 { AXIS_X } else { scale(dir, 1.0 / len(dir)) };
    let c = cross(up, r0);
    let r1 = if len(c) < 1e-4 { perp(r0) } else { scale(c, 1.0 / len(c)) };
    let r2 = cross(r0, r1);
    let mut o = IDENTITY;
    set_row(&mut o, 0, r0, 0.0);
    set_row(&mut o, 1, r1, 0.0);
    set_row(&mut o, 2, r2, 0.0);
    o
}

/// 0x0011ba08: 3x3 rotation taking a onto b.
fn rot_between(a: Vector, b: Vector) -> [Vector; 3] {
    let (la, lb) = (len(a), len(b));
    if la < 1e-4 || lb < 1e-4 {
        return [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    }
    let c = cross(a, b);
    let lc = len(c);
    let axis = if lc > 1e-8 { scale(c, 1.0 / lc) } else { perp(a) };
    let cs = (dot(a, b) / (la * lb)).clamp(-1.0, 1.0);
    rot_sc((1.0 - cs * cs).sqrt(), cs, axis)
}

/// 0x0011b938 TiltUpright(m): rotate so row 2 becomes +Z; row 3 copied, w = 1.
fn tilt_upright(m: Matrix) -> Matrix {
    let r = rot_between(row(&m, 2), [0.0, 0.0, 1.0]);
    let mut o = IDENTITY;
    for i in 0..3 {
        let v = add(add(scale(r[0], m[i * 4]), scale(r[1], m[i * 4 + 1])), scale(r[2], m[i * 4 + 2]));
        set_row(&mut o, i, v, 0.0);
    }
    set_row(&mut o, 3, row(&m, 3), 1.0);
    o
}

/// 0x001d6848 BallisticVelocity(a, b, g, s): s blends the launch angle from the direct elevation
/// toward vertical by sinf(s). g defaults to (0,0,-1750).
fn ballistic_velocity(a: Vector, b: Vector, g: Vector, s: f32) -> Vector {
    let d = sub(b, a);
    let dz = d[2];
    let h = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let th = dz.atan2(h).max(0.0);
    let th2 = th + (PI / 2.0 - th) * s.sin();
    let tn = th2.tan();
    let k = g[2] / (2.0 * (dz - h * tn));
    if k <= 0.0 {
        return [0.0; 3];
    }
    let spd = h * k.sqrt() / th2.cos();
    let v = [d[0], d[1], h * tn];
    scale(v, spd / len(v))
}
