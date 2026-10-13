//! WP-K9: the vault (class 78, classes/k9.rs) on the real levels: build + 300 frames on all 43 levels, the
//! bottle gate (FUN_001c8af0: DAT_003e1bf4 < DAT_003e1bf8 refuses), dialing the combination and the "fVault"
//! save variable.
use sly2_recomp::engine::classes::k8;
use sly2_recomp::engine::classes::k9::{self, Vault, DIAL_2C0FF0};
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::{ClassHost, ClassSet, Inst};
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::Arc;

fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

fn vaults(s: &ClassSet) -> Vec<(usize, Vault)> {
    s.k9.iter().filter_map(|(i, x)| if let Inst::K9(c) = x { Some((*i, (**c).clone())) } else { None }).collect()
}

/// Every vault builds on all 43 levels (closed, bindings resolved) and runs 300 frames untouched.
#[test]
#[ignore = "heavy"]
fn k9_vault_all_levels_300_frames() {
    let (loader, descs) = levels();
    assert_eq!(descs.len(), 43);
    let (mut n, mut lv_with) = (0, 0);
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        s.g.dt = 1.0 / 60.0;
        assert_eq!(s.g.g_3e1bf8, 30);
        let c78 = w.objects.iter().filter(|o| o.class == 78).count();
        assert_eq!(s.k9.len(), c78);
        if c78 > 0 {
            lv_with += 1;
        }
        for (_, v) in vaults(&s) {
            assert_eq!(v.f_180, 0, "vault starts closed");
            assert!((1..=999).contains(&v.f_1f0));
            n += 1;
        }
        for _ in 0..300 {
            s.g.t += s.g.dt;
            s.frame(&mut h);
        }
        for (_, v) in vaults(&s) {
            assert_eq!(v.f_180, 0);
        }
    }
    println!("K9 vaults: {n} on {lv_with} levels");
    assert!(n > 0);
}

/// Test host: oid lookup on the world, a pad (stick + one-shot presses) and sequences that run until stopped.
#[derive(Default)]
struct VaultHost {
    w: Option<Arc<World>>,
    stick: [f32; 2],
    press: u32,
    running: Vec<u32>,
    hud: u32,
    events: Vec<i32>,
    lock: bool,
}
impl ClassHost for VaultHost {
    fn ee_sub_15e938(&mut self, _k: i32, oid: i32, _ctx: u32) -> u32 {
        self.w.as_ref().and_then(|w| w.find_oid(oid).first().map(|&i| i as u32 + 1)).unwrap_or(0)
    }
    fn ee_sub_1618b0(&mut self) {
        self.hud += 1;
    }
    fn ee_sub_1dffd8(&mut self, seq: u32) -> bool {
        self.running.contains(&seq)
    }
    fn ee_sub_1dfd58(&mut self, seq: u32, _p: u32) {
        self.running.push(seq);
    }
    fn ee_sub_1dfeb8(&mut self, seq: u32) {
        self.running.retain(|&x| x != seq);
    }
    fn ee_sub_1e0020(&mut self, _seq: u32, ev: i32) {
        self.events.push(ev);
    }
    fn ee_sub_1a3078(&mut self, on: bool) {
        self.lock = on;
    }
    fn k9_pressed(&mut self, _obj: u32, mask: u32) -> u32 {
        let r = self.press & mask;
        self.press &= !mask;
        r
    }
    fn k9_stick(&mut self, _obj: u32) -> [f32; 2] {
        self.stick
    }
}

fn run(s: &mut ClassSet, h: &mut VaultHost, frames: usize) {
    for _ in 0..frames {
        s.g.t += s.g.dt;
        s.frame(h);
    }
}

/// Dial `digits` (1..=9) with the stick + press, 0.25 s turns.
fn dial(s: &mut ClassSet, h: &mut VaultHost, digits: &[i32]) {
    for &d in digits {
        h.stick = DIAL_2C0FF0[(d - 1) as usize];
        run(s, h, 20);
        h.press = k9::MASK_PRESS;
        run(s, h, 2);
    }
}

