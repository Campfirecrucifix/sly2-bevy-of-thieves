//! WP-K: gameplay class virtual overrides, 1:1 per class (see tools/engineport/class_port.md).
//! Each class keeps the object fields its overrides touch (named by offset) in a state struct;
//! base-class and engine calls that are not ported elsewhere go through [`ClassHost`] hooks.
pub mod actor;
pub mod guard;
pub mod c2cabd8;
pub mod c2cad78;
pub mod c2ce4d0;
pub mod c2ce828;
pub mod c2ceb78;
pub mod c2cf1e8;
pub mod c2cf378;
pub mod c2d33a0;
pub mod c2d6e08;
pub mod c2d97a8;
pub mod c2da840;
pub mod c2dc6f0;
pub mod c2d8c48;
pub mod c2d5120;
pub mod c2cf058;
pub mod c2cf880;
pub mod c2ca9f0;
pub mod c2d9dd0;
pub mod c2d52b0;
pub mod c2dd090;
pub mod c2ce2f8;
pub mod c2dad90;
pub mod c2d56f8;
pub mod c2d5dc8;
pub mod c2d6fe8;
pub mod c2d9040;
pub mod c2d9978;
pub mod c2db0d0;
pub mod c2db6b8;
pub mod k4;
pub mod k7;
pub mod k8;
pub mod k9;
pub mod k10;
pub mod k11;
pub mod k12;
pub mod k13;
pub mod k14;
pub mod k15;
pub mod k15_101;
pub mod k15_79;
pub mod k15_83;
pub mod k15_85;
pub mod k16_55;
pub mod k17_55;
pub mod k18_55;
pub mod k18_54;
pub mod k19_54;
pub mod k20_54;
pub mod k20_77;
pub mod k20_119;
pub mod k21_119;
pub mod k21_77;
pub mod k21_54;
pub mod k21;
pub mod k21_28;
pub mod k21_68;
pub mod k22;
pub mod k22_96;
pub mod k22_115;
pub mod k22_69;
pub mod k22_121;
pub mod k22_seq;
pub mod k23_102;
pub mod k23_32;
pub mod k24;
mod k24_fwd;
pub mod k25;
pub mod k25_127;
pub mod k26;
pub mod k27;
pub mod k28;
pub mod k28_emit;
pub mod k29_cam;
pub mod k30_cam;
pub mod k31;
pub mod shared;
pub mod plugin;

use crate::engine::physics::driven::GameRng;
use crate::engine::world::loads_b4::B4Load;
use crate::engine::world::objects::{LoadData, Payload};
use crate::engine::world::props::Prop;
use crate::engine::world::{class::ClassId, World};

/// Load-time property list of object `i` (B4 loads that carry FUN_00231170 props).
pub fn props_of(w: &World, i: usize) -> Option<&[Prop]> {
    match &w.objects.get(i)?.payload {
        Payload::Loaded(LoadData::B4(b)) => match b.as_ref() {
            B4Load::L258598 { props, .. } | B4Load::L231170 { props, .. } | B4Load::L15c1e8 { props, .. } => Some(props),
            _ => None,
        },
        _ => None,
    }
}

/// Engine globals the ported overrides read/write.
#[derive(Clone, Debug, Default)]
pub struct ClassGlobals {
    /// K31: DAT_003e1acc (class-37 guard index counter, slot 14 FUN_0013cef8)
    pub g_3e1acc: i32,
    /// DAT_003e1204 (change counter)
    pub g_3e1204: i32,
    /// *(DAT_003e1a5c + 0x28)
    pub g_3e1a5c_28: i32,
    /// DAT_003dd268 (non-zero once the emitter/sound manager exists)
    pub g_3dd268: i32,
    /// DAT_002ddf54 (render mask)
    pub g_2ddf54: u8,
    /// DAT_002c3b90
    pub g_2c3b90: u32,
    /// DAT_002dded0 clock t / DAT_002dded4 clock dt (J: clock 0x2dded0)
    pub t: f32,
    pub dt: f32,
    /// DAT_002ddf5c (LOD distance scale) / DAT_002ddfd0 (LOD reference point, camera)
    pub g_2ddf5c: f32,
    pub g_2ddfd0: [f32; 3],
    /// DAT_003e0f8c (class-133 instance counter), DAT_003e0fa0 (current class-133 object handle), DAT_003e0f9c
    pub g_3e0f8c: i32,
    pub g_3e0fa0: u32,
    pub g_3e0f9c: i32,
    /// K25: DAT_002dd318 (class-127 prompts answered)
    pub g_2dd318: i32,
    /// DAT_002ddf10 (alternate clock dt)
    pub g_2ddf10: f32,
    /// DAT_003e12c8 (light grey filter mode), DAT_003e11ec (static light change counter), DAT_003e19e4
    pub g_3e12c8: i32,
    pub g_3e11ec: i32,
    pub g_3e19e4: i32,
    /// g_lsm (0x3dd490) light table
    pub lights: c2d33a0::LightTable,
    /// DAT_003e1114: warping class-26 object (handle)
    pub g_3e1114: u32,
    /// DAT_002dd5bc: the Bentley object (handle; K17 FUN_00134648)
    pub g_2dd5bc: u32,
    /// DAT_002de240 (K4: class-15 volume override)
    pub g_2de240: i32,
    /// DAT_003e1118 / DAT_003e111c (K4: return level / spawn set by class 132)
    pub g_3e1118: i32,
    pub g_3e111c: i32,
    /// DAT_003e1bf0 (class-7 serial counter)
    pub g_3e1bf0: i32,
    /// DAT_003e13cc list (class-13 objects, FUN_0011a220 append / FUN_0011a2e8 remove)
    pub l_3e13cc: Vec<u32>,
    /// DAT_003e13c0 list (class-11 objects)
    pub l_3e13c0: Vec<u32>,
    /// DAT_003e13d8 list (class-39 objects)
    pub l_3e13d8: Vec<u32>,
    /// K5: DAT_003e1450 list (class-98 objects, slots 21 / 22)
    pub l_3e1450: Vec<u32>,
    /// K6: the player-character globals (DAT_002e1e40 / 2f7900 / 2dd5bc / 2dd648): handle, world position
    pub k6_players: Vec<(u32, [f32; 3])>,
    /// K7: guard list 0x3e1444 (slots 22 / 21)
    pub k7_live: Vec<u32>,
    /// K7: the two class-45 highlight slots DAT_003e1a78 (stride 0x28, +0x20 owner handle)
    pub g_3e1a78: [u32; 2],
    /// K8: DAT_003e1bf4 (clue bottles collected), the current world's "afClue" bits (REPLACED save path of
    /// FUN_001c7b18), loot save var counts ("loot"/"mplootkc"/%d), DAT_003e11e8 transform stamp
    pub g_3e1bf4: i32,
    pub af_clue: Vec<u32>,
    pub loot: std::collections::BTreeMap<i32, i32>,
    pub g_3e11e8: i32,
    /// K9: DAT_003e1bf8 (clue bottles per world, world +0x4768 = FUN_001c7d00), the world's "fVault" save variable
    /// (REPLACED save path of FUN_001c8090; None = no world path), world +0x3d80 / +0x3d84 counters
    pub g_3e1bf8: i32,
    pub f_vault: Option<bool>,
    pub w_3d80: i32,
    pub w_3d84: i32,
    /// K10: object lists by address (DAT_003e1414 / 3e1438 / 3e13b4 / 3e1408 / 3e1474 / 3e1420 / 3e142c / 3e13f0)
    pub k10_lists: std::collections::BTreeMap<u32, Vec<u32>>,
    /// K13: act record pool 0x3e123c (REPLACED free list, shared::act)
    pub act_pool: shared::act::ActPool,
}

/// Missing engine calls (K: Findings). Defaults are no-ops / zero results.
#[allow(unused_variables)]
pub trait ClassHost {
    /// base-class virtual chained first (FUN_001a5270, FUN_00231100, FUN_001a5c48, ...); `addr` = callee
    fn base(&mut self, addr: u32, obj: usize) {}
    /// FUN_001a6ca8(obj, 0)
    fn ee_sub_1a6ca8(&mut self, obj: usize, a: i32) {}
    /// K7: FUN_00169198(member + 0x80, owner): class-45 highlight draw of a group member
    fn ee_sub_169198(&mut self, member: u32, owner: usize) {}
    /// FUN_00133448(p, obj): release a reference held by obj
    fn ee_sub_133448(&mut self, p: i32, obj: usize) {}
    /// slot 50 of C2ce4d0: vtable(+0xc)+0x1a4 called on the adjusted sub-object
    fn sub_vcall_1a4(&mut self, obj: usize) {}
    /// FUN_001c1f28(obj + off) -> 16-byte vector
    fn ee_sub_1c1f28(&mut self, obj: usize, off: u32) -> [u32; 4] { [0; 4] }
    /// FUN_001c0d70(v, 0)
    fn ee_sub_1c0d70(&mut self, v: [u32; 4], a: i32) -> i32 { 0 }
    /// FUN_001fe990(0x3dd040, key) + FUN_001fc408(e[3], DAT_003dd268, that, obj, &e[1])
    fn emitter_attach(&mut self, obj: usize, entry: [i32; 4], g_3dd268: i32) {}
    /// FUN_001fb098(0) + FUN_00258040(slot, 0)
    fn emitter_release(&mut self, obj: usize, slot: usize) {}
    /// FUN_001fbb90/1fb2f8/1fb4f0/1fb1f8/1fb2c0 + FUN_00258040(slot k, h); returns the handle
    fn emitter_create(&mut self, obj: usize, k: usize, e: [i32; 5]) -> i32 { 0 }
    /// FUN_001a5e80(obj)
    fn ee_sub_1a5e80(&mut self, obj: usize) {}
    /// FUN_001e8378(p) / FUN_001e9090()
    fn ee_sub_1e8378(&mut self, p: i32) {}
    fn ee_sub_1e9090(&mut self) {}
    /// *(p + 0x4c) for the C2d9040 +0x17c object
    fn state_4c(&mut self, p: i32) -> i32 { 0 }
    /// FUN_001e7840(obj, 0)
    fn ee_sub_1e7840(&mut self, obj: usize, a: i32) {}
    /// FUN_0011a1f8(obj + off, n): clear
    fn ee_sub_11a1f8(&mut self, obj: usize, off: u32, n: u32) {}
    /// FUN_001f3c10(obj) + FUN_00168638(f, f, obj + 0x80, 0, a)
    fn ee_sub_1f3c10(&mut self, obj: usize) {}
    fn ee_sub_168638(&mut self, obj: usize, f: f32, a: i32) {}
    /// *(owner + 0x148) bit 35 for owners that are not C2d9978
    fn owner_bit35(&mut self, owner: usize) -> bool { false }
    /// FUN_0011d948(x, 0x2bdec0)
    fn ee_sub_11d948(&mut self, x: f32, c: u32) -> f32 { 1.0 }
    /// FUN_00256ce8/256d08/256d28/256d18 on the template +0xe0 block, scale `k`
    fn ee_sub_256ce8(&mut self, v: &mut [u32], k: f32) {}
    /// FUN_0015bd20(count, template, link): spawn `count` particles (link = obj+0x104/+0x100/obj when +0xfc set)
    fn ee_sub_15bd20(&mut self, obj: usize, count: i32, t: &[u32; 64], blk: &[u32; 5], link: bool) {}
    /// K28: the FUN_0015bd20 requests queued since the last call (spawned by `ClassSet::k28_spawn`)
    fn k28_take_emits(&mut self) -> Vec<k28_emit::EmitReq> { Vec::new() }
    /// K30: anim event 0x11 (FUN_00210538) requests for the target's vtable +0x19c (target handle, request)
    fn k30_take_reqs(&mut self) -> Vec<(u32, [u32; 15])> { Vec::new() }
    /// slot 51 emit position (template-pointer derefs + optional t[7] matrix + FUN_001a6f00)
    fn emit_pos_15c690(&mut self, t: &[u32; 64], blk: &[u32; 5]) -> [f32; 4] { [0.0; 4] }
    /// FUN_001fbb90(sd[0]) + 1fb2f8(0) + 1fb3f0 + 1fb4f0(sd[1], sd[2]) + 1fb1f8(sd[3]) + 1fb2c0(sd[4])
    fn sound_15c690(&mut self, src: u32, pos: [f32; 4], sd: [u32; 5]) {}

