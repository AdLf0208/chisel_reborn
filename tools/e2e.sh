#!/usr/bin/env bash
# In-game check for Chisel slice 1: the andesite family and the chisel.
#
# STATUS (2026-11-18): this scenario needs a game built by the launcher with this flat SDK
# package (`horizon-launcher native-build ports/chisel --instance <i> --trust-native-code`),
# because a native mod is compiled into `horizon-client`, plus the prepared 26.3 resources
# and `jq`. The migration to mod API 0.2 did NOT re-run it: the launcher is not installed in
# this environment, so there is no game build at `0b14f8a` (recorded as a gap in `notes.md`).
# The 0.1.0 runs remain the behaviour's evidence — the rule (`src/runtime.rs`) is
# byte-identical and every call moved to its documented SDK route; see
# `VALIDATION.md` §4 and `evidence/chisel/e2e*.log` (historical, retired format).
# The retired `.hnmod`'s embedded-content legs (`EMBEDDED`, the `.horizon-cache` checks and
# the "no embedded file" control) are gone: the flat package carries `assets/` and `data/`
# inside itself, so the ids exist exactly when the mod is installed.
#
# Upstream converts on a **mining attempt** (`ChiselItem.canMine`): with a chisel in hand
# the block is never mined, a member of its chisel group is written over it instead, and a
# block outside every group is left alone. The port hears that at `left_click_block`
# (cancel + a default-state write), so the scenario is a real mining attempt in survival,
# not a right click.
#
# Phases (all in The End, on a floor built here, so item drops are countable):
#
#   0. every carried variant places and reads back: 32 `setblock` + `block` round trips,
#      plus the two `axis` variants binding `axis=x`/`axis=z` and defaulting to `axis=y`.
#   0b. the blackstone family's one variant: the same round trip, its two axis bindings and
#      its `axis=y` default (a different base template, `minecraft:blackstone`).
#   1. `chisel:array/andesite` + chisel: becomes `chisel:braid/andesite` (the next member),
#      the block is NOT mined, and nothing drops.
#   2. the group's edges: `chisel:zag/andesite` (last) -> `minecraft:andesite`,
#      `minecraft:andesite` -> `minecraft:polished_andesite` (the vanilla pair the mod's
#      `VanillaChisel` puts first), `minecraft:polished_andesite` -> `chisel:array/andesite`.
#   3. a block outside the family is refused: `minecraft:dirt` stays dirt, nothing drops.
#   4. the target is written as its DEFAULT state: `chisel:panel/andesite` -> next is the
#      `pillar`, which lands with `axis=y` (a property-preserving write would leave `axis`
#      unset only if the source had it; upstream writes `getDefaultState()`).
#   5. the blackstone group, which is mostly vanilla: `minecraft:blackstone` ->
#      `minecraft:polished_blackstone`, `minecraft:cracked_polished_blackstone_bricks` ->
#      `chisel:pillar/blackstone`, `chisel:pillar/blackstone` -> `minecraft:blackstone`
#      (the wrap), then an andesite member again: a conversion never leaves its family.
#   6. the chisel's own shaped recipe is CRAFTED (parity rule): iron ingot + stick in the
#      2x2 grid -> `chisel:chisel` x1, and both ingredients are consumed.
#   7. an id the jar never registers (`chisel:zag/blackstone`, which upstream's lang names)
#      is refused by `give` and by `setblock`.
#
#   tools/e2e.sh                     # the chisel: conversions, no mining, no drops
#   COUNTERFACTUAL=1 tools/e2e.sh    # the same break with minecraft:stone_pickaxe:
#                                    # the block IS mined and drops one of itself, and the
#                                    # blackstone pillar drops its own loot table entry
#
# GAME must be a launcher-built game carrying this package (the run refuses a missing one):
#   GAME=<launcher build>/horizon-client CTL=<a horizon-ctl of the same build> tools/e2e.sh
# CONTROL_GAME=<a game WITHOUT the mod> adds the differential leg: the same `give` finds no
# `chisel:array/andesite` and no `chisel:pillar/blackstone`, so the ids are known only
# because the package is installed.
#
# SELFTEST=1 flips the andesite expectation (phase 1); SELFTEST=blackstone flips only the
# blackstone one (phase 5), so the run fails exactly at the new slice's assertion while every
# other phase stays green. Either must FAIL: the proof the scenario can fail.
set -euo pipefail

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/chisel
# A launcher-built game carries the mod; `horizon-ctl` must come from the same build (the
# shared binary is published per pin, and none exists at 0b14f8a yet). The port's own fast
# build is the default because `tools/horizon.py` is retired: nothing here builds a game.
CTL=${CTL:-$PORT/target/horizon-cargo/fast/horizon-ctl}
GAME=${GAME:-$PORT/target/horizon-cargo/fast/horizon-client}
# An optional game built WITHOUT this package: the differential leg below proves the ids
# come from the package rather than from the engine.
CONTROL_GAME=${CONTROL_GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/chisel-e2e-run.sock}
COUNTERFACTUAL=${COUNTERFACTUAL:-0}
SELFTEST=${SELFTEST:-0}

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL (build it against the same SDK pin as GAME)" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client (a launcher build with this mod): $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }
[ -f "$PORT/chisel.hmod" ] || { echo "missing package: $PORT/chisel.hmod" >&2; exit 1; }

