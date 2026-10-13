//! WP-P7: damage, health, death and respawn of Sly on a real level (level-table row 3), and the 1:1 pieces
//! of the damage component (jt+0xdf0) that do not need a level.
use sly2_recomp::engine::disc::descriptor;
use sly2_recomp::engine::disc::iso::IsoImage;
use sly2_recomp::engine::disc::mods::ModOverlay;
use sly2_recomp::engine::disc::LevelLoader;
use sly2_recomp::engine::game::spawn::{ee_sub_15e9c8, spawn_position};
use sly2_recomp::engine::physics::CollisionWorld;
use sly2_recomp::engine::player::p7_damage::{self as dmg, Cont, DmgEv, Hit, HitSrc, Respawn, HP_MAX, JTS_DMG};
use sly2_recomp::engine::player::{Jt, JtInput, JTS_JUMP, JT_RADIUS};
use sly2_recomp::engine::world::{LoadOptions, World};
use std::sync::{Arc, OnceLock};

const ISO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../Sly 2 - Band of Thieves (USA).iso");
const DT: f32 = 1.0 / 60.0;

fn world() -> Option<&'static (World, descriptor::Descriptor)> {
    static W: OnceLock<Option<(World, descriptor::Descriptor)>> = OnceLock::new();
    W.get_or_init(|| {
        let iso = Arc::new(IsoImage::open(ISO).ok()?);
        let t = descriptor::static_table(&iso.read_file("SCUS_973.16").ok()?)?;
        let l = LevelLoader::new(iso, ModOverlay::default());
        let boot = l.load_blocking(t[25]).ok()?;
        let d = boot.header.level_table.entries[3].desc;
        let lv = l.load_blocking(d).ok()?;
        Some((World::load(&lv, LoadOptions::default()).ok()?, d))
    })
    .as_ref()
}

/// Sly standing on the real level's ground at its first usable spawn object.
fn grounded_jt(w: &World, cw: &CollisionWorld) -> Option<(Jt, i32)> {
    let (_, cands) = ee_sub_15e9c8(w, None, 0x2a);
    for &i in &cands {
        let Some(p) = spawn_position(w, i) else { continue };
        let mut jt = Jt::new([p[0], p[1], p[2] + JT_RADIUS + 5.0]);
        for _ in 0..90 {
            jt.update(cw, JtInput::default(), DT, &mut ());
        }
        if jt.grounded && jt.jts != JTS_JUMP {
            return Some((jt, w.objects[i].oid));
        }
    }
    None
}

fn knock(amount: i32) -> Hit {
    Hit { src: Some(HitSrc { isa: vec![0x33], ..Default::default() }), kind: 0x200, amount, vec: [400.0, 0.0, 600.0] }
}

#[test]
fn table_and_pure_parts() {
    // 0x18d568 SetJtJts table 0x2a5d00
    assert_eq!((0..9).map(|m| dmg::FUN_0018d568(m).unwrap()).collect::<Vec<_>>(), [0, 2, 7, 4, 0x12, 0x13, 0x14, 0x17, 8]);
    assert_eq!(dmg::FUN_0018d568(9), None);
    let mut d = dmg::Dmg::default();
    assert_eq!((d.hp, d.hp_max, d.state, d.flash, d.fade_dur), (HP_MAX, 0x28, -1, -1, 2.0));
    // 0x144418 lethal: max <= -amount, kinds 0 / 1 / 0x80000 / 0x100000 never
    assert!(d.FUN_00144418(&Hit { kind: 0x200, amount: -40, ..Default::default() }));
    assert!(!d.FUN_00144418(&Hit { kind: 0x200, amount: -39, ..Default::default() }));
    assert!(!d.FUN_00144418(&Hit { kind: 1, amount: -100, ..Default::default() }));
    // 0x144188 clamp, 0x1440d0 refill when hpCur == 0
    d.FUN_00144188(99);
    assert_eq!(d.hp, 0x28);
    d.hp = 0;
    d.FUN_001440d0(true);
    assert_eq!(d.hp, 0x28);
    d.hp = 5;
    d.FUN_001440d0(true);
    assert_eq!(d.hp, 5);
    // 0x1a23f8 fallback chain 0x2bfb28: state 0xb -> 8 -> 0
    d.state = 0xb;
    d.aseg[0] = Some(0x77);
    assert_eq!(d.FUN_001a23f8(), Some(0x77));
    d.state = 4;
    assert_eq!(d.FUN_001a23f8(), None);
    // 0x155228 difficulty lerp
    let env = dmg::DmgEnv { diff: [1.0, 2.0, 0.5], ..Default::default() };
    assert_eq!(dmg::FUN_00155228(&env, -10), -15);
    // 0x11e9f8 clamp
    assert_eq!(dmg::FUN_0011e9f8(1200.0, [0.0, 0.0, 2400.0]), [0.0, 0.0, 1200.0]);
}