    // ---- K3 (object handles are index + 1, 0 = null)
    /// FUN_0015e938(0x3dd490, kind, oid, ctx): find an object by oid (0 = none)
    fn ee_sub_15e938(&mut self, kind: i32, oid: i32, ctx: u32) -> u32 { 0 }
    /// FUN_00149a50(0x3dd490, obj, pmf, arg): queue a deferred member call (pmf = its function)
    fn ee_sub_149a50(&mut self, obj: usize, pmf: u32, arg: i32) {}
    /// (vtable +0xf4)(obj + adj, FUN_001a6318(src)) on a created object
    fn vcall_f4_1a6318(&mut self, obj: usize, src: u32) {}
    /// FUN_001394c8(ref, template) / FUN_001cde70(ref, template): push a changed template to a particle system
    fn ee_sub_1394c8(&mut self, r: i32, t: &c2ce4d0::Template) {}
    fn ee_sub_1cde70(&mut self, r: i32, t: &c2ce4d0::Template) {}
    /// FUN_001a5e28(obj): when +0x30 <= t, (vtable +0x1c)(obj, 1, 1, 0, 0, 0)
    fn ee_sub_1a5e28(&mut self, obj: usize) {}
    /// FUN_001a5d98(obj): walk the +0x34 chain (child list) back to front
    fn ee_sub_1a5d98(&mut self, obj: usize) {}
    /// (vtable +0x1c)(obj + adj, a, b, 0, 0, 0): schedule / mark
    fn vcall_1c(&mut self, obj: usize, a: i32, b: i32) {}
    /// *obj != 0 && FUN_00127db8(*obj, id, deep) != 0: the object's method table has handler `id`
    fn ee_sub_127db8(&mut self, obj: usize, id: i32, deep: i32) -> bool { false }
    /// FUN_0020e040(0x3dd490)
    fn ee_sub_20e040(&mut self) -> i32 { 0 }
    /// FUN_0011a070(obj + off, item): list insert
    fn ee_sub_11a070(&mut self, obj: u32, off: u32, item: u32) {}
    /// FUN_001620b0(obj, k, a, b) / FUN_001623d0(obj, k, a, b, c)
    fn ee_sub_1620b0(&mut self, obj: u32, k: i32, a: i32, b: i32) {}
    fn ee_sub_1623d0(&mut self, obj: u32, k: i32, a: i32, b: i32, c: i32) {}
    /// FUN_002581c0(out, path, 0x3d4a60) + FUN_00129ec8(out): save-variable path -> out word 1
    fn ee_sub_2581c0(&mut self, path: &[u16]) -> u32 { 0 }
    /// FUN_00165118(obj, a)
    fn ee_sub_165118(&mut self, obj: u32, a: i32) {}
    /// (vtable +off)(obj + adj) on an object not owned by the set (off = adjust-word offset + 4)
    fn vcall(&mut self, obj: u32, off: u32) {}
    /// FUN_001e1c40(node) / FUN_001e5628(node): class-114 list node update / stop; FUN_001e0120(obj)
    fn ee_sub_1e1c40(&mut self, node: u32) {}
    fn ee_sub_1e5628(&mut self, node: u32) {}
    fn ee_sub_1e0120(&mut self, obj: u32) {}
    /// FUN_001a61a0(obj): +0x28 bit 60 and in the 0x3dd4bc set
    fn ee_sub_1a61a0(&mut self, obj: u32) -> bool { false }
    /// FUN_001a62e8(obj): world position (row 3 of the vcall +0xfc matrix)
    fn ee_sub_1a62e8(&mut self, obj: u32) -> [f32; 3] { [0.0; 3] }
    /// plain field reads of objects not owned by the class set
    fn obj_i32(&mut self, obj: u32, off: u32) -> i32 { 0 }
    fn obj_f32(&mut self, obj: u32, off: u32) -> f32 { 0.0 }
    /// FUN_0011a3c0(obj + off): list length
    fn list_len(&mut self, obj: u32, off: u32) -> i32 { 0 }
    /// FUN_00159a20 particle walk: wrap every particle of system `r` into [lo, hi] (period `span`, FUN_0011d898)
    fn wrap_particles(&mut self, r: i32, lo: [f32; 3], hi: [f32; 3], span: f32) {}
    /// 4x4 matrix stored at obj + off (LO +0x60 = local matrix)
    fn obj_mat(&mut self, obj: u32, off: u32) -> crate::engine::anim::pose::Mat4 { crate::engine::anim::pose::IDENTITY }
    /// FUN_0013e510(obj, owner): clone `obj` into `owner` (FUN_0013e3c8 + vtable +0x144), returns the clone
    fn ee_sub_13e510(&mut self, obj: u32, owner: u32) -> u32 { obj }
    /// (vtable +0x19c)(obj + adj, req): slot 50 emit request; `m` = the request block req[13] points at
    fn vcall_19c(&mut self, obj: u32, req: &[u32; 15], m: &c2ce828::ReqBlock) {}
    /// *(obj + 0x50) != 0 ? *(*(obj + 0x50) + 0xa4) : 0
    fn parent_a4(&mut self, obj: u32) -> i32 { 0 }
    /// FUN_001a3b80(far, g_lsm, bsp, pos, slot): insert a point light into the BSP regions within `far`
    fn ee_sub_1a3b80(&mut self, far: f32, bsp: u32, pos: [f32; 3], slot: i32) {}
    /// FUN_001f0380(obj, target, a, b) -> FUN_001f01b8: create the 0x50-byte link record (list 0x3e126c)
    fn ee_sub_1f0380(&mut self, obj: usize, target: u32, a: u32, b: u32) {}
    /// class 26 (level warp): FUN_0020efc0(0x3dd490, k), FUN_00161f48(0x2deb30, +0x164, +0x168, obj),
    /// FUN_0014e4c8(0x2ddf40, 2, -1, desc, 0, 0), FUN_002280d0(0x3dd490) (next warp object handle),
    /// FUN_001c4840(0x3dd490) (player handle), FUN_001a6f00(0, obj, pos) (world -> local),
    /// FUN_00118fb0(+0xec, 0x7fffffff, pos) (volume test), FUN_001b3a48(0x2f66b0, +0x14c[k]),
    /// FUN_0022b490(0x3dd490, +0x144, level, -1, flags, a)
    fn ee_sub_20efc0(&mut self, k: i32) {}
    fn ee_sub_161f48(&mut self, obj: usize) {}
    fn ee_sub_14e4c8(&mut self, desc: u32) {}
    fn ee_sub_2280d0(&mut self) -> u32 { 0 }
    fn ee_sub_1c4840(&mut self) -> u32 { 0 }
    fn ee_sub_1a6f00(&mut self, obj: usize, pos: [f32; 3]) -> [f32; 3] { pos }
    fn ee_sub_118fb0(&mut self, obj: usize, pos: [f32; 3]) -> bool { false }
    fn ee_sub_1b3a48(&mut self, obj: usize, k: i32) {}
    fn ee_sub_22b490(&mut self, obj: usize, level: i32, flags: i32, a: i32) {}
    /// class 120 +0xb0 binding: FUN_002090d0(obj + 0xb0, anc) init, FUN_00209130 bind, FUN_00209590 clone,
    /// FUN_0020aff0(obj, args) (method 365)
    fn ee_sub_2090d0(&mut self, obj: usize, anc: u32) {}
    fn ee_sub_209130(&mut self, obj: usize, anc: u32) {}
    fn ee_sub_209590(&mut self, obj: usize) {}
    fn ee_sub_20aff0(&mut self, obj: usize, a: [u32; 5]) {}
    /// FUN_001f4c58(alo, oid, 6): child lookup (handle)
    fn ee_sub_1f4c58(&mut self, alo: u32, oid: i32, k: i32) -> u32 { 0 }
    /// FUN_0012f938(anim, k) + FUN_001300b8(*that, 1.0, anim, alo, b, *(a + 0xa0), out, 0) -> out row 3
    fn ee_sub_1300b8(&mut self, anim: u32, k: i32, alo: u32, b: u32, a: u32) -> [u32; 4] { [0; 4] }

    // ---- K4
    /// (vtable +0xfc)(obj + adj): world matrix (row 3 = position, as FUN_001a62e8 reads it)
    fn vcall_fc(&mut self, obj: u32) -> crate::engine::anim::pose::Mat4 { crate::engine::anim::pose::IDENTITY }
    /// class 105 +0xdc mesh groups (geometry the class re-skins; not in the class state)
    fn mesh_dc(&mut self, obj: u32) -> Vec<c2db0d0::Group105> { Vec::new() }
    /// *(+0xc0 + idx * 0x28 + 4) = cb (mesh draw callback)
    fn set_mesh_cb(&mut self, obj: u32, idx: usize, cb: u32) {}
    /// FUN_0016ef00(obj + 0x80, idx)
    fn ee_sub_16ef00(&mut self, obj: u32, idx: usize) {}
    /// SO slot 18 FUN_001f3c70 (draw) with +0x12c = alpha
    fn so_draw_1f3c70(&mut self, obj: usize, alpha: f32) {}
    /// FUN_0014e4c8(0x2ddf40, k, a, desc, b, obj) / FUN_0014e328(0x2ddf40, k, desc, target, obj) (camera requests)
    fn ee_sub_14e4c8_k(&mut self, k: i32, a: i32, desc: u32, b: i32, obj: u32) {}
    fn ee_sub_14e328(&mut self, k: i32, desc: u32, target: u32, obj: u32) {}
    /// K29: FUN_001e57a0 node +0x84 blend descriptor of `pv` (keys before the CM defaults)
    fn k29_cm_move(&mut self, pv: u32, mv: k29_cam::CmMove) {}
    /// K29: FUN_001e5a98 node +0x90 (blend cut on end) of `pv`
    fn k29_cm_move_cut(&mut self, pv: u32, cut: u32) {}
    /// K29: FUN_0014fd50(desc) ? FUN_0014e4c8(g_cm, grf, cpp, desc, pso, pv)
    fn k29_cm_revoke_if_active(&mut self, grf: u32, cpp: i32, desc: u32, pso: i32, pv: u32) {}
    /// K30: camera state of a class object for the CM presets (key = the request's pv / pso handle)
    fn k30_cam_state(&mut self, key: u32, st: k30_cam::CamState) {}
    /// K30: camera rows DAT_002ddfa0 / DAT_002ddfb0 / DAT_002ddfc0 (0x2ddf40 +0x60..+0x80); None = no camera
    fn k30_cam_rows(&mut self) -> Option<[[f32; 4]; 3]> { None }
    /// K30: sequence actions 0x3d-0x41 on the zoom look 0x2de460 (FUN_00150e20 .. FUN_00150ef0)
    fn k30_zoom(&mut self, op: k30_cam::ZoomOp) {}
    /// K30: wait conditions 0x3d / 0x3e / 0x40 (FUN_00150f58 / FUN_00151018 / FUN_001510d8 on 0x2de460)
    fn k30_zoom_wait(&mut self, kind: u32) -> bool { true }
    /// FUN_001c4a48(0x3dd490): current focus object (handle)
    fn ee_sub_1c4a48(&mut self) -> u32 { 0 }
    /// FUN_00118fb0(*(obj + off), 0x7fffffff, pos): point-in-volume test of the volume stored at obj + off
    fn ee_sub_118fb0_at(&mut self, obj: usize, off: u32, pos: [f32; 3]) -> bool { false }
    /// FUN_00132b10(obj, class): first ancestor-or-self of class `class` (handle, 0 = none)
    fn ee_sub_132b10(&mut self, obj: u32, class: i32) -> u32 { 0 }
    /// class 132: FUN_002121e8(seq, id) (sequence `id` of sequencer `seq` still running),
    /// FUN_00213b00(seq, id) (start it), (vtable +off)(obj + adj, a)
    fn ee_sub_2121e8(&mut self, seq: u32, id: i32) -> bool { false }
    fn ee_sub_213b00(&mut self, seq: u32, id: i32) {}
    fn vcall_arg(&mut self, obj: u32, off: u32, a: i32) {}
    /// FUN_001a4e70(0x2e1e88) (transition queue non-empty) / FUN_001a4e80(0x2e1e88) (pop + run one)
    fn ee_sub_1a4e70(&mut self) -> bool { false }
    fn ee_sub_1a4e80(&mut self) {}
    /// *(obj + 0x40) |= bits on an object not owned by the set (tagged +0x40 word)
    fn obj_or_40(&mut self, obj: u32, bits: u32) {}
    /// FUN_002283d0(obj): class-26 warp start (ported in c2dc6f0, run on the owning instance)
    fn ee_sub_2283d0(&mut self, obj: u32) {}
    /// class-133 FUN_00164640(obj, k) / FUN_00164df0(obj, k)
    fn ee_sub_164640(&mut self, obj: u32, k: i32) {}
    fn ee_sub_164df0(&mut self, obj: u32, k: i32) {}
    /// FUN_001619a0(0x2deb30, k, obj), FUN_001abd98(0x2e65e0, a), FUN_0020f030(0x3dd490, k)
    fn ee_sub_1619a0(&mut self, k: i32, obj: u32) {}
    fn ee_sub_1abd98(&mut self, a: i32) {}
    fn ee_sub_20f030(&mut self, k: i32) {}
    /// FUN_001d7e80(0x3e0ea8, path, 0x3d4a60): save-variable word of `path`
    fn ee_sub_1d7e80(&mut self, path: &[u16]) -> u32 { 0 }
    /// FUN_00165dc0(0x3dd490, a) / FUN_00165e10(0x3dd490, a); *(DAT_003e0fa4 + 0x7c) (0 when DAT_003e0fa4 is null)
    fn ee_sub_165dc0(&mut self, a: i32) {}
    fn ee_sub_165e10(&mut self, a: i32) {}
    fn g_3e0fa4_7c(&mut self) -> u32 { 0 }
    /// FUN_00210b70() vs FUN_00210990(FUN_00210a88()): same record (equal, or both non-null with equal +8 ^ +0x38)
    fn ee_sub_210b70_same(&mut self) -> bool { false }
    /// class 7: FUN_001c7bb8(serial), FUN_00149ab8(0x3dd490, obj, pmf, arg) (deferred call, 2nd queue),
    /// FUN_001caa60 after the +0xe8 unlink (detach item), FUN_0011a380 / FUN_0011a2e8(0x3dd4bc, +0x14) set
    /// test / remove, **(obj + 0x14), FUN_00127e90 (rebind method ref to obj), refcount release
    /// (FUN_00257078 at 0), FUN_00259a88 (add ref)
    fn ee_sub_1c7bb8(&mut self, serial: i32) -> bool { false }
    fn ee_sub_149ab8(&mut self, obj: usize, pmf: u32, arg: i32) {}
    fn ee_sub_1caa60(&mut self, obj: usize, item: u32) {}
    fn ee_sub_11a380(&mut self, obj: u32) -> bool { false }
    fn ee_sub_11a2e8(&mut self, obj: u32) {}
    fn ref_14(&mut self, obj: u32) -> u32 { 0 }
    fn ee_sub_127e90(&mut self, r: u32, obj: u32) -> u32 { r }
    fn ref_release(&mut self, r: u32) {}
    fn ee_sub_259a88(&mut self, r: u32) {}
    /// class 13: FUN_001a6ca8((vtable +0x34)(obj + 0x190 sub-block), 3) (FUN_00182c70 tail);
    /// FUN_001e7d90(obj): geometry bounding radius
    fn ee_sub_182c70_tail(&mut self, obj: usize) {}
    fn ee_sub_1e7d90(&mut self, obj: usize) -> f32 { 0.0 }
    /// class 113: FUN_001e9c30(obj) for a path target without a class-0xd / 0xb ancestor
    fn ee_sub_1e9c30(&mut self, obj: u32) {}
    /// class 129: FUN_0021ecd8(text, "%d", v) (counter text buffers 0x2c3ec0 / 0x2c3ef0 / 0x2c3f20)
    fn ee_sub_21ecd8(&mut self, text: u32, v: i32) {}
    /// class 116 slot 18 tail: full-screen quad in `rgba` (FUN_0016f960 + FUN_002270c8 into 0x2dea28) and
    /// the FUN_001cd998 draw record {kind (1 / 0xe / unset = -1), -1.0, DAT_002dea34}
    fn ee_sub_15dd48_draw(&mut self, rgba: u32, kind: i32) {}

