#![allow(unused)]

use super::{
    BlocksTile, BlocksVisibility,  Door, EntityMoved, GameLog, HungerClock, HungerState,
    Item, Map,  Player, Pools, Position, Renderable, RunState, State, TileType, 
    Viewshed, WantsToMelee, WantsToPickupItem, Name, Faction, data::Reaction, Vendor, VendorMode, WantsToCastSpell, Equipped, Weapon, Target
};
use rltk::{console, Point, Rltk, VirtualKeyCode, BEvent};
use rltk::prelude::INPUT;
use specs::prelude::*;

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) -> RunState {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let entities = ecs.entities();
    let combat_stats = ecs.read_storage::<Pools>();
    let map = ecs.fetch::<Map>();
    let items = ecs.read_storage::<Item>();
    
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let mut entity_moved = ecs.write_storage::<EntityMoved>();

    let mut doors = ecs.write_storage::<Door>();
    let mut blocks_visibility = ecs.write_storage::<BlocksVisibility>();
    let mut blocks_movement = ecs.write_storage::<BlocksTile>();
    let mut renderables = ecs.write_storage::<Renderable>();
    let factions = ecs.read_storage::<Faction>();
    let vendors = ecs.read_storage::<Vendor>();

    let mut result = RunState::AwaitingInput;

    let mut swap_entities: Vec<(Entity, i32, i32)> = Vec::new();

    for (entity, _player, pos, viewshed) in
        (&entities, &players, &mut positions, &mut viewsheds).join()
    {
        //map bounds
        if pos.x + delta_x < 1
            || pos.x + delta_x > map.width - 1
            || pos.y + delta_y < 1
            || pos.y + delta_y > map.height - 1
        {
            return RunState::AwaitingInput;
        }

        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        result = crate::spatial::for_each_tile_content_with_gamemode(destination_idx, |potential_target| {
            
            if let Some(_vendor) = vendors.get(potential_target) {
                return Some(RunState::ShowVendor{ vendor: potential_target, mode : VendorMode::Sell });
            }
            
            //handle bystanders: swap positions instead of attacking
            let mut hostile = true;
            if combat_stats.get(potential_target).is_some() {
                if let Some(faction) = factions.get(potential_target) {
                    let reaction = crate::data::faction_reaction(
                        &faction.name, 
                        "Player", 
                        &crate::data::DATA.lock().unwrap()
                    );
                    if reaction != Reaction::Attack { hostile = false; }
                }
            }        

            if !hostile {
                // Note that we want to move the nonhostile mob
                swap_entities.push((potential_target, pos.x, pos.y));

                // Move the player:
                //check bounds
                pos.x = (pos.x + delta_x).clamp(0, map.width - 1);
                pos.y = (pos.y + delta_y).clamp(0, map.height - 1);
                
                //do the move
                entity_moved
                    .insert(entity, EntityMoved {})
                    .expect("Unable to insert marker");

                viewshed.dirty = true;
                let mut ppos = ecs.write_resource::<Point>();
                ppos.x = pos.x;
                ppos.y = pos.y;
                return Some(RunState::Ticking);
                
            } else { //hostile
                let target = combat_stats.get(potential_target);
                if let Some(_target) = target {
                    wants_to_melee
                        .insert(
                            entity,
                            WantsToMelee {
                                target: potential_target,
                            },
                        )
                        .expect("Add target failed");
                    return Some(RunState::Ticking);
                }
            }

            let door = doors.get_mut(potential_target);
            if let Some(door) = door {
                door.open = true;
                blocks_visibility.remove(potential_target);
                blocks_movement.remove(potential_target);
                let glyph = renderables.get_mut(potential_target).unwrap();
                glyph.glyph = rltk::to_cp437('\\');
                viewshed.dirty = true;
                return Some(RunState::Ticking);
            }
            None
        });  //end for_each_tile_content_with_gamemode

        if !crate::spatial::is_blocked(destination_idx)  {
            let old_idx = map.xy_idx(pos.x, pos.y);
            pos.x = (pos.x + delta_x).clamp(0, map.width - 1);
            pos.y = (pos.y + delta_y).clamp(0, map.height - 1);
            let new_idx = map.xy_idx(pos.x, pos.y);

            entity_moved
                .insert(entity, EntityMoved {})
                .expect("Unable to insert marker");
            crate::spatial::move_entity(entity, old_idx, new_idx);

            viewshed.dirty = true;
            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;

            result = RunState::Ticking;
            match map.tiles[destination_idx] {
                TileType::DownStairs => result = RunState::NextLevel,
                TileType::UpStairs => result = RunState::PreviousLevel,
                _ => {}
            }
        }
    }
    
    //list item contents of new space
    {
        let names= ecs.read_storage::<Name>();
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        let mut target_item: Vec<Option<Entity>> = Vec::new();
        let mut ppos = ecs.write_resource::<Point>();

        for (item_entity, _item, position) in (&entities, &items, &positions).join() {
            if position.x == ppos.x && position.y == ppos.y {
                target_item.push( Some(item_entity));
            }
        }

        let mut item_string: String = String::new();
        for item in target_item.iter()
        {
            if item.is_some() {
                //item_string.push_str(&names.get(item.expect("target_item not found")).unwrap().name);
                item_string.push_str(&super::gui::get_item_display_name(ecs, item.unwrap()));
                
                item_string.push_str(", ")
            }
        }
        if !item_string.is_empty(){
            item_string.truncate(item_string.len()-2);
            gamelog.entries.push(format!("You see here: {}", item_string));
        }

    }

    //swap positions
    for m in swap_entities.iter() {
        let their_pos = positions.get_mut(m.0);
        if let Some(their_pos) = their_pos {
            let old_idx = map.xy_idx(their_pos.x, their_pos.y);
            their_pos.x = m.1;
            their_pos.y = m.2;
            let new_idx = map.xy_idx(their_pos.x, their_pos.y);
            crate::spatial::move_entity(m.0, old_idx, new_idx);
            result = RunState::Ticking;
        }
    }

    result
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog
            .entries
            .push("There is nothing here to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup
                .insert(
                    *player_entity,
                    WantsToPickupItem {
                        collected_by: *player_entity,
                        item,
                    },
                )
                .expect("Unable to insert want to pickup");
        }
    }
}

