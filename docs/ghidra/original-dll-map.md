# OPPW4 Original dinput8.dll - Ghidra Map

This document summarizes what is known about the original OPPW4 patcher
`dinput8.dll` from Ghidra exports, decompilation output, and runtime logs.

The goal is not polished end-user documentation. This is a working map for
rebuilding the RDB replacement pipeline safely: which function does what, which
global table is likely involved, what is confirmed, what is inferred, and what
still needs a dedicated Ghidra pass.

## Local Sources

- Analyzed DLL: `oppw4-ghidra/dinput8.dll`
- Main reverse notes: `OPPW4_REVERSE_NOTES.md`
- Decompile exports:
  - `oppw4-ghidra/decompile_64803ed0.txt`
  - `oppw4-ghidra/decompile_rdb_targets.txt`
  - `oppw4-ghidra/decompile_next_targets.txt`
  - `oppw4-ghidra/decompile_table_targets.txt`
  - `oppw4-ghidra/audit_interesting.txt`
- Audit script added for this investigation: `oppw4-ghidra/AuditInteresting.java`

## DLL Identity

- Type: PE32+ x86-64.
- Ghidra image base: `0x64800000`.
- PE timestamp: `Wed Apr 12 08:23:04 2023`.
- Visible role: DirectInput proxy plus virtual file patcher for OPPW4.
- Classic DirectInput exports:
  - `DirectInput8Create`
  - `DllCanUnloadNow`
  - `DllGetClassObject`
  - `DllMain`
  - `DllRegisterServer`
  - `DllUnregisterServer`

The DLL loads the real `C:\Windows\system32\dinput8.dll`, then forwards the
DirectInput exports to the original implementation.

## High-Level Behavior

The original patcher does four main things:

1. Finds the game directory and builds paths for:
   - `File/CMN/AssetRelease/Retail/`
   - `File/CMN/AssetRelease/Retail/data/`
   - `OPPW4_PATCHER/`
2. Scans mod folders by RDB archive:
   - `CharacterEditor`
   - `MaterialEditor`
   - `KIDSSystemResource`
   - `FieldEditor4`
   - `RRPreview`
   - `ScreenLayout`
   - `system`
   - `SequenceEditor`
3. Patches in-memory RDB records that have matching mod files.
4. Installs IAT hooks on Windows file APIs so that `data/<hash>.file` can be
   served from virtual content.

Important point: the game believes it is reading normal files from `data/`, but
the DLL intercepts those opens and can return:

- a mod file with a rebuilt RDB prefix;
- an internal slice of `.rdb.bin`;
- an external/original file;
- the original Windows file handle when nothing matches.

## Debug Configuration

The original patcher reads debug flags from an ini file and stores them in
globals.

| Ini key | Global | Address | Effect |
| --- | --- | ---: | --- |
| `log_virtual` | `DAT_650e3a00` | `0x650e3a00` | Logs virtual opens |
| `log_external` | `DAT_650e3a01` | `0x650e3a01` | Logs external opens |
| `log_internal` | `DAT_650e3a02` | `0x650e3a02` | Logs internal opens |
| `log_numeric` | `DAT_650e3a03` | `0x650e3a03` | Logs detailed reads and numeric requests |

Confirmed related strings:

- `Open virtual %s`
- `Open external %s`
- `Open internal %s`
- `.rdb.bin`
- `.file`
- `data/`

## Important Global Tables

| Global | Address | Likely/confirmed role |
| --- | ---: | --- |
| `DAT_650e3a08` | `0x650e3a08` | Root path for `File/CMN/AssetRelease/Retail/data/` |
| `DAT_650e3a10` | `0x650e3a10` | Root path for `OPPW4_PATCHER/` |
| `DAT_650e3a60` | `0x650e3a60` | Virtual replacement table |
| `DAT_650e3aa0` | `0x650e3aa0` | Observed external-data table/log |
| `DAT_650e3ae0` | `0x650e3ae0` | Tracked `.rdb.bin` handles and archives |
| `DAT_650e3b20` | `0x650e3b20` | Scanned RDB archive contexts |
| `DAT_650e3b60` | `0x650e3b60` | Active virtual handles |
| `DAT_650e3ba0` | `0x650e3ba0` | Debug aliases for `.rdb.bin` handles |
| `DAT_650e3be0` | `0x650e3be0` | Real-handle read tracking for debug logs |

