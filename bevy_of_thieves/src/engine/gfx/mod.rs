//! WP-C: level geometry, textures and materials -> Bevy (Solari-compatible).
//! texture.rs: CLUT / bitmap / shader tables (FUN_001eb748 chain) -> RGBA8.
//! material.rs: FUN_001eb960 GS base-pass state -> alpha / sidedness.
//! mesh.rs: WP-B3 mesh arrays -> triangle lists (VU1 strip/ADC semantics), normals, tangents;
//! C4: mesh types 1 (ink edges) / 2, 3 (shadow volumes), VU1 silhouette rule.
//! scene.rs: ALO joints / instances / owners -> placed primitives.
//! attach.rs (C3): FUN_0022d6b0 bone refs (owner-bone attachments), bone world matrices.
//! light.rs (C3): LIGHT class 4 params (FUN_001a3968) -> Bevy lights; C4: setters, lux unit.
//! skin.rs (C5): weight_refs -> joints / inverse bind poses; ink.rs (C5): type-1 silhouette lines.
//! game.rs (feature `game`): Bevy Mesh / Image / StandardMaterial + spawn system.

pub mod attach;
pub mod fog;
// WP-A6: effect billboards (FUN_001ce888 sparkle / collect, FUN_00156698 hit)
pub mod fx;
pub mod light;
pub mod material;
pub mod mesh;
// WP-A13: class-28 particle quads (FUN_0013a1e8 slot 18)
pub mod particles;
pub mod scene;
pub mod ink;
pub mod skin;
pub mod texture;
// WP-R: ray-traced GI (Solari, feature `rt`) mesh checks + plugin; 32:9 view checks + HUD safe rect.
pub mod rt;
// WP-C8: screen wipe / fade state machine (0x443740), class-97 sky follow
pub mod screen;
pub mod ultrawide;

#[cfg(feature = "game")]
pub mod game;
// WP-C7: kind-3 env-only shaders as light volumes (DATE mask -> back faces, inverted depth test)
#[cfg(feature = "game")]
pub mod envvol;
// WP-A10: screen-pixel -> level prim debug pick (SLY_A10_PICK)
#[cfg(feature = "game")]
pub mod pick;
