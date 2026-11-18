# Chisel (Chisel Reborn) — port notes

Ported from **Chisel Reborn 2.0.1+mc1.21.1** for Minecraft Java 26.3 on Horizon
**`1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`** / horizon-mixins
**`5a64af938ec86b00d53d16a408aabc0038fb3e57`**, and **migrated 2026-11-18 to the SDK
package format** (`api = "0.2"`, version **0.2.0**) on SDK nightly
**`0b14f8acfd97fa2c750e42795249dcc981178ca4`** / horizon-mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`** — see *Migration to the SDK format*.
The jar is
`ports/_shared/licence-cache/chisel-neoforge-2.0.1+mc1.21.1.jar`; the behaviour was read
from the upstream repository at tag `1.21.1-2.0.1`
(`b399d0f218acae743321d64604fb378c6c7b791c`) and from the jar's own data.

- **Class:** `content` — **tier:** `machines` in the pack's row, **ported here as a `data`
  carry** with one `logic` behaviour (the chisel tool). The tier's machines half (the chisel
  GUI: a menu, a screen and the bundle inventory) is *not* in slice 1 and is listed as a
  gap; the blocks themselves are exactly the data tier's shape.
- **Licence:** **GPL-2.0-only** (the jar's `neoforge.mods.toml` says `GPLv2`; the upstream
  tree at the ported tag carries no `LICENSE` file and the jar carries no licence text, so
  the port ships the canonical GPL-2.0 text, SHA-256 `edaef632…33f6`). **Rank 1** in the
  campaign's ordering: publishable under the same licence with the complete corresponding
  source. `LICENSE` and `LICENSE-NOTICE.md` travel inside the package (0.1.0 carried them
  at `mixins/chisel/`; 0.2.0 at the package root).

## The slice, and why this one

Chisel Reborn registers **1,409 blocks** whose registry names are
`chisel:<style>/<material>` (`GeneratedRegister.Register` + `EzReg.Reg`), i.e. 100 material
*families* × their styles. 19,369 jar entries is not one slice.

Slice 1 carries **one family, `andesite`, with all 32 of its variants**, plus the chisel
itself: the smallest family that still exercises the whole content path (30 plain full
cubes, the two `axis` variants `pillar`/`twist`, three weighted-model variants —
`chaotic_bricks` 9 models, `cut` 16, `french` 2 — item definitions, loot, lang, a recipe,
tags and a creative tab). Every variant of the family is shipped: nothing is left out of
the family.

Inventory and provenance: [`FINDINGS.md`](FINDINGS.md). Content generator:
[`tools/gen_content.py`](tools/gen_content.py) (fails loudly if the jar lacks an input).

## Behaviour, from upstream

- **The conversion** is `ChiselItem.canMine` (`common/src/main/java/com/periut/chisel/item/
  ChiselItem.java:126-171`): with a chisel in hand a **mining attempt** looks up the block's
  chisel *group*, writes a group member's `getDefaultState()` over the block, plays
  `chisel:chisel_sound` and returns `false` — the block is never mined. An empty group does
  nothing and still returns `false`, so a chisel never mines anything.
- **The group** for `andesite` is 34 members: `VanillaChisel.Register` puts
  `minecraft:andesite`, `minecraft:polished_andesite` first, then
  `GeneratedRegister.Register` adds the 32 `chisel:<style>/andesite` in file order
  (duplicates dropped keeping the first). The port's `runtime.rs` holds that table.
- **The block class** is `EzReg.Reg`'s choice: `PillarBlock` (an `axis` property, default
  `y`) when the style name holds `pillar` or `twist`, otherwise a plain `Block`, both with
  `AbstractBlock.Settings.copy(Blocks.ANDESITE)` — which is exactly the content
  declaration `{"template": "minecraft:andesite"}` plus, for those two, a declared `axis`.
- **The item** is `maxCount(1)`, a `BundleItem` with no bar, `getMiningSpeed` 500 for every
  block, and one shaped recipe (iron ingot + stick).

## The port

**This table is the retired 0.1.0 `.hnmod` layout.** The delivered 0.2.0 package is one flat
SDK `.hmod`; its file map and helper→hook table are in *Migration to the SDK format* below.
The behaviour description here is unchanged — `src/runtime.rs` is byte-identical.

| half | file | what |
|---|---|---|
| plan | `mixins/chisel/horizon-mixin.toml` | 2 modules + 2 injections (content root, `left_click_block`) |
| runtime | `mixins/chisel/runtime.rs` | the 34-member family and `next_variant`, pure, 7 tests |
| glue | `mixins/chisel/support.rs` | the held chisel, the block's name, the state write |
| helper | `mixins/chisel/click.rs` | one private fn at `native_left_click_block` (API 30) |
| content | `mixins/chisel/embedded_content.rs` | writes `data/chisel-content.hmod` to `.horizon-cache` and appends it to the dev list (AE2 pattern) |
| content | `content/` → `content/chisel.hmod` | 288 files, embedded at `mixins/chisel/data/chisel-content.hmod` |

**Hook choice.** Upstream converts where a dig starts (`canMine`). The pin hears that as
`native_left_click_block` (API 30, "where `ServerPlayerGameMode.handleBlockBreakAction`
takes a dig's start"), which may cancel the click so the block "isn't attacked, dug or
broken on the server". The helper cancels and writes the next member through
`native_set_block` (API 24) — one conversion per dig, and the block keeps its drops because
it is never removed. `destroy_block` (API 14) was measured first and rejected: it fires
after a break *completes*, and with a fast tool the client restarts a dig on every block
change, so a single press converted 3–4 members (`ports/_shared/evidence/chisel/` has the
final logs; the measurement is recorded below).

**Default state, not the old state.** Upstream writes `getDefaultState()`, so a conversion
*into* `pillar`/`twist` always lands on `axis=y`. The e2e asserts that
(`chisel:panel/andesite` → `chisel:pillar/andesite[axis=y]`).

## Verification

### 0.1.0, the retired `.hnmod` (historical — the sources `src/runtime.rs` came from)

Ladder (all through `ports/tools/build-slot.sh`, `CARGO_INCREMENTAL=0`, no `--target-dir`):

| step | command | result |
|---|---|---|
| runtime tests | `cargo test --locked --offline` | **7 passed, 0 failed** |
| fmt | `cargo fmt --check` | clean |
| clippy | `cargo clippy --all-targets --locked --offline -- -D warnings` | **0 warnings** |
| stage | `python3 tools/horizon.py stage` | `stage_changed: true`, pin `1d63e0a` |
| check | `python3 tools/horizon.py check` | `cargo_exit 0` |
| embedded tests | `python3 tools/horizon.py test` | `game::chisel::tests` **7 passed, 0 failed**, 0 warnings |
| build | `python3 tools/horizon.py build` | `cargo_exit 0`, `ctl_exit 0` |
| content package | `cargo horizon-mod build` in `content/` | 288 files, `no code: content only` |
| native package | `cargo horizon-mod check chisel.hnmod --kind native-mod` | ok, format `native-1/1d63e0a/5a64af9` |

Deliverables of 0.1.0 (now in `ports/_shared/evidence/chisel/retired-hnmod-0.1.0/`):

- `chisel.hnmod` — **50,712 B**, SHA-256
  `4a872f764fbb7b001512af560606a4be3445d6af942a89459ac0a047b44a6eeb`, 9 entries
  (`horizon-native.json`, `mixins/chisel/{LICENSE,NOTICE.md,horizon-mixin.toml,*.rs}`,
  `mixins/chisel/data/chisel-content.hmod`).
- `content/chisel.hmod` — **92,069 B**, SHA-256
  `1b0b4436506066b35c074633d6258c11fd1e2c35308578620bc3d91e27fb5c65`, 288 files
  (287 carried files + its own `horizon-mod.toml`).

In-game e2e of 0.1.0 (`tools/e2e.sh`, three legs driven by `tools/e2e-all.sh`; logs in
`ports/_shared/evidence/chisel/`): see [`VALIDATION.md`](VALIDATION.md) §4. The three legs
are `PASS` + `EXIT:0` (chisel, counterfactual), `FAIL` + `EXIT:1` (selftest), and
`check-e2e-logs.py chisel --strict` is green (3/3). **Those logs are historical**: they ran
the retired bundle with the retired embedded-content legs, and the 0.2.0 package has not been
run in game (no launcher here — see *Gaps after the migration*).

One packaging note: the e2e ran before the last `cargo fmt` (which re-wrapped three lines of
`runtime.rs` — whitespace only, no token change), so `chisel.hnmod` was rebuilt afterwards
and the staged `check` + `test` were re-run on the delivered bytes (green, 7/7).

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

## Gaps (slice boundary)

1. **1,377 of the mod's 1,409 blocks** — the other 99 material families — are not carried.
   Only the `andesite` family is.
2. **The chisel GUI** (`ChiselScreenHandler`, a bundle-backed inventory screen, and
   `ChiselItem.createMenu`) and the chisel's **bundle contents** (put a block in the chisel
   to choose the exact variant, `ChiselItem.canMine`'s second branch) are machines-tier
   work: a menu + screen + a component-carrying inventory. Slice 1 has neither; the item is
   declared `max_stack_size` 1 with no bundle component.
3. **The chisel's sound** (`chisel:chisel_sound`, `assets/chisel/sounds.json` +
   `chisel_sound.ogg`) is a **mod** sound event, and `native_play_sound` (API 25) plays
   registered **vanilla** sound events only, so a conversion is silent. The content does not
   ship the sound file either (nothing could play it).
4. **CTM.** 4,796 `assets/chisel/optifine/` files and 4,795 `textures/block/ctm/` sprites
   are the Athena/OptiFine connected-texture data; the pin has no CTM loader. The port
   strips the `athena:loader`/`ctm_textures`/`height`/`width` keys from the 32 blockstates
   and the blocks draw their plain sprite. The 27 CTM sprites the three weighted-model
   variants name *are* carried, because those models reference them directly.
5. **`getMiningSpeed` 500** is declared as the item's `minecraft:tool` component
   (`default_mining_speed` 500, one rule over `#minecraft:mineable/pickaxe`), which is what
   the pin reads for mining speed (`horizon-items/src/items.rs:1135`). It is upstream's
   value, but it is inert in this port: the dig is cancelled before any break, so a player
   never sees the fast mining. Per-stack tool stats remain impossible at the pin
   (`ARCHITECTURE.md` §3), and nothing here needs them.
