//! WP-X3 tests: gaps sweep 3 (binocucom recon targets 0x201d58..0x203790, ...).
use sly2_recomp::engine::world::{LoadOptions, World};

#[path = "engine_k20.rs"]
#[allow(dead_code)]
mod k20;

/// census of class-46 (C2d9f70) objects and their property fields
#[test]
#[ignore]
fn x3_probe_c46() {
    let (loader, descs) = k20::levels();
    for d in descs.into_iter() {
        let Ok(lv) = loader.load_blocking(d) else { continue };
        let Ok(w) = World::load(&lv, LoadOptions::default()) else { continue };
        for (i, o) in w.objects.iter().enumerate().filter(|o| o.1.class == 46) {
            let props = sly2_recomp::engine::classes::props_of(&w, o.clone_of.unwrap_or(i)).unwrap_or(&[]);
            let v: Vec<String> = props.iter().map(|p| format!("{}={:?}", p.field, p.value)).collect();
            println!("lvl {} obj {} oid {:#x} props {}", d.lsn(), i, o.oid, v.join(" "));
        }
    }
}

mod recon {
    use sly2_recomp::engine::camera::cm::Cm;
    use sly2_recomp::engine::ui::binoc_recon::{BinocRecon, ReconHost, ReconTarget, V3};

    #[derive(Default)]
    pub struct H {
        pub log: Vec<String>,
        pub t: f32,
        pub scale: f32,
        pub flags: u32,
        pub playing: bool,
        pub press: bool,
        pub blocked: usize,
        pub done: Vec<usize>,
    }
    impl ReconHost for H {
        fn clock(&self) -> f32 { self.t }
        fn clock_scale(&mut self, s: f32) { self.scale = s; }
        fn elem_hide(&mut self, s: u32, e: i32) { self.log.push(format!("hide {s} {e:#x}")); }
        fn elem_show(&mut self, s: u32, e: i32) { self.log.push(format!("show {s} {e:#x}")); }
        fn elem_set(&mut self, s: u32, e: i32, a: i32) { self.log.push(format!("set {s} {e:#x} {a:#x}")); }
        fn elem_scrub(&mut self, _: u32, _: i32, _: f32) {}
        fn elem_playing(&mut self, _: u32, _: i32) -> bool { self.playing }
        fn screen_214300(&mut self, _: u32, _: i32) {}
        fn screen_msg(&mut self, s: u32, m: i32) { self.log.push(format!("msg {s} {m:#x}")); }
        fn screen_close(&mut self, s: u32) { self.log.push(format!("close {s}")); }
        fn screen_state(&self, _: u32) -> i32 { 2 }
        fn screen_off(&mut self, s: u32) { self.log.push(format!("off {s}")); }
        fn screen_set_target(&mut self, _: u32, _: Option<usize>) {}
        fn screen_word(&self, _: u32, off: u32) -> i32 { off as i32 }
        fn game_msg(&mut self, m: i32) { self.log.push(format!("game {m}")); }
        fn world_pause(&mut self, on: bool) { self.log.push(format!("pause {on}")); }
        fn music_1fe5b8(&mut self, _: i32) {}
        fn cam_preset_off(&mut self) {}
        fn getter_call(&mut self, _: i32) {}
        fn obj_msg(&mut self, _: u32, _: i32, _: i32) {}
        fn sound(&mut self, id: i32) { self.log.push(format!("sound {id:#x}")); }
        fn bn_flags(&self) -> u32 { self.flags }
        fn set_bn_flags(&mut self, v: u32) { self.flags = v; }
        fn bn_eye(&self) -> V3 { [0.0, -2000.0, 480.0] }
        fn bn_mode(&self) -> i32 { 0 }
        fn bn_19cce8(&mut self) { self.log.push("19cce8".into()); }
        fn revoke(&mut self) -> bool { false }
        fn joy_take_pressed(&mut self, _: u32) -> bool { std::mem::take(&mut self.press) }
        fn ray_blocked(&mut self, _: usize, _: V3, _: V3) -> bool {
            if self.blocked > 0 { self.blocked -= 1; true } else { false }
        }
        fn sensor_contains(&mut self, _: V3) -> bool { false }
        fn link_4c(&mut self, _: usize) {}
        fn link_44(&mut self, _: usize) {}
        fn link_1f7bf8(&mut self, _: usize, _: f32) {}
        fn link_1f7c68(&mut self, _: usize, _: f32) {}
        fn job_done(&mut self, t: usize) -> bool { self.done.contains(&t) }
        fn job_complete(&mut self, t: usize) { self.done.push(t); }
    }