pub fn try_next_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::DownStairs {
        true
    } else {
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog
            .entries
            .push("There is no way down from here.".to_string());
        false
    }
}

pub fn try_previous_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::UpStairs {
        true
    } else {
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog.entries.push("There is no way up from here.".to_string());
        false
    }
}


pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {

    let mut input = INPUT.lock();
    
    input.for_each_message(|event| {
        if event == BEvent::CloseRequested {
            ctx.quitting = true;
        }
    });

    // Hotkeys
    let modifier = if cfg!(unix) {input.key_pressed_set().contains(&VirtualKeyCode::LControl) 
        || input.key_pressed_set().contains(&VirtualKeyCode::RControl)
    } else {input.key_pressed_set().contains(&VirtualKeyCode::LShift) 
            || input.key_pressed_set().contains(&VirtualKeyCode::RShift)
    };

    if modifier && ctx.key.is_some() {
        let key: Option<i32> = match ctx.key.unwrap() {
            VirtualKeyCode::Key1 => Some(1),
            VirtualKeyCode::Key2 => Some(2),
            VirtualKeyCode::Key3 => Some(3),
            VirtualKeyCode::Key4 => Some(4),
            VirtualKeyCode::Key5 => Some(5),
            VirtualKeyCode::Key6 => Some(6),
            VirtualKeyCode::Key7 => Some(7),
            VirtualKeyCode::Key8 => Some(8),
            VirtualKeyCode::Key9 => Some(9),
            _ => None,
        };
        if let Some(key) = key {
            return use_hotkey(gs, key - 1);
        }
    }

    // Player movement
    match ctx.key {
        None => return RunState::AwaitingInput, // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                return try_move_player(-1, 0, &mut gs.ecs);
            }

            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                return try_move_player(1, 0, &mut gs.ecs);
            }

            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                return try_move_player(0, -1, &mut gs.ecs);
            }

            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                return try_move_player(0, 1, &mut gs.ecs);
            }

            // Diagonals
            //NE
            VirtualKeyCode::Numpad9 | VirtualKeyCode::U => return try_move_player(1, -1, &mut gs.ecs),
            //NW
            VirtualKeyCode::Numpad7 | VirtualKeyCode::Y => return try_move_player(-1, -1, &mut gs.ecs),
            //SE
            VirtualKeyCode::Numpad3 | VirtualKeyCode::N => return try_move_player(1, 1, &mut gs.ecs),
            //SW
            VirtualKeyCode::Numpad1 | VirtualKeyCode::B => return try_move_player(-1, 1, &mut gs.ecs),

            // Level changes
            VirtualKeyCode::Period => {
                if try_next_level(&mut gs.ecs) {
                    return RunState::NextLevel;
                }
            }
            VirtualKeyCode::Comma => {
                if try_previous_level(&mut gs.ecs) {
                    return RunState::PreviousLevel;
                }
            }
            

            // Skip Turn
            VirtualKeyCode::Numpad5 => return skip_turn(&mut gs.ecs),
            VirtualKeyCode::Space => return skip_turn(&mut gs.ecs),

            //non-movement
            VirtualKeyCode::G /*| VirtualKeyCode::Comma*/ => get_item(&mut gs.ecs),
            VirtualKeyCode::I => return RunState::ShowInventory,
            VirtualKeyCode::D => return RunState::ShowDropItem,
            VirtualKeyCode::R => return RunState::ShowRemoveItem,

            // Save and Quit
            VirtualKeyCode::Escape => return RunState::SaveGame,

            // Cheating!
            VirtualKeyCode::Backslash => return RunState::ShowCheatMenu,

            // Ranged
            VirtualKeyCode::V => {
                cycle_target(&mut gs.ecs);
                return RunState::AwaitingInput;
            },


            _ => return RunState::AwaitingInput,
        },
    }
    RunState::Ticking
}

