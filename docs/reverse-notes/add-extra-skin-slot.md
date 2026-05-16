# Add Extra Costume Slot Notes

Last updated: 2026-05-13

## Goal

The current feature branch is trying to add a new costume/skin slot instead of
replacing an existing asset such as Law Dressrosa.

This is different from the legacy patcher behavior. The old DLL virtualizes RDB
files by hash/name and can replace existing entries, but we have not found any
evidence that it can append new playable costume slots.

The preferred route is data-driven first:

- find the game's own data table that maps character/costume slots to model and
  UI resources;
- patch or extend that data from mods;
- avoid hard-coded RAM patching until we know the data shape.

## Replacement Baseline

Existing replacement now works after fixing the embedded name/hash catalog.

Important result:

- the previous Law portrait problem was caused by a shifted exported catalog;
- `800_294_face_law_dressrosa_External_00.g1t` was resolving to the wrong short
  descriptor-like hash;
- the corrected catalog maps names to the real `.g1t` RDB entries;
- after correction, regular replacements are stable.

This matters because extra slots must build on the same two foundations:

- correct name/hash catalog;
- correct RDB virtualization route.

## External Research Status

Public indexed mod pages mostly document replacement mods, 3Dmigoto texture
injection, or generic mod loaders.

No public indexed English/Japanese/Korean/Chinese write-up has been found that
proves a full "append a new costume slot without replacing another slot" method
for OPPW4.

The useful public clue is indirect: advanced mods exist, but the actual slot
injection method is not documented publicly enough to reuse.

## LinkData Parsing

Main file under investigation:

```text
D:/SteamLibrary/steamapps/common/OPPW4/LINKDATA/CMN/LINKDATA_A.BIN
```

Observed format:

- magic: `0x00077df9`;
- index records start at `0x10`;
- record size: `0x10`;
- record field `+0x00`: data offset in `0x100` units;
- record field `+0x08`: compressed span-like value;
- record field `+0x0c`: declared uncompressed size.

Inflated entry payloads use chunked zlib:

```text
u32 declared_total_size
repeated {
  u32 compressed_chunk_size
  zlib_chunk
}
```

`crates/oppw4-rdb/src/linkdata.rs` contains the current parser and inflater.

## LinkData File Scope

Useful strings for this investigation currently appear only in:

```text
LINKDATA/CMN/LINKDATA_A.BIN
```

Checked groups:

- `CMN/LINKDATA_A.BIN`;
- `CMN/LINKDATA_B.BIN`;
- `CMN/LINKDATA_C.BIN`;
- `CMN/LINKDATA_D.BIN`;
- `CMN/LINKDATA_PLATFORM_DX11.BIN`;
- `CMN/EX/MASTER/LINKDATA_EX_MASTER.BIN`;
- `PATCH/000/LINKDATA_PATCH_000.BIN`;
- language LinkData files;
- region LinkData files.

Only `LINKDATA_A.BIN` produced useful hits for anchors such as:

- `DLC_COSTUME`;
- `MVAR025_Law_DR`;
- `MDLC033_Law_Souhi`;
- `MDLC069_Law_Oni`;
- `806_058_costume_law_dressrosa`.

Current conclusion: the costume/model registry and DLC labels we need are in
`LINKDATA_A`. Other LinkData files may still affect text/localization/platform
behavior, but they are not where the model/layout mapping strings live.

Full-scan note:

- `LINKDATA_A` header reports `0xe7a` entries (`3706`);
- early probes only covered the low entries, so full targeted scans are needed
  for every serious anchor;
- a full scan for known Law/Newgate layout IDs found 21 entries;
- a full scan for Law variant model IDs found 168 entries, many of them noisy
  or UI-like.

Important negative result:

- no simple compact `u16` row has been proven yet that directly links
  `227 -> 58`, `272 -> 111`, and `308 -> 131`;
- entries with the real `806_*` layout IDs mostly look like UI/screen layout
  databases;
- entries with model IDs often look like model/event/cutscene/parameter data
  and do not carry the matching layout IDs nearby.

## Entry 32: Name Registry

`LINKDATA_A` entry `32` is a major string registry.

Header:

- u32 section count: `15`;
- followed by 15 section offsets.

Each section appears to use:

```text
u32 count
repeated {
  u32 string_offset_from_section_start
  u32 string_length
}
```

Important section counts:

```text
[826, 100, 2334, 2502, 200, 161, 862, 4713, 99, 1, 1, 50398, 200, 200, 281]
```

Important sections:

| Section | Current meaning |
| --- | --- |
| `4` | playable/base character-ish names |
| `5` | costume label-ish strings |
| `6` | model/resource names |
| `7` | UI/screen-layout resource names |
| `11` | voice/cutscene/string-heavy data |

Known IDs:

| Section | ID | String |
| --- | ---: | --- |
| `4` | `12` | `MPLC012_Newgate` |
| `4` | `22` | `MPLC026_Law` |
| `5` | `2` | `dressrosa_costume` |
| `6` | `12` | `MPLC012_Newgate` |
| `6` | `26` | `MPLC026_Law` |
| `6` | `227` | `MVAR025_Law_DR` |
| `6` | `272` | `MDLC033_Law_Souhi` |
| `6` | `308` | `MDLC069_Law_Oni` |
| `7` | `2092` | `806_044_costume_newgate` |
| `7` | `2105` | `806_057_costume_law` |
| `7` | `2106` | `806_058_costume_law_dressrosa` |
| `7` | `2159` | `806_111_costume_law_souhi` |
| `7` | `2179` | `806_131_costume_law_oni` |

Additional Law UI IDs from section `7`:

| ID | String |
| ---: | --- |
| `205` | `800_026_face_law` |
| `473` | `800_294_face_law_dressrosa` |
| `643` | `801_026_chara_law` |
| `911` | `801_294_chara_law_dressrosa` |
| `1081` | `802_026_vmes_law` |
| `1349` | `802_294_vmes_law_dressrosa` |
| `1465` | `802_410_vmes_law_souhi` |
| `1480` | `802_425_vmes_law_oni` |
| `1519` | `804_026_facemini_law` |
| `1787` | `804_294_facemini_law_dressrosa` |

## Costume Layout Ordering

There are 141 `806_*costume*` strings in entry `32`, section `7`.

The filtered order matches the `806_XXX` suffix for known rows:

| Costume order | Section 7 ID | Suffix | String |
| ---: | ---: | ---: | --- |
| `44` | `2092` | `044` | `806_044_costume_newgate` |
| `57` | `2105` | `057` | `806_057_costume_law` |
| `58` | `2106` | `058` | `806_058_costume_law_dressrosa` |
| `111` | `2159` | `111` | `806_111_costume_law_souhi` |
| `131` | `2179` | `131` | `806_131_costume_law_oni` |

The known costume list currently reaches suffix `140`. The next append-style
candidate from the global `806_*costume*` list is therefore `141`.

Important correction from the dry-run planner:

- suffix `137` already exists, even though it is not attached to a useful owner
  group in the focused model dump;
- next-suffix logic must use the global layout list from entry `32`, section
  `7`, not only layouts matched to entry `35` owner groups;
- there are still gaps, but reusing a taken suffix would likely corrupt or
  redirect an existing layout.

This strongly suggests two different IDs are relevant:

- model/resource ID, such as `26`, `227`, `272`, `308`;
- UI/costume-layout ID, such as `57`, `58`, `111`, `131`.

The missing table likely connects those two families.

## Entry 35 Owner Groups vs Costume Layouts

Comparison pass:

- entry `35` groups named models by owner through `i16[28]`;
- entry `32`, section `7`, lists `806_*costume*` layouts;
- for Law, the model group and layout group line up perfectly by known
  costume order;
- for other characters, there can be more layouts than model rows, which means
  some costumes probably reuse a model with alternate materials/config instead
  of having a distinct `.g1m` row.

Examples:

| Character | Entry `35` owner models | `806_*costume*` layouts |
| --- | --- | --- |
| Zoro | `MPLC001_Zoro`, `MVAR001_Zoro_NW`, `MDLC038_Zoro_Wa` | base, Alabasta, garb, Sabaody, NW, Wa |
| Nami | `MPLC002_Nami`, `MVAR002_Nami_NW`, `MDLC040_Nami_Wa` | base, Alabasta, W7, NW, Wa |
| Sanji | `MPLC004_Sanji`, `MVAR004_Sanji_NW`, `MVAR028_Sanji_Tuxedo`, `MDLC039_Sanji_Osoba` | base, Mr Prince, NW, groom, Osoba |
| Robin | `MPLC006_Robin`, `MVAR006_Robin_NW`, `MVAR018_Robin_AS` plus submodels owned by `6` | base, NW, All Sunday |
| Hancock | `MPLC010_Hancock`, `MDLC034_Hancock_Ogk` | base, Ougenki |
| Newgate | `MPLC012_Newgate` | base |
| Law | `MPLC026_Law`, `MVAR025_Law_DR`, `MDLC033_Law_Souhi`, `MDLC069_Law_Oni` | base, Dressrosa, Souhi, Oni |

Important nuance:

- entry `35` is probably necessary for adding a model-backed costume;
- it is probably not sufficient by itself;
- the remaining table/config must decide which `806_*` layout appears in the
  costume menu and which model/material configuration it selects.

## RDB Assets Of Interest

Newgate:

| Asset | Hash |
| --- | --- |
| `MPLC012_Newgate.g1m` | `0x3a6cf6e5` |
| `806_044_costume_newgate.kscl` | `0x686b21bc` |
| `806_044_costume_newgate_External_00.g1t` | `0x232103b0` |

Law base:

| Asset | Hash |
| --- | --- |
| `MPLC026_Law.g1m` | `0x8df2d8cb` |
| `806_057_costume_law.kscl` | `0x07bab795` |

Law Dressrosa:

| Asset | Hash |
| --- | --- |
| `MVAR025_Law_DR.g1m` | `0xa2cd7f79` |
| `806_058_costume_law_dressrosa.kscl` | `0x0df13099` |

Law Souhi:

| Asset | Hash |
| --- | --- |
| `MDLC033_Law_Souhi.g1m` | `0xc7512008` |
| `806_111_costume_law_souhi.kscl` | `0x04d3706f` |

Law Oni:

| Asset | Hash |
| --- | --- |
| `MDLC069_Law_Oni.g1m` | currently known from entry `32`, hash still to record |
| `806_131_costume_law_oni.kscl` | hash still to record |

Screen layout KSCL blobs are `IDRK` containers with `LCSK` payloads.

Observed KSCL types:

- Newgate/Law base/Law Dressrosa: `LCSK8500`;
- Law Souhi: `LCSK9500`.

Current interpretation: KSCL files define/render UI layout, not the final
character-to-model slot mapping.

## Entry 29: DLC Entitlement/Unlock Strings

Entry `29` is a strong DLC/unlock table candidate.

It has 21 string sections.

Important strings:

```text
DLC_COSTUME_000_550_051_004
DLC_COSTUME_001_551_052_001
DLC_COSTUME_002_552_053_001
DLC_COSTUME_003_553_055_002
DLC_COSTUME_004_554_010_001
DLC_COSTUME_005_555_026_002
DLC_COSTUME_006_586_026_003
DLC_UNLOCK_CHARACTER_000_042
```

Law-specific DLC rows:

| String | Current interpretation |
| --- | --- |
| `DLC_COSTUME_005_555_026_002` | Law (`026`), costume index `002` |
| `DLC_COSTUME_006_586_026_003` | Law (`026`), costume index `003` |

Entry `29`, section `20`, also has labels:

```text
COSTUME00
COSTUME01
COSTUME02
COSTUME03
COSTUME04
COSTUME05
COSTUME06
STEAMDLC...
```

Current interpretation:

- entry `29` probably controls DLC entitlement/unlock naming;
- it proves official extra costumes are represented by character ID + costume
  index;
- it probably does not by itself map those slots to `.g1m` and layout assets.

## Entry 35: Model Metadata

Entry `35` looks like per-model/per-resource metadata.

Shape:

- count: `848`;
- stride: `96`;
- data starts at `0x10`;
- row index appears to match entry `32`, section `6` model/resource IDs.

Important rows:

| Row | Model | Notes |
| ---: | --- | --- |
| `12` | `MPLC012_Newgate` | base Newgate |
| `26` | `MPLC026_Law` | base Law |
| `227` | `MVAR025_Law_DR` | Law variant |
| `272` | `MDLC033_Law_Souhi` | Law variant |
| `308` | `MDLC069_Law_Oni` | Law variant |

Observed relation fields:

- row `12`: `i16[28] = 12`, `i16[34] = 12`;
- row `26`: `i16[28] = 26`, `i16[34] = 26`;
- row `227`: `i16[28] = 26`, `i16[34] = 168`;
- row `272`: `i16[28] = 26`, `i16[34] = 168`;
- row `308`: `i16[28] = 26`, `i16[34] = 26`.

Current interpretation:

- entry `35` probably defines model metadata and variant ownership;
- `i16[28]` looks especially important because official variants group under
  their base owner model;
- for a new Newgate skin with a new model ID, we may need to clone/adapt row
  `12`;
- this still does not directly expose the `806_*` menu layout ID, but it may be
  one half of the roster mapping.

Owner grouping examples from `i16[28]`:

| Owner | Named rows observed |
| ---: | --- |
| `1` | `MPLC001_Zoro`, `MVAR001_Zoro_NW`, `MDLC038_Zoro_Wano` |
| `2` | `MPLC002_Nami`, `MVAR002_Nami_NW`, `MDLC040_Nami_Wano` |
| `4` | `MPLC004_Sanji`, `MVAR004_Sanji_NW`, `MVAR028_Sanji_Tuxedo`, `MDLC039_Sanji_Osoba` |
| `6` | `MPLC006_Robin`, `MVAR006_Robin_NW`, `MVAR020_Robin_AS` |
| `10` | `MPLC010_Hancock`, `MDLC034_Hancock_Ogk` |
| `12` | `MPLC012_Newgate` |
| `26` | `MPLC026_Law`, `MVAR025_Law_DR`, `MDLC033_Law_Souhi`, `MDLC069_Law_Oni` |

Law owner group details:

| Row | Name | Owner `i16[28]` | Relation `i16[34]` | Notes |
| ---: | --- | ---: | ---: | --- |
| `26` | `MPLC026_Law` | `26` | `26` | base |
| `227` | `MVAR025_Law_DR` | `26` | `168` | Dressrosa variant |
| `272` | `MDLC033_Law_Souhi` | `26` | `168` | DLC variant |
| `308` | `MDLC069_Law_Oni` | `26` | `26` | DLC variant |

Newgate owner group currently has only:

| Row | Name | Owner `i16[28]` | Relation `i16[34]` |
| ---: | --- | ---: | ---: |
| `12` | `MPLC012_Newgate` | `12` | `12` |

This is the strongest data-driven lead so far: adding a Newgate skin probably
requires adding/cloning a new entry `35` row whose owner field is `12`, then
finding the parallel UI/layout side.

## Other Entries Checked

| Entry | Current read |
| ---: | --- |
| `31` | labels/categories such as `CHARA_G`, `COSTUME`, `CATEGORY`; useful as metadata vocabulary |
| `33` | animation/action/resource strings; includes many character action names |
| `34` | one string section, not the slot mapping |
| `39` | counted table, 427 rows, stride `48`; rows `57`, `58`, `111`, `131` are non-empty and may relate to costume/UI parameters |
| `40` | counted table, 200 rows, stride `144`; likely playable character config/stats, row `22` is Law |
| `42` | large table with many sequential small IDs; currently noisy, likely not the slot mapping |
| `52` | huge counted table, 26002 rows, stride `32`; groups by model IDs including `26`, `227`, `272`, `308`; likely voice/name/effect metadata per model, not the final costume roster |
| `54` | false lead; string table for model parts/attachments such as `P_Body`, `P_WP_00`, `P_DRESSROSA` |
| `58` | 200-section table; sections match IDs such as `44`, `57`, `58`, `111`, `131`; likely per-costume config, not ownership roster |
| `61` | voice/cutscene-heavy IDs plus repeated UI refs, not roster |
| `64` | large numeric/float-heavy table; contains incidental IDs but looks like parameter/bounds data |
| `66` | counted table, 646 rows, stride `80`; contains Law variant rows `308`, `272`, `227` close together, but appears to be parameter/bounds-like data |
| `68` | sectioned table with 48-byte records; contains voice/event/UI-ish IDs, currently a false lead for costume roster |
| `59`, `60`, `61`, `243`, `244`, `245`, `247`, `248`, `275` | contain `806_*` layout IDs; current evidence says these are screen/UI layout databases, not the playable costume roster |
| `2427`, `2453`, `2459`, `2460`, `2510`, `2547`, `2549`, `2550` | late entries containing multiple Law variant model IDs; current context looks noisy/KIDSDB-like, not proven roster |

## False Leads

Entry `54` looked promising because Law/model IDs appeared close together, but
the surrounding strings show model part names. Treat it as model attachment/part
metadata, not costume roster.

Entries `245`, `247`, and `248` contain `806_*costume*` references:

- entry `245`: `806_057_costume_law`, `806_058_costume_law_dressrosa`;
- entry `247`: `806_111_costume_law_souhi`;
- entry `248`: `806_131_costume_law_oni`.

These are useful for UI, but current evidence says they describe layout records,
not which costumes are available for a character.

A broader numeric scan found the real section `7` layout IDs mostly in high UI
entries:

| Entry | Layout IDs observed |
| ---: | --- |
| `59` | `806_057_costume_law`, `806_058_costume_law_dressrosa` |
| `60` | `806_058_costume_law_dressrosa` |
| `61` | repeated `806_058_costume_law_dressrosa` |
| `243` | many base/Dressrosa Law layout refs |
| `244` | Newgate layout refs |
| `245` | Law base/Dressrosa layout refs |
| `247` | Law Souhi layout refs |
| `248` | Law Oni layout refs |
| `275` | Law base/Dressrosa layout refs |

Neighboring values in these records resolve to other UI/layout resources such as
`006_00_keyguide`, `007_00_background_tile`, `015_00_unlock`, and many
`826_*_eula_*` IDs. Current interpretation: these entries define how layouts
are rendered or wired in UI databases, not the high-level list of selectable
costumes.

Entry `42` produces many false-positive model/layout numeric windows because it
contains long sequential ID ranges. It should be ignored unless a stronger
record structure appears.

Entry `64` also produces false-positive windows but its bytes are dominated by
float-like parameter data. It is probably not the roster.

Late entries `2427+` contain many occurrences of model IDs `227`, `272`, and
`308`. They are worth keeping in mind because they prove the variant IDs appear
outside entry `35`, but the inspected neighborhoods look like generated
database records or UI/event data, not a clean costume list. Common signs:

- many unrelated character IDs sit next to each other;
- neighboring values resolve to UI names when interpreted through entry `32`,
  section `7`;
- no inspected record directly contains the expected matching `806_*` layout
  IDs.

Current interpretation: these entries may need later patching if they are
runtime preload/UI databases, but they are not yet the missing roster proof.

Entry `52` contains model groups for Law variants, but the pattern looks like
per-model voice/name/effect data. It should not be treated as the slot roster
until proven otherwise.

Entry `66` contains rows for model/resource IDs, including Law variants:

| Row | Interesting values |
| ---: | --- |
| `24` | contains `26` |
| `564` | contains `308` |
| `565` | contains `272` |
| `566` | contains `227` |

The rows are consecutive for `308`, `272`, `227`, but the record shape is
dominated by repeated `-1`, `-10000`, `10000`, and float-looking values. Current
interpretation: this is likely per-resource parameter/bounds/camera-ish data,
not the high-level slot roster.

Entry `68` looked interesting because it has clean records and some IDs near
known UI values, but resolved names show it is probably unrelated:

| Raw ID | Resolved string |
| ---: | --- |
| `500` | `AV005_00_17` |
| `501` | `AV005_00_18` |
| `502` | `AV005_00_19` |
| `503` | `AV005_00_20` |
| `108` | `411_01_kill_count_big` |
| `109` | `411_02_hit_count` |
| `110` | `412_00_powerdash_target` |
| `111` | `413_00_coin` |
| `2003` | `803_072_name_dlc_pc_uta` |
| `2004` | `803_073_name_dlc_pc_shanks_red` |

Current interpretation: entry `68` is event/voice/UI parameter data, not the
costume roster.

Entry `58` is more interesting than first assumed because sections are indexed
by IDs that match costume suffixes:

| Section | Related costume |
| ---: | --- |
| `44` | Newgate costume layout suffix |
| `57` | Law base costume layout suffix |
| `58` | Law Dressrosa costume layout suffix |
| `111` | Law Souhi costume layout suffix |
| `131` | Law Oni costume layout suffix |
| `168` | appears in entry `35` as a variant relation field for Law DR/Souhi |

Entry `58` section format appears to be:

```text
u32 record_count
u32 zero/padding
u32 zero/padding
u32 zero/padding
record_count * {
  u32 key_a
  u32 key_b
  u32 value_a_or_zero
  u32 value_b_or_zero
}
```

Examples:

- section `57` has 14 records and contains pairs such as `[6, 5]`,
  `[6, 53]`, `[8, 7]`, `[9, 8]`, `[13, 12]`;
- section `58` has 14 records and contains pairs such as `[6, 2]`,
  `[6, 5]`, `[8, 7]`, `[9, 8]`, `[13, 12]`;
- section `111` has 5 records and contains `[4, 4]`, `[8, 7]`, `[9, 8]`,
  `[13, 12]`, `[15, 15]`;
- section `131` has 1 record and contains `[15, 15]`;
- section `168` is identical to section `111` in the currently checked bytes.

Current interpretation: entry `58` probably configures secondary per-costume
systems such as skills, categories, flags, or effects. It is very relevant once
we add a costume, but it still does not explicitly say "Law owns this costume".

## Current Main Hypothesis

The real add-slot path probably needs at least these data patches:

1. Add or reuse a model/resource ID in entry `32`, section `6`.
2. Add model metadata in entry `35` by cloning the base/closest model row and
   setting the owner field to the base character/model owner.
3. Add a UI/costume layout string in entry `32`, section `7`.
4. Provide ScreenLayout assets for the new `806_*_costume_*` layout and related
   face/chara/vmes/facemini assets if the menu expects them.
5. Extend the unknown or implicit roster table that maps:

```text
base character ID -> costume index -> model ID -> costume layout ID
```

For Law, the target mapping likely already contains something equivalent to:

| Base character | Costume index | Model/resource | UI layout |
| ---: | ---: | ---: | ---: |
| `26` | `0` or base | `26` | `57` / section 7 ID `2105` |
| `26` | `1` or Dressrosa | `227` | `58` / section 7 ID `2106` |
| `26` | `2` | `272` | `111` / section 7 ID `2159` |
| `26` | `3` | `308` | `131` / section 7 ID `2179` |

The exact index numbering still needs confirmation because entry `29` only
proves DLC strings use `026_002` and `026_003`.

Important refinement: the model side may already be entry `35` owner grouping.
The remaining missing piece is how the game pairs each owned model row with the
matching `806_*` costume layout.

## Next Probes

Priority scans:

- search all LinkData entries for a compact row containing `26`, `227`, `272`,
  `308`, or their layout IDs `57`, `58`, `111`, `131`;
- filter out tables where the hit is only part of sequential ID ranges;
- compare Law with another character that has multiple official DLC costumes;
- inspect entry `39` rows `57`, `58`, `111`, `131` more deeply;
- inspect entry `40` row `22` as playable Law config;
- inspect entry `58` as per-costume config and compare it against actual in-game
  costume behavior;
- inspect entry `66` enough to know whether new model/resource IDs need a row
  there too;
- find any table where row `22` or character/model `26` points to a list of
  costume indexes.
- compare entry `35` owner groups against `806_*costume*` layout groups for
  characters with known official variants, especially Zoro, Nami, Sanji, Robin,
  Hancock, and Law.

## LinkData Proximity Scanner

Added a dedicated multi-threaded scanner to avoid slow ad-hoc REPL brute force:

```text
apps/rdb-tool/src/linkdata_scan.rs
```

Command shape:

```text
oppw4-rdb --linkdata-proximity-scan <linkdata-bin> \
  --owners <comma-separated ids> \
  --models <comma-separated ids> \
  --layouts <comma-separated ids> \
  --radius <bytes> \
  --threads <n> \
  --max-matches <n> \
  --context-bytes <n>
```

Output is JSONL:

- one `scan_start` row;
- zero or more `match` rows;
- one `scan_summary` row.

Example Law scan:

```text
cargo run -q -p oppw4-rdb-tool -- --linkdata-proximity-scan D:\SteamLibrary\steamapps\common\OPPW4\LINKDATA\CMN\LINKDATA_A.BIN --owners 22,26 --models 227,272,308 --layouts 58,111,131,2106,2159,2179 --radius 128 --threads 8 --max-matches 20 --context-bytes 64
```

Strict Law result with radius `128`:

```json
{"event":"scan_summary","entries":3706,"scanned_entries":3706,"matches":0,"inflate_errors":0,"stopped_early":false}
```

Interpretation:

- no compact nearby `owner/model/layout` triple appears for Law at this radius;
- this supports the current theory that the real mapping is either split across
  tables, implicit by ordered groups, or encoded in a less direct shape.

Broad Law scan with radius `1024` finds noisy proximities:

```text
entry 40: model 272 near owner 22 and layouts 111/131, but across different rows
entry 35: model 308 near owner 26 and incidental layouts 111/58
entry 54: many model-part/attachment false positives
entry 42: sequential ID-range false positives
```

Entry `40` broad hit was checked manually and is a false-positive proximity
caused by the large radius crossing row boundaries:

| Row | Entry `32` section `4` name | Relevant incidental values |
| ---: | --- | --- |
| `166` | `MDLC054_Luffy_Oni` | `22`, `111` |
| `167` | `MDLC055_Luffy_N_Oni` | `22`, `111` |
| `168` | `MDLC056_Luffy_N_Gear5` | `22`, `111` |
| `170` | `MDLC067_Kaido_D2` | `272` |
| `175` | `MDLC061_Uta_Armor` | `131` |

Current interpretation: entry `40` is still likely playable-character config,
but this scanner hit is not a Law costume mapping proof.

## LinkData Costume Dump Tool

Added a structured costume-oriented dump command:

```text
apps/rdb-tool/src/linkdata_costume_dump.rs
```

Command shape:

```text
oppw4-rdb --linkdata-costume-dump <linkdata-bin> \
  [--owners <comma-separated owner ids>] \
  [--name-contains <text>]
```

Example:

```text
target\release\oppw4-rdb.exe --linkdata-costume-dump D:\SteamLibrary\steamapps\common\OPPW4\LINKDATA\CMN\LINKDATA_A.BIN --owners 12,26
```

The command emits one JSON object containing, per owner:

- named model rows from entry `35`;
- matched `806_*costume*` layouts from entry `32`, section `7`;
- matching DLC costume strings from entry `29`;
- entry `58` record count and first pairs per layout suffix;
- entry `39` first values per layout suffix.

Verified focused result:

| Owner | Models | Layouts | DLC costume strings |
| ---: | ---: | ---: | ---: |
| `12` Newgate | `1` | `1` | `0` |
| `26` Law | `4` | `4` | `2` |

Law dump:

| Model row | Name | Relation |
| ---: | --- | ---: |
| `26` | `MPLC026_Law` | `26` |
| `227` | `MVAR025_Law_DR` | `168` |
| `272` | `MDLC033_Law_Souhi` | `168` |
| `308` | `MDLC069_Law_Oni` | `26` |

Law layouts:

| Suffix | Section 7 ID | Name | Entry 58 records | Entry 39 present |
| ---: | ---: | --- | ---: | --- |
| `57` | `2105` | `806_057_costume_law` | `14` | yes |
| `58` | `2106` | `806_058_costume_law_dressrosa` | `14` | yes |
| `111` | `2159` | `806_111_costume_law_souhi` | `5` | yes |
| `131` | `2179` | `806_131_costume_law_oni` | `1` | yes |

Multi-owner comparison:

| Owner | Models | Layouts | Note |
| ---: | ---: | ---: | --- |
| `1` Zoro | `3` | `6` | several layouts reuse existing models/material configs |
| `2` Nami | `3` | `5` | same reuse pattern |
| `4` Sanji | `4` | `5` | one extra layout beyond named models |
| `6` Robin | `5` | `3` | includes submodels such as hands/wings, so model count is not slot count |
| `10` Hancock | `2` | `2` | clean DLC-like pattern |
| `12` Newgate | `1` | `1` | target base case |
| `26` Law | `4` | `4` | clean multi-costume pattern |

Current conclusion:

- entry `35` owner groups alone cannot define selectable costume slots;
- layout count can exceed model count when costumes reuse models/materials;
- model count can exceed layout count when owner groups include submodels;
- Law/Hancock are the strongest clone patterns for a model-backed DLC costume;
- adding a Newgate slot probably needs at least:
  - a new named model row or cloned model metadata row in entry `35`;
  - a new `806_*costume_newgate_*` layout string/assets;
  - entry `58` and entry `39` sections/rows for the new layout suffix;
  - possibly a DLC/unlock row if we want the game to treat it like official DLC.

Useful known anchors:

```text
Newgate base character/model: 12
Law playable character: 22
Law base model/resource: 26
Law Dressrosa model/resource: 227
Law Souhi model/resource: 272
Law Oni model/resource: 308
Law base costume layout suffix: 57
Law Dressrosa costume layout suffix: 58
Law Souhi costume layout suffix: 111
Law Oni costume layout suffix: 131
```

Potential proof condition:

- find one row/record that links Law owner `26` or playable char `22` to
  model `227` and layout `58`;
- then find parallel records for `272 -> 111` and `308 -> 131`;
- then clone that pattern for Newgate.

## Tooling To Add Later

Once the mapping table is identified, add first-class tooling instead of ad-hoc
Node scripts:

- parse selected LinkData table structures in `oppw4-rdb`;
- expose an `rdb-tool` command to dump costume mappings;
- add fixture-based tests using small synthetic LinkData blobs;
- keep the current writer offline until the data shape is proven in game;
- keep runtime DLL changes minimal until the data format is proven.

## LinkData Patch Plan Dry Run

Added a non-writing planner command:

```text
oppw4-rdb --linkdata-patch-plan <linkdata-bin> \
  --target-owner <id> \
  --source-owner <id> \
  --source-model-row <row> \
  --source-layout-suffix <suffix> \
  --new-model-name <name> \
  --new-layout-name <806_suffix_costume_name>
```

Current real-data probe using Hancock DLC as the clean source pattern and
Newgate as target:

```text
target\release\oppw4-rdb.exe --linkdata-patch-plan target\reverse\LINKDATA_A.BIN --target-owner 12 --source-owner 10 --source-model-row 273 --source-layout-suffix 112 --new-model-name MDLC999_Newgate_Test --new-layout-name 806_141_costume_newgate_test
```

Observed dry-run proposal:

| Field | Value |
| --- | --- |
| Target owner | `12` / `MPLC012_Newgate` |
| Source owner | `10` / `MPLC010_Hancock` |
| Source model row | `273` / `MDLC034_Hancock_Ogk` |
| Source layout suffix | `112` / `806_112_costume_hancock_ougenki` |
| Proposed model row | `848` |
| Proposed section `6` string ID | `848` |
| Proposed layout suffix | `141` |
| Proposed section `7` string ID | `4713` |
| Proposed layout name | `806_141_costume_newgate_test` |
| Clone entry `35` from | row `273` |
| Clone entry `58` from | suffix `112` |
| Clone entry `39` from | suffix `112` |

Planner safeguards currently implemented:

- if the new layout name embeds a suffix like `806_141_...`, that suffix is
  respected;
- if no suffix is embedded, the planner uses the global costume layout list and
  proposes max suffix + 1;
- if a requested suffix already exists, the dry run prints a warning;
- the planner still warns that DLC/unlock wiring is not solved yet.

## Offline LinkData Apply Probe

Added an offline writer command. It never writes to the game path directly:

```text
oppw4-rdb --linkdata-apply-plan <input-linkdata-bin> <output-linkdata-bin> \
  --target-owner <id> \
  --source-owner <id> \
  --source-model-row <row> \
  --source-layout-suffix <suffix> \
  --new-model-name <name> \
  --new-layout-name <806_suffix_costume_name>
```

Current probe:

```text
target\release\oppw4-rdb.exe --linkdata-apply-plan target\reverse\LINKDATA_A.BIN target\reverse\LINKDATA_A.newgate-test.BIN --target-owner 12 --source-owner 10 --source-model-row 273 --source-layout-suffix 112 --new-model-name MDLC999_Newgate_Test --new-layout-name 806_141_costume_newgate_test
```

Entries patched in the output copy:

| Entry | Change |
| ---: | --- |
| `32` | set section `6` ID `848` to `MDLC999_Newgate_Test`; append section `7` ID `4713` as `806_141_costume_newgate_test` |
| `35` | clone Hancock DLC model row `273` into row `848`, then retarget owner/relation to Newgate `12` |
| `39` | clone costume param row/suffix `112` into suffix `141` |
| `58` | clone costume section/suffix `112` into suffix `141` |

Validation on the patched output:

| Owner | Models | Layouts | DLC costume strings |
| ---: | ---: | ---: | ---: |
| `10` Hancock | `2` | `2` | `1` |
| `12` Newgate | `2` | `2` | `0` |

Newgate patched dump now shows:

| Kind | Value |
| --- | --- |
| New model row | `848` / `MDLC999_Newgate_Test`, owner `12`, relation `12` |
| New layout | suffix `141`, section `7` ID `4713`, `806_141_costume_newgate_test` |
| Entry `58` for suffix `141` | record count `5`, cloned from Hancock DLC suffix `112` |
| Entry `39` for suffix `141` | present, cloned from Hancock DLC suffix `112` |

Open risk:

- no entry `29` DLC/unlock string is added yet;
- if the slot is present in data but hidden in game, the next table to find is
  probably an unlock/menu/count table rather than the model/layout table.

## Boot Test Results

The first offline patch was installed once as:

```text
target/reverse/LINKDATA_A.newgate-test.BIN
```

Result: the game reached a black screen before the normal load/menu flow.

Reason this test is now considered unsafe: it rebuilt the whole `LINKDATA_A`
payload layout. The output file was smaller than the original and moved every
entry, so it likely violated a loader assumption outside our parser.

The second test was a sparse file patch:

```text
target/reverse/LINKDATA_A.newgate-sparse.BIN
```

It preserved most original bytes, but moved entry `32` from `0x211800` to EOF
because the recompressed string registry did not fit its original slot:

| Entry | Original offset | New offset | Original span | New span |
| ---: | ---: | ---: | ---: | ---: |
| `32` | `0x211800` | `0x1fc5800` | `0x3fc49` | `0x3fd76` |
| `35` | `0x2f2500` | `0x2f2500` | `0x1fd5` | `0x1fa3` |
| `39` | `0x2f5c00` | `0x2f5c00` | `0x2a2` | `0x29a` |
| `58` | `0x314f00` | `0x314f00` | `0xe16` | `0xe34` |

Result: black screen as well.

Current hypothesis: the game may require at least some early LinkData entries
to remain at their original offsets, or it may validate the table in a way our
parser does not yet model. Moving entry `32` is the strongest suspect because it
is a large global string registry and the sparse patch still had to relocate it.

The current boot probe is intentionally smaller:

```text
target/reverse/LINKDATA_A.probe-entry35-only.BIN
```

This probe:

- preserves the original file size;
- moves no LinkData entries;
- changes only entry `35`;
- retargets existing Hancock DLC model row `273` from owner/relation `10` to
  owner/relation `12`;
- does not add a new string, layout, DLC row, entry `39` row, or entry `58`
  section.

Expected result: this will not add a real Newgate costume slot. It is only a
boot-safety probe to learn whether a minimal in-place entry `35` mutation is
accepted by the game.

If this probe boots, the next candidate is an in-place-only patch that avoids
entry `32` completely by reusing an existing layout suffix. If this probe also
black-screens, the engine probably validates entry `35` ownership/model rows
early, and the next path should be runtime instrumentation in the DLL rather
than more offline file rewrites.

Result: the game passed the early boot/linkdata load, then crashed just before
the main menu at `OPPW4.exe+0x15E281D`.

Current interpretation: the file structure was accepted, but the content was
not. The probe retargeted Hancock DLC model row `273` from owner `10` to owner
`12`. Hancock still had its DLC costume string and layout data, but no longer
had the matching DLC model row. The crash timing is consistent with the game
building the main menu/costume roster, resolving Hancock DLC data, then hitting
a null or invalid pointer.

The next installed probe is:

```text
target/reverse/LINKDATA_A.probe-duplicate-row738.BIN
```

This probe keeps Hancock intact and duplicates Hancock DLC model row `273` into
inactive row `738` for Newgate:

| Entry | Original offset | New offset | Original span | New span |
| ---: | ---: | ---: | ---: | ---: |
| `32` | `0x211800` | `0x211800` | `0x3fc49` | `0x3fc88` |
| `35` | `0x2f2500` | `0x2f2500` | `0x1fd5` | `0x1fe8` |

Important details:

- the original file size is preserved;
- no LinkData entry moves;
- entry `32` is patched in place by replacing section `6` row `738`
  `H_UI_Island_Chapter00` with `MDLC034_Hancock_Ogk`;
- entry `35` row `738` is a copy of row `273`, retargeted to owner/relation
  `12`;
- Hancock still has both original model rows;
- Newgate now has rows `12` and `738`;
- this still does not add a real selectable slot because no new layout/unlock
  path is wired yet.

Result: black screen.

This makes entry `32` the strongest suspect. The `row738` probe replaced
section `6` row `738` from `H_UI_Island_Chapter00` to
`MDLC034_Hancock_Ogk`. The game appears to need those `H_UI_Island_ChapterXX`
strings during early UI/resource boot, so using them as "free" string slots is
not safe.

Registry slot search:

- a real `MDLC034_Hancock_Ogk\0` string needs around `20` bytes;
- inactive entry `35` rows with section `6` slots large enough are only
  `738..744`;
- those slots are all `H_UI_Island_Chapter00..06`;
- there is no obvious empty inactive section `6` string slot large enough for a
  copied DLC model name.

The next installed probe is:

```text
target/reverse/LINKDATA_A.probe-duplicate-row292.BIN
```

This probe isolates entry `35` only:

| Entry | Original offset | New offset | Original span | New span |
| ---: | ---: | ---: | ---: | ---: |
| `35` | `0x2f2500` | `0x2f2500` | `0x1fd5` | `0x1fa3` |

Details:

- no entry `32` edits;
- no file size change;
- no LinkData entry moves;
- Hancock remains intact;
- row `292` already has the valid existing section `6` name
  `MPLC000_Luffy`;
- row `292` is changed from inactive owner `-1` to owner/relation `12` after
  copying the Hancock DLC row payload.

This is not intended to display the correct skin. It only tests whether adding
an extra active model row without touching the string registry can pass the
early boot path.

The DLL can already load a temporary skin by replacing an existing RDB asset
through `mods/<mod_name>/<archive>/...` or a matching zip. That is useful as a
visual test path, but it only replaces an existing skin; it does not prove that
a new selectable slot can be added.

Result: the game loaded past the early black-screen phase, then crashed before
the main menu with:

```text
OPPW4.exe+0x15E281D
rdx = r8 = 0x2e2
rax = 0
```

The original game file was restored after this probe. The installed
`LINKDATA_A.BIN` now matches the clean captured copy again.

## Crash `OPPW4.exe+0x15E281D`

`OPPW4.exe+0x15E281D` maps to VA `0x1415E281D`. The local disassembly around
that address is:

```asm
1415e2800: sub rsp,0x28
1415e2804: lea eax,[rdx-0x351]
1415e280a: cmp eax,0xc
1415e280d: ja  1415e2816
1415e280f: xor eax,eax
1415e2811: add rsp,0x28
1415e2815: ret
1415e2816: mov rax,QWORD PTR [rcx+0x18]
1415e281a: mov r8d,edx
1415e281d: mov r9,QWORD PTR [rax+0x8]
1415e2821: mov rcx,QWORD PTR [rax+0x10]
1415e2848: imul rax,r8,0x68
1415e284c: add rax,r9
1415e284f: add rsp,0x28
1415e2853: ret
```

Interpretation:

- this function is a resource/model table getter;
- records are `0x68` bytes each;
- IDs `0x351..0x35d` short-circuit to null before touching the table;
- otherwise it expects a vector pointer at `[manager + 0x18]`;
- the crash happens because `[manager + 0x18]` is null and the function still
  dereferences it.

So `0x2e2` is not necessarily the bad raw value inside the patched LinkData
row. It is the ID being looked up when the model/resource table is not ready
for that code path.

Entry `35` row scan for the `row292` probe:

| Row | Name slot | Owner | Relation | Contains raw `0x02e2` |
| ---: | --- | ---: | ---: | --- |
| `273` | `MDLC034_Hancock_Ogk` | `10` | `10` | no |
| `292` original | `MPLC000_Luffy` | `-1` | `162` | no |
| `292` patched | `MPLC000_Luffy` | `12` | `12` | no |
| `738` original | `H_UI_Island_Chapter00` | `-1` | `-1` | no |

This changes the hypothesis:

- the `row738` probe black-screened because it edited entry `32` and replaced a
  used `H_UI_Island_Chapter00` string slot;
- the `row292` probe proves entry `32` is not the only problem;
- blindly copying Hancock DLC row `273` and only retargeting owner/relation is
  invalid;
- the copied row still carries Hancock-specific metadata while the name slot is
  still `MPLC000_Luffy`;
- that inconsistent active row can trigger an early resource lookup before the
  global model/resource table is initialized.

Next safer probes should avoid cross-owner row payloads:

1. clone a known-safe Newgate-like row shape first, not Hancock;
2. avoid changing entry `32` used UI/global string slots;
3. only add layout/DLC/unlock records once the model row can boot safely;
4. if the same crash persists with a Newgate-shaped duplicate, switch to a DLL
   runtime logger around the getter/callers so we can see which object produces
   `edx = 0x2e2`.

Prepared next probe:

```text
target/reverse/LINKDATA_A.probe-newgate-row292.BIN
```

This is entry `35` only:

- original file size preserved;
- no entry moves;
- no entry `32` string registry edits;
- row `292` is cloned from Newgate base row `12`, not Hancock row `273`;
- row `292` owner/relation are both `12`;
- the row `292` name slot is still the original `MPLC000_Luffy`, because the
  safe probe intentionally avoids string registry edits.

Expected result:

- if this boots, the previous crash was caused by the cross-owner Hancock row
  payload;
- if this crashes at the same getter, adding an active row without a complete
  matching layout/unlock/name path is enough to trigger early resource lookup;
- if this black-screens, entry `35` model-row mutation itself has stricter boot
  validation than expected.

Result: same crash again.

```text
OPPW4.exe+0x15E281D
rdx = r8 = 0x2e2
```

The original game file was restored after the test.

Updated conclusion:

- the crash is not Hancock-specific;
- a Newgate-shaped duplicate model row is also rejected when it is activated
  alone;
- entry `35` cannot be treated as an independent list of "available skins";
- official multi-costume characters keep several tables coherent at once:
  model rows, layout strings, layout resource sections, costume params, and
  often DLC/unlock strings;
- Newgate with only one layout but two active model rows likely breaks a menu
  roster/count assumption before the full model/resource table is initialized.

This means the next meaningful data-driven probe must either:

1. add a complete slot atomically without moving entry offsets; or
2. instrument the runtime getter/caller and log why `0x2e2` is queried during
   the menu build.

Testing more entry `35`-only active rows is now low value.

## Entry 32 Compression-Only Probe

Prepared and installed:

```text
target/reverse/LINKDATA_A.probe-entry32-recompress-only.BIN
```

Purpose: isolate whether the game rejects a recompressed `entry 32` payload even
when the inflated bytes are exactly identical.

Details:

- only `entry 32` is recompressed;
- inflated `entry 32` bytes verify byte-identical to the original;
- no logical strings are changed;
- no LinkData entry moves;
- original file size is preserved;
- original `entry 32` compressed span: `261193`;
- recompressed span at zlib level `6`: `260924`;
- available in-place capacity: `261376`.

Important compression observation:

- zlib levels `9` and `8` produced `261494` bytes and did not fit;
- zlib level `7` produced `261379` bytes and missed capacity by `3` bytes;
- zlib level `6` fit safely.

This explains why seemingly tiny `entry 32` edits are fragile: even when the
inflated data is unchanged, compression settings can push the payload over the
original in-place capacity.

Installed game backup:

```text
D:\SteamLibrary\steamapps\common\OPPW4\LINKDATA\CMN\LINKDATA_A.before-entry32-recompress-only.20260510-094755.BIN
```

Expected result:

- if it boots, in-place recompression itself is accepted and capacity/offsets
  are the main file-structure risk;
- if it black-screens, the game may depend on compressed chunk byte layout or a
  loader assumption our parser does not model.

Result: black screen.

Conclusion:

- `entry 32` cannot currently be safely rewritten on disk, even when the
  inflated bytes are exactly identical;
- the game appears sensitive to the compressed payload/chunk layout or some
  loader assumption outside the inflated table data;
- future `entry 32` changes should avoid disk recompression and move toward
  runtime memory patching or a lower-level read/inflate hook.

Restoration note: after this result, restore the clean `LINKDATA_A.BIN` from:

```text
D:\SteamLibrary\steamapps\common\OPPW4\LINKDATA\CMN\LINKDATA_A.before-entry32-recompress-only.20260510-094755.BIN
```

Restored clean game copy:

```text
D:\SteamLibrary\steamapps\common\OPPW4\LINKDATA\CMN\LINKDATA_A.BIN
SHA256 8431BB6DC4CACA80EB65AEECE0F87037BA82DAE149451F61FCF6B3F0BAB9C5F2
```

## Runtime LinkData RAM Probe

The compression-only probe makes disk-side `entry 32` edits unsafe for now. The
next approach is to keep `LINKDATA_A.BIN` byte-clean on disk and observe the
game after it has loaded/decompressed LinkData.

Added a DLL-side RAM probe. The probe now lives in its own crate so the proxy
does not own LinkData-specific experiment logic:

```text
crates/oppw4-linkdata/src/ram_probe.rs
```

Activation file:

```text
D:\SteamLibrary\steamapps\common\OPPW4\mods\_oppw4\linkdata_ram_probe.txt
```

Default patterns searched in writable process memory:

- `MPLC012_Newgate\0`;
- `806_044_costume_newgate\0`;
- `806_112_costume_hancock_ougenki\0`;
- `806_058_costume_law_dressrosa\0`.

Default config installed for the first probe:

```text
enabled=1
passes=45
interval_ms=2000
max_hits=80
```

Expected log lines:

```text
LinkData RAM probe enabled: ...
LinkData RAM hit pattern=<name> address=0x... region=0x...+0x... protect=0x...
```

Interpretation:

- hits in writable memory prove the relevant LinkData strings exist in a
  runtime buffer or parsed table;
- repeated hits across known model/layout strings give us candidate regions for
  an in-RAM patch path;
- no hits means the game may decompress, parse, then free the string buffer
  before our scan sees it, so the next step would be a lower-level inflate/read
  hook or a hook near the LinkData parser.

This probe does not modify memory yet. It only locates candidate RAM addresses.
If the addresses are stable enough, the next proof should be a tightly scoped
same-size RAM patch before attempting any extra-slot mutation.

## Runtime LinkData Same-Size RAM Patch Probe

The first RAM scan produced coherent hits for layout strings in one writable
region. Re-aligning the live addresses with the offline inflated entry `32`
showed:

```text
newgate_layout        live 0x29489f6457f - inflated 0x3e4ca = base 0x29489f260b5
hancock_dlc_layout    live 0x29489f64c48 - inflated 0x3eb93 = base 0x29489f260b5
law_dressrosa_layout  live 0x29489f646c7 - inflated 0x3e612 = base 0x29489f260b5
```

This strongly suggests those strings are part of one coherent runtime copy of
entry `32`.

Added an opt-in same-size RAM patch mode to:

```text
apps/dinput8-proxy/src/linkdata_ram.rs
```

Config syntax:

```text
patch=<source_string>=> <target_string>
```

Whitespace around the arrow is ignored. A trailing NUL is added internally if
omitted. Patches are ignored unless source and target have exactly the same byte
length after the NUL terminator is considered.

Installed first proof patch:

```text
patch=803_012_name_newgate=>803_012_name_NEWGATE
```

Reason for this target:

- same length;
- exists inside entry `32`;
- lower risk than mutating a critical `806_*` asset/layout path;
- useful as a write proof even if the game has already consumed this particular
  name key before the scan pass.

Expected log line:

```text
LinkData RAM patch name=803_012_name_newgate->803_012_name_NEWGATE address=0x... bytes=0x15 ...
```

If this line appears and the game survives, RAM writes are viable and the next
step is to patch either:

1. the runtime string/table before the game consumes it; or
2. the already-parsed costume roster structures if the string table patch is
   too late to affect UI/resource selection.

Latest correction:

- the same-size RAM write proof did work mechanically, but the timestamps showed
  the first patches happened before `LINKDATA_A.BIN` was opened by the game;
- those early hits were therefore probably from our own embedded/catalog memory,
  not from the live game LinkData buffer;
- the current branch keeps this probe as an opt-in LinkData crate, but disables
  the active patch while we return to the first real milestone: make the costume
  swap menu show a duplicate vanilla slot.

Proxy log noise is now quiet by default. To temporarily restore low-level file
I/O logs, create:

```text
D:\SteamLibrary\steamapps\common\OPPW4\mods\_oppw4\proxy_config.txt
```

with:

```text
verbose_io_logs=1
```

## Entry 32 Section 7 Hijack Candidate Tool

Added a registry slot inspection command for the "duplicate vanilla slot" path:

```text
oppw4-rdb --linkdata-registry-slots <linkdata-bin> \
  --section 7 \
  --replacement 806_141_costume_newgate \
  --max-results 24
```

Purpose:

- avoid appending to entry `32`, because disk recompression of entry `32` has
  already black-screened even with identical inflated bytes;
- find an existing section `7` string slot with enough byte capacity to be
  replaced in RAM by a new `806_*costume*` name;
- prefer slots with no aligned numeric references in inflated LinkData entries.

Current best candidates for `806_141_costume_newgate`:

| Section 7 ID | Current name | Bytes | Refs | Notes |
| ---: | --- | ---: | --- | --- |
| `2478` | `821_028_eula_french_pg05` | `25` | `u16=0`, `u32=0` | best current fit, one spare byte |
| `2489` | `821_039_eula_polish_pg04` | `25` | `u16=0`, `u32=0` | equivalent fit |
| `2499` | `821_049_eula_korean_pg01` | `25` | `u16=0`, `u32=0` | equivalent fit |
| `2457` | `821_007_eula_english_pg02` | `26` | `u16=0`, `u32=0` | two spare bytes |

Current duplicate-vanilla hypothesis:

- patch section `7` slot `2478` in the live entry `32` buffer from
  `821_028_eula_french_pg05` to `806_044_costume_newgate`;
- intentionally reuse suffix `44` first, because entry `39` and entry `58`
  already have valid Newgate config for that suffix;
- do not add a new `entry 35` model row for the first visible-slot test;
- the success condition is only that the costume swap menu shows a second
  Newgate slot that still resolves to vanilla Newgate.

Runtime config for this proof:

```text
enabled=1
passes=180
interval_ms=1000
max_hits=40
max_patches=1
entry32_patch=0x40fc3:821_028_eula_french_pg05=>806_044_costume_newgate
```

The offset `0x40fc3` is the string's inflated entry `32` offset. The DLL only
applies this patch if the candidate address also matches known entry `32`
anchors at their offline offsets, so embedded catalog false positives are
skipped.

Latest runtime correction:

- the first entry32 coherence check was too strict for the live game buffer;
- in the real large writable region, the Newgate/Hancock/Law costume layout
  anchors all aligned to the same runtime base;
- the EULA source string was shifted by `-0x390` relative to that base, so an
  exact `address - offline_offset` check rejected the useful candidate;
- the RAM patch now accepts an entry32 patch if the same read window contains
  all three known costume anchors, and logs this as `coherence=anchor_window`.

First successful entry32 RAM hijack log:

```text
LinkData RAM patch name=821_028_eula_french_pg05->806_044_costume_newgate address=0x211f2fe9437 coherence=anchor_window bytes=0x19 region=0x211d6f65000+0x8bf1a000
LinkData RAM hit pattern=newgate_layout address=0x211f2fe9437 region=0x211d6f65000+0x8bf1a000 protect=0x4
```

Interpretation:

- the source registry slot was patched in the same large runtime LinkData
  region that contains known Newgate/Hancock/Law costume layout strings;
- the follow-up `newgate_layout` hit at the exact patched address proves the
  string bytes were changed in memory;
- RAM editing of the runtime entry `32` string registry is now mechanically
  proven;
- if this does not make a duplicate costume appear in the UI, the missing piece
  is not "can we write entry 32", but "which parsed roster/list/count consumes
  the section 7 registry".

Follow-up probe direction:

- Newgate has no costume button by default, so a patched Newgate layout string
  may never be considered by the UI;
- the next probe should target a character that already has the costume button,
  such as Law;
- best current section `7` slot for a Law duplicate layout is ID `2591`,
  `826_031_eula_mss_english_pg10`;
- inflated entry `32` offset for that slot: `0x41bfb`;
- replacement: `806_058_costume_law_dressrosa`;
- if this produces a duplicate Law Dressrosa entry, the section `7` registry
  participates in the already-enabled costume menu path;
- if it still does not show, the menu is driven by an explicit roster/count
  table rather than section `7` string enumeration.

First Law duplicate attempt did not actually patch:

```text
LinkData RAM patch skipped name=826_031_eula_mss_english_pg10->806_058_costume_law_dressrosa address=0x21336eaa635 reason=entry32_coherence_failed
```

The known Newgate/Hancock/Law anchors appeared shortly after in the same large
runtime region, but in the next scan chunk. The source string and anchors are
within roughly `0x8700` bytes of each other, but the 64 KiB scan boundary split
them. The DLL now performs a nearby-region coherence read around entry32 patch
candidates and logs that path as `coherence=nearby_anchor_window`.

Retest result:

```text
LinkData RAM patch name=826_031_eula_mss_english_pg10->806_058_costume_law_dressrosa address=0x18952cff2b5 coherence=nearby_anchor_window bytes=0x1e region=0x18936d54000+0x8bf1a000
LinkData RAM hit pattern=law_dressrosa_layout address=0x18952cff2b5 region=0x18936d54000+0x8bf1a000 protect=0x4
```

The patch applied correctly and the patched address was subsequently seen as a
Law Dressrosa layout string. The in-game Law costume menu still showed the same
three entries.

Conclusion:

- changing the runtime entry `32` string registry is not enough to add a
  visible menu slot;
- this path is only a more elaborate registry/name swap;
- the menu is driven by another explicit roster/list/count structure, probably
  referencing section `7` IDs or costume suffixes rather than enumerating
  string contents.

Entry `52` was rechecked because proximity scans showed `{layout, owner}` pairs
there. Added a focused dump command:

```text
oppw4-rdb --linkdata-entry52-costume-records <linkdata-bin> \
  --owners <ids> \
  --layouts <suffixes>
```

Result for Newgate/Law:

```text
groups: owner=12 layout=44/57/58/111/131 count=1 each
groups: owner=26 layout=44/57/58/111/131 count=1 each
```

Entry `52` is therefore also not the roster: it contains a broad owner/layout
matrix where Newgate already has rows for Law suffixes.

ScreenLayout/KIDS note:

- `105_05_costume_change.kscl` lives in `ScreenLayout` hash `0x386d71a0`;
- `Layout_105_05_costume_change.kidssingletondb` lives in
  `KIDSSystemResource` hash `0xfbfc2a79`;
- the KIDSDB is a better UI-data candidate than the KSCL alone, but the failed
  Law duplicate proves the missing data is not simply the existence of a
  `806_*` layout asset.

KIDS/UI follow-up:

- added `oppw4-rdb --kidsdb-dump` to inspect `IDRK0000` / `IDOK0000`
  containers;
- `Layout_105_05_costume_change.kidssingletondb` has 10 `IDOK` chunks;
- `Layout_105_00_chara_select.kidssingletondb` has 11 `IDOK` chunks;
- both files share the same repeated object-record pattern, so this singleton DB
  currently looks more like a UI object list than the playable costume roster.

The compressed `105_05_costume_change.kscl` payload inflates to `LCSK9500` and
contains UI template names such as:

```text
806_000_costume
806_000_costume_0000
806_000_costume_0001
806_000_costume_0002
806_000_costume_0003
806_000_costume_0004
806_000_costume_0005
```

This means the costume screen already has visual placeholders for up to six
entries. The fact that Law still shows three entries after the entry `32` string
patch strongly suggests that another runtime list/count decides which
placeholders are populated.

DLC gating clue:

- LinkData says Law owns four layout candidates:
  - `806_057_costume_law`;
  - `806_058_costume_law_dressrosa`;
  - `806_111_costume_law_souhi`;
  - `806_131_costume_law_oni`;
- LinkData also lists two Law DLC costume strings:
  - `DLC_COSTUME_005_555_026_002`;
  - `DLC_COSTUME_006_586_026_003`;
- the local `FILE/DLC` directory currently contains
  `DLC_COSTUME_006_586_026_003.bin`, but not
  `DLC_COSTUME_005_555_026_002.bin`;
- this matches the observed count of three visible Law costumes: base,
  Dressrosa, and one installed DLC costume.

Current hypothesis:

```text
visible costume list = base/free layouts + installed DLC costume gates + save/runtime filtering
```

The next runtime probe uses raw hex patterns instead of only strings. The proxy
now supports:

```text
pattern_hex=<name>:<hex bytes>
patch_hex=<name>:<from hex>=> <to hex>
context_bytes=<n>
```

Installed probe config for the next Law test searches the active and full Law
suffix/model sequences:

```text
pattern_hex=law_active_suffixes_u16:39003a008300
pattern_hex=law_full_suffixes_u16:39003a006f008300
pattern_hex=law_active_suffixes_u32:390000003a00000083000000
pattern_hex=law_full_suffixes_u32:390000003a0000006f00000083000000
pattern_hex=law_active_count_suffixes_u32:03000000390000003a00000083000000
pattern_hex=law_full_count_suffixes_u32:04000000390000003a0000006f00000083000000
pattern_hex=law_active_models_u16:1a00e3003401
pattern_hex=law_full_models_u16:1a00e30010013401
pattern_hex=law_active_models_u32:1a000000e300000034010000
pattern_hex=law_full_models_u32:1a000000e30000001001000034010000
```

Success criteria for this probe:

- if `active_*` patterns appear after opening Law's costume menu, dump their
  context and identify the surrounding count/list structure;
- if `full_*` patterns appear but active patterns do not, the installed-DLC
  filter probably transforms LinkData's full set into a smaller runtime set;
- if neither appears, search for costume indexes (`0,1,3`) or section `7`
  string IDs (`2105,2106,2179`) next.

First raw-hex probe correction:

- the first run found the raw patterns immediately, but the contexts contained
  probe/config memory such as `law_active_suffixes_u32`,
  `PROCESSOR_LEVEL=23`, and unrelated file names;
- those were self-hits from the probe's own heap buffers, not game-owned
  runtime lists;
- the RAM probe now ignores addresses that fall inside its own pattern/patch
  byte buffers before logging hits or applying patches.

Retest with region filtering:

- using `min_region_size=0x100000` removed the false small-region hits;
- no crash;
- only the large LinkData string registry hit remained:

```text
806_057_costume_law
806_058_costume_law_dressrosa
806_059_costume_doflamingo
```

- none of the raw active/full Law suffix or model sequences appeared in the
  large region.

Interpretation: exact raw sequences such as `57,58,131` or
`26,227,308` are not the active roster shape in the large LinkData string
region. The visible list may be built elsewhere, or the values are stored
non-contiguously.

Fast DLC-gate probe:

- local `FILE/DLC` had `DLC_COSTUME_006_586_026_003.bin`;
- created a temporary copy as `DLC_COSTUME_005_555_026_002.bin`;
- if Law gains a fourth costume entry, the official slot already existed and
  the missing part was DLC marker gating rather than roster injection.

Ethics/cleanliness correction:

- `DLC_COSTUME_005_555_026_002` is an official costume DLC that is not owned in
  this install;
- the temporary marker file was removed;
- future custom DLC experiments should use patcher-owned custom IDs/names, not
  spoof official DLC ownership.

DLC discovery probe:

- configured the RAM probe to search only for owned/missing DLC evidence and
  likely numeric representations:

```text
pattern=DLC_COSTUME_006_586_026_003
pattern=DLC_COSTUME_005_555_026_002
pattern=DLC_CHARACTER_018_972
pattern=DLC_CHARACTER_021_975
pattern_hex=dlc_owned_costume_006_u16:06004a021a000300
pattern_hex=dlc_owned_costume_006_u32:060000004a0200001a00000003000000
pattern_hex=dlc_owned_costume_006_tail_u16:4a021a000300
pattern_hex=dlc_owned_costume_006_tail_u32:4a0200001a00000003000000
pattern_hex=dlc_missing_costume_005_u16:05002b021a000200
pattern_hex=dlc_missing_costume_005_u32:050000002b0200001a00000002000000
pattern_hex=law_costume_indexes_u16:000001000300
pattern_hex=law_costume_indexes_u32:000000000100000003000000
```

Goal: find where the game stores validated/installed DLC state at runtime.
If a compact owned-DLC table exists, a later custom-DLC path can target a
patcher-owned code such as `DLC_COSTUME_999_999_026_004` without spoofing
official entitlements.

## 2026-05-10 Runtime Slot Experiments

Confirmed Law owner data from active `LINKDATA_A.BIN`:

- owner id: `26`;
- model rows: `26` (`MPLC026_Law`), `227` (`MVAR025_Law_DR`),
  `272` (`MDLC033_Law_Souhi`), `308` (`MDLC069_Law_Oni`);
