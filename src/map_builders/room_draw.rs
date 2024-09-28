use super::{BuilderMap, MetaMapBuilder, Rect, TileType};
use rltk::{console, RandomNumberGenerator};

pub struct RoomDrawer {}

impl MetaMapBuilder for RoomDrawer {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        console::log("RoomDrawer");
        self.build(rng, build_data);
    }
}

impl RoomDrawer {
    #[allow(dead_code)]
    pub fn new() -> Box<RoomDrawer> {
        Box::new(RoomDrawer {})
    }

    fn rectangle(&mut self, build_data: &mut BuilderMap, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                let idx = build_data.map.xy_idx(x, y);
                if idx > 0 && idx < ((build_data.map.width * build_data.map.height) - 1) as usize {
                    build_data.map.tiles[idx] = TileType::Floor;
                }
            }
        }
    }

    fn rhombus(
        &mut self,
        build_data: &mut BuilderMap,
        room: &Rect,
        rng: &mut RandomNumberGenerator,
    ) {
        let x_ang = rng.range(2, 6) * if rng.roll_dice(1, 2) == 2 { -1 } else { 1 };
        let x_offset = (room.y2 - room.y1) / x_ang;
        let x_start = if x_ang > 0 {
            room.x1
        } else {
            room.x1 - x_offset
        };
        let x_end = if x_ang > 0 {
            room.x2 - x_offset
        } else {
            room.x2
        };

        let y_ang = rng.range(2, 6) * if rng.roll_dice(1, 2) == 2 { -1 } else { 1 };
        let y_offset = (room.x2 - room.x1) / y_ang;
        let y_start = if y_ang > 0 {
            room.y1
        } else {
            room.y1 - y_offset
        };
        let y_end = if y_ang > 0 {
            room.y2 - y_offset
        } else {
            room.y2
        };

        for y in y_start..=y_end {
            let dx = (y - y_start) / x_ang;
            for x in x_start..=x_end {
                let dy = (x - x_start) / y_ang;
                //let dx = 0;
                let idx = build_data.map.xy_idx(x + dx, y + dy);
                //console::log(format!("to place: {} {} {}", x + dx, y + dy, idx));
                if idx > 0 && idx < ((build_data.map.width * build_data.map.height) - 1) as usize {
                    build_data.map.tiles[idx] = TileType::Floor;
                }
            }
        }
    }

    fn circle(&mut self, build_data: &mut BuilderMap, room: &Rect) {
        let radius = i32::min(room.x2 - room.x1, room.y2 - room.y1) as f32 / 2.0;
        let center = room.center();
        let center_pt = rltk::Point::new(center.0, center.1);
        for y in room.y1..=room.y2 {
            for x in room.x1..=room.x2 {
                let idx = build_data.map.xy_idx(x, y);
                let distance =
                    rltk::DistanceAlg::Pythagoras.distance2d(center_pt, rltk::Point::new(x, y));
                if idx > 0
                    && idx < ((build_data.map.width * build_data.map.height) - 1) as usize
                    && distance <= radius
                {
                    build_data.map.tiles[idx] = TileType::Floor;
                }
            }
        }
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let rooms: Vec<Rect>;
        if let Some(rooms_builder) = &build_data.rooms {
            rooms = rooms_builder.clone();
        } else {
            panic!("Room Drawing require a builder with room structures");
        }

        for room in rooms.iter() {
            let room_type = rng.roll_dice(1, 5);
            match room_type {
                1 => self.circle(build_data, room),
                2 => self.rhombus(build_data, room, rng),
                _ => self.rectangle(build_data, room),
            }
        }
    }
}
