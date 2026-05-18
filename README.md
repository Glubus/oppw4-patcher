# OPPW4 Patcher RS

Clean Rust rewrite of the One Piece: Pirate Warriors 4 `dinput8.dll` mod
loader and its RDB tooling.

The original community DLL works by pretending to be DirectInput, loading the
real system `dinput8.dll`, then virtualizing selected Windows file reads made by
the game. This project keeps that proven idea, but now treats `dinput8.dll` as a
small modloader host: it owns the shared hooks, logs, Lua runtime, and plugin
ABI, while game-specific features live in official or third-party plugins.

## Why This Exists

The old patcher taught us the important trick: OPPW4 does not need archive files
to be rebuilt if the loader can intercept the right RDB index and payload reads.
That makes fast modding possible, but the original patcher is not a healthy
foundation for long-term work: its source code is not available to this project,
it has effectively gone years without meaningful updates, and every fix requires
reverse engineering the shipped DLL.

The goal of this rewrite is not to erase what the original patcher did well. It
is to preserve the working model while turning it into something we can inspect,
test, extend, and ship without guessing at opaque binary behavior every time a
new issue appears.

The original DLL is also hard to reason about once you get past the high-level
idea: large functions, implicit global state, unclear virtual/internal/external
decisions, and runtime files mixed directly into the patcher folder.

This rewrite aims to make the loader boring in the best way:

- one responsibility per module;
- tested catalog, RDB, zip, and virtual file behavior;
- explicit runtime folders under the game install;
- a compressed name/hash catalog embedded into the DLL;
- logs written through `tracing`;
- reverse-engineering notes kept next to the code.

## How It Works

At runtime, OPPW4 loads `dinput8.dll` from the game directory. This proxy DLL:

1. Initializes host logs under `OPPW4/mods/_oppw4/logs/`.
2. Installs shared game hooks such as file API hooks and active-character state.
3. Creates the host config if `OPPW4/mods/_oppw4/config.toml` is missing.
4. Scans `OPPW4/plugins/<plugin_id>/plugin.toml`.
5. Loads each plugin DLL through the shared `plugin-sdk` ABI.
6. Exposes root `OPPW4/mods/` Lua mods and legacy zip/folder mods to plugins.
7. Routes plugin logs to `OPPW4/plugins/<plugin_id>/logs/`.
8. Forwards DirectInput calls to the real system `dinput8.dll`.

The important design choice is that mods are runtime data, not source files.
This repository contains code, docs, and embedded resources. The game install
contains mods, logs, and release DLLs.

## Runtime Layout

Install the built DLL next to the OPPW4 executable:

```text
OPPW4/
  dinput8.dll
  plugins/
    skin_patcher/
      plugin.toml
      skin_patcher.dll
      logs/
    fx_director/
      plugin.toml
      fx_director.dll
      config.toml
      logs/
  File/
    CMN/
      AssetRelease/
        Retail/
          CharacterEditor.rdb
          CharacterEditor.rdb.bin
          ScreenLayout.rdb
          ScreenLayout.rdb.bin
          ...
  mods/
    LawDressrosa/
      ScreenLayout/
        800_294_face_law_dressrosa_External_00.g1t
    AcePack/
      CharacterEditor/
        MPLC009_Ace.g1m
      MaterialEditor/
        MPR_Bound_Character_MPLC009Ace_skin00_kidsalb.g1t
    legacy/
      CharacterEditor/
        0x3b359352.g1m
    law-pack.zip
    _oppw4/
      config.toml
      disabled_hashes.txt
      name_hash_catalog.txt
      logs/
        oppw4_proxy.log
```

`mods/_oppw4/` is reserved for loader configuration and logs. It is ignored when
the loader searches for mod zip files.

## Plugin Layout

Plugins live under `OPPW4/plugins/<plugin_id>/`.

```text
OPPW4/plugins/fx_director/
  plugin.toml
  fx_director.dll
  config.toml
  logs/
```

`plugin.toml` is intentionally small:

```toml
[plugin]
id = "fx_director"
version = "0.2.0"
entry = "fx_director.dll"
```

