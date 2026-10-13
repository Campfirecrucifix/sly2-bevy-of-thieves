//! SpliceHost: every BIF that touches the game world goes through here (one method per BIF,
//! named after the BIF). Defaults return `SpliceError::HostUnimplemented` — never a silent no-op.
//! WP-B..J implement these on their world/audio/camera resources.

use super::value::Ref;
use super::SpliceError;

macro_rules! host_bifs {
    ($( $idx:literal $name:ident $orig:literal ),* $(,)?) => {
        pub trait SpliceHost {
            /// Invokes a native CMethod (tag 0x22): `fn(out, ctx, argc, argv)` in the original.
            /// WP-J4: a wrapped host that serves every BIF/method this host does not override (layered
            /// hosts, e.g. game::vm_state::PersistentHost over game::splice_host::LevelSpliceHost).
            fn fallback(&mut self) -> Option<&mut dyn SpliceHost> {
                None
            }
            fn call_method(&mut self, method: &super::value::Method, args: &[Ref]) -> Result<Ref, SpliceError> {
                if let Some(f) = self.fallback() {
                    return f.call_method(method, args);
                }
                let _ = args;
                Err(SpliceError::HostUnimplemented(format!("method {}", method.method_id)))
            }
            $(
                #[doc = concat!("BIF ", stringify!($idx), " (", $orig, ")")]
                fn $name(&mut self, args: &[Ref]) -> Result<Ref, SpliceError> {
                    if let Some(f) = self.fallback() {
                        return f.$name(args);
                    }
                    Err(SpliceError::HostUnimplemented(stringify!($name).to_string()))
                }
            )*
        }

        /// Dispatches a host BIF by index; None if the index is not a host BIF.
        pub(crate) fn dispatch_host(host: &mut dyn SpliceHost, idx: usize, args: &[Ref]) -> Option<Result<Ref, SpliceError>> {
            match idx {
                $( $idx => Some(host.$name(args)), )*
                _ => None,
            }
        }

        /// Indices of BIFs routed to the host.
        pub const HOST_BIFS: &[usize] = &[$($idx),*];
    };
}

host_bifs! {
    96 ref_op_current_time "0x00123a08 reads DAT_002dded0 (float clock)",
    97 bif97_splice_bif_97 "0x00123a28 int from FUN_00210a88",
    98 bif98_splice_bif_98 "0x00123a60 int from FUN_00210a98",
    99 bif99_splice_bif_99 "0x00123a98 int from FUN_00210ae0",
    100 bif100_splice_bif_100 "0x00123ad0 object DAT_003e0f9c",
    101 bif101_splice_bif_101 "0x00123b00 object DAT_003e0fa0",
    102 bif102_splice_bif_102 "0x00123b30 object DAT_003e0fa4",
    103 bif103_splice_bif_103 "0x00123b60 float FUN_00154c90(0x2de578)",
    104 bif104_splice_bif_104 "0x00123ba0 int from FUN_00210b68",
    105 ref_op_schedule_callback "0x00123bd8",
    106 bif106_splice_bif_106 "0x00123dd8 FUN_00133130(0x3dd490, arg0) -> void",
    107 ref_op_defer_object_update "0x00123e18",
    108 ref_op_add_o "0x00123fb8 -> FUN_00123eb0 kind 0x6c",
    109 ref_op_ensure_o "0x00123fe8 -> FUN_00123eb0 kind 0x6d",
    110 ref_op_set_o "0x00124018 -> FUN_00123eb0 kind 0x6e",
    111 ref_op_get_o "0x00124048 -> FUN_00123eb0 kind 0x6f",
    112 bif112_splice_bif_112 "0x00124150",
    113 bif113_splice_bif_113 "0x001241e8",
    114 bif114_splice_bif_114 "0x001242b8",
    115 bif115_splice_bif_115 "0x001244e8",
    116 bif116_splice_bif_116 "0x00124578",
    117 ref_op_find_player_object "0x00124640",
    118 ref_op_find_world_object "0x00124690",
    119 ref_op_find_camera_object "0x001246e0",
    120 ref_op_find_class_objects "0x00124730",
    121 ref_op_find_objects_in_bounding_box "0x001247d8",
    122 ref_op_find_objects_in_bounding_sphere "0x001248b0",
    123 ref_op_hit_test_objects "0x00124f00",
    124 ref_op_hit_test_objects_first "0x00124f80",
    125 ref_op_convert_object_position "0x00125000",
    126 ref_op_convert_object_vector "0x00125060",
    127 ref_op_convert_object_matrix "0x001250d8",
    128 ref_op_near_clip_center "0x00125168",
    129 ref_op_start_sound "0x001251e8",
    130 bif130_splice_bif_130 "0x00125248",
    131 ref_op_stop_sound "0x00125398",
    132 ref_op_start_rumble "0x00125450",
    133 bif133_splice_bif_133 "0x00125510",
    135 bif135_splice_bif_135 "0x001255f0",
    136 bif136_splice_bif_136 "0x00125640",
    137 bif137_splice_bif_137 "0x00125690",
    138 bif138_splice_bif_138 "0x001256e0",
    139 ref_op_emit_smoke_cloud "0x00125738",
}

/// A host that implements nothing (every world BIF errors). Useful for pure-script tests.
#[derive(Debug, Default)]
pub struct NullHost;
impl SpliceHost for NullHost {}
