#!/usr/bin/env python3
"""Carry Chisel Reborn's block families into the package root (26.3 shapes).

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

Nothing is invented. Each family names the vanilla block `EzReg.Reg` copies
(`AbstractBlock.Settings.copy(Blocks.<base>)`, which is `template` here), and a style whose
name holds "pillar" or "twist" is a `PillarBlock` with an `axis` property defaulting to `y`.
The families and their styles are the jar's own blockstates, in `GeneratedRegister` order.
"""
import argparse
import json
import shutil
import sys
import zipfile
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
JAR = ROOT.parent / "_shared/licence-cache/chisel-neoforge-2.0.1+mc1.21.1.jar"
# The flat SDK package (mod API 0.2) puts the carried content at the package root; the
# retired `content/` folder and its own manifest are gone.
ASSETS = ROOT / "assets"
DATA = ROOT / "data"


@dataclass(frozen=True)
class Family:
    """One material family: the vanilla base `EzReg.Reg` copies, and its styles in order."""

    material: str
    template: str
    styles: tuple[str, ...]


# `GeneratedRegister.java`, in file order (alphabetical by material, then by style), for the
# families this port carries. Slice 1 is `andesite`; slice 2 adds `blackstone`, whose only
# registered style is the pillar (`GeneratedRegister.java:104`).
FAMILIES = (
    Family(
        material="andesite",
        template="minecraft:andesite",
        styles=(
            "array", "braid", "chaotic_bricks", "chaotic_medium", "chaotic_small", "circular",
            "cracked", "cracked_bricks", "cut", "dent", "encased_bricks", "french", "jellybean",
            "large_tile", "layer", "mosaic", "ornate", "panel", "pillar", "prism", "raw", "road",
            "slant", "small_bricks", "soft_bricks", "solid_bricks", "tiles_medium", "tiles_small",
            "triple_bricks", "twist", "weaver", "zag",
        ),
    ),
    Family(
        material="blackstone",
        template="minecraft:blackstone",
        styles=("pillar",),
    ),
)

# `EzReg.Reg`: a `PillarBlock` (an `axis` property) for a name holding "pillar" or "twist".
AXIS_STYLES = frozenset({"pillar", "twist"})
# Blockstate keys that belong to Athena/OptiFine CTM, which the pin does not load.
CTM_KEYS = ("athena:loader", "ctm_textures", "height", "width")


def block_id(family, style):
    return f"chisel:{style}/{family.material}"


def read(jar, name):
    try:
        return jar.read(name)
    except KeyError:
        raise SystemExit(f"the jar has no {name}: the brief and the artefact disagree") from None


def json_bytes(value):
    return (json.dumps(value, indent=2, ensure_ascii=False) + "\n").encode()


def blockstate(jar, family, style):
    """The jar's blockstate without the CTM keys the pin cannot read (FINDINGS.md §8)."""
    name = f"assets/chisel/blockstates/{style}/{family.material}.json"
    document = json.loads(read(jar, name))
    for key in CTM_KEYS:
        document.pop(key, None)
    if "variants" not in document and "multipart" not in document:
        raise SystemExit(f"{style}/{family.material}: neither variants nor multipart")
    return json_bytes(document)


def loot_table(family, style):
    """The jar's self-drop table in 26.3's typed-condition shape."""
    return json_bytes({
        "type": "minecraft:block",
        "pools": [{
            "rolls": 1,
            "entries": [{
                "type": "minecraft:item",
                "name": block_id(family, style),
                "condition": {"type": "minecraft:survives_explosion"},
            }],
        }],
    })


def block_declaration(family, style):
    """`family.template` is the base block: `EzReg.Reg` copies its settings exactly."""
    declaration = {"template": family.template, "item": True}
    if style in AXIS_STYLES:
        # `PillarBlock`: `axis` with `x`/`y`/`z` and `y` by default.
        declaration["state_properties"] = {"axis": {"type": "enum", "values": ["x", "y", "z"]}}
        declaration["default_state"] = {"axis": "y"}
    return json_bytes(declaration)


