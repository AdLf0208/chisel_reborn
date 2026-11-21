// The conversion rule of Chisel Reborn's chisel, pure and game-free.
//
// Upstream keeps one *group* per material (`ChiselGroupLookup`), and the chisel writes a
// member of the group over the block it is used on (`ChiselItem.canMine`). This module is
// those groups for the families this port carries — `andesite` and `blackstone` — in the
// order the mod registers them, and the member that follows a given one.

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

/// The `blackstone` group, in the order `VanillaChisel.Register` then `GeneratedRegister`
/// register it: the five vanilla blackstone forms first, then the family's one carried
/// style (`GeneratedRegister.java:104`, `Reg("blackstone", "pillar/blackstone")`).
///
/// `EzReg.Reg` adds `minecraft:blackstone` (the material's base block) to the group again
/// before the style; that duplicate is dropped, exactly as it is for andesite.
pub const BLACKSTONE_FAMILY: [&str; 6] = [
    "minecraft:blackstone",
    "minecraft:polished_blackstone",
    "minecraft:chiseled_polished_blackstone",
    "minecraft:polished_blackstone_bricks",
    "minecraft:cracked_polished_blackstone_bricks",
    "chisel:pillar/blackstone",
];

/// Every group this slice carries. A conversion is looked up per family, so a member of one
/// material can never step into another's styles.
pub const FAMILIES: [&[&str]; 2] = [&ANDESITE_FAMILY, &BLACKSTONE_FAMILY];

/// The member after `current` in its own family, wrapping from the last to the first.
///
/// `None` when `current` is in no carried group: upstream leaves such a block alone (the
/// group lookup is empty, so `canMine` does nothing but still refuses the break).
#[must_use]
pub fn next_variant(current: &str) -> Option<&'static str> {
    FAMILIES.iter().find_map(|family| next_in(family, current))
}

/// Whether a chisel carrying `chosen` writes it over `target`.
///
/// This is upstream's second branch (`ChiselItem.canMine`): with a block in its bundle the
/// chisel does not walk the group at all, it writes that block. The two ids are compared on
/// their **last path segment** (`chisel:panel/andesite` answers `andesite`), and either one
/// containing the other is a match, so `minecraft:polished_andesite` and
/// `minecraft:andesite_slab` both take an `andesite` lock. A target whose segment names a
/// stair never matches: upstream refuses stairs outright.
#[must_use]
pub fn lock_matches(target: &str, chosen: &str) -> bool {
    let (target, chosen) = (segment(target), segment(chosen));
    if target.is_empty() || chosen.is_empty() {
        return false;
    }
    if target.contains("stairs") {
        return false;
    }
    target.contains(chosen) || chosen.contains(target)
}

/// The last `/`-separated segment of an id's path (`chisel:panel/andesite` answers `andesite`).
fn segment(id: &str) -> &str {
    let path = id.split_once(':').map_or(id, |(_, path)| path);
    path.rsplit('/').next().unwrap_or(path)
}

