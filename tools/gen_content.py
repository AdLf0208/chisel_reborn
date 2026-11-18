#!/usr/bin/env python3
"""Carry the `andesite` family of Chisel Reborn into the package root (26.3 shapes).

Source of truth is the pack's own jar, never the mod's name: every file written here comes
from `assets/chisel/` or `data/chisel/` of `chisel-neoforge-2.0.1+mc1.21.1.jar`, and the
script fails loudly if one of them is missing. Only the four things 26.3 spells differently
are generated: the block declarations (`data/chisel/horizon/block/`), the loot tables'
condition, the recipe's ingredient form, and the item definitions/lang the jar already has
in 26.3 shape but with the names' dots/slashes fixed (`FINDINGS.md` D1).

    python3 tools/gen_content.py                 # rewrite assets/ + data/ at the package root
    python3 tools/gen_content.py --check         # only verify the jar has every input

The manifest is not generated: `horizon-mod.toml` at the root is the package's identity and
pins, and it is written by hand. Only the two resource packs are regenerated.

Nothing is invented: the block template, state property and default state are the ones
`EzReg.Reg` builds the Java block with (`AbstractBlock.Settings.copy(Blocks.ANDESITE)`, a
`PillarBlock` for `pillar`/`twist`).
"""
import argparse
import json
import shutil
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
JAR = ROOT.parent / "_shared/licence-cache/chisel-neoforge-2.0.1+mc1.21.1.jar"
# The flat SDK package (mod API 0.2) puts the carried content at the package root; the
# retired `content/` folder and its own manifest are gone.
ASSETS = ROOT / "assets"
DATA = ROOT / "data"
MATERIAL = "andesite"

# `GeneratedRegister.java:24-49`, in file order: the 32 styles of the andesite family.
STYLES = [
    "array", "braid", "chaotic_bricks", "chaotic_medium", "chaotic_small", "circular",
    "cracked", "cracked_bricks", "cut", "dent", "encased_bricks", "french", "jellybean",
    "large_tile", "layer", "mosaic", "ornate", "panel", "pillar", "prism", "raw", "road",
    "slant", "small_bricks", "soft_bricks", "solid_bricks", "tiles_medium", "tiles_small",
    "triple_bricks", "twist", "weaver", "zag",
]
# `EzReg.Reg`: a `PillarBlock` (an `axis` property) for a name holding "pillar" or "twist".
AXIS_STYLES = {"pillar", "twist"}
# Blockstate keys that belong to Athena/OptiFine CTM, which the pin does not load.
CTM_KEYS = ("athena:loader", "ctm_textures", "height", "width")


def block_id(style):
    return f"chisel:{style}/{MATERIAL}"


def read(jar, name):
    try:
        return jar.read(name)
    except KeyError:
        raise SystemExit(f"the jar has no {name}: the brief and the artefact disagree") from None


def json_bytes(value):
    return (json.dumps(value, indent=2, ensure_ascii=False) + "\n").encode()


def blockstate(jar, style):
    """The jar's blockstate without the CTM keys the pin cannot read (FINDINGS.md §8)."""
    document = json.loads(read(jar, f"assets/chisel/blockstates/{style}/{MATERIAL}.json"))
    for key in CTM_KEYS:
        document.pop(key, None)
    if "variants" not in document and "multipart" not in document:
        raise SystemExit(f"{style}: the blockstate has neither variants nor multipart")
    return json_bytes(document)


def loot_table(style):
    """The jar's self-drop table in 26.3's typed-condition shape."""
    return json_bytes({
        "type": "minecraft:block",
        "pools": [{
            "rolls": 1,
            "entries": [{
                "type": "minecraft:item",
                "name": block_id(style),
                "condition": {"type": "minecraft:survives_explosion"},
            }],
        }],
    })


def block_declaration(style):
    """`minecraft:andesite` is the template: `EzReg.Reg` copies its settings exactly."""
    declaration = {"template": "minecraft:andesite", "item": True}
    if style in AXIS_STYLES:
        # `PillarBlock`: `axis` with `x`/`y`/`z` and `y` by default.
        declaration["state_properties"] = {"axis": {"type": "enum", "values": ["x", "y", "z"]}}
        declaration["default_state"] = {"axis": "y"}
    return json_bytes(declaration)


def chisel_item():
    """`ChiselItem extends BundleItem`, `maxCount(1)`, no bar, `getMiningSpeed` 500.

    The bundle half is not in slice 1 (no bundle component); the mining speed is, because
    the conversion happens on a mining attempt and upstream's 500 makes it instant. The
    rules list keeps the tool non-empty, which is what makes `default_mining_speed` apply
    to a block no rule matches (`horizon-items/src/items.rs:1135-1153`).
    """
    return json_bytes({
        "template": "minecraft:iron_ingot",
        "max_stack_size": 1,
        "components": {
            "minecraft:tool": {
                "default_mining_speed": 500.0,
                "damage_per_block": 0,
                "rules": [{"blocks": "#minecraft:mineable/pickaxe", "speed": 500.0}],
            },
        },
    })


def recipe():
    """`data/chisel/recipe/chisel.json`, 1.21.1's `{"item": …}` ingredients as 26.3 strings."""
    return json_bytes({
        "type": "minecraft:crafting_shaped",
        "category": "equipment",
        "key": {"I": "minecraft:iron_ingot", "S": "minecraft:stick"},
        "pattern": [" I", "S "],
        "result": {"id": "chisel:chisel", "count": 1},
    })