if [ "$COUNTERFACTUAL" = 1 ]; then
    TOOL=minecraft:stone_pickaxe
    # The pickaxe mines the block: air, and one dropped item.
    EXPECT_MINE=1
else
    TOOL=chisel:chisel
    EXPECT_MINE=0
fi
# Phase 1's expected result, flipped by the selftest.
EXPECT_FIRST=chisel:braid/andesite
if [ "$SELFTEST" = 1 ]; then
    echo "(selftest: asserting the chisel leaves chisel:array/andesite in place)"
    EXPECT_FIRST=chisel:array/andesite
fi
# Phase 5's expected result, flipped by the blackstone selftest only.
EXPECT_BLACKSTONE=minecraft:polished_blackstone
if [ "$SELFTEST" = blackstone ]; then
    echo "(selftest: asserting the chisel leaves minecraft:blackstone in place)"
    EXPECT_BLACKSTONE=minecraft:blackstone
fi

SCRATCH=$(mktemp -d /tmp/opencode/chisel-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")"
rm -f "$SOCK"
# The game's game folder defaults to the working directory: run from scratch so
# saves/options/caches never land in the port tree.
cd "$SCRATCH"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
fail() { echo "FAIL: $*" >&2; exit 1; }
block_id() { ctl block "$1" "$2" "$3" | jq -r '.block // "unknown"'; }
block_state() { ctl block "$1" "$2" "$3" | jq -r '.state // "unknown"'; }
# Every chisel block item the player holds or that lies on the ground: a broken block's
# drop can be picked up (measured: the player 2 blocks away picked one up), so counting only
# item entities reports a lost drop.
chisel_items() {
    local ground held
    ground=$(ctl entities 8 minecraft:item \
        | jq '[.entities[] | .item | select(.item | startswith("chisel:")) | select(.item != "chisel:chisel") | .count] | add // 0')
    held=$(ctl inventory \
        | jq '[.counts | to_entries[] | select(.key | startswith("chisel:")) | select(.key != "chisel:chisel") | .value] | add // 0')
    echo $((ground + held))
}
setblock() {
    local reply
    # `ctl` exits non-zero for an unknown block; without the `||` the assignment's status
    # would end the script under `set -e` before the message below (the control leg).
    reply=$(ctl setblock "$1" "$2" "$3" "$4" 2>&1) \
        || fail "setblock $4 at $1 $2 $3 was refused: $reply"
    case "$reply" in
        *'"ok":true'*) ;;
        *) fail "setblock $4 at $1 $2 $3 was refused: $reply" ;;
    esac
    # A control write lands on the next tick (`ctl give` does too), so a read straight
    # after it sees the previous state.
    ctl wait 1 >/dev/null
}
# One item id, wherever it is: on the ground or in the player's inventory (a mined block's
# drop can be picked up, measured).
count_item() {
    local ground held
    ground=$(ctl entities 8 minecraft:item \
        | jq --arg id "$1" '[.entities[] | .item | select(.item == $id) | .count] | add // 0')
    held=$(ctl inventory | jq --arg id "$1" '.counts[$id] // 0')
    echo $((ground + held))
}

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    rm -rf "$SCRATCH" || true
    # Last line of every run: `check-e2e-logs.py` reads it as the verdict.
    echo "EXIT:$status"
}
trap cleanup EXIT

