# OPPW4 Patcher Reverse Notes

## Scope

Goal: understand the existing `dinput8.dll` patcher for One Piece: Pirate Warriors 4 well enough to rebuild a cleaner mod loader in Rust.

Safety rule: do not modify the live game install while analyzing. Work from copies under this workspace.

## Paths

- Game root: `D:\SteamLibrary\steamapps\common\OPPW4`
- Live patcher DLL: `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.dll`
- Game RDB folder: `D:\SteamLibrary\steamapps\common\OPPW4\File\CMN\AssetRelease\Retail`
- Mod folder used by current patcher: `D:\SteamLibrary\steamapps\common\OPPW4\OPPW4_PATCHER`
- Analysis copy: `C:\Users\Osef\Documents\Codex\2026-05-09\si-je-te-demanderais-avec-du\oppw4-ghidra\dinput8.dll`
- Ghidra project: `C:\Users\Osef\Documents\Codex\2026-05-09\si-je-te-demanderais-avec-du\oppw4-ghidra\oppw4_patcher`
- Exported decompile notes: `C:\Users\Osef\Documents\Codex\2026-05-09\si-je-te-demanderais-avec-du\oppw4-ghidra\decompile_64803ed0.txt`

## Environment

- Ghidra: `C:\ghidra_12.0.4_PUBLIC`
- Headless launcher: `C:\ghidra_12.0.4_PUBLIC\support\analyzeHeadless.bat`
- Java used for Ghidra: `C:\Program Files\JetBrains\DataGrip 2025.2.3\jbr`
- `ghidraRun.bat` exists at `C:\ghidra_12.0.4_PUBLIC\ghidraRun.bat`

## Confirmed Binary Facts

- `dinput8.dll` is PE32+ / x86-64.
- Size: about 9.3 MB.
- Image base: `0x64800000`.
- Timestamp from PE header: `Wed Apr 12 08:23:04 2023`.
- It exports the classic DirectInput proxy surface:
  - `DirectInput8Create`
  - `DllCanUnloadNow`
  - `DllGetClassObject`
  - `DllMain`
  - `DllRegisterServer`
  - `DllUnregisterServer`
- It loads the real system DLL:
  - `C:\Windows\system32\dinput8.dll`
- It forwards/resolves original DirectInput exports via `GetProcAddress`.
- It imports file and memory APIs relevant to patching:
  - `CreateFileA`
  - `CreateFileW`
  - `ReadFile`
  - `CloseHandle`
  - `GetFileSizeEx`
  - `GetFileTime`
  - `SetFilePointerEx`
  - `VirtualProtect`
  - thread snapshot/suspend/resume APIs

## Current Game / Logs

From `logs\modules.log`:

- Game exe: `D:\SteamLibrary\steamapps\common\OPPW4\OPPW4.exe`
- Game version: `ONE PIECE: PIRATE WARRIORS 4 1.0.8.6`
- Patcher module base in observed run: `0x64800000`

From `OPPW4_PATCHER\patcher_log.txt`:

- Logs own path:
  - `Myself path = D:/SteamLibrary/steamapps/common/OPPW4/`
- Logs original DLL:
  - `original DLL path: C:\Windows\system32\dinput8.dll`
- Logs initialization:
  - `OPPW4_PATCHER. Exe base = ... My Dll base = ... My dll name: dinput8.dll`
- Logs virtualized files:
  - `File ".../OPPW4_PATCHER/<ArchiveName>/<asset>" added to virtualization.`
- Logs invalid mod files:
  - `Notice: file "<asset>" doesn't exist in this .rdb (<ArchiveName>.rdb)`
- Logs totals:
  - `Num virtualized files (<ArchiveName>.rdb) = N`

## Config

File: `D:\SteamLibrary\steamapps\common\OPPW4\OPPW4_PATCHER\OPPW4_PATCHER.ini`

```ini
[Debug]
; Log access to files contained in .rdb.bin 
log_internal = false
; Log access to external files (CMN\AssetRelease\Retail\data)
log_external = false
; Log access to files overridden by the patcher
log_virtual = false
```

Ghidra confirms these keys are read in `FUN_64805450`.

## RDB Files

Confirmed RDB/index pairs in `File\CMN\AssetRelease\Retail`:

