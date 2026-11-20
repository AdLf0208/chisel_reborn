# Chisel (Chisel Reborn) — findings

Inventory of the artefact **before** any code was written. Jar:
`ports/_shared/licence-cache/chisel-neoforge-2.0.1+mc1.21.1.jar` (8,268,524 B,
**19,369 entries**). Upstream source read for behaviour only:
`matthewperiut/Chisel-Fabric` tag **`1.21.1-2.0.1`** (`b399d0f218acae743321d64604fb378c6c7b791c`,
`gradle.properties: mod_version = 2.0.1`, `minecraft_version = 1.21.1`), the version the
jar is built from. Clean-room rewrite: the Java was read for behaviour, no code was copied.

## 1. What the jar actually contains

| top level | entries | what |
|---|---:|---|
| `assets/chisel/textures` | 6,334 | block + item sprites (6,328 under `textures/block/`, 4,795 of them `block/ctm/`) |
| `assets/chisel/optifine` | 4,796 | CTM `.properties` for OptiFine/Continuity — client-only, not carried |
| `assets/chisel/models` | 3,925 | `block/<style>/<material>.json` + `item/<style>/<material>.json` |
| `assets/chisel/items` | 1,411 | 26.3 item definitions (`{"model":{"type":"minecraft:model",…}}`) |
| `assets/chisel/blockstates` | 1,410 | `blockstates/<style>/<material>.json`, **two path segments** |
| `assets/chisel/lang` | 3 | `en_us.json` (2,773 lines), `ru_ru`, `zh_cn` |
| `assets/chisel/sounds` | 2 | `chisel_sound.ogg` + `sounds.json` (`chisel:chisel_sound`) |
| `data/chisel/loot_table/blocks` | 1,414 | one self-drop table per block |
| `data/chisel/recipe` | 2 | **only** `chisel.json` (the chisel itself) |
| `data/minecraft/tags/block` | 5 | `mineable/{pickaxe,axe,shovel}`, `needs_{stone,iron}_tool` |
| `data/c/tags/block` | 2 | `glass_blocks` |
| `com/periut/…` | 42 | the mod's classes (not read as code) |

`META-INF/neoforge.mods.toml`: `modId = "chisel"`, `version = "2.0.1+mc1.21.1"`,
`license = "GPLv2"`, `authors = ["Slaincow", "KnowYourKnot", "Original Creator:
AUTOMATIC_MAIDEN"]`, `displayName = "Chisel Reborn"`. Required deps: minecraft, neoforge,
architectury, cryonicconfig. **The jar carries no licence text** (only `MANIFEST.MF` and
`neoforge.mods.toml` under `META-INF/`) and the ported tag has no `LICENSE` file either, so
the port ships the canonical GPL-2.0 text from
<https://www.gnu.org/licenses/old-licenses/gpl-2.0.txt> (338 lines, SHA-256
`edaef632…33f6`), as the brief allows.

**There is no `data/chisel/horizon/`**: the blocks are registered in Java, so every
declaration in this port is written from the jar's own blockstates/models/loot/lang plus
the registration code's properties, never from the mod's name.

## 2. The block id shape — the reason for the slice boundary

`GeneratedRegister.Register()` (`common/src/main/java/com/periut/chisel/block/GeneratedRegister.java`)
calls `Reg("<material>", "<style>/<material>")` 1,394 times; `EzReg.Reg`
(`block/blocks/EzReg.java:35-37,72-76`) registers the block as
`Identifier.of("chisel", name)` — i.e. **`chisel:<style>/<material>`**, a registry name with
a `/` in its path, and registers the block's item under the same id.

So a "family" is one **material**, and its variants are the styles:

- 1,409 blockstates, each `blockstates/<style>/<material>.json`; 117 styles, 100 materials.
- Family sizes (blockstates per material, from the jar): `stone`/`sandstone`/`red_sandstone`/
  `deepslate` 35, `coal_block` 33, the sixteen concretes and `quartz`/`purpur`/`prismarine`/
  `ice`/`glowstone`/`end_stone`/`granite`/`diorite`/`cobblestone`/`bricks`/`andesite` 32,
  `redstone_block` 27, `glass` 19, `iron_block` 15, the planks 14.

