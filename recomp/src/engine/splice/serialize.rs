//! Splice script serialization.
//!
//! Binary (level data) format, 0x0012a3d8 `PpairSerializeIn` (little endian):
//! `u8 tag` — 0xff = end/null list. Bit 7 set = another pair follows (next in the chain).
//! If debug info is enabled (DAT_003e1c34), `u32 + 6 x u16` follow the tag (Sly 1: 5 x u16).
//! Payload by tag (low 7 bits): S32 `i32`; F32 `f32`; Bool `u32 != 0`; Symid/Bifk/Basic `u32`;
//! Vector 3 f32; Matrix 16 f32; Clq 3 f32; Lm 2 f32; Smp 3 f32; Pair = nested list (recursive);
//! keyword tags 0..=0x14 (except 3, 4, 0x13) carry nothing; any other tag yields a Nil ref.
//!
//! Script block (0x001ff858, called from the level loader): `u32 count; count x list` — each list
//! is one top-level expression evaluated in the world's global frame (0x001ffa70) and the block is
//! linked into world+0x39e8. With debug info, each expression is followed by 3 strings (not handled).
//!
//! The text reader/printer is a port convenience (mods, tests): symbol names are interned into
//! `Symbols`; keywords map to their tags (see `FORM_NAMES`).

use std::collections::HashMap;

use super::value::*;

#[derive(Debug)]
pub enum ReadError {
    Eof,
    Syntax(String),
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadError::Eof => write!(f, "unexpected end of splice data"),
            ReadError::Syntax(s) => write!(f, "splice syntax error: {s}"),
        }
    }
}
impl std::error::Error for ReadError {}

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}
impl Cursor<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], ReadError> {
        let s = self.data.get(self.pos..self.pos + n).ok_or(ReadError::Eof)?;
        self.pos += n;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, ReadError> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32, ReadError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn f32(&mut self) -> Result<f32, ReadError> {
        Ok(f32::from_bits(self.u32()?))
    }
    fn floats<const N: usize>(&mut self) -> Result<[f32; N], ReadError> {
        let mut a = [0.0; N];
        for g in &mut a {
            *g = self.f32()?;
        }
        Ok(a)
    }
}

/// 0x0012a3d8 PpairSerializeIn. Returns the list head and bytes consumed.
pub fn ppair_serialize_in(data: &[u8], debug_info: bool) -> Result<(Option<PairRef>, usize), ReadError> {
    let mut c = Cursor { data, pos: 0 };
    let p = read_pair(&mut c, debug_info)?;
    Ok((p, c.pos))
}

fn read_pair(c: &mut Cursor, debug_info: bool) -> Result<Option<PairRef>, ReadError> {
    let tag = c.u8()?;
    if tag == 0xff {
        return Ok(None);
    }
    if debug_info {
        c.take(4 + 6 * 2)?;
    }
    let car = match (tag & 0x7f) as i32 {
        t @ (0..=2 | 5..=0x12 | 0x14) => Ref::Tag(t),
        TAGK_S32 => Ref::S32(c.u32()? as i32),
        TAGK_F32 => Ref::F32(c.f32()?),
        TAGK_BOOL => Ref::Bool(c.u32()? != 0),
        TAGK_SYMID => Ref::Symid(c.u32()?),
        TAGK_BIFK => Ref::Bifk(c.u32()?),
        TAGK_VECTOR => Ref::Vector(c.floats::<3>()?),
        TAGK_MATRIX => Ref::Matrix(c.floats::<16>()?),
        TAGK_CLQ => Ref::Clq(c.floats::<3>()?),
        TAGK_LM => Ref::Lm(c.floats::<2>()?),
        TAGK_SMP => Ref::Smp(c.floats::<3>()?),
        TAGK_PAIR => list_ref(read_pair(c, debug_info)?),
        TAGK_BASIC => Ref::Basic(c.u32()?),
        _ => Ref::Nil,
    };
    let next = if tag & 0x80 != 0 { read_pair(c, debug_info)? } else { None };
    Ok(Some(new_pair(car, next)))
}

