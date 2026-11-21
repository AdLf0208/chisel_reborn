# Package receipt — chisel 0.2.3

| field | value |
|---|---|
| version | `0.2.3` |
| file | `chisel.hmod` |
| bytes | 118,046 |
| sha256 | `a4169ab73a4e04b427d2fc0f0eb834c72fd70182891f88d5a7bca1bed6c44dd5` |
| source | this tag — the tree here is the source that produced the archive |
| game | 26.3 |
| horizon | `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` |
| horizon-mixins | `e59da9094f0c2fdda6c627fe9eff22dec6a906f0` |

## Verification of these exact bytes

`cargo test` 28 passed / 0 failed (8 failed before the fix) · `clippy --all-targets -- -D warnings` 0 warnings · `fmt --check` clean · two builds `cmp`-identical · `check` exit 0 · `fits --build 0a875da` `made for` exit 0 · `fits --build 0b14f8a` `fits` exit 0 · `verify-sdk-package.py` 1/1 · launcher leg `PASS` + `EXIT:0`; control `EXIT:1`; three selftests `EXIT:1` (including `the locked chisel wrote chisel:panel/andesite, expected minecraft:polished_andesite`); positive counterfactual legs `EXIT:0`; `check-e2e-logs --strict` 6/6.
