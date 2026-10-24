//! CRAWL: script-side LO remove / add (method 0x100b88 vtable +0x4c FUN_001a5630 / 0x100a90 +0x44
//! FUN_001a5478) reflected in the class host: +0x28 bit 60, the object's collision parts leave / rejoin the SO
//! set, and a removed object is no longer updated (the R7 cellar laser 136591 / vent blocker 136587).
//!
//! Original chain (EMU4 020 -> 021, t 323.64): Sly knocks out the vent guard (class 58, oid 11539) -> brain goal
//! 7 -> FUN_00209890 -> guard vt+0x1c message 7 {7, -1} -> FUN_001331f8 -> level script entry 128 slot 3:
//! `(case arg ((7) (schedule 0.0 (lambda () ((get-o (find 136591) 40)) ((get-o (find 136587) 40))))))`, sym 40 =
//! method 0x100b88 (LO remove).
//! REPLACED: FUN_001a5630's world-list unlink is the `lo_removed` set (class update loops test it); the
//! collision parts of the object and its child subtree leave / rejoin the SO set.
use super::plugin::EngineHost;
use crate::engine::world::objects::{LoadData, Payload};

impl EngineHost {
    pub fn lo_world_link(&mut self, obj: usize, on: bool) {
        if self.lo_world_link_parts(obj, on) {
            self.rebuild_so();
        }
    }

    fn rebuild_so(&mut self) {
        let Some(w) = self.k7_cache() else { return };
        self.so = Some(std::sync::Arc::new(crate::engine::physics::query::CollisionWorld::build_so_masked(&w, &self.so_parts)));
        self.so_gen = self.so_gen.wrapping_add(1);
    }

    /// FEEL R32: bring the class host (update list `lo_removed`, SO collision parts, +0x2c bit 28) in line with
    /// the scripts' LO +0x28 bit 60 state, e.g. the job toggles 0x1620b0 (`job_flow::apply_toggles`) that only
    /// reach `ObjectFields`. One SO rebuild for the whole change set.
    pub fn sync_in_world(&mut self, objs: &crate::engine::game::fields::ObjectFields) {
        // a new world resets `oow` (k7_cache) before the diff
        let Some(w) = self.k7_cache() else { return };
        // no script Add / Remove yet: the load-time state (once per world)
        let load;
        let new = match objs.out_of_world.as_ref() {
            Some(s) => s,
            None if self.oow.is_some() => return,
            None => {
                load = (0..w.objects.len()).filter(|&i| crate::engine::game::fields::out_of_world_at_load(&w, i)).map(|i| i as u32).collect();
                &load
            }
        };
        // first sync of a world: the SO was built with every object linked; the load-time field-26 Removes
        // (FUN_001a5d08 -> vtable +0x4c, `out_of_world_at_load`) unlink theirs here like any other Remove
        let old = self.oow.get_or_insert_with(Default::default);
        if old == new {
            return;
        }
        let mut gone: Vec<u32> = new.difference(old).copied().collect();
        let mut back: Vec<u32> = old.difference(new).copied().collect();
        gone.sort_unstable();
        back.sort_unstable();
        self.oow = Some(new.clone());
        let mut changed = false;
        for o in gone {
            changed |= self.lo_world_link_parts(o as usize, false);
        }
        for o in back {
            changed |= self.lo_world_link_parts(o as usize, true);
        }
        if changed {
            self.rebuild_so();
        }
    }

    fn lo_world_link_parts(&mut self, obj: usize, on: bool) -> bool {
        if on {
            self.laser_off.retain(|&o| o != obj);
        } else if !self.laser_off.contains(&obj) {
            self.laser_off.push(obj);
        }
        let h = obj as u32 + 1;
        if let Some(w) = self.world.clone() {
            self.k25_fld.set_world(&w);
        }
        let hi = self.k25_fld.get(h, 0x2c).unwrap_or(0);
        self.k25_fld.set(h, 0x2c, if on { hi | 1 << 28 } else { hi & !(1 << 28) });
        let Some(w) = self.k7_cache() else { return false };
        let mut changed = false;
        // FUN_001a5630 / 1a5478 walk the object's child subtree (as `ObjectFields::set_in_world`)
        for o in crate::engine::playerdraw::subtree(&w, obj) {
            // POOL1 R95: a clone (Payload::Clone) collides with its source's parts (build_so_masked, CRAWL R62):
            // unlink those too (an override-row field-26 Remove of a cloned copy kept colliding)
            let src = match w.objects.get(o) {
                Some(x) if matches!(x.payload, Payload::Clone) && std::env::var_os("SLY2_R95_NO_CLONEPARTS").is_none() => x.clone_of.filter(|&c| c != o).unwrap_or(o),
                _ => o,
            };
            let Some(Payload::Loaded(LoadData::Alo { alo, .. })) = w.objects.get(src).map(|x| &x.payload) else { continue };
            for (k, p) in alo.parts.iter().enumerate() {
                let want = on && p.b2 & 1 == 0;
                let cur = self.so_parts.get(&(o, k)).copied().unwrap_or(p.b2 & 1 == 0);
                if cur != want {
                    self.so_parts.insert((o, k), want);
                    changed = true;
                }
            }
        }
        changed
    }
}
