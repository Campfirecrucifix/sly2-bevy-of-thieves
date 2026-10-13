# Publish notes (WP-PUB1, updated WP-PUB2) — review before making this repository public

Created from the development repo (`Sly 2/recomp`) on 2026-10-13 as a single commit with no history.

## Contents

- `recomp/src/` — full crate source (unchanged copy; re-synced from dev for WP-PUB2).
- `recomp/vendor/sly2_db/` — only `final_build.rs` of the dev analysis crate (graphics settings, Hor+, mod
  load order). The database / decompiler / extraction modules and their dependencies (rusqlite, regex, ...)
  are not published; `Cargo.lock` was pruned accordingly by cargo.
- `recomp/tests/` — 48 of the 128 dev test files: the ones that need only the user's ISO. Changes vs dev:
  hard-coded `C:/Users/...` ISO paths -> `<repo>/Sly 2 - Band of Thieves (USA).iso`; repo-root detection
  checks the ISO instead of `db/sly2.sqlite`; `engine_j15` writes its dumps to the OS temp dir;
  `engine_k20::levels()` (shared by d6, k21-k29, x3) enumerates levels from the boot level's level table
  + `level_check::check_level_row` instead of the DB (verified: same 43 LSNs as the DB's `disc_files`).
  Dropped: every test that opens `db/sly2.sqlite`, `extracted/`, `bin/` or checks committed screenshots.
- `recomp/mods/README.md` (= dev `MODDING.md`: overlay paths + load order), `recomp/settings.example.toml`.
- Not included: ISO/ELF/extracted files, `db/*.sqlite`, decompiled C, Ghidra projects, tools, screenshots,
  dev notes (ENGINE_PORT.md / PROGRESS.md / work-package briefs).

## Game-derived tables — none remain (WP-PUB2)

No table copied from the game's executable or disc is embedded in the source any more. The former
generated / hand-copied descriptor tables are read at run time from `SCUS_973.16` inside the user's ISO
(`src/engine/disc/exe.rs`; the game passes its configured ISO path, else `$SLY2_ISO` / the ISO next to the
executable / in a parent directory):

| Table (old location) | Now |
|---|---|
| `FIELDS` (world/fields_gen.rs), `FIELD_BINDINGS` / `FIELD_MEMBER` (game/fieldbind_gen.rs) | `world/field_table.rs`, `game/fieldbind.rs` (0x002ac880, 0x00279ad0) |
| `METHODS` / `BINDINGS` / `GETTERS` / `GETTER_BINDINGS` (game/methods_gen.rs) | `game/method_table.rs`; thunk arg kinds / callee / result setter from the thunk code (`disc/exe_scan.rs`) |
| `CLASSES` (world/classes_gen.rs) | `world/class_table.rs` (0x002baae8; vtable from the ctor code) |
| `JT_TABLE`, `JT_BIND_*`, `PO_BIND_*` (playerdraw), `DAT_002BAAF0` (player/p4), BIF `{fn, nargs, fRest}` (splice/bif) | read from SCUS_973.16 |
| SPU `PAN_TABLE` / `NOTE_TABLE` / `FINE_TABLE` (audio/spu_tables.rs) | read from libsd.irx / 989snd.irx embedded in SCUS_973.16 |
| k27 `SLOT4_OVERRIDES` (analysis-DB list) | vtable +0x2c read from SCUS_973.16 |

Identity with the old tables was proven entry for entry in the dev repo (test against the analysis DB /
old copies; not published). Remaining array literals > 64 entries: only the 169 BIF names (`splice/bif.rs`,
Splice built-in names; the executable has no names). Same scan method as before (top-level elements of
each `[...]` literal, comments/strings skipped).

## Source comments

Comments cite original addresses, `FUN_xxxxxxxx` names and short descriptions of the original logic (no
decompiled C bodies). Two comments mention dev-machine spec paths (`src/engine/player/p5_attach.rs:1`,
`p5_target.rs:4`, `C:/Temp/p5/...`); harmless, could be trimmed.

## Screenshots — kept OUT (candidates for docs/, ask the user)

Candidates from dev `recomp/shots/` (game art = copyrighted; include only a few small ones if wanted):
- `A14/theater_5120.png` (130 KB, 32:9) and `A14/theater_1920.png` (46 KB)
- `R2/t10_off.png` (375 KB) vs `R2/t10_raytraced.png` (1.8 MB) — raster vs ray-traced
- `C6/t20_raster_5120x1440.png` — 32:9 Hor+ in a level

## Before publishing

- `git push` is NOT done; no GitHub repository was created.