- costume layouts: `57`, `58`, `111`, `131`;
- DLC gates:
  - `DLC_COSTUME_005_555_026_002` for costume index `2`;
  - `DLC_COSTUME_006_586_026_003` for costume index `3`.

Observed behavior:

- the game shows only three Law costume slots on this install;
- `DLC_COSTUME_006_586_026_003.bin` is present locally;
- `DLC_COSTUME_005_555_026_002.bin` is not owned/present;
- the RDB virtualization itself is healthy: all seven Law Onigashima
  replacement assets open correctly once a visible Law slot is selected.

Failed approach: direct LinkData table mutation.

- A rebuilt `LINKDATA_A.BIN` that appended a synthetic Law model/layout slot
  caused a black screen at boot.
- An in-place layout-only test using a same-size `LINKDATA_A.BIN` mutation
  also caused a black screen.
- This points to boot-time LinkData invariants or checks outside the local
  costume records we patched.
- Active `LINKDATA_A.BIN` was restored to the clean backup afterward.

Current clean state:

- active `LINKDATA_A.BIN` matches
  `LINKDATA_A.before-law-custom-slot.20260510-135154.BIN` by SHA-256;
- Law dump is back to `model_count=4`, `layout_count=4`;
- `mods/_oppw4/linkdata_ram_probe.txt` is disabled with `enabled=0`;
- black-screen variants were kept as reference files, not installed:
  - `LINKDATA_A.black-screen-patched.20260510-135839.BIN`;
  - `LINKDATA_A.black-screen-layout130.20260510-141601.BIN`.

Next approach:

- stop mutating LinkData costume tables on disk;
- instrument runtime call stacks around `FILE/DLC/DLC_COSTUME_*`,
  `FILE/DLC/DLC_CHARACTER_*`, and save-slot reads;
- identify the function that builds or filters the visible costume list;
- hook that function only for Law (`owner_id=26`) to append or remap a custom
  slot after the game has loaded its normal data;
- avoid a global "DLC owned" hook so official paid DLC entitlements are not
  spoofed.

Runtime list pivot:

- the DLC-file route is now considered a side path: logs show the game directly
  opens the already-known `DLC_COSTUME_006_586_026_003.bin` and does not scan
  `FILE/DLC`;
- that implies a pre-existing runtime list/db has already decided the candidate
  costumes before file I/O;
- the current proxy build triggers a one-shot writable-memory scan directly
  when the owned Law DLC costume file is opened;
- logged candidates use `Law roster scan hit ...` and search compact visible
  and full Law sequences:
  - indexes `0,1,3` and `0,1,2,3`;
  - layouts `57,58,131` and `57,58,111,131`;
  - model rows `26,227,308` and `26,227,272,308`;
  - section IDs `2105,2106,2179` and `2105,2106,2159,2179`;
  - known `806_*costume_law*` strings;
- shader/log self-hits are filtered by context, and the scan is observational
  only: no RAM patching, no LinkData mutation.
- a first threaded/delayed version crashed before the trigger during D3D11
  startup; the active fix removes the background thread and keeps the scan
  synchronous at the Law DLC file-open point.
- first synchronous scan booted and triggered correctly, but saturated its
  96-hit cap with generic index patterns (`0,1,3` and `0,1,2,3`) before the
  useful patterns could prove anything;
- the next scan revision prioritizes strong Law signals (layout IDs, model
  rows, section IDs, `806_*law*` strings), caps hits per pattern, and only logs
  generic index sequences when the surrounding bytes contain a Law-like anchor.

Steam public API probe result:

- a later build hooked static SteamApps imports and dynamic `GetProcAddress`
  SteamApps resolutions;
- with `trace_dlc_stacks=1`, the game still opened only the already-known
  `DLC_COSTUME_006_586_026_003.bin`;
- no `GetProcAddress SteamApps...` or `SteamApps ... app_id=...` line appeared;
- conclusion: the public SteamApps route is not the current useful boundary for
  costume visibility. The probe is now disabled by default and can be restored
  only with `steam_probe=1` in `mods/_oppw4/proxy_config.txt`.

Next diagnostic build:

- broadened stack tracing to include `LINKDATA_A.BIN`, any `FILE/DLC` file,
  DLC character files, and likely save paths;
- for each new game stack frame seen during those opens, the proxy logs a short
  `CreateFileW code ... bytes=<hex>` window around the `OPPW4.exe` frame;
- purpose: use the repeated runtime caller
  `game+0xa78d8b -> game+0xa78bae -> game+0x1147535` as the next real boundary,
  instead of continuing broad memory/hash scans.

## 2026-05-10 Global Costume Table Pivot

The DLC-file path is now treated as an observable symptom, not the source of
truth for selectable costumes. Logs prove the game opens the known Law DLC
costume file only after an already-built runtime list has decided that the slot
can exist.

New primary target: the function that builds or filters the global visible
costume table.

Strong static lead from `OPPW4.exe`:

- `FUN_1416137b0 @ 1416137b0` walks `0xfa` (`250`) rows and writes a sorted
  visible list to its output buffer.
- It reads the table through `DAT_141eba738 + 0x18`.
- Main row storage appears at `*(DAT_141eba738 + 0x18 + 0x28)`.
- Per-row primary stride is `0xdc`.
- A secondary per-row block uses stride `0xc8` (`200`) around offsets
  `0xd7c6` and `0xd7c8`.
- `base + row * 0xc8 + 0xd7c8` is compared with the function's `param_1`.
  Caller audit correction: this is probably a small list/group id, not the
  playable character id. `FUN_1412f7560` and `FUN_1412f7260` both imply this
  value is expected to be `< 10`.
- `base + row * 0xc8 + 0xd7c6` is copied as a sort/order key.
- Row flag `base + row * 0xdc + 0xb6` bit `0` enables the row.
- Row flag `base + row * 0xdc + 0xb6` bit `0x8000` hides/disables the row.
- `FUN_1412f7560(row)` filters rows by a small byte value extracted from the
  secondary block; current interpretation: the same `0xd7c8` list/group id must
  be less than `10`.

Related functions:

- `FUN_1412f7260 @ 1412f7260`: checks whether any visible row exists for a
  given small list/group id using the same `0xd7c8` byte.
- `FUN_1412f7320 @ 1412f7320`: similar existence check for another category
  using primary-row offset `0xb8`.
- `FUN_1412f7630 @ 1412f7630`: validates rows using the primary `0xdc` stride
  and a category byte near `0xb8`.
- `FUN_1412f7140 @ 1412f7140`: reads runtime/save visibility bits from
  `DAT_141eba750 + 0x18`.
- `FUN_1412f77a0 @ 1412f77a0`: updates visibility/unlock bits for many rows in
  `DAT_141eba750 + 0x18 + 0x10`, including offsets around `0x1548`.
- `FUN_14160fbd0 @ 14160fbd0`: another sorted list builder using the same
  global table, probably for a neighboring category or UI mode.

Working hypothesis:

```text
visible costume list =
  global 250-row roster table
  + per-row owner/category/model/layout fields
  + runtime/save visibility bits
  + DLC/ownership gates
```

Next action: audit callers and xrefs around `FUN_1416137b0`,
`FUN_14160fbd0`, `FUN_1412f77a0`, and the `DAT_141eba738` /
`DAT_141eba750` table accesses. The goal is to find the function that fills the
250-row table, then patch or extend the already-parsed runtime table instead of
mutating `LINKDATA_A.BIN` or spoofing DLC ownership.

### Focused Ghidra audit

Script added:

- `oppw4-ghidra/AuditGameCostumeRoster.java`.

Output produced:

- `oppw4-ghidra/game_costume_roster_audit.txt`.

Direct callers of `FUN_1416137b0`:

- `FUN_14119cea0 @ 14119cea0`: builds the global owner-0 list and checks
  runtime row state at `DAT_141eba750 + 0x18 + 0x10 + row * 0x68 + 0x154e`.
- `FUN_1414716a0 @ 1414716a0`: builds the global owner-0 list and checks
  whether a requested row id exists in it.
- `FUN_141560e30 @ 141560e30`: strongest menu lead. It clears
  `param_1 + 0x144`, clears count `param_1 + 0x538`, asks
  `FUN_1415615b0(param_1)` for the current owner/category, then calls
  `FUN_1416137b0(owner, local_rows, 0xfb, param_1 + 0x538)`. It copies the
  returned rows into `param_1 + 0x144`, inserts sentinel `0x7fffffff` in one UI
  mode, and clamps the selected index at `param_1 + 0x550`.
- `FUN_141576320 @ 141576320`: iterates owner/category ids from
  `DAT_141723e28`, calls `FUN_1416137b0`, then checks runtime bits at
  `DAT_141eba750 + 0x18 + 0x10 + row * 0x68 + 0x1548`.
- `FUN_1415784b0 @ 1415784b0`: another UI/list builder. It calls
  `FUN_1416137b0(*(param + 0x70), ...)` while building vectors from a menu
  structure.

Callers of `FUN_141560e30`:

- `FUN_14155f280 @ 14155f280`: reacts to menu state changes and refreshes the
  costume list through `FUN_141560e30`.
- `FUN_141561740 @ 141561740`: initializes/copies menu state from a parameter
  block and calls `FUN_141560e30(param_1, *(param_2 + 4))` for mode `2`.

`FUN_1415ed230 @ 1415ed230` is the best runtime-state initializer lead:

- Clears or initializes blocks at `param_1 + 0x10`, `param_1 + 0x1548`,
  `param_1 + 0xfe6c`, and `param_1 + 0x1026c`.
- Walks the same `0xfa` rows from `DAT_141eba738 + 0x18 + 0x28`.
- Reads static row flags at `row * 0xdc + 0xb6`.
- Reads a per-row marker at `0x19a84 + row * 0x12`.
- Reads the list/group byte at `row * 0xc8 + 0xd7c8`.
- Writes runtime visibility/availability bits at `param_1 + row * 0x68 +
  0x1548`.
- Also updates group-level state near `param_1 + 0xe560 + group`.

Current interpretation:

- `DAT_141eba738` is the static global roster/costume database.
- `DAT_141eba750` is the runtime/save mirror used to decide whether entries are
  visible, owned, new, hidden, or selected.
- `FUN_1415ed230` probably initializes the runtime mirror from the already
  loaded static table.
- `FUN_141560e30` is likely the practical visible-costume-list rebuild for the
  UI.

Important negative result:

- The focused decompile did not show normal gameplay code writing the static
  row group/order fields (`0xd7c8`, `0xd7c6`) directly.
- That makes the original static table look data-loaded rather than
  field-built in this gameplay layer.
- So the true static-table builder may be a generic LinkData/database loader,
  not a costume-specific function near the DLC code.

Implication for a real extra Law skin:

- A pure "append to menu list" hook at `FUN_141560e30` is cleaner than a fake
  custom menu, but it is still only a list-level patch. The row id must still
  point to valid static row data and runtime visibility bits.
- A more genuine route is to clone or repurpose an unused/hidden row inside the
  existing `0xfa` row limit, set its static list group, character/category,
  model/layout fields, then mirror the correct runtime bits in
  `DAT_141eba750`.
- Adding a row beyond `0xfa` is likely much more invasive because the loop
  bounds and buffer sizes are hard-coded in multiple functions.

Next best diagnostic:

- Add a runtime dump of the 250-row table once `DAT_141eba738` and
  `DAT_141eba750` are initialized.
- For each row, log the static flags, `0xd7c8` list group, sort key,
  candidate character/category bytes around `0xb8` and `0xbc`, and runtime
  flags around `0x1548`.
- First filter of interest: rows whose `0xb8` or `0xbc` field equals `26`
  (candidate Law id), plus disabled/hidden/empty rows that could be safe clone
  targets.

### Runtime dump implementation

Implemented a gated runtime diagnostic instead of another broad memory scan.

Files:

- `crates/oppw4-research/src/costume_table.rs`: decodes the global static table
  and runtime mirror through `DAT_141eba738` / `DAT_141eba750`.
- `apps/dinput8-proxy/src/hooks.rs`: triggers the dump once when an interesting
  game file opens.
- `apps/dinput8-proxy/src/loader.rs`: reads the config flag.

Config:

```text
dump_costume_table=1
```

Add that to `mods/_oppw4/proxy_config.txt`. `trace_dlc_stacks=1` is not
required for the table dump, though it can still be enabled at the same time.

Trigger paths:

- `LinkData/CMN/LINKDATA_A.BIN`
- `FILE/DLC/DLC_COSTUME_*`
- Law-pack DLC character files already used by the stack tracer

The dump retries a few times because the first `LINKDATA_A.BIN` open can happen
before both globals are ready. On success it logs:

- static/runtime table addresses;
- total row count;
- enabled, hidden, Law-candidate, and clone-candidate counts;
- group counts for the small `0xd7c8` list/group id;
- compact row lines for enabled rows, runtime-marked rows, and rows whose
  `0xb8` or `0xbc` field equals `26`.

Verification run:

- `cargo test -p oppw4-research --manifest-path oppw4-patcher-rs\Cargo.toml`
  passed.
- `cargo test -p oppw4-dinput8-proxy --manifest-path oppw4-patcher-rs\Cargo.toml`
  passed.
- `cargo check --manifest-path oppw4-patcher-rs\Cargo.toml` passed.
- `cargo build -p oppw4-dinput8-proxy --manifest-path oppw4-patcher-rs\Cargo.toml`
  passed.

Deployment for runtime test:

- Installed `target/debug/dinput8.dll` to
  `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.dll`.
- Previous DLL backup:
  `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-costume-table-dump.20260510-184209.dll`.
- Enabled `dump_costume_table=1` in
  `D:\SteamLibrary\steamapps\common\OPPW4\mods\_oppw4\proxy_config.txt`.

### First runtime dump result

Runtime log `2026-05-10_18-44-28.log` proved the global table path is real:

```text
static_rows=0x2b5712e1760 runtime_rows=0x2b5714b8670 rows=250
enabled=240 hidden=80 groups=0:32,1:33,2:39,3:6
```

Expected early behavior:

- the first `LINKDATA_A.BIN` trigger can happen before `DAT_141eba738` is
  initialized, producing `NullPointer("static database global")`;
- a later DLC trigger succeeds once the static and runtime globals are ready.

Important correction:

- primary row offsets `0xb8` and `0xbc` are not playable character IDs.
- `FUN_1412f7630`, `FUN_1412f7320`, and `FUN_14160fbd0` show those bytes are
  small category/list selectors.
- The previous `law_candidate` naming was misleading; a zero count there does
  not disprove the table.

New signal from the dump:

- the likely Law anchor is the secondary sort/order key `26`, not `0xb8` or
  `0xbc`;
- rows `133` and `139` had `sort=26`, matching Law's known owner/model ID;
- this makes the next focused test: inspect raw static bytes around rows whose
  sort key is `26`, plus nearby known costume/layout keys `57`, `58`, `111`,
  `131`, and relation key `168`.

Instrumentation update after this result:

- renamed logged fields from `char_a` / `char_b` to `cat_a` / `cat_b`;
- renamed summary count from `law_candidates` to `sort26_candidates`;
- added per-group visible row lists sorted like `FUN_1416137b0`;
- added focused raw byte dumps for interesting rows:
  - primary row bytes around `+0x4c`;
  - primary category area around `+0xb0`;
  - secondary area around `+0xd7c0`;
  - runtime mirror bytes around `+0x1548`.

Current working hypothesis:

```text
FUN_141560e30 menu list =
  FUN_1416137b0(current_group, rows...)
  sorted by secondary key at 0xd7c6

Law likely appears as rows where secondary sort key == 26,
while model/layout/costume index fields live elsewhere in the primary row bytes.
```

Next evidence to collect:

- run once with the updated DLL and compare the detailed row dumps for
  `sort=26` against known Law model/layout IDs:
  - model rows `26`, `227`, `272`, `308`;
  - layouts `57`, `58`, `111`, `131`;
  - DLC costume indexes `2`, `3`.
- If these bytes reveal the row schema, use an existing hidden row as a clone
  target before considering any `FUN_141560e30` detour.

### Menu row to layout validation path

The second runtime log confirmed the row dump, and the focused decompile gives a
better path than fixating on DLC file opens:

```text
visible costume list row
  -> selected row id from FUN_141560e30 / FUN_1416137b0
  -> selected layout id at menu state +0x554
  -> FUN_1412f8020(layout_id, selected_row, category_mode)
  -> FUN_1416132b0(layout_id, selected_row, category_mode, out_layout)
```

Important functions:

- `FUN_1415615b0 @ 1415615b0`: returns the current list/category selector used
  by `FUN_141560e30`.
- `FUN_141560e30 @ 141560e30`: rebuilds the visible row list by calling
  `FUN_1416137b0`.
- `FUN_14155e5b0 @ 14155e5b0`: chooses/clamps the selected layout id and calls
  `FUN_1412f8020`.
- `FUN_1412f8020 @ 1412f8020`: validates whether a layout id is available for a
  selected row/category.
- `FUN_1416132b0 @ 1416132b0`: resolves the final variant id from the selected
  layout row, selected costume row, and category mode.

New structural notes from Ghidra:

- `DAT_141eba738 + 0x18 + 0x28` is the already-dumped 250-row visible costume
  table.
- `DAT_141eba738 + 0x18 + 0x08` is a second table used as the selected
  layout/costume table.
- Layout rows are accessed as `layout_id * 0x44`.
- Important layout row offsets:
  - `+0x14`: family/group id used by variant fallback logic;
  - `+0x16`: category/owner id, validated against the runtime category gate;
  - `+0x28`: 16 `u16` variant ids;
  - `+0x4a`: flag byte used by special fallback logic.
- `FUN_1412f8020(layout, row, 0)` reads a row/layout matrix byte at:

```text
static_rows + row * 0xdc + 0xbd + layout_id
```

- For category modes `1` and `2`, `FUN_1412f8020` mainly validates the layout
  row's category through `FUN_1412f9320`.

Runtime dump result from `2026-05-10_18-59-54.log`:

- rows `133` and `139` are the only visible `sort=26` rows;
- their raw byte dumps did not directly contain known Law model ids
  `26/227/272/308` or Law layout ids `57/58/111/131`;
- therefore the 250-row table is most likely the visible menu list, while the
  model/layout variants live in the `static_root + 0x08` layout table.

Instrumentation update:

- Added a layout-table dump alongside the existing 250-row dump.
- The new dump logs:
  - known layout ids `44,57,58,111,131,136,141,0x221`;
  - layouts whose category is Newgate `12` or Law `26`;
  - layouts whose `+0x28` variant list contains known Law model ids
    `26,227,272,308`;
  - row/layout matrix bytes for rows `133,136,139,140` against the known layout
    ids.

Verification:

- `cargo fmt --all` passed.
- `cargo test -p oppw4-research` passed.
- `cargo test -p oppw4-dinput8-proxy` passed.
- `cargo check` passed.
- `cargo build -p oppw4-dinput8-proxy` passed.

Deployment:

- Installed the rebuilt DLL to
  `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.dll`.
- Previous DLL backup:
  `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-layout-table-dump-robust.20260510-192032.dll`.
- The layout dump only reads runtime category gates and raw bytes after a row
  matches the focused logging filter, so unrelated layout rows cannot abort the
  diagnostic as easily.
- Config remains:

```text
verbose_io_logs=0
trace_dlc_stacks=1
dump_costume_table=1
```

Next evidence to collect:

- Run the game once and inspect the new `Global costume layout ...` lines.
- If Law layouts `57/58/111/131` show category `26` and variant ids matching
  `26/227/272/308`, then the next target is cloning one valid layout row plus
  its runtime category gates, not modifying DLC file-open behavior.

### Layout-table dump result

Runtime log `2026-05-10_19-22-26.log` produced the first useful layout-table
proof:

```text
Global costume layout dump ...
static_layouts=0x24c8abe5fc0 static_rows=0x24c8ac17760
logged_layouts=13 logged_law_category_layouts=1 logged_newgate_category_layouts=1
```

Key rows:

| Layout row | Meaning now | Category | Variants |
| --- | --- | --- | --- |
| `12` | Newgate master row candidate | `12` | `36` |
| `26` | Law master row | `26` | `57,58,555,586` |
| `57` | Law/base child row candidate | `57` | `410` |
| `58` | Law/Dressrosa child row candidate | `58` | `411` |
| `555` | not dumped yet; referenced by Law master row | unknown | unknown |
| `586` | not dumped yet; referenced by Law master row and DLC file name | unknown | unknown |

The decisive observation is `layout id=26`:

```text
category=26 variants=57,58,555,586
```

This lines up with the live file-open trace:

```text
DLC_COSTUME_006_586_026_003.bin
```

So `586` is almost certainly Law's installed DLC costume variant id, with owner
category `26` and costume index `3`. The old assumption that `57/58/111/131`
were all equal "Law layout ids" was too broad:

- `57` and `58` are real child rows under Law's master row;
- `111` and `131` are valid string/screen-layout suffixes from other data, but
  in this `static_root + 0x08` table they have category `65535`, so
  `FUN_1412f8020` rejects them as selected layout rows before matrix validation.

The matrix probes also support this split:

```text
row 133 layout 57 = 0x01
row 133 layout 58 = 0x01
row 139 layout 57 = 0x01
row 139 layout 58 = 0x01
row 133/139 layout 141 = 0x00
row 133/139 layout 545 = 0x00
```

Non-zero bytes at `111` and `131` exist, but because those rows have category
`65535`, they are probably incidental bytes for this specific validation path.

Updated model:

```text
visible row list (250 rows)
  -> selected visible row, e.g. 133 or 139 for Law sort=26
  -> master layout row 26
  -> variant list at layout row 26 +0x28 = 57,58,555,586
  -> DLC/resource path uses variant id + owner id + costume index
```

Most useful next diagnostic:

- dump the child variant rows referenced by master rows:
  - Law: `57,58,555,586`;
  - Newgate: `36`;
  - existing target/probe rows: `410,411`;
- decode the variant row fields enough to clone `586` or another installed DLC
  child row into the first free slot of Law master row `26`.

Follow-up instrumentation:

- Expanded the focused layout filter to include master/child rows:
  `6,12,26,36,44,57,58,77,111,131,136,141,358,410,411,511,545,555,559,586`.
- Expanded the variant-interest filter to include the newly observed child ids:
  `410,411,555,586,559,77`.
- This should reveal whether `555` and `586` are normal child rows, DLC-gated
  rows, or indirections to another table.

### Law child-row dump result

Runtime log `2026-05-10_19-37-01.log` clarified the master/child/terminal
chain.

Useful layout rows:

| Row | Category | Variants | Interpretation |
| --- | --- | --- | --- |
| `26` | `26` | `57,58,555,586` | Law master row |
| `57` | `57` | `410` | Law base/free child candidate |
| `58` | `58` | `411` | Law Dressrosa/free child candidate |
| `410` | `65535` | `263` | terminal/resource-like row under `57` |
| `411` | `65535` | `264` | terminal/resource-like row under `58` |
| `555` | `32` | `353` | real child row referenced by Law master |
| `586` | `65535` | all `65535` | terminal/resource id, not a full child row |

Important correction:

- `586` is definitely referenced by Law master row `26`, and the file-open path
  proves it is used by the live DLC costume:

```text
row 26 variants = 57,58,555,586
DLC_COSTUME_006_586_026_003.bin
```

- But layout row `586` itself is empty-like:

```text
family=65535 category=65535 flags_4a=0x00 variants=all 65535
```

So `586` should not be treated as a normal layout row to clone. It is more
likely a terminal DLC/resource id consumed by later path-building/resource
lookup code.

The hierarchy now looks like:

```text
Law master row 26
  slot 0 -> 57  -> 410 -> 263
  slot 1 -> 58  -> 411 -> 264
  slot 2 -> 555 -> 353
  slot 3 -> 586 -> DLC_COSTUME_006_586_026_003.bin
  slot 4 -> free
```

The best low-risk proof for a true extra slot is now:

1. patch Law master row `26` in RAM only;
2. write `586` into the first free variant slot (`+0x28`, index `4`);
3. observe whether the menu shows a duplicate Law DLC costume slot;
4. if the game requests a new file such as an index `004` path, alias it back
   to the existing `003` file as a temporary proof.

If this duplicate-slot proof works, then a real new costume needs:

- a new terminal variant id in row `26`'s variant list;
- a matching file/path/resource mapping for that new variant;
- later, proper strings/layout/UI assets.

Current best candidate for adding a real extra selectable Law costume is no
longer a `FUN_141560e30` detour. It is:

1. append or duplicate a terminal variant id in master row `26`'s free `+0x28`
   list;
2. use `586` as the first duplicate-slot proof because the current game already
   opens `DLC_COSTUME_006_586_026_003.bin`;
3. provide matching file/string/layout resources for the new variant;
4. only then patch visibility/unlock/runtime bits if needed.

### Duplicate Law slot probe

Implemented a guarded runtime probe behind:

```text
duplicate_law_variant_slot=1
```

When enabled, the proxy waits until the global costume layout table is ready,
then patches only one `u16` in RAM:

```text
layout row 26, variant slot index 4: 65535 -> 586
```

The patch logs the address and before/after values:

```text
Law duplicate variant patch layout=26 slot=4 address=... before=65535 after=586 target=586
```

This is intentionally a duplicate-slot proof, not the final new-costume
implementation. The expected next observations are:

- if the menu shows an extra Law slot, the master variant path is confirmed;
- if selecting it requests a path such as `DLC_COSTUME_006_586_026_004.bin`,
  the next probe should alias that path to the existing `_003` file;
- if no extra slot appears, the list/count source is elsewhere and must be
  patched in addition to row `26`'s variant list.

Follow-up timing fix:

- The first probe patched after the original `CreateFileW` returned. That
  proves the RAM write works, but it may be too late if the game builds/caches
  the costume UI list before or during the DLC open.
- The proxy now attempts the table dump and duplicate-slot patch before the
  original file open (`phase=pre_open`) and keeps the old after-open path as a
  fallback (`phase=post_open`).
- Next log should show whether the successful dump/patch happened in
  `phase=pre_open`.

Follow-up count fix:

- The `phase=pre_open` log confirmed the slot write happens early enough:
  `layout=26 slot=4 before=65535 after=586`.
- The menu still showed only the existing slots, so the next suspected gate is
  the active variant count byte in each layout row.
- Layout row `26` has variants `57,58,555,586,65535...` and an active-count
  byte at `+0x48` set to `4`. Comparable rows line up with this interpretation:
  row `511` has three variants and count `3`, row `559` has five variants and
  count `5`, and one-variant rows use count `1`.
- The duplicate-slot probe now also patches layout row `26` active count:

```text
layout row 26, active variant count: 4 -> 5
```

Expected log:

```text
Law duplicate variant count patch layout=26 address=... before=4 after=5 target=5
```

If this still leaves the menu at three visible Law slots, the remaining filter
is likely a UI/save/unlock layer rather than the master layout variant list
alone.

Runtime log `2026-05-10_21-20-01.log` confirmed the count fix writes correctly:

```text
Law duplicate variant patch layout=26 slot=4 ... before=65535 after=586 target=586
Law duplicate variant count patch layout=26 ... before=4 after=5 target=5
```

The same run still resolves the selected Law master layout through the old path:

```text
Costume resolver layout=26 row=70 mode=2 ... result=57 ...
```

It also proves the game opens the known installed Law Oni DLC file after the RAM
patch:

```text
DLC_COSTUME_006_586_026_003.bin
```

Current conclusion:

- the master layout variant array and active-count byte are real, writable, and
  useful;
- they are not sufficient to make the fourth Law costume selectable;
- the remaining gate is probably the menu slot-selection layer around
  `FUN_14155ec30` / `FUN_14155e5b0`, or a save/unlock field consumed before the
  resolver.

Instrumentation update:

- `Costume menu visible rows ...` now logs additional menu-state fields:
  - `slot_index` from menu state `+0xa8`;
  - `row_override` from `+0xac`;
  - `selected_variant` from `+0x558`.

Next evidence to collect:

- Run with `trace_costume_menu=1`, `dump_costume_table=1`, and
  `duplicate_law_variant_slot=1`.
- Inspect `Costume menu visible rows ...` lines after selecting Law and moving
  through costume slots.
- If `slot_index` never reaches `4`, continue around the two hard-coded
  `CMP ..., 0x3` UI bounds.
- If `slot_index` reaches `4` but `selected_variant` or resolver output falls
  back to `57`, focus on the `FUN_14155e5b0` slot-to-layout/variant copy path.

Runtime log `2026-05-10_21-40-58.log`:

- confirmed the release DLL with the menu-state trace was installed;
- confirmed both UI byte patches applied:

```text
Law duplicate UI selected slot bound ... before=0x03 after=0x04
Law duplicate UI variant slot bound ... before=0x03 after=0x04
```

- confirmed the duplicate layout variant and active-count patches still apply:

```text
Law duplicate variant patch layout=26 slot=4 ... before=65535 after=586
Law duplicate variant count patch layout=26 ... before=4 after=5
```

- confirmed the selected Law resolver path still returns the base Law variant:

```text
Costume resolver layout=26 row=70 mode=2 ... result=57
```

- produced no `Costume menu visible rows ...` entries, even though the
  visible-row hook installed successfully.

This means the current Law costume navigation path does not rebuild the visible
row list through `FUN_141560e30` during the observed selection. The useful next
hook is closer to the selection/routing path:

- `FUN_14155e5b0`, which selects/clamps layout and calls `FUN_1416132b0`;
- `FUN_14155ec30`, which consumes row/slot UI state;
- possibly `FUN_141559c60`, the low-level slot-to-row variant helper.

The `Law menu row slot probe` remains:

```text
u16=8,8,8,8,0,1,2,1
```

So `row=70 +0xa8` is not a direct list of final variant ids and does not contain
the injected `586` value.

Selection-state trace DLL installed 2026-05-10 21:55:

- built with `cargo build --release -p oppw4-dinput8-proxy`;
- installed to `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.dll`;
- backup: `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-selection-state-trace.20260510-215513.dll`.

This build traces all three current hypotheses in one run:

- `FUN_14155e5b0` as `Costume selection selected-helper ...`;
- `FUN_14155ec30` as `Costume selection row-slot-ui ...`;
- `FUN_141559c60` as `Costume selection slot-variant ...`.

The expected decision matrix for the next log:

- if `slot-variant row=70 slot=4 row_slot_before=0x024a` appears, the injected
  slot is reachable and the remaining problem is selection state or resolver
  fallback;
- if the selected helper reaches `slot_index=4` but `focus_variant` remains
  `57`, inspect the `+0x558` variant-index copy/clamp path;
- if none of these helpers ever see `slot=4`, keep following the pre-selection
  filter/unlock/list construction path before the UI helpers.

Runtime log `2026-05-10_21-57-04.log`:

- the three selection/UI hooks installed, but produced no
  `Costume selection ...` calls;
- the old validator/resolver path still fired for Law row `70`;
- the duplicate slot/count patch still applied correctly;
- `FUN_1414901a0` / `FUN_141490320` / `FUN_1414906a0` /
  `FUN_1414926a0` are now the more useful path because the resolver stack
  contains `game+0x149025e`, `game+0x1491c7f`, and related frames.