Confirmed points:

- `DAT_650e3a60` is checked by the `CreateFileW` hook to decide whether a
  `data/<hash>.file` path should become a virtual handle.
- `DAT_650e3b60` maps fake handles of the form `0x1000000000000000 + n` to
  virtual read state.
- `DAT_650e3b20` contains archive contexts and also helps recognize `.rdb` and
  `.rdb.bin` files opened by the game.
- `DAT_650e3be0` is mainly for logging real reads when debug flags are enabled.
  It is not the content replacement table.

## Bootstrap and Hook Installation

### `FUN_64805450` - Main Patcher Init

Confirmed role:

- reads debug flags;
- builds root paths;
- scans archives through `FUN_64803ed0`;
- installs IAT hooks through `FUN_64833aa0`.

Hooks installed by the original patcher:

| Windows API | Normal hook | Debug/proxy hook |
| --- | --- | --- |
| `CreateFileW` | `FUN_64803020` | `FUN_64806970` |
| `ReadFile` | `FUN_64803c60` | `FUN_64806eb0` |
| `CloseHandle` | `FUN_64803d60` | `FUN_64806db0` |
| `GetFileSizeEx` | `FUN_64802fc0` | same hook |
| `GetFileTime` | `FUN_64802f00` | same hook |

If hook installation fails, the DLL calls `exit(-1)`.

### `FUN_64833aa0` / `FUN_648339c0` / `FUN_64833690` - IAT Hook Engine

Confirmed role:

- `FUN_648339c0` looks up an import entry by module plus name or ordinal.
- `FUN_64833690` uses `VirtualProtect`, replaces the IAT pointer, then restores
  protection.
- `FUN_64833aa0` is the high-level wrapper called during init.

## Mod Scanner

### `FUN_64803ed0` - Scan and Register Replacements

This is one of the most important functions in the DLL.

Confirmed role:

- enumerates `OPPW4_PATCHER/<Archive>`;
- tries to resolve each mod file to an RDB entry:
  - direct `0xHASH.ext` file names through `FUN_64802ae0`;
  - catalog names through `FUN_6482cca0`;
- retrieves the original RDB entry;
- copies or patches parts of that entry;
- builds a virtual prefix;
- appends a record into `DAT_650e3a60`.

Important observed operations:

- writes `0x10000` into the `+0x2c` field;
- calculates mod file size;
- writes mod size into a size field;
- logs `File "%s" added to virtualization.`;
- logs `Notice: file "%s" doesn't exist in this .rdb (%s.rdb)` when unresolved.

### `FUN_64802ae0` - Hash Parser for File Names

Confirmed role:

- accepts names starting with `0x`;
- parses hex digits until `.` or end of name;
- returns a 32-bit hash.

Used for files named directly by hash, for example `0x93dfb06c.g1t`.

## File Hooks

### `FUN_64803020` - `CreateFileW` Hook

Confirmed role:

- only special-cases `GENERIC_READ` opens (`0x80000000`);
- normalizes the opened path;
- checks whether the path matches:
  - an archive/index file;
  - a `.rdb.bin`;
  - a replaced `data/<hash>.file`;
- returns either a real Windows handle or a fake virtual handle.

Confirmed virtual case:

1. The game opens `File/CMN/AssetRelease/Retail/data/<hash>.file`.
2. The hook finds `<hash>` in `DAT_650e3a60`.
3. The DLL opens the mod file from `OPPW4_PATCHER/<Archive>/...`.
4. It builds a virtual state of about `0x40` bytes.
5. It returns a fake handle like `0x1000000000000000 + id`.

Observed virtual state layout:

| Offset | Meaning |
| ---: | --- |
| `+0x00` | pointer to virtual prefix |
| `+0x08` | prefix length |
| `+0x10` | real backing file handle |
| `+0x18` | current virtual read position |
| `+0x20` | backing size or range |
| `+0x28` | backing offset or related field |
| `+0x30` | possible mode/state field |

Important correction: mod files found directly in `OPPW4_PATCHER/<Archive>` are
registered as virtual replacements. The observed `external` and `internal`
modes in `CreateFileW` are not a filter that says a mod file should be ignored.
They are alternate read paths for records that already point at an existing
backing location.