- `CharacterEditor.rdb` / `CharacterEditor.rdb.bin`
- `FieldEditor4.rdb` / `FieldEditor4.rdb.bin`
- `KIDSSystemResource.rdb` / `KIDSSystemResource.rdb.bin`
- `MaterialEditor.rdb` / `MaterialEditor.rdb.bin`
- `RRPreview.rdb` / `RRPreview.rdb.bin`
- `ScreenLayout.rdb` / `ScreenLayout.rdb.bin`
- `SequenceEditor.rdb` / `SequenceEditor.rdb.bin`
- `system.rdb` / `system.rdb.bin`

Observed RDB header pattern:

- Starts with `_DRK0000`
- Contains many `IDRK0000` blocks
- `.rdb` appears to be the compact index/metadata file
- `.rdb.bin` appears to hold bulk data

## Embedded Asset Name Database

The DLL embeds a large hash-to-name database, with entries like:

```text
0x84f28305,ET_042.efpl
0xc372a1e7,803_002_name_nami_External_08.texinfo
0xb290631c,CharacterEditor.kidssingletondb
0xcb3c8b15,CharacterEditor.kidssingletondb.name
```

This likely helps resolve hashed RDB entries back to names and validate mod files.

## Key Ghidra Functions

### `FUN_64801550`

Role: DirectInput proxy bootstrap.

Confirmed refs:

- `Myself path = %s`
- `original DLL path: %s   base=%p`

Observed behavior:

- Gets own module filename.
- Computes game/base path.
- Builds path to system `dinput8.dll`.
- Loads original `dinput8.dll` with `LoadLibraryA`.
- Resolves original exports with `GetProcAddress`.
- Patches/forwards proxy exports.

### `FUN_64801db0`

Role: main `OPPW4_PATCHER` initialization.

Confirmed refs:

- `OPPW4_PATCHER. Exe base = %p. My Dll base = %p. My dll name: %s`
- `OPPW4_PATCHER/OPPW4_PATCHER.ini`

Observed behavior:

- Calls `FUN_64801550`.
- Gets module handles for exe and own DLL.
- Opens/parses patcher INI.
- Calls `FUN_64805450` for config/debug setup.
- Later code calls the scanner for known archives and installs hooks.

### `FUN_64805450`

Role: reads config flags and likely starts archive setup.

Confirmed refs:

- `log_virtual`
- `log_external`
- `log_internal`
- `OPPW4_PATCHER/`

Important later behavior seen in decompile output:

- Calls `FUN_64803ed0` repeatedly for known archives:
  - `KIDSSystemResource`
  - `FieldEditor4`
  - `RRPreview`
  - `ScreenLayout`
  - `system`
  - `SequenceEditor`
  - Earlier part likely includes `CharacterEditor` and `MaterialEditor`.
- Installs hooks:
  - `CreateFileW`
  - `CloseHandle`
  - `ReadFile`
  - `GetFileSizeEx`
  - `GetFileTime`

### `FUN_64803ed0`

Role: core virtualization scanner/registrar.

Confirmed refs:

- `Notice: file "%s" doesn't exist in this .rdb (%s)`
- `File "%s" added to virtualization.`
- `Num virtualized files (%s) = %Id`

Likely signature:

```c
void scan_archive_mod_folder(rdb_name, patcher_subfolder, out_archive_context)
```

Observed behavior:

- Receives an RDB name/path, a patcher folder, and a pointer to output archive context.
- Enumerates files in the patcher folder.
- For each file:
  - copies/normalizes filename.
  - checks whether name is a raw hash style or a normal asset name.
  - uses `FUN_64802ae0` to parse names like `0x????????`.
  - looks the file up in the parsed RDB archive index.
  - handles `.rigbin.rigbin` fallback by trimming suffix.
  - if found, creates a virtual replacement entry.
  - if not found, logs the notice.
- On success, inserts entries into global tables:
  - `DAT_650e3a60`
  - `DAT_650e3b20`
- Logs the total count.

### `FUN_64802ae0`

Role: parses hash-style asset names.

Confirmed behavior:

- Takes a string and output `uint32`.
- Requires the string to start with `0x`.
- Walks characters after `0x` until `.` or string end.
- Accepts only hex chars `0-9`, `a-f`, `A-F` after lowercasing.
- Converts the string to a `u32`.
- Returns whether parsing succeeded.

### `FUN_64803020`

Role: hooked `CreateFileW`.

Observed signature:

```c
HANDLE FUN_64803020(
    LPCWSTR path,
    DWORD desired_access,
    DWORD share_mode,
    LPSECURITY_ATTRIBUTES security,
    DWORD creation_disposition,
    DWORD flags,
    HANDLE template_file
)
```

Observed behavior:

- Only special-cases read access (`0x80000000` / `GENERIC_READ`).
- Converts/normalizes wide path.
- Checks if path is under the game root.
- Looks up normalized path in `DAT_650e3b20`.
- If not found, checks `DAT_650e3a60`.
- If virtual replacement exists:
  - opens external mod file with `CreateFileA`.
  - creates/allocates virtual file state.
  - stores fake handle mapping in `DAT_650e3b60`.
  - logs `Open virtual %s` if `log_virtual` is enabled.
- If no virtual replacement exists:
  - optionally logs internal/external accesses.
  - falls back to real `CreateFileW`.

### `FUN_64803c60`

Role: hooked `ReadFile`.

Observed behavior:

- Checks whether handle is in `DAT_650e3b60`.
- If not virtual, optionally logs reads and calls real `ReadFile`.
- If virtual, calls `FUN_64802d30` to satisfy the read from virtual state.

### `FUN_64802d30`

Role: virtual read helper used by the `ReadFile` hook.

Observed behavior:

- Maintains a current virtual position at state offset `0x30` (`param_1[6]`).
- Supports non-overlapped reads.
- If an overlapped read includes `hEvent`, it logs:
  - `-----------WARNING: overlapped hEvent not implemented`
- Reads first from an in-memory prefix buffer:
  - `state[0]` = memory buffer pointer
  - `state[1]` = memory buffer size
- If more bytes are needed and `state[2]` is a real file handle:
  - seeks real file to `(state[3] + current_pos) - memory_prefix_size`
  - clamps reads against `state[5]`
  - reads from the real backing file
- Updates the virtual current position and `lpNumberOfBytesRead`.

### `FUN_64803d60`

Role: hooked `CloseHandle`.

Observed behavior:

- Checks whether handle is a fake/virtual handle in `DAT_650e3b60`.
- If not virtual, calls real `CloseHandle`.
- If virtual, closes/free associated replacement file state and removes handle mapping.

### `FUN_64806970`, `FUN_64806db0`, `FUN_64806eb0`

Role: wrapper variants used when debug logging is enabled.

Observed behavior:

- `FUN_64806970` wraps `CreateFileW` and adds extra external/internal path logging.
- `FUN_64806db0` wraps `CloseHandle`.
- `FUN_64806eb0` wraps `ReadFile`.

### `FUN_64833aa0`

Role: hook installer wrapper.

Observed usage:

```c
FUN_64833aa0("KERNEL32.DLL", "CreateFileW", hook_fn, 0)
FUN_64833aa0("KERNEL32.DLL", "CloseHandle", hook_fn, 0)
FUN_64833aa0("KERNEL32.DLL", "ReadFile", hook_fn, 0)
FUN_64833aa0("KERNEL32.DLL", "GetFileSizeEx", hook_fn, 0)
FUN_64833aa0("KERNEL32.DLL", "GetFileTime", hook_fn, 0)
```

Likely wraps import/IAT patching or trampoline hook setup.

Confirmed implementation:

- `FUN_648339c0(module_name, import_name_or_ordinal, target_module, by_ordinal)` finds an IAT slot.
- It calls `GetModuleHandleA(target_module)`.
- It walks the target module import descriptors.
- It matches imported DLL name case-insensitively.
- It then walks thunk entries and matches either:
  - import by name, or
  - import by ordinal when `by_ordinal != 0`.
- It returns the address of the IAT slot.
- `FUN_64833690(iat_slot, hook_fn)` temporarily makes the slot writable with `VirtualProtect`, writes the hook pointer, then restores the old protection.

So this patcher uses direct IAT patching, not MinHook/trampoline hooks, for the file API hooks.

### `FUN_64802f00`

Role: hooked `GetFileTime`.

Observed behavior:

- If handle is not virtual, calls real `GetFileTime`.
- If handle is virtual and has a real backing file handle, calls `GetFileTime` on that backing handle.
- If handle is virtual but has no real backing handle, returns current system time for creation/access/write times.

### `FUN_64802fc0`

Role: hooked `GetFileSizeEx`.

Observed behavior:

- If handle is not virtual, calls real `GetFileSizeEx`.
- If handle is virtual, returns:
  - `state[1] + state[4]`