6. **Translations**: only `en_us` is carried (upstream also ships `ru_ru`, `zh_cn`).
7. **Tags**: only `minecraft:mineable/pickaxe` (upstream also adds `mineable/axe`,
   `mineable/shovel`, `needs_stone_tool`, `needs_iron_tool` and `c:glass_blocks`; none of
   those families are in slice 1). No advancements (upstream has none for these blocks).
8. **The creative tab** carries the chisel and the 32 variants only, not the other 1,377
   blocks upstream puts in it.

## Deviations (deliberate, not defects)

1. **Next member, not a random one.** Upstream picks `items.get(world.random.nextInt(
   items.size()))` — a random group member, which one time in 34 is the block itself (a
   no-op that still plays the sound). Slice 1 walks the group in order (`runtime.rs`), as
   the slice's acceptance requires ("the next variant in its family") and so the conversion
   is predictable and testable. A player notices that repeated digs cycle the family in a
   fixed order instead of jumping.
2. **The dig is refused, the conversion is not a break.** Upstream returns `false` from
   `canMine` for every block, so the chisel never mines anything; the port cancels at
   `left_click_block`, which is the pin's equivalent (the block "isn't attacked, dug or
   broken on the server"). A cancelled click leaves the client's predicted crack for the
   clicking player, which upstream's client-side `canMine` also does.
