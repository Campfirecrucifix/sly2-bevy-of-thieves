//! PORT2: the script method rows on the music manager 0x3dd040 that call into `audio::music_mgr::MusicMgr`:
//!  - 0x00102538 (getter, field 148, F32): `FUN_001fe3f8(0x3dd040)` play time of the top music instance.
//!  - 0x001025a8 (method, 1 arg kind 1): `FUN_001fe9f8(vol, 0x3dd040)` master volume; returns Void (tag 0x24).
//!  - 0x00102630 (method, 2 args kinds DAT_002a3b10 = {1, 1}): `FUN_001fea60(vol, 0x3dd040, bus, 0)`
//!    volume matrix entry (slot 0); returns Void.
//!
//! REPLACED (timing only): the scripts run inside the splice VM, which does not own the Bevy `MusicMgr` /
//! `EmitterSystem` / `Snd989` resources. The setter calls are queued on `ObjectFields::music_link` and
//! executed by the real ported functions in [`apply_music_link`] (same frame, after the script tick);
//! the getter reads the play time `apply_music_link` sampled with the real `ee_sub_1fe3f8` last frame.

use crate::engine::audio::emitter::EmitterSystem;
use crate::engine::audio::emitter_tick::Snd989;
use crate::engine::audio::music_mgr::MusicMgr;
use bevy_ecs::prelude::*;

/// A queued music-manager call of a script method row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MusicCall {
    /// 0x001025a8 -> FUN_001fe9f8(vol, 0x3dd040)
    Master(f32),
    /// 0x00102630 -> FUN_001fea60(vol, 0x3dd040, bus, slot)
    Matrix { vol: f32, bus: i64, slot: u64 },
}

/// Script-side link to the music manager (kept in `ObjectFields`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MusicLink {
    pub calls: Vec<MusicCall>,
    /// last `MusicMgr::ee_sub_1fe3f8` value (seconds)
    pub time: f32,
}

impl MusicLink {
    /// 0x001025a8: FUN_00201360(args, 1, argv, {1}) -> vol; FUN_001fe9f8(vol, obj).
    pub fn ee_sub_1025a8(&mut self, vol: f32) {
        // 0x001025a8
        self.calls.push(MusicCall::Master(vol));
    }
    /// 0x00102630: FUN_00201360(args, 2, argv, {1, 1}) -> (bus, vol); FUN_001fea60(vol, obj, bus, 0).
    pub fn ee_sub_102630(&mut self, bus: i32, vol: f32) {
        // 0x00102630
        self.calls.push(MusicCall::Matrix { vol, bus: bus as i64, slot: 0 });
    }
    /// 0x00102538: FUN_001fe3f8(obj) wrapped as F32 (FUN_00129558).
    pub fn ee_sub_102538(&self) -> f32 {
        // 0x00102538
        self.time
    }

    /// Run the queued calls on the music manager with the ported callees, then sample the play time.
    pub fn apply(&mut self, m: &mut MusicMgr, es: &mut EmitterSystem, snd: &mut Snd989) {
        for c in std::mem::take(&mut self.calls) {
            match c {
                MusicCall::Master(v) => m.ee_sub_1fe9f8(snd, v),
                MusicCall::Matrix { vol, bus, slot } => m.ee_sub_1fea60(es, snd, vol, bus, slot),
            }
        }
        self.time = m.ee_sub_1fe3f8(es);
    }
}

/// Bevy system: drains `ObjectFields::music_link` into the live `MusicMgr` (registered by `app`).
pub fn apply_music_link(
    st: Option<NonSendMut<super::vm_state::SpliceState>>,
    mgr: Option<ResMut<MusicMgr>>,
    es: Option<ResMut<EmitterSystem>>,
    snd: Option<ResMut<Snd989>>,
) {
    let (Some(mut st), Some(mut m), Some(mut es), Some(mut snd)) = (st, mgr, es, snd) else { return };
    st.sched.objs.music_link.apply(&mut m, &mut es, &mut snd);
}

/// FUN_001fb4f0(near, far, e) (method 0x00101f90 on a class-151 emitter): +0x34 far = far <= near ?
/// near + 1 : far, +0x30 near = near (same body as `classes::k25::SoundRt::ee_sub_1fb4f0`, on the
/// script-level emitter pool).
pub fn ee_sub_1fb4f0(es: &mut EmitterSystem, near: f32, far: f32, id: Option<usize>) {
    // 0x001fb4f0
    if let Some(e) = id.and_then(|i| es.get_mut(i)) {
        e.far = if far <= near { near + 1.0 } else { far };
        e.near = near;
    }
}