    pub fn setup() -> (BinocRecon, Cm, H) {
        let mut cm = Cm::default();
        cm.r_mrd = 1.0;
        let mut r = BinocRecon { scr_a: 1, scr_b: 2, ..Default::default() };
        let t = ReconTarget { pos: [0.0, 0.0, 0.0], active: true, mrd: 1.0e6, r_e0: 50.0, r_e4: 50.0, tol_e8: 1.0, job: Some(1), id_f8: 3, snd_fc: -1, f_10c: 0, ..Default::default() };
        r.targets = vec![t.clone(), ReconTarget { pos: [5000.0, 0.0, 0.0], ..t }];
        (r, cm, H::default())
    }
}

/// FUN_00202768 / FUN_00202d38: open -> idle -> search -> framed -> photo -> done (job completed)
#[test]
fn x3_recon_photo_flow() {
    let (mut r, cm, mut h) = recon::setup();
    r.FUN_00202768(&mut h, &cm, 0);
    assert_eq!(r.state, 0);
    assert!(h.log.contains(&"game 9".to_string()));
    h.playing = true; // opening animation 0x314
    r.FUN_00202d38(&mut h, &cm);
    assert_eq!(r.state, 1);
    h.playing = false;
    r.FUN_00202d38(&mut h, &cm);
    assert_eq!(r.state, 2, "targets left -> searching");
    // FUN_00202540 picks the centred target 0 (target 1 is off to the side but farther)
    r.FUN_00202d38(&mut h, &cm);
    assert_eq!(r.cur, Some(0));
    assert_eq!(r.targets[0].vis_108, 1);
    assert!(r.centered >= 1.0, "target at the view centre: {}", r.centered);
    for _ in 0..4 {
        r.FUN_00202d38(&mut h, &cm);
    }
    assert_eq!(r.state, 7, "framed (log {:?})", h.log);
    assert!(h.log.contains(&"sound 0x52".to_string()));
    h.press = true;
    h.flags = 0x41;
    r.FUN_00202d38(&mut h, &cm);
    assert_eq!(r.state, 8);
    assert_eq!((h.scale, h.flags, r.saved_flags), (0.0, 0x143, 0x40));
    assert!(h.log.contains(&"pause true".to_string()));
    r.FUN_00202d38(&mut h, &cm);
    assert_eq!(r.state, 9, "framed photo, +0x10c == 0");
    r.FUN_00202d38(&mut h, &cm);
    assert_eq!(r.state, 2);
    assert_eq!(h.done, vec![0]);
    assert_eq!((h.scale, h.flags), (1.0, 0x41));
    assert!(h.log.contains(&"game 10".to_string()));
    // FUN_00202768(10) / (-1): close
    r.FUN_00202768(&mut h, &cm, 10);
    assert!(h.log.ends_with(&["close 2".into(), "close 1".into(), "game 11".into()]));
    r.FUN_00202d38(&mut h, &cm);
    assert_eq!(r.state, -1);
}

/// FUN_00202378: 4 blocked rays -> hidden (2, not picked), 2 -> partial (3), 0 -> clear (1)
#[test]
fn x3_recon_occlusion() {
    let (mut r, cm, mut h) = recon::setup();
    r.state = 2;
    r.eye = [0.0, -2000.0, 500.0];
    for (n, v) in [(4, 2), (2, 3), (0, 1)] {
        h.blocked = n;
        r.FUN_00202378(&mut h, &cm, 0);
        assert_eq!(r.targets[0].vis_108, v);
    }
    // all rays blocked for both targets -> nothing picked
    h.blocked = 8;
    r.FUN_00202540(&mut h, &cm);
    assert_eq!(r.cur, None);
    // FUN_00202030 / FUN_002021f8: list ids 0x2f5.. with 0x33b (open) / 0x33a (done)
    r.FUN_00202030();
    h.done = vec![1];
    r.FUN_002021f8(&mut h);
    assert_eq!(h.log, vec!["set 1 0x33b 0x2f5", "set 1 0x33a 0x2f6"]);
}

