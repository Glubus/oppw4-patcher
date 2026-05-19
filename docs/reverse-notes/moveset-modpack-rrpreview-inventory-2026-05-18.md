# Moveset Modpack RRPreview Inventory

Date: 2026-05-18

Source folder:

```text
D:\SteamLibrary\steamapps\common\OPPW4\Modded Edition
```

## Corrected Finding

The initial RRPreview lead was wrong for the actual movesets. The loose
`RRPreview` files are audio/BGM/voice resources. The 28 moveset edits are in:

```text
Modded Edition\LINKDATA\CMN\LINKDATA_A.BIN
```

This matches external modder confirmation:

```text
LINKDATA_A.BIN is the moveset file
```

Comparing vanilla `LINKDATA_A.BIN` against the modpack version finds exactly
`28` real payload changes, matching the modpack changelog `[28 MOVESET MODS]`.

Changed LinkData entries:

```text
68, 69, 72, 77, 80, 82, 84, 85, 86, 87, 89, 90, 91, 93, 94, 97,
106, 107, 108,
208, 209, 212, 213,
229, 231, 233,
247, 248
```

Unresolved after the Oden confirmation: `229`, `231`.

Extraction workspace:

```text
C:\tmp\oppw4-moveset-linkdata-extract
```

Extracted folders:

```text
vanilla\
moveset_full\
without_garp_rayleigh\
summary.csv
entry_map_seed.csv
```

Each changed entry was extracted as `entry_00XX.bin`, inflated/unpacked from
LinkData. These are the future patch-at-runtime units.

## Garp/Rayleigh Split

The user provided a `LINKDATA_A.BIN` variant without Garp and Rayleigh:

```text
C:\Users\Osef\Downloads\LINKDATA_A(1).BIN
```

Diff result:

```text
full moveset pack changes:        28 entries
without Garp/Rayleigh changes:    26 entries
removed only in without variant:  247, 248
```

Therefore entries `247` and `248` are Garp/Rayleigh. The exact order still needs
confirmation. Working guess from changelog order is:

```text
247 = Garp
248 = Rayleigh
```

Do not treat that order as confirmed until a runtime test or stronger ID marker
is found.

`characters.json` received from the user (`C:\Users\Osef\Downloads\characters.json`)
contains these relevant ids:

```text
garp          playable=48  runtime=25   boss=25   model=25   MPLC025_Garp
garp_yng      playable=179 runtime=76   boss=25   model=304  MDLC065_Garp_YNG
rayleigh_yng  playable=180 runtime=77   boss=116  model=305  MDLC066_Rayleigh_YNG
```

The moveset pack changelog says `Garp` and `Rayleigh`, but the current
`characters.json` only has a Rayleigh entry for `rayleigh_yng`. So when mapping
these entries, keep both possibilities in mind:

```text
247/248 may target Garp/Rayleigh DLC-style playable rows, not necessarily base Garp.
```

The entry ids themselves (`68`, `69`, ... `248`) do not directly equal the
known `playable_id`, `runtime_id`, `boss_runtime_id`, or `model_id` values.
They are LinkData table entry indices and must be mapped separately.

Runtime test update:

```text
69 = Zoro (New World) confirmed in-game on 2026-05-18.
208 = Luffy Bounceman confirmed in-game on 2026-05-18.
209 = Luffy Snakeman confirmed in-game on 2026-05-18.
212 = Big Mom Temperamented Form probable after retest/reclassification on 2026-05-18.
213 = Kaido Dragon Form corrected after in-game retest on 2026-05-18.
229 = Urouge confirmed in-game on 2026-05-19.
231 = Okiku / Kiku confirmed in-game on 2026-05-19.
233 = Oden confirmed in-game on 2026-05-18.
```

All 28 changed moveset entries from the RRPreview modpack now have a candidate
mapping. The final unresolved pool (`229`, `231`, `233`) was closed by runtime
tests.

Naive integer scanning inside the inflated payloads is noisy because the
payloads contain many zero/small values. Do not infer character ownership from
simple `u16 == id` frequency alone.

