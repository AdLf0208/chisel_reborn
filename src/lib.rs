//! Chisel Reborn for Horizon — the andesite and blackstone families, the chisel that converts
//! between the variants of each, and the block a chisel carries to lock it to one variant.
//!
//! Upstream converts on a **mining attempt**: `ChiselItem.canMine` runs where a dig starts, and
//! returns `false`, so the block is never mined (`ChiselItem.java:126-171`). It has two branches:
//! with a block in the chisel's bundle it writes **that** block, and an empty bundle writes a
//! member of the block's chisel group — in the port, the next member. The SDK hears that moment
//! as [`Hooks::left_click_block`], which may cancel the click so the block "isn't attacked, dug
//! or broken on the server" — the same observable result, once per dig. The rule itself (the
//! groups, the next member and the lock) lives in [`runtime`], game-free; this file only reads
//! the click and the held item, and writes the answer back.
use mod_api::{
    self, BlockUpdateFlags, ComponentId, Flow, Hooks, Host, ItemResource, ItemStack,
    LeftClickBlockEvent, ServerHost, Tag,
};

pub mod runtime;

/// The chisel's item id (`Chisel.init`: `Identifier.of("chisel", "chisel")`).
const CHISEL: &str = "chisel:chisel";

/// The component a chisel carries its chosen block in (`ChiselItem` is a `BundleItem`).
const BUNDLE_CONTENTS: &str = "minecraft:bundle_contents";

/// What a dig does to the clicked block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Dig<'a> {
    /// No chisel in the acting hand: the dig is the game's own.
    Allow,
    /// The chisel is in hand and a block is to be written over the clicked one: convert, and
    /// refuse the dig. The member is the lock's block when the chisel carries one, else the next
    /// member of the clicked block's family.
    Convert(&'a str),
    /// The chisel is in hand but nothing is to be written: refuse the dig, change nothing.
    Refuse,
}

/// The dig `held` (the acting hand's item id, if any) makes on the block `current`, carrying
/// `chosen` (the front block of the chisel's bundle, if it holds one).
///
/// Upstream's `canMine` returns `false` for **every** block a chisel is held over, including one
/// outside every group, so a chisel never mines anything; only the conversion needs a group
/// member. A non-empty bundle replaces the group walk altogether: the carried block is written
/// when its material matches the clicked block's, and nothing is written when it does not — a
/// locked chisel never falls back to walking.
fn dig<'a>(held: Option<&str>, chosen: Option<&'a str>, current: &str, may_build: bool) -> Dig<'a> {
    if held != Some(CHISEL) {
        return Dig::Allow;
    }
    // `left_click_block` runs after the dig's reach and spawn-protection checks but before the
    // adventure restriction that would refuse the break. A player who may not build (adventure
    // or spectator, `Player.mayBuild`) must not have a block converted either, so the write is
    // refused here rather than trusted to a later hook that never runs.
    if !may_build {
        return Dig::Refuse;
    }
    if let Some(chosen) = chosen {
        return if runtime::lock_matches(current, chosen) {
            Dig::Convert(chosen)
        } else {
            Dig::Refuse
        };
    }
    match runtime::next_variant(current) {
        Some(next) => Dig::Convert(next),
        None => Dig::Refuse,
    }
}

/// The front block of `stack`'s `minecraft:bundle_contents`, if it carries one.
///
/// The chisel's contents are the game's own bundle component, so the game's inventory inserts a
/// block clicked onto it and its tooltip lists it; the conversion only has to read the front
/// item, which is the one a bundle gives back first (`BundleContents.get(0)`).
fn bundle_front(host: &dyn Host, stack: &ItemStack) -> Option<String> {
    let component = ComponentId::new(BUNDLE_CONTENTS)?;
    let value = ItemResource::of(stack).value(host, &component)?;
    bundle_ids(&value).into_iter().next()
}