launch() { # launch WORLD [GAME]
    local game=${2:-$GAME}
    rm -f "$SOCK"
    # `--offscreen`, not `--headless`: the recipe phase (6) opens the inventory and clicks
    # grid slots, and a slot click needs the screen's rendered layout (`horizon-ctl
    # click-slot` answers "no renderer yet" under `--headless`). Nothing else in the
    # scenario depends on the renderer.
    "$CTL" --socket "$SOCK" launch \
        --game "$game" --world "$1" --seed 20261112 --offscreen --wait \
        --game-dir "$SCRATCH/game" --resources "$RES" >/dev/null
    ctl wait-ready 120
}

# The differential leg: a game built WITHOUT this package knows neither the blocks nor the
# chisel, so the same `give` the scenario relies on finds nothing. If the id resolved
# there, the run below would prove nothing about the package.
control_leg() {
    echo "== control: a game built without the package =="
    launch ChiselControl "$CONTROL_GAME"
    ctl command "clear @s" >/dev/null
    ctl give chisel:array/andesite 1 >/dev/null 2>&1 || true
    ctl wait 3
    ctl inventory | jq -e '[.hotbar[], .main[] | select(.item == "chisel:array/andesite")] | length == 0' >/dev/null \
        || fail "the control game has chisel:array/andesite, so the ids did not come from the package"
    ctl give chisel:pillar/blackstone 1 >/dev/null 2>&1 || true
    ctl wait 3
    ctl inventory | jq -e '[.hotbar[], .main[] | select(.item == "chisel:pillar/blackstone")] | length == 0' >/dev/null \
        || fail "the control game has chisel:pillar/blackstone, so the ids did not come from the package"
    echo "  without the package: neither chisel:array/andesite nor chisel:pillar/blackstone could be given"
    ctl quit >/dev/null 2>&1 || true
    rm -f "$SOCK"
}

# Aim at the block's south face and hold the attack button: one dig, which the hook
# converts (or refuses, or a real tool completes). The press, the wait and the release run
# as a ctl *script* inside the game, so no socket round trip falls between them: the client
# restarts a dig whenever the target block changes, and each restart is one more
# conversion, which is what a player holding the button gets too.
mine() { # mine X Y Z [TICKS|none]
    ctl look-at-block "$1" "$2" "$3" south || fail "cannot aim at $1 $2 $3"
    ctl wait 2
    # "none" is one press and release in the same frame: exactly one dig, measured (a
    # one-tick hold already gives three, because the client restarts a dig whenever the
    # block changes). A real tool needs the hold to finish the break.
    if [ "${4:-none}" = none ]; then
        printf 'mouse left down\nmouse left up\n' > "$SCRATCH/dig.txt"
    else
        printf 'mouse left down\nwait %s\nmouse left up\n' "$4" > "$SCRATCH/dig.txt"
    fi
    ctl script "$SCRATCH/dig.txt" >/dev/null || fail "the dig script failed"
    ctl wait 6
}

# The player must be where the assertions think it is: `ctl tp` sets the client position
# and the server corrects it ("moved wrongly"), so the server-side teleport is used.
place_player() {
    ctl command "tp @s 0 100 0" >/dev/null
    ctl wait 2
    local pos
    pos=$(ctl state | jq -r '.player.position | @csv')
    echo "player: $pos"
    case "$pos" in
        0,100* | 0.5,100*) ;;
        *) fail "the player is at $pos, not on the platform" ;;
    esac
}

if [ -n "$CONTROL_GAME" ]; then control_leg; fi

echo "== launch (survival world ChiselE2E, the package built into the game) =="
launch ChiselE2E