// ---------------------------------------------------------------- G2 ThiefNet shop (ui::thiefnet)
use sly2_recomp::engine::ui::thiefnet::{Shop, ShopHost, Slot, PAD_SQUARE, PAD_TRIANGLE, PAD_X};

#[derive(Default)]
struct ShopMock {
    press: u32,
    stick: (f32, f32, f32),
    coins: i32,
    owned: Vec<i32>,
    ep: i32,
    loot: Vec<i32>,
    log: Vec<String>,
    paused: bool,
    scale: f32,
}

impl ShopHost for ShopMock {
    fn base_update(&mut self) {}
    fn pressed(&mut self, m: u32) -> bool {
        self.press & m != 0
    }
    fn stick(&self) -> (f32, f32, f32) {
        self.stick
    }
    fn coins(&self) -> i32 {
        self.coins
    }
    fn set_coins(&mut self, v: i32) {
        self.coins = v;
    }
    fn acquired(&mut self, g: i32) -> bool {
        self.owned.contains(&g)
    }
    fn acquire(&mut self, g: i32, _on: bool) {
        self.owned.push(g);
    }
    fn episode(&self) -> i32 {
        self.ep
    }
    fn loot(&mut self, i: i32) -> i32 {
        self.loot[i as usize]
    }
    fn set_loot(&mut self, i: i32, v: i32) {
        self.loot[i as usize] = v;
    }
    fn world_19f0a8(&mut self) {}
    fn sound(&mut self, id: i32) {
        self.log.push(format!("snd {id:#x}"));
    }
    fn time(&self) -> f32 {
        1.0
    }
    fn clock_scale(&mut self, s: f32) {
        self.scale = s;
    }
    fn world_pause(&mut self, on: bool) {
        self.paused = on;
    }
    fn vt_open(&mut self) {}
    fn vt_close(&mut self) {}
    fn scr_state(&self) -> i32 {
        2
    }
    fn scr_close(&mut self) {
        self.log.push("close".into());
    }
    fn scr_ids(&mut self, a: i32, _: i32, _: i32) {
        self.log.push(format!("ids {a:#x}"));
    }
    fn scr_212a70(&mut self, _: i32) {}
    fn scr_212a50(&mut self, _: i32) {}
    fn scr_213a08(&mut self, _: i32, _: i32) {}
    fn scr_213bd8(&mut self, e: i32, row: i32) {
        self.log.push(format!("row {e:#x} {row:#x}"));
    }
    fn scr_place(&mut self, e: i32, s: Slot) {
        self.log.push(format!("place {e} {s:?}"));
    }
    fn scr_xform(&mut self, e: i32, ty: f32) {
        self.log.push(format!("xf {e:#x} {ty}"));
    }
    fn scr_show(&mut self, _: i32) {}
    fn scr_hide(&mut self, _: i32) {}
}

