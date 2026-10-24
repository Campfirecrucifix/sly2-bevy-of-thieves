//! WP-P23: Bentley / Murray anim fields and the mid-mission character switch callers.
//!
//! 1:1: FUN_001a64e8 (bind table rows {flags, oid, offset}: FUN_0015e938 mode 4 (flags & 2) / 5 (flags & 1) /
//! 1, | 0x400 when flags & 0x400, flags & 0x200 activates the found object through vtable +0x4c, the handle is
//! stored at self + offset) over the slot-13 tables of Bentley (FUN_00134770, DAT_002bc390, 0x76 rows) and
//! Murray (FUN_001b4888, DAT_002c0270, 0x89 rows). The class code then finds its ASEG / SM handles
//! (+0x1028 SM 0x1a2, +0x1334 / +0x1374 run entries, +0x14e4 ..., +0x1820 ...) and its SetState requests
//! ASEGAs / SMAs on the K25 run time (`classes::k25::AnimRt`), which `playerdraw::po` draws.
//! REPLACED: the bindings are written by the player layer (`bind_all`, run before the po selection) because the
//! `ClassHost::ee_sub_1a64e8` hook has no access to the class memory; the class's post-load SetState(0) ran
//! before them, so `bind_all` re-enters state 0 (state -1 -> SetState(0), the slot-14 order bind -> SetState).
//! UNPORTED: rows with flags & 0x10 (ALO +0x54 records), & 0x20 (FUN_001f6030, ALO +0xe4 records of 0xb0
//! bytes, + FUN_001f2ad0) and & 0x40 (FUN_001f5ff8 / FUN_001f69f8): the port has no model of those ALO
//! record arrays (they are 0x25d / 0x25e / 0x284.. / 0x297.. / 0x1ee.. rows: sound / effect parts).
use super::p20::{CLASS_BENTLEY, CLASS_MURRAY};
use crate::engine::classes::{ClassHost, ClassSet};
use crate::engine::playerdraw::vis::{PO_BIND_2BC390, PO_BIND_2C0270};
use crate::engine::world::World;

/// ANIM R25: FUN_001a0300 plain blends of a Bentley / Murray: (out field, first entry ASEG member; 8 entries
/// stride 0x20) - Bentley FUN_001347a8 (+0x13e0 -> +0x14e0, +0x14f0 -> +0x15f0), Murray (+0x16f0 -> +0x17f0).
pub fn plain_fields(class: u16) -> &'static [(u32, u32)] {
    match class {
        CLASS_BENTLEY => &[(0x14e0, 0x13e4), (0x15f0, 0x14f4)],
        CLASS_MURRAY => &[(0x17f0, 0x16f4)],
        _ => &[],
    }
}

/// ANIM R25: the plain blend the po's +0x1030 slot plays now: Bentley SetState 5 (FUN_00135248: +0x14e0,
/// flags 9) / state-6 sub-state 1 (FUN_00136040: +0x15f0), Murray state-9 sub-state 2 (FUN_001b5d30: +0x17f0).
/// REPLACED: the blend is not a run-time ASEG handle (the class layer's FUN_001a0300 hook builds nothing), so the
/// request is read back from the class state.
pub fn po_plain(set: &mut crate::engine::classes::ClassSet, o: usize, class: u16) -> Option<u32> {
    match class {
        CLASS_BENTLEY => {
            let b = set.k16_get(o)?;
            match b.state {
                5 => Some(0x14e0),
                6 if b.f_16b0 == 1 => Some(0x15f0),
                _ => None,
            }
        }
        CLASS_MURRAY => {
            let b = &set.k18_murray(o)?.b;
            (b.state == 9 && b.m.i(0x17f4) == 2).then_some(0x17f0)
        }
        _ => None,
    }
}

/// Slot-13 FUN_001a64e8 table of a Bentley / Murray class.
pub fn bind_table(class: u16) -> &'static [(u32, i32, u32)] {
    match class {
        CLASS_BENTLEY => &PO_BIND_2BC390,
        CLASS_MURRAY => &PO_BIND_2C0270,
        _ => &[],
    }
}

/// FUN_001a64e8(obj, n, table): resolved rows as (offset, handle); rows the port cannot resolve are skipped.
pub fn ee_sub_1a64e8(obj: usize, table: &[(u32, i32, u32)], h: &mut dyn ClassHost) -> Vec<(u32, u32)> {
    let mut out = vec![];
    for &(flags, oid, off) in table {
        if flags & 0x70 != 0 {
            continue;
        }
        let mut mode = if flags & 2 != 0 {
            4
        } else if flags & 1 != 0 {
            5
        } else {
            1
        };
        if flags & 0x400 != 0 {
            mode |= 0x400;
        }
        let r = h.ee_sub_15e938(mode, oid, obj as u32 + 1);
        if r == 0 {
            continue;
        }
        if flags & 0x200 != 0 {
            h.vcall(r, 0x4c);
        }
        out.push((off, r));
    }
    out
}