- Interpreted as memory prefix size plus virtual/backing range size.

## Global Tables / Data To Name

Likely tables:

- `DAT_650e3a60`: virtual replacement lookup by asset/path.
- `DAT_650e3b20`: archive path/context lookup by RDB path.
- `DAT_650e3b60`: fake handle -> virtual file state mapping.
- `DAT_650e3be0`: debug/log tracking for real file handles.

Likely flags:

- `DAT_650e3a00`: `log_virtual`
- `DAT_650e3a01`: `log_external`
- `DAT_650e3a02`: `log_internal`
- `DAT_650e3a03`: additional debug file access logging gate.

Needs more confirmation.

## RDB Parser Notes

### File Layout Confirmed From Hex/Parser

Root header:

- Starts with `_DRK0000`
- Offset `0x08`: first block offset. Observed `0x20`.
- Offset `0x10`: count or related total. For `CharacterEditor.rdb`, observed `2038` (`0x7f6`).
- Offset `0x18`: string `data/`

Blocks:

- Start with `IDRK0000`.
- Parser compares the first 4 bytes against `IDRK`, not necessarily the full 8-byte string.
- Block length is at block offset `0x08` as `u32`.
- Next block offset is `align4(current_offset + block_length)`.
- Fixed header appears to be `0x30` bytes.
- Variable payload starts at `block + 0x30`.

Example from `CharacterEditor.rdb`:

```text
root=_DRK0000 firstBlockOff=32 countOrSize=2038 data=data/
#0 off=0x20 sig=IDRK0000 len=0x44 f10=0xc f18=0x100 f20=0x0 f24=0xe3d46 f28=0x56efe45c f2c=0x20000 payload=...11c3971@138
#1 off=0x64 sig=IDRK0000 len=0x76 f10=0xe f18=0x65d38 f20=0xc f24=0x504a4e f28=0x563bdef1 f2c=0x120000 payload=...
#2 off=0xdc sig=IDRK0000 len=0x45 f10=0xd f18=0x2f4 f20=0x0 f24=0x55a1f4 f28=0x5c3e543c f2c=0x20000 payload=...6b4ed0@32c#9
```

### `FUN_6482fcb0`

Role: loads/parses an RDB buffer into an archive object.

Confirmed checks:

- Rejects files smaller than `0x20`.
- Compares root signature with `_DRK`.
- Logs `Invalid RDB file.` if invalid.
- Uses header offset `+0x08` to find first block.
- Uses header offset `+0x10` to reserve/size entry storage.
- Loops over `IDRK` blocks until end.
- Logs `Unknown signature at offset %Ix` if a block signature is not recognized.
- Logs `Something wrong at offset %Ix` if block length math is invalid.

In-memory entry:

- Each parsed entry is `0x68` bytes.
- Entry vector appears at archive object offsets:
  - `archive + 0x18`: begin
  - `archive + 0x20`: end
- Count calculation in several functions:
  - `((end - begin) >> 3) * 0x4ec4ec4ec4ec4ec5`
  - This is compiler division by `0x68`.

Known in-memory entry fields from parser:

```text
entry + 0x00: string/name 1
entry + 0x08: string/name 2
entry + 0x10: string/name 3
entry + 0x18: file/range value from block
entry + 0x20: file/range value from block
entry + 0x28: int from block
entry + 0x2c: int from block
entry + 0x30: flag byte
entry + 0x38: value from block +0x18
entry + 0x40: value from block +0x20
entry + 0x44: primary hash/id used by `FUN_64829050`
entry + 0x48: value from block +0x28
entry + 0x4c: value from block +0x2c
entry + 0x50: payload buffer begin
entry + 0x58: payload buffer current/end used by vector
entry + 0x60: payload buffer capacity/end
```

This mapping is still provisional, but `entry + 0x44` being the lookup hash is confirmed by `FUN_64829050`.

### `FUN_64829050`

Role: lookup RDB entry by `u32` hash/id.

Confirmed behavior:

- If hash table at `archive + 0x110` is absent, linearly scans entries.
- Entry count is derived from `(archive[0x20] - archive[0x18]) / 0x68`.
- Compares requested hash against `*(u32 *)(entry + 0x44)`.
- Returns entry index or `-1`.
- If hash table exists, looks up in table at `archive + 0xf0`.

### `FUN_6482cca0`

