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

- `mods/<mod_name>/mod.toml`
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

Created a Rust Cargo workspace with `oppw4-rdb-tools`, a small crate for validating the `.rdb` format outside Ghidra.

Current workspace layout:

- root `Cargo.toml`: workspace
- `oppw4-rdb-tools/src/rdb.rs`: RDB header/block parser
- `oppw4-rdb-tools/src/address.rs`: RDB address tail parser
- `oppw4-rdb-tools/src/catalog.rs`: embedded DLL `name -> hash` catalog parser
- `oppw4-rdb-tools/src/hash.rs`: `0x...` filename hash parser
- `oppw4-rdb-tools/src/scan.rs`: virtualized mod file scanner
- `oppw4-rdb-tools/src/bytes.rs`: low-level byte helpers
- `oppw4-rdb-tools/src/main.rs`: CLI wiring only
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

- `oppw4-rdb-tools/fixtures/rdb/SequenceEditor.rdb`
- `oppw4-rdb-tools/fixtures/rdb/CharacterEditor.rdb`
- `oppw4-rdb-tools/fixtures/rdb/ScreenLayout.rdb`
- `oppw4-rdb-tools/fixtures/rdb/MaterialEditor.rdb`

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
cargo run -p oppw4-rdb-tools -- --scan-root "D:\SteamLibrary\steamapps\common\OPPW4\OPPW4_PATCHER" --rdb-root "D:\SteamLibrary\steamapps\common\OPPW4\File\CMN\AssetRelease\Retail" --catalog .\oppw4-ghidra\dinput8.dll
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

- `oppw4-rdb-tools/src/virtual_table.rs`
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

- `oppw4-rdb-tools/src/virtual_file.rs`
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

