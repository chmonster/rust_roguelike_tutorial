use super::{
    apply_horizontal_tunnel, apply_room_to_map, apply_vertical_tunnel, BuilderMap,
    InitialMapBuilder, Rect, TileType, MAPHEIGHT, MAPWIDTH,
};
use rltk::{console, RandomNumberGenerator};

pub const RUBBLE: usize = MAPHEIGHT * MAPWIDTH / 3;
pub const TOP_STAIRS: usize = 25;

pub struct RubbleMapBuilder {}

impl InitialMapBuilder for RubbleMapBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        console::log("RubbleMapBuilder");
        self.rubble_map(rng, build_data);
    }
}

impl RubbleMapBuilder {
    pub fn new() -> Box<RubbleMapBuilder> {
        Box::new(RubbleMapBuilder {})
    }

    /// Makes a map with solid boundaries and 400 randomly placed walls. No guarantees that it won't
    /// look magnificent.
    fn rubble_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let room = Rect::new(0, 0, build_data.map.width - 2, build_data.map.height - 2);

        //let mut rng = RandomNumberGenerator::new();

        //let (player_x, player_y) = self.room.center();

        apply_room_to_map(&mut build_data.map, &room);
        build_data.take_snapshot();

        // Now we'll randomly splat a bunch of walls.
        for i in 0..RUBBLE {
            let x = rng.roll_dice(1, build_data.map.width - 1);
            let y = rng.roll_dice(1, build_data.map.height - 1);
            let idx = build_data.map.xy_idx(x, y);

            //if idx != self.map.xy_idx(player_x, player_y) {
            if i > RUBBLE - TOP_STAIRS {
                build_data.map.tiles[idx] = TileType::DownStairs;
            } else {
                build_data.map.tiles[idx] = TileType::Wall;
            }
            if i % 25 == 0 {
                build_data.take_snapshot();
            }
            //}
        }
        let rooms: Vec<Rect> = vec![room];
        build_data.take_snapshot();

        build_data.rooms = Some(rooms);
    }
}
