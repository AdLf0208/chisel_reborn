#!/usr/bin/env bash
# All four legs of the Chisel e2e in one build lease. Each leg's own log lands in
# `ports/_shared/evidence/chisel-e2e/`, and `check-e2e-logs.py` reads their EXIT lines.
#
#   1. the chisel (default): with `CONTROL_GAME` set, the unmodded differential first, then
#      the conversions (andesite and blackstone), the recipe and the unknown-id refusal
#   2. the counterfactual: the same break with a stone pickaxe mines the block and drops it
#   3. the selftest: the andesite conversion assertion flipped, which must FAIL
#   4. the blackstone selftest: only the slice-2 assertion flipped, which must FAIL there
#      while every other phase stays green
#
# The game is a launcher build at the SDK pin that carries this package:
#
#   ports/tools/launcher-game.sh build chisel     # once, or `run` for build + leg 1
#   eval "$(ports/tools/launcher-game.sh env chisel)"
#   ports/tools/build-slot.sh run --job e2e-chisel --port ports/chisel --step e2e -- \
#       bash ports/chisel/tools/e2e-all.sh
#
# The script exits 0 only when legs 1 and 2 pass and legs 3 and 4 fail as required.
set -uo pipefail
CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/chisel
EVIDENCE=$CLONE/ports/_shared/evidence/chisel-e2e
mkdir -p "$EVIDENCE"
cd "$PORT"

run_leg() { # run_leg NAME EXPECTED_EXIT ENV…
    local name=$1 expect=$2; shift 2
    echo "=== leg $name (expecting exit $expect) ==="
    local status
    env "$@" ./tools/e2e.sh 2>&1 | tee "$EVIDENCE/e2e-$name.log"
    status=${PIPESTATUS[0]}
    echo "=== leg $name: exit $status (expected $expect) ==="
    [ "$status" = "$expect" ] || { echo "LEG $name: wrong exit $status, expected $expect"; return 1; }
    return 0
}

rc=0
run_leg launcher 0 || rc=1
run_leg launcher-counterfactual 0 COUNTERFACTUAL=1 || rc=1
run_leg launcher-selftest 1 SELFTEST=1 || rc=1
run_leg launcher-selftest-blackstone 1 SELFTEST=blackstone || rc=1

if [ "$rc" = 0 ]; then
    echo "PASS: the chisel leg, the counterfactual leg and both selftest legs behaved as required"
    echo "EXIT:0"
else
    echo "FAIL: at least one leg did not behave as required"
    echo "EXIT:1"
fi
exit "$rc"