Observed decision in `FUN_64803020` after a replacement lookup:

1. The opened path is checked against the archive table `DAT_650e3b20`.
2. If that does not match, the path is checked against the replacement table
   `DAT_650e3a60`.
3. If no record is found, normal Windows read path is used.
4. If a record is found and `record+0x50 == 0`:
   - log `Open virtual`;
   - open the mod file;
   - serve `patched RDB prefix + mod payload`.
5. If a record is found and `record+0x50 != 0`:
   - if `byte(record+0x42) & 1 == 0`, log `Open internal` and open an internal
     range;
   - otherwise log `Open external` and open the external file.

Implication for Law and RRPreview:

- `CE1_0080_STYLE_IMPACT_HAO.g1e` and `.ktid` are normal virtual replacements.
- Law Dressrosa portrait files are also expected to use the virtual route, but
  only if their names resolve to the real `.g1t` hashes.
- Leaving alias records as passthrough avoids crashes, but it also prevents the
  intended replacement.

### `FUN_64802d30` - Virtual Reader

Confirmed role:

- reads from the virtual prefix first;
- then reads from the backing file;
- tracks current position;
- writes the number of bytes read.

The original function accepts an `OVERLAPPED` pointer and checks the event field
at `overlapped+0x10`, but non-null events appear to go through a warning or
fallback path. In practice, overlapped-with-event is detected but not fully
implemented.

### `FUN_64803c60` - `ReadFile` Hook

Confirmed role:

- if the handle is virtual (`DAT_650e3b60`), calls `FUN_64802d30`;
- otherwise calls the real `ReadFile`;
- can log real reads through `DAT_650e3be0`.

### `FUN_64803d60` - `CloseHandle` Hook

Confirmed role:

- if the handle is virtual:
  - closes the real backing handle;
  - frees or clears the virtual state;
- otherwise calls the real `CloseHandle`;
- also clears debug tracking.

### `FUN_64802fc0` - `GetFileSizeEx` Hook

Confirmed role:

- if the handle is virtual, returns `prefix_len + backing_size`;
- otherwise calls the real `GetFileSizeEx`.

### `FUN_64802f00` - `GetFileTime` Hook

Confirmed role:

- if the handle is virtual and has a valid backing handle, returns backing file
  times;
- otherwise calls the real `GetFileTime`.

## RDB Parsing and Lookup

### `FUN_6482fcb0` - RDB File Parser

Confirmed role:

- loads and parses an RDB file;
- checks signatures:
  - root `_DRK0000`;
  - blocks `IDRK0000`;
- reads header fields:
  - first block offset;
  - declared block count;
  - data prefix, usually `data/`;
- builds parsed entries of size `0x68`.

### `FUN_6482cca0` - Catalog Name Lookup

Likely/confirmed role:

- resolves a file name to an RDB entry/hash;
- uses the embedded name catalog.

### `FUN_6482b1b0` - Materialize or Read an RDB Entry

Likely/confirmed role:

- builds `data/<hash>.file` for external entries;
- reads from `.rdb.bin` or from an external file;
- probably handles decompression;
- uses `entry+0x4e & 0x10` for a compressed/chunked path;
- calls `FUN_64815c30` inside the decompression loop.

This function is probably the best source for understanding exactly how the
game expects small entries and externalized blobs to be shaped.

## Structure Notes

There are at least two structures that must not be confused:

1. Parsed RDB entry, observed size `0x68`.
2. Virtual replacement record stored in `DAT_650e3a60`.

Some offsets look similar in decompilation output, but they do not necessarily
refer to the same structure.

### Parsed RDB Entry, Working Notes

| Offset | Likely meaning |
| ---: | --- |
| `+0x18` | payload or external size |
| `+0x24` | hash used for `data/<hash>.file` in some paths |
| `+0x2c` | flags, often patched to `0x10000` for virtual replacements |
| `+0x30` | likely internal/external flag |
| `+0x42` / `+0x4e` | mode or compression-related flags |
| `+0x44` | primary hash used for lookup |
| `+0x48` / `+0x50` | backing offset/size or related fields |

### Virtual Replacement Record, Working Notes