## LinkData Moveset Entry Structure Notes

The inflated moveset payloads are not arbitrary blobs. They begin with a section
table:

```text
u32 section_count = 18
u32 section_offsets[18]
u32 zero/end marker
```

Each section then starts with a small header. For the known moveset entries, the
first `u32` of a section is usually a record count. The record sizes inferred
from section boundaries are stable:

```text
section 00: 0x10-byte records
section 01: 0x60-byte records
section 02: 0x20-byte records
section 03: 0x20-byte records
section 04: 0x10-byte records
section 05: 0x40-byte records
section 06: 0x20-byte records
section 07: 0x40-byte records
section 08: empty in observed entries
section 09: 0x40-byte records
section 10: 0x30-byte records
section 11: empty in observed entries
section 12: 0x30-byte records
section 13: 0x20-byte records
section 14: 0x20-byte records
section 15: 0x10-byte records
section 16: 0x10-byte records
section 17: 0x10-byte records, sometimes empty
```

The first section looks like an action/state index. Its records are 16 bytes:

```text
u32 internal_action_id
u32 action_slot_or_kind
u32 flags_or_zero
u32 internal_action_id_duplicate
```

Example for entry `247` full pack:

```text
section_count = 18
section00 count = 283
first action id = 0x0000a410
```

Example for entry `248` full pack:

```text
section_count = 18
section00 count = 279
first action id = 0x00007530
```

Important clue: entry `248` shares the same first action id (`0x7530`) and very
similar section counts with entry `97`. That suggests entry `248` may be a
Rayleigh moveset derived from an existing Shanks-like moveset block, which fits
the changelog order and the sword-user theme. This strengthens, but does not
fully prove:

```text
247 = Garp
248 = Rayleigh
```

## RRPreview Side Finding

The modpack contains many model and texture edits. The loose `RRPreview` files
looked promising at first:

Relevant files:

```text
Modded Edition\File\CMN\AssetRelease\Retail\RRPreview.rdb
Modded Edition\File\CMN\AssetRelease\Retail\RRPreview.rdb.bin_9
Modded Edition\OPPW4_PATCHER\RRPreview\0x272c6efb.srst
Modded Edition\OPPW4_PATCHER\RRPreview\0x3a160928.srsa
Modded Edition\OPPW4_PATCHER\RRPreview\0xe469ba0e.srsa
Modded Edition\OPPW4_PATCHER\RRPreview\0xe917add2.srsa
```

## RDB Diff Summary

Comparing vanilla `RRPreview.rdb` against the modded `RRPreview.rdb` by
`primary_hash` shows:

- vanilla blocks: `31153`
- modded blocks: `31153`
- changed logical entries: `34`

All changed entries point to `&9` tails, meaning the changed data is served from:

```text
RRPreview.rdb.bin_9
```

The generated CSV inventory is:

```text
moveset-rrpreview-diff.csv
```

Those RRPreview changes are useful for effects/audio, but they are not the full
moveset pack.

## Named Changed Entries

The changed `RRPreview.rdb` entries currently resolve mostly to Sanji and Mihawk
effect/action resources:

- `CE1_0104_SAN_*`
- `CE1_0114_MIH_SLASH_WAVE.*`
- `CE1_0043_TYPE_FORCE_AURA*`
- `CE1_0112_NEW_SHOCK_AURA*`

Examples:

```text
0x03dfd1c8 CE1_0104_SAN_DIABLE.g1e
0x071edec1 CE1_0104_SAN_KICK.g1e
0xbd6c349f CE1_0104_SAN_KICK_DIABLE.g1e
0xd50b2057 CE1_0114_MIH_SLASH_WAVE.g1e
0xc115530c CE1_0114_MIH_SLASH_WAVE.ktid
```

## Loose OPPW4_PATCHER RRPreview Files

The old patcher folder includes four direct `RRPreview` replacements.

Running the archive scanner against both vanilla and modded RDB roots gives the
same result:

```text
archive RRPreview: files=4 matched=4 missing=0 unresolved=0
virtualization_table: replacements=4 total_mod_bytes=322777264
```

