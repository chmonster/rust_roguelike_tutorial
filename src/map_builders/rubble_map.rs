use super::{
    apply_room_to_map, spawner, Map, MapBuilder, Position, Rect, TileType, SHOW_MAPGEN_VISUALIZER,
};
use rltk::{console, RandomNumberGenerator};
use specs::prelude::*;

pub const RUBBLE: usize = 80 * 43 / 3;
pub const TOP_STAIRS: usize = 25;

pub struct RubbleMapBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    room: Rect,
    history: Vec<Map>,
}

impl MapBuilder for RubbleMapBuilder {
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn build_map(&mut self) {
        self.rubble_map();
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        console::log("spawn_entities");
        spawner::spawn_room(ecs, &self.map, &self.room, self.depth);
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

impl RubbleMapBuilder {
    pub fn new(new_depth: i32) -> RubbleMapBuilder {
        RubbleMapBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            room: Rect::new(0, 0, 0, 0),
            history: Vec::new(),
        }
    }

    /// Makes a map with solid boundaries and 400 randomly placed walls. No guarantees that it won't
    /// look magnificent.
    fn rubble_map(&mut self) {
        self.room = Rect::new(0, 0, self.map.width - 2, self.map.height - 2);

        let mut rng = RandomNumberGenerator::new();

        let (player_x, player_y) = self.room.center();

        apply_room_to_map(&mut self.map, &self.room);
        self.take_snapshot();

        // Now we'll randomly splat a bunch of walls. It won't be pretty, but it's a decent illustration.
        // First, obtain the thread-local RNG:

        for i in 0..RUBBLE {
            let x = rng.roll_dice(1, self.map.width - 1);
            let y = rng.roll_dice(1, self.map.height - 1);
            let idx = self.map.xy_idx(x, y);

            if idx != self.map.xy_idx(player_x, player_y) {
                if i > RUBBLE - TOP_STAIRS {
                    self.map.tiles[idx] = TileType::DownStairs;
                } else {
                    self.map.tiles[idx] = TileType::Wall;
                }
                self.take_snapshot();
            }
        }

        self.starting_position = Position {
            x: player_x,
            y: player_y,
        }
    }
}
