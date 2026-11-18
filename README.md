# Chisel (Chisel Reborn) for Horizon — one block family and the chisel

A Horizon port of **Chisel Reborn 2.0.1** (`chisel`, GPL-2.0-only) carrying the
**`andesite` family** — all 32 of its variants — and the chisel that converts a placed
block to the next member of its family.

- **Class / tier:** content / machines in the pack's row, shipped as a **data** carry plus
  one `logic` behaviour (the chisel tool). The machines half (the chisel GUI) is a gap.
- **Licence:** GPL-2.0-only, rank 1 — publishable under the same licence with the complete
  corresponding source. `LICENSE` + `LICENSE-NOTICE.md` travel inside this package.
- **Format:** one flat SDK package (mod API `0.2`) built against the SDK for Horizon
  `0b14f8acfd97fa2c750e42795249dcc981178ca4` and horizon-mixins
  `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, for Minecraft 26.3.

## The package

`chisel.hmod` — one file: `horizon-mod.toml`, `Cargo.toml`, `src/lib.rs` (the one
`impl mod_api::Hooks`), `src/runtime.rs` (the pure family table and its tests), the carried
`assets/` + `data/` and the licence. 287 content files: 32 blockstates, 58 block models, 32
item models and 26.3 item definitions, 61 textures, 32 loot tables, the lang file, the chisel
item, its recipe, the creative tab and the `mineable/pickaxe` tag.

## What it does in game

A **dig** (left click) on a block with the chisel in hand does not mine it: the block is
replaced by the next member of its family, in the order the mod registers it —

```
minecraft:andesite → minecraft:polished_andesite → chisel:array/andesite → chisel:braid/andesite
→ … → chisel:zag/andesite → minecraft:andesite
```

A block outside the family is left alone (and, as upstream, not mined either). The target is
always written as its **default state**, so a conversion into `pillar`/`twist` lands on
`axis=y`.

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
it from the jar (it fails loudly on a missing input). The ladder, the retired 0.1.0 e2e legs
and the measured harness facts: [`VALIDATION.md`](VALIDATION.md). The jar inventory, the
conversion table's provenance and the upstream defects: [`FINDINGS.md`](FINDINGS.md). Scope,
gaps and deviations: [`notes.md`](notes.md).