fn use_hotkey(gs: &mut State, key: i32) -> RunState {
    use super::{Consumable, InBackpack, WantsToUseItem, KnownSpells};
    use super::data::find_spell_entity;


    let consumables = gs.ecs.read_storage::<Consumable>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    
    let player_entity = gs.ecs.fetch::<Entity>();
    let known_spells_storage = gs.ecs.read_storage::<KnownSpells>();
    let known_spells = &known_spells_storage.get(*player_entity).unwrap().spells;

    let entities = gs.ecs.entities();
    let mut carried_consumables = Vec::new();
    for (entity, carried_by, _consumable) in (&entities, &backpack, &consumables).join() {
        if carried_by.owner == *player_entity {
            carried_consumables.push(entity);
        }
    }

    if (key as usize) < carried_consumables.len() {
        use crate::components::Ranged;
        if let Some(ranged) = gs
            .ecs
            .read_storage::<Ranged>()
            .get(carried_consumables[key as usize])
        {
            return RunState::ShowTargeting {
                range: ranged.range,
                item: carried_consumables[key as usize],
            };
        }
        let mut intent = gs.ecs.write_storage::<WantsToUseItem>();
        intent
            .insert(
                *player_entity,
                WantsToUseItem {
                    item: carried_consumables[key as usize],
                    target: None,
                },
            )
            .expect("Unable to insert intent");
        return RunState::Ticking;
    } else if (key as usize) < (carried_consumables.len() + known_spells.len()) {
        let spell_key = (key as usize) - carried_consumables.len();
        let pools = gs.ecs.read_storage::<Pools>();
        let player_pools = pools.get(*player_entity).unwrap();
        if player_pools.mana.current >= known_spells[spell_key].mana_cost {
            if let Some(spell_entity) = find_spell_entity(&gs.ecs, &known_spells[spell_key].display_name) {
                use crate::components::Ranged;
                if let Some(ranged) = gs.ecs.read_storage::<Ranged>().get(spell_entity) {
                    return RunState::ShowTargeting{ range: ranged.range, item: spell_entity };
                };
                let mut intent = gs.ecs.write_storage::<WantsToCastSpell>();
                intent.insert(
                    *player_entity,
                    WantsToCastSpell{ spell: spell_entity, target: None }
                ).expect("Unable to insert intent");
                return RunState::Ticking;
            }
        } else {
            let mut gamelog = gs.ecs.fetch_mut::<GameLog>();
            gamelog.entries.push("You don't have enough mana to cast that.".to_string());
        }

    }


    RunState::Ticking
}

