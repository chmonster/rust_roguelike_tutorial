//#![allow(unused)]

use crate::{
    particle_system::ParticleBuilder, Confusion, EntityMoved, GameLog, Map, Monster, MyTurn, Name,
    Position, Quips, RunState, Viewshed, WantsToMelee,
};
use rltk::Point; //{Point, RandomNumberGenerator};
use specs::prelude::*;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, MyTurn>,
        WriteStorage<'a, Confusion>,
        WriteExpect<'a, ParticleBuilder>,
        WriteStorage<'a, EntityMoved>,
        WriteStorage<'a, Quips>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        WriteExpect<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_pos,
            player_entity,
            runstate,
            entities,
            mut viewshed,
            monster,
            names,
            mut position,
            mut wants_to_melee,
            turns,
            mut confused,
            mut particle_builder,
            mut entity_moved,
            mut quips,
            mut rng,
            mut gamelog,
        ) = data;

        if *runstate != RunState::Ticking {
            return;
        }

        for (entity, viewshed, _monster, pos, _turn) in
            (&entities, &mut viewshed, &monster, &mut position, &turns).join()
        {
            let mut can_act = true;

            // Possibly quip
            let quip = quips.get_mut(entity);
            if let Some(quip) = quip {
                if !quip.available.is_empty()
                    && viewshed.visible_tiles.contains(&player_pos)
                    && rng.roll_dice(1, 6) == 1
                {
                    let name = names.get(entity);
                    let quip_index = if quip.available.len() == 1 {
                        0
                    } else {
                        (rng.roll_dice(1, quip.available.len() as i32) - 1) as usize
                    };
                    gamelog.entries.push(format!(
                        "{} growls \"{}\"",
                        name.unwrap().name,
                        quip.available[quip_index]
                    ));
                    quip.available.remove(quip_index);
                }
            }

            let is_confused = confused.get_mut(entity);
            if let Some(i_am_confused) = is_confused {
                i_am_confused.turns -= 1;
                if i_am_confused.turns < 1 {
                    confused.remove(entity);
                }
                can_act = false;
                particle_builder.request(
                    pos.x,
                    pos.y,
                    rltk::RGB::named(rltk::MAGENTA),
                    rltk::RGB::named(rltk::BLACK),
                    rltk::to_cp437('?'),
                    200.0,
                );
            }

            if can_act {
                let distance =
                    rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
                if distance < 1.5 {
                    wants_to_melee
                        .insert(
                            entity,
                            WantsToMelee {
                                target: *player_entity,
                            },
                        )
                        .expect("Unable to insert attack");
                } else if viewshed.visible_tiles.contains(&*player_pos) {
                    // Path to the player
                    let path = rltk::a_star_search(
                        map.xy_idx(pos.x, pos.y),
                        map.xy_idx(player_pos.x, player_pos.y),
                        &*map,
                    );
                    if path.success && path.steps.len() > 1 {
                        let mut idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = false;
                        pos.x = path.steps[1] as i32 % map.width;
                        pos.y = path.steps[1] as i32 / map.width;
                        entity_moved
                            .insert(entity, EntityMoved {})
                            .expect("Unable to insert marker");
                        idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = true;
                        viewshed.dirty = true;
                    }
                }
            }
        }
    }
}

/*
//use rltk::{RandomNumberGenerator, Rltk, RGB};

pub struct RandomWalker {}

fn roll_to_deltas(dir: i32) -> (i32, i32) {
    match (dir) {
        1 => (1, 0),
        2 => (0, 1),
        3 => (-1, 0),
        4 => (0, -1),
        _ => (0, 0),
    }
}

impl<'a> System<'a> for RandomWalker {
    type SystemData = (ReadStorage<'a, RandomMover>, WriteStorage<'a, Position>);

    fn run(&mut self, (randy, mut pos): Self::SystemData) {
        let mut rng = rltk::RandomNumberGenerator::new();
        for (_randy, pos) in (&randy, &mut pos).join() {
            let dir = rng.roll_dice(1, 4);
            let (delta_x, delta_y) = roll_to_deltas(dir);
            /*let delta_x = if dir % 2 == 1 {
                if dir == 1 {
                    1
                } else {
                    -1
                }
            } else {
                0
            };
            let delta_y = if dir % 2 == 0 {
                if dir == 2 {
                    1
                } else {
                    -1
                }
            } else {
                0
            };*/

            // let destination_idx = xy_idx(pos.x + delta_x, pos.y + delta_y);
            // if map[destination_idx] != TileType::Wall {
            pos.x = (pos.x + delta_x).clamp(0, 79);
            pos.y = (pos.y + delta_y).clamp(0, 49);
            //}
        }
    }
}
 */
