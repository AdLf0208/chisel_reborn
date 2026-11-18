#!/usr/bin/env bash
# All three legs of the Chisel e2e in one build lease. Each leg's own log lands in
# `ports/_shared/evidence/chisel/`, and `check-e2e-logs.py` reads their EXIT lines.
#
#   1. the chisel (default): with `CONTROL_GAME` set, the unmodded differential first, then
#      the conversions
#   2. the counterfactual: the same break with a stone pickaxe mines the block
#   3. the selftest: the conversion assertion flipped, which must FAIL
#
# STATUS (2026-11-18): the mod API 0.2 package needs a game the launcher built with it
# (`horizon-launcher native-build ports/chisel --instance <i> --trust-native-code`), and the
# launcher is not installed here, so no leg can run until one exists. Set GAME (and CTL)
# when it does. The retired `.hnmod` legs are historical evidence, not current ones.
#
# The script exits 0 only when leg 1 and leg 2 pass and leg 3 fails as required.
set -uo pipefail
CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/chisel
EVIDENCE=$CLONE/ports/_shared/evidence/chisel
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
run_leg chisel 0 || rc=1
run_leg counterfactual 0 COUNTERFACTUAL=1 || rc=1
run_leg selftest 1 SELFTEST=1 || rc=1

if [ "$rc" = 0 ]; then
    echo "PASS: the chisel leg, the counterfactual leg and the selftest leg behaved as required"
    echo "EXIT:0"
else
    echo "FAIL: at least one leg did not behave as required"
    echo "EXIT:1"
fi
exit "$rc"