    // ---- K5 (actor core)
    /// *(obj + 0xe8) physics block: None = null; else (+0xdc != 0, +0xa0, +0xc0)
    fn phys_e8(&mut self, obj: usize) -> Option<actor::alo::PhysE8> { None }
    /// FUN_001f37c8(obj, 0) (sound-source position update)
    fn ee_sub_1f37c8(&mut self, obj: usize) {}
    /// FUN_00149b58(0x3dd490, k, obj, pmf, 0)
    fn ee_sub_149b58(&mut self, k: i32, obj: usize, pmf: u32) {}
    /// ALO child walk (+0x64 first, +0x24 next): child handles in order
    fn alo_children(&mut self, obj: u32) -> Vec<u32> { Vec::new() }
    /// *(obj + 0x28) (u64 LO flags)
    fn obj_flags_28(&mut self, obj: u32) -> u64 { 0 }
    /// FUN_001d6310(obj) / FUN_001d6378(obj): leave / enter the 0x3dd4bc render set
    fn ee_sub_1d6310(&mut self, obj: usize) {}
    fn ee_sub_1d6378(&mut self, obj: usize) {}
    /// FUN_00147e88(0x3dd490, *(obj + 0x14))
    fn ee_sub_147e88(&mut self, obj: usize) {}
    /// (vtable +off)(obj + adj, f)
    fn vcall_f(&mut self, obj: u32, off: u32, f: f32) {}
    /// (vtable +off)(obj + adj) -> int
    fn vcall_ret(&mut self, obj: u32, off: u32) -> i32 { 0 }
    /// (vtable +0x1b4)(obj + adj, other)
    fn vcall_1b4(&mut self, obj: u32, other: u32) {}
    /// the +0xec contact block (head +0x6c, list +100)
    fn contacts_ec(&mut self, obj: usize) -> actor::alo::Contacts { Default::default() }
    /// *entry[2] = 0, entry[0] = 0 of contact `k`
    fn contact_clear(&mut self, obj: usize, k: usize) {}
    /// FUN_001a7e10(0x3dd490, rec): free a contact record
    fn ee_sub_1a7e10(&mut self, rec: u32) {}
    /// FUN_00134320(0x3dd490, obj) (class 86 slot 14 with physics)
    fn ee_sub_134320(&mut self, obj: usize) {}
    /// FUN_001a6290(obj, oid): obj's oid (+0x18) or one of its attached (+0x3c chain) oids == oid
    fn ee_sub_1a6290(&mut self, obj: u32, oid: i32) -> bool { false }
    /// FUN_001f6978 / 1f69f8 (+0xc0 mesh part `id`: show / hide), FUN_001f6ac8 / 1f6b48 (+0xe4 collision part)
    fn ee_sub_1f6978(&mut self, obj: usize, id: i32) {}
    fn ee_sub_1f69f8(&mut self, obj: usize, id: i32) {}
    fn ee_sub_1f6ac8(&mut self, obj: usize, id: i32) {}
    fn ee_sub_1f6b48(&mut self, obj: usize, id: i32) {}
    /// FUN_001f5720(obj, a) / FUN_001f57c0(obj, a)
    fn ee_sub_1f5720(&mut self, obj: usize, a: i32) {}
    fn ee_sub_1f57c0(&mut self, obj: usize, a: i32) {}