echo "== what the game says about content mods =="
ctl logs --lines 400 | grep -iE 'chisel|content mod|refus' | head -20 || true

echo "== a stone platform in The End (y=99), the player on it =="
ctl command "time set noon"
ctl command "difficulty peaceful"
ctl command "kill @e[type=!minecraft:player]" >/dev/null
ctl fill -6 99 -6 6 99 6 minecraft:stone >/dev/null
ctl wait 2
place_player
ctl command "clear @s" >/dev/null

echo "== phase 0: every carried variant places and reads back =="
CELL="0 100 -2"
variants="array braid chaotic_bricks chaotic_medium chaotic_small circular cracked
cracked_bricks cut dent encased_bricks french jellybean large_tile layer mosaic ornate
panel pillar prism raw road slant small_bricks soft_bricks solid_bricks tiles_medium
tiles_small triple_bricks twist weaver zag"
placed=0
for style in $variants; do
    id="chisel:$style/andesite"
    setblock 0 100 -2 "$id"
    got=$(block_id 0 100 -2)
    [ "$got" = "$id" ] || fail "setblock $id read back as $got"
    placed=$((placed + 1))
done
echo "  $placed variants placed and read back"
[ "$placed" -eq 32 ] || fail "$placed variants, expected 32"
setblock 0 100 -2 'chisel:pillar/andesite[axis=x]'
state=$(block_state 0 100 -2)
echo "  chisel:pillar/andesite[axis=x] -> $state"
case "$state" in *"axis=x"*) ;; *) fail "the pillar did not keep axis=x: $state" ;; esac
setblock 0 100 -2 'chisel:twist/andesite[axis=z]'
state=$(block_state 0 100 -2)
echo "  chisel:twist/andesite[axis=z] -> $state"
case "$state" in *"axis=z"*) ;; *) fail "the twist did not keep axis=z: $state" ;; esac
setblock 0 100 -2 'chisel:pillar/andesite'
state=$(block_state 0 100 -2)
echo "  chisel:pillar/andesite (no properties) -> $state"
case "$state" in *"axis=y"*) ;; *) fail "the pillar's default axis is not y: $state" ;; esac

echo "== phase 0b: the blackstone family's variant places and reads back =="
setblock 0 100 -2 chisel:pillar/blackstone
got=$(block_id 0 100 -2)
[ "$got" = "chisel:pillar/blackstone" ] || fail "setblock chisel:pillar/blackstone read back as $got"
setblock 0 100 -2 'chisel:pillar/blackstone[axis=x]'
state=$(block_state 0 100 -2)
echo "  chisel:pillar/blackstone[axis=x] -> $state"
case "$state" in *"axis=x"*) ;; *) fail "the blackstone pillar did not keep axis=x: $state" ;; esac
setblock 0 100 -2 'chisel:pillar/blackstone[axis=z]'
state=$(block_state 0 100 -2)
echo "  chisel:pillar/blackstone[axis=z] -> $state"
case "$state" in *"axis=z"*) ;; *) fail "the blackstone pillar did not keep axis=z: $state" ;; esac
setblock 0 100 -2 chisel:pillar/blackstone
state=$(block_state 0 100 -2)
echo "  chisel:pillar/blackstone (no properties) -> $state"
case "$state" in *"axis=y"*) ;; *) fail "the blackstone pillar's default axis is not y: $state" ;; esac

# The creative tab is data, and the game writes the list it built. Assert it when the build
# exposes it; a build that does not is said so in the log rather than passed silently.
CREATIVE=$SCRATCH/game/.horizon-cache/mod-content/creative-inventory.json
if [ -f "$CREATIVE" ]; then
    for id in chisel:chisel chisel:array/andesite chisel:zag/andesite chisel:pillar/blackstone; do
        grep -q "$id" "$CREATIVE" || fail "the creative tab misses $id"
    done
    echo "  the creative tab lists the chisel, the andesite family and the blackstone pillar"
else
    echo "  (no creative-inventory.json in this build: the tab list is not written to a file)"
fi