- `oppw4-rdb-tools/src/virtual_handles.rs`
- `oppw4-rdb-tools/src/virtual_manager.rs`
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
cargo run -p oppw4-rdb-tools -- --export-catalog .\oppw4-ghidra\dinput8.dll .\name_hash_catalog.txt
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
cargo run -p oppw4-rdb-tools -- .\oppw4-rdb-tools\fixtures\rdb\CharacterEditor.rdb
cargo run -p oppw4-rdb-tools -- .\oppw4-rdb-tools\fixtures\rdb\CharacterEditor.rdb 0x3b359352
cargo run -p oppw4-rdb-tools -- .\oppw4-rdb-tools\fixtures\rdb\CharacterEditor.rdb --scan "D:\SteamLibrary\steamapps\common\OPPW4\OPPW4_PATCHER\CharacterEditor" --catalog .\oppw4-ghidra\dinput8.dll
cargo run -p oppw4-rdb-tools -- --scan-root "D:\SteamLibrary\steamapps\common\OPPW4\OPPW4_PATCHER" --rdb-root "D:\SteamLibrary\steamapps\common\OPPW4\File\CMN\AssetRelease\Retail" --catalog .\oppw4-ghidra\dinput8.dll
```

Next parser step:

- decode the payload/name-ish bytes in each `IDRK` block
- map exact compressed/uncompressed size fields
- cross-check replacement hashes like `0x3b359352.g1m` against `CharacterEditor.rdb`

## Private Model Entry Implication

The Law slot 5 work proves a limit of the current RDB virtualizer: replacing an
existing hash is not enough when the goal is a new costume slot. Existing hashes
are global. If the custom slot reuses `MPLC026_Law.g1m`, base Law can be
affected. If it reuses `MDLC033_Law_Souhi.g1m`, Souhi can be affected. If it
reuses an unrelated live id, the game can crash before the custom file is even
opened because the surrounding LinkData metadata still describes the original
resource.

The clean target is a private Law model/resource entry:

- clone the source LinkData model row, probably entry `35` row `26`, into a
  private id;
- create or emulate a matching private model name in entry `32`, section `6`;
- patch the injected costume variant `699` so its model/resource field points to
  that private id;
- create private RDB index entries for the model and material/texture assets;
- serve the zip through those private RDB entries only.

This requires one of two RDB-side capabilities:

1. Patch the game's loaded RDB archive object in RAM and append a new `0x68`
   entry to the parsed entry vector/hash table.
2. Overlay the `.rdb` file reads so the game parses extra `IDRK` blocks as if
   they were present in the original index.

The existing external-flag patcher only edits entries that already exist, so it
cannot by itself create the private Law model entry.

## Law Row 292 Probe Result

First RAM-first probe installed `2026-05-11 23:13` tried to clone LinkData entry
`35` row `26` into row `292`, patch entry `32` section `6` row `292` to
`MPLC026_Law`, and point variant `699` at model/resource id `292`.

The user test log `2026-05-11_23-15-09.log` showed:

```text
Law private model row clone skipped source_row=26 target_row=292 reason=entry35_base_not_found
```

The first diagnostic build treated that as fatal, so the later slot injection
patch did not run and slot 5 disappeared. Fallback build `2026-05-11 23:21`
keeps the probe but falls back to model/resource id `26` when row `292` cannot
be cloned, allowing slot 5 to stay visible while the real model registry
location is investigated.

Implication for the RDB overlay work:

- do not assume the raw inflated entry `35` payload remains present as a
  writable contiguous buffer;
- either find the parsed model registry in RAM first, or use focused Ghidra to
  locate the LinkData/model table loader;
- private RDB index entries are still needed later, but only after the game can
  resolve a private model/resource row.

Follow-up probe `2026-05-11 23:35` uses the parsed model resource manager
instead of raw entry `35`.

Focused Ghidra export:

```text
oppw4-ghidra/game_resource_manager_targets.txt
```

Relevant game globals/functions:

- model manager global `DAT_141eba7a0` at RVA `0x1eba7a0`;
- `FUN_14016ce30` returns loaded resource pointer by id;
- `FUN_14016ceb0`, `FUN_14016cf30`, and `FUN_14005ad10` check loaded,
  unloaded, and busy states;
- `FUN_14016dc20` enqueues the resource load;
- state slots are addressed as `manager + id * 0x20`.

The installed probe patches variant `699` to model/resource id `292`, then
aliases model-manager requests for `292` back to source resource `26`. This is
not final private RDB support, but it tests whether the costume/variant side can
carry a private id while the model manager supplies a known-good resource.

Result from `2026-05-11_23-39-08.log`:

- no crash;
- slot 5 stayed selectable;
- official Law slots stayed intact;
- slot 5 showed an empty/invisible character;
- model-manager alias logs confirm `292 -> 26` enqueue/load/get calls worked.

RDB implication:

- creating a private model id is only one layer;
- the variant's material/color-variation fields also need to reference valid
  resources for the model to render;
- next diagnostic copies base Law variant `57` material/color bytes into
  variant `699` while keeping model id `292`;
- if this fixes visibility, private RDB overlay must also cover those material
  resources, not only the `.g1m`.

Follow-up: copying base Law variant `57` color/material bytes did not fix the
empty slot. The runtime bytes were `ffff/ffff/ffff/0000`, which means the first
three variation ids are still default/invalid just like the official base Law
path. The next diagnostic mirrors the model manager's internal entry for
resource `26` into private resource id `292`. If that fixes visibility, the RDB
overlay must create not only a private RDB name/hash but also a coherent runtime
model manager entry for the private id.

Follow-up result: the model-manager mirror worked and `manager.get(292)`
returned a real resource pointer, but the slot remained empty. The next
diagnostic hooks `FUN_1403ce790`, the render attach/binding function called
after the model resource is loaded. This should tell whether a future private
RDB overlay is blocked at resource resolution or at render-object attachment.

Follow-up result from `2026-05-12_08-07-39.log`: the selection lookup reached
custom variant `699`, but the scene/preview logs saturated before activation
and no global/private render-attach trace fired. A new diagnostic installed
`2026-05-12 08:19` preserves `scope=important` scene/list traces after the
custom slot becomes active. RDB overlay work should wait for that next log:
if the scene never switches to `699`, the blocker is still menu/preview state;
if the scene switches but no attach fires, the attach target needs a focused
Ghidra follow-up; if attach fires and fails, the blocker is render binding or
resource/material compatibility.

Follow-up result from `2026-05-12_08-21-18.log`: the scene and selected preview
object do switch to `699/slot 4`, while model manager resource id `292` is
already loaded. No render attach hook fired at `game+0x3ce790`. Focused Ghidra
shows the menu preview path sends `preview_variant=699` directly into
`FUN_14148b5f0`, so the next diagnostic hooks that helper. RDB overlay work
should still wait: if `FUN_14148b5f0` takes the hide/default branch for `699`,
the blocker is preview widget eligibility, not private RDB data yet.

Follow-up result from `2026-05-12_17-52-47.log`: the helper hook confirmed that
slot 5 reaches `FUN_14148b5f0` as `preview_variant=699`, but the game passes
`visible=0`, layout `26`, fallback `0`, and the widget fields remain hidden or
unchanged. The private model manager path is still working (`292` mirrors and
loads from `26`). This keeps RDB overlay work on hold: before inventing private
RDB index entries, the menu preview widget must be proven capable of rendering
the already-loaded private resource id.

Diagnostic installed `2026-05-12 18:04`: force only the current custom Law
preview call to `effective_visible=1` and log `forced_visible`. If this makes
the slot visible, the next work is the scene/menu eligibility flag that produces
`visible=0`; if it does not, the next work is inside `FUN_14148bcc0` /
`FUN_1416112e0` / `FUN_141613030`, still before private RDB overlay.

Follow-up from `2026-05-12_18-08-55.log`: forcing `effective_visible=1` did not
make a real preview model appear. It only changed the widget visible/active
flags, produced a black post-select layout, and the game crashed when launching.
The diagnostic was disabled in the `2026-05-12 18:21` installed build
(`SHA256 A04417BA676ECFDEB7EECD5EBABD4D38721DFC8DACAF7F6C67918592DEC21B82`).
RDB overlay work remains on hold: the current blocker is still higher-level
costume/menu/prebattle state, with `FUN_141252cc0` now the focused crash target
if launch still fails.

Follow-up diagnostic installed `2026-05-12 18:35`: `FUN_141252cc0` is now hooked
only for logging, with force-visible still disabled
(`SHA256 822847F703584AD18F3904EE662D92AB5212924ABC030D91941F849B541A19AF`).
Next logs should include `Launch costume state enter` and show whether the
launch path receives private/custom id `699` in validation fields `id1d4`,
`id1d0`, or `id1d8`. Do not resume private RDB overlay work until that confirms
which launch/prebattle table needs the cloned private entry.

Follow-up from `2026-05-12_18-41-52.log`: no `Launch costume state enter` line
appeared, and the current blocker is still before private RDB overlay. The slot
reaches the preview scene as `699`, model id `292` resolves through the manager
alias, but `FUN_1414926a0` skips the normal visible/prep path because the
scene-list flag byte it reads is zero. The relevant active entry had Law layout
`26` at list slot `0`, `list_index=0`, and flags all zero.

Diagnostic installed `2026-05-12 19:02`
(`SHA256 F1BC115EBF77CC70FC69CB6A7015AC1281B2112AD908E0126E989611F60B77E3`)
patches only that current custom Law scene-list flag byte before the original
preview refresh. RDB overlay work remains on hold until a next log proves that
the normal preview path can render the already-loaded model-manager resource.
Search for `Law custom scene-list flag diagnostic`; best case is
`patched=true before=0x00 after=0x01` followed by
`preview_variant=699 visible=1 forced_visible=false`.

Follow-up from `2026-05-12_19-08-24.log`: the scene-list patch did write the
byte, but `preview_variant=699` still reached `visible=0`. The launch crash
path is now the clearer blocker: `FUN_141252cc0` received `id1d0=292`,
`id1d4=22`, `id1d8=65535`, `id1e4=4` immediately before crashing at the known
`OPPW4.exe+0x1252DFB` validation failure. This means the private model id is
also entering gameplay/launch state, not just menu preview state.

Diagnostic installed `2026-05-12 19:21`
(`SHA256 C938E4E65380C2FFD9490B0177A69636FA7221D35EFB25BD62C44EC2BE7766E5`)
maps launch-state `id1d0=292` back to source id `26` only when slot `4` is
active. This is a temporary safety/diagnostic alias. If it removes the
in-game crash, future private RDB/model work must also include the
launch/prebattle table that validates `id1d0`, not only the menu model-manager
and RDB index paths.

Follow-up from `2026-05-12_19-24-07.log`: the launch-state alias did remove the
known top-level crash for that run, so the gameplay blocker around
`FUN_141252cc0` is now understood well enough to keep moving. The remaining
invisible-preview blocker was not private RDB resolution yet: the runtime
published zero custom slot assets because the diagnostic build had disabled the
dormant model/material alias plan. The incoming zip was valid and matched the
Law slot manifest, but `MPLC026_Law.g1m` and the 8 Law base textures were
discarded before RDB matching.

Installed build `2026-05-12 19:33`
(`SHA256 08270B64EBF1A1556544E149ADDDA5D655D1D81B6E5E5B521FC6F9F504FDD4D6`)
re-enables the dormant alias plan:

- `MPLC026_Law.g1m` -> `MDLC033_Law_Souhi.g1m`;
- 8 `MPR_Bound_Character_MPLC026Law_*` textures ->
  `MPR_Bound_Character_MDLC033LawSouhi_*`;
- shared source assets remain disabled;
- always-active dormant assets remain disabled.

Expected next log should prove whether the current RDB-overlay-by-existing-entry
path can serve the custom `.g1m`/`.g1t` files for only slot 5. If
`custom=9 original_fallbacks=9` appears but preview remains invisible, then the
blocker is again higher-level preview/state, not file ingestion. If hashes are
missing or unresolved, resume with RDB catalog/name matching.

Follow-up from `2026-05-12_19-38-01.log`: the zip ingestion did work:

```text
Law custom slot CharacterEditor: files=1 matched=1 hash_missing=0 unresolved=0
Law custom slot MaterialEditor: files=8 matched=8 hash_missing=0 unresolved=0
Law custom slot replacements ready: custom=9 original_fallbacks=9
Law custom slot runtime published: custom=9 original_fallbacks=9
```

But the game crashed while entering the menu, and the user saw no slot 5
image/sprite. Do not overstate this as official Law/Souhi/Oni slots crashing.
The better interpretation is that the injected slot 5 UI/preview/resource path
entered an incoherent state after the dormant shared RDB alias was enabled.

The important pre-crash open was:

```text
Open virtual ... runtime=law-slot-original file=MDLC033_Law_Souhi.g1m
hash=0xc7512008 prefix=0x68 mod_size=0x1267c4
source=...\CharacterEditor.rdb.bin@0x0+0x1267c4
```

The scene was still selected on Law Oni (`selected_variant=586`,
`selected_slot=3`), but later scene state showed the slot 5 object as
`object_variant=699 object_slot=4`. No `DLC_COSTUME_006_699_026_004.bin`
request appeared; the DLC requests remained for
`DLC_COSTUME_006_586_026_003.bin`.

Crash log:

```text
Unhandled Top-Level Exception (80000003)
EXCEPTION_BREAKPOINT
RIP Addr.: OPPW4.exe+00000000003D89FCh
```

Focused Ghidra export maps the breakpoint to `FUN_1403d8560`:

```text
TARGET 1403d89fc -> function FUN_1403d8560 @ 1403d8560
...
TEST RBX,RBX
JNZ 0x1403d8a01
...
INT3
```

This looks like a generic resource construction/load failure path: if the
resource pointer stays null after fallback attempts, the game intentionally
breaks. The failed resource is not identified yet. It may be the slot image/UI
asset, the preview model, or a texture dependency.

Safety rollback installed `2026-05-12 19:53`
(`SHA256 457C1E7F11BAC664149DB7B06AA6CAD3ACC4DB2609EA4B7338754BAAFFED816B`)
disables both dormant model/material alias switches again. The next real RDB
step is not "try the shared alias again"; it is either:

- trace `FUN_1403d8560` parameters/stack to identify the exact null resource; or
- trace/repair the slot 5 UI/DLC/image path so the menu has a valid image entry
  before model/material aliasing is tested again; or
- move directly to true private RDB/model entries so no shared dormant row is
  used for slot 5 assets.

Follow-up from `2026-05-12_20-12-44.log`: the rollback confirmed the UI/image
side is safe again. Slot 5's image appears, there is no menu crash, but the
hover preview model/texture remains invisible. Startup published
`custom=0 original_fallbacks=0`, which is expected because the dormant
model/material aliases are still disabled in the rollback.

The active blocker is the preview visibility flag, not RDB ingestion:

```text
Costume preview model-update ... preview_variant=699 visible=0 effective_visible=0
```

Ghidra shows the previous scene-list flag diagnostic patched too early:
`FUN_1414926a0` calls `FUN_141493820` to rebuild the scene list, and only then
reads `entry + 0x3c + list_index`. New diagnostic installed `2026-05-12 20:25`
hooks `FUN_141493820` at `game+0x1493820` and patches the Law slot 5 list flag
after the rebuild returns.

Installed hash:

```text
C84B77842B42BF5CBAE3178EC4C5D2EAAB89A7B7882038CF1FD0A0D4F5DFD96E
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-scene-list-rebuild-post-flag.20260512-202552.dll
```

Next log should be judged by:

```text
Law custom scene-list flag diagnostic phase=scene-list-rebuild-leave ... patched=true
Costume preview model-update ... preview_variant=699 visible=1
```

Only after `visible=1` is proven should RDB custom assets be re-enabled again.

Follow-up from `2026-05-12_20-30-17.log`: `visible=1` is now proven, but the
game still shows `conditions de deblocage` for slot 5. This is not an RDB
ingestion result yet: startup still published `custom=0 original_fallbacks=0`,
so custom model/material assets remain disabled.

The relevant blocker is now scene admission:

```text
Costume variant unlock-check category=26 variant=699 slot=4 ... result=1
Costume scene scene-available-check ... result=0 ... selected_variant=699 selected_slot=4
```

Installed diagnostic `2026-05-12 20:42` forces only this scene admission check
for Law slot 5:

```text
Costume scene scene-available-check ... result=0 effective_result=1 forced=true ...
installed SHA-256 AC7E8B85D447EE4CA088E4A4B21F5255FDBE8F916C0F1FAA2FBA41F7ED3B1DCB
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-scene-available-force.20260512-204216.dll
```

Do not re-enable private/shared RDB model or material assets until this gate is
understood. The next RDB step still needs a real private row/name path, not the
old shared dormant alias.

## Name Hash Follow-Up

`FUN_6482cca0` is the name lookup path, but the current decompile shows it does
not calculate the RDB `primary_hash` directly from the file name. In the linear
catalog path, it compares the requested name against catalog strings, then calls
`FUN_64829050(param_1, stored_hash)` with the hash stored beside the matched
name. In the hashed catalog path, it hashes the name only to find the catalog
bucket, then again calls `FUN_64829050` with the stored catalog hash.

`FUN_648f5630(name, len, 0xc70f6907)` is the catalog bucket hash. It is a
MurmurHash2-style 64-bit hash using multiplier `0x5bd1e995`, 8-byte little
endian chunks, and a `>> 47` finalization. It does not match known RDB
`primary_hash` values even when truncated to high or low 32 bits:

- `MPLC026_Law.g1m`: catalog-bucket hash `0x61c8cb09a6796139`, RDB hash
  `0x8df2d8cb`;
- `800_294_face_law_dressrosa_External_00.g1t`: catalog-bucket hash
  `0xda5f75c9d6485967`, RDB hash `0x359b9672`.

So the RDB `primary_hash` is still a separate value sourced from the embedded
name/hash catalog or the parsed RDB entry. A private asset name still needs
either a known/derived RDB `primary_hash`, or a real appended RDB index entry
whose `primary_hash` we choose and whose lookup path can reach it.

### OPPW4.exe Hash Check, 2026-05-16

Direct checks in `OPPW4.exe` did not find embedded Law asset names or known RDB
primary hashes:

- no `MPLC026_Law.g1m` string;
- no `.g1m`, `.g1t`, `.kscl`, `.grp`, or `.mtl` asset-name strings in the exe;
- no little-endian or big-endian occurrences of known Law/preview hashes such as
  `0x8df2d8cb`, `0x966d6276`, `0x359b9672`, `0x3bff0f13`, or `0x386d71a0`.

The exe does contain RDB archive names such as `CharacterEditor.rdb`,
`MaterialEditor.rdb`, and `ScreenLayout.rdb`. The main string reference found so
far is `FUN_1415fb060`, which loops through archive names and loads them from the
game data path.

The CRC polynomial `0xedb88320` exists at `OPPW4.exe+0x3bdb90`
(`FUN_1403bdb60`), but the decompile shows a three-`u32` CRC-combine style
helper, not a string-to-RDB-hash function. Ghidra currently shows only a data
pointer reference to that helper, not a direct RDB asset lookup call.

`FUN_1415fb060` also contains a simple string hash for project/shader resource
names:

```text
acc += signed_char(name[i]) * (31 ** (i + 1))
```

This does not match known RDB primary hashes either; for example
`MPLC026_Law.g1m` gives `0x1784098e`, not `0x8df2d8cb`.

Current conclusion: `OPPW4.exe` is not embedding or deriving the Law RDB
`primary_hash` from plain asset names in the obvious paths. For private slot-5
assets, the practical path remains an actual RDB-visible entry/catalog mapping
with a chosen/known primary hash, instead of relying on a hidden exe hash formula.

## Useful Scripts Created

- `oppw4-ghidra/FindPatcherStrings.java`
  - Finds refs to important patcher log/config strings.
- `oppw4-ghidra/DecompileTargets.java`
  - Dumps selected function pseudo-code to console.
- `oppw4-ghidra/ExportVirtualizationFunction.java`
  - Exports the virtualization function and key callees to `decompile_64803ed0.txt`.
- `oppw4-ghidra/ExportResourceManagerTargets.java`
  - Exports model resource manager functions to `game_resource_manager_targets.txt`.
- `oppw4-ghidra/ExportGameHashTargets.java`
  - Exports the focused `OPPW4.exe` CRC/hash candidate around `FUN_1403bdb60`.
- `oppw4-ghidra/ExportGameRdbStringRefs.java`
  - Exports `OPPW4.exe` references to `.rdb` archive-name strings.