/// The vault refuses below the level's bottle count (30) and opens after it is reached + the combination.
#[test]
fn k9_vault_opens_after_bottle_count() {
    let (loader, descs) = levels();
    let mut tested = 0;
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let Some(obj) = w.objects.iter().position(|o| o.class == 78) else { continue };
        let mut h = VaultHost { w: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        s.g.dt = 1.0 / 60.0;
        let v = s.k9_vault(obj).unwrap().clone();
        assert!(v.f_20c[2] != 0, "dialing sequence bound (oid 0x651)");
        let combo: Vec<i32> = v.f_1f0.to_string().bytes().map(|b| (b - b'0') as i32).collect();
        assert!(combo.iter().all(|&c| (1..=9).contains(&c)), "combination {} dialable", v.f_1f0);
        let player = 0x7fff_0000;

        // 29 / 30 bottles: refused, HUD flash, stays closed
        s.g.g_3e1bf4 = 29;
        assert!(s.k9_try_open(obj, player, &mut h));
        run(&mut s, &mut h, 10);
        assert_eq!(s.k9_vault(obj).unwrap().f_180, 0);
        assert_eq!(h.hud, 1);

        // all bottles: approach sequence -> dialing (sequence of state 2 runs while the player dials)
        s.g.g_3e1bf4 = 30;
        s.k9_try_open(obj, player, &mut h);
        assert_eq!(s.k9_vault(obj).unwrap().f_180, 1);
        assert_eq!(s.g.w_3d84, 1);
        h.running.clear(); // approach sequence ends
        run(&mut s, &mut h, 1);
        assert_eq!(s.k9_vault(obj).unwrap().f_180, 2);
        assert!(h.lock);

        // wrong code first: last three digits count
        dial(&mut s, &mut h, &[9, 9]);
        assert_eq!(s.k9_vault(obj).unwrap().f_198, 99);
        dial(&mut s, &mut h, &combo);
        let v = s.k9_vault(obj).unwrap().clone();
        assert_eq!(v.f_198, v.f_1f0);
        assert_eq!(v.f_188, 3, "dial done");
        assert_eq!(h.events, vec![k9::EV_655]);
        h.running.clear(); // dialing sequence ends -> code check -> opening
        run(&mut s, &mut h, 1);
        assert_eq!(s.k9_vault(obj).unwrap().f_180, 3);
        assert!(!h.lock);
        h.running.clear(); // opening sequence ends
        run(&mut s, &mut h, 1);
        let v = s.k9_vault(obj).unwrap().clone();
        assert_eq!(v.f_180, 5, "vault open");
        assert_eq!(v.f_190, 0, "player released");
        assert_eq!(s.g.f_vault, Some(true));
        assert_eq!(s.g.w_3d84, 0);

        // reload with fVault set: post-load starts open
        let mut s2 = ClassSet::build(&w, &mut h);
        s2.g.f_vault = s.g.f_vault;
        s2.post_load(&mut h);
        assert_eq!(s2.k9_vault(obj).unwrap().f_180, 5);

        // wrong combination -> state 4 -> closed again
        let mut s3 = ClassSet::build(&w, &mut h);
        s3.post_load(&mut h);
        s3.g.dt = 1.0 / 60.0;
        s3.g.g_3e1bf4 = 30;
        h.running.clear();
        s3.k9_try_open(obj, player, &mut h);
        h.running.clear();
        run(&mut s3, &mut h, 1);
        let wrong = if combo[0] == 9 { 1 } else { 9 };
        dial(&mut s3, &mut h, &[wrong, wrong, wrong]);
        h.running.clear();
        run(&mut s3, &mut h, 1);
        assert_eq!(s3.k9_vault(obj).unwrap().f_180, 4);
        h.running.clear();
        run(&mut s3, &mut h, 1);
        assert_eq!(s3.k9_vault(obj).unwrap().f_180, 0);
        assert_eq!(s3.g.f_vault, Some(false));
        tested += 1;
    }
    println!("K9 vault levels tested: {tested}");
    assert!(tested > 0);
}

