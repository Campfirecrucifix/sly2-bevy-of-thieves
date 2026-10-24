//! WP-K24: GENERATED (do not edit by hand): `impl ClassHost for K24Host` - every method forwards to the
//! wrapped host except the guard-routed ones, which call `K24Host::r_<name>` (classes/k24.rs). Regenerate
//! with the trait when ClassHost changes (python tools/engineport/k24_gen.py <comma-separated routed names>).
#![allow(unused_imports, clippy::too_many_arguments)]
use super::*;
use super::k24::K24Host;

impl ClassHost for K24Host<'_> {
    fn base(&mut self, addr: u32, obj: usize) {
        self.h.base(addr, obj)
    }
    fn ee_sub_1a6ca8(&mut self, obj: usize, a: i32) {
        self.h.ee_sub_1a6ca8(obj, a)
    }
    fn ee_sub_169198(&mut self, member: u32, owner: usize) {
        self.h.ee_sub_169198(member, owner)
    }
    fn ee_sub_133448(&mut self, p: i32, obj: usize) {
        self.h.ee_sub_133448(p, obj)
    }
    fn sub_vcall_1a4(&mut self, obj: usize) {
        self.h.sub_vcall_1a4(obj)
    }
    fn ee_sub_1c1f28(&mut self, obj: usize, off: u32) -> [u32; 4] {
        self.h.ee_sub_1c1f28(obj, off)
    }
    fn ee_sub_1c0d70(&mut self, v: [u32; 4], a: i32) -> i32 {
        self.h.ee_sub_1c0d70(v, a)
    }
    fn emitter_attach(&mut self, obj: usize, entry: [i32; 4], g_3dd268: i32) {
        self.h.emitter_attach(obj, entry, g_3dd268)
    }
    fn emitter_release(&mut self, obj: usize, slot: usize) {
        self.h.emitter_release(obj, slot)
    }
    fn emitter_create(&mut self, obj: usize, k: usize, e: [i32; 5]) -> i32 {
        self.h.emitter_create(obj, k, e)
    }
    fn ee_sub_1a5e80(&mut self, obj: usize) {
        self.h.ee_sub_1a5e80(obj)
    }
    fn ee_sub_1e8378(&mut self, p: i32) {
        self.h.ee_sub_1e8378(p)
    }
    fn ee_sub_1e9090(&mut self) {
        self.h.ee_sub_1e9090()
    }
    fn state_4c(&mut self, p: i32) -> i32 {
        self.h.state_4c(p)
    }
    fn ee_sub_1e7840(&mut self, obj: usize, a: i32) {
        self.h.ee_sub_1e7840(obj, a)
    }
    fn ee_sub_11a1f8(&mut self, obj: usize, off: u32, n: u32) {
        self.h.ee_sub_11a1f8(obj, off, n)
    }
    fn ee_sub_1f3c10(&mut self, obj: usize) {
        self.h.ee_sub_1f3c10(obj)
    }
    fn ee_sub_168638(&mut self, obj: usize, f: f32, a: i32) {
        self.h.ee_sub_168638(obj, f, a)
    }
    fn owner_bit35(&mut self, owner: usize) -> bool {
        self.h.owner_bit35(owner)
    }
    fn ee_sub_11d948(&mut self, x: f32, c: u32) -> f32 {
        self.h.ee_sub_11d948(x, c)
    }
    fn ee_sub_256ce8(&mut self, v: &mut [u32], k: f32) {
        self.h.ee_sub_256ce8(v, k)
    }
    fn ee_sub_15bd20(&mut self, obj: usize, count: i32, t: &[u32; 64], blk: &[u32; 5], link: bool) {
        self.h.ee_sub_15bd20(obj, count, t, blk, link)
    }
    fn emit_pos_15c690(&mut self, t: &[u32; 64], blk: &[u32; 5]) -> [f32; 4] {
        self.h.emit_pos_15c690(t, blk)
    }
    fn sound_15c690(&mut self, src: u32, pos: [f32; 4], sd: [u32; 5]) {
        self.h.sound_15c690(src, pos, sd)
    }
    fn ee_sub_15e938(&mut self, kind: i32, oid: i32, ctx: u32) -> u32 {
        self.h.ee_sub_15e938(kind, oid, ctx)
    }
    fn ee_sub_149a50(&mut self, obj: usize, pmf: u32, arg: i32) {
        self.h.ee_sub_149a50(obj, pmf, arg)
    }
    fn lo_rt(&mut self, obj: usize, on: bool) {
        self.h.lo_rt(obj, on)
    }
    fn vcall_f4_1a6318(&mut self, obj: usize, src: u32) {
        self.h.vcall_f4_1a6318(obj, src)
    }
    fn ee_sub_1394c8(&mut self, r: i32, t: &c2ce4d0::Template) {
        self.h.ee_sub_1394c8(r, t)
    }
    fn ee_sub_1cde70(&mut self, r: i32, t: &c2ce4d0::Template) {
        self.h.ee_sub_1cde70(r, t)
    }
    fn ee_sub_1a5e28(&mut self, obj: usize) {
        self.h.ee_sub_1a5e28(obj)
    }
    fn ee_sub_1a5d98(&mut self, obj: usize) {
        self.h.ee_sub_1a5d98(obj)
    }
    fn vcall_1c(&mut self, obj: usize, a: i32, b: i32) {
        self.h.vcall_1c(obj, a, b)
    }
    fn k27_script_msg(&mut self, obj: u32, msg: u32, args: &[u32]) {
        self.h.k27_script_msg(obj, msg, args)
    }
    fn ee_sub_228170(&mut self, warp: u32) {
        self.h.ee_sub_228170(warp)
    }
    fn ee_sub_127db8(&mut self, obj: usize, id: i32, deep: i32) -> bool {
        self.h.ee_sub_127db8(obj, id, deep)
    }
    fn ee_sub_20e040(&mut self) -> i32 {
        self.h.ee_sub_20e040()
    }
    fn ee_sub_11a070(&mut self, obj: u32, off: u32, item: u32) {
        self.h.ee_sub_11a070(obj, off, item)
    }
    fn ee_sub_1620b0(&mut self, obj: u32, k: i32, a: i32, b: i32) {
        self.h.ee_sub_1620b0(obj, k, a, b)
    }
    fn ee_sub_1623d0(&mut self, obj: u32, k: i32, a: i32, b: i32, c: i32) {
        self.h.ee_sub_1623d0(obj, k, a, b, c)
    }
    fn ee_sub_2581c0(&mut self, path: &[u16]) -> u32 {
        self.h.ee_sub_2581c0(path)
    }
    fn ee_sub_165118(&mut self, obj: u32, a: i32) {
        self.h.ee_sub_165118(obj, a)
    }
    fn vcall(&mut self, obj: u32, off: u32) {
        self.r_vcall(obj, off)
    }
    fn ee_sub_1e1c40(&mut self, node: u32) {
        self.h.ee_sub_1e1c40(node)
    }
    fn ee_sub_1e5628(&mut self, node: u32) {
        self.h.ee_sub_1e5628(node)
    }
    fn ee_sub_1e0120(&mut self, obj: u32) {
        self.h.ee_sub_1e0120(obj)
    }
    fn ee_sub_1a61a0(&mut self, obj: u32) -> bool {
        self.h.ee_sub_1a61a0(obj)
    }
    fn ee_sub_1a62e8(&mut self, obj: u32) -> [f32; 3] {
        self.r_ee_sub_1a62e8(obj)
    }
    fn obj_i32(&mut self, obj: u32, off: u32) -> i32 {
        self.r_obj_i32(obj, off)
    }
    fn obj_f32(&mut self, obj: u32, off: u32) -> f32 {
        self.h.obj_f32(obj, off)
    }
    fn list_len(&mut self, obj: u32, off: u32) -> i32 {
        self.h.list_len(obj, off)
    }
    fn wrap_particles(&mut self, r: i32, lo: [f32; 3], hi: [f32; 3], span: f32) {
        self.h.wrap_particles(r, lo, hi, span)
    }
    fn obj_mat(&mut self, obj: u32, off: u32) -> crate::engine::anim::pose::Mat4 {
        self.h.obj_mat(obj, off)
    }
    fn ee_sub_13e510(&mut self, obj: u32, owner: u32) -> u32 {
        self.h.ee_sub_13e510(obj, owner)
    }
    fn vcall_19c(&mut self, obj: u32, req: &[u32; 15], m: &c2ce828::ReqBlock) {
        self.h.vcall_19c(obj, req, m)
    }
    fn parent_a4(&mut self, obj: u32) -> i32 {
        self.h.parent_a4(obj)
    }
    fn ee_sub_1a3b80(&mut self, far: f32, bsp: u32, pos: [f32; 3], slot: i32) {
        self.h.ee_sub_1a3b80(far, bsp, pos, slot)
    }
    fn ee_sub_1f0380(&mut self, obj: usize, target: u32, a: u32, b: u32) {
        self.h.ee_sub_1f0380(obj, target, a, b)
    }
    fn ee_sub_20efc0(&mut self, k: i32) {
        self.h.ee_sub_20efc0(k)
    }
    fn ee_sub_161f48(&mut self, obj: usize) {
        self.h.ee_sub_161f48(obj)
    }
    fn ee_sub_14e4c8(&mut self, desc: u32) {
        self.h.ee_sub_14e4c8(desc)
    }
    fn ee_sub_2280d0(&mut self) -> u32 {
        self.h.ee_sub_2280d0()
    }
    fn ee_sub_1c4840(&mut self) -> u32 {
        self.h.ee_sub_1c4840()
    }
    fn ee_sub_1a6f00(&mut self, obj: usize, pos: [f32; 3]) -> [f32; 3] {
        self.h.ee_sub_1a6f00(obj, pos)
    }
    fn vol_contains(&mut self, obj: usize, pos: [f32; 3]) -> bool {
        self.h.vol_contains(obj, pos)
    }
    fn ee_sub_118fb0(&mut self, obj: usize, pos: [f32; 3]) -> bool {
        self.h.ee_sub_118fb0(obj, pos)
    }
    fn ee_sub_1b3a48(&mut self, obj: usize, k: i32) {
        self.h.ee_sub_1b3a48(obj, k)
    }
    fn ee_sub_22b490(&mut self, obj: usize, level: i32, flags: i32, a: i32) {
        self.h.ee_sub_22b490(obj, level, flags, a)
    }
    fn ee_sub_2090d0(&mut self, obj: usize, anc: u32) {
        self.h.ee_sub_2090d0(obj, anc)
    }
    fn ee_sub_209130(&mut self, obj: usize, anc: u32) {
        self.h.ee_sub_209130(obj, anc)
    }
    fn ee_sub_209590(&mut self, obj: usize) {
        self.h.ee_sub_209590(obj)
    }
    fn ee_sub_20aff0(&mut self, obj: usize, a: [u32; 5]) {
        self.h.ee_sub_20aff0(obj, a)
    }
    fn ee_sub_1f4c58(&mut self, alo: u32, oid: i32, k: i32) -> u32 {
        self.h.ee_sub_1f4c58(alo, oid, k)
    }
    fn ee_sub_1300b8(&mut self, anim: u32, k: i32, alo: u32, b: u32, a: u32) -> [u32; 4] {
        self.h.ee_sub_1300b8(anim, k, alo, b, a)
    }
    fn vcall_fc(&mut self, obj: u32) -> crate::engine::anim::pose::Mat4 {
        self.h.vcall_fc(obj)
    }
    fn mesh_dc(&mut self, obj: u32) -> Vec<c2db0d0::Group105> {
        self.h.mesh_dc(obj)
    }
    fn set_mesh_cb(&mut self, obj: u32, idx: usize, cb: u32) {
        self.h.set_mesh_cb(obj, idx, cb)
    }
    fn ee_sub_16ef00(&mut self, obj: u32, idx: usize) {
        self.h.ee_sub_16ef00(obj, idx)
    }
    fn so_draw_1f3c70(&mut self, obj: usize, alpha: f32) {
        self.h.so_draw_1f3c70(obj, alpha)
    }
    fn ee_sub_14e4c8_k(&mut self, k: i32, a: i32, desc: u32, b: i32, obj: u32) {
        self.h.ee_sub_14e4c8_k(k, a, desc, b, obj)
    }
    fn ee_sub_14e328(&mut self, k: i32, desc: u32, target: u32, obj: u32) {
        self.h.ee_sub_14e328(k, desc, target, obj)
    }
    fn ee_sub_1c4a48(&mut self) -> u32 {
        self.h.ee_sub_1c4a48()
    }
    fn ee_sub_118fb0_at(&mut self, obj: usize, off: u32, pos: [f32; 3]) -> bool {
        self.h.ee_sub_118fb0_at(obj, off, pos)
    }
    fn ee_sub_132b10(&mut self, obj: u32, class: i32) -> u32 {
        self.h.ee_sub_132b10(obj, class)
    }
    fn ee_sub_2121e8(&mut self, seq: u32, id: i32) -> bool {
        self.h.ee_sub_2121e8(seq, id)
    }
    fn ee_sub_213b00(&mut self, seq: u32, id: i32) {
        self.h.ee_sub_213b00(seq, id)
    }
    fn vcall_arg(&mut self, obj: u32, off: u32, a: i32) {
        self.h.vcall_arg(obj, off, a)
    }
    fn ee_sub_1a4e70(&mut self) -> bool {
        self.h.ee_sub_1a4e70()
    }
    fn ee_sub_1a4e80(&mut self) {
        self.h.ee_sub_1a4e80()
    }
    fn obj_or_40(&mut self, obj: u32, bits: u32) {
        self.h.obj_or_40(obj, bits)
    }
    fn ee_sub_2283d0(&mut self, obj: u32) {
        self.h.ee_sub_2283d0(obj)
    }
    fn ee_sub_164640(&mut self, obj: u32, k: i32) {
        self.h.ee_sub_164640(obj, k)
    }
    fn ee_sub_164df0(&mut self, obj: u32, k: i32) {
        self.h.ee_sub_164df0(obj, k)
    }
    fn ee_sub_1619a0(&mut self, k: i32, obj: u32) {
        self.h.ee_sub_1619a0(k, obj)
    }
    fn ee_sub_1abd98(&mut self, a: i32) {
        self.h.ee_sub_1abd98(a)
    }
    fn ee_sub_20f030(&mut self, k: i32) {
        self.h.ee_sub_20f030(k)
    }
    fn ee_sub_1d7e80(&mut self, path: &[u16]) -> u32 {
        self.h.ee_sub_1d7e80(path)
    }
    fn ee_sub_165dc0(&mut self, a: i32) {
        self.h.ee_sub_165dc0(a)
    }
    fn ee_sub_165e10(&mut self, a: i32) {
        self.h.ee_sub_165e10(a)
    }
    fn g_3e0fa4_7c(&mut self) -> u32 {
        self.h.g_3e0fa4_7c()
    }
    fn ee_sub_210b70_same(&mut self) -> bool {
        self.h.ee_sub_210b70_same()
    }
    fn ee_sub_1c7bb8(&mut self, serial: i32) -> bool {
        self.h.ee_sub_1c7bb8(serial)
    }
    fn ee_sub_149ab8(&mut self, obj: usize, pmf: u32, arg: i32) {
        self.h.ee_sub_149ab8(obj, pmf, arg)
    }
    fn ee_sub_1caa60(&mut self, obj: usize, item: u32) {
        self.h.ee_sub_1caa60(obj, item)
    }
    fn ee_sub_11a380(&mut self, obj: u32) -> bool {
        self.h.ee_sub_11a380(obj)
    }
    fn ee_sub_11a2e8(&mut self, obj: u32) {
        self.h.ee_sub_11a2e8(obj)
    }
    fn ref_14(&mut self, obj: u32) -> u32 {
        self.h.ref_14(obj)
    }
    fn ee_sub_127e90(&mut self, r: u32, obj: u32) -> u32 {
        self.h.ee_sub_127e90(r, obj)
    }
    fn ref_release(&mut self, r: u32) {
        self.h.ref_release(r)
    }
    fn ee_sub_259a88(&mut self, r: u32) {
        self.h.ee_sub_259a88(r)
    }
    fn ee_sub_182c70_tail(&mut self, obj: usize) {
        self.h.ee_sub_182c70_tail(obj)
    }
    fn ee_sub_1e7d90(&mut self, obj: usize) -> f32 {
        self.h.ee_sub_1e7d90(obj)
    }
    fn ee_sub_1e9c30(&mut self, obj: u32) {
        self.h.ee_sub_1e9c30(obj)
    }
    fn ee_sub_21ecd8(&mut self, text: u32, v: i32) {
        self.h.ee_sub_21ecd8(text, v)
    }
    fn ee_sub_15dd48_draw(&mut self, rgba: u32, kind: i32) {
        self.h.ee_sub_15dd48_draw(rgba, kind)
    }
    fn phys_e8(&mut self, obj: usize) -> Option<actor::alo::PhysE8> {
        self.h.phys_e8(obj)
    }
    fn ee_sub_1f37c8(&mut self, obj: usize) {
        self.h.ee_sub_1f37c8(obj)
    }
    fn ee_sub_149b58(&mut self, k: i32, obj: usize, pmf: u32) {
        self.h.ee_sub_149b58(k, obj, pmf)
    }
    fn alo_children(&mut self, obj: u32) -> Vec<u32> {
        self.h.alo_children(obj)
    }
    fn obj_flags_28(&mut self, obj: u32) -> u64 {
        self.r_obj_flags_28(obj)
    }
    fn ee_sub_1d6310(&mut self, obj: usize) {
        self.h.ee_sub_1d6310(obj)
    }
    fn ee_sub_1d6378(&mut self, obj: usize) {
        self.h.ee_sub_1d6378(obj)
    }
    fn ee_sub_147e88(&mut self, obj: usize) {
        self.h.ee_sub_147e88(obj)
    }
    fn vcall_f(&mut self, obj: u32, off: u32, f: f32) {
        self.h.vcall_f(obj, off, f)
    }
    fn vcall_ret(&mut self, obj: u32, off: u32) -> i32 {
        self.h.vcall_ret(obj, off)
    }
    fn vcall_1b4(&mut self, obj: u32, other: u32) {
        self.h.vcall_1b4(obj, other)
    }
    fn contacts_ec(&mut self, obj: usize) -> actor::alo::Contacts {
        self.h.contacts_ec(obj)
    }
    fn contact_clear(&mut self, obj: usize, k: usize) {
        self.h.contact_clear(obj, k)
    }
    fn ee_sub_1a7e10(&mut self, rec: u32) {
        self.h.ee_sub_1a7e10(rec)
    }
    fn ee_sub_134320(&mut self, obj: usize) {
        self.h.ee_sub_134320(obj)
    }
    fn ee_sub_1a6290(&mut self, obj: u32, oid: i32) -> bool {
        self.h.ee_sub_1a6290(obj, oid)
    }
    fn ee_sub_1f6978(&mut self, obj: usize, id: i32) {
        self.h.ee_sub_1f6978(obj, id)
    }
    fn ee_sub_1f69f8(&mut self, obj: usize, id: i32) {
        self.h.ee_sub_1f69f8(obj, id)
    }
    fn ee_sub_1f6ac8(&mut self, obj: usize, id: i32) {
        self.h.ee_sub_1f6ac8(obj, id)
    }
    fn ee_sub_1f6b48(&mut self, obj: usize, id: i32) {
        self.h.ee_sub_1f6b48(obj, id)
    }
    fn ee_sub_1f5720(&mut self, obj: usize, a: i32) {
        self.h.ee_sub_1f5720(obj, a)
    }
    fn ee_sub_1f57c0(&mut self, obj: usize, a: i32) {
        self.h.ee_sub_1f57c0(obj, a)
    }
    fn ee_sub_20f1b8(&mut self) -> bool {
        self.h.ee_sub_20f1b8()
    }
    fn world_3d84_add(&mut self, d: i32) {
        self.h.world_3d84_add(d)
    }
    fn ee_sub_118668_blockers(&mut self, eye: [f32; 3], tp: [f32; 3]) -> bool {
        self.h.ee_sub_118668_blockers(eye, tp)
    }
    fn ee_sub_1545f0(&mut self, guard: usize, eye: [f32; 3], tp: [f32; 3], f8: u32) -> bool {
        self.h.ee_sub_1545f0(guard, eye, tp, f8)
    }
    fn sensor_event_1c(&mut self, guard: usize, t: u32, old: i32, new: i32) -> i32 {
        self.h.sensor_event_1c(guard, t, old, new)
    }
    fn aim_240(&mut self, t: u32) -> [f32; 3] {
        self.h.aim_240(t)
    }
    fn ee_sub_209618(&mut self, guard: usize, anim: i32, t: f32, rate: f32) -> [f32; 3] {
        self.h.ee_sub_209618(guard, anim, t, rate)
    }
    fn ee_sub_1f7598(&mut self, hit: &guard::slots::GuardHit) {
        self.r_ee_sub_1f7598(hit);
    }
    fn contacts(&mut self, guard: usize) -> Vec<guard::slots::Contact> {
        self.h.contacts(guard)
    }
    fn anim_len(&mut self, guard: usize, anim: i32) -> f32 {
        self.h.anim_len(guard, anim)
    }
    fn ee_sub_209098(&mut self, guard: usize) -> bool {
        self.h.ee_sub_209098(guard)
    }
    fn ee_sub_15ed78(&mut self, k: i32, id: i32, obj: u32) -> u32 {
        self.h.ee_sub_15ed78(k, id, obj)
    }
    fn ee_sub_1fbb90(&mut self, obj: usize, id: i32) {
        self.h.ee_sub_1fbb90(obj, id)
    }
    fn vcall_19c_msg(&mut self, target: u32, a: i32, src: u32) {
        self.h.vcall_19c_msg(target, a, src)
    }
    fn ee_sub_156b98(&mut self, obj: usize, pos: [f32; 3]) {
        self.h.ee_sub_156b98(obj, pos)
    }
    fn ee_sub_1618b0(&mut self) {
        self.h.ee_sub_1618b0()
    }
    fn ee_sub_161868(&mut self) {
        self.h.ee_sub_161868()
    }
    fn ee_sub_164640_src(&mut self, obj: u32, k: i32, src: u32) {
        self.h.ee_sub_164640_src(obj, k, src)
    }
    fn ee_sub_1ce888_fx(&mut self, obj: usize, desc: u32, pos: [f32; 3]) {
        self.h.ee_sub_1ce888_fx(obj, desc, pos)
    }
    fn player_vcall_28c(&mut self, player: u32, hp: i32, mp: i32) {
        self.h.player_vcall_28c(player, hp, mp)
    }
    fn ee_sub_1a58d0(&mut self, obj: usize, a: i32, b: i32) {
        self.h.ee_sub_1a58d0(obj, a, b)
    }
    fn ee_sub_1cb4b8(&mut self, host: u32, obj: u32) {
        self.h.ee_sub_1cb4b8(host, obj)
    }
    fn so_set_v_124(&mut self, obj: u32, v: [f32; 3]) {
        self.h.so_set_v_124(obj, v)
    }
    fn ee_sub_1f0598(&mut self, obj: usize) {
        self.h.ee_sub_1f0598(obj)
    }
    fn ee_sub_1f10f0(&mut self, obj: usize) {
        self.h.ee_sub_1f10f0(obj)
    }
    fn ee_sub_1f0660(&mut self, obj: usize, anim: i32) {
        self.h.ee_sub_1f0660(obj, anim)
    }
    fn ee_sub_1a6318(&mut self, obj: u32) -> [[f32; 4]; 3] {
        self.h.ee_sub_1a6318(obj)
    }
    fn cam_2ddfa0(&mut self) -> [[f32; 4]; 2] {
        self.h.cam_2ddfa0()
    }
    fn ee_sub_1ce888_sparkle(&mut self, at: [f32; 3], target: u32, src: u32, scale: f32) -> bool {
        self.h.ee_sub_1ce888_sparkle(at, target, src, scale)
    }
    fn ee_sub_20faa0_1c5570(&mut self, lo: [f32; 3], hi: [f32; 3]) -> Vec<k8::K8Part> {
        self.h.ee_sub_20faa0_1c5570(lo, hi)
    }
    fn alo_parts_e4(&mut self, alo: u32) -> Vec<k8::K8Part> {
        self.h.alo_parts_e4(alo)
    }
    fn ee_sub_1a93c8(&mut self, radius: f32, pos: [f32; 3], part: u32, max: usize) -> Vec<[[f32; 4]; 2]> {
        self.h.ee_sub_1a93c8(radius, pos, part, max)
    }
    fn ee_sub_1f7b98_f0(&mut self, obj: u32) {
        self.h.ee_sub_1f7b98_f0(obj)
    }
    fn ee_sub_2303a0_off(&mut self, obj: u32, off: u32, k: i32) {
        self.h.ee_sub_2303a0_off(obj, off, k)
    }
    fn ee_sub_1c5fa0_anim(&mut self, obj: usize) -> bool {
        self.h.ee_sub_1c5fa0_anim(obj)
    }
    fn ee_sub_1c5148_anim(&mut self, obj: usize) {
        self.h.ee_sub_1c5148_anim(obj)
    }
    fn rec_15c(&mut self, player: u32) -> k8::PickRec {
        self.h.rec_15c(player)
    }
    fn ee_sub_1eeee0(&mut self, obj: u32, id: i32) -> bool {
        self.h.ee_sub_1eeee0(obj, id)
    }
    fn ee_sub_1fe5f0(&mut self, id: i32, k: i32) -> u32 {
        self.h.ee_sub_1fe5f0(id, k)
    }
    fn ee_sub_1fe738(&mut self) {
        self.h.ee_sub_1fe738()
    }
    fn ee_sub_1dffd8(&mut self, seq: u32) -> bool {
        self.h.ee_sub_1dffd8(seq)
    }
    fn ee_sub_1dfeb8(&mut self, seq: u32) {
        self.h.ee_sub_1dfeb8(seq)
    }
    fn ee_sub_1dfd58(&mut self, seq: u32, player: u32) {
        self.h.ee_sub_1dfd58(seq, player)
    }
    fn ee_sub_1e0020(&mut self, seq: u32, ev: i32) {
        self.h.ee_sub_1e0020(seq, ev)
    }
    fn take_seq_signals(&mut self) -> Vec<(u32, i32)> {
        self.h.take_seq_signals()
    }
    fn ee_sub_1daf38(&mut self, id: i32, on: bool) {
        self.h.ee_sub_1daf38(id, on)
    }
    fn ee_sub_1a3078(&mut self, on: bool) {
        self.h.ee_sub_1a3078(on)
    }
    fn k9_pressed(&mut self, obj: u32, mask: u32) -> u32 {
        self.h.k9_pressed(obj, mask)
    }
    fn k9_stick(&mut self, obj: u32) -> [f32; 2] {
        self.h.k9_stick(obj)
    }
    fn ee_sub_131968(&mut self, a: u32) -> bool {
        self.h.ee_sub_131968(a)
    }
    fn ee_sub_1306a8(&mut self, a: u32) {
        self.h.ee_sub_1306a8(a)
    }
    fn ee_sub_1305b8(&mut self, anim: u32, obj: u32) -> u32 {
        self.h.ee_sub_1305b8(anim, obj)
    }
    fn ee_sub_142140(&mut self, player: u32, pose: u32) {
        self.h.ee_sub_142140(player, pose)
    }
    fn k9_dial_turn(&mut self, obj: u32, dial: u32, player: u32, pose: u32) {
        self.h.k9_dial_turn(obj, dial, player, pose)
    }
    fn vcall_24(&mut self, obj: usize, a: i32, b: u32) {
        self.h.vcall_24(obj, a, b)
    }
    fn ee_sub_20ff70(&mut self, obj: usize, entries: &[[u32; 3]]) {
        self.h.ee_sub_20ff70(obj, entries)
    }
    fn k22_sma_20fff0(&mut self, ctx: u32, entry: &[u32; 4], mode: u32, owner: u32) -> u32 {
        self.h.k22_sma_20fff0(ctx, entry, mode, owner)
    }
    fn k22_release_1e5410(&mut self, owner: u32) {
        self.h.k22_release_1e5410(owner)
    }
    fn k22_sma_msg10(&mut self, sma: u32, label: i32) -> bool {
        self.h.k22_sma_msg10(sma, label)
    }
    fn ee_sub_161f48_list(&mut self, obj: usize, entries: &[[u32; 2]]) {
        self.h.ee_sub_161f48_list(obj, entries)
    }
    fn ee_sub_1b7000(&mut self, user: u32, point: u32) {
        self.h.ee_sub_1b7000(user, point)
    }
    fn take_1b7000(&mut self) -> Vec<(u32, u32)> {
        self.h.take_1b7000()
    }
    fn vol_centre_110(&mut self, obj: usize) -> [f32; 3] {
        self.h.vol_centre_110(obj)
    }
    fn ee_sub_1a6f00_to_world(&mut self, obj: usize, p: [f32; 3]) -> [f32; 3] {
        self.h.ee_sub_1a6f00_to_world(obj, p)
    }
    fn ee_sub_22d348(&mut self, alo: u32, p: [f32; 3]) -> [f32; 3] {
        self.h.ee_sub_22d348(alo, p)
    }
    fn obj_and_40(&mut self, obj: u32, mask: u32) {
        self.h.obj_and_40(obj, mask)
    }
    fn jt_free_2dc_26c(&mut self, p: u32) -> bool {
        self.h.jt_free_2dc_26c(p)
    }
    fn act_vcall(&mut self, vt: u32, off: u32, xf: u32, rec: u32) {
        self.h.act_vcall(vt, off, xf, rec)
    }
    fn ee_sub_130b18(&mut self, handle: u32) {
        self.h.ee_sub_130b18(handle)
    }
    fn ee_sub_12f9b8(&mut self, a: u32, rate: f32, anim: u32, obj: u32, flags: u32) -> u32 {
        self.h.ee_sub_12f9b8(a, rate, anim, obj, flags)
    }
    fn ee_sub_229738(&mut self, obj: u32, water: u32) {
        self.r_ee_sub_229738(obj, water)
    }
    fn vcall_244(&mut self, obj: u32, water: u32) {
        self.h.vcall_244(obj, water)
    }
    fn ee_sub_15c198(&mut self, emitter: u32, kind: i32, pos: [f32; 3]) {
        self.h.ee_sub_15c198(emitter, kind, pos)
    }
    fn so_hit_parts(&mut self, obj: usize) -> Vec<(i32, i16)> {
        self.h.so_hit_parts(obj)
    }
    fn alo_110(&mut self, obj: u32) -> [u8; 26] {
        self.h.alo_110(obj)
    }
    fn ee_sub_1f7598_ok(&mut self, hit: &guard::slots::GuardHit) -> bool {
        self.r_ee_sub_1f7598(hit)
    }
    fn ee_sub_1a5ee8(&mut self, model: u32, m: &[[f32; 4]; 4]) {
        self.h.ee_sub_1a5ee8(model, m)
    }
    fn ee_sub_1f15b8(&mut self, slot: &mut u32) {
        self.h.ee_sub_1f15b8(slot)
    }
    fn ee_sub_1f1448(&mut self, snd: u32, obj: u32, flags: u32) -> u32 {
        self.h.ee_sub_1f1448(snd, obj, flags)
    }
    fn snd_done_1f07c8(&mut self, handle: u32) -> bool {
        self.h.snd_done_1f07c8(handle)
    }
    fn in_world_28(&mut self, obj: u32) -> bool {
        self.r_in_world_28(obj)
    }
    fn ee_sub_210ae0(&mut self) -> i32 {
        self.h.ee_sub_210ae0()
    }
    fn ee_sub_14cc58(&mut self, parts: &[k8::K8Part], a: [f32; 3], b: [f32; 3]) -> Option<crate::engine::physics::bsp::Lsg> {
        self.h.ee_sub_14cc58(parts, a, b)
    }
    fn ee_sub_1c0940(&mut self, region: i32, p: [f32; 3]) -> Option<[f32; 3]> {
        self.h.ee_sub_1c0940(region, p)
    }
    fn ee_sub_1c0980(&mut self, region: i32, a: [f32; 3], b: [f32; 3]) -> bool {
        self.h.ee_sub_1c0980(region, a, b)
    }
    fn ee_sub_154c90(&mut self) -> f32 {
        self.h.ee_sub_154c90()
    }
    fn ee_sub_15c150(&mut self, emitter: u32, obj: u32) {
        self.h.ee_sub_15c150(emitter, obj)
    }
    fn parent_10(&mut self, obj: u32) -> u32 {
        self.h.parent_10(obj)
    }
    fn ee_sub_1a58d0_to(&mut self, obj: usize, other: u32, k: i32) {
        self.h.ee_sub_1a58d0_to(obj, other, k)
    }
    fn ee_sub_12dc18(&mut self, obj: u32, xf: u32, v: [f32; 3]) -> u32 {
        self.h.ee_sub_12dc18(obj, xf, v)
    }
    fn ee_sub_12e460(&mut self, slot: &mut u32) {
        self.h.ee_sub_12e460(slot)
    }
    fn ee_sub_1f7bf8(&mut self, obj: usize, rate: f32) {
        self.h.ee_sub_1f7bf8(obj, rate)
    }
    fn ee_sub_1f0730(&mut self, snd: u32, id: i32) {
        self.h.ee_sub_1f0730(snd, id)
    }
    fn ee_sub_15e9c8_505(&mut self, class: i32, max: usize) -> Vec<u32> {
        self.h.ee_sub_15e9c8_505(class, max)
    }
    fn query_15e9c8(&mut self, class: i32) -> Vec<u32> {
        self.r_query_15e9c8(class)
    }
    fn so_contacts_57(&mut self, obj: usize) -> Vec<k15_101::Contact57> {
        self.h.so_contacts_57(obj)
    }
    fn explode_224bf8(&mut self, obj: u32, kind: u32, pos: [f32; 3], radius: f32, damage: i32) {
        self.r_explode_224bf8(obj, kind, pos, radius, damage)
    }
    fn proj_contacts_13aeb0(&mut self, obj: usize, pos: [f32; 3]) -> Vec<k15_79::ProjContact> {
        self.r_proj_contacts_13aeb0(obj, pos)
    }
    fn proj_sweep_13aeb0(&mut self, obj: usize, p0: [f32; 3], p1: [f32; 3]) -> Option<k15_79::ProjContact> {
        self.r_proj_sweep_13aeb0(obj, p0, p1)
    }
    fn proj_victims_13b5e8(&mut self, obj: u32, pos: [f32; 3], r: f32, r_obj: f32, r_34: f32) -> Vec<u32> {
        self.r_proj_victims_13b5e8(obj, pos, r, r_obj, r_34)
    }
    fn proj_los_13ac18(&mut self, obj: u32, prev: &[k15_79::Los]) -> Vec<k15_79::Los> {
        self.h.proj_los_13ac18(obj, prev)
    }
    fn ee_sub_1a7c18_los(&mut self, obj: usize, other: u32, rec: &k15_79::Los) {
        self.h.ee_sub_1a7c18_los(obj, other, rec)
    }
    fn ee_sub_1c4098(&mut self, obj: u32) -> bool {
        self.h.ee_sub_1c4098(obj)
    }
    fn root_14(&mut self, obj: u32) -> u32 {
        self.h.root_14(obj)
    }
    fn so_root_ec60(&mut self, obj: u32) -> u32 {
        self.h.so_root_ec60(obj)
    }
    fn ee_sub_130598(&mut self, anim: u32, target: u32, rate: f32, mode: u32, w: u32) {
        self.h.ee_sub_130598(anim, target, rate, mode, w)
    }
    fn anim_oid_14(&mut self, anim: u32, owner: u32) -> i32 {
        self.h.anim_oid_14(anim, owner)
    }
    fn sly_18c4(&mut self) -> Option<i32> {
        self.h.sly_18c4()
    }
    fn ee_sub_156698_fx(&mut self, pos: [f32; 3], owner: u32) {
        self.h.ee_sub_156698_fx(pos, owner)
    }
    fn path_len_1e6648(&mut self, path: u32) -> f32 {
        self.h.path_len_1e6648(path)
    }
    fn path_point_1e6f10(&mut self, path: u32, d: f32) -> [f32; 3] {
        self.h.path_point_1e6f10(path, d)
    }
    fn beam_hit_14c6a0(&mut self, other: u32, p0: [f32; 3], p1: [f32; 3]) -> Option<[f32; 3]> {
        self.h.beam_hit_14c6a0(other, p0, p1)
    }
    fn so_overlaps_85(&mut self, obj: usize) -> Vec<u32> {
        self.h.so_overlaps_85(obj)
    }
    fn beam_draw_85(&mut self, obj: usize, segs: &[([f32; 3], [f32; 3])]) {
        self.h.beam_draw_85(obj, segs)
    }
    fn ee_sub_1a64e8(&mut self, obj: usize, k: i32, desc: u32) {
        self.h.ee_sub_1a64e8(obj, k, desc)
    }
    fn ee_sub_135248(&mut self, obj: usize, state: i32) {
        self.h.ee_sub_135248(obj, state)
    }
    fn ee_sub_1f9b20(&mut self, obj: usize) {
        self.h.ee_sub_1f9b20(obj)
    }
    fn ee_sub_204108(&mut self, obj: usize) {
        self.h.ee_sub_204108(obj)
    }
    fn ee_sub_1fac30(&mut self, obj: usize, a: [f32; 3]) {
        self.r_ee_sub_1fac30(obj, a)
    }
    fn k27_po_step(&mut self, obj: usize, m: crate::engine::physics::motor::PoMotor, dt: f32) {
        self.h.k27_po_step(obj, m, dt)
    }
    fn k27_grounded(&mut self, obj: usize) -> bool {
        self.h.k27_grounded(obj)
    }
    fn po_part(&mut self, addr: u32, obj: usize, off: u32) {
        self.h.po_part(addr, obj, off)
    }
    fn vcall_10c(&mut self, obj: usize) -> [f32; 3] {
        self.h.vcall_10c(obj)
    }
    fn ee_sub_1a19a0_po(&mut self, obj: usize, hit: &guard::slots::GuardHit) -> bool {
        self.h.ee_sub_1a19a0_po(obj, hit)
    }
    fn ee_sub_204250(&mut self, obj: usize, rec: &mut crate::engine::player::p5_hull::ContactRec, side: usize) {
        self.h.ee_sub_204250(obj, rec, side)
    }
    fn ee_sub_140ed8(&mut self, obj: usize, kind: i32) -> bool {
        self.h.ee_sub_140ed8(obj, kind)
    }
    fn ee_sub_221268(&mut self, vol: u32, s: i32) {
        self.h.ee_sub_221268(vol, s)
    }
    fn atk_msg18(&mut self, guard: usize, cands: &[(usize, i32)]) -> Option<usize> {
        self.h.atk_msg18(guard, cands)
    }
    fn vcall_1c_query(&mut self, obj: usize, k: i32, msg: (i32, i32)) -> (i32, i32) {
        self.h.vcall_1c_query(obj, k, msg)
    }
    fn g_2de1a0(&mut self) -> u32 {
        self.h.g_2de1a0()
    }
    fn ee_sub_1f03c8(&mut self, sm: u32, obj: u32) -> u32 {
        self.h.ee_sub_1f03c8(sm, obj)
    }
    fn ee_sub_1f01b8(&mut self, sm: u32, obj: u32, a: i32, b: u32) -> u32 {
        self.h.ee_sub_1f01b8(sm, obj, a, b)
    }
    fn ee_sub_1f0380_link(&mut self, sm: u32, obj: u32, a: i32, b: u32) -> u32 {
        self.h.ee_sub_1f0380_link(sm, obj, a, b)
    }
    fn ee_sub_1f0598_link(&mut self, rec: u32) {
        self.h.ee_sub_1f0598_link(rec)
    }
    fn ee_sub_19cd38_po(&mut self, obj: usize, s: i32) {
        self.h.ee_sub_19cd38_po(obj, s)
    }
    fn ee_sub_168638_2(&mut self, obj: usize, a: f32, b: f32, k: i32) {
        self.h.ee_sub_168638_2(obj, a, b, k)
    }
    fn ee_sub_203d00_bones(&mut self, obj: usize) {
        self.h.ee_sub_203d00_bones(obj)
    }
    fn ee_sub_1f58f0(&mut self, obj: usize, rec: &mut k17_55::CamRec) {
        self.h.ee_sub_1f58f0(obj, rec)
    }
    fn po_part_rec(&mut self, addr: u32, obj: usize, off: u32, rec: &mut k17_55::CamRec) {
        self.h.po_part_rec(addr, obj, off, rec)
    }
    fn vcall_30c(&mut self, obj: usize) -> [f32; 3] {
        self.h.vcall_30c(obj)
    }
    fn vcall_114(&mut self, obj: usize) -> [f32; 3] {
        self.h.vcall_114(obj)
    }
    fn is_player_2e1e40_2f7900(&mut self, other: u32) -> bool {
        self.h.is_player_2e1e40_2f7900(other)
    }
    fn world_150_80(&mut self) -> i32 {
        self.h.world_150_80()
    }
    fn ee_sub_1fad00(&mut self, obj: usize, push: [f32; 3], side: u32) {
        self.h.ee_sub_1fad00(obj, push, side)
    }
    fn lo_28_bit60(&mut self, obj: u32) -> bool {
        self.h.lo_28_bit60(obj)
    }
    fn ee_sub_205008(&mut self, obj: usize, other: u32, yaw: f32) {
        self.h.ee_sub_205008(obj, other, yaw)
    }
    fn ee_sub_2041c0(&mut self, obj: usize, v: [f32; 3]) {
        self.h.ee_sub_2041c0(obj, v)
    }
    fn po_part_s(&mut self, addr: u32, obj: usize, off: u32, s: i32) {
        self.h.po_part_s(addr, obj, off, s)
    }
    fn ee_sub_136298(&mut self, obj: usize) {
        self.h.ee_sub_136298(obj)
    }
    fn ee_sub_136530(&mut self, obj: usize, s: i32) {
        self.h.ee_sub_136530(obj, s)
    }
    fn take_136530(&mut self) -> Vec<(usize, i32)> {
        self.h.take_136530()
    }
    fn anim_slot_phase(&mut self, slot: u32) -> f32 {
        self.h.anim_slot_phase(slot)
    }
    fn anim_74(&mut self, anim: u32) -> f32 {
        self.h.anim_74(anim)
    }
    fn ee_sub_1f0400(&mut self, sm: u32, label: i32) -> i32 {
        self.h.ee_sub_1f0400(sm, label)
    }
    fn ee_sub_1f03a8_link(&mut self, sm: u32, obj: u32, label: i32, flags: u32, mask: i32) -> u32 {
        self.h.ee_sub_1f03a8_link(sm, obj, label, flags, mask)
    }
    fn po_call(&mut self, addr: u32, obj: usize, off: u32, args: &[u32]) -> i64 {
        self.h.po_call(addr, obj, off, args)
    }
    fn vcall_ret_arg(&mut self, obj: u32, off: u32, arg: i32) -> i64 {
        self.h.vcall_ret_arg(obj, off, arg)
    }
    fn ee_sub_133448_carry(&mut self, carried: u32, carrier: u32) {
        self.h.ee_sub_133448_carry(carried, carrier)
    }
    fn ee_sub_1333c0_carry(&mut self, carried: u32, carrier: u32) {
        self.h.ee_sub_1333c0_carry(carried, carrier)
    }
    fn ee_sub_224610(&mut self, obj: u32) -> bool {
        self.h.ee_sub_224610(obj)
    }
    fn ee_sub_224590(&mut self, obj: u32) -> bool {
        self.h.ee_sub_224590(obj)
    }
    fn ee_sub_224258_carry(&mut self, obj: u32) {
        self.h.ee_sub_224258_carry(obj)
    }
    fn obj_set_u32(&mut self, obj: u32, off: u32, v: u32) {
        self.h.obj_set_u32(obj, off, v)
    }
    fn ee_sub_204740(&mut self, obj: usize) -> f32 {
        self.h.ee_sub_204740(obj)
    }
    fn ee_sub_204ca8_po(&mut self, obj: usize) {
        self.h.ee_sub_204ca8_po(obj)
    }
    fn ee_sub_2046b0(&mut self, obj: usize, p: [f32; 3]) {
        self.h.ee_sub_2046b0(obj, p)
    }
    fn ee_sub_2046f0(&mut self, obj: usize) {
        self.h.ee_sub_2046f0(obj)
    }
    fn ee_sub_136530_mount(&mut self, obj: usize) -> (u32, u32) {
        self.h.ee_sub_136530_mount(obj)
    }
    fn ee_sub_136530_release(&mut self) {
        self.h.ee_sub_136530_release()
    }
    fn ee_sub_137a30(&mut self, obj: usize, a: i32, b: i32, rec_a: u32, rec_b: u32) {
        self.h.ee_sub_137a30(obj, a, b, rec_a, rec_b)
    }
    fn ee_sub_18f2b0(&mut self, obj: usize, yaw: f32, a: f32, reach: f32, k: i32, v: [f32; 2]) -> Option<[f32; 3]> {
        self.h.ee_sub_18f2b0(obj, yaw, a, reach, k, v)
    }
    fn ee_sub_1a6f00_of(&mut self, a: u32, b: u32) -> [f32; 3] {
        self.h.ee_sub_1a6f00_of(a, b)
    }
    fn ee_sub_1384c0_rec(&mut self, obj: usize, rec: u32) {
        self.h.ee_sub_1384c0_rec(obj, rec)
    }
    fn ee_sub_131b10(&mut self, slot: u32, obj: u32, x: u32) {
        self.h.ee_sub_131b10(slot, obj, x)
    }
    fn ee_sub_131a90(&mut self, slot: u32, t: f32) {
        self.h.ee_sub_131a90(slot, t)
    }
    fn ee_sub_131a00(&mut self, rate: f32, slot: u32) {
        self.h.ee_sub_131a00(rate, slot)
    }
    fn anim_slot_f(&mut self, slot: u32, off: u32) -> f32 {
        self.h.anim_slot_f(slot, off)
    }
    fn root_vel_204e38(&mut self, slot: u32) -> [f32; 3] {
        self.h.root_vel_204e38(slot)
    }
    fn ee_sub_1f09d8(&mut self, sm: u32, label: i32) {
        self.h.ee_sub_1f09d8(sm, label)
    }
    fn ee_sub_1f07e8(&mut self, sma: u32) -> i32 {
        self.h.ee_sub_1f07e8(sma)
    }
    fn ee_sub_1f0638(&mut self, sm: u32) -> bool {
        self.h.ee_sub_1f0638(sm)
    }
    fn ee_sub_1f1350(&mut self, sm: u32, mask: i32) {
        self.h.ee_sub_1f1350(sm, mask)
    }
    fn ee_sub_1357c0(&mut self, obj: usize, k: u32) {
        self.h.ee_sub_1357c0(obj, k)
    }
    fn ground_208(&mut self, ground: u32) -> Option<(u32, u32)> {
        self.h.ground_208(ground)
    }
    fn ee_sub_1cb250(&mut self, host: u32, k: i32) -> u32 {
        self.h.ee_sub_1cb250(host, k)
    }
    fn ee_sub_1cb488(&mut self, host: u32, obj: u32) {
        self.h.ee_sub_1cb488(host, obj)
    }
    fn ee_sub_1cade0(&mut self, host: u32, k: i32) -> bool {
        self.h.ee_sub_1cade0(host, k)
    }
    fn ee_sub_136ed0(&mut self, obj: usize, msg: i32, args: &[u32]) {
        self.h.ee_sub_136ed0(obj, msg, args)
    }
    fn po_jtbs(&self, obj: usize) -> Option<i32> {
        self.h.po_jtbs(obj)
    }
    fn ee_sub_1eeed0_of(&mut self, x: u32) {
        self.h.ee_sub_1eeed0_of(x)
    }
    fn ee_sub_1426e8_rows(&mut self, obj: usize, child: u32) {
        self.h.ee_sub_1426e8_rows(obj, child)
    }
    fn ee_sub_12d020(&mut self, act: u32) {
        self.h.ee_sub_12d020(act)
    }
    fn ee_sub_1a0300(&mut self, obj: usize, a: u32, b: u32) {
        self.h.ee_sub_1a0300(obj, a, b)
    }
    fn ee_sub_20b868(&mut self, obj: usize, blk: u32) {
        self.h.ee_sub_20b868(obj, blk)
    }
    fn world_3d78(&mut self) -> i32 {
        self.h.world_3d78()
    }
    fn g_2dfbc0(&mut self) -> u32 {
        self.h.g_2dfbc0()
    }
    fn ee_sub_14fd50(&mut self) -> bool {
        self.h.ee_sub_14fd50()
    }
    fn ee_sub_2297f8(&mut self, obj: usize, other: u32) {
        self.h.ee_sub_2297f8(obj, other)
    }
    fn ee_sub_1a2788(&mut self, obj: usize, other: u32) -> Option<[f32; 3]> {
        self.h.ee_sub_1a2788(obj, other)
    }
    fn ee_sub_1a28b0_push(&mut self, obj: usize, other: u32, p: [f32; 3]) {
        self.h.ee_sub_1a28b0_push(obj, other, p)
    }
    fn sw_focus_po(&mut self) -> u32 {
        self.h.sw_focus_po()
    }
    fn ee_sub_1c4940(&mut self, obj: u32) {
        self.h.ee_sub_1c4940(obj)
    }
    fn ee_sub_1f82b8(&mut self, obj: usize, v: i32) {
        self.h.ee_sub_1f82b8(obj, v)
    }
    fn anim_attr(&mut self, anim: u32, k: i32) -> Option<u32> {
        self.h.anim_attr(anim, k)
    }
    fn phys_e8_80(&mut self, obj: usize) -> [f32; 3] {
        self.r_phys_e8_80(obj)
    }
    fn vcall_11c_v(&mut self, obj: u32, v: [f32; 3]) {
        self.r_vcall_11c_v(obj, v)
    }
    fn so_carry(&mut self, x: u32, carrier: u32) {
        self.h.so_carry(x, carrier)
    }
    fn so_fly_step(&mut self, dt: f32) -> Vec<(u32, [f32; 3], u32)> {
        self.h.so_fly_step(dt)
    }
    fn so_carried_pos(&mut self, x: u32) -> Option<[f32; 3]> {
        self.h.so_carried_pos(x)
    }
    fn so_land(&mut self, x: u32) {
        self.h.so_land(x)
    }
    fn so_bsphere(&mut self, obj: u32) -> Option<([f32; 3], f32)> {
        self.h.so_bsphere(obj)
    }
    fn so_touch(&mut self, x: u32, p: [f32; 3], other: u32) -> Option<bool> {
        self.h.so_touch(x, p, other)
    }
    fn list_3e13b4(&mut self) -> Vec<u32> {
        self.h.list_3e13b4()
    }
    fn root_track_12ff10(&mut self, anim: u32, obj: usize) -> Option<([f32; 3], [f32; 3])> {
        self.h.root_track_12ff10(anim, obj)
    }
    fn ee_call(&mut self, addr: u32, args: &[u32]) -> i64 {
        self.r_ee_call(addr, args)
    }
    fn ee_sub_1f5860(&mut self, obj: u32) -> [f32; 3] {
        self.h.ee_sub_1f5860(obj)
    }
    fn ee_sub_1eeee0_rec(&mut self, obj: u32, k: i32) -> Option<(u32, i32, u32)> {
        self.r_ee_sub_1eeee0_rec(obj, k)
    }
    fn g_2e0cb4(&mut self) -> u32 {
        self.h.g_2e0cb4()
    }
    fn vcall_124_v(&mut self, obj: u32, v: [f32; 3]) {
        self.h.vcall_124_v(obj, v)
    }
    fn k21_world(&mut self) -> Option<std::sync::Arc<World>> {
        self.h.k21_world()
    }
    fn k21_cam_14f308(&mut self, q: [f32; 3]) -> Option<[f32; 3]> {
        self.h.k21_cam_14f308(q)
    }
    fn k21_cam_pos(&mut self) -> Option<[f32; 3]> {
        self.h.k21_cam_pos()
    }
    fn k21_cam_sphere(&mut self, c: [f32; 3], r: f32) -> Option<bool> {
        self.h.k21_cam_sphere(c, r)
    }
    fn k21_global(&mut self, addr: u32) -> u32 {
        self.h.k21_global(addr)
    }
    fn k21_blocked_14cc58(&mut self, a: [f32; 3], b: [f32; 3]) -> bool {
        self.h.k21_blocked_14cc58(a, b)
    }
    fn k21_ground_173610(&mut self, d0: f32, d1: f32, pos: [f32; 3]) -> Option<[f32; 3]> {
        self.h.k21_ground_173610(d0, d1, pos)
    }
    fn k22_path(&mut self, obj: u32) -> Option<std::sync::Arc<crate::engine::player::p5_target::Path>> {
        self.h.k22_path(obj)
    }
    fn k22_targets_3eb8(&mut self) -> Vec<u32> {
        self.h.k22_targets_3eb8()
    }
    fn k22_speed_10c(&mut self, obj: u32) -> [f32; 3] {
        self.r_k22_speed_10c(obj)
    }
    fn k22_anim_f(&mut self, anim: u32, off: u32) -> f32 {
        self.h.k22_anim_f(anim, off)
    }
    fn k22_vcall_ec_pos(&mut self, obj: u32, p: [f32; 3]) {
        self.h.k22_vcall_ec_pos(obj, p)
    }
    fn k6_live_pos(&mut self, live: &[(u32, [f32; 3])]) {
        self.h.k6_live_pos(live)
    }
    fn k6_live_yaw(&mut self, live: &[(u32, f32)]) {
        self.h.k6_live_yaw(live)
    }
    fn k22_vcall_i(&mut self, obj: u32, off: u32, k: i32) {
        self.h.k22_vcall_i(obj, off, k)
    }
    fn k22_stick(&mut self, obj: u32) -> Option<[f32; 4]> {
        self.h.k22_stick(obj)
    }
    fn k22_setting_3e0fe0(&mut self) -> i32 {
        self.h.k22_setting_3e0fe0()
    }
    fn k22_pin_12dc18(&mut self, obj: u32, v: [f32; 3]) -> u32 {
        self.h.k22_pin_12dc18(obj, v)
    }
    fn k22_pin_12d108(&mut self, handle: u32, v: [f32; 3]) {
        self.h.k22_pin_12d108(handle, v)
    }
    fn k22_radius_ec30(&mut self, obj: u32) -> f32 {
        self.h.k22_radius_ec30(obj)
    }
    fn k22_aspect_2ddf7c(&mut self) -> f32 {
        self.h.k22_aspect_2ddf7c()
    }
    fn k22_launch_1d6b00(&mut self, launcher: u32, obj: u32) {
        self.h.k22_launch_1d6b00(launcher, obj)
    }
    fn k22_music_1fe3f8(&mut self) -> f32 {
        self.h.k22_music_1fe3f8()
    }
    fn k22_title_show_1dd820(&mut self) {
        self.h.k22_title_show_1dd820()
    }
    fn k22_title_active_1d9880(&mut self) -> bool {
        self.h.k22_title_active_1d9880()
    }
    fn k22_seq_action(&mut self, owner: u32, subj: u32, rec: &[u32; 20]) {
        self.h.k22_seq_action(owner, subj, rec)
    }
    fn k22_jt_live(&mut self) -> bool {
        self.h.k22_jt_live()
    }
    fn k22_npc_walk_1e46a0(&mut self, subj: u32, goal: [f32; 3], speed: f32) {
        self.h.k22_npc_walk_1e46a0(subj, goal, speed)
    }
    fn k22_walkto_1448f0(&mut self, subj: u32, pos: [f32; 3], speed: f32) {
        self.h.k22_walkto_1448f0(subj, pos, speed)
    }
    fn k22_walk_r1_1e46a0(&mut self, subj: u32) {
        self.h.k22_walk_r1_1e46a0(subj)
    }
    fn k22_autowalk_1938c0(&mut self, subj: u32, mask: u32, target: u32) {
        self.h.k22_autowalk_1938c0(subj, mask, target)
    }
    fn k22_teleport_1e1e30_16(&mut self, subj: u32, loc: u32, rec5: u32) {
        self.h.k22_teleport_1e1e30_16(subj, loc, rec5)
    }
    fn k22_hop_1e1e30_2a(&mut self, subj: u32, target: u32, rec5: u32) {
        self.h.k22_hop_1e1e30_2a(subj, target, rec5)
    }
    fn k22_hop_done_2a(&mut self, subj: u32) -> Option<bool> {
        self.h.k22_hop_done_2a(subj)
    }
    fn k22_on_path_1a(&mut self, subj: u32) -> Option<bool> {
        self.h.k22_on_path_1a(subj)
    }
    fn k22_climb_1a(&mut self, subj: u32, pos: [f32; 3]) -> Option<f32> {
        self.h.k22_climb_1a(subj, pos)
    }
    fn k22_pose_143ee0(&mut self, subj: u32, oid: i32, flags: u32, idx: u32) {
        self.h.k22_pose_143ee0(subj, oid, flags, idx)
    }
    fn k22_posing_144038(&mut self, subj: u32) -> bool {
        self.h.k22_posing_144038(subj)
    }
    fn k22_po_switch(&mut self, po: u32) {
        self.h.k22_po_switch(po)
    }
    fn live_vcall(&mut self, off: u32, obj: u32, arg: u32) {
        self.h.live_vcall(off, obj, arg)
    }
    fn k22_take_po_switches(&mut self) -> Vec<u32> {
        self.h.k22_take_po_switches()
    }
    fn k23_comp_b0(&mut self, guard: u32) -> Option<i32> {
        self.r_k23_comp_b0(guard)
    }
    fn k23_comp_b0_set(&mut self, guard: u32, s: i32) {
        self.r_k23_comp_b0_set(guard, s)
    }
    fn k23_guard_209840(&mut self, guard: u32, rows: [[f32; 4]; 3]) {
        self.r_k23_guard_209840(guard, rows)
    }
    fn k23_guard_fc4(&mut self, guard: u32) -> u32 {
        self.h.k23_guard_fc4(guard)
    }
    fn k23_link_229710(&mut self, obj: u32, owner: u32) {
        self.r_k23_link_229710(obj, owner)
    }
    fn k23_guard_206eb0(&mut self, obj: u32, k: i32) -> bool {
        self.r_k23_guard_206eb0(obj, k)
    }
    fn k23_brain_209e28(&mut self, guard: u32, a: i32) {
        self.r_k23_brain_209e28(guard, a)
    }
    fn k25_tick(&mut self, dt: f32) {
        self.h.k25_tick(dt)
    }
    fn k25_bodies(&mut self, b: Vec<(usize, super::k25::BodySnap)>) {
        self.h.k25_bodies(b)
    }
    fn k26_take_msgs(&mut self) -> Vec<(u32, i32, crate::engine::anim::sm::SmaMsg)> {
        self.h.k26_take_msgs()
    }
    fn k26_take_msg9(&mut self) -> Vec<(u32, [u32; 3])> {
        self.h.k26_take_msg9()
    }
    fn k26_handled(&mut self, n: usize) {
        self.h.k26_handled(n)
    }
    fn k23_world_1c6fd8(&mut self, kind: i32, pos: [f32; 3], src: u32, k: i32) {
        self.h.k23_world_1c6fd8(kind, pos, src, k)
    }
    fn k8_take_drops(&mut self) -> Vec<(i32, [f32; 3], u32, i32)> {
        self.h.k8_take_drops()
    }
    fn k23_clock_14d1d0(&mut self, v: f32, index: usize) {
        self.h.k23_clock_14d1d0(v, index)
    }
    fn k23_stream_1fe848(&mut self, id: i32) -> bool {
        self.h.k23_stream_1fe848(id)
    }
    // K30 (added by hand: a full regeneration also forwards the K27-K29 hooks, a behaviour change left out)
    fn k30_cam_state(&mut self, key: u32, st: k30_cam::CamState) {
        self.h.k30_cam_state(key, st)
    }
    fn k30_cam_rows(&mut self) -> Option<[[f32; 4]; 3]> {
        self.h.k30_cam_rows()
    }
    fn k30_zoom(&mut self, op: k30_cam::ZoomOp) {
        self.h.k30_zoom(op)
    }
    fn k30_zoom_wait(&mut self, kind: u32) -> bool {
        self.h.k30_zoom_wait(kind)
    }
}