def chisel_item():
    """`ChiselItem extends BundleItem`, `maxCount(1)`, no bar, `getMiningSpeed` 500.

    The bundle half is not carried (no bundle component); the mining speed is, because the
    conversion happens on a mining attempt and upstream's 500 makes it instant. The rules
    list keeps the tool non-empty, which is what makes `default_mining_speed` apply to a
    block no rule matches (`horizon-items/src/items.rs:1135-1153`).
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
    for family in FAMILIES:
        for style in family.styles:
            dotted = f"item.chisel.{style}.{family.material}"
            if dotted not in source:
                raise SystemExit(f"the jar has no lang key {dotted}")
            name = source[dotted]
            # Upstream's spelling, then the key `Util.createTranslationKey` derives from the id.
            carried[dotted] = name
            carried[f"block.chisel.{style}.{family.material}"] = name
            carried[f"block.chisel.{style}/{family.material}"] = name
            carried[f"item.chisel.{style}/{family.material}"] = name
    for key in ("item.chisel.chisel", "gui.chisel.chisel", "itemGroup.chisel.chisel"):
        if key not in source:
            raise SystemExit(f"the jar has no lang key {key}")
        carried[key] = source[key]
    return json_bytes(dict(sorted(carried.items())))


def tab():
    items = ["chisel:chisel"]
    for family in FAMILIES:
        items += [block_id(family, style) for style in family.styles]
    return json_bytes({
        "title": "itemGroup.chisel.chisel",
        "icon": "chisel:chisel",
        "items": items,
    })


def pickaxe_tag():
    values = [block_id(family, style) for family in FAMILIES for style in family.styles]
    return json_bytes({"replace": False, "values": values})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, default=JAR)
    parser.add_argument("--check", action="store_true", help="verify the jar only, write nothing")
    args = parser.parse_args()
    if not args.jar.is_file():
        raise SystemExit(f"missing jar: {args.jar}")
    jar = zipfile.ZipFile(args.jar)
    names = set(jar.namelist())

    # Every input this port carries, named once so a missing one fails here, not in game.
    wanted = []
    for family in FAMILIES:
        for style in family.styles:
            wanted += [
                f"assets/chisel/blockstates/{style}/{family.material}.json",
                f"assets/chisel/items/{style}/{family.material}.json",
                f"assets/chisel/models/item/{style}/{family.material}.json",
                f"data/chisel/loot_table/blocks/{style}/{family.material}.json",
                f"assets/chisel/textures/block/{style}/{family.material}.png",
            ]
            if style in AXIS_STYLES:
                wanted += [
                    f"assets/chisel/models/block/{style}/{family.material}_horizontal.json",
                    f"assets/chisel/textures/block/{style}/{family.material}_top.png",
                ]
    wanted += ["assets/chisel/items/chisel.json", "assets/chisel/models/item/chisel.json",
               "assets/chisel/textures/item/chisel.png", "assets/chisel/lang/en_us.json",
               "data/chisel/recipe/chisel.json"]
    missing = [name for name in wanted if name not in names]
    if missing:
        raise SystemExit("the jar is missing carried inputs:\n  " + "\n  ".join(missing))

    # Every model a blockstate names, and every texture a model names, must exist too.
    models, textures = set(), set()
    for family in FAMILIES:
        for style in family.styles:
            state = json.loads(
                read(jar, f"assets/chisel/blockstates/{style}/{family.material}.json"))
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
    styles = sum(len(family.styles) for family in FAMILIES)
    print(f"jar inventory: {len(FAMILIES)} families, {styles} variants, "
          f"{len(models)} block models, {len(textures)} textures named by them")

    if args.check:
        return 0

    for pack in (ASSETS, DATA):
        if pack.exists():
            shutil.rmtree(pack)

    def write(relative, data):
        path = ROOT / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)

    for family in FAMILIES:
        for style in family.styles:
            write(f"assets/chisel/blockstates/{style}/{family.material}.json",
                  blockstate(jar, family, style))
            write(f"data/chisel/horizon/block/{style}/{family.material}.json",
                  block_declaration(family, style))
            write(f"data/chisel/loot_table/blocks/{style}/{family.material}.json",
                  loot_table(family, style))
            for source in (f"assets/chisel/items/{style}/{family.material}.json",
                           f"assets/chisel/models/item/{style}/{family.material}.json",
                           f"assets/chisel/textures/block/{style}/{family.material}.png"):
                write(source, jar.read(source))
            if style in AXIS_STYLES:
                write(f"assets/chisel/textures/block/{style}/{family.material}_top.png",
                      jar.read(f"assets/chisel/textures/block/{style}/{family.material}_top.png"))
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
