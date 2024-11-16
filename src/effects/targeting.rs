use crate::components::{Equipped, InBackpack, Position, TileSize};
use crate::map::Map;
use crate::rect::Rect;
use specs::prelude::*;

pub fn entity_position(ecs: &World, target: Entity) -> Option<i32> {
    if let Some(pos) = ecs.read_storage::<Position>().get(target) {
        let map = ecs.fetch::<Map>();
        return Some(map.xy_idx(pos.x, pos.y) as i32);
    }
    None
}

pub fn aoe_tiles(map: &Map, target: rltk::Point, radius: i32) -> Vec<i32> {
    let mut blast_tiles = rltk::field_of_view(target, radius, map);
    blast_tiles.retain(|p| p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1);
    let mut result = Vec::new();
    for t in blast_tiles.iter() {
        result.push(map.xy_idx(t.x, t.y) as i32);
    }
    result
}

pub fn rect_tiles(map: &Map, target: rltk::Point, size: &TileSize) -> Vec<i32> {
    let target_rect = Rect::new(target.x, target.y, size.x, size.y);
    let mut result = Vec::new();
    for t in target_rect.get_all_tiles() {
        result.push(map.xy_idx(t.0, t.1) as i32);
    }
    result
}

pub fn find_item_position(ecs: &World, target: Entity, creator: Option<Entity>) -> Option<i32> {
    let positions = ecs.read_storage::<Position>();
    let map = ecs.fetch::<Map>();

    // Easy - it has a position
    if let Some(pos) = positions.get(target) {
        return Some(map.xy_idx(pos.x, pos.y) as i32);
    }

    // Maybe it is carried?
    if let Some(carried) = ecs.read_storage::<InBackpack>().get(target) {
        if let Some(pos) = positions.get(carried.owner) {
            return Some(map.xy_idx(pos.x, pos.y) as i32);
        }
    }

    // Maybe it is equipped?
    if let Some(equipped) = ecs.read_storage::<Equipped>().get(target) {
        if let Some(pos) = positions.get(equipped.owner) {
            return Some(map.xy_idx(pos.x, pos.y) as i32);
        }
    }

    // Maybe the creator has a position?
    if let Some(creator) = creator {
        if let Some(pos) = positions.get(creator) {
            return Some(map.xy_idx(pos.x, pos.y) as i32);
        }
    }

    // No idea - give up
    None
}
