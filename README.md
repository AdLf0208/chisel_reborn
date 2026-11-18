# Chisel Reborn — a Horizon port

**Chisel Reborn** is a Minecraft mod. Chisel brought to modern versions of Minecraft 1.21+. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [matthewperiut/Chisel-Reborn](https://github.com/matthewperiut/Chisel-Reborn) — licence **GPL-2.0-only**.

| | |
|---|---|
| Upstream mod | Chisel Reborn ([source](https://github.com/matthewperiut/Chisel-Reborn)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `chisel.hmod` (install it with the Horizon launcher) |
| Licence | **GPL-2.0-only** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

- **The conversion** is `ChiselItem.canMine` (`common/src/main/java/com/periut/chisel/item/
- **The group** for `andesite` is 34 members: `VanillaChisel.Register` puts
- **The block class** is `EzReg.Reg`'s choice: `PillarBlock` (an `axis` property, default
- **The item** is `maxCount(1)`, a `BundleItem` with no bar, `getMiningSpeed` 500 for every
- **None in the rule.** The group, the order, the wrap and the default-state write are the
- **A missing content half is silent.** 0.1.0 logged "Chisel has no block state for {id}"; the
- **The dig is heard from the same moment as before** (`left_click_block`, the pin's

## Not yet ported

- **1,377 of the mod's 1,409 blocks** — the other 99 material families — are not carried.
- **The chisel GUI** (`ChiselScreenHandler`, a bundle-backed inventory screen, and
- **The chisel's sound** (`chisel:chisel_sound`, `assets/chisel/sounds.json` +
- **CTM.** 4,796 `assets/chisel/optifine/` files and 4,795 `textures/block/ctm/` sprites
- **`getMiningSpeed` 500** is declared as the item's `minecraft:tool` component
- **Translations**: only `en_us` is carried (upstream also ships `ru_ru`, `zh_cn`).
- **Tags**: only `minecraft:mineable/pickaxe` (upstream also adds `mineable/axe`,
- **The creative tab** carries the chisel and the 32 variants only, not the other 1,377

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `chisel.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

Chisel Reborn was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **GPL-2.0-only**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/matthewperiut/Chisel-Reborn

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run chisel` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/chisel/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
