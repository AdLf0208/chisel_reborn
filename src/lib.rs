//! Chisel Reborn for Horizon — the andesite family and the chisel that converts between its
//! variants.
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
fn dig(held: Option<&str>, current: &str) -> Dig {
    if held != Some(CHISEL) {
        return Dig::Allow;
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
        match dig(held_id.as_deref(), &current) {
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
        assert_eq!(dig(None, "chisel:array/andesite"), Dig::Allow);
    }

    #[test]
    fn another_tool_allows_the_dig() {
        assert_eq!(
            dig(Some("minecraft:stone_pickaxe"), "chisel:array/andesite"),
            Dig::Allow
        );
        assert_eq!(
            dig(Some("minecraft:andesite"), "chisel:array/andesite"),
            Dig::Allow
        );
    }

    #[test]
    fn the_chisel_converts_a_family_member_in_order() {
        assert_eq!(
            dig(Some(CHISEL), "chisel:array/andesite"),
            Dig::Convert("chisel:braid/andesite")
        );
        assert_eq!(
            dig(Some(CHISEL), "minecraft:andesite"),
            Dig::Convert("minecraft:polished_andesite")
        );
    }

    #[test]
    fn the_chisel_wraps_the_last_member_to_vanilla() {
        assert_eq!(
            dig(Some(CHISEL), "chisel:zag/andesite"),
            Dig::Convert("minecraft:andesite")
        );
    }

    #[test]
    fn the_chisel_refuses_a_block_outside_the_family() {
        for block in ["minecraft:dirt", "minecraft:air", "chisel:array/stone"] {
            assert_eq!(
                dig(Some(CHISEL), block),
                Dig::Refuse,
                "a held chisel never mines {block}"
            );
        }
    }
}