Scene-state trace DLL installed 2026-05-10 22:06:

- built with `cargo build --release -p oppw4-dinput8-proxy`;
- installed to `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.dll`;
- backup: `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-scene-state-trace.20260510-220657.dll`.

New trace lines:

- `Costume scene scene-presentation ...` for `FUN_1414901a0`;
- `Costume scene scene-available-check ...` for `FUN_141490320`;
- `Costume scene scene-apply ...` for `FUN_1414906a0`;
- `Costume scene scene-state-refresh ...` for `FUN_141491120`;
- `Costume scene scene-preview-refresh ...` for `FUN_1414926a0`;
- `Costume scene scene-fallback-layout ...` for `FUN_141492970`.

The key fields now logged from the real costume scene object:

- `layout` from `+0x404`;
- `row` from `+0x438`;
- `selected_variant` from `+0x43c`;
- `selected_slot`, resolved by scanning the current layout's `+0x28` variant
  list;
- selected object pointer from `+0x110`;
- `object_layout` from selected object `+0x440`;
- `object_variant` from selected object `+0x448`;
- `object_slot`, resolved the same way as `selected_slot`.

Runtime log `2026-05-10_22-08-05.log`:

- the scene hooks finally hit the real Law costume path;
- `scene-available-check` sees `layout=26`, `row=70`, `selected_variant=586`
  and returns `1`;
- this proves the Law Oni variant is accepted by the scene path, but the
  duplicate `586` probe is ambiguous because `586` already exists as Law's
  original slot 3;
- the next diagnostic build writes a unique probe value, `410`, into injected
  slot 4 instead of duplicating `586`.

Expected result for the next log:

- `selected_variant=410 selected_slot=4` means the injected fifth slot is
  reachable;
- no `410` in the scene traces means the slot is still filtered or skipped
  before selection reaches the scene object.

Runtime log `2026-05-10_22-16-30.log`:

- the unique slot probe writes correctly:

```text
Law unique variant probe patch layout=26 slot=4 ... before=65535 after=410 target=410
```

- the scene still reaches Law as `selected_variant=586 selected_slot=3`;
- no scene trace reaches `selected_variant=410`;
- `FUN_140210b60` is now identified as the selected layout/category lookup:

```c
u16 FUN_140210b60(table, category, layout)
```

- for normal layouts it returns `*(u16 *)(table + 0x1148 + category * 2)`;
- the matching setter is `FUN_1400b16c0`, which writes to the same table.

Diagnostic update:

- hook `FUN_140210b60` at `game+0x210b60`;
- when `category=26`, `layout=26`, and the original result is `586`, return
  the unique probe value `410`;
- this tests whether the scene path can accept the injected fifth Law slot once
  the selection cache is forced past the old stored value.

Runtime log `2026-05-10_22-25-24.log`:

- the selection lookup hook does force the cache lookup:

```text
Costume selection lookup ... category=26 layout=26 original_result=586 result=410 forced=true
```

- immediately after that, the resolver still reports Law layout `26` as
  `result=57`;
- the scene object remains the stronger source of truth:

```text
scene-apply ... before=[... selected_variant=58 ... object_variant=586 ...] after=[... selected_variant=586 ... object_variant=586 ...]
```

- when the scene object is `58`, `scene-apply` copies `58`; when it is `586`,
  it copies `586`.

Diagnostic update installed 2026-05-10 22:32:

- built with `cargo build --release -p oppw4-dinput8-proxy`;
- tests: `cargo test -p oppw4-dinput8-proxy` (`29 passed`);
- installed to `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.dll`;
- backup:
  `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-scene-object-probe.20260510-223232.dll`;
- new probe: before `scene-apply`, if Law `layout=26,row=70,mode=2` has a
  selected object with `object_layout=26` and `object_variant=586`, patch only
  selected object `+0x448` to the unique variant `410`.

Expected next-log decision:

- `Law scene object probe patch ... after_object_variant=410` followed by
  `scene-apply ... after=[... selected_variant=410 selected_slot=4 ...]` proves
  that the selected object is the final gate;
- if it immediately returns to `586`, the overwrite happens below
  `scene-apply`;
- if the object reaches `410` but the preview/model fails, the slot is selected
  but asset/model routing still follows the old variant path.

Runtime log `2026-05-10_22-34-30.log`:

- the scene object probe proved the game accepts the fifth Law slot:

```text
Law scene object probe patch ... before_object_variant=586 after_object_variant=410
scene-available-check ... selected_variant=410 selected_slot=4 ... result=1
scene-apply ... forced_object=true ... after=[... selected_variant=410 selected_slot=4 ...]
```

- in-game this produced Robin while playing Law, so `410` is useful as a unique
  proof id but not a valid final Law master-slot id;
- no new UI slot was visible, because the probe hijacked an existing selected
  object rather than making the list expose slot index `4`.

Diagnostic update installed 2026-05-10 22:46:

- removed the `scene-apply` object-force probe to stop hijacking the existing
  Law costume;
- kept the master row `26` slot/index `4` patch and active count `5`;
- added the missing UI visibility bounds in `FUN_141559cd0`:

```text
141559d06 CMP EDX,0x3  -> 0x4
141559d0f CMP EDX,0x4  -> 0x5
```

- existing UI bounds remain:

```text
14155eca1 CMP EDI,0x3   -> 0x4
141559c9c CMP R11D,0x3  -> 0x4
```

- built with `cargo build --release -p oppw4-dinput8-proxy`;
- tests: `cargo test -p oppw4-dinput8-proxy` (`28 passed`);
- installed to `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.dll`;
- backup:
  `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-slot5-ui-bounds.20260510-224607.dll`.

Expected next-log decision:

- if the fifth button appears, `FUN_141559cd0` was the missing visibility gate;
- if no fifth button appears, the remaining gate is earlier list construction,
  probably before `FUN_14155ec30`/`FUN_141559c60`;
- if a fifth button appears but selects Robin, keep the UI path and replace the
  unique probe id with a proper cloned Law child row/resource id.

Runtime log `2026-05-10_22-47-34.log`:

- all four UI bound patches applied successfully;
- the master row still patched slot/index `4` and active count `5`;
- no fifth UI slot appeared;
- the old UI helper hooks (`selected-helper`, `row-slot UI`, `slot-variant`) did
  not fire in this scene path, so the visible list is still built or filtered
  before those helpers;
- `scene-apply` still copied the selected object with `object_variant=586`,
  meaning the forced-Robin test was only hijacking the scene object, not exposing
  a real slot.

Diagnostic update installed 2026-05-10 22:55:

- changed the injected fifth Law master-row slot back from unique probe `410` to
  duplicate Law Oni `586`, so the next test attempts a safe visible slot 5
  instead of a Robin-producing proof id;
- disabled the effective `586 -> 410` selection lookup override by making the
  target equal to `586`;
- added a trace hook for `FUN_141491370` as
  `costume scene update-dispatcher trace hook`, logged as
  `scene-update-dispatcher`, to locate where the selected scene object/list gets
  fixed before `scene-apply`;
- built with `cargo build --release -p oppw4-dinput8-proxy`;
- tests: `cargo test -p oppw4-dinput8-proxy` (`28 passed`);
- installed to `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.dll`;
- backup:
  `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-slot5-duplicate586-dispatcher.20260510-225549.dll`.

Expected next-log decision:

- if a fifth slot appears, the unique probe value was being filtered and the
  duplicate `586` path is usable for UI work;
- if no fifth slot appears, inspect `scene-update-dispatcher` lines to see
  whether `object_variant` or `selected_variant` is already fixed to slot `3`
  before scene apply.

Runtime log `2026-05-10_22-56-53.log`:

- the duplicate `586` slot-5 patch writes correctly:

```text
Law extra slot variant patch layout=26 slot=4 ... before=65535 after=586 target=586
Law duplicate variant count patch ... before=4 after=5 target=5
```

- the UI still shows only three visual slots;
- `scene-update-dispatcher` proves the scene state is assembled directly as
  `layout=26 selected_variant=586 selected_slot=3`;
- because `586` now appears twice in layout row `26`, `selected_slot=3` is
  ambiguous in traces, but the missing visual button means the visible slot list
  is still being built from a separate cache/list before the master row's fifth
  variant is exposed;
- no old UI helper trace appears except installation, so the important boundary
  is now the selection cache setter and the scene-local list at `state+0x120`.

Diagnostic update installed 2026-05-10 23:04:

- added hook `FUN_1400b16c0` as `costume selection setter trace hook`, logged as
  `Costume selection setter ...`, to see every write into the selected variant
  cache around category/layout `26`;
- added `Costume scene list probe ...` for the scene list bank/entry around
  `state+0x120`, with bank starts from `state+0x3e0`, layout ids, and per-entry
  flags;
- kept the safe duplicate slot-5 target `586`;
- built with `cargo build --release -p oppw4-dinput8-proxy`;
- tests: `cargo test -p oppw4-dinput8-proxy` (`28 passed`);
- installed to `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.dll`;
- backup:
  `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-selection-setter-list-probe.20260510-230420.dll`.

Expected next-log decision:

- if the setter only cycles `57,58,586`, the UI/list layer has already collapsed
  the fifth entry before writing the selected cache;
- if the scene list probe contains only three Law-relevant entries, patch/list
  construction must happen around `FUN_141493820`;
- if the list contains five entries but the setter only writes three, the next
  target is the input/navigation clamp rather than the layout table.

Runtime log `2026-05-10_23-05-19.log`:

- `FUN_1400b16c0` hook installed but produced no `Costume selection setter`
  lines on this path, so it is not the observed setter for the visible Law slot
  list;
- the duplicate `586` slot-5 patch still writes and the layout count still
  patches to `5`;
- the scene-local list at `state+0x120` for bank `2` contains layout/category
  ids such as `26,49,50,45,14,11,27,15,10,13`, not Law's terminal variant ids;
- selection lookup for Law category/layout `26/26` cycles `57`, `58`, and `586`,
  but never `555`;
- duplicating `586` therefore cannot prove a visual fifth slot if the visible
  layer deduplicates by final variant id.

Diagnostic update installed 2026-05-10 23:17:

- added hook `FUN_1412f52a0` as `Costume variant unlock-check ...`;
- for this test only, force `category=26 variant=555` from result `0` to `1`;
- this checks whether the missing Law official variant `555` is hidden by the
  unlock/admission gate before the UI has a chance to expose more visual slots;
- built with `cargo build --release -p oppw4-dinput8-proxy`;
- tests: `cargo test -p oppw4-dinput8-proxy` (`29 passed`);
- installed to `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.dll`;
- backup:
  `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-law-555-unlock-test.20260510-231705.dll`.

Next log lines to inspect:

- `Costume variant unlock-check category=26 variant=555 ... forced=true`;
- any new `DLC_COSTUME_006_555...` `CreateFileW` lines;
- whether `Costume selection lookup category=26 layout=26` can now return `555`;
- whether the visible Law menu grows from three slots to four/five.

User validation after the 23:17 build:

- the visible Law menu grew from three slots to four;
- the unlocked fourth slot is playable in-game;
- no missing texture was observed;
- this proves variant `555` is a real Law variant hidden by the
  category/variant unlock gate, not by missing assets or by a hardcoded
  three-slot UI limit.

Stabilization update:

- split the `555` unlock into its own config flag:
  `unlock_law_hidden_variant=1`;
- `duplicate_law_variant_slot=1` remains an experimental fifth-slot probe and
  no longer controls the `555` unlock;
- keep using `trace_costume_menu=1` while researching the real fifth slot, but
  the stable four-slot Law unlock only needs `unlock_law_hidden_variant=1`.

Next direction for a true fifth slot:

- do not duplicate `586`; the visible layer appears to collapse duplicate final
  variant ids;
- create or hijack a unique terminal variant id, then route its metadata and
  file/resource behavior back to Law assets instead of Robin/other characters;
- once a unique Law-like id exists, reuse the same unlock gate hook narrowly for
  that id.

Diagnostic update after the stable `555` split:

- changed the experimental fifth-slot probe from duplicate final variant `586`
  to unique variant id `587`;
- kept the selection lookup override disabled, so the existing `586` slot is not
  hijacked into the probe id;
- when `duplicate_law_variant_slot=1`, the unlock-check hook also forces
  `category=26 variant=587` from `0` to `1`;
- added a narrow `CreateFileW` alias:

```text
DLC_COSTUME_006_587_026_004.bin -> DLC_COSTUME_006_586_026_003.bin
```

Expected next-log decision:

- if a fifth visual slot appears and logs the alias line, the remaining work is
  naming/metadata polish for a real cloned Law variant;
- if no fifth visual slot appears, the visible list is still filtering unknown
  terminal variant ids before the file path stage;
- if the fifth slot appears but still loads the wrong model, the missing piece
  is not the DLC file open but the variant-to-resource metadata table.

User validation of the `587` probe:

- the fifth-slot path is far enough along to load a character;
- variant id `587` loaded Koby, proving `587` is not a free terminal variant id;
- the next diagnostic must stop guessing ids and scan the layout table for:
  - owners of the current probe id;
  - empty layout ids that are not referenced as variants.

Diagnostic update:

- `CostumeLayoutTableDump` now records all terminal variant owners, not only the
  interesting layouts printed in full;
- it also records empty, unreferenced layout ids as free-candidate hints;
- added log line:

```text
Global costume variant id probe variant=587 owners=... free_layout_candidates_first=...
```

Use that line to pick the next probe id or decide where to clone the Law
metadata row.

Dynamic allocation update:

- the fifth-slot probe no longer hardcodes `587` as the inserted variant id;
- at runtime, the patcher reads `free_layout_candidates` from the global
  costume layout dump;
- it picks a candidate only if that id is not already owned by an existing
  terminal variant;
- current policy is to use the highest safe free candidate, so low historical
  ids remain untouched where possible;
- the DLC file alias is generated from the chosen id:

```text
DLC_COSTUME_006_<allocated>_026_004.bin -> DLC_COSTUME_006_586_026_003.bin
```

Current custom allocation table:

| Character | Slot | Source variant/file | Allocated variant | Mode | Status |
| --- | ---: | --- | --- | --- | --- |
| Law | `4` | `586` / `DLC_COSTUME_006_586_026_003.bin` | runtime free id | `auto-free-layout` | experimental fifth-slot probe |

Expected runtime log:

```text
Custom variant allocation table entries=1 rows=Law slot=4 source=586 allocated=<id> mode=auto-free-layout
Law extra slot variant patch layout=26 slot=4 ... target=<id>
Costume variant unlock-check category=26 variant=<id> ... forced=true
```

Important interpretation:

- if the selected free id shows a fifth slot and loads Law Oni assets, file
  aliasing plus free-id allocation is enough for a basic injected slot;
- if the slot appears but loads a blank/wrong model, the next required step is
  a full metadata clone into the allocated id;
- if the slot still does not appear, the remaining filter is before the DLC file
  path stage and the allocated id must be admitted into the visible costume list.

Runtime log `2026-05-11_00-12-59.log`:

- dynamic allocation is working and selected variant id `724`:

```text
Custom variant allocation table entries=1 rows=Law slot=4 source=586 allocated=724 mode=auto-free-layout
Law extra slot variant patch layout=26 slot=4 ... after=724 target=724
Law duplicate variant count patch layout=26 ... after=5 target=5
```

- the stable hidden Law variant unlock still works:

```text
Costume variant unlock-check category=26 variant=555 ... original_result=0 result=1 forced=true
```

- the game never reaches the dynamically allocated id:
  - no `Costume variant unlock-check category=26 variant=724`;
  - no `DLC_COSTUME_006_724_026_004.bin`;
  - no scene trace with `selected_variant=724`;
  - Law selection continues through `selected_variant=586 selected_slot=3`;
  - `Costume selection lookup category=26 layout=26` still returns `586`.

Current conclusion:

- `587` loaded Koby because it was already an owned terminal variant with real
  metadata;
- `724` is a genuinely free id, so it avoids stealing another character, but it
  has no metadata for the game to admit or display;
- therefore the fifth-slot blocker is no longer "find a free id"; it is
  "clone/retarget complete costume metadata into the allocated free id".

Current progress estimate:

| Area | Status |
| --- | --- |
| RDB replacement/virtualization | working |
| Law official hidden slot `555` | working and playable |
| UI hard limits for slot count | patched enough for the official fourth slot |
| Duplicate final variant probe | proven insufficient because visible layer deduplicates/collapses |
| Existing-id proof (`587`) | proved metadata is required, but stole Koby |
| Dynamic free-id allocation (`724`) | working, clean id chosen and written |
| Dynamic DLC file alias | implemented, but not reached for free id yet |
| Metadata clone into free id | next missing step |
| TOML/custom skin manifest | not implemented yet |

Practical estimate: the proof-of-concept extra slot is roughly `85%` there,
with about `15%` remaining for the first real custom-slot prototype. The
remaining work is concentrated in one hard piece: clone/retarget the complete
variant metadata into the allocated free id, then the existing unlock and file
aliasing path should have something valid to select.

Runtime metadata clone probe installed 2026-05-11:

- no disk table patching;
- `LINKDATA_A.BIN` remains untouched;
- the DLL now deep-copies one runtime layout metadata row before inserting the
  allocated id into Law's master row;
- source metadata row is `555`, because `586` is a valid Law DLC file/variant
  id but its layout row is empty;
- asset/file source remains `586` via the dynamic DLC alias path;
- allocation table log now includes both source roles:

```text
Custom variant allocation table entries=1 rows=Law slot=4 source=586 metadata=555 allocated=<id> mode=auto-free-layout
Law extra slot metadata clone source_layout=555 target_layout=<id> ... match=true
Law extra slot variant patch layout=26 slot=4 ... target=<id>
```

Build/install details:

- built with `cargo build --release -p oppw4-dinput8-proxy`;
- tests:
  - `cargo test -p oppw4-research` (`7 passed`);
  - `cargo test -p oppw4-dinput8-proxy` (`37 passed`);
- installed to `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.dll`;
- backup:
  `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-runtime-metadata-clone.20260511-180601.dll`.

Expected next-log decision:

- if `variant=<id>` appears in `Costume variant unlock-check`, the metadata
  clone got the free id admitted into the costume path;
- if `selected_variant=<id>` appears, the fifth slot is now selectable and the
  next step is routing custom assets;
- if the log still never reaches the allocated id, the missing metadata is not
  only the layout row and we must clone/patch another runtime structure.

Runtime metadata clone result from `2026-05-11_18-06-57.log`:

- allocation picked `724`;
- runtime layout metadata clone succeeded:
  `source_layout=555 target_layout=724 bytes=75 match=true`;
- Law master row patch succeeded:
  `slot=4 before=65535 after=724 target=724`;
- active variant count patch succeeded: `4 -> 5`;
- UI bound/count patches were applied;
- no fifth slot appeared in-game;
- the game never called unlock-check for variant `724`;
- no `DLC_COSTUME_006_724_026_004.bin` request appeared;
- Law stayed on `selected_variant=586 selected_slot=3`;
- the visible/selection path still enumerated the existing Law set
  `57,58,555,586` instead of the patched fifth id.

Initial conclusion before the Ghidra follow-up: cloning the 75-byte terminal
static layout row appeared insufficient, because `724` never reached the
unlock-check or DLC alias path. The stack pointed toward the
unlock-check/scene-list builder (`game+0x1490d44`, `game+0x1491d3b`,
`game+0x16150de`) as the next place to inspect.

Follow-up from Ghidra:

- `FUN_141490320` only calls the variant unlock-check when the candidate variant
  id is `< 0x2c0` (`< 704`);
- the dynamic allocator picked `724`, so the game skipped it before unlock-check;
- this explains why there was no `variant=724` unlock log and no DLC alias
  request.

Patch installed 2026-05-11:

- allocator now rejects free ids `>= 704`;
- next runtime allocation should choose the highest free candidate below that
  selectable limit;
- release build installed to `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.dll`;
- backup:
  `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-selectable-id-limit.20260511-181530.dll`.

Runtime log `2026-05-11_18-22-26.log`:

- allocator picked `699`, proving the `< 704` limit is now respected;
- runtime layout metadata clone succeeded: `source_layout=555 target_layout=699`;
- Law master row patch succeeded: `slot=4 before=65535 after=699`;
- active count patch succeeded: `4 -> 5`;
- no fifth visual slot appeared;
- the game still never called unlock-check for `variant=699`;
- the visible path continued to call only `57,58,555,586`.

Ghidra follow-up:

- in `FUN_141490320`, after the `< 0x2c0` check, the game reads a per-variant
  record at `static_layouts + 0xd940 + variant_id * 0x1e`;
- it requires flags from that record before calling `FUN_1412f52a0`
  (unlock-check);
- copying the 75-byte layout row did not populate this per-variant record, so
  `699` was still filtered before unlock-check.

Patch installed 2026-05-11:

- clone per-variant metadata record from source variant `586` into the allocated
  id (`699` in the latest run);
- record address formula:
  `static_layouts + 0xd940 + variant_id * 0x1e`;
- keep layout-row metadata clone and DLC file alias unchanged;
- release build installed to `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.dll`;
- backup:
  `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-variant-record-clone.20260511-183408.dll`.

Expected next-log decision:

- if `Costume variant unlock-check category=26 variant=<allocated>` appears, the
  per-variant record was the admission gate;
- if `DLC_COSTUME_006_<allocated>_026_004.bin` appears, the path reaches file
  routing and the alias should map it to Law Oni `586`;
- if no fifth slot appears but the unlock-check is reached, the next blocker is
  selection/list presentation rather than metadata admission.

Crash follow-up `2026-05-11_18-38-44.log`:

- per-variant metadata clone worked and `variant=699` reached unlock-check;
- the hook then forced `category=26 variant=699` from `0` to `1`;
- the game crashed immediately after this admission step, before any
  `DLC_COSTUME_006_699_026_004.bin` request and before selecting `699`;
- `crash.log` reports top-level exception `e06d7363` via `KERNELBASE.dll`, so
  this is likely a downstream C++ data-invariant failure rather than a direct
  access violation in our hook.

Conclusion:

- `static_layouts + 0xd940 + variant_id * 0x1e` is confirmed as one admission
  gate;
- forcing the newly allocated id through that gate is not safe yet;
- keep the metadata clone for tracing, but disable the experimental forced
  unlock for the allocated id until the next table/list/selection structure is
  identified and cloned.

Runtime log `2026-05-11_18-47-31.log` with the guard enabled:

- no crash was observed in the skin menu path;
- `variant=699` still reaches `FUN_1412f52a0`;
- the hook correctly leaves it alone:

```text
Costume variant unlock-check category=26 variant=699 slot=4 strict=0 original_result=0 result=0 forced=false
```

- no `DLC_COSTUME_006_699_026_004.bin` request appears;
- no `selected_variant=699` appears;
- this confirms the next blocker is inside the real unlock state, not the layout
  row insertion or variant metadata admission.

Ghidra interpretation of `FUN_1412f52a0`:

- after category/variant bounds and `0xd940` metadata checks, it reads a runtime
  byte from:

```text
runtime_root + (category + 10) * 8 -> pointer + 0x160 + slot
```

- for Law, this means category `26`, source slot `3`, target slot `4`;
- next RAM-only experiment: copy the runtime unlock slot byte from Law slot `3`
  to slot `4`, still without forcing the function return.

Runtime log `2026-05-11_18-59-41.log`:

- the runtime byte clone worked:

```text
Law extra slot runtime unlock slot clone category=26 source_slot=3 target_slot=4 before=0x00 source_value=0x0a after=0x0a match=true
```

- `variant=699` still returned `original_result=0 result=0`;
- because source variant `586` has metadata flags `0x07`, the cloned custom
  variant still takes the official DLC entitlement branch in `FUN_1412f52a0`;
- `699` is not present in the entitlement/name tables, so that branch rejects it
  before file aliasing or selection.

Next RAM-only experiment:

- keep the cloned metadata record, but patch the target variant flags from
  `0x07` to `0x05` for the allocated id, clearing the official DLC entitlement
  bit while keeping the record enabled;
- patch the runtime slot byte from `0x0a` to `0x0b`, setting the direct unlock
  bit for slot `4`;
- this should make `FUN_1412f52a0` use its non-DLC custom-like branch instead of
  the entitlement lookup, still without forcing the function return.

Crash follow-up `2026-05-11_19-06-21.log`:

- the non-DLC/custom admission experiment worked mechanically:

```text
Law extra slot variant metadata flags patch target_variant=699 before=0x07 after=0x05
Law extra slot runtime unlock slot clone category=26 source_slot=3 target_slot=4 source_value=0x0a target_value=0x0b after=0x0b
Costume variant unlock-check category=26 variant=699 slot=4 strict=0 original_result=1 result=1 forced=false
```

- the game crashed just after that real admission, before any
  `selected_variant=699` scene trace and before any
  `DLC_COSTUME_006_699_026_004.bin` request;
- `crash.log` again reports top-level exception `e06d7363` through
  `KERNELBASE.dll`;
- conclusion: the flags/runtime-slot branch is sufficient to pass
  `FUN_1412f52a0`, but some downstream scene/list/presentation structure still
  does not exist for the allocated id.

Guardrail:

- keep the layout clone, per-variant metadata clone, runtime slot clone, and
  traces;
- disable the custom/non-DLC admission patch by default:
  - do not patch `0x07 -> 0x05`;
  - clone Law slot `3` runtime byte as `0x0a`, not `0x0b`;
- expected safe trace returns to:

```text
Law extra slot custom non-DLC admission disabled target_variant=699 reason=post_unlock_crash_guard
Costume variant unlock-check category=26 variant=699 slot=4 strict=0 original_result=0 result=0 forced=false
```

Next target:

- find the downstream structure built between a successful unlock-check and the
  costume scene selection/list entry;
- likely around the `game+0x1490d44 -> game+0x1491d3b -> game+0x16150de`
  caller path observed in the crash run.

Runtime log `2026-05-11_19-12-23.log`:

- the crash guard worked:

```text
Law extra slot custom non-DLC admission disabled target_variant=699 reason=post_unlock_crash_guard
Law extra slot runtime unlock slot clone ... source_value=0x0a target_value=0x0a after=0x0a
Costume variant unlock-check category=26 variant=699 slot=4 strict=0 original_result=0 result=0 forced=false
```

- the game did not request `DLC_COSTUME_006_699_026_004.bin`;
- the scene stayed on `selected_variant=586 selected_slot=3`;
- importantly, `FUN_141490320` now enumerates the allocated fifth candidate:

```text
Costume variant unlock-check category=26 variant=57 slot=0 ... result=1
Costume variant unlock-check category=26 variant=58 slot=1 ... result=1
Costume variant unlock-check category=26 variant=555 slot=2 ... result=1 forced=true
Costume variant unlock-check category=26 variant=586 slot=3 ... result=1
Costume variant unlock-check category=26 variant=699 slot=4 ... result=0
```

Conclusion:

- row/count/visibility/per-variant admission are now far enough for the scene
  available-check to see slot `4`;
- the remaining blocker is the real unlock/presentation path for the allocated
  id;
- the last crash likely came from an incoherent clone: layout metadata came from
  `555`, while per-variant metadata came from `586`.

Next RAM-only experiment:

- clone the layout metadata and the per-variant metadata from the same known-good
  hidden Law source id `555`;
- keep the asset/file source alias pointed at Law Oni `586`;
- re-enable the custom/non-DLC admission patch for the allocated id;
- expected decision:
  - if `699` passes and does not crash, the missing invariant was metadata
    coherence;
  - if it still crashes after `original_result=1`, another table after
    `FUN_141490320` still needs cloning.

Runtime log `2026-05-11_19-20-42.log`:

- the coherent `555 -> 699` layout/per-variant metadata clone worked
  mechanically:

```text
Law extra slot metadata clone source_layout=555 target_layout=699 ... match=true
Law extra slot variant metadata clone source_variant=555 target_variant=699 ... match=true
Law extra slot variant metadata flags patch target_variant=699 before=0x07 after=0x05
Law extra slot runtime unlock slot clone ... target_value=0x0b after=0x0b
```

- the custom variant then passed the real unlock function without a forced
  return:

```text
Costume variant unlock-check category=26 variant=699 slot=4 strict=0 original_result=1 result=1 forced=false
```

- the game still crashed immediately after that point, before any
  `selected_variant=699` scene trace and before any
  `DLC_COSTUME_006_699_026_004.bin` request;
- conclusion: metadata coherence was not the missing invariant. The crash is
  downstream of `FUN_141490320`, likely inside or immediately after the
  `FUN_141490c30` call from the `FUN_141491370` dispatcher path:

```text
game+0x1490d44 -> game+0x1491d3b -> game+0x16150de
```

Guardrail restored:

- keep allocation, layout clone, per-variant metadata clone, runtime slot clone,
  file aliasing, and traces;
- disable the custom/non-DLC admission patch by default again:
  - do not patch target variant flags `0x07 -> 0x05`;
  - clone Law slot `3` runtime byte as `0x0a`, not `0x0b`;
- expected safe trace:

```text
Law extra slot custom non-DLC admission disabled target_variant=699 reason=post_unlock_crash_guard
Costume variant unlock-check category=26 variant=699 slot=4 strict=0 original_result=0 result=0 forced=false
```

Next target:

- instrument or decompile the branch after `FUN_141490320` succeeds, especially
  `FUN_141490c30`;
- identify which selected-object/list/presentation structure is missing for a
  newly allocated layout id before allowing `699` to pass admission again.

Runtime log `2026-05-11_19-28-18.log`:

- the restored guard is confirmed in-game:

```text
Law extra slot custom non-DLC admission disabled target_variant=699 reason=post_unlock_crash_guard
Law extra slot runtime unlock slot clone ... source_value=0x0a target_value=0x0a after=0x0a
```

- the runtime still injects the fifth candidate into Law's row:

```text
Law extra slot variant patch layout=26 slot=4 before=65535 after=699 target=699
Law duplicate variant count patch layout=26 before=4 after=5 target=5
```

- `FUN_141490320` still reaches the allocated id, but now refuses it normally:

```text
Costume variant unlock-check category=26 variant=699 slot=4 strict=0 original_result=0 result=0 forced=false
```

- the scene remains stable on the existing Law Oni slot:

```text
selected_layout=26 selected_variant=586 selected_slot=3
```

Interpretation:

- the guard prevents the post-unlock crash path;
- the allocated id is still visible to the internal candidate loop;
- the next experiment must target the downstream object/list/presentation path,
  not the file alias layer.

Diagnostic update `2026-05-11 19:43`:

- exported and instrumented `FUN_141490c30` (`game+0x1490c30`), the helper called
  after `FUN_141490320` returns success in the costume scene dispatcher;
- safe hook prologue length is `14` bytes:

```text
141490c30  PUSH RBX
141490c32  SUB RSP,0x50
141490c36  CMP qword ptr [RCX + 0x110],0x0
```

- important fields consumed by the function:
  - `state+0x110`: selected object, required non-null;
  - `state+0x404`: selected layout;
  - `state+0x438`: selected row;
  - the function builds a stack descriptor from layout/row/global UI pointers,
    calls `FUN_141499180(selected_object, descriptor)`, then calls the selected
    object's virtual method at `vtable+0x20`;
