//! WP-P15: runtime animation blends of the climb / attach states, as the JT builds them at load and the state
//! code re-weights them every frame (consumed by playerdraw::plugin::drive_sly; plain functions so tests can
//! drive a `JtAnimator` without the game feature).
//! 1:1 data path: FUN_00137a30 / FUN_0020b868 / FUN_0020bb58 / FUN_001384c0 are the playerdraw::jtblend ports.

use crate::engine::playerdraw::JtAnimator;
use crate::engine::world::World;

/// WP-P15: jts 8 climb blend jt+0xfc0 (state 2/3 ASEG of FUN_0019eab0, piVar1[0x1c]): FUN_00137a30 over the
/// two entries s+0x50 / s+0x60 (stride 0x10, ASEGs jt+0xfa4 / jt+0xfb4), built at JT load next to the
/// playerdraw::blend_fields blends (FieldBuild::Plain, no phase tables).
pub const FIELD_PIPE_CLIMB: u32 = 0xfc0;
/// WP-P15: attach blends of FUN_001910f8 (FUN_0020b868(b), field b+4, entries b+0x14 / b+0x54): hide a+0x180
/// (jt+0x640) and rope a+0x240 (jt+0x700); 0x192bd8 sub 4 plays a+0x184 / a+0x244 (gadget a+0x1c30 jt-relative =
/// FIELD_RUN2, already a blend_fields entry).
pub const FIELD_ATT_HIDE: u32 = 0x644;
pub const FIELD_ATT_ROPE: u32 = 0x704;
/// WP-P17 (port addition, verification): env `SLY2_ANIM_LOG` prints the anim chain every frame.
pub fn anim_log() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("SLY2_ANIM_LOG").is_some())
}

pub fn jt_blends_build(w: &World, an: &mut JtAnimator) {
    if !an.built {
        an.blends = an.rig.build_blends(w);
        an.built = true;
    }
    if an.blends.contains_key(&FIELD_PIPE_CLIMB) {
        return;
    }
    let m = |f: u32| an.rig.members.get(&f).copied();
    if let Some(b) = crate::engine::playerdraw::jtblend::ee_sub_137a30(w, &an.rig, &[m(0xfa4), m(0xfb4)], &[]) {
        an.blends.insert(FIELD_PIPE_CLIMB, b);
    }
    for f in [FIELD_ATT_HIDE, FIELD_ATT_ROPE] {
        if let Some(b) = crate::engine::playerdraw::jtblend::ee_sub_20b868(w, &an.rig, [m(f + 0x10), m(f + 0x50)]) {
            an.blends.insert(f, b);
        }
    }
}

/// WP-P15: blend weights of the states that play a runtime blend.
/// jts 8: 0x19e1e0 writes the entry weights s+0x50 / s+0x60 (Jt::p14.wt) then FUN_001384c0(jt+0xfc0) (time kept
/// at the same normalised position); FUN_0019eab0 starts it with svt +1 (state 2) / -1 (state 3).
/// jts 7: FUN_0020bb58(w, blend) (p5_attach::FUN_0020bb58 -> AttachExt hide/rope/gadget weights).
pub fn jt_blends_drive(an: &mut JtAnimator, jt: &super::Jt) {
    if anim_log() {
        let cur = match &an.current {
            Some(crate::engine::playerdraw::JtPlay::Blend { field, t, svt }) => format!("blend {field:#x} t {t:.3} svt {svt}"),
            Some(crate::engine::playerdraw::JtPlay::Aseg(o, p)) => format!("aseg obj {o} field {:?} t {:.3}", an.current_field().map(|f| format!("{f:#x}")), p.state.t),
            None => "none".into(),
        };
        let w = an.blends.get(&FIELD_PIPE_CLIMB).map(|b| b.entries.iter().map(|e| e.weight).collect::<Vec<_>>());
        eprintln!("P17 anim: jts {} jtbs {} s8 {} p14.wt {:?} sm {} cur {cur} climb w {w:?}", jt.jts, jt.jtbs, jt.p5.s8.state, jt.p14.wt, an.sm.is_some());
    }
    let Some(crate::engine::playerdraw::JtPlay::Blend { field, t, svt }) = an.current.as_mut() else { return };
    let Some(b) = an.blends.get_mut(field) else { return };
    match (*field, jt.jts) {
        (FIELD_PIPE_CLIMB, 8) => {
            *svt = if jt.p5.s8.state == 3 { -1.0 } else { 1.0 };
            let wt = jt.p14.wt;
            if b.entries.len() == 2 && wt[0] + wt[1] > 0.0 && (b.entries[0].weight != wt[0] || b.entries[1].weight != wt[1]) {
                b.entries[0].weight = wt[0];
                b.entries[1].weight = wt[1];
                b.ee_sub_1384c0(std::slice::from_mut(t));
            }
        }
        // FEEL: 0x18b7e8 case 0xc: FUN_0020bb58(joy +0x1c, jt+0x1c30) walk blend weights
        (crate::engine::playerdraw::FIELD_RUN2, 0xc) => b.ee_sub_20bb58(jt.joy.mag, std::slice::from_mut(t)),
        (FIELD_ATT_HIDE, 7) => b.ee_sub_20bb58(jt.att.hide_blend.w[1], std::slice::from_mut(t)),
        (FIELD_ATT_ROPE, 7) => b.ee_sub_20bb58(jt.att.rope_blend.w[1], std::slice::from_mut(t)),
        (crate::engine::playerdraw::FIELD_RUN2, 7) => b.ee_sub_20bb58(jt.att.gadget_blend.w[1], std::slice::from_mut(t)),
        _ => {}
    }
}