    // ---- K6 (guards, class 56 family)
    /// FUN_0020f1b8(0x3dd490): global "no detection"
    fn ee_sub_20f1b8(&mut self) -> bool { false }
    /// DAT_003e1480 vision-blocker walk (FUN_001a6f00 + FUN_00118668 segment eye -> target): any hit
    fn ee_sub_118668_blockers(&mut self, eye: [f32; 3], tp: [f32; 3]) -> bool { false }
    /// FUN_001545f0(sensor, t): eye -> target LOS raycast (FUN_0020faa0 + FUN_0014cc58) hit; `f8` = sensor flags
    fn ee_sub_1545f0(&mut self, guard: usize, eye: [f32; 3], tp: [f32; 3], f8: u32) -> bool { false }
    /// FUN_001a61e0(sensor, 0x1c, 0, 3, &msg, 0): sensor-state script event; returns the (possibly rewritten) state
    fn sensor_event_1c(&mut self, guard: usize, t: u32, old: i32, new: i32) -> i32 { new }
    /// *(t + 0x240) aim offset of an is-a 0x33 target
    fn aim_240(&mut self, t: u32) -> [f32; 3] { [0.0; 3] }
    /// FUN_00209618 / FUN_00204e38: root-motion local velocity (= guard +0x190 negated) of guard anim `anim`
    /// (table +0x4c0) at playback time `t` and rate `rate` (FUN_00131a00)
    fn ee_sub_209618(&mut self, guard: usize, anim: i32, t: f32, rate: f32) -> [f32; 3] { [0.0; 3] }
    /// K7: FUN_001f7598 hit delivery for a target outside the class set (the player): queued for the JT
    fn ee_sub_1f7598(&mut self, hit: &guard::slots::GuardHit) {}
    /// K7: contact list of a guard (+0xec -> +0x6c pairs) for slot 73 FUN_001f7410
    fn contacts(&mut self, guard: usize) -> Vec<guard::slots::Contact> { Vec::new() }
    /// length (s) of guard anim `anim`: task 0x9d reaches state 1 (FUN_00174fb8) when it has played
    fn anim_len(&mut self, guard: usize, anim: i32) -> f32 { 0.0 }
    /// FUN_00209098(guard): routine interrupt
    fn ee_sub_209098(&mut self, guard: usize) -> bool { false }
    // ---- K8 (pickups, classes/k8.rs)
    /// FUN_0015ed78(world, k, id, obj): child object of obj (k 1, id 0x12 = pickup sparkle)
    fn ee_sub_15ed78(&mut self, k: i32, id: i32, obj: u32) -> u32 { 0 }
    /// FUN_001fbb90(id) + FUN_001fb2f8(.., 0): play sound `id` for obj
    fn ee_sub_1fbb90(&mut self, obj: usize, id: i32) {}
    /// target vt+0x19c(&{a, src}) (listener message)
    fn vcall_19c_msg(&mut self, target: u32, a: i32, src: u32) {}
    /// FUN_00156b98(0x3dd490, 0, pos, obj)
    fn ee_sub_156b98(&mut self, obj: usize, pos: [f32; 3]) {}
    /// FUN_001618b0(world): clue-bottle HUD (world +0x3bb8 vt+0x18c(-1), FUN_00212a30)
    fn ee_sub_1618b0(&mut self) {}
    /// FUN_00161868(world): coin HUD (world +0x3bac)
    fn ee_sub_161868(&mut self) {}
    /// FUN_00164640(obj, k, src)
    fn ee_sub_164640_src(&mut self, obj: u32, k: i32, src: u32) {}
    /// FUN_001ce888(world, 9, 0) collect effect bound to obj with descriptor `desc` at pos
    fn ee_sub_1ce888_fx(&mut self, obj: usize, desc: u32, pos: [f32; 3]) {}
    /// player vt+0x28c (slot 80: FUN_00144268 hp / mp gift)
    fn player_vcall_28c(&mut self, player: u32, hp: i32, mp: i32) {}
    /// FUN_001a58d0(obj, 0, 0)
    fn ee_sub_1a58d0(&mut self, obj: usize, a: i32, b: i32) {}
    /// FUN_001cb4b8(host, obj): clear the attachment record (FUN_001cb410 +8 = 0)
    fn ee_sub_1cb4b8(&mut self, host: u32, obj: u32) {}
    /// FUN_001f0598 + anim vt+0x14: stop the pickup's anim
    fn ee_sub_1f0598(&mut self, obj: usize) {}
    /// FUN_001f10f0(anim): blend in
    fn ee_sub_1f10f0(&mut self, obj: usize) {}
    /// FUN_001f0660(anim, id): switch anim
    fn ee_sub_1f0660(&mut self, obj: usize, anim: i32) {}
    /// FUN_001a6318(obj): world orientation rows (identity = none)
    fn ee_sub_1a6318(&mut self, obj: u32) -> [[f32; 4]; 3] { [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]] }
    /// camera globals DAT_002ddfa0 / DAT_002ddfb0 (0x2ddf40 +0x60 / +0x70)
    fn cam_2ddfa0(&mut self) -> [[f32; 4]; 2] { [[0.0; 4]; 2] }
    /// FUN_001ce888(world, 4, 0) sparkle emitter: vt+0xc(1.0, at), +0xf0 = target, +0x1c = src, +0x18 = scale;
    /// false = no emitter
    fn ee_sub_1ce888_sparkle(&mut self, at: [f32; 3], target: u32, src: u32, scale: f32) -> bool { false }
    /// FUN_0020faa0(0x3dd490, lo, hi, FUN_001c5570, 0): collision parts in the box (filter: flag bit 48 clear,
    /// owner not is-a 0x32 / 0x4f)
    fn ee_sub_20faa0_1c5570(&mut self, lo: [f32; 3], hi: [f32; 3]) -> Vec<k8::K8Part> { Vec::new() }
    /// ALO +0xe0 / +0xe4 collision parts
    fn alo_parts_e4(&mut self, alo: u32) -> Vec<k8::K8Part> { Vec::new() }
    /// FUN_001a93c8(radius, pos, part, 0x7fffffff, 0, 0, max, out): sphere-vs-part contacts (point, plane n|d)
    fn ee_sub_1a93c8(&mut self, radius: f32, pos: [f32; 3], part: u32, max: usize) -> Vec<[[f32; 4]; 2]> { Vec::new() }
    /// FUN_001f7b98(obj) +0xf0 = 1
    fn ee_sub_1f7b98_f0(&mut self, obj: u32) {}
    /// FUN_002303a0(obj, *(obj + off), k)
    fn ee_sub_2303a0_off(&mut self, obj: u32, off: u32, k: i32) {}
    /// FUN_001c5fa0 anim path: segment channel 0xc found (FUN_0012ff10) -> FUN_001a5ee8 draw at that matrix
    fn ee_sub_1c5fa0_anim(&mut self, obj: usize) -> bool { false }
    /// FUN_001c5148 tail: anim root offsets of the parts +0x58 / +0x5c
    fn ee_sub_1c5148_anim(&mut self, obj: usize) {}
    /// player +0x15c pickup record (FUN_001c4bd8)
    fn rec_15c(&mut self, player: u32) -> k8::PickRec { k8::PickRec::default() }
    /// FUN_001eeee0(obj + 0x26c, id)
    fn ee_sub_1eeee0(&mut self, obj: u32, id: i32) -> bool { false }
    // ---- K9 (vault, classes/k9.rs)
    /// FUN_001fe5f0(0x3dd040, id, k) -> handle / FUN_001fe738(0x3dd040) release
    fn ee_sub_1fe5f0(&mut self, id: i32, k: i32) -> u32 { 0 }
    fn ee_sub_1fe738(&mut self) {}
    /// FUN_001dffd8(seq): sequence +0x74 (running)
    fn ee_sub_1dffd8(&mut self, seq: u32) -> bool { false }
    /// FUN_001dfeb8(seq) stop / FUN_001dfd58(seq, player) start / FUN_001e0020(seq, ev) event
    fn ee_sub_1dfeb8(&mut self, seq: u32) {}
    fn ee_sub_1dfd58(&mut self, seq: u32, player: u32) {}
    fn ee_sub_1e0020(&mut self, seq: u32, ev: i32) {}
    /// FUN_001daf38(id, on): save flag "%d" write (+ FUN_001619a0(0x2deb30, 0x3b) when on)
    fn ee_sub_1daf38(&mut self, id: i32, on: bool) {}
    /// FUN_001a3078 (on) / FUN_001a30d0 (off) of 0x2e1e60 (world +0x3d80 edges)
    fn ee_sub_1a3078(&mut self, on: bool) {}
    /// FUN_001c46c0(0.0, obj +0x150, mask): tracker buttons of `mask` pressed now (bits consumed)
    fn k9_pressed(&mut self, obj: u32, mask: u32) -> u32 { 0 }
    /// obj +0x150 tracker +0x10 / +0x14 (stick x, y)
    fn k9_stick(&mut self, obj: u32) -> [f32; 2] { [0.0; 2] }
    /// FUN_00131968(a): anim `a` finished (not in 0x3e1218)
    fn ee_sub_131968(&mut self, a: u32) -> bool { true }
    /// FUN_001306a8: stop anim handle (vt+0x14)
    fn ee_sub_1306a8(&mut self, a: u32) {}
    /// FUN_001305b8(0, 1.0, anim, obj, 0, &handle) -> handle
    fn ee_sub_1305b8(&mut self, anim: u32, obj: u32) -> u32 { 0 }
    /// FUN_00142140(player, pose): player +0x4c0 pose (0 = release)
    fn ee_sub_142140(&mut self, player: u32, pose: u32) {}
    /// FUN_001c8918 sub-state 1 dial motion (FUN_0022fcc8 / FUN_0022fc48 / FUN_002303a0(self, dial, 0x11))
    fn k9_dial_turn(&mut self, obj: u32, dial: u32, player: u32, pose: u32) {}
    // ---- K10 (classes/k10.rs)
    /// (vtable +0x24)(obj + adj, a, b)
    fn vcall_24(&mut self, obj: usize, a: i32, b: u32) {}
    /// FUN_0020ff70(0x3dd490, n, entries, obj, 1): FUN_0020fff0 per +0x11c entry (method-1075 arguments)
    fn ee_sub_20ff70(&mut self, obj: usize, entries: &[[u32; 3]]) {}
    /// FUN_00161f48(0x2deb30, n, entries, obj) over the +0x160 entries (method-1076 arguments)
    fn ee_sub_161f48_list(&mut self, obj: usize, entries: &[[u32; 2]]) {}
    /// FUN_001b7000(bentley, point): +0x13ec set -> +0x13f0 = point, FUN_001b6078(4), FUN_002046f0(point +0x180)
    fn ee_sub_1b7000(&mut self, user: u32, point: u32) {}
    /// FUN_00227bb8 +0x110: local bounding-sphere centre of a class-22 volume
    fn vol_centre_110(&mut self, obj: usize) -> [f32; 3] { [0.0; 3] }
    /// FUN_001a6f00(obj, 0, p): obj-local point -> world
    fn ee_sub_1a6f00_to_world(&mut self, obj: usize, p: [f32; 3]) -> [f32; 3] { p }
    // ---- K11 (classes/k11.rs)
    /// FUN_0022d348(alo, 0, p): world velocity of ALO `alo` at world point `p`
    fn ee_sub_22d348(&mut self, alo: u32, p: [f32; 3]) -> [f32; 3] { [0.0; 3] }
    // ---- K12 (classes/k12.rs)
    /// obj +0x40 &= mask (the +0x40 flag word, or the word it points to when bit 0 is clear)
    fn obj_and_40(&mut self, obj: u32, mask: u32) {}
    /// JT `p` is free: +0x2dc == -1 and (vtable +0x26c)(p) != 2 (FUN_0022a060)
    fn jt_free_2dc_26c(&mut self, p: u32) -> bool { true }
    // ---- K13 (classes/k13.rs, classes/shared/)
    /// virtual call `off` (+0xc activate / +0x14 deactivate / +0x1c notify) on act record `rec` (vtable `vt`)
    /// of the xf `xf` list
    fn act_vcall(&mut self, vt: u32, off: u32, xf: u32, rec: u32) {}
    /// FUN_00130b18(handle): retract a playing anim (anim::play::AsegaHost::retract)
    fn ee_sub_130b18(&mut self, handle: u32) {}
    /// FUN_0012f9b8(a, rate, anim, obj, flags, &slot): anim request -> new handle
    fn ee_sub_12f9b8(&mut self, a: u32, rate: f32, anim: u32, obj: u32, flags: u32) -> u32 { 0 }
    /// FUN_00229738(obj, rec): unlink a water contact record from obj +0x138, FUN_001f5078(obj) (vtable
    /// +0x17c(4), wake xfLin / xfAng FUN_00230598)
    fn ee_sub_229738(&mut self, obj: u32, water: u32) {}
    /// (obj vtable +0x244)(obj, water) (class 89 slot 70)
    fn vcall_244(&mut self, obj: u32, water: u32) {}
    // ---- K14 (classes/k14.rs, classes/shared/c71.rs)
    /// FUN_0015c198(0, 0, emitter, kind, 0, 0, pos): emitter vtable +0x19c (slot 50) with that request
    fn ee_sub_15c198(&mut self, emitter: u32, kind: i32, pos: [f32; 3]) {}
    /// FUN_001f7598(hit) with its result (delivered / accepted)
    fn ee_sub_1f7598_ok(&mut self, hit: &guard::slots::GuardHit) -> bool {
        self.ee_sub_1f7598(hit);
        true
    }
    /// FUN_001a5ee8(1.0, 1.0, model, &m): draw `model` at the row matrix `m` (renderer)
    fn ee_sub_1a5ee8(&mut self, model: u32, m: &[[f32; 4]; 4]) {}
    // ---- K15 (classes/k15.rs)
    /// FUN_001f15b8(&slot): stop the sound in `slot` and clear it
    fn ee_sub_1f15b8(&mut self, slot: &mut u32) {
        *slot = 0;
    }
    /// FUN_001f1448(snd, obj, -1, flags, &slot): play sound object `snd` on `obj` -> handle (0 = none)
    fn ee_sub_1f1448(&mut self, snd: u32, obj: u32, flags: u32) -> u32 { 0 }
    /// FUN_001f07e8() == FUN_001f07c8(handle): the sound is done
    fn snd_done_1f07c8(&mut self, handle: u32) -> bool { true }
    /// obj +0x28 bit 60 (in the world); the player (DAT_002dfb80) always is
    fn in_world_28(&mut self, obj: u32) -> bool {
        obj == guard::PLAYER_HANDLE || (self.obj_flags_28(obj) >> 60) & 1 != 0
    }
    /// FUN_00154c90(0x2de578): BIF 103 (game::vm_state; mean of the save's suck history) -> 0 without a game
    fn ee_sub_154c90(&mut self) -> f32 { 0.0 }
    /// FUN_0015c150(emitter, obj): fire the hit-fx emitter at `obj`
    fn ee_sub_15c150(&mut self, emitter: u32, obj: u32) {}
    // ---- K15 class 101 (classes/k15_101.rs)
    /// obj +0x10 (parent object; 0 = none)
    fn parent_10(&mut self, obj: u32) -> u32 { 0 }
    /// FUN_001a58d0(obj, other, k) with an object argument
    fn ee_sub_1a58d0_to(&mut self, obj: usize, other: u32, k: i32) {}
    /// FUN_0012dc18(obj, xf (+0x58 lin / +0x5c ang), 0, 7, 2, v): pin constraint -> handle (0 = none)
    fn ee_sub_12dc18(&mut self, obj: u32, xf: u32, v: [f32; 3]) -> u32 { 0 }
    /// FUN_0012e460(&slot): release the constraint in `slot` and clear it
    fn ee_sub_12e460(&mut self, slot: &mut u32) {
        *slot = 0;
    }
    /// FUN_001f7bf8(rate, obj)
    fn ee_sub_1f7bf8(&mut self, obj: usize, rate: f32) {}
    /// FUN_001f0730(sound handle, id): sound cue
    fn ee_sub_1f0730(&mut self, snd: u32, id: i32) {}
    /// FUN_0015e9c8(0x3dd490, 0x105, class, 0, 0x20, out): objects of `class` (handles)
    fn query_15e9c8(&mut self, class: i32) -> Vec<u32> { Vec::new() }
    /// SO contacts of class-101 object `obj` (slot 57 walk of +0xec +0x64)
    fn so_contacts_57(&mut self, obj: usize) -> Vec<k15_101::Contact57> { Vec::new() }
    /// FUN_00224bf8 explosion (REPLACED, see k15_101.rs)
    fn explode_224bf8(&mut self, obj: u32, pos: [f32; 3], radius: f32, damage: i32) {}
    // ---- K15 projectiles 79-82 (classes/k15_79.rs)
    /// SO contacts of projectile `obj` (slot 57 FUN_0013aeb0 walk of +0xec +0x64)
    fn proj_contacts_13aeb0(&mut self, obj: usize) -> Vec<k15_79::ProjContact> { Vec::new() }
    /// FUN_0013b5e8 victims of an explosion at `pos` (query radius `r`, line radius `r_obj` / `r_34` for class
    /// 0x34 targets): object handles
    fn proj_victims_13b5e8(&mut self, obj: u32, pos: [f32; 3], r: f32, r_obj: f32, r_34: f32) -> Vec<u32> { Vec::new() }
    /// FUN_0013ac18: new line-of-sight records of projectile `obj` (the previous ones' pairs re-enabled)
    fn proj_los_13ac18(&mut self, obj: u32, prev: &[k15_79::Los]) -> Vec<k15_79::Los> { Vec::new() }
    /// FUN_001a7c18 on a line-of-sight record for a contact with `other` (slot 59)
    fn ee_sub_1a7c18_los(&mut self, obj: usize, other: u32, rec: &k15_79::Los) {}
    /// FUN_001c4098(obj): +0x150 record present (its +0x80 != -1)
    fn ee_sub_1c4098(&mut self, obj: u32) -> bool { false }
    /// **(obj + 0x14): root object
    fn root_14(&mut self, obj: u32) -> u32 { obj }
    /// *(obj + 0xec) + 0x60: SO root
    fn so_root_ec60(&mut self, obj: u32) -> u32 { obj }
    // ---- K15 class 83 (classes/k15_83.rs)
    /// FUN_00130598 -> FUN_001304e0: play `anim` on `target` (rate, mode 1 / 0 / 2 reversed, word; argument
    /// order partly lost in the decompile)
    fn ee_sub_130598(&mut self, anim: u32, target: u32, rate: f32, mode: u32, w: u32) {}
    /// **(anim + 0x14) + 0x18: the oid of the anim's owner (default: `owner`'s)
    fn anim_oid_14(&mut self, anim: u32, owner: u32) -> i32 {
        self.obj_i32(owner, 0x18)
    }
    /// DAT_002e1e40 +0x18c4 while +0x18c8 == 3 (Sly's active power-up), else None
    fn sly_18c4(&mut self) -> Option<i32> { None }
    /// FUN_00156698(DAT_003e14bc, pos, owner) explosion fx (+0x54 = 1)
    fn ee_sub_156698_fx(&mut self, pos: [f32; 3], owner: u32) {}
    // ---- K15 class 85 (classes/k15_85.rs)
    /// FUN_001e6648: length of the class-10 path `path`
    fn path_len_1e6648(&mut self, path: u32) -> f32 { 0.0 }
    /// FUN_001e6f10(d, path, out, 0): world point at distance `d` along `path`
    fn path_point_1e6f10(&mut self, path: u32, d: f32) -> [f32; 3] { [0.0; 3] }
    /// FUN_0014c6a0(other SO, p0, p1, 1, out) / FUN_00205670 (class-0x33 owner): segment p0 -> p1 crosses `other`
    fn beam_hit_14c6a0(&mut self, other: u32, p0: [f32; 3], p1: [f32; 3]) -> Option<[f32; 3]> { None }
    /// objects whose SO overlaps the class-85 beam geometry of `obj` (the slot 59 / 53 callers, REPLACED)
    fn so_overlaps_85(&mut self, obj: usize) -> Vec<u32> { Vec::new() }
    /// slot 18 FUN_001df5b0 (REPLACED): the clipped beams of `obj`
    fn beam_draw_85(&mut self, obj: usize, segs: &[([f32; 3], [f32; 3])]) {}
    /// FUN_001a64e8(obj, k, desc)
    fn ee_sub_1a64e8(&mut self, obj: usize, k: i32, desc: u32) {}
    // ---- K16 class 55 Bentley (classes/k16_55.rs)
    /// FUN_00135248(obj, state): Bentley SetState (unported cluster 55h)
    fn ee_sub_135248(&mut self, obj: usize, state: i32) {}
    /// FUN_001f9b20(obj): ALO gravity / forces of a non-JT object
    fn ee_sub_1f9b20(&mut self, obj: usize) {}
    /// FUN_00204108(obj): PO move forces of a non-JT object
    fn ee_sub_204108(&mut self, obj: usize) {}
    /// FUN_001fac30(obj, a): add acceleration `a`
    fn ee_sub_1fac30(&mut self, obj: usize, a: [f32; 3]) {}
    /// K27: one physics frame (FUN_001f3980 / FUN_001f8dd8 with the PO motor `m`) of a non-JT character body
    /// after its slot 69 forces (classes/k27.rs)
    fn k27_po_step(&mut self, obj: usize, m: crate::engine::physics::motor::PoMotor, dt: f32) {}
    /// `addr`(obj + `off`): an embedded JT-layout component function (FUN_00196df8 +0x2c0, FUN_00194170 +0x4c0,
    /// FUN_001a1148 +0xdf0, FUN_0019df70 +0xf50) of a non-JT player character
    fn po_part(&mut self, addr: u32, obj: usize, off: u32) {}
    /// (vtable +0x10c)(obj): velocity
    fn vcall_10c(&mut self, obj: usize) -> [f32; 3] { [0.0; 3] }
    /// FUN_001a19a0(obj + 0xdf0, hit): TakeDamage of a non-JT player character (accepted)
    fn ee_sub_1a19a0_po(&mut self, obj: usize, hit: &guard::slots::GuardHit) -> bool { false }
    /// FUN_00204250(obj, rec, side): PO contact part of a non-JT player character
    fn ee_sub_204250(&mut self, obj: usize, rec: &mut crate::engine::player::p5_hull::ContactRec, side: usize) {}
    /// FUN_00140ed8(obj, kind): base hit veto (immunity chain) of a non-JT player character
    fn ee_sub_140ed8(&mut self, obj: usize, kind: i32) -> bool { false }
    /// FUN_00221268(vol, s): forced state of the class-15 volume `vol` (handle)
    fn ee_sub_221268(&mut self, vol: u32, s: i32) {}
    /// (vtable +0x1c)(obj, k, 0, 0, 0, &msg): query message `msg` (+0 kind, +4 value) as the object leaves it
    fn vcall_1c_query(&mut self, obj: usize, k: i32, msg: (i32, i32)) -> (i32, i32) { msg }
    /// DAT_002de1a0 (object handle, 0 none)
    fn g_2de1a0(&mut self) -> u32 { 0 }
    // ---- K17 class 55 Bentley (classes/k17_55.rs)
    /// FUN_001f03c8(sm, obj): link record of `obj` in the state machine's +0x68 list (+0x28 == obj; 0 none)
    fn ee_sub_1f03c8(&mut self, sm: u32, obj: u32) -> u32 { 0 }
    /// FUN_001f01b8(sm, obj, a, b, &out): create a link record of `obj` in `sm` -> record (0 none)
    fn ee_sub_1f01b8(&mut self, sm: u32, obj: u32, a: i32, b: u32) -> u32 { 0 }
    /// FUN_001f0380(sm, obj, a, b) -> link record handle (FUN_001f01b8)
    fn ee_sub_1f0380_link(&mut self, sm: u32, obj: u32, a: i32, b: u32) -> u32 { 0 }
    /// FUN_001f0598(rec): unlink a link record
    fn ee_sub_1f0598_link(&mut self, rec: u32) {}
    /// FUN_0019cd38(obj + 0xf00, s): SetBinocState of a non-JT player character
    fn ee_sub_19cd38_po(&mut self, obj: usize, s: i32) {}
    /// FUN_00168638(a, b, obj + 0x80, 0, k): ALO draw with the two alphas (renderer; default: the one-alpha hook)
    fn ee_sub_168638_2(&mut self, obj: usize, a: f32, b: f32, k: i32) {
        self.ee_sub_168638(obj, a, k);
    }
    /// FUN_00203d00 flag branch: bone rows of +0x258 rebased onto the +0x25c ASEG (FUN_002333b4, FUN_001f4bf8)
    fn ee_sub_203d00_bones(&mut self, obj: usize) {}
    /// FUN_001f58f0(obj, rec): SO camera target record (position, FUN_0022d348, yaw, speed 100, t)
    fn ee_sub_1f58f0(&mut self, obj: usize, rec: &mut k17_55::CamRec) {}
    /// `addr`(obj + `off`, rec): JT-layout component camera-record part (FUN_0018a4e8 +0x4c0, FUN_0019ea00 +0xf50)
    fn po_part_rec(&mut self, addr: u32, obj: usize, off: u32, rec: &mut k17_55::CamRec) {}
    /// (vtable +0x30c)(obj, &out): camera focus point
    fn vcall_30c(&mut self, obj: usize) -> [f32; 3] { [0.0; 3] }
    /// (vtable +0x114)(obj): velocity used for the camera speed scale
    fn vcall_114(&mut self, obj: usize) -> [f32; 3] { [0.0; 3] }
    /// other == DAT_002e1e40 (Sly) || other == DAT_002f7900 (Murray)
    fn is_player_2e1e40_2f7900(&mut self, other: u32) -> bool { false }
    /// *(world +0x150) +0x80
    fn world_150_80(&mut self) -> i32 { 0 }
    /// FUN_001fad00(obj, .., rec, .., side): SO push response of rec +0x40 (negated for side 1)
    fn ee_sub_1fad00(&mut self, obj: usize, push: [f32; 3], side: u32) {}
    /// LO +0x28 bit 60 (in the world)
    fn lo_28_bit60(&mut self, obj: u32) -> bool { false }
    /// FUN_00205008(obj, flags, other, ..): PO attach to `other` with a relative yaw
    fn ee_sub_205008(&mut self, obj: usize, other: u32, yaw: f32) {}
    /// FUN_002041c0(obj, v): PO translate (SO FUN_001f40a0, +0x230 = FUN_001a6f00(obj, 0, +0x240))
    fn ee_sub_2041c0(&mut self, obj: usize, v: [f32; 3]) {}
    /// `addr`(obj + `off`, s): JT-layout component SetState of a non-JT player character (FUN_00195fb8 +0x2c0,
    /// FUN_00192bd8 +0x4c0, FUN_001a0b40 +0xdf0, FUN_002206c8 +0x11b0, FUN_0019eab0 +0xf50)
    fn po_part_s(&mut self, addr: u32, obj: usize, off: u32, s: i32) {}
    /// FUN_00136298(obj): Bentley drops the carried object (+0x186c, +0x16b4 record; FUN_00224610, FUN_0018d698,
    /// FUN_001cb4b8); the two fields are cleared by the caller
    fn ee_sub_136298(&mut self, obj: usize) {}
    /// FUN_00136530(obj, s): Bentley state-8 sub-state
    fn ee_sub_136530(&mut self, obj: usize, s: i32) {}
    /// anim handle +0x1c / (*(handle +0x14)) +0x74: playback phase of an anim slot (0 none)
    fn anim_slot_phase(&mut self, slot: u32) -> f32 { 0.0 }
    /// anim +0x74 (length)
    fn anim_74(&mut self, anim: u32) -> f32 { 0.0 }
    /// FUN_001f0400(sm, label): state of `label` in the state machine (< 0 none)
    fn ee_sub_1f0400(&mut self, sm: u32, label: i32) -> i32 { -1 }
    /// FUN_001f03a8(sm, obj, label, flags, mask) -> link record (0 none)
    fn ee_sub_1f03a8_link(&mut self, sm: u32, obj: u32, label: i32, flags: u32, mask: i32) -> u32 { 0 }
    // ---- K18 Bentley rest / Murray (classes/k18_55.rs, classes/k18_54.rs)
    /// `addr`(obj + `off`, args..): an embedded JT-layout component / class-52 base function of a non-JT player
    /// character (Bentley / Murray) that player/ ports on a `Jt` (FUN_00195a00 +0x2c0, FUN_00197500, FUN_00197f98,
    /// FUN_00191a10 / 1911a8 / 1942f0 / 194558 / 1928f8 / 1938c0 +0x4c0, FUN_001a12f0 / 1a1358 +0xdf0, FUN_0019cc50
    /// / 19cce8 / 19d330 +0xf00, FUN_0019e1e0 / 19ea60 / 19eef0 +0xf50, FUN_0021f418 / 2206c8 +0x11b0, FUN_0019feb0
    /// +0x9e0, FUN_0020bb58 / 20bc08 on a blend block, FUN_00140af8 / 194a40 / 1a0450 / 1403d0 on the object)
    fn po_call(&mut self, addr: u32, obj: usize, off: u32, args: &[u32]) -> i64 { 0 }
    /// (vtable `off`)(obj, arg) -> value
    fn vcall_ret_arg(&mut self, obj: u32, off: u32, arg: i32) -> i64 { 0 }
    /// FUN_00133448(carried, carrier): release
    fn ee_sub_133448_carry(&mut self, carried: u32, carrier: u32) {}
    /// FUN_001333c0(carried, carrier): take
    fn ee_sub_1333c0_carry(&mut self, carried: u32, carrier: u32) {}
    /// FUN_00224610(obj): carried object keeps a carrier link
    fn ee_sub_224610(&mut self, obj: u32) -> bool { false }
    /// FUN_00224590(obj)
    fn ee_sub_224590(&mut self, obj: u32) -> bool { false }
    /// FUN_00224258(obj, obj +0x1a0): carried kind-2 object put down
    fn ee_sub_224258_carry(&mut self, obj: u32) {}
    /// *(obj + off) = v on a world object
    fn obj_set_u32(&mut self, obj: u32, off: u32, v: u32) {}
    /// FUN_00204740(obj): PO yaw of the velocity / facing
    fn ee_sub_204740(&mut self, obj: usize) -> f32 { 0.0 }
    /// FUN_00204ca8(obj): PO move of a non-guard object
    fn ee_sub_204ca8_po(&mut self, obj: usize) {}
    /// FUN_002046b0(obj, p): PO face point
    fn ee_sub_2046b0(&mut self, obj: usize, p: [f32; 3]) {}
    /// FUN_002046f0(obj): PO face (FUN_001a62e8 of the target)
    fn ee_sub_2046f0(&mut self, obj: usize) {}
    /// Bentley FUN_00136530(0) Murray mount (DAT_002f7900 +0x191c vcall +0x104, FUN_0022ff40, vcall +0x54 / +0x11c
    /// / +0x124, FUN_0012dc18 x2) -> the +0x1810 / +0x1814 act records
    fn ee_sub_136530_mount(&mut self, obj: usize) -> (u32, u32) { (0, 0) }
    /// Bentley FUN_00136530(-1): Murray (state 0xf, +0x1914 != 7) FUN_002206c8(+0x1910, 0)
    fn ee_sub_136530_release(&mut self) {}
    /// FUN_00137a30(obj, a, b, rec_a, 0, 0, 0, rec_b): blend record init (playerdraw::jtblend for the JT)
    fn ee_sub_137a30(&mut self, obj: usize, a: i32, b: i32, rec_a: u32, rec_b: u32) {}
    /// FUN_0018f2b0(yaw, 0.15, reach, obj, 2, v, &hit, &out): aim probe -> hit point
    fn ee_sub_18f2b0(&mut self, obj: usize, yaw: f32, a: f32, reach: f32, k: i32, v: [f32; 2]) -> Option<[f32; 3]> { None }
    /// FUN_001a6f00(a, b, 0)
    fn ee_sub_1a6f00_of(&mut self, a: u32, b: u32) -> [f32; 3] { [0.0; 3] }
    /// FUN_001384c0(rec): blend record `rec` (byte offset in obj) apply
    fn ee_sub_1384c0_rec(&mut self, obj: usize, rec: u32) {}
    /// FUN_00131b10(slot, obj, x)
    fn ee_sub_131b10(&mut self, slot: u32, obj: u32, x: u32) {}
    /// FUN_00131a90(slot, t)
    fn ee_sub_131a90(&mut self, slot: u32, t: f32) {}
    /// FUN_00131a00(rate, slot): anim +0x44 = rate, +0x40 = rate * +0x48 * +0x4c
    fn ee_sub_131a00(&mut self, rate: f32, slot: u32) {}
    /// anim slot float at `off` (+0x40 effective rate)
    fn anim_slot_f(&mut self, slot: u32, off: u32) -> f32 { 0.0 }
    /// FUN_00204e38(anim slot +0x1c): root-motion velocity
    fn root_vel_204e38(&mut self, slot: u32) -> [f32; 3] { [0.0; 3] }
    /// FUN_001f09d8(sm, label)
    fn ee_sub_1f09d8(&mut self, sm: u32, label: i32) {}
    /// FUN_001f0638(sm): state machine finished
    fn ee_sub_1f0638(&mut self, sm: u32) -> bool { false }
    /// FUN_001f1350(sm, mask)
    fn ee_sub_1f1350(&mut self, sm: u32, mask: i32) {}
    /// FUN_001357c0(obj, k): Bentley gadget fire (class-81 launcher FUN_001d6b00 path)
    fn ee_sub_1357c0(&mut self, obj: usize, k: u32) {}
    /// ground +0x208: Some((+0x40, +0x60)) when +0x88 > 0.7 and its owner +0x148 bit 29 is clear
    fn ground_208(&mut self, ground: u32) -> Option<(u32, u32)> { None }
    /// FUN_001cb250(host, k) -> spawned carry object (0 none)
    fn ee_sub_1cb250(&mut self, host: u32, k: i32) -> u32 { 0 }
    /// FUN_001cb488(host, obj)
    fn ee_sub_1cb488(&mut self, host: u32, obj: u32) {}
    /// FUN_001cade0(host, k)
    fn ee_sub_1cade0(&mut self, host: u32, k: i32) -> bool { false }
    /// FUN_00136ed0(obj + 0x1710, msg, args): aim block messages
    fn ee_sub_136ed0(&mut self, obj: usize, msg: i32, args: &[u32]) {}
    /// FUN_001eeed0(owner(x) + 0x26c)
    fn ee_sub_1eeed0_of(&mut self, x: u32) {}
    /// FUN_001426e8 rows: +0x1150 / +0x1160 / +0x1170 = FUN_0011b340(child +0x60 +0x40)
    fn ee_sub_1426e8_rows(&mut self, obj: usize, child: u32) {}
    /// FUN_0012d020(act)
    fn ee_sub_12d020(&mut self, act: u32) {}
    /// FUN_001a0300(obj, obj + a, obj + b)
    fn ee_sub_1a0300(&mut self, obj: usize, a: u32, b: u32) {}
    /// FUN_0020b868(obj + blk): blend block init
    fn ee_sub_20b868(&mut self, obj: usize, blk: u32) {}
    /// world +0x3d78 (FUN_0020f0b0)
    fn world_3d78(&mut self) -> i32 { 0 }
    /// DAT_002dfbc0
    fn g_2dfbc0(&mut self) -> u32 { 0 }
    /// FUN_0014fd50(0x2de2e0)
    fn ee_sub_14fd50(&mut self) -> bool { false }
    /// FUN_002297f8(obj, other): attachment contact (physics::driven::Attachment)
    fn ee_sub_2297f8(&mut self, obj: usize, other: u32) {}
    /// FUN_001a2788(obj, other, &out): landing probe
    fn ee_sub_1a2788(&mut self, obj: usize, other: u32) -> Option<[f32; 3]> { None }
    /// FUN_001a28b0 landing push (FUN_00228c18, FUN_0011d1c0, FUN_001fac30)
    fn ee_sub_1a28b0_push(&mut self, obj: usize, other: u32, p: [f32; 3]) {}
    /// FUN_001c4940(world, obj): focus change camera / HUD part (0 = none)
    fn ee_sub_1c4940(&mut self, obj: u32) {}
    /// FUN_001f82b8(obj, v, 0)
    fn ee_sub_1f82b8(&mut self, obj: usize, v: i32) {}
    // ---- K19 (Murray, classes/k19_54.rs)
    /// FUN_0012f938(anim, k): raw word of anim attribute `k` (None: absent)
    fn anim_attr(&mut self, anim: u32, k: i32) -> Option<u32> { None }
    /// *(obj + 0xe8) + 0x80: the PO's gravity vector
    fn phys_e8_80(&mut self, obj: usize) -> [f32; 3] { [0.0; 3] }
    /// (vtable +0x11c)(obj, v): set velocity
    fn vcall_11c_v(&mut self, obj: u32, v: [f32; 3]) {}
    /// DAT_003e13b4 object list (chained through +0xe0)
    fn list_3e13b4(&mut self) -> Vec<u32> { Vec::new() }
    /// FUN_0012ff10(anim, obj, 1, 1) root track: (pose at 0, pose at the anim end +0x74) (None: no track)
    fn root_track_12ff10(&mut self, anim: u32, obj: usize) -> Option<([f32; 3], [f32; 3])> { None }
    // ---- K20 (classes/k20_*.rs)
    /// unported engine function FUN_<addr>(args) on raw words (object handles / values); 0 = default
    fn ee_call(&mut self, addr: u32, args: &[u32]) -> i64 { 0 }
    /// FUN_001f5860(obj): *(obj + 0xe8) + 0x80 gravity of any object (zero without a PO)
    fn ee_sub_1f5860(&mut self, obj: u32) -> [f32; 3] { [0.0; 3] }
    /// FUN_001eeee0(obj + 0x26c, k): Some((record, record +0x50, record +0x5c)); None = no record
    fn ee_sub_1eeee0_rec(&mut self, obj: u32, k: i32) -> Option<(u32, i32, u32)> { None }
    /// DAT_002e0cb4 (pause gate bits)
    fn g_2e0cb4(&mut self) -> u32 { 0 }
    /// (vtable +0x124)(obj, v): set angular velocity
    fn vcall_124_v(&mut self, obj: u32, v: [f32; 3]) {}
    /// K21: the loaded world (the run-time spawns of FUN_001cb250 build their objects from it)
    fn k21_world(&mut self) -> Option<std::sync::Arc<World>> { None }
    /// K21: FUN_0014f308(0x2ddf40, q) camera projection (x, y, depth); None = no camera
    fn k21_cam_14f308(&mut self, q: [f32; 3]) -> Option<[f32; 3]> { None }
    /// K21: DAT_002ddfd0 camera position (0x2ddf40 +0x90); None = `ClassGlobals::g_2ddfd0`
    fn k21_cam_pos(&mut self) -> Option<[f32; 3]> { None }
    /// K21: a global word of the EE data (DAT_<addr>; 0 = unset)
    fn k21_global(&mut self, addr: u32) -> u32 { 0 }
    /// K21: FUN_0014cc58 segment a -> b against the world collision: any record = blocked
    fn k21_blocked_14cc58(&mut self, a: [f32; 3], b: [f32; 3]) -> bool { false }
    /// K21: FUN_00173610(d0, d1, &pos, 1, 0): first surface along pos + z d0 -> pos + z d1 (None = no hit)
    fn k21_ground_173610(&mut self, d0: f32, d1: f32, pos: [f32; 3]) -> Option<[f32; 3]> { None }
    // ---- K22 (classes/k22_*.rs)
    /// K22: the class-10 path of object handle `obj` (FUN_001e66xx / 1e67b8 / 1e6f60 / 1e77e8 run on it)
    fn k22_path(&mut self, obj: u32) -> Option<std::sync::Arc<crate::engine::player::p5_target::Path>> { None }
    /// K22: world (0x3dd490) +0x3ebc list entries with +0x28 bit 60 (FUN_001c4af8 candidates)
    fn k22_targets_3eb8(&mut self) -> Vec<u32> { Vec::new() }
    /// K22: (vtable +0x10c)(obj): velocity
    fn k22_speed_10c(&mut self, obj: u32) -> [f32; 3] { [0.0; 3] }
    /// K22: anim float at `off` (+0x78 rate)
    fn k22_anim_f(&mut self, anim: u32, off: u32) -> f32 { 0.0 }
    /// K22: (vtable +0xec)(obj, p): place at p
    fn k22_vcall_ec_pos(&mut self, obj: u32, p: [f32; 3]) {}
    /// K22: (vtable +off)(obj, k)
    fn k22_vcall_i(&mut self, obj: u32, off: u32, k: i32) {}
    /// K22: obj +0x150 tracker +0x10 / +0x14 / +0x20 / +0x24 (sticks; None = +0x150 null)
    fn k22_stick(&mut self, obj: u32) -> Option<[f32; 4]> { None }
    /// K22: FUN_002581c0(out, DAT_003e0fe0, 0x3d4a60) out word 1 (camera y-invert save setting)
    fn k22_setting_3e0fe0(&mut self) -> i32 { 0 }
    /// K22: FUN_0012dc18(obj, +0x5c, 1, 5, 3, v): rotation constraint -> handle
    fn k22_pin_12dc18(&mut self, obj: u32, v: [f32; 3]) -> u32 { 0 }
    /// K22: FUN_0012d108(handle, v): retarget the constraint
    fn k22_pin_12d108(&mut self, handle: u32, v: [f32; 3]) {}
    /// K22: *(obj +0xec) +0x30 (bounding radius; 0 = no +0xec)
    fn k22_radius_ec30(&mut self, obj: u32) -> f32 { 0.0 }
    /// K22: DAT_002ddf7c (render aspect factor, BSS)
    fn k22_aspect_2ddf7c(&mut self) -> f32 { 1.0 }
    /// K22: FUN_001d6b00(launcher, obj): launch the spawned object
    fn k22_launch_1d6b00(&mut self, launcher: u32, obj: u32) {}
    /// K22: FUN_001fe3f8(0x3dd040): the music stream position (s)
    fn k22_music_1fe3f8(&mut self) -> f32 { 0.0 }
    /// K22 sequence action 0x44: FUN_001dd820(0x3da530) shows the title / episode screen
    fn k22_title_show_1dd820(&mut self) {}
    /// K22: FUN_001d9880(0x3da530) the title / episode screen is active
    fn k22_title_active_1d9880(&mut self) -> bool { false }
    /// K22: an UNPORTED sequence action (owner handle, subject, the record)
    fn k22_seq_action(&mut self, owner: u32, subj: u32, rec: &[u32; 20]) {}
    /// K22 sequence action 7: (po vtable +0x27c)(po, 1) = slot 78, switch the controlled character
    fn k22_po_switch(&mut self, po: u32) {}
    /// K22: the action-7 switches of this frame (applied by `ClassSet::frame_k22` through `k18_set_active`)
    fn k22_take_po_switches(&mut self) -> Vec<u32> { Vec::new() }
    // ---- K23 (classes/k23_*.rs)
    /// K23: FUN_00205458(guard, 0xb0) hold component (+0x26c list): Some(its +0x30 state), None = absent
    fn k23_comp_b0(&mut self, guard: u32) -> Option<i32> { None }
    /// K23: FUN_0017a8c8(component 0xb0 of `guard`, s): set its state
    fn k23_comp_b0_set(&mut self, guard: u32, s: i32) {}
    /// K23: FUN_00209840(guard + 0xfd0, rows): the hold target orientation
    fn k23_guard_209840(&mut self, guard: u32, rows: [[f32; 4]; 3]) {}
    /// K23: guard +0xfc4 scale constraint handle (0 = none)
    fn k23_guard_fc4(&mut self, guard: u32) -> u32 { 0 }
    /// K23: FUN_00229710(obj, rec): link the record (owner `owner`) into obj +0x138, FUN_001f5078(obj)
    fn k23_link_229710(&mut self, obj: u32, owner: u32) {}
    /// K23: FUN_00206eb0(obj, k): the guard accepts hit kind `k`
    fn k23_guard_206eb0(&mut self, obj: u32, k: i32) -> bool { false }
    /// K23: FUN_00209e28(guard, a): brain (component 0x9c) message 6 with +0x10 = a (FUN_00176750)
    fn k23_brain_209e28(&mut self, guard: u32, a: i32) {}
    /// K25: once per class frame before the class slots: advance the run-time anim objects (classes/k25.rs)
    fn k25_tick(&mut self, dt: f32) {}
    /// K25: the class-owned bodies (`ClassSet::k25_bodies`) at the top of the class frame, for the physics hooks
    fn k25_bodies(&mut self, b: Vec<(usize, k25::BodySnap)>) {}
    /// K26: the FUN_001f13a8 messages (SMA owner handle, msg, {old, new}) queued by the run-time SMAs
    fn k26_take_msgs(&mut self) -> Vec<(u32, i32, crate::engine::anim::sm::SmaMsg)> { Vec::new() }
    /// K26: how many of them a ported slot-4 handler received (statistics)
    fn k26_handled(&mut self, n: usize) {}
    /// K27: the owner's slot 4 (vtable +0x2c) is the base FUN_00133590 (empty: the message is consumed)
    fn k27_slot4_base(&mut self, obj: usize) -> bool { false }
    /// K27: FUN_001331f8(obj, msg, 0, 1, args, 0): the object's level-script message `msg` (posted)
    fn k27_script_msg(&mut self, obj: u32, msg: u32, args: &[u32]) {}
    /// K27: FUN_0014f750(g_cm, kind) (camera shake)
    fn k27_shake_14f750(&mut self, kind: i32) {}
    /// K27: FUN_00187e28(duration, g_pad, id) (rumble)
    fn k27_rumble_187e28(&mut self, duration: f32, id: i32) {}
    /// K27: FUN_0020efc0(0x3dd490, flags) + FUN_0020f0c0 (world +0x3d78 / +0x3d7c counters up)
    fn k27_world_20efc0(&mut self, flags: u32) {}
    /// K27: FUN_0020f030(0x3dd490, flags) + FUN_0020f0f0 (counters down)
    fn k27_world_20f030(&mut self, flags: u32) {}
    /// K23: FUN_001c6fd8(0x3dd490, kind, pos, src, k) (1644 B, unported)
    fn k23_world_1c6fd8(&mut self, kind: i32, pos: [f32; 3], src: u32, k: i32) {}
    /// K23: FUN_0014d1d0(v, 0x2dded0, index): set a scale of the global clock (sequence action 0x10)
    fn k23_clock_14d1d0(&mut self, v: f32, index: usize) {}
    /// K23: FUN_001fe848(0x3dd040, id): stream `id` (started by FUN_001fe5f0) still playing
    fn k23_stream_1fe848(&mut self, id: i32) -> bool { false }
}

