# Sly 2: Band of Thieves — PC port (Rust / Bevy) — EARLY ACCESS

> **⚠ Early Access / work in progress.** This is an unfinished recompilation. **Episode 1 is not fully
> playable yet**: levels load and you can move Sly around, but many objects, missions, cutscenes and
> menus are incomplete or missing. Expect bugs, crashes and visual glitches.

A fan-made PC port of the PlayStation 2 game *Sly 2: Band of Thieves* (NTSC-U, `SCUS_973.16`). The game
code is being recompiled function by function into Rust; rendering, audio and input run on
[Bevy](https://bevyengine.org/) 0.20.

**No game assets are included.** This repository contains only code. Every game file — levels, models,
textures, sounds, videos, scripts and text — is read at run time from **your own disc image**.

## Progress

**42.5 %** of the game's EE code ported (2174 functions ported/replaced + 234 partial of 6362, as of 2026-10-13;
partially ported functions count half).

**Playable: 34 %** of the Episode 1 release checklist passes (boot -> title -> New Game -> prologue works; the
Paris hub loads; missions, pickpocketing, respawn and save/load are still being finished). Expect bugs.

## What works

- Reads the original ISO directly (ISO 9660, level descriptors, LZ-compressed level streams); nothing is
  extracted to disk.
- Boot flow: intro movie (PSS/MPEG-2 playback), title screen, menus, save/load (memory card files on disk).
- All 43 level files load: geometry, textures/CLUTs, lighting, Sly's animations and movement, camera,
  collision, guards and many object classes, HUD, text/fonts, sound effects and music.
- The Splice script VM with its built-in functions drives level logic.

Not yet: complete missions/jobs, many object behaviours, several cutscenes and menus — see `recomp/` source
and its tests for the current scope.

## Features

- **Global ray-traced illumination** (Bevy Solari) as a normal graphics option; raster lighting is the
  default and always available. Requires a build with feature `rt` and a ray-tracing capable GPU.
- **Ultrawide up to 32:9**: Hor+ (constant vertical FOV, wider view, no stretching); HUD stays centred in a
  16:9 safe area.
- **Mods**: `mods/<name>/` folders overlay disc files, levels, textures and sounds by path, with a load order
  file — see [recomp/mods/README.md](recomp/mods/README.md).
- Keyboard + mouse and gamepads, rebindable (`controls.toml`).

## Requirements

- Rust (stable, edition 2024) with `cargo`.
- A disc image of *Sly 2: Band of Thieves* (USA, `SCUS-97316`) dumped from **your own** disc:
  `Sly 2 - Band of Thieves (USA).iso`.
- A GPU supported by Bevy/wgpu (Vulkan, DX12 or Metal). Ray tracing needs hardware RT support.

## Build

```sh
cd recomp
cargo build --release --features game        # raster only
cargo build --release --features game,rt     # + ray-traced global illumination option
```

The executable is `recomp/target/release/game(.exe)`.

## Run

```sh
cargo run --release --features game --bin game -- "path/to/Sly 2 - Band of Thieves (USA).iso"
```

ISO lookup order: command-line path, else `iso` in `settings.toml`, else
`Sly 2 - Band of Thieves (USA).iso` next to the executable.

Developer options: `--level <static-table row>` / `--table <level-table row>` start directly in a level;
`--width W --height H` window size; `--screenshot out.png --frames N` capture a frame and exit.

## Controls (defaults)

| PS2 | Keyboard / mouse | Gamepad |
|---|---|---|
| Left stick | W A S D | Left stick |
| Right stick (camera) | I J K L | Right stick |
| Cross (jump) | Space | South (A) |
| Circle | E | East (B) |
| Square (attack) | Q / left mouse | West (X) |
| Triangle | F | North (Y) |
| L1 / R1 | Left Shift / right mouse | LB / RB |
| L2 / R2 | Z / C | LT / RT |
| L3 / R3 | Left Ctrl / middle mouse | stick clicks |
| Start / Select | Esc or Enter / Tab | Start / Select |
| D-pad | arrow keys | D-pad |

Rebind in `controls.toml` next to the executable (written with the defaults on first run).

## Settings

`settings.toml` next to the executable (written on first run; see
[recomp/settings.example.toml](recomp/settings.example.toml)): ISO path, resolution, fullscreen, vsync,
Hor+, `global_illumination = "off" | "raytraced"`, ink outlines, volumes and the mod load order.
Most options are also in the in-game options menu.

## Tests

```sh
cd recomp
cargo test --release
```

Most tests read the game from `Sly 2 - Band of Thieves (USA).iso` in the repository root (next to
`recomp/`); put your ISO (or a link to it) there. The ISO is git-ignored.

## Contributing

Issues and pull requests are welcome. Ground rules:

- Port game code 1:1 (same behaviour and quirks), named after the original function and commented with
  its address (`// 0x001b3bd0`); platform code (GS/VU/DMA, IOP, PS2 SDK) is replaced by Bevy equivalents.
- Never commit game data: no ISO, ELF, extracted files, decompiled code or copyrighted screenshots.
- `cargo build --release --features game` and `cargo test --release` must pass. Do not run `cargo fmt`
  on the whole tree (generated tables are hand-formatted).

## License

Code is MIT licensed (see [LICENSE](LICENSE)). The license does not cover the game or its assets.

## Legal

This is an unofficial fan project, **not affiliated with or endorsed by Sucker Punch Productions or Sony
Interactive Entertainment**. *Sly Cooper* and *Sly 2: Band of Thieves* are trademarks of Sony Interactive
Entertainment. No copyrighted game data is distributed with this project; you must use a disc image you
dumped yourself from a legally owned copy of the game.
