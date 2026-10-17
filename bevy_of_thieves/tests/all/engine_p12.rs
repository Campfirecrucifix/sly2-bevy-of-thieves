//! WP-P12: player <-> classes hookup (K8 pickups / gifts, K7 hits / contacts -> P7, HUD counters), swing
//! points / poles from the level, and the JMP body ASEG events (double jump / ledge hang / damage).
use sly2_recomp::engine::classes::plugin::EngineHost;
use sly2_recomp::engine::classes::ClassSet;
use sly2_recomp::engine::disc::descriptor::Descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::world::{LoadOptions, World};
use sly2_recomp::engine::classes::guard::slots::Contact;
use sly2_recomp::engine::classes::guard::PLAYER_HANDLE;
use sly2_recomp::engine::classes::k8::Pickup;
use sly2_recomp::engine::classes::Inst;
use sly2_recomp::engine::physics::CollisionWorld;
use sly2_recomp::engine::player::p5_target::TargetGeom;
use sly2_recomp::engine::player::p6::{self, P6Level};
use sly2_recomp::engine::player::plugin::{link_classes, PlayerHud};
use sly2_recomp::engine::player::{self as pl, Jt, JtAnim, JtInput, JTS_JUMP};
use std::sync::Arc;

struct Rec(Vec<(u32, i32, i32)>);
impl JtAnim for Rec {
    fn play_aseg(&mut self, f: u32, jts: i32, jtbs: i32) {
        self.0.push((f, jts, jtbs));
    }
}

fn world(lsn: u32) -> Arc<World> {
    let (loader, descs) = levels();
    let d = descs.into_iter().find(|d| d.lsn() == lsn).unwrap();
    let lv = crate::common::load_blocking(&loader, d).unwrap();
    Arc::new(World::load(&lv, LoadOptions::default()).unwrap())
}

fn setup(w: &Arc<World>) -> (ClassSet, EngineHost) {
    let mut h = EngineHost { world: Some(w.clone()), player_pos: [1.0e9; 3], ..Default::default() };
    let mut s = ClassSet::build(w, &mut h);
    s.g.k6_players = vec![(PLAYER_HANDLE, [1.0e9; 3])];
    s.post_load(&mut h);
    s.g.dt = 1.0 / 60.0;
    (s, h)
}

/// one game frame in plugin order: player (tick + classes_link) then classes (tick_classes)
fn frame(jt: &mut Jt, s: &mut ClassSet, h: &mut EngineHost, hud: &mut PlayerHud, rec: &mut Rec) {
    link_classes(jt, s, h, rec, hud);
    hud.hp = jt.dmg.hp;
    s.g.k6_players = vec![(PLAYER_HANDLE, jt.phys.pos)];
    s.g.t += s.g.dt;
    s.frame(h);
}

/// Sly walks (moves through the pickup record) into a real clue bottle: K8 collects it, the HUD counter rises.
#[test]
fn sly_collects_real_clue_bottle() {
    let w = world(353856);
    let (mut s, mut h) = setup(&w);
    let hosts: Vec<usize> = w.objects.iter().enumerate().filter(|(_, o)| o.class == 7).map(|(i, _)| i).collect();
    let bottle = s.k8.iter().find_map(|(i, x)| if let Inst::K8(c) = x { (c.class == 95).then_some(*i) } else { None }).unwrap();
    s.k8_attach(bottle, hosts[0] as u32 + 1, &mut h);
    let p = s.k8_pickup(bottle).map(|c: &mut Pickup| c.xf_pos).unwrap();
    let mut hud = PlayerHud::default();
    let mut rec = Rec(vec![]);
    // Sly approaches along +x from 400 units away
    let mut jt = Jt::new([p[0] - 400.0, p[1], p[2]]);
    frame(&mut jt, &mut s, &mut h, &mut hud, &mut rec);
    assert_eq!(h.player_rec.unwrap().alive, 1);
    assert_eq!(hud.bottles, 0);
    let mut got = None;
    for k in 0..60 {
        jt.phys.pos[0] += 10.0;
        frame(&mut jt, &mut s, &mut h, &mut hud, &mut rec);
        link_classes(&mut jt, &mut s, &mut h, &mut rec, &mut hud);
        if hud.bottles == 1 {
            got = Some(k);
            break;
        }
    }
    let k = got.expect("bottle collected");
    println!("bottle collected after {k} frames at dx {}", jt.phys.pos[0] - p[0]);
    assert!(jt.phys.pos[0] - p[0] < 0.0, "only within reach (before passing the centre)");
    assert_eq!(s.k8_pickup(bottle).unwrap().f_150, 2);
    // gifts: slot 80 0x144268 heals clamped to max
    jt.dmg.hp = 1;
    h.player_gifts.push((2, 0));
    h.player_gifts.push((1000, 0));
    link_classes(&mut jt, &mut s, &mut h, &mut rec, &mut hud);
    assert_eq!(jt.dmg.hp, jt.dmg.hp_max);
    assert_eq!(hud.gifts, 2);
    assert!(h.player_gifts.is_empty());
}

