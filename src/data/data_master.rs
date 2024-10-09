use super::{Data, RandomTable};
use crate::components::*;
use specs::prelude::*;
use std::collections::{HashMap, HashSet};

pub enum SpawnType {
    AtPosition { x: i32, y: i32 },
}

pub struct DataMaster {
    data: Data,
    item_index: HashMap<String, usize>,
    mob_index: HashMap<String, usize>,
    prop_index: HashMap<String, usize>,
}

impl DataMaster {
    pub fn empty() -> DataMaster {
        DataMaster {
            data: Data {
                items: Vec::new(),
                mobs: Vec::new(),
                props: Vec::new(),
                spawn_table: Vec::new(),
            },
            item_index: HashMap::new(),
            mob_index: HashMap::new(),
            prop_index: HashMap::new(),
        }
    }

    pub fn load(&mut self, data: Data) {
        self.data = data;
        self.item_index = HashMap::new();
        let mut used_names: HashSet<String> = HashSet::new();
        for (i, item) in self.data.items.iter().enumerate() {
            if used_names.contains(&item.name) {
                rltk::console::log(format!(
                    "WARNING -  duplicate item name in data [{}]",
                    item.name
                ));
            }
            self.item_index.insert(item.name.clone(), i);
            used_names.insert(item.name.clone());
        }
        for (i, mob) in self.data.mobs.iter().enumerate() {
            if used_names.contains(&mob.name) {
                rltk::console::log(format!(
                    "WARNING -  duplicate mob name in data [{}]",
                    mob.name
                ));
            }
            self.mob_index.insert(mob.name.clone(), i);
            used_names.insert(mob.name.clone());
        }
        for (i, prop) in self.data.props.iter().enumerate() {
            if used_names.contains(&prop.name) {
                rltk::console::log(format!(
                    "WARNING -  duplicate prop name in data [{}]",
                    prop.name
                ));
            }
            self.prop_index.insert(prop.name.clone(), i);
            used_names.insert(prop.name.clone());
        }

        for spawn in self.data.spawn_table.iter() {
            if !used_names.contains(&spawn.name) {
                rltk::console::log(format!(
                    "WARNING - Spawn tables references unspecified entity {}",
                    spawn.name
                ));
            }
        }
    }
}

fn spawn_position(pos: SpawnType, new_entity: EntityBuilder) -> EntityBuilder {
    let mut eb = new_entity;

    // Spawn in the specified location
    match pos {
        SpawnType::AtPosition { x, y } => {
            eb = eb.with(Position { x, y });
        }
    }
    eb
}

fn get_renderable_component(
    renderable: &super::item_structs::Renderable,
) -> crate::components::Renderable {
    crate::components::Renderable {
        glyph: rltk::to_cp437(renderable.glyph.chars().next().unwrap()),
        fg: rltk::RGB::from_hex(&renderable.fg).expect("Invalid RGB"),
        bg: rltk::RGB::from_hex(&renderable.bg).expect("Invalid RGB"),
        render_order: renderable.order,
    }
}

pub fn get_spawn_table_for_depth(data: &DataMaster, depth: i32) -> RandomTable {
    use super::SpawnTableEntry;

    let available_options: Vec<&SpawnTableEntry> = data
        .data
        .spawn_table
        .iter()
        .filter(|a| depth >= a.min_depth && depth <= a.max_depth)
        .collect();

    let mut rt = RandomTable::new();
    for e in available_options.iter() {
        let mut weight = e.weight;
        if e.add_map_depth_to_weight.is_some() {
            weight += depth;
        }
        rt = rt.add(e.name.clone(), weight);
    }

    rt
}

pub fn spawn_named_entity(
    data: &DataMaster,
    new_entity: EntityBuilder,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if data.item_index.contains_key(key) {
        return spawn_named_item(data, new_entity, key, pos);
    } else if data.mob_index.contains_key(key) {
        return spawn_named_mob(data, new_entity, key, pos);
    } else if data.prop_index.contains_key(key) {
        return spawn_named_prop(data, new_entity, key, pos);
    }

    None
}

