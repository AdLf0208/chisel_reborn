# Chisel — validation

> **Status 2026-11-18: the 0.1.0 `.hnmod` evidence below is historical.** The port is now a
> flat SDK package (mod API 0.2, version 0.2.0) on SDK `0b14f8acfd97fa2c750e42795249dcc981178ca4`
> / mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`. `src/runtime.rs` is byte-identical, so
> the rule and its 7 tests below still hold; the glue is new (`src/lib.rs`, 5 tests) and the
> 0.2.0 ladder plus the missing in-game run are recorded in
> `notes.md` → *Migration to the SDK format*. The 0.1.0 commands (`tools/horizon.py`,
> `mixins/chisel/`, `content/`, the embedded-content legs) no longer exist in the port; the
> retired tree is `ports/_shared/evidence/chisel/retired-hnmod-0.1.0/`.

Every 0.1.0 command below ran through the campaign's build registry
(`ports/tools/build-slot.sh`), with `CARGO_INCREMENTAL=0` and no `--target-dir`.
Pin: Horizon `1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`, horizon-mixins
`5a64af938ec86b00d53d16a408aabc0038fb3e57`; `game.rs` SHA-256
`64813cf4db0b9c3983d42908609bb633b9fbbc705f0c890396f940a6c7cef80f` (`compatibility.json`).

## 1. The pure runtime (red → green)

`cargo test --locked --offline` in the port, before `next_variant` was implemented:

```
test result: FAILED. 3 passed; 4 failed
  next_of_vanilla_andesite_is_polished_andesite      → left: None  right: Some("minecraft:polished_andesite")
  the_first_chisel_style_follows_the_vanilla_pair    → left: None  right: Some("chisel:array/andesite")
  the_last_member_wraps_to_the_first                 → left: None  right: Some("minecraft:andesite")
  walking_the_group_visits_every_member_and_returns  → "every member has a successor"
```

After the implementation:

```
running 7 tests
test tests::a_block_outside_the_family_is_refused ... ok
test tests::every_member_appears_once ... ok
test tests::the_first_chisel_style_follows_the_vanilla_pair ... ok
test tests::the_group_is_the_upstream_registration_order ... ok
test tests::the_last_member_wraps_to_the_first ... ok
test tests::vanilla_andesite_converts_to_its_polished_form ... ok
test tests::walking_the_group_visits_every_member_and_returns ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured
```

## 2. Format, lints, the staged game

| step | command | result |
|---|---|---|
| fmt | `cargo fmt --check` | clean |
| clippy | `cargo clippy --all-targets --locked --offline -- -D warnings` | `Finished` with **0 warnings** |
| stage | `python3 tools/horizon.py stage` | `stage_changed: true`, `staging_seconds 93` |
| check | `python3 tools/horizon.py check` | `cargo_exit: 0`, 0 warnings (`ports/_shared/evidence/chisel/ladder-check.log`) |
| embedded tests | `python3 tools/horizon.py test` | `test result: ok. 7 passed; 0 failed; 1240 filtered out`, 0 warnings (`ladder-test.log`) |
| build | `python3 tools/horizon.py build` | `cargo_exit: 0`, `ctl_exit: 0`, `cargo_seconds 985` |

## 3. Packages

```
$ cargo horizon-mod build            # in content/
Built ./target/horizon-mod/chisel.hmod
$ cargo horizon-mod check content/chisel.hmod
chisel 0.1.0 (mod, API 0.2, both)
  92069 bytes, SHA-256 1b0b4436506066b35c074633d6258c11fd1e2c35308578620bc3d91e27fb5c65, unsigned
  may add blocks, items, mob effects and entity types to your worlds
  no code: content only, nothing to run
     288 files
$ cargo horizon-mod check chisel.hnmod --kind native-mod
chisel 0.1.0 (native-mod, format native-1/1d63e0a5…/5a64af93…, for 26.3)
  50712 bytes, SHA-256 4a872f764fbb7b001512af560606a4be3445d6af942a89459ac0a047b44a6eeb, unsigned