/// Bind every Bentley / Murray of the set once (rows already written are left alone) and re-enter state 0 so
/// the post-load SetState(0) sees its SM. Returns the objects bound now.
pub fn bind_all(w: &World, set: &mut ClassSet, h: &mut dyn ClassHost) -> Vec<usize> {
    let mut done = vec![];
    let objs: Vec<(usize, u16)> = w.objects.iter().enumerate().filter(|(_, o)| o.class == CLASS_BENTLEY || o.class == CLASS_MURRAY).map(|(i, o)| (i, o.class)).collect();
    for (o, class) in objs {
        let rows = ee_sub_1a64e8(o, bind_table(class), h);
        // slot 14 base JtVis_load_1400b8: FUN_001a64e8(self, 0x35, 0x2bc9f0) (carry-machine ASEGs +0x11b0 / +0x1910 ...)
        let base_rows = ee_sub_1a64e8(o, &crate::engine::playerdraw::vis::JT_BIND_2BC9F0, h);
        let mem = match class {
            CLASS_BENTLEY => set.k16_get(o).map(|b| &mut b.m),
            _ => set.k18_murray(o).map(|m| &mut m.b.m),
        };
        let Some(m) = mem else { continue };
        // +0x1028 (SM 0x1a2, first row of both tables) set = already bound
        if m.u(0x1028) != 0 {
            continue;
        }
        for (off, r) in rows.into_iter().chain(base_rows) {
            if (off as usize) / 4 < m.0.len() {
                m.set_u(off, r);
            }
        }
        crate::engine::player::p26::ee_sub_21f388_k(m, if class == CLASS_BENTLEY { 0x11b0 } else { 0x1910 });
        match class {
            CLASS_BENTLEY => {
                set.k18_with(o, h, |c, cx| {
                    c.state = -1;
                    c.ee_sub_135248(o, 0, cx.g, cx.rng, cx.h);
                });
            }
            _ => {
                set.k19_murray_with(o, h, |c, cx| {
                    c.b.state = -1;
                    c.ee_sub_1b6078(o, 0, cx);
                });
            }
        }
        done.push(o);
    }
    done
}

/// Live ASEGAs of the run time whose target is object `po` (ASEG object, time), oldest first.
pub fn po_plays(rt: &crate::engine::classes::k25::AnimRt, po: usize) -> Vec<(usize, f32)> {
    let mut v: Vec<(u32, usize, f32)> = rt.asegas.iter().filter(|(_, a)| a.obj == po as u32 + 1 && a.st.alive).map(|(h, a)| (*h, a.aseg, a.st.t)).collect();
    v.sort_by_key(|x| x.0);
    v.into_iter().map(|x| (x.1, x.2)).collect()
}

// ---------------------------------------------------------------- WP-P23 item 2: mid-mission switch callers

/// The class code's character-switch requests of one class frame, queued by the class host
/// (`EngineHost::vcall_arg` +0x27c, `EngineHost::po_part_s` / `po_call` 0x2206c8) and applied by the player
/// layer (`apply_switches`, `player::plugin::po_link`) with the class set in hand.
#[derive(Default, Clone, Debug)]
pub struct SwitchQueue {
    /// (vtable +0x27c)(obj, k): slot 78 (object handle = index + 1, k)
    pub slot78: Vec<(u32, i32)>,
    /// FUN_002206c8(obj + off, s): the carry machine of a Bentley (+0x11b0) / Murray (+0x1910)
    pub carry: Vec<(usize, u32, i32)>,
}

/// FUN_002206c8 slot-78 part: the carry machine at `off` of Bentley / Murray `obj` goes from `prev` to `s`.
/// Returns None for `s == prev` (return), else the slot-78(1) calls in order: leaving state 3 for any state
/// but 4 gives DAT_002dd5bc (the first Bentley) slot 78(1), entering state 3 gives the owner slot 78(1).
pub fn ee_sub_2206c8_switch(prev: i32, s: i32, owner: usize, bentley: Option<usize>) -> Option<Vec<usize>> {
    if s == prev {
        return None;
    }
    let mut v = vec![];
    if prev == 3 && s != 4 {
        v.extend(bentley);
    }
    if s == 3 {
        v.push(owner);
    }
    Some(v)
}

