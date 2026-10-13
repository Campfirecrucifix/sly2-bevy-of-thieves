//! `SpliceHost` for the running game: the BIFs that now have a backing in the port.
//! Everything else keeps the trait default `SpliceError::HostUnimplemented`.
//!
//! What the BIF bodies really do in Sly 2 (the Sly 1 names in `splice_bifs` are misleading for 131/132):
//!  - 96  RefOpCurrentTime 0x00123a08: F32 of DAT_002dded0 (clock t).
//!  - 119 RefOpFindCameraObject 0x001246e0 / 128 RefOpNearClipCenter 0x00125168: WP-I `camera::bifs`.
//!  - 129 RefOpStartSound 0x001251e8: new sound emitter (FUN_001fbb90) + 2D flag (FUN_001fb2f8) -> Basic ref.
//!  - 131 (named StopSound) 0x00125398: list -> u16 path (FUN_001252d0) -> FUN_001d7ed8 save-variable READ.
//!  - 132 (named StartRumble) 0x00125450: same path + FUN_001d80c8 save-variable WRITE (argv[1] = value);
//!    returns the Void CRef. Rumble itself is started by script through this write (e.g. the vibration
//!    variables) and by game code via FUN_00187e28 — see `RumbleRequest`.

use crate::engine::audio::emitter::{list_to_path_1252d0, EmitterSystem};
use crate::engine::camera::Cm;
use crate::engine::game::save::{GameData, SaveValue, SaveVars};
use crate::engine::splice::value::{list_items, Ref};
use crate::engine::splice::{SpliceError, SpliceHost};

/// Tag 0x21 payloads the host hands out: emitter ids are offset so they never collide with object ids.
pub const BASIC_EMITTER: u32 = 0x4000_0000;

/// A rumble the game asked for (FUN_00187e28(pad, pattern id, duration)); drained by the input backend.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RumbleRequest {
    pub pattern: usize,
    pub duration: f32,
}

/// Borrowed engine state for one script evaluation.
pub struct GameSpliceHost<'a> {
    /// DAT_002dded0
    pub t: f32,
    pub cm: Option<&'a Cm>,
    pub emitters: &'a mut EmitterSystem,
    pub vars: Option<&'a SaveVars>,
    pub data: &'a mut GameData,
}

fn save_to_ref(v: SaveValue) -> Ref {
    match v {
        SaveValue::Nil => Ref::Nil,
        SaveValue::Int(i) => Ref::S32(i),
        SaveValue::Float(f) => Ref::F32(f),
        SaveValue::Bool(b) => Ref::Bool(b),
        // kind 5 = vector (x,y,z,w); kind 6 = CLQ (3 floats)
        SaveValue::Quad(5, q) => Ref::Vector([f32::from_bits(q[0]), f32::from_bits(q[1]), f32::from_bits(q[2])]),
        SaveValue::Quad(_, q) => Ref::Clq([f32::from_bits(q[0]), f32::from_bits(q[1]), f32::from_bits(q[2])]),
        // kinds 7/8 (8/12-byte records) have no CRef counterpart we know yet
        SaveValue::Pair(_) | SaveValue::Triple(_) => Ref::Nil,
    }
}

fn ref_to_save(r: &Ref) -> SaveValue {
    // FUN_001d80c8 stores CRef+4 blindly (payload word); vectors/CLQs copy the 16-byte payload
    match r {
        Ref::S32(i) => SaveValue::Int(*i),
        Ref::F32(f) => SaveValue::Float(*f),
        Ref::Bool(b) => SaveValue::Bool(*b),
        Ref::Vector(v) => SaveValue::Quad(5, [v[0].to_bits(), v[1].to_bits(), v[2].to_bits(), 0]),
        Ref::Clq(v) => SaveValue::Quad(6, [v[0].to_bits(), v[1].to_bits(), v[2].to_bits(), 0]),
        other => SaveValue::Int(other.payload() as i32),
    }
}

/// argv[0] list -> FUN_001252d0 path.
fn path_of(r: Option<&Ref>) -> Vec<u16> {
    let items = r.and_then(|r| r.pair()).map(|p| list_items(Some(p))).unwrap_or_default();
    let tagged: Vec<(i32, u32)> = items.iter().map(|x| (x.tagk(), x.payload())).collect();
    list_to_path_1252d0(&tagged)
}