/// Inverse of `ppair_serialize_in` (no debug info). Used for tests and mod tooling.
/// Note: an empty list (tag 0) and a null pair-chain both exist; a `Pair` car always writes a
/// nested chain, an empty nested list is written as tag 0.
pub fn ppair_serialize_out(head: Option<&PairRef>, out: &mut Vec<u8>) {
    let Some(p) = head else {
        out.push(0xff);
        return;
    };
    let pb = p.borrow();
    let more = if pb.next.is_some() { 0x80u8 } else { 0 };
    let tag = match &pb.car {
        Ref::Nil | Ref::Err | Ref::Void | Ref::Proc(_) | Ref::Method(_) => 0x7f,
        r => r.tagk() as u8,
    };
    out.push(tag | more);
    let fl = |out: &mut Vec<u8>, a: &[f32]| a.iter().for_each(|g| out.extend(g.to_le_bytes()));
    match &pb.car {
        Ref::S32(n) => out.extend(n.to_le_bytes()),
        Ref::F32(g) => out.extend(g.to_le_bytes()),
        Ref::Bool(b) => out.extend((*b as u32).to_le_bytes()),
        Ref::Symid(s) | Ref::Bifk(s) | Ref::Basic(s) => out.extend(s.to_le_bytes()),
        Ref::Vector(v) | Ref::Clq(v) | Ref::Smp(v) => fl(out, v),
        Ref::Matrix(m) => fl(out, m),
        Ref::Lm(l) => fl(out, l),
        Ref::Pair(q) => ppair_serialize_out(Some(q), out),
        _ => {}
    }
    if let Some(n) = &pb.next {
        ppair_serialize_out(Some(n), out);
    }
}

/// 0x001ff858: reads a script block (`u32 count` + `count` expression lists).
pub fn read_script_block(data: &[u8], debug_info: bool) -> Result<(Vec<Option<PairRef>>, usize), ReadError> {
    if debug_info {
        return Err(ReadError::Syntax("script block debug strings (FUN_00118420) not supported".into()));
    }
    let mut c = Cursor { data, pos: 0 };
    let n = c.u32()?;
    let mut v = Vec::with_capacity(n as usize);
    for _ in 0..n {
        v.push(read_pair(&mut c, false)?);
    }
    Ok((v, c.pos))
}

/// Symbol table (symid <-> name). Symids in level data are opaque u32s; names are only known for
/// symbols introduced through the text reader or registered by the host.
#[derive(Debug, Default, Clone)]
pub struct Symbols {
    by_name: HashMap<String, u32>,
    by_id: HashMap<u32, String>,
    next: u32,
}

impl Symbols {
    pub fn intern(&mut self, name: &str) -> u32 {
        if let Some(&id) = self.by_name.get(name) {
            return id;
        }
        while self.by_id.contains_key(&self.next) {
            self.next += 1;
        }
        let id = self.next;
        self.register(name, id);
        id
    }
    pub fn register(&mut self, name: &str, id: u32) {
        self.by_name.insert(name.to_string(), id);
        self.by_id.insert(id, name.to_string());
    }
    pub fn name(&self, id: u32) -> Option<&str> {
        self.by_id.get(&id).map(|s| s.as_str())
    }
    pub fn id(&self, name: &str) -> Option<u32> {
        self.by_name.get(name).copied()
    }
}

/// Text reader: parses all top-level expressions. Each expression is returned as the pair chain
/// the binary loader would produce (a list for `(...)`, a one-element chain for an atom).
/// `#t #f`, integers, floats (with `.`), `'x` = `(quote x)`, `#v(x y z)`, `#m(16 floats)`,
/// `#clq(a b c)`, `#lm(a b)`, `#smp(a b c)`, `#bif(n)`, `#obj(n)`, `;` comments.
pub fn read_text(src: &str, syms: &mut Symbols) -> Result<Vec<Ref>, ReadError> {
    let toks = tokenize(src);
    let mut pos = 0;
    let mut out = Vec::new();
    while pos < toks.len() {
        out.push(parse(&toks, &mut pos, syms)?);
    }
    Ok(out)
}