/// Apply a frame's `SwitchQueue`: carry machine state words (+off+4 = s, +off+8 = DAT_002dded0) and the
/// slot-78 calls on the player characters: Bentley / Murray k == 1 -> `p22::set_active_character(Some(o))`
/// (slot 78 FUN_001406e8(o, 1) = `k18_set_active`), k <= 0 on the controlled one -> released to Sly; Sly
/// (class 53) k == 1 -> the controlled Bentley / Murray is released (Sly's JT is the player layer's).
/// Returns the objects made / released active, in order.
pub fn apply_switches(w: &World, set: &mut ClassSet, h: &mut dyn ClassHost, q: SwitchQueue) -> Vec<Option<usize>> {
    let bentley = w.objects.iter().position(|o| o.class == CLASS_BENTLEY);
    let mut calls: Vec<(u32, i32)> = vec![];
    let t = set.g.t;
    let dt = set.g.dt;
    let bh = bentley.map_or(0, |b| b as u32 + 1);
    // WP-P26: the full carry machine FUN_002206c8 (player::p26) runs on the class memory
    let b1800 = bentley.and_then(|b| set.k16_get(b).map(|x| x.m.i(0x1800))).unwrap_or(1);
    for (o, off, s) in q.carry {
        let mem = match w.objects.get(o).map(|x| x.class) {
            Some(CLASS_BENTLEY) => set.k16_get(o).map(|b| &mut b.m),
            Some(CLASS_MURRAY) => set.k18_murray(o).map(|m| &mut m.b.m),
            _ => None,
        };
        let Some(m) = mem else { continue };
        // carried [4] +0x10 / reference [3] +0x10 (FUN_002206c8 -1: same parent -> the carried is dropped, state 4)
        let (carried, refo) = (m.u(off + 0x10), m.u(off + 0xc));
        let (carried_10, ref_10) = (h.parent_10(carried), h.parent_10(if refo != 0 { refo } else { o as u32 + 1 }));
        let cx = super::p26::CarryCx { owner: o as u32 + 1, bentley: bh, bentley_1800: b1800, t, dt, carried_10, ref_10, ..Default::default() };
        let Some(r) = super::p26::ee_sub_2206c8(m, off, s, &cx, h, &mut |_, _| None) else { continue };
        // POOL1: run the machine's anim play (FUN_001305b8(0, 1.0, anim, owner, mode, owner + 0x1030)) and the
        // carried's vt +0x264 state calls (pool copies: FUN_00224ec8)
        for &(a, x, k) in &r.calls {
            match a {
                0x001305b8 if x != 0 => {
                    let mut sl = m.u(0x1034);
                    h.ee_sub_1f15b8(&mut sl);
                    m.set_u(0x1034, sl);
                    let mut a = m.u(0x1030);
                    crate::engine::classes::shared::anim::ee_sub_1305b8(0, 1.0, x, o as u32 + 1, k as u32, &mut a, h);
                    m.set_u(0x1030, a);
                    h.vcall(o as u32 + 1, 0x2d4);
                }
                0x264 if x != 0 => h.vcall_arg(x, 0x264, k),
                _ => {}
            }
        }
        // the leave-3 call precedes the state write in the original; both precede the enter-3 call
        calls.extend(r.slot78.into_iter().map(|x| (x, 1)));
    }
    calls.extend(q.slot78);
    let mut out = vec![];
    for (hdl, k) in calls {
        let Some(o) = (hdl as usize).checked_sub(1).filter(|&o| o < w.objects.len()) else { continue };
        match w.objects[o].class {
            CLASS_BENTLEY | CLASS_MURRAY if k == 1 => {
                super::p22::set_active_character(w, set, h, Some(o));
                out.push(Some(o));
            }
            CLASS_BENTLEY | CLASS_MURRAY if set.k18_active == Some(o) => {
                super::p22::set_active_character(w, set, h, None);
                out.push(None);
            }
            super::p20::CLASS_SLY if k == 1 && set.k18_active.is_some() => {
                super::p22::set_active_character(w, set, h, None);
                out.push(None);
            }
            // WIRE-MG: class 69 (C2dbd80 turret / scope) inherits C2d5898 slot 78 FUN_001c3ed8: k == 1 makes it
            // the DAT_003e138c focus (old focus +0x27c(0), FUN_001c4940 -> +0x274(1) = slot 77 camera policy 9 on);
            // k <= 0 on the focus drops it (FUN_001c4940(0) -> +0x274(0))
            69 if k == 1 && set.mg_focus != Some(o) => {
                if set.k18_active.is_some() {
                    super::p22::set_active_character(w, set, h, None);
                }
                set.mg_focus = Some(o);
                if let Some(crate::engine::classes::Inst::K22C69(c)) = set.k22c(o) {
                    c.ee_sub_225ae8(o, true, h);
                }
            }
            69 if k <= 0 && set.mg_focus == Some(o) => {
                set.mg_focus = None;
                if let Some(crate::engine::classes::Inst::K22C69(c)) = set.k22c(o) {
                    c.ee_sub_225ae8(o, false, h);
                }
            }
            // other classes' slot 78 is their own (vault class 78, class 70 FUN_001d7418): no switch
            _ => {}
        }
    }
    out
}

