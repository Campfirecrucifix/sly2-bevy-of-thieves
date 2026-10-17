# Sly 2: Bevy of Thieves — modding

Mods are folders under `mods/` next to `game.exe`. Nothing is unpacked or patched on disk: the game reads the
original ISO and, for every file/level/texture/sound it loads, first asks the mod overlay
(`engine::disc::mods::ModOverlay`) whether a mod provides a replacement. No mods = byte-identical disc data.

```
game.exe
settings.toml
mods/
  load_order.txt            optional
  my_mod/
    SYSTEM.CNF              ISO file override (any ISO path, e.g. IRX/FOO.IRX)
    levels/0350144.bin      level override (decompressed level bytes)
    textures/350144/1628.png  texture override
    sounds/0350720/12.wav   sound override
```

## Load order

1. `settings.toml` `[mods] order = ["a", "b"]` if non-empty (only these mods, in this order; unknown names are logged
   and skipped).
2. Otherwise `mods/load_order.txt`: one mod folder name per line (`#` comments, blank lines ignored). Listed mods
   load in that order; mod folders not listed are appended afterwards, alphabetically.
3. No `load_order.txt`: all mod folders, alphabetically.

**Later wins**: when several mods provide the same path, the last one in the order is used. Overrides are per file
(per texture / per sound), never merged inside a file.

## Paths

| What | Path inside a mod | Notes |
|---|---|---|
| ISO file | same path as on the disc (`SCUS_973.16`, `IRX/X.IRX`) | `cdrom0:\` prefix and `;1` version are ignored |
| Level | `levels/<lsn, 7 digits>.bin` | the full **decompressed** level stream (not LZ). `lsn` = the level descriptor's start sector (`Descriptor::lsn`). It is loaded through the plain stream path and parsed exactly like a disc level (directory, objects, scripts, textures, sounds) |
| Texture | `textures/<level lsn>/<TEX id>.png` | level lsn **not** zero-padded. TEX id = the shader's first frame id (`gfx::texture::texture_override_path`). Any size, RGBA PNG; replaces the decoded base texture of every material using that shader in that level; the shader's wrap/clamp is kept and the material is treated as alpha-capable |
| Sound | `sounds/<bank lsn, 7 digits>/<sound index>.wav` | bank lsn = the level's 989snd bank descriptor (FUN_001fd0e8 block +0xa8, `LevelGame::bank`), sound index = the bank sound number used by Splice `StartSound` / emitters. PCM WAV, 8 or 16 bit, mono or stereo, any rate |

Finding ids: the `game.exe` log prints the level lsn on load (`level transition: loading lsn …`); the bank lsn is
`audio::level_sound_files(&header.block_3dd040)` (kind `Bank`); `tests/engine_r.rs` shows how to enumerate TEX ids (`LevelScene::textures[..].shaders[..].frames[0].id`)
and bank sounds.

### Sounds in detail

At bank load (`disc::mods_sound::apply_sound_mods_system`, after `poll_level_audio`) each override WAV is mixed to
mono, resampled to the tone's playback rate at the default note (so it plays at its own pitch), encoded to SPU2
ADPCM (one-shot, no loop) and appended to the bank's sample memory; every tone grain of that sound points at it.
The original sound's volume, pan, voice group and priority are kept. Sounds without a tone grain cannot be
replaced. Looping sounds become one-shot.

### Levels in detail

A level override replaces the whole level (geometry, objects, textures, scripts, sound descriptors). It is the way to
ship changed Splice scripts: the level script blocks (FUN_001ff858 per directory entry) are part of the level stream.
Easiest workflow: take a decompressed level (`extracted/levels/<lsn>.bin`, produced by `tools/extract_assets.py`),
edit, save as `mods/<mod>/levels/<lsn>.bin`.

## Splice scripts: BIF ABI

Scripts call engine built-ins (BIFs) **by index** (tag `0x19` Bifk + index); names are not stored in the binary.
The index table is the original's (`.data 0x002bbb90`, 169 entries `{fn, nargs, fRest}`), mirrored 1:1 by
`engine::splice::bif::BIF_TABLE` (index = BIFK, name, original address, arg count, rest flag). This numbering is a
**stable ABI**: indices never move; new port-only built-ins would be appended after 168. Argument rules:

- `nargs` fixed args; with `fRest` the remaining args arrive as ONE list argument; max 32 args.
- Values are tagged (`splice::value`): S32 0x15, F32 0x16, Bool 0x17, Symid 0x18, Bifk 0x19, Vector 0x1a, Matrix 0x1b,
  Pair 0x1f, Proc 0x20, Basic (engine object) 0x21, Method 0x22, Void 0x24, Nil -1.
- Original quirks are kept (see ENGINE_PORT.md Findings F/F2/J2/J3): e.g. `if` is true only for payload 1, `while`
  evaluates only its first body expression, BIFs 140-168 are compiled-out no-ops returning Nil.

The full list with names/argument counts: `db/sly2.sqlite` table `splice_bifs`, or `BIF_TABLE` in
`recomp/src/engine/splice/bif.rs`.

## Graphics options relevant to mods

`settings.toml [video] global_illumination = "off" | "raytraced"`: with "raytraced" (build with feature `rt`, GPU
with ray queries) every static opaque level mesh is also traced by Bevy Solari, so replacement textures light the
scene indirectly. Vertex colours (the original baked lighting) only modulate directly visible surfaces.

## Tests

`recomp/tests/engine_r.rs` generates fake mods in a temp dir (one texture, one sound in two competing mods, one level)
and checks that the override is used, that the load order decides, and that everything else stays vanilla.