def lang(jar):
    """Upstream's en_us plus the id-derived names the game actually asks for (FINDINGS.md D1)."""
    source = json.loads(read(jar, "assets/chisel/lang/en_us.json"))
    carried = {}
    for style in STYLES:
        dotted = f"item.chisel.{style}.{MATERIAL}"
        if dotted not in source:
            raise SystemExit(f"the jar has no lang key {dotted}")
        name = source[dotted]
        # Upstream's spelling, then the key `Util.createTranslationKey` derives from the id.
        carried[dotted] = name
        carried[f"block.chisel.{style}.{MATERIAL}"] = name
        carried[f"block.chisel.{style}/{MATERIAL}"] = name
        carried[f"item.chisel.{style}/{MATERIAL}"] = name
    for key in ("item.chisel.chisel", "gui.chisel.chisel", "itemGroup.chisel.chisel"):
        if key not in source:
            raise SystemExit(f"the jar has no lang key {key}")
        carried[key] = source[key]
    return json_bytes(dict(sorted(carried.items())))


def tab():
    return json_bytes({
        "title": "itemGroup.chisel.chisel",
        "icon": "chisel:chisel",
        "items": ["chisel:chisel"] + [block_id(style) for style in STYLES],
    })


def pickaxe_tag():
    return json_bytes({"replace": False, "values": [block_id(style) for style in STYLES]})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, default=JAR)
    parser.add_argument("--check", action="store_true", help="verify the jar only, write nothing")
    args = parser.parse_args()
    if not args.jar.is_file():
        raise SystemExit(f"missing jar: {args.jar}")
    jar = zipfile.ZipFile(args.jar)
    names = set(jar.namelist())

    # Every input this slice carries, named once so a missing one fails here, not in game.
    wanted = []
    for style in STYLES:
        wanted += [
            f"assets/chisel/blockstates/{style}/{MATERIAL}.json",
            f"assets/chisel/items/{style}/{MATERIAL}.json",
            f"assets/chisel/models/item/{style}/{MATERIAL}.json",
            f"data/chisel/loot_table/blocks/{style}/{MATERIAL}.json",
            f"assets/chisel/textures/block/{style}/{MATERIAL}.png",
        ]
    for style in ("pillar", "twist"):
        wanted += [
            f"assets/chisel/models/block/{style}/{MATERIAL}_horizontal.json",
            f"assets/chisel/textures/block/{style}/{MATERIAL}_top.png",
        ]
    wanted += ["assets/chisel/items/chisel.json", "assets/chisel/models/item/chisel.json",
               "assets/chisel/textures/item/chisel.png", "assets/chisel/lang/en_us.json",
               "data/chisel/recipe/chisel.json"]
    missing = [name for name in wanted if name not in names]
    if missing:
        raise SystemExit("the jar is missing carried inputs:\n  " + "\n  ".join(missing))

    # Every model a blockstate names, and every texture a model names, must exist too.
    models, textures = set(), set()
    for style in STYLES:
        state = json.loads(read(jar, f"assets/chisel/blockstates/{style}/{MATERIAL}.json"))
        for variant in state["variants"].values():
            for entry in variant if isinstance(variant, list) else [variant]:
                models.add(entry["model"])
    for model in sorted(models):
        path = "assets/chisel/models/" + model.split(":", 1)[1] + ".json"
        if path not in names:
            raise SystemExit(f"missing model {path}")
        document = json.loads(jar.read(path))
        for value in document.get("textures", {}).values():
            if isinstance(value, str) and ":" in value and not value.startswith("#"):
                textures.add(value)
    for texture in sorted(textures):
        path = "assets/chisel/textures/" + texture.split(":", 1)[1] + ".png"
        if path not in names:
            raise SystemExit(f"missing texture {path}")
    print(f"jar inventory: {len(STYLES)} variants, {len(models)} block models, "
          f"{len(textures)} textures named by them")

    if args.check:
        return 0

    for pack in (ASSETS, DATA):
        if pack.exists():
            shutil.rmtree(pack)
    def write(relative, data):
        path = ROOT / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        return path

    for style in STYLES:
        write(f"assets/chisel/blockstates/{style}/{MATERIAL}.json", blockstate(jar, style))
        write(f"data/chisel/horizon/block/{style}/{MATERIAL}.json", block_declaration(style))
        write(f"data/chisel/loot_table/blocks/{style}/{MATERIAL}.json", loot_table(style))
        for source in (f"assets/chisel/items/{style}/{MATERIAL}.json",
                       f"assets/chisel/models/item/{style}/{MATERIAL}.json",
                       f"assets/chisel/textures/block/{style}/{MATERIAL}.png"):
            write(source, jar.read(source))
        if style in AXIS_STYLES:
            write(f"assets/chisel/textures/block/{style}/{MATERIAL}_top.png",
                  jar.read(f"assets/chisel/textures/block/{style}/{MATERIAL}_top.png"))
    for model in sorted(models):
        write("assets/chisel/models/" + model.split(":", 1)[1] + ".json", jar.read(
            "assets/chisel/models/" + model.split(":", 1)[1] + ".json"))
    for texture in sorted(textures):
        relative = "assets/chisel/textures/" + texture.split(":", 1)[1] + ".png"
        write(relative, jar.read(relative))
    for source in ("assets/chisel/items/chisel.json", "assets/chisel/models/item/chisel.json",
                   "assets/chisel/textures/item/chisel.png"):
        write(source, jar.read(source))
    write("assets/chisel/lang/en_us.json", lang(jar))
    write("data/chisel/horizon/item/chisel.json", chisel_item())
    write("data/chisel/horizon/creative_tab/chisel.json", tab())
    write("data/chisel/recipe/chisel.json", recipe())
    write("data/minecraft/tags/block/mineable/pickaxe.json", pickaxe_tag())

    files = sorted(
        p.relative_to(ROOT).as_posix()
        for pack in (ASSETS, DATA)
        for p in pack.rglob("*")
        if p.is_file()
    )
    print(f"wrote {len(files)} files under {ASSETS} and {DATA}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