Role: lookup RDB entry by string/name.

Observed behavior:

- Uses a name lookup table when present at `archive + 0xd0`.
- Otherwise linearly scans a linked/list table at `archive + 0x88`.
- Ultimately resolves a hash/id and calls `FUN_64829050`.
- Returns entry pointer/index or `-1`.

### `FUN_6482b150`

Role: get/materialize an entry name for logging/debug.

Observed behavior:

- Calls `FUN_64829050`.
- If found, calls `FUN_6482aea0`.

### `FUN_6482aea0`

Role: materialize/extract entry name-ish string by entry index.

Observed behavior:

- Bounds-checks entry index against parsed entry vector count.
- Computes entry pointer as `archive.entries_begin + index * 0x68`.
- Reads `entry + 0x44`.
- More detailed string extraction still needs cleanup.

## Current Mental Model

The patcher is not primarily an internal engine hook. It is mostly a Windows file API virtualizer.

Flow:

```text
game loads local dinput8.dll
  -> patcher loads real system dinput8.dll
  -> patcher scans OPPW4_PATCHER/<ArchiveName> folders
  -> patcher parses/loads matching game .rdb indexes
  -> patcher builds virtual replacement tables
  -> patcher hooks KERNEL32 file APIs

game opens/reads RDB-backed asset data
  -> hooked CreateFileW normalizes path
  -> if path maps to a virtual replacement, returns fake handle
  -> hooked ReadFile serves bytes from mod file / virtual state
  -> hooked CloseHandle cleans virtual state
```

This is good news for a Rust rewrite: cloning the loader does not require immediately hooking game-specific functions.

## Rust Reimplementation Milestones

### Milestone 1: Passive DLL Proxy

- Build Rust `cdylib` named `dinput8.dll`.
- Export the same DirectInput functions.
- Load real system `dinput8.dll`.
- Forward exports.
- Write `OPPW4_PATCHER/patcher_log.txt`.
- Confirm the game starts.

### Milestone 2: RDB/Folder Scanner

- Parse `OPPW4_PATCHER.ini`.
- Scan `OPPW4_PATCHER/<ArchiveName>`.
- Parse enough of `.rdb` to validate whether a modded asset exists.
- Log added/invalid files matching current patcher style.

### Milestone 3: File API Virtualization

- Hook `CreateFileW`, `ReadFile`, `CloseHandle`, `GetFileSizeEx`, `GetFileTime`.
- Build fake handle table.
- Serve replacement files.
- Match current behavior for `.g1m` / `.g1t` mods.

### Milestone 4: Modern Loader Features

- `plugins/<plugin_id>/plugin.toml`
- `plugins/<plugin_id>/mods/`
- load order / priorities
- conflict detection
- profiles
- better logs
- version compatibility
- clean validation messages

### Milestone 5: Content Expansion Research

- Find character/costume tables.
- Look for unused slots.
- Understand UI references.
- Experiment with real additions, not only replacements.

## Next Reverse Targets

1. Confirm `FUN_64802ae0` behavior as hash parser.
2. Map structs used by `FUN_64803ed0` virtual replacement entries.
3. Map fake handle state struct allocated by `FUN_64803020`.
4. Confirm exact hook installer implementation:
   - `FUN_64833aa0`
   - `FUN_648339c0`
   - `FUN_64833690`
5. Extract a minimal RDB parser:
   - header `_DRK0000`
   - `IDRK0000` entry layout
   - name/hash -> offset/size mapping
6. Rename functions/globals in Ghidra project once confidence is high.

## Rust RDB Parser Progress

Created a Rust Cargo workspace with `rdb-tools`, a small crate for validating the `.rdb` format outside Ghidra.

Current workspace layout:

- root `Cargo.toml`: workspace
- `rdb-tools/src/rdb.rs`: RDB header/block parser
- `rdb-tools/src/address.rs`: RDB address tail parser
- `rdb-tools/src/catalog.rs`: embedded DLL `name -> hash` catalog parser
- `rdb-tools/src/hash.rs`: `0x...` filename hash parser
- `rdb-tools/src/scan.rs`: virtualized mod file scanner
- `rdb-tools/src/bytes.rs`: low-level byte helpers
- `rdb-tools/src/main.rs`: CLI wiring only
- `oppw4-dinput8-proxy`: Rust `cdylib` passive `dinput8.dll` proxy

Current parser supports:

- root magic `_DRK0000`
- first block offset at `0x08`
- declared entry count at `0x10`
- data prefix string at `0x18` (`data/`)
- aligned `IDRK0000` blocks
- block length at `+0x08`
- likely data offset at `+0x18`
- primary hash at `+0x24`
- payload from `+0x30..block_end`
- patcher-style hash names: `0x3b359352.g1m` -> `0x3b359352`
- address tail extraction using `block_length - field_10`
- embedded DLL catalog extraction: `Name.ext` + `0xhash`
- mod folder scanning against an RDB index

Copied real index files into local fixtures:

- `rdb-tools/fixtures/rdb/SequenceEditor.rdb`
- `rdb-tools/fixtures/rdb/CharacterEditor.rdb`
- `rdb-tools/fixtures/rdb/ScreenLayout.rdb`
- `rdb-tools/fixtures/rdb/MaterialEditor.rdb`

Validation results:

- `SequenceEditor.rdb`: declared `202`, parsed `202`
- `CharacterEditor.rdb`: declared `2038`, parsed `2038`
- `ScreenLayout.rdb`: declared `23869`, parsed `23869`
- `MaterialEditor.rdb`: declared `33643`, parsed `33643`
- `cargo test` passes with fake fixtures plus real `SequenceEditor.rdb`
- hash search works in the CLI
- `cargo test` currently passes: `12` tests

Embedded catalog:

- `dinput8.dll` contains a large CRLF-separated catalog of `asset_name` -> `0xhash`.
- Parsed catalog count from current DLL copy: `70830` entries.
- Example mappings:
  - `MDLC038_Zoro_Wa.g1m` -> `0x1ad9ce2c`
  - `MPLC009_Ace.g1m` -> `0xa8bd2e49`
  - `MVAR006_Robin_NW.g1m` -> `0x7dabea76`

Folder scan results using the catalog:

- `CharacterEditor`: `12/12` files matched.
- `ScreenLayout`: `4/4` files matched.
- `MaterialEditor`: `90/91` files matched.
  - Missing: `0x93dfb06c.g1t`.
  - This matches the original patcher log: `0x93dfb06c.g1t` does not exist in `MaterialEditor.rdb`, but does exist in `ScreenLayout.rdb`.
- `RRPreview`: `2/2` files matched after adding `.g1e` to known catalog extensions.

Global scan command:

```powershell
cargo run -p rdb-tools -- --scan-root "D:\SteamLibrary\steamapps\common\OPPW4\OPPW4_PATCHER" --rdb-root "D:\SteamLibrary\steamapps\common\OPPW4\File\CMN\AssetRelease\Retail" --catalog .\oppw4-ghidra\dinput8.dll
```

Global scan result:

- catalog entries parsed: `73045`
- total mod files scanned: `109`
- matched: `108`
- missing in archive: `1`
- unresolved by name/hash: `0`
- only current missing file: `0x93dfb06c.g1t` in `MaterialEditor`, expected because it belongs to `ScreenLayout`.
- virtualization table replacements: `108`
- total replacement file bytes: `151093964`

Current archive counts:

- `CharacterEditor`: files `12`, matched `12`
- `FieldEditor4`: files `0`, matched `0`
- `KIDSSystemResource`: files `0`, matched `0`
- `MaterialEditor`: files `91`, matched `90`, missing `1`
- `RRPreview`: files `2`, matched `2`
- `ScreenLayout`: files `4`, matched `4`
- `SequenceEditor`: files `0`, matched `0`
- `system`: files `0`, matched `0`

Virtualization table:

- `rdb-tools/src/virtual_table.rs`
- Builds `VirtualReplacement` entries from matched scan results only.
- Each entry currently stores:
  - archive name
  - source mod file name
  - source mod path
  - resolved RDB hash
  - RDB block offset
  - original RDB data offset
  - optional mod file size
- `attach_mod_file_sizes` reads file metadata and fills replacement sizes.

This is not the final hook table yet, but it is the clean pre-hook representation that can feed the future Rust `dinput8.dll`.

Virtual file model:

- `rdb-tools/src/virtual_file.rs`
- `VirtualFile<R>` wraps any `Read + Seek` source.
- Tracks:
  - virtual size
  - current read position
- Supports:
  - bounded reads
  - EOF at virtual size
  - `SeekFrom::Start`
  - `SeekFrom::Current`
  - `SeekFrom::End`