- runtime trace now logs:

```text
Costume scene scene-post-available-enter ...
Costume scene scene-post-available-leave ...
```

- the detail payload includes `state+0xf0/f8/100/108`, `selected_object`,
  selected-object vtable, selected-object `vtable+0x20`, object flags, and
  global action/costume widget pointers;
- custom/non-DLC admission for `699` remains disabled. This diagnostic DLL is
  meant to first capture the normal `586` path and prove the hook is stable
  before re-enabling the crash path;
- release DLL installed:
  `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.dll`;
- backup:
  `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-post-available-transition-trace.20260511-194346.dll`;
- verification:
  - `cargo test -p oppw4-dinput8-proxy`: 41 passed;
  - `cargo test -p oppw4-research`: 7 passed;
  - `cargo build --release -p oppw4-dinput8-proxy`: passed;
  - installed SHA-256 matched the release build.

Next log request:

- confirm hook installation:

```text
costume scene post-available transition trace hook installed
```

- then capture the normal path around Law/Oni:

```text
Costume scene scene-post-available-enter ...
Costume scene scene-post-available-leave ...
Costume variant unlock-check category=26 variant=699 ...
```

If the normal path is stable, the next dangerous test is to re-enable
custom/non-DLC admission only after this hook is confirmed. If the game crashes
inside `FUN_141490c30`, the `scene-post-available-enter` line should still tell
which selected object/list descriptor was handed to the crashing path.

Runtime log `2026-05-11_19-47-52.log`:

- `FUN_141490c30` hook is confirmed stable on the normal Law Oni path:

```text
costume scene post-available transition trace hook installed
Costume scene scene-post-available-enter ...
Costume scene scene-post-available-leave ...
```

- inside that function, the selected-object helper enumerates all Law variants,
  including the injected fifth candidate:

```text
Costume variant unlock-check category=26 variant=57 slot=0 ... result=1
Costume variant unlock-check category=26 variant=58 slot=1 ... result=1
Costume variant unlock-check category=26 variant=555 slot=2 ... result=1 forced=true
Costume variant unlock-check category=26 variant=586 slot=3 ... result=1
Costume variant unlock-check category=26 variant=699 slot=4 ... result=0
```

- `scene-post-available-leave` shows the normal path survives and still selects
  `586`.

Ghidra follow-up for `FUN_141499180`:

- `FUN_141490c30` passes a small descriptor to `FUN_141499180`;
- that helper builds the selected-object presentation list;
- after a variant passes `FUN_1412f52a0`, it later reads the variant display
  fields from:

```text
static_layouts + 0xd92c + variant_id * 0x1e
static_layouts + 0xd938 + variant_id * 0x1e
static_layouts + 0xd940 + variant_id * 0x1e
static_layouts + 0xd946 + variant_id * 0x1e
```

- previous code cloned only from `0xd940`, which was enough for the
  unlock/admission flag but left the presentation fields at `0xd92c` and
  `0xd938` empty/stale for `699`;
- this explains the observed crash pattern: `699` could pass unlock, then the
  selected-object presentation builder consumed incomplete metadata before any
  DLC file request.

Patch installed `2026-05-11 19:57`:

- variant metadata clone now starts at `0xd92c`;
- clone size remains `0x1e`;
- admission flags live at `record + 0x14` (`0xd940`);
- custom/non-DLC admission is re-enabled for the allocated id so the next run can
  test the corrected full record:

```text
Law extra slot variant metadata clone source_variant=555 target_variant=699 ... bytes=30
Law extra slot variant metadata flags patch target_variant=699 ... before=0x07 after=0x05
Law extra slot runtime unlock slot clone ... target_value=0x0b
```

- release DLL installed:
  `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.dll`;
- backup:
  `D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-full-variant-record-admission-test.20260511-195740.dll`;
- verification:
  - `cargo test -p oppw4-dinput8-proxy`: 42 passed;
  - `cargo test -p oppw4-research`: 7 passed;
  - `cargo build --release -p oppw4-dinput8-proxy`: passed;
  - installed SHA-256 matched the release build.

Next-log decision:

- if `selected_variant=699` or `DLC_COSTUME_006_699_026_004.bin` appears, the
  missing root cause was the incomplete variant metadata record;
- if it still crashes after `variant=699 ... result=1`, compare whether
  `scene-post-available-enter` appears without `scene-post-available-leave`.

Runtime patch update `2026-05-11 20:29`:

- the first custom-skin zip test is staged outside the global mod scanner:

```text
mods/_oppw4/incoming/Casual Trafalgar Law.zip
```

- incoming zips require a tiny `mod.toml` manifest before they are considered
  for the slot-specific path:

```toml
character = "law"
source = "DLC_COSTUME_006_586_026_003"
```

- `_oppw4/incoming` is intentionally skipped by `archive_assets`, so the
  original/global patcher path will not replace all Law costumes with this zip;
- a new Law custom-slot virtual runtime scans matching incoming zip assets for
  the usual RDB archives (`CharacterEditor`, `MaterialEditor`, etc.) and keeps
  them separate from the normal virtual manager;
- the RDB index external fields are patched for those assets, but external file
  opens route through a slot-aware gate:
  - when the selected/requested Law variant is the allocated custom id (`699` in
    the current run), the custom zip entry is served;
  - otherwise an original RDB-bin range fallback is served for the same file, so
    existing Law slots should remain visually original;
- virtual handles now encode their runtime kind (`global`, `law-slot-custom`,
  `law-slot-original`) to avoid handle id collisions between the managers.

Expected log signals for the next run:

```text
Law custom slot replacements ready: custom=N original_fallbacks=N
Law custom slot runtime published: custom=N original_fallbacks=N
Law custom slot active state active=true reason=selection-lookup variant=699
Open virtual ... runtime=law-slot-custom ... source=...\Casual Trafalgar Law.zip!...
```

Crash follow-up `2026-05-11_20-31-15.log`:

- incoming zip scan worked:

```text
Law custom slot CharacterEditor: files=1 matched=1
Law custom slot MaterialEditor: files=8 matched=8
Law custom slot replacements ready: custom=9 original_fallbacks=9
```

- the crash did **not** happen after serving the custom zip. It happened on the
  normal Law Oni slot (`selected_variant=586 selected_slot=3`) after the shared
  RDB index had been patched and the game opened Law's normal model through the
  fallback runtime:

```text
Open virtual ... runtime=law-slot-original ... file=MPLC026_Law.g1m ...
```

- `crash.log` reports `EXCEPTION_BREAKPOINT` at `OPPW4.exe+0x3D89FC`, about
  11 ms after that fallback open;
- conclusion: patching the shared RDB entry for `MPLC026_Law.g1m`/Law material
  hashes is not slot-specific. It changes the path used by existing Law slots.
  The fallback-original virtual file route is not safe for the game.

Guardrail installed `2026-05-11 20:35`:

- keep scanning incoming slot zips, but do not patch the shared RDB index for
  Law custom-slot assets yet;
- only serve custom virtual files when the custom slot is actually active;
- remove the `law-slot-original` fallback from normal slot loads;
- expected behavior: slot 5 remains stable/playable again, but the custom zip
  will not visually replace it until we create slot-unique asset identifiers.

Next real implementation target:

- generate/clone unique RDB virtual entries for the slot 5 assets instead of
  replacing the existing Law hashes;
- patch the allocated variant/layout metadata for `699` to point at those
  unique asset ids;
- then the existing virtual table technique can be used safely, because only the
  custom slot will reference the new virtual entries.

Runtime patch update `2026-05-11 20:56`:

- first step toward slot-unique resources is installed;
- after cloning the `555 -> 699` variant metadata, the patch now overwrites the
  model/resource field at variant metadata `+0x00` with resource id `738`;
- resource id `738` resolves through entry `32`, section `6` to
  `H_UI_Island_Chapter00`, which has a real `CharacterEditor` RDB entry and is
  not one of Law's normal costume resources;
- the incoming custom model `MPLC026_Law.g1m` is now virtualized under the
  surrogate target name `H_UI_Island_Chapter00.g1m`;
- shared Law material names are intentionally skipped for now, so this build
  tests the clean model-resource route first instead of patching
  `MPLC026_Law.g1m`/Law material hashes globally again.

Expected log signals:

```text
Law custom slot CharacterEditor: files=1 matched=1
Law custom slot MaterialEditor: skipped shared asset names until slot-unique aliases exist
Law extra slot model resource patch target_variant=699 ... target=738 match=true
RDB INDEX EXTERNAL CharacterEditor ...   # only if verbose_io_logs=1
Open virtual ... runtime=law-slot-custom ... file=H_UI_Island_Chapter00.g1m ...
```

If this boots and the slot reaches gameplay, the next step is the material side:
either find/assign slot-unique MaterialEditor resource names or patch the model
resource path far enough that the material lookups are also unique.

Crash result `2026-05-11 20:57`:

- the game crashed at startup before the main menu with
  `EXCEPTION_BREAKPOINT` at `OPPW4.exe+0x3d4709`;
- the last relevant patch signal was
  `Law extra slot model resource patch ... before=272 after=738`;
- there was no `Open virtual ... H_UI_Island_Chapter00.g1m` before the crash,
  so the failure happened before our custom model was actually opened;
- conclusion: hijacking a live resource id such as `738` is unsafe. The next
  clean route is a runtime-only deep copy with private IDs: clone every linked
  metadata/resource row needed by the custom slot, point variant `699` at those
  private rows, and only then virtualize the custom files behind those private
  names.

Recovery build:

- the surrogate model/resource patch is disabled again;
- Law custom slot RDB index patching is disabled again;
- the slot 5 row clone/unlock path remains active, so the stable slot test is
  preserved while the private deep-copy path is designed.

Runtime patch update `2026-05-11 21:20`:

- runtime/disk rule is unchanged: do not patch `LINKDATA_A.BIN` on disk;
- first "deep copy" test uses the dormant Law Souhi resource path instead of a
  random free-looking id:
  - variant/layout `555 -> 699` is still cloned in RAM;
  - the variant model/resource id is left as `272`
    (`MDLC033_Law_Souhi`);
  - incoming `CharacterEditor/MPLC026_Law.g1m` is aliased to
    `CharacterEditor/MDLC033_Law_Souhi.g1m`;
  - incoming base Law cloth/skin textures are aliased to the matching Souhi
    `body_*` and `skin_*` material names;
  - Law custom-slot RDB index patching is enabled again, but it now targets the
    dormant Souhi names instead of the shared `MPLC026_Law` names.

Expected log signals:

```text
Law custom slot CharacterEditor: files=1 matched=1
Law custom slot MaterialEditor: files=8 matched=8
Law extra slot model resource patch disabled ... reason=requires_private_deep_copy
RDB INDEX EXTERNAL CharacterEditor ...
RDB INDEX EXTERNAL MaterialEditor ...
Open virtual ... runtime=law-slot-custom ... file=MDLC033_Law_Souhi.g1m ...
Open virtual ... runtime=law-slot-custom ... file=MPR_Bound_Character_MDLC033LawSouhi_...
```

This is not the final generic private-id allocator yet. It is the safer
intermediate proof: slot 5 should use a Law-compatible dormant official resource
path, while normal Law slots stay on their original shared assets.

Crash follow-up `2026-05-11_21-22-46.log`:

- the slot-specific zip scan worked and matched all staged files:

```text
Law custom slot CharacterEditor: files=1 matched=1
Law custom slot MaterialEditor: files=8 matched=8
```

- `699` now passes the real unlock/admission branch inside the scene object list
  builder:

```text
Costume variant unlock-check category=26 variant=699 slot=4 strict=0 original_result=1 result=1 forced=false
```

- no `selected_variant=699` and no `Open virtual ... MDLC033_Law_Souhi.g1m`
  appeared before the crash;
- in game, the fifth slot appeared as an empty/blank item before the crash;
- working hypothesis: the RDB index points at the dormant Souhi virtual names,
  but the slot-aware virtual file gate still waited for `selected_variant=699`.
  The UI/list path can request the button/preview resources before the selection
  cache switches to `699`, so the custom file gate must allow dormant Souhi
  resources to open before formal selection.

Patch update:

- keep the dormant Souhi target names;
- keep the shared Law assets untouched;
- allow Law custom-slot virtual files to open whenever the requested path
  matches the dormant custom-slot table, even if `law_custom_slot_active()` is
  not true yet;
- expected new signal:

```text
Open virtual ... runtime=law-slot-custom ... file=MDLC033_Law_Souhi.g1m ...
```

If that signal appears and the crash still happens, the next missing piece is
more likely UI/ScreenLayout/KIDS presentation data rather than file routing.

Current handoff:

- the concise restart point for a new chat is now
  `docs/reverse-notes/current-handoff.md`;
- it captures the latest `2026-05-11_21-31-06.log` crash, what is proven to
  work, what failed, and the next clean debug/deep-copy target.

Diagnostic result `2026-05-11_22-05-54.log`:

- the follow-up build disabled only the `CharacterEditor` dormant model alias;
- the log confirmed the model alias was absent:

```text
Law custom slot CharacterEditor: no dormant aliases matched incoming assets
```

- the `MaterialEditor` dormant aliases remained active and were opened through
  the slot runtime:

```text
Law custom slot MaterialEditor: files=8 matched=8
Open virtual ... runtime=law-slot-custom file=MPR_Bound_Character_MDLC033LawSouhi_...
```

- user-visible result:
  - Souhi used the official model shape rather than the custom base Law model;
  - custom textures still applied;
  - the official/base Souhi slot was also affected.

Conclusion:

- dormant Souhi material aliases are also shared RDB entries, so they are not
  slot-local;
- the whole dormant-alias experiment proves routing, but it cannot be the final
  custom-slot method;
- both dormant alias families must stay disabled while implementing true private
  resource ids/names.

Cleanup build installed `2026-05-11 22:10`:

- `LAW_CUSTOM_SLOT_DORMANT_MODEL_ALIAS_ENABLED = false`;
- `LAW_CUSTOM_SLOT_DORMANT_MATERIAL_ALIAS_ENABLED = false`;
- expected next log:

```text
Law custom slot CharacterEditor: no dormant aliases matched incoming assets
Law custom slot MaterialEditor: no dormant aliases matched incoming assets
Law custom slot replacements ready: custom=0 original_fallbacks=0
```

- backup:

```text
D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-disable-all-dormant-aliases.20260511-221038.dll
```

Next implementation target remains runtime-only private resource deep copy:
allocate private names/ids for the custom slot, patch variant `699` to those
private resources, and serve the zip only behind those private entries.

Gated shared-base diagnostic installed `2026-05-11 22:27`:

- stages incoming Law zip files against base Law names rather than dormant Souhi
  names;
- patches cloned variant `699` model resource to base Law id `26`;
- disables always-active custom slot routing;
- routes inactive matching paths through `law-slot-original` fallback, so shared
  RDB external-flag patches should not visually modify official slots;
- keeps this as a diagnostic only, not the final private-resource allocator.

Verification before install:

```text
cargo test -p oppw4-dinput8-proxy: 51 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
installed SHA-256 8C49CE7455736DC5F4497D0CD42EC99545AA73DF43A4094767894A374F985FDD
```

Backup:

```text
D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-shared-base-gated-slot.20260511-222710.dll
```

Expected result:

- official Law/base and Souhi stay normal;
- slot 5 loads the custom zip using base Law model/material resource ids;
- logs should show custom runtime opens only after variant `699` becomes active,
  and original fallback opens for inactive shared base entries.

Crash result for that diagnostic:

- selected/current menu slot was Oni (`586`), which is normal because the user
  spawns on that slot before moving;
- the scene builder still evaluated slot `699` and let it pass;
- then the shared Law base model was opened through `law-slot-original`;
- the game hit `EXCEPTION_BREAKPOINT` at `OPPW4.exe+0x3D89FC`;
- conclusion: the shared RDB virtual/fallback route is unsafe again.

Important correction:

- model/resource id `26` and character/category id `26` are different concepts;
- here, resource id `26` also names `MPLC026_Law`, so the variant patch to `26`
  is a model-resource experiment, not a character patch;
- isolate that experiment by disabling shared RDB asset staging.

Isolation build installed `2026-05-11 22:43`:

- `LAW_CUSTOM_SLOT_SHARED_SOURCE_ASSETS_ENABLED = false`;
- keep `LAW_EXTRA_SLOT_CUSTOM_MODEL_RESOURCE_ID = 26`;
- keep `LAW_EXTRA_SLOT_CUSTOM_MODEL_RESOURCE_PATCH_ENABLED = true`;
- expected log:

```text
Law custom slot CharacterEditor: no slot asset plan matched incoming assets
Law custom slot MaterialEditor: no slot asset plan matched incoming assets
Law custom slot replacements ready: custom=0 original_fallbacks=0
Law extra slot model resource patch target_variant=699 ... target=26 match=true
```

- there should be no `runtime=law-slot-original` open for `MPLC026_Law.g1m`.

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 51 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
installed SHA-256 7186740941694CF18672FBBAF162FDD706765280871DCCA99758676EDD116D7F
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-model-resource-only-slot-test.20260511-224306.dll
```

User decision note `2026-05-11`:

The clean solution is not another shared alias. It is to create a new private
Law model/resource entry and attach the slot-specific model/material assets to
that entry.

The current visible result where slot 5 uses the base Law model is intentional
for the diagnostic build:

- variant `699` is the injected fifth costume slot;
- model/resource id `26` is the base Law model row, `MPLC026_Law`;
- this is different from character/category id `26`, even though both numbers
  are `26` for Law;
- the custom model/texture zip is not expected to apply in this isolation build
  because shared RDB asset staging is disabled.

Target shape for the real fix:

1. Clone base Law model metadata from entry `35`, row `26`, into a private
   resource id owned by Law.
2. Create or emulate a matching private name in entry `32`, section `6`, for
   example a short generated name like `MPLC026LawX`.
3. Patch variant `699` to point at that private model/resource id.
4. Create private RDB index entries for the model and any material/texture names
   needed by that private model.
5. Serve the custom zip files only behind those private names.
6. Leave official Law base, Souhi, and Oni RDB hashes untouched.

Reasoning:

- Reusing `MPLC026_Law.g1m` changes a shared base Law hash.
- Reusing `MDLC033_Law_Souhi.g1m` changes a shared Souhi hash.
- Hijacking a random live id such as `738` is unsafe because the rest of the
  game's metadata still treats it as its original resource.
- A private Law model entry keeps the new slot local and makes texture/material
  replacement belong only to slot 5.

Open implementation problem:

- the existing virtualizer can patch/replace RDB entries that already exist;
- a private model entry needs the game to see a new RDB name/hash too;
- therefore the next work is either RAM cloning of the loaded RDB archive entry
  table or a virtual `.rdb` overlay that appends private `IDRK` blocks at read
  time.

Private row `292` RAM diagnostic installed `2026-05-11 23:13`:

- the loader reads clean `LINKDATA_A.BIN` only to prepare expected bytes;
- the DLL scans writable process memory for inflated entry `35`;
- row `26` (`MPLC026_Law`) is cloned into row `292`;
- entry `32`, section `6`, row `292` is patched in RAM to `MPLC026_Law` with
  nul padding;
- variant `699` model/resource id is patched to `292`;
- shared custom RDB asset staging remains disabled, so this is not expected to
  load the zip yet.

Expected result:

- slot 5 still displays base Law;
- logs prove whether the game accepts a cloned private model/resource row;
- if stable, the next test is to make row `292` use a private/hash-style name
  and add a private RDB index entry instead of using the shared
  `MPLC026_Law.g1m` hash.

Private row `292` first test result `2026-05-11 23:15`:

```text
Law private model RAM plan ready source_row=26 target_row=292 target_name=MPLC026_Law ...
Law private model RAM plan published source_row=26 target_row=292 ...
Law private model row clone skipped source_row=26 target_row=292 reason=entry35_base_not_found
```

The slot disappeared because the diagnostic returned immediately when entry
`35` was not found in writable RAM. That skipped the later slot-array/count
patches, so Law never received slot index `4` for that run.

Fallback build installed `2026-05-11 23:21`:

- entry `35` clone failure is now non-fatal;
- if row `292` cannot be cloned, variant `699` falls back to model/resource id
  `26`;
- the slot injection continues, so slot 5 should reappear with base Law;
- private RDB/model assets are still not expected to apply yet.

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 54 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
installed SHA-256 FEE80252A7E22AD48AD1423D1D7129BC5F838463B7B2174CAC247F4F951CE134
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-private-model-row292-fallback.20260511-232122.dll
```

Expected fallback logs:

```text
Law private model row clone skipped ... reason=entry35_base_not_found
Law private model RAM clone unavailable target_variant=699 fallback_model_resource=26
Law extra slot model resource patch target_variant=699 ... target=26 match=true
Law extra slot variant patch layout=26 slot=4 before=65535 after=699
```

Next research branch:

- if the fallback build restores slot 5, stop treating raw entry `35` scan as
  guaranteed;
- either locate the parsed model registry/table in RAM from runtime pointers,
  or use focused Ghidra on the LinkData/model registry loader;
- only after row `292` can be made real should the private RDB overlay/hash work
  resume.

Private model manager alias probe installed `2026-05-11 23:35`:

Ghidra target export:

```text
C:\Users\Osef\Documents\Codex\2026-05-09\si-je-te-demanderais-avec-du\oppw4-ghidra\game_resource_manager_targets.txt
```

Key Ghidra result:

- model resource manager global: `DAT_141eba7a0`, RVA `0x1eba7a0`;
- `FUN_14016ce30(manager, id)` returns loaded resource pointer when state is
  `2`;
- `FUN_14016ceb0(manager, id)` checks loaded state `2`;
- `FUN_14016cf30(manager, id)` checks unloaded state `0`;
- `FUN_14005ad10(manager, id)` checks busy/loading state `1`;
- `FUN_14016dc20(manager, id, task, ...)` enqueues a load and writes state `1`;
- resource state/object slots are indexed as `manager + id * 0x20`.

New runtime probe:

- keep row `292` RAM clone attempt for evidence;
- if row clone fails, variant `699` still gets model/resource id `292`;
- model manager calls with requested id `292` are aliased to source id `26`;
- alias is guarded by the manager pointer equaling `DAT_141eba7a0`;
- enqueue-load also temporarily changes the stack task id field from `292` to
  `26` while calling the original function.

Expected logs:

```text
Law private model manager alias hooks installed=5 active=true target_resource=292 source_resource=26
Law private model row clone skipped ... reason=entry35_base_not_found
Law private model RAM clone unavailable target_variant=699 manager_alias_resource=292 alias_source_resource=26
Law extra slot model resource patch target_variant=699 ... target=292 match=true
Law private model manager alias action=... requested=292 mapped=26 ...
```

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 55 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
installed SHA-256 DDE9271D62E4F0854E8D76E62F8DC94CB9083C9C002625D077BEF119DE7EE6CE
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-private-model-manager-alias-fmt.20260511-233712.dll
```

Interpretation target:

- stable slot 5 with these logs means private id `292` is viable in variant
  metadata, and entry `35` raw cloning can stop being the only route;
- the next step would be making id `292` produce a real private model resource,
  either by cloning the manager's resource object/table entry or by appending a
  private RDB entry and making the load task resolve that private name;
- if this crashes, inspect `FUN_14016dc20` task cloning and `FUN_14016ce30`
  returned resource pointer for requested id `292`.

Private model manager alias test result `2026-05-11 23:39`:

- slot 5 remained visible and the game did not crash;
- official Law base, Souhi, and Oni slots remained valid;
- slot 5 rendered as an empty/invisible character.

Key evidence:

```text
Law extra slot model resource patch target_variant=699 ... target=292 match=true
Law private model manager alias action=enqueue-load ... requested=292 mapped=26 task_patched=true
Law private model manager alias action=loaded-check ... requested=292 mapped=26 result=1
Law private model manager alias action=get ... requested=292 mapped=26 result=0x...
```

Meaning:

- variant `699 -> model/resource 292` is accepted by the game;
- the alias `292 -> 26` works at the model resource manager level;
- the empty character is likely caused by missing/incompatible material or
  color-variation fields in the cloned variant metadata, not by the slot or
  model-manager id itself.

Material variation diagnostic installed `2026-05-11 23:47`:

- keep variant `699` cloned from hidden Law variant `555`;
- keep model/resource id `292`;
- copy the 8 bytes at variant metadata offset `0x0c` from base Law variant `57`
  into target variant `699`;
- log `before`, `source_data`, and `after` for the material patch.

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 56 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
installed SHA-256 B52943064623A1599E1987C639B1813BD7EA5716C73612C53DB9F41189A260C8
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-color-variation-patch.20260511-234754.dll
```

Material variation diagnostic result `2026-05-11 23:50`:

- slot 5 was still empty;
- log confirmed the patch was applied:

```text
Law extra slot color variation patch source_variant=57 target_variant=699 ... before=ffffffffffff6800 source_data=ffffffffffff0000 after=ffffffffffff0000 match=true
```

This shows base Law variant `57` does not provide concrete color variation ids
in the first three fields either. The empty model is more likely caused by the
private model id `292` not being represented in the model manager's internal
entry table.

Model manager slot mirror diagnostic installed `2026-05-11 23:58`:

- model/resource field remains `292`;
- source model load still maps to known-good resource `26`;
- the internal model-manager entry at `manager + 0x28 + 26 * 0x20` is copied to
  `manager + 0x28 + 292 * 0x20`;
- when that mirror succeeds, model manager get/check hooks call the original
  function with requested id `292`, so later code sees a populated private id
  instead of only receiving an aliased `26` pointer.

Expected logs:

```text
Law private model manager slot mirror source_id=26 target_id=292 ... match=true
Law private model manager alias action=get ... requested=292 mapped=26 call=292 result=0x...
```

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 57 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
installed SHA-256 6A51CE305F6CCA07B0C3357F3957261E91BDC5782C2001C42995A0BE7F8B9983
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-model-manager-slot-mirror.20260511-235837.dll
```

Model manager slot mirror result `2026-05-12 00:00`:

- slot 5 was still empty;
- the internal manager mirror did work:

```text
Law private model manager slot mirror source_id=26 target_id=292 ... match=true ... after_state=2
Law private model manager alias action=get ... requested=292 mapped=26 call=292 result=0x...
```

So the empty character is now downstream of model resource loading: id `292` is
populated in the manager and returns a real pointer.

Render attach trace installed `2026-05-12 00:07`:

- exported `FUN_1403ce790`, `FUN_14050f720`, and `FUN_14016a3d0` with Ghidra;
- hooked `FUN_1403ce790` at `game+0x3ce790`;
- log starts only after the first successful `get` for requested model id
  `292`, to capture the calls that should attach the loaded model to the
  preview/render object.

Expected logs:

```text
model render attach trace hook installed target=game+0x3ce790 ...
Law private model render attach object=0x... model_object=0x... result=0x... frames=...
```

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 58 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
installed SHA-256 7E1723DAC6AE51360BB2AF436A9559D7B82ED718DAF894BA12CD587C65723F9B
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-render-attach-trace.20260512-000754.dll
```

Render attach trace result `2026-05-12 00:11`:

- user still reported the slot as invisible;
- log confirms slot 5/variant `699` still exists and model id `292` still
  loads through the mirrored manager entry;
- however this run did not capture a formal slot-5 selection:

```text
selected_variant=586 selected_slot=3
```

- there was no `selected_variant=699`, no `selected_slot=4`, no
  `Law custom slot active state active=true`, and no private render attach log.

Interpretation:

- `292` loading during slot enumeration is proven;
- the scene/preview state remained on Oni in this capture;
- the blank result may be happening in the menu/preview layer before the
  selection cache switches to `699`, or the test did not actually include the
  moment of moving onto slot 5.

Global/private render attach probe installed `2026-05-12 00:20`:

- `FUN_1403ce790` now logs the first global attach calls even before `292` is
  seen:

```text
Model render attach trace scope=global ...
```

- the hook now remembers the private model pointer from either the `get` result
  or the mirrored manager entry;
- later attach calls after `292` is known log as:

```text
Model render attach trace scope=private ...
```

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 60 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
installed SHA-256 998E68F5197AAF15C2E830E33990BC9FF6EA22282FE2A1975AC9E4B92DFBA182
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-render-global-trace.20260512-002018.dll
```

Global/private render attach probe result `2026-05-12 08:07`:

- user reached the custom slot;
- model manager entry `292` was mirrored from `26` and returned a live pointer;
- selection lookup returned custom variant `699`;
- scene traces were exhausted before the custom-slot activation, so the last
  captured scene still showed Oni:

```text
selected_variant=586 selected_slot=3
Law custom slot active state active=true reason=selection-lookup variant=699
```

- there were no `Model render attach trace scope=global` or `scope=private`
  lines in the run.

Interpretation:

- the slot injection and selection lookup path now reach `699`;
- the invisible slot is not explained yet, because the scene/preview state was
  not captured after activation;
- the render-attach hook at `game+0x3ce790` may be the wrong path for this menu
  preview, or the preview never reaches render attach for the custom slot.

Scene-important trace diagnostic installed `2026-05-12 08:19`:

- dynamically allocated variant ids, currently `699`, are treated as Law values
  by menu/scene trace filters;
- scene state logs and scene list probes now have a reserved important budget
  after the regular cap is exhausted when the custom slot is active;
- reserved lines include `scope=important`.

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 62 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
installed SHA-256 8F559E9B903A73B55B7D56D5AC8432992C477F0A79DA82D99ED4A111085CBC4A
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-scene-important-trace.20260512-081938.dll
```

Scene-important trace result `2026-05-12 08:21`:

- the scene really switches to custom variant `699` and slot `4`;
- the selected preview object also carries `object_variant=699` and
  `object_slot=4`;
- model/resource id `292` was loaded earlier in the same run;
- no `Model render attach trace scope=global/private` lines appeared.

Key transition:

```text
Law custom slot active state active=true reason=selection-lookup variant=699
Costume scene scene-apply scope=important ... before=[... selected_variant=586 selected_slot=3 ... object_variant=699 object_slot=4] after=[... selected_variant=699 selected_slot=4 ... object_variant=699 object_slot=4]
Costume scene scene-update-dispatcher scope=important ... selected_variant=699 selected_slot=4 ... object_variant=699 object_slot=4
```

Focused Ghidra reading:

- `FUN_1414906a0` and `FUN_1414926a0` call `FUN_14148b5f0`;
- for variants below `0x2c0`, `FUN_1414906a0` passes the selected variant id
  directly instead of going through the resolver;
- therefore the slot 5 preview sends `preview_variant=699` to
  `FUN_14148b5f0`;
- `FUN_14148b5f0` chooses either the visible preview branch or hide/default
  branch, then tail-calls `FUN_14148bcc0`.

Preview model-update diagnostic installed `2026-05-12 08:47`:

- hooks `FUN_14148b5f0` at `game+0x148b5f0`;
- expected line prefix:

```text
Costume preview model-update scope=custom ...
```

