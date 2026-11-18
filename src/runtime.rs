// The conversion rule of Chisel Reborn's chisel, pure and game-free.
//
// Upstream keeps one *group* per material (`ChiselGroupLookup`), and the chisel writes a
// member of the group over the block it is used on (`ChiselItem.canMine`). This module is
// that group for the `andesite` family in the order the mod registers it, and the member
// that follows a given one.

/// The `andesite` group, in the order `VanillaChisel.Register` then `GeneratedRegister`
/// register it (`block/BlockRegister.java:7-11`; duplicates dropped keeping the first).
///
/// The first two are vanilla blocks: they are group members, so the chisel turns vanilla
/// andesite into a chisel variant and a variant back into vanilla andesite.
pub const ANDESITE_FAMILY: [&str; 34] = [
    "minecraft:andesite",
    "minecraft:polished_andesite",
    "chisel:array/andesite",
    "chisel:braid/andesite",
    "chisel:chaotic_bricks/andesite",
    "chisel:chaotic_medium/andesite",
    "chisel:chaotic_small/andesite",
    "chisel:circular/andesite",
    "chisel:cracked/andesite",
    "chisel:cracked_bricks/andesite",
    "chisel:cut/andesite",
    "chisel:dent/andesite",
    "chisel:encased_bricks/andesite",
    "chisel:french/andesite",
    "chisel:jellybean/andesite",
    "chisel:large_tile/andesite",
    "chisel:layer/andesite",
    "chisel:mosaic/andesite",
    "chisel:ornate/andesite",
    "chisel:panel/andesite",
    "chisel:pillar/andesite",
    "chisel:prism/andesite",
    "chisel:raw/andesite",
    "chisel:road/andesite",
    "chisel:slant/andesite",
    "chisel:small_bricks/andesite",
    "chisel:soft_bricks/andesite",
    "chisel:solid_bricks/andesite",
    "chisel:tiles_medium/andesite",
    "chisel:tiles_small/andesite",
    "chisel:triple_bricks/andesite",
    "chisel:twist/andesite",
    "chisel:weaver/andesite",
    "chisel:zag/andesite",
];

/// The member after `current` in [`ANDESITE_FAMILY`], wrapping from the last to the first.
///
/// `None` when `current` is not in the group: upstream leaves such a block alone (the group
/// lookup is empty, so `canMine` does nothing but still refuses the break).
#[must_use]
pub fn next_variant(current: &str) -> Option<&'static str> {
    let index = ANDESITE_FAMILY
        .iter()
        .position(|member| *member == current)?;
    let next = (index + 1) % ANDESITE_FAMILY.len();
    ANDESITE_FAMILY.get(next).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The 32 styles of the `andesite` family, in `GeneratedRegister` order: the group
    /// without its two vanilla members.
    const CARRIED: [&str; 32] = [
        "chisel:array/andesite",
        "chisel:braid/andesite",
        "chisel:chaotic_bricks/andesite",
        "chisel:chaotic_medium/andesite",
        "chisel:chaotic_small/andesite",
        "chisel:circular/andesite",
        "chisel:cracked/andesite",
        "chisel:cracked_bricks/andesite",
        "chisel:cut/andesite",
        "chisel:dent/andesite",
        "chisel:encased_bricks/andesite",
        "chisel:french/andesite",
        "chisel:jellybean/andesite",
        "chisel:large_tile/andesite",
        "chisel:layer/andesite",
        "chisel:mosaic/andesite",
        "chisel:ornate/andesite",
        "chisel:panel/andesite",
        "chisel:pillar/andesite",
        "chisel:prism/andesite",
        "chisel:raw/andesite",
        "chisel:road/andesite",
        "chisel:slant/andesite",
        "chisel:small_bricks/andesite",
        "chisel:soft_bricks/andesite",
        "chisel:solid_bricks/andesite",
        "chisel:tiles_medium/andesite",
        "chisel:tiles_small/andesite",
        "chisel:triple_bricks/andesite",
        "chisel:twist/andesite",
        "chisel:weaver/andesite",
        "chisel:zag/andesite",
    ];

    #[test]
    fn vanilla_andesite_converts_to_its_polished_form() {
        assert_eq!(
            next_variant("minecraft:andesite"),
            Some("minecraft:polished_andesite")
        );
    }

    #[test]
    fn the_first_chisel_style_follows_the_vanilla_pair() {
        assert_eq!(
            next_variant("minecraft:polished_andesite"),
            Some("chisel:array/andesite")
        );
        assert_eq!(
            next_variant("chisel:array/andesite"),
            Some("chisel:braid/andesite")
        );
    }

    #[test]
    fn the_last_member_wraps_to_the_first() {
        assert_eq!(
            next_variant("chisel:zag/andesite"),
            Some("minecraft:andesite")
        );
    }

    #[test]
    fn a_block_outside_the_family_is_refused() {
        for block in [
            "minecraft:dirt",
            "minecraft:air",
            "minecraft:stone",
            "chisel:array/stone", // the same style, a family this slice does not carry
            "chisel:array/andesite_", // a near miss, not the id
            "",
        ] {
            assert_eq!(
                next_variant(block),
                None,
                "{block} is not in the andesite group"
            );
        }
    }

    #[test]
    fn the_group_is_the_upstream_registration_order() {
        assert_eq!(
            ANDESITE_FAMILY.len(),
            34,
            "two vanilla members plus 32 styles"
        );
        assert_eq!(
            &ANDESITE_FAMILY[..2],
            &["minecraft:andesite", "minecraft:polished_andesite"]
        );
        assert_eq!(&ANDESITE_FAMILY[2..], &CARRIED);
    }

    #[test]
    fn every_member_appears_once() {
        let unique: std::collections::BTreeSet<&str> = ANDESITE_FAMILY.iter().copied().collect();
        assert_eq!(
            unique.len(),
            ANDESITE_FAMILY.len(),
            "the group has no duplicate"
        );
    }

    #[test]
    fn walking_the_group_visits_every_member_and_returns() {
        let mut seen = Vec::new();
        let mut current = ANDESITE_FAMILY[0];
        for _ in 0..ANDESITE_FAMILY.len() {
            seen.push(current);
            current = next_variant(current).expect("every member has a successor");
        }
        assert_eq!(current, ANDESITE_FAMILY[0], "the walk wraps after 34 steps");
        assert_eq!(
            seen, ANDESITE_FAMILY,
            "the walk visits each member once, in order"
        );
    }
}