pub fn spawn_named_item(
    data: &DataMaster,
    new_entity: EntityBuilder,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if data.item_index.contains_key(key) {
        let item_template = &data.data.items[data.item_index[key]];

        let mut eb = new_entity;

        // Spawn in the specified location
        eb = spawn_position(pos, eb);

        // Renderable
        if let Some(renderable) = &item_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }

        eb = eb.with(Name {
            name: item_template.name.clone(),
        });

        eb = eb.with(crate::components::Item {});

        if let Some(consumable) = &item_template.consumable {
            eb = eb.with(crate::components::Consumable {});
            for effect in consumable.effects.iter() {
                let effect_name = effect.0.as_str();
                match effect_name {
                    "provides_healing" => {
                        eb = eb.with(ProvidesHealing {
                            heal_amount: effect.1.parse::<i32>().unwrap(),
                        })
                    }
                    "ranged" => {
                        eb = eb.with(Ranged {
                            range: effect.1.parse::<i32>().unwrap(),
                        })
                    }
                    "damage" => {
                        eb = eb.with(InflictsDamage {
                            damage: effect.1.parse::<i32>().unwrap(),
                        })
                    }
                    "area_of_effect" => {
                        eb = eb.with(AreaOfEffect {
                            radius: effect.1.parse::<i32>().unwrap(),
                        })
                    }
                    "confusion" => {
                        eb = eb.with(Confusion {
                            turns: effect.1.parse::<i32>().unwrap(),
                        })
                    }
                    "magic_mapping" => eb = eb.with(MagicMapper {}),
                    "food" => eb = eb.with(ProvidesFood {}),
                    _ => {
                        rltk::console::log(format!(
                            "Warning: consumable effect {} not implemented.",
                            effect_name
                        ));
                    }
                }
            }
        }

        if let Some(weapon) = &item_template.weapon {
            eb = eb.with(Equippable {
                slot: EquipmentSlot::Melee,
            });
            eb = eb.with(MeleePowerBonus {
                power: weapon.power_bonus,
            });
        }

        if let Some(shield) = &item_template.shield {
            eb = eb.with(Equippable {
                slot: EquipmentSlot::Shield,
            });
            eb = eb.with(DefenseBonus {
                defense: shield.defense_bonus,
            });
        }

        return Some(eb.build());
    }
    None
}

pub fn spawn_named_mob(
    data: &DataMaster,
    new_entity: EntityBuilder,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if data.mob_index.contains_key(key) {
        let mob_template = &data.data.mobs[data.mob_index[key]];

        let mut eb = new_entity;

        // Spawn in the specified location
        eb = spawn_position(pos, eb);

        // Renderable
        if let Some(renderable) = &mob_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }

        if let Some(quips) = &mob_template.quips {
            eb = eb.with(Quips {
                available: quips.clone(),
            });
        }

        eb = eb.with(Name {
            name: mob_template.name.clone(),
        });

        match mob_template.ai.as_ref() {
            "melee" => eb = eb.with(Monster {}),
            "bystander" => eb = eb.with(Bystander {}),
            "vendor" => eb = eb.with(Vendor {}),

            _ => {}
        }

        if mob_template.blocks_tile {
            eb = eb.with(BlocksTile {});
        }
        eb = eb.with(CombatStats {
            max_hp: mob_template.stats.max_hp,
            hp: mob_template.stats.hp,
            power: mob_template.stats.power,
            defense: mob_template.stats.defense,
        });
        eb = eb.with(Viewshed {
            visible_tiles: Vec::new(),
            range: mob_template.vision_range,
            dirty: true,
        });

        return Some(eb.build());
    }
    None
}

pub fn spawn_named_prop(
    data: &DataMaster,
    new_entity: EntityBuilder,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if data.prop_index.contains_key(key) {
        let prop_template = &data.data.props[data.prop_index[key]];

        let mut eb = new_entity;

        // Spawn in the specified location
        eb = spawn_position(pos, eb);

        // Renderable
        if let Some(renderable) = &prop_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }

        eb = eb.with(Name {
            name: prop_template.name.clone(),
        });

        if let Some(hidden) = prop_template.hidden {
            if hidden {
                eb = eb.with(Hidden {})
            };
        }
        if let Some(blocks_tile) = prop_template.blocks_tile {
            if blocks_tile {
                eb = eb.with(BlocksTile {})
            };
        }
        if let Some(blocks_visibility) = prop_template.blocks_visibility {
            if blocks_visibility {
                eb = eb.with(BlocksVisibility {})
            };
        }
        if let Some(door_open) = prop_template.door_open {
            eb = eb.with(Door { open: door_open });
        }
        if let Some(entry_trigger) = &prop_template.entry_trigger {
            eb = eb.with(EntryTrigger {});
            for effect in entry_trigger.effects.iter() {
                match effect.0.as_str() {
                    "damage" => {
                        eb = eb.with(InflictsDamage {
                            damage: effect.1.parse::<i32>().unwrap(),
                        })
                    }
                    "single_activation" => eb = eb.with(SingleActivation {}),
                    _ => {}
                }
            }
        }

        return Some(eb.build());
    }
    None
}