- logs `preview_variant`, `visible`, branch label, layout, fallback, and
  before/after widget fields:
  `child58`, `mapped294`, `current_layout2dc`, `visible2a0`, `active2a1`,
  `child_b4`, and child pointers.

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 64 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
installed SHA-256 CDB8CA41C4F9416B63BB115591F08625A6A2CB35EDDE9CC1AB94A88DFF274789
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-preview-model-update-trace.20260512-084725.dll
```

Preview model-update trace result `2026-05-12_17-52-47.log`:

- `FUN_14148b5f0` hook installed correctly;
- private model/resource id `292` still loads and mirrors from base Law resource
  id `26`;
- scene and selected preview object both reach `699/slot 4`;
- custom preview update call:

```text
Costume preview model-update scope=custom ... preview_variant=699 visible=0 branch=conditional-or-hide layout=26 fallback=0 ...
```

- widget fields stayed effectively unchanged for `699`:

```text
mapped294=643 current_layout2dc=421 visible2a0=0x00 active2a1=0x00
```

- scene-list flags for the selected Law entry were all zero:

```text
flags=0000000000000000000000000000 selected_layout=26 selected_variant=699 selected_slot=4
```

Interpretation:

- slot injection, selected object state, and model manager resource loading are
  not the current blockers;
- the remaining blank preview is now isolated to the preview model widget path;
- `FUN_14148b5f0` receives `visible=0` for `699` and leaves the widget hidden or
  unchanged.

Preview force-visible diagnostic installed `2026-05-12 18:04`:

- added `law_custom_preview_model_visible_arg`;
- for only the current custom Law variant on layout `26`, currently
  `preview_variant=699`, if the game passes `visible=0`, the hook calls the
  original helper with `effective_visible=1`;
- logs now include `visible`, `effective_visible`, and `forced_visible`.

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 65 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
installed SHA-256 C98994A5927C4D5311F7C3DEB411FA4933628765ACD102DB74A2B6BCA1565312
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-preview-force-visible.20260512-180433.dll
```

Next interpretation:

- if the slot becomes visible, the proper fix is not to keep the forced hook
  forever; instead find the menu/scene eligibility flag that made
  `uVar21/uVar9 = 0` for the custom slot;
- if it stays invisible even with `forced_visible=true`, inspect
  `FUN_14148bcc0`, `FUN_1416112e0`, and `FUN_141613030`, because the tail may
  still map/update the widget to a non-rendering state.

Follow-up result from `2026-05-12_18-08-55.log`: the force-visible diagnostic
did fire for slot 5 (`preview_variant=699`, `visible=0`,
`effective_visible=1`, `forced_visible=true`), and the widget flags changed to
visible/active, but the idle costume preview still had no model. After selecting
the slot, the user saw a black layout, then launching a game crashed instead of
returning to the main menu. Conclusion: the forced-visible patch only enables an
empty/incomplete preview container and makes the downstream state less safe. It
was disabled in the next installed build.

Installed safe-backoff build `2026-05-12 18:21`:

```text
cargo test -p oppw4-dinput8-proxy: 65 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
installed SHA-256 A04417BA676ECFDEB7EECD5EBABD4D38721DFC8DACAF7F6C67918592DEC21B82
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-disable-preview-force.20260512-182121.dll
```

Next target: if the crash persists without the forced-visible diagnostic,
inspect/hook the launch/prebattle function that crashed at
`OPPW4.exe+0x1252DFB` (`FUN_141252cc0`). If the crash disappears but preview is
still invisible, continue from the preview widget tail, but without forcing
`visible=1` blindly.

Focused Ghidra follow-up: `OPPW4.exe+0x1252DFB` is inside `FUN_141252cc0` on
the failure path after global-list validation of state fields at byte offsets
`0x1d4`, `0x1d0`, then optionally `0x1d8`. The exact crash-site instruction is
`MOV dword ptr [0x00000220],0x1`; the bad state probably enters before that,
not at the instruction itself.

Diagnostic installed `2026-05-12 18:35`: hook `FUN_141252cc0` without mutating
state and log:

```text
Launch costume state enter ... id1d0=... id1d4=... id1d8=... flags20=...
```

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 66 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
installed SHA-256 822847F703584AD18F3904EE662D92AB5212924ABC030D91941F849B541A19AF
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-launch-state-hook.20260512-183510.dll
```

Next log interpretation: if `id1d0` or `id1d4` is `699`, the custom variant is
being sent into a launch/prebattle table that does not know it yet. If those
fields are official ids but the slot is still invisible, the preview widget path
remains the active blocker.

Follow-up from `2026-05-12_18-41-52.log`: the launch/prebattle hook installed,
but the run did not emit `Launch costume state enter`, and `crash.log` did not
show a new top-level exception. The useful evidence is back in the preview path:
slot 5 reaches `selected_variant=699` / `object_variant=699`, model id `292`
still loads through the manager alias, but the active scene-list entry has all
visibility flags zero. Focused Ghidra shows `FUN_1414926a0` reads
`entry + 0x3c + list_index` directly when `list_index < 14`; in the log
`list_index=0`, so flag slot `0` is the byte preventing the normal preview prep
path.

Diagnostic installed `2026-05-12 19:02`: before calling the original preview
refresh hook, patch only the current custom Law scene-list flag byte to `1`.
The old late force-visible hook stays disabled. Search the next log for:

```text
Law custom scene-list flag diagnostic
```

Expected result is `patched=true before=0x00 after=0x01`, followed by a normal
`Costume preview model-update` for `preview_variant=699` with `visible=1` and
`forced_visible=false`.

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 69 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
installed SHA-256 F1BC115EBF77CC70FC69CB6A7015AC1281B2112AD908E0126E989611F60B77E3
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-scene-list-flag-diagnostic.20260512-190237.dll
```

Result from `2026-05-12_19-08-24.log`: the scene-list flag diagnostic did patch
the expected byte (`before=0x00 after=0x01 patched=true`), but the original
preview path still called the model-update helper with `preview_variant=699`
and `visible=0`. The flag did not persist into later scene-list probes either,
so preview invisibility is not solved by this single byte.

The in-game crash is now captured at launch time. `crash.log` again points to
`OPPW4.exe+0x1252DFB`, and the launch hook logged:

```text
Launch costume state enter ... flags20=0x00001043 id1d0=292 id1d4=22 id1d8=65535 id1dc=0 id1e4=4
```

Diagnostic installed `2026-05-12 19:21`: before `FUN_141252cc0`, if slot `4`
is active and launch-state `id1d0` equals private model id `292`, write source
model id `26` to `state + 0x1d0`. This is a targeted crash test, not final
private gameplay support. Search next log for:

```text
Launch costume private-model alias diagnostic
```

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 70 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
installed SHA-256 C938E4E65380C2FFD9490B0177A69636FA7221D35EFB25BD62C44EC2BE7766E5
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-launch-private-model-alias.20260512-192129.dll
```

Result from `2026-05-12_19-24-07.log`: the launch alias fired and the previous
top-level crash did not recur, but preview stayed invisible because no custom
runtime assets were published. The key startup lines were:

```text
Law custom slot CharacterEditor: no slot asset plan matched incoming assets
Law custom slot MaterialEditor: no slot asset plan matched incoming assets
Law custom slot replacements ready: custom=0 original_fallbacks=0
```

The incoming zip itself was valid: it matched `character = "law"` and
`source = "DLC_COSTUME_006_586_026_003"`, and contained `MPLC026_Law.g1m` plus
the 8 expected base Law texture files. The problem was that the diagnostic
build still had the dormant alias plan disabled for both model and materials.

Installed build `2026-05-12 19:33` re-enables only that asset plan:

```text
LAW_CUSTOM_SLOT_DORMANT_MODEL_ALIAS_ENABLED = true
LAW_CUSTOM_SLOT_DORMANT_MATERIAL_ALIAS_ENABLED = true
LAW_CUSTOM_SLOT_SHARED_SOURCE_ASSETS_ENABLED = false
LAW_CUSTOM_SLOT_DORMANT_ASSETS_ALWAYS_ACTIVE = false
```

The zip assets should now be remapped to the dormant Souhi RDB names for the
slot runtime while preserving original fallbacks for official slots. Expected
next log: `Law custom slot replacements ready` should no longer show
`custom=0`; if all names match, expect about `custom=9 original_fallbacks=9`.

Verification:

```text
cargo test -p oppw4-dinput8-proxy law_custom_slot -- --nocapture: 6 passed
cargo test -p oppw4-dinput8-proxy: 70 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
installed SHA-256 08270B64EBF1A1556544E149ADDDA5D655D1D81B6E5E5B521FC6F9F504FDD4D6
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-law-slot-assets-alias.20260512-193356.dll
```

Result from `2026-05-12_19-38-01.log`: the build did ingest the zip assets
correctly, but the game crashed immediately when entering the menu and the
slot 5 sprite/image was gone. This should not be described as "official slots
crash"; the better working diagnosis is that slot 5 has an invalid or missing
UI/preview resource and hits an undefined resource path before the player can
even test the model in-game.

Useful evidence:

```text
Law custom slot CharacterEditor: files=1 matched=1 hash_missing=0 unresolved=0
Law custom slot MaterialEditor: files=8 matched=8 hash_missing=0 unresolved=0
Law custom slot replacements ready: custom=9 original_fallbacks=9
Law custom slot runtime published: custom=9 original_fallbacks=9
```

So the zip plan was valid. The suspicious part is that the log still did not
show any `DLC_COSTUME_006_699_026_004.bin` request for slot 5; it only showed
the official Oni image path requests such as `DLC_COSTUME_006_586_026_003.bin`.
That matches the user-visible symptom: no slot 5 image/sprite, then a menu
crash. The crash is likely around slot 5 UI/preview/resource preparation, not
around Law base/Souhi/Oni themselves.

Focused Ghidra export for the crash address:

```text
OPPW4.exe+0x3D89FC -> FUN_1403d8560
1403d89b1 TEST RBX,RBX
1403d89b4 JNZ 0x1403d8a01
1403d89fc INT3
```

Interpretation: this is a generic resource construction/load path. The `INT3`
happens after the produced resource pointer remains null after fallback
attempts. It does not yet prove whether the failed resource is the slot image,
the preview model, a texture, or another linked UI file, but the missing
`699` DLC image request makes the slot 5 image path the next best target.

Safety rollback installed after this test:

```text
LAW_CUSTOM_SLOT_DORMANT_MODEL_ALIAS_ENABLED = false
LAW_CUSTOM_SLOT_DORMANT_MATERIAL_ALIAS_ENABLED = false
installed SHA-256 457C1E7F11BAC664149DB7B06AA6CAD3ACC4DB2609EA4B7338754BAAFFED816B
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-disable-shared-dormant-alias.20260512-195336.dll
```

Expected behavior for the rollback build: slot 5 should return to the safer
baseline. The custom model/textures are not expected to load in this build.

Next diagnostic route: do not blindly re-enable the shared dormant alias.
Either hook/log the parameters around `FUN_1403d8560` so the failing resource
hash/name is visible, or trace the slot 5 UI/DLC image path and find why the
game never asks for the expected `699` costume image resource.

Result from `2026-05-12_20-12-44.log`: the rollback did exactly what it was
supposed to do. The slot 5 image/sprite appears again, and the menu does not
crash. The model/texture preview is still invisible when hovering slot 5.

Startup confirms the safe build had custom assets disabled:

```text
Law custom slot CharacterEditor: no slot asset plan matched incoming assets
Law custom slot MaterialEditor: no slot asset plan matched incoming assets
Law custom slot replacements ready: custom=0 original_fallbacks=0
Law custom slot runtime published: custom=0 original_fallbacks=0
```

So this test should not be treated as a model/texture load failure. It is a
preview-state diagnostic. The important line is still:

```text
Costume preview model-update ... preview_variant=699 visible=0 effective_visible=0
```

Focused Ghidra on `FUN_1414926a0` explains why the previous scene-list flag
diagnostic was ineffective: the game calls `FUN_141493820` to rebuild the list,
then reads `entry + 0x3c + list_index` to choose the `visible` argument. The
old patch happened before the rebuild and was overwritten back to `0`.

Installed diagnostic `2026-05-12 20:25`:

```text
hook FUN_141493820 at game+0x1493820
patch current Law slot 5 scene-list flag after the original rebuild returns
keep dormant model/material aliases disabled
installed SHA-256 C84B77842B42BF5CBAE3178EC4C5D2EAAB89A7B7882038CF1FD0A0D4F5DFD96E
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-scene-list-rebuild-post-flag.20260512-202552.dll
```

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 71 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

### 2026-05-13 19:30 preview list builder boundary

`2026-05-13_19-21-07.log` narrowed the preview-invisible issue again:

- The custom slot is selected (`selected_variant=699`, `selected_slot=4`).
- The selected object is also correct (`object_layout=26`, `object_variant=699`,
  `object_slot=4`).
- The model-manager alias for private model id `292` resolves to base Law id
  `26` and returns a loaded pointer.
- The active Law preview-list entry still has zero display flags:

```text
active_layouts=26,49,50,45,14,11,27,15,10,13,0,0,0,0
active_flags=0000000000000000000000000000
flag_gate=flag-zero
```

The important change: this zero-flag state is already present around
`FUN_141493820`, so the next diagnostic moves one layer earlier into
`FUN_141493220`, the function that builds/sorts/copies the costume preview list.

Installed read-only hook:

```text
game+0x1493220 FUN_141493220(list_base, param2, param3, param4)
stolen_len=20
```

The next log should contain `Costume scene list-build detail`, including source
vtable pointers and all eight generated bank entries. If bank 2 is built with
zero flags directly, investigate the builder's availability calls, especially
`FUN_1412f92c0(layout)`.

Installed:

```text
SHA-256 E193DED42D115764C66B3F34DE47AF0211958CF3444F979916BC2D51B8AF09C5
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-scene-list-build-detail.20260513-193024.dll
```

### 2026-05-13 19:37 scene-list flag patch re-enabled

The `2026-05-13_19-32-36.log` builder trace proved the Law preview bank is
built with zero flags directly:

```text
selected_variant=699 selected_slot=4
bank 2 layouts=26,49,50,45,14,11,27,15,10,13,0,0,0,0
bank 2 flags=0000000000000000000000000000
flag_gate=flag-zero
```

So the earlier scene-list flag patch was moved from "guess" to confirmed
minimal diagnostic: the game's own builder refuses to mark Law's preview entry
as displayable in this context.

Re-enabled:

```text
LAW_EXTRA_SLOT_SCENE_LIST_FLAG_DIAGNOSTIC_ENABLED = true
```

Still disabled:

```text
LAW_EXTRA_SLOT_FORCE_SCENE_AVAILABLE_DIAGNOSTIC_ENABLED = false
LAW_EXTRA_SLOT_CLEAR_SCENE_LOCKED_DIAGNOSTIC_ENABLED = false
LAW_EXTRA_SLOT_MODEL_READY_OVERRIDE_ENABLED = false
LAW_EXTRA_SLOT_LATE_MODEL_READY_OVERRIDE_ENABLED = false
LAW_EXTRA_SLOT_MODEL_READY_FLAG_PATCH_ENABLED = false
```

Expected next proof:

```text
Law custom scene-list flag diagnostic ... patched=true
Costume preview model-update scope=custom preview_variant=699 visible=1
```

Installed:

```text
SHA-256 9F477FBAF4367140B2679AD1F2B44C93B1A1B8EF364AECFC5DFBDD2352ABE2EF
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-scene-list-flag-patch.20260513-193710.dll
```

Expected next log:

```text
Law custom scene-list flag diagnostic phase=scene-list-rebuild-leave ... patched=true
Costume preview model-update ... preview_variant=699 visible=1
```

If the first line appears but `visible` remains `0`, inspect the
`FUN_141492e20` condition. If `visible` becomes `1` but the model is still
blank, move downstream to the preview widget/model attach path.

### 2026-05-12 20:30 log: "conditions de deblocage" is a second scene gate

Player result: the slot 5 image appeared and there was no menu crash, but
selecting/hovering slot 5 showed `conditions de deblocage` and the preview
model was still empty.

Important lines:

```text
Law custom scene-list flag diagnostic phase=scene-list-rebuild-leave ... patched=true
Costume preview model-update ... preview_variant=699 visible=1 effective_visible=1 ...
Costume variant unlock-check category=26 variant=699 slot=4 strict=0 original_result=1 result=1 forced=false
Costume scene scene-available-check ... result=0 ... selected_variant=699 selected_slot=4 ... object_variant=699 object_slot=4
```

Conclusion: the classic variant unlock path is not the blocker anymore. The
message is consistent with `FUN_141490320` (`scene-available-check`) returning
0 after the scene-list visibility issue was fixed.

Installed diagnostic `2026-05-12 20:42`:

```text
hook remains FUN_141490320 at game+0x1490320
return effective_result=1 only when original result=0 and the trace is Law layout 26 / variant 699
keep custom model/material RDB ingestion disabled
installed SHA-256 AC7E8B85D447EE4CA088E4A4B21F5255FDBE8F916C0F1FAA2FBA41F7ED3B1DCB
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-scene-available-force.20260512-204216.dll
```

New expected diagnostic line:

```text
Costume scene scene-available-check ... result=0 effective_result=1 forced=true ...
```

If the unlock-condition text disappears, then `FUN_141490320` is confirmed as
the current gate and the next target is the downstream model attach/resource
path. If the text remains, another UI lock flag is being read separately from
the return value.

### 2026-05-12 20:46 log: scene-available force is not the final UI lock

Player result: the slot 5 preview model is still transparent/invisible, and the
character-selection screen still shows `conditions de deblocage`.

Important lines:

```text
Costume scene scene-available-check ... result=0 effective_result=1 forced=true ... selected_variant=699 selected_slot=4 ... object_variant=699 object_slot=4
Costume scene scene-apply ... after=[layout=26 ... selected_variant=699 selected_slot=4 ... object_variant=699 object_slot=4]
Costume preview model-update ... preview_variant=699 visible=1 effective_visible=1 ... after=[... mapped294=643 ... child48=none]
```

Conclusion: forcing `FUN_141490320` proves that this specific return value is
not enough to remove the later character-screen text. The scene really applies
variant `699`, so the next useful evidence is lower in the preview path.

Ghidra split for `FUN_14148b5f0`:

```text
visible branch: FUN_14148af40(widget, layout_id)
tail update:    FUN_14148bcc0(widget, preview_variant, visible)
```

This matters because the visible branch receives layout `26`, while the tail
receives variant `699`. If `FUN_14148af40` prepares a Law/base object and
`FUN_14148bcc0` only changes the mapped preview id, the widget can remain
empty even though `preview_variant=699 visible=1` is logged.

Installed diagnostic `2026-05-12 21:10`:

```text
trace FUN_14148af40 at game+0x148af40 (visible branch)
trace FUN_14148bcc0 at game+0x148bcc0 (tail update)
use a dedicated trampoline for FUN_14148af40 because the first stolen bytes contain a RIP-relative cookie load
keep custom model/material RDB ingestion disabled
installed SHA-256 3D7BE95E9CEB3C881158BF103AD4859845A163FCBB038576CFB64D95AC3A834C
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-preview-branch-trace.20260512-211038.dll
```

Expected next log:

```text
Costume preview branch label=visible-branch ... layout=26 ... before=[...] after=[...] before_res=[...] after_res=[...]
Costume preview branch label=tail-update ... preview_variant=699 visible=1 ... before=[...] after=[...] before_res=[...] after_res=[...]
```

Compare `child48`, `mapped294`, and `res1d0_*` across those two lines and
against any official Law slot line. If visible branch does not create/attach
the expected object, the fix is likely in the setup branch. If it creates one
and tail update breaks it, target `FUN_14148bcc0` / `FUN_141613030`.

### 2026-05-12 21:18 log: slot 5 applies, but preview mapping stays base Law

Correction from the user: `selected_variant=586` before applying slot 5 is not
an error. Oni is the default selected skin at menu entry. The meaningful target
state is `object_variant=699`, then `scene-apply` ending with
`selected_variant=699`.

Important sequence:

```text
Costume resolver layout=26 row=70 mode=2 ... result=57
Costume preview model-update ... preview_variant=57 ... mapped294=643
Costume preview model-update ... preview_variant=699 ... mapped294=643
Costume scene scene-apply ... after=[... selected_variant=699 selected_slot=4 ...]
```

So the slot is accepted/applied, but the preview mapper keeps resolving the
custom variant like base Law. `FUN_1416112e0` reads a 16-bit value from
`variant_metadata + 0x06` and returns `value + 0x269` when the linked preview
resource is valid. For base Law, value `26` gives `mapped294=643`. Oni logs
show `mapped294=911`, so the Oni value should be `294`.

Installed diagnostic `2026-05-12 21:35`:

```text
copy only variant_metadata+0x06 from Oni variant 586 to custom variant 699
do not change the full metadata clone source, model id, or texture/material RDB behavior
expected patch line: before=26 before_mapped=643 source_value=294 source_mapped=911 after=294 after_mapped=911
installed SHA-256 597BAF276F89DEEC7D4FB226E9FE52DE87730031E67A695CB69BFDD77D6F6D36
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-preview-mapping-oni.20260512-213505.dll
```

Expected next result:

```text
Law extra slot preview mapping patch ... after_mapped=911 match=true
Costume preview model-update ... preview_variant=699 ... mapped294=911
```

If this displays an Oni/Base-compatible model, the private slot needs a proper
preview mapping field in addition to model resource id. If it remains
invisible, trace below the mapper (`FUN_141613030` / child attach), because the
widget id will no longer be stuck on base Law.

### 2026-05-12 21:36 log: preview mapping fixed, UI lock flag still set

The mapping test succeeded:

```text
Law extra slot preview mapping patch ... before_mapped=643 source_mapped=911 after_mapped=911 match=true
Costume preview model-update ... preview_variant=699 visible=1 ... mapped294=911
```

However the user still sees `conditions de deblocage`. This is not the classic
variant unlock hook anymore. In the same log:

```text
Costume variant unlock-check category=26 variant=699 ... original_result=1 result=1
Costume scene scene-available-check ... result=0 effective_result=1 forced=true ... locked=0x01
Costume scene scene-presentation ... selected_variant=699 ... locked=0x01
Costume scene scene-apply ... after=[... selected_variant=699 selected_slot=4 ... locked=0x01]
```

So the scene-local lock byte at `state + 0x44d` survives even after the
available-check override. Installed diagnostic `2026-05-12 21:51` clears that
byte only for Law layout `26` when the current private slot variant `699` is in
`selected_variant` or `object_variant`.

```text
expected log: Law custom scene locked-flag patch ... after=0x00 patched=true
installed SHA-256 AF2C627D117208FB33F98EF20320FF1580FDBB58EC7134E8BCEA61002B9D89F6
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-clear-scene-locked.20260512-215158.dll
```

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 77 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

If this removes the text but the preview stays invisible, resume below
`FUN_14148bcc0` at `FUN_141613030` / preview child attach. The mapper has
already been proven to hand `911` to that layer.

Follow-up from `2026-05-12_21-52-35.log`: this was a negative test. Clearing
`state + 0x44d` makes the later character selection treat Law as locked after
slot 5 is selected, even though the slot state shows `locked=0x00`. Do not use
that flag clear as a fix.

Rollback installed `2026-05-12 21:57`:

```text
LAW_EXTRA_SLOT_CLEAR_SCENE_LOCKED_DIAGNOSTIC_ENABLED = false
installed SHA-256 8E6ECA1753E1DEF477F2E364D9CE2A96331D549BBFE5B1AF77109077982F2130
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-disable-scene-locked-clear.20260512-215756.dll
```

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 78 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

Next model path remains unchanged: trace below the mapper at `FUN_141613030`
or the preview child attach. The UI lock byte is now known to be a dangerous
side path.

Follow-up correction: the slot itself was never the object showing `conditions
de deblocage`. The character Law becomes locked after slot 5 selection, as if
applying costume `699` makes the next character-selection screen reject Law.
Disabling only the locked-byte clear was not enough. The remaining risky
diagnostic was `LAW_EXTRA_SLOT_FORCE_SCENE_AVAILABLE_DIAGNOSTIC_ENABLED`: it
lets the menu apply `699` before the downstream character-selection state knows
how to validate that costume.

Rollback installed `2026-05-12 22:04`:

```text
LAW_EXTRA_SLOT_FORCE_SCENE_AVAILABLE_DIAGNOSTIC_ENABLED = false
LAW_EXTRA_SLOT_CLEAR_SCENE_LOCKED_DIAGNOSTIC_ENABLED = false
installed SHA-256 EA116C5FC31418A74F10277F9D7E0F02334975E9F33FAC38CA70FE4AB600B62B
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-disable-available-force.20260512-220451.dll
```

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 79 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

Do not force scene availability again until the private costume state is valid
for both the costume menu and the next character-selection gate. If Law still
locks after this rollback, trace the selection setter/cache carrying variant
`699` into the character screen. The preview invisibility path still needs
`FUN_141613030` / preview attach instrumentation.

### 2026-05-12 22:15 rollback proof: Law character lock boundary found

The user asked to use the many DLL backups instead of stacking more patches.
Restored the safe pre-lock backup:

```text
D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-scene-list-rebuild-post-flag.20260512-202552.dll
```

User result: Law is no longer locked after selecting/testing slot 5. This
separates two bugs:

```text
1. slot 5 preview/model is still invisible
2. Law character lock was a regression introduced after the restored backup
```

The bad boundary is the later post-rebuild scene-list flag path. It made
`preview_variant=699` more accepted by the costume scene, but that also pushed
the custom costume farther into downstream selection/validation where `699` is
not a complete official costume. The scene-local `locked=0x01` byte is not the
same thing as the Law character lock, and clearing it was a false fix.

Source guardrail added and installed as a guarded test build:

```text
LAW_EXTRA_SLOT_SCENE_LIST_FLAG_DIAGNOSTIC_ENABLED = false
LAW_EXTRA_SLOT_FORCE_SCENE_AVAILABLE_DIAGNOSTIC_ENABLED = false
LAW_EXTRA_SLOT_CLEAR_SCENE_LOCKED_DIAGNOSTIC_ENABLED = false
installed SHA-256 9B12A1B0787703759E2150310B393A184808FA3942ABA2B5EC21DE0C699DC0ED
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-source-guarded-no-lock.20260512-221620.dll
```

Do not re-enable the scene-list flag patch, scene availability force, or locked
byte clear unless a new trace proves exactly how the character-selection gate
validates a private costume. Next useful work is back on the model invisibility
path: trace below `FUN_1416112e0` / `FUN_141613030` and the preview child attach
while keeping the safe pre-lock DLL as the gameplay baseline.

### 2026-05-12 22:31 read-only attach trace installed

User confirmed the guarded source build does not lock Law anymore. Latest safe
log still shows the custom preview hidden:

```text
Costume preview model-update ... preview_variant=699 visible=0 effective_visible=0 forced_visible=false ... mapped294=911 ... child48=0x0
Costume scene list probe ... selected_variant=699 selected_slot=4 ... flags=0000000000000000000000000000
```

The next build adds no state mutation. It hooks `FUN_141613030`, called by
`FUN_14148bcc0` after the mapper writes `mapped294`, and logs only the mapped
resource attach/resolve queue.

```text
trace FUN_141613030 at game+0x1613030
stolen_len=14 from Ghidra-confirmed prologue
log mapped_resource=643 for Law base and mapped_resource=911 for Oni/custom slot 5
installed SHA-256 D4AD9C6546D24760000CFF626711C2465CE861D7F7B39039F327F007CE99C91A
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-preview-resource-attach-trace.20260512-223127.dll
```

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 81 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

Expected next evidence:

```text
Costume preview resource-attach ... mapped_resource=911 ... before=[...] after=[...] result=0x...
```

This should tell whether `911` reaches the attach helper and whether the helper
returns success for the custom preview path. Do not re-enable the scene-list
flag, scene-available force, or locked-byte clear while collecting this log.

### 2026-05-12 22:39 attach succeeds; preview gate probe installed

Result from `2026-05-12_22-34-31.log`:

```text
Costume preview resource-attach ... mapped_resource=911 ... result=0x1
after=[... values=1053,1033,1022,649,637,643,911]
Costume preview model-update ... preview_variant=699 visible=0 effective_visible=0 ... mapped294=911 ... child48=0x0
```

So `FUN_141613030` is not rejecting the custom mapped preview resource. It
accepts `911` and the queue contains it. The remaining blocker is why
`FUN_14148b5f0` chooses the invisible/conditional path and never calls the
visible branch for slot 5 in the safe build.

Installed diagnostic:

```text
add model-update gate probe:
gates=[layout_preview=... runtime_mode=... static_flag=... static_flag20=...]
installed SHA-256 4CB39D17ACF58C8E764B2692C7B41451BFF8C70FDEA3C5079E7B11B4033AED84
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-preview-gate-probe.20260512-223941.dll
```

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 82 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

Next log should compare gates for `preview_variant=586`, `57`, and `699`.
If those gates look good but the branch is still hidden, the next focused
targets are `FUN_1412ffbd0` and `FUN_1412f9320`.

### 2026-05-12 22:53 decision gate trace installed

Follow-up from `2026-05-12_22-40-36.log`: the user confirmed Law is no longer
locked. The safe path now applies slot 5 without forcing scene availability:

```text
Law extra slot variant patch ... slot=4 ... after=699
Law duplicate variant count patch ... before=4 after=5
Costume variant unlock-check category=26 variant=699 slot=4 original_result=1 result=1 forced=false
Costume scene scene-available-check ... selected_variant=699 selected_slot=4 ... result=1 effective_result=1 forced=false
Costume scene scene-apply ... selected_variant=699 selected_slot=4 ... object_variant=699 object_slot=4
```

The private model diagnostic also reaches a loaded pointer through the manager
alias/mirror:

```text
Law private model manager alias ... requested=292 mapped=26 ... result=0x... source_state=2 after_state=2
```

Preview resource attach is not the reject point:

```text
Costume preview resource-attach ... mapped_resource=911 ... result=0x1
```

The remaining blocker is still:

```text
Costume preview model-update ... preview_variant=699 visible=0 effective_visible=0 ... mapped294=911 ... child48=0x0
gates=[layout_preview=26 runtime_mode=0x02 static_flag=0x01 static_flag20=false]
```

New diagnostic is read-only and hooks the two Ghidra-confirmed gates inside the
conditional branch below `FUN_14148b5f0`:

```text
FUN_1412ffbd0 at game+0x12ffbd0, stolen_len=14
FUN_1412f9320 at game+0x12f9320, stolen_len=15
installed SHA-256 5032528A88CF89E256C3E7726A3F780B91EC5749AAA6F3E26D5A32549716D2CC
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-preview-decision-gate-probe.20260512-225248.dll
```

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 83 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

Expected next log lines:

```text
Costume preview gate kind=global-dlc ...
Costume preview gate kind=layout-mode layout_preview=26 mode=4 result=...
```

If they do not appear for the slot 5 model-update call, the branch is skipped
because `static_flag20=false` or another earlier condition fails. If they do
appear, compare the returned values between official Law/Oni and slot 5 before
patching anything.

### 2026-05-12 23:00 hidden/conditional preview branch trace installed

Result from `2026-05-12_22-53-11.log`: the decision-gate hooks installed, but
they never fire from the slot 5 model-update path.

```text
Costume preview model-update ... preview_variant=699 visible=0 effective_visible=0 ... branch=conditional-or-hide
gates=[layout_preview=26 runtime_mode=0x02 static_flag=0x01 static_flag20=false]
Costume preview gate kind=layout-mode ... from_model_update=false
```

There are no `kind=global-dlc` calls and no `from_model_update=true` calls.
That means `FUN_14148b5f0` skips the gated visible fallback before reaching
`FUN_1412ffbd0` / `FUN_1412f9320`. Since official Law/Oni also pass through
`visible=0` in these logs, do not patch `static_flag20` blindly; that branch
looks like a fallback path, not necessarily the normal preview renderer.

New diagnostic remains read-only. It hooks the two direct branch helpers below
`FUN_14148b5f0`:

```text
FUN_14148ae20 hidden branch at game+0x148ae20, stolen_len=16
FUN_14148ac40 conditional-visible branch at game+0x148ac40, stolen_len=17
installed SHA-256 A91032AB1FEBBAA271AC5C8DE61F2C402E4FC77827DCF1B9CE53E20D6E858BD9
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-preview-branch-trace.20260512-230057.dll
```

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 83 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

Expected next evidence:

```text
Costume preview branch label=hidden-branch fallback=...
Costume preview branch label=conditional-visible-branch ...
```

Compare what `FUN_14148ae20` changes on the widget before the tail-update. The
current known bad final state is still `child48=0x0` after `mapped294=911`.

### 2026-05-12 23:13 resource resolve trace installed

Result from `2026-05-12_23-02-13.log`: the hidden-branch probe shows that slot
5 reaches the hide path, but that helper does not construct a visible preview
object for the custom slot.

```text
Costume preview branch label=hidden-branch fallback=0 ... active2a1=0 ... child48=0x0
Costume preview branch label=tail-update ... preview_variant=699 visible=0 ... after=[... mapped294=911 ... child48=0x0]
Costume preview resource-attach ... mapped_resource=911 ... result=0x1
```

There are still no `conditional-visible-branch` calls for `699`. This narrows
the problem: resource `911` is accepted into the preview queue, but the model
preview object is not being created/activated on the widget.

New diagnostic is read-only and hooks the downstream resolver called by
`FUN_141613030`:

```text
FUN_141582f50 resource resolve at game+0x1582f50, stolen_len=17
installed SHA-256 E846D53F159023FF8F8D32C7DD3FEDF690F57F3DA20649AC37B337015F3F2AB6
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-preview-resource-resolve-trace.20260512-231334.dll
```

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 84 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

Expected next evidence:

```text
Costume preview resource-resolve ... mapped_resource=911 ... before=[...] after=[...]
```

If `911` gets a resolver-table slot but `child48` stays null, inspect the
child/widget activation helpers called by the visible branch next. Keep the
force-visible, scene-list flag, scene-available force, and locked-byte
diagnostics disabled.

### 2026-05-12 23:21 conditional-visible diagnostic installed

Result from `2026-05-12_23-15-25.log`: the resolver path is valid for mapped
preview resource `911`.

```text
Costume preview resource-resolve ... mapped_resource=911 ... result=0x1
after=[... match=index:645 ... id=911 state=1 ...]
Costume preview resource-attach ... mapped_resource=911 ... result=0x1
Costume preview model-update ... preview_variant=699 ... mapped294=911 ... child48=0x0
```

So the preview resource is no longer the suspect. Ghidra shows
`FUN_14148b5f0` only calls `FUN_14148ac40` when the layout preview row has
static bit `0x20` and the two gates pass. Slot 5 Law has
`static_flag20=false`, so the game chooses hidden branch `FUN_14148ae20`.

New diagnostic is intentionally narrower than the old failed force-visible
test:

```text
only preview_variant=699, layout=26, visible=0, fallback=0
call FUN_14148ac40 conditional-visible branch directly
then call FUN_14148bcc0 tail update so 699 maps to 911
installed SHA-256 6B3660113BE114ADDAA005257A9337491FFC372B4F9BEBD3AEF92DF1449979B0
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-conditional-preview-diagnostic.20260512-232143.dll
```

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 85 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

Expected next log:

```text
Costume preview model-update ... forced_conditional=true branch=conditional-visible-diagnostic ...
```

If that makes the slot visible, the permanent solution is to provide the
missing conditional-preview metadata/flag for private variants instead of
forcing the branch. If it fails, inspect `FUN_1416046a0` and `FUN_141489700`
with this new evidence.

### 2026-05-12 23:29 preview child entry trace installed

Result from `2026-05-12_23-23-15.log`: the direct conditional-visible
diagnostic is safe and hits slot 5 only. It flips the widget from inactive to
active, and the tail still maps private variant `699` to preview resource
`911`.

```text
forced_conditional=true branch=conditional-visible-diagnostic
before=[... mapped294=643 ... active2a1=0x00 child48=0x0]
after=[... mapped294=911 ... active2a1=0x01 child48=0x0]
```

Because `child48` stays null, the trace now scans the real child collection
used by the Ghidra-visible branches. `before_res` and `after_res` include:

```text
children=[child58=... collection=... found=6:obj=...,flags30=...,b112=...;...]
```

The scanned ids are `6/7/8/11/15`, matching the known branch helpers under
`FUN_14148b5f0`.

Installed:

```text
SHA-256 9BAC21AB3EC3AAA99A51D86D635FC43221D1FB2068FBE468F85220A141D26673
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-preview-child-entry-trace.20260512-232911.dll
```

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 86 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

Next decision point:

- child `6` active after forced conditional but still invisible: follow model
  creation/binding below the widget branch;
- child ids absent or unchanged: inspect child lookup/build helpers
  `FUN_1416045b0`, `FUN_1416046a0`, `FUN_1416170d0`.

### 2026-05-12 23:36 model-update child trace installed

User result with the previous build:

- selecting slot 5 shows a different `conditions de deblocage` message;
- Law no longer becomes locked;
- going in-game does not crash;
- gameplay uses Law base;
- selection preview is still empty.

The gameplay fallback is explained by the current safety alias:

```text
Launch costume private-model alias diagnostic ... target=26 before=292 after=26 patched=true
```

The child probe was present in branch/tail logs, but the forced conditional
diagnostic bypasses the branch hook by calling the original trampoline. The
important `Costume preview model-update ... preview_variant=699
forced_conditional=true` line therefore still lacked `children=[...]`.

New installed diagnostic keeps behavior the same and moves the child/resource
probe onto `model-update` itself:

```text
Costume preview model-update ... before_res=[... children=[...]] after_res=[... children=[...]]
```

Installed:

```text
SHA-256 A639A732B54C7C380040C3650B67AE37F56010CC378471D24649091CDA3460AC
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-model-update-child-trace.20260512-233602.dll
```

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 86 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

Next log should finally show whether the direct forced conditional path mutates
child ids `6/7/8/11/15` on the slot 5 widget.

### 2026-05-12 23:41 forced conditional preview is a negative test

Result from `2026-05-12_23-38-00.log`: the direct forced conditional path does
mutate the child collection, but it still does not produce a model.

```text
preview_variant=699 forced_conditional=true
before: active2a1=0x00 mapped294=643
after:  active2a1=0x01 mapped294=911
children before: 6 flags30=0x007fff80, 8 flags30=0x007fff95
children after:  6 flags30=0x007fff85, 8 flags30=0x007fff90
```

The user still sees `conditions de deblocage`, but Law does not become locked.
The log shows classic unlock and scene availability are accepted, while the
scene-local lock byte remains set:

```text
Costume variant unlock-check category=26 variant=699 ... result=1
Costume scene scene-available-check ... result=1 effective_result=1 forced=false ... locked=0x01
Costume scene scene-presentation ... locked=0x01
Costume scene scene-apply ... selected_variant=699 selected_slot=4 ... locked=0x01
```

Do not clear that byte as a fix; it already caused a character-lock regression
in an earlier diagnostic. The conditional-visible force is now disabled because
it was a negative proof and likely contributes to the odd UI message.

Installed:

```text
LAW_EXTRA_SLOT_FORCE_CONDITIONAL_PREVIEW_DIAGNOSTIC_ENABLED = false
SHA-256 EFBF8ACFBE99100B08AE030353ED80ECB225B98D2D0AB4020DCDAD5470569E58
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-disable-conditional-preview-force.20260512-234130.dll
```

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 86 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

Next target is not the visible/hidden branch anymore. Trace the model
creation/binding path below the accepted preview resource queue.

### 2026-05-12 23:54 companion preview update trace installed

User confirmed the `conditions de deblocage` message disappeared after the
conditional-preview force was disabled:

```text
ca a bien disparu !
```

This confirms the forced conditional branch was only a diagnostic and should
not become the fix. It mutates existing UI child flags and active state, but it
does not create or bind the missing Law slot 5 preview model.

Keep these risky toggles disabled:

```text
LAW_EXTRA_SLOT_FORCE_PREVIEW_VISIBLE_DIAGNOSTIC_ENABLED = false
LAW_EXTRA_SLOT_FORCE_CONDITIONAL_PREVIEW_DIAGNOSTIC_ENABLED = false
LAW_EXTRA_SLOT_SCENE_LIST_FLAG_DIAGNOSTIC_ENABLED = false
LAW_EXTRA_SLOT_FORCE_SCENE_AVAILABLE_DIAGNOSTIC_ENABLED = false
LAW_EXTRA_SLOT_CLEAR_SCENE_LOCKED_DIAGNOSTIC_ENABLED = false
```

The next installed build adds a read-only hook for `FUN_141489700`, the
companion/parallel preview update helper called from `FUN_1414906a0` after the
main preview update path:

```text
FUN_141489700 at game+0x1489700
stolen_len=19
logs widget/layout/resource queue and child ids 6/7/8/11/15/23/31
expected line: Costume companion preview-update ...
```

Why this function matters:

- it receives `widget=*(state+0x100)`, `layout=state+0x404`,
  `force_refresh=1`, and the scene-available flag;
- it maps the layout preview with `FUN_141611460`;
- it calls `FUN_141613030(widget+0x2c0, mapped, ...)`, so it touches the same
  preview resource queue family that already accepted mapped resource `911`;
- it manipulates child ids `7`, `15`, `23`, and `31`, which were not fully
  covered by the earlier visible/hidden branch diagnosis.

Installed:

```text
SHA-256 1C86B62324536E953B75448ED6D659D387742F92EFD27DBF3E4477EB119FCE83
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-companion-preview-trace.20260512-235439.dll
```

Verification:

```text
cargo fmt --check -p oppw4-dinput8-proxy: passed
cargo test -p oppw4-dinput8-proxy: 87 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