/// take damage -> lose health -> knocked back -> lands -> recovers; flash fades after 2 s.
#[test]
fn take_damage_on_real_level() {
    let Some((w, _)) = world() else {
        eprintln!("ISO missing: skipped");
        return;
    };
    let cw = CollisionWorld::build(w);
    let (mut jt, _) = grounded_jt(w, &cw).expect("a spawn with ground");
    let env = jt.dmg_env;
    assert!(dmg::FUN_001a19a0(&mut jt, &knock(-10), &env, &mut ()));
    assert_eq!(jt.dmg.hp, 30);
    assert_eq!(jt.jts, JTS_DMG);
    assert_eq!(jt.dmg.state, 7);
    assert_eq!(jt.dmg.flash, 1);
    assert_eq!(jt.dmg.cont, Cont::C1a1f68);
    assert!(jt.dmg.ev.contains(&DmgEv::Event(0xf)));
    // class 0x33 source: 1.25 * hit vector
    let v = jt.phys.v;
    let vi = [jt.phys.v[0] + jt.phys.imp_lin[0], v[1] + jt.phys.imp_lin[1], v[2] + jt.phys.imp_lin[2]];
    assert!((vi[0] - 500.0).abs() < 1e-3 && (vi[2] - 750.0).abs() < 1e-3, "{vi:?}");
    // hurt flash: further hits are ignored (0x1a2230: state != -1 / flash == 1)
    assert!(!dmg::FUN_001a19a0(&mut jt, &knock(-10), &env, &mut ()));
    let mut left = None;
    for f in 0..600 {
        jt.update(&cw, JtInput::default(), DT, &mut ());
        if jt.jts != JTS_DMG {
            left = Some(f);
            break;
        }
    }
    let f = left.expect("recovered from the knockback");
    eprintln!("knockback: {} frames, pos {:?}", f, jt.phys.pos);
    assert!(f as f32 * DT > 0.25, "state 7 lasts until landing after 0.25 s ({f})");
    assert_eq!(jt.dmg.state, -1);
    assert_eq!(jt.dmg.hp, 30);
    assert_eq!(jt.dmg.flash, 1);
    for _ in 0..200 {
        jt.update(&cw, JtInput::default(), DT, &mut ());
    }
    assert_eq!(jt.dmg.flash, -1, "flash fades after FLASH_DUR");
    assert_eq!(jt.dmg.alpha, 0.0);
    assert!(jt.respawn.is_none());
}