/// No-op host.
pub struct NullHost;
impl ClassHost for NullHost {}

/// World class id -> vtable for the ported classes.
pub const PORTED: &[(ClassId, u32)] = &[(43, 0x2cabd8), (33, 0x2ce4d0), (124, 0x2d5120), (10, 0x2d9040), (48, 0x2d9978)];

/// K3 world class id -> vtable (kept in `ClassSet::more`).
pub const PORTED_K3: &[(ClassId, u32)] = &[(29, 0x2ceb78), (6, 0x2d6e08), (133, 0x2cf1e8), (114, 0x2d8c48), (31, 0x2ce828), (4, 0x2d33a0), (111, 0x2d97a8), (26, 0x2dc6f0), (120, 0x2da840)];

/// K3 classes whose only overrides are stateless (slot 5 descriptor getters): no `Inst`.
pub const PORTED_K3_STATELESS: &[(ClassId, u32)] = &[(109, 0x2cad78)];

/// One ported instance.
#[derive(Clone, Debug)]
pub enum Inst {
    C2cabd8(c2cabd8::C2cabd8),
    C2ce4d0(c2ce4d0::C2ce4d0),
    C2ce828(c2ce828::C2ce828),
    C2ceb78(c2ceb78::C2ceb78),
    C2cf1e8(c2cf1e8::C2cf1e8),
    C2d33a0(c2d33a0::C2d33a0),
    C2d6e08(c2d6e08::C2d6e08),
    C2d97a8(c2d97a8::C2d97a8),
    C2dc6f0(c2dc6f0::C2dc6f0),
    C2da840(c2da840::C2da840),
    C2d8c48(c2d8c48::C2d8c48),
    C2d5120(c2d5120::C2d5120),
    C2d9040(c2d9040::C2d9040),
    C2d9978(c2d9978::C2d9978),
    C2db0d0(Box<c2db0d0::C2db0d0>),
    C2cf378(Box<c2cf378::C2cf378>),
    C2d5dc8(c2d5dc8::C2d5dc8),
    C2d6fe8(c2d6fe8::C2d6fe8),
    C2d56f8(c2d56f8::C2d56f8),
    C2cf880(c2cf880::C2cf880),
    C2dad90(c2dad90::C2dad90),
    C2cf058(c2cf058::C2cf058),
    C2d52b0(c2d52b0::C2d52b0),
    C2dd090(c2dd090::C2dd090),
    C2ce2f8(c2ce2f8::C2ce2f8),
    C2d9dd0(c2d9dd0::C2d9dd0),
    C2ca9f0(c2ca9f0::C2ca9f0),
    C2db6b8(c2db6b8::C2db6b8),
    K7(k7::K7),
    K8(Box<k8::Pickup>),
    K9(Box<k9::Vault>),
    K10(Box<k10::K10>),
    K11(Box<k11::K11>),
    C2dcc98(k12::C2dcc98),
    K13(Box<k13::C2d3d20>),
    K13Water(Box<k13::C2dca30>),
    K14(Box<k14::C2d0b30>),
    K15(Box<k15::C2d1080>),
    K15b(Box<k15_101::C2c9810>),
    K15c(Box<k15_79::C2cb780>),
    K15d(Box<k15_83::C2cc950>),
    K15e(Box<k15_85::C2d8738>),
    K16(Box<k16_55::C2cb230>),
    K18Murray(Box<k18_54::C2d44c8>),
    /// K20 class 77
    K20Truck(Box<k20_77::C2d7ca8>),
    /// K20 class 119
    K20Spawner(Box<k20_119::C2cfa90>),
    /// K21 classes (classes/k21_*.rs)
    K21C28(Box<k21_28::C2cb5d8>),
    K21C68(Box<k21_68::C2cd748>),
    /// K22 class 96 (classes/k22_96.rs)
    K22C96(Box<k22_96::C2d3838>),
    K22C115(Box<k22_115::C2dcf00>),
    K22C69(Box<k22_69::C2dbd80>),
    K22C121(Box<k22_121::C2cd5b8>),
    /// K23 class 102 (the class-101 bomb with the C2db8a0 members, classes/k23_102.rs)
    K23C102(Box<k15_101::C2c9810>),
    /// K23 class 32 (classes/k23_32.rs)
    K23C32(Box<k23_32::C2ca098>),
    /// K25 class 127 (classes/k25_127.rs)
    K25C127(k25_127::C2dabf0),
    C2caf88(Box<actor::c2caf88::C2caf88>),
    C2d4b78(Box<actor::c2d4b78::C2d4b78>),
    C2d4260(Box<actor::c2d4260::C2d4260>),
    Guard(Box<guard::c56::Guard>),
}