echo "== the tool: $TOOL =="
ctl give "$TOOL" 1 >/dev/null || fail "the content mod did not provide $TOOL"
ctl wait 2
ctl select "$TOOL" >/dev/null || fail "cannot select $TOOL"
ctl wait 2
held=$(ctl inventory | jq -r '.held.item // "none"')
echo "  held: $held"
[ "$held" = "$TOOL" ] || fail "the held item is $held, expected $TOOL"

if [ "$COUNTERFACTUAL" = 0 ]; then
    echo "== phase 1: the chisel converts chisel:array/andesite to the next member =="
    setblock 0 100 -2 chisel:array/andesite
    before=$(chisel_items)
    mine 0 100 -2
    after=$(chisel_items)
    got=$(block_id 0 100 -2)
    echo "  after the mining attempt: $got (chisel block items $before -> $after)"
    [ "$got" = "$EXPECT_FIRST" ] \
        || fail "the chisel left $got, expected $EXPECT_FIRST"
    [ "$after" -eq "$before" ] || fail "the conversion mined $((after - before)) items"

    echo "== phase 1b: an adventure player (no build permission) converts nothing =="
    ctl command "gamemode adventure @s" >/dev/null
    ctl wait 2
    setblock 4 100 -2 chisel:array/andesite
    before=$(chisel_items)
    mine 4 100 -2
    after=$(chisel_items)
    got=$(block_id 4 100 -2)
    echo "  adventure + chisel on chisel:array/andesite: $got (chisel block items $before -> $after)"
    [ "$got" = "chisel:array/andesite" ] || fail "a restricted player converted the block to $got"
    [ "$after" -eq "$before" ] || fail "a restricted dig dropped $((after - before)) items"
    ctl command "gamemode creative @s" >/dev/null
    ctl wait 2

    echo "== phase 2: the group's edges (last wraps, the vanilla pair is in the group) =="
    setblock 1 100 -2 chisel:zag/andesite
    mine 1 100 -2
    got=$(block_id 1 100 -2)
    echo "  chisel:zag/andesite -> $got"
    [ "$got" = "minecraft:andesite" ] || fail "the last member wrapped to $got, expected minecraft:andesite"

    setblock 2 100 -2 minecraft:andesite
    mine 2 100 -2
    got=$(block_id 2 100 -2)
    echo "  minecraft:andesite -> $got"
    [ "$got" = "minecraft:polished_andesite" ] \
        || fail "vanilla andesite became $got, expected minecraft:polished_andesite"

    setblock 2 100 -2 minecraft:polished_andesite
    mine 2 100 -2
    got=$(block_id 2 100 -2)
    echo "  minecraft:polished_andesite -> $got"
    [ "$got" = "chisel:array/andesite" ] \
        || fail "polished andesite became $got, expected chisel:array/andesite"

    echo "== phase 3: a block outside the family is refused =="
    setblock 3 100 -2 minecraft:dirt
    before=$(chisel_items)
    mine 3 100 -2
    after=$(chisel_items)
    got=$(block_id 3 100 -2)
    echo "  minecraft:dirt with the chisel: $got (chisel block items $before -> $after)"
    [ "$got" = "minecraft:dirt" ] || fail "the chisel turned dirt into $got"
    [ "$after" -eq "$before" ] || fail "the refused block dropped $((after - before)) items"

    echo "== phase 4: the target is written as its default state (axis=y) =="
    setblock -1 100 -2 chisel:panel/andesite
    mine -1 100 -2
    got=$(block_id -1 100 -2)
    state=$(block_state -1 100 -2)
    echo "  chisel:panel/andesite -> $got $state"
    [ "$got" = "chisel:pillar/andesite" ] || fail "panel became $got, expected chisel:pillar/andesite"
    case "$state" in *"axis=y"*) ;; *) fail "the pillar landed as $state, not its default axis=y" ;; esac

    echo "== phase 5: the blackstone group (mostly vanilla) and family isolation =="
    setblock 0 100 -1 minecraft:blackstone
    before=$(chisel_items)
    mine 0 100 -1
    after=$(chisel_items)
    got=$(block_id 0 100 -1)
    echo "  minecraft:blackstone -> $got (chisel block items $before -> $after)"
    [ "$got" = "$EXPECT_BLACKSTONE" ] \
        || fail "vanilla blackstone became $got, expected $EXPECT_BLACKSTONE"
    [ "$after" -eq "$before" ] || fail "the conversion mined $((after - before)) items"

    setblock 0 100 -1 minecraft:cracked_polished_blackstone_bricks
    mine 0 100 -1
    got=$(block_id 0 100 -1)
    echo "  minecraft:cracked_polished_blackstone_bricks -> $got"
    [ "$got" = "chisel:pillar/blackstone" ] \
        || fail "the group's last vanilla member became $got, expected chisel:pillar/blackstone"

    setblock 0 100 -1 chisel:pillar/blackstone
    mine 0 100 -1
    got=$(block_id 0 100 -1)
    echo "  chisel:pillar/blackstone -> $got"
    [ "$got" = "minecraft:blackstone" ] \
        || fail "the blackstone style wrapped to $got, expected minecraft:blackstone"

    setblock 0 100 -1 chisel:braid/andesite
    mine 0 100 -1
    got=$(block_id 0 100 -1)
    echo "  chisel:braid/andesite -> $got (the other family still converts)"
    [ "$got" = "chisel:chaotic_bricks/andesite" ] \
        || fail "braid became $got, expected chisel:chaotic_bricks/andesite"

    echo "== phase 6: the chisel's own recipe is crafted (iron ingot + stick) =="
    # Survival: the creative inventory screen has no 2x2 crafting grid, only the item list.
    ctl command "gamemode survival @s" >/dev/null
    ctl command "clear @s" >/dev/null
    ctl wait 2
    ctl give minecraft:iron_ingot 1 >/dev/null || fail "cannot give the iron ingot"
    ctl give minecraft:stick 1 >/dev/null || fail "cannot give the stick"
    ctl wait 3
    echo "  inventory: $(ctl inventory | jq -c '.counts')"
    ctl open-inventory >/dev/null || fail "cannot open the inventory"
    ctl wait 3
    # The 2x2 grid is slots 100-103 and the result 200. The recipe is " I" / "S ", so the
    # iron ingot goes top-right (101) and the stick bottom-left (102). One tick between the
    # clicks: a click is processed on the next tick, so a click that follows one immediately
    # can still see the previous cursor state.
    grid() { ctl ui | jq -c '[.widgets[] | select(.id|test("slot-10[0-3]")) | {id, item: .stack.item}]'; }
    ctl click-slot 0 >/dev/null || fail "click-slot 0 failed"
    ctl wait 1
    ctl click-slot 101 >/dev/null || fail "click-slot 101 failed"
    ctl wait 1
    ctl click-slot 1 >/dev/null || fail "click-slot 1 failed"
    ctl wait 1
    ctl click-slot 102 >/dev/null || fail "click-slot 102 failed"
    ctl wait 3
    echo "  grid: $(grid)"
    [ "$(ctl ui | jq -r '[.widgets[] | select(.id=="slot-101") | .stack.item][0] // "empty"')" = "minecraft:iron_ingot" ] \
        || fail "the iron ingot is not in grid slot 101: $(grid)"
    [ "$(ctl ui | jq -r '[.widgets[] | select(.id=="slot-102") | .stack.item][0] // "empty"')" = "minecraft:stick" ] \
        || fail "the stick is not in grid slot 102: $(grid)"
    result_item=$(ctl ui | jq -r '[.widgets[] | select(.id=="slot-200") | .stack.item][0] // "empty"')
    result_count=$(ctl ui | jq -r '[.widgets[] | select(.id=="slot-200") | .stack.count][0] // 0')
    echo "  craft result: $result_item x$result_count"
    [ "$result_item" = "chisel:chisel" ] || fail "the recipe result is $result_item, expected chisel:chisel"
    [ "$result_count" = 1 ] || fail "the recipe result count is $result_count, expected 1"
    # Take the result into the empty third hotbar slot, then close.
    ctl click-slot 200 >/dev/null || fail "taking the craft result failed"
    ctl wait 1
    ctl click-slot 2 >/dev/null || fail "placing the crafted chisel failed"
    ctl wait 3
    ctl close >/dev/null
    ctl wait 2
    counts=$(ctl inventory | jq -c '.counts')
    echo "  inventory after crafting: $counts"
    [ "$(echo "$counts" | jq -r '.["chisel:chisel"] // 0')" = 1 ] \
        || fail "the crafted chisel is not in the inventory: $counts"
    [ "$(echo "$counts" | jq -r '.["minecraft:iron_ingot"] // 0')" = 0 ] \
        || fail "the iron ingot was not consumed: $counts"
    [ "$(echo "$counts" | jq -r '.["minecraft:stick"] // 0')" = 0 ] \
        || fail "the stick was not consumed: $counts"
    ctl command "gamemode creative @s" >/dev/null
    ctl wait 2

    echo "== phase 7: an id the jar never registers is refused =="
    ctl command "clear @s" >/dev/null
    ctl wait 2
    # Upstream's lang names a "Blackstone Zag" but the jar registers no such block.
    reply=$(ctl give chisel:zag/blackstone 1 2>&1) || true
    ctl wait 2
    held=$(ctl inventory | jq -r '[.counts["chisel:zag/blackstone"] // 0] | add')
    echo "  give chisel:zag/blackstone -> $reply (in the inventory: $held)"
    [ "$held" = 0 ] || fail "give accepted chisel:zag/blackstone, which the jar never registers"
    ctl setblock 6 100 -2 minecraft:air >/dev/null
    ctl wait 1
    reply=$(ctl setblock 6 100 -2 chisel:zag/blackstone 2>&1) || true
    ctl wait 2
    got=$(block_id 6 100 -2)
    echo "  setblock chisel:zag/blackstone -> $got"
    [ "$got" = "minecraft:air" ] || fail "setblock accepted chisel:zag/blackstone (block is $got)"