- `open_virtual_replacement` opens a `VirtualReplacement` from disk and returns a `VirtualFile<File>`.
- Verified with a real mod file:
  - `OPPW4_PATCHER/CharacterEditor/0x3b359352.g1m`
  - header bytes read as ASCII: `_M1G7300`

This models the core behavior the future hooked `ReadFile`/`SetFilePointer` path will need, before touching Windows hooks.

Virtual handle / manager model:

- `rdb-tools/src/virtual_handles.rs`
- `rdb-tools/src/virtual_manager.rs`
- `VirtualHandleTable` allocates monotonic fake handles.
- Supports:
  - open replacement
  - read
  - seek
  - close
  - contains/lookup
- `VirtualManager` resolves `(archive_name, hash)` to a `VirtualReplacement`, opens a virtual handle, then delegates reads/seeks/closes to the handle table.

End-to-end probe:

- Global scan builds `108` replacements.
- Manager opens first replacement by `(CharacterEditor, 0x3b359352)`.
- Virtual read returns first bytes `_M1G7300`.

This is now very close to the future DLL flow:

```text
CreateFileW/RDB request
  -> resolve archive + hash
  -> open VirtualManager handle
ReadFile(fake handle)
  -> VirtualHandleTable read
CloseHandle(fake handle)
  -> VirtualHandleTable close
```

## Rust dinput8 Proxy Progress

Created `oppw4-dinput8-proxy`.

Build command:

```powershell
cargo build -p oppw4-dinput8-proxy
```

Output:

- `target/debug/dinput8.dll`

Current exports verified from PE export table:

- `DirectInput8Create`
- `DllCanUnloadNow`
- `DllGetClassObject`
- `DllMain`
- `DllRegisterServer`
- `DllUnregisterServer`

Current behavior:

- `DllMain` calls `loader::initialize_once()`.
- Loader init initializes file logging and starts a worker thread.
- Each exported DirectInput/COM function loads the real system `dinput8.dll` from `GetSystemDirectoryW() + "\\dinput8.dll"` and forwards to the real export via `GetProcAddress`.
- Log path when loaded from a game folder:
  - `OPPW4_PATCHER/rust_patcher_log.txt`
- Passive worker currently:
  - logs patcher/RDB roots
  - loads `OPPW4_PATCHER/name_hash_catalog.txt` when present
  - scans known archive folders in read-only mode
  - parses matching RDB indexes
  - resolves both direct `0x...` names and catalog file names
  - builds a `VirtualReplacement` table with mod file sizes
  - publishes the table to the runtime
  - attempts to install main-module IAT hooks for `CreateFileW`, `ReadFile`, `CloseHandle`, `GetFileSizeEx`, and `SetFilePointerEx`

Hook status:

- `oppw4-dinput8-proxy/src/hooks.rs` now contains the first hook layer.
- Hooked `CreateFileW` currently supports direct virtual asset opens by matching the requested path basename against the replacement table.
- Hooked `ReadFile`, `CloseHandle`, `GetFileSizeEx`, and `SetFilePointerEx` support the fake virtual handles returned by direct virtual opens.
- Real Windows calls remain the fallback for all normal handles and unmatched paths.
- `.rdb` diagnostic tracking is now installed:
  - `CreateFileW` tracks real handles whose basename ends in `.rdb`
  - `ReadFile` logs up to 80 reads per tracked RDB and 800 reads globally
  - `CloseHandle` untracks those handles
- This is not yet full RDB read-range virtualization; the next step is using the diagnostic read offsets to map exact replacement windows.

Live hook test:

- User launched OPPW4 with hook-layer DLL.
- Log confirmed:
  - `virtual replacements ready: 108`
  - `virtual runtime published: 108 replacements`
  - `IAT hooks installed: 5`
- This proves the game executable imports all five hooked file APIs through the patched IAT.

Out-of-game smoke test:

- Loaded `target/debug/dinput8.dll` via `LoadLibraryW`.
- Called `DllCanUnloadNow` through `GetProcAddress`.
- Result: `HRESULT 0x00000000`.
- Log confirmed:
  - proxy loaded
  - worker started
  - attempted passive archive scan
  - original `C:\Windows\system32\dinput8.dll` loaded successfully

Hook-layer smoke test:

- Loaded `target/debug/dinput8.dll` via `LoadLibraryW` from PowerShell.
- The simulated layout produced:
  - `virtual replacements ready: 2`
  - `virtual runtime published: 2 replacements`
  - `IAT hooks installed: 0`
- `0` hooks in this smoke test is expected because the PowerShell host does not import the target file APIs like the native game executable should.

Catalog export command:

```powershell
cargo run -p rdb-tools -- --export-catalog .\oppw4-ghidra\dinput8.dll .\name_hash_catalog.txt
```

Generated:

- `name_hash_catalog.txt`
- entries: `73045`

Local DLL smoke test with simulated game layout:

- copied `name_hash_catalog.txt` to `target/debug/OPPW4_PATCHER/name_hash_catalog.txt`
- copied `CharacterEditor.rdb` to `target/debug/File/CMN/AssetRelease/Retail/CharacterEditor.rdb`
- created sample mod filenames:
  - `0x3b359352.g1m`
  - `MDLC038_Zoro_Wa.g1m`
- loaded `target/debug/dinput8.dll`
- log result:
  - `name catalog entries: 73045`
  - `CharacterEditor: files=2 matched=2 hash_missing=0 unresolved=0`

Important:

- This DLL is now ready only for a smoke test in the live game folder.
- It is still a passive proxy foundation only.
- It will not load mods yet because hooks are not installed.
- Next steps before live testing:
  - install IAT hooks
  - connect hooks to `VirtualManager`

Known hash lookups in `CharacterEditor.rdb`:

- `0x3b359352`: 1 match, block offset `0x8f28`, data offset `0x420a0c`, address `0@268005#9`
- `0x4ce275fb`: 1 match, block offset `0xb778`, data offset `0x3ddf54`, address `572570@262db0#b`
- `0xc4db47b5`: 1 match, block offset `0x1e408`, data offset `0x1c3808`, address `1268d0@11c28b#8`
- `0xd31cb784`: 1 match, block offset `0x1fd30`, data offset `0x13bb64`, address `813450@b9bda#6`

Known hash lookup in `ScreenLayout.rdb`:

- `0x93dfb06c`: 1 match, block offset `0x11b25c`, data offset `0x200038`, address `f8c90@babcb#b`

Observed payload tail strings:

- `0@268005#9`
- `572570@262db0#b`

Ghidra cross-check:

- `FUN_6482fcb0` computes the address string start as `block + (block_length - field_10)`.
- It splits the string around `@`, `#`, and sometimes `&`.
- It parses the split pieces as hex/integer values and stores them in the parsed `0x68`-byte entry.

Open meaning:

- The address is archive metadata, but not a simple raw offset to decompressed `.g1m` / `.g1t`.
- Reading `CharacterEditor.rdb.bin` at `data_offset` or the address middle value does not produce a visible `G1M_` header.
- The replacement file `0x3b359352.g1m` itself starts with `_M1G7300`, so the virtual loader serves clean replacement bytes while using the original RDB entry as routing metadata.

CLI command:

```powershell
cargo run -p rdb-tools -- .\rdb-tools\fixtures\rdb\CharacterEditor.rdb
cargo run -p rdb-tools -- .\rdb-tools\fixtures\rdb\CharacterEditor.rdb 0x3b359352
cargo run -p rdb-tools -- .\rdb-tools\fixtures\rdb\CharacterEditor.rdb --scan "D:\SteamLibrary\steamapps\common\OPPW4\OPPW4_PATCHER\CharacterEditor" --catalog .\oppw4-ghidra\dinput8.dll
cargo run -p rdb-tools -- --scan-root "D:\SteamLibrary\steamapps\common\OPPW4\OPPW4_PATCHER" --rdb-root "D:\SteamLibrary\steamapps\common\OPPW4\File\CMN\AssetRelease\Retail" --catalog .\oppw4-ghidra\dinput8.dll
```

Next parser step:

- decode the payload/name-ish bytes in each `IDRK` block
- map exact compressed/uncompressed size fields
- cross-check replacement hashes like `0x3b359352.g1m` against `CharacterEditor.rdb`

## Useful Scripts Created

- `oppw4-ghidra/FindPatcherStrings.java`
  - Finds refs to important patcher log/config strings.
- `oppw4-ghidra/DecompileTargets.java`
  - Dumps selected function pseudo-code to console.
- `oppw4-ghidra/ExportVirtualizationFunction.java`
  - Exports the virtualization function and key callees to `decompile_64803ed0.txt`.
