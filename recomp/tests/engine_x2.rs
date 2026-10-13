//! WP-X2 tests: gaps sweep 2 (audio leftovers of G3 + next gaps.md functions).
use sly2_recomp::engine::audio::emitter::EmitterSystem;
use sly2_recomp::engine::audio::emitter_tick::{EmitterWorld, NoWorld, Snd989, SndCmd, SoundTable, STEP_2A82A8, STEP_ROW_2A8350};
use sly2_recomp::engine::audio::music_mgr::{DialogLine, MusicClock, MusicHost, MusicMgr, NoMusicHost};

#[derive(Default)]
struct Rec {
    calls: Vec<(char, u32, i32)>,
}
impl MusicHost for Rec {
    fn ee_sub_1f80d0(&mut self, arg: i32, obj: u32) {
        self.calls.push(('a', obj, arg));
    }
    fn ee_sub_1f0660(&mut self, obj: u32, arg: i32) {
        self.calls.push(('b', obj, arg));
    }
    fn release_obj(&mut self, obj: u32) {
        self.calls.push(('r', obj, 0));
    }
}

struct Duck;
impl EmitterWorld for Duck {
    fn listener(&self) -> [f32; 3] {
        [0.0; 3]
    }
    fn to_camera(&self, v: [f32; 3]) -> [f32; 3] {
        v
    }
    fn attach_live(&self, _: i32) -> bool {
        false
    }
    fn attach_pos(&self, _: i32, p: [f32; 3]) -> [f32; 3] {
        p
    }
    fn duck_test(&self, _: i32) -> bool {
        true
    }
}

#[test]
fn x2_pickup_jingle_1feed0() {
    // tables read from the ELF (DAT_002a82a8 / DAT_002a8350): row r ends with r cells of +1
    for r in 0..7 {
        let row = &STEP_2A82A8[r * 6..r * 6 + 6];
        assert_eq!(row.iter().filter(|&&x| x == 1).count(), r);
    }
    assert_eq!(STEP_ROW_2A8350[0], 6);
    assert_eq!(STEP_ROW_2A8350[24], 0);
    let mut snd = Snd989::default();
    snd.table = SoundTable { kinds: vec![0; 4], map: (0..0x30).map(|i| if i == 0x28 { 3 } else { -1 }).collect(), bank: 1 };
    let mut seen = vec![];
    for _ in 0..500 {
        snd.ee_sub_1feed0(7);
        assert!((-12..=12).contains(&snd.step_138));
        seen.push(snd.step_138);
        match snd.cmds.last().unwrap() {
            SndCmd::Play { sound, bank_sound, mask, pitch, .. } => {
                assert_eq!((*sound, *bank_sound, *mask), (0x28, 3, 0x13));
                assert_eq!(*pitch as i32, snd.step_138 * 127);
            }
            c => panic!("{c:?}"),
        }
    }
    assert!(seen.iter().any(|&s| s > 0) && seen.iter().any(|&s| s < 0));
    // +0x64 set: nothing
    snd.f64 = 1;
    let n = snd.cmds.len();
    snd.ee_sub_1feed0(7);
    assert_eq!(snd.cmds.len(), n);
    // wired into FUN_001fbc58: a kind-0 start whose duck test matches plays the jingle too
    let mut snd = Snd989::default();
    snd.table = SoundTable { kinds: vec![0; 2], map: vec![0, 1], bank: 1 };
    let mut es = EmitterSystem::default();
    let e = es.new_emitter_1fbb90(1);
    es.ee_sub_1fbc58(&mut snd, &Duck, e);
    assert_eq!(snd.duck_1feed0, 1);
    assert_eq!(snd.cmds.len(), 2);
    assert!(matches!(snd.cmds[1], SndCmd::Play { sound: 0x28, mask: 0x13, .. }));
    es.ee_sub_1fbc58(&mut snd, &NoWorld, e);
    assert_eq!(snd.duck_1feed0, 1);
}