/// K8 hooks finished in K9: FUN_001c55e0 ground bounce against a host contact plane (state 4: friction +
/// restitution, first slow bounce jumps +0x1d4), wall reflection, FUN_001c5b20 sphere-part reach,
/// FUN_001c4ef8 binding of oid 0x182 -> +0x164, FUN_001c5fa0 draw routing.
#[derive(Default)]
struct BounceHost {
    floor: Option<f32>,
    parts: Vec<k8::K8Part>,
    draws: usize,
    anim_draws: usize,
    bound: Vec<i32>,
}
impl ClassHost for BounceHost {
    fn ee_sub_20faa0_1c5570(&mut self, _lo: [f32; 3], _hi: [f32; 3]) -> Vec<k8::K8Part> {
        vec![k8::K8Part { id: 1, f_74: 1, ..Default::default() }]
    }
    fn ee_sub_1a93c8(&mut self, r: f32, pos: [f32; 3], part: u32, _max: usize) -> Vec<[[f32; 4]; 2]> {
        let Some(z) = self.floor else { return Vec::new() };
        if part == 1 && pos[2] - r < z {
            vec![[[pos[0], pos[1], pos[2] - r, 0.0], [0.0, 0.0, 1.0, z]]]
        } else {
            Vec::new()
        }
    }
    fn alo_parts_e4(&mut self, _alo: u32) -> Vec<k8::K8Part> {
        self.parts.clone()
    }
    fn ee_sub_15e938(&mut self, _k: i32, oid: i32, _ctx: u32) -> u32 {
        self.bound.push(oid);
        77
    }
    fn so_draw_1f3c70(&mut self, _obj: usize, _alpha: f32) {
        self.draws += 1;
    }
    fn ee_sub_1c5fa0_anim(&mut self, _obj: usize) -> bool {
        self.anim_draws += 1;
        true
    }
}