// ---------------------------------------------------------------- WP-P23 item 3: player anim events

/// Timed events of `aseg` reached by a forward play from `t0` to `t1` (FUN_00130f88 cursor order): times in
/// (t0, t1], or for a wrap (t1 < t0, looped through the end) (t0, length] then [0, t1]. `t0 < 0` = a new
/// play (events at time 0 included). Returns event indices in firing order.
pub fn events_between(aseg: &crate::engine::anim::aseg::Aseg, t0: f32, t1: f32) -> Vec<usize> {
    let timed: Vec<(usize, f32)> = aseg.events1.iter().enumerate().filter_map(|(i, e)| e.time.map(|t| (i, t))).collect();
    window(&timed, t0, t1)
}

/// `events_between` over explicit (index, time) pairs.
pub fn window(timed: &[(usize, f32)], t0: f32, t1: f32) -> Vec<usize> {
    let sorted = |mut v: Vec<(usize, f32)>| {
        v.sort_by(|x, y| x.1.total_cmp(&y.1));
        v.into_iter().map(|x| x.0)
    };
    if t1 >= t0 {
        return sorted(timed.iter().copied().filter(|&(_, t)| t > t0 && t <= t1).collect()).collect();
    }
    let mut a: Vec<usize> = sorted(timed.iter().copied().filter(|&(_, t)| t > t0).collect()).collect();
    a.extend(sorted(timed.iter().copied().filter(|&(_, t)| t <= t1).collect()));
    a
}

/// FUN_00137a30 event source (+0xd8) of a class-0x6e blend: the first entry ASEG with any of its three event
/// lists (+0x98 / +0xa8 / +0xa0) non-empty; the blend shares its lists.
pub fn blend_event_src(w: &crate::engine::world::World, b: &crate::engine::playerdraw::jtblend::JtBlend) -> Option<usize> {
    b.entries.iter().filter_map(|e| e.aseg).find(|&o| crate::engine::playerdraw::aseg_of(w, o).is_some_and(|a| !a.events0.is_empty() || !a.events2.is_empty() || !a.events1.is_empty()))
}

/// Timed events of blend `b` reached from `t0` to `t1` (`events_between` order): FUN_001384c0 rescales the
/// shared +0xa4 times by blend length / source length. Returns (source ASEG object, event index).
pub fn blend_events_between(w: &crate::engine::world::World, b: &crate::engine::playerdraw::jtblend::JtBlend, t0: f32, t1: f32) -> Vec<(usize, usize)> {
    let Some(src) = blend_event_src(w, b) else { return vec![] };
    let a = crate::engine::playerdraw::aseg_of(w, src).expect("aseg");
    let k = if a.length > 0.0 { b.length / a.length } else { 1.0 };
    let timed: Vec<(usize, f32)> = a.events1.iter().enumerate().filter_map(|(i, e)| e.time.map(|t| (i, t * k))).collect();
    window(&timed, t0, t1).into_iter().map(|i| (src, i)).collect()
}

/// FUN_00131290 for fired events `(index, reverse)` of `aseg` into `sink` (+0x1c = the event time, as
/// anim::play::fire sets it). Returns the effect calls made (kind 6 retract has none).
/// event index of a `dispatch_events` entry meaning the whole retract list (+0xa8, `Aseg::events2`)
pub const RETRACT_EVENTS: usize = usize::MAX;
/// HEIST r16: event index meaning the whole start list (+0x98, `Aseg::events0`) FUN_0012f9b8 fires at creation
pub const START_EVENTS: usize = usize::MAX - 1;
pub fn dispatch_events(aseg: &crate::engine::anim::aseg::Aseg, fired: &[(usize, bool)], sink: &mut dyn crate::engine::anim::events::EffectSink) -> usize {
    use crate::engine::anim::play::Asega;
    let mut n = 0;
    for &(i, rev) in fired {
        // R53: `RETRACT_EVENTS` = the +0xa8 list FUN_00130b18 fires when the ASEGA is retracted
        let evs: &[crate::engine::anim::aseg::Event] = if i == RETRACT_EVENTS { &aseg.events2 } else if i == START_EVENTS { &aseg.events0 } else { aseg.events1.get(i..=i).unwrap_or(&[]) };
        for ev in evs {
            let mut a = Asega { t: ev.time.unwrap_or(0.0), ..Default::default() };
            if let Some(c) = crate::engine::anim::events::ee_sub_131290(&mut a, ev, rev) {
                sink.call(&mut a, c);
                n += 1;
            }
        }
    }
    n
}