/// The item ids a `minecraft:bundle_contents` value holds, front first.
///
/// 26.3 stores them as a list of item templates: a compound with `id` (and `count`, and an
/// optional `components` patch), or a bare id string.
fn bundle_ids(value: &Tag) -> Vec<String> {
    let Tag::List(entries) = value else {
        return Vec::new();
    };
    entries
        .iter()
        .filter_map(|entry| match entry {
            Tag::String(id) => Some(id.to_string_lossy()),
            entry => match entry.get("id") {
                Some(Tag::String(id)) => Some(id.to_string_lossy()),
                _ => None,
            },
        })
        .collect()
}

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// A player starts digging the block at `event.pos` with a chisel in the acting hand: the
    /// block becomes the chisel's carried block when it holds one, else the next member of its
    /// family, and the dig never starts.
    ///
    /// The target is written as the chosen block's default state (`block_state_default`), which
    /// is upstream's `getDefaultState()`: a conversion into `pillar`/`twist` always lands on
    /// `axis=y`. The click is cancelled even where nothing converts, so — as upstream — a chisel
    /// never mines.
    fn left_click_block(&self, host: &mut dyn ServerHost, event: &mut LeftClickBlockEvent) -> Flow {
        let Some(player) = host.players().acting_player() else {
            return Flow::Continue;
        };
        let Some(living) = host.living(player.entity()) else {
            return Flow::Continue;
        };
        let Some(level) = player.level(&*host) else {
            return Flow::Continue;
        };
        let held = living.hand_item(&*host, mod_api::Hand::Main);
        let held_id = held.as_ref().map(|stack| stack.item().id(&*host));
        let chosen = held.as_ref().and_then(|stack| bundle_front(&*host, stack));
        let current = event.state.block().id(&*host);
        // A stale player handle answers nothing; treat that as "may not build" rather than
        // letting an unreadable actor write.
        let may_build = player.may_build(&*host) == Some(true);
        match dig(held_id.as_deref(), chosen.as_deref(), &current, may_build) {
            Dig::Allow => Flow::Continue,
            Dig::Refuse => Flow::Cancel,
            Dig::Convert(target) => {
                // The target is one of the carried family or the chisel's own carried block, so
                // a missing key or default state means the content half is absent (or the bundle
                // holds something that is not a block): refuse the dig and leave the block as it
                // is, rather than break something upstream never breaks. The SDK has no logger,
                // so this is silent (gaps in `notes.md`).
                if let Some(state) = host
                    .block_key(target)
                    .and_then(|key| host.block_state_default(key))
                {
                    let _ = level.set_block(
                        &mut *host,
                        event.pos,
                        state,
                        BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS,
                    );
                }
                Flow::Cancel
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_held_item_allows_the_dig() {
        assert_eq!(dig(None, None, "chisel:array/andesite", true), Dig::Allow);
    }

    #[test]
    fn another_tool_allows_the_dig() {
        assert_eq!(
            dig(
                Some("minecraft:stone_pickaxe"),
                None,
                "chisel:array/andesite",
                true
            ),
            Dig::Allow
        );
        assert_eq!(
            dig(
                Some("minecraft:andesite"),
                None,
                "chisel:array/andesite",
                true
            ),
            Dig::Allow
        );
    }

    #[test]
    fn the_chisel_converts_a_family_member_in_order() {
        assert_eq!(
            dig(Some(CHISEL), None, "chisel:array/andesite", true),
            Dig::Convert("chisel:braid/andesite")
        );
        assert_eq!(
            dig(Some(CHISEL), None, "minecraft:andesite", true),
            Dig::Convert("minecraft:polished_andesite")
        );
    }

    #[test]
    fn the_chisel_wraps_the_last_member_to_vanilla() {
        assert_eq!(
            dig(Some(CHISEL), None, "chisel:zag/andesite", true),
            Dig::Convert("minecraft:andesite")
        );
    }

    #[test]
    fn the_chisel_refuses_a_block_outside_the_family() {
        for block in ["minecraft:dirt", "minecraft:air", "chisel:array/stone"] {
            assert_eq!(
                dig(Some(CHISEL), None, block, true),
                Dig::Refuse,
                "a held chisel never mines {block}"
            );
        }
    }

    #[test]
    fn a_restricted_player_converts_nothing() {
        // The hook fires before the adventure check, so the port enforces it: an adventure or
        // spectator player (`mayBuild` false) must not turn andesite into another variant.
        for block in ["minecraft:andesite", "chisel:array/andesite"] {
            assert_eq!(
                dig(Some(CHISEL), None, block, false),
                Dig::Refuse,
                "a restricted player must not convert {block}"
            );
        }
    }

    #[test]
    fn the_chisel_converts_a_blackstone_member_inside_its_own_family() {
        // `dig` asks the family table, so the second family needs no branch of its own; this
        // covers the route rather than the rule (the rule's tests are in `runtime`).
        assert_eq!(
            dig(Some(CHISEL), None, "minecraft:blackstone", true),
            Dig::Convert("minecraft:polished_blackstone")
        );
        assert_eq!(
            dig(
                Some(CHISEL),
                None,
                "minecraft:cracked_polished_blackstone_bricks",
                true
            ),
            Dig::Convert("chisel:pillar/blackstone")
        );
        assert_eq!(
            dig(Some(CHISEL), None, "chisel:pillar/blackstone", true),
            Dig::Convert("minecraft:blackstone"),
            "the blackstone style wraps inside blackstone, not into andesite"
        );
    }

    #[test]
    fn a_chisel_carrying_a_block_writes_that_block() {
        // Upstream's second branch: the bundle wins over the group walk.
        assert_eq!(
            dig(
                Some(CHISEL),
                Some("chisel:panel/andesite"),
                "minecraft:andesite",
                true
            ),
            Dig::Convert("chisel:panel/andesite"),
            "the lock replaces the walk's next member"
        );
        assert_eq!(
            dig(
                Some(CHISEL),
                Some("chisel:panel/andesite"),
                "chisel:array/andesite",
                true
            ),
            Dig::Convert("chisel:panel/andesite")
        );
    }

    #[test]
    fn a_lock_of_another_material_writes_nothing() {
        // A locked chisel never falls back to walking: it refuses the dig and changes nothing.
        assert_eq!(
            dig(
                Some(CHISEL),
                Some("chisel:panel/andesite"),
                "minecraft:blackstone",
                true
            ),
            Dig::Refuse
        );
        assert_eq!(
            dig(
                Some(CHISEL),
                Some("minecraft:blackstone"),
                "chisel:pillar/andesite",
                true
            ),
            Dig::Refuse
        );
    }

    #[test]
    fn a_restricted_player_converts_nothing_even_with_a_lock() {
        assert_eq!(
            dig(
                Some(CHISEL),
                Some("chisel:panel/andesite"),
                "minecraft:andesite",
                false
            ),
            Dig::Refuse
        );
    }

    #[test]
    fn bundle_contents_decode_to_the_ids_inside() {
        let value = Tag::List(vec![
            Tag::compound([("id", Tag::String("chisel:panel/andesite".into()))]),
            Tag::String("minecraft:andesite".into()),
            Tag::compound([
                ("id", Tag::String("minecraft:dirt".into())),
                ("count", Tag::Int(3)),
            ]),
        ]);
        assert_eq!(
            bundle_ids(&value),
            [
                "chisel:panel/andesite",
                "minecraft:andesite",
                "minecraft:dirt"
            ],
            "the front item first, a bare id accepted"
        );
    }

    #[test]
    fn bundle_contents_that_hold_nothing_decode_to_nothing() {
        assert!(bundle_ids(&Tag::List(Vec::new())).is_empty());
        assert!(bundle_ids(&Tag::compound(Vec::<(&str, Tag)>::new())).is_empty());
        assert!(bundle_ids(&Tag::Int(1)).is_empty());
        assert!(
            bundle_ids(&Tag::List(vec![Tag::compound([("count", Tag::Int(1))])])).is_empty(),
            "an entry without an id is not an item"
        );
    }
}