3. **The recipe's ingredient form** is 26.3's plain id string (`"I": "minecraft:iron_ingot"`)
   rather than 1.21.1's `{"item": …}` object; the pattern, category and result are
   upstream's.

## Upstream defects fixed

**D1 — every block and item name could never resolve.** The jar's lang spells names with
**dots** where the id has a **slash**:

```
assets/chisel/lang/en_us.json:   "block.chisel.array.andesite": "Array Andesite"
GeneratedRegister.java:24:       Reg("andesite", "array/andesite")  →  chisel:array/andesite
```

A block's name key comes from its id — `block.<namespace>.<path>` (at this pin
`format!("block.{}", name.replace(':', "."))`, `game/commands.rs:1360`) — so the game asks
for `block.chisel.array/andesite`, which the jar never declares, and every Chisel block and
item shows its raw key. **The port ships both spellings**: upstream's dotted keys unchanged
(for anything that already reads them) *and* the slash keys the game derives. What a user
notices: the items are called "Array Andesite", "Twisted Andesite" and so on instead of
`block.chisel.array/andesite`. Proved in game — the e2e's `/loot spawn` reply reads
`Dropped 1 [Array Andesite]` and the inventory shows the carried item by name.

Worth an upstream report: yes (a one-line lang generator fix), not filed — the campaign
files reports only when the orchestrator asks.