else
    echo "== counterfactual: the same break with a stone pickaxe mines the block =="
    setblock 0 100 -2 chisel:array/andesite
    before=$(chisel_items)
    mine 0 100 -2 20
    after=$(chisel_items)
    got=$(block_id 0 100 -2)
    echo "  after the mining attempt: $got (chisel block items $before -> $after)"
    if [ "$EXPECT_MINE" = 1 ]; then
        [ "$got" = "minecraft:air" ] || fail "the pickaxe left $got, expected the block to be mined"
        [ $((after - before)) -eq 1 ] \
            || fail "the mined block left $((after - before)) items, expected 1"
        ctl entities 8 minecraft:item | jq -c '{items: [.entities[] | .item]}'
        ctl inventory | jq -c '.counts'
    fi

    echo "== counterfactual: the blackstone pillar's loot table drops itself =="
    setblock 1 100 -2 chisel:pillar/blackstone
    mine 1 100 -2 20
    got=$(block_id 1 100 -2)
    dropped=$(count_item chisel:pillar/blackstone)
    echo "  chisel:pillar/blackstone with the pickaxe: $got (dropped $dropped)"
    [ "$got" = "minecraft:air" ] || fail "the pickaxe left $got, expected the pillar to be mined"
    [ "$dropped" -eq 1 ] \
        || fail "the mined pillar dropped $dropped of itself, expected 1 (loot table)"
fi

ctl quit >/dev/null 2>&1 || true
trap - EXIT
sleep 1
rm -rf "$SCRATCH" || true
if [ "$COUNTERFACTUAL" = 1 ]; then
    echo "PASS: (counterfactual, stone pickaxe) the block was mined and dropped one of itself, and the blackstone pillar dropped its own loot table entry"
else
    echo "PASS: 32 andesite variants and the blackstone pillar placed and read back; the chisel converted array->braid, refused a restricted (adventure) player, wrapped zag->andesite, walked the vanilla pair, refused dirt, wrote axis=y, walked the blackstone group (blackstone->polished_blackstone->...->pillar->blackstone) without leaving its family, crafted the chisel from an iron ingot and a stick (both consumed), and refused chisel:zag/blackstone; nothing was mined or dropped"
fi
echo "EXIT:0"
