//! WP-X1: largest-gaps sweep (language selection, memory-card dialog, ...).
#![allow(non_snake_case)]
use std::path::PathBuf;

fn root() -> PathBuf {
    let r = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    if r.join("Sly 2 - Band of Thieves (USA).iso").exists() { r } else { PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/..")) }
}
fn elf() -> Vec<u8> {
    let p = root().join("bin/SCUS_973.16");
    if let Ok(d) = std::fs::read(&p) {
        return d;
    }
    sly2_recomp::engine::disc::iso::IsoImage::open(root().join("Sly 2 - Band of Thieves (USA).iso")).expect("ISO").read_file("SCUS_973.16").unwrap()
}
/// ELF word at a virtual address (program headers)
fn word(d: &[u8], a: u32) -> u32 {
    let u = |o: usize| u32::from_le_bytes(d[o..o + 4].try_into().unwrap());
    let (ph, n) = (u(0x1c) as usize, u16::from_le_bytes([d[0x2c], d[0x2d]]) as usize);
    for i in 0..n {
        let b = ph + i * 32;
        let (off, va, fs) = (u(b + 4), u(b + 8), u(b + 16));
        if va <= a && a < va + fs {
            return u((off + a - va) as usize);
        }
    }
    panic!("{a:#x} not in the ELF")
}

mod language {
    use super::*;
    use sly2_recomp::engine::ui::language::*;

    #[test]
    fn x1_language_tables_match_elf() {
        let d = elf();
        for (r, list) in REGIONS_2BFBD0.iter().enumerate() {
            let a = 0x2bfbd0 + 8 * r as u32;
            assert_eq!(word(&d, a) as usize, list.len(), "region {r} count");
            let p = word(&d, a + 4);
            for (i, v) in list.iter().enumerate() {
                assert_eq!(word(&d, p + 4 * i as u32) as i32, *v, "region {r}[{i}]");
            }
        }
        for (i, v) in SCF_LANGUAGE_2A69E8.iter().enumerate() {
            assert_eq!(word(&d, 0x2a69e8 + 4 * i as u32) as i32, *v, "2a69e8[{i}]");
        }
    }

    #[test]
    fn x1_language_selection() {
        // US boot: region 0, English, one language -> no language menu
        let mut l = Language::boot();
        assert_eq!((l.region, l.lang), (0, 0));
        assert_eq!(l.FUN_001a2c98(), 1);
        // PAL region 2 with a German system (sce 4) -> game language 3
        let mut p = Language::default();
        p.FUN_001a2c20(2);
        p.FUN_001a2da0(4);
        assert_eq!(p.lang, 3);
        assert_eq!(p.FUN_001a2c98(), 5);
        // Dutch (sce 6 -> index 5) is not offered in region 2 -> first language
        p.FUN_001a2da0(6);
        assert_eq!(p.lang, 0);
        // region > 6 resets to 0 (written back)
        p.FUN_001a2c20(9);
        assert_eq!(p.FUN_001a2c48(), &[0]);
        assert_eq!(p.region, 0);
        // -1 region -> 0; -1 language -> region's first
        p.FUN_001a2c20(4);
        p.FUN_001a2bd0(-1);
        assert_eq!(p.lang, 11);
        assert_eq!(l.FUN_001dd8e0(2).msg, 0x189);
        assert_eq!(l.lang, 2);
    }

    #[test]
    fn x1_stored_language_ignored_on_us_region() {
        use sly2_recomp::engine::ui::settings::OptState;
        let st = OptState::default();
        assert_eq!((st.region, st.lang), (0, 0));
    }
}

mod card_dialog {
    use sly2_recomp::engine::ui::card_dialog::*;

    #[derive(Default)]
    struct H {
        now: f32,
        ready: Option<f32>,
        busy_world: bool,
        pressed: u32,
        thread_busy: bool,
        thread_error: bool,
        jobs: Vec<u32>,
        card_state: i32,
        shown: Vec<i32>,
        hidden: u32,
        f6330: i32,
        f6338: i32,
        e659c: i32,
        warned: i32,
        dt: f32,
        saves: u32,
        title: bool,
        b0160: bool,
        save_menu: bool,
        log: Vec<&'static str>,
    }
    impl CardDialogHost for H {
        fn now(&self) -> f32 { self.now }
        fn screen_ready(&self) -> Option<f32> { self.ready }
        fn world_busy(&self) -> bool { self.busy_world }
        fn pressed(&mut self, mask: u32) -> bool { self.pressed & mask != 0 }
        fn thread_busy(&self) -> bool { self.thread_busy }
        fn thread_error(&self) -> bool { self.thread_error }
        fn start_job(&mut self, job: u32) { self.jobs.push(job) }
        fn card_state(&self) -> i32 { self.card_state }
        fn screen_show(&mut self, msg: i32) { self.shown.push(msg) }
        fn screen_hide(&mut self) { self.hidden += 1 }
        fn set_f6330_from_f6338(&mut self) { self.f6330 = self.f6338 }
        fn clear_f6330(&mut self) { self.f6330 = 0 }
        fn clear_f6338(&mut self) { self.f6338 = 0 }
        fn set_f6338(&mut self, slot: i32) { self.f6338 = slot + 1 }
        fn set_e659c(&mut self, v: i32) { self.e659c = v }
        fn autosave_warned(&self) -> i32 { self.warned }
        fn set_autosave_warned(&mut self) { self.warned = 1 }
        fn slot_dt(&self, _slot: i32) -> f32 { self.dt }
        fn save(&mut self) { self.saves += 1 }
        fn title_active(&self) -> bool { self.title }
        fn menu_b0160(&self) -> bool { self.b0160 }
        fn is_save_menu(&self) -> bool { self.save_menu }
        fn menu_close_all(&mut self) { self.log.push("close_all") }
        fn menu_pop(&mut self) { self.log.push("pop") }
        fn new_game(&mut self) { self.log.push("new_game") }
        fn quit_to_title(&mut self) { self.log.push("quit") }
    }
    fn dlg() -> CardDialog {
        CardDialog { screen: true, pad: true, ..CardDialog::default() }
    }

    #[test]
    fn x1_card_dialog_button_layouts() {
        let mut d = CardDialog::default();
        assert_eq!(d.msg, -1);
        for (m, k) in [(0x11, 1), (0x2e, 1), (0x12, 2), (0x20, 2), (0x13, 0), (0x1a, 0)] {
            d.msg = m;
            assert_eq!(d.FUN_001acc80(), k, "{m:#x}");
        }
    }

    #[test]
    fn x1_card_dialog_waits_for_screen() {
        let mut h = H { now: 1.0, ready: Some(0.95), pressed: 0x80, ..H::default() };
        let mut d = dlg();
        d.FUN_001acb58(&mut h, 0x14);
        d.FUN_001accc8(&mut h); // only 0.05 s shown
        assert_eq!(d.msg, 0x14);
        h.now = 1.2;
        h.busy_world = true;
        d.FUN_001accc8(&mut h);
        assert_eq!(d.msg, 0x14);
        h.busy_world = false;
        h.save_menu = true;
        d.FUN_001accc8(&mut h); // X acknowledges -> hide, mode 2, pop the save menu
        assert_eq!((d.msg, d.mode, h.hidden), (-1, 2, 1));
        assert_eq!(h.log, ["pop"]);
    }

    #[test]
    fn x1_card_dialog_format_flow() {
        // 0x12 "format?" yes -> job 0x2a6e88 + 0x13; thread done -> job 0x2a6d88 + 0x16; done -> hide
        let mut h = H { now: 5.0, ready: Some(0.0), pressed: 0x80, ..H::default() };
        let mut d = dlg();
        d.FUN_001acb58(&mut h, 0x12);
        d.FUN_001accc8(&mut h);
        assert_eq!((d.msg, h.jobs.clone()), (0x13, vec![JOB_2A6E88]));
        h.pressed = 0;
        h.thread_busy = true;
        d.FUN_001accc8(&mut h);
        assert_eq!(d.msg, 0x13);
        h.thread_busy = false;
        d.FUN_001accc8(&mut h);
        assert_eq!((d.msg, h.jobs.clone()), (0x16, vec![JOB_2A6E88, JOB_2A6D88]));
        h.thread_error = true;
        d.FUN_001accc8(&mut h);
        assert_eq!(d.msg, 0x17);
        // triangle on a 2-button message = no
        d.FUN_001acb58(&mut h, 0x20);
        h.pressed = 0x10;
        d.FUN_001accc8(&mut h);
        assert_eq!(d.msg, -1);
        // without a pad a 2-button message waits for its own step: 0x20 closes unless mode 1
        let mut d = CardDialog { screen: true, mode: 1, ..CardDialog::default() };
        d.FUN_001acb58(&mut h, 0x20);
        d.FUN_001accc8(&mut h);
        assert_eq!(d.msg, 0x20);
    }

    #[test]
    fn x1_card_dialog_save_slot_flow() {
        let mut h = H { now: 5.0, ready: Some(0.0), ..H::default() };
        let mut d = dlg();
        d.purpose = 1;
        // empty slot, autosave not yet warned -> 0x22
        assert_eq!(d.FUN_001ad480(&mut h, 2), 2);
        assert_eq!((d.msg, h.f6330, h.e659c), (0x22, 3, 2));
        // X on 0x22 -> warned, save, 0x1a
        h.pressed = 0x80;
        d.FUN_001accc8(&mut h);
        assert_eq!((d.msg, h.warned, h.saves), (0x1a, 1, 1));
        // save thread finished OK -> hide + quit to title (purpose 1)
        h.pressed = 0;
        d.FUN_001accc8(&mut h);
        assert_eq!(d.msg, -1);
        assert_eq!(h.log, ["quit"]);
        // used slot -> overwrite question 0x1c, yes -> direct save (already warned)
        h.dt = 12.5;
        d.FUN_001ad480(&mut h, 0);
        assert_eq!(d.msg, 0x1c);
        h.pressed = 0x80;
        d.FUN_001accc8(&mut h);
        assert_eq!((d.msg, h.saves), (0x1a, 2));
        // save error -> 0x1d (purpose != 3)
        h.pressed = 0;
        h.thread_error = true;
        d.FUN_001accc8(&mut h);
        assert_eq!(d.msg, 0x1d);
    }

    #[test]
    fn x1_card_dialog_no_card_on_title_starts_new_game() {
        let mut h = H { now: 5.0, ready: Some(0.0), pressed: 0x80, title: true, b0160: true, ..H::default() };
        let mut d = dlg();
        d.FUN_001acb58(&mut h, 0x11);
        d.FUN_001accc8(&mut h);
        assert_eq!(d.msg, -1);
        assert_eq!(h.log, ["close_all", "new_game"]);
        // 0x11 without input closes itself once the card manager reaches state 6
        h.pressed = 0;
        d.FUN_001acb58(&mut h, 0x11);
        d.FUN_001accc8(&mut h);
        assert_eq!(d.msg, 0x11);
        h.card_state = 6;
        d.FUN_001accc8(&mut h);
        assert_eq!(d.msg, -1);
    }
}

mod card_busy {
    use sly2_recomp::engine::ui::card_dialog::*;
    struct H {
        s: Option<ScreenElem>,
        busy: bool,
        log: Vec<&'static str>,
    }
    impl CardBusyHost for H {
        fn screen(&self) -> Option<ScreenElem> { self.s }
        fn show(&mut self) { self.log.push("show") }
        fn hide(&mut self) { self.log.push("hide") }
        fn thread_busy(&self) -> bool { self.busy }
    }

    #[test]
    fn x1_card_busy_min_one_second() {
        let mut b = CardBusy::default();
        let mut h = H { s: None, busy: false, log: vec![] };
        b.FUN_001ade38(&mut h);
        assert!(h.log.is_empty() && !b.FUN_001adea8(&h));
        h.s = Some(ScreenElem { s58: 2, s64: 2, t5c: 10.0 });
        b.FUN_001ade38(&mut h);
        assert!(b.FUN_001adea8(&h));
        b.FUN_001adee8(&mut h, 10.9);
        h.busy = true;
        b.FUN_001adee8(&mut h, 11.5);
        assert_eq!(h.log, ["show"]);
        h.busy = false;
        b.FUN_001adee8(&mut h, 11.5);
        assert_eq!(h.log, ["show", "hide"]);
        // quirk kept: visible while +0x58 >= 2 OR +0x64 >= 2
        h.s = Some(ScreenElem { s58: 1, s64: 2, t5c: 0.0 });
        assert!(b.FUN_001adea8(&h));
        h.s = Some(ScreenElem { s58: 1, s64: 1, t5c: 0.0 });
        assert!(!b.FUN_001adea8(&h));
    }
}