#[test]
fn k8_hooks_bounce_reach_load_draw() {
    use sly2_recomp::engine::classes::k8::{Pickup, PickRec};
    let mut h = BounceHost { floor: Some(0.0), ..Default::default() };
    let mut p = Pickup { class: 92, ..Default::default() };
    p.init(92, 0, &mut h);
    // load: oid 0x182 bound to the anim owner
    p.slot13(0, &mut h);
    assert!(h.bound.contains(&0x182));
    assert_eq!(p.f_164, 77);

    // ground bounce in state 4: 10 below the floor, falling at 1000 and sliding at 400
    p.f_150 = 4;
    p.xf_pos = [0.0, 0.0, p.f_1d8 - 10.0, 1.0];
    p.f_170 = p.xf_pos;
    p.f_180 = [400.0, 0.0, -1000.0, 0.0];
    p.ee_sub_1c55e0(0, &mut h);
    assert!((p.f_170[2] - p.f_1d8).abs() < 1e-3, "pushed out of the floor: {:?}", p.f_170);
    // horizontal: min(0.6 * 400, 300) = 240; vertical: min(0.6 * 1000, 600) = 600 (>= +0x1d0 500: no jump)
    assert!((p.f_180[0] - 240.0).abs() < 1e-3 && (p.f_180[2] - 600.0).abs() < 1e-3, "{:?}", p.f_180);
    assert_eq!(p.f_1c4, 0);
    // slow bounce: 0.6 * 300 = 180 < 500 -> jumps +0x1d4 = 250 once
    p.xf_pos[2] = p.f_1d8 - 1.0;
    p.f_180 = [0.0, 0.0, -300.0, 0.0];
    p.ee_sub_1c55e0(0, &mut h);
    assert!((p.f_180[2] - 250.0).abs() < 1e-3 && p.f_1c4 == 1, "{:?}", p.f_180);
    // moving away from the floor: no response
    p.xf_pos[2] = p.f_1d8 - 1.0;
    p.f_180 = [0.0, 0.0, 50.0, 0.0];
    p.ee_sub_1c55e0(0, &mut h);
    assert_eq!(p.f_180, [0.0, 0.0, 50.0, 0.0]);
    // state 0 (thrown): v = -e v_n + f v_t
    p.f_150 = 0;
    p.f_180 = [100.0, 0.0, -200.0, 0.0];
    p.ee_sub_1c55e0(0, &mut h);
    assert!((p.f_180[0] - 60.0).abs() < 1e-3 && (p.f_180[2] - 120.0).abs() < 1e-3, "{:?}", p.f_180);

    // reach: sphere part (bit 49) within +0x1dc + radius; skipped part (bit 50) never
    let q = [0.0, 0.0, 0.0, 1.0];
    p.xf_pos = q;
    p.f_1c8 = 1;
    p.f_1cc = f32::MAX;
    p.f_150 = 0;
    h.parts = vec![k8::K8Part { flags: 1 << 50 | 1 << 49, center: [10.0, 0.0, 0.0, 1.0], radius: 100.0, ..Default::default() }];
    let rec = PickRec { alive: 1, radius: 0.0, obj: 5 };
    let mut g = sly2_recomp::engine::classes::ClassGlobals::default();
    let mut rng = sly2_recomp::engine::physics::driven::GameRng::default();
    let mut cx = k8::Cx { g: &mut g, rng: &mut rng };
    assert!(!p.ee_sub_1c5b20(0, &rec, &mut cx, &mut h));
    h.parts[0].flags = 1 << 49;
    h.parts[0].center = [p.f_1dc + 99.0, 0.0, 0.0, 1.0];
    assert!(p.ee_sub_1c5b20(0, &rec, &mut cx, &mut h));
    assert!(p.f_150 == 1 || p.f_150 == 2, "collected: {}", p.f_150);
    p.f_150 = 0;

    // draw: anim path with a playing anim, else SO draw; nothing in state 2
    p.f_168 = None;
    p.ee_sub_1c5fa0(0, &mut h);
    p.f_168 = Some(0x184);
    p.ee_sub_1c5fa0(0, &mut h);
    p.f_150 = 2;
    p.ee_sub_1c5fa0(0, &mut h);
    assert_eq!((h.draws, h.anim_draws), (1, 1));
}

/// FUN_001c55e0 on the real level geometry (EngineHost: SO collision parts + FUN_001a93c8): an idle pickup
/// thrown into the ground at its placement bounces back on the levels where it rests on geometry.
#[test]
fn k8_bounce_on_level_geometry() {
    let (loader, descs) = levels();
    let (mut tried, mut bounced) = (0, 0);
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let mut s = ClassSet::build(&w, &mut h);
        s.post_load(&mut h);
        s.g.dt = 1.0 / 60.0;
        let objs: Vec<usize> = s.k8.iter().filter(|(_, x)| matches!(x, Inst::K8(c) if c.in_world && c.f_150 == 0)).map(|x| x.0).take(4).collect();
        for o in objs {
            let p = s.k8_pickup(o).unwrap();
            p.f_1d8 = 35.0;
            p.f_1e0 = 0.5;
            p.f_1e4 = 0.5;
            p.f_180 = [0.0, 0.0, -3000.0, 0.0];
            tried += 1;
            for _ in 0..30 {
                s.g.t += s.g.dt;
                s.frame(&mut h);
                if s.k8_pickup(o).unwrap().f_180[2] > 0.0 {
                    bounced += 1;
                    break;
                }
            }
        }
    }
    println!("K8 bounce: {bounced} / {tried} bottles bounced off level geometry");
    assert!(tried > 0 && bounced > 0);
}
