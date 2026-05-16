# Collaboration Rules

These rules describe how to work on this repository. They are written for both
humans and LLM agents.

## Project Goals

- Keep the loader small, predictable, and easy to audit.
- Prefer explicit archive/file decisions over hidden heuristics.
- Preserve compatibility with OPPW4's RDB layout and the original patcher behavior
  when that behavior is understood.
- Keep runtime mod content outside the source tree. The game-side layout owns
  `mods/`, `_oppw4/`, and future release artifacts.

## Repository Boundaries

- Source code lives under `apps/` and `crates/`.
- Shared reverse-engineering notes live under `docs/`.
- Embedded static resources live under `resources/`.
- The proxy DLL scans game-side `OPPW4/mods/`, not a source-tree mod folder.
- Do not add generated binaries, loose mods, or release archives to the source
  tree unless a task explicitly asks for a fixture.

## Code Style

- Keep one clear responsibility per file.
- Split thick parsing, scanning, and hook functions into named helpers.
- Avoid creating many narrow sibling modules with repeated prefixes. Prefer one
  focused folder with a `mod.rs` when a domain needs multiple files.
- Favor structured parsing and typed data over ad-hoc string manipulation.
- Keep unsafe Windows hook code isolated from pure parsing and table-building
  logic.
- Use `tracing` for new logging paths.

## Testing Rules

- Add or update tests for every parser, scanner, mode decision, or archive lookup
  change.
- Tests should cover the bug shape, not only the happy path.
- Keep tests mandatory for future rewrites:
  - catalog parsing;
  - RDB entry classification;
  - virtual/internal/external replacement decisions;
  - loose mod and zip mod scanning;
  - disabled hash filtering;
  - virtual file reads and size prefix behavior.
- Run the narrow relevant test first, then the workspace test before calling a
  change done.

## Documentation Rules

- Documentation must be in English.
- Reverse-engineering notes should separate confirmed facts from guesses.
- When a Ghidra name is uncertain, label it as probable instead of presenting it
  as final.
- Record offsets, hashes, function names, and log evidence when they explain a
  fix.
- Keep docs useful to a fresh LLM session: include file paths, known behavior,
  open questions, and the next likely investigation step.

## Git Rules

- Do not commit unless the user explicitly asks for a commit.
- Do not rewrite history or discard local changes unless the user explicitly asks.
- Before a commit, summarize changed files and verification results.

## LLM Workflow

- Read the local code before proposing architecture changes.
- Follow existing project shape before inventing a new abstraction.
- Make small, reversible edits.
- Prefer `rg` for file and text search.
- Use `apply_patch` for manual edits.
- Verify changes with formatting and tests when possible.
- If a behavior comes from the original DLL, cite the relevant docs or log lines.
- If a behavior is newly chosen for the Rust rewrite, document it as a design
  decision.

## Current Runtime Contract

- `dinput8.dll` is installed next to the OPPW4 executable.
- For game testing, always build the DLL in release mode with
  `cargo build --release -p oppw4-dinput8-proxy`, then place
  `target/release/dinput8.dll` in `D:\SteamLibrary\steamapps\common\OPPW4`.
- Loose mods use `OPPW4/mods/<ModName>/<ArchiveName>/<asset>`.
- Old loose archive folders can be migrated as
  `OPPW4/mods/legacy/<ArchiveName>/<asset>`.
- Zip mods can contain `<ArchiveName>/<asset>` or
  `<ModName>/<ArchiveName>/<asset>`.
- `OPPW4/mods/_oppw4/` is reserved for loader configuration and logs.
- The name/hash catalog is embedded in the DLL by default and can be overridden
  with `OPPW4/mods/_oppw4/name_hash_catalog.txt` for debugging.
## OPPW4 LinkData Gameplay Overrides

- Do not use compressed LinkData rebuilds for gameplay tests. In this project,
  generated `LINKDATA_A.BIN` files made with compressed rebuild mode have caused
  black screens even when the logical data looked correct.
- Use raw-expanded LinkData overrides for gameplay tests unless a future test
  explicitly proves compressed output safe again.
