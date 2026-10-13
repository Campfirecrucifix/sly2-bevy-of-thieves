//! WP-E physics: collision derived data, BSP segment queries, camera CmSegmentQuery.
//! WP-E2: point-in-BSP (0x00118fb0/0x00119090), point-cloud surfaces (0x00118d08), sphere prune (0x00118a88).
//! WP-E4: dynamics (driver, impulse, contact response, ground, platforms, camera callback 0x14f040).
//! WP-E7: driven xfs 0x22de88 kinds 4..0xb, parent frames 0x22fcc8, attachments 0x229798, game RNG (driven.rs); mesh narrowphase.
//! WP-E6: multi-body broadphase/islands/island solver (island.rs).
//! WP-E5: real Sly 2 driver chain 0x1f8dd8 (driver.rs), deep penetration 0x1a9108 (sphere.rs).
//! WP-E3: sphere contacts (0x001a93c8), bounded LCP (0x0011ad30), sphere sweep (0x0014eda0). SO/PO dynamics and broadphase are not ported yet (see ENGINE_PORT.md "E3:").
pub mod bsp;
pub mod driven;
pub mod driver;
pub mod dynamics;
pub mod geom;
pub mod island;
pub mod motor;
pub mod point;
pub mod query;
pub mod sphere;

pub use query::CollisionWorld;