Confirmed by usage in `FUN_64803ed0` and `FUN_64803020`:

| Field | Likely meaning |
| --- | --- |
| original entry | pointer to the patched RDB entry |
| data path | normalized `data/<hash>.file` |
| mod path | source file under `OPPW4_PATCHER/<Archive>` |
| backing info | original offset/size or external path metadata |
| prefix ptr/len | rebuilt prefix served through the virtual handle |
| mode flag | distinguishes pure virtual, internal, and external paths |

The exact layout still needs a dedicated Ghidra pass.

## What This Explains in Our Rust Logs

The old Rust logs showed many `RDB BIN HIT ScreenLayout` events around
`0x93dfb06c.g1t` and `0x7fb56ccf.g1t`, but the initial Law hit was on a tiny
`0xa4` blob. That looked like a metadata alias problem.

Later comparison showed the real root cause:

- the exported name/hash catalog was shifted by one entry;
- `800_294_face_law_dressrosa_External_00.g1t` was incorrectly mapped to
  `0xa46e63e6`, which is a small `.texinfo`/descriptor-like entry;
- the real `.g1t` hash in `ScreenLayout.rdb` is `0x359b9672`, with original
  blob `88ddf6@723f` and `data_offset=0x20038`;
- `801_294_chara_law_dressrosa_External_00.g1t` was incorrectly mapped to
  `0x2cb16d09`;
- the real `.g1t` hash in `ScreenLayout.rdb` is `0x3bff0f13`, with original
  blob `b770000@60fa2` and `data_offset=0x200038`.

Conclusion: the priority Law fix was not to serve a texture through the short
descriptor entry. It was to regenerate the catalog in the correct
`0xHASH,name` format so that names resolve to the real `.g1t` records.

With the corrected catalog:

- Law Dressrosa files are expected to go through `Open virtual`;
- the virtual route is the same route used by the original DLL for mod files
  found under `OPPW4_PATCHER/<Archive>`;
- using the shifted catalog made the Rust loader virtualize the wrong
  descriptors.

## Functions Worth Renaming in Ghidra

Suggested names:

| Address/name | Suggested name |
| --- | --- |
| `FUN_64805450` | `patcher_init` |
| `FUN_64803ed0` | `scan_archive_mod_folder` |
| `FUN_64803020` | `hook_CreateFileW` |
| `FUN_64803c60` | `hook_ReadFile` |
| `FUN_64803d60` | `hook_CloseHandle` |
| `FUN_64802fc0` | `hook_GetFileSizeEx` |
| `FUN_64802f00` | `hook_GetFileTime` |
| `FUN_64802d30` | `read_virtual_handle` |
| `FUN_64802ae0` | `parse_hash_filename` |
| `FUN_64833aa0` | `install_iat_hooks` |
| `FUN_648339c0` | `find_import_entry` |
| `FUN_64833690` | `patch_iat_slot` |
| `FUN_6482fcb0` | `parse_rdb_file` |
| `FUN_6482cca0` | `lookup_rdb_entry_by_name` |
| `FUN_6482b1b0` | `read_or_materialize_rdb_entry` |

## Remaining Questions

1. Exact layout of records in `DAT_650e3a60`.
2. Exact layout of the parsed `0x68` RDB entry, with clean separation between
   offsets, sizes, flags, and hashes.
3. Exact handling of:
   - pure virtual replacement;
   - internal range;
   - external file.
4. Whether the original DLL patches any additional hash/name table around alias
   entries.
5. Exact handling of compressed/chunked entries through `FUN_64815c30`.

## Useful Next Ghidra Pass

Recommended next pass:

- apply manual structs to `FUN_64803ed0` and `FUN_64803020`;
- export every access to `DAT_650e3a60` with offsets;
- verify exactly how the virtual prefix is allocated and copied;
- focus on `FUN_6482b1b0` and `FUN_64815c30`;
- document the `0x68` parsed-entry format;
- confirm compression, external, and internal flags.

## Current Summary

We already have the important pieces of the original DLL mapped: bootstrap,
archive scanner, file hooks, replacement table, virtual handles, and a good part
of the RDB parser.

The main missing piece is no longer "where is the logic". It is the exact layout
of the replacement and RDB-entry structs, especially around metadata aliases and
mode flags.
