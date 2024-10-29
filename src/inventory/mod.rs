use super::{
    particle_system::ParticleBuilder, AreaOfEffect, Confusion, Consumable, EquipmentChanged,
    Equippable, Equipped, GameLog, HungerClock, HungerState, IdentifiedItem, InBackpack,
    InflictsDamage, Item, MagicItem, MagicMapper, Map, MasterDungeonMap, Name, ObfuscatedName,
    Pools, Position, ProvidesFood, ProvidesHealing, RunState, SufferDamage, TownPortal,
    WantsToDropItem, WantsToPickupItem, WantsToRemoveItem, WantsToUseItem,
};
use specs::prelude::*;

mod collection_system;
pub use collection_system::ItemCollectionSystem;
mod use_system;
pub use use_system::ItemUseSystem;
mod drop_system;
pub use drop_system::ItemDropSystem;
mod remove_system;
pub use remove_system::ItemRemoveSystem;
mod identification_system;
pub use identification_system::ItemIdentificationSystem;

pub fn obfuscate_name(
    item: Entity,
    names: &ReadStorage<Name>,
    magic_items: &ReadStorage<MagicItem>,
    obfuscated_names: &ReadStorage<ObfuscatedName>,
    dm: &MasterDungeonMap,
) -> String {
    if let Some(name) = names.get(item) {
        if magic_items.get(item).is_some() {
            if dm.identified_items.contains(&name.name) {
                name.name.clone()
            } else if let Some(obfuscated) = obfuscated_names.get(item) {
                obfuscated.name.clone()
            } else {
                "Unidentified magic item".to_string()
            }
        } else {
            name.name.clone()
        }
    } else {
        "Nameless item (bug)".to_string()
    }
}
