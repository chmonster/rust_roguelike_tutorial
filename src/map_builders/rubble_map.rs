use super::{
    apply_room_to_map, spawner, Map, MapBuilder, Position, Rect, TileType, SHOW_MAPGEN_VISUALIZER,
};
use rltk::{/*console,*/ RandomNumberGenerator};
use specs::prelude::*;

pub const RUBBLE: usize = 80 * 43 / 3;
pub const TOP_STAIRS: usize = 25;

pub struct RubbleMapBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    room: Rect,
    history: Vec<Map>,
    spawn_list: Vec<(usize, String)>,
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

    fn get_spawn_list(&self) -> &Vec<(usize, String)> {
        &self.spawn_list
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
            spawn_list: Vec::new(),
        }
    }

    /// Makes a map with solid boundaries and 400 randomly placed walls. No guarantees that it won't
    /// look magnificent.
    fn rubble_map(&mut self) {
        self.room = Rect::new(0, 0, self.map.width - 2, self.map.height - 2);

        let mut rng = RandomNumberGenerator::new();

        //let (player_x, player_y) = self.room.center();

        apply_room_to_map(&mut self.map, &self.room);
        self.take_snapshot();

        // Now we'll randomly splat a bunch of walls.
        for i in 0..RUBBLE {
            let x = rng.roll_dice(1, self.map.width - 1);
            let y = rng.roll_dice(1, self.map.height - 1);
            let idx = self.map.xy_idx(x, y);

            //if idx != self.map.xy_idx(player_x, player_y) {
            if i > RUBBLE - TOP_STAIRS {
                self.map.tiles[idx] = TileType::DownStairs;
            } else {
                self.map.tiles[idx] = TileType::Wall;
            }
            if i % 25 == 0 {
                self.take_snapshot();
            }
            //}
        }
        self.take_snapshot();

        //routines taken from cellular_automata

        // Find a starting point; start at the middle and walk left until we find an open tile
        self.starting_position = Position {
            x: self.map.width / 2,
            y: self.map.height / 2,
        };
        let mut start_idx = self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y);
        while self.map.tiles[start_idx] != TileType::Floor {
            self.starting_position.x -= 1;
            start_idx = self
                .map
                .xy_idx(self.starting_position.x, self.starting_position.y);
        }

        // Find all tiles we can reach from the starting point
        let map_starts: Vec<usize> = vec![start_idx];
        let dijkstra_map = rltk::DijkstraMap::new(
            self.map.width,
            self.map.height,
            &map_starts,
            &self.map,
            200.0,
        );
        let mut exit_tile = (0, 0.0f32);
        for (i, tile) in self.map.tiles.iter_mut().enumerate() {
            if *tile == TileType::Floor {
                let distance_to_start = dijkstra_map.map[i];
                // We can't get to this tile - so we'll make it a wall
                if distance_to_start == f32::MAX {
                    *tile = TileType::Wall;
                } else {
                    // If it is further away than our current exit candidate, move the exit
                    if distance_to_start > exit_tile.1 {
                        exit_tile.0 = i;
                        exit_tile.1 = distance_to_start;
                    }
                }
            }
        }
        self.take_snapshot();

        self.map.tiles[exit_tile.0] = TileType::DownStairs;
        self.take_snapshot();

        spawner::spawn_room(
            &self.map,
            &mut rng,
            &self.room,
            self.depth,
            &mut self.spawn_list,
        );
    }
}