#[test]
fn x2_dialogue_line_end_1fe738() {
    let (mut m, mut es, mut snd) = (MusicMgr::default(), EmitterSystem::default(), Snd989::default());
    let a = es.new_emitter_1fbb90(1);
    es.add_ref(a); // the +0x200 reference
    es.set_flags_1fb658(a, 1, 0);
    m.cur = Some(a);
    m.objs = [0x100, 0, 0, 0x200, 0, 0x300];
    m.line = Some(DialogLine { duration: 0.25, cues: vec![(600, 0, 5), (30, 3, 7)], cycles: 0 });
    let mut h = Rec::default();
    let clk = MusicClock { game_state: 3, dt: 1.0 / 60.0, dt_unscaled: 0.5, ticks: 1 };
    m.ee_sub_1fd830(&mut es, &mut snd, &NoWorld, &mut h, clk);
    assert_eq!(m.line_end, 1);
    // remaining cues fired, slot-5 SMA 0x611, objects released and cleared, instance stopped and dropped
    assert_eq!(h.calls, vec![('b', 0x100, 5), ('a', 0x200, 7), ('b', 0x300, 0x611), ('r', 0x100, 0), ('r', 0x200, 0), ('r', 0x300, 0)]);
    assert_eq!((m.cur, m.objs, m.cue, m.cue_t, m.elapsed), (None, [0; 6], 0, 0.0, 0.0));
    assert!(m.line.is_none());
    // the stop request (flag 8) was consumed by the same step's FUN_001fb8e0 walk; the last reference went with it
    assert!(es.get(a).is_none());
    assert_eq!(es.freed, vec![(a, 1)]);
}

#[test]
fn x2_music_mgr_small() {
    let (mut m, mut es, mut snd) = (MusicMgr::default(), EmitterSystem::default(), Snd989::default());
    assert_eq!(m.ee_sub_1fe3f8(&es), 0.0);
    let a = es.new_emitter_1fbb90(1);
    es.get_mut(a).unwrap().cycles = 294_912_000;
    m.stack.push(a);
    assert!((m.ee_sub_1fe3f8(&es) - 1.0).abs() < 1e-4);
    // dialogue time: elapsed while idle with a zero clock, else the clock
    assert_eq!(m.ee_sub_1fe888(&es), 0.0);
    let d = es.new_emitter_1fbb90(2);
    es.add_ref(d);
    es.set_flags_1fb658(d, 1, 0);
    m.cur = Some(d);
    m.line = Some(DialogLine { duration: 1.0, cues: vec![], cycles: 0 });
    m.elapsed = 0.75;
    assert_eq!(m.ee_sub_1fe888(&es), 0.75);
    m.line.as_mut().unwrap().cycles = 294_912_000 * 2;
    assert!((m.ee_sub_1fe888(&es) - 2.0).abs() < 1e-4);
    // +0x68 bits
    m.f68 = 0;
    m.ee_sub_1fe5b8(3, 1);
    m.ee_sub_1fe5b8(1, 0);
    assert_eq!(m.f68, 2);
    // output mode / master volume only on change
    m.ee_sub_1fe9c8(&mut snd, 1);
    m.ee_sub_1fe9c8(&mut snd, 1);
    m.ee_sub_1fe9f8(&mut snd, 1.0);
    m.ee_sub_1fe9f8(&mut snd, 2.0);
    m.ee_sub_1fe9f8(&mut snd, 0.5);
    assert_eq!(snd.cmds, vec![SndCmd::OutputMode { mode: 1 }, SndCmd::BusVolume { bus: 0x10, vol: 0.5 }]);
    // FUN_001fedf0 keeps slot 1 (options)
    m.set_option_volumes(&mut es, &mut snd, 0.5, 0.25);
    m.vol[4][2] = 0.1;
    m.ee_sub_1fedf0(&mut es, &mut snd);
    assert_eq!(m.vol[4][2], 1.0);
    assert_eq!(m.vol[1], [0.25, 0.5, 1.0, 0.25]);
    assert_eq!(m.master, 1.0);
    // FUN_001fd700 level teardown
    snd.step_138 = 5;
    snd.table = SoundTable { kinds: vec![1], map: vec![0], bank: 9 };
    m.ee_sub_1fd700(&mut es, &mut snd, &mut NoMusicHost);
    assert!(snd.cmds.contains(&SndCmd::StopAll) && snd.cmds.contains(&SndCmd::Sukr { cmd: 10 }));
    assert_eq!((snd.step_138, snd.table.bank, snd.table.map.len()), (0, 9, 0));
    assert_eq!((m.cur, m.stack.len(), m.f68), (None, 0, 3));
    assert!(es.active.is_empty() && es.slots.is_empty());
}

use sly2_recomp::engine::ui::movie::*;

