//! Chisel Reborn for Horizon — the andesite and blackstone families and the chisel that
//! converts between the variants of each.
//!
//! Upstream converts on a **mining attempt**: `ChiselItem.canMine` runs where a dig starts, writes
//! a member of the block's chisel group over it, and returns `false`, so the block is never mined
//! (`ChiselItem.java:126-171`). The SDK hears that moment as [`Hooks::left_click_block`], which may
//! cancel the click so the block "isn't attacked, dug or broken on the server" — the same
//! observable result, once per dig. The rule itself (the group and the next member) lives in
//! [`runtime`], byte-identical to the retired `.hnmod` bundle's `runtime.rs`; this file only turns
//! the click and the held item into that call and writes the answer back.
use mod_api::{self, BlockUpdateFlags, Flow, Hooks, LeftClickBlockEvent, ServerHost};

pub mod runtime;

/// The chisel's item id (`Chisel.init`: `Identifier.of("chisel", "chisel")`).
const CHISEL: &str = "chisel:chisel";

/// What a dig does to the clicked block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Dig {
    /// No chisel in the acting hand: the dig is the game's own.
    Allow,
    /// The chisel is in hand and the block has a next member: convert, and refuse the dig.
    Convert(&'static str),
    /// The chisel is in hand but the block has no next member: refuse the dig, change nothing.
    Refuse,
}

/// The dig `held` (the acting hand's item id, if any) makes on the block `current`.
///
/// Upstream's `canMine` returns `false` for **every** block a chisel is held over, including one
/// outside every group, so a chisel never mines anything; only the conversion needs a group member.
fn dig(held: Option<&str>, current: &str, may_build: bool) -> Dig {
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
    match runtime::next_variant(current) {
        Some(next) => Dig::Convert(next),
        None => Dig::Refuse,
    }
}

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// A player starts digging the block at `event.pos` with a chisel in the acting hand: the
    /// block becomes the next member of its family and the dig never starts.
    ///
    /// The target is written as the family table's default state (`block_state_default`), which is
    /// upstream's `getDefaultState()`: a conversion into `pillar`/`twist` always lands on `axis=y`.
    /// The click is cancelled even where nothing converts, so — as upstream — a chisel never mines.
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
        let current = event.state.block().id(&*host);
        // A stale player handle answers nothing; treat that as "may not build" rather than
        // letting an unreadable actor write.
        let may_build = player.may_build(&*host) == Some(true);
        match dig(held_id.as_deref(), &current, may_build) {
            Dig::Allow => Flow::Continue,
            Dig::Refuse => Flow::Cancel,
            Dig::Convert(next) => {
                // The next member is one of the carried family (the runtime table is the source),
                // so a missing key or default state means the content half is absent: refuse the
                // dig and leave the block as it is, rather than break something upstream never
                // breaks. The SDK has no logger, so this is silent (gaps in `notes.md`).
                if let Some(state) = host
                    .block_key(next)
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
        assert_eq!(dig(None, "chisel:array/andesite", true), Dig::Allow);
    }

    #[test]
    fn another_tool_allows_the_dig() {
        assert_eq!(
            dig(
                Some("minecraft:stone_pickaxe"),
                "chisel:array/andesite",
                true
            ),
            Dig::Allow
        );
        assert_eq!(
            dig(Some("minecraft:andesite"), "chisel:array/andesite", true),
            Dig::Allow
        );
    }

    #[test]
    fn the_chisel_converts_a_family_member_in_order() {
        assert_eq!(
            dig(Some(CHISEL), "chisel:array/andesite", true),
            Dig::Convert("chisel:braid/andesite")
        );
        assert_eq!(
            dig(Some(CHISEL), "minecraft:andesite", true),
            Dig::Convert("minecraft:polished_andesite")
        );
    }

    #[test]
    fn the_chisel_wraps_the_last_member_to_vanilla() {
        assert_eq!(
            dig(Some(CHISEL), "chisel:zag/andesite", true),
            Dig::Convert("minecraft:andesite")
        );
    }

    #[test]
    fn the_chisel_refuses_a_block_outside_the_family() {
        for block in ["minecraft:dirt", "minecraft:air", "chisel:array/stone"] {
            assert_eq!(
                dig(Some(CHISEL), block, true),
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
                dig(Some(CHISEL), block, false),
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
            dig(Some(CHISEL), "minecraft:blackstone", true),
            Dig::Convert("minecraft:polished_blackstone")
        );
        assert_eq!(
            dig(
                Some(CHISEL),
                "minecraft:cracked_polished_blackstone_bricks",
                true
            ),
            Dig::Convert("chisel:pillar/blackstone")
        );
        assert_eq!(
            dig(Some(CHISEL), "chisel:pillar/blackstone", true),
            Dig::Convert("minecraft:blackstone"),
            "the blackstone style wraps inside blackstone, not into andesite"
        );
    }
}