/// FUN_001dc260 open, FUN_001dbd10 list (episode 2 gadgets minus owned),
/// FUN_001db870 X -> confirm dialog -> X buys; stick down scrolls; square -> loot page; X sells; triangle closes.
#[test]
fn x3_thiefnet_shop() {
    let mut h = ShopMock { coins: 1000, ep: 2, owned: vec![7], scale: 1.0, loot: vec![0; 0x40], ..Default::default() };
    h.loot[3] = 2;
    let mut s = Shop { loot_price: vec![11, 19, 27, 17], ..Default::default() };
    s.FUN_001dc260(&mut h, 0);
    assert_eq!((s.page, h.paused, h.scale), (0, true, 0.0));
    // episode-1 gadgets not owned: shop episode +0x10 == 1 or less, ids 7..0x2a (7 owned)
    let want: Vec<i32> = (8..0x2b).filter(|&g| {
        let o = sly2_recomp::engine::player::p18_gadget::GADGETS[g as usize].order;
        o != -1 && o <= 2
    }).collect();
    assert_eq!(s.count as usize, want.len());
    assert_eq!(s.ent[0][0], want[0]);
    assert_eq!((s.sel, s.gadget), (0, want[0]));
    let price = sly2_recomp::engine::player::p18_gadget::GADGETS[want[0] as usize].price;
    h.press = PAD_X;
    s.FUN_001db870(&mut h);
    assert_eq!((s.dialog, s.dialog_msg), (0, 0x34), "confirm dialog");
    s.FUN_001db870(&mut h);
    assert_eq!(h.coins, 1000 - price);
    assert!(h.owned.contains(&want[0]));
    assert_eq!(s.count as usize, want.len() - 1);
    assert_eq!(s.dialog, -1);
    // stick down (y < 0 -> direction 6) selects the next row
    h.press = 0;
    h.stick = (0.0, -1.0, 1.0);
    s.FUN_001db870(&mut h);
    assert_eq!((s.dir, s.sel), (6, 1));
    s.FUN_001db870(&mut h);
    assert_eq!(s.sel, 1, "held direction does not repeat");
    h.stick = (0.0, 0.0, 0.0);
    s.FUN_001db870(&mut h);
    assert_eq!(s.dir, -1);
    assert!(h.log.contains(&"xf 0x437 40".to_string()), "cursor one row down");
    // square -> loot page, X sells one of kind 3
    h.press = PAD_SQUARE;
    s.FUN_001db870(&mut h);
    assert_eq!((s.page, s.count, s.loot), (1, 1, 3));
    let c = h.coins;
    h.press = PAD_X;
    s.FUN_001db870(&mut h);
    assert_eq!((h.loot[3], h.coins), (1, c + 17));
    // triangle closes the screen
    h.press = PAD_TRIANGLE;
    s.FUN_001db870(&mut h);
    assert_eq!((s.page, h.paused, h.scale), (-1, false, 1.0));
    assert!(h.log.contains(&"close".to_string()));
}

/// Not enough coins -> error dialog 0x35 (timed), FUN_001dbba0 / FUN_001dbbe0 scrolling past 6 rows.
#[test]
fn x3_thiefnet_scroll_and_poor() {
    let mut h = ShopMock { coins: 0, ep: 8, scale: 1.0, loot: vec![0; 0x40], ..Default::default() };
    let mut s = Shop::default();
    s.FUN_001dc260(&mut h, 0);
    assert!(s.count > 6);
    h.press = PAD_X;
    s.FUN_001db870(&mut h);
    assert_eq!((s.dialog, s.dialog_msg, s.dialog_t), (1, 0x35, 6.0));
    s.FUN_001dbbe0(&mut h, 9);
    assert_eq!((s.sel, s.first, s.FUN_001dbba0()), (9, 4, 10));
    s.FUN_001dbbe0(&mut h, 2);
    assert_eq!(s.first, 2);
    s.da570 = true;
    s.FUN_001dbbe0(&mut h, 9);
    assert_eq!((s.first, s.FUN_001dbba0()), (6, 10));
}

/// Census: DAT_002c3288 loot prices and DAT_002c3588 row ids / DAT_002c35b4 rows from SCUS_973.16.
#[test]
fn x3_thiefnet_elf_tables() {
    let iso = sly2_recomp::engine::disc::iso::IsoImage::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso")).expect("ISO");
    let elf = iso.read_file("SCUS_973.16").unwrap();
    let lp = sly2_recomp::engine::ui::thiefnet::loot_prices(&elf).unwrap();
    assert_eq!(&lp[..4], &[11, 19, 27, 17]);
    let w = |a: u32| i32::from_le_bytes(sly2_recomp::engine::disc::descriptor::elf_bytes(&elf, a, 4).unwrap().try_into().unwrap());
    for (i, e) in sly2_recomp::engine::ui::thiefnet::ROW_ELEMS.iter().enumerate() {
        assert_eq!(w(0x2c3588 + 4 * i as u32), *e);
    }
    assert_eq!(w(0x2c35b4), sly2_recomp::engine::ui::thiefnet::ROWS);
    assert_eq!(f32::from_bits(w(0x2c35b0) as u32), sly2_recomp::engine::ui::thiefnet::ROW_H);
    assert_eq!(f32::from_bits(w(0x2c3254) as u32), sly2_recomp::engine::ui::thiefnet::DIALOG_T);
}
