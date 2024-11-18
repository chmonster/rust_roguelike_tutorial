use super::{BuilderMap, MetaMapBuilder, TileType};
use rltk::{console, RandomNumberGenerator};

pub struct RoomBasedStairs {}

impl MetaMapBuilder for RoomBasedStairs {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        //console::log("RoomBasedStairs");
        self.build(rng, build_data);
    }
}

impl RoomBasedStairs {
    #[allow(dead_code)]
    pub fn new() -> Box<RoomBasedStairs> {
        Box::new(RoomBasedStairs {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        if let Some(rooms) = &build_data.rooms {
            //let stairs_position = rooms[rooms.len() - 1].center();
            let final_room = rooms[rooms.len() - 1];
            let stairs_position = (
                rng.range(final_room.x1, final_room.x2),
                rng.range(final_room.y1, final_room.y2),
            );
            let stairs_idx = build_data.map.xy_idx(stairs_position.0, stairs_position.1);
            build_data.map.tiles[stairs_idx] = TileType::DownStairs;
            build_data.take_snapshot();
        } else {
            panic!("Room Based Stairs only works after rooms have been created");
        }
    }
}