/// Ported instances of a level (object index, state).
#[derive(Clone, Debug, Default)]
pub struct ClassSet {
    pub g: ClassGlobals,
    pub insts: Vec<(usize, Inst)>,
    /// K3+ classes, kept apart so the K/K2 per-class counts of `insts` stay exact. Objects the overrides
    /// create at run time (class-29 groups) get indices from `w.objects.len()` up.
    pub more: Vec<(usize, Inst)>,
    /// FUN_0011d448 state (rand 0x2342a0)
    pub rng: GameRng,
    /// next run-time object index
    pub next_obj: usize,
    /// K4 classes (`PORTED_K4`), kept apart so the K3 counts of `more` stay exact
    pub k4: Vec<(usize, Inst)>,
    /// K4: FUN_00149a50 queue entries of K4 objects (object, member fn), drained at the end of the frame
    /// (FUN_00149c80 deferred phase)
    pub k4_deferred: Vec<(usize, u32)>,
    /// K5 classes (`actor::PORTED_K5`)
    pub k5: Vec<(usize, Inst)>,
    /// K6 guards (`guard::PORTED_K6`) and the level routines they resolve against
    pub k6: Vec<(usize, Inst)>,
    pub k6l: guard::K6Level,
    /// K7 classes (`k7::PORTED_K7`)
    pub k7: Vec<(usize, Inst)>,
    /// K8 pickups (`k8::PORTED_K8`)
    pub k8: Vec<(usize, Inst)>,
    /// K9 classes (`k9::PORTED_K9`)
    pub k9: Vec<(usize, Inst)>,
    /// K10 classes (`k10::PORTED_K10`)
    pub k10: Vec<(usize, Inst)>,
    /// K11 classes (`k11::PORTED_K11`)
    pub k11: Vec<(usize, Inst)>,
    /// K31 class 37 (`k31::PORTED_K31`)
    pub k31: Vec<(usize, k31::C2cb9f8)>,
    /// K12 classes (`k12::PORTED_K12`)
    pub k12: Vec<(usize, Inst)>,
    /// K13 classes (`k13::PORTED_K13`)
    pub k13: Vec<(usize, Inst)>,
    /// K14 classes (`k14::PORTED_K14`)
    pub k14: Vec<(usize, Inst)>,
    /// K14: FUN_00149a50 deferred slot-81 calls (object, other), drained at the end of `frame_k14`
    pub k14_deferred: Vec<(usize, u32)>,
    /// K15 classes (`k15::PORTED_K15`)
    pub k15: Vec<(usize, Inst)>,
    /// K15: FUN_00149a50 deferred slot-81 calls (object, other), drained at the end of `frame_k15`
    pub k15_deferred: Vec<(usize, u32)>,
    /// K16 class 55 Bentley (`k16_55::PORTED_K16`)
    pub k16: Vec<(usize, Inst)>,
    /// K18 class 54 Murray (`k18_54`)
    pub k18: Vec<(usize, Inst)>,
    /// K18: DAT_003e138c / world +0x3efc focus among the ported Bentley / Murray instances (object index)
    pub k18_active: Option<usize>,
    /// K20 classes (class 77 `k20_77`, class 119 `k20_119`)
    pub k20: Vec<(usize, Inst)>,
    /// K21: class-119 spawn state (alert records, template copy owners, spawned guards)
    pub k21: k21_119::K21,
    /// K21 classes 28, 68 (classes/k21_*.rs) and 49 (actor/c2d4260.rs)
    pub k21c: Vec<(usize, Inst)>,
    /// K28: emitter object -> its class-28 system (the FUN_0015bd20 ctx slot, C2ceb78 +0x150)
    pub k28_sys: Vec<(usize, usize)>,
    /// K22 classes (classes/k22_*.rs)
    pub k22c: Vec<(usize, Inst)>,
    /// K23 classes (classes/k23_*.rs)
    pub k23c: Vec<(usize, Inst)>,
    /// K25 classes (classes/k25_*.rs)
    pub k25c: Vec<(usize, Inst)>,
}

