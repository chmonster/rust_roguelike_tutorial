use super::{Position, RandomMover};
use specs::prelude::*;

//use rltk::{RandomNumberGenerator, Rltk, RGB};

pub struct RandomWalker {}

impl<'a> System<'a> for RandomWalker {
    type SystemData = (ReadStorage<'a, RandomMover>, WriteStorage<'a, Position>);

    fn run(&mut self, (randy, mut pos): Self::SystemData) {
        let mut rng = rltk::RandomNumberGenerator::new();
        for (_randy, pos) in (&randy, &mut pos).join() {
            let dir = rng.roll_dice(1, 4);
            let delta_x = if dir % 2 == 1 {
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
            };

            // let destination_idx = xy_idx(pos.x + delta_x, pos.y + delta_y);
            // if map[destination_idx] != TileType::Wall {
            pos.x = (pos.x + delta_x).clamp(0, 79);
            pos.y = (pos.y + delta_y).clamp(0, 49);
            //}
        }
    }
}
