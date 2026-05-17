# Cheat Engine NPC / Status Notes - 2026-05-17

Source: `C:\Users\Osef\Downloads\OPPW4.CT`

## High-value findings

The table exposes a separate runtime character ID family. This is not the same as the LinkData `playable_id` currently stored in `struct-api`.

Example:

- LinkData Law: `playable_id=22`, `model_id=26`
- Cheat Engine runtime Law: `26`

The runtime value appears to be what the game stores in player/actor selection structures.

## Useful pointer chains

### P1 Modifier

- Address: `"OPPW4.exe"+01EC24E0`
- Type: `2 Bytes`
- Offsets: `270 -> 38 -> 0 -> 220 -> 40`
- Dropdown contains normal playable roster IDs.

This likely points to a menu/selection-side P1 character ID.

### Luffy Slot Replacer

- Address: `"OPPW4.exe"+01EC24D8`
- Type: `2 Bytes`
- Offsets: `10 -> 6F8 -> 0 -> 250 -> 18`
- Dropdown contains playable IDs plus NPC/boss IDs.

This is useful for NPC playable experiments.

### Zoro Slot Replacer

- Address: `"OPPW4.exe"+01EC24E0`
- Type: `2 Bytes`
- Offsets: `10 -> 6D0 -> 0 -> 0 -> 20`

Same family as Luffy slot replacement, but for another slot.

### P1Modifier(InBattle)

- Address: `"OPPW4.exe"+01EC24E8`
- Type: `2 Bytes`
- Offsets: `648 -> 140 -> 328 -> 600 -> 240`
- Dropdown contains playable IDs plus NPC/boss IDs.

This is the best current candidate for an in-battle player character probe.

### ACTIVATE PLAYER

- Address: `"OPPW4.exe"+01ECA7D8`
- Type: `8 Bytes`
- Offsets: `460 -> 2F8 -> 170 -> 150 -> 98`
- Known values:
  - `0000000400000004`: disabled
  - `0000000000000000`: power activated
  - `0000000100000001`: speed activated
  - `0000000200000002`: technique activated
  - `0000000300000003`: air activated

Probably a class/type or activation state, not a global game state by itself.

### MOVE ID FINDER

- Address: `"OPPW4.exe"+01ECA7D0`
- Type: `8 Bytes`
- Offsets: `10 -> 2D8 -> 170 -> 150 -> A8`

### ANIMATION ID FINDER

- Address: `"OPPW4.exe"+01ECA7D8`
- Type: `8 Bytes`
- Offsets: `728 -> 2D0 -> 170 -> 150 -> 98`

These two are useful for future character action diagnostics.

## Runtime character IDs from CT dropdowns

Base/DLC roster:

```text
0 Luffy
1 Zoro
2 Nami NPC
3 Usopp
4 Sanji
5 Chopper
6 Nico Robin NPC
7 Franky NPC
8 Brook no moves
9 Ace
10 Hancock
11 Jimbei
12 Whitebeard
13 Buggy
14 Mihawk
15 Crocodile
16 Teech
18 Kizaru
19 Kuzan
20 Akainu
23 Smoker
24 Marco
25 Garp NPC
26 Law
27 Doflamingo
28 Tashigi
30 Fujitora
31 Sabo
32 Lucci
35 Ivankov
36 Shanks
37 Bartolomeo
38 Cavendish
40 Carrot
41 Reiju
42 Ichiji
43 Niji
44 Yonji
45 Bege
46 Big Mom
47 Katakuri
48 Kaido
49 Kid
50 Hawkins
51 New World Luffy
52 New World Zoro
53 New World Nami
54 New World Usopp
55 New World Sanji
56 New World Chopper
57 New World Robin
58 New World Franky
59 New World Brook
60 Smoothie DLC
61 Cracker DLC
62 Judge DLC
63 Drake DLC
64 Killer DLC
65 Urouge DLC
66 Okiku DLC
67 Kin'emon DLC
68 Oden DLC
69 Onigashima Battle Luffy DLC
70 Onigashima Battle Kaido DLC
71 Yamato DLC
72 Uta DLC
73 Shanks DLC
74 Koby DLC
75 Gold Roger DLC
76 Young Garp DLC
77 Young Rayleigh DLC
65535 None
```

NPC/boss slot replacer IDs:

```text
105 Mr.2 NPC
106 Mr.3 NPC
107 Beast Kaku NPC
108 Beast Jabra NPC
109 Blueno NPC
110 Sentomaru NPC
111 Pacifista NPC
114 Jozu NPC
115 Vista NPC
119 Mr.1 NPC
120 Bellamy NPC
125 Burgess NPC
126 Koby NPC
128 Sengoku NPC
129 Kin'emon NPC
132 Pica NPC
135 Giant Pica NPC
138 Diamante NPC
140 Jack NPC
141 Perospero NPC
142 Drake NPC
200 Pirate NPC AXE
201 Pirate NPC ALT AXE
202 Pirate CLUB
204 Marine NPC FISTS
208 Pirate NPC CLAWS
209 Pirate NPC MUSCLE
211 Marine NPC LEGS
219 Franky Family NPC CLUB
220 Giant Marine NPC
222 Nutcracker NPC
223 Big Mom Soldier NPC
224 Wano Soldier NPC
226 Germa Soldier NPC
545 Giant Cracker
565 Kaido Dragon
```

## Boss battle scripts

The `CUSTOM BOSS BATTLES` scripts repeatedly patch three code sites:

```asm
OPPW4.exe+141758E: movzx eax, word ptr [rsi+00000148]
OPPW4.exe+12F613A: movzx r10d, word ptr [rax+rcx*2+10]
OPPW4.exe+14B714B: movzx edx, word ptr [rbx+00000148]
```

Each character script forces the same constant into all three paths:

```asm
mov dword ptr [rsi+00000148],(int)<runtime_id>
mov dword ptr [rax+rcx*2+10],(int)<runtime_id>
mov dword ptr [rbx+00000148],(int)<runtime_id>
```

Current interpretation:

- `+0x148` is very likely a runtime character/actor ID field.
- `rax + rcx*2 + 0x10` is likely a slot/list table of 16-bit character IDs.
- The three sites probably cover selection/load path plus actor/runtime path.

## Game status idea

This CT does not directly expose a clean enum like `main_menu`, `character_select`, `battle`.

It does give enough pointers to build a better status probe:

- If the P1 modifier chain resolves, the roster/selection state is probably initialized.
- If `P1Modifier(InBattle)` resolves and reads a sane runtime ID, we are probably in battle.
- If move/animation chains resolve, the local actor/action structures are probably live.
- If `ACTIVATE PLAYER` resolves, the player runtime state is probably live.

Suggested next implementation:

1. Add a diagnostic reader for these pointer chains in `hooks`.
2. Log only transitions:
   - pointer chain unresolved -> resolved
   - runtime ID changed
   - move/animation ID changed
3. Extend `Oppw4GameStatus` later with flags such as:
   - `CHARACTER_SELECT_CHAIN_SEEN`
   - `BATTLE_PLAYER_CHAIN_SEEN`
   - `LOCAL_ACTION_CHAIN_SEEN`

Avoid writing through these chains until read-only diagnostics prove stable.
