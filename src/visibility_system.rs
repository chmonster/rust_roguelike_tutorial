use super::{BlocksVisibility, Hidden, Map, Name, Player, Position, Viewshed};
use rltk::{field_of_view, Point};
use specs::prelude::*;

const FIND_TRAP_ODDS: i32 = 24;
pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, Hidden>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, BlocksVisibility>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            entities,
            mut viewshed,
            pos,
            player,
            mut hidden,
            mut rng,
            names,
            blocks_visibility,
        ) = data;

        map.view_blocked.clear();
        for (block_pos, _block) in (&pos, &blocks_visibility).join() {
            let idx = map.xy_idx(block_pos.x, block_pos.y);
            map.view_blocked.insert(idx);
        }

        for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                viewshed.visible_tiles.clear();
                viewshed.visible_tiles =
                    field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
                viewshed
                    .visible_tiles
                    .retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

                // If this is player, reveal what they can see
                let p: Option<&Player> = player.get(ent);
                if let Some(_p) = p {
                    for t in map.visible_tiles.iter_mut() {
                        *t = false
                    }
                    for vis in viewshed.visible_tiles.iter() {
                        let idx = map.xy_idx(vis.x, vis.y);
                        map.revealed_tiles[idx] = true;
                        map.visible_tiles[idx] = true;

                        // Chance to reveal hidden things
                        crate::spatial::for_each_tile_content(idx, |e| {
                            let maybe_hidden = hidden.get(e);
                            if let Some(_maybe_hidden) = maybe_hidden {
                                if rng.roll_dice(1, FIND_TRAP_ODDS) == 1 {
                                    let name = names.get(e);
                                    if let Some(name) = name {
                                        crate::gamelog::Logger::new()
                                            .append("You spotted:")
                                            .color(rltk::RED)
                                            .append(&name.name)
                                            .log();
                                    }
                                    hidden.remove(e);
                                }
                            }
                        });
                    }
                }
            }
        }
    }
}