impl ClassSet {
    /// Construct + init (slot 6) every ported-class object of `w` (clones included).
    pub fn build(w: &World, h: &mut dyn ClassHost) -> ClassSet {
        let mut s = ClassSet::default();
        for (i, o) in w.objects.iter().enumerate() {
            let inst = match o.class {
                43 => {
                    // K26: the +0xe8 / +0xf4 entry lists from the property methods (FUN_0012eb78)
                    let mut c = c2cabd8::C2cabd8 { added: !crate::engine::game::fields::out_of_world_at_load(w, i), ..Default::default() };
                    if let Some(p) = props_of(w, o.clone_of.unwrap_or(i)) {
                        c.feed(p);
                    }
                    Inst::C2cabd8(c)
                }
                33 => {
                    let mut c = c2ce4d0::C2ce4d0 { tpl: k23_32::tpl_new(), ..Default::default() };
                    if let Some(Payload::Loaded(LoadData::B4(b))) = w.objects.get(o.clone_of.unwrap_or(i)).map(|x| &x.payload) {
                        if let B4Load::L258598 { props, kind, .. } = &**b {
                            // K23: class-32 template fields (FUN_002590f0) + load kind
                            k23_32::feed_tpl(&mut c.tpl, &mut c.f_e0, &mut c.bx, props, *kind);
                        }
                    }
                    if let Some(p) = props_of(w, o.clone_of.unwrap_or(i)) {
                        c.feed(p);
                    }
                    Inst::C2ce4d0(c)
                }
                124 => Inst::C2d5120(c2d5120::C2d5120::default()),
                10 => Inst::C2d9040(c2d9040::C2d9040::default()),
                48 => Inst::C2d9978(c2d9978::C2d9978 { owner: o.owner, ..Default::default() }),
                _ => continue,
            };
            s.insts.push((i, inst));
        }
        for k in 0..s.insts.len() {
            let i = s.insts[k].0;
            // C2d9978 init reads the owner's +0x148 bit 35
            let owner_bit = match &s.insts[k].1 {
                Inst::C2d9978(c) => c.owner.map(|ow| s.bit35_of(ow).unwrap_or_else(|| h.owner_bit35(ow))),
                _ => None,
            };
            match &mut s.insts[k].1 {
                Inst::C2cabd8(c) => c.ee_sub_12eb48(i, h),
                Inst::C2ce4d0(_) => {}
                Inst::C2d5120(c) => c.ee_sub_1c2048(i, h),
                Inst::C2d9040(c) => c.ee_sub_1e5d80(i, h),
                Inst::C2d9978(c) => c.ee_sub_1f2108(i, owner_bit, s.g.g_2c3b90, h),
                _ => {}
            }
        }
        s.build_more(w, h);
        s.build_k4(w, h);
        s.build_k5(w, h);
        s.build_k6(w, h);
        s.build_k7(w, h);
        s.build_k8(w, h);
        s.build_k9(w, h);
        s.build_k10(w, h);
        s.build_k11(w, h);
        s.build_k12(w, h);
        s.build_k13(w, h);
        s.build_k14(w, h);
        s.build_k15(w, h);
        s.build_k15_101(w, h);
        s.build_k15_79(w, h);
        s.build_k15_83(w, h);
        s.build_k15_85(w, h);
        s.build_k16(w, h);
        s.build_k18_54(w, h);
        s.build_k20_77(w, h);
        s.build_k21_77(w);
        s.build_k20_119(w, h);
        s.build_k21(w, h);
        s.build_k22(w, h);
        s.build_k23(w, h);
        s.build_k25(w, h);
        s.build_k31(w, h);
        s
    }
    /// Fire slot 51 (FUN_0015c690) of every class-33 instance with request flags `r`.
    pub fn emit_all(&mut self, r: &[u32; 15], rng: &mut GameRng, h: &mut dyn ClassHost) {
        for (i, x) in &mut self.insts {
            if let Inst::C2ce4d0(c) = x {
                c.ee_sub_15c690(*i, r, rng, h);
            }
        }
    }
    fn bit35_of(&self, obj: usize) -> Option<bool> {
        self.insts.iter().find(|(i, _)| *i == obj).and_then(|(_, x)| match x {
            Inst::C2d9978(c) => Some(c.bit35()),
            _ => None,
        })
    }
    /// Load-post (slot 14) of every instance.
    pub fn post_load(&mut self, h: &mut dyn ClassHost) {
        for (i, x) in &mut self.insts {
            match x {
                Inst::C2cabd8(c) => c.ee_sub_12ec50(*i, &self.g, h),
                Inst::C2d5120(c) => c.ee_sub_1c2170(*i, h),
                _ => {}
            }
        }
        self.post_load_more(h);
        self.post_load_k4(h);
        self.post_load_k5(h);
        self.post_load_k6(h);
        self.post_load_k7(h);
        self.post_load_k8(h);
        self.post_load_k9(h);
        self.post_load_k10(h);
        self.post_load_k11(h);
        self.post_load_k12(h);
        self.post_load_k13(h);
        self.post_load_k14(h);
        self.post_load_k15(h);
        self.post_load_k15_101(h);
        self.post_load_k15_79(h);
        self.post_load_k15_83(h);
        self.post_load_k15_85(h);
        self.post_k18_55(h);
        self.post_k18_54(h);
        self.post_load_k20_77(h);
        self.post_load_k20_119(h);
        self.post_load_k21(h);
        self.post_load_k22(h);
        self.post_load_k23(h);
        self.post_load_k25(h);
        self.post_load_k26(h);
        self.post_load_k31(h);
    }
    /// One frame of the per-frame overrides (C2d9040 slot 16, C2d9978 slot 18).
    pub fn frame(&mut self, h: &mut dyn ClassHost) {
        // K25: live ASEGAs / SMAs (FUN_00130918 over 0x3e1218) before the class slots read them
        h.k25_tick(self.g.dt);
        // K25: velocities / gravity of the class-owned bodies for vcall_10c / FUN_0022d348 / +0xe8 reads
        h.k25_bodies(self.k25_bodies());
        // K26: SMA state-change messages FUN_001f13a8 of the tick -> owner slot 4 (classes/k26.rs)
        self.frame_k26(h);
        for (i, x) in &mut self.insts {
            match x {
                Inst::C2d9040(c) => c.ee_sub_1e6340(*i, h),
                Inst::C2d9978(c) => c.ee_sub_1f3c70(*i, self.g.g_2ddf54, h),
                _ => {}
            }
        }
        self.frame_more(h);
        self.frame_k4(h);
        self.frame_k5(h);
        self.frame_k6(h);
        self.frame_k7_classes(h);
        self.frame_k8(h);
        self.frame_k9(h);
        self.frame_k10(h);
        self.frame_k11(h);
        self.frame_k12(h);
        self.frame_k13(h);
        self.frame_k14(h);
        self.frame_k15(h);
        // K24: class 101 / 102 with the guard-routing host (classes/k24.rs)
        self.with_k24(h, |s, h| s.frame_k15_101(h));
        self.frame_k15_79(h);
        self.frame_k15_83(h);
        self.frame_k15_85(h);
        self.frame_k18_55(h);
        self.frame_k18_54(h);
        self.frame_k20_77(h);
        self.frame_k21_119(h);
        self.frame_k21(h);
        self.frame_k22(h);
        self.frame_k25(h);
    }