So the old `OPPW4_PATCHER/RRPreview` files are valid direct hash replacements.
They do not require catalog names to resolve.

### `0xe469ba0e.srsa`

- size: `0x245bc0`
- original RDB tail: `16750000@1035bc`
- magic: `ASRS`
- contains readable action names for `Av018`
- examples:
  - `Av018_ATTACK_0`
  - `Av018_EX_ACTION_00`
  - `Av018_SP_START`

This likely maps to one playable character/action bank.

Working guess: `Av018` may line up with character/model id `018` (`Kizaru` in
the asset catalog), but this still needs runtime confirmation.

### `0xe917add2.srsa`

- size: `0xcae50`
- original RDB tail: `6dfbbb0@94616#8`
- magic: `ASRS`
- contains readable action names for `Av036`
- examples:
  - `Av036_ATTACK_0`
  - `Av036_EX_ACTION_00`
  - `Av036_SP_START`

This likely maps to one playable character/action bank.

Working guess: `Av036` may line up with character/model id `036` (`Shanks` in
the asset catalog), but this still needs runtime confirmation.

### `0x3a160928.srsa`

- size: `0x29bd0`
- original RDB tail: `938@3e46`
- magic: `ASRS`
- no obvious `Av###_...` strings in the first scan
- starts with a readable `bgm001` entry near offset `0x80`, so it is a valid
  `ASRS/KTSR` resource, but probably not an action-bank file like the two below.
- original RDB size: `0x3e46`
- modded size: `0x29bd0`
- `SRSxtool_1.0` extracts it as an audio index:
  `500` `.subd` files plus `500` `.ext_audio` placeholders.

### `0x272c6efb.srst`

- size: `0x13098ad0`
- original RDB entry is a special zero-like payload:
  `data_offset=0x11cbc120`, payload bytes `00 00 00 00 00 00 00 00`
- magic: `TSRS`
- second magic: `KTSR`
- huge container-style replacement
- does not match a raw slice of vanilla `RRPreview.rdb.bin`
- contains `500` `KOVS` blocks, each followed by an `OfeP` payload marker
- has no clean `Av###_...` strings in a full binary scan
- has a small number of accidental-looking `Av0` / `CE1_` / `G1E` byte hits,
  but no readable action labels like `_ATTACK`
- replacement probe first bytes: `TSRS....`
- first compression sanity check failed for `zlib`, raw deflate, gzip, `bz2`,
  and `lzma` from the obvious payload offsets, so `OfeP` is likely a KT/Koei
  container codec or an encrypted/packed stream rather than a stock wrapper.
- `SRSxtool_1.0` can extract it when `0x3a160928.srsa` is copied/renamed next
  to it as `0x272c6efb.srsa`.
- Extracted output: `500` `.ogg` files named `bgm001.ogg` through
  `bgm500.ogg`, total extracted bytes `319355248`.

Conclusion: this pair is BGM/audio, not the bulk of the 28 moveset mods.

Extraction workspace used for the successful test:

```text
C:\tmp\oppw4-srsxtool-test-0x3a160928\0x272c6efb
```

Important naming quirk:

```text
0x272c6efb.srst
0x3a160928.srsa -> copy/rename to 0x272c6efb.srsa for SRSxtool
```

## `RRPreview.rdb.bin_9` Payload Check

The 34 changed RDB entries in `RRPreview.rdb.bin_9` are small `IDRK0000`
payloads. They are useful, but they do not look like the full moveset pack.

Observed payload starts:

```text
CE1_0104_SAN_DIABLE.g1e              -> IDRK0000, size 0x40a8
CE1_0104_SAN_KICK.g1e                -> IDRK0000, size 0x40a8
CE1_0104_SAN_CROSSE_STRIKE.g1e       -> IDRK0000, size 0x847
CE1_0043_TYPE_FORCE_AURA.g1e         -> IDRK0000, size 0xe07
CE1_0114_MIH_SLASH_WAVE.g1e          -> IDRK0000, size 0x8b28
```