fn tokenize(src: &str) -> Vec<String> {
    let mut toks = Vec::new();
    let mut cur = String::new();
    let mut chars = src.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            ';' => {
                for c in chars.by_ref() {
                    if c == '\n' {
                        break;
                    }
                }
            }
            '(' | ')' | '\'' if !(ch == '(' && cur.starts_with('#')) => {
                if !cur.is_empty() {
                    toks.push(std::mem::take(&mut cur));
                }
                toks.push(ch.to_string());
            }
            '(' => {
                cur.push('(');
                toks.push(std::mem::take(&mut cur));
            }
            c if c.is_whitespace() => {
                if !cur.is_empty() {
                    toks.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        toks.push(cur);
    }
    toks
}

fn parse(toks: &[String], pos: &mut usize, syms: &mut Symbols) -> Result<Ref, ReadError> {
    let t = toks.get(*pos).ok_or(ReadError::Eof)?.clone();
    *pos += 1;
    match t.as_str() {
        "(" => {
            let mut items = Vec::new();
            loop {
                match toks.get(*pos).map(|s| s.as_str()) {
                    None => return Err(ReadError::Eof),
                    Some(")") => {
                        *pos += 1;
                        break;
                    }
                    _ => items.push(parse(toks, pos, syms)?),
                }
            }
            Ok(list_from(items))
        }
        ")" => Err(ReadError::Syntax("unexpected )".into())),
        "'" => {
            let q = parse(toks, pos, syms)?;
            Ok(list_from(vec![Ref::Tag(TAGK_QUOTE), q]))
        }
        _ if t.starts_with('#') && t.ends_with('(') => {
            let mut nums = Vec::new();
            loop {
                let s = toks.get(*pos).ok_or(ReadError::Eof)?;
                *pos += 1;
                if s == ")" {
                    break;
                }
                nums.push(s.parse::<f64>().map_err(|_| ReadError::Syntax(format!("bad number {s}")))?);
            }
            let f = |n: usize| -> Result<Vec<f32>, ReadError> {
                if nums.len() != n {
                    return Err(ReadError::Syntax(format!("{t} needs {n} values")));
                }
                Ok(nums.iter().map(|&x| x as f32).collect())
            };
            Ok(match t.as_str() {
                "#v(" => Ref::Vector(f(3)?.try_into().unwrap()),
                "#m(" => Ref::Matrix(f(16)?.try_into().unwrap()),
                "#clq(" => Ref::Clq(f(3)?.try_into().unwrap()),
                "#lm(" => Ref::Lm(f(2)?.try_into().unwrap()),
                "#smp(" => Ref::Smp(f(3)?.try_into().unwrap()),
                "#bif(" => Ref::Bifk(f(1)?[0] as u32),
                "#obj(" => Ref::Basic(f(1)?[0] as u32),
                _ => return Err(ReadError::Syntax(format!("unknown literal {t}"))),
            })
        }
        "#t" => Ok(Ref::Bool(true)),
        "#f" => Ok(Ref::Bool(false)),
        _ => {
            if let Some((_, tag)) = FORM_NAMES.iter().find(|(n, _)| *n == t) {
                return Ok(Ref::Tag(*tag));
            }
            if let Ok(n) = t.parse::<i32>() {
                return Ok(Ref::S32(n));
            }
            if t.contains('.') || t.contains('e') {
                if let Ok(g) = t.parse::<f32>() {
                    return Ok(Ref::F32(g));
                }
            }
            Ok(Ref::Symid(syms.intern(&t)))
        }
    }
}

/// Printer (inverse of `read_text` for data the reader can produce).
pub fn print_ref(r: &Ref, syms: &Symbols) -> String {
    let fl = |a: &[f32]| a.iter().map(|g| fmt_f32(*g)).collect::<Vec<_>>().join(" ");
    match r {
        Ref::Nil => "#nil".into(),
        Ref::Tag(TAGK_NONE) => "()".into(),
        Ref::Tag(t) => FORM_NAMES.iter().find(|(_, x)| x == t).map(|(n, _)| n.to_string()).unwrap_or(format!("#tag{t}")),
        Ref::S32(n) => n.to_string(),
        Ref::F32(g) => fmt_f32(*g),
        Ref::Bool(b) => if *b { "#t" } else { "#f" }.into(),
        Ref::Symid(s) => syms.name(*s).map(str::to_string).unwrap_or(format!("#sym{s}")),
        Ref::Bifk(k) => format!("#bif({k})"),
        Ref::Vector(v) => format!("#v({})", fl(v)),
        Ref::Matrix(m) => format!("#m({})", fl(m)),
        Ref::Clq(v) => format!("#clq({})", fl(v)),
        Ref::Lm(v) => format!("#lm({})", fl(v)),
        Ref::Smp(v) => format!("#smp({})", fl(v)),
        Ref::Pair(p) => {
            let items = list_items(Some(p.clone()));
            format!("({})", items.iter().map(|i| print_ref(i, syms)).collect::<Vec<_>>().join(" "))
        }
        Ref::Proc(_) => "#proc".into(),
        Ref::Basic(b) => format!("#obj({b})"),
        Ref::Method(_) => "#method".into(),
        Ref::Err => "#err".into(),
        Ref::Void => "#void".into(),
    }
}

fn fmt_f32(g: f32) -> String {
    let s = format!("{g:?}");
    if s.contains('.') || s.contains('e') || s.contains("inf") || s.contains("NaN") { s } else { format!("{s}.0") }
}