**Slice 1 = the `andesite` family, all 32 of its variants.** It is the smallest family that
is still a complete, representative one: 30 plain full cubes, plus the two `axis` variants
(`pillar/andesite`, `twist/andesite`), three weighted-model variants
(`chaotic_bricks`, `cut`, `french` — 9, 16 and 2 models), and the 26.3 item definitions.
1,377 other blocks (99.9 % of the jar's blocks) were outside slice 1 and are recorded as a
gap.

**Slice 2 (0.2.2) adds the `blackstone` family**, its one registered variant
(`chisel:pillar/blackstone`, `GeneratedRegister.java:104`), on the `minecraft:blackstone`
base. It is the smallest complete family in the jar and the only one whose group is mostly
*vanilla* blocks: `VanillaChisel` puts five blackstone forms in it, so the chisel walks
`blackstone → polished_blackstone → chiseled_polished_blackstone →
polished_blackstone_bricks → cracked_polished_blackstone_bricks → chisel:pillar/blackstone →
blackstone`. **1,376 blocks in 98 families remain outside both slices.**

## 3. The group the chisel converts within (the conversion table)

`ChiselItem.canMine` (`common/src/main/java/com/periut/chisel/item/ChiselItem.java:126-171`)
is the conversion path: with a chisel in hand, a **mining attempt** on a block

1. reads `ChiselGroupLookup.getBlocksInGroup(state.getBlock().asItem())`;
2. if the group is empty → nothing happens and the method still returns `false`
   (the block is **not** mined);
3. if the chisel's bundle is empty → `Registries.BLOCK.get(random member).getDefaultState()`
   is written to the block's position and `chisel:chisel_sound` plays;
4. if the bundle holds a block → the last `/`-segment of the held item's path must contain,
   or be contained by, the target's, and the held block is written instead.

`getMiningSpeed` returns **500** for every block (`ChiselItem.java:173-181`), so the mining
attempt completes at once; `isItemBarVisible` is `false` (`:39-42`) — no durability.

Group membership and order come from two registration passes, in this order
(`block/BlockRegister.java:7-11`):

1. `VanillaChisel.Register()` (`block/blocks/VanillaChisel.java:11-47`) adds the vanilla
   block first, then its polished form: `"andesite", "polished_andesite"`;
2. `GeneratedRegister.Register()` adds `minecraft:andesite` again (dropped as a duplicate by
   `ChiselGroupLookup.ChiselGroup.getItems`, which keeps the first occurrence,
   `block/ChiselGroupLookup.java:141-166`), then `chisel:<style>/andesite` in the file's
   order (alphabetical by style).

The `andesite` group is therefore **34 members**, in this order:

```
minecraft:andesite, minecraft:polished_andesite,
chisel:array/andesite, chisel:braid/andesite, chisel:chaotic_bricks/andesite,
chisel:chaotic_medium/andesite, chisel:chaotic_small/andesite, chisel:circular/andesite,
chisel:cracked/andesite, chisel:cracked_bricks/andesite, chisel:cut/andesite,
chisel:dent/andesite, chisel:encased_bricks/andesite, chisel:french/andesite,
chisel:jellybean/andesite, chisel:large_tile/andesite, chisel:layer/andesite,
chisel:mosaic/andesite, chisel:ornate/andesite, chisel:panel/andesite,
chisel:pillar/andesite, chisel:prism/andesite, chisel:raw/andesite, chisel:road/andesite,
chisel:slant/andesite, chisel:small_bricks/andesite, chisel:soft_bricks/andesite,
chisel:solid_bricks/andesite, chisel:tiles_medium/andesite, chisel:tiles_small/andesite,
chisel:triple_bricks/andesite, chisel:twist/andesite, chisel:weaver/andesite,
chisel:zag/andesite
```

Two of the 34 are vanilla blocks (they are group members, not registrations: the chisel
turns vanilla andesite into a chisel variant and a variant back into vanilla andesite).

## 4. Per-variant inventory carried by slice 1

For each of the 32 styles: blockstate, block model(s), item model, 26.3 item definition,
loot table, block texture. Verified against the jar — **nothing is missing**:

| # | id | blockstate variants | models | texture |
|---:|---|---|---:|---|
| 1 | `chisel:array/andesite` | `""` (Athena giant) | 1 | 1 |
| 2 | `chisel:braid/andesite` | `""` | 1 | 1 |
| 3 | `chisel:chaotic_bricks/andesite` | `""` weighted ×9 | 9 | 9 (CTM sprites) |
| 4 | `chisel:chaotic_medium/andesite` | `""` | 1 | 1 |
| 5 | `chisel:chaotic_small/andesite` | `""` | 1 | 1 |
| 6 | `chisel:circular/andesite` | `""` (Athena CTM) | 1 | 1 |
| 7 | `chisel:cracked/andesite` | `""` | 1 | 1 |
| 8 | `chisel:cracked_bricks/andesite` | `""` | 1 | 1 |
| 9 | `chisel:cut/andesite` | `""` weighted ×16 | 16 | 16 (CTM sprites) |
| 10 | `chisel:dent/andesite` | `""` (Athena CTM) | 1 | 1 |
| 11 | `chisel:encased_bricks/andesite` | `""` (Athena CTM) | 1 | 1 |
| 12 | `chisel:french/andesite` | `""` weighted ×2 | 2 | 2 (CTM sprites) |
| 13 | `chisel:jellybean/andesite` | `""` (Athena giant) | 1 | 1 |
| 14 | `chisel:large_tile/andesite` | `""` (Athena CTM) | 1 | 1 |
| 15 | `chisel:layer/andesite` | `""` | 1 | 1 |
| 16 | `chisel:mosaic/andesite` | `""` (Athena CTM) | 1 | 1 |
| 17 | `chisel:ornate/andesite` | `""` | 1 | 1 |
| 18 | `chisel:panel/andesite` | `""` | 1 | 1 |
| 19 | `chisel:pillar/andesite` | `axis=x/y/z` | 2 (`+_horizontal`) | 2 (`+_top`) |
| 20 | `chisel:prism/andesite` | `""` | 1 | 1 |
| 21 | `chisel:raw/andesite` | `""` | 1 | 1 |
| 22 | `chisel:road/andesite` | `""` | 1 | 1 |
| 23 | `chisel:slant/andesite` | `""` (Athena giant) | 1 | 1 |
| 24 | `chisel:small_bricks/andesite` | `""` | 1 | 1 |
| 25 | `chisel:soft_bricks/andesite` | `""` | 1 | 1 |
| 26 | `chisel:solid_bricks/andesite` | `""` | 1 | 1 |
| 27 | `chisel:tiles_medium/andesite` | `""` | 1 | 1 |
| 28 | `chisel:tiles_small/andesite` | `""` | 1 | 1 |
| 29 | `chisel:triple_bricks/andesite` | `""` | 1 | 1 |
| 30 | `chisel:twist/andesite` | `axis=x/y/z` | 2 (`+_horizontal`) | 2 (`+_top`) |
| 31 | `chisel:weaver/andesite` | `""` (Athena CTM) | 1 | 1 |
| 32 | `chisel:zag/andesite` | `""` (Athena giant) | 1 | 1 |

Totals: 32 blockstates, **58 block models**, 32 item models, 32 item definitions, 32 loot
tables, **61 textures** (34 plain + 27 CTM sprites the weighted models name directly), 3
model parents — all vanilla (`block/cube_all`, `minecraft:block/cube_column`,
`minecraft:block/cube_column_horizontal`), so no shared mod parent model is needed.

**No animated texture**: none of the 61 PNGs has a `.png.mcmeta` (checked).

## 5. The two block kinds

`EzReg.Reg` (`EzReg.java:57-77`) builds each block with `AbstractBlock.Settings.copy(baseBlock)`
where `baseBlock` is the vanilla material's block (`minecraft:andesite` for this family), and
picks the class by the style name:

- `pillar = name.contains("pillar") || name.contains("twist")` → `PillarBlock`, which adds
  the `axis` property (`x`/`y`/`z`, default `y`) and its `cube_column`/`cube_column_horizontal`
  models. That is exactly the two variants whose blockstate has `axis=`.
- everything else → plain `Block` with andesite's settings (hardness 1.5, blast resistance
  6, `requiresCorrectToolForDrops`, stone sound, stone map colour).