The `entry` value must be a file name only, not a path. If `plugin.toml` is
missing and the folder contains exactly one DLL, the host creates a default
manifest automatically. If several DLLs exist, the plugin is skipped until the
manifest is explicit.

The shared plugin SDK lives in:

```text
crates/plugins/sdk/
```

Official plugins live in:

```text
plugins/
  skin_patcher/
  fx_director/
```

## Mod Formats

Preferred loose mods use a named mod folder, then the archive name:

```text
OPPW4/mods/<ModName>/<ArchiveName>/<asset>
```

Examples:

```text
OPPW4/mods/AcePack/CharacterEditor/MPLC009_Ace.g1m
OPPW4/mods/AcePack/MaterialEditor/MPR_Bound_Character_MPLC009Ace_skin00_kidsalb.g1t
OPPW4/mods/LawDressrosa/ScreenLayout/800_294_face_law_dressrosa_External_00.g1t
```

If you want to keep old loose archive folders without reorganizing every file,
put them under a named folder such as `legacy`:

```text
OPPW4/mods/legacy/<ArchiveName>/<asset>
```

Zip files can be placed anywhere under `OPPW4/mods/`, except under
`OPPW4/mods/_oppw4/`. Zip entries can be either direct:

```text
ScreenLayout/800_294_face_law_dressrosa_External_00.g1t
```

or nested under a mod name:

```text
LawDressrosa/ScreenLayout/800_294_face_law_dressrosa_External_00.g1t
```

Loose files currently override same-named zip entries for the same archive.

## Name/Hash Catalog

The loader needs a filename-to-hash catalog because many RDB entries only expose
hashed asset names. The source catalog is stored at:

```text
resources/name_hash_catalog.txt
```

The release DLL embeds:

```text
resources/name_hash_catalog.zip
```

That keeps the DLL self-contained while reducing binary size. For debugging, the
embedded catalog can be overridden with:

```text
OPPW4/mods/_oppw4/name_hash_catalog.txt
```

## Configuration

Optional disabled hashes can be listed here:

```text
OPPW4/mods/_oppw4/disabled_hashes.txt
```

Use one hash per line. Hex forms such as `0xa46e63e6` are accepted.

## Repository Layout

```text
apps/
  dinput8-proxy/      # DLL loaded by the game
  rdb-tool/           # command-line probes and catalog tooling
crates/
  asm/                # low-level x64 patch byte emission helpers
  hooks/              # shared host hooks and process memory helpers
  lua-api/            # Lua mod discovery/runtime helpers
  plugins/sdk/        # plugin ABI, host loader, and plugin-facing API
  rdb/                # RDB/catalog parsing
  struct-api/         # editable structured game data
plugins/
  skin_patcher/       # official RDB/skin replacement plugin
  fx_director/        # official runtime FX plugin
resources/
  name_hash_catalog.txt
  name_hash_catalog.zip
```

See `docs/collaborations_rules.md` for the rules used while editing this repo.

## Build And Test

Run the full test suite:

```powershell
cargo test --workspace
```

Build the release DLL:

```powershell
cargo build --release -p oppw4-dinput8-proxy
```

The DLL output is:

```text
target/release/dinput8.dll
```

Copy it to the OPPW4 install directory as `dinput8.dll`.

## Current Compatibility

The new loader reads the clean runtime layout under:

```text
OPPW4/mods/
```

It does not currently scan the legacy folder:

```text
OPPW4/OPPW4_PATCHER/
```

Legacy loose mods can usually be moved from:

```text
OPPW4/OPPW4_PATCHER/<ArchiveName>/<asset>
```

to:

```text
OPPW4/mods/legacy/<ArchiveName>/<asset>
```

`legacy` is not special to the loader. It is just a normal mod folder name that
makes migration easy while still using the clean
`OPPW4/mods/<ModName>/<ArchiveName>/<asset>` layout.

The original DLL behavior is documented under `docs/`, especially the Ghidra map
and RDB virtualization notes. Keep new compatibility decisions documented there
or in this README as they become real.