/// A guard on a real level: Sly next to it yields a body contact (no hit), an attack contact yields a K7 hit that
/// lands in TakeDamage through classes_link and lowers hp.
#[test]
fn guard_hit_lowers_sly_hp() {
    let (loader, descs) = levels();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let (mut s, mut h) = setup(&w);
        let Some(gi) = s.k6.first().map(|x| x.0) else { continue };
        let mut hud = PlayerHud::default();
        let mut rec = Rec(vec![]);
        let g = s.k6_guard(gi).unwrap().body.pos;
        let mut jt = Jt::new([g[0] + 10.0, g[1], g[2]]);
        let hp0 = jt.dmg.hp;
        link_classes(&mut jt, &mut s, &mut h, &mut rec, &mut hud);
        assert!(h.k7_contacts.iter().any(|(i, c)| *i == gi && c.other == PLAYER_HANDLE), "body contact");
        s.g.t += s.g.dt;
        s.frame(&mut h);
        assert!(h.player_hits.is_empty(), "body contact is no hit");
        h.k7_contacts.push((gi, Contact { kind: 0x200, amount: -10, other: PLAYER_HANDLE, vec: [300.0, 0.0, 400.0, 0.0], ..Default::default() }));
        s.g.t += s.g.dt;
        s.frame(&mut h);
        assert_eq!(h.player_hits.len(), 1);
        link_classes(&mut jt, &mut s, &mut h, &mut rec, &mut hud);
        assert!(h.player_hits.is_empty(), "drained");
        assert_eq!(hud.hits_taken, 1);
        assert!(jt.dmg.hp < hp0, "hp {hp0} -> {}", jt.dmg.hp);
        assert!(rec.0.iter().any(|e| e.0 >= 0xe90 && e.0 < 0xe90 + 0x40), "damage ASEG event: {:?}", rec.0);
        assert_eq!(h.player_rec.unwrap().alive, 0, "+0x15c cleared while the damage state runs");
        println!("lsn {} guard {gi}: hp {hp0} -> {}", d.lsn(), jt.dmg.hp);
        return;
    }
    panic!("no level with guards");
}

/// Swing points (class 19) of a real level are P5 targets; O in the air near one flies to it and hangs.
#[test]
fn swing_point_on_real_level() {
    let w = world(730464);
    let lvl = P6Level::load(&w);
    let pts: Vec<_> = lvl.targets.iter().filter(|t| matches!(t.obj.geom, TargetGeom::Point { .. })).collect();
    assert!(pts.len() >= 10, "points {}", pts.len());
    // WP-P15: points inside placed prefab copies are keyed by the copy root (a clone, any class)
    assert!(pts.iter().all(|t| (w.objects[t.obj.id].class == 19 || w.objects[t.obj.id].clone_of.is_some()) && t.obj.mask == 7));
    assert_eq!(p6::level_point_targets(&w).len(), pts.len());
    let at = |t: &pl::P5Target| if let TargetGeom::Point { pos, .. } = t.obj.geom { pos } else { unreachable!() };
    // (objects 50 / 610 sit at the origin: owner-bone frames of unplaced templates)
    let t = *pts.iter().find(|t| at(t)[0].abs() > 1.0).unwrap();
    let pos = at(t);
    let cw = CollisionWorld::default();
    let mut jt = Jt::new([pos[0] - 150.0, pos[1], pos[2] + 50.0]);
    lvl.apply(&mut jt);
    jt.t = 10.0;
    jt.mask_1000 = 1;
    jt.set_jts(JTS_JUMP, pl::JTBS_JUMP_FALL, &mut ());
    jt.set_velocity_pub([300.0, 0.0, 200.0]);
    let step = |jt: &mut Jt, held, pressed| {
        jt.pad_pending = Some((held, pressed));
        jt.update(&cw, JtInput::default(), 1.0 / 60.0, &mut ());
    };
    use sly2_recomp::engine::input::pad::BTN_CIRCLE;
    step(&mut jt, BTN_CIRCLE, BTN_CIRCLE);
    assert_eq!((jt.jts, jt.jtbs), (JTS_JUMP, 0xe), "O picks the swing point (kind 0xe flight)");
    let mut n = 0;
    while jt.jts == JTS_JUMP {
        step(&mut jt, BTN_CIRCLE, 0);
        n += 1;
        assert!(n < 400, "never arrived");
    }
    // 0x19c688 case 0xe -> enter_swing: SetJtJts(3, 1) (class-19 +0x124 != 3), else SetJtJts(0x16, -1)
    // (WP-P14: +0x124 = prop 1011 is now read)
    let ride = lvl.p14.swing.iter().any(|s| s.0 == t.obj.id && s.1 == 3);
    assert_eq!((jt.jts, jt.jtbs), if ride { (0x16, -1) } else { (3, 1) }, "arrival swings on the point");
    let d = [jt.phys.pos[0] - pos[0], jt.phys.pos[1] - pos[1], jt.phys.pos[2] - pos[2]];
    let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    println!("point {} at {:?}: swing after {n} frames, dist {dist}", t.obj.id, pos);
    assert!(dist < 300.0, "near the point: {dist}");
}

