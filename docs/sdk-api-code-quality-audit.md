# SDK/API Code Quality Audit

Scope: `crates/plugins/sdk`, `crates/plugins/host`, `crates/lua-api`, `crates/hooks`, `crates/struct-api`.

Out of scope: `plugins/*`, `apps/*`, test-only functions.

## Current Rule

- Functions over 20 production lines are noted and classified.
- Rust files over 150 lines are split only when they carry multiple responsibilities.
- Tests may be long when they act as integration-style fixtures, but critical behavior should still get focused helpers.

## LinkData Refactor Status

Status: split now, completed for the first pass.

- Public archive API remains under `plugin_sdk::linkdata::{LinkDataArchive, LinkDataEntry, LinkDataError}`.
- `linkdata/archive/` now separates entry metadata, errors, table parsing, inflation, and rebuilding.
- `linkdata/sections/` now separates section errors, section parsing, row operations, and rebuild orchestration.
- `plugin-host/src/runtime/linkdata/state/` now separates runtime file state, edit ownership, row patch application, I/O, and tests.
- `plugin-host/src/runtime/linkdata/registry/` now separates registry orchestration, logging, and virtual open handling.
- `plugin-sdk/src/api/linkdata/` now separates entry calls, row calls, and row target types.

Critical tests added:

- compressed LINKDATA entry payload inflation.
- same-plugin entry replacement plus row edit materialization.
- SDK virtual file provider builder conversion into the raw host ABI provider.

## Remaining Modules Over 150 Lines

Split now:

- `crates/hooks/src/winapi_file/mod.rs` - 968 lines. Mixes IAT scanning/patching, hooked WinAPI exports, provider registry, virtual file handles, read/seek/size/time dispatch, and logging. Next major split candidate.
- `crates/lua-api/src/runtime.rs` - 558 lines. Mixes Lua globals, require hook, Character API, unsafe character lookup, custom character handles, and local player handle.
- `crates/lua-api/src/manifest.rs` - 341 lines. Mixes manifest schema, TOML parsing, zip manifest discovery, path inference, and tests.

Keep justified for now:

- `crates/struct-api/src/characters.rs` - 293 lines. Mostly static character catalog plus validation; split only when catalog grows into multiple domains.
- `crates/plugins/sdk/src/api/unsafe.rs` - 203 lines. Unsafe host callback boundary; keep cohesive until more callbacks are added.
- `crates/hooks/src/memory.rs` - 187 lines. Memory helpers and safety checks are cohesive.
- `crates/plugins/sdk/src/manifest.rs` - 180 lines. Public manifest model and parser are related.

Ignored by rule:

- `crates/plugins/sdk/src/linkdata/tests.rs` - test fixture file.

## Remaining Production Functions Over 20 Lines

Backlog, high priority:

- `crates/hooks/src/winapi_file/mod.rs`: `read_virtual_file` 69, `patch_import_by_name` 51, `hooked_create_file_w` 41, `dispatch_patch_read` 35, `get_virtual_file_time` 31, `seek_virtual_file` 30, plus smaller hook/provider helpers.
- `crates/lua-api/src/runtime.rs`: `custom_character_handle_table` 69, `install_character_core` 68, `character_handle_table` 37, `parse_unsafe_character_id` 30.
- `crates/lua-api/src/manifest.rs`: `parse_mod_manifest` 48, `read_zip_manifest` 26.
- `crates/plugins/host/src/runtime/manifest.rs`: `infer_entry_file` 40, `read_from_dir` 34.

Keep justified for now:

- `crates/plugins/host/src/runtime/linkdata/api.rs`: `row_patch_from_abi` 23. Small ABI conversion match; split would hide the operation mapping.
- `crates/plugins/host/src/runtime/linkdata/state/patch.rs`: `key` 26, `apply` 24. Enum branch mapping is explicit and localized.
- `crates/plugins/host/src/runtime/linkdata/virtual_file.rs`: `read` 27, `seek` 26. Small virtual file cursor operations; good candidates for tests before further split.
- `crates/plugins/sdk/src/api/linkdata/entry.rs`: `replace_entry` 22. Public host call wrapper; readable as-is.
- `crates/plugins/sdk/src/linkdata/archive/error.rs`: `fmt` 23. Error display match.
- `crates/plugins/sdk/src/linkdata/archive/inflate.rs`: `inflate_chunk` 22. Single chunk decode path with error mapping.
- `crates/plugins/sdk/src/linkdata/sections/error.rs`: `fmt` 25. Error display match.
- `crates/plugins/sdk/src/manifest.rs`: `parse_toml` 28. Public manifest parsing boundary.
- `crates/struct-api/src/characters.rs`: `validate_characters` 28. Catalog invariant check.

Low priority support helpers:

- `crates/hooks/src/inline.rs`: `install_abs_jump` 23.
- `crates/hooks/src/memory.rs`: `is_readable_range` 26.
- `crates/hooks/src/status.rs`: `mark_file_open` 25.
- `crates/plugins/abi/src/helpers.rs`: `null_api` 23.
- `crates/plugins/host/src/runtime/ffi/api.rs`: `build_api` 30.
- `crates/plugins/host/src/runtime/loader/plugin.rs`: `load_plugin` 28, `initialize_plugin` 24.
- `crates/plugins/host/src/runtime/lua/mod.rs`: `register_module` 31.
- `crates/plugins/host/src/runtime/lua/runner.rs`: `run_mod` 31.
- `crates/plugins/host/src/runtime/lua/state.rs`: `reload_changed_directory_mods` 30.
- `crates/plugins/sdk/src/api/unsafe.rs`: `collect_plugin_mod` 31.
