//! WIRE-EP8: the binocucom recon (photo) targets of a level (class 46 C2d9f70) wired to the job manager.
//! `ui::binoc_recon::BinocRecon` (1:1 FUN_00202768 / FUN_00202d38 ...) runs on these targets; its
//! FUN_002036e8 completion goes to `JobReconHost::job_complete` = FUN_00164640(target +0xec task, 2).
//! Target props (field -> offset, field table 0x2ac880 word 2): 1195 +0xe0, 1196 +0xe4, 1198 +0xf4,
//! 1200 +0xfc, 1201 +0xec (task oid).
//! REPLACED: the C2d9f70 ctor defaults are not ported: +0x38 mrd = 1e6 (always in render range), +0xe8 zoom
//! tolerance = 1.0, +0xf8 list id -1, +0x100 no link; FUN_0019d948(jt + 0xf24) of FUN_002036e8 (player
//! binocucom refresh) and the HUD screen objects are not modelled (`JobReconHost` screen calls are no-ops,
//! animations never playing).
use crate::engine::jobs::JobMgr;
use crate::engine::ui::binoc_recon::{ReconHost, ReconTarget, V3};
use crate::engine::world::props::PropValue;
use crate::engine::world::World;

/// A level's recon targets: (object index, task index in `JobMgr::tasks`, target)
pub fn recon_targets(w: &World, m: &JobMgr, pos: &mut dyn FnMut(usize) -> V3) -> Vec<(usize, Option<usize>, ReconTarget)> {
    let mut out = Vec::new();
    for (i, o) in w.objects.iter().enumerate().filter(|(_, o)| o.class == 46) {
        let mut t = ReconTarget { active: true, mrd: 1.0e6, tol_e8: 1.0, id_f8: -1, snd_fc: -1, ..Default::default() };
        let mut task = None;
        for p in crate::engine::game::fields::object_props(o).unwrap_or(&[]) {
            let PropValue::Scalar(b) = &p.value else { continue };
            let wd = |b: &[u8]| if b.len() >= 4 { u32::from_le_bytes([b[0], b[1], b[2], b[3]]) } else if b.len() >= 2 { u16::from_le_bytes([b[0], b[1]]) as i16 as i32 as u32 } else { 0 };
            let v = wd(b);
            match p.field {
                1195 => t.r_e0 = f32::from_bits(v),
                1196 => t.r_e4 = f32::from_bits(v),
                1198 => t.f_f4 = v as i32,
                1200 => t.snd_fc = v as i32,
                1201 => task = m.tasks.iter().position(|k| k.oid == v as i32),
                _ => {}
            }
        }
        t.pos = pos(i);
        t.job = task.map(|k| m.tasks[k].state);
        out.push((i, task, t));
    }
    out
}

/// ReconHost over the job manager (screens / sounds / world pause not modelled).
pub struct JobReconHost<'a> {
    pub m: &'a mut JobMgr,
    /// task index per recon target
    pub tasks: Vec<Option<usize>>,
    pub t: f32,
    pub eye: V3,
    pub flags: u32,
    /// the photo button this frame (X / triangle, FUN_001c46c0 pressed)
    pub press: bool,
}

impl ReconHost for JobReconHost<'_> {
    fn clock(&self) -> f32 { self.t }
    fn clock_scale(&mut self, _: f32) {}
    fn elem_hide(&mut self, _: u32, _: i32) {}
    fn elem_show(&mut self, _: u32, _: i32) {}
    fn elem_set(&mut self, _: u32, _: i32, _: i32) {}
    fn elem_scrub(&mut self, _: u32, _: i32, _: f32) {}
    /// FUN_002121e8: the open animation 0x314 is reported played (state 0 -> 1), the photo / done ones finished
    fn elem_playing(&mut self, _: u32, e: i32) -> bool { e == 0x314 }
    fn screen_214300(&mut self, _: u32, _: i32) {}
    fn screen_msg(&mut self, _: u32, _: i32) {}
    fn screen_close(&mut self, _: u32) {}
    fn screen_state(&self, _: u32) -> i32 { 2 }
    fn screen_off(&mut self, _: u32) {}
    fn screen_set_target(&mut self, _: u32, _: Option<usize>) {}
    fn screen_word(&self, _: u32, _: u32) -> i32 { -1 }
    /// FUN_001619a0(0x2deb30, msg, 0): job event (9 open, 10 photo done, 11 closed)
    fn game_msg(&mut self, msg: i32) { self.m.FUN_001619a0(msg, 0); }
    fn world_pause(&mut self, _: bool) {}
    fn music_1fe5b8(&mut self, _: i32) {}
    fn cam_preset_off(&mut self) {}
    fn getter_call(&mut self, _: i32) {}
    fn obj_msg(&mut self, _: u32, _: i32, _: i32) {}
    fn sound(&mut self, _: i32) {}
    fn bn_flags(&self) -> u32 { self.flags }
    fn set_bn_flags(&mut self, v: u32) { self.flags = v; }
    fn bn_eye(&self) -> V3 { self.eye }
    fn bn_mode(&self) -> i32 { 0 }
    fn bn_19cce8(&mut self) {}
    fn revoke(&mut self) -> bool { false }
    fn joy_take_pressed(&mut self, _: u32) -> bool { std::mem::take(&mut self.press) }
    fn ray_blocked(&mut self, _: usize, _: V3, _: V3) -> bool { false }
    fn sensor_contains(&mut self, _: V3) -> bool { false }
    fn link_4c(&mut self, _: usize) {}
    fn link_44(&mut self, _: usize) {}
    fn link_1f7bf8(&mut self, _: usize, _: f32) {}
    fn link_1f7c68(&mut self, _: usize, _: f32) {}
    /// FUN_00164030(+0xec): task done (state >= 2)
    fn job_done(&mut self, t: usize) -> bool {
        self.tasks.get(t).copied().flatten().is_some_and(|k| self.m.tasks[k].state >= 2)
    }
    /// FUN_002036e8 job branch: FUN_00164640(+0xec, 2)
    fn job_complete(&mut self, t: usize) {
        if let Some(k) = self.tasks.get(t).copied().flatten() {
            self.m.FUN_00164640(Some(k), 2, 0);
        }
    }
}