impl SpliceHost for GameSpliceHost<'_> {
    fn ref_op_current_time(&mut self, _args: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00123a08
        Ok(Ref::F32(self.t))
    }

    fn ref_op_find_camera_object(&mut self, _args: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x001246e0: Basic ref to g_cm (WP-I camera::bifs)
        Ok(Ref::Basic(crate::engine::camera::bifs::RefOpFindCameraObject()))
    }

    fn ref_op_near_clip_center(&mut self, _args: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00125168 (WP-I camera::bifs)
        let Some(cm) = self.cm else { return Err(SpliceError::HostUnimplemented("near_clip_center without camera".into())) };
        Ok(Ref::Vector(crate::engine::camera::bifs::RefOpNearClipCenter(cm)))
    }

    fn ref_op_start_sound(&mut self, args: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x001251e8: argv[0]+4 sound, argv[1]+4 flag (payloads, no type check)
        let sound = args.first().map(|a| a.payload() as i32).unwrap_or(0);
        let flag = args.get(1).map(|a| a.payload() as i32).unwrap_or(0);
        let id = self.emitters.bif_start_sound_1251e8(sound, flag);
        Ok(Ref::Basic(BASIC_EMITTER | id as u32))
    }

    fn ref_op_stop_sound(&mut self, args: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00125398 (save-variable read)
        let Some(vars) = self.vars else { return Ok(Ref::Nil) };
        Ok(save_to_ref(vars.ee_sub_1d7ed8(&path_of(args.first()), self.data)))
    }

    fn ref_op_start_rumble(&mut self, args: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00125450 (save-variable write; result Void)
        if let (Some(vars), Some(v)) = (self.vars, args.get(1)) {
            vars.ee_sub_1d80c8(&path_of(args.first()), ref_to_save(v), self.data);
        }
        Ok(Ref::Void)
    }
}

// ---------------------------------------------------------------- WP-J3: level host (running evaluation)

use crate::engine::game::spawn::SpawnAliases;
use crate::engine::splice::value::Method;

/// Basic handles of the engine globals some BIFs return (FUN_00129a10(ref, &global); payload = address,
/// as `camera::bifs::RefOpFindCameraObject` does for g_cm).
pub const BASIC_PLAYER_GLOBAL: u32 = 0x003d_d490; // BIF 117 0x00124640
pub const BASIC_WORLD_GLOBAL: u32 = 0x003d_d040; // BIF 118 0x00124690
pub const BASIC_137_GLOBAL: u32 = 0x003d_a0c0; // BIF 137 0x00125690

/// CMethod ids (= original function address) of the spawn alias-list methods (WP-J2 finding).
pub const METHOD_ADD_ALIAS_189: u32 = 0x0010_b8f8;
pub const METHOD_ADD_ALIAS_18A: u32 = 0x0010_b970;
pub const METHOD_ADD_ALIAS_ARG: u32 = 0x0010_b9e8;

/// CMethod ref bound to object index `object` (the method table that hands these to scripts is not
/// identified yet; the host binds them explicitly). nargs = FUN_00201360 second argument.
pub fn method_ref(object: u32, method_id: u32) -> Option<Ref> {
    let nargs = match method_id {
        METHOD_ADD_ALIAS_189 | METHOD_ADD_ALIAS_18A => 0,
        METHOD_ADD_ALIAS_ARG => 1,
        _ => return None,
    };
    Some(Ref::Method(std::rc::Rc::new(Method { object, method_id, nargs })))
}

/// Host for scripts evaluated inside the running game (level script blocks, FUN_001ffa70): the J2
/// `GameSpliceHost` BIFs plus mission, globals, rumble (BIF 130), bit ops, and the alias methods.
pub struct LevelSpliceHost<'a> {
    pub base: GameSpliceHost<'a>,
    /// DAT_003e1110 (FUN_00210a88), -1 = none
    pub mission: i32,
    pub aliases: &'a mut SpawnAliases,
    /// FUN_00187e28 requests (drained into `app::RumbleQueue`)
    pub rumble: &'a mut Vec<RumbleRequest>,
}