/// health runs out -> death state 8 -> die (event 0x10) -> reload of the current level at the spawn it was
/// entered with -> the new JT has full health (0x1440d0).
#[test]
fn die_and_respawn_at_checkpoint() {
    let Some((w, desc)) = world() else {
        eprintln!("ISO missing: skipped");
        return;
    };
    let cw = CollisionWorld::build(w);
    let (mut jt, spawn_oid) = grounded_jt(w, &cw).expect("a spawn with ground");
    jt.dmg_env.spawn = (spawn_oid, -1);
    let env = jt.dmg_env;
    let mut hits = 0;
    while jt.dmg.hp > 0 {
        // each hit waits for the previous one to clear (state / flash immunity)
        jt.dmg.flash = -1;
        if jt.jts == JTS_DMG {
            for _ in 0..600 {
                jt.update(&cw, JtInput::default(), DT, &mut ());
                if jt.jts != JTS_DMG || jt.respawn.is_some() {
                    break;
                }
            }
            jt.dmg.flash = -1;
        }
        assert!(dmg::FUN_001a19a0(&mut jt, &knock(-15), &env, &mut ()), "hit {hits} taken");
        hits += 1;
    }
    assert_eq!(hits, 3, "40 hp: 25, 10, 0");
    assert_eq!(jt.dmg.hp, 0);
    assert_eq!(jt.dmg.flash, -1, "no new flash on the killing hit");
    let mut died = false;
    for _ in 0..600 {
        jt.update(&cw, JtInput::default(), DT, &mut ());
        if jt.respawn.is_some() {
            died = true;
            break;
        }
    }
    assert!(died, "death reached");
    assert!((8..=9).contains(&jt.dmg.state), "death state {}", jt.dmg.state);
    assert!(jt.dmg.ev.contains(&DmgEv::Die) && jt.dmg.ev.contains(&DmgEv::Event(0x10)));
    let r = jt.respawn.unwrap();
    eprintln!("died: spawn oid {spawn_oid}, state {}", jt.dmg.state);
    assert_eq!(r, Respawn { level: 0, spawn: spawn_oid, spawn_parent: -1, flags: 0, a: 0x30a });
    let req = r.level_request(*desc);
    assert_eq!((req.desc, req.spawn, req.spawn_parent, req.flags), (*desc, spawn_oid, -1, 2));
    // the reload: spawn object -> new JT (0x1400b8 -> 0x1440d0: hpCur 0 -> max)
    let i = w.objects.iter().position(|o| o.oid == spawn_oid).unwrap();
    let p = spawn_position(w, i).unwrap();
    let mut jt2 = Jt::new([p[0], p[1], p[2] + JT_RADIUS + 5.0]);
    jt2.dmg.hp = jt.dmg.hp;
    jt2.dmg.FUN_001440d0(true);
    assert_eq!(jt2.dmg.hp, HP_MAX);
    for _ in 0..90 {
        jt2.update(&cw, JtInput::default(), DT, &mut ());
    }
    assert!(jt2.grounded, "respawned on the checkpoint's ground");
}

/// lethal hit (0x144418): delta -0x8000, kind 0x4000 with no health -> state 0xb (no continuation) -> die.
#[test]
fn lethal_hit_and_heal() {
    let mut jt = Jt::new([0.0, 0.0, 0.0]);
    let env = jt.dmg_env;
    // heal while hurt: +5, heal flash 0
    jt.dmg.hp = 20;
    assert!(dmg::FUN_001a19a0(&mut jt, &Hit { kind: dmg::K_HEAL, amount: 5, ..Default::default() }, &env, &mut ()));
    assert_eq!((jt.dmg.hp, jt.dmg.flash, jt.dmg.state), (25, 0, -1));
    // jts 7 / 3 / 8 / 0x15 / 0x16 veto of kind 0x4000 (0x1a21a0)
    jt.jts = 3;
    assert!(!dmg::FUN_001a19a0(&mut jt, &Hit { kind: 0x4000, amount: -1, ..Default::default() }, &env, &mut ()));
    jt.jts = 0;
    assert!(dmg::FUN_001a19a0(&mut jt, &Hit { kind: 0x4000, amount: -100, ..Default::default() }, &env, &mut ()));
    assert_eq!((jt.dmg.hp, jt.jts, jt.dmg.state, jt.dmg.delta), (0, JTS_DMG, 0xb, dmg::LETHAL));
    // already dying: lethal hits are refused
    assert!(!dmg::FUN_001a19a0(&mut jt, &Hit { kind: 0x200, amount: -100, ..Default::default() }, &env, &mut ()));
    dmg::FUN_001a12f0(&mut jt, &env, &mut ());
    assert!(jt.dmg.ev.contains(&DmgEv::Die));
    assert_eq!(jt.respawn.map(|r| r.spawn), Some(0x189));
}
