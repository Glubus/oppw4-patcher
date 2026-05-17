# Changelog

## 0.2.0 - Modloader architecture

This release turns the old patcher into a plugin-based OPPW4 modloader foundation.

### Added

- Root `plugins/` layout with per-plugin `plugin.toml`, DLL entry, and `logs/`.
- `plugin-host` loader used by `dinput8.dll`.
- Shared `plugin-api` ABI for official plugins.
- Lua mod support from `mods/`, including directory mods and zip mods.
- Directory Lua hot reload for live mod iteration.
- `require(...)`-based Lua module imports.
- `character` Lua API with `find`, `unsafe_find`, `new`, `all`, and `local_player`.
- Host active-character service exposed to plugins.
- Official `fx_director` plugin for runtime effect activation experiments.
- `fx_director` Lua API:
  - `character.find("zoro"):add_fx(...)`
  - `fx_director.cycle(...)`
  - cycle mode `after_animation`
- Effect ID observation diagnostics behind debug config.
- Structured character data in editable JSON.
- Missing host/plugin TOML configs are created automatically without overwriting existing files.

### Changed

- `skin_patcher` is now an official plugin instead of hardwired dinput logic.
- Legacy RDB parsing remains available through the renamed `rdb` crate.
- Internal crates were renamed and split into shorter architecture crates:
  - `asm`
  - `hooks`
  - `lua-api`
  - `plugin-api`
  - `plugin-host`
  - `rdb`
  - `struct-api`
- Runtime patching logic moved out of generic hooks and into plugin-owned code.
- Plugin logs are isolated under each plugin folder.
- Noisy `fx_director` live-cycle logs are reduced; hot reload and definition changes remain visible.

### Removed

- Old monolithic `weapon_aura` prototype plugin.
- Old `oppw4-*` crate names after the architecture split.
- Hard dependency between global hooks and RDB patching internals.

### Notes

- `dinput8.dll` is now effectively the modloader host.
- Mods live under `mods/`; plugins live under `plugins/`.
- Zip mods load normally, but hot reload is only for directory mods.
- This is still experimental and intended for fast iteration before public launcher polish.
