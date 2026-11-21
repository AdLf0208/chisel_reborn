# Chisel (Chisel Reborn) for Horizon — two block families and the chisel

A Horizon port of **Chisel Reborn 2.0.1** (`chisel`, GPL-2.0-only) carrying the
**`andesite` family** (all 32 of its variants) and the **`blackstone` family** (its pillar),
plus the chisel that converts a placed block to the next member of its own family — or to the
block it carries, when one is put inside it.

- **Class / tier:** content / machines in the pack's row, shipped as a **data** carry plus
  the chisel's `logic` behaviour and its component-carrying inventory (the bundle the chisel
  holds its chosen block in). The custom GUI of 1.21.1 is not carried: at this pin a bundle's
  contents are edited by the game's own inventory, which is the same capability.
- **Licence:** GPL-2.0-only, rank 1 — publishable under the same licence with the complete
  corresponding source. `LICENSE` + `LICENSE-NOTICE.md` travel inside this package.
- **Format:** one flat SDK package (mod API `0.2`) built against the SDK for Horizon
  `0b14f8acfd97fa2c750e42795249dcc981178ca4` and horizon-mixins
  `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, for Minecraft 26.3.

## The package

`chisel.hmod` — one file: `horizon-mod.toml`, `Cargo.toml`, `src/lib.rs` (the one
`impl mod_api::Hooks`), `src/runtime.rs` (the pure family tables, the lock rule and their
tests), the carried `assets/` + `data/` and the licence. 296 content files: 33 blockstates,
60 block models, 33 item models and 26.3 item definitions, 63 textures, 33 loot tables, the
lang file, the chisel item, its recipe, the creative tab and the `mineable/pickaxe` tag.

## What it does in game

A **dig** (left click) on a block with the chisel in hand does not mine it: the block is
replaced by the next member of **its own** family, in the order the mod registers it —

```
andesite:  minecraft:andesite → minecraft:polished_andesite → chisel:array/andesite
           → chisel:braid/andesite → … → chisel:zag/andesite → minecraft:andesite

blackstone: minecraft:blackstone → minecraft:polished_blackstone
           → minecraft:chiseled_polished_blackstone
           → minecraft:polished_blackstone_bricks
           → minecraft:cracked_polished_blackstone_bricks
           → chisel:pillar/blackstone → minecraft:blackstone
```

A block outside every carried family is left alone (and, as upstream, not mined either), and a
member of one family never steps into another. The target is always written as its **default
state**, so a conversion into `pillar`/`twist` lands on `axis=y`.

**Choose the variant.** The chisel is a bundle: put a block in it in your inventory (pick the
block up and click it onto the chisel) and the chisel stops walking the family — every block of
that block's material becomes **the block you put in**. A chisel carrying `chisel:panel/andesite`
turns any andesite block into the panel; a chisel carrying a blackstone block leaves andesite
alone and changes nothing. The carried block stays in the chisel, so the lock lasts; put another
block in and the newest one becomes the lock.

The chisel itself is crafted from an iron ingot and a stick (`" I"` / `"S "`).

## Working on it

```sh
# the pure rule and the glue, without the game
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check

# the package (the SDK is fetched into .horizon/sdk/ on first use and never packed)
rm -f Cargo.lock
cargo horizon-mod build                 # -> target/horizon-mod/chisel.hmod
cargo horizon-mod check chisel.hmod     # the registry's own check
cargo horizon-mod fits chisel.hmod --build nightly
```

The content is carried from the jar and must stay unchanged; `tools/gen_content.py` rebuilds
it from the jar (it fails loudly on a missing input, and `--check` proves every input exists).
The ladder, the in-game legs and the measured harness facts: [`VALIDATION.md`](VALIDATION.md).
The jar inventory, the conversion table's provenance and the upstream defects:
[`FINDINGS.md`](FINDINGS.md). Scope, gaps, the coverage table and deviations:
[`notes.md`](notes.md).