/// The member after `current` within `family`, or `None` when `current` is not in it.
fn next_in(family: &[&'static str], current: &str) -> Option<&'static str> {
    let index = family.iter().position(|member| *member == current)?;
    let next = (index + 1) % family.len();
    family.get(next).copied()
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
            // Upstream's lang names "Blackstone Zag" but the jar registers no such block.
            "chisel:zag/blackstone",
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

    /// The five vanilla blackstone forms `VanillaChisel` puts in the group, in its order,
    /// then the one style `GeneratedRegister` adds.
    const BLACKSTONE_VANILLA: [&str; 5] = [
        "minecraft:blackstone",
        "minecraft:polished_blackstone",
        "minecraft:chiseled_polished_blackstone",
        "minecraft:polished_blackstone_bricks",
        "minecraft:cracked_polished_blackstone_bricks",
    ];

    #[test]
    fn the_blackstone_group_is_the_upstream_registration_order() {
        assert_eq!(
            BLACKSTONE_FAMILY.len(),
            6,
            "five vanilla members plus the one carried style"
        );
        assert_eq!(&BLACKSTONE_FAMILY[..5], &BLACKSTONE_VANILLA);
        assert_eq!(BLACKSTONE_FAMILY[5], "chisel:pillar/blackstone");
    }

    #[test]
    fn vanilla_blackstone_walks_its_vanilla_forms() {
        for pair in BLACKSTONE_VANILLA.windows(2) {
            assert_eq!(
                next_variant(pair[0]),
                Some(pair[1]),
                "{} should become {}",
                pair[0],
                pair[1]
            );
        }
        assert_eq!(
            next_variant("minecraft:cracked_polished_blackstone_bricks"),
            Some("chisel:pillar/blackstone"),
            "the vanilla forms lead into the one carried style"
        );
    }

    #[test]
    fn the_blackstone_style_wraps_to_vanilla() {
        assert_eq!(
            next_variant("chisel:pillar/blackstone"),
            Some("minecraft:blackstone")
        );
    }

    #[test]
    fn a_conversion_never_leaves_its_own_family() {
        for family in FAMILIES {
            for member in family {
                let next = next_variant(member).expect("every member has a successor");
                assert!(
                    family.contains(&next),
                    "{member} converted to {next}, which is in another family"
                );
            }
        }
        // The two directions that a single shared table would get wrong.
        assert_eq!(
            next_variant("chisel:pillar/blackstone"),
            Some("minecraft:blackstone"),
            "a blackstone style must not step into the andesite group"
        );
        assert_eq!(
            next_variant("chisel:zag/andesite"),
            Some("minecraft:andesite"),
            "an andesite style must not step into the blackstone group"
        );
    }

    #[test]
    fn no_block_belongs_to_two_families() {
        let total: usize = FAMILIES.iter().map(|family| family.len()).sum();
        let unique: std::collections::BTreeSet<&str> = FAMILIES
            .iter()
            .flat_map(|family| family.iter().copied())
            .collect();
        assert_eq!(
            unique.len(),
            total,
            "a block appears in more than one family"
        );
        assert_eq!(FAMILIES.len(), 2, "andesite and blackstone");
    }

    #[test]
    fn a_held_block_locks_the_conversion_to_its_material() {
        // `ChiselItem.canMine`'s second branch: the bundle's front block replaces the walk.
        for (target, chosen) in [
            ("chisel:array/andesite", "chisel:panel/andesite"),
            ("minecraft:andesite", "chisel:pillar/andesite"),
            ("minecraft:polished_andesite", "chisel:panel/andesite"),
            ("chisel:zag/andesite", "minecraft:polished_andesite"),
        ] {
            assert!(
                lock_matches(target, chosen),
                "{target} should take a lock on {chosen}"
            );
        }
    }

    #[test]
    fn a_lock_matches_by_containment_either_way() {
        // The comparison is `contains`, not equality, on the last path segment.
        assert!(
            lock_matches("minecraft:andesite_slab", "chisel:panel/andesite"),
            "a slab's segment contains the lock's"
        );
        assert!(
            lock_matches("minecraft:polished_andesite", "minecraft:andesite"),
            "a polished variant's segment contains the lock's"
        );
        assert!(
            lock_matches("chisel:array/andesite", "minecraft:andesite"),
            "the lock's segment is contained by the target's"
        );
    }

    #[test]
    fn a_lock_of_another_material_matches_nothing() {
        for (target, chosen) in [
            ("minecraft:blackstone", "chisel:panel/andesite"),
            ("chisel:pillar/andesite", "minecraft:blackstone"),
            ("minecraft:dirt", "minecraft:andesite"),
            ("minecraft:andesite", "minecraft:blackstone"),
            ("", "chisel:panel/andesite"),
            ("minecraft:andesite", ""),
            ("", ""),
        ] {
            assert!(
                !lock_matches(target, chosen),
                "{target} must not take a lock on {chosen}"
            );
        }
    }

    #[test]
    fn a_stair_is_never_locked() {
        // Upstream refuses `compare[1].contains("stairs")` before it compares anything; a
        // slab of the same material is not refused, so this is the stair rule alone.
        assert!(!lock_matches(
            "minecraft:andesite_stairs",
            "chisel:panel/andesite"
        ));
        assert!(lock_matches(
            "minecraft:andesite_slab",
            "chisel:panel/andesite"
        ));
    }
}
