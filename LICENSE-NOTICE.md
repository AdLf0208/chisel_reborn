# Chisel for Horizon — licence notice

This package is a **derivative work** of **Chisel Reborn** and of the original **Chisel**,
and it is distributed under the **same licence as the original: GNU General Public License,
version 2 only (GPL-2.0-only)**. The full licence text is `LICENSE` beside this file (it is
also the canonical text from <https://www.gnu.org/licenses/old-licenses/gpl-2.0.txt>).

## What this port is

- **Original mod:** Chisel Reborn 2.0.1 for Minecraft 1.21.1 NeoForge, mod id `chisel`
  (jar `chisel-neoforge-2.0.1+mc1.21.1.jar`). Its `META-INF/neoforge.mods.toml` declares
  `license = "GPLv2"`, `displayName = "Chisel Reborn"` and the authors
  **Slaincow**, **KnowYourKnot** and **Original Creator: AUTOMATIC_MAIDEN**.
- **Maintainer / source:** Chisel Reborn — <https://github.com/matthewperiut/Chisel-Fabric>
  (the port was written against tag `1.21.1-2.0.1`, commit
  `b399d0f218acae743321d64604fb378c6c7b791c`).
- **Original Chisel:** **ttterrag1098** and contributors —
  <https://github.com/Chisel-Team/Chisel>.
- **This port:** a clean-room Rust rewrite for **Horizon** (Minecraft Java 26.3), written
  from the mod's own data files, its public documentation and its observable behaviour. It
  is not a translation of the mod's Java code and contains none of it.

## What this package carries

- The **code**: `mixins/chisel/` — `horizon-mixin.toml` (the plan), `runtime.rs` (the
  conversion table, with its unit tests), `support.rs` (the game glue), `click.rs` (the hook
  helper) and `embedded_content.rs` (which loads the content half from this same file).
- The **content**: `mixins/chisel/data/chisel-content.hmod` — the 32 `andesite` block
  variants of the original mod (blockstates, models, textures, item definitions, loot
  tables, language file, the chisel item, its recipe and the creative tab), carried from the
  original jar and converted to the 26.3 data and resource formats.
- **Complete corresponding source.** The source of everything in this package travels with
  it: every `.rs` and `.toml` above is the compiled source of the native half, and the
  content half is generated reproducibly from the original jar by `tools/gen_content.py`
  in the port tree. Anyone who redistributes this package must ship that source under
  GPL-2.0-only as well.

## Authors of the port

The Rust code and the content conversion were written for this campaign; the assets,
models, blockstates and names are the original mod's and remain under GPL-2.0-only. The
port keeps the original's licence, gives the original authors credit, and makes no claim of
endorsement by them.