#[derive(Default)]
struct MvRec {
    log: Vec<String>,
}
impl MovieHost for MvRec {
    fn ee_sub_217ca8(&mut self, mv: &mut Movie, sprite: usize, name: i32, state: usize, _m: &[f32; 6], _a: u32, _b: u32) {
        self.log.push(format!("start {sprite} {name:#x} {state}"));
        mv.sprites[sprite].states[state].def = 0; // the clip of name 0x314
    }
    fn sound_start(&mut self, s: i32) -> u32 {
        self.log.push(format!("snd {s}"));
        0x55
    }
    fn sound_stop(&mut self, h: u32) {
        self.log.push(format!("stop {h:#x}"));
    }
    fn sound_play(&mut self, h: u32) {
        self.log.push(format!("play {h:#x}"));
    }
    fn music_1fe738(&mut self) {
        self.log.push("1fe738".into());
    }
}

fn pl(id: u16, elem: u16, x: f32, tween: u8, colour: i16) -> Placement {
    Placement { id, elem, mask: 0, colour, m: [1.0, 0.0, 0.0, 1.0, x, 0.0], tween, b31: 0, w34: 0 }
}

/// root sprite 0 (one clip, 3 frames) placing proto 1 = nested sprite 1 (one clip, one frame) in element 0
fn movie() -> Movie {
    let child = Sprite {
        defs: vec![ClipDef { frames: vec![Frame { time: 0.0, places: vec![pl(0, 0, 0.0, 0, -1)], ..Default::default() }], len: 1.0, name: 0x314 }],
        states: vec![ClipState::new(1), ClipState::new(1)],
        ..Default::default()
    };
    let f0 = Frame { time: 0.0, places: vec![pl(1, 0, 0.0, 1, 0)], colours: vec![(0xffff_ffff, 0x7f7f_7f7f)], cues: vec![(0, 9)], ..Default::default() };
    let f1 = Frame { time: 1.0, places: vec![pl(1, 0, 10.0, 0, 0)], colours: vec![(0x7f7f_7f7f, 0x7f7f_7f7f)], ..Default::default() };
    let f2 = Frame { time: 2.0, resets: vec![0], cmds: vec![7], cues: vec![(3, 0)], ..Default::default() };
    let root = Sprite { defs: vec![ClipDef { frames: vec![f0, f1, f2], len: 3.0, name: 1 }], states: vec![ClipState::new(2)], ..Default::default() };
    Movie { sprites: vec![root, child], protos: vec![Proto::default(), Proto { id: 1, kind: 4, sprite: 1, handle: 0 }] }
}

#[test]
fn x2_movie_helpers() {
    assert_eq!(mat_mul(&[2.0, 0.0, 0.0, 3.0, 1.0, 1.0], &[1.0, 0.0, 0.0, 1.0, 5.0, 6.0]), [2.0, 0.0, 0.0, 3.0, 6.0, 7.0]);
    assert_eq!(lerp_colour(0x0000_0010, 0x0000_0020, 0.5), 0x0000_0018);
    assert_eq!(lerp_colour(0x0000_0020, 0x0000_0010, 0.5), 0x0000_0018);
    assert_eq!(lerp_colour(0x10, 0x20, 0.0), 0x10);
    // neutral parent colour transform (mul 0xff.., add 0x7f..) keeps the element's
    assert_eq!(ee_sub_218790(0xffff_ffff, 0x7f7f_7f7f, 0x7f7f_7f7f, 0x8080_8080), (0x8080_8080, 0x8080_8080));
    assert_eq!(ee_sub_218790(0xffff_ffff, 0x7f7f_7f7f, 0xffff_ffff, 0x7f7f_7f7f), (0xffff_ffff, 0x7f7f_7f7f));
    // half parent multiplier halves the element's
    assert_eq!(ee_sub_218790(0xbfbf_bfbf, 0x7f7f_7f7f, 0xffff_ffff, 0xffff_ffff).0 & 0xff, 0xc0);
    let e = Elem::default();
    assert_eq!((e.m0, e.add, e.mul, e.w64, e.obj), (IDENT, ADD0, MUL0, 0xffff_ffff, None));
}