## 6. Data carried besides the blocks

- **Loot**: `data/chisel/loot_table/blocks/<style>/andesite.json` — one pool, one item entry
  (`chisel:<style>/andesite`), one condition. 1.21.1 spells it `"conditions": [{"condition":
  "minecraft:survives_explosion"}]`; 26.3 reads the singular typed form
  (`docs/mods.md`, "Loot tables must use the singular typed condition"), so the port writes
  `"condition": {"type": "minecraft:survives_explosion"}` — the same table in the target
  shape.
- **Tags**: the 32 ids are in `data/minecraft/tags/block/mineable/pickaxe.json` upstream; the
  port adds exactly those 32 to that vanilla tag. They are **not** in
  `needs_stone_tool`/`needs_iron_tool` (that file lists iron/lapis blocks only).
- **Recipe**: `data/chisel/recipe/chisel.json`, shaped `" I"` / `"S "` from iron ingot and
  stick → 1 `chisel:chisel`. 1.21.1 spells an ingredient `{"item": "minecraft:iron_ingot"}`;
  26.3 spells it as a plain id string (vanilla's own `data/minecraft/recipe/stone_pickaxe.json`
  in the prepared root). The port writes the 26.3 form.
- **Lang**: `item.chisel.<style>.<material>` and `block.chisel.<style>.<material>` (dots),
  `gui.chisel.chisel`, `item.chisel.chisel`, `itemGroup.chisel.chisel`,
  `subtitles.chisel.chisel_sound`.
