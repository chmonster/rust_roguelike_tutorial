use super::{
    CursedItem, EquipmentChanged, Equippable, Equipped, /*GameLog,*/ IdentifiedItem,
    InBackpack, Name, WantsToUseItem,
};
use specs::prelude::*;

pub struct ItemEquipOnUse {}

impl<'a> System<'a> for ItemEquipOnUse {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        //WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Equippable>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, EquipmentChanged>,
        WriteStorage<'a, IdentifiedItem>,
        ReadStorage<'a, CursedItem>,
    );

    #[allow(clippy::cognitive_complexity)]
    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            //mut gamelog,
            entities,
            mut wants_use,
            names,
            equippable,
            mut equipped,
            mut backpack,
            mut dirty,
            mut identified,
            cursed,
        ) = data;

        let mut remove_use: Vec<Entity> = Vec::new();
        for (target, useitem) in (&entities, &wants_use).join() {
            // If it is equippable, then we want to equip it - and unequip whatever else was in that slot
            if let Some(can_equip) = equippable.get(useitem.item) {
                let target_slot = can_equip.slot;
                let mut can_equip = true;
                let /*mut*/ log_entries: Vec<String> = Vec::new();
                let mut to_unequip: Vec<Entity> = Vec::new();

                for (item_entity, already_equipped, name) in (&entities, &equipped, &names).join() {
                    if already_equipped.owner == target && already_equipped.slot == target_slot {
                        if cursed.get(item_entity).is_some() {
                            crate::gamelog::Logger::new()
                                .color(rltk::RED)
                                .append("You cannot unequip")
                                .item_name(&name.name)
                                .color(rltk::RED)
                                .append("- it is cursed!")
                                .log();
                            can_equip = false;
                        } else {
                            to_unequip.push(item_entity);
                            if target == *player_entity {
                                crate::gamelog::Logger::new()
                                    .append("You unequip")
                                    .item_name(&name.name)
                                    .log();
                            }
                        }
                    }
                }

                if can_equip {
                    // Identify the item
                    if target == *player_entity {
                        identified
                            .insert(
                                target,
                                IdentifiedItem {
                                    name: names.get(useitem.item).unwrap().name.clone(),
                                },
                            )
                            .expect("Unable to insert");
                        crate::gamelog::Logger::new()
                            .append("You unequip")
                            .item_name(names.get(useitem.item).unwrap().name.clone())
                            .log();
                    }

                    for item in to_unequip.iter() {
                        equipped.remove(*item);
                        backpack
                            .insert(*item, InBackpack { owner: target })
                            .expect("Unable to insert backpack entry");
                    }

                    for le in log_entries.iter() {
                        crate::gamelog::Logger::new()
                            .color(rltk::BLUE)
                            .append(le.to_string())
                            .log();
                    }

                    // Wield the item
                    equipped
                        .insert(
                            useitem.item,
                            Equipped {
                                owner: target,
                                slot: target_slot,
                            },
                        )
                        .expect("Unable to insert equipped component");
                    backpack.remove(useitem.item);
                    if target == *player_entity {
                        crate::gamelog::Logger::new()
                            .append("You equip ")
                            .item_name(&names.get(useitem.item).unwrap().name)
                            .log();
                    }
                }
                // Done with item
                remove_use.push(target);
            }
        }

        remove_use.iter().for_each(|e| {
            dirty
                .insert(*e, EquipmentChanged {})
                .expect("Unable to insert");
            wants_use.remove(*e).expect("Unable to remove");
        });
    }
}