Next log should show whether the companion preview path runs for slot 5 and
whether it queues/binds `layout=26` differently from official Law/Oni. If the
new hook does not install, check the log for `unexpected_prologue` and refresh
the Ghidra bytes before touching behavior.

### 2026-05-13 00:02 model manager caller frames installed

Result from `2026-05-12_23-57-02.log`: the companion preview hook installed
and fired, but it is not the missing idle model path.

Key lines:

```text
Costume preview model-update scope=custom preview_variant=699 ... mapped294=643 -> 911
Costume preview resource-attach ... mapped_resource=911 ... result=0x1
Costume companion preview-update ... layout=26 layout_preview=26 ... mapped2d8=1957
Costume scene scene-apply ... selected_variant=699 selected_slot=4
```

Interpretation:

- the main preview path still accepts `911` and keeps the slot 5 widget without
  a visible model object;
- the companion preview path queues/keeps `1957`, not `911`, and only shows
  child ids `6/7`;
- forcing visible/conditional preview already proved negative, so do not go
  back to UI branch mutation;
- the custom zip is still inactive in this safety build:

```text
Law custom slot runtime published: custom=0 original_fallbacks=0
```

The useful remaining question is who asks the model manager for private model
resource `292`. Existing logs prove the alias/mirror works, but lacked caller
frames:

```text
Law private model manager alias action=... requested=292 mapped=26 ...
```

Installed a small read-only logging extension: every private model-manager
alias line now appends `frames=...`.

```text
SHA-256 2BC87CD6D5934D4917C4D9A5B040B100C933D388252519DA2D92A0ADBEEAB9A2
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-model-manager-frames.20260513-000235.dll
```

Verification:

```text
cargo fmt --check -p oppw4-dinput8-proxy: passed
cargo test -p oppw4-dinput8-proxy: 87 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

Next log: inspect `frames=` on all `requested=292` model-manager lines. If the
frames point at preview/menu code, trace that exact caller. If they point only
at launch/gameplay/background preload, stop treating model resource `292` as
the idle preview path and inspect the UI/model object builder around
`FUN_14148e990`, `FUN_14148de00`, and `FUN_1416046a0`.

### 2026-05-13 00:18 selected costume object update trace installed

User supplied the EXE path for Ghidra:

```text
C:\Users\Osef\Documents\Codex\2026-05-09\si-je-te-demanderais-avec-du\OPPW4.exe
```

The `oppw4_game` Ghidra project already had `OPPW4.exe`; the successful export
used `-process OPPW4.exe`.

Result from `2026-05-13_00-04-28.log`: private model resource `292` is requested
from the selected costume object update path, not from the direct preview model
helper.

```text
Law private model manager alias action=can-start-load requested=292 ... frames=game+0x129e8fb game+0x129ec11 game+0x135b57d game+0x1498574 ...
Law private model manager alias action=loaded-check requested=292 ... frames=game+0x1354f15 game+0x2c4741 ...
Law private model manager alias action=get requested=292 ... frames=game+0x1354f28 game+0x2c4741 ...
```

Ghidra identifies `game+0x1498574` inside `FUN_141498580`. This function:

- reads selected index at `state+0x174`;
- reads layout array at `state+0x70`;
- reads slot array at `state+0xb0`;
- resolves `layout + slot -> variant`;
- reads variant metadata model resource from `static_layouts + 0xd92c + variant * 0x1e`;
- calls `FUN_141497f50` with that model resource.

Installed a read-only trace hook:

```text
Costume object update trace hook
RVA 0x1498580
stolen_len 13
expected log: Costume object update ... selected_layout=... selected_slot=... selected_variant=... selected_model_resource=... entries=[...]
```

Installed DLL:

```text
SHA-256 59DC2457081E7D26C26F308BBBD1EB4687F9BFA92E889385A01937F972B34F2A
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-object-update-trace.20260513-001837.dll
```

Verification:

```text
cargo fmt --check -p oppw4-dinput8-proxy: passed
cargo test -p oppw4-dinput8-proxy: 89 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

Next log should be inspected for `Costume object update`. If selected slot 4
reports `variant=699 model=292`, the slot metadata is correct and the next
failure is after `FUN_141497f50`, likely object creation/attach. If it reports
another variant or model, fix the layout/slot metadata path before touching
render/preview code.

### 2026-05-13 00:26 object update trace hook length fixed

Result from `2026-05-13_00-22-41.log`: the installed build did not actually
run the `FUN_141498580` trace because the trampoline failed:

```text
costume object update trace hook failed target=game+0x1498580 error=trampoline_failed
```

Root cause is local to the hook install, not to the game behavior:
`COSTUME_OBJECT_UPDATE_STOLEN_LEN` was `13`, below the generic absolute-jump
minimum of `14`.

Ghidra prologue:

```text
141498580  push rbx                  2 bytes
141498582  sub rsp,0x30              4 bytes
141498586  cmp byte [rcx+0x44d],0    7 bytes
14149858d  mov rbx,rcx               3 bytes
```

Changed stolen length to `16`; no RIP-relative instruction is included.

Installed:

```text
SHA-256 9BDA247E6C1991A72464BC6E15E8004109C73A51C9C12BB160B02C3F865D73FF
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-object-update-trace-len16.20260513-002605.dll
```

Verification:

```text
cargo fmt --check -p oppw4-dinput8-proxy: passed
cargo test -p oppw4-dinput8-proxy: 89 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

Next log should first prove the hook installed, then produce:

```text
Costume object update ... selected_layout=... selected_slot=... selected_variant=... selected_model_resource=... entries=[...]
```

### 2026-05-13 00:31 slot metadata confirmed, loader args trace installed

Result from `2026-05-13_00-26-38.log`: the `FUN_141498580` hook now installs
and proves the selected costume object list is correct.

```text
costume object update trace hook installed target=game+0x1498580
Costume object update ... before=[... selected=3 ... selected_variant=586 selected_model_resource=308]
Costume object update ... after=[... selected=4 ... selected_variant=699 selected_model_resource=292]
entries=[0:layout=26 slot=0 kind=0 variant=57 model=26;1:layout=26 slot=1 kind=0 variant=58 model=227;2:layout=26 slot=2 kind=0 variant=555 model=272;3:layout=26 slot=3 kind=0 variant=586 model=308;4:layout=26 slot=4 kind=0 variant=699 model=292]
```

The private model manager alias still loads `292` successfully through Law base:

```text
Law private model manager alias action=busy-check requested=292 ... result=0 ... after_state=2
Law private model manager alias action=loaded-check requested=292 ... result=1
Law private model manager alias action=get requested=292 ... result=0x...
```

There are no `Model render attach` logs, so the current failure is after slot
metadata/model selection and before or inside preview object creation/attach.

Installed a richer read-only trace on the same object-update hook. New fields:

```text
object_child=...
object_pending=...
controller_loader=...
load_args=<state+0x178>/<state+0x1b8>/<state+0x1f8>
entries=[... args=... variant=... model=...]
```

These are the three values passed into `FUN_14135b7c0` from `FUN_141498580`.
They should be compared between official Oni slot 3 and custom slot 5.

Installed:

```text
SHA-256 22DE0877129CEB3F52C94AF41C1D258FDCD3FA3B755BDB9B8F80F2337E45D0D8
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-object-update-args-trace.20260513-003140.dll
```

Verification:

```text
cargo fmt --check -p oppw4-dinput8-proxy: passed
cargo test -p oppw4-dinput8-proxy: 89 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

Next log: compare `load_args` and `entries args` for slot 3 vs slot 5. If slot
5 args are missing/zero/wrong while slot 3 args are valid, patch those object
update arrays for the private slot. If args are valid, trace `FUN_14135b7c0`
or the helper it calls next.

### 2026-05-13 00:41 model-ready list trace installed

Result from `2026-05-13_00-34-11.log`: the richer object-update trace proves
the custom slot args are valid and match the private model id:

```text
selected=4 selected_layout=26 selected_slot=4 selected_kind=0
load_args=292/65535/0 selected_variant=699 selected_model_resource=292
entries=[... 3:layout=26 slot=3 args=308/65535/0 variant=586 model=308;4:layout=26 slot=4 args=292/65535/0 variant=699 model=292]
object_child=0x... object_pending=0x00 controller_loader=0x...
```

Ghidra export of `FUN_14135b7c0`:

```text
TARGET 14135b7c0 -> function FUN_14135b7c0
prologue stolen window = 15 bytes, no RIP-relative instruction
returns 1 when loader list contains object ids param_2/param_3/param_4
returns 0 when no matching object exists
compares object+0x378, object+0x37c, object+0x384
```

Installed a read-only hook at `game+0x135b7c0` to log the ready-check boundary:

```text
Costume object model-ready check loader=0x... args=<model>/<arg1>/<arg2> result=...
before=[loader=... root=... sentinel=... scanned=... matches=... entries=[...]]
after=[...]
```

Interpretation for next log:

- `args=308/65535/0` is the official Oni comparison row.
- `args=292/65535/0` is the private slot 5 row.
- If `292` has `matches=0` while official rows have a match, the preview
  object/list builder never creates a private-id object even though the model
  manager alias loads resource `292`.
- If `292` has `matches>0` and `result=1`, then the next failing boundary is
  after the ready check, probably `FUN_141494c20` or the selected object
  presentation/update path.

Installed:

```text
SHA-256 0C251C762BA4D18FDB7CC563EF3D49A1D7C6774FAE23F5F1B66E355EF37562A3
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-model-ready-trace.20260513-004138.dll
```

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 90 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

### 2026-05-13 21:55 strict ready timeline installed

The next diagnostic should stop comparing isolated log chunks and instead
compare one ordered chain. A new compact line is emitted as:

```text
Law ready timeline seq=N stage=<stage> ...
```

Stages:

```text
can-start          292 can-start effective result
loader-step        308/292 state28, phase2c, flags20 transitions
model-ready-check  FUN_14135b7c0 result for 308/65535/0 and 292/65535/0
apply-ready        FUN_141494c20 ready value with full selected slot context
child-toggle       FUN_141604620(child58, 0x37, ready)
```

The timeline keeps only the two useful baselines:

```text
slot3-oni:    Law slot 3, variant 586, model 308
slot5-custom: Law slot 5, variant 699, model 292
```

No new behavior patch was added: no force-visible, no force-ready, no child48
write, no scene/UI mutation. The next log should be judged by order:

- `apply-ready ready=0` before `292 flags20=0x05/state28=7`: missing retry after
  load completion;
- `292 flags20=0x05/state28=7` then `model-ready-check result=0`: failure inside
  ready-check conversion;
- `model-ready-check result=1` then `apply-ready ready=0`: ready bool lost
  between the check and `FUN_141494c20`;
- `apply-ready ready=1` but no `child-toggle enabled=1`: problem in
  `FUN_141604620` or the child collection.

Verification:

```text
cargo fmt --check: passed
cargo test -p oppw4-dinput8-proxy: 105 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

Installed:

```text
SHA-256 6877A23FA9A1433E7F2EA0581EB6DE612F54941745FBD0DCF49F34C6E9DEE053
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-ready-timeline.20260513-213530.dll
```

### 2026-05-13 22:05 ready wait gate diagnostic + hooks split

Result from `2026-05-13_21-36-38.log`: the custom loader is no longer the
blocker.

```text
can-start 292 effective result=1
292 loader reaches state28=7 flags20=0x05
FUN_14135b7c0(292/65535/0) still returns 0
apply-ready receives ready=0
child=0x37 receives enabled=0
```

Installed a read-only diagnostic for the next boundary:

```text
FUN_14135af30(object)
  logs object ids, flags20, state28, phase2c, wait3c4, wait3c8, result

FUN_14016e250(manager, resource_id)
  logs manager entry state for resource ids 356022 and 420000
```

This should prove whether `292` fails because `wait3c4=356022` or
`wait3c8=420000` is not globally loaded/ready (`state != 2`) when
`FUN_14135af30` runs.

No behavior mutation was added: no force-visible, no force-ready, no `child48`
write, no new scene/UI patch.

`hooks.rs` was also split mechanically into:

```text
hooks/mod.rs
hooks/costume.rs
hooks/model.rs
hooks/io.rs
hooks/core.rs
hooks/legacy/negative_probes.rs
```

The split uses `include!` to preserve one Rust module and avoid changing
private visibility while the reverse-engineering branch is still moving.

Verification:

```text
cargo fmt --check: passed
cargo test -p oppw4-dinput8-proxy: 106 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

Installed:

```text
SHA-256 251F4F9160407807C87E528964BF853E2CD04CD781D025B46C7818311439F461
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-ready-wait-gate.20260513-214922.dll
```

### 2026-05-13 21:25 child 0x37 toggle probe

The `2026-05-13_21-11-28.log` result moved the investigation below model
loading:

```text
292 reaches flags20=0x05
FUN_141494c20 / apply-ready receives ready=1 for load_args=292/65535/0
FUN_141613030 / resource-attach accepts mapped preview resource 911
child58 is present but child48 remains 0
```

Installed a read-only hook on `FUN_141604620` (`game+0x1604620`), focused only
on the apply-ready child toggle:

```text
FUN_141604620(parent=child58, child_id=0x37, enabled=ready)
```

The log now emits:

```text
UI child toggle parent=... child=0x37 enabled=... before=[...] after=[...]
```

Use it to compare:

```text
Oni slot 3 / model 308:
  child=0x37 enabled=1, lookup ok/missing, child48 before/after

Custom slot 5 / model 292:
  child=0x37 enabled=1, lookup ok/missing, child48 before/after
```

Interpretation:

```text
292 call missing:
  apply-ready context is still not reaching the same child toggle path

292 call present with enabled=0:
  a later apply-ready disables the child after the good ready=1 window

292 call present, lookup missing:
  the parent/child collection for the private path lacks child 0x37

292 call present, lookup ok, state changes, child48 remains 0:
  child48 is written lower than this helper; use the child object as next pivot
```

Verification:

```text
cargo fmt --check: passed
cargo test -p oppw4-dinput8-proxy: 104 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

Installed:

```text
SHA-256 9CA38C7A26FC0B2B66C96A905318D8111D91926B028E8209698D0913EEC6DC09
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-ui-child-toggle.20260513-212413.dll
```

### 2026-05-13 19:43 scene-list flag patch disabled again

`2026-05-13_19-38-15.log` proved that forcing the generated Law bank flag from
`0` to `1` is not a safe preview fix. It makes Law enter the
`conditions de deblocage` UI path:

```text
Law custom scene-list flag diagnostic ... selected_variant=699 selected_slot=4
locked=0x01 ... before=0x00 after=0x01 patched=true
```

Rollback:

```text
LAW_EXTRA_SLOT_SCENE_LIST_FLAG_DIAGNOSTIC_ENABLED = false
```

Keep this as a negative proof. The next useful step is tracing the source
availability path in/around `FUN_141493220` and `FUN_1412f92c0(layout)`, not
patching the post-build scene-list flag byte.

Installed:

```text
SHA-256 CA82D886E4864C3BAE40394A2C2C00E0A1179F5F38354E978E5A3EE8E790ADE6
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-disable-scene-list-flag-patch.20260513-194306.dll
```

Verification:

```text
cargo fmt --check: passed with existing canonicalize warning
cargo test -p oppw4-dinput8-proxy: 97 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

### 2026-05-13 08:25 ready override reverted, apply-ready trace installed

Result from `2026-05-13_08-16-30.log`: the forced model-ready diagnostic was
taken, but it created a loading-loop style failure. The private object exists
and the override changed `original_result=0` to `result=1`, but the selected
scene object stayed unresolved:

```text
args=292/65535/0 original_result=0 result=1
override=[forced=true object=0x... flags20=0x04 wait3c4=356022 wait3c8=420000]

selected_variant=699 selected_slot=4 refresh=0x01 locked=0x01
object_layout=4294967295 object_variant=4294967295 object_slot=none
```

Conclusion: returning ready from `FUN_14135b7c0` is not a valid fix. It only
moves the state machine forward with an object that is not finalized yet.
`LAW_EXTRA_SLOT_MODEL_READY_OVERRIDE_ENABLED` is now `false` again.

Ghidra export added `FUN_141494c20`:

```text
TARGET 141494c20 -> FUN_141494c20
prologue stolen window = 15 bytes
signature: void FUN_141494c20(longlong object, byte slot, undefined4 ready)
core write: *(char *)(object + 0x2ac + slot) = (char)ready
slot 0 also calls FUN_141604620(child58, 0x37, ready)
```

Installed a read-only hook at `game+0x1494c20` named
`Costume object apply-ready`. It logs:

```text
slot=<slot> ready=<ready>
status2ac=<object+0x2ac+slot>
selector290=<object+0x290+slot*4>
pending2b4=<object+0x2b4>
layout440=<object+0x440>
variant448=<object+0x448>
child58/child28/child38/child48/child54/childb4
```

What to look for in the next log:

- If `ready=0` for the custom slot while official slots reach `ready=1`, the
  real blocker remains the private resource readiness check.
- If `ready=1` but `layout440/variant448` stay `4294967295`, the blocker is in
  the selected object presentation/finalization path after `FUN_141494c20`.
- If status/layout/variant become valid but preview stays invisible, return to
  the preview widget branch (`FUN_14148b5f0` / `FUN_14148bcc0`).

Installed:

```text
SHA-256 82C73694B6AF6CC457CC0B16DBCD6C5EB9A5047438B51528D18F92452D23EC20
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-apply-ready-trace.20260513-082503.dll
```

Verification:

```text
cargo fmt --check -p oppw4-dinput8-proxy: passed
cargo test -p oppw4-dinput8-proxy: 91 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

### 2026-05-13 18:04 late model-ready diagnostic installed

Result from `2026-05-13_17-56-31.log`: the `status_trace=true` DLL was active.
The new trace proves the secondary status id `356022` is the timing/gate the
private preview is stuck behind:

```text
Model resource status-check ... id=356022 result=0 ... state=1
Model resource status-check ... id=356022 result=1 ... state=2
```

The private model manager alias is still working:

```text
Law private model manager alias action=loaded-check requested=292 mapped=26 result=1
Law private model manager alias action=get requested=292 mapped=26 result=0x...
```

After the user selects slot 5, the selected object does finalize:

```text
selected=4 selected_layout=26 selected_slot=4
load_args=292/65535/0 selected_variant=699 selected_model_resource=292
object_pending=0x00
```

But the ready boundary remains false:

```text
Costume object model-ready check ... args=292/65535/0
original_result=0 result=0 override=[forced=false]
entries=[... ids=292/65535/0,flags20=0x04,wait=356022/420000]
Costume object apply-ready ... ready=0
```

Important distinction from the failed 08:14 override: the earlier override
forced `FUN_14135b7c0` immediately and caused a loading-loop state where the
selected object never finalized. This new diagnostic only enables the forced
ready return after the object update trace proves the private selection is
finalized:

```text
selected_layout=26
selected_slot=4
selected_load_arg0=292
selected_load_arg1=65535
selected_load_arg2=0
selected_model_resource=292
object_pending=0
```

Implemented as:

```text
LAW_EXTRA_SLOT_LATE_MODEL_READY_OVERRIDE_ENABLED = true
LAW_EXTRA_SLOT_MODEL_READY_OVERRIDE_ENABLED = false
```

Expected next-log signal:

```text
Costume object model-ready check ... args=292/65535/0
original_result=0 result=1 override=[late_forced=true ...]
```

If that makes the preview appear, the root cause is confirmed as the private
object's post-finalization ready gate. If it loops/crashes again, then even a
late ready return is not sufficient and the next step is tracing
`FUN_141497f50`/resource attach after the `ready=0` path.

Installed:

```text
SHA-256 8BD57BF4443FE534F1126836ABE6BFAE62D457168F48A936B2B6734E72018D9A
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-late-ready-diagnostic.20260513-180401.dll
```

Verification:

```text
cargo fmt --check -p oppw4-dinput8-proxy: passed
cargo test -p oppw4-dinput8-proxy: 93 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

### 2026-05-13 18:13 late model-ready diagnostic reverted, metadata trace installed

Result from `2026-05-13_18-06-14.log`: the late ready diagnostic triggered, but
it produced the loading-loop behavior reported by the user.

The key signal:

```text
Costume object model-ready check loader=... args=292/65535/0
original_result=0 result=1 override=[late_forced=true ...]
Costume object apply-ready ... ready=1
```

Immediately after that, the scene repeatedly reports the slot 5 selection with
an invalid/non-finalized selected object:

```text
selected_variant=699 selected_slot=4 refresh=0x01 locked=0x01
object_layout=4294967295 object_variant=4294967295 object_slot=none
```