fn skip_turn(ecs: &mut World) -> RunState {
    let player_entity = ecs.fetch::<Entity>();
    let viewshed_components = ecs.read_storage::<Viewshed>();
    let factions = ecs.read_storage::<Faction>();

    let worldmap_resource = ecs.fetch::<Map>();

    let mut can_heal = true;
    let viewshed = viewshed_components.get(*player_entity).unwrap();
    for tile in viewshed.visible_tiles.iter() {
        let idx = worldmap_resource.xy_idx(tile.x, tile.y);
        crate::spatial::for_each_tile_content(idx, |entity_id| {
            let faction = factions.get(entity_id);
            match faction {
                None => {}
                Some(faction) => {
                    let reaction = crate::data::faction_reaction(
                        &faction.name,
                        "Player",
                        &crate::data::DATA.lock().unwrap()
                    );
                    if reaction == Reaction::Attack {
                        can_heal = false;
                    }
                }
            }
        });
        
    }
    let hunger_clocks = ecs.read_storage::<HungerClock>();
    let hc = hunger_clocks.get(*player_entity);
    if let Some(hc) = hc {
        match hc.state {
            HungerState::Hungry => can_heal = false,
            HungerState::Starving => can_heal = false,
            _ => {}
        }
    }

    if can_heal {
        let mut health_components = ecs.write_storage::<Pools>();
        let pools = health_components.get_mut(*player_entity).unwrap();
        pools.hit_points.current = i32::min(pools.hit_points.current + 1, pools.hit_points.max);
        let mut rng = ecs.fetch_mut::<rltk::RandomNumberGenerator>();
        if rng.roll_dice(1,6)==1 {
            pools.mana.current = i32::min(pools.mana.current + 1, pools.mana.max);
        }
    }
    

    RunState::Ticking
}

fn get_player_target_list(ecs : &mut World) -> Vec<(f32,Entity)> {
    let mut possible_targets : Vec<(f32,Entity)> = Vec::new();
    let viewsheds = ecs.read_storage::<Viewshed>();
    let player_entity = ecs.fetch::<Entity>();
    let equipped = ecs.read_storage::<Equipped>();
    let weapon = ecs.read_storage::<Weapon>();
    let map = ecs.fetch::<Map>();
    let positions = ecs.read_storage::<Position>();
    let factions = ecs.read_storage::<Faction>();
    for (equipped, weapon) in (&equipped, &weapon).join() {
        if equipped.owner == *player_entity && weapon.range.is_some() {
            let range = weapon.range.unwrap();

            if let Some(vs) = viewsheds.get(*player_entity) {
                let player_pos = positions.get(*player_entity).unwrap();
                for tile_point in vs.visible_tiles.iter() {
                    let tile_idx = map.xy_idx(tile_point.x, tile_point.y);
                    let distance_to_target = rltk::DistanceAlg::Pythagoras.distance2d(*tile_point, rltk::Point::new(player_pos.x, player_pos.y));
                    if distance_to_target < range as f32 {
                        crate::spatial::for_each_tile_content(tile_idx, |possible_target| {
                            if possible_target != *player_entity && factions.get(possible_target).is_some() {
                                possible_targets.push((distance_to_target, possible_target));
                            }
                        });
                    }
                }
            }
        }
    }

    possible_targets.sort_by(|a,b| a.0.partial_cmp(&b.0).unwrap());
    possible_targets
}

pub fn end_turn_targeting(ecs: &mut World) {
    let possible_targets = get_player_target_list(ecs);
    let mut targets = ecs.write_storage::<Target>();
    targets.clear();

    if !possible_targets.is_empty() {
        targets.insert(possible_targets[0].1, Target{}).expect("Insert fail");
    }
}

fn cycle_target(ecs: &mut World) {
    let possible_targets = get_player_target_list(ecs);
    let mut targets = ecs.write_storage::<Target>();
    let entities = ecs.entities();
    let mut current_target : Option<Entity> = None;

    for (e,_t) in (&entities, &targets).join() {
        current_target = Some(e);
    }

    targets.clear();
    if let Some(current_target) = current_target {
        if !possible_targets.len() > 1 {
            let mut index = 0;
            for (i, target) in possible_targets.iter().enumerate() {
                if target.1 == current_target {
                    index = i;
                }
            }

            if index > possible_targets.len()-2 {
                targets.insert(possible_targets[0].1, Target{});
            } else {
                targets.insert(possible_targets[index+1].1, Target{});
            }
        }
    }
}