#[test]
fn x2_movie_timeline() {
    let mut mv = movie();
    let mut h = MvRec::default();
    mv.sprites[0].states[0].def = 0;
    // frame 0: sound cue, nested sprite created (free state 0 of sprite 1 started with name 0x314)
    mv.ee_sub_2183e8(&mut h, 0.0, 0, 0, &IDENT, ADD0, MUL0);
    assert_eq!(h.log, vec!["snd 9", "start 1 0x314 0"]);
    assert_eq!(mv.sprites[0].states[0].sound, 0x55);
    let e = &mv.sprites[0].states[0].elems[0];
    assert_eq!((e.obj, e.clip, e.t0, e.t1, e.m1.map(|m| m[4]), e.last), (Some(1), 0, 0.0, 1.0, Some(10.0), 0.0));
    assert_eq!(mv.sprites[0].states[0].next, 1);
    // tween half way: x = 5, mul halfway between 0xff and 0x7f
    mv.ee_sub_2183e8(&mut h, 0.5, 0, 0, &IDENT, ADD0, MUL0);
    let e = &mv.sprites[0].states[0].elems[0];
    assert_eq!((e.m[4], e.last), (5.0, 0.5)); // without a due frame every live element is updated at the state time
    // frame 1 at t = 1.0 (keeps proto 1: no restart of the nested clip)
    mv.ee_sub_2183e8(&mut h, 0.5, 0, 0, &IDENT, ADD0, MUL0);
    let e = &mv.sprites[0].states[0].elems[0];
    assert_eq!((e.m[4], e.last, e.mul & 0xff), (10.0, 1.0, 0x80));
    assert_eq!(h.log.len(), 2);
    // frame 2: music cue, stop command (looping sound restarted), element 0 removed (nested state stopped)
    mv.ee_sub_2183e8(&mut h, 1.0, 0, 0, &IDENT, ADD0, MUL0);
    assert_eq!(h.log[2..], ["1fe738", "play 0x55"]);
    assert_eq!(mv.sprites[0].states[0].mode, 1);
    assert_eq!(mv.sprites[0].states[0].elems[0].obj, None);
    assert_eq!((mv.sprites[1].states[0].def, mv.sprites[1].last_def), (-1, 0));
    assert_eq!(mv.ee_sub_217ef8(1), 0);
    // stopped: FUN_002188b0 (host) instead of the timeline
    mv.ee_sub_2183e8(&mut h, 1.0, 0, 0, &IDENT, ADD0, MUL0);
    assert_eq!(mv.sprites[0].states[0].time, 2.0); // no tick while stopped
    // stopping the root state drops its sound
    mv.ee_sub_218660(&mut h, 0, 0);
    assert_eq!(h.log.last().unwrap(), "stop 0x55");
    assert_eq!(mv.sprites[0].states[0].def, -1);
}

#[test]
fn x2_movie_restart_releases() {
    let mut mv = movie();
    let mut h = MvRec::default();
    mv.sprites[0].states[0].def = 0;
    mv.ee_sub_2183e8(&mut h, 0.0, 0, 0, &IDENT, ADD0, MUL0);
    // a live element frame 0 does not place (element 1) is released on the restart pass
    mv.sprites[0].states[0].elems[1].obj = Some(1);
    mv.sprites[0].states[0].elems[1].clip = 1;
    mv.sprites[0].states[0].elems[1].last = 0.25;
    mv.sprites[1].states[1].def = 0;
    mv.sprites[0].states[0].next = 0;
    mv.sprites[0].states[0].time = 0.0;
    mv.ee_sub_219fe0(&mut h, 0.0, 0, 0, 0, &IDENT, ADD0, MUL0);
    assert_eq!(mv.sprites[0].states[0].elems[1].obj, None);
    assert_eq!(mv.sprites[1].states[1].def, -1);
    assert_eq!(mv.sprites[0].states[0].elems[0].obj, Some(1));
}

#[test]
fn x2_movie_stopped_2188b0() {
    let mut mv = movie();
    let mut h = MvRec::default();
    mv.sprites[0].states[0].def = 0;
    mv.ee_sub_2183e8(&mut h, 0.0, 0, 0, &IDENT, ADD0, MUL0);
    // stopped at 0.25: no tick, no frames; the tweened element is drawn at the state time, empty ones take it
    let st = &mut mv.sprites[0].states[0];
    st.mode = 1;
    st.time = 0.25;
    mv.ee_sub_2183e8(&mut h, 1.0, 0, 0, &[1.0, 0.0, 0.0, 1.0, 0.0, 2.0], ADD0, MUL0);
    let st = &mv.sprites[0].states[0];
    assert_eq!((st.time, st.next), (0.25, 1));
    assert_eq!((st.elems[0].m[4], st.elems[0].m[5], st.elems[0].last), (2.5, 2.0, 0.25));
    assert_eq!(st.elems[1].last, 0.25);
    assert_eq!(h.log.len(), 2);
}