Interpretation: forcing `FUN_14135b7c0` to return ready is not a valid fix. It
skips a branch in `FUN_141498580` that likely performs a required fallback or
state write before the preview object can become valid.

Active diagnostic changed to:

```text
LAW_EXTRA_SLOT_MODEL_READY_OVERRIDE_ENABLED = false
LAW_EXTRA_SLOT_LATE_MODEL_READY_OVERRIDE_ENABLED = false
```

The `Costume object update` trace now logs the presentation metadata read from
the selected variant:

```text
selected_model_resource=<u16>
selected_preview_mapping=<u16>
selected_preview_mapped=<u32>
entries=[... model=<u16> preview_mapping=<u16> preview_mapped=<u32>]
```

Expected slot 5 values:

```text
selected_variant=699
selected_model_resource=292
selected_preview_mapping=294
selected_preview_mapped=911
```

If the next log shows those values while the preview is still invisible, the
slot/metadata layer is confirmed good and the remaining failure is below it:
model-ready fallback/object attach, not preview mapping or allocation.

Installed:

```text
SHA-256 2B557B7C88A506EEDD54D176EBA5DF5CB720560694D5019B63DA4DC8EAA9DB3F
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-preview-metadata-trace.20260513-181332.dll
```

Verification:

```text
cargo fmt --check -p oppw4-dinput8-proxy: passed with existing canonicalize warning
cargo test -p oppw4-dinput8-proxy: 93 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

### 2026-05-13 18:19 private model-ready flags20 patch diagnostic installed

Result from `2026-05-13_18-15-27.log`: the preview metadata trace confirmed
that slot 5's metadata is correct:

```text
4:layout=26 slot=4 args=292/65535/0 variant=699 model=292 preview_mapping=294 preview_mapped=911
```

The preview side also resolves/attaches the Oni preview resource for variant
699:

```text
Costume preview resource-resolve ... mapped_resource=911 ... result=0x1
Costume preview resource-attach ... mapped_resource=911 ... result=0x1
Costume preview model-update scope=custom ... preview_variant=699 ... mapped294=911
```

The scene object is no longer in the invalid loop produced by forced ready:

```text
selected_variant=699 selected_slot=4
object_layout=26 object_variant=699 object_slot=4
```

The remaining blocker is the private loader entry:

```text
official Oni: ids=308/65535/0,flags20=0x05
private Law:  ids=292/65535/0,flags20=0x04
```

Installed a narrow mutation diagnostic:

```text
LAW_EXTRA_SLOT_MODEL_READY_OVERRIDE_ENABLED = false
LAW_EXTRA_SLOT_LATE_MODEL_READY_OVERRIDE_ENABLED = false
LAW_EXTRA_SLOT_MODEL_READY_FLAG_PATCH_ENABLED = true
```

For the private model only, and only after the finalized slot 5 selection gate,
the hook patches `object+0x20` from `0x04` to `0x05`, then calls the original
`FUN_14135b7c0`. It does not force the result.

Expected log:

```text
override=[flag_patch=true object=0x... before=0x04 target=0x05 after=0x05 write_ok=true ...]
Costume object model-ready check ... args=292/65535/0 original_result=<normal> result=<same>
```

If `original_result` becomes `1` and the preview appears, the root cause is the
missing object-ready bit on the private loader entry. If it still returns `0`,
the next step is tracing `FUN_14135af30` directly to compare the official
`flags20=0x05` object with the private object after the flag write.

Installed:

```text
SHA-256 1DEB782EB8E5621BEDC754F5F8EF2B71249C5E840C8A2EA73782FBC496E2D334
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-model-ready-flag-patch.20260513-181931.dll
```

Verification:

```text
cargo fmt --check -p oppw4-dinput8-proxy: passed with existing canonicalize warning
cargo test -p oppw4-dinput8-proxy: 94 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

### 2026-05-13 18:39 model-ready state-machine detail diagnostic installed

Result from `2026-05-13_18-21-06.log`: the `flags20` patch wrote correctly,
but the preview stayed invisible/loading. Treat this as a negative probe:

```text
override=[flag_patch=true object=0x... before=0x04 target=0x05 after=0x05 write_ok=true ...]
Costume object model-ready check ... args=292/65535/0 original_result=0 result=0
```

Important correction after re-reading the Ghidra export: `FUN_14135b7c0`
returning `0` is not automatically a failure. The function returns `1` only
while a matching loader object still needs waiting. Once `flags20 & 0x04` is set
and `FUN_14135af30(object)` reports ready, `result=0` can mean "nothing
pending". So the old expected signal `original_result=1` was wrong.

New Ghidra export targets from `OPPW4.exe` in project `oppw4_game`:

```text
14129e8fb -> FUN_14129e8c0
14129ec11 -> FUN_14129eb90
14135b57d -> FUN_14135b460
141354f15 / 141354f28 / 141354fa8 -> FUN_141354ef0
14135453e -> FUN_141354510
```

`FUN_14129e8c0` is the useful state machine:

```text
object+0x18 = model resource id
object+0x1c = color variation id
object+0x20 = flags: bit 1 model queued, bit 2 color queued, bit 4 complete
object+0x28 = state
object+0x2c = phase used by FUN_14129eb90
```

Active diagnostic changed back to read-only for this layer:

```text
LAW_EXTRA_SLOT_MODEL_READY_OVERRIDE_ENABLED = false
LAW_EXTRA_SLOT_LATE_MODEL_READY_OVERRIDE_ENABLED = false
LAW_EXTRA_SLOT_MODEL_READY_FLAG_PATCH_ENABLED = false
```

The model-ready loader entries now include:

```text
detail=[load=<+18>/<+1c> state28=<+28> phase2c=<+2c> flags20=<+20> attach08=<+8> token10=<+10> attach30/34=<+30>/<+34> f22c=<+22c> wait=<+3c4>/<+3c8>]
```

Next log goal: compare official Oni `308/65535/0` and private Law
`292/65535/0`. If private Law reaches the same `state28/phase2c/flags20` as
official but preview remains invisible, the blocker is after loader completion:
likely `FUN_141354ef0`/`FUN_141354510` model attach or the preview widget object
creation path. If private Law has a different state/color/pointer pattern, stay
inside the loader state machine.

Installed:

```text
SHA-256 AA91E20BAFA562D20CDFF0756427B2DF5CFE7F260C6A253D5674B38707381F9A
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-model-ready-state-detail.20260513-183943.dll
```

Verification:

```text
cargo fmt --check -p oppw4-dinput8-proxy: passed with existing canonicalize warning
cargo test -p oppw4-dinput8-proxy: 94 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

### 2026-05-13 18:53 model color/material apply trace installed

Result from `2026-05-13_18-44-35.log`: the private slot is still metadata-safe
and the selected object remains finalized:

```text
selected_layout=26 selected_slot=4 load_args=292/65535/0 selected_variant=699 selected_model_resource=292
object_layout=26 object_variant=699 object_slot=4
```

The loader object comparison is now precise. Official rows and private row all
reach `state28=7`, `phase2c=2`, and the same wait ids. The remaining visible
difference is `flags20`:

```text
ids=308/65535/0 detail=[load=308/65535 state28=7 phase2c=2 flags20=0x05 ... wait=356022/420000]
ids=292/65535/0 detail=[load=292/65535 state28=7 phase2c=2 flags20=0x04 ... wait=356022/420000]
```

`can-start-load` explains why the model bit is absent:

```text
Law private model manager alias action=can-start-load requested=292 mapped=26 result=0 source_state=1 before_state=0 after_state=1
```

But forcing that bit was already a negative probe, so do not treat `flags20` as
the sole root cause. The next boundary is after the model manager reports a
loaded model. Ghidra shows:

```text
FUN_141354ef0(state)
  loaded-check *(state+0x30)
  get model pointer
  *(state+0x20) = model pointer
  FUN_141354510(state, *(u16 *)(state+0x34))
```

Installed a read-only hook on `FUN_141354510` at `game+0x1354510`. Its prologue
is safe for the generic trampoline:

```text
141354510 +5 MOV [RSP+0x18],RBX
141354515 +5 MOV [RSP+0x20],RBP
14135451a +2 PUSH R14
14135451c +7 SUB RSP,0x230
stolen length = 19
```

Expected log:

```text
Model color apply trace state=0x... color_arg=...
before=[... model_object20=... model30=292 color34=65535 ...]
after=[...]
```

If `model30=292` appears, the private row reaches the material/color apply path
and the blocker is later in preview widget/render attach. If no `292` line
appears, the next target is `FUN_141354ef0` or the call site around
`game+0x1354f15`, with a hook strategy that does not break RIP-relative
prologue instructions.

Installed:

```text
SHA-256 4C7BFAF29E6D0F99FF2F35CB09DCDB2CDB2F860000304A122C24A63C987BA184
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-model-color-apply-trace.20260513-185314.dll
```

Verification:

```text
cargo fmt --check -p oppw4-dinput8-proxy: passed with existing canonicalize warning
cargo test -p oppw4-dinput8-proxy: 95 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

### 2026-05-13 19:06 preview visible-decision trace installed

Result from `2026-05-13_18-55-29.log`: `model30=292` appears in
`Model color apply trace`, so the private model id reaches the
model/material-color apply path:

```text
Model color apply trace ... model30=292 color34=65535 ...
```

This is a useful positive proof: model manager aliasing/loading is not the
current blocker. The preview path still hides the slot:

```text
Costume preview model-update ... preview_variant=699 visible=0 ... branch=conditional-or-hide ...
Costume scene list probe ... bank=2 index=0 layouts=26,49,50,45,14,11,27,15,10,13,0,0,0,0 flags=0000000000000000000000000000
```

The old render-attach hook at `game+0x03ce790` still does not fire for the
slot-5 preview path. The active diagnostic therefore stays read-only and logs
the exact scene-list visibility decision after `FUN_1414926a0` refreshes the
scene state:

```text
Costume preview visible-decision reason=scene-preview-refresh ...
rule=<direct-index|layout-scan>
direct_slot=... direct_flag=...
layout_slot=... layout_flag=...
decision_slot=... decision_flag=...
flag_gate=<flag-zero|flag-open|missing-slot>
would_call_492e20=<true|false>
```

This mirrors the Ghidra logic in `FUN_1414906a0`: when `list_index < 14`, the
game uses the direct flag slot first and does not scan by layout. If that flag
is zero, it sets the preview-visible value to zero before calling the model
preview helper.

Next log goal: search for `Costume preview visible-decision`.

- If `decision_flag=0` and `would_call_492e20=false`, the next investigation is
  why `FUN_1414926a0`/scene-list rebuild leaves Law's direct preview flag zero.
- If `decision_flag` is nonzero but the preview still hides, hook
  `FUN_141492e20` next because that later check can still force visibility to
  zero.

Installed:

```text
SHA-256 CB0ABC704B265934571245AC7BD8A24456488067E6515536DF613F082773D442
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-preview-visible-decision-trace.20260513-190604.dll
```

### 2026-05-13 19:17 scene-list rebuild detail trace installed

Result from `2026-05-13_19-09-01.log`: the previous
`Costume preview visible-decision` diagnostic proved the preview is hidden
before `FUN_141492e20`. For Law slot 5 the direct scene-list flag is zero, so
`FUN_1414926a0` does not call the later busy/ready check:

```text
Costume preview visible-decision ... selected_variant=699 selected_slot=4 ...
layouts=26,49,50,45,14,11,27,15,10,13,0,0,0,0
flags=0000000000000000000000000000
rule=direct-index direct_slot=0 direct_flag=0x00
decision_slot=0 decision_flag=0x00
flag_gate=flag-zero would_call_492e20=false
```

The selected object and model path are still stable:

```text
Model color apply trace ... model30=292 color34=65535 ...
Costume object update ... selected_variant=699 selected_model_resource=292 ...
object_layout=26 object_variant=699 object_slot=4
```

Installed a read-only detail trace in the existing `FUN_141493820`
scene-list rebuild hook. It logs `list_base`, `param2`, `param2[7]`, fields
`+0x2e0/+0x2e4/+0x2e8/+0x2ec/+0x2f0/+0x2f8/+0x300/+0x308`, and the active
entry layouts/flags before and after the rebuild. It also embeds the
before/after preview visibility decision detail.

Search next log for:

```text
Costume scene list-rebuild detail
```

Interpretation:

- `before_list` zero and `after_list` zero means the builder/source input is
  already producing a non-previewable active entry.
- nonzero before and zero after means `FUN_141493820` is clearing the flags.
- bank/index disagreement with `Costume preview visible-decision` means the
  active preview selection is wrong rather than the model resource.

Installed:

```text
SHA-256 0E5B5F441C80F04B8002AE6CBAAA6F0BA06DD7CB0A9CA5B69EBDB48B1FDFEBB0
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-scene-list-rebuild-detail.20260513-191742.dll
```

Verification:

```text
cargo fmt --check: passed with only the existing canonicalize warning
cargo test -p oppw4-dinput8-proxy: 97 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

Verification:

```text
cargo fmt --check -p oppw4-dinput8-proxy: passed with existing canonicalize warning
cargo test -p oppw4-dinput8-proxy: 97 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

### 2026-05-13 08:45 global resource status-check trace installed

Result from `2026-05-13_08-29-05.log`: preview still invisible, but this log
was produced by the previous DLL. Its startup line is:

```text
Law private model manager alias hooks installed=5 active=true attach_trace=true
```

The newer status-check trace is not present there yet; after the 08:45 build
that line should include `status_trace=true`.

The useful part of the 08:29 log is that the selected costume object now
finalizes correctly again:

```text
selected_variant=699 selected_slot=4 refresh=0x00
object_layout=26 object_variant=699 object_slot=4
```

So the previous forced ready override was correctly removed. The remaining
blocker is the private model-ready gate:

```text
Costume object model-ready check ... args=292/65535/0
original_result=0 result=0 override=[forced=false]
entries=[... ids=292/65535/0,flags20=0x04,wait=356022/420000]
```

At this point the model manager alias is not the failing layer anymore:

```text
Law private model manager alias action=loaded-check requested=292 mapped=26 result=1
Law private model manager alias action=get requested=292 mapped=26 result=0x...
```

Ghidra export of `FUN_14016e250`:

```text
TARGET 14016e250 -> FUN_14016e250
prologue stolen window = 15 bytes
returns 1 when *(manager + resource_id * 0x20 + 0x30) == 2, else 0
```

`FUN_14135af30` calls this global status check for the object's wait ids
`356022` and `420000`. Installed a read-only trace at `game+0x016e250` to log
those two ids only:

```text
Model resource status-check manager=0x... id=356022 result=...
before=[entry=0x... ptr=0x... state=...]
after=[entry=0x... ptr=0x... state=...]
```

Interpretation for the next log:

- If `id=356022` returns `1` with `state=2`, then `FUN_14135af30` returns true
  and `FUN_14135b7c0` rejects the private object because `flags20=0x04`.
- If `id=356022` returns `0`, then the ready-check result contradicts the
  helper logic and the next step is tracing the call timing/state mutation
  around `FUN_14135b7c0`.
- Do not re-enable the old forced ready override as a fix. It caused the
  loading-loop state in `2026-05-13_08-16-30.log`.

Installed:

```text
SHA-256 F06BEA0982998A114B2184DA171B0DFB15480DB7BA7467A4163CBA7766CAAFCF
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-status-check-trace.20260513-084502.dll
```

Verification:

```text
cargo fmt --check -p oppw4-dinput8-proxy: passed
cargo test -p oppw4-dinput8-proxy: 92 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

### 2026-05-13 19:56 `FUN_1412f92c0(layout)` availability trace

Result from `2026-05-13_19-44-59.log`: the unsafe scene-list flag write was
successfully rolled back (`Law custom scene-list flag diagnostic` no longer
appears), and the private model alias still works:

```text
Law private model manager alias action=loaded-check requested=292 mapped=26 result=1
Costume object update ... selected_layout=26 selected_slot=4 load_args=292/65535/0 selected_variant=699 selected_model_resource=292 selected_preview_mapping=294 selected_preview_mapped=911
```

The failing boundary is still before preview visibility:

```text
selected_variant=699 selected_slot=4 list_bank=2 list_index=0 locked=0x01 object_layout=26 object_variant=699 object_slot=4
layouts=26,49,50,45,14,11,27,15,10,13,0,0,0,0 flags=0000000000000000000000000000
```

Do not clear `locked=0x01` and do not force the scene-list flag again. Those
paths already caused Law/global lock regressions.

Installed a read-only no-trampoline hook on `FUN_1412f92c0(layout)` at
`game+0x12f92c0`. Ghidra shows this function decides layout availability from:

```text
row flag: layout * 0x44 + 0x4a, bit 2
target index: layout * 0x44 + 0x14
availability table: base + 0xc1b4, 300 entries, stride 2, entry bit 2 clear
```

The hook reimplements that exact logic and logs:

```text
Costume layout availability-check layout=... row_flag=... target_index=... preview=... matched_index=... matched_flag=... result=... reason=...
```

Next interpretation:

- `row-flag-bit2-zero`: the layout row itself is rejected.
- `target-index-blocked-or-missing`: the row passes, but the table index is
  blocked/missing.
- `matched-open-index`: this function accepts the layout, so the caller inside
  `FUN_141493220` is the next target.

Installed:

```text
SHA-256 0DA9F8C92DCD781BE59FD8D794F560085AD31F8DA7DD89ACE4DB8979072EBE49
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-layout-availability-trace.20260513-195608.dll
```

Verification:

```text
cargo fmt --check: passed with existing canonicalize warning
cargo test -p oppw4-dinput8-proxy: 99 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

### 2026-05-13 20:07 selection setter trampoline fix

Result from `2026-05-13_20-02-22.log`: the availability hook works, but the
game crashed when entering the character menu before Law slot 5 could be
tested. The useful line before the crash:

```text
Costume layout availability-check layout=82 row_flag=0x03 target_index=63 preview=82 matched_index=none matched_flag=none result=0 reason=target-index-blocked-or-missing
```

`crash.log` points at:

```text
RIP Addr.: +0000014CE4890015h
```

The same log mapped the trampoline base:

```text
costume selection setter trace hook installed target=game+0xb16c0 trampoline=0x14ce4890000
```

Root cause: the selection setter hook copied only 15 bytes from
`FUN_1400b16c0`, splitting `MOV R11,RCX`.

Ghidra instruction boundary:

```text
1400b16c0  4053                  PUSH RBX
1400b16c2  4883ec20              SUB RSP,0x20
1400b16c6  4863da                MOVSXD RBX,EDX
1400b16c9  450fb7d0              MOVZX R10D,R8W
1400b16cd  4c8bd9                MOV R11,RCX
```

Fix:

```text
COSTUME_SELECTION_SETTER_STOLEN_LEN = 16
```

Installed:

```text
SHA-256 DCE05DE5C681555FDEB650FFF9A7A530D392DD27E344FA5D1A0E2C57598E602E
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-selection-setter-trampoline-fix.20260513-200711.dll
```

Verification:

```text
cargo fmt --check: passed with existing canonicalize warning
cargo test -p oppw4-dinput8-proxy: 100 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

### 2026-05-13 20:12 availability hook preserves internal volatile registers

Result from `2026-05-13_20-08-31.log`: fixing the selection-setter trampoline
length moved the crash from trampoline code back into the original game
function:

```text
RIP Addr.: OPPW4.exe+00000000000B1707h
```

Ghidra explains it:

```text
1400b16c9  MOVZX R10D,R8W
1400b16cd  MOV R11,RCX
1400b16dc  CALL FUN_1412f92c0
1400b1707  MOV word ptr [R11 + RBX*0x2 + 0x1148],R10W
```

The game's internal callsite keeps `R10` and `R11` live across
`FUN_1412f92c0`. A normal Rust `extern "system"` replacement can clobber those
registers, so the availability hook was corrupting the caller even though the
return value was correct.

Fixed with a naked wrapper around the Rust implementation:

```text
push r10
push r11
sub rsp, 0x28
call inner
add rsp, 0x28
pop r11
pop r10
ret
```

Installed:

```text
SHA-256 22714587E88F4562D9C42FA87F3F62C9AA6E0D23FFEB0C6E63EFFEA32F2D4FA8
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-availability-preserve-r10-r11.20260513-201216.dll
```

Verification:

```text
cargo fmt --check: passed with existing canonicalize warning
cargo test -p oppw4-dinput8-proxy: 101 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

### 2026-05-13 20:20 availability hook preserves R9 too

Result from `2026-05-13_20-13-59.log`: entering the character menu no longer
crashed, but selecting slot 5 crashed at `OPPW4.exe+0x1490A35`.

`FUN_1414906a0` keeps `R9` live across `FUN_1412f92c0(layout)`, then writes the
selected variant through that pointer:

```text
1414909fc  MOV R9,[RCX + 0x10]
141490a0c  CALL FUN_1412f92c0
141490a35  MOV word ptr [R9 + RAX*2 + 0x1148],SI
```

The original helper only clobbers `RAX/RCX/RDX/R8`; it leaves `R9/R10/R11`
alone. Updated the naked wrapper accordingly:

```text
push r9
push r10
push r11
sub rsp, 0x20
call inner
add rsp, 0x20
pop r11
pop r10
pop r9
ret
```

Installed:

```text
SHA-256 7FA81DFE32A03B01DBCB16DA7C1C7F68431FF7041CAFA9767D9C98493ABA6E6E
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-availability-preserve-r9-r10-r11.20260513-202048.dll
```

Verification:

```text
cargo fmt --check: passed with existing canonicalize warning
cargo test -p oppw4-dinput8-proxy: 101 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

### 2026-05-13 20:29 preview visible-argument diagnostic re-enabled narrowly

`2026-05-13_20-21-08.log` changed the diagnosis again:

- no crash when selecting slot 5;
- no obvious crash when launching in-game;
- in-game model setup reaches private model id `292`;
- preview helper reaches custom `preview_variant=699`;
- preview remains invisible because the helper receives `visible=0`.

Key lines:

```text
Model color apply trace ... model30=292 color34=65535 ...
Costume preview model-update ... preview_variant=699 visible=0 effective_visible=0
... layout=26 ... mapped294=911 ... visible2a0=0x00 active2a1=0x00 ...
Costume scene list probe ... flags=0000000000000000000000000000
selected_layout=26 selected_variant=699 selected_slot=4
```

So the next test is deliberately smaller than the unsafe scene-list flag patch:
do not write scene-list flags, do not force scene availability, do not touch
locked flags. Only force the `visible` argument passed to
`FUN_14148b5f0` for the current Law custom variant:

```text
preview_variant == allocated Law slot 5 variant
layout_id == 26
visible == 0
=> effective_visible = 1
```

Installed:

```text
SHA-256 9D902002B19D9FDE3941C367395EBE3FE4F5A74D4E8F1101E85479B7E0FE53AD
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-preview-visible-param.20260513-202928.dll
```

Verification:

```text
cargo fmt --check: passed with existing canonicalize warning
cargo test -p oppw4-dinput8-proxy: 101 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

Expected next log should include:

```text
Costume preview model-update ... preview_variant=699 visible=0 effective_visible=1 forced_visible=true
```

Interpretation:

- if preview appears, find the real upstream source for the custom-slot
  visibility decision;
- if preview stays invisible, inspect preview model construction/resource attach
  after `FUN_14148af40`/`FUN_14148bcc0`.

### 2026-05-13 20:42 preview visible-argument diagnostic reverted

The narrow force-visible build was a negative proof. User reported the
character-level unlock text came back:

```text
deblocage de law a marine ford
```

Therefore the visible-argument override is unsafe too, even when scoped to
`variant=699/layout=26`. Disabled again:

```text
LAW_EXTRA_SLOT_FORCE_PREVIEW_VISIBLE_DIAGNOSTIC_ENABLED = false
```

Installed:

```text
SHA-256 D453A99F4DFFBA65B295260643FE555B125B471F7597A7E8817D0D19BC3B7ACB
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-disable-preview-visible-force.20260513-204223.dll
```

Verification:

```text
cargo fmt --check: passed with existing canonicalize warning
cargo test -p oppw4-dinput8-proxy: 101 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

Do not pursue force-visible as a fix. The next useful path is the private
model/loader object registration comparison: official `308` versus private
`292`.

### 2026-05-13 20:48 model load state-step trace installed

User confirmed the `20:42` DLL removed the Law unlock regression again. The next
diagnostic follows the external recommendation: compare the model-loader object
state machine for official Oni `308` and private `292`, without forcing any UI,
lock, availability, or ready gate.

Hooked:

```text
FUN_14129e8c0 at game+0x129e8c0
stolen_len=14
```

This function is the loader object stepper:

```text
state28=1: can-start-load, enqueue model, flags20 |= 1
state28=2: wait until model not busy
state28=3/4: optional color variation path
state28=5/6: runtime attach/create path
state28=7: flags20 |= 4, return ready
```

Expected log:

```text
Model load state-step object=0x... result=...
before=[detail=[load=... state28=... phase2c=... flags20=... ...] compare_ids=... manager=[...]]
after=[detail=[...] compare_ids=... manager=[...]]
```

Interpretation:

- if `308` and `292` take different transitions, inspect that transition's
  branch/called resource manager function;
- if both take the same path and only `flags20` differs by bit 0, the missing
  bit is a construction/registration property set before this state machine;
- if `292` reaches the same state as `308` but preview stays invisible, the next
  target is the preview widget/resource attach path after the loader object.

Installed:

```text
SHA-256 178565703A87880E4A998891E7364FFB4AD3722D257DC159EA055E0C7356C35F
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-model-load-state-trace.20260513-204834.dll
```

Verification:

```text
cargo fmt --check: passed with existing canonicalize warning
cargo test -p oppw4-dinput8-proxy: 102 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

### 2026-05-13 20:58 private model can-start branch diagnostic installed

Result from `2026-05-13_20-50-36.log`: the model-load state-machine trace
proved the first real divergence between official Oni `308` and private Law
`292`.

Official rows set bit 0 in `flags20` during the `state28=1 -> 2` transition:

```text
load=308/65535 state28=1 flags20=0x00 ... after state28=2 flags20=0x01
```

The private row reaches the same transition but does not set the bit:

```text
load=292/65535 state28=1 flags20=0x00 ... after state28=2 flags20=0x00
```

The manager alias log explains why:

```text
Law private model manager alias action=can-start-load requested=292 mapped=26
call=292 result=0 mirror=present attempted=true write_ok=true match=true
source_state=1 before_state=0 after_state=1
```

The mirror copies source `26` into target `292`, changing the target manager
entry from state `0` to state `1`. Then the original `can-start-load(292)` sees
the target as already loading and returns `0`. Because `FUN_14129e8c0` gets
`0`, it skips the normal enqueue branch and never performs `flags20 |= 1` for
the private loader object. That leaves `292` at `flags20=0x04` later, while
official `308` reaches `0x05`.

Installed a targeted diagnostic:

```text
LAW_EXTRA_SLOT_PRIVATE_MODEL_CAN_START_DIAGNOSTIC_ENABLED = true
```

For `can-start-load` only, the hook changes the effective result to `1` when:

```text
requested=292
mapped=26
original_result=0
mirror present, write_ok, after_matches
before_state=0
after_state=1
source_state is 1 or 2
```

Log line now includes:

```text
original_result=<original> result=<effective> forced_can_start=<bool>
```

Expected next-log proof:

```text
Law private model manager alias action=can-start-load ... original_result=0 result=1 forced_can_start=true
Model load state-step ... load=292/65535 state28=1 ... after ... state28=2 flags20=0x01
Model load state-step ... load=292/65535 state28=7 phase2c=2 flags20=0x05
```

This is deliberately not a UI/visibility/unlock/model-ready force. It only lets
the private model loader take the same initial enqueue branch as an official
model resource. If `292` reaches `flags20=0x05` but the preview stays invisible,
move the investigation forward to preview attach/resource binding.

Installed:

```text
SHA-256 98AD666D3B31AC34EDE6F280C944744928678BECF15F2029D05FAAD5415390AB
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-private-can-start-diagnostic.20260513-205758.dll
```

Verification:

```text
cargo fmt --check: passed with existing canonicalize warning
cargo test -p oppw4-dinput8-proxy: 103 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```

### 2026-05-13 21:09 apply-ready / child attach checkpoint

`2026-05-13_20-59-39.log` confirms the intended private loader behavior:

```text
292: state28=7 phase2c=2 flags20=0x05
308: state28=7 phase2c=2 flags20=0x05
```

The key comparison after selecting slot 5:

```text
Costume object update ... selected_slot=4 load_args=292/65535/0 selected_variant=699 selected_model_resource=292
Costume object apply-ready ... ready=1 ... child58=0x... child48=0x0 ... after ... child48=0x0
```

Verdict: `FUN_141494c20` receives `ready=1` for the private slot path. The
failure is no longer `can-start-load` or `flags20`; it is below that, in preview
widget child creation / render attach. The hook at `game+0x3ce790` was installed
but did not produce a real attach log in this run, so the next probe should find
the function that writes or consumes `child58+0x48`, rather than forcing
visibility or model-ready.

Current DLL adds only read-only context to the apply-ready log:

```text
Costume object apply-ready ... update=[... selected_slot=4 ... load_args=292/65535/0 selected_variant=699 ...]
```

Installed:

```text
SHA-256 ABE1B17A91102884618FAB0BFBA391F73026BE0AF6DC3C113CC63A8D8E34170F
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-apply-ready-context.20260513-210945.dll
```

### 2026-05-13 08:14 private model-ready override diagnostic installed

Result from `2026-05-13_08-07-31.log`: the private object is present in the
model-ready loader list, but `FUN_14135b7c0` returns `0` for the private model.

```text
args=292/65535/0 result=0
entries=[... ids=308/65535/0,flags20=0x05; ids=292/65535/0,flags20=0x04]
```

This means the failure is no longer "private object missing". It is a readiness
gate on the private object. `FUN_14135af30` was exported from Ghidra:

```text
FUN_14135af30(object)
  reads object+0x3c4 and object+0x3c8
  if either value is below 420000 and global resource check returns 0, returns 0
  otherwise returns 1
```

`FUN_14135b7c0` accepts a matching object when:

```text
(object+0x20 & 0x04) == 0 || FUN_14135af30(object) == 0
```

Installed a narrow diagnostic override at the `FUN_14135b7c0` hook:

```text
for args=292/65535/0 only:
  call original and log original_result
  if a matching private object exists and original_result is 0, return 1
```

The line now includes:

```text
original_result=0 result=1 override=[forced=true object=0x... flags20=0x04 wait3c4=... wait3c8=...]
```

Installed:

```text
SHA-256 DF8FF2EEE9FDAC9ABF3E42EBAF21C72A559F317F9F36B0DDF6CB8D670AE3848D
backup D:\SteamLibrary\steamapps\common\OPPW4\dinput8.before-model-ready-override.20260513-081439.dll
```

Verification:

```text
cargo test -p oppw4-dinput8-proxy: 90 passed
cargo test -p oppw4-rdb: 39 passed
cargo build --release -p oppw4-dinput8-proxy: passed
```