    /// K3: construct + feed + init (slot 6) the `PORTED_K3` classes.
    fn build_more(&mut self, w: &World, h: &mut dyn ClassHost) {
        self.next_obj = w.objects.len();
        for (i, o) in w.objects.iter().enumerate() {
            let src = o.clone_of.unwrap_or(i);
            let b4 = match w.objects.get(src).map(|x| &x.payload) {
                Some(Payload::Loaded(LoadData::B4(b))) => Some(b.as_ref()),
                _ => None,
            };
            let inst = match o.class {
                29 => {
                    let mut c = c2ceb78::C2ceb78::default();
                    c.ee_sub_1591b0(i, h);
                    if let Some(B4Load::L258598 { props, kind, .. }) = b4 {
                        c.feed(props, *kind);
                    }
                    Inst::C2ceb78(c)
                }
                133 => {
                    let mut c = c2cf1e8::C2cf1e8::default();
                    c.ee_sub_1635a0(i, &mut self.g, h);
                    if let Some(B4Load::L1635f8 { props, a, b, .. }) = b4 {
                        c.feed(props, a, b, &mut |oid| h.ee_sub_15e938(0, oid, 0));
                    }
                    Inst::C2cf1e8(c)
                }
                114 => {
                    let mut c = c2d8c48::C2d8c48::default();
                    c.ee_sub_1dfad8(i, h);
                    if let Some(Payload::Loaded(LoadData::Lo(props))) = w.objects.get(src).map(|x| &x.payload) {
                        c.feed(props);
                    }
                    Inst::C2d8c48(c)
                }
                4 => {
                    let mut c = c2d33a0::C2d33a0::default();
                    if let Some(p) = props_of(w, src) {
                        c.feed(p);
                    }
                    Inst::C2d33a0(c)
                }
                120 => {
                    let mut c = c2da840::C2da840::default();
                    c.oid = o.oid;
                    let mut a = o.owner;
                    while let Some(k) = a {
                        if crate::engine::world::class::ee_sub_15e988(w.objects[k].class, 0x38) {
                            break;
                        }
                        a = w.objects[k].owner;
                    }
                    c.anc = a.map(|k| k as u32 + 1).unwrap_or(0);
                    c.ee_sub_20ad38(i, h);
                    if let Some(Payload::Loaded(LoadData::Lo(props))) = w.objects.get(src).map(|x| &x.payload) {
                        c.feed(i, props, h);
                    }
                    Inst::C2da840(c)
                }
                26 => {
                    let mut c = c2dc6f0::C2dc6f0::default();
                    c.ee_sub_2281e8(i, h);
                    if let Some(Payload::Loaded(LoadData::L2282b0 { solid, first, list })) = w.objects.get(src).map(|x| &x.payload) {
                        c.feed(first.is_some(), list.len(), &solid.props, &mut self.g, h, i);
                    }
                    Inst::C2dc6f0(c)
                }
                111 => {
                    let mut c = c2d97a8::C2d97a8::default();
                    c.ee_sub_1efa90(i, h);
                    if let Some(B4Load::L1efac0 { props, .. }) = b4 {
                        c.feed(props);
                    }
                    Inst::C2d97a8(c)
                }
                31 => {
                    let mut c = c2ce828::C2ce828::default();
                    c.feed(w.life.attached.get(&i).map(|v| v.as_slice()).unwrap_or(&[]));
                    Inst::C2ce828(c)
                }
                6 => {
                    h.base(0x1ca788, i); // slot 6 inherited from C2d5dc8 (unported)
                    Inst::C2d6e08(c2d6e08::C2d6e08::default())
                }
                _ => continue,
            };
            self.more.push((i, inst));
        }
    }
    /// K3 load-post (slot 14).
    fn post_load_more(&mut self, h: &mut dyn ClassHost) {
        let mut created: Vec<(i32, usize)> = Vec::new();
        for k in 0..self.more.len() {
            let i = self.more[k].0;
            match &mut self.more[k].1 {
                Inst::C2d6e08(c) => c.ee_sub_1a5c48(i, h),
                Inst::C2d97a8(c) => c.ee_sub_1f00c8(i, h),
                Inst::C2da840(c) => {
                    let (anc, oid) = (c.anc, c.oid);
                    c.ee_sub_20ae18(i, anc, oid, h)
                }
                Inst::C2d33a0(c) => {
                    c.ee_sub_1a3400(i, &mut self.g, h);
                    // assumption: lights enter the world (slot 22, FUN_001a3d00 registration) right after load
                    c.ee_sub_1a3518(i, &mut self.g.lights, h);
                }
                Inst::C2cf1e8(c) => c.ee_sub_1638f8(i, &self.g, h),
                _ => {}
            }
            let Inst::C2ceb78(c) = &mut self.more[k].1 else { continue };
            let c2ceb78::Group::Join(oid) = c.ee_sub_159500(i, &mut self.rng, &self.g, h) else { continue };
            // FUN_0015e938(0x3dd490, 5, oid, 0), else FUN_0013e3c8(0x1d, 0, oid, -1) + state copy
            let found = created.iter().find(|x| x.0 == oid).map(|x| x.1).or_else(|| match h.ee_sub_15e938(5, oid, 0) {
                0 => None,
                r => Some(r as usize - 1),
            });
            let g = match found {
                Some(g) => g,
                None => {
                    let g = self.next_obj;
                    self.next_obj += 1;
                    let n = c.new_group(h, g);
                    created.push((oid, g));
                    self.more.push((g, Inst::C2ceb78(n)));
                    g
                }
            };
            // FUN_0011a220(group + 0x11c, obj)
            if let Some((_, Inst::C2ceb78(gc))) = self.more.iter_mut().find(|x| x.0 == g) {
                gc.f_11c.push(i as u32 + 1);
            }
        }
    }
    /// K3 per-frame slots (C2ceb78 slot 15).
    fn frame_more(&mut self, h: &mut dyn ClassHost) {
        let mut chain = Vec::new();
        // K27: sequence guard actions 0x55 / 0x56 (FUN_00171cf8 / FUN_00171d58), run on the guards after
        // the sequence object's update
        let mut running = self.k27_running();
        let mut ops = Vec::new();
        for (i, x) in &mut self.more {
            match x {
                Inst::C2dc6f0(c) => chain.extend(c.ee_sub_228538(*i, &mut self.g, h)),
                Inst::C2ceb78(c) => c.ee_sub_159c80(*i, &mut self.rng, &self.g, h),
                Inst::C2cf1e8(c) => c.ee_sub_163b48(*i, &self.g, h),
                Inst::C2d8c48(c) => {
                    c.k27_running.clone_from(&running);
                    c.ee_sub_1dfc50(*i, &self.g, h);
                    running.clone_from(&c.k27_running);
                    ops.append(&mut c.k27_ops);
                    for f in c.k27_world.drain(..) {
                        h.k27_world_20f030(f);
                    }
                }
                _ => {}
            }
        }
        for op in ops {
            self.k27_guard_op(op, h);
        }
        // FUN_002283d0 hand-over (FUN_002280d0 object); ends because it only acts in state 3 and leaves it
        while let Some(o) = chain.pop() {
            if let Some((i, Inst::C2dc6f0(c))) = self.more.iter_mut().find(|x| x.0 + 1 == o as usize) {
                chain.extend(c.ee_sub_2283d0(*i, &mut self.g, h));
            }
        }
    }
}