```

`chisel.hnmod` = 9 entries: `horizon-native.json`, `mixins/chisel/{LICENSE,NOTICE.md,
horizon-mixin.toml,runtime.rs,support.rs,click.rs,embedded_content.rs}`,
`mixins/chisel/data/chisel-content.hmod` (byte-identical to `content/chisel.hmod`).

## 4. In-game e2e — three legs, one build lease (0.1.0, historical)

> **These legs are 0.1.0 evidence.** They launch the retired `.hnmod` with its embedded content
> and its `.horizon-cache` control, none of which exists in the 0.2.0 package. The 0.2.0 script
> keeps the same phases, the counterfactual and the selftest, replaces the embedded-content
> control with an optional `CONTROL_GAME` (a game built without the package), and needs a game
> the launcher built with the package. **It has not been run**: no launcher and no game build at
> `0b14f8a` in this environment (`notes.md`, gap 9).

`ports/tools/build-slot.sh e2e --job chisel-e2e-all --port ports/chisel -- tools/e2e-all.sh`
runs the scenario three times; `tools/e2e.sh` is the scenario and `tools/e2e-all.sh` the
driver. Logs: `ports/_shared/evidence/chisel/e2e-{embedded,counterfactual,selftest}.log`.

Every leg launches a **survival** world (`ChiselE2E`, seed 20261112, `--headless`) with
**only the `.hnmod`** and no `--dev-mod`, builds a stone platform at y=99, teleports the
player onto it and runs:

| phase | assertion |
|---|---|
| 0 | all **32** carried variants place and read back (`setblock` + `block` round trip each) |
| 0 | `chisel:pillar/andesite[axis=x]` and `chisel:twist/andesite[axis=z]` keep their axis; a pillar with no properties is `axis=y` |
| 1 | a dig with `chisel:chisel` turns `chisel:array/andesite` into `chisel:braid/andesite`, mines nothing and leaves no chisel block item anywhere (held + on the ground) |
| 2 | the group's edges: `chisel:zag/andesite` → `minecraft:andesite` (wrap), `minecraft:andesite` → `minecraft:polished_andesite`, `minecraft:polished_andesite` → `chisel:array/andesite` |
| 3 | `minecraft:dirt` with the chisel stays dirt, no item appears (the family refuses it) |
| 4 | `chisel:panel/andesite` → `chisel:pillar/andesite[axis=y]` (the target is written as its default state) |
| control | with `.horizon-cache` made unusable, the same binary knows **no** `chisel:array/andesite` — so the content really comes from the embedded file |
| embedded | the file the bundle wrote at startup is byte-identical to `content/chisel.hmod` (92,069 B) |
| counterfactual | the same single break with `minecraft:stone_pickaxe` mines the block (`minecraft:air`) and leaves exactly **1** `chisel:array/andesite` (held or on the ground) — the carried loot table, and the proof the chisel leg is not "nothing ever happens" |
| selftest | the conversion assertion flipped to the block that was already there, which must **FAIL** |

### Results (2026-11-12)

```
$ python3 ports/tools/check-e2e-logs.py chisel --strict
ok       chisel                 e2e-counterfactual.log     ? + EXIT:0
ok       chisel                 e2e-embedded.log           PASS + EXIT:0
ok       chisel                 e2e-selftest.log           FAIL + EXIT:1
logs=3  ok=3  missing-exit=0  bad-exit=0
```

The chisel leg (`e2e-embedded.log`), with the control leg first and the embedded-content
byte check after it:

```
== control: the same game with no embedded content file ==
  without the embedded file: no chisel:array/andesite could be given
== launch (survival world ChiselE2E, one .hnmod, no --dev-mod) ==
  embedded content: 92069 B at …/.horizon-cache/chisel-embedded-content.hmod, identical to content/chisel.hmod
== phase 0: every carried variant places and reads back ==
  32 variants placed and read back
  chisel:pillar/andesite[axis=x] -> chisel:pillar/andesite[axis=x]
  chisel:twist/andesite[axis=z] -> chisel:twist/andesite[axis=z]
  chisel:pillar/andesite (no properties) -> chisel:pillar/andesite[axis=y]
== phase 1: the chisel converts chisel:array/andesite to the next member ==
  after the mining attempt: chisel:braid/andesite (chisel block items 0 -> 0)
== phase 2: the group's edges (last wraps, the vanilla pair is in the group) ==
  chisel:zag/andesite -> minecraft:andesite
  minecraft:andesite -> minecraft:polished_andesite
  minecraft:polished_andesite -> chisel:array/andesite
== phase 3: a block outside the family is refused ==
  minecraft:dirt with the chisel: minecraft:dirt (chisel block items 0 -> 0)
== phase 4: the target is written as its default state (axis=y) ==
  chisel:panel/andesite -> chisel:pillar/andesite chisel:pillar/andesite[axis=y]
PASS: …
EXIT:0
```

The counterfactual (`e2e-counterfactual.log`):

```
== counterfactual: the same break with a stone pickaxe mines the block ==
  after the mining attempt: minecraft:air (chisel block items 0 -> 1)
PASS (counterfactual, stone pickaxe): the block was mined and dropped one of itself
EXIT:0
```

The selftest (`e2e-selftest.log`), which must fail:

```
(selftest: asserting the chisel leaves chisel:array/andesite in place)
  after the mining attempt: chisel:braid/andesite (chisel block items 0 -> 0)
FAIL: the chisel left chisel:braid/andesite, expected chisel:array/andesite
EXIT:1
```

**After the e2e**, `cargo fmt` re-wrapped three lines of `runtime.rs` (whitespace only: no
token changes) and `chisel.hnmod` was rebuilt to 50,712 B / SHA-256 `4a872f76…a6eeb`. The
staged `stage → check → test` were re-run on the delivered sources: `cargo_exit 0`,
`game::chisel::tests` **7 passed**, 0 warnings.

### Measurements that shaped the harness (kept here so nobody re-learns them)

- **A control write lands on the next tick.** `ctl setblock` replies `ok` and a `ctl block`
  straight after it reads the *previous* state: phase 0's round trip needs `ctl wait 1`
  between the write and the read. Without it every variant "read back as `minecraft:air`".
- **A held dig converts more than once.** The client restarts a dig whenever the target
  block changes, and each restart is one more conversion (upstream behaves the same way:
  `canMine` runs again on the new block). Measured on the same block, from
  `chisel:array/andesite`: press and release in one frame → **1** step (braid); a one-tick
  hold → 3; a two-tick hold → 2. The harness therefore sends `mouse left down` and
  `mouse left up` with **no wait between them, inside one `ctl script`** (a socket round
  trip between them already costs a tick), which gives exactly one conversion.
- **A drop can be picked up at two blocks.** The counterfactual's mined block was first
  reported as dropping nothing; the item was in the player's inventory (measured:
  `inventory: {"chisel:array/andesite":1}` beside `items near: [...]`). The harness counts
  held **and** ground chisel block items together.
- **`destroy_block` (API 14) was the wrong hook.** It fires after a break *completes*, so a
  single press converted 3–4 members; `left_click_block` (API 30) fires where the dig
  starts — upstream's `canMine` moment — and cancels the dig, so one press is one
  conversion.
- **`ctl command "setblock … chisel:array/andesite"` fails** ("Could not set the block")
  while `ctl setblock`, `/loot spawn chisel:blocks/array/andesite`, placement, read-back,
  conversion and drops all work: the pin's command parser, recorded in `notes.md`.
