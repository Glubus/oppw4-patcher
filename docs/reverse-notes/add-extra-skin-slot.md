# Add Extra Costume Slot Notes

Last updated: 2026-05-09

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

The known costume list ends around suffix `136`
(`806_136_costume_dlc_pc_SSnake`). There are gaps.

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
- add a dry-run patch plan that prints every data mutation before writing;
- keep runtime DLL changes minimal until the data format is proven.