- **Sound**: `sounds.json` names `chisel:chisel_sound` (`assets/chisel/sounds/chisel_sound.ogg`).
- **Creative tab**: `ItemGroupRegistry.CLAY_GROUP` — one tab, id `chisel:chisel`, title
  `itemGroup.chisel.chisel`, icon the chisel item (`ItemGroupRegistry.java:14-19`).

## 7. Upstream defects found in the artefact (fixed in the port)

**D1 — the display names can never resolve.** The lang file spells every block and item name
with **dots** where the id has a **slash**:

```
assets/chisel/lang/en_us.json:  "block.chisel.array.andesite": "Array Andesite"
GeneratedRegister.java:24:      Reg("andesite", "array/andesite")   →  chisel:array/andesite
```

A block's name key is derived from its id (`Util.createTranslationKey("block", id)`, and at
this pin `format!("block.{}", name.replace(':', "."))`, `game/commands.rs:1360`,
`game/lan_actions.rs:1377`), so the key the game asks for is
`block.chisel.array/andesite` — which the jar never declares. Every Chisel block and item
therefore shows its raw key. The port ships **both** spellings: upstream's dotted key
(unchanged, for anything that already reads it) and the id-derived slash key the game
actually looks up. User-visible effect: items are called "Array Andesite" instead of
`block.chisel.array/andesite`.

**D2 — the chisel picks a random member, which can be the block itself.** `canMine` uses
`items.get(world.random.nextInt(items.size()))`, so one conversion in 34 is a no-op that
still plays the sound. Documented as a **deviation**, not silently "fixed": slice 1 walks
the group in order (see `notes.md`).

## 8. What the port cannot express at this pin (gaps, not fakes)

- The **GUI** (`ChiselScreenHandler`, a bundle-backed inventory) — a machines-tier menu +
  screen; not in slice 1.
- The chisel's **bundle contents** (choosing the exact variant to write) — the 26.3 item is
  declared with `max_stack_size` 1 and no bundle; slice 1 converts to the next variant.
- The **mod sound** `chisel:chisel_sound`: `native_play_sound` (API 25) plays registered
  **vanilla** sound events only, so the conversion is silent.
- **CTM** (Athena/OptiFine connected textures, 4,796 `optifine/` files + 4,795 `ctm/`
  sprites): the pin has no CTM loader; the port strips the `athena:loader`/`ctm_textures`
  keys from the blockstates and the blocks draw their plain sprite (the 27 CTM sprites the
  weighted variants name *are* carried, because those models reference them directly).
- The other **1,376 blocks** (98 materials × their styles), the `c:glass_blocks` tag, the
  `ru_ru`/`zh_cn` translations, `mineable/axe`/`shovel`, and the advancements.

## 9. Licence

`GPLv2` per `META-INF/neoforge.mods.toml`; `LICENSE` in the upstream repo at the ported tag
is the GPL-2.0 text. Rank 1 in the campaign's ordering (publishable under the same licence
with the complete source). `LICENSE` + `LICENSE-NOTICE.md` travel inside the `.hnmod`.