/// 0x195fb8 tail: double jump / ledge hang send the JMP body ASEG fields; damage sends 0xe90 + 4s (P7).
#[test]
fn jmp_body_anim_events() {
    assert_eq!(pl::jmp_body_aseg_195fb8(pl::JTBS_DJUMP_INIT, 2), Some(0x3fc));
    assert_eq!(pl::jmp_body_aseg_195fb8(pl::JTBS_LEDGE, 0), Some(0x414));
    assert_eq!(pl::jmp_body_aseg_195fb8(pl::JTBS_JUMP_FALL, 0), None);
    let mut jt = Jt::new([0.0, 0.0, 500.0]);
    let mut rec = Rec(vec![]);
    jt.jmp_idx = 1;
    jt.FUN_0018c250(JTS_JUMP, pl::JTBS_DJUMP_INIT, &mut rec);
    assert_eq!(rec.0.last(), Some(&(0x3f8, JTS_JUMP, pl::JTBS_DJUMP_INIT)), "{:?}", rec.0);
    jt.FUN_0018c250(JTS_JUMP, pl::JTBS_LEDGE, &mut rec);
    assert_eq!(rec.0.last(), Some(&(0x414, JTS_JUMP, pl::JTBS_LEDGE)), "{:?}", rec.0);
}

fn levels() -> (LevelLoader, Vec<Descriptor>) {
    crate::common::levels() // public: ISO level table instead of the analysis DB
}

#[test]
#[ignore]
fn probe() {
    use sly2_recomp::engine::player::p4::FUN_00132b10;
    let (loader, descs) = levels();
    for d in descs {
        let lv = crate::common::load_blocking(&loader, d).unwrap();
        let w = Arc::new(World::load(&lv, LoadOptions::default()).unwrap());
        let mut cnt = std::collections::BTreeMap::<(i32, u16), usize>::new();
        for o in &w.objects {
            for k in [0x13, 0x22, 0x26, 0xb, 0x1b, 0x23, 0xc, 0x5b, 95] {
                if FUN_00132b10(o.class as i32, k) {
                    *cnt.entry((k, o.class)).or_default() += 1;
                }
            }
        }
        for o in &w.objects {
            if matches!(o.class, 19 | 38 | 35 | 36 | 37 | 27) {
                let props: String = match &o.payload {
                    sly2_recomp::engine::world::objects::Payload::Loaded(ld) => format!("{:?}", ld).chars().take(600).collect::<String>(),
                    _ => String::new(),
                };
                if o.class == 38 {
                    for k in [88746, 88744, 88748, 88750] {
                        for j in w.find_oid(k) {
                            println!("  MEMBER {} -> cls {} ", k, w.objects[*j].class);
                        }
                    }
                }
                println!("  cls {} oid {} {}", o.class, o.oid, props);
            }
        }
        let mut h = EngineHost { world: Some(w.clone()), ..Default::default() };
        let s = ClassSet::build(&w, &mut h);
        let mut gp = vec![];
        for (i, _) in s.k6.iter().take(3) {
            let o = &w.objects[*i];
            let alo = sly2_recomp::engine::gfx::attach::alo_of(&w, *i);
            let parts: Vec<_> = alo.map(|a| a.parts.iter().filter(|p| p.w1 != 0).map(|p| (p.w0, p.w1, p.s0, p.f, p.b0, p.b1, p.b2, p.r#ref)).collect()).unwrap_or_default();
            gp.push((o.class, o.children.len(), parts));
        }
        println!("lsn {} cls {:?} guards {} parts {:?}", d.lsn(), cnt, s.k6.len(), gp);
    }
}
