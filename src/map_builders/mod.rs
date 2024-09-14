use super::{spawner, Map, Position, Rect, TileType, SHOW_MAPGEN_VISUALIZER};
//use rltk::RandomNumberGenerator;
mod simple_map;
use simple_map::SimpleMapBuilder;
mod rubble_map;
use rubble_map::RubbleMapBuilder;
mod bsp_dungeon;
use bsp_dungeon::BspDungeonBuilder;
mod common;
use common::*;
use specs::prelude::*;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    // Note that until we have a second map type, this isn't even slightly random
    let mut rng = rltk::RandomNumberGenerator::new();
    let type_roll = rng.roll_dice(1, 1);
    match type_roll {
        1 => Box::new(BspDungeonBuilder::new(new_depth)),
        2 => Box::new(RubbleMapBuilder::new(new_depth)),
        _ => Box::new(SimpleMapBuilder::new(new_depth)),
    }

    //Box::new(SimpleMapBuilder::new(new_depth))
}
