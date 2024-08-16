use super::{/*Map,*/ Monster, Name, /*Position,*/ Viewshed};
use rltk::{console, /*field_of_view,*/ Point};
use specs::prelude::*;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        ReadExpect<'a, Point>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_pos, viewshed, monster, name) = data;

        for (viewshed, _monster, name) in (&viewshed, &monster, &name).join() {
            if viewshed.visible_tiles.contains(&*player_pos) {
                console::log(&format!("{} shouts insults", name.name));
            } else {
                console::log(&format!("{} considers their own existence", name.name));
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