Two RDB tails currently decode to offsets outside the physical
`RRPreview.rdb.bin_9` size:

```text
0x05086ee4 CE1_0112_NEW_SHOCK_AURA.g1e  C40b0@97c8&9
0xfdd1f9a0 CE1_0104_SAN_SKY_WALK.g1e    c30160@40a8&9
```

Those may be flag-encoded offsets rather than plain bin offsets. Do not treat
them as missing files yet.

## Old RRPreview Hypothesis Rejected

The clean audio/BGM extraction target is:

```text
RRPreview/
  0x272c6efb.srst
  0x3a160928.srsa
  0xe469ba0e.srsa
  0xe917add2.srsa
```

plus, for this specific modpack version:

```text
File/CMN/AssetRelease/Retail/RRPreview.rdb
File/CMN/AssetRelease/Retail/RRPreview.rdb.bin_9
```

The `.srsa` files are readable enough to confirm voice/action-label data. The
large `.srst` looked suspicious at first because it is huge, but `SRSxtool`
proves it is BGM/audio.

This means the actual 28 moveset mods are not in the four loose
`OPPW4_PATCHER/RRPreview` replacements. They are in `LINKDATA_A.BIN`.

## CharacterEditor.rdb Moveset Link Hint

`CharacterEditor.rdb` contains at least one strong link back to the moveset
entry table. Searching for little-endian `247` finds exactly one hit in both
vanilla and the modpack:

```text
CharacterEditor.rdb block hash: 0x0555b895
block offset vanilla: 0x00000e68
block offset modpack: 0x00000e64
data_offset: 0x001ffc60
tail: 3f20000@1225e9
```

Payload words:

```text
w00 = 0x00000000
w01 = 0x00000003
w02 = 0x00000005
w03 = 0x00000001
w04 = 0xd2d2d5af
w05 = 0x00000005
w06 = 0x00000001
w07 = 0xbaf0df79
w08 = 0x00000004
w09 = 0x00000001
w10 = 0xd3c00659
w11 = 0xf74ff20e
w12 = 0x00000000
w13 = 0xffffffff
tail = 3f20000@1225e9
```

Correction: `247` is **not** a standalone `u32` here. The byte `0xf7` is the
high byte of `w11 = 0xf74ff20e`. Across `CharacterEditor.rdb`, this common
record family appears as `14` little-endian `u32` words followed by an ascii
RDB address tail. In `37` records, the high byte of `w11` matches one of the
changed moveset entry ids. This may be a packed/composite id field, but it
could still be a coincidental byte match until confirmed against more known
characters.

Structured dump:

```text
target/reverse/rdb-charactereditor-format/README.md
target/reverse/rdb-charactereditor-format/charactereditor_common_family_w11_high_byte.csv
```

## Next Steps

1. Map each changed LinkData entry to a character.
2. Confirm `247/248` order by testing Garp/Rayleigh or finding stronger markers.
3. Build a LinkData runtime patcher that replaces only selected inflated
   entries instead of copying the entire `LINKDATA_A.BIN`.
4. Define a moveset mod format that stores entry id + patched payload.
5. Keep RRPreview `.srsa/.srst` support separately for audio/BGM/voice mods.

## Garp Entry 247 Isolated Test Build

Structured dump:

```text
target/reverse/linkdata-moveset-garp-entry247/README.md
target/reverse/linkdata-moveset-garp-entry247/modded_sections.csv
target/reverse/linkdata-moveset-garp-entry247/modded_section_00_records.csv
...
```

Generated test LinkData that replaces only entry `247` with the modded Garp
payload while leaving every other vanilla entry unchanged after inflation:

```text
target/reverse/linkdata-moveset-garp-entry247/LINKDATA_A.entry247_garp_only.raw-test.BIN
```

Verification:

```text
entry247_size=282752
entry247_sha=5837ec96745b2389e66e757ed53380b0c7394ee4de928ca2a681a0d3eb2bfa94
match=true
```

This file is intentionally a raw rebuilt LinkData test artifact. It is not yet
the final runtime patcher format.