impl SpliceHost for LevelSpliceHost<'_> {
    fn call_method(&mut self, m: &Method, args: &[Ref]) -> Result<Ref, SpliceError> {
        let obj = m.object as usize;
        match m.method_id {
            // 0x0010b8f8: FUN_002289b0 = FUN_00228988(obj, 0x189); returns Void
            METHOD_ADD_ALIAS_189 => self.aliases.ee_sub_228988(obj, 0x189),
            // 0x0010b970: FUN_002289d0 = FUN_00228988(obj, 0x18a)
            METHOD_ADD_ALIAS_18A => self.aliases.ee_sub_228988(obj, 0x18a),
            // 0x0010b9e8: FUN_00201360(.., 1 arg, kind 6) -> FUN_00228988(obj, arg)
            METHOD_ADD_ALIAS_ARG => self.aliases.ee_sub_228988(obj, args.first().map(|a| a.payload() as i32).ok_or(SpliceError::Arity)?),
            id => return Err(SpliceError::HostUnimplemented(format!("method {id:#x}"))),
        }
        Ok(Ref::Void)
    }
    fn ref_op_current_time(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        self.base.ref_op_current_time(a)
    }
    fn ref_op_find_camera_object(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        self.base.ref_op_find_camera_object(a)
    }
    fn ref_op_near_clip_center(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        self.base.ref_op_near_clip_center(a)
    }
    fn ref_op_start_sound(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        self.base.ref_op_start_sound(a)
    }
    fn ref_op_stop_sound(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        self.base.ref_op_stop_sound(a)
    }
    fn ref_op_start_rumble(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        self.base.ref_op_start_rumble(a)
    }
    fn bif97_splice_bif_97(&mut self, _: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00123a28: S32 FUN_00210a88() = DAT_003e1110 (mission)
        Ok(Ref::S32(self.mission))
    }
    fn bif104_splice_bif_104(&mut self, _: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00123ba0: S32 FUN_00210b68() = constant 0 (stubbed in the shipped ELF)
        Ok(Ref::S32(0))
    }
    fn ref_op_find_player_object(&mut self, _: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00124640: Basic of 0x3dd490
        Ok(Ref::Basic(BASIC_PLAYER_GLOBAL))
    }
    fn ref_op_find_world_object(&mut self, _: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00124690: Basic of 0x3dd040
        Ok(Ref::Basic(BASIC_WORLD_GLOBAL))
    }
    fn bif137_splice_bif_137(&mut self, _: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00125690: Basic of 0x3da0c0
        Ok(Ref::Basic(BASIC_137_GLOBAL))
    }
    fn bif130_splice_bif_130(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00125248: FUN_00187e28(duration = (float)argv[1] (FUN_00129da8), &g_pad 0x2dfc00, id = argv[0]+4);
        // returns Void
        let id = a.first().map(|r| r.payload() as usize).unwrap_or(0);
        let duration = a.get(1).map(|r| r.as_f32()).unwrap_or(0.0);
        self.rumble.push(RumbleRequest { pattern: id, duration });
        Ok(Ref::Void)
    }
    fn bif135_splice_bif_135(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x001255f0: S32 AND of every list element's payload (start 0xffffffff), no type check
        Ok(Ref::S32(rest_payloads(a).fold(u32::MAX, |x, p| x & p) as i32))
    }
    fn bif136_splice_bif_136(&mut self, a: &[Ref]) -> Result<Ref, SpliceError> {
        // 0x00125640: S32 OR of every list element's payload (start 0)
        Ok(Ref::S32(rest_payloads(a).fold(0, |x, p| x | p) as i32))
    }
}

/// Rest-argument list (argv[0]) payloads, in order.
fn rest_payloads(a: &[Ref]) -> impl Iterator<Item = u32> {
    a.first().and_then(|r| r.pair()).map(|p| list_items(Some(p))).unwrap_or_default().into_iter().map(|r| r.payload())
}

/// Original game-code callers of FUN_00187e28 (duration f32 in $f12, &g_pad, pattern id); these feed
/// `app::RumbleQueue` once their owning systems are ported (BIF 130 above is live):
///  0x00131290 case 3 (event list: id = +8, duration = +0xc); 0x001e1e30 case 0x33 (piVar[5] id, piVar[4] dur);
///  0x001bb648 case 9 (0.3 s, id puVar[1], when +0x168 & 4); 0x00174bd0 (1.0 s, id 8, player hit);
///  0x00179328 / 0x0017b1a8 (0.15 s = 0x3e19999a, id 3, player contact).
pub fn rumble_174bd0() -> RumbleRequest {
    RumbleRequest { pattern: 8, duration: 1.0 }
}
pub fn rumble_179328() -> RumbleRequest {
    RumbleRequest { pattern: 3, duration: 0.15 }
}
pub fn rumble_1bb648(id: usize) -> RumbleRequest {
    RumbleRequest { pattern: id, duration: 0.3 }
}