**Not a defect, recorded as a pin limitation:** the game's own `/setblock` cannot set a
block whose id holds a `/` (measured: `ctl command "setblock … chisel:array/andesite"`
answers "Could not set the block" while the control client's `setblock` and `/loot spawn
chisel:blocks/array/andesite` both work, and the block places, reads back, converts and
drops normally). This is the pin's command parser, not the mod's data; the port keeps the
upstream ids rather than flattening them.

## Files

0.2.0 (the delivered layout):

- `horizon-mod.toml`, `Cargo.toml` — the manifest and the crate.
- `src/lib.rs` — the one `impl mod_api::Hooks` and the pure `dig` decision.
- `src/runtime.rs` — the family table and `next_variant`, **byte-identical** to 0.1.0.
- `assets/`, `data/` — the carried content (287 files, byte-identical to the retired package).
- `chisel.hmod` — the one deliverable.
- `LICENSE`, `LICENSE-NOTICE.md` — the licence and the derivative notice.
- `FINDINGS.md`, `VALIDATION.md`, `README.md` — the inventory, the ladder, the player-facing text.
- `tools/gen_content.py`, `tools/e2e.sh`, `tools/e2e-all.sh`.
- `ports/_shared/evidence/chisel/retired-hnmod-0.1.0/` — the retired tree (`mixins/`, the
  `.hnmod`, the built content package and its manifest, `compatibility.json`,
  `rust-toolchain.toml`, `tools/horizon.py`, `tools/package.py`).

## Migration to the SDK format (0.2.0, 2026-11-18)

Horizon's mod-API merge (#16941, launcher ≥ 0.2.70, PR #17992) retired `.hnmod`. The registry
closed this port's former submission, **PR #897** (`chisel` 0.1.0), with the format purge, so
**0.2.0** is the same behaviour republished as one flat SDK `.hmod`. Pins: SDK nightly
**`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (2026-11-18, `package_format` 6, `content_api`
"0.2"), horizon-mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, mod API `0.2`. The SDK
is cached in-tree at `ports/_shared/sdk/0b14f8a/` and fetched into `.horizon/sdk/` (gitignored,
21 MB) by the build; it is never packed.

**Done in this order.** The retired tree was moved to
`ports/_shared/evidence/chisel/retired-hnmod-0.1.0/` **before** the first
`cargo horizon-mod build` (the tool refuses a project root holding retired package paths); then
`src/runtime.rs` was copied with `cp` and proved with `cmp` + `sha256sum`; `cargo test` ran on
the new crate **before any glue existed** (7/7); the glue tests were written against a
`todo!()` and watched fail (5 red); only then was the hook implemented (12/12 green).

### Where the retired tree is, and the file map

| 0.1.0 path | 0.2.0 | Evidence |
|---|---|---|
| `mixins/chisel/runtime.rs` | `src/runtime.rs`, **copied**, `cmp` + sha256 identical (`ea0835fe…a20e`), and identical inside the package | `retired-hnmod-0.1.0/mixins/chisel/runtime.rs` |
| `mixins/chisel/support.rs` (the conversion glue) | `src/lib.rs`: `Hooks::left_click_block` + the pure `dig()` decision (5 tests) | `retired-hnmod-0.1.0/mixins/chisel/` |
| `mixins/chisel/click.rs` (one helper, `native_left_click_block`) | the same `Hooks::left_click_block` body — no per-hook file | `retired-hnmod-0.1.0/mixins/chisel/click.rs` |
| `mixins/chisel/horizon-mixin.toml` (2 `[[module]]`, 2 `[[injection]]`) | no plan: one typed `Hooks` method | `retired-hnmod-0.1.0/mixins/chisel/horizon-mixin.toml` |
| `mixins/chisel/embedded_content.rs` + `data/chisel-content.hmod` (AE2 pattern) | **gone**: the flat package carries `assets/` and `data/` itself | `retired-hnmod-0.1.0/mixins/chisel/` |
| `content/assets/`, `content/data/` (287 files) | `assets/chisel/`, `data/chisel/`, `data/minecraft/` — **every file byte-identical** (size + sha256 diff of the unpacked 0.1.0 package against the new root: 0 differences; `tools/gen_content.py`, updated to write the root, reproduces the same 287 files) | `retired-hnmod-0.1.0/content-package-0.1.0/` |
| `content/horizon-mod.toml` | merged into the root `horizon-mod.toml` (`id`/`name`/`api`/`side`/`capabilities`, `version` 0.1.0 → 0.2.0) | `retired-hnmod-0.1.0/content/horizon-mod.toml` |
| `content/chisel.hmod` (92,069 B content-only package) | not needed: content is inside the one package | `retired-hnmod-0.1.0/content/chisel-content-0.1.0.hmod` |
| `chisel.hnmod` (50,712 B, embedded content) | `chisel.hmod` (109,579 B, native + content) | `retired-hnmod-0.1.0/chisel.hnmod` |
| `compatibility.json`, `rust-toolchain.toml` | `[native]` in `horizon-mod.toml`; the pinned commit decides the toolchain | `retired-hnmod-0.1.0/` |
| `tools/horizon.py`, `tools/package.py` | deleted: staging and `.hnmod` packaging belong to the retired format | `retired-hnmod-0.1.0/` |
| `Cargo.lock` | not kept: not needed without registry deps and it changes the artifact (trap 5) | not kept |

**Kept and updated:** `tools/e2e.sh` (now requires a launcher-built game, with an optional
`CONTROL_GAME` differential; the embedded-content legs are gone), `tools/e2e-all.sh`,
`tools/gen_content.py`, `notes.md`, `FINDINGS.md`, `VALIDATION.md`, `README.md` (rewritten: it is
packed inside the `.hmod`), `LICENSE`, `LICENSE-NOTICE.md`.

### Helper → hook table (every `[[injection]]` and `[[module]]` mapped)

| 0.1.0 | 0.2.0 | Mapping notes |
|---|---|---|
| `[[module]] runtime` (`crate::game::chisel`) | `src/runtime.rs` as an ordinary module (`pub mod runtime;`) | `ANDESITE_FAMILY`, `next_variant` and the 7 tests unchanged |
| `[[module]] state` (`chisel_state`, `support.rs`) | the `dig()` decision in `src/lib.rs` + 5 glue tests | `Dig::{Allow, Convert, Refuse}`; the host I/O stays in the hook body |
| `[[injection]] embedded-content` (`content_root` HEAD, cancellable) | **none** — the SDK `.hmod` loads its own `assets/`+`data/` | the one gap this removes is the cache write and its `log::warn!` |
| `[[injection]] left-click` (`left_click_block` v1, HEAD, `click.rs`) | `Hooks::left_click_block` returning `Flow::Continue`/`Flow::Cancel` | the SDK method is the same stamped hook, typed |
| `game.held()` | `host.players().acting_player()` → `host.living(player.entity())` → `living.hand_item(&*host, Hand::Main)` | the three handle hops are the SDK's acting-player shape (`transfer_labels` uses it too) |
| `game.pack.assets.registry.state(state).name` | `event.state.block().id(&*host)` | one read instead of a registry walk |
| `game.pack.assets.registry.default_state(next)` | `host.block_key(next)` → `host.block_state_default(key)` | the same default state, so `pillar`/`twist` still land on `axis=y` |
| `game.native_set_block(dim, pos, state, UPDATE_ALL)` | `level.set_block(&mut *host, pos, state, BlockUpdateFlags::NEIGHBORS \| CLIENTS)` with `level = player.level(&*host)` | `UPDATE_ALL` is exactly those two bits (`buildinggadgets2` uses the same) |
| `log::warn!("… has no block state")` | dropped | the SDK exports no logger and a native mod may not add a crate (gap 10) |

### Behaviour changes caused by the mapping

1. **None in the rule.** The group, the order, the wrap and the default-state write are the
   same code (`runtime.rs` is byte-identical) and the same write.
2. **A missing content half is silent.** 0.1.0 logged "Chisel has no block state for {id}"; the
   SDK has no logger, so the port refuses the dig and changes nothing, silently (gap 10).
3. **The dig is heard from the same moment as before** (`left_click_block`, the pin's
   `ServerPlayerGameMode.handleBlockBreakAction`), not from `destroy_block`; that choice was
   measured in 0.1.0 and is unchanged.

### The ladder (measured 2026-11-18)

Every compile ran in a `ports/tools/build-slot.sh` lease with `BUILD_LOCAL=1` (see trap 5),
`CARGO_INCREMENTAL=0`, no `--target-dir`, and the `cd` inside the wrapped command.

| Step | Command | Result | Evidence |
|---|---|---|---|
| runtime only, no glue | `cargo test --offline` | **7 passed**, 0 failed | `sdk-test-runtime-only.log` |
| glue red (trap 21) | `cargo test --offline` with `dig` = `todo!()` | 7 passed, **5 failed** (`not yet implemented: the conversion decision`) | `sdk-test-glue-red.log` |
| tests | `cargo test --offline` | **12 passed**, 0 failed (7 `runtime::`, 5 `tests::`) | `sdk-test.log` |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings | `sdk-clippy.log` |
| format | `cargo fmt --check` (**not** `--all`) | exit 0, empty diff | `sdk-fmt.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | `SDK 0b14f8a … in ./.horizon/sdk` / `Built ./target/horizon-mod/chisel.hmod` | `sdk-build.log` |
| registry check | `cargo horizon-mod check chisel.hmod` | see the verbatim output below, **EXIT 0** | `sdk-check.log` |
| fits | `cargo horizon-mod fits chisel.hmod --build nightly` | `Chisel Reborn:` / `0b14f8a  made for`, **EXIT 0** | `sdk-check.log` |
| second build | `rm -f Cargo.lock && cargo horizon-mod build` | identical bytes (`cmp`), same sha256 | `sdk-build-second.log`, `sdk-sha256.log` |
| runtime untouched | `unzip -p chisel.hmod src/runtime.rs \| sha256sum` | `ea0835fe…a20e` = the retired file | `sdk-sha256.log` |

```
$ cargo horizon-mod check chisel.hmod
chisel 0.2.0 (native-mod, format native-1/0b14f8acfd97fa2c750e42795249dcc981178ca4/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)
  109579 bytes, SHA-256 36a2b520881ff80f1fd067da22e44a875b45e3cee2b0a8c63c096426f0157d42, unsigned
  native source: unsandboxed; not executed by the registry check. Compile and test against the exact pinned Horizon and Mixins revisions before review.

$ cargo horizon-mod fits chisel.hmod --build nightly
Chisel Reborn:
0b14f8a  made for
```

### Package

`chisel.hmod` — **109,579 B**, SHA-256
`36a2b520881ff80f1fd067da22e44a875b45e3cee2b0a8c63c096426f0157d42`, **295 entries**:
`horizon-mod.toml`, `Cargo.toml`, `LICENSE`, `LICENSE-NOTICE.md`, `README.md`, `src/lib.rs`,
`src/runtime.rs`, `uses.json`, 219 `assets/` files and 68 `data/` files. Built **twice with
identical bytes** (reproducible). No `Cargo.lock`. The licence files travel **inside** the
package (the GPL's condition); `LICENSE` and `LICENSE-NOTICE.md` are at the root, where both
`cargo horizon-mod build` and a reader find them.

`uses.json` is 67 entries; the subscriptions this mod really makes are
`hook:left_click_block` (+ `.pos`, `.state`), `method:Hooks::left_click_block` and
`capability:registries`. The rest is the tool's over-approximation from names in the source
(`handle:LevelRef::block`, `handle:ItemStack::new`, `handle:EntityRef::push` and the like are
named by the API but not called here); `check` passes with them, so they are not a claim.

### Traps **new** to this port (the reference recipes' traps were already known)

1. **A key's `id(host)` is an owned `String`, not a `&str`.** `BlockKey::id` / `ItemKey::id`
   answer `String`, so a helper taking `&str` needs `&current` / `.as_deref()`; comparing with
   `== CHISEL` inside the hook would have hidden it.
2. **`host.player_level(..)` is declared but not reachable as a host method here.** The level is
   on the handle: `player.level(&*host)` (the compiler suggested `player`). Same shape as the
   `PlayerRef` inventory/level reads in the other migrations.
3. **The "unchanged content" diff needs the retired *package*, not the retired directory.**
   The strong evidence is `unzip` the 0.1.0 `content/chisel.hmod` into evidence and diff
   `find … -printf '%s %p'` + `sha256sum` against the new root: 287 files, 0 differences.
   Comparing against the old `content/` directory alone would have missed a stale zip.
4. **`check --json` reports `capabilities: []` and `side: null` for a native package** (the
   same fields `enderstorage` got). The packed `horizon-mod.toml` does carry
   `capabilities = ["registries"]` and `side = "both"` — do not "fix" the manifest to chase
   that report.
5. **Every step ran with `BUILD_LOCAL=1`.** The SDK is a 21 MB tree inside the port
   (`.horizon/sdk/`), and the slot runner's automatic forward to the far box would sync the
   port; the local lease is the supported path for an SDK package.
6. **`cargo test` needs `.horizon/sdk/` before the first `cargo horizon-mod` command.** It was
   copied from `ports/framedblocks/.horizon/sdk` (same pin, gitignored), so the whole ladder
   stayed offline.
7. **The 0.1.0 e2e's `EMBEDDED`/`CONTROL` legs died with the format.** They proved the bundle
   wrote its own content out; a flat package has no such step. The replacement differential is
   `CONTROL_GAME`: a game built **without** the package cannot `give` `chisel:array/andesite`,
   so the ids demonstrably come from the package. The selftest leg is unchanged.

### Gaps after the migration

Everything in *Gaps (slice boundary)* above still stands (1,377 blocks, the GUI, the sound, CTM,
mining speed, translations, tags, the creative tab). Added by this format:

9. **No in-game run of the 0.2.0 package.** The e2e needs a game the launcher built with this
   package (`horizon-launcher native-build … --trust-native-code`), and `horizon-launcher` is not
   installed in this environment — there is no game build at `0b14f8a` and no shared
   `horizon-ctl` for that pin (`ports/_shared/bin/` has `1d63e0a5` and `41f822b6` only). The
   compile-and-test gate (`cargo test` 12/12, clippy, `check`, `fits`, a reproducible package)
   is green; the *behaviour* evidence is the 0.1.0 run, whose rule is the same bytes and whose
   calls moved to their documented SDK routes. `tools/e2e.sh` is updated for the launcher-built
   game and refuses to run without one.
10. **No diagnostics.** The SDK exports no logger and a native mod may not add a crate, so the
   0.1.0 `log::warn!` on a missing block state is gone: a dug block whose next family member is
   not in the content is refused silently (it still never mines). A readout command would be
   new surface the mod does not have, so it was not added.
11. **`minecraft:andesite` and `minecraft:polished_andesite` are group members**, as upstream —
    so the chisel converts vanilla andesite too. That is not a gap, but it is why the family
    table has 34 entries for 32 carried blocks; a reader checking the count should know.

### Orchestrator actions (outside this job's write ownership)

- `ports/ledger.csv` row `chisel`: the jar's `modLicense = "GPLv2"` is read by
  `precheck-mod.py` as **`GPL`** (rank 1) and the tool asks the ledger to say that rather than
  `GPL-2.0-only`. The port keeps `GPL-2.0-only` in `Cargo.toml` (the canonical GPL-2.0 text it
  ships) and is **local-only** until the maintainer approves publishing (ARCHITECTURE.md §8).
- The ledger's `hooks` column says `left_click_block(30)`; that is the 0.1.0 API level and the
  hook is unchanged in 0.2.0, so no edit is strictly needed.

### Changelog (0.2.0)

> **0.2.0 — republished as an SDK `.hmod`.** Horizon's mod-API merge (launcher ≥ 0.2.70) retired
> `.hnmod` and closed **#897** (`chisel` 0.1.0); this version supersedes it. One flat
> `chisel.hmod` now carries the code *and* the content (the andesite family's 32 variants and
> the chisel): `src/lib.rs` is the one `impl mod_api::Hooks`, `src/runtime.rs` is byte-identical
> to 0.1.0, and `assets/`+`data/` are the 287 carried files unchanged. Built against SDK
> `0b14f8a` / mixins `e59da90` for 26.3. The `.hnmod`, the mixin plan, the two per-hook helpers
> and the embedded-content injection are gone; the conversion is
> `Hooks::left_click_block` written through `LevelRef::set_block` with the same default state.
> Same feature set and the same gaps (1,377 blocks, the bundle GUI, the sound, CTM).

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port
